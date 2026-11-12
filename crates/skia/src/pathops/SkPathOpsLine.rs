// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsLine.h/.cpp.
use super::{SkPathOpsPoint::SkDPoint, SkPathOpsTypes::*};
use crate::src::core::SkPoint::SkPoint;
use core::ops::{Index, IndexMut};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDLine {
    pub fPts: [SkDPoint; 2],
}
impl Index<usize> for SkDLine {
    type Output = SkDPoint;
    fn index(&self, n: usize) -> &SkDPoint {
        &self.fPts[n]
    }
}
impl IndexMut<usize> for SkDLine {
    fn index_mut(&mut self, n: usize) -> &mut SkDPoint {
        &mut self.fPts[n]
    }
}
impl SkDLine {
    pub fn set(&mut self, p: &[SkPoint; 2]) -> &Self {
        self.fPts = p.map(SkDPoint::from);
        self
    }
    pub fn ptAtT(&self, t: f64) -> SkDPoint {
        if t == 0.0 {
            return self[0];
        }
        if t == 1.0 {
            return self[1];
        }
        SkDPoint {
            fX: (1.0 - t) * self[0].fX + t * self[1].fX,
            fY: (1.0 - t) * self[0].fY + t * self[1].fY,
        }
    }
    pub fn exactPoint(&self, xy: &SkDPoint) -> f64 {
        if *xy == self[0] {
            0.0
        } else if *xy == self[1] {
            1.0
        } else {
            -1.0
        }
    }
    pub fn nearPoint(&self, xy: &SkDPoint, unequal: Option<&mut bool>) -> f64 {
        if !AlmostBetweenUlps(self[0].fX, xy.fX, self[1].fX)
            || !AlmostBetweenUlps(self[0].fY, xy.fY, self[1].fY)
        {
            return -1.0;
        }
        let len = self[1] - self[0];
        let denom = len.fX * len.fX + len.fY * len.fY;
        let ab0 = *xy - self[0];
        let numer = len.fX * ab0.fX + ab0.fY * len.fY;
        if !between(0.0, numer, denom) {
            return -1.0;
        }
        if denom == 0.0 {
            return 0.0;
        }
        let mut t = numer / denom;
        let dist = self.ptAtT(t).distance(xy);
        let largest = self.largestMagnitude();
        if !AlmostEqualUlps_Pin(largest, largest + dist) {
            return -1.0;
        }
        if let Some(unequal) = unequal {
            *unequal = largest as f32 != (largest + dist) as f32;
        }
        t = SkPinT(t);

        t
    }
    fn largestMagnitude(&self) -> f64 {
        let tiniest = self[0].fX.min(self[0].fY).min(self[1].fX).min(self[1].fY);
        let largest = self[0].fX.max(self[0].fY).max(self[1].fX).max(self[1].fY);
        largest.max(-tiniest)
    }
    pub fn nearRay(&self, xy: &SkDPoint) -> bool {
        let len = self[1] - self[0];
        let denom = len.fX * len.fX + len.fY * len.fY;
        let ab0 = *xy - self[0];
        let numer = len.fX * ab0.fX + ab0.fY * len.fY;
        let t = numer / denom;
        let dist = self.ptAtT(t).distance(xy);
        let largest = self.largestMagnitude();
        RoughlyEqualUlps(largest, largest + dist)
    }
    pub fn ExactPointH(xy: &SkDPoint, left: f64, right: f64, y: f64) -> f64 {
        if xy.fY == y {
            if xy.fX == left {
                return 0.0;
            }
            if xy.fX == right {
                return 1.0;
            }
        }
        -1.0
    }
    pub fn ExactPointV(xy: &SkDPoint, top: f64, bottom: f64, x: f64) -> f64 {
        if xy.fX == x {
            if xy.fY == top {
                return 0.0;
            }
            if xy.fY == bottom {
                return 1.0;
            }
        }
        -1.0
    }
    pub fn NearPointH(xy: &SkDPoint, left: f64, right: f64, y: f64) -> f64 {
        if !AlmostBequalUlps(xy.fY, y) || !AlmostBetweenUlps(left, xy.fX, right) {
            return -1.0;
        }
        let t = SkPinT((xy.fX - left) / (right - left));

        let real = (1.0 - t) * left + t * right;
        let dx = xy.fY - y;
        let dy = xy.fX - real;
        let dist = (dx * dx + dy * dy).sqrt();
        let tiniest = y.min(left).min(right);
        let largest = y.max(left).max(right).max(-tiniest);
        if !AlmostEqualUlps(largest, largest + dist) {
            -1.0
        } else {
            t
        }
    }
    pub fn NearPointV(xy: &SkDPoint, top: f64, bottom: f64, x: f64) -> f64 {
        if !AlmostBequalUlps(xy.fX, x) || !AlmostBetweenUlps(top, xy.fY, bottom) {
            return -1.0;
        }
        let t = SkPinT((xy.fY - top) / (bottom - top));

        let real = (1.0 - t) * top + t * bottom;
        let dx = xy.fX - x;
        let dy = xy.fY - real;
        let dist = (dx * dx + dy * dy).sqrt();
        let tiniest = x.min(top).min(bottom);
        let largest = x.max(top).max(bottom).max(-tiniest);
        if !AlmostEqualUlps(largest, largest + dist) {
            -1.0
        } else {
            t
        }
    }
}
