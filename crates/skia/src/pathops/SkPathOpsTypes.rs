// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkPathOpsTypes.h and SkPathOpsTypes.cpp.
#![allow(non_snake_case, non_upper_case_globals)]
pub use super::SkOpGlobalState::SkOpGlobalState;
use crate::src::core::{SkFloatBits::SkFloatAs2sCompliment, SkPath::Verb};

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathOpsMask {
    kWinding_PathOpsMask = -1,
    kNo_PathOpsMask = 0,
    kEvenOdd_PathOpsMask = 1,
}
pub use SkPathOpsMask::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkOpPhase {
    kNoChange,
    kIntersecting,
    kWalking,
    kFixWinding,
}

// Source: SkPathOpsTypes.cpp:18-108. Keep the float operations in float
// precision; the double overloads deliberately narrow to SkScalar upstream.
fn arguments_denormalized(a: f32, b: f32, epsilon: i32) -> bool {
    let check = f32::EPSILON * epsilon as f32 / 2.0;
    a.abs() <= check && b.abs() <= check
}
fn equal_ulps(a: f32, b: f32, epsilon: i32, depsilon: i32) -> bool {
    if arguments_denormalized(a, b, depsilon) {
        return true;
    }
    equal_ulps_no_normal_check(a, b, epsilon)
}
fn equal_ulps_no_normal_check(a: f32, b: f32, epsilon: i32) -> bool {
    let a = SkFloatAs2sCompliment(a);
    let b = SkFloatAs2sCompliment(b);
    a < b.wrapping_add(epsilon) && b < a.wrapping_add(epsilon)
}
fn equal_ulps_pin(a: f32, b: f32, epsilon: i32, depsilon: i32) -> bool {
    a.is_finite() && b.is_finite() && equal_ulps(a, b, epsilon, depsilon)
}
fn not_equal_ulps(a: f32, b: f32, epsilon: i32) -> bool {
    if arguments_denormalized(a, b, epsilon) {
        return false;
    }
    !equal_ulps_no_normal_check(a, b, epsilon)
}
fn less_ulps(a: f32, b: f32, epsilon: i32) -> bool {
    if arguments_denormalized(a, b, epsilon) {
        return a <= b - f32::EPSILON * epsilon as f32;
    }
    SkFloatAs2sCompliment(a) <= SkFloatAs2sCompliment(b).wrapping_sub(epsilon)
}
fn less_or_equal_ulps(a: f32, b: f32, epsilon: i32) -> bool {
    if arguments_denormalized(a, b, epsilon) {
        return a < b + f32::EPSILON * epsilon as f32;
    }
    SkFloatAs2sCompliment(a) < SkFloatAs2sCompliment(b).wrapping_add(epsilon)
}
macro_rules! ulps_pair {
    ($name:ident, $body:expr) => {
        #[inline]
        pub fn $name(a: impl Into<f64>, b: impl Into<f64>) -> bool {
            ($body)(a.into() as f32, b.into() as f32)
        }
    };
}
ulps_pair!(AlmostBequalUlps, |a, b| equal_ulps(a, b, 2, 2));
ulps_pair!(AlmostPequalUlps, |a, b| equal_ulps(a, b, 8, 8));
ulps_pair!(AlmostEqualUlps, |a, b| equal_ulps(a, b, 16, 16));
ulps_pair!(AlmostEqualUlpsNoNormalCheck, |a, b| {
    equal_ulps_no_normal_check(a, b, 16)
});
ulps_pair!(AlmostEqualUlps_Pin, |a, b| equal_ulps_pin(a, b, 16, 16));
ulps_pair!(NotAlmostEqualUlps, |a, b| not_equal_ulps(a, b, 16));
ulps_pair!(NotAlmostEqualUlps_Pin, |a: f32, b: f32| a.is_finite()
    && b.is_finite()
    && not_equal_ulps(a, b, 16));
ulps_pair!(NotAlmostDequalUlps, |a, b| !equal_ulps_no_normal_check(
    a, b, 16
));
ulps_pair!(RoughlyEqualUlps, |a, b| equal_ulps(a, b, 256, 1024));
ulps_pair!(AlmostLessUlps, |a, b| less_ulps(a, b, 16));
ulps_pair!(AlmostLessOrEqualUlps, |a, b| less_or_equal_ulps(a, b, 16));

