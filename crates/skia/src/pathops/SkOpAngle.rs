// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkOpAngle.h/.cpp. Native release angle ordering.
#![allow(non_snake_case)]
use super::{
    SkIntersections::SkIntersections,
    SkLineParameters::SkLineParameters,
    SkOpSegment::SkOpSegment,
    SkOpSpan::{SkOpSpan, SkOpSpanBase},
    SkPathOpsCubic::SkDCubic,
    SkPathOpsCurve::{CurveDPointAtT, CurveDSlopeAtT, CurveIntersectRay, SkDCurve, SkDCurveSweep},
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsTypes::*,
};
use crate::src::core::{SkPath::Verb, SkPoint::SkPoint, SkTSort::SkTQSortNatural};
use core::ptr::null_mut;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncludeType {
    kUnaryWinding,
    kUnaryXor,
    kBinarySingle,
    kBinaryOpp,
}
#[derive(Default)]
pub struct SkOpAngle {
    fOriginalCurvePart: SkDCurve,
    fPart: SkDCurveSweep,
    fSide: f64,
    fTangentHalf: SkLineParameters,
    fNext: *mut SkOpAngle,
    fLastMarked: *mut SkOpSpanBase,
    fStart: *mut SkOpSpanBase,
    fEnd: *mut SkOpSpanBase,
    fComputedEnd: *mut SkOpSpanBase,
    fSectorMask: u32,
    fSectorStart: i32,
    fSectorEnd: i32,
    fUnorderable: bool,
    fComputeSector: bool,
    fComputedSector: bool,
    fCheckCoincidence: bool,
    fTangentsAmbiguous: bool,
}
impl SkOpAngle {
    pub fn end(&self) -> *mut SkOpSpanBase {
        self.fEnd
    }
    pub fn start(&self) -> *mut SkOpSpanBase {
        self.fStart
    }
    pub fn next(&self) -> *mut Self {
        self.fNext
    }
    pub fn setLastMarked(&mut self, marked: *mut SkOpSpanBase) {
        self.fLastMarked = marked;
    }
    pub fn tangentsAmbiguous(&self) -> bool {
        self.fTangentsAmbiguous
    }
    pub fn unorderable(&self) -> bool {
        self.fUnorderable
    }
    pub unsafe fn segment(&self) -> *mut SkOpSegment {
        (*self.fStart).segment()
    }
    pub unsafe fn starter(&mut self) -> *mut SkOpSpan {
        (*self.fStart).starter(self.fEnd)
    }
    unsafe fn globalState(&self) -> *mut SkOpGlobalState {
        (*self.segment()).globalState()
    }
    unsafe fn midT(&self) -> f64 {
        ((*self.fStart).t() + (*self.fEnd).t()) / 2.0
    }
    fn checkCrossesZero(&self) -> bool {
        (self.fSectorEnd - self.fSectorStart).abs() > 16
    }
    fn oppositePlanes(&self, rh: &Self) -> bool {
        (rh.fSectorStart - self.fSectorStart).abs() >= 8
    }

