use super::immediate_crash::PA_IMMEDIATE_CRASH;
use super::{
    scoped_clear_last_error::ScopedClearLastError, strings::cstring_builder::CStringBuilder,
};
use core::{
    ffi::{c_char, c_int},
    mem::MaybeUninit,
    pin::Pin,
    ptr,
};
pub type LogSeverity = c_int;
pub const LOGGING_VERBOSE: LogSeverity = -1;
pub const LOGGING_INFO: LogSeverity = 0;
pub const LOGGING_WARNING: LogSeverity = 1;
pub const LOGGING_ERROR: LogSeverity = 2;
pub const LOGGING_FATAL: LogSeverity = 3;
pub const LOGGING_NUM_SEVERITIES: LogSeverity = 4;
#[cfg(pa_dchecks_are_on)]
pub const LOGGING_DFATAL: LogSeverity = LOGGING_FATAL;
#[cfg(not(pa_dchecks_are_on))]
pub const LOGGING_DFATAL: LogSeverity = LOGGING_ERROR;
#[cfg(pa_dcheck_is_configurable)]
pub static mut LOGGING_DCHECK: LogSeverity = LOGGING_ERROR;
pub type LogMessageHandlerFunction = Option<
    unsafe extern "C-unwind" fn(LogSeverity, *const c_char, c_int, usize, *const c_char) -> bool,
