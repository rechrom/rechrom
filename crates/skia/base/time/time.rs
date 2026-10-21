// Source: base/time/time.h; base/time/time.cc. Native microsecond representation.
#![allow(non_snake_case, non_upper_case_globals)]
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};
// Source: base/time/time.h:275-280,892-931. The primitive language adapters
// invoke the actual selected ClampedNumeric operators, including mixed unsigned
// promotion, compiler overflow writes, floating arithmetic and final saturation.
use crate::base::numerics::checked_math::CheckedNumeric;
use crate::base::numerics::clamped_math::{ClampMul, ClampedNumeric};
use crate::base::numerics::safe_conversions::{saturated_cast, strict_cast};
pub trait TimeNumber: Copy {
    fn is_zero(self) -> bool;
    fn unit(self, mul: i64, div: i64) -> i64;
    fn multiply(self, value: i64) -> i64;
    fn divide(self, value: i64) -> i64;
    fn hertz_period(self) -> i64;
    fn is_nan(self) -> bool {
        false
    }
}
macro_rules! native_number{($($t:ty),*)=>{$(impl TimeNumber for $t{
 fn is_zero(self)->bool{self==0 as $t}
 fn unit(self,mul:i64,div:i64)->i64{
  // Actual factory expressions have either one multiply, one divide, or
  // only the conversion. Do not introduce an intermediate int64 saturation.
  if div!=1{(ClampedNumeric::<$t>::new(self)/div).Into::<i64>()}
  else if mul!=1{(ClampedNumeric::<$t>::new(self)*mul).Into::<i64>()}
  else{saturated_cast::<i64,_>(self)}
 }
 fn multiply(self,value:i64)->i64{(ClampedNumeric::<i64>::new(value)*self).Into::<i64>()}
 fn divide(self,value:i64)->i64{(ClampedNumeric::<i64>::new(value)/self).Into::<i64>()}
 fn hertz_period(self)->i64{(1_000_000i64/ClampedNumeric::<$t>::new(self)).Into::<i64>()}
})*}}
native_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
macro_rules! native_float{($($t:ty),*)=>{$(impl TimeNumber for $t{
 fn is_zero(self)->bool{self==0.0}
 fn unit(self,mul:i64,div:i64)->i64{if div!=1{(ClampedNumeric::<$t>::new(self)/div).Into::<i64>()}else if mul!=1{(ClampedNumeric::<$t>::new(self)*mul).Into::<i64>()}else{saturated_cast::<i64,_>(self)}}
 fn multiply(self,value:i64)->i64{(ClampedNumeric::<i64>::new(value)*self).Into::<i64>()}
 fn divide(self,value:i64)->i64{(ClampedNumeric::<i64>::new(value)/self).Into::<i64>()}
 fn hertz_period(self)->i64{(1_000_000i64/ClampedNumeric::<$t>::new(self)).Into::<i64>()}
 fn is_nan(self)->bool{self.is_nan()}
})*}}
native_float!(f32, f64);
#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta {
    delta_: i64,
}
impl TimeDelta {
    pub const fn new() -> Self {
        Self { delta_: 0 }
    }
    pub const fn Max() -> Self {
        Self { delta_: i64::MAX }
    }
    pub const fn Min() -> Self {
        Self { delta_: i64::MIN }
    }
    pub const fn FiniteMax() -> Self {
        Self {
            delta_: i64::MAX - 1,
        }
    }
    pub const fn FiniteMin() -> Self {
        Self {
            delta_: i64::MIN + 1,
        }
    }
    pub const fn is_zero(self) -> bool {
        self.delta_ == 0
    }
    pub const fn is_positive(self) -> bool {
        self.delta_ > 0
    }
    pub const fn is_negative(self) -> bool {
        self.delta_ < 0
    }
    pub const fn is_max(self) -> bool {
        self.delta_ == i64::MAX
    }
    pub const fn is_min(self) -> bool {
        self.delta_ == i64::MIN
    }
    pub const fn is_inf(self) -> bool {
        self.is_max() || self.is_min()
    }
    pub const fn magnitude(self) -> Self {
        Self {
            delta_: ClampedNumeric::<i64>::const_new(self.delta_)
                .const_abs()
                .RawValue(),
        }
    }
    pub const fn InMicroseconds(self) -> i64 {
        self.delta_
    }
    pub fn InMicrosecondsF(self) -> f64 {
        self.ToDouble()
    }
    pub fn ToDouble(self) -> f64 {
        if self.is_max() {
            f64::INFINITY
        } else if self.is_min() {
            f64::NEG_INFINITY
        } else {
            saturated_cast::<f64, _>(self.delta_)
        }
    }
    pub fn InDays(self) -> i32 {
        if self.is_inf() {
            if self.is_negative() {
                i32::MIN
            } else {
                i32::MAX
            }
        } else {
            (self.delta_ / 86_400_000_000) as i32
        }
    }
    pub fn InDaysFloored(self) -> i32 {
        if self.is_inf() {
            self.InDays()
        } else {
            let q = self.delta_ / 86_400_000_000;
            (q - i64::from(q * 86_400_000_000 > self.delta_)) as i32
        }
    }
    pub fn InHours(self) -> i32 {
        saturated_cast::<i32, _>(self.delta_ / 3_600_000_000)
    }
    pub fn InMinutes(self) -> i32 {
        saturated_cast::<i32, _>(self.delta_ / 60_000_000)
    }
    pub fn InSeconds(self) -> i64 {
        if self.is_inf() {
            self.delta_
        } else {
            self.delta_ / 1_000_000
        }
    }
    pub fn InSecondsFloored(self) -> i64 {
        if self.is_inf() {
            self.delta_
        } else {
            let q = self.delta_ / 1_000_000;
            q - i64::from(q * 1_000_000 > self.delta_)
        }
    }
    pub fn InSecondsF(self) -> f64 {
        self.ToDouble() / 1_000_000.0
    }
    pub fn InMilliseconds(self) -> i64 {
        if self.is_inf() {
            self.delta_
        } else {
            self.delta_ / 1000
        }
    }
    pub fn InMillisecondsF(self) -> f64 {
        self.ToDouble() / 1000.0
    }
    pub fn InMillisecondsRoundedUp(self) -> i64 {
        if self.is_inf() {
            self.delta_
        } else {
            let q = self.delta_ / 1000;
            q + i64::from(self.delta_ > q * 1000)
        }
    }
    pub fn InNanoseconds(self) -> i64 {
        ClampMul(self.delta_, 1000i64).RawValue()
    }
    pub fn ToHz(self) -> f64 {
        Seconds(1) / self
    }
    pub fn IntDiv(self, a: Self) -> i64 {
        if !self.is_inf() && !a.is_zero() {
            return (ClampedNumeric::<i64>::new(self.delta_) / a.delta_).Into::<i64>();
        }
        crate::CHECK!(@location crate::FROM_HERE!(@function b"IntDiv\0".as_ptr().cast();@file b"base/time/time.h\0".as_ptr().cast();@line 312); @condition b"!is_zero() || !a.is_zero()\0".as_ptr().cast(); !self.is_zero()||!a.is_zero());
        crate::CHECK!(@location crate::FROM_HERE!(@function b"IntDiv\0".as_ptr().cast();@file b"base/time/time.h\0".as_ptr().cast();@line 313); @condition b"!is_inf() || !a.is_inf()\0".as_ptr().cast(); !self.is_inf()||!a.is_inf());
        if self.is_negative() == a.is_negative() {
            i64::MAX
        } else {
            i64::MIN
        }
    }
    pub fn CeilToMultiple(self, i: Self) -> Self {
        if self.is_inf() || i.is_zero() {
            return self;
        }
        let r = self % i;
        if self.is_negative() {
            self - r
        } else if r.is_zero() {
            self
        } else {
            self - r + i.magnitude()
        }
    }
    pub fn FloorToMultiple(self, i: Self) -> Self {
        if self.is_inf() || i.is_zero() {
            return self;
        }
        let r = self % i;
        if self.is_negative() {
            if r.is_zero() {
                self
            } else {
                self - r - i.magnitude()
            }
        } else {
            self - r
        }
    }
    pub fn RoundToMultiple(self, i: Self) -> Self {
        if self.is_inf() || i.is_zero() {
            return self;
        }
        if i.is_inf() {
            return Self::new();
        }
        let half = i.magnitude() / 2;
        if self.is_negative() {
            (self - half).CeilToMultiple(i)
        } else {
            (self + half).FloorToMultiple(i)
        }
    }
}
impl Neg for TimeDelta {
    type Output = Self;
    fn neg(self) -> Self {
        if self.is_min() {
            Self::Max()
        } else if self.is_max() {
            Self::Min()
        } else {
            Self {
                delta_: -self.delta_,
            }
        }
    }
}
impl Add for TimeDelta {
    type Output = Self;
    fn add(self, a: Self) -> Self {
        if !a.is_inf() {
            Self {
                delta_: (ClampedNumeric::<i64>::new(self.delta_) + a.delta_).RawValue(),
            }
        } else {
            crate::CHECK!(@location crate::FROM_HERE!(@function b"operator+\0".as_ptr().cast();@file b"base/time/time.h\0".as_ptr().cast();@line 366);@condition b"!is_inf() || (delta_ == other.delta_)\0".as_ptr().cast();!self.is_inf()||self.delta_==a.delta_);
            a
        }
    }
}
impl Sub for TimeDelta {
    type Output = Self;
    fn sub(self, a: Self) -> Self {
        if !a.is_inf() {
            Self {
                delta_: (ClampedNumeric::<i64>::new(self.delta_) - a.delta_).RawValue(),
            }
        } else {
            crate::CHECK_NE!(@location crate::FROM_HERE!(@function b"operator-\0".as_ptr().cast();@file b"base/time/time.h\0".as_ptr().cast();@line 376);@condition b"int64_t{delta_} != int64_t{other.delta_}\0".as_ptr().cast();self.delta_,a.delta_);
            if a.is_negative() {
                Self::Max()
            } else {
                Self::Min()
            }
        }
    }
}
impl<T: TimeNumber> Mul<T> for TimeDelta {
    type Output = Self;
    fn mul(self, a: T) -> Self {
        Self {
            delta_: a.multiply(self.delta_),
        }
    }
}
impl<T: TimeNumber> Div<T> for TimeDelta {
    type Output = Self;
    fn div(self, a: T) -> Self {
        Self {
            delta_: a.divide(self.delta_),
        }
    }
}
impl Div for TimeDelta {
    type Output = f64;
    fn div(self, a: Self) -> f64 {
        crate::CHECK!(@location crate::FROM_HERE!(@function b"operator/\0".as_ptr().cast();@file b"base/time/time.h\0".as_ptr().cast();@line 300);@condition b"!is_zero() || !a.is_zero()\0".as_ptr().cast();!self.is_zero()||!a.is_zero());
        crate::CHECK!(@location crate::FROM_HERE!(@function b"operator/\0".as_ptr().cast();@file b"base/time/time.h\0".as_ptr().cast();@line 301);@condition b"!is_inf() || !a.is_inf()\0".as_ptr().cast();!self.is_inf()||!a.is_inf());
        self.ToDouble() / a.ToDouble()
    }
}
impl Rem for TimeDelta {
    type Output = Self;
    fn rem(self, a: Self) -> Self {
        if self.is_inf() || a.is_zero() || a.is_inf() {
            self
        } else {
            Self {
                delta_: (ClampedNumeric::<i64>::new(self.delta_) % a.delta_).RawValue(),
            }
        }
    }
}
impl AddAssign for TimeDelta {
    fn add_assign(&mut self, a: Self) {
        *self = *self + a
    }
}
impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, a: Self) {
        *self = *self - a
    }
}
impl<T: TimeNumber> MulAssign<T> for TimeDelta {
    fn mul_assign(&mut self, a: T) {
        *self = *self * a
    }
}
impl<T: TimeNumber> DivAssign<T> for TimeDelta {
    fn div_assign(&mut self, a: T) {
        *self = *self / a
    }
}
macro_rules! factory {
    ($name:ident,$m:expr,$d:expr) => {
        pub fn $name<T: TimeNumber>(n: T) -> TimeDelta {
            TimeDelta {
                delta_: n.unit($m, $d),
            }
        }
    };
}
factory!(Days, 86_400_000_000, 1);
factory!(Hours, 3_600_000_000, 1);
factory!(Minutes, 60_000_000, 1);
factory!(Seconds, 1_000_000, 1);
factory!(Milliseconds, 1000, 1);
factory!(Microseconds, 1, 1);
factory!(Nanoseconds, 1, 1000);
pub fn Hertz<T: TimeNumber>(n: T) -> TimeDelta {
    if n.is_zero() {
        TimeDelta::Max()
    } else {
        TimeDelta {
            delta_: n.hertz_period(),
        }
    }
}
macro_rules! time_base {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            pub(crate) us_: i64,
        }
        impl $name {
            pub const fn new() -> Self {
                Self { us_: 0 }
            }
            pub const fn Max() -> Self {
                Self { us_: i64::MAX }
            }
            pub const fn Min() -> Self {
                Self { us_: i64::MIN }
            }
            pub const fn is_null(self) -> bool {
                self.us_ == 0
            }
            pub const fn is_max(self) -> bool {
                self.us_ == i64::MAX
            }
            pub const fn is_min(self) -> bool {
                self.us_ == i64::MIN
            }
            pub const fn is_inf(self) -> bool {
                self.is_max() || self.is_min()
            }
            pub const fn since_origin(self) -> TimeDelta {
                TimeDelta { delta_: self.us_ }
            }
            pub const fn ToInternalValue(self) -> i64 {
                self.us_
            }
            pub fn SnappedToNextTick(self, phase: Self, interval: TimeDelta) -> Self {
                let mut off = (phase - self) % interval;
                if !off.is_zero() && phase < self {
                    off += interval
                }
                self + off
            }
            pub const kHoursPerDay: i64 = 24;
            pub const kSecondsPerMinute: i64 = 60;
            pub const kMinutesPerHour: i64 = 60;
            pub const kSecondsPerHour: i64 = 3600;
            pub const kMillisecondsPerSecond: i64 = 1000;
            pub const kMillisecondsPerDay: i64 = 86_400_000;
            pub const kMicrosecondsPerMillisecond: i64 = 1000;
            pub const kMicrosecondsPerSecond: i64 = 1_000_000;
            pub const kMicrosecondsPerMinute: i64 = 60_000_000;
            pub const kMicrosecondsPerHour: i64 = 3_600_000_000;
            pub const kMicrosecondsPerDay: i64 = 86_400_000_000;
            pub const kMicrosecondsPerWeek: i64 = 604_800_000_000;
            pub const kNanosecondsPerMicrosecond: i64 = 1000;
            pub const kNanosecondsPerMillisecond: i64 = 1_000_000;
            pub const kNanosecondsPerSecond: i64 = 1_000_000_000;
        }
        impl Add<TimeDelta> for $name {
            type Output = Self;
            fn add(self, d: TimeDelta) -> Self {
                Self {
                    us_: (TimeDelta { delta_: self.us_ } + d).delta_,
                }
            }
        }
        impl Sub<TimeDelta> for $name {
            type Output = Self;
            fn sub(self, d: TimeDelta) -> Self {
                Self {
                    us_: (TimeDelta { delta_: self.us_ } - d).delta_,
                }
            }
        }
        impl Sub for $name {
            type Output = TimeDelta;
            fn sub(self, a: Self) -> TimeDelta {
                TimeDelta {
                    delta_: (ClampedNumeric::<i64>::new(self.us_) - a.us_).RawValue(),
                }
            }
        }
        impl AddAssign<TimeDelta> for $name {
            fn add_assign(&mut self, a: TimeDelta) {
                *self = *self + a
            }
        }
        impl SubAssign<TimeDelta> for $name {
            fn sub_assign(&mut self, a: TimeDelta) {
                *self = *self - a
            }
        }
    };
}
time_base!(Time);
time_base!(TimeTicks);
time_base!(LiveTicks);
time_base!(ThreadTicks);
#[derive(Clone, Copy, Debug)]
pub struct Exploded {
    pub year: i32,
    pub month: i32,
    pub day_of_week: i32,
    pub day_of_month: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub millisecond: i32,
}
impl Exploded {
    pub fn HasValidValues(&self) -> bool {
        (1..=12).contains(&self.month)
            && (0..=6).contains(&self.day_of_week)
            && (1..=31).contains(&self.day_of_month)
            && (0..=23).contains(&self.hour)
            && (0..=59).contains(&self.minute)
            && (0..=60).contains(&self.second)
            && (0..=999).contains(&self.millisecond)
    }
}
impl Time {
    pub const kMicrosecondsFromWindowsToUnixEpoch: i64 = 11_644_473_600_000_000;
    pub const fn UnixEpoch() -> Self {
        Self {
            us_: Self::kMicrosecondsFromWindowsToUnixEpoch,
        }
    }
    pub const fn FromDeltaSinceWindowsEpoch(d: TimeDelta) -> Self {
        Self { us_: d.delta_ }
    }
    pub const fn ToDeltaSinceWindowsEpoch(self) -> TimeDelta {
        self.since_origin()
    }
    pub const fn FromInternalValue(us: i64) -> Self {
        Self { us_: us }
    }
    pub fn FromTimeT(t: libc::time_t) -> Self {
        if t == 0 {
            Self::new()
        } else if t == libc::time_t::MAX {
            Self::Max()
        } else {
            Self::UnixEpoch() + Seconds(t)
        }
    }
    pub fn ToTimeT(self) -> libc::time_t {
        if self.is_null() {
            0
        } else if !self.is_inf() {
            saturated_cast::<libc::time_t, _>((self - Self::UnixEpoch()).InSecondsFloored())
        } else if self.is_min() {
            libc::time_t::MIN
        } else {
            libc::time_t::MAX
        }
    }
    pub fn FromSecondsSinceUnixEpoch(t: f64) -> Self {
        if t == 0.0 || t.is_nan() {
            Self::new()
        } else {
            Self::UnixEpoch() + Seconds(t)
        }
    }
    pub fn InSecondsFSinceUnixEpoch(self) -> f64 {
        if self.is_null() {
            0.0
        } else if !self.is_inf() {
            (self - Self::UnixEpoch()).InSecondsF()
        } else if self.is_min() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    }
    pub fn FromTimeSpec(ts: &libc::timespec) -> Self {
        Self::FromSecondsSinceUnixEpoch(ts.tv_sec as f64 + ts.tv_nsec as f64 / 1_000_000_000.0)
    }
    pub fn FromMillisecondsSinceUnixEpoch<T: MillisecondsSinceUnixNumber>(t: T) -> Self {
        t.into_time()
    }
    pub fn FromMillisecondsSinceUnixEpochChecked(ms: i64, out: &mut Self) -> bool {
        let mut checked = CheckedNumeric::<i64>::new(ms);
        checked *= 1000i64;
        checked += Self::kMicrosecondsFromWindowsToUnixEpoch;
        out.us_ = checked.ValueOrDefault(0i32).RawValue();
        checked.IsValid()
    }
    pub fn InMillisecondsSinceUnixEpoch(self) -> i64 {
        if self.is_null() {
            0
        } else if self.is_inf() {
            self.us_
        } else {
            (self - Self::UnixEpoch()).InMilliseconds()
        }
    }
    pub fn InMillisecondsFSinceUnixEpoch(self) -> f64 {
        if self.is_null() {
            0.0
        } else {
            self.InMillisecondsFSinceUnixEpochIgnoringNull()
        }
    }
    pub fn InMillisecondsFSinceUnixEpochIgnoringNull(self) -> f64 {
        if self.is_inf() {
            if self.is_min() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        } else {
            (self - Self::UnixEpoch()).InMillisecondsF()
        }
    }
    pub fn ToRoundedDownMillisecondsSinceUnixEpoch(self) -> i64 {
        self.us_ / 1000
            - Self::kMicrosecondsFromWindowsToUnixEpoch / 1000
            - i64::from(self.us_ % 1000 < 0)
    }
    pub fn ExplodedMostlyEquals(a: &Exploded, b: &Exploded) -> bool {
        (
            a.year,
            a.month,
            a.day_of_month,
            a.hour,
            a.minute,
            a.second,
            a.millisecond,
        ) == (
            b.year,
            b.month,
            b.day_of_month,
            b.hour,
            b.minute,
            b.second,
            b.millisecond,
        )
    }
}
#[repr(i32)]
#[derive(Clone, Copy)]
pub enum Clock {
    FUCHSIA_ZX_CLOCK_MONOTONIC,
    LINUX_CLOCK_MONOTONIC,
    IOS_CF_ABSOLUTE_TIME_MINUS_KERN_BOOTTIME,
    MAC_MACH_ABSOLUTE_TIME,
    WIN_QPC,
    WIN_ROLLOVER_PROTECTED_TIME_GET_TIME,
}
impl TimeTicks {
    pub const fn FromInternalValue(us: i64) -> Self {
        Self { us_: us }
    }
}
// Source: time.h:165-167,197-197,1379-1388,1413-1415.
impl TimeDelta {
    pub const fn FromInternalValue(delta: i64) -> Self {
        Self { delta_: delta }
    }
    pub const fn ToInternalValue(self) -> i64 {
        self.delta_
    }
}
impl ThreadTicks {
    pub const fn FromInternalValue(us: i64) -> Self {
        Self { us_: us }
    }
    #[cfg(any(
        base_posix_thread_cputime,
        target_vendor = "apple",
        target_os = "android",
        target_os = "fuchsia"
    ))]
    pub const fn IsSupported() -> bool {
        true
    }
    #[cfg(all(
        not(windows),
        not(any(
            base_posix_thread_cputime,
            target_vendor = "apple",
            target_os = "android",
            target_os = "fuchsia"
        ))
    ))]
    pub const fn IsSupported() -> bool {
        false
    }
    #[cfg(not(windows))]
    pub fn WaitUntilInitialized() {}
}
// Source: time.h:380-383,520-523. Native arithmetic symmetry, no new timeline.
macro_rules! reverse_mul{($($t:ty),*)=>{$(impl Mul<TimeDelta> for $t{type Output=TimeDelta;fn mul(self,d:TimeDelta)->TimeDelta{d*self}})*}}
reverse_mul!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);
macro_rules! reverse_add{($($t:ty),*)=>{$(impl Add<$t> for TimeDelta{type Output=$t;fn add(self,t:$t)->$t{t+self}})*}}
reverse_add!(Time, TimeTicks, LiveTicks, ThreadTicks);
// Source: time.h:563-584. Only original platform branches select year limits.
impl Time {
    #[cfg(windows)]
    pub const kExplodedMinYear: i32 = 1601;
    #[cfg(windows)]
    pub const kExplodedMaxYear: i32 = 30827;
    #[cfg(all(target_os = "ios", target_pointer_width = "32"))]
    pub const kExplodedMinYear: i32 = i32::MIN;
    #[cfg(all(target_os = "ios", target_pointer_width = "32"))]
    pub const kExplodedMaxYear: i32 = i32::MAX;
    #[cfg(any(
        all(
            target_vendor = "apple",
            not(all(target_os = "ios", target_pointer_width = "32"))
        ),
        target_os = "android"
    ))]
    pub const kExplodedMinYear: i32 = 1902;
    #[cfg(any(
        all(
            target_vendor = "apple",
            not(all(target_os = "ios", target_pointer_width = "32"))
        ),
        target_os = "android"
    ))]
    pub const kExplodedMaxYear: i32 = i32::MAX;
    #[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
    pub const kExplodedMinYear: i32 = if core::mem::size_of::<libc::time_t>() == 4 {
        1902
    } else {
        i32::MIN
    };
    #[cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]
    pub const kExplodedMaxYear: i32 = if core::mem::size_of::<libc::time_t>() == 4 {
        2037
    } else {
        i32::MAX
    };
    // Source: time.cc:143-161. Native parser producer remains external, without invented parser.
    /// Safety: string is NUL terminated and nonnull; output follows native mutable ownership.
    pub unsafe fn FromStringInternal(
        string: *const libc::c_char,
        local: bool,
        out: &mut Self,
    ) -> bool {
        crate::DCHECK!(@location crate::FROM_HERE!(@function b"FromStringInternal\0".as_ptr().cast();@file b"base/time/time.cc\0".as_ptr().cast();@line 138);@condition b"time_string\0".as_ptr().cast();!string.is_null());
        if unsafe { *string } == 0 {
            return false;
        }
        let mut result_time = 0i64;
        let result = unsafe {
            crate::base::third_party::nspr::prtime::PR_ParseTimeString(
                string,
                if local { 0 } else { 1 },
                &mut result_time,
            )
        };
        if result != 0 {
            return false;
        }
        *out = Self::UnixEpoch() + Microseconds(result_time);
        true
    }
    pub unsafe fn FromString(string: *const libc::c_char, out: &mut Self) -> bool {
        unsafe { Self::FromStringInternal(string, true, out) }
    }
    pub unsafe fn FromUTCString(string: *const libc::c_char, out: &mut Self) -> bool {
        unsafe { Self::FromStringInternal(string, false, out) }
    }
}
// Source: time.h:696-702. C++ float arguments promote to the double overload;
// only the constrained integral domains select the int64_t overload.
pub trait MillisecondsSinceUnixNumber: Copy {
    fn into_time(self) -> Time;
}
macro_rules! millis_integer{($($t:ty),*)=>{$(impl MillisecondsSinceUnixNumber for $t{fn into_time(self)->Time{Time::UnixEpoch()+Milliseconds(strict_cast::<i64,_>(self))}})*}}
millis_integer!(i8, i16, i32, i64, isize, u8, u16, u32);
#[cfg(target_pointer_width = "32")]
impl MillisecondsSinceUnixNumber for usize {
    fn into_time(self) -> Time {
        Time::UnixEpoch() + Milliseconds(strict_cast::<i64, _>(self))
    }
}
impl MillisecondsSinceUnixNumber for f64 {
    fn into_time(self) -> Time {
        if self.is_nan() {
            Time::new()
        } else {
            Time::UnixEpoch() + Milliseconds(self)
        }
    }
}
impl MillisecondsSinceUnixNumber for f32 {
    fn into_time(self) -> Time {
        (self as f64).into_time()
    }
}