    pub unsafe fn set(&mut self, start: *mut SkOpSpanBase, end: *mut SkOpSpanBase) {
        self.fStart = start;
        self.fComputedEnd = end;
        self.fEnd = end;

        self.fNext = null_mut();
        self.fComputeSector = false;
        self.fComputedSector = false;
        self.fCheckCoincidence = false;
        self.fTangentsAmbiguous = false;
        self.setSpans();
        self.setSector();
    }
    pub unsafe fn lastMarked(&self) -> *mut SkOpSpanBase {
        if !self.fLastMarked.is_null() {
            if (*self.fLastMarked).chased() {
                return null_mut();
            }
            (*self.fLastMarked).setChased(true);
        }
        self.fLastMarked
    }
    pub unsafe fn loopCount(&self) -> i32 {
        let first = self as *const Self as *mut Self;
        let mut next = first;
        let mut count = 0;
        loop {
            next = (*next).fNext;
            count += 1;
            if next.is_null() || next == first {
                return count;
            }
        }
    }
    pub unsafe fn loopContains(&self, angle: *const Self) -> bool {
        if self.fNext.is_null() {
            return false;
        }
        let first = self as *const Self;
        let mut current = first;
        let tSegment = (*(*angle).fStart).segment();
        let tStart = (*(*angle).fStart).t();
        let tEnd = (*(*angle).fEnd).t();
        loop {
            let lSegment = (*(*current).fStart).segment();
            if lSegment == tSegment
                && (*(*current).fStart).t() == tEnd
                && (*(*current).fEnd).t() == tStart
            {
                return true;
            }
            current = (*current).fNext;
            if current == first {
                return false;
            }
        }
    }
    pub unsafe fn previous(&self) -> *mut Self {
        let this = self as *const Self as *mut Self;
        let mut last = self.fNext;
        loop {
            let next = (*last).fNext;
            if next == this {
                return last;
            }
            last = next;
        }
    }
    pub unsafe fn insert(&mut self, angle: *mut Self) -> bool {
        if !(*angle).fNext.is_null() {
            if self.loopCount() >= (*angle).loopCount() {
                if !self.merge(angle) {
                    return true;
                }
            } else if !self.fNext.is_null() {
                if !(*angle).merge(self) {
                    return true;
                }
            } else {
                (*angle).insert(self);
            }
            return true;
        }
        let this = self as *mut Self;
        let singleton = self.fNext.is_null();
        if singleton {
            self.fNext = this;
        }
        let mut next = self.fNext;
        if (*next).fNext == this {
            if singleton || (*angle).after(this) {
                self.fNext = angle;
                (*angle).fNext = next;
            } else {
                (*next).fNext = angle;
                (*angle).fNext = this;
            }
            return true;
        }
        let mut last = this;
        let mut flipAmbiguity = false;
        loop {
            if (*angle).after(last) ^ ((*angle).tangentsAmbiguous() & flipAmbiguity) {
                (*last).fNext = angle;
                (*angle).fNext = next;
                return true;
            }
            last = next;
            if last == this {
                if flipAmbiguity {
                    return false;
                }
                flipAmbiguity = true;
            }
            next = (*next).fNext;
        }
    }
    unsafe fn merge(&mut self, angle: *mut Self) -> bool {
        let this = self as *mut Self;

        let mut working = angle;
        loop {
            if working == this {
                return false;
            }
            working = (*working).fNext;
            if working == angle {
                break;
            }
        }
        loop {
            let next = (*working).fNext;
            (*working).fNext = null_mut();
            self.insert(working);
            working = next;
            if working == angle {
                return true;
            }
        }
    }
    // Source: SkOpAngle.cpp:72-232. The three-way comparator preserves native
    // ambiguity handling, including original-position line checks.
    unsafe fn after(&mut self, test: *mut Self) -> bool {
        let this = self as *mut Self;
        let lh = test;
        let rh = (*lh).fNext;

        self.fPart.fCurve = self.fOriginalCurvePart;
        (*lh).fPart.fCurve = (*lh).fOriginalCurvePart;
        (&mut (*lh).fPart.fCurve)[0] = self.fPart.fCurve[0];
        (*rh).fPart.fCurve = (*rh).fOriginalCurvePart;
        (&mut (*rh).fPart.fCurve)[0] = self.fPart.fCurve[0];
        if (*lh).fComputeSector && !(*lh).computeSector() {
            return true;
        }
        if self.fComputeSector && !self.computeSector() {
            return true;
        }
        if (*rh).fComputeSector && !(*rh).computeSector() {
            return true;
        }
        let ltrOverlap = ((*lh).fSectorMask | (*rh).fSectorMask) & self.fSectorMask != 0;
        let lrOverlap = (*lh).fSectorMask & (*rh).fSectorMask != 0;
        let lrOrder;
        if !lrOverlap {
            if !ltrOverlap {
                return ((*lh).fSectorEnd > (*rh).fSectorStart)
                    ^ (self.fSectorStart > (*lh).fSectorEnd)
                    ^ (self.fSectorStart > (*rh).fSectorStart);
            }
            let gap = ((*rh).fSectorStart - (*lh).fSectorStart + 32) & 0x1f;
            lrOrder = if gap > 20 {
                0
            } else if gap > 11 {
                -1
            } else {
                1
            };
        } else {
            lrOrder = (*lh).orderable(rh);
            if !ltrOverlap && lrOrder >= 0 {
                return lrOrder == 0;
            }
        }
        let mut ltOrder = if (*lh).fSectorMask & self.fSectorMask != 0 {
            (*lh).orderable(this)
        } else {
            let gap = (self.fSectorStart - (*lh).fSectorStart + 32) & 0x1f;
            if gap > 20 {
                0
            } else if gap > 11 {
                -1
            } else {
                1
            }
        };
        let mut trOrder = if (*rh).fSectorMask & self.fSectorMask != 0 {
            self.orderable(rh)
        } else {
            let gap = ((*rh).fSectorStart - self.fSectorStart + 32) & 0x1f;
            if gap > 20 {
                0
            } else if gap > 11 {
                -1
            } else {
                1
            }
        };
        self.alignmentSameSide(&*lh, &mut ltOrder);
        self.alignmentSameSide(&*rh, &mut trOrder);
        if lrOrder >= 0 && ltOrder >= 0 && trOrder >= 0 {
            return if lrOrder != 0 {
                ltOrder & trOrder != 0
            } else {
                ltOrder | trOrder != 0
            };
        }
        if ltOrder == 0 && lrOrder == 0 {
            return (*lh).oppositePlanes(self);
        } else if ltOrder == 1 && trOrder == 0 {
            return self.oppositePlanes(&*rh);
        } else if lrOrder == 1 && trOrder == 1 {
            return (*lh).oppositePlanes(&*rh);
        }
        if self.fUnorderable || (*lh).fUnorderable || (*rh).fUnorderable {
            if !self.fPart.isCurve() && !(*lh).fPart.isCurve() && !(*rh).fPart.isCurve() {
                let ltShare = ((&(*lh).fOriginalCurvePart)[0] == self.fOriginalCurvePart[0]) as i32;
                let lrShare =
                    ((&(*lh).fOriginalCurvePart)[0] == (&(*rh).fOriginalCurvePart)[0]) as i32;
                let trShare = (self.fOriginalCurvePart[0] == (&(*rh).fOriginalCurvePart)[0]) as i32;
                if ltShare + lrShare + trShare == 1 {
                    if lrShare != 0 {
                        let lt = (*lh).linesOnOriginalSide(self);
                        let rt = (*rh).linesOnOriginalSide(self);
                        if rt ^ lt == 1 {
                            return lt != 0;
                        }
                    } else if trShare != 0 {
                        let tl = self.linesOnOriginalSide(&*lh);
                        let rl = (*rh).linesOnOriginalSide(&*lh);
                        if tl ^ rl == 1 {
                            return rl != 0;
                        }
                    } else {
                        let tr = (*rh).linesOnOriginalSide(self);
                        let lr = (*lh).linesOnOriginalSide(&*rh);
                        if lr ^ tr == 1 {
                            return tr != 0;
                        }
                    }
                }
            }
        }
        if lrOrder < 0 {
            return if ltOrder < 0 {
                trOrder != 0
            } else {
                ltOrder != 0
            };
        }
        lrOrder == 0
    }
    unsafe fn lineOnOneSideFrom(
        &self,
        origin: &SkDPoint,
        line: &SkDVector,
        test: &Self,
        useOriginal: bool,
    ) -> i32 {
        let mut crosses = [0.0; 3];
        let verb = (*test.segment()).verb();
        let count = SkPathOpsVerbToPoints(verb) as usize;
        let curve = if useOriginal {
            &test.fOriginalCurvePart
        } else {
            &test.fPart.fCurve
        };
        for index in 1..=count {
            let xy1 = line.fX * (curve[index].fY - origin.fY);
            let xy2 = line.fY * (curve[index].fX - origin.fX);
            crosses[index - 1] = if AlmostBequalUlps(xy1, xy2) {
                0.0
            } else {
                xy1 - xy2
            };
        }
        if crosses[0] * crosses[1] < 0.0 {
            return -1;
        }
        if verb == Verb::kCubic_Verb
            && (crosses[0] * crosses[2] < 0.0 || crosses[1] * crosses[2] < 0.0)
        {
            return -1;
        }
        if crosses[0] != 0.0 {
            return (crosses[0] < 0.0) as i32;
        }
        if crosses[1] != 0.0 {
            return (crosses[1] < 0.0) as i32;
        }
        if verb == Verb::kCubic_Verb && crosses[2] != 0.0 {
            return (crosses[2] < 0.0) as i32;
        }
        -2
    }
    unsafe fn lineOnOneSide(&mut self, test: &Self, useOriginal: bool) -> i32 {
        let origin = self.fPart.fCurve[0];
        let line = self.fPart.fCurve[1] - origin;
        let result = self.lineOnOneSideFrom(&origin, &line, test, useOriginal);
        if result == -2 {
            self.fUnorderable = true;
            -1
        } else {
            result
        }
    }
    fn linesOnOriginalSide(&mut self, test: &Self) -> i32 {
        let origin = self.fOriginalCurvePart[0];
        let line = self.fOriginalCurvePart[1] - origin;
        let mut dots = [0.0; 2];
        let mut crosses = [0.0; 2];
        for index in 0..2 {
            let testLine = test.fOriginalCurvePart[index] - origin;
            let xy1 = line.fX * testLine.fY;
            let xy2 = line.fY * testLine.fX;
            dots[index] = line.fX * testLine.fX + line.fY * testLine.fY;
            crosses[index] = if AlmostBequalUlps(xy1, xy2) {
                0.0
            } else {
                xy1 - xy2
            };
        }
        if crosses[0] * crosses[1] < 0.0 {
            return -1;
        }
        if crosses[0] != 0.0 {
            return (crosses[0] < 0.0) as i32;
        }
        if crosses[1] != 0.0 {
            return (crosses[1] < 0.0) as i32;
        }
        if (dots[0] == 0.0 && dots[1] < 0.0) || (dots[0] < 0.0 && dots[1] == 0.0) {
            return 2;
        }
        self.fUnorderable = true;
        -1
    }
    unsafe fn alignmentSameSide(&self, test: &Self, order: &mut i32) {
        if *order < 0 || self.fPart.isCurve() || test.fPart.isCurve() {
            return;
        }
        let xOrigin = test.fPart.fCurve[0];
        let oOrigin = test.fOriginalCurvePart[0];
        if xOrigin == oOrigin {
            return;
        }
        let count = SkPathOpsVerbToPoints((*self.segment()).verb()) as usize;
        let xLine = test.fPart.fCurve[1] - xOrigin;
        let oLine = test.fOriginalCurvePart[1] - oOrigin;
        for index in 1..=count {
            let pt = self.fPart.fCurve[index];
            let xCross = oLine.crossCheck(&(pt - xOrigin));
            let oCross = xLine.crossCheck(&(pt - oOrigin));
            if oCross * xCross < 0.0 {
                *order ^= 1;
                break;
            }
        }
    }
    unsafe fn computeSector(&mut self) -> bool {
        if self.fComputedSector {
            return !self.fUnorderable;
        }
        self.fComputedSector = true;
        let stepUp = (*self.fStart).t() < (*self.fEnd).t();
        let mut checkEnd = self.fEnd;
        if (*checkEnd).final_() && stepUp {
            self.fUnorderable = true;
            return false;
        }
        'advance: loop {
            let other = (*checkEnd).segment();
            let mut oSpan = (*other).head().cast::<SkOpSpanBase>();
            loop {
                if (*oSpan).segment() == self.segment()
                    && oSpan != checkEnd
                    && approximately_equal((*oSpan).t(), (*checkEnd).t())
                {
                    break 'advance;
                }
                if (*oSpan).final_() {
                    break;
                }
                oSpan = (*(*oSpan).upCast()).next();
                if oSpan.is_null() {
                    break;
                }
            }
            checkEnd = if stepUp {
                if !(*checkEnd).final_() {
                    (*(*checkEnd).upCast()).next()
                } else {
                    null_mut()
                }
            } else {
                (*checkEnd).prev().cast::<SkOpSpanBase>()
            };
            if checkEnd.is_null() {
                break;
            }
        }
        let computedEnd = if stepUp {
            if !checkEnd.is_null() {
                (*checkEnd).prev().cast::<SkOpSpanBase>()
            } else {
                (*(*self.fEnd).segment()).head().cast::<SkOpSpanBase>()
            }
        } else if !checkEnd.is_null() {
            (*(*checkEnd).upCast()).next()
        } else {
            (*(*self.fEnd).segment()).tail()
        };
        if checkEnd == self.fEnd
            || computedEnd == self.fEnd
            || computedEnd == self.fStart
            || stepUp != ((*self.fStart).t() < (*computedEnd).t())
        {
            self.fUnorderable = true;
            return false;
        }
        let saveEnd = self.fEnd;
        self.fComputedEnd = computedEnd;
        self.fEnd = computedEnd;
        self.setSpans();
        self.setSector();
        self.fEnd = saveEnd;
        !self.fUnorderable
    }
    unsafe fn checkParallel(&mut self, rh: *mut Self) -> bool {
        let sweep = if self.fPart.isOrdered() {
            self.fPart.fSweep[0]
        } else {
            self.fPart.fCurve[1] - self.fPart.fCurve[0]
        };
        let tweep = if (*rh).fPart.isOrdered() {
            (*rh).fPart.fSweep[0]
        } else {
            (&(*rh).fPart.fCurve)[1] - (&(*rh).fPart.fCurve)[0]
        };
        let cross = sweep.crossCheck(&tweep);
        if self.tangentsDiverge(&*rh, cross) {
            return cross < 0.0;
        }
        let mut inside = false;
        if !(*self.fEnd).contains((*rh).fEnd) {
            if self.endToSide(&*rh, &mut inside) {
                return inside;
            }
            if (*rh).endToSide(self, &mut inside) {
                return !inside;
            }
        }
        if self.midToSide(&*rh, &mut inside) {
            return inside;
        }
        if (*rh).midToSide(self, &mut inside) {
            return !inside;
        }
        let m0 = (*self.segment()).dPtAtT(self.midT()) - self.fPart.fCurve[0];
        let m1 = (*(*rh).segment()).dPtAtT((*rh).midT()) - (&(*rh).fPart.fCurve)[0];
        let cross = m0.crossCheck(&m1);
        if cross == 0.0 {
            self.fUnorderable = true;
            (*rh).fUnorderable = true;
            return true;
        }
        cross < 0.0
    }
    unsafe fn convexHullOverlaps(&mut self, rh: &Self) -> i32 {
        let sweep = &self.fPart.fSweep;
        let tweep = &rh.fPart.fSweep;
        let s0xs1 = sweep[0].crossCheck(&sweep[1]);
        let s0xt0 = sweep[0].crossCheck(&tweep[0]);
        let s1xt0 = sweep[1].crossCheck(&tweep[0]);
        let mut tBetweenS = if s0xs1 > 0.0 {
            s0xt0 > 0.0 && s1xt0 < 0.0
        } else {
            s0xt0 < 0.0 && s1xt0 > 0.0
        };
        let s0xt1 = sweep[0].crossCheck(&tweep[1]);
        let s1xt1 = sweep[1].crossCheck(&tweep[1]);
        tBetweenS |= if s0xs1 > 0.0 {
            s0xt1 > 0.0 && s1xt1 < 0.0
        } else {
            s0xt1 < 0.0 && s1xt1 > 0.0
        };
        let t0xt1 = tweep[0].crossCheck(&tweep[1]);
        if tBetweenS || (s0xt0 == 0.0 && s1xt1 == 0.0) || (s1xt0 == 0.0 && s0xt1 == 0.0) {
            return -1;
        }
        let mut sBetweenT = if t0xt1 > 0.0 {
            s0xt0 < 0.0 && s0xt1 > 0.0
        } else {
            s0xt0 > 0.0 && s0xt1 < 0.0
        };
        sBetweenT |= if t0xt1 > 0.0 {
            s1xt0 < 0.0 && s1xt1 > 0.0
        } else {
            s1xt0 > 0.0 && s1xt1 < 0.0
        };
        if sBetweenT {
            return -1;
        }
        if s0xt0 >= 0.0 && s0xt1 >= 0.0 && s1xt0 >= 0.0 && s1xt1 >= 0.0 {
            return 0;
        }
        if s0xt0 <= 0.0 && s0xt1 <= 0.0 && s1xt0 <= 0.0 && s1xt1 <= 0.0 {
            return 1;
        }
        let m0 = (*self.segment()).dPtAtT(self.midT()) - self.fPart.fCurve[0];
        let m1 = (*rh.segment()).dPtAtT(rh.midT()) - rh.fPart.fCurve[0];
        let cross = m0.crossCheck(&m1);
        if s0xt0 > 0.0 && cross > 0.0 {
            return 0;
        }
        if s0xt0 < 0.0 && cross < 0.0 {
            return 1;
        }
        if self.tangentsDiverge(rh, s0xt0) {
            (s0xt0 < 0.0) as i32
        } else {
            (cross < 0.0) as i32
        }
    }
    pub unsafe fn distEndRatio(&self, dist: f64) -> f64 {
        let segment = &*self.segment();
        let count = SkPathOpsVerbToPoints(segment.verb()) as usize;
        let pts = segment.pts();
        let mut longest = 0.0_f64;
        for a in 0..count {
            for b in a + 1..=count {
                let mut v = SkDVector::default();
                v.set(&(*pts.add(b) - *pts.add(a)));
                longest = longest.max(v.lengthSquared());
            }
        }
        longest.sqrt() / dist
    }
    unsafe fn tangentsDiverge(&mut self, rh: &Self, cross: f64) -> bool {
        if cross == 0.0 {
            return false;
        }
        let sweep = &self.fPart.fSweep;
        let tweep = &rh.fPart.fSweep;
        let dot = sweep[0].dot(&tweep[0]);
        if dot == 0.0 {
            return true;
        }
        let m = cross / dot;
        let sDist = sweep[0].length() * m;
        let tDist = tweep[0].length() * m;
        let useS = sDist.abs() < tDist.abs();
        let factor = if useS {
            self.distEndRatio(sDist)
        } else {
            rh.distEndRatio(tDist)
        }
        .abs();
        self.fTangentsAmbiguous = factor >= 50.0 && factor < 200.0;
        factor < 50.0
    }
    unsafe fn endToSide(&self, rh: &Self, inside: &mut bool) -> bool {
        let segment = &*self.segment();
        let verb = segment.verb();
        let mut ray = SkDLine::default();
        ray[0].set((*self.fEnd).pt());
        ray[1] = ray[0];
        let slope = CurveDSlopeAtT(verb, segment.pts(), segment.weight(), (*self.fEnd).t());
        ray[1].fX += slope.fY;
        ray[1].fY -= slope.fX;
        let mut hits = SkIntersections::default();
        let opp = &*rh.segment();
        let oppVerb = opp.verb();
        CurveIntersectRay(oppVerb, opp.pts(), opp.weight(), &ray, &mut hits);
        let mut endDist = 0.0;
        let closest = hits.closestTo((*rh.fStart).t(), (*rh.fEnd).t(), &ray[0], &mut endDist);
        if closest < 0 || endDist == 0.0 {
            return false;
        }
        let start = SkDPoint::from((*self.fStart).pt());
        let curve = &rh.fPart.fCurve;
        let width = Self::curveMaxWidth(curve, SkPathOpsVerbToPoints(oppVerb) as usize);
        endDist /= width;
        if !(endDist >= 5e-12) {
            return false;
        }
        let oppPt = hits.pt(closest as usize);
        let left = ray[0] - start;
        let right = *oppPt - start;
        let dir = left.crossNoNormalCheck(&right);
        if dir == 0.0 {
            return false;
        }
        *inside = dir < 0.0;
        true
    }
    unsafe fn midToSide(&self, rh: &Self, inside: &mut bool) -> bool {
        let segment = &*self.segment();
        let verb = segment.verb();
        let start = (*self.fStart).pt();
        let end = (*self.fEnd).pt();
        let dStart = SkDPoint::from(start);
        let mut ray = SkDLine::default();
        // The source adds/subtracts SkScalar values before widening to double.
        ray[0].fX = ((start.fX + end.fX) / 2.0) as f64;
        ray[0].fY = ((start.fY + end.fY) / 2.0) as f64;
        ray[1].fX = ray[0].fX + (end.fY - start.fY) as f64;
        ray[1].fY = ray[0].fY - (end.fX - start.fX) as f64;
        let mut hits = SkIntersections::default();
        CurveIntersectRay(verb, segment.pts(), segment.weight(), &ray, &mut hits);
        let outside = hits.mostOutside((*self.fStart).t(), (*self.fEnd).t(), &dStart);
        if outside < 0 {
            return false;
        }
        let opp = &*rh.segment();
        let mut oppHits = SkIntersections::default();
        CurveIntersectRay(opp.verb(), opp.pts(), opp.weight(), &ray, &mut oppHits);
        let oppOutside = oppHits.mostOutside((*rh.fStart).t(), (*rh.fEnd).t(), &dStart);
        if oppOutside < 0 {
            return false;
        }
        let iSide = *hits.pt(outside as usize) - dStart;
        let oppSide = *oppHits.pt(oppOutside as usize) - dStart;
        let dir = iSide.crossCheck(&oppSide);
        if dir == 0.0 {
            return false;
        }
        *inside = dir < 0.0;
        true
    }
    fn curveMaxWidth(curve: &SkDCurve, count: usize) -> f64 {
        let mut minX = f64::INFINITY;
        let mut minY = f64::INFINITY;
        let mut maxX = f64::NEG_INFINITY;
        let mut maxY = f64::NEG_INFINITY;
        for index in 0..=count {
            let p = curve[index];
            if p.fX < minX {
                minX = p.fX;
            }
            if p.fY < minY {
                minY = p.fY;
            }
            if maxX < p.fX {
                maxX = p.fX;
            }
            if maxY < p.fY {
                maxY = p.fY;
            }
        }
        if maxX - minX < maxY - minY {
            maxY - minY
        } else {
            maxX - minX
        }
    }
    unsafe fn endsIntersect(&mut self, rh: *mut Self) -> bool {
        let lVerb = (*self.segment()).verb();
        let rVerb = (*(*rh).segment()).verb();
        let lPts = SkPathOpsVerbToPoints(lVerb) as usize;
        let rPts = SkPathOpsVerbToPoints(rVerb) as usize;
        let rays = [
            SkDLine {
                fPts: [self.fPart.fCurve[0], (&(*rh).fPart.fCurve)[rPts]],
            },
            SkDLine {
                fPts: [self.fPart.fCurve[0], self.fPart.fCurve[lPts]],
            },
        ];
        if (*self.fEnd).contains((*rh).fEnd) {
            return self.checkParallel(rh);
        }
        let mut smallTs = [-1.0; 2];
        let mut limited = [false; 2];
        for index in 0..2 {
            let cVerb = if index != 0 { rVerb } else { lVerb };
            if cVerb == Verb::kLine_Verb {
                continue;
            }
            let owner = if index != 0 { &*rh } else { &*self };
            let segment = &*owner.segment();
            let mut hits = SkIntersections::default();
            CurveIntersectRay(
                cVerb,
                segment.pts(),
                segment.weight(),
                &rays[index],
                &mut hits,
            );
            let start = (*owner.fStart).t();
            let end = (*owner.fComputedEnd).t();
            let ascends = start < end;
            let mut t = if ascends { 0.0_f64 } else { 1.0_f64 };
            for hit in 0..hits.used() {
                let test = hits[0][hit];
                if !approximately_between_orderable(start, test, end)
                    || approximately_equal_orderable(start, test)
                {
                    continue;
                }
                t = if ascends { t.max(test) } else { t.min(test) };
                smallTs[index] = t;
                limited[index] = approximately_equal_orderable(t, end);
            }
        }
        let mut sRayLonger = false;
        let mut sCept = SkDVector::default();
        let mut sCeptT = -1.0;
        let mut sIndex = 0_usize;
        let mut useIntersect = false;
        for index in 0..2 {
            if smallTs[index] < 0.0 {
                continue;
            }
            let owner = if index != 0 { &*rh } else { &*self };
            let segment = &*owner.segment();
            let point = segment.dPtAtT(smallTs[index]);
            let cept = point - rays[index][0];
            if (if index != 0 { lPts } else { rPts }) == 1 {
                let total = rays[index][1] - rays[index][0];
                if cept.lengthSquared() * 2.0 < total.lengthSquared() {
                    continue;
                }
            }
            let end = rays[index][1] - rays[index][0];
            if cept.fX * end.fX < 0.0 || cept.fY * end.fY < 0.0 {
                continue;
            }
            let rayDist = cept.length();
            let endDist = end.length();
            let rayLonger = rayDist > endDist;
            if limited[0] && limited[1] && rayLonger {
                useIntersect = true;
                sRayLonger = rayLonger;
                sCept = cept;
                sCeptT = smallTs[index];
                sIndex = index;
                break;
            }
            let curve = if index != 0 {
                &(*rh).fPart.fCurve
            } else {
                &self.fPart.fCurve
            };
            let count = if index != 0 { rPts } else { lPts };
            let delta = (rayDist - endDist).abs() / Self::curveMaxWidth(curve, count);
            if delta < 4e-3
                && delta > 1e-3
                && !useIntersect
                && self.fPart.isCurve()
                && (*rh).fPart.isCurve()
                && self.fOriginalCurvePart[0] != self.fPart.fCurve[0]
            {
                let origin = (&(*rh).fOriginalCurvePart)[0];
                let count = SkPathOpsVerbToPoints((*(*rh).segment()).verb()) as usize;
                let line = (&(*rh).fOriginalCurvePart)[count] - origin;
                let original = (*rh).lineOnOneSideFrom(&origin, &line, self, true);
                if original >= 0 {
                    let translated = (*rh).lineOnOneSideFrom(&origin, &line, self, false);
                    if original != translated {
                        continue;
                    }
                }
            }
            if delta > 1e-3 {
                useIntersect ^= true;
                if useIntersect {
                    sRayLonger = rayLonger;
                    sCept = cept;
                    sCeptT = smallTs[index];
                    sIndex = index;
                }
            }
        }
        if useIntersect {
            let owner = if sIndex != 0 { &*rh } else { &*self };
            let segment = &*owner.segment();
            let start = (*owner.fStart).t();
            let mid = segment.dPtAtT(start + (sCeptT - start) / 2.0) - owner.fPart.fCurve[0];
            let dir = mid.crossCheck(&sCept);
            if dir == 0.0 {
                return self.checkParallel(rh);
            }
            sRayLonger ^ (sIndex == 0) ^ (dir < 0.0)
        } else {
            self.checkParallel(rh)
        }
    }
    unsafe fn orderable(&mut self, rh: *mut Self) -> i32 {
        let mut unorderable = false;
        if !self.fPart.isCurve() {
            if !(*rh).fPart.isCurve() {
                let leftX = self.fTangentHalf.dx();
                let leftY = self.fTangentHalf.dy();
                let rightX = (*rh).fTangentHalf.dx();
                let rightY = (*rh).fTangentHalf.dy();
                let xy = leftX * rightY;
                let yx = rightX * leftY;
                if xy == yx {
                    if leftX * rightX < 0.0 || leftY * rightY < 0.0 {
                        return 1;
                    }
                    unorderable = true;
                } else {
                    return (xy < yx) as i32;
                }
            } else {
                let result = self.lineOnOneSide(&*rh, false);
                if result >= 0 {
                    return result;
                }
                if self.fUnorderable || approximately_zero((*rh).fSide) {
                    unorderable = true;
                }
            }
        } else if !(*rh).fPart.isCurve() {
            let result = (*rh).lineOnOneSide(self, false);
            if result >= 0 {
                return if result != 0 { 0 } else { 1 };
            }
            if (*rh).fUnorderable || approximately_zero(self.fSide) {
                unorderable = true;
            }
        } else {
            let result = self.convexHullOverlaps(&*rh);
            if result >= 0 {
                return result;
            }
        }
        if unorderable {
            self.fUnorderable = true;
            (*rh).fUnorderable = true;
            -1
        } else {
            self.endsIntersect(rh) as i32
        }
    }
    fn findSector(&self, verb: Verb, x: f64, y: f64) -> i32 {
        let absX = x.abs();
        let absY = y.abs();
        let xy = if verb == Verb::kLine_Verb || !AlmostEqualUlps(absX, absY) {
            absX - absY
        } else {
            0.0
        };
        const SEDECIMANT: [[[i32; 3]; 3]; 3] = [
            [[4, 3, 2], [7, -1, 15], [10, 11, 12]],
            [[5, -1, 1], [-1, -1, -1], [9, -1, 13]],
            [[6, 3, 0], [7, -1, 15], [8, 11, 14]],
        ];
        SEDECIMANT[(xy >= 0.0) as usize + (xy > 0.0) as usize]
            [(y >= 0.0) as usize + (y > 0.0) as usize][(x >= 0.0) as usize + (x > 0.0) as usize]
            * 2
            + 1
    }
    unsafe fn setSpans(&mut self) {
        self.fUnorderable = false;
        self.fLastMarked = null_mut();
        if self.fStart.is_null() {
            self.fUnorderable = true;
            return;
        }
        let segment = &*(*self.fStart).segment();
        let pts = segment.pts();
        segment.subDivide(self.fStart, self.fEnd, &mut self.fPart.fCurve);
        self.fOriginalCurvePart = self.fPart.fCurve;
        let verb = segment.verb();
        self.fPart.setCurveHullSweep(verb);
        if verb != Verb::kLine_Verb && !self.fPart.isCurve() {
            self.fPart.fCurve[1] = self.fPart.fCurve[SkPathOpsVerbToPoints(verb) as usize];
            self.fOriginalCurvePart[1] = self.fPart.fCurve[1];
            let mut line = SkDLine::default();
            line[0].set(&self.fPart.fCurve[0].asSkPoint());
            line[1].set(&self.fPart.fCurve[1].asSkPoint());
            self.fTangentHalf.lineEndPoints(&line);
            self.fSide = 0.0;
        }
        match verb {
            Verb::kLine_Verb => {
                let cp = *pts.add(((*self.fStart).t() < (*self.fEnd).t()) as usize);
                let mut line = SkDLine::default();
                line[0].set((*self.fStart).pt());
                line[1].set(&cp);
                self.fTangentHalf.lineEndPoints(&line);
                self.fSide = 0.0;
            }
            Verb::kQuad_Verb | Verb::kConic_Verb => {
                let mut tangent = SkLineParameters::default();
                tangent.quadEndPoints(&self.fPart.fCurve.fQuad);
                self.fSide = -tangent.pointDistance(&self.fPart.fCurve[2]);
            }
            Verb::kCubic_Verb => {
                let mut tangent = SkLineParameters::default();
                tangent.cubicPart(&self.fPart.fCurve.fCubic);
                self.fSide = -tangent.pointDistance(&self.fPart.fCurve[3]);
                let mut ts = [0.0_f64; 4];
                let mut count = SkDCubic::FindInflections(&*pts.cast::<[SkPoint; 4]>(), &mut ts);
                let start = (*self.fStart).t();
                let end = (*self.fEnd).t();
                for index in 0..count {
                    if !between(start, ts[index], end) {
                        ts[index] = -1.0;
                    }
                }
                ts[count] = start;
                count += 1;
                ts[count] = end;
                count += 1;
                SkTQSortNatural(&mut ts[..count]);
                let mut best = 0.0_f64;
                let cases = (count << 1) - 1;
                let mut index = 0;
                while ts[index] < 0.0 {
                    index += 1;
                }
                index <<= 1;
                while index < cases {
                    let testIndex = index >> 1;
                    let mut t = ts[testIndex];
                    if index & 1 != 0 {
                        t = (t + ts[testIndex + 1]) / 2.0;
                    }
                    let pt = CurveDPointAtT(Verb::kCubic_Verb, pts, segment.weight(), t);
                    let mut part = SkLineParameters::default();
                    part.cubicEndPoints(&self.fPart.fCurve.fCubic);
                    let side = part.pointDistance(&pt);
                    if best.abs() < side.abs() {
                        best = side;
                    }
                    index += 1;
                }
                self.fSide = -best;
            }
            _ => {}
        }
    }
    unsafe fn setSector(&mut self) {
        if self.fStart.is_null() {
            self.fUnorderable = true;
            return;
        }
        let verb = (*(*self.fStart).segment()).verb();
        self.fSectorStart = self.findSector(verb, self.fPart.fSweep[0].fX, self.fPart.fSweep[0].fY);
        if self.fSectorStart >= 0 {
            if !self.fPart.isCurve() {
                self.fSectorEnd = self.fSectorStart;
                self.fSectorMask = 1_u32 << self.fSectorStart;
                return;
            }
            self.fSectorEnd =
                self.findSector(verb, self.fPart.fSweep[1].fX, self.fPart.fSweep[1].fY);
        }
        if self.fSectorStart < 0 || self.fSectorEnd < 0 {
            self.fSectorStart = -1;
            self.fSectorEnd = -1;
            self.fSectorMask = 0;
            self.fComputeSector = true;
            return;
        }
        if self.fSectorEnd == self.fSectorStart && (self.fSectorStart & 3) != 3 {
            self.fSectorMask = 1_u32 << self.fSectorStart;
            return;
        }
        let crosses = self.checkCrossesZero();
        let start = self.fSectorStart.min(self.fSectorEnd);
        let ccw = (self.fSectorStart == start) ^ crosses;
        if self.fSectorStart & 3 == 3 {
            self.fSectorStart = (self.fSectorStart + if ccw { 1 } else { 31 }) & 0x1f;
        }
        if self.fSectorEnd & 3 == 3 {
            self.fSectorEnd = (self.fSectorEnd + if ccw { 31 } else { 1 }) & 0x1f;
        }
        let crosses = self.checkCrossesZero();
        let start = self.fSectorStart.min(self.fSectorEnd);
        let end = self.fSectorStart.max(self.fSectorEnd);
        self.fSectorMask = if !crosses {
            u32::MAX >> (31 - end + start) << start
        } else {
            u32::MAX >> (31 - start) | (u32::MAX << end)
        };
    }
}
