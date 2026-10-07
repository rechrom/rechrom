//! quickjs-libc.c Windows branch, QuickJS 2026-06-04 (Bellard/Gordon, MIT).
//! Win32 and MinGW-w64 CRT calls are tool-only. This module is independently
//! metadata-checkable on the host; it is not evidence of Windows execution.
pub use super::quickjs_libc::{
    create_json_module, get_bool_option, js_loadScript, js_module_check_attributes,
    js_module_loader, js_module_set_import_meta, js_std_exit, js_std_gc, js_std_loadFile,
};
use super::quickjs_libc_windows_state::*;
use super::quickjs_libc_windows_worker::*;
use quickjs::{list::*, quickjs::*, quickjs_header::*};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::{
    ffi::{c_char, c_void},
    ptr,
};
pub type HANDLE = *mut c_void;
#[repr(C)]
pub struct COORD {
    pub X: i16,
    pub Y: i16,
}
#[repr(C)]
pub struct SMALL_RECT {
    pub Left: i16,
    pub Top: i16,
    pub Right: i16,
    pub Bottom: i16,
}
#[repr(C)]
pub struct CONSOLE_SCREEN_BUFFER_INFO {
    pub dwSize: COORD,
    pub dwCursorPosition: COORD,
    pub wAttributes: u16,
    pub srWindow: SMALL_RECT,
    pub dwMaximumWindowSize: COORD,
}
#[cfg_attr(windows, link(name = "kernel32"))]
extern "system" {
    pub fn CreateEventA(
        attributes: *mut c_void,
        manual_reset: i32,
        initial_state: i32,
        name: *const c_char,
    ) -> HANDLE;
    pub fn SetEvent(handle: HANDLE) -> i32;
    pub fn ResetEvent(handle: HANDLE) -> i32;
    pub fn CloseHandle(handle: HANDLE) -> i32;
    pub fn WaitForMultipleObjects(
        count: u32,
        handles: *const HANDLE,
        wait_all: i32,
        timeout: u32,
    ) -> u32;
    pub fn Sleep(milliseconds: u32);
    pub fn GetConsoleScreenBufferInfo(handle: HANDLE, info: *mut CONSOLE_SCREEN_BUFFER_INFO)
        -> i32;
    pub fn SetConsoleMode(handle: HANDLE, mode: u32) -> i32;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn GetProcessAffinityMask(
        process: HANDLE,
        process_mask: *mut usize,
        system_mask: *mut usize,
    ) -> i32;
}
/// Original C's default MinGW-w64 configuration: off_t=32, time_t=64.
/// Avoid libc::stat, which binds _stat64 and silently widens st_size.
#[repr(C)]
pub struct WinStat {
    pub st_dev: u32,
    pub st_ino: u16,
    pub st_mode: u16,
    pub st_nlink: i16,
    pub st_uid: i16,
    pub st_gid: i16,
    pub st_rdev: u32,
    pub st_size: i32,
    pub st_atime: i64,
    pub st_mtime: i64,
    pub st_ctime: i64,
}
#[repr(C)]
pub struct WinUtime {
    pub actime: i64,
    pub modtime: i64,
}
#[repr(C)]
pub struct WinTimeval {
    pub tv_sec: i32,
    pub tv_usec: i32,
}
/// Exact Winpthreads ABI (mingw-w64 pthread.h/sched.h): pthread_mutex_t
/// intptr_t, pthread_t uintptr_t, sched_param contains one int. These are
/// declarations of native host APIs, not a C engine or pthread implementation.
pub mod crt {
    use super::*;
    pub enum FILE {}
    pub enum DIR {}
    #[repr(C)]
    pub struct dirent {
        pub d_ino: i32,
        pub d_reclen: u16,
        pub d_namlen: u16,
        pub d_name: [c_char; 260],
    }
    pub type sighandler_t = usize;
    pub const SIG_DFL: usize = 0;
    pub const SIG_IGN: usize = 1;
    pub const SIGABRT: i32 = 22;
    pub const SIGFPE: i32 = 8;
    pub const SIGILL: i32 = 4;
    pub const SIGSEGV: i32 = 11;
    pub const SIGTERM: i32 = 15;
    pub const O_RDONLY: i32 = 0;
    pub const O_WRONLY: i32 = 1;
    pub const O_RDWR: i32 = 2;
    pub const O_APPEND: i32 = 8;
    pub const O_CREAT: i32 = 256;
    pub const O_EXCL: i32 = 1024;
    pub const O_TRUNC: i32 = 512;
    pub const S_IFMT: i32 = 0xf000;
    pub const S_IFIFO: i32 = 0x1000;
    pub const S_IFCHR: i32 = 0x2000;
    pub const S_IFDIR: i32 = 0x4000;
    pub const S_IFBLK: i32 = 0x6000;
    pub const S_IFREG: i32 = 0x8000;
    pub type c_longlong = i64;
    pub type pthread_mutex_t = isize;
    pub type pthread_t = usize;
    pub type pthread_cond_t = isize;
    pub type clock_t = i32;
    pub type time_t = i64;
    pub type c_long = i32;
    pub const CLOCK_REALTIME: i32 = 0;
    pub const CLOCK_MONOTONIC: i32 = 1;
    #[repr(C)]
    pub struct timespec {
        pub tv_sec: time_t,
        pub tv_nsec: c_long,
    }
    #[repr(C)]
    pub struct pthread_attr_t {
        pub p_state: u32,
        pub stack: *mut c_void,
        pub s_size: usize,
        pub sched_priority: i32,
    }
    pub const PTHREAD_CREATE_DETACHED: i32 = 0x04;
    pub const O_TEXT: i32 = 0x4000;
    pub const O_BINARY: i32 = 0x8000;
    pub const SIGINT: i32 = 2;
    pub const ENOENT: i32 = 2;
    pub const EPERM: i32 = 1;
    pub const EIO: i32 = 5;
    pub const EISDIR: i32 = 21;
    pub const EBADF: i32 = 9;
    pub const EACCES: i32 = 13;
    pub const EBUSY: i32 = 16;
    pub const EEXIST: i32 = 17;
    pub const EINVAL: i32 = 22;
    pub const ENOSPC: i32 = 28;
    pub const EPIPE: i32 = 32;
    pub const ENOSYS: i32 = 40;
    pub const SEEK_SET: i32 = 0;
    pub const SEEK_CUR: i32 = 1;
    pub const SEEK_END: i32 = 2;
    extern "C" {
        pub fn _errno() -> *mut i32;
        pub fn getenv(name: *const c_char) -> *mut c_char;
        pub fn __p__environ() -> *mut *mut *mut c_char;
        pub fn _putenv(value: *const c_char) -> i32;
        pub fn _fullpath(buffer: *mut c_char, path: *const c_char, capacity: usize) -> *mut c_char;
        #[link_name = "_close"]
        pub fn close(fd: i32) -> i32;
        pub fn _lseek(fd: i32, offset: i32, whence: i32) -> i32;
        pub fn _read(fd: i32, buf: *mut c_void, count: u32) -> i32;
        pub fn _write(fd: i32, buf: *const c_void, count: u32) -> i32;
        #[link_name = "_isatty"]
        pub fn isatty(fd: i32) -> i32;
        pub fn rename(old: *const c_char, new: *const c_char) -> i32;
        pub fn signal(sig: i32, handler: usize) -> usize;
        pub fn _getcwd(buf: *mut c_char, capacity: i32) -> *mut c_char;
        #[link_name = "_chdir"]
        pub fn chdir(path: *const c_char) -> i32;
        pub fn opendir(path: *const c_char) -> *mut DIR;
        pub fn readdir(dir: *mut DIR) -> *mut dirent;
        pub fn closedir(dir: *mut DIR) -> i32;
        pub fn _get_osfhandle(fd: i32) -> isize;
        pub fn _setmode(fd: i32, mode: i32) -> i32;
        pub fn _open(path: *const c_char, flags: i32, ...) -> i32;
        pub fn _mkdir(path: *const c_char) -> i32;
        pub fn _rmdir(path: *const c_char) -> i32;
        pub fn _unlink(path: *const c_char) -> i32;
        pub fn _stat64i32(path: *const c_char, out: *mut WinStat) -> i32;
        pub fn _utime64(path: *const c_char, times: *const WinUtime) -> i32;
        pub fn gettimeofday(tv: *mut WinTimeval, timezone: *mut c_void) -> i32;
        pub fn malloc(size: usize) -> *mut c_void;
        pub fn realloc(pointer: *mut c_void, size: usize) -> *mut c_void;
        pub fn _msize(pointer: *mut c_void) -> usize;
        pub fn calloc(count: usize, size: usize) -> *mut c_void;
        pub fn free(p: *mut c_void);
        #[link_name = "_strdup"]
        pub fn strdup(s: *const c_char) -> *mut c_char;
        pub fn strlen(s: *const c_char) -> usize;
        pub fn strcmp(a: *const c_char, b: *const c_char) -> i32;
        pub fn strspn(a: *const c_char, b: *const c_char) -> usize;
        pub fn atoi(s: *const c_char) -> i32;
        pub fn strerror(error: i32) -> *mut c_char;
        pub fn strtol(input: *const c_char, end: *mut *mut c_char, base: i32) -> c_long;
        pub fn perror(message: *const c_char);
        pub fn __acrt_iob_func(index: u32) -> *mut FILE;
        // Original Makefile requests __USE_MINGW_ANSI_STDIO.
        #[link_name = "__mingw_snprintf"]
        pub fn snprintf(out: *mut c_char, capacity: usize, format: *const c_char, ...) -> i32;
        pub fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE;
        #[link_name = "_fdopen"]
        pub fn fdopen(fd: i32, mode: *const c_char) -> *mut FILE;
        #[link_name = "_popen"]
        pub fn popen(command: *const c_char, mode: *const c_char) -> *mut FILE;
        #[link_name = "_pclose"]
        pub fn pclose(file: *mut FILE) -> i32;
        pub fn tmpfile() -> *mut FILE;
        pub fn fclose(file: *mut FILE) -> i32;
        pub fn fflush(file: *mut FILE) -> i32;
        pub fn fwrite(data: *const c_void, size: usize, count: usize, file: *mut FILE) -> usize;
        pub fn fread(data: *mut c_void, size: usize, count: usize, file: *mut FILE) -> usize;
        pub fn fgetc(file: *mut FILE) -> i32;
        pub fn fputc(ch: i32, file: *mut FILE) -> i32;
        pub fn feof(file: *mut FILE) -> i32;
        pub fn ferror(file: *mut FILE) -> i32;
        pub fn clearerr(file: *mut FILE);
        #[link_name = "_fileno"]
        pub fn fileno(file: *mut FILE) -> i32;
        pub fn ftello(file: *mut FILE) -> i32;
        pub fn fseeko(file: *mut FILE, offset: i32, whence: i32) -> i32;
    }
    pub unsafe fn lseek(fd: i32, offset: i64, whence: i32) -> i64 {
        _lseek(fd, offset as i32, whence) as i64
    }
    pub unsafe fn read(fd: i32, buf: *mut c_void, count: usize) -> isize {
        _read(fd, buf, count as u32) as isize
    }
    pub unsafe fn write(fd: i32, buf: *const c_void, count: usize) -> isize {
        _write(fd, buf, count as u32) as isize
    }
    pub unsafe fn getcwd(buf: *mut c_char, capacity: usize) -> *mut c_char {
        _getcwd(buf, capacity as i32)
    }
    // Retain the independently translated common FILE bodies' i64 locals,
    // narrowing at the original Windows off_t parameter/return boundary.
    pub unsafe fn ftell(file: *mut FILE) -> i64 {
        ftello(file) as i64
    }
    pub unsafe fn fseek(file: *mut FILE, pos: i64, whence: i32) -> i32 {
        fseeko(file, pos as i32, whence)
    }
    #[cfg_attr(windows, link(name = "winpthread"))]
    extern "C" {
        pub fn clock_gettime(clock: i32, result: *mut timespec) -> i32;
        pub fn usleep(microseconds: u32) -> i32;
        pub fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *const u32) -> i32;
        pub fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> i32;
        pub fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> i32;
        pub fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> i32;
        pub fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const u32) -> i32;
        pub fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> i32;
        pub fn pthread_cond_signal(cond: *mut pthread_cond_t) -> i32;
        pub fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> i32;
        pub fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> i32;
        pub fn pthread_cond_timedwait(
            cond: *mut pthread_cond_t,
            mutex: *mut pthread_mutex_t,
            timeout: *const timespec,
        ) -> i32;
        pub fn pthread_join(thread: pthread_t, result: *mut *mut c_void) -> i32;
        pub fn pthread_attr_init(attr: *mut pthread_attr_t) -> i32;
        pub fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> i32;
        pub fn pthread_attr_setdetachstate(attr: *mut pthread_attr_t, state: i32) -> i32;
        pub fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, size: usize) -> i32;
        pub fn pthread_create(
            tid: *mut pthread_t,
            attr: *const pthread_attr_t,
            start: extern "C" fn(*mut c_void) -> *mut c_void,
            arg: *mut c_void,
        ) -> i32;
    }
}
// quickjs-libc.c:395..438. Keep CRT byte paths and Windows' 32-bit long
// boundary. Null-context buffers use the same Rust allocator/free adapter as
// the Unix host; nonnull buffers retain the engine's public allocator API.
pub unsafe fn js_load_file(
    ctx: *mut JSContext,
    pbuf_len: *mut usize,
    filename: *const c_char,
) -> *mut u8 {
    let file = crt::fopen(filename, c"rb".as_ptr());
    if file.is_null() {
        return ptr::null_mut();
    }
    let mut output_length = 0;
    let result = (|| {
        if crt::fseek(file, 0, crt::SEEK_END) < 0 {
            return ptr::null_mut();
        }
        let length = crt::ftell(file);
        if length < 0 {
            return ptr::null_mut();
        }
        if length == i32::MAX as i64 {
            *crt::_errno() = crt::EISDIR;
            return ptr::null_mut();
        }
        let length = length as usize;
        if crt::fseek(file, 0, crt::SEEK_SET) < 0 {
            return ptr::null_mut();
        }
        let buffer = if ctx.is_null() {
            std::alloc::alloc(std::alloc::Layout::array::<u8>(length + 1).unwrap())
        } else {
            js_malloc(ctx, length + 1).cast::<u8>()
        };
        if buffer.is_null() {
            return buffer;
        }
        if crt::fread(buffer.cast(), 1, length, file) != length {
            *crt::_errno() = crt::EIO;
            super::quickjs_libc::js_free_file_buffer(ctx, buffer, length);
            return ptr::null_mut();
        }
        *buffer.add(length) = 0;
        output_length = length;
        buffer
    })();
    crt::fclose(file);
    if !result.is_null() {
        *pbuf_len = output_length;
    }
    result
}

