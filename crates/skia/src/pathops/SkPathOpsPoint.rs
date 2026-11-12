// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsPoint.h. Release double-precision point values.
use super::SkPathOpsTypes::*;
use crate::src::core::SkPoint::SkPoint;
use core::{borrow::Borrow, ops::*};

pub fn AlmostEqualUlpsPoint(a: &SkPoint, b: &SkPoint) -> bool {
    AlmostEqualUlps(a.fX, b.fX) && AlmostEqualUlps(a.fY, b.fY)
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkDVector {
    pub fX: f64,
    pub fY: f64,
}
impl SkDVector {
    pub fn set(&mut self, pt: &SkPoint) -> &mut Self {
        self.fX = pt.fX as f64;
        self.fY = pt.fY as f64;
        self
    }
    pub fn asSkVector(&self) -> SkPoint {
        SkPoint {
            fX: self.fX as f32,
            fY: self.fY as f32,
        }
    }
    pub fn cross(&self, a: &Self) -> f64 {
        self.fX * a.fY - self.fY * a.fX
    }
    pub fn crossCheck(&self, a: &Self) -> f64 {
        let xy = self.fX * a.fY;
        let yx = self.fY * a.fX;
        if AlmostEqualUlps(xy, yx) {
            0.0
        } else {
            xy - yx
        }
    }
    pub fn crossNoNormalCheck(&self, a: &Self) -> f64 {
        let xy = self.fX * a.fY;
        let yx = self.fY * a.fX;
        if AlmostEqualUlpsNoNormalCheck(xy, yx) {
            0.0
        } else {
            xy - yx
        }
    }
    pub fn dot(&self, a: &Self) -> f64 {
        self.fX * a.fX + self.fY * a.fY
    }
    pub fn lengthSquared(&self) -> f64 {
        self.fX * self.fX + self.fY * self.fY
    }
    pub fn length(&self) -> f64 {
        self.lengthSquared().sqrt()
    }
    pub fn normalize(&mut self) -> &mut Self {
        let inv = 1.0 / self.length();
        self.fX *= inv;
        self.fY *= inv;
        self
    }
    pub fn isFinite(&self) -> bool {
        self.fX.is_finite() && self.fY.is_finite()
    }
}
impl AddAssign for SkDVector {
    fn add_assign(&mut self, v: Self) {
        self.fX += v.fX;
        self.fY += v.fY;
    }
}
impl SubAssign for SkDVector {
    fn sub_assign(&mut self, v: Self) {
        self.fX -= v.fX;
        self.fY -= v.fY;
    }
}
impl MulAssign<f64> for SkDVector {
    fn mul_assign(&mut self, s: f64) {
        self.fX *= s;
        self.fY *= s;
    }
}
impl DivAssign<f64> for SkDVector {
    fn div_assign(&mut self, s: f64) {
        self.fX /= s;
        self.fY /= s;
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkDPoint {
    pub fX: f64,
    pub fY: f64,
}
impl From<SkPoint> for SkDPoint {
    fn from(p: SkPoint) -> Self {
        Self {
            fX: p.fX as f64,
            fY: p.fY as f64,
        }
    }
}
impl From<&SkPoint> for SkDPoint {
    fn from(p: &SkPoint) -> Self {
        Self::from(*p)
    }
}
impl From<&SkDPoint> for SkDPoint {
    fn from(p: &SkDPoint) -> Self {
        *p
    }
}
impl Sub for SkDPoint {
    type Output = SkDVector;
    fn sub(self, b: Self) -> SkDVector {
        SkDVector {
            fX: self.fX - b.fX,
            fY: self.fY - b.fY,
        }
    }
}
impl AddAssign<SkDVector> for SkDPoint {
    fn add_assign(&mut self, v: SkDVector) {
        self.fX += v.fX;
        self.fY += v.fY;
    }
}
impl SubAssign<SkDVector> for SkDPoint {
    fn sub_assign(&mut self, v: SkDVector) {
        self.fX -= v.fX;
        self.fY -= v.fY;
    }
}
impl Add<SkDVector> for SkDPoint {
    type Output = Self;
    fn add(mut self, v: SkDVector) -> Self {
        self += v;
        self
    }
}
impl Sub<SkDVector> for SkDPoint {
    type Output = Self;
    fn sub(mut self, v: SkDVector) -> Self {
        self -= v;
        self
    }
}
fn min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}
fn max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}
impl SkDPoint {
    pub fn set(&mut self, pt: &SkPoint) {
        self.fX = pt.fX as f64;
        self.fY = pt.fY as f64;
    }
    fn largestMagnitude(&self, a: &Self) -> f64 {
        let tiniest = min(min(min(self.fX, a.fX), self.fY), a.fY);
        let largest = max(max(max(self.fX, a.fX), self.fY), a.fY);
        max(largest, -tiniest)
    }
    pub fn approximatelyDEqual(&self, a: impl Into<Self>) -> bool {
        let a = a.into();
        if approximately_equal(self.fX, a.fX) && approximately_equal(self.fY, a.fY) {
            return true;
        }
        if !RoughlyEqualUlps(self.fX, a.fX) || !RoughlyEqualUlps(self.fY, a.fY) {
            return false;
        }
        let dist = self.distance(&a);
        let largest = self.largestMagnitude(&a);
        AlmostDequalUlps(largest, largest + dist)
    }
    pub fn approximatelyEqual(&self, a: impl Into<Self>) -> bool {
        let a = a.into();
        if approximately_equal(self.fX, a.fX) && approximately_equal(self.fY, a.fY) {
            return true;
        }
        if !RoughlyEqualUlps(self.fX, a.fX) || !RoughlyEqualUlps(self.fY, a.fY) {
            return false;
        }
        let dist = self.distance(&a);
        let largest = self.largestMagnitude(&a);
        AlmostPequalUlps(largest, largest + dist)
    }
    pub fn ApproximatelyEqual(a: impl Borrow<SkPoint>, b: impl Borrow<SkPoint>) -> bool {
        let a = a.borrow();
        let b = b.borrow();
        if approximately_equal(a.fX as f64, b.fX as f64)
            && approximately_equal(a.fY as f64, b.fY as f64)
        {
            return true;
        }
        if !RoughlyEqualUlps(a.fX, b.fX) || !RoughlyEqualUlps(a.fY, b.fY) {
            return false;
        }
        let aa = Self::from(a);
        let bb = Self::from(b);
        let dist = aa.distance(&bb);
        let largest = aa.largestMagnitude(&bb);
        AlmostDequalUlps(largest, largest + dist)
    }
    pub fn approximatelyZero(&self) -> bool {
        approximately_zero(self.fX) && approximately_zero(self.fY)
    }
    pub fn asSkPoint(&self) -> SkPoint {
        SkPoint {
            fX: self.fX as f32,
            fY: self.fY as f32,
        }
    }
    pub fn distance(&self, a: &Self) -> f64 {
        (*self - *a).length()
    }
    pub fn distanceSquared(&self, a: &Self) -> f64 {
        (*self - *a).lengthSquared()
    }
    pub fn Mid(a: &Self, b: &Self) -> Self {
        Self {
            fX: (a.fX + b.fX) / 2.0,
            fY: (a.fY + b.fY) / 2.0,
        }
    }
    pub fn roughlyEqual(&self, a: &Self) -> bool {
        if roughly_equal(self.fX, a.fX) && roughly_equal(self.fY, a.fY) {
            return true;
        }
        let dist = self.distance(a);
        let largest = self.largestMagnitude(a);
        RoughlyEqualUlps(largest, largest + dist)
    }
    pub fn RoughlyEqual(a: &SkPoint, b: &SkPoint) -> bool {
        if !RoughlyEqualUlps(a.fX, b.fX) && !RoughlyEqualUlps(a.fY, b.fY) {
            return false;
        }
        let aa = Self::from(a);
        let bb = Self::from(b);
        let dist = aa.distance(&bb);
        let largest = aa.largestMagnitude(&bb);
        RoughlyEqualUlps(largest, largest + dist)
    }
    pub fn WayRoughlyEqual(a: &SkPoint, b: &SkPoint) -> bool {
        let largest = max(
            a.fX.abs() as f64,
            max(a.fY.abs() as f64, max(b.fX.abs() as f64, b.fY.abs() as f64)),
        ) as f32;
        let diffs = *a - *b;
        let diff = max(diffs.fX.abs() as f64, diffs.fY.abs() as f64) as f32;
        roughly_zero_when_compared_to(diff as f64, largest as f64)
    }
}
