//! Native drag-session transport. DOM/DataTransfer owners never leave Page.
use crate::page_presentation::Document;
use interaction::{
    default::core::page::drag_controller::{SystemDragRequest, SystemDragSupport},
    dispatch::core::clipboard::data_transfer::DragOperation,
};
use std::{
    sync::{mpsc, Arc},
    time::Instant,
};

#[cfg(target_os = "linux")]
mod linux_payload;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod payload;
#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "linux")]
enum LinuxSession {
    X11(x11::Session),
    Wayland(wayland::Session),
}
#[cfg(target_os = "linux")]
thread_local! {
    static LINUX_SESSION: std::cell::RefCell<Option<LinuxSession>> = const { std::cell::RefCell::new(None) };
}

#[derive(Clone)]
pub(crate) struct Start {
    pub document: Document,
    pub request: Arc<SystemDragRequest>,
    pub origin: (f64, f64),
    pub reply: mpsc::Sender<crate::page_runtime::Command>,
}
impl std::fmt::Debug for Start {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeDragStart")
            .field("document", &self.document)
            .field("pointer_id", &self.request.pointer_id)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Failure {
    Unsupported,
    InvalidPayload,
    BusyOrUnavailable,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Outcome {
    Ended(DragOperation),
    Cancelled,
    Failed(Failure),
}
#[derive(Clone, Debug)]
pub(crate) struct Completion {
    pub document: Document,
    pub pointer_id: i32,
    pub position: Option<(f32, f32)>,
    /// Wayland does not expose a global drop coordinate to its drag source.
    pub screen_position: Option<(f32, f32)>,
    pub completed_at: Instant,
    pub outcome: Outcome,
}
impl Start {
    pub(crate) fn fail(self, failure: Failure) {
        self.complete_unlocated(Outcome::Failed(failure));
    }
    fn complete(self, position: (f32, f32), screen_position: (f32, f32), outcome: Outcome) {
        self.complete_with_positions(Some(position), Some(screen_position), outcome);
    }
    fn complete_unlocated(self, outcome: Outcome) {
        self.complete_with_positions(None, None, outcome);
    }
    fn complete_with_positions(
        self,
        position: Option<(f32, f32)>,
        screen_position: Option<(f32, f32)>,
        outcome: Outcome,
    ) {
        let _ = self
            .reply
            .send(crate::page_runtime::Command::SystemDragEnded(Completion {
                document: self.document,
                pointer_id: self.request.pointer_id,
                position: position.map(|position| {
                    (
                        position.0 - self.origin.0 as f32,
                        position.1 - self.origin.1 as f32,
                    )
                }),
                screen_position,
                completed_at: Instant::now(),
                outcome,
            }));
    }
}
/// winit currently exposes incoming file drops, not outgoing OS drag sessions.
/// Only an implemented native source backend advertises support.
pub(crate) fn support(window: bool, request: &SystemDragRequest) -> SystemDragSupport {
    if cfg!(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux"
    )) && window
        && request.data.file_system_files.is_empty()
        && request
            .data
            .file_items
            .iter()
            .all(|file| file.HasBackingFile() || file.CachedBytes().is_ok())
    {
        SystemDragSupport::Supported
    } else {
        SystemDragSupport::Unsupported
    }
}
pub(crate) fn start(window: Option<&winit::window::Window>, start: Start) {
    #[cfg(target_os = "macos")]
    if let Some(window) = window {
        macos::start(window, start);
        return;
    }
    #[cfg(target_os = "windows")]
    if let Some(window) = window {
        windows::start(window, start);
        return;
    }
    #[cfg(target_os = "linux")]
    if let Some(window) = window {
        use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
        LINUX_SESSION.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                start.fail(Failure::BusyOrUnavailable);
                return;
            }
            let session = match window.display_handle().map(|handle| handle.as_raw()) {
                Ok(RawDisplayHandle::Xlib(_)) => {
                    x11::Session::new(window, start).map(LinuxSession::X11)
                }
                Ok(RawDisplayHandle::Wayland(_)) => {
                    wayland::Session::new(window, start).map(LinuxSession::Wayland)
                }
                _ => {
                    start.fail(Failure::Unsupported);
                    return;
                }
            };
            match session {
                Ok(session) => *slot = Some(session),
                Err((start, failure)) => start.fail(failure),
            }
        });
        return;
    }
    let _ = window;
    start.fail(Failure::Unsupported);
}

