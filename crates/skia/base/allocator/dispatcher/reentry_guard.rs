// Source: base/allocator/dispatcher/reentry_guard.h:31-82; .cc:24-69
use core::marker::PhantomData;
#[cfg(any(target_vendor = "apple", target_os = "android"))]
static mut ENTERED_KEY: libc::pthread_key_t = 0;
pub struct ReentryGuard {
    #[cfg(any(target_vendor = "apple", target_os = "android"))]
    allowed_: bool,
    thread_: PhantomData<*mut ()>,
}
impl ReentryGuard {
    /// InitTLSSlot must have completed before hooks, with no concurrent initialization.
    pub unsafe fn new() -> Self {
        #[cfg(any(target_vendor = "apple", target_os = "android"))]
        {
            let allowed_ = libc::pthread_getspecific(ENTERED_KEY).is_null();
            libc::pthread_setspecific(ENTERED_KEY, core::ptr::without_provenance_mut(1));
            Self {
                allowed_,
                thread_: PhantomData,
            }
        }
        #[cfg(not(any(target_vendor = "apple", target_os = "android")))]
        {
            Self {
                thread_: PhantomData,
            }
        }
    }
    pub fn is_allowed(&self) -> bool {
        #[cfg(any(target_vendor = "apple", target_os = "android"))]
        {
            self.allowed_
        }
        #[cfg(not(any(target_vendor = "apple", target_os = "android")))]
        {
            true
        }
    }
    pub unsafe fn InitTLSSlot() {
        #[cfg(any(target_vendor = "apple", target_os = "android"))]
        {
            if ENTERED_KEY == 0 {
                let error = libc::pthread_key_create(core::ptr::addr_of_mut!(ENTERED_KEY), None);
                crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"!error\0".as_ptr().cast();error==0);
                libc::pthread_setspecific(ENTERED_KEY, core::ptr::null_mut());
            }
            crate::DCHECK!(@location crate::base::location::Location::Current(); @condition b"entered_key_ != kNullKey\0".as_ptr().cast();ENTERED_KEY!=0);
        }
    }
    pub unsafe fn RecordTLSSlotToCrashKey() {
        use crate::base::debug::crash_logging::{
            AllocateCrashKeyString, CrashKeySize, SetCrashKeyString,
        };
        // Actual C++ local-static pointer initialization; pointee intentionally not owned/freed here.
        struct Key(*mut crate::base::debug::crash_logging::CrashKeyString);
        unsafe impl Send for Key {}
        unsafe impl Sync for Key {}
        static KEY: std::sync::OnceLock<Key> = std::sync::OnceLock::new();
        let key = KEY
            .get_or_init(|| {
                Key(AllocateCrashKeyString(
                    "reentry_guard_tls_slot",
                    CrashKeySize::Size32,
                ))
            })
            .0;
        #[cfg(any(target_vendor = "apple", target_os = "android"))]
        SetCrashKeyString(
            key,
            crate::base::strings::string_number_conversions::NumberToString(ENTERED_KEY),
        );
        #[cfg(not(any(target_vendor = "apple", target_os = "android")))]
        SetCrashKeyString(key, "unused");
    }
}
impl Drop for ReentryGuard {
    fn drop(&mut self) {
        #[cfg(any(target_vendor = "apple", target_os = "android"))]
        if core::intrinsics::likely(self.allowed_) {
            unsafe {
                libc::pthread_setspecific(ENTERED_KEY, core::ptr::null_mut());
            }
        }
    }
}
