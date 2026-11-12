// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkOpSegment.h and SkOpSegment.cpp.
#![allow(non_snake_case)]
use super::{
    SkOpAngle::{IncludeType, SkOpAngle},
    SkOpContour::SkOpContour,
    SkOpSpan::{Collapsed, SkOpPtT, SkOpSpan, SkOpSpanBase},
    SkPathOpsBounds::SkPathOpsBounds,
    SkPathOpsCurve::{SkDCurve, SkDCurveSweep},
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsTypes::*,
    SkPathWriter::SkPathWriter,
};
use crate::{
    include::pathops::SkPathOps::SkPathOp,
    src::core::{SkPath::Verb, SkPoint::SkPoint},
};
use core::ptr::{addr_of, addr_of_mut, null_mut};

pub struct SkOpSegment {
    pub(super) fHead: SkOpSpan,
    pub(super) fTail: SkOpSpanBase,
    pub(super) fContour: *mut SkOpContour,
    pub(super) fNext: *mut Self,
    pub(super) fPrev: *mut Self,
    pub(super) fPts: *mut SkPoint,
    pub(super) fBounds: SkPathOpsBounds,
    pub(super) fWeight: f32,
    pub(super) fCount: i32,
    pub(super) fDoneCount: i32,
    pub(super) fVerb: Verb,
    pub(super) fVisited: bool,
}
impl Default for SkOpSegment {
    fn default() -> Self {
        Self {
            fHead: SkOpSpan::default(),
            fTail: SkOpSpanBase::default(),
            fContour: null_mut(),
            fNext: null_mut(),
            fPrev: null_mut(),
            fPts: null_mut(),
            fBounds: SkPathOpsBounds::default(),
            fWeight: 0.0,
            fCount: 0,
            fDoneCount: 0,
            fVerb: Verb::kMove_Verb,
            fVisited: false,
        }
    }
}
const kUnaryActiveEdge: [[bool; 2]; 2] = [[false, true], [true, false]];
const kActiveEdge: [[[[[bool; 2]; 2]; 2]; 2]; 4] = [
    [
        [
            [[false, false], [false, false]],
            [[true, false], [true, false]],
        ],
        [
            [[true, true], [false, false]],
            [[false, true], [true, false]],
        ],
    ],
    [
        [
            [[false, false], [false, false]],
            [[false, true], [false, true]],
        ],
        [
            [[false, false], [true, true]],
            [[false, true], [true, false]],
        ],
    ],
    [
        [
            [[false, true], [true, false]],
            [[true, true], [false, false]],
        ],
        [
            [[true, false], [true, false]],
            [[false, false], [false, false]],
        ],
    ],
    [
        [
            [[false, true], [true, false]],
            [[true, false], [false, true]],
        ],
        [
            [[true, false], [false, true]],
            [[false, true], [true, false]],
        ],
    ],
];
impl SkOpSegment {
    pub fn lessThan(&self, other: &Self) -> bool {
        self.fBounds.fTop < other.fBounds.fTop
    }
    pub fn bounds(&self) -> &SkPathOpsBounds {
        &self.fBounds
    }
    pub fn bumpCount(&mut self) {
        self.fCount += 1;
    }
    pub fn contour(&self) -> *mut SkOpContour {
        self.fContour
    }
    pub fn count(&self) -> i32 {
        self.fCount
    }
    pub fn done(&self) -> bool {
        self.fDoneCount == self.fCount
    }
    pub unsafe fn doneAngle(&self, angle: *const SkOpAngle) -> bool {
        (*(*(*angle).start()).starter((*angle).end())).done()
    }
    pub fn head(&self) -> *mut SkOpSpan {
        addr_of!(self.fHead).cast_mut()
    }
    pub fn tail(&self) -> *mut SkOpSpanBase {
        addr_of!(self.fTail).cast_mut()
    }
    pub fn next(&self) -> *mut Self {
        self.fNext
    }
    pub fn prev(&self) -> *mut Self {
        self.fPrev
    }
    pub fn pts(&self) -> *mut SkPoint {
        self.fPts
    }
    pub fn verb(&self) -> Verb {
        self.fVerb
    }
    pub fn weight(&self) -> f32 {
        self.fWeight
    }
    pub fn isHorizontal(&self) -> bool {
        self.fBounds.fTop == self.fBounds.fBottom
    }
    pub fn isVertical(&self) -> bool {
        self.fBounds.fLeft == self.fBounds.fRight
    }
    pub unsafe fn isSimple(&self, end: &mut *mut SkOpSpanBase, step: &mut i32) -> *mut Self {
        self.nextChase(end, step, null_mut(), null_mut())
    }
    pub unsafe fn joinEnds(&mut self, start: *mut Self) {
        let other = (*(*start).head()).ptT();
        self.fTail.fPtT.addOpp(other, other);
    }
    pub unsafe fn lastPt(&self) -> &SkPoint {
        &*self.fPts.add(SkPathOpsVerbToPoints(self.fVerb) as usize)
    }
    pub fn resetVisited(&mut self) {
        self.fVisited = false;
    }
    pub fn setContour(&mut self, contour: *mut SkOpContour) {
        self.fContour = contour;
    }
    pub fn setNext(&mut self, next: *mut Self) {
        self.fNext = next;
    }
    pub fn setPrev(&mut self, prev: *mut Self) {
        self.fPrev = prev;
    }
    pub fn visited(&mut self) -> bool {
        if !self.fVisited {
            self.fVisited = true;
            false
        } else {
            true
        }
    }
    pub unsafe fn globalState(&self) -> *mut SkOpGlobalState {
        (*self.fContour).globalState()
    }
    pub unsafe fn operand(&self) -> bool {
        (*self.fContour).operand()
    }
    pub unsafe fn oppXor(&self) -> bool {
        (*self.fContour).oppXor()
    }
    pub unsafe fn isXor(&self) -> bool {
        (*self.fContour).isXor()
    }
    pub unsafe fn OppSign(start: *const SkOpSpanBase, end: *const SkOpSpanBase) -> i32 {
        if (*start).t() < (*end).t() {
            -(*(*start).upCast()).oppValue()
        } else {
            (*(*end).upCast()).oppValue()
        }
    }
    pub unsafe fn SpanSign(start: *const SkOpSpanBase, end: *const SkOpSpanBase) -> i32 {
        if (*start).t() < (*end).t() {
            -(*(*start).upCast()).windValue()
        } else {
            (*(*end).upCast()).windValue()
        }
    }
    pub unsafe fn setUpWinding(
        &self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        max: &mut i32,
        sum: &mut i32,
    ) {
        let delta = Self::SpanSign(start, end);
        *max = *sum;
        if *sum == i32::MIN {
            return;
        }
        *sum -= delta;
    }
    pub unsafe fn spanToAngle(
        &self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
    ) -> *mut SkOpAngle {
        if (*start).t() < (*end).t() {
            (*(*start).upCast()).toAngle()
        } else {
            (*start).fromAngle()
        }
    }
    pub unsafe fn insert(&mut self, prev: *mut SkOpSpan) -> *mut SkOpSpan {
        let global = self.globalState();
        (*global).setAllocatedOpSpan();
        let result = (*(*global).allocator()).make(SkOpSpan::default());
        let next = (*prev).next();
        (*result).setPrev(prev);
        (*prev).setNext(result.cast());
        (*result).setNext(next);
        if !next.is_null() {
            (*next).setPrev(result);
        }
        result
    }
    pub unsafe fn activeAngle(
        &mut self,
        start: *mut SkOpSpanBase,
        startPtr: &mut *mut SkOpSpanBase,
        endPtr: &mut *mut SkOpSpanBase,
        done: &mut bool,
    ) -> *mut SkOpAngle {
        let result = self.activeAngleInner(start, startPtr, endPtr, done);
        if !result.is_null() {
            return result;
        }
        self.activeAngleOther(start, startPtr, endPtr, done)
    }
    pub unsafe fn activeAngleInner(
        &self,
        start: *mut SkOpSpanBase,
        startPtr: &mut *mut SkOpSpanBase,
        endPtr: &mut *mut SkOpSpanBase,
        done: &mut bool,
    ) -> *mut SkOpAngle {
        let up = (*start).upCastable();
        if !up.is_null() {
            if (*up).windValue() != 0 || (*up).oppValue() != 0 {
                let next = (*up).next();
                if (*endPtr).is_null() {
                    *startPtr = start;
                    *endPtr = next;
                }
                if !(*up).done() {
                    if (*up).windSum() != i32::MIN {
                        return self.spanToAngle(start, next);
                    }
                    *done = false;
                }
            } else {
            }
        }
        let down = (*start).prev();
        if !down.is_null() {
            if (*down).windValue() != 0 || (*down).oppValue() != 0 {
                if (*endPtr).is_null() {
                    *startPtr = start;
                    *endPtr = down.cast();
                }
                if !(*down).done() {
                    if (*down).windSum() != i32::MIN {
                        return self.spanToAngle(start, down.cast());
                    }
                    *done = false;
                }
            } else {
            }
        }
        null_mut()
    }
    pub unsafe fn activeAngleOther(
        &self,
        start: *mut SkOpSpanBase,
        startPtr: &mut *mut SkOpSpanBase,
        endPtr: &mut *mut SkOpSpanBase,
        done: &mut bool,
    ) -> *mut SkOpAngle {
        let otherPt = (*(*start).ptT()).next();
        (*(*otherPt).segment()).activeAngleInner((*otherPt).span(), startPtr, endPtr, done)
    }
    pub unsafe fn activeOp(
        &mut self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        xorMiMask: i32,
        xorSuMask: i32,
        op: SkPathOp,
    ) -> bool {
        let mut mi = self.updateWinding(end, start);
        let mut su = self.updateOppWinding(end, start);
        if self.operand() {
            core::mem::swap(&mut mi, &mut su);
        }
        self.activeOpSums(xorMiMask, xorSuMask, start, end, op, &mut mi, &mut su)
    }
    pub unsafe fn activeOpSums(
        &mut self,
        xorMiMask: i32,
        xorSuMask: i32,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        op: SkPathOp,
        mi: &mut i32,
        su: &mut i32,
    ) -> bool {
        let (mut max, mut sum, mut oppMax, mut oppSum) = (0, 0, 0, 0);
        self.setUpWindings(
            start,
            end,
            mi,
            su,
            &mut max,
            &mut sum,
            &mut oppMax,
            &mut oppSum,
        );
        let (mf, mt, sf, st) = if self.operand() {
            (
                (oppMax & xorMiMask) != 0,
                (oppSum & xorMiMask) != 0,
                (max & xorSuMask) != 0,
                (sum & xorSuMask) != 0,
            )
        } else {
            (
                (max & xorMiMask) != 0,
                (sum & xorMiMask) != 0,
                (oppMax & xorSuMask) != 0,
                (oppSum & xorSuMask) != 0,
            )
        };
        kActiveEdge[op as usize][mf as usize][mt as usize][sf as usize][st as usize]
    }
    pub unsafe fn activeWinding(
        &mut self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
    ) -> bool {
        let mut sum = self.updateWinding(end, start);
        self.activeWindingSum(start, end, &mut sum)
    }
    pub unsafe fn activeWindingSum(
        &self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        sum: &mut i32,
    ) -> bool {
        let mut max = 0;
        self.setUpWinding(start, end, &mut max, sum);
        kUnaryActiveEdge[(max != 0) as usize][(*sum != 0) as usize]
    }
    pub unsafe fn addExpanded(
        &mut self,
        newT: f64,
        test: *const SkOpSpanBase,
        startOver: &mut bool,
    ) -> bool {
        if self.contains(newT) {
            return true;
        }
        let global = self.globalState();
        (*global).resetAllocatedOpSpan();
        if !between(0.0, newT, 1.0) {
            return false;
        }
        let newPt = self.addT(newT);
        *startOver |= (*global).allocatedOpSpan();
        if newPt.is_null() {
            return false;
        }
        (*newPt).fPt = self.ptAtT(newT);
        let prev = (*(*test).ptT()).oppPrev(newPt);
        if !prev.is_null() {
            let test = test.cast_mut();
            (*test).mergeMatches((*newPt).span());
            (*(*test).ptT()).addOpp(newPt, prev);
            (*test).checkForCollapsedCoincidence();
        }
        true
    }
    pub unsafe fn addTPoint(&mut self, t: f64, pt: &SkPoint) -> *mut SkOpPtT {
        let mut spanBase = self.head().cast::<SkOpSpanBase>();
        loop {
            let result = (*spanBase).ptT();
            if t == (*result).fT || (!zero_or_one(t) && self.match_(result, self, t, pt)) {
                (*spanBase).bumpSpanAdds();
                return result;
            }
            if t < (*result).fT {
                let prev = (*(*result).span()).prev();
                if prev.is_null() {
                    return null_mut();
                }
                let span = self.insert(prev);
                (*span).init(self, prev, t, pt);
                (*span).bumpSpanAdds();
                return (*span).ptT();
            }
            if spanBase == self.tail() {
                return null_mut();
            }
            spanBase = (*(*spanBase).upCast()).next();
            if spanBase.is_null() {
                return null_mut();
            }
        }
    }
    pub unsafe fn addT(&mut self, t: f64) -> *mut SkOpPtT {
        let point = self.ptAtT(t);
        self.addTPoint(t, &point)
    }
    pub unsafe fn addStartSpan(&mut self) -> *mut SkOpAngle {
        let angle = (*(*self.globalState()).allocator()).make(SkOpAngle::default());
        (*angle).set(self.head().cast(), self.fHead.next());
        self.fHead.setToAngle(angle);
        angle
    }
    pub unsafe fn addEndSpan(&mut self) -> *mut SkOpAngle {
        let angle = (*(*self.globalState()).allocator()).make(SkOpAngle::default());
        (*angle).set(self.tail(), self.fTail.prev().cast());
        self.fTail.setFromAngle(angle);
        angle
    }
    pub unsafe fn calcAngles(&mut self) {
        let mut activePrior = !self.fHead.isCanceled();
        if activePrior && !self.fHead.simple() {
            self.addStartSpan();
        }
        let mut prior = self.head();
        let mut spanBase = self.fHead.next();
        while spanBase != self.tail() {
            if activePrior {
                let angle = (*(*self.globalState()).allocator()).make(SkOpAngle::default());
                (*angle).set(spanBase, prior.cast());
                (*spanBase).setFromAngle(angle);
            }
            let span = (*spanBase).upCast();
            let active = !(*span).isCanceled();
            let next = (*span).next();
            if active {
                let angle = (*(*self.globalState()).allocator()).make(SkOpAngle::default());
                (*angle).set(span.cast(), next);
                (*span).setToAngle(angle);
            }
            activePrior = active;
            prior = span;
            spanBase = next;
        }
        if activePrior && !self.fTail.simple() {
            self.addEndSpan();
        }
    }
    pub unsafe fn clearAll(&mut self) {
        let mut span = self.head();
        loop {
            self.clearOne(span);
            span = (*(*span).next()).upCastable();
            if span.is_null() {
                break;
            }
        }
        (*(*self.globalState()).coincidence()).release(self);
    }
    pub unsafe fn clearOne(&mut self, span: *mut SkOpSpan) {
        (*span).setWindValue(0);
        (*span).setOppValue(0);
        self.markDone(span);
    }
    pub unsafe fn collapsed(&self, s: f64, e: f64) -> Collapsed {
        let mut span = self.head().cast::<SkOpSpanBase>();
        loop {
            let result = (*span).collapsed(s, e);
            if result != Collapsed::kNo {
                return result;
            }
            let up = (*span).upCastable();
            if up.is_null() {
                return Collapsed::kNo;
            }
            span = (*up).next();
            if span.is_null() {
                return Collapsed::kNo;
            }
        }
    }
    pub unsafe fn contains(&self, newT: f64) -> bool {
        let mut span = self.head().cast::<SkOpSpanBase>();
        loop {
            if (*(*span).ptT()).containsSegmentT(self, newT) {
                return true;
            }
            if span == self.tail() {
                break;
            }
            span = (*(*span).upCast()).next();
        }
        false
    }
    pub unsafe fn release(&mut self, span: *const SkOpSpan) {
        if (*span).done() {
            self.fDoneCount -= 1;
        }
        self.fCount -= 1;
    }
}

