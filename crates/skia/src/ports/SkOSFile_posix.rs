//! Native POSIX file and directory operations; current additions are unverified.
use crate::src::core::SkOSFile::{kRead_SkFILE_Flag, kWrite_SkFILE_Flag, SkFILE_Flags, FILE};
use std::ffi::{c_char, c_int, c_void};

// Source: src/ports/SkOSFile_posix.cpp:27-32
pub unsafe fn sk_fsync(f: *mut FILE) {
    #[cfg(not(any(target_os = "android", target_env = "uclibc", target_env = "newlib")))]
    {
        let fd = libc::fileno(f);
        libc::fsync(fd);
    }
}

// Source: src/ports/SkOSFile_posix.cpp:34-52
pub unsafe fn sk_exists(path: *const c_char, flags: SkFILE_Flags) -> bool {
    let mut mode = libc::F_OK;
    if flags & kRead_SkFILE_Flag != 0 {
        mode |= libc::R_OK;
    }
    if flags & kWrite_SkFILE_Flag != 0 {
        mode |= libc::W_OK;
    }
    // Source: src/ports/SkOSFile_posix.cpp:43-48
    #[cfg(SK_BUILD_FOR_IOS)]
    {
        if libc::access(path, mode) == 0 {
            true
        } else {
            flags == kRead_SkFILE_Flag
                && crate::src::ports::SkOSFile_ios::ios_get_path_in_bundle(path, None)
        }
    }
    // Source: src/ports/SkOSFile_posix.cpp:50-50
    #[cfg(not(SK_BUILD_FOR_IOS))]
    {
        libc::access(path, mode) == 0
    }
}

// Source: src/ports/SkOSFile_posix.cpp:54-57
struct SkFILEID {
    dev: libc::dev_t,
    ino: libc::ino_t,
}

// Source: src/ports/SkOSFile_posix.cpp:59-71
unsafe fn sk_ino(a: *mut FILE, id: &mut SkFILEID) -> bool {
    let fd = libc::fileno(a);
    if fd < 0 {
        return false;
    }
    let mut status: libc::stat = std::mem::zeroed();
    if libc::fstat(fd, &mut status) != 0 {
        return false;
    }
    id.dev = status.st_dev;
    id.ino = status.st_ino;
    true
}

// Source: src/ports/SkOSFile_posix.cpp:73-78
pub unsafe fn sk_fidentical(a: *mut FILE, b: *mut FILE) -> bool {
    // Zero initialization only avoids uninitialized Rust values. Fields are consumed
    // only after both native producer calls have succeeded.
    let mut aID = SkFILEID { dev: 0, ino: 0 };
    let mut bID = SkFILEID { dev: 0, ino: 0 };
    sk_ino(a, &mut aID) && sk_ino(b, &mut bID) && aID.ino == bID.ino && aID.dev == bID.dev
}

// Source: src/ports/SkOSFile_posix.cpp:80-82
pub unsafe fn sk_fmunmap(addr: *const c_void, length: usize) {
    libc::munmap(addr.cast_mut(), length);
}

// Source: src/ports/SkOSFile_posix.cpp:84-104
/// The returned mapping must be unmapped with the returned length.
/// The underlying file bytes must remain readable and immutable while exposed as SkData.
pub unsafe fn sk_fdmmap(fd: c_int, size: &mut usize) -> *mut c_void {
    let mut status: libc::stat = std::mem::zeroed();
    if libc::fstat(fd, &mut status) != 0 {
        return std::ptr::null_mut();
    }
    if status.st_mode & libc::S_IFMT != libc::S_IFREG {
        return std::ptr::null_mut();
    }
    // Native SkTFitsIn<size_t> rejects negative and nonrepresentable file lengths.
    let Ok(fileSize) = usize::try_from(status.st_size) else {
        return std::ptr::null_mut();
    };
    let addr = libc::mmap(
        std::ptr::null_mut(),
        fileSize,
        libc::PROT_READ,
        libc::MAP_PRIVATE,
        fd,
        0,
    );
    if addr == libc::MAP_FAILED {
        return std::ptr::null_mut();
    }
    *size = fileSize;
    addr
}

// Source: src/ports/SkOSFile_posix.cpp:106-108
pub unsafe fn sk_fileno(f: *mut FILE) -> c_int {
    libc::fileno(f)
}

// Source: src/ports/SkOSFile_posix.cpp:110-117
pub unsafe fn sk_fmmap(f: *mut FILE, size: &mut usize) -> *mut c_void {
    let fd = sk_fileno(f);
    if fd < 0 {
        return std::ptr::null_mut();
    }
    sk_fdmmap(fd, size)
}

// Source: src/ports/SkOSFile_posix.cpp:119-129
/// Requires a live FILE and a writable span of `count` bytes.
/// Mirrors the native single pread call, including short reads and EINTR failure.
pub unsafe fn sk_qread(file: *mut FILE, buffer: *mut c_void, count: usize, offset: usize) -> usize {
    let fd = sk_fileno(file);
    if fd < 0 {
        return usize::MAX;
    }
    let bytesRead = libc::pread(fd, buffer, count, offset as libc::off_t);
    if bytesRead < 0 {
        return usize::MAX;
    }
    bytesRead as usize
}

