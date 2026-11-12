// Copyright 2013 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkOpCoincidence.h and SkOpCoincidence.cpp.
#![allow(non_snake_case)]
pub use super::SkCoincidentSpans::SkCoincidentSpans;
use super::{
    SkOpSegment::SkOpSegment,
    SkOpSpan::{SkOpPtT, SkOpSpan, SkOpSpanBase},
    SkPathOpsPoint::SkDPoint,
    SkPathOpsTypes::*,
};
use core::ptr::null_mut;
pub struct SkOpCoincidence {
    fHead: *mut SkCoincidentSpans,
    fTop: *mut SkCoincidentSpans,
    fGlobalState: *mut SkOpGlobalState,
    fContinue: bool,
    fSpanDeleted: bool,
    fPtAllocated: bool,
    fCoinExtended: bool,
    fSpanMerged: bool,
}
impl SkOpCoincidence {
    // Publish `this` after placement, never from the returning constructor.
    pub fn new(state: *mut SkOpGlobalState) -> Self {
        Self {
            fHead: null_mut(),
            fTop: null_mut(),
            fGlobalState: state,
            fContinue: false,
            fSpanDeleted: false,
            fPtAllocated: false,
            fCoinExtended: false,
            fSpanMerged: false,
        }
    }
    pub unsafe fn bind(&mut self) {
        (*self.fGlobalState).setCoincidence(self);
    }
    pub fn globalState(&self) -> *mut SkOpGlobalState {
        self.fGlobalState
    }
    pub fn isEmpty(&self) -> bool {
        self.fHead.is_null() && self.fTop.is_null()
    }
    pub unsafe fn Ordered(a: *const SkOpPtT, b: *const SkOpPtT) -> bool {
        Self::OrderedSegments((*a).segment(), (*b).segment())
    }
    pub unsafe fn extend(
        &mut self,
        mut cs: *const SkOpPtT,
        mut ce: *const SkOpPtT,
        mut os: *const SkOpPtT,
        mut oe: *const SkOpPtT,
    ) -> bool {
        let mut test = self.fHead;
        if test.is_null() {
            return false;
        }
        let mut seg = (*cs).segment();
        let mut opp = (*os).segment();
        if !Self::Ordered(cs, os) {
            core::mem::swap(&mut seg, &mut opp);
            core::mem::swap(&mut cs, &mut os);
            core::mem::swap(&mut ce, &mut oe);
            if (*cs).fT > (*ce).fT {
                core::mem::swap(&mut cs, &mut ce);
                core::mem::swap(&mut os, &mut oe);
            }
        }
        let omin = (*os).fT.min((*oe).fT);
        while !test.is_null() {
            if seg == (*(*test).coinPtTStart()).segment()
                && opp == (*(*test).oppPtTStart()).segment()
            {
                let tmin = (*(*test).oppPtTStart()).fT.min((*(*test).oppPtTEnd()).fT);
                let tmax = (*(*test).oppPtTStart()).fT.max((*(*test).oppPtTEnd()).fT);
                if ((*(*test).coinPtTStart()).fT <= (*ce).fT
                    && (*cs).fT <= (*(*test).coinPtTEnd()).fT)
                    || (tmin <= tmax && omin <= tmax)
                {
                    (*test).extend(cs, ce, os, oe);
                    return true;
                }
            }
            test = (*test).next();
        }
        false
    }
    pub unsafe fn add(
        &mut self,
        mut cs: *mut SkOpPtT,
        mut ce: *mut SkOpPtT,
        mut os: *mut SkOpPtT,
        mut oe: *mut SkOpPtT,
    ) {
        if !Self::Ordered(cs, os) {
            if (*os).fT < (*oe).fT {
                self.add(os, oe, cs, ce);
            } else {
                self.add(oe, os, ce, cs);
            }
            return;
        }
        cs = (*(*cs).span()).ptT();
        ce = (*(*ce).span()).ptT();
        os = (*(*os).span()).ptT();
        oe = (*(*oe).span()).ptT();

        let rec = (*(*self.globalState()).allocator()).make(SkCoincidentSpans::default());
        (*rec).init();
        (*rec).set(self.fHead, cs, ce, os, oe);
        self.fHead = rec;
    }
    unsafe fn addEndMovedSpansBase(
        &mut self,
        base: *const SkOpSpan,
        test: *const SkOpSpanBase,
    ) -> bool {
        let stop = (*test).ptT();
        let mut p = (*stop).next();
        let baseSeg = (*base).segment();
        let mut escape = 100000;
        while p != stop {
            escape -= 1;
            if escape <= 0 {
                return false;
            }
            let seg = (*p).segment();
            if !(*p).deleted()
                && seg != baseSeg
                && (*(*p).span()).ptT() == p
                && !self.containsSegments(baseSeg, seg, (*p).fT)
            {
                let slope = (*baseSeg).dSlopeAtT((*base).t());
                let point = *(*base).pt();
                let ray = super::SkPathOpsLine::SkDLine {
                    fPts: [
                        SkDPoint::from(point),
                        SkDPoint {
                            fX: point.fX as f64 + slope.fY,
                            fY: point.fY as f64 - slope.fX,
                        },
                    ],
                };
                let mut i = super::SkIntersections::SkIntersections::default();
                super::SkPathOpsCurve::CurveIntersectRay(
                    (*seg).verb(),
                    (*seg).pts(),
                    (*seg).weight(),
                    &ray,
                    &mut i,
                );
                for index in 0..i.used() {
                    let t = i[0][index];
                    if !between(0.0, t, 1.0) || !i.pt(index).approximatelyEqual(point) {
                        continue;
                    }
                    let oppStart = (*seg).addT(t);
                    if oppStart == p {
                        continue;
                    }
                    (*(*oppStart).span()).addOpp(base.cast_mut().cast());
                    if (*oppStart).deleted() {
                        continue;
                    }
                    let (mut coin, mut opp) = ((*base).segment(), (*oppStart).segment());
                    let (mut cs, mut ce, mut os, mut oe);
                    if Self::OrderedSegments(coin, opp) {
                        cs = (*base).t();
                        ce = (*test).t();
                        os = (*oppStart).fT;
                        oe = (*p).fT;
                    } else {
                        core::mem::swap(&mut coin, &mut opp);
                        cs = (*oppStart).fT;
                        ce = (*p).fT;
                        os = (*base).t();
                        oe = (*test).t();
                    }
                    if cs > ce {
                        core::mem::swap(&mut cs, &mut ce);
                        core::mem::swap(&mut os, &mut oe);
                    }
                    let mut added = false;
                    if !self.addOrOverlap(coin, opp, cs, ce, os, oe, &mut added) {
                        return false;
                    }
                }
            }
            p = (*p).next();
        }
        true
    }
    unsafe fn addEndMovedSpansPt(&mut self, p: *const SkOpPtT) -> bool {
        let base = (*(*p).span()).upCastable();
        if base.is_null() {
            return false;
        }
        let prev = (*base).prev();
        if prev.is_null() {
            return false;
        }
        if !(*prev).isCanceled() && !self.addEndMovedSpansBase(base, prev.cast()) {
            return false;
        }
        if !(*base).isCanceled() && !self.addEndMovedSpansBase(base, (*base).next()) {
            return false;
        }
        true
    }
    pub unsafe fn addEndMovedSpans(&mut self) -> bool {
        let mut span = self.fHead;
        if span.is_null() {
            return true;
        }
        self.fTop = span;
        self.fHead = null_mut();
        while !span.is_null() {
            let (cs, os) = ((*span).coinPtTStart(), (*span).oppPtTStart());
            if (*cs).fPt != (*os).fPt {
                if (*cs).fT == 1.0 {
                    return false;
                }
                let end = (*cs).fT == 0.0;
                let oend = zero_or_one((*os).fT);
                if end {
                    if !oend && !self.addEndMovedSpansPt(os) {
                        return false;
                    }
                } else if oend && !self.addEndMovedSpansPt(cs) {
                    return false;
                }
            }
            let (ce, oe) = ((*span).coinPtTEnd(), (*span).oppPtTEnd());
            if (*ce).fPt != (*oe).fPt {
                let end = (*ce).fT == 1.0;
                let oend = zero_or_one((*oe).fT);
                if end {
                    if !oend && !self.addEndMovedSpansPt(oe) {
                        return false;
                    }
                } else if oend && !self.addEndMovedSpansPt(ce) {
                    return false;
                }
            }
            span = (*span).next();
        }
        self.restoreHead();
        true
    }
    pub unsafe fn addExpanded(&mut self) -> bool {
        let mut coin = self.fHead;
        while !coin.is_null() {
            let sp = (*coin).coinPtTStart();
            let osp = (*coin).oppPtTStart();
            let mut prior = (*sp).fT;
            let mut oprior = (*osp).fT;
            if !(*sp).contains(osp) {
                return false;
            }
            let start = (*sp).span();
            let ostart = (*osp).span();
            let mut end = (*(*coin).coinPtTEnd()).span();
            let mut oend = (*(*coin).oppPtTEnd()).span();
            if (*oend).deleted() || (*start).upCastable().is_null() {
                return false;
            }
            let mut test = (*(*start).upCast()).next();
            if !(*coin).flipped() && (*ostart).upCastable().is_null() {
                return false;
            }
            let mut otest = if (*coin).flipped() {
                (*ostart).prev().cast()
            } else {
                (*(*ostart).upCast()).next()
            };
            if otest.is_null() {
                return false;
            }
            let seg = (*start).segment();
            let oseg = (*ostart).segment();
            while test != end || otest != oend {
                let containedOpp = (*(*test).ptT()).containsSegment(oseg);
                let containedThis = (*(*otest).ptT()).containsSegment(seg);
                if containedOpp.is_null() || containedThis.is_null() {
                    let (next, onext);
                    if !containedOpp.is_null() {
                        next = (*test).t();
                        onext = (*containedOpp).fT;
                    } else if !containedThis.is_null() {
                        next = (*containedThis).fT;
                        onext = (*otest).t();
                    } else {
                        let mut walk = test;
                        let found;
                        loop {
                            if (*walk).upCastable().is_null() {
                                return false;
                            }
                            walk = (*(*walk).upCast()).next();
                            let opposite = (*(*walk).ptT()).containsSegment(oseg);
                            if !opposite.is_null() {
                                found = opposite;
                                break;
                            }
                            if walk == (*(*coin).coinPtTEnd()).span() {
                                return false;
                            }
                        }
                        next = (*walk).t();
                        onext = (*found).fT;
                    }
                    let range = next - prior;
                    if range == 0.0 {
                        return false;
                    }
                    let part = ((*test).t() - prior) / range;
                    let orange = onext - oprior;
                    if orange == 0.0 {
                        return false;
                    }
                    let opart = ((*otest).t() - oprior) / orange;
                    if part == opart {
                        return false;
                    }
                    let addOpp = if containedOpp.is_null() && containedThis.is_null() {
                        part < opart
                    } else {
                        !containedThis.is_null()
                    };
                    let mut restart = false;
                    let success = if addOpp {
                        (*oseg).addExpanded(oprior + orange * part, test, &mut restart)
                    } else {
                        (*seg).addExpanded(prior + range * opart, otest, &mut restart)
                    };
                    if !success {
                        return false;
                    }
                    if restart {
                        test = start;
                        otest = ostart;
                    }
                    end = (*(*coin).coinPtTEnd()).span();
                    oend = (*(*coin).oppPtTEnd()).span();
                }
                if test != end {
                    if (*test).upCastable().is_null() {
                        return false;
                    }
                    prior = (*test).t();
                    test = (*(*test).upCast()).next();
                }
                if otest != oend {
                    oprior = (*otest).t();
                    if (*coin).flipped() {
                        otest = (*otest).prev().cast();
                    } else {
                        if (*otest).upCastable().is_null() {
                            return false;
                        }
                        otest = (*(*otest).upCast()).next();
                    }
                    if otest.is_null() {
                        return false;
                    }
                }
            }
            coin = (*coin).next();
        }
        true
    }
    unsafe fn TRange(over: *const SkOpPtT, t: f64, seg: *const SkOpSegment) -> f64 {
        let mut work = (*over).span();
        let (mut fs, mut fe, mut cs, mut ce): (
            *mut SkOpPtT,
            *mut SkOpPtT,
            *mut SkOpPtT,
            *mut SkOpPtT,
        ) = (null_mut(), null_mut(), null_mut(), null_mut());
        loop {
            let found = (*work).containsSegment(seg);
            if found.is_null() {
                if (*work).final_() {
                    break;
                }
            } else {
                if (*work).t() <= t {
                    cs = found;
                    fs = (*work).ptT();
                }
                if (*work).t() >= t {
                    ce = found;
                    fe = (*work).ptT();
                    break;
                }
            }
            work = (*(*work).upCast()).next();
            if work.is_null() {
                break;
            }
        }
        if cs.is_null() || ce.is_null() {
            return 1.0;
        }
        let denom = (*fe).fT - (*fs).fT;
        let ratio = if denom != 0.0 {
            (t - (*fs).fT) / denom
        } else {
            1.0
        };
        (*cs).fT + ((*ce).fT - (*cs).fT) * ratio
    }
}