impl SkOpSegment {
    pub unsafe fn ComputeOneSum(
        baseAngle: *const SkOpAngle,
        nextAngle: *mut SkOpAngle,
        include: IncludeType,
    ) -> bool {
        let base = (*baseAngle).segment();
        let mut mi = (*base).updateWindingReverse(baseAngle);
        let mut su = 0;
        let binary = include as i32 >= IncludeType::kBinarySingle as i32;
        if binary {
            su = (*base).updateOppWindingReverse(baseAngle);
            if (*base).operand() {
                core::mem::swap(&mut mi, &mut su);
            }
        }
        let next = (*nextAngle).segment();
        let (mut max, mut sum) = (0, 0);
        let mut last = null_mut();
        if binary {
            let (mut om, mut os) = (0, 0);
            (*next).setUpWindings(
                (*nextAngle).start(),
                (*nextAngle).end(),
                &mut mi,
                &mut su,
                &mut max,
                &mut sum,
                &mut om,
                &mut os,
            );
            if !(*next).markAngleOp(max, sum, om, os, nextAngle, &mut last) {
                return false;
            }
        } else {
            (*next).setUpWindingsUnary(
                (*nextAngle).start(),
                (*nextAngle).end(),
                &mut mi,
                &mut max,
                &mut sum,
            );
            if !(*next).markAngle(max, sum, nextAngle, &mut last) {
                return false;
            }
        }
        (*nextAngle).setLastMarked(last);
        true
    }
    pub unsafe fn ComputeOneSumReverse(
        baseAngle: *mut SkOpAngle,
        nextAngle: *mut SkOpAngle,
        include: IncludeType,
    ) -> bool {
        let base = (*baseAngle).segment();
        let mut mi = (*base).updateWindingAngle(baseAngle);
        let mut su = 0;
        let binary = include as i32 >= IncludeType::kBinarySingle as i32;
        if binary {
            su = (*base).updateOppWindingAngle(baseAngle);
            if (*base).operand() {
                core::mem::swap(&mut mi, &mut su);
            }
        }
        let next = (*nextAngle).segment();
        let (mut max, mut sum) = (0, 0);
        let mut last = null_mut();
        if binary {
            let (mut om, mut os) = (0, 0);
            (*next).setUpWindings(
                (*nextAngle).end(),
                (*nextAngle).start(),
                &mut mi,
                &mut su,
                &mut max,
                &mut sum,
                &mut om,
                &mut os,
            );
            if !(*next).markAngleOp(max, sum, om, os, nextAngle, &mut last) {
                return false;
            }
        } else {
            (*next).setUpWindingsUnary(
                (*nextAngle).end(),
                (*nextAngle).start(),
                &mut mi,
                &mut max,
                &mut sum,
            );
            if !(*next).markAngle(max, sum, nextAngle, &mut last) {
                return false;
            }
        }
        (*nextAngle).setLastMarked(last);
        true
    }
    pub unsafe fn computeSum(
        &mut self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        include: IncludeType,
    ) -> i32 {
        let mut first = self.spanToAngle(end, start);
        if first.is_null() || (*first).next().is_null() {
            return i32::MIN;
        }
        let mut base: *mut SkOpAngle = null_mut();
        let mut reverse = false;
        let mut angle = (*first).previous();
        let mut next = (*angle).next();
        first = next;
        loop {
            let prior = angle;
            angle = next;
            next = (*angle).next();

            if (*prior).unorderable() || (*angle).unorderable() || (*next).unorderable() {
                base = null_mut();
            } else {
                let winding = (*(*angle).starter()).windSum();
                if winding != i32::MIN {
                    base = angle;
                    reverse = true;
                } else if !base.is_null() {
                    Self::ComputeOneSum(base, angle, include);
                    base = if (*(*angle).starter()).windSum() != i32::MIN {
                        angle
                    } else {
                        null_mut()
                    };
                }
            }
            if next == first {
                break;
            }
        }
        if !base.is_null() && (*(*first).starter()).windSum() == i32::MIN {
            first = base;
            reverse = true;
        }
        if reverse {
            base = null_mut();
            let mut prior = first;
            loop {
                angle = prior;
                prior = (*angle).previous();

                next = (*angle).next();
                if (*prior).unorderable() || (*angle).unorderable() || (*next).unorderable() {
                    base = null_mut();
                } else {
                    let winding = (*(*angle).starter()).windSum();
                    if winding != i32::MIN {
                        base = angle;
                    } else if !base.is_null() {
                        Self::ComputeOneSumReverse(base, angle, include);
                        base = if (*(*angle).starter()).windSum() != i32::MIN {
                            angle
                        } else {
                            null_mut()
                        };
                    }
                }
                if prior == first {
                    break;
                }
            }
        }
        (*(*start).starter(end)).windSum()
    }
    pub unsafe fn init(
        &mut self,
        pts: *mut SkPoint,
        weight: f32,
        contour: *mut SkOpContour,
        verb: Verb,
    ) {
        self.fContour = contour;
        self.fNext = null_mut();
        self.fPts = pts;
        self.fWeight = weight;
        self.fVerb = verb;
        self.fCount = 0;
        self.fDoneCount = 0;
        self.fVisited = false;
        let zero = self.head();
        (*zero).init(self, null_mut(), 0.0, &*self.fPts);
        let one = self.tail();
        (*zero).setNext(one);
        (*one).initBase(
            self,
            zero,
            1.0,
            &*self.fPts.add(SkPathOpsVerbToPoints(self.fVerb) as usize),
        );
    }
    pub unsafe fn markAllDone(&mut self) {
        let mut span = self.head();
        loop {
            self.markDone(span);
            span = (*(*span).next()).upCastable();
            if span.is_null() {
                break;
            }
        }
    }
    pub unsafe fn markAndChaseDone(
        &mut self,
        mut start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        found: *mut *mut SkOpSpanBase,
    ) -> bool {
        let mut step = (*start).step(end);
        let mut min = (*start).starter(end);
        self.markDone(min);
        let mut last = null_mut();
        let mut other = self as *mut Self;
        let (mut priorDone, mut lastDone) = (null_mut(), null_mut());
        let mut safety = 1000;
        loop {
            other = (*other).nextChase(&mut start, &mut step, &mut min, &mut last);
            if other.is_null() {
                break;
            }
            safety -= 1;
            if safety == 0 {
                return false;
            }
            if (*other).done() {
                break;
            }
            if lastDone == min || priorDone == min {
                if !found.is_null() {
                    *found = null_mut();
                }
                return true;
            }
            (*other).markDone(min);
            priorDone = lastDone;
            lastDone = min;
        }
        if !found.is_null() {
            *found = last;
        }
        true
    }
    pub unsafe fn markAndChaseWinding(
        &mut self,
        mut start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        winding: i32,
        lastPtr: *mut *mut SkOpSpanBase,
    ) -> bool {
        let mut span = (*start).starter(end);
        let mut step = (*start).step(end);
        let success = self.markWinding(span, winding);
        let mut last = null_mut();
        let mut other = self as *mut Self;
        let mut safety = 1000;
        loop {
            other = (*other).nextChase(&mut start, &mut step, &mut span, &mut last);
            if other.is_null() {
                break;
            }
            safety -= 1;
            if safety == 0 {
                return false;
            }
            if (*span).windSum() != i32::MIN {
                break;
            }
            (*other).markWinding(span, winding);
        }
        if !lastPtr.is_null() {
            *lastPtr = last;
        }
        success
    }
    pub unsafe fn markAndChaseWindingOp(
        &mut self,
        mut start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        winding: i32,
        opp: i32,
        lastPtr: *mut *mut SkOpSpanBase,
    ) -> bool {
        let mut span = (*start).starter(end);
        let mut step = (*start).step(end);
        let success = self.markWindingOp(span, winding, opp);
        let mut last = null_mut();
        let mut other = self as *mut Self;
        let mut safety = 1000;
        loop {
            other = (*other).nextChase(&mut start, &mut step, &mut span, &mut last);
            if other.is_null() {
                break;
            }
            safety -= 1;
            if safety == 0 {
                return false;
            }
            if (*span).windSum() != i32::MIN {
                if self.operand() == (*other).operand() {
                    if (*span).windSum() != winding || (*span).oppSum() != opp {
                        (*self.globalState()).setWindingFailed();
                        return true;
                    }
                } else if (*span).windSum() != opp || (*span).oppSum() != winding {
                    return false;
                }

                break;
            }
            if self.operand() == (*other).operand() {
                (*other).markWindingOp(span, winding, opp);
            } else {
                (*other).markWindingOp(span, opp, winding);
            }
        }
        if !lastPtr.is_null() {
            *lastPtr = last;
        }
        success
    }
    pub unsafe fn markAngle(
        &mut self,
        mut max: i32,
        sum: i32,
        angle: *const SkOpAngle,
        result: *mut *mut SkOpSpanBase,
    ) -> bool {
        if Self::UseInnerWinding(max, sum) {
            max = sum;
        }
        self.markAndChaseWinding((*angle).start(), (*angle).end(), max, result)
    }
    pub unsafe fn markAngleOp(
        &mut self,
        mut max: i32,
        sum: i32,
        mut oppMax: i32,
        oppSum: i32,
        angle: *const SkOpAngle,
        result: *mut *mut SkOpSpanBase,
    ) -> bool {
        if Self::UseInnerWinding(max, sum) {
            max = sum;
        }
        if oppMax != oppSum && Self::UseInnerWinding(oppMax, oppSum) {
            oppMax = oppSum;
        }
        self.markAndChaseWindingOp((*angle).start(), (*angle).end(), max, oppMax, result)
    }
    pub unsafe fn markDone(&mut self, span: *mut SkOpSpan) {
        if (*span).done() {
            return;
        }
        (*span).setDone(true);
        self.fDoneCount += 1;
    }
    pub unsafe fn markWinding(&mut self, span: *mut SkOpSpan, winding: i32) -> bool {
        if (*span).done() {
            return false;
        }
        (*span).setWindSum(winding);
        true
    }
    pub unsafe fn markWindingOp(&mut self, span: *mut SkOpSpan, winding: i32, opp: i32) -> bool {
        if (*span).done() {
            return false;
        }
        (*span).setWindSum(winding);
        (*span).setOppSum(opp);
        true
    }
    pub unsafe fn match_(
        &self,
        base: *const SkOpPtT,
        parent: *const Self,
        t: f64,
        point: &SkPoint,
    ) -> bool {
        if core::ptr::eq(self, parent) && precisely_equal((*base).fT, t) {
            return true;
        }
        if !SkDPoint::ApproximatelyEqual(point, &(*base).fPt) {
            return false;
        }
        !core::ptr::eq(self, parent) || !self.ptsDisjoint((*base).fT, &(*base).fPt, t, point)
    }
    pub unsafe fn nextChase(
        &self,
        startPtr: &mut *mut SkOpSpanBase,
        stepPtr: &mut i32,
        minPtr: *mut *mut SkOpSpan,
        last: *mut *mut SkOpSpanBase,
    ) -> *mut Self {
        let orig = *startPtr;
        let step = *stepPtr;
        let mut end = if step > 0 {
            (*(*orig).upCast()).next()
        } else {
            (*orig).prev().cast()
        };

        let angle = if step > 0 {
            (*end).fromAngle()
        } else {
            (*(*end).upCast()).toAngle()
        };
        let (found, otherEnd, other);
        if angle.is_null() {
            if (*end).t() != 0.0 && (*end).t() != 1.0 {
                return null_mut();
            }
            let p = (*(*end).ptT()).next();
            other = (*p).segment();
            found = (*p).span();
            otherEnd = if step > 0 {
                if (*found).upCastable().is_null() {
                    null_mut()
                } else {
                    (*(*found).upCast()).next()
                }
            } else {
                (*found).prev().cast()
            };
        } else {
            if (*angle).loopCount() > 2 {
                return set_last(last, end);
            }
            let next = (*angle).next();
            if next.is_null() {
                return null_mut();
            }
            other = (*next).segment();
            found = (*next).start();
            end = found;
            otherEnd = (*next).end();
        }
        if otherEnd.is_null() {
            return null_mut();
        }
        let foundStep = (*found).step(otherEnd);
        if *stepPtr != foundStep {
            return set_last(last, end);
        }
        let origMin = if step < 0 {
            (*orig).prev()
        } else {
            (*orig).upCast()
        };
        let foundMin = (*found).starter(otherEnd);
        if (*foundMin).windValue() != (*origMin).windValue()
            || (*foundMin).oppValue() != (*origMin).oppValue()
        {
            return set_last(last, end);
        }
        *startPtr = found;
        *stepPtr = foundStep;
        if !minPtr.is_null() {
            *minPtr = foundMin;
        }
        other
    }
    pub unsafe fn ClearVisited(mut span: *mut SkOpSpanBase) {
        loop {
            let stop = (*span).ptT();
            let mut p = (*stop).next();
            while p != stop {
                (*(*p).segment()).resetVisited();
                p = (*p).next();
            }
            if (*span).final_() {
                break;
            }
            span = (*(*span).upCast()).next();
            if span.is_null() {
                break;
            }
        }
    }
    pub unsafe fn setUpWindingsUnary(
        &self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        mi: &mut i32,
        max: &mut i32,
        sum: &mut i32,
    ) {
        let delta = Self::SpanSign(start, end);
        *max = *mi;
        *mi -= delta;
        *sum = *mi;
    }
    pub unsafe fn setUpWindings(
        &self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
        mi: &mut i32,
        su: &mut i32,
        max: &mut i32,
        sum: &mut i32,
        oppMax: &mut i32,
        oppSum: &mut i32,
    ) {
        let delta = Self::SpanSign(start, end);
        let oppDelta = Self::OppSign(start, end);
        if self.operand() {
            *max = *su;
            *su -= delta;
            *sum = *su;
            *oppMax = *mi;
            *mi -= oppDelta;
            *oppSum = *mi;
        } else {
            *max = *mi;
            *mi -= delta;
            *sum = *mi;
            *oppMax = *su;
            *su -= oppDelta;
            *oppSum = *su;
        }
    }
    pub unsafe fn updateOppWinding(
        &self,
        start: *const SkOpSpanBase,
        end: *const SkOpSpanBase,
    ) -> i32 {
        let lesser = (*start).starter(end);
        let mut winding = (*lesser).oppSum();
        let delta = Self::OppSign(start, end);
        if delta != 0
            && Self::UseInnerWinding(winding.wrapping_sub(delta), winding)
            && winding != i32::MAX
        {
            winding -= delta;
        }
        winding
    }
    pub unsafe fn updateOppWindingAngle(&self, angle: *const SkOpAngle) -> i32 {
        self.updateOppWinding((*angle).end(), (*angle).start())
    }
    pub unsafe fn updateOppWindingReverse(&self, angle: *const SkOpAngle) -> i32 {
        self.updateOppWinding((*angle).start(), (*angle).end())
    }
    pub unsafe fn updateWinding(
        &mut self,
        start: *mut SkOpSpanBase,
        end: *mut SkOpSpanBase,
    ) -> i32 {
        let lesser = (*start).starter(end);
        let mut winding = (*lesser).windSum();
        if winding == i32::MIN {
            winding = (*lesser).computeWindSum();
        }
        if winding == i32::MIN {
            return winding;
        }
        let delta = Self::SpanSign(start, end);
        if winding != 0
            && Self::UseInnerWinding(winding.wrapping_sub(delta), winding)
            && winding != i32::MAX
        {
            winding -= delta;
        }
        winding
    }
    pub unsafe fn updateWindingAngle(&mut self, angle: *const SkOpAngle) -> i32 {
        self.updateWinding((*angle).end(), (*angle).start())
    }
    pub unsafe fn updateWindingReverse(&mut self, angle: *const SkOpAngle) -> i32 {
        self.updateWinding((*angle).start(), (*angle).end())
    }
    pub fn UseInnerWinding(outer: i32, inner: i32) -> bool {
        let absOut = outer.wrapping_abs();
        let absIn = inner.wrapping_abs();
        if absOut == absIn {
            outer < 0
        } else {
            absOut < absIn
        }
    }
    pub unsafe fn windSumAngle(&self, angle: *const SkOpAngle) -> i32 {
        (*(*(*angle).start()).starter((*angle).end())).windSum()
    }
    pub unsafe fn undoneSpan(&self) -> *mut SkOpSpan {
        let mut span = self.head();
        loop {
            let next = (*span).next();
            if !(*span).done() {
                return span;
            }
            if (*next).final_() {
                return null_mut();
            }
            span = (*next).upCast();
        }
    }
}
unsafe fn set_last(last: *mut *mut SkOpSpanBase, end: *mut SkOpSpanBase) -> *mut SkOpSegment {
    if !last.is_null() {
        *last = end;
    }
    null_mut()
}

