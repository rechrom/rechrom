use super::super::scoped_clear_last_error::errno_location;
use core::ffi::{c_char, c_int};
// Original header overload resolution is supplied by actual libc/compiler inputs, never host inference.
#[cfg(pa_strerror_r_abi = "gnu")]
unsafe extern "C" {
    #[cfg_attr(pa_strerror_r_symbol = "strerror_r", link_name = "strerror_r")]
    #[cfg_attr(
        pa_strerror_r_symbol = "__gnu_strerror_r",
        link_name = "__gnu_strerror_r"
    )]
    fn native_strerror_r(err: c_int, buf: *mut c_char, len: usize) -> *mut c_char;
}
#[cfg(pa_strerror_r_abi = "posix")]
unsafe extern "C" {
    #[cfg_attr(pa_strerror_r_symbol = "strerror_r", link_name = "strerror_r")]
    #[cfg_attr(
        pa_strerror_r_symbol = "__xpg_strerror_r",
        link_name = "__xpg_strerror_r"
    )]
    fn native_strerror_r(err: c_int, buf: *mut c_char, len: usize) -> c_int;
}
#[cfg(not(any(pa_strerror_r_abi = "gnu", pa_strerror_r_abi = "posix")))]
compile_error!("actual native strerror_r overload ABI input required");
#[cfg(not(any(
    pa_strerror_r_symbol = "strerror_r",
    pa_strerror_r_symbol = "__xpg_strerror_r",
    pa_strerror_r_symbol = "__gnu_strerror_r"
)))]
compile_error!("actual native strerror_r selected linkage symbol required");
// Source: safe_strerror.cc: actual GNU overload.
unsafe fn wrap_gnu(
    f: unsafe extern "C" fn(c_int, *mut c_char, usize) -> *mut c_char,
    err: c_int,
    buf: *mut c_char,
    len: usize,
) {
    let rc = f(err, buf, len);
    if rc != buf {
        *buf = 0;
        libc::strncat(buf, rc, len - 1);
    }
}
// Source: safe_strerror.cc: actual POSIX overload and original snprintf error path.
unsafe fn wrap_posix(
    f: unsafe extern "C" fn(c_int, *mut c_char, usize) -> c_int,
    err: c_int,
    buf: *mut c_char,
    len: usize,
) {
    let old_errno = *errno_location();
    let result = f(err, buf, len);
    if result == 0 {
        *buf.add(len - 1) = 0;
    } else {
        let new_errno = *errno_location();
        let error = if new_errno != old_errno {
            new_errno
        } else {
            result
        };
        libc::snprintf(
            buf,
            len,
            b"Error %d while retrieving error %d\0".as_ptr().cast(),
            error,
            err,
        );
    }
    *errno_location() = old_errno;
}
pub unsafe fn safe_strerror_r(err: c_int, buf: *mut c_char, len: usize) {
    if buf.is_null() || len == 0 {
        return;
    }
    #[cfg(pa_strerror_r_abi = "gnu")]
    wrap_gnu(native_strerror_r, err, buf, len);
    #[cfg(pa_strerror_r_abi = "posix")]
    wrap_posix(native_strerror_r, err, buf, len);
}
// Native std::string safe_strerror(int) is not needed by PA LogMessage and requires the real stdlib owner binding.
