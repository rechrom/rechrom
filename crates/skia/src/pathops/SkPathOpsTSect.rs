// Copyright 2014 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsTSect.h/.cpp. Exact subdivision PathOps owner.
use super::{
    SkIntersections::SkIntersections,
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsRect::SkDRect,
    SkPathOpsTCurve::SkTCurve,
    SkPathOpsTypes::*,
};
use crate::src::core::{
    SkArenaAlloc::{SkArenaAlloc, SkSTArenaAlloc},
    SkTSort::SkTQSort,
};
use core::pin::Pin;
use core::ptr::null_mut;
#[derive(Clone, Copy)]
pub struct SkTCoincident {
    pub(super) fPerpPt: SkDPoint,
    pub(super) fPerpT: f64,
    pub(super) fMatch: bool,
}
impl Default for SkTCoincident {
    fn default() -> Self {
        Self {
            fPerpPt: SkDPoint {
                fX: f64::NAN,
                fY: f64::NAN,
            },
            fPerpT: -1.0,
            fMatch: false,
        }
    }
}
impl SkTCoincident {
    pub fn init(&mut self) {
        *self = Self::default();
    }
    pub fn isMatch(&self) -> bool {
        self.fMatch
    }
    pub fn markCoincident(&mut self) {
        if !self.fMatch {
            self.fPerpT = -1.0;
        }
        self.fMatch = true;
    }
    pub fn perpPt(&self) -> SkDPoint {
        self.fPerpPt
    }
    pub fn perpT(&self) -> f64 {
        self.fPerpT
    }
    pub fn setPerp(
        &mut self,
        c1: &dyn SkTCurve,
        t: f64,
        pt: impl Into<SkDPoint>,
        c2: &dyn SkTCurve,
    ) {
        let pt = pt.into();
        let slope = c1.dxdyAtT(t);
        let perp = SkDLine {
            fPts: [
                pt,
                SkDPoint {
                    fX: pt.fX + slope.fY,
                    fY: pt.fY - slope.fX,
                },
            ],
        };
        let mut i = SkIntersections::default();
        let used = c2.intersectRay(&mut i, &perp);
        if used == 0 || used == 3 {
            self.init();
            return;
        }
        self.fPerpT = i[0][0];
        self.fPerpPt = *i.pt(0);

        if used == 2 && (*i.pt(1) - pt).lengthSquared() < (self.fPerpPt - pt).lengthSquared() {
            self.fPerpT = i[0][1];
            self.fPerpPt = *i.pt(1);
        }
        self.fMatch = pt.approximatelyEqual(self.fPerpPt);
    }
}
pub struct SkTSpanBounded {
    pub(super) fBounded: *mut SkTSpan,
    pub(super) fNext: *mut Self,
}
pub struct SkTSpan {
    pub(super) fPart: *mut dyn SkTCurve,
    pub(super) fCoinStart: SkTCoincident,
    pub(super) fCoinEnd: SkTCoincident,
    pub(super) fBounded: *mut SkTSpanBounded,
    pub(super) fPrev: *mut Self,
    pub(super) fNext: *mut Self,
    pub(super) fBounds: SkDRect,
    pub(super) fStartT: f64,
    pub(super) fEndT: f64,
    pub(super) fBoundsMax: f64,
    pub(super) fCollapsed: bool,
    pub(super) fHasPerp: bool,
    pub(super) fIsLinear: bool,
    pub(super) fIsLine: bool,
    pub(super) fDeleted: bool,
}
impl SkTSpan {
    pub unsafe fn new(c: &dyn SkTCurve, heap: *mut SkArenaAlloc) -> Self {
        Self {
            fPart: c.make(&mut *heap),
            fCoinStart: SkTCoincident::default(),
            fCoinEnd: SkTCoincident::default(),
            fBounded: null_mut(),
            fPrev: null_mut(),
            fNext: null_mut(),
            fBounds: SkDRect::default(),
            fStartT: 0.0,
            fEndT: 1.0,
            fBoundsMax: 0.0,
            fCollapsed: false,
            fHasPerp: false,
            fIsLinear: false,
            fIsLine: false,
            fDeleted: false,
        }
    }
    pub fn startT(&self) -> f64 {
        self.fStartT
    }
    pub fn endT(&self) -> f64 {
        self.fEndT
    }
    pub fn isBounded(&self) -> bool {
        !self.fBounded.is_null()
    }
    pub fn next(&self) -> *mut Self {
        self.fNext
    }
    pub fn reset(&mut self) {
        self.fBounded = null_mut();
    }
    pub unsafe fn part(&self) -> &dyn SkTCurve {
        &*self.fPart
    }
    pub unsafe fn pointCount(&self) -> usize {
        (*self.fPart).pointCount()
    }
    pub unsafe fn pointFirst(&self) -> SkDPoint {
        *(*self.fPart).point(0)
    }
    pub unsafe fn pointLast(&self) -> SkDPoint {
        *(*self.fPart).point((*self.fPart).pointLast())
    }
    pub fn markCoincident(&mut self) {
        self.fCoinStart.markCoincident();
        self.fCoinEnd.markCoincident();
    }
    pub unsafe fn addBounded(&mut self, span: *mut Self, heap: *mut SkArenaAlloc) {
        self.fBounded = (*heap).make(SkTSpanBounded {
            fBounded: span,
            fNext: self.fBounded,
        });
    }
    pub unsafe fn closestBoundedT(&self, pt: &SkDPoint) -> f64 {
        let mut result = -1.0;
        let mut closest = f64::MAX;
        let mut bounded = self.fBounded;
        while !bounded.is_null() {
            let test = (*bounded).fBounded;
            let start = (*test).pointFirst().distanceSquared(pt);
            if closest > start {
                closest = start;
                result = (*test).fStartT;
            }
            let end = (*test).pointLast().distanceSquared(pt);
            if closest > end {
                closest = end;
                result = (*test).fEndT;
            }
            bounded = (*bounded).fNext;
        }

        result
    }
    pub unsafe fn contains(&self, t: f64) -> bool {
        let mut work = self as *const Self;
        loop {
            if between((*work).fStartT, t, (*work).fEndT) {
                return true;
            }
            work = (*work).fNext;
            if work.is_null() {
                return false;
            }
        }
    }
    pub unsafe fn findOppSpan(&self, opp: *const Self) -> *mut Self {
        let mut bounded = self.fBounded;
        while !bounded.is_null() {
            let test = (*bounded).fBounded;
            if test.cast_const() == opp {
                return test;
            }
            bounded = (*bounded).fNext;
        }
        null_mut()
    }
    pub unsafe fn oppT(&self, t: f64) -> *mut Self {
        let mut b = self.fBounded;
        while !b.is_null() {
            let s = (*b).fBounded;
            if between((*s).fStartT, t, (*s).fEndT) {
                return s;
            }
            b = (*b).fNext;
        }
        null_mut()
    }
    pub unsafe fn findOppT(&self, t: f64) -> *mut Self {
        let result = self.oppT(t);

        result
    }
    pub unsafe fn hasOppT(&self, t: f64) -> bool {
        !self.oppT(t).is_null()
    }
    unsafe fn hullCheck(&mut self, opp: *const Self, start: &mut bool, oppStart: &mut bool) -> i32 {
        if self.fIsLinear {
            return -1;
        }
        let mut common = false;
        if self.onlyEndPointsInCommon(opp, start, oppStart, &mut common) {
            return 2;
        }
        let mut linear = false;
        if (*self.fPart).hullIntersects(&*(*opp).fPart, &mut linear) {
            if !linear {
                return 1;
            }
            self.fIsLinear = true;
            self.fIsLine = (*self.fPart).controlsInside();
            return if common { 1 } else { -1 };
        }
        (common as i32) << 1
    }
    pub unsafe fn hullsIntersect(
        &mut self,
        opp: *mut Self,
        start: &mut bool,
        oppStart: &mut bool,
    ) -> i32 {
        if !self.fBounds.intersects(&(*opp).fBounds) {
            return 0;
        }
        let result = self.hullCheck(opp, start, oppStart);
        if result >= 0 {
            return result;
        }
        let result = (*opp).hullCheck(self, oppStart, start);
        if result >= 0 {
            return result;
        }
        -1
    }
    pub unsafe fn init(&mut self, c: &dyn SkTCurve) {
        self.fPrev = null_mut();
        self.fNext = null_mut();
        self.fStartT = 0.0;
        self.fEndT = 1.0;
        self.fBounded = null_mut();
        self.resetBounds(c);
    }
    pub unsafe fn initBounds(&mut self, c: &dyn SkTCurve) -> bool {
        if self.fStartT.is_nan() || self.fEndT.is_nan() {
            return false;
        }
        c.subDivide(self.fStartT, self.fEndT, &mut *self.fPart);
        (*self.fPart).setBounds(&mut self.fBounds);
        self.fCoinStart.init();
        self.fCoinEnd.init();
        self.fBoundsMax = cpp_max(self.fBounds.width(), self.fBounds.height());
        self.fCollapsed = (*self.fPart).collapsed();
        self.fHasPerp = false;
        self.fDeleted = false;
        self.fBounds.valid()
    }
    pub unsafe fn resetBounds(&mut self, c: &dyn SkTCurve) {
        self.fIsLinear = false;
        self.fIsLine = false;
        self.initBounds(c);
    }
    pub unsafe fn linearsIntersect(&self, span: *mut Self) -> bool {
        let mut result = self.linearIntersects(&*(*span).fPart);
        if result <= 1 {
            return result != 0;
        }

        result = (*span).linearIntersects(&*self.fPart);
        result != 0
    }
    pub unsafe fn linearT(&self, pt: &SkDPoint) -> f64 {
        let len = self.pointLast() - self.pointFirst();
        if len.fX.abs() > len.fY.abs() {
            (pt.fX - self.pointFirst().fX) / len.fX
        } else {
            (pt.fY - self.pointFirst().fY) / len.fY
        }
    }
    unsafe fn linearIntersects(&self, q2: &dyn SkTCurve) -> i32 {
        let part = &*self.fPart;
        let mut start = 0;
        let mut end = part.pointLast();
        if !part.controlsInside() {
            let mut dist = 0.0;
            for outer in 0..self.pointCount() - 1 {
                for inner in outer + 1..self.pointCount() {
                    let test = (*part.point(outer) - *part.point(inner)).lengthSquared();
                    if dist > test {
                        continue;
                    }
                    dist = test;
                    start = outer;
                    end = inner;
                }
            }
        }
        let orig = part.point(start);
        let adj = part.point(end).fX - orig.fX;
        let opp = part.point(end).fY - orig.fY;
        let maxPart = cpp_max(adj.abs(), opp.abs());
        let mut sign = 0.0;
        for n in 0..q2.pointCount() {
            let p = q2.point(n);
            let dx = p.fY - orig.fY;
            let dy = p.fX - orig.fX;
            let maxVal = cpp_max(maxPart, cpp_max(dx.abs(), dy.abs()));
            let test = (p.fY - orig.fY) * adj - (p.fX - orig.fX) * opp;
            if precisely_zero_when_compared_to(test, maxVal) {
                return 1;
            }
            if approximately_zero_when_compared_to(test, maxVal) {
                return 3;
            }
            if n == 0 {
                sign = test;
                continue;
            }
            if test * sign < 0.0 {
                return 1;
            }
        }
        0
    }
    pub unsafe fn onlyEndPointsInCommon(
        &self,
        opp: *const Self,
        start: &mut bool,
        oppStart: &mut bool,
        common: &mut bool,
    ) -> bool {
        if (*opp).pointFirst() == self.pointFirst() {
            *start = true;
            *oppStart = true;
        } else if (*opp).pointFirst() == self.pointLast() {
            *start = false;
            *oppStart = true;
        } else if (*opp).pointLast() == self.pointFirst() {
            *start = true;
            *oppStart = false;
        } else if (*opp).pointLast() == self.pointLast() {
            *start = false;
            *oppStart = false;
        } else {
            *common = false;
            return false;
        }
        *common = true;
        let part = &*self.fPart;
        let other = &*(*opp).fPart;
        let baseIndex = if *start { 0 } else { part.pointLast() };
        let base = *part.point(baseIndex);
        // Source otherPts enumerates in index order except the common endpoint.
        let mut a = [core::ptr::null(); 4];
        let mut b = [core::ptr::null(); 4];
        part.otherPts(baseIndex, &mut a);
        other.otherPts(if *oppStart { 0 } else { other.pointLast() }, &mut b);
        for o1 in 0..self.pointCount() - 1 {
            let v1 = *a[o1] - base;
            for o2 in 0..(*opp).pointCount() - 1 {
                let v2 = *b[o2] - base;
                if v2.dot(&v1) >= 0.0 {
                    return false;
                }
            }
        }
        true
    }
    pub unsafe fn removeAllBounded(&self) -> bool {
        let mut delete = false;
        let mut b = self.fBounded;
        while !b.is_null() {
            delete |= (*(*b).fBounded).removeBounded(self);
            b = (*b).fNext;
        }
        delete
    }
    pub unsafe fn removeBounded(&mut self, opp: *const Self) -> bool {
        if self.fHasPerp {
            let mut start = false;
            let mut end = false;
            let mut b = self.fBounded;
            while !b.is_null() {
                let test = (*b).fBounded;
                if test.cast_const() != opp {
                    start |= between((*test).fStartT, self.fCoinStart.perpT(), (*test).fEndT);
                    end |= between((*test).fStartT, self.fCoinEnd.perpT(), (*test).fEndT);
                }
                b = (*b).fNext;
            }
            if !start || !end {
                self.fHasPerp = false;
                self.fCoinStart.init();
                self.fCoinEnd.init();
            }
        }
        let mut b = self.fBounded;
        let mut prev: *mut SkTSpanBounded = null_mut();
        while !b.is_null() {
            let next = (*b).fNext;
            if opp == (*b).fBounded.cast_const() {
                if !prev.is_null() {
                    (*prev).fNext = next;
                    return false;
                }
                self.fBounded = next;
                return self.fBounded.is_null();
            }
            prev = b;
            b = next;
        }

        false
    }
    pub unsafe fn split(&mut self, work: *mut Self, heap: *mut SkArenaAlloc) -> bool {
        self.splitAt(work, ((*work).fStartT + (*work).fEndT) * 0.5, heap)
    }
    pub unsafe fn splitAt(&mut self, work: *mut Self, t: f64, heap: *mut SkArenaAlloc) -> bool {
        self.fStartT = t;
        self.fEndT = (*work).fEndT;
        if self.fStartT == self.fEndT {
            self.fCollapsed = true;
            return false;
        }
        (*work).fEndT = t;
        if (*work).fStartT == (*work).fEndT {
            (*work).fCollapsed = true;
            return false;
        }
        self.fPrev = work;
        self.fNext = (*work).fNext;
        self.fIsLinear = (*work).fIsLinear;
        self.fIsLine = (*work).fIsLine;
        (*work).fNext = self;
        if !self.fNext.is_null() {
            (*self.fNext).fPrev = self;
        }
        let mut b = (*work).fBounded;
        self.fBounded = null_mut();
        while !b.is_null() {
            self.addBounded((*b).fBounded, heap);
            b = (*b).fNext;
        }
        b = self.fBounded;
        while !b.is_null() {
            (*(*b).fBounded).addBounded(self, heap);
            b = (*b).fNext;
        }
        true
    }
}