impl SkOpCoincidence {
    unsafe fn checkOverlap(
        &self,
        mut check: *mut SkCoincidentSpans,
        coin: *const SkOpSegment,
        opp: *const SkOpSegment,
        cs: f64,
        ce: f64,
        mut os: f64,
        mut oe: f64,
        overlaps: &mut Vec<*mut SkCoincidentSpans>,
    ) -> bool {
        if !Self::OrderedSegments(coin, opp) {
            return if os < oe {
                self.checkOverlap(check, opp, coin, os, oe, cs, ce, overlaps)
            } else {
                self.checkOverlap(check, opp, coin, oe, os, ce, cs, overlaps)
            };
        }
        let swapped = os > oe;
        if swapped {
            core::mem::swap(&mut os, &mut oe);
        }
        while !check.is_null() {
            if (*(*check).coinPtTStart()).segment().cast_const() == coin
                && (*(*check).oppPtTStart()).segment().cast_const() == opp
            {
                let ts = (*(*check).coinPtTStart()).fT;
                let te = (*(*check).coinPtTEnd()).fT;
                let outside = ce < ts || cs > te;
                let (mut ots, mut ote) = ((*(*check).oppPtTStart()).fT, (*(*check).oppPtTEnd()).fT);
                if swapped {
                    if ots <= ote {
                        return false;
                    }
                    core::mem::swap(&mut ots, &mut ote);
                }
                let ooutside = oe < ots || os > ote;
                if !(outside && ooutside) {
                    if ce <= te && cs >= ts && oe <= ote && os >= ots {
                        return false;
                    }
                    overlaps.push(check);
                }
            }
            check = (*check).next();
        }
        true
    }
    unsafe fn addIfMissing(
        &mut self,
        over1: *const SkOpPtT,
        over2: *const SkOpPtT,
        ts: f64,
        te: f64,
        coin: *mut SkOpSegment,
        opp: *mut SkOpSegment,
        added: &mut bool,
    ) -> bool {
        let (mut cs, mut ce) = (Self::TRange(over1, ts, coin), Self::TRange(over1, te, coin));
        let result = (*coin).collapsed(cs, ce);
        if result != super::SkOpSpan::Collapsed::kNo {
            return result == super::SkOpSpan::Collapsed::kYes;
        }
        let (mut os, mut oe) = (Self::TRange(over2, ts, opp), Self::TRange(over2, te, opp));
        let result = (*opp).collapsed(os, oe);
        if result != super::SkOpSpan::Collapsed::kNo {
            return result == super::SkOpSpan::Collapsed::kYes;
        }
        if cs > ce {
            core::mem::swap(&mut cs, &mut ce);
            core::mem::swap(&mut os, &mut oe);
        }
        self.addOrOverlap(coin, opp, cs, ce, os, oe, added);
        true
    }
    unsafe fn addOrOverlap(
        &mut self,
        coin: *mut SkOpSegment,
        opp: *mut SkOpSegment,
        cts: f64,
        cte: f64,
        ots: f64,
        ote: f64,
        added: &mut bool,
    ) -> bool {
        let mut overlaps = Vec::new();
        if self.fTop.is_null() {
            return false;
        }
        if !self.checkOverlap(self.fTop, coin, opp, cts, cte, ots, ote, &mut overlaps) {
            return true;
        }
        if !self.fHead.is_null()
            && !self.checkOverlap(self.fHead, coin, opp, cts, cte, ots, ote, &mut overlaps)
        {
            return true;
        }
        let overlap = overlaps.first().copied().unwrap_or(null_mut());
        for &test in &overlaps[1.min(overlaps.len())..] {
            if (*(*overlap).coinPtTStart()).fT > (*(*test).coinPtTStart()).fT {
                (*overlap).setCoinPtTStart((*test).coinPtTStart());
            }
            if (*(*overlap).coinPtTEnd()).fT < (*(*test).coinPtTEnd()).fT {
                (*overlap).setCoinPtTEnd((*test).coinPtTEnd());
            }
            if if (*overlap).flipped() {
                (*(*overlap).oppPtTStart()).fT < (*(*test).oppPtTStart()).fT
            } else {
                (*(*overlap).oppPtTStart()).fT > (*(*test).oppPtTStart()).fT
            } {
                (*overlap).setOppPtTStart((*test).oppPtTStart());
            }
            if if (*overlap).flipped() {
                (*(*overlap).oppPtTEnd()).fT > (*(*test).oppPtTEnd()).fT
            } else {
                (*(*overlap).oppPtTEnd()).fT < (*(*test).oppPtTEnd()).fT
            } {
                (*overlap).setOppPtTEnd((*test).oppPtTEnd());
            }
            if self.fHead.is_null()
                || !Self::releaseRecord(core::ptr::addr_of_mut!(self.fHead), test)
            {
                let success = Self::releaseRecord(core::ptr::addr_of_mut!(self.fTop), test);
            }
        }
        let mut cs = (*coin).existing(cts, opp);
        let mut ce = (*coin).existing(cte, opp);
        if !overlap.is_null() && !cs.is_null() && !ce.is_null() && (*overlap).contains(cs, ce) {
            return true;
        }
        if !cs.is_null() && cs == ce {
            return false;
        }
        let mut os = (*opp).existing(ots, coin);
        let mut oe = (*opp).existing(ote, coin);
        if !overlap.is_null() && !os.is_null() && !oe.is_null() && (*overlap).contains(os, oe) {
            return true;
        }
        if (!cs.is_null() && (*cs).deleted())
            || (!os.is_null() && (*os).deleted())
            || (!ce.is_null() && (*ce).deleted())
            || (!oe.is_null() && (*oe).deleted())
        {
            return false;
        }
        let cse = if cs.is_null() {
            (*coin).existing(cts, core::ptr::null())
        } else {
            null_mut()
        };
        let cee = if ce.is_null() {
            (*coin).existing(cte, core::ptr::null())
        } else {
            null_mut()
        };
        if !cse.is_null() && cse == cee {
            return false;
        }
        if !cee.is_null() && (cee == cs || (*cee).contains(if !cse.is_null() { cse } else { cs })) {
            return false;
        }
        let ose = if os.is_null() {
            (*opp).existing(ots, core::ptr::null())
        } else {
            null_mut()
        };
        let oee = if oe.is_null() {
            (*opp).existing(ote, core::ptr::null())
        } else {
            null_mut()
        };
        if !ose.is_null() && ose == oee {
            return false;
        }
        if !ose.is_null() && (ose == oe || (*ose).contains(if !oee.is_null() { oee } else { oe })) {
            return false;
        }
        if !oee.is_null() && (oee == os || (*oee).contains(if !ose.is_null() { ose } else { os })) {
            return false;
        }
        if cs.is_null() || os.is_null() {
            let cw = if !cs.is_null() { cs } else { (*coin).addT(cts) };
            if cw == ce {
                return true;
            }
            let ow = if !os.is_null() { os } else { (*opp).addT(ots) };
            if cw.is_null() || ow.is_null() {
                return false;
            }
            (*(*cw).span()).addOpp((*ow).span());
            cs = cw;
            os = (*ow).active();
            if os.is_null() {
                return false;
            }
            if (!ce.is_null() && (*ce).deleted()) || (!oe.is_null() && (*oe).deleted()) {
                return false;
            }
        }
        if ce.is_null() || oe.is_null() {
            let cw = if !ce.is_null() { ce } else { (*coin).addT(cte) };
            let ow = if !oe.is_null() { oe } else { (*opp).addT(ote) };
            if !(*(*cw).span()).addOpp((*ow).span()) {
                return false;
            }
            ce = cw;
            oe = ow;
        }
        if (*cs).deleted()
            || (*os).deleted()
            || (*ce).deleted()
            || (*oe).deleted()
            || (*cs).contains(ce)
            || (*os).contains(oe)
        {
            return false;
        }
        let result;
        if !overlap.is_null() {
            if (*(*overlap).coinPtTStart()).segment() == coin {
                result = (*overlap).extend(cs, ce, os, oe);
            } else {
                if (*os).fT > (*oe).fT {
                    core::mem::swap(&mut cs, &mut ce);
                    core::mem::swap(&mut os, &mut oe);
                }
                result = (*overlap).extend(os, oe, cs, ce);
            }
        } else {
            self.add(cs, ce, os, oe);
            result = true;
        }
        if result {
            *added = true;
        }
        true
    }
    pub unsafe fn addMissing(&mut self, added: &mut bool) -> bool {
        let mut outer = self.fHead;
        *added = false;
        if outer.is_null() {
            return true;
        }
        self.fTop = outer;
        self.fHead = null_mut();
        while !outer.is_null() {
            let ocs = (*outer).coinPtTStart();
            if (*ocs).deleted() {
                return false;
            }
            let oc = (*ocs).segment();
            if (*oc).done() {
                return false;
            }
            let oos = (*outer).oppPtTStart();
            if (*oos).deleted() {
                return true;
            }
            let oo = (*oos).segment();
            let mut inner = (*outer).next();
            while !inner.is_null() {
                let ics = (*inner).coinPtTStart();
                if (*ics).deleted() {
                    return false;
                }
                let ic = (*ics).segment();
                if (*ic).done() {
                    return false;
                }
                let ios = (*inner).oppPtTStart();
                if (*ios).deleted() {
                    return false;
                }
                let io = (*ios).segment();
                let (mut s, mut e) = (0.0, 0.0);
                if oc == ic {
                    let oce = (*outer).coinPtTEnd();
                    if (*oce).deleted() {
                        return true;
                    }
                    let ice = (*inner).coinPtTEnd();
                    if (*ice).deleted() {
                        return false;
                    }
                    if oo != io
                        && self.overlap(ocs, oce, ics, ice, &mut s, &mut e)
                        && !self.addIfMissing(
                            (*ocs).starter(oce),
                            (*ics).starter(ice),
                            s,
                            e,
                            oo,
                            io,
                            added,
                        )
                    {
                        return false;
                    }
                } else if oc == io {
                    let oce = (*outer).coinPtTEnd();
                    if (*oce).deleted() {
                        return false;
                    }
                    let ioe = (*inner).oppPtTEnd();
                    if (*ioe).deleted() {
                        return false;
                    }
                    if oo != ic
                        && self.overlap(ocs, oce, ios, ioe, &mut s, &mut e)
                        && !self.addIfMissing(
                            (*ocs).starter(oce),
                            (*ios).starter(ioe),
                            s,
                            e,
                            oo,
                            ic,
                            added,
                        )
                    {
                        return false;
                    }
                } else if oo == ic {
                    let ooe = (*outer).oppPtTEnd();
                    if (*ooe).deleted() {
                        return false;
                    }
                    let ice = (*inner).coinPtTEnd();
                    if (*ice).deleted() {
                        return false;
                    }
                    if self.overlap(oos, ooe, ics, ice, &mut s, &mut e)
                        && !self.addIfMissing(
                            (*oos).starter(ooe),
                            (*ics).starter(ice),
                            s,
                            e,
                            oc,
                            io,
                            added,
                        )
                    {
                        return false;
                    }
                } else if oo == io {
                    let ooe = (*outer).oppPtTEnd();
                    if (*ooe).deleted() {
                        return false;
                    }
                    let ioe = (*inner).oppPtTEnd();
                    if (*ioe).deleted() {
                        return true;
                    }
                    if self.overlap(oos, ooe, ios, ioe, &mut s, &mut e)
                        && !self.addIfMissing(
                            (*oos).starter(ooe),
                            (*ios).starter(ioe),
                            s,
                            e,
                            oc,
                            ic,
                            added,
                        )
                    {
                        return false;
                    }
                }
                inner = (*inner).next();
            }
            outer = (*outer).next();
        }
        self.restoreHead();
        true
    }
    unsafe fn addOverlap(
        &mut self,
        s1: *const SkOpSegment,
        s1o: *const SkOpSegment,
        s2: *const SkOpSegment,
        s2o: *const SkOpSegment,
        overS: *const SkOpPtT,
        overE: *const SkOpPtT,
    ) -> bool {
        let (mut a, mut b) = ((*overS).find(s1), (*overE).find(s1));
        if a.is_null() || b.is_null() {
            return false;
        }
        if (*(*(*a).starter(b)).span())
            .upCast()
            .as_ref()
            .unwrap()
            .windValue()
            == 0
        {
            a = (*overS).find(s1o);
            b = (*overE).find(s1o);
            if a.is_null() || b.is_null() {
                return false;
            }
            if (*(*(*a).starter(b)).span())
                .upCast()
                .as_ref()
                .unwrap()
                .windValue()
                == 0
            {
                return true;
            }
        }
        let (mut c, mut d) = ((*overS).find(s2), (*overE).find(s2));
        if c.is_null() || d.is_null() {
            return false;
        }
        if (*(*(*c).starter(d)).span())
            .upCast()
            .as_ref()
            .unwrap()
            .windValue()
            == 0
        {
            c = (*overS).find(s2o);
            d = (*overE).find(s2o);
            if c.is_null() || d.is_null() {
                return false;
            }
            if (*(*(*c).starter(d)).span())
                .upCast()
                .as_ref()
                .unwrap()
                .windValue()
                == 0
            {
                return true;
            }
        }
        if (*a).segment() == (*c).segment() {
            return true;
        }
        if (*a).fT > (*b).fT {
            core::mem::swap(&mut a, &mut b);
            core::mem::swap(&mut c, &mut d);
        }
        self.add(a, b, c, d);
        true
    }
    unsafe fn containsList(
        &self,
        mut coin: *mut SkCoincidentSpans,
        seg: *const SkOpSegment,
        opp: *const SkOpSegment,
        t: f64,
    ) -> bool {
        while !coin.is_null() {
            if (*(*coin).coinPtTStart()).segment().cast_const() == seg
                && (*(*coin).oppPtTStart()).segment().cast_const() == opp
                && between((*(*coin).oppPtTStart()).fT, t, (*(*coin).oppPtTEnd()).fT)
            {
                return true;
            }
            if (*(*coin).oppPtTStart()).segment().cast_const() == seg
                && (*(*coin).coinPtTStart()).segment().cast_const() == opp
                && between((*(*coin).coinPtTStart()).fT, t, (*(*coin).coinPtTEnd()).fT)
            {
                return true;
            }
            coin = (*coin).next();
        }
        false
    }
    unsafe fn containsSegments(
        &self,
        seg: *const SkOpSegment,
        opp: *const SkOpSegment,
        t: f64,
    ) -> bool {
        self.containsList(self.fHead, seg, opp, t) || self.containsList(self.fTop, seg, opp, t)
    }
    pub unsafe fn contains(
        &self,
        mut cs: *const SkOpPtT,
        mut ce: *const SkOpPtT,
        mut os: *const SkOpPtT,
        mut oe: *const SkOpPtT,
    ) -> bool {
        let mut test = self.fHead;
        if test.is_null() {
            return false;
        }
        let (mut coin, mut opp) = ((*cs).segment(), (*os).segment());
        if !Self::Ordered(cs, os) {
            core::mem::swap(&mut coin, &mut opp);
            core::mem::swap(&mut cs, &mut os);
            core::mem::swap(&mut ce, &mut oe);
            if (*cs).fT > (*ce).fT {
                core::mem::swap(&mut cs, &mut ce);
                core::mem::swap(&mut os, &mut oe);
            }
        }
        let min = (*os).fT.min((*oe).fT);
        let max = (*os).fT.max((*oe).fT);
        while !test.is_null() {
            if coin == (*(*test).coinPtTStart()).segment()
                && (*cs).fT >= (*(*test).coinPtTStart()).fT
                && (*ce).fT <= (*(*test).coinPtTEnd()).fT
                && opp == (*(*test).oppPtTStart()).segment()
                && min >= (*(*test).oppPtTStart()).fT.min((*(*test).oppPtTEnd()).fT)
                && max <= (*(*test).oppPtTStart()).fT.max((*(*test).oppPtTEnd()).fT)
            {
                return true;
            }
            test = (*test).next();
        }
        false
    }
    pub unsafe fn correctEnds(&mut self) {
        let mut coin = self.fHead;
        while !coin.is_null() {
            (*coin).correctEnds();
            coin = (*coin).next();
        }
    }
}

