//! Native destination data-device transport. Page owns drag policy and default actions.
use std::io::Read;
use std::os::fd::{AsFd, AsRawFd};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use sctk::reexports::calloop::{generic::Generic, Interest, Mode, PostAction, RegistrationToken};
use sctk::reexports::client::backend::ObjectId;
use sctk::reexports::client::protocol::{
    wl_data_device::{self, WlDataDevice},
    wl_data_device_manager::{DndAction, WlDataDeviceManager},
    wl_data_offer::{self, WlDataOffer},
    wl_seat::WlSeat,
};
use sctk::reexports::client::{Connection, Dispatch, Proxy, QueueHandle, WEnum};

use crate::dpi::LogicalPosition;
use crate::event::{NativeDragDropEvent, NativeDragDropPhase, NativeDragFileOffer, WindowEvent};
use crate::platform_impl::wayland::{make_wid, state::WinitState, WindowId};

#[derive(Debug)]
pub struct DragDeviceData {
    pub seat: ObjectId,
}
#[derive(Debug)]
pub struct NativeOffer {
    mime_types: Vec<String>,
    source_operations: u32,
}
impl NativeOffer {
    fn new(offer: WlDataOffer) -> Self {
        Self {
            source_operations: if offer.version() < 3 { 1 } else { 0 },
            mime_types: Vec::new(),
        }
    }
}
#[derive(Debug)]
pub struct NativeDestination {
    offer: WlDataOffer,
    window: WindowId,
    serial: u32,
    mime: Option<String>,
    pending: usize,
    entered: bool,
    dropped: bool,
    event: NativeDragDropEvent,
    last_operation: Option<u32>,
    transfers: Vec<RegistrationToken>,
    left: bool,
    finished: bool,
    selected_operation: u32,
}
impl NativeDestination {
    fn finish(&mut self) {
        if self.dropped
            && self.pending == 0
            && !self.finished
            && self.selected_operation != 0
            && self.offer.version() >= 3
        {
            self.offer.finish();
            self.finished = true;
        }
    }
}
#[derive(Debug)]
pub struct NativeTransfer {
    offer: ObjectId,
    mime: String,
    bytes: Option<Vec<u8>>,
}

impl super::WinitSeatState {
    pub fn bind_data_device(
        &mut self,
        manager: Option<&WlDataDeviceManager>,
        queue: &QueueHandle<WinitState>,
        seat: &WlSeat,
    ) {
        self.data_device = manager.map(|manager| {
            manager.get_data_device(seat, queue, DragDeviceData { seat: seat.id() })
        });
    }
}

