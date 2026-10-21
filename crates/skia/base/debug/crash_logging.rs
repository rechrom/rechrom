use core::{ffi::c_char, ptr::NonNull};
// Source: base/debug/crash_logging.h:10-10
// Only the genuine iosfwd ostream specialization identity is represented.
// No Rust stream value can be constructed; ostream language operations remain pending.
unsafe extern "C" {
    pub type StdOStream;
}
// Source: base/debug/crash_logging.h:68-73
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashKeySize {
    Size32 = 32,
    Size64 = 64,
    Size256 = 256,
    Size1024 = 1024,
}
// Source: base/debug/crash_logging.h:205-210
// Actual base metadata, not a fabricated crash reporter allocation.
pub struct CrashKeyString {
    pub name: *const c_char,
    pub size: CrashKeySize,
}
impl CrashKeyString {
    pub const unsafe fn new(name: *const c_char, size: CrashKeySize) -> Self {
        Self { name, size }
    }
}
// Source: base/debug/crash_logging.h:189-197
// Actual pure virtual provider: every method requires the real implementation.
// Global calls use shared raw leases; an implementation retains the original
// synchronization obligations without invented unique mutable provenance.
pub trait CrashKeyImplementation {
    unsafe fn Allocate(&self, name: *const c_char, size: CrashKeySize) -> *mut CrashKeyString;
    unsafe fn Set(&self, key: *mut CrashKeyString, value: &[u8]);
    unsafe fn Clear(&self, key: *mut CrashKeyString);
    unsafe fn OutputCrashKeysToStream(&self, out: *mut StdOStream);
}
// Source: base/debug/crash_logging.cc:17-17
static mut g_crash_key_impl: Option<NonNull<dyn CrashKeyImplementation>> = None;
// Source: base/debug/crash_logging.cc:21-46
/// Safety: name is a stable native NUL-terminated string. Provider replacement
/// excludes all active calls; each provider obeys its native synchronization.
pub unsafe fn AllocateCrashKeyString(
    name: *const c_char,
    value_length: CrashKeySize,
) -> *mut CrashKeyString {
    #[cfg(all(
        any(not(sk_native_ndebug), DCHECK_ALWAYS_ON),
        not(target_os = "android")
    ))]
    {
        let name_piece = unsafe { core::ffi::CStr::from_ptr(name).to_bytes() };
        let found = name_piece
            .iter()
            .position(|&c| c == b':')
            .unwrap_or(usize::MAX);
        // Exact native op-check diagnostics/stream remain required, not a bool
        // assertion or no-op in their place. Their producer is independently pending.
        crate::DCHECK_EQ!(@location unsafe{crate::base::location::Location::CurrentWithoutFunctionName(b"base/debug/crash_logging.cc\0".as_ptr().cast(),31)}; @condition "std::string_view::npos == name_piece.find(':')"; usize::MAX,found; @stream b"; name_piece = ",name_piece);
        crate::DCHECK_LT!(@location unsafe{crate::base::location::Location::CurrentWithoutFunctionName(b"base/debug/crash_logging.cc\0".as_ptr().cast(),38)}; @condition "name_piece.size() < 40u"; name_piece.len(),40usize);
    }
    let Some(implementation) = (unsafe { g_crash_key_impl }) else {
        return core::ptr::null_mut();
    };
    unsafe { implementation.as_ref().Allocate(name, value_length) }
}
// Source: base/debug/crash_logging.cc:48-54
/// Safety: provider/key lifetimes and original concurrent-call contracts hold.
pub unsafe fn SetCrashKeyString(key: *mut CrashKeyString, value: &[u8]) {
    let Some(implementation) = (unsafe { g_crash_key_impl }) else {
        return;
    };
    if key.is_null() {
        return;
    }
    unsafe {
        implementation.as_ref().Set(key, value);
    }
}
// Source: base/debug/crash_logging.cc:56-62
pub unsafe fn ClearCrashKeyString(key: *mut CrashKeyString) {
    let Some(implementation) = (unsafe { g_crash_key_impl }) else {
        return;
    };
    if key.is_null() {
        return;
    }
    unsafe {
        implementation.as_ref().Clear(key);
    }
}
// Source: base/debug/crash_logging.cc:64-70
pub unsafe fn OutputCrashKeysToStream(out: *mut StdOStream) {
    let Some(implementation) = (unsafe { g_crash_key_impl }) else {
        return;
    };
    unsafe {
        implementation.as_ref().OutputCrashKeysToStream(out);
    }
}
// Source: base/debug/crash_logging.h:102-119
#[must_use]
pub struct ScopedCrashKeyString {
    crash_key_: *mut CrashKeyString,
}
impl ScopedCrashKeyString {
    // Source: base/debug/crash_logging.cc:72-76
    pub unsafe fn new(key: *mut CrashKeyString, value: &[u8]) -> Self {
        unsafe {
            SetCrashKeyString(key, value);
        }
        Self { crash_key_: key }
    }
    // Source: base/debug/crash_logging.cc:78-79
    // Rust move extraction leaves the same null moved-from object for Drop.
    pub fn move_from(other: &mut Self) -> Self {
        Self {
            crash_key_: core::mem::replace(&mut other.crash_key_, core::ptr::null_mut()),
        }
    }
}
// Source: base/debug/crash_logging.cc:81-83
impl Drop for ScopedCrashKeyString {
    fn drop(&mut self) {
        unsafe {
            ClearCrashKeyString(self.crash_key_);
        }
    }
}
// Source: base/debug/crash_logging.cc:85-88
/// Safety: replacement excludes every active callback and old scoped key use;
/// ownership of the new implementation transfers exactly as unique_ptr::release.
pub unsafe fn SetCrashKeyImplementation(implementation: Option<Box<dyn CrashKeyImplementation>>) {
    let old = unsafe { g_crash_key_impl };
    if let Some(old) = old {
        unsafe {
            drop(Box::from_raw(old.as_ptr()));
        }
    }
    unsafe {
        g_crash_key_impl = implementation.map(|value| NonNull::new_unchecked(Box::into_raw(value)));
    }
}
// Source: base/debug/crash_logging.h:130-148
// One call-site static key is retained even if its first allocation is null.
// Rust naming is caller-supplied only to express the C++ lexical guard variable.
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_STRING_INTERNAL {
    ($guard:ident,$category:literal,$name:literal,$data:expr,$size:expr) => {
        const {
            let name = concat!($category, "-", $name, "\0").as_bytes();
            assert!(
                name.len() < 40,
                "Crash key names must be shorter than 40 characters."
            );
            let mut i = 0;
            while i < name.len() {
                assert!(
                    name[i] != b':',
                    "Crash key names must not contain the ':' character."
                );
                i += 1;
            }
        }
        let $guard = {
            struct KeySlot(
                core::cell::UnsafeCell<*mut $crate::base::debug::crash_logging::CrashKeyString>,
            );
            unsafe impl Sync for KeySlot {}
            static ONCE: std::sync::Once = std::sync::Once::new();
            static KEY: KeySlot = KeySlot(core::cell::UnsafeCell::new(core::ptr::null_mut()));
            ONCE.call_once_force(|_| unsafe {
                KEY.0
                    .get()
                    .write($crate::base::debug::crash_logging::AllocateCrashKeyString(
                        concat!($category, "-", $name, "\0").as_ptr().cast(),
                        $size,
                    ));
            });
            unsafe {
                $crate::base::debug::crash_logging::ScopedCrashKeyString::new(
                    KEY.0.get().read(),
                    $data,
                )
            }
        };
    };
}