// The double overload retains its own overflow branch, unlike the other
// scalar-narrowing overloads. Source: SkPathOpsTypes.cpp:126-135.
pub trait DequalScalar: Copy {
    fn dequal(self, other: Self) -> bool;
}
impl DequalScalar for f32 {
    fn dequal(self, other: Self) -> bool {
        equal_ulps_no_normal_check(self, other, 16)
    }
}
impl DequalScalar for f64 {
    fn dequal(self, other: Self) -> bool {
        let a = self;
        let b = other;
        if a.abs() < f32::MAX as f64 && b.abs() < f32::MAX as f64 {
            return equal_ulps_no_normal_check(a as f32, b as f32, 16);
        }
        let aa = a.abs();
        let bb = b.abs();
        let max = if aa < bb { bb } else { aa };
        (a - b).abs() / max < (f32::EPSILON * 16.0) as f64
    }
}
pub fn AlmostDequalUlps<T: DequalScalar>(a: T, b: T) -> bool {
    a.dequal(b)
}

pub fn AlmostBetweenUlps(a: impl Into<f64>, b: impl Into<f64>, c: impl Into<f64>) -> bool {
    let a = a.into() as f32;
    let b = b.into() as f32;
    let c = c.into() as f32;
    if a <= c {
        less_or_equal_ulps(a, b, 2) && less_or_equal_ulps(b, c, 2)
    } else {
        less_or_equal_ulps(b, a, 2) && less_or_equal_ulps(c, b, 2)
    }
}
pub fn UlpsDistance(a: impl Into<f64>, b: impl Into<f64>) -> i32 {
    let a = a.into() as f32;
    let b = b.into() as f32;
    let aa = a.to_bits() as i32;
    let bb = b.to_bits() as i32;
    if (aa < 0) != (bb < 0) {
        return if a == b { 0 } else { i32::MAX };
    }
    aa.wrapping_sub(bb).wrapping_abs()
}

// Source: SkPathOpsTypes.h:311-330.
pub const FLT_EPSILON: f64 = f32::EPSILON as f64;
pub const DBL_EPSILON: f64 = f64::EPSILON;
pub const FLT_EPSILON_CUBED: f64 = FLT_EPSILON * FLT_EPSILON * FLT_EPSILON;
pub const FLT_EPSILON_HALF: f64 = FLT_EPSILON / 2.0;
pub const FLT_EPSILON_DOUBLE: f64 = FLT_EPSILON * 2.0;
pub const FLT_EPSILON_ORDERABLE_ERR: f64 = FLT_EPSILON * 16.0;
pub const FLT_EPSILON_SQUARED: f64 = FLT_EPSILON * FLT_EPSILON;
pub const FLT_EPSILON_SQRT: f64 = 0.00034526697709225118;
pub const FLT_EPSILON_INVERSE: f64 = 1.0 / FLT_EPSILON;
pub const DBL_EPSILON_ERR: f64 = DBL_EPSILON * 4.0;
pub const DBL_EPSILON_SUBDIVIDE_ERR: f64 = DBL_EPSILON * 16.0;
pub const ROUGH_EPSILON: f64 = FLT_EPSILON * 64.0;
pub const MORE_ROUGH_EPSILON: f64 = FLT_EPSILON * 256.0;
pub const WAY_ROUGH_EPSILON: f64 = FLT_EPSILON * 2048.0;
pub const BUMP_EPSILON: f64 = FLT_EPSILON * 4096.0;
pub const INVERSE_NUMBER_RANGE: f32 = FLT_EPSILON_ORDERABLE_ERR as f32;

#[inline]
pub fn zero_or_one(x: f64) -> bool {
    x == 0.0 || x == 1.0
}

#[inline]
pub fn approximately_zero(x: f64) -> bool {
    (x).abs() < FLT_EPSILON
}

#[inline]
pub fn precisely_zero(x: f64) -> bool {
    (x).abs() < DBL_EPSILON_ERR
}

#[inline]
pub fn precisely_subdivide_zero(x: f64) -> bool {
    (x).abs() < DBL_EPSILON_SUBDIVIDE_ERR
}

#[inline]
pub fn approximately_zero_half(x: f64) -> bool {
    (x).abs() < FLT_EPSILON_HALF
}

#[inline]
pub fn approximately_zero_double(x: f64) -> bool {
    (x).abs() < FLT_EPSILON_DOUBLE
}