>;
static mut g_log_message_handler: LogMessageHandlerFunction = None;
pub static mut g_swallow_stream: *mut CStringBuilder = ptr::null_mut();
pub unsafe fn SetLogMessageHandler(handler: LogMessageHandlerFunction) {
    g_log_message_handler = handler;
}
pub unsafe fn GetLogMessageHandler() -> LogMessageHandlerFunction {
    g_log_message_handler
}
// Source: partition_alloc_base/log_message.h:68-105. No owner/virtual dispatch surrogate.
pub struct LogMessage {
    severity_: LogSeverity,
    stream_: CStringBuilder,
    message_start_: usize,
    file_: *const c_char,
    line_: c_int,
    last_error_: ScopedClearLastError,
}
impl LogMessage {
    // Native same-object construction; caller retains original pinned storage.
    pub unsafe fn init_at(out: *mut Self, file: *const c_char, line: c_int, severity: LogSeverity) {
        core::ptr::addr_of_mut!((*out).severity_).write(severity);
        CStringBuilder::init_at(core::ptr::addr_of_mut!((*out).stream_));
        core::ptr::addr_of_mut!((*out).file_).write(file);
        core::ptr::addr_of_mut!((*out).line_).write(line);
        ScopedClearLastError::init_at(core::ptr::addr_of_mut!((*out).last_error_));
        Pin::new_unchecked(&mut *out).Init(file, line);
    }
    pub unsafe fn init_condition_at(
        out: *mut Self,
        file: *const c_char,
        line: c_int,
        condition: *const c_char,
    ) {
        Self::init_at(out, file, line, LOGGING_FATAL);
        let mut this = Pin::new_unchecked(&mut *out);
        this.as_mut()
            .stream()
            .put_cstr(b"Check failed: \0".as_ptr().cast());
        this.as_mut().stream().put_cstr(condition);
        this.as_mut().stream().put_cstr(b". \0".as_ptr().cast());
    }
    pub fn stream(self: Pin<&mut Self>) -> Pin<&mut CStringBuilder> {
        unsafe { self.map_unchecked_mut(|s| &mut s.stream_) }
    }
    pub fn severity(&self) -> LogSeverity {
        self.severity_
    }
    pub unsafe fn c_str(self: Pin<&mut Self>) -> *const c_char {
        self.stream().c_str()
    }
    // Source: partition_alloc_base/log_message.cc:135-156
    unsafe fn Init(mut self: Pin<&mut Self>, file: *const c_char, line: c_int) {
        let last = super::strings::string_util::FindLastOf(file, b"\\/\0".as_ptr().cast());
        let filename = if last.is_null() { file } else { last.add(1) };
        self.as_mut().stream().put_char(b'[' as c_char);
        let severity = self.severity_;
        if severity >= 0 {
            self.as_mut().stream().put_cstr(log_severity_name(severity));
        } else {
            self.as_mut()
                .stream()
                .put_cstr(b"VERBOSE\0".as_ptr().cast());
            self.as_mut().stream().put_int(-severity);
        }
        self.as_mut().stream().put_cstr(b":\0".as_ptr().cast());
        self.as_mut().stream().put_cstr(filename);
        self.as_mut().stream().put_char(b'(' as c_char);
        self.as_mut().stream().put_int(line);
        self.as_mut().stream().put_cstr(b")] \0".as_ptr().cast());
        let len = libc::strlen(self.as_mut().stream().c_str());
        self.get_unchecked_mut().message_start_ = len;
    }
}
unsafe fn log_severity_name(severity: c_int) -> *const c_char {
    if severity >= 0 && severity < LOGGING_NUM_SEVERITIES {
        [
            b"INFO\0".as_ptr(),
            b"WARNING\0".as_ptr(),
            b"ERROR\0".as_ptr(),
            b"FATAL\0".as_ptr(),
        ][severity as usize]
            .cast()
    } else {
        b"UNKNOWN\0".as_ptr().cast()
    }
}
impl Drop for LogMessage {
    // Source: partition_alloc_base/log_message.cc:99-132
    fn drop(&mut self) {
        unsafe {
            let mut stream = Pin::new_unchecked(&mut self.stream_);
            stream.as_mut().put_char(b'\n' as c_char);
            let text = stream.as_mut().c_str();
            if let Some(handler) = g_log_message_handler {
                if handler(
                    self.severity_,
                    self.file_,
                    self.line_,
                    self.message_start_,
                    text,
                ) {
                    return;
                }
            }
            super::logging::RawLog(self.severity_, text);
            #[cfg(all(
                not(pa_official),
                any(pa_is_posix, pa_is_win),
                not(pa_uclibc),
                not(pa_is_aix)
            ))]
            if self.severity_ == LOGGING_FATAL {
                let mut traces = [MaybeUninit::<*const core::ffi::c_void>::uninit(); 32];
                let count =
                    super::debug::stack_trace::CollectStackTrace(traces.as_mut_ptr().cast(), 32);
                super::debug::stack_trace::PrintStackTrace(traces.as_ptr().cast(), count);
            }
            if self.severity_ == LOGGING_FATAL {
                PA_IMMEDIATE_CRASH!();
            }
        }
    }
}
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
pub type SystemErrorCode = c_int;
#[cfg(pa_is_win)]
pub type SystemErrorCode = core::ffi::c_ulong;
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
pub unsafe fn GetLastSystemErrorCode() -> SystemErrorCode {
    *super::scoped_clear_last_error::errno_location()
}
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
pub struct ErrnoLogMessage {
    err_: SystemErrorCode,
    parent: LogMessage,
}
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
impl ErrnoLogMessage {
    pub unsafe fn init_at(
        out: *mut Self,
        file: *const c_char,
        line: c_int,
        severity: LogSeverity,
        err: SystemErrorCode,
    ) {
        LogMessage::init_at(core::ptr::addr_of_mut!((*out).parent), file, line, severity);
        core::ptr::addr_of_mut!((*out).err_).write(err);
    }
    pub fn stream(self: Pin<&mut Self>) -> Pin<&mut CStringBuilder> {
        unsafe { self.map_unchecked_mut(|s| &mut s.parent) }.stream()
    }
}
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
pub unsafe fn SystemErrorCodeToStream(
    mut os: Pin<&mut CStringBuilder>,
    error_code: SystemErrorCode,
) {
    let mut buffer = [MaybeUninit::<c_char>::uninit(); 256];
    super::posix::safe_strerror::safe_strerror_r(error_code, buffer.as_mut_ptr().cast(), 256);
    os.as_mut().put_cstr(buffer.as_ptr().cast());
    os.as_mut().put_cstr(b" (\0".as_ptr().cast());
    os.as_mut().put_int(error_code);
    os.as_mut().put_cstr(b")\0".as_ptr().cast());
}
#[cfg(any(pa_is_posix, pa_is_fuchsia))]
impl Drop for ErrnoLogMessage {
    fn drop(&mut self) {
        unsafe {
            let err = self.err_;
            let mut pinned = Pin::new_unchecked(&mut *self);
            pinned.as_mut().stream().put_cstr(b": \0".as_ptr().cast());
            SystemErrorCodeToStream(pinned.as_mut().stream(), err);
            let last_error = err;
            super::debug::alias::Alias((&last_error as *const SystemErrorCode).cast());
        }
    }
}
// Windows FormatMessageA, GetLastError, and Win32ErrorLogMessage remain exact platform pending.
