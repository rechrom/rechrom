//! XDND source/selection owner. This connection owns a private source window;
//! it never drains winit's X event queue or impersonates clipboard ownership.
use super::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::{collections::HashMap, ffi::CStr, time::Duration};
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{xproto::*, Event},
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};
type Error = Box<dyn std::error::Error>;
struct Incremental {
    bytes: Arc<[u8]>,
    sent: usize,
    target: Atom,
}
pub(super) struct Session {
    start: Option<Start>,
    connection: RustConnection,
    source: Window,
    root: Window,
    origin: Window,
    scale: f32,
    atoms: HashMap<&'static str, Atom>,
    offers: Vec<(Atom, Arc<[u8]>)>,
    target: Window,
    proxy: Window,
    version: u32,
    accepted: DragOperation,
    waiting_status: bool,
    queued_position: Option<(i16, i16, u32, KeyButMask)>,
    last_position: (i16, i16),
    last_time: u32,
    released: bool,
    released_at: Option<Instant>,
    dropped: Option<Instant>,
    transfers: HashMap<(Window, Atom), Incremental>,
    icon: Option<Window>,
    escape: Option<u8>,
}
impl Session {
    pub(super) fn new(
        window: &winit::window::Window,
        start: Start,
    ) -> Result<Self, (Start, Failure)> {
        match Self::prepare(window, &start) {
            Ok(mut session) => {
                session.start = Some(start);
                Ok(session)
            }
            Err(error) => Err((start, error)),
        }
    }
    fn prepare(window: &winit::window::Window, start: &Start) -> Result<Self, Failure> {
        let display = window
            .display_handle()
            .map_err(|_| Failure::BusyOrUnavailable)?;
        let RawDisplayHandle::Xlib(display) = display.as_raw() else {
            return Err(Failure::Unsupported);
        };
        let handle = window
            .window_handle()
            .map_err(|_| Failure::BusyOrUnavailable)?;
        let RawWindowHandle::Xlib(handle) = handle.as_raw() else {
            return Err(Failure::Unsupported);
        };
        let xlib = x11_dl::xlib::Xlib::open().map_err(|_| Failure::Unsupported)?;
        let raw_display = display.display.ok_or(Failure::BusyOrUnavailable)?;
        // Resolve the actual display selected by winit, including non-default
        // displays. A fresh connection avoids consuming any host-owned events.
        let name = unsafe { (xlib.XDisplayString)(raw_display.as_ptr().cast()) };
        if name.is_null() {
            return Err(Failure::BusyOrUnavailable);
        }
        let name = unsafe { CStr::from_ptr(name) }
            .to_str()
            .map_err(|_| Failure::Unsupported)?;
        let (connection, _) = x11rb::connect(Some(name)).map_err(|_| Failure::BusyOrUnavailable)?;
        let root = connection
            .setup()
            .roots
            .get(display.screen as usize)
            .ok_or(Failure::Unsupported)?
            .root;
        let origin = u32::try_from(handle.window).map_err(|_| Failure::Unsupported)?;
        let (values, direct_save) = super::linux_payload::build(&start.request)?;
        let build = || -> Result<Self, Error> {
            if connection.get_geometry(origin)?.reply()?.root != root {
                return Err("native source belongs to another display".into());
            }
            let pointer = connection.query_pointer(root)?.reply()?;
            if !pointer.same_screen || !pointer.mask.contains(KeyButMask::BUTTON1) {
                return Err("pointer button grab has ended".into());
            }
            let source = connection.generate_id()?;
            connection
                .create_window(
                    x11rb::COPY_DEPTH_FROM_PARENT,
                    source,
                    root,
                    -1,
                    -1,
                    1,
                    1,
                    0,
                    WindowClass::INPUT_OUTPUT,
                    0,
                    &CreateWindowAux::new()
                        .override_redirect(1)
                        .event_mask(EventMask::PROPERTY_CHANGE),
                )?
                .check()?;
            let mut atoms = HashMap::new();
            for name in [
                "XdndSelection",
                "XdndAware",
                "XdndProxy",
                "XdndEnter",
                "XdndPosition",
                "XdndStatus",
                "XdndLeave",
                "XdndDrop",
                "XdndFinished",
                "XdndTypeList",
                "XdndActionList",
                "XdndActionCopy",
                "XdndActionMove",
                "XdndActionLink",
                "XdndActionDirectSave",
                "XdndDirectSave0",
                "TARGETS",
                "INCR",
            ] {
                atoms.insert(
                    name,
                    connection
                        .intern_atom(false, name.as_bytes())?
                        .reply()?
                        .atom,
                );
            }
            let mut offers = Vec::new();
            for (mime, bytes) in values {
                let atom = connection
                    .intern_atom(false, mime.as_bytes())?
                    .reply()?
                    .atom;
                offers.push((atom, bytes.clone()));
                if mime == "text/plain" {
                    offers.push((
                        connection.intern_atom(false, b"UTF8_STRING")?.reply()?.atom,
                        bytes.clone(),
                    ));
                    offers.push((
                        connection
                            .intern_atom(false, b"text/plain;charset=utf-8")?
                            .reply()?
                            .atom,
                        bytes,
                    ));
                }
            }
            if let Some((name, bytes)) = direct_save {
                // XDS has one filename slot. Named MIME offers above continue
                // to expose all promises; only a single promise gets XDS.
                let promises = start.request.data.files.len()
                    + start
                        .request
                        .data
                        .file_items
                        .iter()
                        .filter(|file| !file.HasBackingFile())
                        .count();
                if promises == 1 {
                    connection.change_property8(
                        PropMode::REPLACE,
                        source,
                        atoms["XdndDirectSave0"],
                        AtomEnum::STRING,
                        name.as_bytes(),
                    )?;
                    offers.push((atoms["XdndDirectSave0"], Arc::from(&b"F"[..])));
                    offers.push((
                        connection
                            .intern_atom(false, b"application/octet-stream")?
                            .reply()?
                            .atom,
                        bytes,
                    ));
                }
            }
            let types: Vec<_> = offers.iter().map(|(atom, _)| *atom).collect();
            connection.change_property32(
                PropMode::REPLACE,
                source,
                atoms["XdndTypeList"],
                AtomEnum::ATOM,
                &types,
            )?;
            let mut actions = Vec::new();
            for (mask, name) in [
                (1, "XdndActionCopy"),
                (16, "XdndActionMove"),
                (2, "XdndActionLink"),
            ] {
                if start.request.source_operations & mask != 0 {
                    actions.push(atoms[name]);
                }
            }
            if types.contains(&atoms["XdndDirectSave0"]) {
                actions.push(atoms["XdndActionDirectSave"]);
            }
            connection.change_property32(
                PropMode::REPLACE,
                source,
                atoms["XdndActionList"],
                AtomEnum::ATOM,
                &actions,
            )?;
            connection
                .set_selection_owner(source, atoms["XdndSelection"], x11rb::CURRENT_TIME)?
                .check()?;
            if connection
                .get_selection_owner(atoms["XdndSelection"])?
                .reply()?
                .owner
                != source
            {
                return Err("selection ownership failed".into());
            }
            let icon = if start.request.image.is_some() {
                Some(make_icon(&connection, root, start)?)
            } else {
                None
            };
            let grab = connection
                .grab_pointer(
                    false,
                    source,
                    EventMask::POINTER_MOTION | EventMask::BUTTON_RELEASE,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                    x11rb::NONE,
                    x11rb::NONE,
                    x11rb::CURRENT_TIME,
                )?
                .reply()?;
            if grab.status != GrabStatus::SUCCESS {
                return Err("pointer already grabbed".into());
            }
            let keyboard = connection
                .grab_keyboard(
                    false,
                    source,
                    x11rb::CURRENT_TIME,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )?
                .reply()?;
            if keyboard.status != GrabStatus::SUCCESS {
                return Err("keyboard already grabbed".into());
            }
            let first = connection.setup().min_keycode;
            let last = connection.setup().max_keycode;
            let mapping = connection
                .get_keyboard_mapping(first, last - first + 1)?
                .reply()?;
            let escape = mapping
                .keysyms
                .chunks(mapping.keysyms_per_keycode as usize)
                .position(|keys| keys.contains(&0xff1b))
                .map(|index| first + index as u8);
            connection.flush()?;
            Ok(Self {
                start: None,
                connection,
                source,
                root,
                origin,
                scale: window.scale_factor() as f32,
                atoms,
                offers,
                target: 0,
                proxy: 0,
                version: 0,
                accepted: DragOperation::None,
                waiting_status: false,
                queued_position: None,
                last_position: (pointer.root_x, pointer.root_y),
                last_time: 0,
                released: false,
                released_at: None,
                dropped: None,
                transfers: HashMap::new(),
                icon,
                escape,
            })
        };
        build().map_err(|_| Failure::BusyOrUnavailable)
    }
    fn atom(&self, name: &'static str) -> Atom {
        self.atoms[name]
    }
    fn message(&self, target: Window, kind: &'static str, data: [u32; 5]) -> Result<(), Error> {
        self.connection
            .send_event(
                false,
                target,
                EventMask::NO_EVENT,
                ClientMessageEvent::new(32, self.target, self.atom(kind), data),
            )?
            .check()?;
        Ok(())
    }
    fn property(&self, window: Window, name: &'static str, ty: Atom) -> Result<Vec<u32>, Error> {
        Ok(self
            .connection
            .get_property(false, window, self.atom(name), ty, 0, 8)?
            .reply()?
            .value32()
            .map(|values| values.collect())
            .unwrap_or_default())
    }
    fn locate_target(&self) -> Result<(Window, Window, u32), Error> {
        let mut cursor = self.root;
        let mut result = (0, 0, 0);
        for _ in 0..64 {
            if cursor != self.root && Some(cursor) != self.icon && cursor != self.source {
                let aware = self.property(cursor, "XdndAware", AtomEnum::ATOM.into())?;
                if let Some(version) = aware.first() {
                    let mut proxy = cursor;
                    if let Some(proposed) = self
                        .property(cursor, "XdndProxy", AtomEnum::WINDOW.into())?
                        .first()
                        .copied()
                    {
                        if self
                            .property(proposed, "XdndProxy", AtomEnum::WINDOW.into())?
                            .first()
                            == Some(&proposed)
                        {
                            proxy = proposed;
                        }
                    }
                    result = (cursor, proxy, (*version).min(5));
                }
            }
            let child = self.connection.query_pointer(cursor)?.reply()?.child;
            if child == 0 {
                break;
            }
            cursor = child;
        }
        Ok(result)
    }
    fn position(&mut self, x: i16, y: i16, time: u32, state: KeyButMask) -> Result<(), Error> {
        self.last_position = (x, y);
        self.last_time = time;
        if let (Some(icon), Some(start)) = (self.icon, &self.start) {
            self.connection.configure_window(
                icon,
                &ConfigureWindowAux::new()
                    .x(i32::from(x) - start.request.cursor_offset.0)
                    .y(i32::from(y) - start.request.cursor_offset.1),
            )?;
        }
        let (target, proxy, version) = self.locate_target()?;
        if target != self.target {
            if self.target != 0 {
                self.message(self.proxy, "XdndLeave", [self.source, 0, 0, 0, 0])?;
            }
            self.target = target;
            self.proxy = proxy;
            self.version = version;
            self.accepted = DragOperation::None;
            self.waiting_status = false;
            self.queued_position = None;
            if target != 0 {
                let mut types = [0; 3];
                for (out, (atom, _)) in types.iter_mut().zip(&self.offers) {
                    *out = *atom;
                }
                self.message(
                    proxy,
                    "XdndEnter",
                    [
                        self.source,
                        (version << 24) | u32::from(self.offers.len() > 3),
                        types[0],
                        types[1],
                        types[2],
                    ],
                )?;
            }
        }
        if target == 0 {
            return Ok(());
        }
        if self.waiting_status {
            self.queued_position = Some((x, y, time, state));
            return Ok(());
        }
        let allowed = self
            .start
            .as_ref()
            .map_or(0, |start| start.request.source_operations);
        let preferred =
            if state.contains(KeyButMask::CONTROL | KeyButMask::SHIFT) && allowed & 2 != 0 {
                "XdndActionLink"
            } else if state.contains(KeyButMask::CONTROL) && allowed & 1 != 0 {
                "XdndActionCopy"
            } else if state.contains(KeyButMask::SHIFT) && allowed & 16 != 0 {
                "XdndActionMove"
            } else if allowed & 1 != 0 {
                "XdndActionCopy"
            } else if allowed & 16 != 0 {
                "XdndActionMove"
            } else {
                "XdndActionLink"
            };
        self.message(
            proxy,
            "XdndPosition",
            [
                self.source,
                0,
                (u32::from(x as u16) << 16) | u32::from(y as u16),
                time,
                self.atom(preferred),
            ],
        )?;
        self.waiting_status = true;
        Ok(())
    }
    fn release(&mut self) -> Result<Option<Outcome>, Error> {
        self.released = true;
        self.released_at.get_or_insert_with(Instant::now);
        self.connection.ungrab_pointer(x11rb::CURRENT_TIME)?;
        self.connection.ungrab_keyboard(x11rb::CURRENT_TIME)?;
        if self.waiting_status || self.queued_position.is_some() {
            return Ok(None);
        }
        if self.target == 0 || self.accepted == DragOperation::None {
            return Ok(Some(Outcome::Cancelled));
        }
        self.message(
            self.proxy,
            "XdndDrop",
            [self.source, 0, self.last_time, 0, 0],
        )?;
        self.dropped = Some(Instant::now());
        Ok(None)
    }
    fn selection(&mut self, request: SelectionRequestEvent) -> Result<(), Error> {
        let property = if request.property == 0 {
            request.target
        } else {
            request.property
        };
        let mut success = false;
        if request.selection == self.atom("XdndSelection") {
            if request.target == self.atom("TARGETS") {
                let mut atoms: Vec<_> = self.offers.iter().map(|(atom, _)| *atom).collect();
                atoms.push(self.atom("TARGETS"));
                self.connection.change_property32(
                    PropMode::REPLACE,
                    request.requestor,
                    property,
                    AtomEnum::ATOM,
                    &atoms,
                )?;
                success = true;
            } else if let Some((_, bytes)) =
                self.offers.iter().find(|(atom, _)| *atom == request.target)
            {
                if bytes.len() <= 64 * 1024 {
                    self.connection.change_property8(
                        PropMode::REPLACE,
                        request.requestor,
                        property,
                        request.target,
                        bytes,
                    )?;
                } else {
                    self.connection.change_window_attributes(
                        request.requestor,
                        &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
                    )?;
                    self.connection.change_property32(
                        PropMode::REPLACE,
                        request.requestor,
                        property,
                        self.atom("INCR"),
                        &[bytes.len().min(u32::MAX as usize) as u32],
                    )?;
                    self.transfers.insert(
                        (request.requestor, property),
                        Incremental {
                            bytes: bytes.clone(),
                            sent: 0,
                            target: request.target,
                        },
                    );
                }
                success = true;
            }
        }
        self.connection.send_event(
            false,
            request.requestor,
            EventMask::NO_EVENT,
            SelectionNotifyEvent {
                response_type: SELECTION_NOTIFY_EVENT,
                sequence: 0,
                time: request.time,
                requestor: request.requestor,
                selection: request.selection,
                target: request.target,
                property: if success { property } else { 0 },
            },
        )?;
        Ok(())
    }
    fn event(&mut self, event: Event) -> Result<Option<Outcome>, Error> {
        match event {
            Event::MotionNotify(event) if !self.released => {
                self.position(event.root_x, event.root_y, event.time, event.state)?
            }
            Event::ButtonRelease(event) if event.detail == 1 && !self.released => {
                self.position(event.root_x, event.root_y, event.time, event.state)?;
                return self.release();
            }
            Event::KeyPress(event) if Some(event.detail) == self.escape => {
                return Ok(Some(Outcome::Cancelled))
            }
            Event::SelectionClear(_) => return Ok(Some(Outcome::Cancelled)),
            Event::SelectionRequest(request) => self.selection(request)?,
            Event::PropertyNotify(event) if event.state == Property::DELETE => {
                let key = (event.window, event.atom);
                if let Some(transfer) = self.transfers.get_mut(&key) {
                    let end = (transfer.sent + 64 * 1024).min(transfer.bytes.len());
                    self.connection.change_property8(
                        PropMode::REPLACE,
                        event.window,
                        event.atom,
                        transfer.target,
                        &transfer.bytes[transfer.sent..end],
                    )?;
                    if end == transfer.sent {
                        self.transfers.remove(&key);
                    } else {
                        transfer.sent = end;
                    }
                }
            }
            Event::ClientMessage(event) if event.format == 32 => {
                let data = event.data.as_data32();
                if data[0] != self.target {
                    return Ok(None);
                }
                if event.type_ == self.atom("XdndStatus") && self.dropped.is_none() {
                    self.waiting_status = false;
                    let mask = self
                        .start
                        .as_ref()
                        .map_or(0, |start| start.request.source_operations);
                    self.accepted = if data[1] & 1 == 0 {
                        DragOperation::None
                    } else if data[4] == self.atom("XdndActionMove") && mask & 16 != 0 {
                        DragOperation::Move
                    } else if data[4] == self.atom("XdndActionCopy") && mask & 1 != 0 {
                        DragOperation::Copy
                    } else if data[4] == self.atom("XdndActionLink") && mask & 2 != 0 {
                        DragOperation::Link
                    } else if data[4] == self.atom("XdndActionDirectSave") && mask & 1 != 0 {
                        DragOperation::Copy
                    } else {
                        DragOperation::None
                    };
                    if let Some((x, y, time, state)) = self.queued_position.take() {
                        self.position(x, y, time, state)?;
                    }
                    if self.released {
                        return self.release();
                    }
                } else if event.type_ == self.atom("XdndFinished") && self.dropped.is_some() {
                    if self.version >= 5 && data[1] & 1 == 0 {
                        return Ok(Some(Outcome::Cancelled));
                    }
                    let final_operation = if self.version < 5 {
                        self.accepted
                    } else if data[2] == self.atom("XdndActionMove") {
                        DragOperation::Move
                    } else if data[2] == self.atom("XdndActionCopy")
                        || data[2] == self.atom("XdndActionDirectSave")
                    {
                        DragOperation::Copy
                    } else if data[2] == self.atom("XdndActionLink") {
                        DragOperation::Link
                    } else {
                        DragOperation::None
                    };
                    let mask = self
                        .start
                        .as_ref()
                        .map_or(0, |start| start.request.source_operations);
                    return Ok(Some(
                        if final_operation == DragOperation::None
                            || mask & final_operation as u32 == 0
                        {
                            Outcome::Cancelled
                        } else {
                            Outcome::Ended(final_operation)
                        },
                    ));
                }
            }
            Event::Error(_) => return Ok(Some(Outcome::Failed(Failure::BusyOrUnavailable))),
            _ => {}
        }
        Ok(None)
    }
    pub(super) fn poll(&mut self) -> bool {
        let result = (|| -> Result<Option<Outcome>, Error> {
            while let Some(event) = self.connection.poll_for_event()? {
                if let Some(outcome) = self.event(event)? {
                    return Ok(Some(outcome));
                }
            }
            // Also send the initial position without waiting for pointer motion.
            if self.target == 0 && !self.released {
                let pointer = self.connection.query_pointer(self.root)?.reply()?;
                self.position(pointer.root_x, pointer.root_y, self.last_time, pointer.mask)?;
            }
            self.connection.flush()?;
            if self
                .dropped
                .or(self.released_at)
                .is_some_and(|time| time.elapsed() >= Duration::from_secs(5))
            {
                return Ok(Some(Outcome::Failed(Failure::BusyOrUnavailable)));
            }
            Ok(None)
        })();
        let outcome = match result {
            Ok(None) => return true,
            Ok(Some(outcome)) => outcome,
            Err(_) => Outcome::Failed(Failure::BusyOrUnavailable),
        };
        self.complete(outcome);
        false
    }
    fn complete(&mut self, outcome: Outcome) {
        let Some(start) = self.start.take() else {
            return;
        };
        let (x, y) = self.last_position;
        let local = self
            .connection
            .translate_coordinates(self.root, self.origin, x, y)
            .ok()
            .and_then(|reply| reply.reply().ok())
            .map_or((0., 0.), |point| {
                (
                    point.dst_x as f32 / self.scale,
                    point.dst_y as f32 / self.scale,
                )
            });
        start.complete(
            local,
            (x as f32 / self.scale, y as f32 / self.scale),
            outcome,
        );
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.complete(Outcome::Cancelled);
        if self.target != 0 && self.dropped.is_none() {
            let _ = self.message(self.proxy, "XdndLeave", [self.source, 0, 0, 0, 0]);
        }
        let _ = self.connection.ungrab_pointer(x11rb::CURRENT_TIME);
        let _ = self.connection.ungrab_keyboard(x11rb::CURRENT_TIME);
        if let Some(icon) = self.icon {
            let _ = self.connection.destroy_window(icon);
        }
        let _ = self.connection.destroy_window(self.source);
        let _ = self.connection.flush();
    }
}
fn make_icon(connection: &RustConnection, root: Window, start: &Start) -> Result<Window, Error> {
    let image = start.request.image.as_ref().ok_or("missing image")?;
    let screen = connection
        .setup()
        .roots
        .iter()
        .find(|screen| screen.root == root)
        .ok_or("missing screen")?;
    let visual = screen
        .allowed_depths
        .iter()
        .find(|depth| depth.depth == 32)
        .and_then(|depth| {
            depth
                .visuals
                .iter()
                .find(|visual| visual.class == VisualClass::TRUE_COLOR)
        })
        .ok_or("32-bit drag icon visual unavailable")?;
    let icon = connection.generate_id()?;
    let colormap = connection.generate_id()?;
    connection
        .create_colormap(ColormapAlloc::NONE, colormap, root, visual.visual_id)?
        .check()?;
    let width = u16::try_from(image.width)?;
    let height = u16::try_from(image.height)?;
    connection
        .create_window(
            32,
            icon,
            root,
            -32768,
            -32768,
            width,
            height,
            0,
            WindowClass::INPUT_OUTPUT,
            visual.visual_id,
            &CreateWindowAux::new()
                .override_redirect(1)
                .colormap(colormap)
                .border_pixel(0)
                .background_pixel(0),
        )?
        .check()?;
    // An empty input shape keeps the icon out of pointer target discovery.
    connection
        .shape_rectangles(
            shape::SO::SET,
            shape::SK::INPUT,
            ClipOrdering::UNSORTED,
            icon,
            0,
            0,
            &[],
        )?
        .check()?;
    let gc = connection.generate_id()?;
    connection.create_gc(gc, icon, &CreateGCAux::new())?;
    let mut bytes = Vec::with_capacity(image.width as usize * image.height as usize * 4);
    let little = connection.setup().image_byte_order == ImageOrder::LSB_FIRST;
    for row in 0..image.height as usize {
        for pixel in image.rgba8
            [row * image.row_bytes..row * image.row_bytes + image.width as usize * 4]
            .chunks_exact(4)
        {
            let value = (u32::from(pixel[3]) << 24)
                | (u32::from(pixel[0]) << 16)
                | (u32::from(pixel[1]) << 8)
                | u32::from(pixel[2]);
            bytes.extend_from_slice(&if little {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            });
        }
    }
    // Bound each request to avoid exceeding the server's maximum request size.
    let stride = image.width as usize * 4;
    let rows = ((connection.maximum_request_bytes().saturating_sub(64)) / stride)
        .max(1)
        .min(height as usize);
    for row in (0..height as usize).step_by(rows) {
        let count = rows.min(height as usize - row);
        connection
            .put_image(
                ImageFormat::Z_PIXMAP,
                icon,
                gc,
                width,
                count as u16,
                0,
                row as i16,
                0,
                32,
                &bytes[row * stride..(row + count) * stride],
            )?
            .check()?;
    }
    connection.free_gc(gc)?;
    connection.free_colormap(colormap)?;
    connection.map_window(icon)?;
    Ok(icon)
}