#[inline]
pub fn approximately_zero_orderable(x: f64) -> bool {
    (x).abs() < FLT_EPSILON_ORDERABLE_ERR
}

#[inline]
pub fn approximately_zero_squared(x: f64) -> bool {
    (x).abs() < FLT_EPSILON_SQUARED
}

#[inline]
pub fn approximately_zero_sqrt(x: f64) -> bool {
    (x).abs() < FLT_EPSILON_SQRT
}

#[inline]
pub fn roughly_zero(x: f64) -> bool {
    (x).abs() < ROUGH_EPSILON
}

#[inline]
pub fn approximately_zero_inverse(x: f64) -> bool {
    (x).abs() > FLT_EPSILON_INVERSE
}

#[inline]
pub fn approximately_zero_when_compared_to(x: f64, y: f64) -> bool {
    x == 0.0 || (x).abs() < (y * FLT_EPSILON).abs()
}

#[inline]
pub fn precisely_zero_when_compared_to(x: f64, y: f64) -> bool {
    x == 0.0 || (x).abs() < (y * DBL_EPSILON).abs()
}

#[inline]
pub fn roughly_zero_when_compared_to(x: f64, y: f64) -> bool {
    x == 0.0 || (x).abs() < (y * ROUGH_EPSILON).abs()
}

#[inline]
pub fn approximately_equal(x: f64, y: f64) -> bool {
    approximately_zero(x - y)
}

#[inline]
pub fn precisely_equal(x: f64, y: f64) -> bool {
    precisely_zero(x - y)
}

#[inline]
pub fn precisely_subdivide_equal(x: f64, y: f64) -> bool {
    precisely_subdivide_zero(x - y)
}

#[inline]
pub fn approximately_equal_half(x: f64, y: f64) -> bool {
    approximately_zero_half(x - y)
}

#[inline]
pub fn approximately_equal_double(x: f64, y: f64) -> bool {
    approximately_zero_double(x - y)
}

#[inline]
pub fn approximately_equal_orderable(x: f64, y: f64) -> bool {
    approximately_zero_orderable(x - y)
}

#[inline]
pub fn approximately_equal_squared(x: f64, y: f64) -> bool {
    approximately_equal(x, y)
}

#[inline]
pub fn approximately_greater(x: f64, y: f64) -> bool {
    x - FLT_EPSILON >= y
}

#[inline]
pub fn approximately_greater_double(x: f64, y: f64) -> bool {
    x - FLT_EPSILON_DOUBLE >= y
}

#[inline]
pub fn approximately_greater_orderable(x: f64, y: f64) -> bool {
    x - FLT_EPSILON_ORDERABLE_ERR >= y
}

#[inline]
pub fn approximately_greater_or_equal(x: f64, y: f64) -> bool {
    x + FLT_EPSILON > y
}

#[inline]
pub fn approximately_greater_or_equal_double(x: f64, y: f64) -> bool {
    x + FLT_EPSILON_DOUBLE > y
}

#[inline]
pub fn approximately_greater_or_equal_orderable(x: f64, y: f64) -> bool {
    x + FLT_EPSILON_ORDERABLE_ERR > y
}

#[inline]
pub fn approximately_lesser(x: f64, y: f64) -> bool {
    x + FLT_EPSILON <= y
}

#[inline]
pub fn approximately_lesser_double(x: f64, y: f64) -> bool {
    x + FLT_EPSILON_DOUBLE <= y
}

#[inline]
pub fn approximately_lesser_orderable(x: f64, y: f64) -> bool {
    x + FLT_EPSILON_ORDERABLE_ERR <= y
}

#[inline]
pub fn approximately_lesser_or_equal(x: f64, y: f64) -> bool {
    x - FLT_EPSILON < y
}

#[inline]
pub fn approximately_lesser_or_equal_double(x: f64, y: f64) -> bool {
    x - FLT_EPSILON_DOUBLE < y
}

#[inline]
pub fn approximately_lesser_or_equal_orderable(x: f64, y: f64) -> bool {
    x - FLT_EPSILON_ORDERABLE_ERR < y
}

#[inline]
pub fn approximately_greater_than_one(x: f64) -> bool {
    x > 1.0 - FLT_EPSILON
}

#[inline]
pub fn precisely_greater_than_one(x: f64) -> bool {
    x > 1.0 - DBL_EPSILON_ERR
}