// Original setenv ignores overwrite and the _putenv result; keep both choices.
pub unsafe fn setenv(name: *const c_char, value: *const c_char, _overwrite: i32) {
    let n = crt::strlen(name);
    let v = crt::strlen(value);
    let s = crt::malloc(n + v + 2).cast::<c_char>();
    ptr::copy_nonoverlapping(name, s, n);
    *s.add(n) = b'=' as c_char;
    ptr::copy_nonoverlapping(value, s.add(n + 1), v);
    *s.add(n + v + 1) = 0;
    crt::_putenv(s);
    crt::free(s.cast());
}
pub unsafe fn unsetenv(name: *const c_char) {
    setenv(name, c"".as_ptr(), 1);
}
pub unsafe fn realpath(path: *const c_char, buf: *mut c_char) -> *mut c_char {
    if crt::_fullpath(buf, path, 4096).is_null() {
        *crt::_errno() = crt::ENOENT;
        ptr::null_mut()
    } else {
        buf
    }
}
pub unsafe fn js_module_loader_so(ctx: *mut JSContext, _name: *const c_char) -> *mut JSModuleDef {
    JS_ThrowReferenceError(
        ctx,
        c"shared library modules are not supported yet".as_ptr(),
    );
    ptr::null_mut()
}
pub unsafe fn get_time_ms() -> i64 {
    let mut tv: WinTimeval = std::mem::zeroed();
    crt::gettimeofday(&mut tv, ptr::null_mut());
    (tv.tv_sec as i64)
        .wrapping_mul(1000)
        .wrapping_add((tv.tv_usec / 1000) as i64)
}
pub unsafe fn get_time_ns() -> i64 {
    let mut tv: WinTimeval = std::mem::zeroed();
    crt::gettimeofday(&mut tv, ptr::null_mut());
    (tv.tv_sec as i64)
        .wrapping_mul(1_000_000_000)
        .wrapping_add((tv.tv_usec as i64) * 1000)
}
pub unsafe fn js_os_open(
    ctx: *mut JSContext,
    _: JSValue,
    argc: i32,
    argv: *mut JSValue,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    let mut flags = 0;
    let mut mode = 0o666;
    if JS_ToInt32(ctx, &mut flags, *argv.add(1)) != 0
        || (argc >= 3
            && JS_IsUndefined(*argv.add(2)) == 0
            && JS_ToInt32(ctx, &mut mode, *argv.add(2)) != 0)
    {
        JS_FreeCString(ctx, name);
        return JS_EXCEPTION;
    }
    if flags & crt::O_TEXT == 0 {
        flags |= crt::O_BINARY;
    }
    let ret = js_get_errno(crt::_open(name, flags, mode) as isize);
    JS_FreeCString(ctx, name);
    JS_NewInt32(ctx, ret as i32)
}
pub unsafe fn js_os_ttyGetWinSize(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    let mut fd = 0;
    if JS_ToInt32(ctx, &mut fd, *argv) != 0 {
        return JS_EXCEPTION;
    }
    let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
    if GetConsoleScreenBufferInfo(crt::_get_osfhandle(fd) as HANDLE, &mut info) == 0 {
        return JS_NULL;
    }
    let obj = JS_NewArray(ctx);
    if JS_IsException(obj) != 0 {
        return obj;
    }
    JS_DefinePropertyValueUint32(
        ctx,
        obj,
        0,
        JS_NewInt32(ctx, info.dwSize.X as i32),
        JS_PROP_C_W_E,
    );
    JS_DefinePropertyValueUint32(
        ctx,
        obj,
        1,
        JS_NewInt32(ctx, info.dwSize.Y as i32),
        JS_PROP_C_W_E,
    );
    obj
}
pub unsafe fn js_os_ttySetRaw(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    let mut fd = 0;
    if JS_ToInt32(ctx, &mut fd, *argv) != 0 {
        return JS_EXCEPTION;
    }
    SetConsoleMode(crt::_get_osfhandle(fd) as HANDLE, 0x0008 | 0x0200);
    crt::_setmode(fd, crt::O_BINARY);
    if fd == 0 {
        SetConsoleMode(crt::_get_osfhandle(1) as HANDLE, 0x0001 | 0x0002 | 0x0004);
    }
    JS_UNDEFINED
}
pub unsafe fn js_os_remove(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    let mut st: WinStat = std::mem::zeroed();
    let result = if crt::_stat64i32(name, &mut st) == 0 && st.st_mode & 0xf000 == 0x4000 {
        crt::_rmdir(name)
    } else {
        crt::_unlink(name)
    };
    let ret = js_get_errno(result as isize);
    JS_FreeCString(ctx, name);
    JS_NewInt32(ctx, ret as i32)
}
pub unsafe fn js_os_mkdir(
    ctx: *mut JSContext,
    _: JSValue,
    argc: i32,
    argv: *mut JSValue,
) -> JSValue {
    let mut mode = 0o777;
    if argc >= 2 && JS_ToInt32(ctx, &mut mode, *argv.add(1)) != 0 {
        return JS_EXCEPTION;
    }
    let path = JS_ToCString(ctx, *argv);
    if path.is_null() {
        return JS_EXCEPTION;
    }
    let ret = js_get_errno(crt::_mkdir(path) as isize);
    JS_FreeCString(ctx, path);
    JS_NewInt32(ctx, ret as i32)
}
unsafe fn make_obj_error(ctx: *mut JSContext, obj: JSValue, err: i32) -> JSValue {
    if JS_IsException(obj) != 0 {
        return obj;
    }
    let result = JS_NewArray(ctx);
    if JS_IsException(result) != 0 {
        return JS_EXCEPTION;
    }
    JS_DefinePropertyValueUint32(ctx, result, 0, obj, JS_PROP_C_W_E);
    JS_DefinePropertyValueUint32(ctx, result, 1, JS_NewInt32(ctx, err), JS_PROP_C_W_E);
    result
}
pub unsafe fn js_os_stat(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
    _is_lstat: i32,
) -> JSValue {
    let path = JS_ToCString(ctx, *argv);
    if path.is_null() {
        return JS_EXCEPTION;
    }
    let mut st: WinStat = std::mem::zeroed();
    let ret = crt::_stat64i32(path, &mut st);
    let err = if ret < 0 { *crt::_errno() } else { 0 };
    JS_FreeCString(ctx, path);
    let obj = if ret < 0 {
        JS_NULL
    } else {
        let obj = JS_NewObject(ctx);
        if JS_IsException(obj) != 0 {
            return obj;
        }
        for (name, v) in [
            (c"dev", st.st_dev as i64),
            (c"ino", st.st_ino as i64),
            (c"mode", st.st_mode as i64),
            (c"nlink", st.st_nlink as i64),
            (c"uid", st.st_uid as i64),
            (c"gid", st.st_gid as i64),
            (c"rdev", st.st_rdev as i64),
            (c"size", st.st_size as i64),
            (c"atime", st.st_atime.wrapping_mul(1000)),
            (c"mtime", st.st_mtime.wrapping_mul(1000)),
            (c"ctime", st.st_ctime.wrapping_mul(1000)),
        ] {
            JS_DefinePropertyValueStr(ctx, obj, name.as_ptr(), JS_NewInt64(ctx, v), JS_PROP_C_W_E);
        }
        obj
    };
    make_obj_error(ctx, obj, err)
}
pub unsafe fn js_os_utimes(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    let (mut a, mut m) = (0, 0);
    if JS_ToInt64(ctx, &mut a, *argv.add(1)) != 0 || JS_ToInt64(ctx, &mut m, *argv.add(2)) != 0 {
        return JS_EXCEPTION;
    }
    let path = JS_ToCString(ctx, *argv);
    if path.is_null() {
        return JS_EXCEPTION;
    }
    let times = WinUtime {
        actime: a / 1000,
        modtime: m / 1000,
    };
    let ret = js_get_errno(crt::_utime64(path, &times) as isize);
    JS_FreeCString(ctx, path);
    JS_NewInt32(ctx, ret as i32)
}
pub unsafe fn js_os_sleep(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    let mut delay = 0;
    if JS_ToInt64(ctx, &mut delay, *argv) != 0 {
        return JS_EXCEPTION;
    }
    Sleep(delay.clamp(0, i32::MAX as i64) as u32);
    JS_NewInt32(ctx, 0)
}
// C:2282..2302: manual-reset event; queue code only signals empty->nonempty
// and only resets after popping the last queued message.
pub unsafe fn js_waker_init(w: *mut JSWaker) -> i32 {
    (*w).handle = CreateEventA(ptr::null_mut(), 1, 0, ptr::null());
    if (*w).handle.is_null() {
        -1
    } else {
        0
    }
}
pub unsafe fn js_waker_signal(w: *mut JSWaker) {
    SetEvent((*w).handle);
}
pub unsafe fn js_waker_clear(w: *mut JSWaker) {
    ResetEvent((*w).handle);
}
pub unsafe fn js_waker_close(w: *mut JSWaker) {
    CloseHandle((*w).handle);
    (*w).handle = (-1isize) as HANDLE;
}
pub static OS_PENDING_SIGNALS: AtomicU64 = AtomicU64::new(0);
static OS_POLL_ENABLED: AtomicBool = AtomicBool::new(false);
pub fn js_os_enable_poll() {
    OS_POLL_ENABLED.store(true, Ordering::Relaxed);
}
pub unsafe fn interrupt_handler(_: *mut JSRuntime, _: *mut c_void) -> i32 {
    ((OS_PENDING_SIGNALS.load(Ordering::Relaxed) >> crt::SIGINT) & 1) as i32
}
pub unsafe fn free_rw_handler(rt: *mut JSRuntime, p: *mut JSOSRWHandler) {
    list_del(&mut (*p).link);
    JS_FreeValueRT(rt, (*p).rw_func[0]);
    JS_FreeValueRT(rt, (*p).rw_func[1]);
    js_free_rt(rt, p.cast());
}
pub unsafe fn free_sh(rt: *mut JSRuntime, p: *mut JSOSSignalHandler) {
    list_del(&mut (*p).link);
    JS_FreeValueRT(rt, (*p).func);
    js_free_rt(rt, p.cast());
}
pub unsafe fn free_timer(rt: *mut JSRuntime, p: *mut JSOSTimer) {
    list_del(&mut (*p).link);
    JS_FreeValueRT(rt, (*p).func);
    js_free_rt(rt, p.cast());
}
pub unsafe fn call_handler(ctx: *mut JSContext, func: JSValue) {
    let f = JS_DupValue(ctx, func);
    let val = JS_Call(ctx, f, JS_UNDEFINED, 0, ptr::null_mut());
    JS_FreeValue(ctx, f);
    if JS_IsException(val) != 0 {
        js_std_dump_error(ctx);
    }
    JS_FreeValue(ctx, val);
}
// C:2422..2513, retain original stdin-priority dispatch (not a generalized
// event selector) and absence of Windows signal processing.
pub unsafe fn js_os_poll(ctx: *mut JSContext) -> i32 {
    let rt = JS_GetRuntime(ctx);
    let ts = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    if list_empty(&mut (*ts).os_rw_handlers) != 0
        && list_empty(&mut (*ts).os_timers) != 0
        && list_empty(&mut (*ts).port_list) != 0
    {
        return -1;
    }
    let mut min_delay = -1;
    if list_empty(&mut (*ts).os_timers) == 0 {
        let now = get_time_ms();
        min_delay = 10000;
        for el in ListIter::new(&mut (*ts).os_timers, false, false) {
            let th = el.cast::<JSOSTimer>();
            let delay = (*th).timeout.wrapping_sub(now);
            if delay <= 0 {
                let f = (*th).func;
                (*th).func = JS_UNDEFINED;
                free_timer(rt, th);
                call_handler(ctx, f);
                JS_FreeValue(ctx, f);
                return 0;
            } else if delay < (min_delay as i64) {
                min_delay = delay as i32;
            }
        }
    }
    let mut handles = [ptr::null_mut(); 64];
    let mut count = 0usize;
    for el in ListIter::new(&mut (*ts).os_rw_handlers, false, false) {
        let rh = el.cast::<JSOSRWHandler>();
        if (*rh).fd == 0 && JS_IsNull((*rh).rw_func[0]) == 0 {
            handles[count] = crt::_get_osfhandle((*rh).fd) as HANDLE;
            count += 1;
            if count == handles.len() {
                break;
            }
        }
    }
    for el in ListIter::new(&mut (*ts).port_list, false, false) {
        let port = el.cast::<JSWorkerMessageHandler>();
        if JS_IsNull((*port).on_message_func) != 0 {
            continue;
        }
        handles[count] = (*(*port).recv_pipe).waker.handle;
        count += 1;
        if count == handles.len() {
            break;
        }
    }
    if count > 0 {
        let timeout = if min_delay == -1 {
            u32::MAX
        } else {
            min_delay as u32
        };
        let ret = WaitForMultipleObjects(count as u32, handles.as_ptr(), 0, timeout);
        if ret < count as u32 {
            for el in ListIter::new(&mut (*ts).os_rw_handlers, false, false) {
                let rh = el.cast::<JSOSRWHandler>();
                if (*rh).fd == 0 && JS_IsNull((*rh).rw_func[0]) == 0 {
                    call_handler(ctx, (*rh).rw_func[0]);
                    return 0;
                }
            }
            for el in ListIter::new(&mut (*ts).port_list, false, false) {
                let port = el.cast::<JSWorkerMessageHandler>();
                if JS_IsNull((*port).on_message_func) == 0
                    && (*(*port).recv_pipe).waker.handle == handles[ret as usize]
                    && handle_posted_message(rt, ctx, port) != 0
                {
                    return 0;
                }
            }
        }
    } else {
        Sleep(min_delay as u32);
    }
    0
}
include!("quickjs_libc_windows_loop.rs");
include!("quickjs_libc_windows_stdio.rs");