pub struct SkTSect<'a> {
    pub(super) fCurve: &'a dyn SkTCurve,
    pub(super) fHeap: Pin<Box<SkSTArenaAlloc<1024>>>,
    pub(super) fHead: *mut SkTSpan,
    pub(super) fCoincident: *mut SkTSpan,
    pub(super) fDeleted: *mut SkTSpan,
    pub(super) fActiveCount: i32,
    pub(super) fRemovedStartT: bool,
    pub(super) fRemovedEndT: bool,
    pub(super) fHung: bool,
}
impl<'a> SkTSect<'a> {
    pub unsafe fn new(c: &'a dyn SkTCurve) -> Self {
        let mut result = Self {
            fCurve: c,
            fHeap: Box::pin(SkSTArenaAlloc::new(core::mem::size_of::<SkTSpan>() * 4)),
            fHead: null_mut(),
            fCoincident: null_mut(),
            fDeleted: null_mut(),
            fActiveCount: 0,
            fRemovedStartT: false,
            fRemovedEndT: false,
            fHung: false,
        };
        result.fHead = result.addOne();
        (*result.fHead).init(c);
        result
    }
    pub(super) unsafe fn heap(&mut self) -> *mut SkArenaAlloc {
        self.fHeap.as_mut().arena()
    }
    pub(super) unsafe fn addOne(&mut self) -> *mut SkTSpan {
        let result = if !self.fDeleted.is_null() {
            let r = self.fDeleted;
            self.fDeleted = (*r).fNext;
            r
        } else {
            let heap = self.heap();
            let value = SkTSpan::new(self.fCurve, heap);
            (*heap).make(value)
        };
        (*result).reset();
        (*result).fHasPerp = false;
        (*result).fDeleted = false;
        self.fActiveCount += 1;
        result
    }
    pub(super) unsafe fn pointLast(&self) -> &SkDPoint {
        self.fCurve.point(self.fCurve.pointLast())
    }
    pub(super) fn resetRemovedEnds(&mut self) {
        self.fRemovedStartT = false;
        self.fRemovedEndT = false;
    }
    pub(super) unsafe fn addSplitAt(&mut self, span: *mut SkTSpan, t: f64) -> *mut SkTSpan {
        let result = self.addOne();
        (*result).splitAt(span, t, self.heap());
        (*result).initBounds(self.fCurve);
        (*span).initBounds(self.fCurve);
        result
    }
    pub(super) unsafe fn addFollowing(&mut self, prior: *mut SkTSpan) -> *mut SkTSpan {
        let result = self.addOne();
        (*result).fStartT = if prior.is_null() { 0.0 } else { (*prior).fEndT };
        let next = if prior.is_null() {
            self.fHead
        } else {
            (*prior).fNext
        };
        (*result).fEndT = if next.is_null() { 1.0 } else { (*next).fStartT };
        (*result).fPrev = prior;
        (*result).fNext = next;
        if prior.is_null() {
            self.fHead = result;
        } else {
            (*prior).fNext = result;
        }
        if !next.is_null() {
            (*next).fPrev = result;
        }
        (*result).resetBounds(self.fCurve);
        result
    }
    pub(super) unsafe fn addForPerp(&mut self, span: *mut SkTSpan, t: f64) {
        if !(*span).hasOppT(t) {
            let mut prior = null_mut();
            let mut opp = self.spanAtT(t, &mut prior);
            if opp.is_null() {
                opp = self.addFollowing(prior);
            }
            (*opp).addBounded(span, self.heap());
            (*span).addBounded(opp, self.heap());
        }
    }
    pub(super) unsafe fn binarySearchCoin(
        &mut self,
        sect2: *mut SkTSect<'_>,
        tStart: f64,
        mut tStep: f64,
        resultT: &mut f64,
        oppT: &mut f64,
        oppFirst: &mut *mut SkTSpan,
    ) -> bool {
        let mut work = SkTSpan::new(self.fCurve, self.heap());
        let mut result = tStart;
        work.fStartT = tStart;
        work.fEndT = tStart;
        let mut last = self.fCurve.ptAtT(tStart);
        let mut oppPt = SkDPoint::default();
        let mut flip = false;
        let mut contained = false;
        let down = tStep < 0.0;
        let opp = (*sect2).fCurve;
        loop {
            tStep *= 0.5;
            work.fStartT += tStep;
            if flip {
                tStep = -tStep;
                flip = false;
            }
            work.initBounds(self.fCurve);
            if work.fCollapsed {
                return false;
            }
            if last.approximatelyEqual(work.pointFirst()) {
                break;
            }
            last = work.pointFirst();
            work.fCoinStart
                .setPerp(self.fCurve, work.fStartT, &last, opp);
            if work.fCoinStart.isMatch() {
                let test = work.fCoinStart.perpT();
                if (*(*sect2).fHead).contains(test) {
                    *oppT = test;
                    oppPt = work.fCoinStart.perpPt();
                    contained = true;
                    if if down {
                        result <= work.fStartT
                    } else {
                        result >= work.fStartT
                    } {
                        *oppFirst = null_mut();
                        return false;
                    }
                    result = work.fStartT;
                    continue;
                }
            }
            tStep = -tStep;
            flip = true;
        }
        if !contained {
            return false;
        }
        if last.approximatelyEqual(self.fCurve.point(0)) {
            result = 0.0;
        } else if last.approximatelyEqual(self.pointLast()) {
            result = 1.0;
        }
        if oppPt.approximatelyEqual(opp.point(0)) {
            *oppT = 0.0;
        } else if oppPt.approximatelyEqual((*sect2).pointLast()) {
            *oppT = 1.0;
        }
        *resultT = result;
        true
    }
    pub(super) unsafe fn boundsMax(&mut self) -> *mut SkTSpan {
        let mut test = self.fHead;
        let mut largest = self.fHead;
        let mut collapsed = (*largest).fCollapsed;
        let mut safety = 10000;
        loop {
            test = (*test).fNext;
            if test.is_null() {
                break;
            }
            safety -= 1;
            if safety == 0 {
                self.fHung = true;
                return null_mut();
            }
            let c = (*test).fCollapsed;
            if (collapsed && !c) || (collapsed == c && (*largest).fBoundsMax < (*test).fBoundsMax) {
                largest = test;
                collapsed = c;
            }
        }
        largest
    }
}