#[inline]
pub fn approximately_less_than_zero(x: f64) -> bool {
    x < FLT_EPSILON
}

#[inline]
pub fn precisely_less_than_zero(x: f64) -> bool {
    x < DBL_EPSILON_ERR
}

#[inline]
pub fn approximately_negative(x: f64) -> bool {
    x < FLT_EPSILON
}

#[inline]
pub fn approximately_negative_orderable(x: f64) -> bool {
    x < FLT_EPSILON_ORDERABLE_ERR
}

#[inline]
pub fn precisely_negative(x: f64) -> bool {
    x < DBL_EPSILON_ERR
}

#[inline]
pub fn approximately_one_or_less(x: f64) -> bool {
    x < 1.0 + FLT_EPSILON
}

#[inline]
pub fn approximately_one_or_less_double(x: f64) -> bool {
    x < 1.0 + FLT_EPSILON_DOUBLE
}

#[inline]
pub fn approximately_positive(x: f64) -> bool {
    x > -FLT_EPSILON
}

#[inline]
pub fn approximately_positive_squared(x: f64) -> bool {
    x > -(FLT_EPSILON_SQUARED)
}

#[inline]
pub fn approximately_zero_or_more(x: f64) -> bool {
    x > -FLT_EPSILON
}

#[inline]
pub fn approximately_zero_or_more_double(x: f64) -> bool {
    x > -FLT_EPSILON_DOUBLE
}

#[inline]
pub fn roughly_equal(x: f64, y: f64) -> bool {
    (x - y).abs() < ROUGH_EPSILON
}

#[inline]
pub fn roughly_negative(x: f64) -> bool {
    x < ROUGH_EPSILON
}

#[inline]
pub fn more_roughly_equal(x: f64, y: f64) -> bool {
    (x - y).abs() < MORE_ROUGH_EPSILON
}

#[inline]
pub fn approximately_between_orderable(a: f64, b: f64, c: f64) -> bool {
    if a <= c {
        approximately_negative_orderable(a - b) && approximately_negative_orderable(b - c)
    } else {
        approximately_negative_orderable(b - a) && approximately_negative_orderable(c - b)
    }
}
#[inline]
pub fn approximately_between(a: f64, b: f64, c: f64) -> bool {
    if a <= c {
        approximately_negative(a - b) && approximately_negative(b - c)
    } else {
        approximately_negative(b - a) && approximately_negative(c - b)
    }
}
#[inline]
pub fn precisely_between(a: f64, b: f64, c: f64) -> bool {
    if a <= c {
        precisely_negative(a - b) && precisely_negative(b - c)
    } else {
        precisely_negative(b - a) && precisely_negative(c - b)
    }
}
#[inline]
pub fn roughly_between(a: f64, b: f64, c: f64) -> bool {
    if a <= c {
        roughly_negative(a - b) && roughly_negative(b - c)
    } else {
        roughly_negative(b - a) && roughly_negative(c - b)
    }
}
#[inline]
pub fn between(a: f64, b: f64, c: f64) -> bool {
    (a - b) * (c - b) <= 0.0
}
pub fn SkPathOpsPointsToVerb(points: i32) -> Verb {
    match (1_i32 << points) >> 1 {
        0 => Verb::kMove_Verb,
        1 => Verb::kLine_Verb,
        2 => Verb::kQuad_Verb,
        4 => Verb::kCubic_Verb,
        _ => panic!("invalid path point count"),
    }
}
pub fn SkPathOpsVerbToPoints(verb: Verb) -> i32 {
    let verb = verb as i32;
    verb - ((verb + 1) >> 2)
}
#[inline]
pub fn SkDInterp(A: f64, B: f64, t: f64) -> f64 {
    A + (B - A) * t
}
#[inline]
pub fn SkDSign(x: f64) -> i32 {
    (x > 0.0) as i32 - (x < 0.0) as i32
}
#[inline]
pub fn SKDSide(x: f64) -> i32 {
    (x > 0.0) as i32 + (x >= 0.0) as i32
}
#[inline]
pub fn SkDSideBit(x: f64) -> i32 {
    1 << SKDSide(x)
}
#[inline]
pub fn SkPinT(t: f64) -> f64 {
    if precisely_less_than_zero(t) {
        0.0
    } else if precisely_greater_than_one(t) {
        1.0
    } else {
        t
    }
}
