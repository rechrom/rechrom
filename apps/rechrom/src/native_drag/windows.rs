//! OLE runs its native drag loop on the window's STA/UI sequence.
use super::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;

#[repr(C)]
struct StringItem {
    mime: *const u16,
    mime_len: usize,
    text: *const u16,
    len: usize,
}
#[repr(C)]
struct FileItem {
    name: *const u16,
    name_len: usize,
    path: *const u16,
    path_len: usize,
    bytes: *const u8,
    len: usize,
}
#[repr(C)]
struct Image {
    rgba: *const u8,
    width: u32,
    height: u32,
    stride: usize,
}
unsafe extern "C" {
    fn rechrom_do_drag_drop(
        window: *mut c_void,
        strings: *const StringItem,
        strings_len: usize,
        files: *const FileItem,
        files_len: usize,
        image: *const Image,
        x: i32,
        y: i32,
        operations: u32,
        result: *mut u32,
        point: *mut i32,
    ) -> i32;
}
pub(super) fn start(window: &winit::window::Window, start: Start) {
    let Ok(handle) = window.window_handle() else {
        start.fail(Failure::BusyOrUnavailable);
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        start.fail(Failure::Unsupported);
        return;
    };
    let payload = match super::payload::build(&start.request) {
        Ok(payload) => payload,
        Err(error) => {
            start.fail(error);
            return;
        }
    };
    let mimes: Vec<Vec<u16>> = payload
        .strings
        .iter()
        .map(|(mime, _)| mime.encode_utf16().collect())
        .collect();
    let strings: Vec<_> = payload
        .strings
        .iter()
        .zip(&mimes)
        .map(|((_, text), mime)| StringItem {
            mime: mime.as_ptr(),
            mime_len: mime.len(),
            text: text.as_ptr(),
            len: text.len(),
        })
        .collect();
    let paths: Vec<Vec<u16>> = payload
        .files
        .iter()
        .map(|file| file.path.as_deref().unwrap_or("").encode_utf16().collect())
        .collect();
    let files: Vec<_> = payload
        .files
        .iter()
        .zip(&paths)
        .map(|(file, path)| FileItem {
            name: file.name.as_ptr(),
            name_len: file.name.len(),
            path: path.as_ptr(),
            path_len: path.len(),
            bytes: file.bytes.as_ptr(),
            len: file.bytes.len(),
        })
        .collect();
    let image = start.request.image.as_ref().map(|image| Image {
        rgba: image.rgba8.as_ptr(),
        width: image.width,
        height: image.height,
        stride: image.row_bytes,
    });
    let mut effect = 0;
    let mut point = [0i32; 4];
    // The bridge retains COM data through DoDragDrop, including delayed stream
    // requests; Start remains here and replies only after the native result.
    let status = unsafe {
        rechrom_do_drag_drop(
            handle.hwnd.get() as *mut c_void,
            strings.as_ptr(),
            strings.len(),
            files.as_ptr(),
            files.len(),
            image.as_ref().map_or(std::ptr::null(), |image| image),
            start.request.cursor_offset.0,
            start.request.cursor_offset.1,
            start.request.source_operations,
            &mut effect,
            point.as_mut_ptr(),
        )
    };
    let operation = match effect {
        1 => DragOperation::Copy,
        2 => DragOperation::Link,
        16 => DragOperation::Move,
        _ => DragOperation::None,
    };
    let outcome = match status {
        0 => Outcome::Cancelled,
        1 => Outcome::Ended(operation),
        _ => Outcome::Failed(Failure::BusyOrUnavailable),
    };
    let scale = window.scale_factor() as f32;
    start.complete(
        (point[0] as f32 / scale, point[1] as f32 / scale),
        (point[2] as f32 / scale, point[3] as f32 / scale),
        outcome,
    );
}