/// Dispatch only native drag transport on the UI sequence. The Page continues
/// asynchronously; no document drag decisions live in the host event loop.
pub(crate) fn poll() -> Option<Instant> {
    #[cfg(target_os = "linux")]
    return LINUX_SESSION.with(|slot| {
        let mut slot = slot.borrow_mut();
        let active = match slot.as_mut() {
            Some(LinuxSession::X11(session)) => session.poll(),
            Some(LinuxSession::Wayland(session)) => session.poll(),
            None => false,
        };
        if !active {
            *slot = None;
            return None;
        }
        Some(Instant::now() + std::time::Duration::from_millis(8))
    });
    #[cfg(not(target_os = "linux"))]
    None
}
/// Destroy native sources before winit destroys the borrowed display/window.
pub(crate) fn shutdown() {
    #[cfg(target_os = "linux")]
    LINUX_SESSION.with(|slot| {
        slot.borrow_mut().take();
    });
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::{
        ffi::{c_char, c_void, CString},
        ptr,
    };
    #[repr(C)]
    struct NativeString {
        mime: *const c_char,
        value: *const u16,
        length: usize,
    }
    #[repr(C)]
    struct NativeFile {
        name: *const u16,
        name_length: usize,
        mime: *const c_char,
        path: *const c_char,
        bytes: *const u8,
        length: usize,
    }
    #[repr(C)]
    struct NativeImage {
        rgba: *const u8,
        length: usize,
        width: u32,
        height: u32,
        row_bytes: usize,
    }
    unsafe extern "C" {
        fn rechrom_start_native_drag(
            view: *mut c_void,
            strings: *const NativeString,
            strings_len: usize,
            files: *const NativeFile,
            files_len: usize,
            image: *const NativeImage,
            offset_x: i32,
            offset_y: i32,
            operation_mask: u32,
            context: *mut c_void,
            complete: unsafe extern "C" fn(*mut c_void, f64, f64, f64, f64, u32),
        ) -> bool;
    }
    struct OwnedString {
        mime: CString,
        value: Vec<u16>,
    }
    struct OwnedFile {
        name: Vec<u16>,
        mime: CString,
        path: Option<CString>,
        bytes: Arc<[u8]>,
    }
    fn text(mime: &str, value: Vec<u16>) -> Result<OwnedString, Failure> {
        Ok(OwnedString {
            mime: CString::new(mime).map_err(|_| Failure::InvalidPayload)?,
            value,
        })
    }
    fn payload(request: &SystemDragRequest) -> Result<(Vec<OwnedString>, Vec<OwnedFile>), Failure> {
        let mut strings = Vec::new();
        for (mime, value) in &request.data.strings {
            strings.push(text(mime, value.encode_utf16().collect())?);
        }
        for (mime, value) in &request.data.utf16_strings {
            strings.push(text(mime, value.to_vec())?);
        }
        for (mime, value) in [
            (
                "org.chromium.source-effect-allowed",
                &request.data.source_effect_allowed,
            ),
            ("org.chromium.html-base-url", &request.data.html_base_url),
            ("public.url-name", &request.data.url_title),
        ] {
            if let Some(value) = value {
                strings.push(text(mime, value.encode_utf16().collect())?);
            }
        }
        let mut files = Vec::new();
        for file in &request.data.files {
            if !file.is_image_accessible {
                return Err(Failure::Unsupported);
            }
            let name = net_filename::GetSafeFilenameForImageFileContents(
                &file.source_url,
                &file.content_disposition,
                &file.filename_extension,
            )
            .ok_or(Failure::Unsupported)?;
            files.push(OwnedFile {
                name: name.filename.encode_utf16().collect(),
                mime: CString::new(name.well_known_mime_type)
                    .map_err(|_| Failure::InvalidPayload)?,
                path: None,
                bytes: file.bytes.clone(),
            });
        }
        for file in &request.data.file_items {
            let path = if file.HasBackingFile() {
                Some(CString::new(file.GetPath()).map_err(|_| Failure::InvalidPayload)?)
            } else {
                None
            };
            let bytes = if path.is_some() {
                Arc::from([])
            } else {
                file.CachedBytes().map_err(|_| Failure::InvalidPayload)?
            };
            files.push(OwnedFile {
                name: file.name().encode_utf16().collect(),
                mime: CString::new(file.type_()).map_err(|_| Failure::InvalidPayload)?,
                path,
                bytes,
            });
        }
        Ok((strings, files))
    }
    pub(super) fn start(window: &winit::window::Window, start: Start) {
        if support(true, &start.request) != SystemDragSupport::Supported {
            start.fail(Failure::Unsupported);
            return;
        }
        let Ok(handle) = window.window_handle() else {
            start.fail(Failure::BusyOrUnavailable);
            return;
        };
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            start.fail(Failure::Unsupported);
            return;
        };
        let (strings, files) = match payload(&start.request) {
            Ok(data) => data,
            Err(error) => {
                start.fail(error);
                return;
            }
        };
        let native_strings: Vec<_> = strings
            .iter()
            .map(|value| NativeString {
                mime: value.mime.as_ptr(),
                value: value.value.as_ptr(),
                length: value.value.len(),
            })
            .collect();
        let native_files: Vec<_> = files
            .iter()
            .map(|value| NativeFile {
                name: value.name.as_ptr(),
                name_length: value.name.len(),
                mime: value.mime.as_ptr(),
                path: value.path.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
                bytes: value.bytes.as_ptr(),
                length: value.bytes.len(),
            })
            .collect();
        let image = start.request.image.as_ref().map(|image| NativeImage {
            rgba: image.rgba8.as_ptr(),
            length: image.rgba8.len(),
            width: image.width,
            height: image.height,
            row_bytes: image.row_bytes,
        });
        if image.as_ref().is_some_and(|image| {
            image.width == 0
                || image.height == 0
                || image.row_bytes < image.width as usize * 4
                || image
                    .row_bytes
                    .checked_mul(image.height as usize)
                    .is_none_or(|length| length > image.length)
        }) {
            start.fail(Failure::InvalidPayload);
            return;
        }
        let offset = start.request.cursor_offset;
        let operations = start.request.source_operations;
        let context = Box::into_raw(Box::new(start));
        // SAFETY: App calls this only on winit's AppKit/UI sequence. The bridge
        // copies payloads synchronously and owns context until one end callback.
        let accepted = unsafe {
            rechrom_start_native_drag(
                handle.ns_view.as_ptr(),
                native_strings.as_ptr(),
                native_strings.len(),
                native_files.as_ptr(),
                native_files.len(),
                image.as_ref().map_or(ptr::null(), |image| image),
                offset.0,
                offset.1,
                operations,
                context.cast(),
                complete,
            )
        };
        if !accepted {
            unsafe { Box::from_raw(context) }.fail(Failure::BusyOrUnavailable);
        }
    }
    unsafe extern "C" fn complete(
        context: *mut c_void,
        x: f64,
        y: f64,
        sx: f64,
        sy: f64,
        operations: u32,
    ) {
        // Ownership is consumed exactly once by NSDraggingSource::endedAtPoint.
        let start = unsafe { Box::from_raw(context.cast::<Start>()) };
        let operation = if operations & 16 != 0 {
            DragOperation::Move
        } else if operations & 1 != 0 {
            DragOperation::Copy
        } else if operations & 2 != 0 {
            DragOperation::Link
        } else {
            DragOperation::None
        };
        let outcome = if operation == DragOperation::None {
            Outcome::Cancelled
        } else {
            Outcome::Ended(operation)
        };
        start.complete((x as f32, y as f32), (sx as f32, sy as f32), outcome);
    }
}

