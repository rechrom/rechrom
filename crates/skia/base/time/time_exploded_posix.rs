// Source: base/time/time_exploded_posix.cc. libc calendar conversion is serialized
// by the original single, never-destructed base::Lock, initialized in final storage.
#![cfg(all(
    unix,
    not(all(target_os = "android", target_pointer_width = "32")),
    not(target_os = "aix")
))]
#![allow(non_snake_case)]
use super::time::{Exploded, Time};
use crate::base::synchronization::lock::{AutoLock, Lock};
use core::{cell::UnsafeCell, mem::MaybeUninit, pin::Pin};
struct LockSlot(UnsafeCell<MaybeUninit<Lock>>);
unsafe impl Sync for LockSlot {}
static LOCK: LockSlot = LockSlot(UnsafeCell::new(MaybeUninit::uninit()));
static INIT: std::sync::Once = std::sync::Once::new();
fn GetSysTimeToTimeStructLock() -> Pin<&'static Lock> {
    INIT.call_once_force(|_| unsafe {
        Lock::new(Pin::new_unchecked(&mut *LOCK.0.get()));
    });
    unsafe { Pin::new_unchecked(&*LOCK.0.get().cast::<Lock>()) }
}
unsafe fn SysTimeFromTimeStruct(tm: *mut libc::tm, local: bool) -> libc::time_t {
    let _locked = unsafe { AutoLock::new(GetSysTimeToTimeStructLock()) };
    unsafe {
        if local {
            libc::mktime(tm)
        } else {
            libc::timegm(tm)
        }
    }
}
unsafe fn SysTimeToTimeStruct(t: libc::time_t, tm: *mut libc::tm, local: bool) {
    let _locked = unsafe { AutoLock::new(GetSysTimeToTimeStructLock()) };
    unsafe {
        if local {
            libc::localtime_r(&t, tm);
        } else {
            libc::gmtime_r(&t, tm);
        }
    }
}
impl Time {
    // Native ignores libc's null return and reads its output; callers retain the
    // original successful-calendar-conversion precondition rather than a fake value.
    pub unsafe fn Explode(self, local: bool, out: *mut Exploded) {
        let millis = self.ToRoundedDownMillisecondsSinceUnixEpoch();
        if core::mem::size_of::<libc::time_t>() < 8 {
            #[cfg(all(not(base_is_castos), not(base_is_cast_android)))]
            {
                unsafe { Self::ExplodeUsingIcu(millis, local, out) };
                return;
            }
        }
        let rawseconds = millis / 1000;
        let mut seconds: libc::time_t =
            if rawseconds >= libc::time_t::MIN as i64 && rawseconds <= libc::time_t::MAX as i64 {
                rawseconds as libc::time_t
            } else {
                crate::base::numerics::safe_conversions_impl::CheckOnFailure::HandleFailure()
            };
        let mut ms = millis % 1000;
        if ms < 0 {
            seconds -= 1;
            ms += 1000
        }
        let mut tm = MaybeUninit::<libc::tm>::uninit();
        unsafe { SysTimeToTimeStruct(seconds, tm.as_mut_ptr(), local) }
        let tm = unsafe { tm.assume_init() };
        unsafe {
            out.write(Exploded {
                year: tm.tm_year + 1900,
                month: tm.tm_mon + 1,
                day_of_week: tm.tm_wday,
                day_of_month: tm.tm_mday,
                hour: tm.tm_hour,
                minute: tm.tm_min,
                second: tm.tm_sec,
                millisecond: ms as i32,
            });
        }
    }
    pub unsafe fn FromExploded(local: bool, e: &Exploded, out: &mut Self) -> bool {
        let Some(month) = e.month.checked_sub(1) else {
            *out = Self::new();
            return false;
        };
        let Some(year) = e.year.checked_sub(1900) else {
            *out = Self::new();
            return false;
        };
        // Each native tm field is initialized explicitly; padding is not read.
        let mut storage = MaybeUninit::<libc::tm>::uninit();
        let p = storage.as_mut_ptr();
        unsafe {
            core::ptr::addr_of_mut!((*p).tm_sec).write(e.second);
            core::ptr::addr_of_mut!((*p).tm_min).write(e.minute);
            core::ptr::addr_of_mut!((*p).tm_hour).write(e.hour);
            core::ptr::addr_of_mut!((*p).tm_mday).write(e.day_of_month);
            core::ptr::addr_of_mut!((*p).tm_mon).write(month);
            core::ptr::addr_of_mut!((*p).tm_year).write(year);
            core::ptr::addr_of_mut!((*p).tm_wday).write(e.day_of_week);
            core::ptr::addr_of_mut!((*p).tm_yday).write(0);
            core::ptr::addr_of_mut!((*p).tm_isdst).write(-1);
            #[cfg(not(target_os = "solaris"))]
            {
                core::ptr::addr_of_mut!((*p).tm_gmtoff).write(0);
                core::ptr::addr_of_mut!((*p).tm_zone).write(core::ptr::null());
            }
        }
        let mut tm = unsafe { storage.assume_init() };
        let tm0 = tm;
        let mut seconds = unsafe { SysTimeFromTimeStruct(&mut tm, local) } as i64;
        if seconds == -1 {
            tm = tm0;
            tm.tm_isdst = 0;
            let s0 = unsafe { SysTimeFromTimeStruct(&mut tm, local) } as i64;
            tm = tm0;
            tm.tm_isdst = 1;
            let s1 = unsafe { SysTimeFromTimeStruct(&mut tm, local) } as i64;
            seconds = if s0 < 0 {
                s1
            } else if s1 < 0 {
                s0
            } else {
                s0.min(s1)
            }
        }
        let millis = if seconds == -1 && (e.year < 1969 || e.year > 1970) {
            let min = if core::mem::size_of::<libc::time_t>() < 8 {
                libc::time_t::MIN as i64
            } else {
                i32::MIN as i64
            };
            let max = if core::mem::size_of::<libc::time_t>() < 8 {
                libc::time_t::MAX as i64
            } else {
                i32::MAX as i64
            };
            if e.year < 1969 {
                min * 1000
            } else {
                max * 1000 + 999
            }
        } else {
            let Some(ms) = seconds
                .checked_mul(1000)
                .and_then(|v| v.checked_add(e.millisecond as i64))
            else {
                *out = Self::new();
                return false;
            };
            ms
        };
        let mut converted = Self::new();
        if !Self::FromMillisecondsSinceUnixEpochChecked(millis, &mut converted) {
            *out = Self::new();
            return false;
        }
        let mut exploded = MaybeUninit::<Exploded>::uninit();
        unsafe { converted.Explode(local, exploded.as_mut_ptr()) }
        let exploded = unsafe { exploded.assume_init() };
        if Self::ExplodedMostlyEquals(&exploded, e) {
            *out = converted;
            true
        } else {
            *out = Self::new();
            false
        }
    }
    pub unsafe fn UTCExplode(self, out: *mut Exploded) {
        unsafe { self.Explode(false, out) }
    }
    pub unsafe fn LocalExplode(self, out: *mut Exploded) {
        unsafe { self.Explode(true, out) }
    }
    pub unsafe fn FromUTCExploded(e: &Exploded, out: &mut Self) -> bool {
        unsafe { Self::FromExploded(false, e, out) }
    }
    pub unsafe fn FromLocalExploded(e: &Exploded, out: &mut Self) -> bool {
        unsafe { Self::FromExploded(true, e, out) }
    }
    pub unsafe fn Midnight(self, local: bool) -> Self {
        let mut e = MaybeUninit::<Exploded>::uninit();
        unsafe { self.Explode(local, e.as_mut_ptr()) }
        let mut e = unsafe { e.assume_init() };
        e.hour = 0;
        e.minute = 0;
        e.second = 0;
        e.millisecond = 0;
        let mut out = Self::new();
        if unsafe { Self::FromExploded(local, &e, &mut out) } {
            return out;
        }
        crate::DCHECK!(@location crate::FROM_HERE!(@function b"Midnight\0".as_ptr().cast();@file b"base/time/time.cc\0".as_ptr().cast();@line 118);@condition b"is_local\0".as_ptr().cast();local);
        e.hour = 1;
        let result = unsafe { Self::FromExploded(local, &e, &mut out) };
        #[cfg(not(all(base_is_chromeos, any(target_arch = "arm", target_arch = "aarch64"))))]
        crate::DCHECK!(@location crate::FROM_HERE!(@function b"Midnight\0".as_ptr().cast();@file b"base/time/time.cc\0".as_ptr().cast();@line 129);@condition b"result\0".as_ptr().cast();result);
        out
    }
    pub unsafe fn UTCMidnight(self) -> Self {
        unsafe { self.Midnight(false) }
    }
    pub unsafe fn LocalMidnight(self) -> Self {
        unsafe { self.Midnight(true) }
    }
}