impl WinitState {
    fn begin_native_destination(
        &mut self,
        device: &DragDeviceData,
        serial: u32,
        surface: &sctk::reexports::client::protocol::wl_surface::WlSurface,
        x: f64,
        y: f64,
        offer: WlDataOffer,
    ) {
        self.leave_native_destination(&device.seat);
        let window_id = make_wid(surface);
        let Some(window) = self.windows.get_mut().get(&window_id) else {
            return;
        };
        let scale = window.lock().unwrap().scale_factor();
        let native_offer = self
            .drag_offers
            .entry(offer.id())
            .or_insert_with(|| NativeOffer::new(offer.clone()));
        // Receive each MIME through nonblocking event sources. A source may
        // stream data only after this dispatch returns.
        let mimes: Vec<_> = native_offer.mime_types.iter().take(32).cloned().collect();
        let operations = native_offer.source_operations;
        let mut destination = NativeDestination {
            offer: offer.clone(),
            window: window_id,
            serial,
            mime: mimes.first().cloned(),
            pending: 0,
            entered: false,
            dropped: false,
            last_operation: None,
            transfers: Vec::new(),
            left: false,
            finished: false,
            selected_operation: if offer.version() < 3 { 1 } else { 0 },
            event: NativeDragDropEvent {
                phase: NativeDragDropPhase::Enter,
                paths: Vec::new(),
                files: Vec::new(),
                virtual_files: Vec::new(),
                strings: Vec::new(),
                data: Vec::new(),
                position: LogicalPosition::new(x, y).to_physical(scale),
                screen_position: None,
                modifiers: 0,
                source_operations: operations,
                operation: Arc::new(AtomicU32::new(0)),
                observed_at: std::time::Instant::now(),
            },
        };
        offer.accept(serial, destination.mime.clone());
        if offer.version() >= 3 {
            offer.set_actions(DndAction::empty(), DndAction::empty());
        }
        for mime in mimes {
            let Ok((reader, writer)) = rustix::pipe::pipe() else {
                continue;
            };
            offer.receive(mime.clone(), writer.as_fd());
            drop(writer);
            let flags = unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_GETFL) };
            if flags < 0
                || unsafe {
                    libc::fcntl(reader.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK)
                } < 0
            {
                continue;
            }
            let offer_id = offer.id();
            let mut bytes = Vec::new();
            let source = Generic::<_, std::io::Error>::new(
                std::fs::File::from(reader),
                Interest::READ,
                Mode::Level,
            );
            if let Ok(token) = self
                .loop_handle
                .insert_source(source, move |_, file, state| {
                    const LIMIT: usize = 64 * 1024 * 1024;
                    // Bound work per event-loop turn as well as retained payload size.
                    let mut buffer = [0; 16384];
                    let mut budget: usize = 256 * 1024;
                    let mut input: &std::fs::File = file.as_ref();
                    let result = loop {
                        match input.read(&mut buffer) {
                            Ok(0) => break Some(std::mem::take(&mut bytes)),
                            Ok(count) => {
                                if bytes.len() + count > LIMIT {
                                    break None;
                                }
                                bytes.extend_from_slice(&buffer[..count]);
                                budget = budget.saturating_sub(count);
                                if budget == 0 {
                                    return Ok(PostAction::Continue);
                                }
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                return Ok(PostAction::Continue)
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                                continue
                            }
                            Err(_) => break None,
                        }
                    };
                    state.complete_native_transfer(NativeTransfer {
                        offer: offer_id.clone(),
                        mime: mime.clone(),
                        bytes: result,
                    });
                    Ok(PostAction::Remove)
                })
            {
                destination.pending += 1;
                destination.transfers.push(token);
            }
        }
        self.drag_destinations
            .insert(device.seat.clone(), destination);
    }

    fn complete_native_transfer(&mut self, transfer: NativeTransfer) {
        let Some(destination) = self
            .drag_destinations
            .values_mut()
            .find(|destination| destination.offer.id() == transfer.offer)
        else {
            return;
        };
        destination.pending = destination.pending.saturating_sub(1);
        if let Some(bytes) = transfer.bytes {
            if transfer.mime == "text/uri-list" {
                for path in local_uri_paths(&bytes) {
                    if let Ok(file) = NativeDragFileOffer::open(path.clone()) {
                        destination.event.paths.push(path);
                        destination.event.files.push(file);
                    }
                }
            }
            let mime = if transfer.mime == "text/plain;charset=utf-8" {
                "text/plain".into()
            } else {
                transfer.mime
            };
            if !destination
                .event
                .strings
                .iter()
                .any(|(name, _)| name == &mime)
            {
                destination.event.strings.push((
                    mime.clone(),
                    String::from_utf8_lossy(&bytes).encode_utf16().collect(),
                ));
                destination.event.data.push((mime, Arc::from(bytes)));
            }
        }
        if destination.pending != 0 {
            return;
        }
        if !destination.event.files.is_empty() || !destination.event.strings.is_empty() {
            destination.entered = true;
            for path in &destination.event.paths {
                self.events_sink
                    .push_window_event(WindowEvent::HoveredFile(path.clone()), destination.window);
            }
            self.events_sink.push_window_event(
                WindowEvent::NativeDragDrop(destination.event.clone()),
                destination.window,
            );
            if destination.dropped {
                destination.event.phase = NativeDragDropPhase::Drop;
                self.events_sink.push_window_event(
                    WindowEvent::NativeDragDrop(destination.event.clone()),
                    destination.window,
                );
            }
            self.dispatched_events = true;
        }
        if destination.dropped {
            destination.finish();
        }
        let left = destination.left;
        if left {
            let seat = self
                .drag_destinations
                .iter()
                .find_map(|(seat, destination)| {
                    (destination.offer.id() == transfer.offer).then(|| seat.clone())
                });
            if let Some(seat) = seat {
                self.leave_native_destination(&seat);
            }
        }
    }

    pub(super) fn leave_native_destination(&mut self, seat: &ObjectId) {
        if self
            .drag_destinations
            .get(seat)
            .is_some_and(|d| d.dropped && d.pending != 0)
        {
            self.drag_destinations.get_mut(seat).unwrap().left = true;
            return;
        }
        if let Some(mut destination) = self.drag_destinations.remove(seat) {
            for token in destination.transfers.drain(..) {
                self.loop_handle.remove(token);
            }
            if destination.entered && !destination.dropped {
                destination.event.phase = NativeDragDropPhase::Leave;
                destination.event.observed_at = std::time::Instant::now();
                self.events_sink.push_window_event(
                    WindowEvent::NativeDragDrop(destination.event.clone()),
                    destination.window,
                );
                self.events_sink
                    .push_window_event(WindowEvent::HoveredFileCancelled, destination.window);
            }
            if destination.dropped {
                destination.finish();
            }
            self.drag_offers.remove(&destination.offer.id());
            destination.offer.destroy();
        }
    }

    /// Apply asynchronous Page feedback on the Wayland sequence. No compositor
    /// preference or document operation policy is selected by this backend.
    pub fn flush_native_drag_feedback(&mut self) {
        for destination in self.drag_destinations.values_mut() {
            if destination.dropped {
                continue;
            }
            let operation = destination.event.operation.load(Ordering::Acquire)
                & destination.event.source_operations;
            if destination.last_operation == Some(operation) {
                continue;
            }
            destination.last_operation = Some(operation);
            let native = to_wayland_action(operation);
            if destination.offer.version() >= 3 {
                destination.offer.set_actions(native, native);
            }
            destination.offer.accept(
                destination.serial,
                (operation != 0).then(|| destination.mime.clone()).flatten(),
            );
        }
    }
}

