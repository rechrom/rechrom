use core::sync::atomic::{AtomicBool, Ordering};

// Source: src/core/SkSpinlock.h:16-46
pub struct SkSpinlock {
    fLocked: AtomicBool,
}
impl SkSpinlock {
    // Source: src/core/SkSpinlock.h:18,46
    pub const fn new() -> Self {
        Self {
            fLocked: AtomicBool::new(false),
        }
    }
    // Source: src/core/SkSpinlock.h:20-26
    pub fn acquire(&self) {
        if self.fLocked.swap(true, Ordering::Acquire) {
            self.contendedAcquire();
        }
    }
    // Source: src/core/SkSpinlock.h:29-36
    pub fn tryAcquire(&self) -> bool {
        if self.fLocked.swap(true, Ordering::Acquire) {
            return false;
        }
        true
    }
    // Source: src/core/SkSpinlock.h:38-41
    /// Safety: caller owns this lock, and ends all protected accesses before
    /// release. Object remains live through the store. No access follows it.
    pub unsafe fn release_raw(this: *const Self) {
        unsafe { &(*this).fLocked }.store(false, Ordering::Release);
    }
    // Same actual native release; borrowed convenience requires a live lock.
    pub unsafe fn release(&self) {
        unsafe { Self::release_raw(self) };
    }
    // Source: src/core/SkSpinlock.cpp:44-53
    fn contendedAcquire(&self) {
        debug_trace();
        while self.fLocked.swap(true, Ordering::Acquire) {
            do_pause();
        }
    }
}
// Source: src/core/SkSpinlock.h:18
impl Default for SkSpinlock {
    fn default() -> Self {
        Self::new()
    }
}

// Source: src/core/SkSpinlock.h:49-58
// Single borrowed guard, no heap allocation or additional lock provider.
pub struct SkAutoSpinlock<'a> {
    fSpinlock: &'a SkSpinlock,
}
impl<'a> SkAutoSpinlock<'a> {
    // Source: src/core/SkSpinlock.h:51-53
    pub fn new(mutex: &'a SkSpinlock) -> Self {
        let result = Self { fSpinlock: mutex };
        result.fSpinlock.acquire();
        result
    }
}
// Source: src/core/SkSpinlock.h:54
impl Drop for SkAutoSpinlock<'_> {
    fn drop(&mut self) {
        unsafe { SkSpinlock::release_raw(self.fSpinlock) };
    }
}

// Source: src/core/SkSpinlock.cpp:13-33
// Native #if 0 tracing remains disabled: no backtrace, mutex or stderr work is
// enabled by this translation. The selected #else is the real empty function.
fn debug_trace() {}

// Source: src/core/SkSpinlock.cpp:36-42
#[cfg(sk_features_x64_ge_sse2)]
fn do_pause() {
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    compile_error!("native configured SSE2 pause requires the actual x86 intrinsic target");
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_pause()
    };
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::x86::_mm_pause()
    };
}
#[cfg(not(sk_features_x64_ge_sse2))]
fn do_pause() { /* native empty spin */
}
