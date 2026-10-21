#[cfg(any(pa_is_posix, pa_is_fuchsia))]
use super::log_message::ErrnoLogMessage as ErrorLogMessage;
#[cfg(pa_is_win)]
use super::log_message::Win32ErrorLogMessage as ErrorLogMessage;
use super::log_message::{LogMessage, LogSeverity, SystemErrorCode, LOGGING_ERROR, LOGGING_FATAL};
use super::strings::cstring_builder::CStringBuilder;
use core::{
    ffi::{c_char, c_int},
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
};
// Source: partition_alloc_base/check.h: CheckError actual active-member union, no extra count/owner.
union CheckMessage {
    log_message_: ManuallyDrop<LogMessage>,
    errno_log_message_: ManuallyDrop<ErrorLogMessage>,
}
pub struct CheckError {
    message: CheckMessage,
    has_errno: bool,
}
impl CheckError {
    // Source: partition_alloc_base/check.cc:17-34
    pub unsafe fn init_at(
        out: *mut Self,
        file: *const c_char,
        line: c_int,
        severity: LogSeverity,
        condition: Option<*const c_char>,
    ) {
        let log = core::ptr::addr_of_mut!((*out).message.log_message_).cast::<LogMessage>();
        LogMessage::init_at(log, file, line, severity);
        core::ptr::addr_of_mut!((*out).has_errno).write(false);
        if let Some(condition) = condition {
            let mut stream = Pin::new_unchecked(&mut *log).stream();
            stream
                .as_mut()
                .put_cstr(b"Check failed: \0".as_ptr().cast());
            stream.as_mut().put_cstr(condition);
            stream.as_mut().put_cstr(b". \0".as_ptr().cast());
        }
    }
    pub unsafe fn init_errno_at(
        out: *mut Self,
        file: *const c_char,
        line: c_int,
        severity: LogSeverity,
        condition: *const c_char,
        err: SystemErrorCode,
    ) {
        let log =
            core::ptr::addr_of_mut!((*out).message.errno_log_message_).cast::<ErrorLogMessage>();
        ErrorLogMessage::init_at(log, file, line, severity, err);
        core::ptr::addr_of_mut!((*out).has_errno).write(true);
        let mut stream = Pin::new_unchecked(&mut *log).stream();
        stream
            .as_mut()
            .put_cstr(b"Check failed: \0".as_ptr().cast());
        stream.as_mut().put_cstr(condition);
        stream.as_mut().put_cstr(b". \0".as_ptr().cast());
    }
    // Source: partition_alloc_base/check.cc:66-68
    pub unsafe fn stream(self: Pin<&mut Self>) -> Pin<&mut CStringBuilder> {
        let s = self.get_unchecked_mut();
        if !s.has_errno {
            Pin::new_unchecked(&mut *s.message.log_message_).stream()
        } else {
            Pin::new_unchecked(&mut *s.message.errno_log_message_).stream()
        }
    }
}
impl Drop for CheckError {
    // Source: partition_alloc_base/check.cc:70-83. NOMERGE attribute guarantee remains pending.
    fn drop(&mut self) {
        unsafe {
            if !self.has_errno {
                ManuallyDrop::drop(&mut self.message.log_message_);
            } else {
                ManuallyDrop::drop(&mut self.message.errno_log_message_);
            }
        }
    }
}
// Source: partition_alloc_base/check.cc:85-88
pub unsafe fn RawCheckFailure(message: *const c_char) -> ! {
    super::logging::RawLog(LOGGING_FATAL, message);
    super::immediate_crash::PA_IMMEDIATE_CRASH!();
}
// A borrowed cleanup lease for the caller's original stack storage; no allocated/moved log object.
pub struct CheckErrorLease<'a> {
    object: Pin<&'a mut CheckError>,
}
impl<'a> CheckErrorLease<'a> {
    pub unsafe fn new(
        storage: Pin<&'a mut MaybeUninit<CheckError>>,
        file: *const c_char,
        line: c_int,
        severity: LogSeverity,
        condition: Option<*const c_char>,
        errno: bool,
    ) -> Self {
        let p = storage.get_unchecked_mut().as_mut_ptr();
        if errno {
            CheckError::init_errno_at(
                p,
                file,
                line,
                severity,
                condition.unwrap_or(b"\0".as_ptr().cast()),
                super::log_message::GetLastSystemErrorCode(),
            );
        } else {
            CheckError::init_at(p, file, line, severity, condition);
        }
        Self {
            object: Pin::new_unchecked(&mut *p),
        }
    }
    pub unsafe fn stream(&mut self) -> Pin<&mut CStringBuilder> {
        self.object.as_mut().stream()
    }
}
impl Drop for CheckErrorLease<'_> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.object.as_mut().get_unchecked_mut());
        }
    }
}
#[cfg(all(pa_official, pa_is_debug))]
compile_error!("Debug builds are not expected to be optimized as official builds.");
// Original C++ expression spelling/source are explicit metadata. Never stringify Rust replacements.
#[macro_export]
macro_rules! PA_BASE_CHECK {
    (@source $file:expr,$line:expr; @condition $cpp:expr; $condition:expr) => {{
        if !$condition {
            #[cfg(all(pa_official,not(pa_dchecks_are_on)))]
            $crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::immediate_crash::PA_IMMEDIATE_CRASH!();
            #[cfg(not(all(pa_official,not(pa_dchecks_are_on))))]
            unsafe {
                let mut storage=core::pin::pin!(core::mem::MaybeUninit::uninit());
                let _error=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::check::CheckErrorLease::new(storage.as_mut(),$file,$line,3,Some($cpp),false);
            }
        }
    }};
}
#[macro_export]
macro_rules! PA_BASE_DCHECK {
    ($($tokens:tt)*) => {{#[cfg(pa_dchecks_are_on)] $crate::PA_BASE_DCHECK_ENABLED!($($tokens)*);}};
}
#[macro_export]
macro_rules! PA_BASE_DCHECK_ENABLED {
    (@source $file:expr,$line:expr; @condition $cpp:expr; $condition:expr) => {{
        if !$condition {unsafe{
            let mut storage=core::pin::pin!(core::mem::MaybeUninit::uninit());
            let severity=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::logging::LOGGING_DCHECK;
            let _error=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::check::CheckErrorLease::new(storage.as_mut(),$file,$line,severity,Some($cpp),false);
        }}
    }};
}
// PA_BASE_{P,D P}CHECK and stream-expression/operator forms retain exact pending protocol, not silent no-ops.

// Source: partition_alloc_base/check.h: check_error derived declarations; check.cc:36-74.
// Ordinary constructors initialize the original parent's storage, never a substitute factory.
pub mod check_error {
    use super::*;
    pub struct Check {
        parent: CheckError,
    }
    pub struct DCheck {
        parent: CheckError,
    }
    pub struct PCheck {
        parent: CheckError,
    }
    pub struct DPCheck {
        parent: CheckError,
    }
    pub struct NotImplemented {
        parent: CheckError,
    }
    impl Check {
        pub unsafe fn init_at(
            out: *mut Self,
            file: *const c_char,
            line: c_int,
            condition: *const c_char,
        ) {
            CheckError::init_at(
                core::ptr::addr_of_mut!((*out).parent),
                file,
                line,
                LOGGING_FATAL,
                Some(condition),
            );
        }
    }
    impl DCheck {
        pub unsafe fn init_at(
            out: *mut Self,
            file: *const c_char,
            line: c_int,
            condition: *const c_char,
        ) {
            CheckError::init_at(
                core::ptr::addr_of_mut!((*out).parent),
                file,
                line,
                super::super::logging::LOGGING_DCHECK,
                Some(condition),
            );
        }
    }
    impl PCheck {
        pub unsafe fn init_at(
            out: *mut Self,
            file: *const c_char,
            line: c_int,
            condition: *const c_char,
        ) {
            CheckError::init_errno_at(
                core::ptr::addr_of_mut!((*out).parent),
                file,
                line,
                LOGGING_FATAL,
                condition,
                super::super::log_message::GetLastSystemErrorCode(),
            );
        }
        pub unsafe fn init_no_condition_at(out: *mut Self, file: *const c_char, line: c_int) {
            Self::init_at(out, file, line, b"\0".as_ptr().cast());
        }
    }
    impl DPCheck {
        pub unsafe fn init_at(
            out: *mut Self,
            file: *const c_char,
            line: c_int,
            condition: *const c_char,
        ) {
            CheckError::init_errno_at(
                core::ptr::addr_of_mut!((*out).parent),
                file,
                line,
                super::super::logging::LOGGING_DCHECK,
                condition,
                super::super::log_message::GetLastSystemErrorCode(),
            );
        }
    }
    impl NotImplemented {
        pub unsafe fn init_at(
            out: *mut Self,
            file: *const c_char,
            line: c_int,
            function: *const c_char,
        ) {
            let parent = core::ptr::addr_of_mut!((*out).parent);
            CheckError::init_at(parent, file, line, LOGGING_ERROR, None);
            let mut stream = Pin::new_unchecked(&mut *parent).stream();
            stream
                .as_mut()
                .put_cstr(b"Not implemented reached in \0".as_ptr().cast());
            stream.as_mut().put_cstr(function);
        }
    }
}
// Source: partition_alloc_base/check.h: VoidifyStream default/bool ctor and operator&.
pub struct VoidifyStream;
impl VoidifyStream {
    pub const fn new() -> Self {
        Self
    }
    pub fn from_bool(_ignored: bool) -> Self {
        Self
    }
    pub fn consume(&self, _stream: Pin<&mut CStringBuilder>) {}
}
#[macro_export]
macro_rules! PA_BASE_PCHECK{
    (@source $file:expr,$line:expr; @condition $cpp:expr; $condition:expr)=>{{if !$condition{unsafe{
        let mut storage=core::pin::pin!(core::mem::MaybeUninit::uninit());
        #[cfg(all(pa_official,not(pa_dchecks_are_on)))]let cpp=b"\0".as_ptr().cast();
        #[cfg(not(all(pa_official,not(pa_dchecks_are_on))))]let cpp=$cpp;
        let _error=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::check::CheckErrorLease::new(storage.as_mut(),$file,$line,3,Some(cpp),true);
    }}}};
}
#[macro_export]
macro_rules! PA_BASE_DPCHECK{
    (@source $file:expr,$line:expr; @condition $cpp:expr; $condition:expr)=>{{#[cfg(pa_dchecks_are_on)]if !$condition{unsafe{
        let mut storage=core::pin::pin!(core::mem::MaybeUninit::uninit());
        let severity=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::logging::LOGGING_DCHECK;
        let _error=$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::check::CheckErrorLease::new(storage.as_mut(),$file,$line,severity,Some($cpp),true);
    }}}};
}
#[macro_export]
macro_rules! PA_RAW_CHECK{
    (@condition $cpp:literal; $condition:expr)=>{{if !$condition{unsafe{$crate::base::allocator::partition_allocator::src::partition_alloc::partition_alloc_base::check::RawCheckFailure(concat!("Check failed: ",$cpp,"\n\0").as_ptr().cast());}}}};
}
