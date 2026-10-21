//! Actual Chromium printf growth algorithm on the original native std::string.
//! Formatting and va_list/errno SDK operations use genuine primitive bindings.
#[cfg(not(sk_native_ostream_abi_configured))]
compile_error!("Actual native std::string/ostream ABI metadata must be supplied before Chromium StringAppendV logging can instantiate");
use crate::interop::native_ostream::{
    rechrom_native_errno_address, rechrom_native_vspanprintf, NativeStdString, StdStringStorage,
};
use core::{
    ffi::{c_int, VaList},
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
};

// Source: base/scoped_clear_last_error.h:28-39
pub(crate) struct ScopedClearLastError {
    last_errno: c_int,
    #[cfg(windows)]
    last_system_error: core::ffi::c_ulong,
}
pub(crate) unsafe fn errno_ptr() -> *mut c_int {
    unsafe { rechrom_native_errno_address() }
}
impl ScopedClearLastError {
    pub(crate) unsafe fn new() -> Self {
        let p = unsafe { errno_ptr() };
        let last_errno = unsafe { *p };
        unsafe {
            *p = 0;
        }
        // Source: base/scoped_clear_last_error_win.cc:11-14
        #[cfg(windows)]
        {
            let last_system_error =
                unsafe { crate::interop::native_ostream::rechrom_native_get_last_error() };
            unsafe {
                crate::interop::native_ostream::rechrom_native_set_last_error(0);
            }
            Self {
                last_errno,
                last_system_error,
            }
        }
        #[cfg(not(windows))]
        {
            Self { last_errno }
        }
    }
}
impl Drop for ScopedClearLastError {
    fn drop(&mut self) {
        unsafe {
            // Source: base/scoped_clear_last_error_win.cc:16-18
            #[cfg(windows)]
            crate::interop::native_ostream::rechrom_native_set_last_error(self.last_system_error);
            // Native base destructor follows derived destructor.
            *errno_ptr() = self.last_errno;
        }
    }
}

// Source: base/strings/span_printf.h:26-33
unsafe fn VSpanPrintf(buffer: &mut [u8], format: *const u8, args: &VaList<'_>) -> i32 {
    // ManuallyDrop is the transparent borrowed FFI cursor carrier only. Its
    // real owner performs va_end; this call never adds another owner/end.
    unsafe {
        rechrom_native_vspanprintf(
            buffer.as_mut_ptr(),
            buffer.len(),
            format,
            ManuallyDrop::new(core::ptr::read(args)),
        )
    }
}

// Source: base/strings/stringprintf.cc:20-24
/// Native by-value std::string return maps to its caller's actual pinned return
/// slot. S supplies genuine exact native ABI storage; format/args obey printf.
pub unsafe fn StringPrintV_at<S: StdStringStorage>(
    out: *mut NativeStdString<S>,
    format: *const u8,
    ap: &VaList<'_>,
) {
    unsafe {
        NativeStdString::<S>::initialize_empty_at_assuming_native_abi(out);
    }
    unsafe {
        StringAppendV(Pin::new_unchecked(&mut *out), format, ap);
    }
}

// Source: base/strings/stringprintf.cc:26-88
/// format is a native terminated C format; all promoted operands and pointees
/// remain live. ap is borrowed from the caller's existing cursor: each clone is
/// actual va_copy and is explicitly dropped (va_end) at the original source line.
pub unsafe fn StringAppendV<S: StdStringStorage>(
    mut dst: Pin<&mut NativeStdString<S>>,
    format: *const u8,
    ap: &VaList<'_>,
) {
    // Source: base/strings/stringprintf.cc:30-37
    let mut stack_buf = [0u8; 1024];
    let ap_copy = <VaList<'_> as Clone>::clone(ap);
    let _last_error = unsafe { ScopedClearLastError::new() };
    let mut result = unsafe { VSpanPrintf(&mut stack_buf, format, &ap_copy) };
    drop(ap_copy);
    // Source: base/strings/stringprintf.cc:39-43
    if result >= 0 && (result as usize) < stack_buf.len() {
        dst.as_mut().append(&stack_buf[..result as usize]);
        return;
    }
    // Source: base/strings/stringprintf.cc:46-65
    let mut mem_length = stack_buf.len();
    loop {
        if result < 0 {
            #[cfg(windows)]
            {
                return;
            }
            #[cfg(not(windows))]
            {
                let errno = unsafe { *errno_ptr() };
                if errno != 0 && errno != libc::EOVERFLOW {
                    return;
                }
                mem_length = mem_length.wrapping_mul(2);
            }
        } else {
            mem_length = result as usize + 1;
        }
        // Source: base/strings/stringprintf.cc:66-72
        if mem_length > 32 * 1024 * 1024 {
            #[cfg(any(not(sk_native_ndebug), DCHECK_ALWAYS_ON))]
            unsafe {
                log_size_warning();
            }
            return;
        }
        // Source: base/strings/stringprintf.cc:74-85
        let mut mem_buf = vec![0u8; mem_length];
        let ap_copy = <VaList<'_> as Clone>::clone(ap);
        result = unsafe { VSpanPrintf(&mut mem_buf, format, &ap_copy) };
        drop(ap_copy);
        if result >= 0 && (result as usize) < mem_length {
            dst.as_mut().append(&mem_buf[..result as usize]);
            return;
        }
    }
}

