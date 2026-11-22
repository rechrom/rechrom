//! OLE drag destination for the browser window. winit's own IDropTarget is
//! disabled with `WindowAttributesExtWindows::with_drag_and_drop(false)`; this
//! target is registered on the same HWND and runs on the window's STA/UI
//! sequence while that sequence dispatches messages, as winit's did.
//!
//! Callbacks queue one `NativeDragDropEvent` packet per OLE call in arrival
//! order. The App routes the queue before any later winit callback (and at
//! `about_to_wait`), so packets keep their order relative to the window events
//! produced after them on the same thread.
mod virtual_file;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::dpi::PhysicalPosition;
use winit::event::{
    NativeDragDropEvent, NativeDragDropPhase, NativeDragFileOffer, NativeDragVirtualFileOffer,
};

type HRESULT = i32;
type Handle = *mut c_void;

const S_OK: HRESULT = 0;
const E_NOINTERFACE: HRESULT = 0x8000_4002_u32 as i32;
const E_POINTER: HRESULT = 0x8000_4003_u32 as i32;
const OLE_E_WRONGCOMPOBJ: HRESULT = 0x8004_000E_u32 as i32;
const RPC_E_CHANGED_MODE: HRESULT = 0x8001_0106_u32 as i32;
const DVASPECT_CONTENT: u32 = 1;
const TYMED_HGLOBAL: u32 = 1;
const TYMED_ISTREAM: u32 = 4;
const CF_HDROP: u16 = 15;
const CF_UNICODETEXT: u16 = 13;
const VK_MENU: i32 = 0x12;
const VK_LWIN: i32 = 0x5b;
const VK_RWIN: i32 = 0x5c;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PointL {
    x: i32,
    y: i32,
}
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct FileTime {
    low: u32,
    high: u32,
}
#[repr(C)]
struct FormatEtc {
    format: u16,
    target_device: *mut c_void,
    aspect: u32,
    index: i32,
    tymed: u32,
}
/// STGMEDIUM. Every member of its handle union is pointer-sized, so the union
/// is one pointer: HGLOBAL for TYMED_HGLOBAL, IStream* for TYMED_ISTREAM.
#[repr(C)]
struct StgMedium {
    tymed: u32,
    handle: *mut c_void,
    release_owner: *mut c_void,
}

#[repr(C)]
struct IUnknownVtbl {
    query_interface:
        unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}
/// IDataObject prefix: only GetData, the first method after IUnknown, is called.
#[repr(C)]
struct IDataObjectVtbl {
    unknown: IUnknownVtbl,
    get_data: unsafe extern "system" fn(*mut c_void, *const FormatEtc, *mut StgMedium) -> HRESULT,
}
/// IDropTarget. POINTL is passed by value (oleidl.h), not by pointer.
#[repr(C)]
struct IDropTargetVtbl {
    unknown: IUnknownVtbl,
    drag_enter:
        unsafe extern "system" fn(*mut c_void, *mut c_void, u32, PointL, *mut u32) -> HRESULT,
    drag_over: unsafe extern "system" fn(*mut c_void, u32, PointL, *mut u32) -> HRESULT,
    drag_leave: unsafe extern "system" fn(*mut c_void) -> HRESULT,
    drop: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, PointL, *mut u32) -> HRESULT,
}
#[repr(C)]
struct IDropTarget {
    vtbl: *const IDropTargetVtbl,
}

#[link(name = "ole32")]
extern "system" {
    fn OleInitialize(reserved: *mut c_void) -> HRESULT;
    fn RegisterDragDrop(window: Handle, target: *mut c_void) -> HRESULT;
    fn ReleaseStgMedium(medium: *mut StgMedium);
}
#[link(name = "user32")]
extern "system" {
    fn ScreenToClient(window: Handle, point: *mut Point) -> i32;
    fn GetKeyState(key: i32) -> i16;
    fn RegisterClipboardFormatW(name: *const u16) -> u32;
}
#[link(name = "shell32")]
extern "system" {
    fn DragQueryFileW(drop: Handle, index: u32, name: *mut u16, length: u32) -> u32;
}
#[link(name = "kernel32")]
extern "system" {
    fn GlobalLock(data: Handle) -> Handle;
    fn GlobalUnlock(data: Handle) -> i32;
    fn GlobalSize(data: Handle) -> usize;
}

thread_local! {
    /// Packets produced by OLE callbacks on this UI thread, in arrival order.
    static PENDING: RefCell<VecDeque<NativeDragDropEvent>> = const { RefCell::new(VecDeque::new()) };
    /// This sequence's reference to the registered target. OLE holds its own
    /// reference until winit's WM_DESTROY calls RevokeDragDrop.
    static REGISTERED: RefCell<Option<FileDropHandler>> = const { RefCell::new(None) };
}