// Source: base/debug/crash_logging.h:158-172
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_STRING32 {
    ($guard:ident,$category:literal,$name:literal,$data:expr) => {
        $crate::SCOPED_CRASH_KEY_STRING_INTERNAL!(
            $guard,
            $category,
            $name,
            $data,
            $crate::base::debug::crash_logging::CrashKeySize::Size32
        );
    };
}
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_STRING64 {
    ($guard:ident,$category:literal,$name:literal,$data:expr) => {
        $crate::SCOPED_CRASH_KEY_STRING_INTERNAL!(
            $guard,
            $category,
            $name,
            $data,
            $crate::base::debug::crash_logging::CrashKeySize::Size64
        );
    };
}
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_STRING256 {
    ($guard:ident,$category:literal,$name:literal,$data:expr) => {
        $crate::SCOPED_CRASH_KEY_STRING_INTERNAL!(
            $guard,
            $category,
            $name,
            $data,
            $crate::base::debug::crash_logging::CrashKeySize::Size256
        );
    };
}
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_STRING1024 {
    ($guard:ident,$category:literal,$name:literal,$data:expr) => {
        $crate::SCOPED_CRASH_KEY_STRING_INTERNAL!(
            $guard,
            $category,
            $name,
            $data,
            $crate::base::debug::crash_logging::CrashKeySize::Size1024
        );
    };
}
// Source: base/debug/crash_logging.h:174-177
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_BOOL {
    ($guard:ident,$category:literal,$name:literal,$data:expr) => {
        $crate::SCOPED_CRASH_KEY_STRING32!(
            $guard,
            $category,
            $name,
            if $data {
                b"true" as &[u8]
            } else {
                b"false" as &[u8]
            }
        );
    };
}
// Source: base/debug/crash_logging.h:179-180
#[macro_export]
macro_rules! SCOPED_CRASH_KEY_NUMBER {
    ($guard:ident,$category:literal,$name:literal,$data:expr) => {
        $crate::SCOPED_CRASH_KEY_STRING32!(
            $guard,
            $category,
            $name,
            &$crate::base::strings::string_number_conversions::NumberToString($data)
        );
    };
}
