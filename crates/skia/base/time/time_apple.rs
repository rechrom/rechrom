// Source: base/time/time_apple.mm (the upstream TU; time_mac.cc does not exist).
#![cfg(target_vendor = "apple")]
#![allow(non_snake_case)]
use super::time::{Clock, Microseconds, Seconds, ThreadTicks, Time, TimeDelta, TimeTicks};
use core::{cell::UnsafeCell, mem::MaybeUninit};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mach_timebase_info_data_t {
    pub numer: u32,
    pub denom: u32,
}
unsafe extern "C" {
    fn mach_timebase_info(info: *mut mach_timebase_info_data_t) -> libc::c_int;
    fn mach_absolute_time() -> u64;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFAbsoluteTimeGetCurrent() -> f64;
    static kCFAbsoluteTimeIntervalSince1970: f64;
}
macro_rules! check {($function:expr,$line:expr,$cpp:expr,$cond:expr)=>{crate::CHECK!(@location crate::FROM_HERE!(@function $function.as_ptr().cast();@file b"base/time/time_apple.mm\0".as_ptr().cast();@line $line);@condition $cpp.as_ptr().cast();$cond)}}
struct TimebaseSlot(UnsafeCell<MaybeUninit<mach_timebase_info_data_t>>);
unsafe impl Sync for TimebaseSlot {}
static TIMEBASE: TimebaseSlot = TimebaseSlot(UnsafeCell::new(MaybeUninit::uninit()));
static INIT: std::sync::Once = std::sync::Once::new();
fn MachTimebaseInfo() -> *mut mach_timebase_info_data_t {
    INIT.call_once_force(|_|unsafe{
 let mut info=MaybeUninit::<mach_timebase_info_data_t>::uninit();let kr=mach_timebase_info(info.as_mut_ptr());
 // Actual streamed MACH_DCHECK producer is still a direct dependency, not a CHECK substitute.
 crate::MACH_DCHECK!(@location crate::FROM_HERE!(@function b"MachTimebaseInfo\0".as_ptr().cast();@file b"base/time/time_apple.mm\0".as_ptr().cast();@line 32);@condition b"kr == KERN_SUCCESS\0".as_ptr().cast();kr==0,kr;@stream b"mach_timebase_info\0".as_ptr().cast());
 let info=info.assume_init();
 crate::DCHECK!(@location crate::FROM_HERE!(@function b"MachTimebaseInfo\0".as_ptr().cast();@file b"base/time/time_apple.mm\0".as_ptr().cast();@line 33);@condition b"info.numer\0".as_ptr().cast();info.numer!=0);
 crate::DCHECK!(@location crate::FROM_HERE!(@function b"MachTimebaseInfo\0".as_ptr().cast();@file b"base/time/time_apple.mm\0".as_ptr().cast();@line 34);@condition b"info.denom\0".as_ptr().cast();info.denom!=0);
 (*TIMEBASE.0.get()).write(info);
 });
    TIMEBASE.0.get().cast()
}
fn MachTimeToMicroseconds(mach_time: u64) -> i64 {
    let info = unsafe { &*MachTimebaseInfo() };
    if info.numer == info.denom {
        return (mach_time / 1000) as i64;
    }
    let divisor = info.denom as u64 * 1000;
    let mut micros = mach_time / divisor;
    let remainder = mach_time % divisor;
    let (product, overflow) = micros.overflowing_mul(info.numer as u64);
    micros = product;
    check!(
        b"MachTimeToMicroseconds\0",
        64,
        b"!__builtin_umulll_overflow(microseconds, timebase_info->numer, &microseconds)\0",
        !overflow
    );
    let least = remainder.wrapping_mul(info.numer as u64) / divisor;
    let (sum, overflow) = micros.overflowing_add(least);
    micros = sum;
    check!(b"MachTimeToMicroseconds\0",71,b"!__builtin_uaddll_overflow(microseconds, least_significant_microseconds, &microseconds)\0",!overflow);
    if micros <= i64::MAX as u64 {
        micros as i64
    } else {
        crate::base::numerics::safe_conversions_impl::CheckOnFailure::HandleFailure()
    }
}
fn ComputeCurrentTicks() -> i64 {
    MachTimeToMicroseconds(unsafe { mach_absolute_time() })
}
fn ComputeThreadTicks() -> i64 {
    let mut ts: libc::timespec = unsafe { core::mem::zeroed() };
    check!(
        b"ComputeThreadTicks\0",
        92,
        b"clock_gettime(CLOCK_THREAD_CPUTIME_ID, &ts) == 0\0",
        unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) == 0 }
    );
    let us = (ts.tv_sec as i64)
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(ts.tv_nsec as i64 / 1000));
    match us {
        Some(v) => v,
        None => crate::base::numerics::safe_conversions_impl::CheckOnFailure::HandleFailure(),
    }
}
pub fn TimeNowIgnoringOverride() -> Time {
    Time::FromCFAbsoluteTime(unsafe { CFAbsoluteTimeGetCurrent() })
}
pub fn TimeNowFromSystemTimeIgnoringOverride() -> Time {
    TimeNowIgnoringOverride()
}
impl Time {
    pub fn FromCFAbsoluteTime(t: f64) -> Self {
        if t == 0.0 {
            Self::new()
        } else if t == f64::INFINITY {
            Self::Max()
        } else {
            Self::UnixEpoch() + Seconds(t + unsafe { kCFAbsoluteTimeIntervalSince1970 })
        }
    }
    pub fn ToCFAbsoluteTime(self) -> f64 {
        if self.is_null() {
            0.0
        } else if self.is_max() {
            f64::INFINITY
        } else {
            (self - Self::UnixEpoch()).InSecondsF() - unsafe { kCFAbsoluteTimeIntervalSince1970 }
        }
    }
}
// NSDate construction/message dispatch is not replaced by a Rust object or CF pointer cast.
impl TimeDelta {
    pub fn FromMachTime(t: u64) -> Self {
        Microseconds(MachTimeToMicroseconds(t))
    }
}
pub fn TimeTicksNowIgnoringOverride() -> TimeTicks {
    TimeTicks::new() + Microseconds(ComputeCurrentTicks())
}
pub fn TimeTicksLowResolutionNowIgnoringOverride() -> TimeTicks {
    TimeTicks::new()
        + Microseconds(MachTimeToMicroseconds(unsafe {
            libc::clock_gettime_nsec_np(libc::CLOCK_MONOTONIC_RAW_APPROX)
        }))
}
impl TimeTicks {
    pub fn IsHighResolution() -> bool {
        true
    }
    pub fn IsConsistentAcrossProcesses() -> bool {
        true
    }
    pub fn FromMachAbsoluteTime(t: u64) -> Self {
        Self {
            us_: MachTimeToMicroseconds(t),
        }
    }
    /// Safety: native testing mutation must not race any timebase read.
    pub unsafe fn SetMachTimebaseInfoForTesting(
        timebase: mach_timebase_info_data_t,
    ) -> mach_timebase_info_data_t {
        unsafe {
            let p = MachTimebaseInfo();
            let old = *p;
            *p = timebase;
            old
        }
    }
    pub fn GetClock() -> Clock {
        Clock::MAC_MACH_ABSOLUTE_TIME
    }
}
pub fn ThreadTicksNowIgnoringOverride() -> ThreadTicks {
    ThreadTicks::new() + Microseconds(ComputeThreadTicks())
}
