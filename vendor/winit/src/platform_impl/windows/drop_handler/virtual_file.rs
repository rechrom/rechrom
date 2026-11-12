//! OLE virtual-file descriptors and apartment-correct retained content streams.
use std::ffi::{c_void, OsString};
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::sync::{mpsc, Arc};
use std::time::{Duration, SystemTime};

use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::{FILETIME, POINTL};
use windows_sys::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, IDataObject,
    Marshal::{CoMarshalInterThreadInterfaceInStream, CoReleaseMarshalData},
    StructuredStorage::CoGetInterfaceAndReleaseStream,
    DVASPECT_CONTENT, FORMATETC, STATSTG, STGMEDIUM, TYMED_HGLOBAL, TYMED_ISTREAM,
};
use windows_sys::Win32::System::Ole::ReleaseStgMedium;

use super::{FileDropHandler, IDataObjectVtbl, IUnknownVtbl};
use crate::event::{NativeDragDataReader, NativeDragFileMetadata, NativeDragVirtualFileOffer};

const IID_DATA_OBJECT: GUID = GUID {
    data1: 0x10e,
    data2: 0,
    data3: 0,
    data4: [0xc0, 0, 0, 0, 0, 0, 0, 0x46],
};

pub(super) fn clipboard_files() -> io::Result<Vec<NativeDragVirtualFileOffer>> {
    #[link(name = "ole32")]
    extern "system" {
        fn OleGetClipboard(object: *mut *mut IDataObject) -> HRESULT;
    }
    // Clipboard IDataObject belongs to a live STA. Keep that apartment pumping
    // until the existing content-reader session releases its final proxy.
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let (reply, receive) = mpsc::channel();
    std::thread::Builder::new()
        .name("clipboard-file-offer".into())
        .spawn(move || unsafe {
            let result = CoInitializeEx(ptr::null(), 2); // APARTMENTTHREADED
            if result < 0 {
                let _ = reply.send(Err(io::Error::other(format!(
                    "clipboard COM apartment: {result:#x}"
                ))));
                return;
            }
            let _apartment = Apartment;
            let mut object = ptr::null_mut();
            let result = OleGetClipboard(&mut object);
            if result < 0 || object.is_null() {
                let _ = reply.send(Err(io::Error::other(format!(
                    "clipboard IDataObject: {result:#x}"
                ))));
                return;
            }
            let admitted = Session::new(object);
            release(object.cast());
            let Some((session, files)) = admitted else {
                let _ = reply.send(Err(io::Error::other(
                    "clipboard virtual-file descriptors unavailable",
                )));
                return;
            };
            let (done, finished) = mpsc::channel::<()>();
            std::thread::spawn(move || {
                session.run();
                drop(done);
            });
            let _ = reply.send(Ok(files));
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, TranslateMessage, MSG,
                PM_REMOVE, QS_ALLINPUT,
            };
            let mut message: MSG = std::mem::zeroed();
            while matches!(finished.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                while PeekMessageW(&mut message, 0, 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                MsgWaitForMultipleObjectsEx(0, ptr::null(), 50, QS_ALLINPUT, 0);
            }
        })?;
    receive.recv_timeout(Duration::from_secs(5)).map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "clipboard virtual-file admission timed out",
        )
    })?
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Descriptor {
    flags: u32,
    class_id: GUID,
    size: [i32; 2],
    point: POINTL,
    attributes: u32,
    created: FILETIME,
    accessed: FILETIME,
    modified: FILETIME,
    size_high: u32,
    size_low: u32,
    name: [u16; 260],
}
#[derive(Debug)]
struct MarshaledObject(usize);
impl Drop for MarshaledObject {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                CoReleaseMarshalData(self.0 as *mut c_void);
                release(self.0 as *mut c_void);
            }
        }
    }
}
#[derive(Debug)]
enum Request {
    Metadata {
        index: usize,
        reply: mpsc::Sender<io::Result<NativeDragFileMetadata>>,
    },
    Read {
        index: usize,
        offset: u64,
        length: usize,
        reply: mpsc::Sender<io::Result<Vec<u8>>>,
    },
}
#[derive(Debug)]
struct Reader {
    index: usize,
    sender: mpsc::Sender<Request>,
}
fn disconnected() -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        "native virtual-file session ended",
    )
}
impl NativeDragDataReader for Reader {
    fn metadata(&self) -> io::Result<NativeDragFileMetadata> {
        let (reply, receive) = mpsc::channel();
        self.sender
            .send(Request::Metadata {
                index: self.index,
                reply,
            })
            .map_err(|_| disconnected())?;
        receive.recv().map_err(|_| disconnected())?
    }
    fn read_range(&self, offset: u64, length: usize) -> io::Result<Vec<u8>> {
        let (reply, receive) = mpsc::channel();
        self.sender
            .send(Request::Read {
                index: self.index,
                offset,
                length,
                reply,
            })
            .map_err(|_| disconnected())?;
        receive.recv().map_err(|_| disconnected())?
    }
}
#[derive(Debug)]
pub(super) struct Session {
    object: MarshaledObject,
    requests: mpsc::Receiver<Request>,
    indices: Vec<i32>,
    format: u16,
}
impl Session {
    pub(super) unsafe fn new(
        data: *const IDataObject,
    ) -> Option<(Self, Vec<NativeDragVirtualFileOffer>)> {
        let descriptor_format = FileDropHandler::clipboard_format("FileGroupDescriptorW");
        let bytes = unsafe { FileDropHandler::read_global(data, descriptor_format) }?;
        if bytes.len() < 4 {
            return None;
        }
        let count = u32::from_ne_bytes(bytes[..4].try_into().ok()?) as usize;
        if count == 0 || count > (bytes.len() - 4) / std::mem::size_of::<Descriptor>() {
            return None;
        }
        let mut stream = ptr::null_mut();
        let result = unsafe {
            CoMarshalInterThreadInterfaceInStream(
                &IID_DATA_OBJECT,
                data as *mut c_void,
                &mut stream,
            )
        };
        if result < 0 || stream.is_null() {
            return None;
        }
        let (sender, requests) = mpsc::channel();
        let mut offers = Vec::new();
        let mut indices = Vec::new();
        for index in 0..count {
            let descriptor = unsafe {
                ptr::read_unaligned(
                    bytes
                        .as_ptr()
                        .add(4 + index * std::mem::size_of::<Descriptor>())
                        .cast::<Descriptor>(),
                )
            };
            if descriptor.flags & 4 != 0 && descriptor.attributes & 0x10 != 0 {
                continue;
            }
            let length = descriptor
                .name
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(260);
            let name = OsString::from_wide(&descriptor.name[..length]);
            let path = std::path::Path::new(&name);
            if path.file_name() != Some(name.as_os_str())
                || name.is_empty()
                || name == "."
                || name == ".."
            {
                continue;
            }
            let metadata = (descriptor.flags & 0x40 != 0).then(|| NativeDragFileMetadata {
                length: (u64::from(descriptor.size_high) << 32) | u64::from(descriptor.size_low),
                modified: (descriptor.flags & 0x20 != 0)
                    .then(|| file_time(descriptor.modified))
                    .flatten(),
            });
            let slot = indices.len();
            indices.push(i32::try_from(index).ok()?);
            offers.push(NativeDragVirtualFileOffer {
                name,
                content_type: None,
                metadata,
                reader: Arc::new(Reader {
                    index: slot,
                    sender: sender.clone(),
                }),
            });
        }
        Some((
            Self {
                object: MarshaledObject(stream as usize),
                requests,
                indices,
                format: FileDropHandler::clipboard_format("FileContents"),
            },
            offers,
        ))
    }
    pub(super) fn start(self) {
        std::thread::spawn(move || unsafe { self.run() });
    }
    unsafe fn run(mut self) {
        let initialized = unsafe { CoInitializeEx(ptr::null(), 0) };
        if initialized < 0 {
            return;
        }
        let mut object = ptr::null_mut();
        let marshal = std::mem::replace(&mut self.object.0, 0) as *mut c_void;
        let result =
            unsafe { CoGetInterfaceAndReleaseStream(marshal, &IID_DATA_OBJECT, &mut object) };
        if result < 0 || object.is_null() {
            unsafe {
                CoUninitialize();
            }
            return;
        }
        let mut contents: Vec<Option<Content>> = (0..self.indices.len()).map(|_| None).collect();
        while let Ok(request) = self.requests.recv() {
            let index = match &request {
                Request::Read { index, .. } | Request::Metadata { index, .. } => *index,
            };
            if contents[index].is_none() {
                contents[index] =
                    Some(unsafe { Content::new(object, self.format, self.indices[index]) });
            }
            let content = contents[index].as_mut().unwrap();
            match request {
                Request::Metadata { reply, .. } => {
                    let _ = reply.send(unsafe { content.metadata() });
                }
                Request::Read {
                    offset,
                    length,
                    reply,
                    ..
                } => {
                    let _ = reply.send(unsafe { content.read(offset, length) });
                }
            }
        }
        // Release every STGMEDIUM/IStream on the apartment where it was obtained.
        drop(contents);
        unsafe {
            release(object);
            CoUninitialize();
        }
    }
}

