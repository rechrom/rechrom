//! Translation of src/ports/SkOSFile_win.cpp (Skia cacf77bdba7b, SK_BUILD_FOR_WIN):
//! Win32 file identity, memory mapping, positional reads and directory
//! iteration over CRT FILEs.
use crate::src::core::SkContainers::{sk_free, sk_malloc_throw};
use crate::src::core::SkOSFile::{kRead_SkFILE_Flag, kWrite_SkFILE_Flag, SkFILE_Flags, FILE};
use std::ffi::{c_char, c_int, c_void};

type HANDLE = *mut c_void;
type BOOL = i32;
type DWORD = u32;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const PAGE_READONLY: DWORD = 0x02;
const FILE_MAP_READ: DWORD = 0x04;
const ERROR_HANDLE_EOF: DWORD = 38;
const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x10;
const MAX_PATH: usize = 260;

#[repr(C)]
struct FILETIME {
    dwLowDateTime: DWORD,
    dwHighDateTime: DWORD,
}

#[repr(C)]
struct BY_HANDLE_FILE_INFORMATION {
    dwFileAttributes: DWORD,
    ftCreationTime: FILETIME,
    ftLastAccessTime: FILETIME,
    ftLastWriteTime: FILETIME,
    dwVolumeSerialNumber: DWORD,
    nFileSizeHigh: DWORD,
    nFileSizeLow: DWORD,
    nNumberOfLinks: DWORD,
    nFileIndexHigh: DWORD,
    nFileIndexLow: DWORD,
}

#[repr(C)]
struct OVERLAPPED {
    Internal: usize,
    InternalHigh: usize,
    Offset: DWORD,
    OffsetHigh: DWORD,
    hEvent: HANDLE,
}

#[repr(C)]
struct WIN32_FIND_DATAW {
    dwFileAttributes: DWORD,
    ftCreationTime: FILETIME,
    ftLastAccessTime: FILETIME,
    ftLastWriteTime: FILETIME,
    nFileSizeHigh: DWORD,
    nFileSizeLow: DWORD,
    dwReserved0: DWORD,
    dwReserved1: DWORD,
    cFileName: [u16; MAX_PATH],
    cAlternateFileName: [u16; 14],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetFileInformationByHandle(file: HANDLE, info: *mut BY_HANDLE_FILE_INFORMATION) -> BOOL;
    fn GetFileSizeEx(file: HANDLE, size: *mut i64) -> BOOL;
    fn CreateFileMappingW(
        file: HANDLE,
        attributes: *mut c_void,
        protect: DWORD,
        maximum_size_high: DWORD,
        maximum_size_low: DWORD,
        name: *const u16,
    ) -> HANDLE;
    fn MapViewOfFile(
        mapping: HANDLE,
        access: DWORD,
        offset_high: DWORD,
        offset_low: DWORD,
        bytes: usize,
    ) -> *mut c_void;
    fn UnmapViewOfFile(address: *const c_void) -> BOOL;
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn ReadFile(
        file: HANDLE,
        buffer: *mut c_void,
        bytes: DWORD,
        read: *mut DWORD,
        overlapped: *mut OVERLAPPED,
    ) -> BOOL;
    fn GetLastError() -> DWORD;
    fn FindFirstFileW(path: *const u16, data: *mut WIN32_FIND_DATAW) -> HANDLE;
    fn FindNextFileW(find: HANDLE, data: *mut WIN32_FIND_DATAW) -> BOOL;
    fn FindClose(find: HANDLE) -> BOOL;
}

unsafe extern "C" {
    fn _fileno(file: *mut FILE) -> c_int;
    fn _get_osfhandle(fd: c_int) -> isize;
    fn _access(path: *const c_char, mode: c_int) -> c_int;
    fn _commit(fd: c_int) -> c_int;
}

// Source: src/ports/SkOSFile_win.cpp:23-25
pub unsafe fn sk_fsync(f: *mut FILE) {
    _commit(sk_fileno(f));
}