// Source: src/ports/SkOSFile_posix.cpp:133-138
// Actual SkString owners occupy the native inline fSelf storage; no byte-vector
// path/suffix owner or entry pointer escapes the original DIR lifetime.
#[repr(C)]
struct SkOSFileIterData {
    fDIR: *mut libc::DIR,
    fPath: core::mem::ManuallyDrop<crate::src::core::SkString::SkString>,
    fSuffix: core::mem::ManuallyDrop<crate::src::core::SkString::SkString>,
}
// Source: src/ports/SkOSFile_posix.cpp:134-136
impl SkOSFileIterData {
    fn new() -> Self {
        Self {
            fDIR: core::ptr::null_mut(),
            fPath: core::mem::ManuallyDrop::new(crate::src::core::SkString::SkString::new()),
            fSuffix: core::mem::ManuallyDrop::new(crate::src::core::SkString::SkString::new()),
        }
    }
}
const _: () = assert!(core::mem::size_of::<SkOSFileIterData>() <= Iter::kStorageSize);
// Source: src/core/SkOSFile.h:81-99
#[repr(C)]
union IterStorage {
    bytes: core::mem::MaybeUninit<[u8; 40]>,
    pointer_alignment: *mut core::ffi::c_void,
    double_alignment: f64,
}
#[repr(C)]
pub struct Iter {
    fSelf: IterStorage,
}
impl Iter {
    pub const kStorageSize: usize = 40;
    // Native reinterpret_cast projects only the initialized selected-platform payload.
    fn data(&mut self) -> &mut SkOSFileIterData {
        unsafe { &mut *self.fSelf.bytes.as_mut_ptr().cast::<SkOSFileIterData>() }
    }
    // Source: src/ports/SkOSFile_posix.cpp:140-140
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
                .write(SkOSFileIterData::new());
        }
        value
    }
    // Source: src/ports/SkOSFile_posix.cpp:142-145
    /// Safety: nonnull path/suffix are readable NUL-terminated native byte strings.
    pub unsafe fn from_path(path: *const c_char, suffix: *const c_char) -> Self {
        let mut value = Self::new();
        value.reset(path, suffix);
        value
    }
    // Source: src/core/SkOSFile.h:84-85
    pub unsafe fn from_path_default(path: *const c_char) -> Self {
        Self::from_path(path, core::ptr::null())
    }
}
// Source: src/ports/SkOSFile_posix.cpp:147-153
impl Drop for Iter {
    fn drop(&mut self) {
        let state = self.data();
        unsafe {
            if !state.fDIR.is_null() {
                libc::closedir(state.fDIR);
            }
            core::mem::ManuallyDrop::drop(&mut state.fSuffix);
            core::mem::ManuallyDrop::drop(&mut state.fPath);
        }
    }
}
impl Iter {
    // Source: src/ports/SkOSFile_posix.cpp:155-175
    pub unsafe fn reset(&mut self, path: *const c_char, suffix: *const c_char) {
        let state = self.data();
        if !state.fDIR.is_null() {
            libc::closedir(state.fDIR);
            state.fDIR = core::ptr::null_mut();
        }
        state.fPath.set_cstr(path.cast());
        if !path.is_null() {
            state.fDIR = libc::opendir(path);
            // Source: src/ports/SkOSFile_posix.cpp:167-168
            #[cfg(SK_BUILD_FOR_IOS)]
            if state.fDIR.is_null()
                && crate::src::ports::SkOSFile_ios::ios_get_path_in_bundle(
                    path,
                    Some(&mut *state.fPath),
                )
            {
                state.fDIR = libc::opendir(state.fPath.c_str().cast());
            }
            state.fSuffix.set_cstr(suffix.cast());
        } else {
            state.fSuffix.reset();
        }
    }
    // Source: src/core/SkOSFile.h:87-87
    pub unsafe fn reset_default(&mut self, path: *const c_char) {
        self.reset(path, core::ptr::null());
    }
}
// Source: src/ports/SkOSFile_posix.cpp:178-184
unsafe fn issuffixfor(suffix: &crate::src::core::SkString::SkString, value: *const c_char) -> bool {
    let suffixLen = suffix.size();
    let strLen = libc::strlen(value);
    strLen >= suffixLen
        && libc::memcmp(
            suffix.c_str().cast(),
            value.add(strLen - suffixLen).cast(),
            suffixLen,
        ) == 0
}

impl Iter {
    // Source: src/ports/SkOSFile_posix.cpp:186-220
    pub fn next(
        &mut self,
        name: Option<&mut crate::src::core::SkString::SkString>,
        getDir: bool,
    ) -> bool {
        let state = self.data();
        if !state.fDIR.is_null() {
            unsafe {
                let mut entry;
                loop {
                    entry = libc::readdir(state.fDIR);
                    if entry.is_null() {
                        break;
                    }
                    let mut status: libc::stat = core::mem::zeroed();
                    let mut path = (*state.fPath).clone();
                    if !path.endsWith(b"/\0".as_ptr()) && !path.endsWith(b"\\\0".as_ptr()) {
                        path.append_cstr(b"/\0".as_ptr());
                    }
                    path.append_cstr((*entry).d_name.as_ptr().cast());
                    if libc::stat(path.c_str().cast(), &mut status) == 0 {
                        if getDir {
                            if status.st_mode & libc::S_IFDIR != 0 {
                                break;
                            }
                        } else if status.st_mode & libc::S_IFDIR == 0
                            && issuffixfor(&state.fSuffix, (*entry).d_name.as_ptr())
                        {
                            break;
                        }
                    }
                }
                if !entry.is_null() {
                    if let Some(name) = name {
                        name.set_cstr((*entry).d_name.as_ptr().cast());
                    }
                    return true;
                }
            }
        }
        false
    }
    // Source: src/core/SkOSFile.h:92-92
    pub fn next_default(
        &mut self,
        name: Option<&mut crate::src::core::SkString::SkString>,
    ) -> bool {
        self.next(name, false)
    }
}
