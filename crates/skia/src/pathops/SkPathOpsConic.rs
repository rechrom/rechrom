// Copyright 2015 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsConic.h/.cpp.
use super::{
    SkPathOpsCubic::SkDCubic,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsQuad::SkDQuad,
    SkPathOpsTypes::*,
};
use crate::src::core::SkPoint::SkPoint;
use core::ops::{Index, IndexMut};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDConic {
    pub fPts: SkDQuad,
    pub fWeight: f32,
}
impl Index<usize> for SkDConic {
    type Output = SkDPoint;
    fn index(&self, n: usize) -> &SkDPoint {
        &self.fPts[n]
    }
}
impl IndexMut<usize> for SkDConic {
    fn index_mut(&mut self, n: usize) -> &mut SkDPoint {
        &mut self.fPts[n]
    }
}
fn deriv(src: [f64; 3], w: f32) -> [f64; 3] {
    let p20 = src[2] - src[0];
    let p10 = src[1] - src[0];
    let wp10 = w as f64 * p10;
    [w as f64 * p20 - p20, p20 - 2.0 * wp10, wp10]
}
fn numerator(src: [f64; 3], w: f32, t: f64) -> f64 {
    let sw = src[1] * w as f64;
    let c = src[0];
    let a = src[2] - 2.0 * sw + c;
    let b = 2.0 * (sw - c);
    (a * t + b) * t + c
}
fn denominator(w: f32, t: f64) -> f64 {
    let b = (2.0_f32 * (w - 1.0)) as f64;
    (-b * t + b) * t + 1.0
}
impl SkDConic {
    pub const kPointCount: usize = 3;
    pub const kPointLast: usize = 2;
    pub const kMaxIntersections: usize = 4;
    pub fn pointCount() -> usize {
        3
    }
    pub fn pointLast() -> usize {
        2
    }
    pub fn maxIntersections() -> usize {
        4
    }
    pub fn IsConic() -> bool {
        true
    }
    pub fn set(&mut self, pts: &[SkPoint; 3], w: f32) -> &Self {
        self.fPts.set(pts);
        self.fWeight = w;
        self
    }
    pub fn collapsed(&self) -> bool {
        self.fPts.collapsed()
    }
    pub fn controlsInside(&self) -> bool {
        self.fPts.controlsInside()
    }
    pub fn flip(&self) -> Self {
        Self {
            fPts: self.fPts.flip(),
            fWeight: self.fWeight,
        }
    }
    pub fn align(&self, end: usize, dst: &mut SkDPoint) {
        self.fPts.align(end, dst)
    }
    pub fn FindExtrema(src: &[f64; 3], w: f32, t: &mut [f64]) -> usize {
        let c = deriv(*src, w);
        let mut ts = [0.0; 2];
        let roots = SkDQuad::RootsValidT(c[0], c[1], c[2], &mut ts);
        if roots == 1 {
            t[0] = ts[0];
            1
        } else {
            0
        }
    }
    pub fn dxdyAtT(&self, t: f64) -> SkDVector {
        let x = deriv(self.fPts.fPts.map(|p| p.fX), self.fWeight);
        let y = deriv(self.fPts.fPts.map(|p| p.fY), self.fWeight);
        let mut result = SkDVector {
            fX: t * (t * x[0] + x[1]) + x[2],
            fY: t * (t * y[0] + y[1]) + y[2],
        };
        if result.fX == 0.0 && result.fY == 0.0 && zero_or_one(t) {
            result = self[2] - self[0];
        }
        result
    }
    pub fn hullIntersectsQuad(&self, q: &SkDQuad, l: &mut bool) -> bool {
        self.fPts.hullIntersectsQuad(q, l)
    }
    pub fn hullIntersectsConic(&self, q: &Self, l: &mut bool) -> bool {
        self.fPts.hullIntersectsQuad(&q.fPts, l)
    }
    pub fn hullIntersectsCubic(&self, q: &SkDCubic, l: &mut bool) -> bool {
        q.hullIntersectsConic(self, l)
    }
    pub fn isLinear(&self, s: usize, e: usize) -> bool {
        self.fPts.isLinear(s, e)
    }
    pub fn monotonicInX(&self) -> bool {
        self.fPts.monotonicInX()
    }
    pub fn monotonicInY(&self) -> bool {
        self.fPts.monotonicInY()
    }
    pub fn otherPts(&self, odd: usize) -> [&SkDPoint; 2] {
        self.fPts.otherPts(odd)
    }
    pub fn AddValidTs(s: &[f64], roots: usize, t: &mut [f64]) -> usize {
        SkDQuad::AddValidTs(s, roots, t)
    }
    pub fn RootsReal(a: f64, b: f64, c: f64, t: &mut [f64]) -> usize {
        SkDQuad::RootsReal(a, b, c, t)
    }
    pub fn RootsValidT(a: f64, b: f64, c: f64, t: &mut [f64]) -> usize {
        SkDQuad::RootsValidT(a, b, c, t)
    }
    pub fn ptAtT(&self, t: f64) -> SkDPoint {
        if t == 0.0 {
            return self[0];
        }
        if t == 1.0 {
            return self[2];
        }
        let d = denominator(self.fWeight, t);
        SkDPoint {
            fX: numerator(self.fPts.fPts.map(|p| p.fX), self.fWeight, t) / d,
            fY: numerator(self.fPts.fPts.map(|p| p.fY), self.fWeight, t) / d,
        }
    }
    pub fn subDivide(&self, t1: f64, t2: f64) -> Self {
        let x = self.fPts.fPts.map(|p| p.fX);
        let y = self.fPts.fPts.map(|p| p.fY);
        let w = self.fWeight;
        let (ax, ay, az) = if t1 == 0.0 {
            (self[0].fX, self[0].fY, 1.0)
        } else if t1 != 1.0 {
            (numerator(x, w, t1), numerator(y, w, t1), denominator(w, t1))
        } else {
            (self[2].fX, self[2].fY, 1.0)
        };
        let mid = (t1 + t2) / 2.0;
        let dx = numerator(x, w, mid);
        let dy = numerator(y, w, mid);
        let dz = denominator(w, mid);
        let (cx, cy, cz) = if t2 == 1.0 {
            (self[2].fX, self[2].fY, 1.0)
        } else if t2 != 0.0 {
            (numerator(x, w, t2), numerator(y, w, t2), denominator(w, t2))
        } else {
            (self[0].fX, self[0].fY, 1.0)
        };
        let bx = 2.0 * dx - (ax + cx) / 2.0;
        let by = 2.0 * dy - (ay + cy) / 2.0;
        let mut bz = 2.0 * dz - (az + cz) / 2.0;
        if bz == 0.0 {
            bz = 1.0;
        }
        Self {
            fPts: SkDQuad {
                fPts: [
                    SkDPoint {
                        fX: ax / az,
                        fY: ay / az,
                    },
                    SkDPoint {
                        fX: bx / bz,
                        fY: by / bz,
                    },
                    SkDPoint {
                        fX: cx / cz,
                        fY: cy / cz,
                    },
                ],
            },
            fWeight: (bz / (az * cz).sqrt()) as f32,
        }
    }
    pub fn SubDivide(p: &[SkPoint; 3], w: f32, t1: f64, t2: f64) -> Self {
        let mut q = Self::default();
        q.set(p, w);
        q.subDivide(t1, t2)
    }
    pub fn subDivideControl(
        &self,
        _a: &SkDPoint,
        _c: &SkDPoint,
        t1: f64,
        t2: f64,
        w: &mut f32,
    ) -> SkDPoint {
        let chopped = self.subDivide(t1, t2);
        *w = chopped.fWeight;
        chopped[1]
    }
    pub fn SubDivideControl(
        p: &[SkPoint; 3],
        weight: f32,
        a: &SkDPoint,
        c: &SkDPoint,
        t1: f64,
        t2: f64,
        w: &mut f32,
    ) -> SkDPoint {
        let mut q = Self::default();
        q.set(p, weight);
        q.subDivideControl(a, c, t1, t2, w)
    }
}

pub use super::SkPathOpsTCurve::SkTConic;