// Source: src/ports/SkOSFile_win.cpp:27-36
pub unsafe fn sk_exists(path: *const c_char, flags: SkFILE_Flags) -> bool {
    let mut mode = 0; // existence
    if flags & kRead_SkFILE_Flag != 0 {
        mode |= 4; // read
    }
    if flags & kWrite_SkFILE_Flag != 0 {
        mode |= 2; // write
    }
    _access(path, mode) == 0
}

// Source: src/ports/SkOSFile_win.cpp:38-42
struct SkFILEID {
    fVolume: u64,
    fLsbSize: u64,
    fMsbSize: u64,
}

// Source: src/ports/SkOSFile_win.cpp:44-65
unsafe fn sk_ino(f: *mut FILE, id: &mut SkFILEID) -> bool {
    let fileno = _fileno(f);
    if fileno < 0 {
        return false;
    }
    let file = _get_osfhandle(fileno) as HANDLE;
    if file == INVALID_HANDLE_VALUE {
        return false;
    }
    //TODO: call GetFileInformationByHandleEx on Vista and later with FileIdInfo.
    let mut info: BY_HANDLE_FILE_INFORMATION = std::mem::zeroed();
    if GetFileInformationByHandle(file, &mut info) == 0 {
        return false;
    }
    id.fVolume = u64::from(info.dwVolumeSerialNumber);
    id.fLsbSize = u64::from(info.nFileIndexLow) + (u64::from(info.nFileIndexHigh) << 32);
    id.fMsbSize = 0;
    true
}

// Source: src/ports/SkOSFile_win.cpp:67-73
pub unsafe fn sk_fidentical(a: *mut FILE, b: *mut FILE) -> bool {
    let mut a_id = SkFILEID { fVolume: 0, fLsbSize: 0, fMsbSize: 0 };
    let mut b_id = SkFILEID { fVolume: 0, fLsbSize: 0, fMsbSize: 0 };
    sk_ino(a, &mut a_id)
        && sk_ino(b, &mut b_id)
        && a_id.fLsbSize == b_id.fLsbSize
        && a_id.fMsbSize == b_id.fMsbSize
        && a_id.fVolume == b_id.fVolume
}

