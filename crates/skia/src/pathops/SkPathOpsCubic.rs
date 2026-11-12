// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsCubic.h/.cpp.
use super::{
    SkLineParameters::SkLineParameters,
    SkPathOpsConic::SkDConic,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsQuad::SkDQuad,
    SkPathOpsTypes::*,
};
use crate::src::core::{
    SkGeometry::{SkClassifyCubic, SkCubicType},
    SkPoint::SkPoint,
    SkTSort::SkTQSortNatural,
};
use core::ops::{Index, IndexMut};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchAxis {
    kXAxis,
    kYAxis,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDCubic {
    pub fPts: [SkDPoint; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDCubicPair {
    pub pts: [SkDPoint; 7],
}
impl SkDCubicPair {
    pub fn first(&self) -> SkDCubic {
        SkDCubic {
            fPts: self.pts[..4].try_into().unwrap(),
        }
    }
    pub fn second(&self) -> SkDCubic {
        SkDCubic {
            fPts: self.pts[3..].try_into().unwrap(),
        }
    }
}
impl Index<usize> for SkDCubic {
    type Output = SkDPoint;
    fn index(&self, n: usize) -> &SkDPoint {
        &self.fPts[n]
    }
}
impl IndexMut<usize> for SkDCubic {
    fn index_mut(&mut self, n: usize) -> &mut SkDPoint {
        &mut self.fPts[n]
    }
}
pub fn other_two(one: usize, two: usize) -> usize {
    (1 >> (3 - (one ^ two))) ^ 3
}
fn derivative(src: [f64; 4], t: f64) -> f64 {
    let one = 1.0 - t;
    3.0 * ((src[1] - src[0]) * one * one
        + 2.0 * (src[2] - src[1]) * t * one
        + (src[3] - src[2]) * t * t)
}
fn interp(s: [f64; 4], t: f64) -> f64 {
    let ab = SkDInterp(s[0], s[1], t);
    let bc = SkDInterp(s[1], s[2], t);
    let cd = SkDInterp(s[2], s[3], t);
    SkDInterp(SkDInterp(ab, bc, t), SkDInterp(bc, cd, t), t)
}
impl SkDCubic {
    pub const kPointCount: usize = 4;
    pub const kPointLast: usize = 3;
    pub const kMaxIntersections: usize = 9;
    pub const gPrecisionUnit: i32 = 256;
    pub fn IsConic() -> bool {
        false
    }
    pub fn pointCount() -> usize {
        4
    }
    pub fn pointLast() -> usize {
        3
    }
    pub fn maxIntersections() -> usize {
        9
    }
    pub fn set(&mut self, p: &[SkPoint; 4]) -> &Self {
        self.fPts = p.map(SkDPoint::from);
        self
    }
    pub fn collapsed(&self) -> bool {
        self[0].approximatelyEqual(self[1])
            && self[0].approximatelyEqual(self[2])
            && self[0].approximatelyEqual(self[3])
    }
    pub fn controlsInside(&self) -> bool {
        let v01 = self[0] - self[1];
        let v02 = self[0] - self[2];
        let v03 = self[0] - self[3];
        let v13 = self[1] - self[3];
        let v23 = self[2] - self[3];
        v03.dot(&v01) > 0.0 && v03.dot(&v02) > 0.0 && v03.dot(&v13) > 0.0 && v03.dot(&v23) > 0.0
    }
    pub fn align(&self, end: usize, ctrl: usize, dst: &mut SkDPoint) {
        if self[end].fX == self[ctrl].fX {
            dst.fX = self[end].fX;
        }
        if self[end].fY == self[ctrl].fY {
            dst.fY = self[end].fY;
        }
    }
    pub fn binarySearch(&self, min: f64, max: f64, intercept: f64, axis: SearchAxis) -> f64 {
        let coord = |p: SkDPoint| {
            if axis == SearchAxis::kXAxis {
                p.fX
            } else {
                p.fY
            }
        };
        let mut t = (min + max) / 2.0;
        let mut step = (t - min) / 2.0;
        let mut at = self.ptAtT(t);
        let mut pos = coord(at);
        let mut dist = pos - intercept;
        loop {
            let prior = min.max(t - step);
            let less = self.ptAtT(prior);
            if approximately_equal_half(less.fX, at.fX) && approximately_equal_half(less.fY, at.fY)
            {
                return -1.0;
            }
            let lessDist = coord(less) - intercept;
            let lastStep = step;
            step /= 2.0;
            if if dist > 0.0 {
                dist > lessDist
            } else {
                dist < lessDist
            } {
                t = prior;
            } else {
                let next = t + lastStep;
                if next > max {
                    return -1.0;
                }
                let more = self.ptAtT(next);
                if approximately_equal_half(more.fX, at.fX)
                    && approximately_equal_half(more.fY, at.fY)
                {
                    return -1.0;
                }
                let moreDist = coord(more) - intercept;
                if if dist > 0.0 {
                    dist <= moreDist
                } else {
                    dist >= moreDist
                } {
                    if approximately_equal(pos, intercept) {
                        return t;
                    }
                    continue;
                }
                t = next;
            }
            at = self.ptAtT(t);
            pos = coord(at);
            dist = pos - intercept;
            if approximately_equal(pos, intercept) {
                return t;
            }
        }
    }
    pub fn calcPrecision(&self) -> f64 {
        ((self[1] - self[0]).length() + (self[2] - self[1]).length() + (self[3] - self[2]).length())
            / Self::gPrecisionUnit as f64
    }
    pub fn chopAt(&self, t: f64) -> SkDCubicPair {
        let mut dst = SkDCubicPair::default();
        for axis in 0..2 {
            let s = self.fPts.map(|p| if axis == 0 { p.fX } else { p.fY });
            let a = if t == 0.5 {
                [
                    s[0],
                    (s[0] + s[1]) / 2.0,
                    (s[0] + 2.0 * s[1] + s[2]) / 4.0,
                    (s[0] + 3.0 * (s[1] + s[2]) + s[3]) / 8.0,
                    (s[1] + 2.0 * s[2] + s[3]) / 4.0,
                    (s[2] + s[3]) / 2.0,
                    s[3],
                ]
            } else {
                let ab = SkDInterp(s[0], s[1], t);
                let bc = SkDInterp(s[1], s[2], t);
                let cd = SkDInterp(s[2], s[3], t);
                let abc = SkDInterp(ab, bc, t);
                let bcd = SkDInterp(bc, cd, t);
                [s[0], ab, abc, SkDInterp(abc, bcd, t), bcd, cd, s[3]]
            };
            for i in 0..7 {
                if axis == 0 {
                    dst.pts[i].fX = a[i];
                } else {
                    dst.pts[i].fY = a[i];
                }
            }
        }
        dst
    }
    pub fn Coefficients(s: &[f64; 4]) -> (f64, f64, f64, f64) {
        let mut a = s[3];
        let mut b = s[2] * 3.0;
        let mut c = s[1] * 3.0;
        let d = s[0];
        a -= d - c + b;
        b += 3.0 * d - 2.0 * c;
        c -= 3.0 * d;
        (a, b, c, d)
    }
    pub fn endsAreExtremaInXOrY(&self) -> bool {
        (between(self[0].fX, self[1].fX, self[3].fX) && between(self[0].fX, self[2].fX, self[3].fX))
            || (between(self[0].fY, self[1].fY, self[3].fY)
                && between(self[0].fY, self[2].fY, self[3].fY))
    }
    pub fn hullIntersectsPoints(&self, p: &[SkDPoint], isLinear: &mut bool) -> bool {
        let mut linear = true;
        let mut order = [0; 4];
        let count = self.convexHull(&mut order);
        let mut end1 = order[0];
        let mut hi = 0;
        loop {
            hi = (hi + 1) % count;
            let end2 = order[hi];
            let ox = self[end1].fX;
            let oy = self[end1].fY;
            let adj = self[end2].fX - ox;
            let opp = self[end2].fY - oy;
            let mask = other_two(end1, end2);
            let odd = end1 ^ mask;
            let odd2 = end2 ^ mask;
            let mut sign = (self[odd].fY - oy) * adj - (self[odd].fX - ox) * opp;
            let sign2 = (self[odd2].fY - oy) * adj - (self[odd2].fX - ox) * opp;
            if sign * sign2 < 0.0 {
                if hi == 0 {
                    break;
                }
                continue;
            }
            if approximately_zero(sign) {
                sign = sign2;
                if approximately_zero(sign) {
                    if hi == 0 {
                        break;
                    }
                    continue;
                }
            }
            linear = false;
            let mut outlier = false;
            for p in p {
                let test = (p.fY - oy) * adj - (p.fX - ox) * opp;
                if test * sign > 0.0 && !precisely_zero(test) {
                    outlier = true;
                    break;
                }
            }
            if !outlier {
                return false;
            }
            end1 = end2;
            if hi == 0 {
                break;
            }
        }
        *isLinear = linear;
        true
    }
    pub fn hullIntersectsQuad(&self, q: &SkDQuad, l: &mut bool) -> bool {
        self.hullIntersectsPoints(&q.fPts, l)
    }
    pub fn hullIntersectsConic(&self, q: &SkDConic, l: &mut bool) -> bool {
        self.hullIntersectsQuad(&q.fPts, l)
    }
    pub fn hullIntersectsCubic(&self, q: &Self, l: &mut bool) -> bool {
        self.hullIntersectsPoints(&q.fPts, l)
    }
    pub fn isLinear(&self, s: usize, e: usize) -> bool {
        if self[0].approximatelyDEqual(self[3]) {
            return SkDQuad {
                fPts: [self[0], self[1], self[2]],
            }
            .isLinear(0, 2);
        }
        let mut line = SkLineParameters::default();
        line.cubicEndPointsRange(self, s, e);
        line.normalize();
        let tiny = self
            .fPts
            .iter()
            .fold(self[0].fX, |a, p| a.min(p.fX).min(p.fY));
        let largest = self
            .fPts
            .iter()
            .fold(self[0].fX, |a, p| a.max(p.fX).max(p.fY))
            .max(-tiny);
        if !approximately_zero_when_compared_to(line.controlPtDistanceCubic(self, 1), largest) {
            return false;
        }
        approximately_zero_when_compared_to(line.controlPtDistanceCubic(self, 2), largest)
    }
    pub fn ComplexBreak(points: &[SkPoint; 4], t: &mut [f32; 3]) -> usize {
        let mut cubic = Self::default();
        cubic.set(points);
        if cubic.monotonicInX() && cubic.monotonicInY() {
            return 0;
        }
        let mut tt = [0.0; 2];
        let mut ss = [0.0; 2];
        let kind = SkClassifyCubic(points, Some(&mut tt), Some(&mut ss), None);
        if kind == SkCubicType::kLoop
            && roughly_between(0.0, tt[0], ss[0])
            && roughly_between(0.0, tt[1], ss[1])
        {
            t[0] = ((tt[0] * ss[1] + tt[1] * ss[0]) / (2.0 * ss[0] * ss[1])) as f32;
            return (t[0] > 0.0 && t[0] < 1.0) as usize;
        }
        if matches!(
            kind,
            SkCubicType::kLoop
                | SkCubicType::kSerpentine
                | SkCubicType::kLocalCusp
                | SkCubicType::kCuspAtInfinity
        ) {
            let mut inf = [0.0; 2];
            let infCount = cubic.findInflections(&mut inf);
            let mut curvature = [0.0; 3];
            let roots = cubic.findMaxCurvature(&mut curvature);
            if infCount == 2 {
                for &v in &curvature[..roots] {
                    if between(inf[0], v, inf[1]) {
                        t[0] = v as f32;
                        return (t[0] > 0.0 && t[0] < 1.0) as usize;
                    }
                }
            } else {
                let mut result = 0;
                let precision = cubic.calcPrecision() * 2.0;
                for &v in &curvature[..roots] {
                    if v <= 0.0 || v >= 1.0 {
                        continue;
                    }
                    let d = SkDVector {
                        fX: derivative(cubic.fPts.map(|p| p.fX), v),
                        fY: derivative(cubic.fPts.map(|p| p.fY), v),
                    };
                    if d.length() < precision {
                        t[result] = v as f32;
                        result += 1;
                    }
                }
                if result == 0 && infCount == 1 {
                    t[0] = inf[0] as f32;
                    result = (t[0] > 0.0 && t[0] < 1.0) as usize;
                }
                return result;
            }
        }
        0
    }
    pub fn monotonicInX(&self) -> bool {
        precisely_between(self[0].fX, self[1].fX, self[3].fX)
            && precisely_between(self[0].fX, self[2].fX, self[3].fX)
    }
    pub fn monotonicInY(&self) -> bool {
        precisely_between(self[0].fY, self[1].fY, self[3].fY)
            && precisely_between(self[0].fY, self[2].fY, self[3].fY)
    }
    pub fn otherPts(&self, index: usize) -> [&SkDPoint; 3] {
        let offset = (index == 0) as usize;
        [&self[offset], &self[offset + 1], &self[offset + 2]]
    }
    pub fn searchRoots(
        &self,
        extreme: &mut [f64; 6],
        mut count: usize,
        intercept: f64,
        axis: SearchAxis,
        valid: &mut [f64],
    ) -> usize {
        count += self.findInflections(&mut extreme[count..]);
        extreme[count] = 0.0;
        count += 1;
        extreme[count] = 1.0;

        SkTQSortNatural(&mut extreme[..=count]);
        let mut found = 0;
        for index in 0..count {
            let min = extreme[index];
            let max = extreme[index + 1];
            if min == max {
                continue;
            }
            let t = self.binarySearch(min, max, intercept, axis);
            if t >= 0.0 {
                if found >= 3 {
                    return 0;
                }
                valid[found] = t;
                found += 1;
            }
        }
        found
    }
    pub fn RootsValidT(a: f64, b: f64, c: f64, d: f64, t: &mut [f64]) -> usize {
        let mut s = [0.0; 3];
        let roots = Self::RootsReal(a, b, c, d, &mut s);
        let mut found = SkDQuad::AddValidTs(&s, roots, t);
        'root: for &v in &s[..roots] {
            let value = if !approximately_one_or_less(v) && between(1.0, v, 1.00005) {
                1.0
            } else if !approximately_zero_or_more(v) && between(-0.00005, v, 0.0) {
                0.0
            } else {
                continue;
            };
            for &old in &t[..found] {
                if approximately_equal(old, value) {
                    continue 'root;
                }
            }

            t[found] = value;
            found += 1;
        }
        found
    }
    pub fn RootsReal(mut a: f64, b: f64, c: f64, d: f64, s: &mut [f64]) -> usize {
        if approximately_zero(a)
            && approximately_zero_when_compared_to(a, b)
            && approximately_zero_when_compared_to(a, c)
            && approximately_zero_when_compared_to(a, d)
        {
            return SkDQuad::RootsReal(b, c, d, s);
        }
        if approximately_zero_when_compared_to(d, a)
            && approximately_zero_when_compared_to(d, b)
            && approximately_zero_when_compared_to(d, c)
        {
            let n = SkDQuad::RootsReal(a, b, c, s);
            if s[..n].iter().any(|&v| approximately_zero(v)) {
                return n;
            }
            s[n] = 0.0;
            return n + 1;
        }
        if approximately_zero(a + b + c + d) {
            let n = SkDQuad::RootsReal(a, a + b, -d, s);
            if s[..n].iter().any(|&v| AlmostDequalUlps(v, 1.0)) {
                return n;
            }
            s[n] = 1.0;
            return n + 1;
        }
        let inv = 1.0 / a;
        let aa = b * inv;
        let bb = c * inv;
        let cc = d * inv;
        let a2 = aa * aa;
        let q = (a2 - bb * 3.0) / 9.0;
        let r = (2.0 * a2 * aa - 9.0 * aa * bb + 27.0 * cc) / 54.0;
        let r2 = r * r;
        let q3 = q * q * q;
        let delta = r2 - q3;
        let adiv3 = aa / 3.0;
        let mut n = 0;
        if delta < 0.0 {
            let theta = (r / q3.sqrt()).clamp(-1.0, 1.0).acos();
            let neg = -2.0 * q.sqrt();
            s[n] = neg * (theta / 3.0).cos() - adiv3;
            n += 1;
            let v = neg * ((theta + 2.0 * core::f64::consts::PI) / 3.0).cos() - adiv3;
            if !AlmostDequalUlps(s[0], v) {
                s[n] = v;
                n += 1;
            }
            let v = neg * ((theta - 2.0 * core::f64::consts::PI) / 3.0).cos() - adiv3;
            if !AlmostDequalUlps(s[0], v) && (n == 1 || !AlmostDequalUlps(s[1], v)) {
                s[n] = v;
                n += 1;
            }
        } else {
            a = (r.abs() + delta.sqrt()).cbrt();
            if r > 0.0 {
                a = -a;
            }
            if a != 0.0 {
                a += q / a;
            }
            s[n] = a - adiv3;
            n += 1;
            if AlmostDequalUlps(r2, q3) {
                let v = -a / 2.0 - adiv3;
                if !AlmostDequalUlps(s[0], v) {
                    s[n] = v;
                    n += 1;
                }
            }
        }
        n
    }
    pub fn dxdyAtT(&self, t: f64) -> SkDVector {
        let mut v = SkDVector {
            fX: derivative(self.fPts.map(|p| p.fX), t),
            fY: derivative(self.fPts.map(|p| p.fY), t),
        };
        if v.fX == 0.0 && v.fY == 0.0 {
            if t == 0.0 {
                v = self[2] - self[0];
            } else if t == 1.0 {
                v = self[3] - self[1];
            }
            if v.fX == 0.0 && v.fY == 0.0 && zero_or_one(t) {
                v = self[3] - self[0];
            }
        }
        v
    }
    pub fn findInflections(&self, t: &mut [f64]) -> usize {
        let ax = self[1].fX - self[0].fX;
        let ay = self[1].fY - self[0].fY;
        let bx = self[2].fX - 2.0 * self[1].fX + self[0].fX;
        let by = self[2].fY - 2.0 * self[1].fY + self[0].fY;
        let cx = self[3].fX + 3.0 * (self[1].fX - self[2].fX) - self[0].fX;
        let cy = self[3].fY + 3.0 * (self[1].fY - self[2].fY) - self[0].fY;
        SkDQuad::RootsValidT(bx * cy - by * cx, ax * cy - ay * cx, ax * by - ay * bx, t)
    }
    pub fn FindInflections(p: &[SkPoint; 4], t: &mut [f64]) -> usize {
        let mut c = Self::default();
        c.set(p);
        c.findInflections(t)
    }
    pub fn FindExtrema(s: &[f64; 4], t: &mut [f64]) -> usize {
        let a = s[0];
        let b = s[1];
        let c = s[2];
        let d = s[3];
        SkDQuad::RootsValidT(d - a + 3.0 * (b - c), 2.0 * (a - b - b + c), b - a, t)
    }
    pub fn findMaxCurvature(&self, t: &mut [f64]) -> usize {
        fn coeff(s: [f64; 4]) -> [f64; 4] {
            let a = s[1] - s[0];
            let b = s[2] - 2.0 * s[1] + s[0];
            let c = s[3] + 3.0 * (s[1] - s[2]) - s[0];
            [c * c, 3.0 * b * c, 2.0 * b * b + c * a, a * b]
        }
        let mut x = coeff(self.fPts.map(|p| p.fX));
        let y = coeff(self.fPts.map(|p| p.fY));
        for i in 0..4 {
            x[i] += y[i];
        }
        Self::RootsValidT(x[0], x[1], x[2], x[3], t)
    }
    pub fn ptAtT(&self, t: f64) -> SkDPoint {
        if t == 0.0 {
            return self[0];
        }
        if t == 1.0 {
            return self[3];
        }
        let one = 1.0 - t;
        let one2 = one * one;
        let a = one2 * one;
        let b = 3.0 * one2 * t;
        let t2 = t * t;
        let c = 3.0 * one * t2;
        let d = t2 * t;
        SkDPoint {
            fX: a * self[0].fX + b * self[1].fX + c * self[2].fX + d * self[3].fX,
            fY: a * self[0].fY + b * self[1].fY + c * self[2].fY + d * self[3].fY,
        }
    }
    pub fn subDivide(&self, t1: f64, t2: f64) -> Self {
        if t1 == 0.0 || t2 == 1.0 {
            if t1 == 0.0 && t2 == 1.0 {
                return *self;
            }
            let pair = self.chopAt(if t1 == 0.0 { t2 } else { t1 });
            return if t1 == 0.0 {
                pair.first()
            } else {
                pair.second()
            };
        }
        let mut dst = Self::default();
        for axis in 0..2 {
            let s = self.fPts.map(|p| if axis == 0 { p.fX } else { p.fY });
            let a = interp(s, t1);
            let e = interp(s, (t1 * 2.0 + t2) / 3.0);
            let f = interp(s, (t1 + t2 * 2.0) / 3.0);
            let d = interp(s, t2);
            let m = e * 27.0 - a * 8.0 - d;
            let n = f * 27.0 - a - d * 8.0;
            let values = [a, (m * 2.0 - n) / 18.0, (n * 2.0 - m) / 18.0, d];
            for i in 0..4 {
                if axis == 0 {
                    dst[i].fX = values[i];
                } else {
                    dst[i].fY = values[i];
                }
            }
        }
        dst
    }
    pub fn SubDivide(p: &[SkPoint; 4], t1: f64, t2: f64) -> Self {
        let mut c = Self::default();
        c.set(p);
        c.subDivide(t1, t2)
    }
    pub fn subDivideControl(
        &self,
        a: &SkDPoint,
        d: &SkDPoint,
        t1: f64,
        t2: f64,
        dst: &mut [SkDPoint; 2],
    ) {
        let sub = self.subDivide(t1, t2);
        dst[0] = sub[1] + (*a - sub[0]);
        dst[1] = sub[2] + (*d - sub[3]);
        if t1 == 0.0 || t2 == 0.0 {
            self.align(0, 1, &mut dst[if t1 == 0.0 { 0 } else { 1 }]);
        }
        if t1 == 1.0 || t2 == 1.0 {
            self.align(3, 2, &mut dst[if t1 == 1.0 { 0 } else { 1 }]);
        }
        if AlmostBequalUlps(dst[0].fX, a.fX) {
            dst[0].fX = a.fX;
        }
        if AlmostBequalUlps(dst[0].fY, a.fY) {
            dst[0].fY = a.fY;
        }
        if AlmostBequalUlps(dst[1].fX, d.fX) {
            dst[1].fX = d.fX;
        }
        if AlmostBequalUlps(dst[1].fY, d.fY) {
            dst[1].fY = d.fY;
        }
    }
    pub fn SubDivideControl(
        p: &[SkPoint; 4],
        a: &SkDPoint,
        d: &SkDPoint,
        t1: f64,
        t2: f64,
        dst: &mut [SkDPoint; 2],
    ) {
        let mut c = Self::default();
        c.set(p);
        c.subDivideControl(a, d, t1, t2, dst)
    }
    pub fn toFloatPoints(&self, pts: &mut [SkPoint; 4]) -> bool {
        for i in 0..4 {
            pts[i] = self[i].asSkPoint();
            if pts[i].fX.abs() < FLT_EPSILON_ORDERABLE_ERR as f32 {
                pts[i].fX = 0.0;
            }
            if pts[i].fY.abs() < FLT_EPSILON_ORDERABLE_ERR as f32 {
                pts[i].fY = 0.0;
            }
        }
        pts.iter().all(|p| p.fX.is_finite() && p.fY.is_finite())
    }
    pub fn top(&self, dCurve: &Self, start: f64, end: f64, pt: &mut SkDPoint) -> f64 {
        let mut ts = [0.0; 2];
        let roots = Self::FindExtrema(&self.fPts.map(|p| p.fY), &mut ts);
        let mut top = -1.0;
        for &v in &ts[..roots] {
            let t = start + (end - start) * v;
            let mid = dCurve.ptAtT(t);
            if pt.fY > mid.fY || (pt.fY == mid.fY && pt.fX > mid.fX) {
                top = t;
                *pt = mid;
            }
        }
        top
    }
    pub fn toQuad(&self) -> SkDQuad {
        let c1 = SkDPoint {
            fX: (3.0 * self[1].fX - self[0].fX) / 2.0,
            fY: (3.0 * self[1].fY - self[0].fY) / 2.0,
        };
        let c2 = SkDPoint {
            fX: (3.0 * self[2].fX - self[3].fX) / 2.0,
            fY: (3.0 * self[2].fY - self[3].fY) / 2.0,
        };
        SkDQuad {
            fPts: [self[0], SkDPoint::Mid(&c1, &c2), self[3]],
        }
    }
}

pub use super::SkPathOpsTCurve::SkTCubic;