// Native bool promotes to int for arithmetic with the int64_t unit/value.
impl TimeNumber for bool {
    fn is_zero(self) -> bool {
        !self
    }
    fn unit(self, m: i64, d: i64) -> i64 {
        strict_cast::<i32, _>(self).unit(m, d)
    }
    fn multiply(self, v: i64) -> i64 {
        strict_cast::<i32, _>(self).multiply(v)
    }
    fn divide(self, v: i64) -> i64 {
        strict_cast::<i32, _>(self).divide(v)
    }
    fn hertz_period(self) -> i64 {
        strict_cast::<i32, _>(self).hertz_period()
    }
}
impl Mul<TimeDelta> for bool {
    type Output = TimeDelta;
    fn mul(self, d: TimeDelta) -> TimeDelta {
        d * self
    }
}
impl MillisecondsSinceUnixNumber for bool {
    fn into_time(self) -> Time {
        Time::UnixEpoch() + Milliseconds(strict_cast::<i64, _>(self))
    }
}

// Source: base/time/time.h:325-327. Native remainder assignment delegates once.
impl RemAssign for TimeDelta {
    fn rem_assign(&mut self, other: Self) {
        *self = *self % other
    }
}
// Source: base/time/time.h:914-918. Explicitly supported primitive CTFE domain;
// the ordinary generic factory remains runtime rather than a pretend const trait.
#[macro_export]
macro_rules! NativeMicrosecondsConst {
    ($source:ident,$value:expr) => {
        const {
            $crate::base::time::time::TimeDelta::FromInternalValue(
                $crate::NativeSaturatedCastConst!(i64, $source, $value),
            )
        }
    };
}
