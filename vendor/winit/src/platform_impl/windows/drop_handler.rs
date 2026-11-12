mod virtual_file;

pub(crate) fn clipboard_virtual_files() -> std::io::Result<Vec<crate::event::NativeDragVirtualFileOffer>> {
    virtual_file::clipboard_files()
}

use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use windows_sys::core::{IUnknown, GUID, HRESULT};
use windows_sys::Win32::Foundation::{DV_E_FORMATETC, HWND, POINT, POINTL, S_OK};
use windows_sys::Win32::System::Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
use windows_sys::Win32::System::Ole::CF_HDROP;
use windows_sys::Win32::UI::Shell::{DragQueryFileW, HDROP};

use tracing::debug;

use crate::platform_impl::platform::definitions::{
    IDataObjectVtbl, IDropTarget, IDropTargetVtbl, IUnknownVtbl,
};
use crate::platform_impl::platform::WindowId;

use crate::event::Event;
use crate::window::WindowId as RootWindowId;

#[repr(C)]
pub struct FileDropHandlerData {
    pub interface: IDropTarget,
    refcount: AtomicUsize,
    window: HWND,
    send_event: Box<dyn Fn(Event<()>)>,
    target_drag: Option<crate::event::NativeDragDropEvent>,
    virtual_session: Option<virtual_file::Session>,
    hovered_is_valid: bool, /* If the currently hovered item is not valid there must not be any
                             * `HoveredFileCancelled` emitted */
}

impl FileDropHandlerData {
    unsafe fn queue_native(
        &mut self,
        phase: crate::event::NativeDragDropPhase,
        keys: u32,
        point: Option<&POINTL>,
        effect: *mut u32,
    ) {
        let allowed = unsafe { *effect };
        let Some(mut event) = self.target_drag.clone() else {
            unsafe { *effect = 0 };
            return;
        };
        if phase != crate::event::NativeDragDropPhase::Leave {
            event.source_operations = (allowed & 1) | ((allowed & 4) >> 1) | ((allowed & 2) << 3);
        }
        if let Some(point) = point {
            let mut local = POINT {
                x: point.x,
                y: point.y,
            };
            if unsafe { windows_sys::Win32::Graphics::Gdi::ScreenToClient(self.window, &mut local) }
                == 0
            {
                unsafe { *effect = 0 };
                return;
            }
            event.position = crate::dpi::PhysicalPosition::new(local.x as f64, local.y as f64);
            event.screen_position = Some(crate::dpi::PhysicalPosition::new(
                point.x as f64,
                point.y as f64,
            ));
        }
        let alt = unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x12) } < 0;
        let meta = unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x5b) }
            < 0
            || unsafe { windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x5c) } < 0;
        event.modifiers = i32::from(keys & 4 != 0)
            | i32::from(keys & 8 != 0) << 1
            | i32::from(alt) << 2
            | i32::from(meta) << 3;
        event.phase = phase;
        event.observed_at = std::time::Instant::now();
        let operation = event.operation.load(Ordering::Acquire) & event.source_operations;
        unsafe {
            *effect = (operation & 1) | ((operation & 2) << 1) | ((operation & 16) >> 3);
        }
        self.target_drag = Some(event.clone());
        self.send_event(Event::WindowEvent {
            window_id: RootWindowId(WindowId(self.window)),
            event: crate::event::WindowEvent::NativeDragDrop(event),
        });
    }
}
pub struct FileDropHandler {
    pub data: *mut FileDropHandlerData,
}

#[allow(non_snake_case)]
impl FileDropHandler {
    pub fn new(window: HWND, send_event: Box<dyn Fn(Event<()>)>) -> FileDropHandler {
        let data = Box::new(FileDropHandlerData {
            interface: IDropTarget {
                lpVtbl: &DROP_TARGET_VTBL as *const IDropTargetVtbl,
            },
            refcount: AtomicUsize::new(1),
            window,
            send_event,
            hovered_is_valid: false,
            target_drag: None,
            virtual_session: None,
        });
        FileDropHandler {
            data: Box::into_raw(data),
        }
    }