struct Content {
    medium: Option<STGMEDIUM>,
    failure: Option<String>,
}
impl Content {
    unsafe fn new(object: *mut c_void, format: u16, index: i32) -> Self {
        let request = FORMATETC {
            cfFormat: format,
            ptd: ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: index,
            tymed: (TYMED_ISTREAM | TYMED_HGLOBAL) as u32,
        };
        let mut medium = unsafe { std::mem::zeroed() };
        let table = unsafe { *(object as *const *const IDataObjectVtbl) };
        let result = unsafe { ((*table).GetData)(object.cast(), &request, &mut medium) };
        if result < 0 {
            return Self {
                medium: None,
                failure: Some(format!("native FileContents GetData failed: {result:#x}")),
            };
        }
        if medium.tymed != TYMED_ISTREAM as u32 && medium.tymed != TYMED_HGLOBAL as u32 {
            unsafe {
                ReleaseStgMedium(&mut medium);
            }
            return Self {
                medium: None,
                failure: Some("native FileContents returned an unsupported storage medium".into()),
            };
        }
        Self {
            medium: Some(medium),
            failure: None,
        }
    }
    fn medium(&self) -> io::Result<&STGMEDIUM> {
        self.medium
            .as_ref()
            .ok_or_else(|| io::Error::other(self.failure.clone().unwrap_or_default()))
    }
    unsafe fn metadata(&self) -> io::Result<NativeDragFileMetadata> {
        let medium = self.medium()?;
        if medium.tymed == TYMED_ISTREAM as u32 {
            let stream = unsafe { medium.u.pstm };
            let table = unsafe { *(stream as *const *const StreamTable) };
            let mut stat: STATSTG = unsafe { std::mem::zeroed() };
            let result = unsafe { ((*table).stat)(stream, &mut stat, 1) };
            check(result)?;
            Ok(NativeDragFileMetadata {
                length: stat.cbSize,
                modified: file_time(stat.mtime),
            })
        } else {
            let length =
                unsafe { windows_sys::Win32::System::Memory::GlobalSize(medium.u.hGlobal) };
            Ok(NativeDragFileMetadata {
                length: length as u64,
                modified: None,
            })
        }
    }
    unsafe fn read(&mut self, offset: u64, length: usize) -> io::Result<Vec<u8>> {
        let medium = self.medium()?;
        if medium.tymed == TYMED_ISTREAM as u32 {
            let stream = unsafe { medium.u.pstm };
            let table = unsafe { *(stream as *const *const StreamTable) };
            let position = i64::try_from(offset).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "native stream offset overflow")
            })?;
            check(unsafe { ((*table).seek)(stream, position, 0, ptr::null_mut()) })?;
            let mut result = vec![0; length];
            let mut consumed = 0;
            while consumed < length {
                let request = (length - consumed).min(u32::MAX as usize) as u32;
                let mut count = 0;
                check(unsafe {
                    ((*table).read)(
                        stream,
                        result[consumed..].as_mut_ptr().cast(),
                        request,
                        &mut count,
                    )
                })?;
                if count > request {
                    return Err(io::Error::other(
                        "native stream returned an invalid byte count",
                    ));
                }
                if count == 0 {
                    break;
                }
                consumed += count as usize;
            }
            result.truncate(consumed);
            Ok(result)
        } else {
            use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
            let memory = unsafe { medium.u.hGlobal };
            let size = unsafe { GlobalSize(memory) };
            let start = usize::try_from(offset).unwrap_or(usize::MAX).min(size);
            let count = length.min(size - start);
            if count == 0 {
                return Ok(Vec::new());
            }
            let pointer = unsafe { GlobalLock(memory) };
            if pointer.is_null() {
                return Err(io::Error::last_os_error());
            }
            let bytes =
                unsafe { std::slice::from_raw_parts(pointer.cast::<u8>().add(start), count) }
                    .to_vec();
            unsafe {
                GlobalUnlock(memory);
            }
            Ok(bytes)
        }
    }
}
impl Drop for Content {
    fn drop(&mut self) {
        if let Some(medium) = &mut self.medium {
            unsafe {
                ReleaseStgMedium(medium);
            }
        }
    }
}
#[repr(C)]
struct StreamTable {
    unknown: IUnknownVtbl,
    read: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, *mut u32) -> HRESULT,
    write: usize,
    seek: unsafe extern "system" fn(*mut c_void, i64, u32, *mut u64) -> HRESULT,
    set_size: usize,
    copy_to: usize,
    commit: usize,
    revert: usize,
    lock_region: usize,
    unlock_region: usize,
    stat: unsafe extern "system" fn(*mut c_void, *mut STATSTG, u32) -> HRESULT,
    clone: usize,
}
unsafe fn release(object: *mut c_void) {
    let table = unsafe { *(object as *const *const IUnknownVtbl) };
    unsafe {
        ((*table).Release)(object.cast());
    }
}
fn check(result: HRESULT) -> io::Result<()> {
    if result < 0 {
        Err(io::Error::other(format!(
            "native content stream failed: {result:#x}"
        )))
    } else {
        Ok(())
    }
}
fn file_time(time: FILETIME) -> Option<SystemTime> {
    let ticks = (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
    let epoch = 116_444_736_000_000_000u64;
    if ticks >= epoch {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_nanos((ticks - epoch).checked_mul(100)?))
    } else {
        SystemTime::UNIX_EPOCH.checked_sub(Duration::from_nanos((epoch - ticks).checked_mul(100)?))
    }
}
