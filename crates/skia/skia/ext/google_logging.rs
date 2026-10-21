//! Actual FileLine producer algorithms. The native compiler shim only owns the
//! C variadic va_start/va_end protocol; no logging algorithm/provider is native.
use crate::include::private::SkLogPriority::SkLogPriority;
#[cfg(sk_native_ostream_abi_configured)]
use crate::interop::native_ostream::ConfiguredStdString;
use core::{
    ffi::{c_int, c_void, VaList},
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
};
#[cfg(not(sk_native_ostream_abi_configured))]
compile_error!("FileLine logging requires genuine supplied native string/stream ABI and the real LogMessage owner");

// Source: skia/ext/google_logging.cc:13-17
// Actual explicit C shim symbols, defined in interop/native_ostream.cpp. No
// guessed ordinary C++ mangled symbol or Rust auto-va_start before filtering.
unsafe extern "C" {
    #[link_name = "rechrom_google_sklog_fileline"]
    pub fn SkLog_FileLine(
        file: *const u8,
        line: c_int,
        priority: SkLogPriority,
        format: *const u8,
        ...
    );
    #[link_name = "rechrom_google_debug_fileline"]
    pub fn SkDebugf_FileLine(file: *const u8, line: c_int, format: *const u8, ...);
    #[link_name = "rechrom_google_abort_fileline"]
    pub fn SkAbort_FileLine(file: *const u8, line: c_int, format: *const u8, ...);
}
// Source: skia/ext/google_logging.cc:18-36
fn severity_from_raw(priority: c_int) -> i32 {
    if crate::base::dcheck_is_on::DCHECK_IS_ON {
        match priority {
            x if x == SkLogPriority::kError as c_int => crate::base::logging::LOGGING_ERROR,
            x if x == SkLogPriority::kWarning as c_int => crate::base::logging::LOGGING_WARNING,
            _ => crate::base::logging::LOGGING_INFO,
        }
    } else {
        crate::base::logging::LOGGING_INFO
    }
}
pub fn SkLog_FileLine_severity(priority: SkLogPriority) -> i32 {
    severity_from_raw(priority as c_int)
}
// Source: skia/ext/google_logging.cc:52-56
pub fn SkDebugf_FileLine_severity() -> i32 {
    if crate::base::dcheck_is_on::DCHECK_IS_ON {
        crate::base::logging::LOGGING_ERROR
    } else {
        crate::base::logging::LOGGING_INFO
    }
}
// Source: skia/ext/google_logging.cc:71-71
pub const fn SkAbort_FileLine_severity() -> i32 {
    crate::base::logging::LOGGING_FATAL
}

// Source: skia/ext/google_logging.cc:37-39
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rechrom_google_sklog_prepare(
    priority: c_int,
    severity: *mut c_int,
) -> bool {
    let value = severity_from_raw(priority);
    unsafe {
        severity.write(value);
    }
    value >= unsafe { crate::base::logging::GetMinLogLevel() }
}
// Source: skia/ext/google_logging.cc:57-58
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rechrom_google_debug_prepare(severity: *mut c_int) -> bool {
    let value = SkDebugf_FileLine_severity();
    unsafe {
        severity.write(value);
    }
    value >= unsafe { crate::base::logging::GetMinLogLevel() }
}

// Source: skia/ext/google_logging.cc:71-71
#[unsafe(no_mangle)]
pub extern "C" fn rechrom_google_abort_prepare() -> c_int {
    SkAbort_FileLine_severity()
}

// The only Rust owner is a cleanup projection of the original same stack value;
// MaybeUninit does not destroy it a second time. No heap/wrapper message is made.
struct StackValue<'a, T> {
    value: Pin<&'a mut T>,
}
impl<T> Drop for StackValue<'_, T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.value.as_mut().get_unchecked_mut());
        }
    }
}

