// Copyright 2013 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkOpCoincidence.h:23-135, SkOpCoincidence.cpp:21-220.
use super::{
    SkOpCoincidence::SkOpCoincidence,
    SkOpSpan::{SkOpPtT, SkOpSpanBase},
};
use core::ptr::null_mut;

#[repr(C)]
pub struct SkCoincidentSpans {
    pub(super) fNext: *mut Self,
    pub(super) fCoinPtTStart: *mut SkOpPtT,
    pub(super) fCoinPtTEnd: *mut SkOpPtT,
    pub(super) fOppPtTStart: *mut SkOpPtT,
    pub(super) fOppPtTEnd: *mut SkOpPtT,
}
impl Default for SkCoincidentSpans {
    fn default() -> Self {
        Self {
            fNext: null_mut(),
            fCoinPtTStart: null_mut(),
            fCoinPtTEnd: null_mut(),
            fOppPtTStart: null_mut(),
            fOppPtTEnd: null_mut(),
        }
    }
}
impl SkCoincidentSpans {
    pub fn init(&mut self) {
        *self = Self::default();
    }
    pub fn next(&self) -> *mut Self {
        self.fNext
    }
    pub fn nextPtr(&mut self) -> *mut *mut Self {
        &mut self.fNext
    }
    pub fn setNext(&mut self, next: *mut Self) {
        self.fNext = next;
    }
    pub fn coinPtTStart(&self) -> *mut SkOpPtT {
        self.fCoinPtTStart
    }
    pub fn coinPtTEnd(&self) -> *mut SkOpPtT {
        self.fCoinPtTEnd
    }
    pub fn oppPtTStart(&self) -> *mut SkOpPtT {
        self.fOppPtTStart
    }
    pub fn oppPtTEnd(&self) -> *mut SkOpPtT {
        self.fOppPtTEnd
    }
    pub fn coinPtTStartWritable(&self) -> *mut SkOpPtT {
        self.fCoinPtTStart
    }
    pub fn coinPtTEndWritable(&self) -> *mut SkOpPtT {
        self.fCoinPtTEnd
    }
    pub fn oppPtTStartWritable(&self) -> *mut SkOpPtT {
        self.fOppPtTStart
    }
    pub fn oppPtTEndWritable(&self) -> *mut SkOpPtT {
        self.fOppPtTEnd
    }
    pub unsafe fn flipped(&self) -> bool {
        (*self.fOppPtTStart).fT > (*self.fOppPtTEnd).fT
    }
    pub unsafe fn setCoinPtTStart(&mut self, p: *const SkOpPtT) {
        let p = p.cast_mut();

        self.fCoinPtTStart = p;
        (*p).setCoincident();
    }
    pub unsafe fn setCoinPtTEnd(&mut self, p: *const SkOpPtT) {
        let p = p.cast_mut();

        self.fCoinPtTEnd = p;
        (*p).setCoincident();
    }
    pub unsafe fn setOppPtTStart(&mut self, p: *const SkOpPtT) {
        let p = p.cast_mut();

        self.fOppPtTStart = p;
        (*p).setCoincident();
    }
    pub unsafe fn setOppPtTEnd(&mut self, p: *const SkOpPtT) {
        let p = p.cast_mut();

        self.fOppPtTEnd = p;
        (*p).setCoincident();
    }
    pub unsafe fn setStarts(&mut self, a: *const SkOpPtT, b: *const SkOpPtT) {
        self.setCoinPtTStart(a);
        self.setOppPtTStart(b);
    }
    pub unsafe fn setEnds(&mut self, a: *const SkOpPtT, b: *const SkOpPtT) {
        self.setCoinPtTEnd(a);
        self.setOppPtTEnd(b);
    }
    pub unsafe fn set(
        &mut self,
        next: *mut Self,
        cs: *mut SkOpPtT,
        ce: *mut SkOpPtT,
        os: *mut SkOpPtT,
        oe: *mut SkOpPtT,
    ) {
        self.fNext = next;
        self.setStarts(cs, os);
        self.setEnds(ce, oe);
    }
    pub unsafe fn collapsed(&self, test: *const SkOpPtT) -> bool {
        (self.fCoinPtTStart.cast_const() == test && (*self.fCoinPtTEnd).contains(test))
            || (self.fCoinPtTEnd.cast_const() == test && (*self.fCoinPtTStart).contains(test))
            || (self.fOppPtTStart.cast_const() == test && (*self.fOppPtTEnd).contains(test))
            || (self.fOppPtTEnd.cast_const() == test && (*self.fOppPtTStart).contains(test))
    }
    pub unsafe fn correctOneEnd(
        &mut self,
        get: fn(&Self) -> *mut SkOpPtT,
        set: unsafe fn(&mut Self, *const SkOpPtT),
    ) {
        let orig = get(self);
        let span = (*orig).span();
        let prev = (*span).prev();
        let test = if !prev.is_null() {
            (*(*prev).next()).ptT()
        } else {
            (*(*(*(*span).upCast()).next()).prev()).ptT()
        };
        if orig != test {
            set(self, test);
        }
    }
    pub unsafe fn correctEnds(&mut self) {
        self.correctOneEnd(Self::coinPtTStart, Self::setCoinPtTStart);
        self.correctOneEnd(Self::coinPtTEnd, Self::setCoinPtTEnd);
        self.correctOneEnd(Self::oppPtTStart, Self::setOppPtTStart);
        self.correctOneEnd(Self::oppPtTEnd, Self::setOppPtTEnd);
    }
    pub unsafe fn expand(&mut self) -> bool {
        let mut expanded = false;
        let segment = (*self.coinPtTStart()).segment();
        let opp = (*self.oppPtTStart()).segment();
        loop {
            let start = (*(*self.coinPtTStart()).span()).upCast();
            let prev = (*start).prev();
            if prev.is_null() {
                break;
            }
            let oppPtT = (*prev).containsSegment(opp);
            if oppPtT.is_null() {
                break;
            }
            let t = ((*prev).t() + (*start).t()) / 2.0;
            if !(*segment).isClose(t, opp) {
                break;
            }
            self.setStarts((*prev).ptT(), oppPtT);
            expanded = true;
        }
        loop {
            let end = (*self.coinPtTEnd()).span();
            let next = if (*end).final_() {
                null_mut()
            } else {
                (*(*end).upCast()).next()
            };
            if !next.is_null() && (*next).deleted() {
                break;
            }
            if next.is_null() {
                break;
            }
            let oppPtT = (*next).containsSegment(opp);
            if oppPtT.is_null() {
                break;
            }
            let t = ((*end).t() + (*next).t()) / 2.0;
            if !(*segment).isClose(t, opp) {
                break;
            }
            self.setEnds((*next).ptT(), oppPtT);
            expanded = true;
        }
        expanded
    }
    pub unsafe fn extend(
        &mut self,
        cs: *const SkOpPtT,
        ce: *const SkOpPtT,
        os: *const SkOpPtT,
        oe: *const SkOpPtT,
    ) -> bool {
        let mut result = false;
        if (*self.fCoinPtTStart).fT > (*cs).fT
            || if self.flipped() {
                (*self.fOppPtTStart).fT < (*os).fT
            } else {
                (*self.fOppPtTStart).fT > (*os).fT
            }
        {
            self.setStarts(cs, os);
            result = true;
        }
        if (*self.fCoinPtTEnd).fT < (*ce).fT
            || if self.flipped() {
                (*self.fOppPtTEnd).fT > (*oe).fT
            } else {
                (*self.fOppPtTEnd).fT < (*oe).fT
            }
        {
            self.setEnds(ce, oe);
            result = true;
        }
        result
    }
    pub unsafe fn contains(&self, mut s: *const SkOpPtT, mut e: *const SkOpPtT) -> bool {
        if (*s).fT > (*e).fT {
            core::mem::swap(&mut s, &mut e);
        }
        if (*s).segment() == (*self.fCoinPtTStart).segment() {
            (*self.fCoinPtTStart).fT <= (*s).fT && (*e).fT <= (*self.fCoinPtTEnd).fT
        } else {
            let mut ts = (*self.fOppPtTStart).fT;
            let mut te = (*self.fOppPtTEnd).fT;
            if ts > te {
                core::mem::swap(&mut ts, &mut te);
            }
            ts <= (*s).fT && (*e).fT <= te
        }
    }
    pub unsafe fn ordered(&self, result: &mut bool) -> bool {
        let start = (*self.coinPtTStart()).span();
        let end = (*self.coinPtTEnd()).span();
        let mut next = (*(*start).upCast()).next();
        if next == end {
            *result = true;
            return true;
        }
        let flipped = self.flipped();
        let opp = (*self.oppPtTStart()).segment();
        let mut last = (*self.fOppPtTStart).fT;
        loop {
            let o = (*next).containsSegment(opp);
            if o.is_null() {
                return false;
            }
            if (last > (*o).fT) != flipped {
                *result = false;
                return true;
            }
            last = (*o).fT;
            if next == end {
                break;
            }
            if (*next).upCastable().is_null() {
                *result = false;
                return true;
            }
            next = (*(*next).upCast()).next();
        }
        *result = true;
        true
    }
}
