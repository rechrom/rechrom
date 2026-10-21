// Source: base/check.h:57-65
// The class has no fields. Both constructors and operator& have actual empty
// bodies; accepting an already evaluated argument preserves ordinary call rules.
pub struct VoidifyStream;
impl VoidifyStream {
    pub const fn new() -> Self {
        Self
    }
    pub fn from_bool(_: bool) -> Self {
        Self
    }
    pub fn ignore_stream(&self, _stream: &mut crate::interop::native_ostream::NativeOStream<'_>) {}
}
// Source: base/check.h:73-73
// Genuine native incomplete declaration: no fabricated LogMessage object,
// stream implementation, destructor or allocation provider.
unsafe extern "C" {
    pub type LogMessage;
}
// Source: base/check.cc:49-55
#[cfg(FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION)]
pub const kCheckFailureMessageSeparator: &[u8] = b"; ";
#[cfg(not(FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION))]
pub const kCheckFailureMessageSeparator: &[u8] = b". ";
// Source: base/check.h:68-71
// Rust requires the same boolean expressions to type-check but never evaluates
// either them or optional stream argument expressions on the disabled path.
#[macro_export]
macro_rules! EAT_CHECK_STREAM_PARAMS {
    ($condition:expr $(,$stream:expr)* $(,)?)=>{{
        if false {let _:bool=$condition;$(let _=&$stream;)*}
    }};
}
// Source: base/check.h:244-246
#[cfg(all(OFFICIAL_BUILD, not(sk_native_ndebug)))]
compile_error!("Debug builds are not expected to be optimized as official builds.");
// Source: base/check.h:248-275
pub const CHECK_WILL_STREAM: bool =
    !cfg!(all(OFFICIAL_BUILD, sk_native_ndebug, not(DCHECK_ALWAYS_ON)));
// Source: base/check.h:290-304
// The enabled branch calls the actual CheckError producer. Its native owning
// LogMessage implementation remains a dependency, rather than an abort/no-op.
// A caller supplies the actual native Location, preserving its source identity.
#[macro_export]
macro_rules! DCHECK {
    (@location $location:expr; @condition $native:expr; $condition:expr) => {{
        #[cfg(any(not(sk_native_ndebug), DCHECK_ALWAYS_ON))]
        {
            if !$condition {
                let _error = $crate::base::check::CheckError::DCheck($native, &$location);
            }
        }
        #[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
        {
            $crate::EAT_CHECK_STREAM_PARAMS!(!$condition);
        }
    }};
}
// Source: base/check.h:295-304
#[macro_export]
macro_rules! DPCHECK {
    (@location $location:expr; @condition $native:expr; $condition:expr) => {{
        #[cfg(any(not(sk_native_ndebug), DCHECK_ALWAYS_ON))]
        {
            if !$condition {
                let _error = $crate::base::check::CheckError::DPCheck($native, &$location);
            }
        }
        #[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
        {
            $crate::EAT_CHECK_STREAM_PARAMS!(!$condition);
        }
    }};
}

// Source: base/check.h:257-259
#[cfg(all(OFFICIAL_BUILD, sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
pub fn CheckFailure() -> ! {
    crate::base::immediate_crash::ImmediateCrash()
}
// Source: base/check.h:264-280
#[macro_export]
macro_rules! CHECK {
    (@location $location:expr; @condition $native:expr; $condition:expr) => {{
        if !$condition {
            #[cfg(all(OFFICIAL_BUILD, sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
            {
                $crate::base::check::CheckFailure();
            }
            #[cfg(not(all(OFFICIAL_BUILD, sk_native_ndebug, not(DCHECK_ALWAYS_ON))))]
            {
                let _error = $crate::base::check::CheckNoreturnError::Check($native, &$location);
            }
        }
    }};
    (@location $location:expr; @condition $native:expr; $condition:expr, $milestone:expr) => {{
        if !$condition {
            let _error = $crate::base::check::CheckError::Check($native, $milestone, &$location);
        }
    }};
}
// Source: base/check.h:283-288
#[macro_export]
macro_rules! PCHECK {
    (@location $location:expr; @condition $native:expr; $condition:expr) => {{
        if !$condition {
            #[cfg(all(OFFICIAL_BUILD, sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
            {
                let _error = $crate::base::check::CheckNoreturnError::PCheck_location(&$location);
            }
            #[cfg(not(all(OFFICIAL_BUILD, sk_native_ndebug, not(DCHECK_ALWAYS_ON))))]
            {
                let _error = $crate::base::check::CheckNoreturnError::PCheck($native, &$location);
            }
        }
    }};
}
// Source: base/check.h:331-335
#[macro_export]
macro_rules! DUMP_WILL_BE_CHECK {
    (@location $location:expr; @condition $native:expr; $condition:expr) => {{
        if !$condition {
            let _error = $crate::base::check::CheckError::DumpWillBeCheck($native, &$location);
        }
    }};
}
// Source: base/check.h:337-345
#[macro_export]
macro_rules! RAW_CHECK {
    (@condition $native:literal; $condition:expr) => {{
        if !$condition {
            unsafe {
                $crate::base::check::RawCheckFailure(
                    concat!("Check failed: ", $native, "\n\0").as_ptr().cast(),
                );
            }
        }
    }};
}

use crate::base::debug::crash_logging::{CrashKeySize, CrashKeyString};
use crate::base::not_fatal_until::NotFatalUntil;

// Source: base/check.cc:58-67
/// Safety: actual LOGGING_DCHECK global configuration follows its native
/// synchronization contract. This symbol is not replaced by a guessed severity.
unsafe fn GetDumpSeverity() -> i32 {
    #[cfg(OFFICIAL_BUILD)]
    {
        if crate::base::dcheck_is_on::DCHECK_IS_ON {
            unsafe { crate::base::logging::LOGGING_DCHECK }
        } else {
            crate::base::logging::LOGGING_ERROR
        }
    }
    #[cfg(not(OFFICIAL_BUILD))]
    {
        crate::base::logging::LOGGING_FATAL
    }
}

// Source: base/check_version_internal.h.in:10-10
// The generated native integer token is an actual build input. There is no
// fallback from HOST, current date, browser version guess or source enum value.
const NATIVE_CHECK_VERSION: i32 = {
    let bytes = env!("BASE_CHECK_VERSION_INTERNAL").as_bytes();
    assert!(
        !bytes.is_empty(),
        "Actual generated BASE_CHECK_VERSION_INTERNAL is required"
    );
    let mut result = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        assert!(
            bytes[i] >= b'0' && bytes[i] <= b'9',
            "Actual native check version must be a decimal integer"
        );
        result = match result.checked_mul(10) {
            Some(value) => value,
            None => panic!("native check version outside int domain"),
        };
        result = match result.checked_add((bytes[i] - b'0') as i32) {
            Some(value) => value,
            None => panic!("native check version outside int domain"),
        };
        i += 1;
    }
    result
};
// Source: base/check.cc:69-75
unsafe fn GetNotFatalUntilSeverity(fatal_milestone: NotFatalUntil) -> i32 {
    if fatal_milestone != NotFatalUntil::NoSpecifiedMilestoneInternal
        && (fatal_milestone as i32) <= NATIVE_CHECK_VERSION
    {
        return crate::base::logging::LOGGING_FATAL;
    }
    unsafe { GetDumpSeverity() }
}
// Source: base/check.cc:77-83
unsafe fn GetCheckSeverity(fatal_milestone: NotFatalUntil) -> i32 {
    if fatal_milestone == NotFatalUntil::NoSpecifiedMilestoneInternal {
        return crate::base::logging::LOGGING_FATAL;
    }
    unsafe { GetNotFatalUntilSeverity(fatal_milestone) }
}