// C:750..851: common environment JS entry points must use the same narrow CRT
// environment as the Windows setenv/unsetenv helpers (not Rust wide env APIs).
pub unsafe fn js_std_getenv(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    let value = crt::getenv(name);
    JS_FreeCString(ctx, name);
    if value.is_null() {
        JS_UNDEFINED
    } else {
        JS_NewString(ctx, value)
    }
}
pub unsafe fn js_std_setenv(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    let value = JS_ToCString(ctx, *argv.add(1));
    if value.is_null() {
        JS_FreeCString(ctx, name);
        return JS_EXCEPTION;
    }
    setenv(name, value, 1);
    JS_FreeCString(ctx, name);
    JS_FreeCString(ctx, value);
    JS_UNDEFINED
}
pub unsafe fn js_std_unsetenv(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    unsetenv(name);
    JS_FreeCString(ctx, name);
    JS_UNDEFINED
}
pub unsafe fn js_std_getenviron(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    _: *mut JSValue,
) -> JSValue {
    let obj = JS_NewObject(ctx);
    if JS_IsException(obj) != 0 {
        return JS_EXCEPTION;
    }
    let mut envp = *crt::__p__environ();
    while !(*envp).is_null() {
        let name = *envp;
        let bytes = std::ffi::CStr::from_ptr(name).to_bytes();
        envp = envp.add(1);
        let Some(n) = bytes.iter().position(|&b| b == b'=') else {
            continue;
        };
        let atom = JS_NewAtomLen(ctx, name, n);
        if atom == 0 {
            JS_FreeValue(ctx, obj);
            return JS_EXCEPTION;
        }
        let result = JS_DefinePropertyValue(
            ctx,
            obj,
            atom,
            JS_NewString(ctx, name.add(n + 1)),
            JS_PROP_C_W_E,
        );
        JS_FreeAtom(ctx, atom);
        if result < 0 {
            JS_FreeValue(ctx, obj);
            return JS_EXCEPTION;
        }
    }
    obj
}