impl SkOpCoincidence {
    pub unsafe fn apply(&mut self) -> bool {
        let mut coin = self.fHead;
        while !coin.is_null() {
            let base = (*(*coin).coinPtTStartWritable()).span();
            if (*base).upCastable().is_null() {
                return false;
            }
            let mut start = (*base).upCast();
            if !(*start).deleted() {
                let end = (*(*coin).coinPtTEnd()).span();
                if start != (*start).starter(end) {
                    return false;
                }
                let flipped = (*coin).flipped();
                let obase = (*if flipped {
                    (*coin).oppPtTEndWritable()
                } else {
                    (*coin).oppPtTStartWritable()
                })
                .span();
                if (*obase).upCastable().is_null() {
                    return false;
                }
                let mut ostart = (*obase).upCast();
                if !(*ostart).deleted() {
                    let oend = (*if flipped {
                        (*coin).oppPtTStart()
                    } else {
                        (*coin).oppPtTEnd()
                    })
                    .span();
                    let seg = (*start).segment();
                    let oseg = (*ostart).segment();
                    let swapped = (*seg).operand() != (*oseg).operand();
                    if flipped {
                        if (*oend).deleted() {
                            coin = (*coin).next();
                            continue;
                        }
                        loop {
                            let next = (*ostart).next();
                            if next == oend {
                                break;
                            }
                            if (*next).upCastable().is_null() {
                                return false;
                            }
                            ostart = (*next).upCast();
                        }
                    }
                    loop {
                        let (mut w, mut o, mut ow, mut oo) = (
                            (*start).windValue(),
                            (*start).oppValue(),
                            (*ostart).windValue(),
                            (*ostart).oppValue(),
                        );
                        let mut wd = if swapped { oo } else { ow };
                        let mut owd = if swapped { o } else { w };
                        if !flipped {
                            wd = -wd;
                            owd = -owd;
                        }
                        let mut toStart = w != 0 && (w > wd || (w == wd && ow <= owd));
                        if if toStart {
                            (*start).done()
                        } else {
                            (*ostart).done()
                        } {
                            toStart ^= true;
                        }
                        if toStart {
                            if swapped {
                                core::mem::swap(&mut ow, &mut oo);
                            }
                            if flipped {
                                w -= ow;
                                o -= oo;
                            } else {
                                w += ow;
                                o += oo;
                            }
                            if (*seg).isXor() {
                                w &= 1;
                            }
                            if (*seg).oppXor() {
                                o &= 1;
                            }
                            ow = 0;
                            oo = 0;
                        } else {
                            if swapped {
                                core::mem::swap(&mut w, &mut o);
                            }
                            if flipped {
                                ow -= w;
                                oo -= o;
                            } else {
                                ow += w;
                                oo += o;
                            }
                            if (*oseg).isXor() {
                                ow &= 1;
                            }
                            if (*oseg).oppXor() {
                                oo &= 1;
                            }
                            w = 0;
                            o = 0;
                        }
                        if w <= -1 {
                            return false;
                        }
                        (*start).setWindValue(w);
                        (*start).setOppValue(o);
                        if ow <= -1 {
                            return false;
                        }
                        (*ostart).setWindValue(ow);
                        (*ostart).setOppValue(oo);
                        if w == 0 && o == 0 {
                            (*seg).markDone(start);
                        }
                        if ow == 0 && oo == 0 {
                            (*oseg).markDone(ostart);
                        }
                        let next = (*start).next();
                        let mut onext = if flipped {
                            (*ostart).prev().cast()
                        } else {
                            (*ostart).next()
                        };
                        if next == end {
                            break;
                        }
                        if (*next).upCastable().is_null() {
                            return false;
                        }
                        start = (*next).upCast();
                        if onext.is_null() || (*onext).upCastable().is_null() {
                            onext = ostart.cast();
                        }
                        ostart = (*onext).upCast();
                    }
                }
            }
            coin = (*coin).next();
        }
        true
    }
    unsafe fn releaseRecord(
        mut head: *mut *mut SkCoincidentSpans,
        remove: *mut SkCoincidentSpans,
    ) -> bool {
        while !(*head).is_null() {
            let coin = *head;
            if coin == remove {
                *head = (*coin).next();
                return true;
            }
            head = (*coin).nextPtr();
        }
        false
    }
    unsafe fn releaseDeletedList(mut head: *mut *mut SkCoincidentSpans) {
        while !(*head).is_null() {
            let coin = *head;
            if (*(*coin).coinPtTStart()).deleted() {
                *head = (*coin).next();
            } else {
                head = (*coin).nextPtr();
            }
        }
    }
    pub unsafe fn releaseDeleted(&mut self) {
        Self::releaseDeletedList(core::ptr::addr_of_mut!(self.fHead));
        Self::releaseDeletedList(core::ptr::addr_of_mut!(self.fTop));
    }
    unsafe fn restoreHead(&mut self) {
        let mut head = core::ptr::addr_of_mut!(self.fHead);
        while !(*head).is_null() {
            head = (**head).nextPtr();
        }
        *head = self.fTop;
        self.fTop = null_mut();
        head = core::ptr::addr_of_mut!(self.fHead);
        while !(*head).is_null() {
            let test = *head;
            if (*(*(*test).coinPtTStart()).segment()).done()
                || (*(*(*test).oppPtTStart()).segment()).done()
            {
                *head = (*test).next();
            } else {
                head = (*test).nextPtr();
            }
        }
    }
    pub unsafe fn expand(&mut self) -> bool {
        let mut coin = self.fHead;
        if coin.is_null() {
            return false;
        }
        let mut expanded = false;
        while !coin.is_null() {
            if (*coin).expand() {
                let mut test = self.fHead;
                while !test.is_null() {
                    if coin != test
                        && (*coin).coinPtTStart() == (*test).coinPtTStart()
                        && (*coin).oppPtTStart() == (*test).oppPtTStart()
                    {
                        Self::releaseRecord(core::ptr::addr_of_mut!(self.fHead), test);
                        break;
                    }
                    test = (*test).next();
                }
                expanded = true;
            }
            coin = (*coin).next();
        }
        expanded
    }
    pub unsafe fn FindOverlaps(source: *const Self, overlaps: *mut Self) -> bool {
        (*overlaps).fHead = null_mut();
        (*overlaps).fTop = null_mut();
        let mut outer = (*source).fHead;
        while !outer.is_null() {
            let oc = (*(*outer).coinPtTStart()).segment();
            let oo = (*(*outer).oppPtTStart()).segment();
            let mut inner = (*outer).next();
            while !inner.is_null() {
                let ic = (*(*inner).coinPtTStart()).segment();
                if oc != ic {
                    let io = (*(*inner).oppPtTStart()).segment();
                    let (mut s, mut e) = (core::ptr::null(), core::ptr::null());
                    if (oo == ic
                        && SkOpPtT::Overlaps(
                            (*outer).oppPtTStart(),
                            (*outer).oppPtTEnd(),
                            (*inner).coinPtTStart(),
                            (*inner).coinPtTEnd(),
                            &mut s,
                            &mut e,
                        ))
                        || (oc == io
                            && SkOpPtT::Overlaps(
                                (*outer).coinPtTStart(),
                                (*outer).coinPtTEnd(),
                                (*inner).oppPtTStart(),
                                (*inner).oppPtTEnd(),
                                &mut s,
                                &mut e,
                            ))
                        || (oo == io
                            && SkOpPtT::Overlaps(
                                (*outer).oppPtTStart(),
                                (*outer).oppPtTEnd(),
                                (*inner).oppPtTStart(),
                                (*inner).oppPtTEnd(),
                                &mut s,
                                &mut e,
                            ))
                    {
                        if !(*overlaps).addOverlap(oc, oo, ic, io, s, e) {
                            return false;
                        }
                    }
                }
                inner = (*inner).next();
            }
            outer = (*outer).next();
        }
        true
    }
    pub unsafe fn fixUp(&mut self, deleted: *mut SkOpPtT, kept: *const SkOpPtT) {
        if !self.fHead.is_null() {
            Self::fixUpList(core::ptr::addr_of_mut!(self.fHead), deleted, kept);
        }
        if !self.fTop.is_null() {
            Self::fixUpList(core::ptr::addr_of_mut!(self.fTop), deleted, kept);
        }
    }
    unsafe fn fixUpList(
        mut head: *mut *mut SkCoincidentSpans,
        deleted: *mut SkOpPtT,
        kept: *const SkOpPtT,
    ) {
        while !(*head).is_null() {
            let coin = *head;
            if (*coin).coinPtTStart() == deleted {
                if (*(*coin).coinPtTEnd()).span() == (*kept).span() {
                    *head = (*coin).next();
                    continue;
                }
                (*coin).setCoinPtTStart(kept);
            }
            if (*coin).coinPtTEnd() == deleted {
                if (*(*coin).coinPtTStart()).span() == (*kept).span() {
                    *head = (*coin).next();
                    continue;
                }
                (*coin).setCoinPtTEnd(kept);
            }
            if (*coin).oppPtTStart() == deleted {
                if (*(*coin).oppPtTEnd()).span() == (*kept).span() {
                    *head = (*coin).next();
                    continue;
                }
                (*coin).setOppPtTStart(kept);
            }
            if (*coin).oppPtTEnd() == deleted {
                if (*(*coin).oppPtTStart()).span() == (*kept).span() {
                    *head = (*coin).next();
                    continue;
                }
                (*coin).setOppPtTEnd(kept);
            }
            head = (*coin).nextPtr();
        }
    }
    pub unsafe fn mark(&mut self) -> bool {
        let mut coin = self.fHead;
        while !coin.is_null() {
            let base = (*(*coin).coinPtTStartWritable()).span();
            if (*base).upCastable().is_null() {
                return false;
            }
            let start = (*base).upCast();
            if (*start).deleted() {
                return false;
            }
            let end = (*(*coin).coinPtTEndWritable()).span();
            let mut os = (*(*coin).oppPtTStartWritable()).span();
            let mut oe = (*(*coin).oppPtTEndWritable()).span();
            if (*oe).deleted() {
                return false;
            }
            let flipped = (*coin).flipped();
            if flipped {
                core::mem::swap(&mut os, &mut oe);
            }
            if (*os).upCastable().is_null() {
                return false;
            }
            (*start).insertCoincidence((*os).upCast());
            (*end).insertCoinEnd(oe);
            let seg = (*start).segment();
            let oseg = (*os).segment();
            let mut next = start.cast::<SkOpSpanBase>();
            let mut onext = os;
            let mut ordered = false;
            if !(*coin).ordered(&mut ordered) {
                return false;
            }
            loop {
                next = (*(*next).upCast()).next();
                if next == end {
                    break;
                }
                if (*next).upCastable().is_null() {
                    return false;
                }
                if !(*(*next).upCast()).insertCoincidenceSegment(oseg, flipped, ordered) {
                    return false;
                }
            }
            loop {
                onext = (*(*onext).upCast()).next();
                if onext == oe {
                    break;
                }
                if (*onext).upCastable().is_null() {
                    return false;
                }
                if !(*(*onext).upCast()).insertCoincidenceSegment(seg, flipped, ordered) {
                    return false;
                }
            }
            coin = (*coin).next();
        }
        true
    }
    unsafe fn markCollapsedList(mut head: *mut *mut SkCoincidentSpans, test: *mut SkOpPtT) {
        while !(*head).is_null() {
            let coin = *head;
            if (*coin).collapsed(test) {
                if zero_or_one((*(*coin).coinPtTStart()).fT)
                    && zero_or_one((*(*coin).coinPtTEnd()).fT)
                {
                    (*(*(*coin).coinPtTStartWritable()).segment()).markAllDone();
                }
                if zero_or_one((*(*coin).oppPtTStart()).fT)
                    && zero_or_one((*(*coin).oppPtTEnd()).fT)
                {
                    (*(*(*coin).oppPtTStartWritable()).segment()).markAllDone();
                }
                *head = (*coin).next();
            } else {
                head = (*coin).nextPtr();
            }
        }
    }
    pub unsafe fn markCollapsed(&mut self, test: *mut SkOpPtT) {
        Self::markCollapsedList(core::ptr::addr_of_mut!(self.fHead), test);
        Self::markCollapsedList(core::ptr::addr_of_mut!(self.fTop), test);
    }
    pub unsafe fn OrderedSegments(coin: *const SkOpSegment, opp: *const SkOpSegment) -> bool {
        if ((*coin).verb() as u32) < (*opp).verb() as u32 {
            return true;
        }
        if ((*coin).verb() as u32) > (*opp).verb() as u32 {
            return false;
        }
        let count = ((SkPathOpsVerbToPoints((*coin).verb()) + 1) * 2) as usize;
        let cp = (*coin).pts().cast::<f32>();
        let op = (*opp).pts().cast::<f32>();
        for i in 0..count {
            if *cp.add(i) < *op.add(i) {
                return true;
            }
            if *cp.add(i) > *op.add(i) {
                return false;
            }
        }
        true
    }
    unsafe fn overlap(
        &self,
        a: *const SkOpPtT,
        b: *const SkOpPtT,
        c: *const SkOpPtT,
        d: *const SkOpPtT,
        s: &mut f64,
        e: &mut f64,
    ) -> bool {
        *s = (*a).fT.min((*b).fT).max((*c).fT.min((*d).fT));
        *e = (*a).fT.max((*b).fT).min((*c).fT.max((*d).fT));
        *s < *e
    }
    unsafe fn releaseSegment(mut head: *mut *mut SkCoincidentSpans, deleted: *const SkOpSegment) {
        while !(*head).is_null() {
            let coin = *head;
            if (*(*coin).coinPtTStart()).segment().cast_const() == deleted
                || (*(*coin).coinPtTEnd()).segment().cast_const() == deleted
                || (*(*coin).oppPtTStart()).segment().cast_const() == deleted
                || (*(*coin).oppPtTEnd()).segment().cast_const() == deleted
            {
                *head = (*coin).next();
            } else {
                head = (*coin).nextPtr();
            }
        }
    }
    pub unsafe fn release(&mut self, deleted: *const SkOpSegment) {
        Self::releaseSegment(core::ptr::addr_of_mut!(self.fHead), deleted);
        Self::releaseSegment(core::ptr::addr_of_mut!(self.fTop), deleted);
    }
}
