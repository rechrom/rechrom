// Source: base/time/time_conversion_posix.cc:19-99.
#![cfg(unix)]
#![allow(non_snake_case)]
use super::time::{Microseconds, Nanoseconds, Seconds, Time, TimeDelta};
impl TimeDelta {
    pub fn FromTimeSpec(ts: &libc::timespec) -> Self {
        let d = Seconds(ts.tv_sec) + Nanoseconds(ts.tv_nsec);
        if d.is_positive() {
            d
        } else {
            Self::new()
        }
    }
    pub fn ToTimeSpec(self) -> libc::timespec {
        if self.is_negative() {
            return libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
        }
        let extra = self.InMicroseconds() % 1_000_000;
        libc::timespec {
            tv_sec: self
                .InSeconds()
                .clamp(libc::time_t::MIN as i64, libc::time_t::MAX as i64)
                as libc::time_t,
            tv_nsec: (extra * 1000).clamp(libc::c_long::MIN as i64, libc::c_long::MAX as i64)
                as libc::c_long,
        }
    }
}
impl Time {
    pub fn FromTimeVal(t: libc::timeval) -> Self {
        crate::DCHECK_LT!(@location crate::FROM_HERE!(@function b"FromTimeVal\0".as_ptr().cast();@file b"base/time/time_conversion_posix.cc\0".as_ptr().cast();@line 49);@condition b"t.tv_usec < static_cast<int>(kMicrosecondsPerSecond)\0".as_ptr().cast();t.tv_usec,1_000_000);
        crate::DCHECK_GE!(@location crate::FROM_HERE!(@function b"FromTimeVal\0".as_ptr().cast();@file b"base/time/time_conversion_posix.cc\0".as_ptr().cast();@line 50);@condition b"t.tv_usec >= 0\0".as_ptr().cast();t.tv_usec,0);
        if t.tv_usec == 999999 && t.tv_sec == libc::time_t::MAX {
            Self::Max()
        } else {
            Self::UnixEpoch() + Seconds(t.tv_sec) + Microseconds(t.tv_usec)
        }
    }
    /// Safety: native signed subtraction requires this nonnull time's raw Windows
    /// microseconds minus the epoch offset to be representable in int64_t.
    pub unsafe fn ToTimeVal(self) -> libc::timeval {
        if self.is_null() {
            return libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
        }
        let us = unsafe {
            self.us_
                .unchecked_sub(Self::kMicrosecondsFromWindowsToUnixEpoch)
        };
        libc::timeval {
            tv_sec: if self.is_max() {
                libc::time_t::MAX
            } else {
                (us / 1_000_000).clamp(libc::time_t::MIN as i64, libc::time_t::MAX as i64)
                    as libc::time_t
            },
            tv_usec: if self.is_max() {
                999999
            } else {
                (us % 1_000_000).clamp(libc::suseconds_t::MIN as i64, libc::suseconds_t::MAX as i64)
                    as libc::suseconds_t
            },
        }
    }
}