// Source: src/ports/SkOSFile_win.cpp:75-84 (SkAutoNullKernelHandle / SkAutoWinMMap).
// The destructor closes unconditionally, as the source's.
struct SkAutoWinMMap(HANDLE);
impl SkAutoWinMMap {
    fn isValid(&self) -> bool {
        !self.0.is_null()
    }
}
impl Drop for SkAutoWinMMap {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

// Source: src/ports/SkOSFile_win.cpp:86-88
pub unsafe fn sk_fmunmap(addr: *const c_void, _length: usize) {
    UnmapViewOfFile(addr);
}

// Source: src/ports/SkOSFile_win.cpp:90-121
pub unsafe fn sk_fdmmap(fileno: c_int, length: &mut usize) -> *mut c_void {
    let file = _get_osfhandle(fileno) as HANDLE;
    if file == INVALID_HANDLE_VALUE {
        return std::ptr::null_mut();
    }
    let mut file_size = 0i64;
    if GetFileSizeEx(file, &mut file_size) == 0 {
        //TODO: use SK_TRACEHR(GetLastError(), "Could not get file size.") to report.
        return std::ptr::null_mut();
    }
    let Ok(size) = usize::try_from(file_size) else {
        return std::ptr::null_mut();
    };
    let mmap = SkAutoWinMMap(CreateFileMappingW(
        file,
        std::ptr::null_mut(),
        PAGE_READONLY,
        0,
        0,
        std::ptr::null(),
    ));
    if !mmap.isValid() {
        //TODO: use SK_TRACEHR(GetLastError(), "Could not create file mapping.") to report.
        return std::ptr::null_mut();
    }
    // Eventually call UnmapViewOfFile
    let addr = MapViewOfFile(mmap.0, FILE_MAP_READ, 0, 0, 0);
    if addr.is_null() {
        //TODO: use SK_TRACEHR(GetLastError(), "Could not map view of file.") to report.
        return std::ptr::null_mut();
    }
    *length = size;
    addr
}

// Source: src/ports/SkOSFile_win.cpp:123-125
pub unsafe fn sk_fileno(f: *mut FILE) -> c_int {
    _fileno(f)
}

// Source: src/ports/SkOSFile_win.cpp:127-134
pub unsafe fn sk_fmmap(f: *mut FILE, length: &mut usize) -> *mut c_void {
    let fileno = sk_fileno(f);
    if fileno < 0 {
        return std::ptr::null_mut();
    }
    sk_fdmmap(fileno, length)
}

// Source: src/ports/SkOSFile_win.cpp:135-161
pub unsafe fn sk_qread(file: *mut FILE, buffer: *mut c_void, count: usize, offset: usize) -> usize {
    let fileno = sk_fileno(file);
    let file_handle = _get_osfhandle(fileno) as HANDLE;
    // The source compares the FILE* (not fileHandle) with INVALID_HANDLE_VALUE.
    if file.cast::<c_void>() == INVALID_HANDLE_VALUE {
        return usize::MAX;
    }
    let mut overlapped: OVERLAPPED = std::mem::zeroed();
    let win_offset = offset as u64;
    overlapped.Offset = win_offset as DWORD;
    overlapped.OffsetHigh = (win_offset >> 32) as DWORD;
    let count = DWORD::try_from(count).unwrap_or(DWORD::MAX);
    let mut bytes_read: DWORD = 0;
    if ReadFile(file_handle, buffer, count, &mut bytes_read, &mut overlapped) != 0 {
        return bytes_read as usize;
    }
    if GetLastError() == ERROR_HANDLE_EOF {
        return 0;
    }
    usize::MAX
}

// Source: src/ports/SkOSFile_win.cpp:165-170
#[repr(C)]
struct SkOSFileIterData {
    fHandle: HANDLE,
    fPath16: *mut u16,
}
const _: () = assert!(core::mem::size_of::<SkOSFileIterData>() <= Iter::kStorageSize);

// Source: src/core/SkOSFile.h:81-99
#[repr(C)]
union IterStorage {
    bytes: core::mem::MaybeUninit<[u8; 40]>,
    pointer_alignment: *mut c_void,
    double_alignment: f64,
}
#[repr(C)]
pub struct Iter {
    fSelf: IterStorage,
}

// Source: src/ports/SkOSFile_win.cpp:172-195
unsafe fn concat_to_16(src: *const c_char, suffix: *const c_char) -> *mut u16 {
    let len = libc::strlen(src);
    let len2 = 3 + if suffix.is_null() { 0 } else { libc::strlen(suffix) };
    let dst = sk_malloc_throw((len + len2) * std::mem::size_of::<u16>()).cast::<u16>();
    let mut i = 0;
    while i < len {
        *dst.add(i) = *src.add(i) as u8 as u16;
        i += 1;
    }
    if i > 0 && *dst.add(i - 1) != u16::from(b'/') {
        *dst.add(i) = u16::from(b'/');
        i += 1;
    }
    *dst.add(i) = u16::from(b'*');
    i += 1;
    if !suffix.is_null() {
        let mut suffix = suffix;
        while *suffix != 0 {
            *dst.add(i) = *suffix as u8 as u16;
            i += 1;
            suffix = suffix.add(1);
        }
    }
    *dst.add(i) = 0;
    crate::SkASSERT!(@source b"src/ports/SkOSFile_win.cpp\0".as_ptr(),192; @condition b"i + 1 <= len + len2\0".as_ptr(); i + 1 <= len + len2);
    dst
}

impl Iter {
    pub const kStorageSize: usize = 40;
    fn data(&mut self) -> &mut SkOSFileIterData {
        unsafe { &mut *self.fSelf.bytes.as_mut_ptr().cast::<SkOSFileIterData>() }
    }
    // Source: src/ports/SkOSFile_win.cpp:166,197
    pub fn new() -> Self {
        let mut value = Self {
            fSelf: IterStorage {
                bytes: core::mem::MaybeUninit::uninit(),
            },
        };
        unsafe {
            value
                .fSelf
                .bytes
                .as_mut_ptr()
                .cast::<SkOSFileIterData>()
                .write(SkOSFileIterData {
                    fHandle: std::ptr::null_mut(),
                    fPath16: std::ptr::null_mut(),
                });
        }
        value
    }
    // Source: src/ports/SkOSFile_win.cpp:199-202
    pub unsafe fn from_path(path: *const c_char, suffix: *const c_char) -> Self {
        let mut value = Self::new();
        value.reset(path, suffix);
        value
    }
    // Source: src/core/SkOSFile.h:84-85
    pub unsafe fn from_path_default(path: *const c_char) -> Self {
        Self::from_path(path, core::ptr::null())
    }