/// Register the drop target on `window`. Called on the UI thread right after
/// window creation, in place of winit's `OleInitialize`/`RegisterDragDrop`.
pub(crate) fn register(window: &winit::window::Window) -> std::io::Result<()> {
    let handle = window
        .window_handle()
        .map_err(|error| std::io::Error::other(format!("drop target window handle: {error}")))?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Err(std::io::Error::other("drop target requires a Win32 window"));
    };
    let hwnd = handle.hwnd.get() as Handle;
    let ole_init_result = unsafe { OleInitialize(ptr::null_mut()) };
    // It is ok if the initialize result is `S_FALSE` because it might happen that
    // multiple windows are created on the same thread.
    if ole_init_result == OLE_E_WRONGCOMPOBJ {
        return Err(std::io::Error::other(
            "OleInitialize failed! Result was: `OLE_E_WRONGCOMPOBJ`",
        ));
    } else if ole_init_result == RPC_E_CHANGED_MODE {
        return Err(std::io::Error::other(
            "OleInitialize failed! Result was: `RPC_E_CHANGED_MODE`. Make sure other crates are \
             not using multithreaded COM library on the same thread.",
        ));
    }
    let file_drop_handler = FileDropHandler::new(hwnd);
    let handler_interface_ptr =
        unsafe { &mut (*file_drop_handler.data).interface as *mut _ as *mut c_void };
    let result = unsafe { RegisterDragDrop(hwnd, handler_interface_ptr) };
    if result != S_OK {
        return Err(std::io::Error::other(format!(
            "RegisterDragDrop failed: {result:#x}"
        )));
    }
    REGISTERED.with(|slot| *slot.borrow_mut() = Some(file_drop_handler));
    Ok(())
}

/// Take the packets queued since the last call, in arrival order.
pub(crate) fn take_events() -> VecDeque<NativeDragDropEvent> {
    PENDING.with(|pending| std::mem::take(&mut *pending.borrow_mut()))
}

pub(crate) fn clipboard_virtual_files() -> std::io::Result<Vec<NativeDragVirtualFileOffer>> {
    virtual_file::clipboard_files()
}

#[repr(C)]
struct FileDropHandlerData {
    interface: IDropTarget,
    refcount: AtomicUsize,
    window: Handle,
    target_drag: Option<NativeDragDropEvent>,
    virtual_session: Option<virtual_file::Session>,
}

impl FileDropHandlerData {
    unsafe fn queue_native(
        &mut self,
        phase: NativeDragDropPhase,
        keys: u32,
        point: Option<&PointL>,
        effect: *mut u32,
    ) {
        let allowed = unsafe { *effect };
        let Some(mut event) = self.target_drag.clone() else {
            unsafe { *effect = 0 };
            return;
        };
        if phase != NativeDragDropPhase::Leave {
            event.source_operations = (allowed & 1) | ((allowed & 4) >> 1) | ((allowed & 2) << 3);
        }
        if let Some(point) = point {
            let mut local = Point {
                x: point.x,
                y: point.y,
            };
            if unsafe { ScreenToClient(self.window, &mut local) } == 0 {
                unsafe { *effect = 0 };
                return;
            }
            event.position = PhysicalPosition::new(local.x as f64, local.y as f64);
            event.screen_position = Some(PhysicalPosition::new(point.x as f64, point.y as f64));
        }
        let alt = unsafe { GetKeyState(VK_MENU) } < 0;
        let meta = unsafe { GetKeyState(VK_LWIN) } < 0 || unsafe { GetKeyState(VK_RWIN) } < 0;
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
        PENDING.with(|pending| pending.borrow_mut().push_back(event));
    }
}

struct FileDropHandler {
    data: *mut FileDropHandlerData,
}

#[allow(non_snake_case)]
impl FileDropHandler {
    fn new(window: Handle) -> FileDropHandler {
        let data = Box::new(FileDropHandlerData {
            interface: IDropTarget {
                vtbl: &DROP_TARGET_VTBL as *const IDropTargetVtbl,
            },
            refcount: AtomicUsize::new(1),
            window,
            target_drag: None,
            virtual_session: None,
        });
        FileDropHandler {
            data: Box::into_raw(data),
        }
    }

