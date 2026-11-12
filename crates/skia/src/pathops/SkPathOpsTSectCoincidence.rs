// Copyright 2014 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkPathOpsTSect.cpp:651-1617, release span/coincidence lifecycle.
use super::{
    SkIntersections::SkIntersections,
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::SkDPoint,
    SkPathOpsTCurve::SkTCurve,
    SkPathOpsTSect::{SkTCoincident, SkTSect, SkTSpan},
    SkPathOpsTypes::*,
};
use core::ptr::null_mut;
impl<'a> SkTSect<'a> {
    pub(super) unsafe fn coincidentCheck(&mut self, sect2: *mut SkTSect<'_>) -> bool {
        let mut first = self.fHead;
        if first.is_null() {
            return false;
        }
        loop {
            let mut last = null_mut();
            let consecutive = self.countConsecutiveSpans(first, &mut last);
            let next = (*last).fNext;
            if consecutive >= 9 {
                self.computePerpendiculars(sect2, first, last);
                let mut coinStart = first;
                loop {
                    if !self.extractCoincident(sect2, coinStart, last, &mut coinStart) {
                        return false;
                    }
                    if coinStart.is_null() || (*last).fDeleted {
                        break;
                    }
                }
                if self.fHead.is_null()
                    || (*sect2).fHead.is_null()
                    || next.is_null()
                    || (*next).fDeleted
                {
                    break;
                }
            }
            first = next;
            if first.is_null() {
                break;
            }
        }
        true
    }
    pub(super) unsafe fn coincidentForce(
        &mut self,
        sect2: *mut SkTSect<'_>,
        start1s: f64,
        start1e: f64,
    ) {
        let first = self.fHead;
        let last = self.tail();
        let oppFirst = (*sect2).fHead;
        let oppLast = (*sect2).tail();
        if last.is_null() || oppLast.is_null() {
            return;
        }
        let mut delete = self.updateBounded(first, last, oppFirst);
        delete |= (*sect2).updateBounded(oppFirst, oppLast, first);
        self.removeSpanRange(first, last);
        (*sect2).removeSpanRange(oppFirst, oppLast);
        (*first).fStartT = start1s;
        (*first).fEndT = start1e;
        (*first).resetBounds(self.fCurve);
        (*first)
            .fCoinStart
            .setPerp(self.fCurve, start1s, *self.fCurve.point(0), (*sect2).fCurve);
        (*first)
            .fCoinEnd
            .setPerp(self.fCurve, start1e, self.pointLast(), (*sect2).fCurve);
        let matched = (*first).fCoinStart.perpT() < (*first).fCoinEnd.perpT();
        let mut s = if (*first).fCoinStart.perpT() == -1.0 {
            0.0
        } else {
            cpp_max(0.0, (*first).fCoinStart.perpT())
        };
        let mut e = if (*first).fCoinEnd.perpT() == -1.0 {
            1.0
        } else {
            cpp_min(1.0, (*first).fCoinEnd.perpT())
        };
        if !matched {
            core::mem::swap(&mut s, &mut e);
        }
        (*oppFirst).fStartT = s;
        (*oppFirst).fEndT = e;
        (*oppFirst).resetBounds((*sect2).fCurve);
        self.removeCoincident(first, false);
        (*sect2).removeCoincident(oppFirst, true);
        if delete {
            self.deleteEmptySpans();
            (*sect2).deleteEmptySpans();
        }
    }
    pub(super) unsafe fn coincidentHasT(&self, t: f64) -> bool {
        let mut p = self.fCoincident;
        while !p.is_null() {
            if between((*p).fStartT, t, (*p).fEndT) {
                return true;
            }
            p = (*p).fNext;
        }
        false
    }
    pub(super) unsafe fn collapsed(&self) -> i32 {
        let mut count = 0;
        let mut p = self.fHead;
        while !p.is_null() {
            if (*p).fCollapsed {
                count += 1;
            }
            p = (*p).fNext;
        }
        count
    }
    pub(super) unsafe fn computePerpendiculars(
        &mut self,
        sect2: *mut SkTSect<'_>,
        first: *mut SkTSpan,
        last: *mut SkTSpan,
    ) {
        if last.is_null() {
            return;
        }
        let opp = (*sect2).fCurve;
        let mut work = first;
        let mut prior: *mut SkTSpan = null_mut();
        loop {
            if !(*work).fHasPerp && !(*work).fCollapsed {
                if !prior.is_null() {
                    (*work).fCoinStart = (*prior).fCoinEnd;
                } else {
                    (*work).fCoinStart.setPerp(
                        self.fCurve,
                        (*work).fStartT,
                        (*work).pointFirst(),
                        opp,
                    );
                }
                if (*work).fCoinStart.isMatch() {
                    let t = (*work).fCoinStart.perpT();
                    if (*sect2).coincidentHasT(t) {
                        (*work).fCoinStart.init();
                    } else {
                        (*sect2).addForPerp(work, t);
                    }
                }
                (*work)
                    .fCoinEnd
                    .setPerp(self.fCurve, (*work).fEndT, (*work).pointLast(), opp);
                if (*work).fCoinEnd.isMatch() {
                    let t = (*work).fCoinEnd.perpT();
                    if (*sect2).coincidentHasT(t) {
                        (*work).fCoinEnd.init();
                    } else {
                        (*sect2).addForPerp(work, t);
                    }
                }
                (*work).fHasPerp = true;
            }
            if work == last {
                break;
            }
            prior = work;
            work = (*work).fNext;
        }
    }
    pub(super) unsafe fn countConsecutiveSpans(
        &self,
        first: *mut SkTSpan,
        lastPtr: &mut *mut SkTSpan,
    ) -> i32 {
        let mut n = 1;
        let mut last = first;
        loop {
            let next = (*last).fNext;
            if next.is_null() || (*next).fStartT > (*last).fEndT {
                break;
            }
            n += 1;
            last = next;
        }
        *lastPtr = last;
        n
    }
    pub unsafe fn hasBounded(&self, span: *const SkTSpan) -> bool {
        let mut p = self.fHead;
        while !p.is_null() {
            if !(*p).findOppSpan(span).is_null() {
                return true;
            }
            p = (*p).fNext;
        }
        false
    }
    pub(super) unsafe fn deleteEmptySpans(&mut self) -> bool {
        let mut next = self.fHead;
        let mut safety = 1000;
        while !next.is_null() {
            let p = next;
            next = (*p).fNext;
            if (*p).fBounded.is_null() && !self.removeSpan(p) {
                return false;
            }
            safety -= 1;
            if safety < 0 {
                return false;
            }
        }
        true
    }
    pub(super) unsafe fn extractCoincident(
        &mut self,
        sect2: *mut SkTSect<'_>,
        mut first: *mut SkTSpan,
        mut last: *mut SkTSpan,
        result: &mut *mut SkTSpan,
    ) -> bool {
        first = self.findCoincidentRun(first, &mut last);
        if first.is_null() || last.is_null() {
            *result = null_mut();
            return true;
        }
        let startT = (*first).fStartT;
        let mut oppStartT = 0.0;
        let mut oppEndT = 0.0;
        let prev = (*first).fPrev;

        let mut oppFirst = (*first).findOppT((*first).fCoinStart.perpT());

        let matched = (*first).fCoinStart.perpT() < (*first).fCoinEnd.perpT();
        let mut coinStart = 0.0;
        let mut cutFirst = null_mut();
        if !prev.is_null()
            && (*prev).fEndT == startT
            && self.binarySearchCoin(
                sect2,
                startT,
                (*prev).fStartT - startT,
                &mut coinStart,
                &mut oppStartT,
                &mut oppFirst,
            )
            && (*prev).fStartT < coinStart
            && coinStart < startT
            && {
                cutFirst = (*prev).oppT(oppStartT);
                !cutFirst.is_null()
            }
        {
            oppFirst = cutFirst;
            first = self.addSplitAt(prev, coinStart);
            (*first).markCoincident();
            (*prev).fCoinEnd.markCoincident();
            if (*oppFirst).fStartT < oppStartT && oppStartT < (*oppFirst).fEndT {
                let half = (*sect2).addSplitAt(oppFirst, oppStartT);
                if matched {
                    (*oppFirst).fCoinEnd.markCoincident();
                    (*half).markCoincident();
                    oppFirst = half;
                } else {
                    (*oppFirst).markCoincident();
                    (*half).fCoinStart.markCoincident();
                }
            }
        } else if oppFirst.is_null() {
            return false;
        }
        let mut oppLast = (*last).findOppT((*last).fCoinEnd.perpT());
        if !matched {
            core::mem::swap(&mut oppFirst, &mut oppLast);
            core::mem::swap(&mut oppStartT, &mut oppEndT);
        }
        if oppFirst.is_null() || oppLast.is_null() {
            *result = null_mut();
            return true;
        }
        let mut delete = self.updateBounded(first, last, oppFirst);
        delete |= (*sect2).updateBounded(oppFirst, oppLast, first);
        self.removeSpanRange(first, last);
        (*sect2).removeSpanRange(oppFirst, oppLast);
        (*first).fEndT = (*last).fEndT;
        (*first).resetBounds(self.fCurve);
        (*first).fCoinStart.setPerp(
            self.fCurve,
            (*first).fStartT,
            (*first).pointFirst(),
            (*sect2).fCurve,
        );
        (*first).fCoinEnd.setPerp(
            self.fCurve,
            (*first).fEndT,
            (*first).pointLast(),
            (*sect2).fCurve,
        );
        oppStartT = (*first).fCoinStart.perpT();
        oppEndT = (*first).fCoinEnd.perpT();
        if between(0.0, oppStartT, 1.0) && between(0.0, oppEndT, 1.0) {
            if !matched {
                core::mem::swap(&mut oppStartT, &mut oppEndT);
            }
            (*oppFirst).fStartT = oppStartT;
            (*oppFirst).fEndT = oppEndT;
            (*oppFirst).resetBounds((*sect2).fCurve);
        }
        last = (*first).fNext;
        if !self.removeCoincident(first, false) || !(*sect2).removeCoincident(oppFirst, true) {
            return false;
        }
        if delete && (!self.deleteEmptySpans() || !(*sect2).deleteEmptySpans()) {
            *result = null_mut();
            return false;
        }
        *result = if !last.is_null()
            && !(*last).fDeleted
            && !self.fHead.is_null()
            && !(*sect2).fHead.is_null()
        {
            last
        } else {
            null_mut()
        };
        true
    }
    pub(super) unsafe fn findCoincidentRun(
        &self,
        mut first: *mut SkTSpan,
        lastPtr: &mut *mut SkTSpan,
    ) -> *mut SkTSpan {
        let mut work = first;
        let mut candidate = null_mut();
        first = null_mut();
        loop {
            if (*work).fCoinStart.isMatch() {
                if !(*work).fCoinEnd.isMatch() {
                    break;
                }
                candidate = work;
                if first.is_null() {
                    first = work;
                }
            } else if !first.is_null() && (*work).fCollapsed {
                *lastPtr = candidate;
                return first;
            } else {
                candidate = null_mut();
            }
            if work == *lastPtr {
                return first;
            }
            work = (*work).fNext;
            if work.is_null() {
                return null_mut();
            }
        }
        if !candidate.is_null() {
            *lastPtr = candidate;
        }
        first
    }
    pub(super) unsafe fn intersects(
        &mut self,
        span: *mut SkTSpan,
        opp: *mut SkTSect<'_>,
        oppSpan: *mut SkTSpan,
        oppResult: &mut i32,
    ) -> i32 {
        let (mut start, mut oppStart) = (false, false);
        let mut result = (*span).hullsIntersect(oppSpan, &mut start, &mut oppStart);
        if result >= 0 {
            if result == 2 {
                if (*span).fBounded.is_null() || (*(*span).fBounded).fNext.is_null() {
                    if start {
                        (*span).fEndT = (*span).fStartT;
                    } else {
                        (*span).fStartT = (*span).fEndT;
                    }
                } else {
                    result = 1;
                }
                if (*oppSpan).fBounded.is_null() || (*(*oppSpan).fBounded).fNext.is_null() {
                    if !(*oppSpan).fBounded.is_null() && (*(*oppSpan).fBounded).fBounded != span {
                        return 0;
                    }
                    if oppStart {
                        (*oppSpan).fEndT = (*oppSpan).fStartT;
                    } else {
                        (*oppSpan).fStartT = (*oppSpan).fEndT;
                    }
                    *oppResult = 2;
                } else {
                    *oppResult = 1;
                }
            } else {
                *oppResult = 1;
            }
            return result;
        }
        if (*span).fIsLine && (*oppSpan).fIsLine {
            let mut i = SkIntersections::default();
            let count = self.linesIntersect(span, opp, oppSpan, &mut i);
            if count == 2 {
                *oppResult = 1;
                return 1;
            }
            if count == 0 {
                return -1;
            }
            self.removedEndCheck(span);
            (*span).fStartT = i[0][0];
            (*span).fEndT = i[0][0];
            (*opp).removedEndCheck(oppSpan);
            (*oppSpan).fStartT = i[1][0];
            (*oppSpan).fEndT = i[1][0];
            *oppResult = 2;
            return 2;
        }
        if (*span).fIsLinear || (*oppSpan).fIsLinear {
            *oppResult = (*span).linearsIntersect(oppSpan) as i32;
            return *oppResult;
        }
        *oppResult = 1;
        1
    }
    pub(super) unsafe fn linesIntersect(
        &mut self,
        span: *mut SkTSpan,
        opp: *mut SkTSect<'_>,
        oppSpan: *mut SkTSpan,
        i: &mut SkIntersections,
    ) -> i32 {
        let mut thisRay = SkIntersections::default();
        let mut oppRay = SkIntersections::default();
        let mut thisLine = SkDLine {
            fPts: [(*span).pointFirst(), (*span).pointLast()],
        };
        let mut oppLine = SkDLine {
            fPts: [(*oppSpan).pointFirst(), (*oppSpan).pointLast()],
        };
        let mut loops = 0;
        let mut best = f64::MAX;
        if (*opp).fCurve.intersectRay(&mut thisRay, &thisLine) == 0
            || self.fCurve.intersectRay(&mut oppRay, &oppLine) == 0
        {
            return 0;
        }
        if thisRay.used() > 1 {
            let mut matches = 0;
            for n in 0..thisRay.used() {
                for p in &thisLine.fPts {
                    matches += thisRay.pt(n).approximatelyEqual(p) as i32;
                }
            }
            if matches == 2 || is_parallel(&thisLine, (*opp).fCurve) {
                return 2;
            }
        }
        if oppRay.used() > 1 {
            let mut matches = 0;
            for n in 0..oppRay.used() {
                for p in &oppLine.fPts {
                    matches += oppRay.pt(n).approximatelyEqual(p) as i32;
                }
            }
            if matches == 2 || is_parallel(&oppLine, self.fCurve) {
                return 2;
            }
        }
        loop {
            let mut closest = f64::MAX;
            let (mut closeIndex, mut oppCloseIndex) = (0, 0);
            for index in 0..oppRay.used() {
                if !roughly_between((*span).fStartT, oppRay[0][index], (*span).fEndT) {
                    continue;
                }
                for oIndex in 0..thisRay.used() {
                    if !roughly_between((*oppSpan).fStartT, thisRay[0][oIndex], (*oppSpan).fEndT) {
                        continue;
                    }
                    let d = thisRay.pt(index).distanceSquared(&oppRay.pt(oIndex));
                    if closest > d {
                        closest = d;
                        closeIndex = index;
                        oppCloseIndex = oIndex;
                    }
                }
            }
            if closest == f64::MAX {
                break;
            }
            let oppPt = thisRay.pt(oppCloseIndex);
            let pt = oppRay.pt(closeIndex);
            if between((*span).fStartT, oppRay[0][closeIndex], (*span).fEndT)
                && between(
                    (*oppSpan).fStartT,
                    thisRay[0][oppCloseIndex],
                    (*oppSpan).fEndT,
                )
                && oppPt.approximatelyEqual(pt)
            {
                i.merge(&oppRay, closeIndex, &thisRay, oppCloseIndex);
                return i.used() as i32;
            }
            let dist = oppPt.distanceSquared(&pt);
            loops += 1;
            if best < dist || loops > 5 {
                return 0;
            }
            best = dist;
            let oppStart = oppRay[0][closeIndex];
            thisLine[0] = self.fCurve.ptAtT(oppStart);
            thisLine[1] = thisLine[0] + self.fCurve.dxdyAtT(oppStart);
            if (*opp).fCurve.intersectRay(&mut thisRay, &thisLine) == 0 {
                break;
            }
            let start = thisRay[0][oppCloseIndex];
            oppLine[0] = (*opp).fCurve.ptAtT(start);
            oppLine[1] = oppLine[0] + (*opp).fCurve.dxdyAtT(start);
            if self.fCurve.intersectRay(&mut oppRay, &oppLine) == 0 {
                break;
            }
        }
        let mut coinS = SkTCoincident::default();
        let mut coinE = SkTCoincident::default();
        coinS.setPerp(
            (*opp).fCurve,
            (*oppSpan).fStartT,
            (*oppSpan).pointFirst(),
            self.fCurve,
        );
        coinE.setPerp(
            (*opp).fCurve,
            (*oppSpan).fEndT,
            (*oppSpan).pointLast(),
            self.fCurve,
        );
        let mut start = coinS.perpT();
        let mut end = coinE.perpT();
        let swapped = start > end;
        if swapped {
            core::mem::swap(&mut start, &mut end);
        }
        start = cpp_max(start, (*span).fStartT);
        end = cpp_min(end, (*span).fEndT);
        if start > end {
            return 0;
        }
        let perpS = if start == (*span).fStartT {
            let mut coin = SkTCoincident::default();
            coin.setPerp(
                self.fCurve,
                (*span).fStartT,
                (*span).pointFirst(),
                (*opp).fCurve,
            );
            (*span).pointFirst() - coin.perpPt()
        } else if swapped {
            coinE.perpPt() - (*oppSpan).pointLast()
        } else {
            coinS.perpPt() - (*oppSpan).pointFirst()
        };
        let perpE = if end == (*span).fEndT {
            let mut coin = SkTCoincident::default();
            coin.setPerp(
                self.fCurve,
                (*span).fEndT,
                (*span).pointLast(),
                (*opp).fCurve,
            );
            (*span).pointLast() - coin.perpPt()
        } else if swapped {
            coinS.perpPt() - (*oppSpan).pointFirst()
        } else {
            coinE.perpPt() - (*oppSpan).pointLast()
        };
        if perpS.dot(&perpE) >= 0.0 {
            return 0;
        }
        let mut coin = SkTCoincident::default();
        let mut workT = start;
        let mut step = end - start;
        let workPt;
        loop {
            step *= 0.5;
            if precisely_zero(step) {
                return 0;
            }
            workT += step;
            let point = self.fCurve.ptAtT(workT);
            coin.setPerp(self.fCurve, workT, point, (*opp).fCurve);
            let perpT = coin.perpT();
            if if coin.isMatch() {
                !between((*oppSpan).fStartT, perpT, (*oppSpan).fEndT)
            } else {
                perpT < 0.0
            } {
                continue;
            }
            let perp = point - coin.perpPt();
            if (perpS.dot(&perp) >= 0.0) == (step < 0.0) {
                step = -step;
            }
            if point.approximatelyEqual(coin.perpPt()) {
                workPt = point;
                break;
            }
        }
        let t = coin.perpT();
        if !(*(*opp).fHead).contains(t) {
            return 0;
        }
        i.setMax(1);
        i.insert(workT, t, workPt);
        1
    }
    pub(super) unsafe fn markSpanGone(&mut self, span: *mut SkTSpan) -> bool {
        self.fActiveCount -= 1;
        if self.fActiveCount < 0 {
            return false;
        }
        (*span).fNext = self.fDeleted;
        self.fDeleted = span;

        (*span).fDeleted = true;
        true
    }
    pub(super) unsafe fn matchedDirection(
        &self,
        t: f64,
        sect2: *const SkTSect<'_>,
        t2: f64,
    ) -> bool {
        self.fCurve.dxdyAtT(t).dot(&(*sect2).fCurve.dxdyAtT(t2)) >= 0.0
    }
    pub(super) unsafe fn matchedDirCheck(
        &self,
        t: f64,
        sect2: *const SkTSect<'_>,
        t2: f64,
        calc: &mut bool,
        matched: &mut bool,
    ) {
        if *calc {
        } else {
            *matched = self.matchedDirection(t, sect2, t2);
            *calc = true;
        }
    }
    pub(super) unsafe fn mergeCoincidence(&mut self, sect2: *mut SkTSect<'_>) {
        let mut smallLimit = 0.0;
        loop {
            let mut smaller: *mut SkTSpan = null_mut();
            let mut test = self.fCoincident;
            loop {
                if test.is_null() {
                    return;
                }
                if !((*test).fStartT < smallLimit)
                    && (smaller.is_null() || !((*smaller).fEndT < (*test).fStartT))
                {
                    smaller = test;
                }
                test = (*test).fNext;
                if test.is_null() {
                    break;
                }
            }
            if smaller.is_null() {
                return;
            }
            smallLimit = (*smaller).fEndT;
            let mut prior: *mut SkTSpan = null_mut();
            let mut larger: *mut SkTSpan = null_mut();
            let mut largerPrior = null_mut();
            test = self.fCoincident;
            loop {
                if !((*test).fStartT < (*smaller).fEndT) {
                    if larger.is_null() || !((*larger).fStartT < (*test).fStartT) {
                        largerPrior = prior;
                        larger = test;
                    }
                }
                prior = test;
                test = (*test).fNext;
                if test.is_null() {
                    break;
                }
            }
            if larger.is_null() {
                continue;
            }
            let mid = ((*smaller).fEndT + (*larger).fStartT) / 2.0;
            let point = self.fCurve.ptAtT(mid);
            let mut coin = SkTCoincident::default();
            coin.setPerp(self.fCurve, mid, point, (*sect2).fCurve);
            if coin.isMatch() {
                (*smaller).fEndT = (*larger).fEndT;
                (*smaller).fCoinEnd = (*larger).fCoinEnd;
                if !largerPrior.is_null() {
                    (*largerPrior).fNext = (*larger).fNext;
                } else {
                    self.fCoincident = (*larger).fNext;
                }
            }
        }
    }
    pub(super) unsafe fn prev(&self, span: *mut SkTSpan) -> *mut SkTSpan {
        let mut result = null_mut();
        let mut test = self.fHead;
        while span != test {
            result = test;
            test = (*test).fNext;
        }
        result
    }
    pub(super) unsafe fn recoverCollapsed(&mut self) {
        let mut deleted = self.fDeleted;
        while !deleted.is_null() {
            let next = (*deleted).fNext;
            if (*deleted).fCollapsed {
                let mut slot = core::ptr::addr_of_mut!(self.fHead);
                while !(*slot).is_null() && (**slot).fEndT <= (*deleted).fStartT {
                    slot = core::ptr::addr_of_mut!((**slot).fNext);
                }
                (*deleted).fNext = *slot;
                *slot = deleted;
            }
            deleted = next;
        }
    }
    pub(super) unsafe fn removeAllBut(
        &mut self,
        keep: *const SkTSpan,
        span: *mut SkTSpan,
        opp: *mut SkTSect<'_>,
    ) {
        let mut bounded = (*span).fBounded;
        while !bounded.is_null() {
            let p = (*bounded).fBounded;
            let next = (*bounded).fNext;
            if p.cast_const() != keep && !(*p).fDeleted {
                let empty = (*span).removeBounded(p);

                if (*p).removeBounded(span) {
                    (*opp).removeSpan(p);
                }
            }
            bounded = next;
        }
    }
    pub(super) unsafe fn removeByPerpendicular(&mut self, opp: *mut SkTSect<'_>) -> bool {
        let mut test = self.fHead;
        loop {
            let next = (*test).fNext;
            if !((*test).fCoinStart.perpT() < 0.0 || (*test).fCoinEnd.perpT() < 0.0) {
                let s = (*test).fCoinStart.perpPt() - (*test).pointFirst();
                let e = (*test).fCoinEnd.perpPt() - (*test).pointLast();
                if !(s.dot(&e) <= 0.0) && !self.removeSpans(test, opp) {
                    return false;
                }
            }
            test = next;
            if test.is_null() {
                break;
            }
        }
        true
    }
    pub(super) unsafe fn removeCoincident(&mut self, span: *mut SkTSpan, isBetween: bool) -> bool {
        if !self.unlinkSpan(span) {
            return false;
        }
        if isBetween || between(0.0, (*span).fCoinStart.perpT(), 1.0) {
            self.fActiveCount -= 1;
            (*span).fNext = self.fCoincident;
            self.fCoincident = span;
        } else {
            self.markSpanGone(span);
        }
        true
    }
    pub(super) unsafe fn removedEndCheck(&mut self, span: *mut SkTSpan) {
        if (*span).fStartT == 0.0 {
            self.fRemovedStartT = true;
        }
        if (*span).fEndT == 1.0 {
            self.fRemovedEndT = true;
        }
    }
    pub(super) unsafe fn removeSpan(&mut self, span: *mut SkTSpan) -> bool {
        self.removedEndCheck(span);
        if !self.unlinkSpan(span) {
            return false;
        }
        self.markSpanGone(span)
    }
    pub(super) unsafe fn removeSpanRange(&mut self, first: *mut SkTSpan, last: *mut SkTSpan) {
        if first == last {
            return;
        }

        let final_ = (*last).fNext;
        let mut next = (*first).fNext;
        while !next.is_null() && next != final_ {
            let span = next;
            next = (*span).fNext;
            self.markSpanGone(span);
        }
        if !final_.is_null() {
            (*final_).fPrev = first;
        }
        (*first).fNext = final_;
    }
    pub(super) unsafe fn removeSpans(&mut self, span: *mut SkTSpan, opp: *mut SkTSect<'_>) -> bool {
        let mut bounded = (*span).fBounded;
        while !bounded.is_null() {
            let p = (*bounded).fBounded;
            let next = (*bounded).fNext;
            if (*span).removeBounded(p) {
                self.removeSpan(span);
            }
            if (*p).removeBounded(span) {
                (*opp).removeSpan(p);
            }
            if (*span).fDeleted && (*opp).hasBounded(span) {
                return false;
            }
            bounded = next;
        }
        true
    }
    pub(super) unsafe fn spanAtT(&self, t: f64, prior: &mut *mut SkTSpan) -> *mut SkTSpan {
        let mut test = self.fHead;
        let mut prev = null_mut();
        while !test.is_null() && (*test).fEndT < t {
            prev = test;
            test = (*test).fNext;
        }
        *prior = prev;
        if !test.is_null() && (*test).fStartT <= t {
            test
        } else {
            null_mut()
        }
    }
    pub(super) unsafe fn tail(&self) -> *mut SkTSpan {
        let mut result = self.fHead;
        let mut next = self.fHead;
        let mut safety = 100000;
        loop {
            next = (*next).fNext;
            if next.is_null() {
                break;
            }
            safety -= 1;
            if safety == 0 {
                return null_mut();
            }
            if (*next).fEndT > (*result).fEndT {
                result = next;
            }
        }
        result
    }
    pub(super) unsafe fn trim(&mut self, span: *mut SkTSpan, opp: *mut SkTSect<'_>) -> bool {
        if !(*span).initBounds(self.fCurve) {
            return false;
        }
        let mut bounded = (*span).fBounded;
        while !bounded.is_null() {
            let test = (*bounded).fBounded;
            let next = (*bounded).fNext;
            let mut oppSects = 0;
            let sects = self.intersects(span, opp, test, &mut oppSects);
            if sects >= 1 {
                if oppSects == 2 {
                    (*test).initBounds((*opp).fCurve);
                    (*opp).removeAllBut(span, test, self);
                }
                if sects == 2 {
                    (*span).initBounds(self.fCurve);
                    self.removeAllBut(test, span, opp);
                    return true;
                }
            } else {
                if (*span).removeBounded(test) {
                    self.removeSpan(span);
                }
                if (*test).removeBounded(span) {
                    (*opp).removeSpan(test);
                }
            }
            bounded = next;
        }
        true
    }
    pub(super) unsafe fn unlinkSpan(&mut self, span: *mut SkTSpan) -> bool {
        let prev = (*span).fPrev;
        let next = (*span).fNext;
        if !prev.is_null() {
            (*prev).fNext = next;
            if !next.is_null() {
                (*next).fPrev = prev;
                if (*next).fStartT > (*next).fEndT {
                    return false;
                }
            }
        } else {
            self.fHead = next;
            if !next.is_null() {
                (*next).fPrev = null_mut();
            }
        }
        true
    }
    pub(super) unsafe fn updateBounded(
        &mut self,
        first: *mut SkTSpan,
        last: *mut SkTSpan,
        oppFirst: *mut SkTSpan,
    ) -> bool {
        let mut test = first;
        let final_ = (*last).fNext;
        let mut delete = false;
        loop {
            delete |= (*test).removeAllBounded();
            test = (*test).fNext;
            if test == final_ || test.is_null() {
                break;
            }
        }
        (*first).fBounded = null_mut();
        (*first).addBounded(oppFirst, self.heap());
        delete
    }
}
fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}
fn cpp_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}
unsafe fn is_parallel(line: &SkDLine, opp: &dyn SkTCurve) -> bool {
    if !opp.IsConic() {
        return false;
    }
    let mut finds = 0;
    let mut perp = SkDLine {
        fPts: [
            SkDPoint {
                fX: line[1].fX + (line[1].fY - line[0].fY),
                fY: line[1].fY + (line[0].fX - line[1].fX),
            },
            line[1],
        ],
    };
    let mut i = SkIntersections::default();
    opp.intersectRay(&mut i, &perp);
    for n in 0..i.used() {
        finds += i.pt(n).approximatelyEqual(perp[1]) as i32;
    }
    perp[1] = SkDPoint {
        fX: line[0].fX + (line[1].fY - line[0].fY),
        fY: line[0].fY + (line[0].fX - line[1].fX),
    };
    perp[0] = line[0];
    opp.intersectRay(&mut i, &perp);
    for n in 0..i.used() {
        finds += i.pt(n).approximatelyEqual(perp[0]) as i32;
    }
    finds >= 2
}