    // Source: src/ports/SkOSFile_win.cpp:213-225
    pub unsafe fn reset(&mut self, path: *const c_char, suffix: *const c_char) {
        let state = self.data();
        if !state.fHandle.is_null() {
            FindClose(state.fHandle);
            state.fHandle = std::ptr::null_mut();
        }
        let path = if path.is_null() { b"\0".as_ptr().cast() } else { path };
        sk_free(state.fPath16.cast());
        state.fPath16 = concat_to_16(path, suffix);
    }
    // Source: src/core/SkOSFile.h:87-87
    pub unsafe fn reset_default(&mut self, path: *const c_char) {
        self.reset(path, core::ptr::null());
    }
}

// Source: src/ports/SkOSFile_win.cpp:227-230
unsafe fn is_magic_dir(dir: *const u16) -> bool {
    // return true for "." and ".."
    *dir == u16::from(b'.')
        && (*dir.add(1) == 0 || (*dir.add(1) == u16::from(b'.') && *dir.add(2) == 0))
}

// Source: src/ports/SkOSFile_win.cpp:232-266
unsafe fn get_the_file(
    handle: HANDLE,
    name: Option<&mut crate::src::core::SkString::SkString>,
    mut dataPtr: *mut WIN32_FIND_DATAW,
    getDir: bool,
) -> bool {
    let mut data: WIN32_FIND_DATAW = std::mem::zeroed();
    if dataPtr.is_null() {
        if FindNextFileW(handle, &mut data) != 0 {
            dataPtr = &mut data;
        } else {
            return false;
        }
    }
    loop {
        if getDir {
            if (*dataPtr).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0
                && !is_magic_dir((*dataPtr).cFileName.as_ptr())
            {
                break;
            }
        } else if (*dataPtr).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
            break;
        }
        if FindNextFileW(handle, dataPtr) == 0 {
            return false;
        }
    }
    // if we get here, we've found a file/dir
    if let Some(name) = name {
        let utf16name = (*dataPtr).cFileName.as_ptr();
        let mut ptr = utf16name;
        while *ptr != 0 {
            ptr = ptr.add(1);
        }
        *name = crate::src::core::SkStringUtils::SkStringFromUTF16(
            utf16name,
            ptr.offset_from(utf16name) as usize,
        );
    }
    true
}

impl Iter {
    // Source: src/ports/SkOSFile_win.cpp:268-284
    pub fn next(
        &mut self,
        name: Option<&mut crate::src::core::SkString::SkString>,
        getDir: bool,
    ) -> bool {
        let state = self.data();
        unsafe {
            let mut data: WIN32_FIND_DATAW = std::mem::zeroed();
            let mut dataPtr: *mut WIN32_FIND_DATAW = std::ptr::null_mut();
            if state.fHandle.is_null() {
                // our first time
                if state.fPath16.is_null() || *state.fPath16 == 0 {
                    // check for no path
                    return false;
                }
                state.fHandle = FindFirstFileW(state.fPath16, &mut data);
                if !state.fHandle.is_null() && state.fHandle != INVALID_HANDLE_VALUE {
                    dataPtr = &mut data;
                }
            }
            state.fHandle != INVALID_HANDLE_VALUE && get_the_file(state.fHandle, name, dataPtr, getDir)
        }
    }
    // Source: src/core/SkOSFile.h:92-92
    pub fn next_default(&mut self, name: Option<&mut crate::src::core::SkString::SkString>) -> bool {
        self.next(name, false)
    }
}

// Source: src/ports/SkOSFile_win.cpp:204-211
impl Drop for Iter {
    fn drop(&mut self) {
        let state = self.data();
        unsafe {
            sk_free(state.fPath16.cast());
            if !state.fHandle.is_null() {
                FindClose(state.fHandle);
            }
        }
    }
}
