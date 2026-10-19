//! Signed microsecond values crossing the compositor time boundary.
//! The host supplies monotonic timestamps; this module never reads a clock.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeDelta(i64);

impl TimeDelta {
    pub const fn Microseconds(value: i64) -> Self {
        Self(value)
    }
    pub const fn Milliseconds(value: i64) -> Self {
        Self(value.saturating_mul(1000))
    }
    pub const fn Seconds(value: i64) -> Self {
        Self(value.saturating_mul(1_000_000))
    }
    pub const fn InMicroseconds(self) -> i64 {
        self.0
    }
    // Source dependency: base/time/time.h:1018-1024.
    pub fn InMicrosecondsF(self) -> f64 {
        if self == Self::Max() {
            f64::INFINITY
        } else if self == Self::Min() {
            f64::NEG_INFINITY
        } else {
            self.0 as f64
        }
    }
    // Source dependency: base/time/time.h:970-976.
    pub fn InSecondsF(self) -> f64 {
        self.InMicrosecondsF() / 1_000_000.0
    }
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }
    pub const fn Max() -> Self {
        Self(i64::MAX)
    }
    pub const fn Min() -> Self {
        Self(i64::MIN)
    }
}

impl Add for TimeDelta {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sub for TimeDelta {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}
impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl Neg for TimeDelta {
    type Output = Self;
    fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }
}
impl Mul<i32> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Self(self.0.saturating_mul(i64::from(rhs)))
    }
}
impl Mul<f64> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self((self.0 as f64 * rhs) as i64)
    }
}
impl Mul<f32> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        self * f64::from(rhs)
    }
}
impl Mul<TimeDelta> for f64 {
    type Output = TimeDelta;
    fn mul(self, rhs: TimeDelta) -> TimeDelta {
        rhs * self
    }
}
impl Mul<TimeDelta> for f32 {
    type Output = TimeDelta;
    fn mul(self, rhs: TimeDelta) -> TimeDelta {
        rhs * self
    }
}
impl Div<i32> for TimeDelta {
    type Output = Self;
    fn div(self, rhs: i32) -> Self {
        Self(self.0 / i64::from(rhs))
    }
}
// Source dependency: base/time/time.h:277-281;
// base/numerics/clamped_math_impl.h:278-296. Scale the stored microsecond
// integer (not InMicrosecondsF's sentinel projection), then saturated-cast.
impl Div<f64> for TimeDelta {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::Microseconds((self.0 as f64 / rhs) as i64)
    }
}
impl Div<TimeDelta> for TimeDelta {
    type Output = f64;
    fn div(self, rhs: TimeDelta) -> f64 {
        // Source dependency: base/time/time.h:295-303,348-353.
        let left_inf = self == Self::Max() || self == Self::Min();
        let right_inf = rhs == Self::Max() || rhs == Self::Min();
        assert!(!self.is_zero() || !rhs.is_zero());
        assert!(!left_inf || !right_inf);
        self.InMicrosecondsF() / rhs.InMicrosecondsF()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeTicks(i64);

impl TimeTicks {
    /// Zero is Chromium's null tick. The host should supply nonzero ticks.
    pub const fn FromMicroseconds(value: i64) -> Self {
        Self(value)
    }
    pub const fn since_origin(self) -> TimeDelta {
        TimeDelta::Microseconds(self.0)
    }
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
    pub const fn Min() -> Self {
        Self(i64::MIN)
    }
    pub const fn Max() -> Self {
        Self(i64::MAX)
    }
}
impl Add<TimeDelta> for TimeTicks {
    type Output = Self;
    fn add(self, rhs: TimeDelta) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sub<TimeDelta> for TimeTicks {
    type Output = Self;
    fn sub(self, rhs: TimeDelta) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}
impl Sub for TimeTicks {
    type Output = TimeDelta;
    fn sub(self, rhs: Self) -> TimeDelta {
        TimeDelta::Microseconds(self.0.saturating_sub(rhs.0))
    }
}
impl AddAssign<TimeDelta> for TimeTicks {
    fn add_assign(&mut self, rhs: TimeDelta) {
        *self = *self + rhs;
    }
}
impl SubAssign<TimeDelta> for TimeTicks {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        *self = *self - rhs;
    }
}

impl Mul<u64> for TimeDelta {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self {
        Self(
            (i128::from(self.0) * i128::from(rhs)).clamp(i128::from(i64::MIN), i128::from(i64::MAX))
                as i64,
        )
    }
}