    // Implement IUnknown
    pub unsafe extern "system" fn QueryInterface(
        _this: *mut IUnknown,
        _riid: *const GUID,
        _ppvObject: *mut *mut c_void,
    ) -> HRESULT {
        if _ppvObject.is_null() {
            return windows_sys::Win32::Foundation::E_POINTER;
        }
        unsafe {
            *_ppvObject = ptr::null_mut();
        }
        if _riid.is_null() {
            return windows_sys::Win32::Foundation::E_NOINTERFACE;
        }
        // IUnknown and IDropTarget share this object's canonical interface.
        let iid = unsafe { &*_riid };
        if (iid.data1 == 0
            && iid.data2 == 0
            && iid.data3 == 0
            && iid.data4 == [0xc0, 0, 0, 0, 0, 0, 0, 0x46])
            || (iid.data1 == 0x122
                && iid.data2 == 0
                && iid.data3 == 0
                && iid.data4 == [0xc0, 0, 0, 0, 0, 0, 0, 0x46])
        {
            unsafe {
                *_ppvObject = _this.cast();
                Self::AddRef(_this);
            }
            S_OK
        } else {
            windows_sys::Win32::Foundation::E_NOINTERFACE
        }
    }

    pub unsafe extern "system" fn AddRef(this: *mut IUnknown) -> u32 {
        let drop_handler_data = unsafe { Self::from_interface(this) };
        let count = drop_handler_data.refcount.fetch_add(1, Ordering::Relaxed) + 1;
        count as u32
    }

    pub unsafe extern "system" fn Release(this: *mut IUnknown) -> u32 {
        let drop_handler = unsafe { Self::from_interface(this) };
        let count = drop_handler.refcount.fetch_sub(1, Ordering::Release) - 1;
        if count == 0 {
            std::sync::atomic::fence(Ordering::Acquire);
            // Destroy the underlying data
            drop(unsafe { Box::from_raw(drop_handler as *mut FileDropHandlerData) });
        }
        count as u32
    }