impl SkOpSegment {
    pub unsafe fn findNextOp(
        &mut self,
        chase: &mut Vec<*mut SkOpSpanBase>,
        nextStart: &mut *mut SkOpSpanBase,
        nextEnd: &mut *mut SkOpSpanBase,
        unsortable: &mut bool,
        simple: &mut bool,
        op: SkPathOp,
        xorMiMask: i32,
        xorSuMask: i32,
    ) -> *mut Self {
        let start = *nextStart;
        let end = *nextEnd;

        let mut step = (*start).step(end);
        let other = self.isSimple(nextStart, &mut step);
        *simple = !other.is_null();
        if *simple {
            let span = (*start).starter(end);
            if (*span).done() {
                return null_mut();
            }
            self.markDone(span);
            *nextEnd = if step > 0 {
                (*(**nextStart).upCast()).next()
            } else {
                (**nextStart).prev().cast()
            };
            return other;
        }
        let near = if step > 0 {
            (*(**nextStart).upCast()).next()
        } else {
            (**nextStart).prev().cast()
        };

        if self.computeSum(start, near, IncludeType::kBinaryOpp) == i32::MIN {
            *unsortable = true;
            self.markDone((*start).starter(end));
            return null_mut();
        }
        let angle = self.spanToAngle(end, start);
        if (*angle).unorderable() {
            *unsortable = true;
            self.markDone((*start).starter(end));
            return null_mut();
        }
        let mut mi = self.updateWinding(end, start);
        if mi == i32::MIN {
            *unsortable = true;
            self.markDone((*start).starter(end));
            return null_mut();
        }
        let mut su = self.updateOppWinding(end, start);
        if self.operand() {
            core::mem::swap(&mut mi, &mut su);
        }
        let mut next = (*angle).next();
        let mut found: *mut SkOpAngle = null_mut();
        let mut foundDone = false;
        let mut count = 0;
        loop {
            let segment = (*next).segment();
            let active = (*segment).activeOpSums(
                xorMiMask,
                xorSuMask,
                (*next).start(),
                (*next).end(),
                op,
                &mut mi,
                &mut su,
            );
            if active {
                count += 1;
                if found.is_null() || (foundDone && (count & 1) != 0) {
                    found = next;
                    foundDone = (*segment).doneAngle(next);
                }
            }
            if !(*segment).done() {
                if !active {
                    (*segment).markAndChaseDone((*next).start(), (*next).end(), null_mut());
                }
                let last = (*next).lastMarked();
                if !last.is_null() {
                    chase.push(last);
                }
            }
            next = (*next).next();
            if next == angle {
                break;
            }
        }
        (*(*start).segment()).markDone((*start).starter(end));
        if found.is_null() {
            return null_mut();
        }
        *nextStart = (*found).start();
        *nextEnd = (*found).end();
        (*found).segment()
    }
    pub unsafe fn findNextWinding(
        &mut self,
        chase: &mut Vec<*mut SkOpSpanBase>,
        nextStart: &mut *mut SkOpSpanBase,
        nextEnd: &mut *mut SkOpSpanBase,
        unsortable: &mut bool,
    ) -> *mut Self {
        let start = *nextStart;
        let end = *nextEnd;

        let mut step = (*start).step(end);
        let other = self.isSimple(nextStart, &mut step);
        if !other.is_null() {
            let span = (*start).starter(end);
            if (*span).done() {
                return null_mut();
            }
            self.markDone(span);
            *nextEnd = if step > 0 {
                (*(**nextStart).upCast()).next()
            } else {
                (**nextStart).prev().cast()
            };
            return other;
        }
        let near = if step > 0 {
            (*(**nextStart).upCast()).next()
        } else {
            (**nextStart).prev().cast()
        };

        if self.computeSum(start, near, IncludeType::kUnaryWinding) == i32::MIN {
            *unsortable = true;
            self.markDone((*start).starter(end));
            return null_mut();
        }
        let angle = self.spanToAngle(end, start);
        if (*angle).unorderable() {
            *unsortable = true;
            self.markDone((*start).starter(end));
            return null_mut();
        }
        let mut sum = self.updateWinding(end, start);
        let mut next = (*angle).next();
        let mut found: *mut SkOpAngle = null_mut();
        let mut foundDone = false;
        let mut count = 0;
        loop {
            let segment = (*next).segment();
            let active = (*segment).activeWindingSum((*next).start(), (*next).end(), &mut sum);
            if active {
                count += 1;
                if found.is_null() || (foundDone && (count & 1) != 0) {
                    found = next;
                    foundDone = (*segment).doneAngle(next);
                }
            }
            if !(*segment).done() {
                if !active {
                    (*segment).markAndChaseDone((*next).start(), (*next).end(), null_mut());
                }
                let last = (*next).lastMarked();
                if !last.is_null() {
                    chase.push(last);
                }
            }
            next = (*next).next();
            if next == angle {
                break;
            }
        }
        (*(*start).segment()).markDone((*start).starter(end));
        if found.is_null() {
            return null_mut();
        }
        *nextStart = (*found).start();
        *nextEnd = (*found).end();
        (*found).segment()
    }
    pub unsafe fn findNextXor(
        &mut self,
        nextStart: &mut *mut SkOpSpanBase,
        nextEnd: &mut *mut SkOpSpanBase,
        unsortable: &mut bool,
    ) -> *mut Self {
        let start = *nextStart;
        let end = *nextEnd;

        let mut step = (*start).step(end);
        let other = self.isSimple(nextStart, &mut step);
        if !other.is_null() {
            let span = (*start).starter(end);
            if (*span).done() {
                return null_mut();
            }
            self.markDone(span);
            *nextEnd = if step > 0 {
                (*(**nextStart).upCast()).next()
            } else {
                (**nextStart).prev().cast()
            };
            return other;
        }
        let angle = self.spanToAngle(end, start);
        if angle.is_null() || (*angle).unorderable() {
            *unsortable = true;
            self.markDone((*start).starter(end));
            return null_mut();
        }
        let mut next = (*angle).next();
        let mut found: *mut SkOpAngle = null_mut();
        let mut foundDone = false;
        let mut count = 0;
        loop {
            if next.is_null() {
                return null_mut();
            }
            let segment = (*next).segment();
            count += 1;
            if found.is_null() || (foundDone && (count & 1) != 0) {
                found = next;
                foundDone = (*segment).doneAngle(next);
                if !foundDone {
                    break;
                }
            }
            next = (*next).next();
            if next == angle {
                break;
            }
        }
        (*(*start).segment()).markDone((*start).starter(end));
        if found.is_null() {
            return null_mut();
        }
        *nextStart = (*found).start();
        *nextEnd = (*found).end();
        (*found).segment()
    }
    pub unsafe fn existing(&self, t: f64, opp: *const Self) -> *mut SkOpPtT {
        let mut test = self.head().cast::<SkOpSpanBase>();
        let point = self.ptAtT(t);
        let found;
        loop {
            let p = (*test).ptT();
            if (*p).fT == t {
                found = p;
                break;
            }
            if !self.match_(p, self, t, &point) {
                if t < (*p).fT {
                    return null_mut();
                }
            } else {
                if opp.is_null() {
                    return p;
                }
                let mut walk = (*p).next();
                let mut matched = false;
                while walk != p {
                    if core::ptr::eq((*walk).segment(), self)
                        && (*walk).fT == t
                        && (*walk).fPt == point
                    {
                        matched = true;
                        break;
                    }
                    walk = (*walk).next();
                }
                if !matched {
                    return null_mut();
                }
                found = p;
                break;
            }
            test = (*(*test).upCast()).next();
            if test.is_null() {
                return null_mut();
            }
        }
        if !opp.is_null() && (*test).containsSegment(opp).is_null() {
            null_mut()
        } else {
            found
        }
    }
    pub unsafe fn missingCoincidence(&mut self) -> bool {
        if self.done() {
            return false;
        }
        let mut prior: *mut SkOpSpan = null_mut();
        let mut span = self.head().cast::<SkOpSpanBase>();
        let mut result = false;
        let mut safety = 1000;
        loop {
            let stop = (*span).ptT();
            let mut p = (*stop).next();

            while p != stop {
                safety -= 1;
                if safety == 0 {
                    return false;
                }
                if !(*p).deleted() {
                    let opp = (*(*p).span()).segment();
                    if !(*opp).done()
                        && (*opp).visited()
                        && span != self.head().cast()
                        && !core::ptr::eq((*p).segment(), self)
                    {
                        let up = (*span).upCastable();
                        if (up.is_null() || !(*up).containsCoincidenceSegment(opp))
                            && !(*span).containsCoinEndSegment(opp)
                        {
                            let mut priorP = null_mut();
                            let mut priorOpp: *mut Self = null_mut();
                            let mut priorTest = (*span).prev();
                            while priorOpp.is_null() && !priorTest.is_null() {
                                let priorStop = (*priorTest).ptT();
                                priorP = (*priorStop).next();
                                while priorP != priorStop {
                                    if !(*priorP).deleted() && (*(*priorP).span()).segment() == opp
                                    {
                                        prior = priorTest;
                                        priorOpp = opp;
                                        break;
                                    }
                                    priorP = (*priorP).next();
                                }
                                priorTest = (*priorTest).prev();
                            }
                            if !priorOpp.is_null() && priorP != p {
                                let mut oppStart = (*prior).ptT();
                                let mut oppEnd = (*span).ptT();
                                let swapped = (*priorP).fT > (*p).fT;
                                if swapped {
                                    core::mem::swap(&mut priorP, &mut p);
                                    core::mem::swap(&mut oppStart, &mut oppEnd);
                                }
                                let coins = (*self.globalState()).coincidence();
                                let rootPrior = (*(*priorP).span()).ptT();
                                let root = (*(*p).span()).ptT();
                                let rootOppStart = (*(*oppStart).span()).ptT();
                                let rootOppEnd = (*(*oppEnd).span()).ptT();
                                if !(*coins).contains(rootPrior, root, rootOppStart, rootOppEnd)
                                    && self.testForCoincidence(
                                        rootPrior,
                                        root,
                                        prior.cast(),
                                        span,
                                        opp,
                                    )
                                {
                                    if !(*coins).extend(rootPrior, root, rootOppStart, rootOppEnd) {
                                        (*coins).add(rootPrior, root, rootOppStart, rootOppEnd);
                                    }
                                    result = true;
                                }
                                if swapped {
                                    core::mem::swap(&mut priorP, &mut p);
                                }
                            }
                        }
                    }
                }
                p = (*p).next();
            }
            span = if (*span).final_() {
                null_mut()
            } else {
                (*(*span).upCast()).next()
            };
            if span.is_null() {
                break;
            }
        }
        Self::ClearVisited(self.head().cast());
        result
    }
    pub unsafe fn moveMultiples(&mut self) -> bool {
        let mut test = self.head().cast::<SkOpSpanBase>();
        'next_span: loop {
            let count = (*test).spanAddsCount();
            if count > 1 {
                let start = (*test).ptT();
                let mut p = start;
                let mut safety = 1000;
                loop {
                    safety -= 1;
                    if safety == 0 {
                        return false;
                    }
                    let oppSpan = (*p).span();
                    if (*oppSpan).spanAddsCount() != count && !(*oppSpan).deleted() {
                        let oppSegment = (*oppSpan).segment();
                        if !core::ptr::eq(oppSegment, self) {
                            let mut prev = oppSpan;
                            let mut first = oppSpan;
                            loop {
                                prev = (*prev).prev().cast();
                                if prev.is_null() {
                                    break;
                                }
                                if !roughly_equal((*prev).t(), (*oppSpan).t()) {
                                    break;
                                }
                                if (*prev).spanAddsCount() != count && !(*prev).deleted() {
                                    first = prev;
                                }
                            }
                            let mut next = oppSpan;
                            let mut last = oppSpan;
                            loop {
                                next = if (*next).final_() {
                                    null_mut()
                                } else {
                                    (*(*next).upCast()).next()
                                };
                                if next.is_null() {
                                    break;
                                }
                                if !roughly_equal((*next).t(), (*oppSpan).t()) {
                                    break;
                                }
                                if (*next).spanAddsCount() != count && !(*next).deleted() {
                                    last = next;
                                }
                            }
                            if first != last {
                                let mut candidate = first;
                                loop {
                                    if candidate != oppSpan {
                                        let oppStart = (*candidate).ptT();
                                        let mut oppP = (*oppStart).next();
                                        while oppP != oppStart {
                                            let segment = (*oppP).segment();
                                            if core::ptr::eq(segment, self) {
                                                break;
                                            }
                                            let mut matched = start;
                                            let mut found = false;
                                            loop {
                                                if (*matched).segment() == segment {
                                                    found = true;
                                                    break;
                                                }
                                                matched = (*matched).next();
                                                if matched == start {
                                                    break;
                                                }
                                            }
                                            if !found {
                                                break;
                                            }
                                            (*candidate).mergeMatches(oppSpan);
                                            (*candidate).addOpp(oppSpan);
                                            test = if (*test).final_() {
                                                null_mut()
                                            } else {
                                                (*(*test).upCast()).next()
                                            };
                                            if test.is_null() {
                                                return true;
                                            }
                                            continue 'next_span;
                                        }
                                    }
                                    if candidate == last {
                                        break;
                                    }
                                    candidate = (*(*candidate).upCast()).next();
                                    if candidate.is_null() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    p = (*p).next();
                    if p == start {
                        break;
                    }
                }
            }
            test = if (*test).final_() {
                null_mut()
            } else {
                (*(*test).upCast()).next()
            };
            if test.is_null() {
                break;
            }
        }
        true
    }
    pub unsafe fn moveNearby(&mut self) -> bool {
        let mut span = self.head().cast::<SkOpSpanBase>();
        let mut escape = 9999;
        loop {
            let head = (*span).ptT();
            let mut p = (*head).next();
            while p != head {
                escape -= 1;
                if escape == 0 {
                    return false;
                }
                let test = (*p).span();
                if core::ptr::eq((*p).segment(), self)
                    && !(*p).deleted()
                    && test != span
                    && (*test).ptT() == p
                {
                    if (*test).final_() {
                        if span == self.head().cast() {
                            self.clearAll();
                            return true;
                        }
                        (*(*span).upCast()).release(p);
                    } else if !(*test).prev().is_null() {
                        (*(*test).upCast()).release(head);
                    }
                    break;
                }
                p = (*p).next();
            }
            span = (*(*span).upCast()).next();
            if (*span).final_() {
                break;
            }
        }
        span = self.head().cast();
        loop {
            let test = (*(*span).upCast()).next();
            let mut found = false;
            if !self.spansNearby(span, test, &mut found) {
                return false;
            }
            if found {
                if (*test).final_() {
                    if !(*span).prev().is_null() {
                        (*test).merge((*span).upCast());
                    } else {
                        self.clearAll();
                        return true;
                    }
                } else {
                    (*span).merge((*test).upCast());
                }
            }
            span = test;
            if (*span).final_() {
                break;
            }
        }
        true
    }
    pub unsafe fn sortAngles(&mut self) -> bool {
        let mut span = self.head().cast::<SkOpSpanBase>();
        loop {
            let from = (*span).fromAngle();
            let to = if (*span).final_() {
                null_mut()
            } else {
                (*(*span).upCast()).toAngle()
            };
            if !from.is_null() || !to.is_null() {
                let mut base = from;
                if !from.is_null() && !to.is_null() {
                    if !(*from).insert(to) {
                        return false;
                    }
                } else if from.is_null() {
                    base = to;
                }
                let stop = (*span).ptT();
                let mut p = stop;
                let mut safety = 1000;
                loop {
                    safety -= 1;
                    if safety == 0 {
                        return false;
                    }
                    let other = (*p).span();
                    if other != span {
                        let mut angle = (*other).fromAngle();
                        if !angle.is_null() && !(*angle).loopContains(base) {
                            (*base).insert(angle);
                        }
                        if !(*other).final_() {
                            angle = (*(*other).upCast()).toAngle();
                            if !angle.is_null() && !(*angle).loopContains(base) {
                                (*base).insert(angle);
                            }
                        }
                    }
                    p = (*p).next();
                    if p == stop {
                        break;
                    }
                }
                if (*base).loopCount() == 1 {
                    (*span).setFromAngle(null_mut());
                    if !to.is_null() {
                        (*(*span).upCast()).setToAngle(null_mut());
                    }
                }
            }
            if (*span).final_() {
                break;
            }
            span = (*(*span).upCast()).next();
            if span.is_null() {
                break;
            }
        }
        true
    }
}

impl SkOpSegment {
    pub unsafe fn dPtAtT(&self, t: f64) -> SkDPoint {
        super::SkPathOpsCurve::CurveDPointAtT(self.fVerb, self.fPts, self.fWeight, t)
    }
    pub unsafe fn dSlopeAtT(&self, t: f64) -> SkDVector {
        super::SkPathOpsCurve::CurveDSlopeAtT(self.fVerb, self.fPts, self.fWeight, t)
    }
    pub unsafe fn ptAtT(&self, t: f64) -> SkPoint {
        super::SkPathOpsCurve::CurvePointAtT(self.fVerb, self.fPts, self.fWeight, t)
    }
    pub unsafe fn isVerticalSpan(
        &self,
        start: *const SkOpSpanBase,
        end: *const SkOpSpanBase,
    ) -> bool {
        super::SkPathOpsCurve::CurveIsVertical(
            self.fVerb,
            self.fPts,
            self.fWeight,
            (*start).t(),
            (*end).t(),
        )
    }
    pub unsafe fn addLine(&mut self, pts: *mut SkPoint, parent: *mut SkOpContour) -> *mut Self {
        self.init(pts, 1.0, parent, Verb::kLine_Verb);
        self.fBounds.setBounds(core::slice::from_raw_parts(pts, 2));
        self
    }
    pub unsafe fn addQuad(&mut self, pts: *mut SkPoint, parent: *mut SkOpContour) -> *mut Self {
        self.init(pts, 1.0, parent, Verb::kQuad_Verb);
        let mut curve = SkDCurve::default();
        curve.fQuad.set(&*pts.cast::<[SkPoint; 3]>());
        curve.setQuadBounds(
            &*pts.cast::<[SkPoint; 3]>(),
            1.0,
            0.0,
            1.0,
            &mut self.fBounds,
        );
        self
    }
    pub unsafe fn addConic(
        &mut self,
        pts: *mut SkPoint,
        weight: f32,
        parent: *mut SkOpContour,
    ) -> *mut Self {
        self.init(pts, weight, parent, Verb::kConic_Verb);
        let mut curve = SkDCurve::default();
        curve.fConic.set(&*pts.cast::<[SkPoint; 3]>(), weight);
        curve.setConicBounds(
            &*pts.cast::<[SkPoint; 3]>(),
            weight,
            0.0,
            1.0,
            &mut self.fBounds,
        );
        self
    }
    pub unsafe fn addCubic(&mut self, pts: *mut SkPoint, parent: *mut SkOpContour) -> *mut Self {
        self.init(pts, 1.0, parent, Verb::kCubic_Verb);
        let mut curve = SkDCurve::default();
        curve.fCubic.set(&*pts.cast::<[SkPoint; 4]>());
        curve.setCubicBounds(
            &*pts.cast::<[SkPoint; 4]>(),
            1.0,
            0.0,
            1.0,
            &mut self.fBounds,
        );
        self
    }
    pub unsafe fn addCurveTo(
        &self,
        start: *const SkOpSpanBase,
        end: *const SkOpSpanBase,
        path: *mut SkPathWriter,
    ) -> bool {
        let span = (*start).starter(end);
        if (*span).alreadyAdded() {
            return false;
        }
        (*span).markAdded();
        let mut part = SkDCurveSweep::default();
        (*(*start).segment()).subDivide(start, end, &mut part.fCurve);
        part.setCurveHullSweep(self.fVerb);
        let verb = if part.isCurve() {
            self.fVerb
        } else {
            Verb::kLine_Verb
        };
        (*path).deferredMove((*start).ptT());
        match verb {
            Verb::kLine_Verb => {
                if !(*path).deferredLine((*end).ptT()) {
                    return false;
                }
            }
            Verb::kQuad_Verb => (*path).quadTo(&part.fCurve.fQuad[1].asSkPoint(), (*end).ptT()),
            Verb::kConic_Verb => (*path).conicTo(
                &part.fCurve.fConic[1].asSkPoint(),
                (*end).ptT(),
                part.fCurve.fConic.fWeight,
            ),
            Verb::kCubic_Verb => (*path).cubicTo(
                &part.fCurve.fCubic[1].asSkPoint(),
                &part.fCurve.fCubic[2].asSkPoint(),
                (*end).ptT(),
            ),
            _ => (),
        }
        true
    }
    pub unsafe fn isClose(&self, t: f64, opp: *const Self) -> bool {
        let point = self.dPtAtT(t);
        let slope = self.dSlopeAtT(t);
        let ray = super::SkPathOpsLine::SkDLine {
            fPts: [
                point,
                SkDPoint {
                    fX: point.fX + slope.fY,
                    fY: point.fY - slope.fX,
                },
            ],
        };
        let mut intersections = super::SkIntersections::SkIntersections::default();
        super::SkPathOpsCurve::CurveIntersectRay(
            (*opp).verb(),
            (*opp).pts(),
            (*opp).weight(),
            &ray,
            &mut intersections,
        );
        for i in 0..intersections.used() {
            if point.roughlyEqual(intersections.pt(i)) {
                return true;
            }
        }
        false
    }
    pub unsafe fn ptsDisjoint(&self, t1: f64, point1: &SkPoint, t2: f64, point2: &SkPoint) -> bool {
        if self.fVerb == Verb::kLine_Verb {
            return false;
        }
        let point = self.ptAtT((t1 + t2) / 2.0);
        let dist = (crate::src::core::SkPointPriv::SkPointPriv::DistanceToSqd(*point1, *point2)
            * 2.0) as f64;
        let dist = if dist < FLT_EPSILON * 2.0 {
            FLT_EPSILON * 2.0
        } else {
            dist
        };
        crate::src::core::SkPointPriv::SkPointPriv::DistanceToSqd(point, *point1) as f64 > dist
            || crate::src::core::SkPointPriv::SkPointPriv::DistanceToSqd(point, *point2) as f64
                > dist
    }
    pub unsafe fn ptsDisjointPtT(&self, span: &SkOpPtT, test: &SkOpPtT) -> bool {
        self.ptsDisjoint(span.fT, &span.fPt, test.fT, &test.fPt)
    }
    pub unsafe fn ptsDisjointPt(&self, span: &SkOpPtT, t: f64, point: &SkPoint) -> bool {
        self.ptsDisjoint(span.fT, &span.fPt, t, point)
    }
    pub unsafe fn spansNearby(
        &self,
        refSpan: *const SkOpSpanBase,
        checkSpan: *const SkOpSpanBase,
        found: &mut bool,
    ) -> bool {
        let refHead = (*refSpan).ptT();
        let checkHead = (*checkSpan).ptT();
        if !SkDPoint::WayRoughlyEqual(&(*refHead).fPt, &(*checkHead).fPt) {
            *found = false;
            return true;
        }
        let mut best = f32::MAX;
        let mut refBest: *mut SkOpPtT = null_mut();
        let mut checkBest: *mut SkOpPtT = null_mut();
        let mut reference = refHead;
        'distance: loop {
            'nextRef: {
                if !(*reference).deleted() {
                    while (*reference).ptAlreadySeen(refHead) {
                        reference = (*reference).next();
                        if reference == refHead {
                            break 'distance;
                        }
                    }
                    let mut check = checkHead;
                    let refSeg = (*reference).segment();
                    let mut escape = 100;
                    loop {
                        if !(*check).deleted() {
                            while (*check).ptAlreadySeen(checkHead) {
                                check = (*check).next();
                                if check == checkHead {
                                    break 'nextRef;
                                }
                            }
                            let dist = crate::src::core::SkPointPriv::SkPointPriv::DistanceToSqd(
                                (*reference).fPt,
                                (*check).fPt,
                            );
                            if best > dist
                                && (refSeg != (*check).segment()
                                    || !(*refSeg).ptsDisjointPtT(&*reference, &*check))
                            {
                                best = dist;
                                refBest = reference;
                                checkBest = check;
                            }
                            escape -= 1;
                            if escape <= 0 {
                                return false;
                            }
                        }
                        check = (*check).next();
                        if check == checkHead {
                            break;
                        }
                    }
                }
            }
            reference = (*reference).next();
            if reference == refHead {
                break;
            }
        }
        *found = !checkBest.is_null()
            && (*(*refBest).segment()).match_(
                refBest,
                (*checkBest).segment(),
                (*checkBest).fT,
                &(*checkBest).fPt,
            );
        true
    }
    pub unsafe fn subDivide(
        &self,
        start: *const SkOpSpanBase,
        end: *const SkOpSpanBase,
        edge: &mut SkDCurve,
    ) -> bool {
        let startPt = &*(*start).ptT();
        let endPt = &*(*end).ptT();
        edge.fCubic[0].set(&startPt.fPt);
        let points = SkPathOpsVerbToPoints(self.fVerb) as usize;
        edge.fCubic[points].set(&endPt.fPt);
        if self.fVerb == Verb::kLine_Verb {
            return false;
        }
        let st = startPt.fT;
        let et = endPt.fT;
        if (st == 0.0 || et == 0.0) && (st == 1.0 || et == 1.0) {
            if self.fVerb == Verb::kQuad_Verb {
                edge.fLine[1].set(&*self.fPts.add(1));
                return false;
            }
            if self.fVerb == Verb::kConic_Verb {
                edge.fConic[1].set(&*self.fPts.add(1));
                edge.fConic.fWeight = self.fWeight;
                return false;
            }

            if st == 0.0 {
                edge.fCubic[1].set(&*self.fPts.add(1));
                edge.fCubic[2].set(&*self.fPts.add(2));
            } else {
                edge.fCubic[1].set(&*self.fPts.add(2));
                edge.fCubic[2].set(&*self.fPts.add(1));
            }
            return false;
        }
        if self.fVerb == Verb::kQuad_Verb {
            let a = edge.fQuad[0];
            let c = edge.fQuad[2];
            edge.fQuad[1] = super::SkPathOpsQuad::SkDQuad::SubDivideControl(
                &*self.fPts.cast::<[SkPoint; 3]>(),
                &a,
                &c,
                st,
                et,
            );
        } else if self.fVerb == Verb::kConic_Verb {
            let a = edge.fQuad[0];
            let c = edge.fQuad[2];
            let mut weight = 0.0;
            edge.fConic[1] = super::SkPathOpsConic::SkDConic::SubDivideControl(
                &*self.fPts.cast::<[SkPoint; 3]>(),
                self.fWeight,
                &a,
                &c,
                st,
                et,
                &mut weight,
            );
            edge.fConic.fWeight = weight;
        } else {
            let a = edge.fCubic[0];
            let d = edge.fCubic[3];
            let mut control = [SkDPoint::default(); 2];
            super::SkPathOpsCubic::SkDCubic::SubDivideControl(
                &*self.fPts.cast::<[SkPoint; 4]>(),
                &a,
                &d,
                st,
                et,
                &mut control,
            );
            edge.fCubic[1] = control[0];
            edge.fCubic[2] = control[1];
        }
        true
    }
    pub unsafe fn testForCoincidence(
        &self,
        priorPt: *const SkOpPtT,
        pt: *const SkOpPtT,
        prior: *const SkOpSpanBase,
        span: *const SkOpSpanBase,
        opp: *const Self,
    ) -> bool {
        let mid = self.ptAtT(((*prior).t() + (*span).t()) / 2.0);
        let mut coincident = true;
        if !SkDPoint::ApproximatelyEqual((*priorPt).fPt, mid)
            && !SkDPoint::ApproximatelyEqual((*pt).fPt, mid)
        {
            if (*priorPt).span() == (*pt).span() {
                return false;
            }
            coincident = false;
            let mut intersections = super::SkIntersections::SkIntersections::default();
            let mut part = SkDCurve::default();
            self.subDivide(prior, span, &mut part);
            let slope = super::SkPathOpsCurve::CurveDDSlopeAtT(self.fVerb, &part, 0.5);
            let point = super::SkPathOpsCurve::CurveDDPointAtT(self.fVerb, &part, 0.5);
            let ray = super::SkPathOpsLine::SkDLine {
                fPts: [
                    SkDPoint {
                        fX: mid.fX as f64,
                        fY: mid.fY as f64,
                    },
                    SkDPoint {
                        fX: point.fX + slope.fY,
                        fY: point.fY - slope.fX,
                    },
                ],
            };
            let mut oppPart = SkDCurve::default();
            (*opp).subDivide((*priorPt).span(), (*pt).span(), &mut oppPart);
            super::SkPathOpsCurve::CurveDIntersectRay(
                (*opp).verb(),
                &oppPart,
                &ray,
                &mut intersections,
            );
            for i in 0..intersections.used() {
                if !between(0.0, intersections[0][i], 1.0) {
                    continue;
                }
                if intersections.pt(i).approximatelyDEqual(mid) {
                    coincident = true;
                }
            }
        }
        coincident
    }
}
