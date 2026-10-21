#[cfg(pa_dcheck_is_configurable)]
pub use super::log_message::LOGGING_DCHECK;
pub use super::log_message::{LOGGING_FATAL, LOGGING_INFO};
use core::ffi::{c_char, c_int};
#[cfg(not(pa_dcheck_is_configurable))]
pub const LOGGING_DCHECK: c_int = LOGGING_FATAL;
// Source: partition_alloc_base/logging.cc:42
static mut g_min_log_level: c_int = 0;
// Source: partition_alloc_base/logging.cc:46-59. Native rv is int even when write returns ssize_t.
#[cfg(not(pa_is_win))]
unsafe fn WriteToStderr(data: *const c_char, length: usize) {
    let mut bytes_written = 0;
    while bytes_written < length {
        let rv = super::posix::eintr_wrapper::WrapEINTR(|| {
            libc::write(
                libc::STDERR_FILENO,
                data.add(bytes_written).cast(),
                length - bytes_written,
            )
        })() as c_int;
        if rv < 0 {
            break;
        }
        bytes_written += rv as usize;
    }
}
// Source: partition_alloc_base/logging.cc:82-102
pub unsafe fn SetMinLogLevel(level: c_int) {
    g_min_log_level = LOGGING_FATAL.min(level);
}
pub unsafe fn GetMinLogLevel() -> c_int {
    g_min_log_level
}
pub unsafe fn ShouldCreateLogMessage(severity: c_int) -> bool {
    if severity < g_min_log_level {
        return false;
    }
    true
}
pub unsafe fn GetVlogVerbosity() -> c_int {
    (-1).max(LOGGING_INFO - GetMinLogLevel())
}
// Source: partition_alloc_base/logging.cc:104-121
#[cfg(not(pa_is_win))]
pub unsafe fn RawLog(level: c_int, message: *const c_char) {
    if level >= g_min_log_level && !message.is_null() {
        let length = libc::strlen(message);
        WriteToStderr(message, length);
        if length > 0 && *message.add(length - 1) != b'\n' as c_char {
            WriteToStderr(b"\n".as_ptr().cast(), 1);
        }
    }
}
// Windows WriteToStderr/GetStdHandle/WriteFile and macro-only header families remain pending exact ABI.