// MinGW-w64 limits.h PATH_MAX, in the original Makefile's default Windows
// configuration (_POSIX_ is not defined).
pub const WIN_PATH_MAX: usize = 260;
unsafe extern "C" fn os_signal_handler(sig: i32) {
    OS_PENDING_SIGNALS.fetch_or(1u64 << sig, Ordering::Relaxed);
}
pub unsafe fn js_os_now(ctx: *mut JSContext, _: JSValue, _: i32, _: *mut JSValue) -> JSValue {
    JS_NewFloat64(ctx, get_time_ns() as f64 / 1e6)
}
include!("quickjs_libc_windows_os_common.rs");
include!("quickjs_libc_windows_os_tables.rs");
unsafe fn js_os_init(ctx: *mut JSContext, m: *mut JSModuleDef) -> i32 {
    js_os_enable_poll();
    js_worker_init(ctx, m);
    JS_SetModuleExportList(ctx, m, js_os_funcs.as_ptr(), js_os_funcs.len() as i32)
}
pub unsafe fn js_init_module_os(ctx: *mut JSContext, name: *const c_char) -> *mut JSModuleDef {
    let m = JS_NewCModule(ctx, name, Some(js_os_init));
    if m.is_null() {
        return m;
    }
    JS_AddModuleExportList(ctx, m, js_os_funcs.as_ptr(), js_os_funcs.len() as i32);
    JS_AddModuleExport(ctx, m, c"Worker".as_ptr());
    m
}

