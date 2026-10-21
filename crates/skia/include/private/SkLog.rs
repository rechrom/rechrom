// Source: include/private/SkLog.h:20-20
#[cfg(all(
    skia_use_default_log,
    not(any(target_os = "windows", target_os = "android"))
))]
pub use crate::src::ports::SkLog_stdio::SkLogVAList;
// Real platform-defined entry: its Windows/Android body is outside this admitted
// stdio source. This declaration is not an output fallback or implemented sink.
#[cfg(all(
    skia_use_default_log,
    any(target_os = "windows", target_os = "android")
))]
unsafe extern "C" {
    pub fn SkLogVAList(
        priority: crate::include::private::SkLogPriority::SkLogPriority,
        format: *const u8,
        args: core::ffi::VaList<'_>,
    );
}
// Source: include/private/SkLog.h:21-21
use crate::include::private::SkLogPriority::SkLogPriority;
#[cfg(skia_use_default_log)]
pub use crate::src::core::SkLog::SkLog;
// Source: include/private/SkLog.h:24-30
// Native user-configuration macro value maps to a build cfg value. Absent an
// override, effective native sk_debug comes from the actual config stream;
// Cargo debug assertions do not override explicit native SK_RELEASE.
#[cfg(skia_lowest_active_log_priority = "error")]
pub const SKIA_LOWEST_ACTIVE_LOG_PRIORITY: SkLogPriority = SkLogPriority::kError;
#[cfg(skia_lowest_active_log_priority = "warning")]
pub const SKIA_LOWEST_ACTIVE_LOG_PRIORITY: SkLogPriority = SkLogPriority::kWarning;
#[cfg(skia_lowest_active_log_priority = "info")]
pub const SKIA_LOWEST_ACTIVE_LOG_PRIORITY: SkLogPriority = SkLogPriority::kInfo;
#[cfg(skia_lowest_active_log_priority = "debug")]
pub const SKIA_LOWEST_ACTIVE_LOG_PRIORITY: SkLogPriority = SkLogPriority::kDebug;
#[cfg(all(
    not(any(
        skia_lowest_active_log_priority = "error",
        skia_lowest_active_log_priority = "warning",
        skia_lowest_active_log_priority = "info",
        skia_lowest_active_log_priority = "debug"
    )),
    sk_debug
))]
pub const SKIA_LOWEST_ACTIVE_LOG_PRIORITY: SkLogPriority = SkLogPriority::kDebug;
#[cfg(all(
    not(any(
        skia_lowest_active_log_priority = "error",
        skia_lowest_active_log_priority = "warning",
        skia_lowest_active_log_priority = "info",
        skia_lowest_active_log_priority = "debug"
    )),
    not(sk_debug)
))]
pub const SKIA_LOWEST_ACTIVE_LOG_PRIORITY: SkLogPriority = SkLogPriority::kInfo;
// Source: include/private/SkLog.h:32-37
#[macro_export]
macro_rules! SKIA_LOG{
    (@source $file:expr,$line:expr; $priority:expr,$fmt:expr $(,$argument:expr)* $(,)?)=>{{
        const ACTIVE:bool=($priority as i32)<=($crate::include::private::SkLog::SKIA_LOWEST_ACTIVE_LOG_PRIORITY as i32);
        if ACTIVE{$crate::SkLog!(@source $file,$line; $priority,concat!("[skia] ",$fmt,"\n\0").as_ptr() $(,$argument)*);}
    }};

    ($priority:expr,$fmt:expr $(,$argument:expr)* $(,)?)=>{{
        const ACTIVE:bool=($priority as i32)<=($crate::include::private::SkLog::SKIA_LOWEST_ACTIVE_LOG_PRIORITY as i32);
        if ACTIVE{
            $crate::SkLog!($priority,concat!("[skia] ",$fmt,"\n\0").as_ptr() $(,$argument)*);
        }
    }};
}
// Source: include/private/SkLog.h:39-39
#[macro_export]
macro_rules! SKIA_LOG_E{(@source $file:expr,$line:expr; $fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!(@source $file,$line; $crate::include::private::SkLogPriority::SkLogPriority::kError,concat!("** ERROR ** ",$fmt) $(,$argument)*);
};
($fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!($crate::include::private::SkLogPriority::SkLogPriority::kError,concat!("** ERROR ** ",$fmt) $(,$argument)*);
};}
// Source: include/private/SkLog.h:40-40
#[macro_export]
macro_rules! SKIA_LOG_W{(@source $file:expr,$line:expr; $fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!(@source $file,$line; $crate::include::private::SkLogPriority::SkLogPriority::kWarning,concat!("WARNING - ",$fmt) $(,$argument)*);
};
($fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!($crate::include::private::SkLogPriority::SkLogPriority::kWarning,concat!("WARNING - ",$fmt) $(,$argument)*);
};}
// Source: include/private/SkLog.h:41-41
#[macro_export]
macro_rules! SKIA_LOG_I{(@source $file:expr,$line:expr; $fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!(@source $file,$line; $crate::include::private::SkLogPriority::SkLogPriority::kInfo,$fmt $(,$argument)*);
};
($fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!($crate::include::private::SkLogPriority::SkLogPriority::kInfo,$fmt $(,$argument)*);
};}
// Source: include/private/SkLog.h:42-42
#[macro_export]
macro_rules! SKIA_LOG_D{(@source $file:expr,$line:expr; $fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!(@source $file,$line; $crate::include::private::SkLogPriority::SkLogPriority::kDebug,$fmt $(,$argument)*);
};
($fmt:expr $(,$argument:expr)* $(,)?)=>{
    $crate::SKIA_LOG!($crate::include::private::SkLogPriority::SkLogPriority::kDebug,$fmt $(,$argument)*);
};}
