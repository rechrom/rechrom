// Source: src/core/SkDebug.cpp:15-19
#[cfg(sk_build_for_google3)]
pub unsafe extern "C" fn SkDebugfForDumpStackTrace(
    data: *const u8,
    _unused: *mut core::ffi::c_void,
) {
    crate::SkDebugf!(@source b"src/core/SkDebug.cpp\0".as_ptr(),17; b"%s\0".as_ptr(),data);
}

// Source: include/private/SkDebug.h:15-17
// Source: src/core/SkDebug.cpp:22-27
/// Safety: printf format and promoted arguments correspond, and every pointer
/// remains live for synchronous platform logging. No formatted-copy substitute.
#[cfg(skia_use_default_debugf)]
pub unsafe extern "C" fn SkDebugf(format: *const u8, args: ...) {
    // The actual variadic binder performs va_start. Passing the real list to
    // the native sink transfers its one va_end obligation to that Rust frame.
    unsafe {
        crate::include::private::SkLog::SkLogVAList(
            crate::include::private::SkLogPriority::SkLogPriority::kDebug,
            format,
            args,
        );
    }
}

// Source: include/private/SkDebug.h:19-25
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkDEBUGCODE{($($code:tt)*)=>{$($code)*};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkDEBUGCODE {
    ($($code:tt)*) => {};
}
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkDEBUGF{
(@source $file:expr,$line:expr; $format:expr $(,$argument:expr)* $(,)?)=>{$crate::SkDebugf!(@source $file,$line; $format $(,$argument)*);};
($format:expr $(,$argument:expr)* $(,)?)=>{
     $crate::SkDebugf!($format $(,$argument)*);
};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkDEBUGF {
    ($($argument:tt)*) => {};
}

// Source: skia/config/SkUserConfig.h:60-65
// Actual user override is selected by default, not a variadic forwarding shim.
unsafe extern "C" {
    pub fn SkDebugf_FileLine(file: *const u8, line: core::ffi::c_int, format: *const u8, ...);
}
#[cfg(not(skia_use_default_debugf))]
#[macro_export]
macro_rules! SkDebugf {
    (@source $file:expr,$line:expr; $format:expr $(,$argument:expr)* $(,)?)=>{{
        unsafe{$crate::src::core::SkDebug::SkDebugf_FileLine(
            $file,$line as core::ffi::c_int,$format $(,$argument)*);}
    }};
    ($format:expr $(,$argument:expr)* $(,)?)=>{compile_error!("Chromium logging requires @source native_file_ptr,native_line; format,args to preserve original C++ caller location")};
}
#[cfg(skia_use_default_debugf)]
#[macro_export]
macro_rules! SkDebugf {
    (@source $file:expr,$line:expr; $format:expr $(,$argument:expr)* $(,)?)=>{{
        unsafe{$crate::src::core::SkDebug::SkDebugf($format $(,$argument)*);}
    }};
    ($format:expr $(,$argument:expr)* $(,)?)=>{{
        unsafe{$crate::src::core::SkDebug::SkDebugf($format $(,$argument)*);}
    }};
}