    // Implement IUnknown
    unsafe extern "system" fn QueryInterface(
        this: *mut c_void,
        riid: *const Guid,
        object: *mut *mut c_void,
    ) -> HRESULT {
        if object.is_null() {
            return E_POINTER;
        }
        unsafe {
            *object = ptr::null_mut();
        }
        if riid.is_null() {
            return E_NOINTERFACE;
        }
        // IUnknown and IDropTarget share this object's canonical interface.
        let iid = unsafe { &*riid };
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
                *object = this;
                Self::AddRef(this);
            }
            S_OK
        } else {
            E_NOINTERFACE
        }
    }

    unsafe extern "system" fn AddRef(this: *mut c_void) -> u32 {
        let drop_handler_data = unsafe { Self::from_interface(this) };
        let count = drop_handler_data.refcount.fetch_add(1, Ordering::Relaxed) + 1;
        count as u32
    }

    unsafe extern "system" fn Release(this: *mut c_void) -> u32 {
        let drop_handler = unsafe { Self::from_interface(this) };
        let count = drop_handler.refcount.fetch_sub(1, Ordering::Release) - 1;
        if count == 0 {
            std::sync::atomic::fence(Ordering::Acquire);
            // Destroy the underlying data
            drop(unsafe { Box::from_raw(drop_handler as *mut FileDropHandlerData) });
        }
        count as u32
    }

    unsafe extern "system" fn DragEnter(
        this: *mut c_void,
        data: *mut c_void,
        keys: u32,
        point: PointL,
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
            .filter_map(|path| NativeDragFileOffer::open(path).ok())
            .collect();
        let paths: Vec<_> = files.iter().map(|file| file.path.clone()).collect();
        let (virtual_session, virtual_files) = unsafe { virtual_file::Session::new(data) }
            .map_or((None, Vec::new()), |(session, files)| {
                (Some(session), files)
            });
        target.virtual_session = virtual_session;
        let mut strings: Vec<(String, Vec<u16>)> = Vec::new();
        for (format, mime, wide) in [
            (CF_UNICODETEXT, "text/plain", true),
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
        let hovered_is_valid =
            !paths.is_empty() || !virtual_files.is_empty() || !strings.is_empty();
        if !hovered_is_valid {
            unsafe { *effect = 0 };
            return S_OK;
        }
        target.target_drag = Some(NativeDragDropEvent {
            phase: NativeDragDropPhase::Enter,
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
            position: PhysicalPosition::new(0., 0.),
            screen_position: None,
            modifiers: 0,
            source_operations: 0,
            operation: Arc::new(AtomicU32::new(0)),
            observed_at: std::time::Instant::now(),
        });
        unsafe {
            target.queue_native(NativeDragDropPhase::Enter, keys, Some(&point), effect);
        }
        S_OK
    }
    unsafe extern "system" fn DragOver(
        this: *mut c_void,
        keys: u32,
        point: PointL,
        effect: *mut u32,
    ) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        unsafe {
            target.queue_native(NativeDragDropPhase::Over, keys, Some(&point), effect);
        }
        S_OK
    }
    unsafe extern "system" fn DragLeave(this: *mut c_void) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        let mut effect = 0;
        unsafe {
            target.queue_native(NativeDragDropPhase::Leave, 0, None, &mut effect);
        }
        target.target_drag = None;
        target.virtual_session = None;
        S_OK
    }
    unsafe extern "system" fn Drop(
        this: *mut c_void,
        _data: *mut c_void,
        keys: u32,
        point: PointL,
        effect: *mut u32,
    ) -> HRESULT {
        let target = unsafe { Self::from_interface(this) };
        if let Some(session) = target.virtual_session.take() {
            let operation = target.target_drag.as_ref().map_or(0, |event| {
                event.operation.load(Ordering::Acquire) & event.source_operations
            });
            if operation != 0 {
                session.start();
            }
        }
        unsafe {
            target.queue_native(NativeDragDropPhase::Drop, keys, Some(&point), effect);
        }
        target.target_drag = None;
        S_OK
    }
    fn clipboard_format(name: &str) -> u16 {
        let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        unsafe { RegisterClipboardFormatW(name.as_ptr()) as u16 }
    }
    unsafe fn read_global(data: *mut c_void, format: u16) -> Option<Vec<u8>> {
        let format = FormatEtc {
            format,
            target_device: ptr::null_mut(),
            aspect: DVASPECT_CONTENT,
            index: -1,
            tymed: TYMED_HGLOBAL,
        };
        let mut medium: StgMedium = unsafe { std::mem::zeroed() };
        let table = unsafe { *(data as *const *const IDataObjectVtbl) };
        if unsafe { ((*table).get_data)(data, &format, &mut medium) } < 0 {
            return None;
        }
        let memory = medium.handle;
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

    unsafe fn iterate_filenames<F>(data_obj: *mut c_void, callback: F) -> Option<Handle>
    where
        F: Fn(PathBuf),
    {
        let drop_format = FormatEtc {
            format: CF_HDROP,
            target_device: ptr::null_mut(),
            aspect: DVASPECT_CONTENT,
            index: -1,
            tymed: TYMED_HGLOBAL,
        };

        let mut medium: StgMedium = unsafe { std::mem::zeroed() };
        let table = unsafe { *(data_obj as *const *const IDataObjectVtbl) };
        let get_data_result = unsafe { ((*table).get_data)(data_obj, &drop_format, &mut medium) };
        if get_data_result >= 0 {
            let hdrop = medium.handle;

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
                ReleaseStgMedium(&mut medium);
            }
            Some(hdrop)
        } else {
            // DV_E_FORMATETC means the dropped item is not a file; any other
            // failure is unexpected. Neither offers files.
            None
        }
    }
}

impl Drop for FileDropHandler {
    fn drop(&mut self) {
        unsafe {
            FileDropHandler::Release(self.data as *mut c_void);
        }
    }
}

static DROP_TARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    unknown: IUnknownVtbl {
        query_interface: FileDropHandler::QueryInterface,
        add_ref: FileDropHandler::AddRef,
        release: FileDropHandler::Release,
    },
    drag_enter: FileDropHandler::DragEnter,
    drag_over: FileDropHandler::DragOver,
    drag_leave: FileDropHandler::DragLeave,
    drop: FileDropHandler::Drop,
};