// Native function-local pointer initialization has a language initialization
// guard and no process-exit delete of the returned pointer. Each function below
// gets its own same-address slot/Once. Even an initial null result is memoized.
macro_rules! native_crash_key {
    ($function:ident,$name:literal,$size:ident) => {
        unsafe fn $function() -> *mut CrashKeyString {
            struct Slot(core::cell::UnsafeCell<*mut CrashKeyString>);
            unsafe impl Sync for Slot {}
            static VALUE: Slot = Slot(core::cell::UnsafeCell::new(core::ptr::null_mut()));
            static INIT: std::sync::Once = std::sync::Once::new();
            INIT.call_once_force(|_| unsafe {
                VALUE
                    .0
                    .get()
                    .write(crate::base::debug::crash_logging::AllocateCrashKeyString(
                        concat!($name, "\0").as_ptr().cast(),
                        CrashKeySize::$size,
                    ));
            });
            unsafe { VALUE.0.get().read() }
        }
    };
}
// Source: base/check.cc:85-89
native_crash_key!(
    GetNotReachedCrashKey,
    "Logging-NOTREACHED_MESSAGE",
    Size1024
);
// Source: base/check.cc:91-95
native_crash_key!(GetDCheckCrashKey, "Logging-DCHECK_MESSAGE", Size1024);
// Source: base/check.cc:97-102
native_crash_key!(
    GetDumpWillBeCheckCrashKey,
    "Logging-DUMP_WILL_BE_CHECK_MESSAGE",
    Size1024
);
// Source: base/check.cc:104-108
native_crash_key!(GetFatalMilestoneCrashKey, "Logging-FATAL_MILESTONE", Size32);

// Source: base/check.cc:110-117
unsafe fn MaybeSetFatalMilestoneCrashKey(fatal_milestone: NotFatalUntil) {
    if fatal_milestone == NotFatalUntil::NoSpecifiedMilestoneInternal {
        return;
    }
    let text =
        crate::base::strings::string_number_conversions::NumberToString(fatal_milestone as i32);
    unsafe {
        crate::base::debug::crash_logging::SetCrashKeyString(GetFatalMilestoneCrashKey(), &text);
    }
}

// Source: base/check.cc:458-461
/// Safety: message is a valid NUL-terminated string; the actual RawLog producer
/// must retain its native fatal nonreturning behavior before unreachable.
pub unsafe fn RawCheckFailure(message: *const core::ffi::c_char) -> ! {
    unsafe {
        crate::base::logging::RawLog(crate::base::logging::LOGGING_FATAL, message.cast());
        core::hint::unreachable_unchecked()
    }
}