fn cpp_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}
fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}
const kZeroS1Set: i32 = 1;
const kOneS1Set: i32 = 2;
const kZeroS2Set: i32 = 4;
const kOneS2Set: i32 = 8;
#[derive(Clone, Copy)]
struct SkClosestRecord {
    fC1Span: *const SkTSpan,
    fC2Span: *const SkTSpan,
    fC1StartT: f64,
    fC1EndT: f64,
    fC2StartT: f64,
    fC2EndT: f64,
    fClosest: f64,
    fC1Index: usize,
    fC2Index: usize,
}
impl Default for SkClosestRecord {
    fn default() -> Self {
        Self {
            fC1Span: core::ptr::null(),
            fC2Span: core::ptr::null(),
            fC1StartT: 0.0,
            fC1EndT: 0.0,
            fC2StartT: 0.0,
            fC2EndT: 0.0,
            fClosest: f32::MAX as f64,
            fC1Index: 0,
            fC2Index: 0,
        }
    }
}
impl SkClosestRecord {
    unsafe fn addIntersection(&self, i: &mut SkIntersections) {
        let a = if self.fC1Index != 0 {
            (*self.fC1Span).endT()
        } else {
            (*self.fC1Span).startT()
        };
        let b = if self.fC2Index != 0 {
            (*self.fC2Span).endT()
        } else {
            (*self.fC2Span).startT()
        };
        i.insert(a, b, *(*self.fC1Span).part().point(self.fC1Index));
    }
    unsafe fn findEnd(&mut self, a: *const SkTSpan, b: *const SkTSpan, ai: usize, bi: usize) {
        let ac = (*a).part();
        let bc = (*b).part();
        if !ac.point(ai).approximatelyEqual(bc.point(bi)) {
            return;
        }
        let dist = ac.point(ai).distanceSquared(bc.point(bi));
        if self.fClosest < dist {
            return;
        }
        self.fC1Span = a;
        self.fC2Span = b;
        self.fC1StartT = (*a).startT();
        self.fC1EndT = (*a).endT();
        self.fC2StartT = (*b).startT();
        self.fC2EndT = (*b).endT();
        self.fC1Index = ai;
        self.fC2Index = bi;
        self.fClosest = dist;
    }
    unsafe fn matesWith(&self, m: &Self) -> bool {
        self.fC1Span == m.fC1Span
            || (*self.fC1Span).endT() == (*m.fC1Span).startT()
            || (*self.fC1Span).startT() == (*m.fC1Span).endT()
            || self.fC2Span == m.fC2Span
            || (*self.fC2Span).endT() == (*m.fC2Span).startT()
            || (*self.fC2Span).startT() == (*m.fC2Span).endT()
    }
    fn merge(&mut self, m: &Self) {
        self.fC1Span = m.fC1Span;
        self.fC2Span = m.fC2Span;
        self.fClosest = m.fClosest;
        self.fC1Index = m.fC1Index;
        self.fC2Index = m.fC2Index;
    }
    fn reset(&mut self) {
        self.fClosest = f32::MAX as f64;
    }
    fn update(&mut self, m: &Self) {
        self.fC1StartT = cpp_min(self.fC1StartT, m.fC1StartT);
        self.fC1EndT = cpp_max(self.fC1EndT, m.fC1EndT);
        self.fC2StartT = cpp_min(self.fC2StartT, m.fC2StartT);
        self.fC2EndT = cpp_max(self.fC2EndT, m.fC2EndT);
    }
}
struct SkClosestSect {
    fClosest: Vec<SkClosestRecord>,
    fUsed: usize,
}
impl SkClosestSect {
    fn new() -> Self {
        Self {
            fClosest: vec![SkClosestRecord::default()],
            fUsed: 0,
        }
    }
    unsafe fn find(&mut self, a: *const SkTSpan, b: *const SkTSpan) -> bool {
        let r = &mut self.fClosest[self.fUsed];
        r.findEnd(a, b, 0, 0);
        r.findEnd(a, b, 0, (*b).part().pointLast());
        r.findEnd(a, b, (*a).part().pointLast(), 0);
        r.findEnd(a, b, (*a).part().pointLast(), (*b).part().pointLast());
        if r.fClosest == f32::MAX as f64 {
            return false;
        }
        let record = *r;
        for index in 0..self.fUsed {
            let test = &mut self.fClosest[index];
            if test.matesWith(&record) {
                if test.fClosest > record.fClosest {
                    test.merge(&record);
                }
                test.update(&record);
                self.fClosest[self.fUsed].reset();
                return false;
            }
        }
        self.fUsed += 1;
        self.fClosest.push(SkClosestRecord::default());
        true
    }
    unsafe fn finish(&self, i: &mut SkIntersections) {
        let mut closest: Vec<_> = self.fClosest[..self.fUsed].iter().collect();
        SkTQSort(&mut closest, &|a, b| a.fClosest < b.fClosest);
        for r in closest {
            r.addIntersection(i);
        }
    }
}
impl<'a> SkTSect<'a> {
    unsafe fn EndsEqual(
        a: *const SkTSect<'_>,
        b: *const SkTSect<'_>,
        i: &mut SkIntersections,
    ) -> i32 {
        let mut set = 0;
        let af = *(*a).fCurve.point(0);
        let al = *(*a).pointLast();
        let bf = *(*b).fCurve.point(0);
        let bl = *(*b).pointLast();
        if af == bf {
            set |= kZeroS1Set | kZeroS2Set;
            i.insert(0.0, 0.0, af);
        }
        if af == bl {
            set |= kZeroS1Set | kOneS2Set;
            i.insert(0.0, 1.0, af);
        }
        if al == bf {
            set |= kOneS1Set | kZeroS2Set;
            i.insert(1.0, 0.0, al);
        }
        if al == bl {
            set |= kOneS1Set | kOneS2Set;
            i.insert(1.0, 1.0, al);
        }
        if set & (kZeroS1Set | kZeroS2Set) == 0 && af.approximatelyEqual(bf) {
            set |= kZeroS1Set | kZeroS2Set;
            i.insertNear(0.0, 0.0, af, bf);
        }
        if set & (kZeroS1Set | kOneS2Set) == 0 && af.approximatelyEqual(bl) {
            set |= kZeroS1Set | kOneS2Set;
            i.insertNear(0.0, 1.0, af, bl);
        }
        if set & (kOneS1Set | kZeroS2Set) == 0 && al.approximatelyEqual(bf) {
            set |= kOneS1Set | kZeroS2Set;
            i.insertNear(1.0, 0.0, al, bf);
        }
        if set & (kOneS1Set | kOneS2Set) == 0 && al.approximatelyEqual(bl) {
            set |= kOneS1Set | kOneS2Set;
            i.insertNear(1.0, 1.0, al, bl);
        }
        set
    }
    pub unsafe fn BinarySearch(a: *mut SkTSect<'_>, b: *mut SkTSect<'_>, i: &mut SkIntersections) {
        i.reset();
        i.setMax((*a).fCurve.maxIntersections() + 4);
        let span1 = (*a).fHead;
        let span2 = (*b).fHead;
        let mut oppSect = 0;
        let sect = (*a).intersects(span1, b, span2, &mut oppSect);
        if sect == 0 {
            return;
        }
        if sect == 2 && oppSect == 2 {
            Self::EndsEqual(a, b, i);
            return;
        }
        (*span1).addBounded(span2, (*a).heap());
        (*span2).addBounded(span1, (*b).heap());
        const MAX_COIN_LOOP: i32 = 8;
        const COINCIDENT_SPAN_COUNT: i32 = 9;
        let mut coinLoop = MAX_COIN_LOOP;
        let mut start1s = 0.0;
        let mut start1e = 0.0;
        loop {
            let largest1 = (*a).boundsMax();
            if largest1.is_null() {
                if (*a).fHung {
                    return;
                }
                break;
            }
            let largest2 = (*b).boundsMax();
            if largest2.is_null()
                || (!largest1.is_null()
                    && ((*largest1).fBoundsMax > (*largest2).fBoundsMax
                        || (!(*largest1).fCollapsed && (*largest2).fCollapsed)))
            {
                if (*b).fHung {
                    return;
                }
                if (*largest1).fCollapsed {
                    break;
                }
                (*a).resetRemovedEnds();
                (*b).resetRemovedEnds();
                let half = (*a).addOne();
                if !(*half).split(largest1, (*a).heap()) {
                    break;
                }
                if !(*a).trim(largest1, b) || !(*a).trim(half, b) {
                    return;
                }
            } else {
                if (*largest2).fCollapsed {
                    break;
                }
                (*a).resetRemovedEnds();
                (*b).resetRemovedEnds();
                let half = (*b).addOne();
                if !(*half).split(largest2, (*b).heap()) {
                    break;
                }
                if !(*b).trim(largest2, a) || !(*b).trim(half, a) {
                    return;
                }
            }
            if (*a).fActiveCount >= COINCIDENT_SPAN_COUNT
                && (*b).fActiveCount >= COINCIDENT_SPAN_COUNT
            {
                if coinLoop == MAX_COIN_LOOP {
                    start1s = (*(*a).fHead).fStartT;
                    start1e = (*(*a).tail()).fEndT;
                }
                if !(*a).coincidentCheck(b) {
                    return;
                }
                coinLoop -= 1;
                if coinLoop == 0 && !(*a).fHead.is_null() && !(*b).fHead.is_null() {
                    (*a).coincidentForce(b, start1s, start1e);
                }
            }
            if (*a).fActiveCount >= COINCIDENT_SPAN_COUNT
                && (*b).fActiveCount >= COINCIDENT_SPAN_COUNT
            {
                if (*a).fHead.is_null() {
                    return;
                }
                (*a).computePerpendiculars(b, (*a).fHead, (*a).tail());
                if (*b).fHead.is_null() {
                    return;
                }
                (*b).computePerpendiculars(a, (*b).fHead, (*b).tail());
                if !(*a).removeByPerpendicular(b) {
                    return;
                }
                if (*a).collapsed() > (*a).fCurve.maxIntersections() as i32 {
                    break;
                }
            }
            if (*a).fHead.is_null() || (*b).fHead.is_null() {
                break;
            }
        }
        let mut coincident = (*a).fCoincident;
        if !coincident.is_null() {
            if !(*coincident).fNext.is_null() {
                (*a).mergeCoincidence(b);
                coincident = (*a).fCoincident;
            }

            loop {
                if coincident.is_null() {
                    return;
                }
                if (*coincident).fCoinStart.isMatch() && (*coincident).fCoinEnd.isMatch() {
                    let t = (*coincident).fCoinStart.perpT();
                    if t < 0.0 {
                        return;
                    }
                    let index =
                        i.insertCoincident((*coincident).fStartT, t, (*coincident).pointFirst());
                    if i.insertCoincident(
                        (*coincident).fEndT,
                        (*coincident).fCoinEnd.perpT(),
                        (*coincident).pointLast(),
                    ) < 0
                        && index >= 0
                    {
                        i.clearCoincidence(index as usize);
                    }
                }
                coincident = (*coincident).fNext;
                if coincident.is_null() {
                    break;
                }
            }
        }
        let set = Self::EndsEqual(a, b, i);
        for (removed, bit, curve, t, pt, opp, reverse) in [
            (
                (*a).fRemovedStartT,
                kZeroS1Set,
                (*a).fCurve,
                0.0,
                *(*a).fCurve.point(0),
                (*b).fCurve,
                false,
            ),
            (
                (*a).fRemovedEndT,
                kOneS1Set,
                (*a).fCurve,
                1.0,
                *(*a).pointLast(),
                (*b).fCurve,
                false,
            ),
            (
                (*b).fRemovedStartT,
                kZeroS2Set,
                (*b).fCurve,
                0.0,
                *(*b).fCurve.point(0),
                (*a).fCurve,
                true,
            ),
            (
                (*b).fRemovedEndT,
                kOneS2Set,
                (*b).fCurve,
                1.0,
                *(*b).pointLast(),
                (*a).fCurve,
                true,
            ),
        ] {
            if removed && set & bit == 0 {
                let mut perp = SkTCoincident::default();
                perp.setPerp(curve, t, pt, opp);
                if perp.isMatch() {
                    if reverse {
                        i.insert(perp.perpT(), t, perp.perpPt());
                    } else {
                        i.insert(t, perp.perpT(), perp.perpPt());
                    }
                }
            }
        }
        if (*a).fHead.is_null() || (*b).fHead.is_null() {
            return;
        }
        (*a).recoverCollapsed();
        (*b).recoverCollapsed();
        let mut result1 = (*a).fHead;
        let head1 = result1;
        if set & kZeroS1Set == 0
            && approximately_less_than_zero((*head1).fStartT)
            && (*head1).isBounded()
        {
            let start = (*a).fCurve.point(0);
            let t = (*head1).closestBoundedT(start);
            if (*b).fCurve.ptAtT(t).approximatelyEqual(start) {
                i.insert(0.0, t, *start);
            }
        }
        let head2 = (*b).fHead;
        if set & kZeroS2Set == 0
            && approximately_less_than_zero((*head2).fStartT)
            && (*head2).isBounded()
        {
            let start = (*b).fCurve.point(0);
            let t = (*head2).closestBoundedT(start);
            if (*a).fCurve.ptAtT(t).approximatelyEqual(start) {
                i.insert(t, 0.0, *start);
            }
        }
        if set & kOneS1Set == 0 {
            let tail = (*a).tail();
            if tail.is_null() {
                return;
            }
            if approximately_greater_than_one((*tail).fEndT) && (*tail).isBounded() {
                let end = (*a).pointLast();
                let t = (*tail).closestBoundedT(end);
                if (*b).fCurve.ptAtT(t).approximatelyEqual(end) {
                    i.insert(1.0, t, *end);
                }
            }
        }
        if set & kOneS2Set == 0 {
            let tail = (*b).tail();
            if tail.is_null() {
                return;
            }
            if approximately_greater_than_one((*tail).fEndT) && (*tail).isBounded() {
                let end = (*b).pointLast();
                let t = (*tail).closestBoundedT(end);
                if (*a).fCurve.ptAtT(t).approximatelyEqual(end) {
                    i.insert(t, 1.0, *end);
                }
            }
        }
        let mut closest = SkClosestSect::new();
        loop {
            while !result1.is_null()
                && (*result1).fCoinStart.isMatch()
                && (*result1).fCoinEnd.isMatch()
            {
                result1 = (*result1).fNext;
            }
            if result1.is_null() {
                break;
            }
            let mut result2 = (*b).fHead;
            while !result2.is_null() {
                closest.find(result1, result2);
                result2 = (*result2).fNext;
            }
            result1 = (*result1).fNext;
            if result1.is_null() {
                break;
            }
        }
        closest.finish(i);
        let mut last = i.used() as i32 - 1;
        let mut index = 0;
        while index < last {
            let idx = index as usize;
            if i.isCoincident(idx) && i.isCoincident(idx + 1) {
                index += 1;
                continue;
            }
            let mid = (i[0][idx] + i[0][idx + 1]) / 2.0;
            let pt = (*a).fCurve.ptAtT(mid);
            let mut perp = SkTCoincident::default();
            perp.setPerp((*a).fCurve, mid, pt, (*b).fCurve);
            if !perp.isMatch() {
                index += 1;
                continue;
            }
            if i.isCoincident(idx) {
                i.removeOne(idx);
                last -= 1;
            } else if i.isCoincident(idx + 1) {
                i.removeOne(idx + 1);
                last -= 1;
            } else {
                i.setCoincident(idx);
                index += 1;
            }
            i.setCoincident(index as usize);
        }
    }
}

