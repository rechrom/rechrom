use crate::include::private::SkLogPriority::SkLogPriority;
use crate::src::utils::SkLogHandler::SkLogHandler;
// Source: include/private/SkLog.h:21-21
// Source: src/core/SkLog.cpp:16-29
/// Safety: format matches the actual C-promoted arguments; pointer arguments
/// remain live throughout handler callbacks and synchronous platform forwarding.
#[cfg(skia_use_default_log)]
pub unsafe extern "C" fn SkLog(priority: SkLogPriority, format: *const u8, args: ...) {
    // Binder is actual va_start. Clone is compiler-provided va_copy; callback
    // frame Drop performs va_end of that independent copy before handler unref.
    {
        let owner = <dyn SkLogHandler>::GetInstance();
        if let Some(handler) = owner.get() {
            unsafe {
                handler.onLog(priority, format, args.clone());
            }
        }
    }
    // Original list is forwarded without consuming it in C: the actual Rust
    // sink frame ends this list exactly once after its libc forwarding returns.
    unsafe {
        crate::include::private::SkLog::SkLogVAList(priority, format, args);
    }
}

// Source: skia/config/SkUserConfig.h:61-70
unsafe extern "C" {
    pub fn SkLog_FileLine(
        file: *const u8,
        line: core::ffi::c_int,
        priority: SkLogPriority,
        format: *const u8,
        ...
    );
}
#[cfg(not(skia_use_default_log))]
#[macro_export]
macro_rules! SkLog {
    (@source $file:expr,$line:expr; $priority:expr,$format:expr $(,$argument:expr)* $(,)?)=>{{
        unsafe{$crate::src::core::SkLog::SkLog_FileLine(
            $file,$line as core::ffi::c_int,$priority,$format $(,$argument)*);}
    }};
    ($priority:expr,$format:expr $(,$argument:expr)* $(,)?)=>{compile_error!("Chromium logging requires @source native_file_ptr,native_line; format,args to preserve original C++ caller location")};
}
#[cfg(skia_use_default_log)]
#[macro_export]
macro_rules! SkLog {
    (@source $file:expr,$line:expr; $priority:expr,$format:expr $(,$argument:expr)* $(,)?)=>{{
        unsafe{$crate::src::core::SkLog::SkLog($priority,$format $(,$argument)*);}
    }};
    ($priority:expr,$format:expr $(,$argument:expr)* $(,)?)=>{{
        unsafe{$crate::src::core::SkLog::SkLog($priority,$format $(,$argument)*);}
    }};
}