// C4212..4225/4063..4131: FILE printing and Worker helpers use this Windows
// state/stream graph, never the Unix production context's opaque structure.
pub unsafe fn js_std_dump_error1(ctx: *mut JSContext, error: JSValue) {
    JS_PrintValue(
        ctx,
        Some(stdio_print_value_write),
        stdio_stderr().cast(),
        error,
        ptr::null(),
    );
    crt::fputc(b'\n' as i32, stdio_stderr());
}
pub unsafe fn js_std_dump_error(ctx: *mut JSContext) {
    let error = JS_GetException(ctx);
    js_std_dump_error1(ctx, error);
    JS_FreeValue(ctx, error);
}
pub unsafe fn js_print(ctx: *mut JSContext, _: JSValue, argc: i32, argv: *mut JSValue) -> JSValue {
    for i in 0..argc {
        if i != 0 {
            crt::fputc(b' ' as i32, stdio_stdout());
        }
        let value = *argv.add(i as usize);
        if JS_IsString(value) != 0 {
            let mut len = 0;
            let text = JS_ToCStringLen(ctx, &mut len, value);
            if text.is_null() {
                return JS_EXCEPTION;
            }
            crt::fwrite(text.cast(), 1, len, stdio_stdout());
            JS_FreeCString(ctx, text);
        } else {
            JS_PrintValue(
                ctx,
                Some(stdio_print_value_write),
                stdio_stdout().cast(),
                value,
                ptr::null(),
            );
        }
    }
    crt::fputc(b'\n' as i32, stdio_stdout());
    JS_UNDEFINED
}
pub unsafe fn js_console_log(
    ctx: *mut JSContext,
    this: JSValue,
    argc: i32,
    argv: *mut JSValue,
) -> JSValue {
    let value = js_print(ctx, this, argc, argv);
    crt::fflush(stdio_stdout());
    value
}
pub unsafe fn js_std_add_helpers(ctx: *mut JSContext, argc: i32, argv: *mut *mut c_char) {
    let global = JS_GetGlobalObject(ctx);
    let console = JS_NewObject(ctx);
    JS_SetPropertyStr(
        ctx,
        console,
        c"log".as_ptr(),
        JS_NewCFunction(ctx, Some(js_console_log), c"log".as_ptr(), 1),
    );
    JS_SetPropertyStr(ctx, global, c"console".as_ptr(), console);
    let performance = JS_NewObject(ctx);
    JS_SetPropertyStr(
        ctx,
        performance,
        c"now".as_ptr(),
        JS_NewCFunction(ctx, Some(js_os_now), c"now".as_ptr(), 0),
    );
    JS_SetPropertyStr(ctx, global, c"performance".as_ptr(), performance);
    if argc >= 0 {
        let args = JS_NewArray(ctx);
        for i in 0..argc {
            JS_SetPropertyUint32(
                ctx,
                args,
                i as u32,
                JS_NewString(ctx, *argv.add(i as usize)),
            );
        }
        JS_SetPropertyStr(ctx, global, c"scriptArgs".as_ptr(), args);
    }
    JS_SetPropertyStr(
        ctx,
        global,
        c"print".as_ptr(),
        JS_NewCFunction(ctx, Some(js_print), c"print".as_ptr(), 1),
    );
    JS_SetPropertyStr(
        ctx,
        global,
        c"__loadScript".as_ptr(),
        JS_NewCFunction(ctx, Some(js_loadScript), c"__loadScript".as_ptr(), 1),
    );
    JS_FreeValue(ctx, global);
}