impl SkIntersections {
    pub fn intersectQuadQuad(
        &mut self,
        a: &super::SkPathOpsQuad::SkDQuad,
        b: &super::SkPathOpsQuad::SkDQuad,
    ) -> usize {
        let a = super::SkPathOpsTCurve::SkTQuad::new(a);
        let b = super::SkPathOpsTCurve::SkTQuad::new(b);
        unsafe {
            let mut a = SkTSect::new(&a);
            let mut b = SkTSect::new(&b);
            SkTSect::BinarySearch(&mut a, &mut b, self);
        }
        self.used()
    }
    pub fn intersectConicQuad(
        &mut self,
        a: &super::SkPathOpsConic::SkDConic,
        b: &super::SkPathOpsQuad::SkDQuad,
    ) -> usize {
        let a = super::SkPathOpsTCurve::SkTConic::new(a);
        let b = super::SkPathOpsTCurve::SkTQuad::new(b);
        unsafe {
            let mut a = SkTSect::new(&a);
            let mut b = SkTSect::new(&b);
            SkTSect::BinarySearch(&mut a, &mut b, self);
        }
        self.used()
    }
    pub fn intersectConicConic(
        &mut self,
        a: &super::SkPathOpsConic::SkDConic,
        b: &super::SkPathOpsConic::SkDConic,
    ) -> usize {
        let a = super::SkPathOpsTCurve::SkTConic::new(a);
        let b = super::SkPathOpsTCurve::SkTConic::new(b);
        unsafe {
            let mut a = SkTSect::new(&a);
            let mut b = SkTSect::new(&b);
            SkTSect::BinarySearch(&mut a, &mut b, self);
        }
        self.used()
    }
    pub fn intersectCubicQuad(
        &mut self,
        a: &super::SkPathOpsCubic::SkDCubic,
        b: &super::SkPathOpsQuad::SkDQuad,
    ) -> usize {
        let a = super::SkPathOpsTCurve::SkTCubic::new(a);
        let b = super::SkPathOpsTCurve::SkTQuad::new(b);
        unsafe {
            let mut a = SkTSect::new(&a);
            let mut b = SkTSect::new(&b);
            SkTSect::BinarySearch(&mut a, &mut b, self);
        }
        self.used()
    }
    pub fn intersectCubicConic(
        &mut self,
        a: &super::SkPathOpsCubic::SkDCubic,
        b: &super::SkPathOpsConic::SkDConic,
    ) -> usize {
        let a = super::SkPathOpsTCurve::SkTCubic::new(a);
        let b = super::SkPathOpsTCurve::SkTConic::new(b);
        unsafe {
            let mut a = SkTSect::new(&a);
            let mut b = SkTSect::new(&b);
            SkTSect::BinarySearch(&mut a, &mut b, self);
        }
        self.used()
    }
    pub fn intersectCubicCubic(
        &mut self,
        a: &super::SkPathOpsCubic::SkDCubic,
        b: &super::SkPathOpsCubic::SkDCubic,
    ) -> usize {
        let a = super::SkPathOpsTCurve::SkTCubic::new(a);
        let b = super::SkPathOpsTCurve::SkTCubic::new(b);
        unsafe {
            let mut a = SkTSect::new(&a);
            let mut b = SkTSect::new(&b);
            SkTSect::BinarySearch(&mut a, &mut b, self);
        }
        self.used()
    }
}