// The cleanup borrow destroys exactly the same initialized stack complete
// LogMessage, never a second owner/allocation. Its MaybeUninit slot does not Drop.
struct StackLog<'a> {
    value: Pin<&'a mut crate::base::logging::LogMessage>,
}
impl Drop for StackLog<'_> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.value.as_mut().get_unchecked_mut());
        }
    }
}
// Source: base/strings/stringprintf.cc:70-70
#[cfg(any(not(sk_native_ndebug), DCHECK_ALWAYS_ON))]
unsafe fn log_size_warning() {
    let severity = crate::base::logging::LOGGING_WARNING;
    if !unsafe { crate::base::logging::ShouldCreateLogMessage(severity) } {
        return;
    }
    let mut slot = core::pin::pin!(MaybeUninit::<crate::base::logging::LogMessage>::uninit());
    match unsafe {
        crate::base::logging::LogMessage::construct_at(
            slot.as_mut().get_unchecked_mut().as_mut_ptr(),
            b"base/strings/stringprintf.cc\0".as_ptr(),
            70,
            severity,
        )
    } {
        Ok(()) => {}
        Err(error) => panic!("supplied native logging ABI mismatch: {error:?}"),
    }
    let mut log = StackLog {
        value: unsafe {
            slot.as_mut()
                .map_unchecked_mut(|slot| slot.assume_init_mut())
        },
    };
    log.value
        .as_mut()
        .stream()
        .insert_bytes(b"Unable to printf the requested string due to size.");
}

// The following are genuine fixed FormatSpec instantiations, not a variadic
// substitute for Absl's general compile-time typed StringPrintf template.
#[cfg(sk_native_ostream_abi_configured)]
use crate::interop::native_ostream::ConfiguredStdString;
unsafe extern "C" {
    fn rechrom_absl_hex8_at(out: *mut core::ffi::c_void, value: u32);
    fn rechrom_absl_system_hex_at(out: *mut core::ffi::c_void, value: core::ffi::c_ulong);
    fn rechrom_absl_system_error_at(
        out: *mut core::ffi::c_void,
        last: core::ffi::c_ulong,
        value: core::ffi::c_ulong,
    );
    fn rechrom_absl_system_decimal_at(out: *mut core::ffi::c_void, value: c_int);
    fn rechrom_absl_crash_string_at(
        out: *mut core::ffi::c_void,
        file: *const u8,
        line: c_int,
        data: *const u8,
        len: usize,
    );
}
#[cfg(sk_native_ostream_abi_configured)]
pub unsafe fn StringPrintfHex8_at(out: *mut ConfiguredStdString, value: u32) {
    unsafe {
        rechrom_absl_hex8_at(ConfiguredStdString::native_return_slot(out), value);
    }
}
#[cfg(sk_native_ostream_abi_configured)]
pub unsafe fn StringPrintfSystemHex_at(out: *mut ConfiguredStdString, value: core::ffi::c_ulong) {
    unsafe {
        rechrom_absl_system_hex_at(ConfiguredStdString::native_return_slot(out), value);
    }
}
#[cfg(sk_native_ostream_abi_configured)]
pub unsafe fn StringPrintfSystemError_at(
    out: *mut ConfiguredStdString,
    last: core::ffi::c_ulong,
    value: core::ffi::c_ulong,
) {
    unsafe {
        rechrom_absl_system_error_at(ConfiguredStdString::native_return_slot(out), last, value);
    }
}
#[cfg(sk_native_ostream_abi_configured)]
pub unsafe fn StringPrintfSystemDecimal_at(out: *mut ConfiguredStdString, value: c_int) {
    unsafe {
        rechrom_absl_system_decimal_at(ConfiguredStdString::native_return_slot(out), value);
    }
}
#[cfg(sk_native_ostream_abi_configured)]
pub unsafe fn BuildCrashStringFormat_at(
    out: *mut ConfiguredStdString,
    file: *const u8,
    line: i32,
    message: &[u8],
) {
    unsafe {
        rechrom_absl_crash_string_at(
            ConfiguredStdString::native_return_slot(out),
            file,
            line,
            message.as_ptr(),
            message.len(),
        );
    }
}