// Source: skia/ext/google_logging.cc:42-49
// Source: skia/ext/google_logging.cc:61-68
/// ap is the compiler's borrowed original cursor, never a new va_end owner.
/// ap_slot is the actual native va_list object created after filtering in the
/// explicit shim. Rust tells the real compiler to end that same slot exactly
/// at line47/66, before constructing the complete-expression LogMessage.
#[cfg(sk_native_ostream_abi_configured)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rechrom_google_log_body(
    file: *const u8,
    line: c_int,
    severity: c_int,
    format: *const u8,
    ap: ManuallyDrop<VaList<'_>>,
    ap_slot: *mut c_void,
) {
    let mut msg_slot = core::pin::pin!(MaybeUninit::<ConfiguredStdString>::uninit());
    unsafe {
        ConfiguredStdString::initialize_empty_at_assuming_native_abi(
            msg_slot.as_mut().get_unchecked_mut().as_mut_ptr(),
        );
    }
    let mut msg = StackValue {
        value: unsafe {
            msg_slot
                .as_mut()
                .map_unchecked_mut(|slot| slot.assume_init_mut())
        },
    };
    unsafe {
        crate::base::strings::stringprintf::StringAppendV(msg.value.as_mut(), format, &ap);
    }
    unsafe {
        crate::interop::native_ostream::rechrom_native_va_end_slot(ap_slot);
    }
    {
        let mut log_slot =
            core::pin::pin!(MaybeUninit::<crate::base::logging::LogMessage>::uninit());
        match unsafe {
            crate::base::logging::LogMessage::construct_at(
                log_slot.as_mut().get_unchecked_mut().as_mut_ptr(),
                file,
                line,
                severity,
            )
        } {
            Ok(()) => {}
            Err(error) => panic!("supplied native logging ABI mismatch: {error:?}"),
        }
        let mut log = StackValue {
            value: unsafe {
                log_slot
                    .as_mut()
                    .map_unchecked_mut(|slot| slot.assume_init_mut())
            },
        };
        log.value
            .as_mut()
            .stream()
            .insert_string(msg.value.as_ref());
    } // Full expression: real LogMessage::Drop/Flush before msg destruction.
}

// Source: skia/ext/google_logging.cc:73-84
#[cfg(sk_native_ostream_abi_configured)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rechrom_google_abort_body(
    file: *const u8,
    line: c_int,
    severity: c_int,
    format: *const u8,
    ap: ManuallyDrop<VaList<'_>>,
    ap_slot: *mut c_void,
) {
    // This call contains the original same local msg and real full-expression
    // LogMessage lifetime. In native SkAbort, msg lives until function exit;
    // abort never exits. Keep it here through both abort calls, not in a helper.
    let mut msg_slot = core::pin::pin!(MaybeUninit::<ConfiguredStdString>::uninit());
    unsafe {
        ConfiguredStdString::initialize_empty_at_assuming_native_abi(
            msg_slot.as_mut().get_unchecked_mut().as_mut_ptr(),
        );
    }
    let mut msg = StackValue {
        value: unsafe {
            msg_slot
                .as_mut()
                .map_unchecked_mut(|slot| slot.assume_init_mut())
        },
    };
    unsafe {
        crate::base::strings::stringprintf::StringAppendV(msg.value.as_mut(), format, &ap);
    }
    unsafe {
        crate::interop::native_ostream::rechrom_native_va_end_slot(ap_slot);
    }
    {
        let mut log_slot =
            core::pin::pin!(MaybeUninit::<crate::base::logging::LogMessage>::uninit());
        match unsafe {
            crate::base::logging::LogMessage::construct_at(
                log_slot.as_mut().get_unchecked_mut().as_mut_ptr(),
                file,
                line,
                severity,
            )
        } {
            Ok(()) => {}
            Err(error) => panic!("supplied native logging ABI mismatch: {error:?}"),
        }
        let mut log = StackValue {
            value: unsafe {
                log_slot
                    .as_mut()
                    .map_unchecked_mut(|slot| slot.assume_init_mut())
            },
        };
        log.value
            .as_mut()
            .stream()
            .insert_string(msg.value.as_ref());
    }
    unsafe {
        crate::include::private::SkAssert::sk_abort_no_print();
    }
    unsafe {
        libc::abort();
    }
}
