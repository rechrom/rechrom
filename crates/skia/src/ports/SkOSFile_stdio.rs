//! Translation of the selected desktop POSIX stdio branch.
//! Calls retain native FILE-position and error semantics; no Rust File owner is substituted.
use crate::src::core::SkOSFile::{kRead_SkFILE_Flag, kWrite_SkFILE_Flag, SkFILE_Flags, FILE};
use std::ffi::{c_char, c_void};

// System ABI spelling only, not another logging implementation.
#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    static mut __stderrp: *mut FILE;
}
#[cfg(not(target_vendor = "apple"))]
unsafe extern "C" {
    static mut stderr: *mut FILE;
}
unsafe fn native_stderr() -> *mut FILE {
    #[cfg(target_vendor = "apple")]
    {
        __stderrp
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        stderr
    }
}

// Source: src/ports/SkOSFile_stdio.cpp:64-97
/// `path` must be a live NUL-terminated native path.
pub unsafe fn sk_fopen(path: *const c_char, flags: SkFILE_Flags) -> *mut FILE {
    let mut perm = [0 as c_char; 4];
    let mut p = 0;
    if flags & kRead_SkFILE_Flag != 0 {
        perm[p] = b'r' as c_char;
        p += 1;
    }
    if flags & kWrite_SkFILE_Flag != 0 {
        perm[p] = b'w' as c_char;
        p += 1;
    }
    perm[p] = b'b' as c_char;
    let file = libc::fopen(path, perm.as_ptr());
    if file.is_null() && flags & kWrite_SkFILE_Flag != 0 {
        #[cfg(all(sk_debug, target_vendor = "apple", target_arch = "aarch64"))]
        {
            let error = *libc::__error();
            crate::SkDebugf!(@source b"src/ports/SkOSFile_stdio.cpp\0".as_ptr(),93;
                b"sk_fopen: fopen(\"%s\", \"%s\") returned nullptr (errno:%d): %s\n\0".as_ptr(),
                path,perm.as_ptr(),error,libc::strerror(error),
            );
        }
    }
    file
}

// Source: src/ports/SkOSFile_stdio.cpp:99-115
/// Requires a live stream with exclusive access to its FILE position.
pub unsafe fn sk_fgetsize(f: *mut FILE) -> usize {
    crate::SkASSERT!(@source b"src/ports/SkOSFile_stdio.cpp\0".as_ptr(),100; @condition b"f\0".as_ptr(); !f.is_null());
    let curr = libc::ftell(f);
    if curr < 0 {
        return 0;
    }
    libc::fseek(f, 0, libc::SEEK_END);
    let mut size = libc::ftell(f);
    if size < 0 {
        size = 0;
    }
    libc::fseek(f, curr, libc::SEEK_SET);
    size as usize
}

// Source: src/ports/SkOSFile_stdio.cpp:117-120
pub unsafe fn sk_fwrite(buffer: *const c_void, byteCount: usize, f: *mut FILE) -> usize {
    crate::SkASSERT!(@source b"src/ports/SkOSFile_stdio.cpp\0".as_ptr(),118; @condition b"f\0".as_ptr(); !f.is_null());
    libc::fwrite(buffer, 1, byteCount, f)
}

// Source: src/ports/SkOSFile_stdio.cpp:122-125
pub unsafe fn sk_fflush(f: *mut FILE) {
    crate::SkASSERT!(@source b"src/ports/SkOSFile_stdio.cpp\0".as_ptr(),123; @condition b"f\0".as_ptr(); !f.is_null());
    libc::fflush(f);
}

// Source: src/ports/SkOSFile_stdio.cpp:127-133
pub unsafe fn sk_ftell(f: *mut FILE) -> usize {
    let curr = libc::ftell(f);
    if curr < 0 {
        return 0;
    }
    curr as usize
}

// Source: src/ports/SkOSFile_stdio.cpp:135-139
/// Consumes the native close obligation; null has the native no-op behavior.
pub unsafe fn sk_fclose(f: *mut FILE) {
    if !f.is_null() {
        libc::fclose(f);
    }
}

// Source: src/ports/SkOSFile_stdio.cpp:141-157
pub unsafe fn sk_isdir(path: *const c_char) -> bool {
    let mut status: libc::stat = std::mem::zeroed();
    if libc::stat(path, &mut status) == 0 {
        return status.st_mode & libc::S_IFDIR != 0;
    }
    false
}

// Source: src/ports/SkOSFile_stdio.cpp:159-180
pub unsafe fn sk_mkdir(path: *const c_char) -> bool {
    if sk_isdir(path) {
        return true;
    }
    // Native default argument is zero (existence only).
    if crate::src::core::SkOSFile::sk_exists(path, 0) {
        libc::fprintf(
            native_stderr(),
            b"sk_mkdir: path '%s' already exists but is not a directory\n\0"
                .as_ptr()
                .cast(),
            path,
        );
        return false;
    }
    let retval = libc::mkdir(path, 0o777);
    if retval != 0 {
        libc::perror(b"mkdir() failed with error: \0".as_ptr().cast());
    }
    retval == 0
}
