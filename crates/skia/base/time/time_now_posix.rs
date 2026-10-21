// Source: base/time/time_now_posix.cc. Excludes native Apple and Fuchsia TUs.
#![cfg(all(unix, not(target_vendor = "apple"), not(target_os = "fuchsia")))]
#![allow(non_snake_case)]
use super::time::{Clock, Microseconds, ThreadTicks, Time, TimeTicks};
use core::mem::MaybeUninit;
macro_rules! check {($function:expr,$line:expr,$cpp:expr,$cond:expr)=>{crate::CHECK!(@location crate::FROM_HERE!(@function $function.as_ptr().cast();@file b"base/time/time_now_posix.cc\0".as_ptr().cast();@line $line);@condition $cpp.as_ptr().cast();$cond)}}
fn ConvertTimespecToMicros(ts: &libc::timespec) -> i64 {
    if core::mem::size_of_val(&ts.tv_sec) <= 4 && core::mem::size_of_val(&ts.tv_nsec) <= 8 {
        return ts.tv_sec as i64 * 1_000_000 + ts.tv_nsec as i64 / 1000;
    }
    let value = (ts.tv_sec as i64)
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(ts.tv_nsec as i64 / 1000));
    // CheckedNumeric::ValueOrDie invokes the genuine numeric CheckOnFailure trap.
    match value {
        Some(v) => v,
        None => crate::base::numerics::safe_conversions_impl::CheckOnFailure::HandleFailure(),
    }
}
// Input cfg mirrors the actual unistd _POSIX_MONOTONIC_CLOCK >= 0 branch;
// BSD and Android are explicitly selected by native source, independently.
#[cfg(any(
    base_posix_monotonic_clock,
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "android"
))]
fn ClockNow(id: libc::clockid_t) -> i64 {
    let mut ts = MaybeUninit::uninit();
    check!(
        b"ClockNow\0",
        55,
        b"clock_gettime(clk_id, &ts) == 0\0",
        unsafe { libc::clock_gettime(id, ts.as_mut_ptr()) == 0 }
    );
    ConvertTimespecToMicros(unsafe { ts.assume_init_ref() })
}
#[cfg(any(
    base_posix_monotonic_clock,
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "android"
))]
fn MaybeClockNow(id: libc::clockid_t) -> Option<i64> {
    let mut ts = MaybeUninit::uninit();
    let res = unsafe { libc::clock_gettime(id, ts.as_mut_ptr()) };
    if res == 0 {
        Some(ConvertTimespecToMicros(unsafe { ts.assume_init_ref() }))
    } else {
        None
    }
}
pub fn TimeNowIgnoringOverride() -> Time {
    let mut tv = MaybeUninit::<libc::timeval>::uninit();
    let mut tz = libc::timezone {
        tz_minuteswest: 0,
        tz_dsttime: 0,
    };
    check!(
        b"TimeNowIgnoringOverride\0",
        82,
        b"gettimeofday(&tv, &tz) == 0\0",
        unsafe {
            libc::gettimeofday(tv.as_mut_ptr(), (&mut tz as *mut libc::timezone).cast()) == 0
        }
    );
    let tv = unsafe { tv.assume_init() };
    Time::new()
        + Microseconds(
            (tv.tv_sec as i64 * 1_000_000 + tv.tv_usec as i64)
                + Time::kMicrosecondsFromWindowsToUnixEpoch,
        )
}
pub fn TimeNowFromSystemTimeIgnoringOverride() -> Time {
    TimeNowIgnoringOverride()
}
pub fn TimeTicksNowIgnoringOverride() -> TimeTicks {
    TimeTicks::new() + Microseconds(ClockNow(libc::CLOCK_MONOTONIC))
}
pub fn MaybeTimeTicksNowIgnoringOverride() -> Option<TimeTicks> {
    MaybeClockNow(libc::CLOCK_MONOTONIC).map(|now| TimeTicks::new() + Microseconds(now))
}
pub fn TimeTicksLowResolutionNowIgnoringOverride() -> TimeTicks {
    TimeTicks::new() + Microseconds(ClockNow(libc::CLOCK_MONOTONIC_COARSE))
}
impl TimeTicks {
    pub fn GetClock() -> Clock {
        Clock::LINUX_CLOCK_MONOTONIC
    }
    pub fn IsHighResolution() -> bool {
        true
    }
    pub fn IsConsistentAcrossProcesses() -> bool {
        true
    }
}
#[cfg(any(base_posix_thread_cputime, target_os = "android"))]
pub fn ThreadTicksNowIgnoringOverride() -> ThreadTicks {
    ThreadTicks::new() + Microseconds(ClockNow(libc::CLOCK_THREAD_CPUTIME_ID))
}
// Native alternative NOTREACHED requires its actual diagnostic producer; no fake clock.
#[cfg(not(any(
    base_posix_monotonic_clock,
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "android"
)))]
compile_error!("No usable tick clock function on this platform.");
#[cfg(not(any(base_posix_thread_cputime, target_os = "android")))]
pub fn ThreadTicksNowIgnoringOverride() -> ThreadTicks {
    crate::NOTREACHED!(@location crate::FROM_HERE!(@function b"ThreadTicksNowIgnoringOverride\0".as_ptr().cast();@file b"base/time/time_now_posix.cc\0".as_ptr().cast();@line 140));
}
