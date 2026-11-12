use std::os::raw::*;
use std::path::PathBuf;
use std::str::Utf8Error;
use std::sync::Arc;

use percent_encoding::percent_decode;
use x11rb::protocol::xproto::{self, ConnectionExt};

use super::atoms::AtomName::None as DndNone;
use super::atoms::*;
use super::{util, CookieResultExt, X11Error, XConnection};

#[derive(Debug, Clone, Copy)]
pub enum DndState {
    Accepted(u32),
    Rejected,
}

#[derive(Debug)]
pub enum DndDataParseError {
    EmptyData,
    InvalidUtf8(#[allow(dead_code)] Utf8Error),
}

impl From<Utf8Error> for DndDataParseError {
    fn from(e: Utf8Error) -> Self {
        DndDataParseError::InvalidUtf8(e)
    }
}

pub struct Dnd {
    xconn: Arc<XConnection>,
    // Populated by XdndEnter event handler
    pub version: Option<c_long>,
    pub type_list: Option<Vec<xproto::Atom>>,
    // Populated by XdndPosition event handler
    pub source_window: Option<xproto::Window>,
    pub target_window: Option<xproto::Window>,
    pub incremental: bool,
    pub incremental_data: Vec<u8>,
    pub incremental_discard: bool,
    pub source_action_list: bool,
    pub native: Option<crate::event::NativeDragDropEvent>,
    pub pending_types: Vec<xproto::Atom>,
    pub type_mimes: Vec<(xproto::Atom, String)>,
    pub selected_type: Option<xproto::Atom>,
    pub entered: bool,
    pub pending_drop: bool,
}

impl Dnd {
    pub fn new(xconn: Arc<XConnection>) -> Result<Self, X11Error> {
        Ok(Dnd {
            xconn,
            version: None,
            type_list: None,
            source_window: None,
            target_window: None,
            incremental: false,
            incremental_data: Vec::new(),
            incremental_discard: false,
            source_action_list: false,
            native: None,
            pending_types: Vec::new(),
            type_mimes: Vec::new(),
            selected_type: None,
            entered: false,
            pending_drop: false,
        })
    }

    pub fn reset(&mut self) {
        self.version = None;
        self.type_list = None;
        self.source_window = None;
        self.target_window = None;
        self.incremental = false;
        self.incremental_data.clear();
        self.incremental_discard = false;
        self.source_action_list = false;
        self.native = None;
        self.pending_types.clear();
        self.type_mimes.clear();
        self.selected_type = None;
        self.entered = false;
        self.pending_drop = false;
    }

    pub unsafe fn send_status(
        &self,
        this_window: xproto::Window,
        target_window: xproto::Window,
        state: DndState,
    ) -> Result<(), X11Error> {
        let atoms = self.xconn.atoms();
        let (accepted, action) = match state {
            DndState::Accepted(operation) => (
                1,
                match operation {
                    1 => atoms[XdndActionCopy],
                    2 => atoms[XdndActionLink],
                    16 => atoms[XdndActionMove],
                    _ => atoms[DndNone],
                },
            ),
            DndState::Rejected => (0, atoms[DndNone]),
        };
        self.xconn
            .send_client_msg(
                target_window,
                target_window,
                atoms[XdndStatus] as _,
                None,
                [this_window, accepted | 2, 0, 0, action as _],
            )?
            .ignore_error();

        Ok(())
    }

    pub unsafe fn send_finished(
        &self,
        this_window: xproto::Window,
        target_window: xproto::Window,
        state: DndState,
    ) -> Result<(), X11Error> {
        let atoms = self.xconn.atoms();
        let (accepted, action) = match state {
            DndState::Accepted(operation) => (
                1,
                match operation {
                    1 => atoms[XdndActionCopy],
                    2 => atoms[XdndActionLink],
                    16 => atoms[XdndActionMove],
                    _ => atoms[DndNone],
                },
            ),
            DndState::Rejected => (0, atoms[DndNone]),
        };
        self.xconn
            .send_client_msg(
                target_window,
                target_window,
                atoms[XdndFinished] as _,
                None,
                [this_window, accepted, action as _, 0, 0],
            )?
            .ignore_error();

        Ok(())
    }

    pub unsafe fn get_type_list(
        &self,
        source_window: xproto::Window,
    ) -> Result<Vec<xproto::Atom>, util::GetPropertyError> {
        let atoms = self.xconn.atoms();
        self.xconn.get_property(
            source_window,
            atoms[XdndTypeList],
            xproto::Atom::from(xproto::AtomEnum::ATOM),
        )
    }

    pub unsafe fn convert_selection(&self, window: xproto::Window, time: xproto::Timestamp) {
        let atoms = self.xconn.atoms();
        self.xconn
            .xcb_connection()
            .convert_selection(
                window,
                atoms[XdndSelection],
                self.selected_type.unwrap_or(atoms[TextUriList]),
                atoms[XdndSelection],
                time,
            )
            .expect_then_ignore_error("Failed to send XdndSelection event")
    }

    pub unsafe fn read_data(&self, window: xproto::Window) -> Option<Vec<c_uchar>> {
        let atoms = self.xconn.atoms();
        let selected = self.selected_type?;
        let reply = self
            .xconn
            .xcb_connection()
            .get_property(
                false,
                window,
                atoms[XdndSelection],
                selected,
                0,
                64 * 1024 * 1024 / 4,
            )
            .ok()?
            .reply()
            .ok()?;
        (reply.type_ == selected && reply.format == 8 && reply.bytes_after == 0)
            .then_some(reply.value)
    }

    pub fn parse_data(&self, data: &mut [c_uchar]) -> Result<Vec<PathBuf>, DndDataParseError> {
        use std::os::unix::ffi::OsStringExt;
        if data.is_empty() {
            return Err(DndDataParseError::EmptyData);
        }
        let text = std::str::from_utf8(data)?;
        Ok(text
            .lines()
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| {
                let rest = line.trim().strip_prefix("file://")?;
                let path = if rest.starts_with('/') {
                    rest
                } else {
                    rest.strip_prefix("localhost")?
                };
                if !path.starts_with('/') {
                    return None;
                }
                let decoded: Vec<u8> = percent_decode(path.as_bytes()).collect();
                if decoded.contains(&0) {
                    return None;
                }
                Some(std::ffi::OsString::from_vec(decoded).into())
            })
            .collect())
    }
}
