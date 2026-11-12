// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkOpSpan.h and SkOpSpan.cpp, release topology.
#![allow(non_snake_case)]
use super::{
    SkOpAngle::SkOpAngle,
    SkOpContour::SkOpContour,
    SkOpSegment::SkOpSegment,
    SkPathOpsTypes::{between, zero_or_one, SkOpGlobalState},
};
use crate::src::core::SkPoint::SkPoint;
use core::{
    ops::{Deref, DerefMut},
    ptr::null_mut,
};

#[repr(C)]
pub struct SkOpPtT {
    pub fT: f64,
    pub fPt: SkPoint,
    pub(super) fSpan: *mut SkOpSpanBase,
    pub(super) fNext: *mut SkOpPtT,
    pub(super) fDeleted: bool,
    pub(super) fDuplicatePt: bool,
    pub(super) fCoincident: bool,
}
impl Default for SkOpPtT {
    fn default() -> Self {
        Self {
            fT: 0.0,
            fPt: SkPoint::default(),
            fSpan: null_mut(),
            fNext: null_mut(),
            fDeleted: false,
            fDuplicatePt: false,
            fCoincident: false,
        }
    }
}
impl SkOpPtT {
    pub const kIsAlias: i32 = 1;
    pub const kIsDuplicate: i32 = 1;
    pub unsafe fn alias(&self) -> bool {
        (*self.fSpan).ptT() != self as *const Self as *mut Self
    }
    pub unsafe fn active(&self) -> *mut Self {
        let stop = self as *const Self as *mut Self;
        if !self.fDeleted {
            return stop;
        }
        let mut p = self.fNext;
        while p != stop {
            if (*p).fSpan == self.fSpan && !(*p).fDeleted {
                return p;
            }
            p = (*p).fNext;
        }
        null_mut()
    }
    pub unsafe fn addOpp(&mut self, opp: *mut Self, oppPrev: *mut Self) {
        let oldNext = self.fNext;

        self.fNext = opp;

        (*oppPrev).fNext = oldNext;
    }
    pub fn coincident(&self) -> bool {
        self.fCoincident
    }
    pub fn deleted(&self) -> bool {
        self.fDeleted
    }
    pub fn duplicate(&self) -> bool {
        self.fDuplicatePt
    }
    pub fn next(&self) -> *mut Self {
        self.fNext
    }
    pub fn span(&self) -> *mut SkOpSpanBase {
        self.fSpan
    }
    pub unsafe fn contains(&self, check: *const Self) -> bool {
        let stop = self as *const Self;

        let mut p = self.fNext;
        while p.cast_const() != stop {
            if p.cast_const() == check {
                return true;
            }
            p = (*p).fNext;
        }
        false
    }
    pub unsafe fn containsSegmentPoint(&self, segment: *const SkOpSegment, pt: &SkPoint) -> bool {
        let stop = self as *const Self;
        let mut p = self.fNext;
        while p.cast_const() != stop {
            if (*p).fPt == *pt && (*p).segment().cast_const() == segment {
                return true;
            }
            p = (*p).fNext;
        }
        false
    }
    pub unsafe fn containsSegmentT(&self, segment: *const SkOpSegment, t: f64) -> bool {
        let stop = self as *const Self;
        let mut p = self.fNext;
        while p.cast_const() != stop {
            if (*p).fT == t && (*p).segment().cast_const() == segment {
                return true;
            }
            p = (*p).fNext;
        }
        false
    }
    pub unsafe fn containsSegment(&self, segment: *const SkOpSegment) -> *mut Self {
        let stop = self as *const Self;
        let mut p = self.fNext;
        while p.cast_const() != stop {
            if (*p).segment().cast_const() == segment && !(*p).deleted() {
                return p;
            }
            p = (*p).fNext;
        }
        null_mut()
    }
    pub unsafe fn contour(&self) -> *mut SkOpContour {
        (*self.segment()).contour()
    }
    pub unsafe fn globalState(&self) -> *mut SkOpGlobalState {
        (*self.contour()).globalState()
    }
    pub unsafe fn segment(&self) -> *mut SkOpSegment {
        (*self.fSpan).segment()
    }
    pub unsafe fn find(&self, segment: *const SkOpSegment) -> *mut Self {
        let stop = self as *const Self as *mut Self;
        let mut p = stop;
        loop {
            if (*p).segment().cast_const() == segment && !(*p).deleted() {
                return p;
            }
            p = (*p).fNext;
            if p == stop {
                return null_mut();
            }
        }
    }
    pub unsafe fn init(&mut self, span: *mut SkOpSpanBase, t: f64, pt: &SkPoint, duplicate: bool) {
        self.fT = t;
        self.fPt = *pt;
        self.fSpan = span;
        self.fNext = self;
        self.fDuplicatePt = duplicate;
        self.fDeleted = false;
        self.fCoincident = false;
    }
    pub unsafe fn insert(&mut self, span: *mut Self) {
        (*span).fNext = self.fNext;
        self.fNext = span;
    }
    pub unsafe fn onEnd(&self) -> bool {
        let span = self.fSpan;
        if (*span).ptT() != self as *const Self as *mut Self {
            return false;
        }
        let segment = self.segment();
        span == (*segment).head().cast::<SkOpSpanBase>() || span == (*segment).tail()
    }
    pub unsafe fn oppPrev(&self, opp: *const Self) -> *mut Self {
        let this = self as *const Self as *mut Self;
        let mut p = (*opp).fNext;
        if p == this {
            return null_mut();
        }
        while (*p).fNext.cast_const() != opp {
            p = (*p).fNext;
            if p == this {
                return null_mut();
            }
        }
        p
    }
    pub unsafe fn Overlaps(
        s1: *const Self,
        e1: *const Self,
        s2: *const Self,
        e2: *const Self,
        sOut: &mut *const Self,
        eOut: &mut *const Self,
    ) -> bool {
        let start1 = if (*s1).fT < (*e1).fT { s1 } else { e1 };
        let start2 = if (*s2).fT < (*e2).fT { s2 } else { e2 };
        *sOut = if between((*s1).fT, (*start2).fT, (*e1).fT) {
            start2
        } else if between((*s2).fT, (*start1).fT, (*e2).fT) {
            start1
        } else {
            core::ptr::null()
        };
        let end1 = if (*s1).fT < (*e1).fT { e1 } else { s1 };
        let end2 = if (*s2).fT < (*e2).fT { e2 } else { s2 };
        *eOut = if between((*s1).fT, (*end2).fT, (*e1).fT) {
            end2
        } else if between((*s2).fT, (*end1).fT, (*e2).fT) {
            end1
        } else {
            core::ptr::null()
        };
        if *sOut == *eOut {
            return false;
        }
        !sOut.is_null() && !eOut.is_null()
    }
    pub unsafe fn ptAlreadySeen(&self, mut check: *const Self) -> bool {
        while !core::ptr::eq(self, check) {
            if self.fPt == (*check).fPt {
                return true;
            }
            check = (*check).fNext;
        }
        false
    }
    pub unsafe fn prev(&self) -> *mut Self {
        let this = self as *const Self as *mut Self;
        let mut result = this;
        let mut next = self.fNext;
        while next != this {
            result = next;
            next = (*next).fNext;
        }

        result
    }
    pub fn setCoincident(&mut self) {
        self.fCoincident = true;
    }
    pub fn setDeleted(&mut self) {
        self.fDeleted = true;
    }
    pub fn setSpan(&mut self, span: *const SkOpSpanBase) {
        self.fSpan = span.cast_mut();
    }
    pub unsafe fn starter(&self, end: *const Self) -> *mut Self {
        if self.fT < (*end).fT {
            self as *const Self as *mut Self
        } else {
            end.cast_mut()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collapsed {
    kNo,
    kYes,
    kError,
}
#[repr(C)]
pub struct SkOpSpanBase {
    pub(super) fPtT: SkOpPtT,
    pub(super) fSegment: *mut SkOpSegment,
    pub(super) fCoinEnd: *mut Self,
    pub(super) fFromAngle: *mut SkOpAngle,
    pub(super) fPrev: *mut SkOpSpan,
    pub(super) fSpanAdds: i32,
    pub(super) fAligned: bool,
    pub(super) fChased: bool,
}
impl Default for SkOpSpanBase {
    fn default() -> Self {
        Self {
            fPtT: SkOpPtT::default(),
            fSegment: null_mut(),
            fCoinEnd: null_mut(),
            fFromAngle: null_mut(),
            fPrev: null_mut(),
            fSpanAdds: 0,
            fAligned: false,
            fChased: false,
        }
    }
}
impl SkOpSpanBase {
    pub unsafe fn addOpp(&mut self, opp: *mut Self) -> bool {
        let oppPrev = self.fPtT.oppPrev((*opp).ptT());
        if oppPrev.is_null() {
            return true;
        }
        if !self.mergeMatches(opp) {
            return false;
        }
        self.fPtT.addOpp((*opp).ptT(), oppPrev);
        self.checkForCollapsedCoincidence();
        true
    }
    pub fn bumpSpanAdds(&mut self) {
        self.fSpanAdds += 1;
    }
    pub fn chased(&self) -> bool {
        self.fChased
    }
    pub fn coinEnd(&self) -> *mut Self {
        self.fCoinEnd
    }
    pub unsafe fn collapsed(&self, s: f64, e: f64) -> Collapsed {
        let start = self.ptT();
        let mut startNext = null_mut();
        let mut walk = start;
        let mut min = (*walk).fT;
        let mut max = min;
        let segment = self.segment();
        let mut safetyNet = 100000;
        loop {
            walk = (*walk).next();
            if walk == start {
                return Collapsed::kNo;
            }
            safetyNet -= 1;
            if safetyNet == 0 || walk == startNext {
                return Collapsed::kError;
            }
            if (*walk).segment() != segment {
                continue;
            }
            min = min.min((*walk).fT);
            max = max.max((*walk).fT);
            if between(min, s, max) && between(min, e, max) {
                return Collapsed::kYes;
            }
            startNext = (*start).next();
        }
    }
    pub unsafe fn contains(&self, span: *const Self) -> bool {
        self.fPtT.contains((*span).ptT())
    }
    pub unsafe fn containsSegment(&self, segment: *const SkOpSegment) -> *mut SkOpPtT {
        let start = self.ptT();
        let mut walk = (*start).next();
        while walk != start {
            if !(*walk).deleted()
                && (*walk).segment().cast_const() == segment
                && (*(*walk).span()).ptT() == walk
            {
                return walk;
            }
            walk = (*walk).next();
        }
        null_mut()
    }
    pub unsafe fn containsCoinEnd(&self, coin: *const Self) -> bool {
        let this = self as *const Self;

        let mut next = self.fCoinEnd;
        while next.cast_const() != this {
            if next.cast_const() == coin {
                return true;
            }
            next = (*next).fCoinEnd;
        }
        false
    }
    pub unsafe fn containsCoinEndSegment(&self, segment: *const SkOpSegment) -> bool {
        let this = self as *const Self;
        let mut next = self.fCoinEnd;
        while next.cast_const() != this {
            if (*next).segment().cast_const() == segment {
                return true;
            }
            next = (*next).fCoinEnd;
        }
        false
    }
    pub unsafe fn contour(&self) -> *mut SkOpContour {
        (*self.segment()).contour()
    }
    pub unsafe fn globalState(&self) -> *mut SkOpGlobalState {
        (*self.contour()).globalState()
    }
    pub fn deleted(&self) -> bool {
        self.fPtT.deleted()
    }
    pub fn final_(&self) -> bool {
        self.fPtT.fT == 1.0
    }
    pub fn fromAngle(&self) -> *mut SkOpAngle {
        self.fFromAngle
    }
    pub unsafe fn initBase(
        &mut self,
        segment: *mut SkOpSegment,
        prev: *mut SkOpSpan,
        t: f64,
        pt: &SkPoint,
    ) {
        self.fSegment = segment;
        let this = self as *mut Self;
        self.fPtT.init(this, t, pt, false);
        self.fCoinEnd = this;
        self.fFromAngle = null_mut();
        self.fPrev = prev;
        self.fSpanAdds = 0;
        self.fAligned = true;
        self.fChased = false;
    }
    pub unsafe fn insertCoinEnd(&mut self, coin: *mut Self) {
        if self.containsCoinEnd(coin) {
            return;
        }

        let next = (*coin).fCoinEnd;
        (*coin).fCoinEnd = self.fCoinEnd;
        self.fCoinEnd = next;
    }
    pub fn prev(&self) -> *mut SkOpSpan {
        self.fPrev
    }
    pub fn pt(&self) -> &SkPoint {
        &self.fPtT.fPt
    }
    pub fn ptT(&self) -> *mut SkOpPtT {
        core::ptr::addr_of!(self.fPtT).cast_mut()
    }
    pub fn segment(&self) -> *mut SkOpSegment {
        self.fSegment
    }
    pub fn setAligned(&mut self) {
        self.fAligned = true;
    }
    pub fn setChased(&mut self, chased: bool) {
        self.fChased = chased;
    }
    pub fn setFromAngle(&mut self, angle: *mut SkOpAngle) {
        self.fFromAngle = angle;
    }
    pub fn setPrev(&mut self, prev: *mut SkOpSpan) {
        self.fPrev = prev;
    }
    pub unsafe fn simple(&self) -> bool {
        (*self.fPtT.next()).next() == self.ptT()
    }
    pub fn spanAddsCount(&self) -> i32 {
        self.fSpanAdds
    }
    pub unsafe fn starter(&self, end: *const Self) -> *mut SkOpSpan {
        let result = if self.t() < (*end).t() {
            self as *const Self as *mut Self
        } else {
            end.cast_mut()
        };
        (*result).upCast()
    }
    pub unsafe fn starterWithEnd(&self, endPtr: &mut *mut Self) -> *mut SkOpSpan {
        let end = *endPtr;

        let result = if self.t() < (*end).t() {
            self as *const Self as *mut Self
        } else {
            *endPtr = self as *const Self as *mut Self;
            end
        };
        (*result).upCast()
    }
    pub unsafe fn step(&self, end: *const Self) -> i32 {
        if self.t() < (*end).t() {
            1
        } else {
            -1
        }
    }
    pub fn t(&self) -> f64 {
        self.fPtT.fT
    }
    pub fn unaligned(&mut self) {
        self.fAligned = false;
    }
    pub fn upCast(&self) -> *mut SkOpSpan {
        (self as *const Self).cast_mut().cast()
    }
    pub fn upCastable(&self) -> *mut SkOpSpan {
        if self.final_() {
            null_mut()
        } else {
            self.upCast()
        }
    }
    pub unsafe fn merge(&mut self, span: *mut SkOpSpan) {
        let spanPtT = (*span).ptT();

        (*span).release(self.ptT());
        if self.contains(span.cast()) {
            return;
        }
        let mut remainder = (*spanPtT).next();
        self.fPtT.insert(spanPtT);
        while remainder != spanPtT {
            let next = (*remainder).next();
            let mut compare = (*spanPtT).next();
            let mut duplicate = false;
            while compare != spanPtT {
                let nextC = (*compare).next();
                if (*nextC).span() == (*remainder).span() && (*nextC).fT == (*remainder).fT {
                    duplicate = true;
                    break;
                }
                compare = nextC;
            }
            if !duplicate {
                (*spanPtT).insert(remainder);
            }
            remainder = next;
        }
        self.fSpanAdds += (*span).base.fSpanAdds;
    }
    pub unsafe fn checkForCollapsedCoincidence(&mut self) {
        let coins = (*self.globalState()).coincidence();
        if (*coins).isEmpty() {
            return;
        }
        let head = self.ptT();
        let mut test = head;
        loop {
            if (*test).coincident() {
                (*coins).markCollapsed(test);
            }
            test = (*test).next();
            if test == head {
                break;
            }
        }
        (*coins).releaseDeleted();
    }
    pub unsafe fn mergeMatches(&mut self, opp: *mut Self) -> bool {
        let mut test = self.ptT();
        let stop = test;
        let mut safetyHatch = 1000000;
        loop {
            safetyHatch -= 1;
            if safetyHatch == 0 {
                return false;
            }
            let testNext = (*test).next();
            if !(*test).deleted() {
                let testBase = (*test).span();

                let segment = (*test).segment();
                if !(*segment).done() {
                    let mut inner = (*opp).ptT();
                    let innerStop = inner;
                    loop {
                        if (*inner).segment() == segment && !(*inner).deleted() {
                            let innerBase = (*inner).span();

                            if !zero_or_one((*inner).fT) {
                                (*(*innerBase).upCast()).release(test);
                            } else {
                                if !zero_or_one((*test).fT) {
                                    (*(*testBase).upCast()).release(inner);
                                } else {
                                    (*segment).markAllDone();
                                    (*test).setDeleted();
                                    (*inner).setDeleted();
                                }
                            }
                            break;
                        }
                        inner = (*inner).next();
                        if inner == innerStop {
                            break;
                        }
                    }
                }
            }
            test = testNext;
            if test == stop {
                break;
            }
        }
        self.checkForCollapsedCoincidence();
        true
    }
}

#[repr(C)]
pub struct SkOpSpan {
    pub base: SkOpSpanBase,
    pub(super) fCoincident: *mut Self,
    pub(super) fToAngle: *mut SkOpAngle,
    pub(super) fNext: *mut SkOpSpanBase,
    pub(super) fWindSum: i32,
    pub(super) fOppSum: i32,
    pub(super) fWindValue: i32,
    pub(super) fOppValue: i32,
    pub(super) fTopTTry: i32,
    pub(super) fDone: bool,
    pub(super) fAlreadyAdded: bool,
}
impl Deref for SkOpSpan {
    type Target = SkOpSpanBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for SkOpSpan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl Default for SkOpSpan {
    fn default() -> Self {
        Self {
            base: SkOpSpanBase::default(),
            fCoincident: null_mut(),
            fToAngle: null_mut(),
            fNext: null_mut(),
            fWindSum: 0,
            fOppSum: 0,
            fWindValue: 0,
            fOppValue: 0,
            fTopTTry: 0,
            fDone: false,
            fAlreadyAdded: false,
        }
    }
}
impl SkOpSpan {
    pub fn alreadyAdded(&self) -> bool {
        self.fAlreadyAdded
    }
    pub fn clearCoincident(&mut self) -> bool {
        let this = self as *mut Self;
        if self.fCoincident == this {
            false
        } else {
            self.fCoincident = this;
            true
        }
    }
    pub unsafe fn computeWindSum(&mut self) -> i32 {
        let globals = self.globalState();
        let contourHead = (*globals).contourHead().cast::<SkOpContour>();
        let mut windTry = 0;
        while !self.sortableTop(contourHead) {
            windTry += 1;
            if windTry >= SkOpGlobalState::kMaxWindingTries {
                break;
            }
        }
        self.windSum()
    }
    pub unsafe fn containsCoincidenceSegment(&self, segment: *const SkOpSegment) -> bool {
        let mut next = self.fCoincident;
        loop {
            if (*next).segment().cast_const() == segment {
                return true;
            }
            next = (*next).fCoincident;
            if core::ptr::eq(next, self) {
                break;
            }
        }
        false
    }
    pub unsafe fn containsCoincidence(&self, coin: *const Self) -> bool {
        let this = self as *const Self;

        let mut next = self.fCoincident;
        while next.cast_const() != this {
            if next.cast_const() == coin {
                return true;
            }
            next = (*next).fCoincident;
        }
        false
    }
    pub fn done(&self) -> bool {
        self.fDone
    }
    pub unsafe fn init(
        &mut self,
        segment: *mut SkOpSegment,
        prev: *mut Self,
        t: f64,
        pt: &SkPoint,
    ) {
        self.initBase(segment, prev, t, pt);
        self.fCoincident = self;
        self.fToAngle = null_mut();
        self.fWindSum = i32::MIN;
        self.fOppSum = i32::MIN;
        self.fWindValue = 1;
        self.fOppValue = 0;
        self.fTopTTry = 0;
        self.fChased = false;
        self.fDone = false;
        (*segment).bumpCount();
        self.fAlreadyAdded = false;
    }
    pub unsafe fn insertCoincidenceSegment(
        &mut self,
        segment: *const SkOpSegment,
        flipped: bool,
        ordered: bool,
    ) -> bool {
        if self.containsCoincidenceSegment(segment) {
            return true;
        }
        let stop = self.ptT();
        let mut next = (*stop).next();
        while next != stop {
            if (*next).segment().cast_const() == segment {
                let base = (*next).span();
                let span;
                if !ordered {
                    let endPtT = (*self.fNext).containsSegment(segment);
                    if endPtT.is_null() {
                        return false;
                    }
                    let end = (*endPtT).span();
                    let start = (*(*base).ptT()).starter((*end).ptT());
                    if (*(*start).span()).upCastable().is_null() {
                        return false;
                    }
                    span = (*(*start).span()).upCast();
                } else if flipped {
                    span = (*base).prev();
                    if span.is_null() {
                        return false;
                    }
                } else {
                    span = (*base).upCastable();
                    if span.is_null() {
                        return false;
                    }
                }
                self.insertCoincidence(span);
                return true;
            }
            next = (*next).next();
        }
        true
    }
    pub unsafe fn insertCoincidence(&mut self, coin: *mut Self) {
        if self.containsCoincidence(coin) {
            return;
        }

        let next = (*coin).fCoincident;
        (*coin).fCoincident = self.fCoincident;
        self.fCoincident = next;
    }
    pub fn isCanceled(&self) -> bool {
        self.fWindValue == 0 && self.fOppValue == 0
    }
    pub fn isCoincident(&self) -> bool {
        !core::ptr::eq(self.fCoincident, self)
    }
    pub fn markAdded(&mut self) {
        self.fAlreadyAdded = true;
    }
    pub fn next(&self) -> *mut SkOpSpanBase {
        self.fNext
    }
    pub fn oppSum(&self) -> i32 {
        self.fOppSum
    }
    pub fn oppValue(&self) -> i32 {
        self.fOppValue
    }
    pub unsafe fn release(&mut self, kept: *const SkOpPtT) {
        let prev = self.prev();

        let next = self.next();

        (*prev).setNext(next);
        (*next).setPrev(prev);
        let this = self as *mut Self;
        (*self.segment()).release(this);
        let coincidence = (*self.globalState()).coincidence();
        if !coincidence.is_null() {
            (*coincidence).fixUp(self.ptT(), kept);
        }
        (*self.ptT()).setDeleted();
        let stop = self.ptT();
        let mut test = stop;
        let keptSpan = (*kept).span();
        loop {
            if (*test).span() == this.cast() {
                (*test).setSpan(keptSpan);
            }
            test = (*test).next();
            if test == stop {
                break;
            }
        }
    }
    pub fn setDone(&mut self, done: bool) {
        self.fDone = done;
    }
    pub fn setNext(&mut self, next: *mut SkOpSpanBase) {
        self.fNext = next;
    }
    pub unsafe fn setOppSum(&mut self, sum: i32) {
        if self.fOppSum != i32::MIN && self.fOppSum != sum {
            (*self.globalState()).setWindingFailed();
            return;
        }
        self.fOppSum = sum;
    }
    pub fn setOppValue(&mut self, value: i32) {
        self.fOppValue = value;
    }
    pub fn setToAngle(&mut self, angle: *mut SkOpAngle) {
        self.fToAngle = angle;
    }
    pub unsafe fn setWindSum(&mut self, sum: i32) {
        if self.fWindSum != i32::MIN && self.fWindSum != sum {
            (*self.globalState()).setWindingFailed();
            return;
        }
        self.fWindSum = sum;
    }
    pub fn setWindValue(&mut self, value: i32) {
        self.fWindValue = value;
    }
    pub fn toAngle(&self) -> *mut SkOpAngle {
        self.fToAngle
    }
    pub fn windSum(&self) -> i32 {
        self.fWindSum
    }
    pub fn windValue(&self) -> i32 {
        self.fWindValue
    }
}