    pub unsafe extern "system" fn DragEnter(
        this: *mut IDropTarget,
        data: *const IDataObject,
        keys: u32,
        point: POINTL,
        effect: *mut u32,
    ) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        let paths = std::cell::RefCell::new(Vec::new());
        unsafe {
            Self::iterate_filenames(data, |path| paths.borrow_mut().push(path));
        }
        let files: Vec<_> = paths
            .into_inner()
            .into_iter()
            .filter_map(|path| crate::event::NativeDragFileOffer::open(path).ok())
            .collect();
        let paths: Vec<_> = files.iter().map(|file| file.path.clone()).collect();
        let (virtual_session, virtual_files) = unsafe { virtual_file::Session::new(data) }
            .map_or((None, Vec::new()), |(session, files)| {
                (Some(session), files)
            });
        target.virtual_session = virtual_session;
        let mut strings: Vec<(String, Vec<u16>)> = Vec::new();
        for (format, mime, wide) in [
            (13u16, "text/plain", true),
            (
                Self::clipboard_format("UniformResourceLocatorW"),
                "text/uri-list",
                true,
            ),
            (Self::clipboard_format("HTML Format"), "text/html", false),
        ] {
            if let Some(bytes) = unsafe { Self::read_global(data, format) } {
                let value = if wide {
                    let units: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|pair| u16::from_ne_bytes([pair[0], pair[1]]))
                        .take_while(|c| *c != 0)
                        .collect();
                    Some(units)
                } else {
                    std::str::from_utf8(&bytes).ok().map(|html| {
                        let offset = |name: &str| {
                            html.lines().find_map(|line| {
                                line.strip_prefix(name)
                                    .and_then(|value| value.trim().parse::<usize>().ok())
                            })
                        };
                        let body = offset("StartFragment:")
                            .zip(offset("EndFragment:"))
                            .and_then(|(start, end)| html.get(start..end))
                            .unwrap_or(html.trim_end_matches('\0'));
                        body.encode_utf16().collect()
                    })
                };
                if let Some(value) = value {
                    strings.push((mime.into(), value));
                }
            }
        }
        target.hovered_is_valid =
            !paths.is_empty() || !virtual_files.is_empty() || !strings.is_empty();
        if !target.hovered_is_valid {
            unsafe { *effect = 0 };
            return S_OK;
        }
        for path in &paths {
            target.send_event(crate::event::Event::WindowEvent {
                window_id: RootWindowId(WindowId(target.window)),
                event: crate::event::WindowEvent::HoveredFile(path.clone()),
            });
        }
        target.target_drag = Some(crate::event::NativeDragDropEvent {
            phase: crate::event::NativeDragDropPhase::Enter,
            files,
            virtual_files,
            paths,
            data: strings
                .iter()
                .map(|(mime, value)| {
                    (
                        mime.clone(),
                        Arc::<[u8]>::from(String::from_utf16_lossy(value).into_bytes()),
                    )
                })
                .collect(),
            strings,
            position: crate::dpi::PhysicalPosition::new(0., 0.),
            screen_position: None,
            modifiers: 0,
            source_operations: 0,
            operation: Arc::new(AtomicU32::new(0)),
            observed_at: std::time::Instant::now(),
        });
        unsafe {
            target.queue_native(
                crate::event::NativeDragDropPhase::Enter,
                keys,
                Some(&point),
                effect,
            );
        }
        S_OK
    }
    pub unsafe extern "system" fn DragOver(
        this: *mut IDropTarget,
        keys: u32,
        point: POINTL,
        effect: *mut u32,
    ) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        unsafe {
            target.queue_native(
                crate::event::NativeDragDropPhase::Over,
                keys,
                Some(&point),
                effect,
            );
        }
        S_OK
    }
    pub unsafe extern "system" fn DragLeave(this: *mut IDropTarget) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        let mut effect = 0;
        unsafe {
            target.queue_native(
                crate::event::NativeDragDropPhase::Leave,
                0,
                None,
                &mut effect,
            );
        }
        if target.hovered_is_valid {
            target.send_event(Event::WindowEvent {
                window_id: RootWindowId(WindowId(target.window)),
                event: crate::event::WindowEvent::HoveredFileCancelled,
            });
        }
        target.hovered_is_valid = false;
        target.target_drag = None;
        target.virtual_session = None;
        S_OK
    }
    pub unsafe extern "system" fn Drop(
        this: *mut IDropTarget,
        _data: *const IDataObject,
        keys: u32,
        point: POINTL,
        effect: *mut u32,
    ) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        if let Some(event) = target.target_drag.as_ref() {
            for path in &event.paths {
                target.send_event(Event::WindowEvent {
                    window_id: RootWindowId(WindowId(target.window)),
                    event: crate::event::WindowEvent::DroppedFile(path.clone()),
                });
            }
        }
        if let Some(session) = target.virtual_session.take() {
            let operation = target.target_drag.as_ref().map_or(0, |event| {
                event.operation.load(Ordering::Acquire) & event.source_operations
            });
            if operation != 0 {
                session.start();
            }
        }
        unsafe {
            target.queue_native(
                crate::event::NativeDragDropPhase::Drop,
                keys,
                Some(&point),
                effect,
            );
        }
        target.target_drag = None;
        target.hovered_is_valid = false;
        S_OK
    }
    fn clipboard_format(name: &str) -> u16 {
        let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        unsafe {
            windows_sys::Win32::System::DataExchange::RegisterClipboardFormatW(name.as_ptr()) as u16
        }
    }
    unsafe fn read_global(data: *const IDataObject, format: u16) -> Option<Vec<u8>> {
        use windows_sys::Win32::System::{
            Memory::{GlobalLock, GlobalSize, GlobalUnlock},
            Ole::ReleaseStgMedium,
        };
        let format = FORMATETC {
            cfFormat: format,
            ptd: ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL as u32,
        };
        let mut medium = unsafe { std::mem::zeroed() };
        let get = unsafe { (*(*data).cast::<IDataObjectVtbl>()).GetData };
        if unsafe { get(data as *mut _, &format, &mut medium) } < 0 {
            return None;
        }
        let memory = unsafe { medium.u.hGlobal };
        let size = unsafe { GlobalSize(memory) };
        let value = unsafe { GlobalLock(memory) };
        let result = if value.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(value.cast::<u8>(), size) }.to_vec())
        };
        if !value.is_null() {
            unsafe {
                GlobalUnlock(memory);
            }
        }
        unsafe {
            ReleaseStgMedium(&mut medium);
        }
        result
    }
    unsafe fn from_interface<'a, InterfaceT>(this: *mut InterfaceT) -> &'a mut FileDropHandlerData {
        unsafe { &mut *(this as *mut _) }
    }

    unsafe fn iterate_filenames<F>(data_obj: *const IDataObject, callback: F) -> Option<HDROP>
    where
        F: Fn(PathBuf),
    {
        let drop_format = FORMATETC {
            cfFormat: CF_HDROP,
            ptd: ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL as u32,
        };

        let mut medium = unsafe { std::mem::zeroed() };
        let get_data_fn = unsafe { (*(*data_obj).cast::<IDataObjectVtbl>()).GetData };
        let get_data_result = unsafe { get_data_fn(data_obj as *mut _, &drop_format, &mut medium) };
        if get_data_result >= 0 {
            let hdrop = unsafe { medium.u.hGlobal as HDROP };

            // The second parameter (0xFFFFFFFF) instructs the function to return the item count
            let item_count = unsafe { DragQueryFileW(hdrop, 0xffffffff, ptr::null_mut(), 0) };

            for i in 0..item_count {
                // Get the length of the path string NOT including the terminating null character.
                // Previously, this was using a fixed size array of MAX_PATH length, but the
                // Windows API allows longer paths under certain circumstances.
                let character_count =
                    unsafe { DragQueryFileW(hdrop, i, ptr::null_mut(), 0) as usize };
                let str_len = character_count + 1;

                // Fill path_buf with the null-terminated file name
                let mut path_buf = Vec::with_capacity(str_len);
                unsafe {
                    DragQueryFileW(hdrop, i, path_buf.as_mut_ptr(), str_len as u32);
                    path_buf.set_len(str_len);
                }

                callback(OsString::from_wide(&path_buf[0..character_count]).into());
            }

            unsafe {
                windows_sys::Win32::System::Ole::ReleaseStgMedium(&mut medium);
            }
            Some(hdrop)
        } else if get_data_result == DV_E_FORMATETC {
            // If the dropped item is not a file this error will occur.
            // In this case it is OK to return without taking further action.
            debug!("Error occurred while processing dropped/hovered item: item is not a file.");
            None
        } else {
            debug!("Unexpected error occurred while processing dropped/hovered item.");
            None
        }
    }
}

impl FileDropHandlerData {
    fn send_event(&self, event: Event<()>) {
        (self.send_event)(event);
    }
}

impl Drop for FileDropHandler {
    fn drop(&mut self) {
        unsafe {
            FileDropHandler::Release(self.data as *mut IUnknown);
        }
    }
}

static DROP_TARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    parent: IUnknownVtbl {
        QueryInterface: FileDropHandler::QueryInterface,
        AddRef: FileDropHandler::AddRef,
        Release: FileDropHandler::Release,
    },
    DragEnter: FileDropHandler::DragEnter,
    DragOver: FileDropHandler::DragOver,
    DragLeave: FileDropHandler::DragLeave,
    Drop: FileDropHandler::Drop,
};