fn to_wayland_action(operation: u32) -> DndAction {
    // wl_data_device has no link operation; it must never be advertised as copy.
    match operation {
        1 => DndAction::Copy,
        16 => DndAction::Move,
        _ => DndAction::empty(),
    }
}
fn from_wayland_action(action: DndAction) -> u32 {
    u32::from(action.contains(DndAction::Copy)) | (u32::from(action.contains(DndAction::Move)) << 4)
}

impl Dispatch<WlDataDeviceManager, ()> for WinitState {
    fn event(
        _: &mut Self,
        _: &WlDataDeviceManager,
        _: <WlDataDeviceManager as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<WlDataDevice, DragDeviceData> for WinitState {
    fn event(
        state: &mut Self,
        _: &WlDataDevice,
        event: wl_data_device::Event,
        data: &DragDeviceData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => {
                state
                    .drag_offers
                    .entry(id.id())
                    .or_insert_with(|| NativeOffer::new(id));
            }
            wl_data_device::Event::Enter {
                serial,
                surface,
                x,
                y,
                id: Some(offer),
            } => {
                state.begin_native_destination(data, serial, &surface, x, y, offer);
            }
            wl_data_device::Event::Motion { x, y, .. } => {
                if let Some(destination) = state.drag_destinations.get_mut(&data.seat) {
                    if let Some(window) = state.windows.get_mut().get(&destination.window) {
                        destination.event.position = LogicalPosition::new(x, y)
                            .to_physical(window.lock().unwrap().scale_factor());
                    }
                    if let Some(seat) = state.seats.get(&data.seat) {
                        destination.event.modifiers = i32::from(seat.modifiers.state().shift_key())
                            | i32::from(seat.modifiers.state().control_key()) << 1
                            | i32::from(seat.modifiers.state().alt_key()) << 2
                            | i32::from(seat.modifiers.state().super_key()) << 3;
                    }
                    destination.event.observed_at = std::time::Instant::now();
                    if destination.entered {
                        destination.event.phase = NativeDragDropPhase::Over;
                        state.events_sink.push_window_event(
                            WindowEvent::NativeDragDrop(destination.event.clone()),
                            destination.window,
                        );
                    }
                }
                state.flush_native_drag_feedback();
            }
            wl_data_device::Event::Drop => {
                if let Some(destination) = state.drag_destinations.get_mut(&data.seat) {
                    destination.dropped = true;
                    if destination.entered {
                        for path in &destination.event.paths {
                            state.events_sink.push_window_event(
                                WindowEvent::DroppedFile(path.clone()),
                                destination.window,
                            );
                        }
                        destination.event.phase = NativeDragDropPhase::Drop;
                        destination.event.observed_at = std::time::Instant::now();
                        state.events_sink.push_window_event(
                            WindowEvent::NativeDragDrop(destination.event.clone()),
                            destination.window,
                        );
                    }
                    destination.finish();
                }
            }
            wl_data_device::Event::Leave => state.leave_native_destination(&data.seat),
            wl_data_device::Event::Selection { id: Some(offer) } => {
                // Clipboard ownership belongs to the clipboard service, not drag.
                state.drag_offers.remove(&offer.id());
                offer.destroy();
            }
            _ => (),
        }
    }
    sctk::reexports::client::event_created_child!(WinitState, WlDataDevice, [0 => (WlDataOffer, ())]);
}
impl Dispatch<WlDataOffer, ()> for WinitState {
    fn event(
        state: &mut Self,
        offer: &WlDataOffer,
        event: wl_data_offer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let native = state
            .drag_offers
            .entry(offer.id())
            .or_insert_with(|| NativeOffer::new(offer.clone()));
        match event {
            wl_data_offer::Event::Offer { mime_type } => {
                if !native.mime_types.contains(&mime_type) {
                    native.mime_types.push(mime_type);
                }
            }
            wl_data_offer::Event::Action {
                dnd_action: WEnum::Value(action),
            } => {
                for destination in state
                    .drag_destinations
                    .values_mut()
                    .filter(|d| d.offer == *offer)
                {
                    destination.selected_operation = from_wayland_action(action);
                }
            }
            wl_data_offer::Event::SourceActions {
                source_actions: WEnum::Value(actions),
            } => {
                native.source_operations = from_wayland_action(actions);
                for destination in state
                    .drag_destinations
                    .values_mut()
                    .filter(|d| d.offer == *offer)
                {
                    destination.event.source_operations = native.source_operations;
                }
            }
            _ => (),
        }
    }
}

fn local_uri_paths(bytes: &[u8]) -> Vec<std::path::PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Vec::new();
    };
    text.lines()
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
            let mut decoded = Vec::new();
            let mut bytes = path.as_bytes().iter().copied();
            while let Some(byte) = bytes.next() {
                if byte == b'%' {
                    let high = (bytes.next()? as char).to_digit(16)?;
                    let low = (bytes.next()? as char).to_digit(16)?;
                    decoded.push((high * 16 + low) as u8);
                } else {
                    decoded.push(byte);
                }
            }
            if decoded.contains(&0) {
                return None;
            }
            Some(std::ffi::OsString::from_vec(decoded).into())
        })
        .collect()
}
