// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsQuad.h/.cpp.
use super::{
    SkIntersections::SkIntersections,
    SkLineParameters::SkLineParameters,
    SkPathOpsConic::SkDConic,
    SkPathOpsCubic::SkDCubic,
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsTypes::*,
};
use crate::src::core::SkPoint::SkPoint;
use core::ops::{Index, IndexMut};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDQuad {
    pub fPts: [SkDPoint; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDQuadPair {
    pub pts: [SkDPoint; 5],
}
impl SkDQuadPair {
    pub fn first(&self) -> &SkDQuad {
        unsafe { &*self.pts.as_ptr().cast::<SkDQuad>() }
    }
    pub fn second(&self) -> &SkDQuad {
        unsafe { &*self.pts.as_ptr().add(2).cast::<SkDQuad>() }
    }
}
impl Index<usize> for SkDQuad {
    type Output = SkDPoint;
    fn index(&self, n: usize) -> &SkDPoint {
        &self.fPts[n]
    }
}
impl IndexMut<usize> for SkDQuad {
    fn index_mut(&mut self, n: usize) -> &mut SkDPoint {
        &mut self.fPts[n]
    }
}
fn pointInTriangle(p: &[SkDPoint; 3], test: &SkDPoint) -> bool {
    let v0 = p[2] - p[0];
    let v1 = p[1] - p[0];
    let v2 = *test - p[0];
    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d02 = v0.dot(&v2);
    let d11 = v1.dot(&v1);
    let d12 = v1.dot(&v2);
    let denom = d00 * d11 - d01 * d01;
    let u = d11 * d02 - d01 * d12;
    let v = d00 * d12 - d01 * d02;
    if denom >= 0.0 {
        u >= 0.0 && v >= 0.0 && u + v < denom
    } else {
        u <= 0.0 && v <= 0.0 && u + v > denom
    }
}
fn interp(src: [f64; 3], t: f64) -> f64 {
    if t == 0.0 {
        return src[0];
    }
    if t == 1.0 {
        return src[2];
    }
    SkDInterp(
        SkDInterp(src[0], src[1], t),
        SkDInterp(src[1], src[2], t),
        t,
    )
}
impl SkDQuad {
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
        false
    }
    pub fn set(&mut self, p: &[SkPoint; 3]) -> &Self {
        self.fPts = p.map(SkDPoint::from);
        self
    }
    pub fn collapsed(&self) -> bool {
        self[0].approximatelyEqual(self[1]) && self[0].approximatelyEqual(self[2])
    }
    pub fn controlsInside(&self) -> bool {
        let v01 = self[0] - self[1];
        let v02 = self[0] - self[2];
        let v12 = self[1] - self[2];
        v02.dot(&v01) > 0.0 && v02.dot(&v12) > 0.0
    }
    pub fn flip(&self) -> Self {
        Self {
            fPts: [self[2], self[1], self[0]],
        }
    }
    pub fn otherPts(&self, oddMan: usize) -> [&SkDPoint; 2] {
        let index = |opp: usize| {
            let e = (oddMan ^ opp) as isize - oddMan as isize;
            (e & !(e >> 2)) as usize
        };
        [&self[index(1)], &self[index(2)]]
    }
    pub fn hullIntersectsQuad(&self, q2: &Self, isLinear: &mut bool) -> bool {
        let mut linear = true;
        for odd in 0..3 {
            let ends = self.otherPts(odd);
            let ox = ends[0].fX;
            let oy = ends[0].fY;
            let adj = ends[1].fX - ox;
            let opp = ends[1].fY - oy;
            let sign = (self[odd].fY - oy) * adj - (self[odd].fX - ox) * opp;
            if approximately_zero(sign) {
                continue;
            }
            linear = false;
            let mut found = false;
            for p in q2.fPts {
                let test = (p.fY - oy) * adj - (p.fX - ox) * opp;
                if test * sign > 0.0 && !precisely_zero(test) {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        if linear && self[0] != q2[0] && self[2] != q2[0] && self[0] != q2[2] && self[2] != q2[2] {
            if pointInTriangle(&self.fPts, &q2[0]) || pointInTriangle(&self.fPts, &q2[2]) {
                linear = false;
            }
        }
        *isLinear = linear;
        true
    }
    pub fn hullIntersectsConic(&self, q: &SkDConic, l: &mut bool) -> bool {
        q.hullIntersectsQuad(self, l)
    }
    pub fn hullIntersectsCubic(&self, c: &SkDCubic, l: &mut bool) -> bool {
        c.hullIntersectsQuad(self, l)
    }
    pub fn AddValidTs(s: &[f64], realRoots: usize, t: &mut [f64]) -> usize {
        let mut found = 0;
        'root: for &value in &s[..realRoots] {
            let mut value = value;
            if approximately_zero_or_more(value) && approximately_one_or_less(value) {
                if approximately_less_than_zero(value) {
                    value = 0.0;
                } else if approximately_greater_than_one(value) {
                    value = 1.0;
                }
                for &old in &t[..found] {
                    if approximately_equal(old, value) {
                        continue 'root;
                    }
                }
                t[found] = value;
                found += 1;
            }
        }
        found
    }
    pub fn RootsValidT(a: f64, b: f64, c: f64, t: &mut [f64]) -> usize {
        let mut s = [0.0; 2];
        let n = Self::RootsReal(a, b, c, &mut s);
        Self::AddValidTs(&s, n, t)
    }
    pub fn RootsReal(a: f64, b: f64, c: f64, s: &mut [f64]) -> usize {
        fn zero(b: f64, c: f64, s: &mut [f64]) -> usize {
            if approximately_zero(b) {
                s[0] = 0.0;
                return (c == 0.0) as usize;
            }
            s[0] = -c / b;
            1
        }
        if a == 0.0 {
            return zero(b, c, s);
        }
        let p = b / (2.0 * a);
        let q = c / a;
        if approximately_zero(a) && (approximately_zero_inverse(p) || approximately_zero_inverse(q))
        {
            return zero(b, c, s);
        }
        let p2 = p * p;
        if !AlmostDequalUlps(p2, q) && p2 < q {
            return 0;
        }
        let sqrt = if p2 > q { (p2 - q).sqrt() } else { 0.0 };
        s[0] = sqrt - p;
        s[1] = -sqrt - p;
        1 + (!AlmostDequalUlps(s[0], s[1])) as usize
    }
    pub fn isLinear(&self, start: usize, end: usize) -> bool {
        let mut line = SkLineParameters::default();
        line.quadEndPointsRange(self, start, end);
        line.normalize();
        let distance = line.controlPtDistanceQuad(self);
        let tiny = self
            .fPts
            .iter()
            .fold(self[0].fX, |a, p| a.min(p.fX).min(p.fY));
        let largest = self
            .fPts
            .iter()
            .fold(self[0].fX, |a, p| a.max(p.fX).max(p.fY))
            .max(-tiny);
        approximately_zero_when_compared_to(distance, largest)
    }
    pub fn dxdyAtT(&self, t: f64) -> SkDVector {
        let a = t - 1.0;
        let b = 1.0 - 2.0 * t;
        let mut result = SkDVector {
            fX: a * self[0].fX + b * self[1].fX + t * self[2].fX,
            fY: a * self[0].fY + b * self[1].fY + t * self[2].fY,
        };
        if result.fX == 0.0 && result.fY == 0.0 && zero_or_one(t) {
            result = self[2] - self[0];
        }
        result
    }
    pub fn ptAtT(&self, t: f64) -> SkDPoint {
        if t == 0.0 {
            return self[0];
        }
        if t == 1.0 {
            return self[2];
        }
        let one = 1.0 - t;
        let a = one * one;
        let b = 2.0 * one * t;
        let c = t * t;
        SkDPoint {
            fX: a * self[0].fX + b * self[1].fX + c * self[2].fX,
            fY: a * self[0].fY + b * self[1].fY + c * self[2].fY,
        }
    }
    pub fn monotonicInX(&self) -> bool {
        between(self[0].fX, self[1].fX, self[2].fX)
    }
    pub fn monotonicInY(&self) -> bool {
        between(self[0].fY, self[1].fY, self[2].fY)
    }
    pub fn subDivide(&self, t1: f64, t2: f64) -> Self {
        if t1 == 0.0 && t2 == 1.0 {
            return *self;
        }
        let x = self.fPts.map(|p| p.fX);
        let y = self.fPts.map(|p| p.fY);
        let a = SkDPoint {
            fX: interp(x, t1),
            fY: interp(y, t1),
        };
        let c = SkDPoint {
            fX: interp(x, t2),
            fY: interp(y, t2),
        };
        let d = SkDPoint {
            fX: interp(x, (t1 + t2) / 2.0),
            fY: interp(y, (t1 + t2) / 2.0),
        };
        let b = SkDPoint {
            fX: 2.0 * d.fX - (a.fX + c.fX) / 2.0,
            fY: 2.0 * d.fY - (a.fY + c.fY) / 2.0,
        };
        Self { fPts: [a, b, c] }
    }
    pub fn SubDivide(p: &[SkPoint; 3], t1: f64, t2: f64) -> Self {
        let mut q = Self::default();
        q.set(p);
        q.subDivide(t1, t2)
    }
    pub fn align(&self, end: usize, dst: &mut SkDPoint) {
        if self[end].fX == self[1].fX {
            dst.fX = self[end].fX;
        }
        if self[end].fY == self[1].fY {
            dst.fY = self[end].fY;
        }
    }
    pub fn subDivideControl(&self, a: &SkDPoint, c: &SkDPoint, t1: f64, t2: f64) -> SkDPoint {
        let sub = self.subDivide(t1, t2);
        let b0 = SkDLine {
            fPts: [*a, sub[1] + (*a - sub[0])],
        };
        let b1 = SkDLine {
            fPts: [*c, sub[1] + (*c - sub[2])],
        };
        let mut i = SkIntersections::default();
        i.intersectRayLine(&b0, &b1);
        let mut b = if i.used() == 1 && i[0][0] >= 0.0 && i[1][0] >= 0.0 {
            *i.pt(0)
        } else {
            return SkDPoint::Mid(&b0[1], &b1[1]);
        };
        if t1 == 0.0 || t2 == 0.0 {
            self.align(0, &mut b);
        }
        if t1 == 1.0 || t2 == 1.0 {
            self.align(2, &mut b);
        }
        if AlmostBequalUlps(b.fX, a.fX) {
            b.fX = a.fX;
        } else if AlmostBequalUlps(b.fX, c.fX) {
            b.fX = c.fX;
        }
        if AlmostBequalUlps(b.fY, a.fY) {
            b.fY = a.fY;
        } else if AlmostBequalUlps(b.fY, c.fY) {
            b.fY = c.fY;
        }
        b
    }
    pub fn SubDivideControl(
        p: &[SkPoint; 3],
        a: &SkDPoint,
        c: &SkDPoint,
        t1: f64,
        t2: f64,
    ) -> SkDPoint {
        let mut q = Self::default();
        q.set(p);
        q.subDivideControl(a, c, t1, t2)
    }
    pub fn chopAt(&self, t: f64) -> SkDQuadPair {
        let ab = SkDPoint {
            fX: SkDInterp(self[0].fX, self[1].fX, t),
            fY: SkDInterp(self[0].fY, self[1].fY, t),
        };
        let bc = SkDPoint {
            fX: SkDInterp(self[1].fX, self[2].fX, t),
            fY: SkDInterp(self[1].fY, self[2].fY, t),
        };
        let abc = SkDPoint {
            fX: SkDInterp(ab.fX, bc.fX, t),
            fY: SkDInterp(ab.fY, bc.fY, t),
        };
        SkDQuadPair {
            pts: [self[0], ab, abc, bc, self[2]],
        }
    }
    pub fn FindExtrema(src: &[f64; 3], t: &mut [f64]) -> usize {
        let mut numer = src[0] - src[1];
        let mut denom = src[0] - src[1] - src[1] + src[2];
        if numer < 0.0 {
            numer = -numer;
            denom = -denom;
        }
        if denom == 0.0 || numer == 0.0 || numer >= denom {
            return 0;
        }
        let r = numer / denom;
        if r == 0.0 {
            return 0;
        }
        t[0] = r;
        1
    }
    pub fn SetABC(quad: &[f64; 3]) -> (f64, f64, f64) {
        let mut a = quad[0];
        let mut b = 2.0 * quad[1];
        let c = quad[2];
        b -= c;
        a -= b;
        b -= c;
        (a, b, c)
    }
}

pub use super::SkPathOpsTCurve::SkTQuad;
