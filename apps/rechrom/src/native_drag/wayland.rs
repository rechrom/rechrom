//! Wayland data-device drag source on winit's display and UI sequence.
//! Follows Chromium wayland_data_drag_controller.cc: StartSession, source
//! action negotiation, and data-source finished/cancelled lifecycle.
use super::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::{fs::OpenOptions, io::Write, os::fd::AsFd};
use wayland_client::{
    backend::{Backend, ObjectId},
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer, wl_compositor, wl_data_device, wl_data_device_manager, wl_data_offer,
        wl_data_source, wl_registry, wl_seat, wl_shm, wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};
use winit::platform::wayland::WindowExtWayland;

struct State {
    offers: super::linux_payload::Offers,
    operation: DragOperation,
    outcome: Option<Outcome>,
}
pub(super) struct Session {
    start: Option<Start>,
    connection: Connection,
    queue: EventQueue<State>,
    state: State,
    source: wl_data_source::WlDataSource,
    device: wl_data_device::WlDataDevice,
    icon: Option<(wl_surface::WlSurface, wl_buffer::WlBuffer)>,
}
impl Session {
    pub(super) fn new(
        window: &winit::window::Window,
        start: Start,
    ) -> Result<Self, (Start, Failure)> {
        match Self::prepare(window, &start) {
            Ok((connection, queue, state, source, device, icon)) => Ok(Self {
                start: Some(start),
                connection,
                queue,
                state,
                source,
                device,
                icon,
            }),
            Err(error) => Err((start, error)),
        }
    }
    fn prepare(
        window: &winit::window::Window,
        start: &Start,
    ) -> Result<
        (
            Connection,
            EventQueue<State>,
            State,
            wl_data_source::WlDataSource,
            wl_data_device::WlDataDevice,
            Option<(wl_surface::WlSurface, wl_buffer::WlBuffer)>,
        ),
        Failure,
    > {
        let (seat, serial) = window
            .drag_source_seat()
            .ok_or(Failure::BusyOrUnavailable)?;
        let display = window
            .display_handle()
            .map_err(|_| Failure::BusyOrUnavailable)?;
        let RawDisplayHandle::Wayland(display) = display.as_raw() else {
            return Err(Failure::Unsupported);
        };
        let handle = window
            .window_handle()
            .map_err(|_| Failure::BusyOrUnavailable)?;
        let RawWindowHandle::Wayland(handle) = handle.as_raw() else {
            return Err(Failure::Unsupported);
        };
        let (offers, _) = super::linux_payload::build(&start.request)?;
        // A foreign backend owns only its private queue and newly created
        // proxies. Winit retains display, origin surface, seat and their listeners.
        let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };
        let connection = Connection::from_backend(backend);
        let (globals, queue) =
            registry_queue_init::<State>(&connection).map_err(|_| Failure::BusyOrUnavailable)?;
        let qh = queue.handle();
        // v3 supplies a terminal dnd_finished event and negotiated action.
        let manager: wl_data_device_manager::WlDataDeviceManager = globals
            .bind(&qh, 3..=3, ())
            .map_err(|_| Failure::Unsupported)?;
        let seat_id =
            unsafe { ObjectId::from_ptr(wl_seat::WlSeat::interface(), seat.as_ptr().cast()) }
                .map_err(|_| Failure::BusyOrUnavailable)?;
        let seat = wl_seat::WlSeat::from_id(&connection, seat_id)
            .map_err(|_| Failure::BusyOrUnavailable)?;
        let surface_id = unsafe {
            ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                handle.surface.as_ptr().cast(),
            )
        }
        .map_err(|_| Failure::BusyOrUnavailable)?;
        let origin = wl_surface::WlSurface::from_id(&connection, surface_id)
            .map_err(|_| Failure::BusyOrUnavailable)?;
        let icon = if let Some(image) = &start.request.image {
            Some(make_icon(
                &globals,
                &qh,
                image,
                start.request.cursor_offset,
                window.scale_factor(),
            )?)
        } else {
            None
        };
        let device = manager.get_data_device(&seat, &qh, ());
        let source = manager.create_data_source(&qh, ());
        for (mime, _) in &offers {
            source.offer(mime.clone());
        }
        let operations = start.request.source_operations;
        let mut actions = wl_data_device_manager::DndAction::empty();
        // Wayland has no link action. Chromium advertises COPY for LINK.
        if operations & (1 | 2) != 0 {
            actions |= wl_data_device_manager::DndAction::Copy;
        }
        if operations & 16 != 0 {
            actions |= wl_data_device_manager::DndAction::Move;
        }
        source.set_actions(actions);
        device.start_drag(
            Some(&source),
            &origin,
            icon.as_ref().map(|(surface, _)| surface),
            serial,
        );
        connection.flush().map_err(|_| Failure::BusyOrUnavailable)?;
        Ok((
            connection,
            queue,
            State {
                offers,
                operation: DragOperation::None,
                outcome: None,
            },
            source,
            device,
            icon,
        ))
    }
    pub(super) fn poll(&mut self) -> bool {
        if self.queue.dispatch_pending(&mut self.state).is_err() || self.connection.flush().is_err()
        {
            self.state.outcome = Some(Outcome::Failed(Failure::BusyOrUnavailable));
        }
        if let Some(outcome) = self.state.outcome.take() {
            if let Some(start) = self.start.take() {
                // The protocol does not disclose the final drop coordinates.
                // Page keeps its own last input coordinates when these native
                // values are explicitly unavailable.
                start.complete_unlocated(outcome);
            }
            return false;
        }
        true
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.source.destroy();
        if self.device.version() >= 2 {
            self.device.release();
        }
        if let Some((surface, buffer)) = self.icon.take() {
            surface.destroy();
            buffer.destroy();
        }
        let _ = self.connection.flush();
        if let Some(start) = self.start.take() {
            start.complete_unlocated(Outcome::Cancelled);
        }
    }
}
fn make_icon(
    globals: &wayland_client::globals::GlobalList,
    qh: &QueueHandle<State>,
    image: &interaction::dispatch::core::clipboard::data_transfer::DragImageBitmap,
    offset: (i32, i32),
    scale: f64,
) -> Result<(wl_surface::WlSurface, wl_buffer::WlBuffer), Failure> {
    let compositor: wl_compositor::WlCompositor = globals
        .bind(qh, 1..=4, ())
        .map_err(|_| Failure::Unsupported)?;
    let shm: wl_shm::WlShm = globals
        .bind(qh, 1..=1, ())
        .map_err(|_| Failure::Unsupported)?;
    let length = (image.width as usize)
        .checked_mul(4)
        .and_then(|row| row.checked_mul(image.height as usize))
        .ok_or(Failure::InvalidPayload)?;
    let length_i32 = i32::try_from(length).map_err(|_| Failure::InvalidPayload)?;
    // An unlinked, real shared-memory backing is consumed by wl_shm, not a
    // temporary URL or cached clipboard surrogate for a promised file.
    let mut file = None;
    for nonce in 0..32 {
        let path =
            std::env::temp_dir().join(format!("rechrom-drag-icon-{}-{nonce}", std::process::id()));
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(backing) => {
                let _ = std::fs::remove_file(path);
                file = Some(backing);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(Failure::BusyOrUnavailable),
        }
    }
    let mut file = file.ok_or(Failure::BusyOrUnavailable)?;
    let mut bytes = Vec::with_capacity(length);
    for row in 0..image.height as usize {
        for pixel in image.rgba8
            [row * image.row_bytes..row * image.row_bytes + image.width as usize * 4]
            .chunks_exact(4)
        {
            bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }
    file.write_all(&bytes)
        .map_err(|_| Failure::BusyOrUnavailable)?;
    let pool = shm.create_pool(file.as_fd(), length_i32, qh, ());
    let buffer = pool.create_buffer(
        0,
        image.width as i32,
        image.height as i32,
        image.width as i32 * 4,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );
    pool.destroy();
    let surface = compositor.create_surface(qh, ());
    if surface.version() >= 3 {
        surface.set_buffer_scale(scale.round().max(1.) as i32);
    }
    surface.attach(Some(&buffer), -offset.0, -offset.1);
    surface.damage(0, 0, i32::MAX, i32::MAX);
    surface.commit();
    Ok((surface, buffer))
}
impl Dispatch<wl_data_source::WlDataSource, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_data_source::WlDataSource,
        event: wl_data_source::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_source::Event::Send { mime_type, fd } => {
                if let Some((_, bytes)) = state.offers.iter().find(|(mime, _)| *mime == mime_type) {
                    let bytes = bytes.clone();
                    // Pipe backpressure must never block the window event loop.
                    let _ = std::thread::Builder::new()
                        .name("drag-data-write".into())
                        .spawn(move || {
                            let mut pipe = std::fs::File::from(fd);
                            let _ = pipe.write_all(&bytes);
                        });
                }
            }
            wl_data_source::Event::Action { dnd_action } => {
                state.operation = match dnd_action {
                    WEnum::Value(wl_data_device_manager::DndAction::Copy) => DragOperation::Copy,
                    WEnum::Value(wl_data_device_manager::DndAction::Move) => DragOperation::Move,
                    _ => DragOperation::None,
                };
            }
            wl_data_source::Event::Cancelled => state.outcome = Some(Outcome::Cancelled),
            wl_data_source::Event::DndFinished => {
                state.outcome = Some(if state.operation == DragOperation::None {
                    Outcome::Cancelled
                } else {
                    Outcome::Ended(state.operation)
                })
            }
            _ => {}
        }
    }
}
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_data_device::WlDataDevice, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_data_device::Event::DataOffer { id } = event {
            id.destroy();
        }
        let _ = qh;
    }
    wayland_client::event_created_child!(State, wl_data_device::WlDataDevice, [0 => (wl_data_offer::WlDataOffer, ())]);
}
wayland_client::delegate_noop!(State: ignore wl_data_offer::WlDataOffer);
wayland_client::delegate_noop!(State: ignore wl_data_device_manager::WlDataDeviceManager);
wayland_client::delegate_noop!(State: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(State: ignore wl_shm::WlShm);
wayland_client::delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
wayland_client::delegate_noop!(State: ignore wl_buffer::WlBuffer);
wayland_client::delegate_noop!(State: ignore wl_surface::WlSurface);
