// Migrated from tiny-skia-path 0.12.0 src/lib.rs (SkPoint and its methods).

use crate::path::{f32x2, Scalar};
/// A point.
///
/// Doesn't guarantee to be finite.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct SkPoint {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for SkPoint {
    #[inline]
    fn from(v: (f32, f32)) -> Self {
        SkPoint { x: v.0, y: v.1 }
    }
}

impl SkPoint {
    /// Creates a new `SkPoint`.
    pub fn from_xy(x: f32, y: f32) -> Self {
        SkPoint { x, y }
    }

    /// Creates a new `SkPoint` from `f32x2`.
    pub fn from_f32x2(r: f32x2) -> Self {
        SkPoint::from_xy(r.x(), r.y())
    }

    /// Converts a `SkPoint` into a `f32x2`.
    pub fn to_f32x2(&self) -> f32x2 {
        f32x2::new(self.x, self.y)
    }

    /// Creates a point at 0x0 position.
    pub fn zero() -> Self {
        SkPoint { x: 0.0, y: 0.0 }
    }

    /// Returns true if x and y are both zero.
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    /// Returns true if both x and y are measurable values.
    ///
    /// Both values are other than infinities and NaN.
    pub fn is_finite(&self) -> bool {
        (self.x * self.y).is_finite()
    }

    /// Checks that two `SkPoint`s are almost equal.
    pub(crate) fn almost_equal(&self, other: SkPoint) -> bool {
        !(*self - other).can_normalize()
    }

    /// Checks that two `SkPoint`s are almost equal using the specified tolerance.
    pub(crate) fn equals_within_tolerance(&self, other: SkPoint, tolerance: f32) -> bool {
        (self.x - other.x).is_nearly_zero_within_tolerance(tolerance)
            && (self.y - other.y).is_nearly_zero_within_tolerance(tolerance)
    }

    /// Scales (fX, fY) so that length() returns one, while preserving ratio of fX to fY,
    /// if possible.
    ///
    /// If prior length is nearly zero, sets vector to (0, 0) and returns
    /// false; otherwise returns true.
    pub fn normalize(&mut self) -> bool {
        self.set_length_from(self.x, self.y, 1.0)
    }

    /// Sets vector to (x, y) scaled so length() returns one, and so that (x, y)
    /// is proportional to (x, y).
    ///
    /// If (x, y) length is nearly zero, sets vector to (0, 0) and returns false;
    /// otherwise returns true.
    pub fn set_normalize(&mut self, x: f32, y: f32) -> bool {
        self.set_length_from(x, y, 1.0)
    }

    pub(crate) fn can_normalize(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && (self.x != 0.0 || self.y != 0.0)
    }

    /// Returns the Euclidean distance from origin.
    pub fn length(&self) -> f32 {
        let mag2 = self.x * self.x + self.y * self.y;
        if mag2.is_finite() {
            mag2.sqrt()
        } else {
            let xx = f64::from(self.x);
            let yy = f64::from(self.y);
            (xx * xx + yy * yy).sqrt() as f32
        }
    }

    /// Scales vector so that distanceToOrigin() returns length, if possible.
    ///
    /// If former length is nearly zero, sets vector to (0, 0) and return false;
    /// otherwise returns true.
    pub fn set_length(&mut self, length: f32) -> bool {
        self.set_length_from(self.x, self.y, length)
    }

    /// Sets vector to (x, y) scaled to length, if possible.
    ///
    /// If former length is nearly zero, sets vector to (0, 0) and return false;
    /// otherwise returns true.
    pub fn set_length_from(&mut self, x: f32, y: f32, length: f32) -> bool {
        set_point_length(self, x, y, length, &mut None)
    }

    /// Returns the Euclidean distance from origin.
    pub fn distance(&self, other: SkPoint) -> f32 {
        (*self - other).length()
    }

    /// Returns the dot product of two points.
    pub fn dot(&self, other: SkPoint) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the cross product of vector and vec.
    ///
    /// Vector and vec form three-dimensional vectors with z-axis value equal to zero.
    /// The cross product is a three-dimensional vector with x-axis and y-axis values
    /// equal to zero. The cross product z-axis component is returned.
    pub fn cross(&self, other: SkPoint) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub(crate) fn distance_to_sqd(&self, pt: SkPoint) -> f32 {
        let dx = self.x - pt.x;
        let dy = self.y - pt.y;
        dx * dx + dy * dy
    }

    pub(crate) fn length_sqd(&self) -> f32 {
        self.dot(*self)
    }

    /// Scales SkPoint in-place by scale.
    pub fn scale(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
    }

    pub(crate) fn scaled(&self, scale: f32) -> Self {
        SkPoint::from_xy(self.x * scale, self.y * scale)
    }

    pub(crate) fn swap_coords(&mut self) {
        core::mem::swap(&mut self.x, &mut self.y);
    }

    pub(crate) fn rotate_cw(&mut self) {
        self.swap_coords();
        self.x = -self.x;
    }

    pub(crate) fn rotate_ccw(&mut self) {
        self.swap_coords();
        self.y = -self.y;
    }
}

// We have to worry about 2 tricky conditions:
// 1. underflow of mag2 (compared against nearlyzero^2)
// 2. overflow of mag2 (compared w/ isfinite)
//
// If we underflow, we return false. If we overflow, we compute again using
// doubles, which is much slower (3x in a desktop test) but will not overflow.
fn set_point_length(
    pt: &mut SkPoint,
    mut x: f32,
    mut y: f32,
    length: f32,
    orig_length: &mut Option<f32>,
) -> bool {
    // our mag2 step overflowed to infinity, so use doubles instead.
    // much slower, but needed when x or y are very large, other wise we
    // divide by inf. and return (0,0) vector.
    let xx = x as f64;
    let yy = y as f64;
    let dmag = (xx * xx + yy * yy).sqrt();
    let dscale = length as f64 / dmag;
    x *= dscale as f32;
    y *= dscale as f32;

    // check if we're not finite, or we're zero-length
    if !x.is_finite() || !y.is_finite() || (x == 0.0 && y == 0.0) {
        *pt = SkPoint::zero();
        return false;
    }

    let mut mag = 0.0;
    if orig_length.is_some() {
        mag = dmag as f32;
    }

    *pt = SkPoint::from_xy(x, y);

    if orig_length.is_some() {
        *orig_length = Some(mag);
    }

    true
}

impl core::ops::Neg for SkPoint {
    type Output = SkPoint;

    fn neg(self) -> Self::Output {
        SkPoint {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl core::ops::Add for SkPoint {
    type Output = SkPoint;

    fn add(self, other: SkPoint) -> Self::Output {
        SkPoint::from_xy(self.x + other.x, self.y + other.y)
    }
}

impl core::ops::AddAssign for SkPoint {
    fn add_assign(&mut self, other: SkPoint) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl core::ops::Sub for SkPoint {
    type Output = SkPoint;

    fn sub(self, other: SkPoint) -> Self::Output {
        SkPoint::from_xy(self.x - other.x, self.y - other.y)
    }
}

impl core::ops::SubAssign for SkPoint {
    fn sub_assign(&mut self, other: SkPoint) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl core::ops::Mul for SkPoint {
    type Output = SkPoint;

    fn mul(self, other: SkPoint) -> Self::Output {
        SkPoint::from_xy(self.x * other.x, self.y * other.y)
    }
}

impl core::ops::MulAssign for SkPoint {
    fn mul_assign(&mut self, other: SkPoint) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

/// Legacy Rust API alias; canonical implementation name follows Skia.
pub use SkPoint as Point;
