// Source: base/time/time_override.h; base/time/time_override.cc; time.cc:25-45.
#![allow(non_snake_case, non_upper_case_globals)]
use super::time::{LiveTicks, ThreadTicks, Time, TimeTicks};
#[cfg(target_vendor = "apple")]
use super::time_apple as platform;
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "fuchsia")))]
use super::time_now_posix as platform;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicPtr, Ordering};
pub type TimeNowFunction = fn() -> Time;
pub type TimeTicksNowFunction = fn() -> TimeTicks;
pub type TimeTicksLowResolutionNowFunction = fn() -> TimeTicks;
pub type LiveTicksNowFunction = fn() -> LiveTicks;
pub type ThreadTicksNowFunction = fn() -> ThreadTicks;
// Source: time.cc:25-45 and time_override.h:90-96.
// Private atomic bits only transport the original fn()->T value. They are never
// dereferenced as data, never reconstructed with a different return domain,
// and cannot represent nullptr because store/new require a real function.
// This Rust language adapter is not exposed as a C++ std::atomic object layout;
// targets without function/data pointer round-trip representation stay pending.
pub struct AtomicNow<T> {
    pointer: AtomicPtr<()>,
    signature: PhantomData<fn() -> T>,
}
impl<T> AtomicNow<T> {
    pub const fn new(f: fn() -> T) -> Self {
        Self {
            pointer: AtomicPtr::new(f as *mut ()),
            signature: PhantomData,
        }
    }
    pub fn load(&self, o: Ordering) -> fn() -> T {
        let address = self.pointer.load(o);
        unsafe { core::mem::transmute::<*mut (), fn() -> T>(address) }
    }
    pub fn store(&self, f: fn() -> T, o: Ordering) {
        self.pointer.store(f as *mut (), o)
    }
}
pub static g_time_now_function: AtomicNow<Time> = AtomicNow::new(platform::TimeNowIgnoringOverride);
pub static g_time_now_from_system_time_function: AtomicNow<Time> =
    AtomicNow::new(platform::TimeNowFromSystemTimeIgnoringOverride);
pub static g_time_ticks_now_function: AtomicNow<TimeTicks> =
    AtomicNow::new(platform::TimeTicksNowIgnoringOverride);
pub static g_time_ticks_low_resolution_now_function: AtomicNow<TimeTicks> =
    AtomicNow::new(platform::TimeTicksLowResolutionNowIgnoringOverride);
pub static g_live_ticks_now_function: AtomicNow<LiveTicks> =
    AtomicNow::new(LiveTicksNowIgnoringOverride);
pub static g_thread_ticks_now_function: AtomicNow<ThreadTicks> =
    AtomicNow::new(platform::ThreadTicksNowIgnoringOverride);
pub use platform::{
    ThreadTicksNowIgnoringOverride, TimeNowFromSystemTimeIgnoringOverride, TimeNowIgnoringOverride,
    TimeTicksLowResolutionNowIgnoringOverride, TimeTicksNowIgnoringOverride,
};
pub fn LiveTicksNowIgnoringOverride() -> LiveTicks {
    LiveTicks::new() + (TimeTicks::Now() - TimeTicks::new())
}
impl Time {
    pub fn Now() -> Self {
        let now = g_time_now_function.load(Ordering::Relaxed);
        now()
    }
    pub fn NowFromSystemTime() -> Self {
        let now = g_time_now_from_system_time_function.load(Ordering::Relaxed);
        now()
    }
}
impl TimeTicks {
    pub fn Now() -> Self {
        let now = g_time_ticks_now_function.load(Ordering::Relaxed);
        now()
    }
    pub fn LowResolutionNow() -> Self {
        let now = g_time_ticks_low_resolution_now_function.load(Ordering::Relaxed);
        now()
    }
}
impl LiveTicks {
    pub fn Now() -> Self {
        let now = g_live_ticks_now_function.load(Ordering::Relaxed);
        now()
    }
}
impl ThreadTicks {
    pub fn Now() -> Self {
        let now = g_thread_ticks_now_function.load(Ordering::Relaxed);
        now()
    }
}
static mut overrides_active_: bool = false;
pub struct ScopedTimeClockOverrides {
    _private: (),
}
impl ScopedTimeClockOverrides {
    /// Safety: native process-global override scope must be externally synchronized,
    /// nonnested, and outlive every concurrently executing callback it installs.
    pub unsafe fn new(
        time: Option<TimeNowFunction>,
        ticks: Option<TimeTicksNowFunction>,
        thread: Option<ThreadTicksNowFunction>,
        live: Option<LiveTicksNowFunction>,
        low: Option<TimeTicksLowResolutionNowFunction>,
    ) -> Self {
        crate::DCHECK!(@location crate::FROM_HERE!(@function b"ScopedTimeClockOverrides\0".as_ptr().cast();@file b"base/time/time_override.cc\0".as_ptr().cast();@line 20);@condition b"!overrides_active_\0".as_ptr().cast();unsafe{!overrides_active_});
        unsafe {
            overrides_active_ = true;
        }
        if let Some(f) = time {
            g_time_now_function.store(f, Ordering::Relaxed);
            g_time_now_from_system_time_function.store(f, Ordering::Relaxed)
        }
        if let Some(f) = ticks {
            g_time_ticks_now_function.store(f, Ordering::Relaxed)
        }
        if let Some(f) = live {
            g_live_ticks_now_function.store(f, Ordering::Relaxed)
        }
        if let Some(f) = thread {
            g_thread_ticks_now_function.store(f, Ordering::Relaxed)
        }
        if let Some(f) = low {
            g_time_ticks_low_resolution_now_function.store(f, Ordering::Relaxed)
        }
        Self { _private: () }
    }
    /// Safety: reading the native non-atomic debug state requires scope synchronization.
    pub unsafe fn overrides_active() -> bool {
        unsafe { overrides_active_ }
    }
}
impl Drop for ScopedTimeClockOverrides {
    fn drop(&mut self) {
        g_time_now_function.store(TimeNowIgnoringOverride, Ordering::SeqCst);
        g_time_now_from_system_time_function
            .store(TimeNowFromSystemTimeIgnoringOverride, Ordering::SeqCst);
        g_time_ticks_now_function.store(TimeTicksNowIgnoringOverride, Ordering::SeqCst);
        g_live_ticks_now_function.store(LiveTicksNowIgnoringOverride, Ordering::SeqCst);
        g_thread_ticks_now_function.store(ThreadTicksNowIgnoringOverride, Ordering::SeqCst);
        g_time_ticks_low_resolution_now_function
            .store(TimeTicksLowResolutionNowIgnoringOverride, Ordering::SeqCst);
        unsafe {
            overrides_active_ = false;
        }
    }
}