/// Host target packet after native platform conversion. The native feedback
/// owner is independent of Page lifetime and never contains a DOM callback.
#[derive(Clone)]
pub(crate) struct Target {
    pub phase: winit::event::NativeDragDropPhase,
    pub data: Option<Arc<interaction::dispatch::core::clipboard::data_transfer::WebDragData>>,
    pub position: (f32, f32),
    pub screen_position: Option<(f32, f32)>,
    pub modifiers: i32,
    pub source_operations: u32,
    pub operation: Arc<std::sync::atomic::AtomicU32>,
    /// Monotonic host request identity. Every routed enter/update/leave/drop
    /// publishes its token before enqueueing the Page command; a late reply
    /// from an older surface may not overwrite feedback for the newer target.
    pub request_token: u64,
    pub feedback_generation: Arc<std::sync::atomic::AtomicU64>,
    pub observed_at: Instant,
}
impl Target {
    pub(crate) fn for_request(mut self) -> Self {
        static NEXT_REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let token = NEXT_REQUEST.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.feedback_generation
            .store(token, std::sync::atomic::Ordering::Release);
        self.request_token = token;
        self
    }

    pub(crate) fn from_native(
        event: &winit::event::NativeDragDropEvent,
        scale: f64,
    ) -> std::io::Result<Self> {
        let data = if event.phase == winit::event::NativeDragDropPhase::Enter {
            if event.paths.len() != event.files.len()
                || !event
                    .paths
                    .iter()
                    .zip(&event.files)
                    .all(|(path, offer)| path == &offer.path)
            {
                return Err(std::io::Error::other(
                    "native drag files require matching retained readable offer handles",
                ));
            }
            let files = event
                .files
                .iter()
                .map(crate::native_file::from_offer)
                .chain(
                    event
                        .virtual_files
                        .iter()
                        .map(crate::native_file::from_virtual_offer),
                )
                .collect::<std::io::Result<Vec<_>>>()?;
            Some(Arc::new(
                interaction::dispatch::core::clipboard::data_transfer::WebDragData {
                    strings: Vec::new(),
                    utf16_strings: event
                        .strings
                        .iter()
                        .map(|(kind, value)| (kind.clone(), Arc::from(value.as_slice())))
                        .collect(),
                    source_effect_allowed: None,
                    html_base_url: None,
                    url_title: None,
                    files: Vec::new(),
                    file_items: files,
                    file_system_files: Vec::new(),
                },
            ))
        } else {
            None
        };
        Ok(Self {
            phase: event.phase,
            data,
            position: (
                (event.position.x / scale) as f32,
                (event.position.y / scale) as f32,
            ),
            screen_position: event
                .screen_position
                .map(|p| ((p.x / scale) as f32, (p.y / scale) as f32)),
            modifiers: event.modifiers,
            source_operations: event.source_operations,
            operation: event.operation.clone(),
            request_token: 0,
            feedback_generation: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            observed_at: event.observed_at,
        })
    }
}
