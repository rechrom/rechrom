// Copyright 2015 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsWinding.cpp. Native ray-based winding discovery.
use super::{
    SkOpContour::{SkOpContour, SkOpContourHead},
    SkOpSegment::SkOpSegment,
    SkOpSpan::{SkOpSpan, SkOpSpanBase},
    SkPathOpsCurve::curve_intercept,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsTypes::*,
};
use crate::src::core::{
    SkArenaAlloc::{SkArenaAlloc, SkSTArenaAlloc},
    SkPath::Verb,
    SkPoint::SkPoint,
    SkRect::SkRect,
    SkTSort::SkTQSort,
};
use core::ptr::null_mut;
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum SkOpRayDir {
    kLeft,
    kTop,
    kRight,
    kBottom,
}
fn xy_index(dir: SkOpRayDir) -> usize {
    dir as usize & 1
}
fn pt_xy(pt: &SkPoint, dir: SkOpRayDir) -> f32 {
    if xy_index(dir) == 0 {
        pt.fX
    } else {
        pt.fY
    }
}
fn pt_yx(pt: &SkPoint, dir: SkOpRayDir) -> f32 {
    if xy_index(dir) == 0 {
        pt.fY
    } else {
        pt.fX
    }
}
fn pt_dxdy(pt: &SkDVector, dir: SkOpRayDir) -> f64 {
    if xy_index(dir) == 0 {
        pt.fX
    } else {
        pt.fY
    }
}
fn pt_dydx(pt: &SkDVector, dir: SkOpRayDir) -> f64 {
    if xy_index(dir) == 0 {
        pt.fY
    } else {
        pt.fX
    }
}
fn rect_side(r: &SkRect, dir: SkOpRayDir) -> f32 {
    match dir {
        SkOpRayDir::kLeft => r.fLeft,
        SkOpRayDir::kTop => r.fTop,
        SkOpRayDir::kRight => r.fRight,
        SkOpRayDir::kBottom => r.fBottom,
    }
}
fn sideways_overlap(r: &SkRect, p: &SkPoint, d: SkOpRayDir) -> bool {
    if xy_index(d) == 0 {
        approximately_between(r.fTop as f64, p.fY as f64, r.fBottom as f64)
    } else {
        approximately_between(r.fLeft as f64, p.fX as f64, r.fRight as f64)
    }
}
fn less_than(d: SkOpRayDir) -> bool {
    d as usize & 2 == 0
}
fn ccw_dxdy(v: &SkDVector, d: SkOpRayDir) -> bool {
    (pt_dydx(v, d) > 0.0) == ((d as usize + 1) & 2 != 0)
}
pub struct SkOpRayHit {
    pub fNext: *mut Self,
    pub fSpan: *mut SkOpSpan,
    pub fPt: SkPoint,
    pub fT: f64,
    pub fSlope: SkDVector,
    pub fValid: bool,
}
impl Default for SkOpRayHit {
    fn default() -> Self {
        Self {
            fNext: null_mut(),
            fSpan: null_mut(),
            fPt: SkPoint::default(),
            fT: 0.0,
            fSlope: SkDVector::default(),
            fValid: false,
        }
    }
}
impl SkOpRayHit {
    unsafe fn makeTestBase(&mut self, span: *mut SkOpSpan, t: f64) -> SkOpRayDir {
        self.fNext = null_mut();
        self.fSpan = span;
        self.fT = (*span).t() * (1.0 - t) + (*(*span).next()).t() * t;
        let seg = (*span).segment();
        self.fSlope = (*seg).dSlopeAtT(self.fT);
        self.fPt = (*seg).ptAtT(self.fT);
        self.fValid = true;
        if self.fSlope.fX.abs() < self.fSlope.fY.abs() {
            SkOpRayDir::kLeft
        } else {
            SkOpRayDir::kTop
        }
    }
}
impl SkOpContour {
    pub unsafe fn rayCheck(
        &self,
        base: &SkOpRayHit,
        dir: SkOpRayDir,
        hits: &mut *mut SkOpRayHit,
        allocator: *mut SkArenaAlloc,
    ) {
        let baseXY = pt_xy(&base.fPt, dir);
        let boundsXY = rect_side(&self.fBounds, dir);
        let less = less_than(dir);
        if !approximately_equal(baseXY as f64, boundsXY as f64) && (baseXY < boundsXY) == less {
            return;
        }
        let mut seg = self.first();
        loop {
            (*seg).rayCheck(base, dir, hits, allocator);
            seg = (*seg).next();
            if seg.is_null() {
                break;
            }
        }
    }
    pub unsafe fn findSortableTop(&mut self, head: *mut SkOpContour) -> *mut SkOpSpan {
        let mut allDone = true;
        if self.fCount != 0 {
            let mut segment = self.first();
            loop {
                if !(*segment).done() {
                    allDone = false;
                    let result = (*segment).findSortableTop(head);
                    if !result.is_null() {
                        return result;
                    }
                }
                segment = (*segment).next();
                if segment.is_null() {
                    break;
                }
            }
        }
        if allDone {
            self.fDone = true;
        }
        null_mut()
    }
}
impl SkOpSegment {
    pub unsafe fn rayCheck(
        &self,
        base: &SkOpRayHit,
        dir: SkOpRayDir,
        hits: &mut *mut SkOpRayHit,
        allocator: *mut SkArenaAlloc,
    ) {
        if !sideways_overlap(&self.fBounds, &base.fPt, dir) {
            return;
        }
        let baseXY = pt_xy(&base.fPt, dir);
        let boundsXY = rect_side(&self.fBounds, dir);
        let less = less_than(dir);
        if !approximately_equal(baseXY as f64, boundsXY as f64) && (baseXY < boundsXY) == less {
            return;
        }
        let mut tVals = [0.0; 3];
        let baseYX = pt_yx(&base.fPt, dir);
        let roots = curve_intercept(
            self.fVerb,
            xy_index(dir),
            self.fPts,
            self.fWeight,
            baseYX,
            &mut tVals,
        );
        for i in 0..roots {
            let t = tVals[i];
            if (*base.fSpan).segment() == self as *const Self as *mut Self
                && approximately_equal(base.fT, t)
            {
                continue;
            }
            let mut slope = SkDVector::default();
            let pt;
            let mut valid = false;
            if approximately_zero(t) {
                pt = *self.fPts;
            } else if approximately_equal(t, 1.0) {
                pt = *self.fPts.add(SkPathOpsVerbToPoints(self.fVerb) as usize);
            } else {
                pt = self.ptAtT(t);
                if SkDPoint::ApproximatelyEqual(pt, base.fPt) {
                    if (*base.fSpan).segment() == self as *const Self as *mut Self {
                        continue;
                    }
                } else {
                    let xy = pt_xy(&pt, dir);
                    if !approximately_equal(baseXY as f64, xy as f64) && (baseXY < xy) == less {
                        continue;
                    }
                    slope = self.dSlopeAtT(t);
                    if self.fVerb == Verb::kCubic_Verb
                        && (*base.fSpan).segment() == self as *const Self as *mut Self
                        && roughly_equal(base.fT, t)
                        && SkDPoint::RoughlyEqual(&pt, &base.fPt)
                    {
                        continue;
                    }
                    if (pt_dydx(&slope, dir) * 10000.0).abs() > pt_dxdy(&slope, dir).abs() {
                        valid = true;
                    }
                }
            }
            let span = self.windingSpanAtT(t);
            if span.is_null() {
                valid = false;
            } else if (*span).windValue() == 0 && (*span).oppValue() == 0 {
                continue;
            }
            let hit = (*allocator).make(SkOpRayHit {
                fNext: *hits,
                fSpan: span,
                fPt: pt,
                fT: t,
                fSlope: slope,
                fValid: valid,
            });
            *hits = hit;
        }
    }
    pub unsafe fn windingSpanAtT(&self, t: f64) -> *mut SkOpSpan {
        let mut span = self.head();
        loop {
            let next = (*span).next();
            if approximately_equal(t, (*next).t()) {
                return null_mut();
            }
            if t < (*next).t() {
                return span;
            }
            if (*next).final_() {
                return null_mut();
            }
            span = (*next).upCast();
        }
    }
    pub unsafe fn findSortableTop(&mut self, head: *mut SkOpContour) -> *mut SkOpSpan {
        let mut span = self.head();
        loop {
            let next = (*span).next();
            if !(*span).done() && ((*span).windSum() != i32::MIN || (*span).sortableTop(head)) {
                return span;
            }
            if (*next).final_() {
                return null_mut();
            }
            span = (*next).upCast();
        }
    }
}
fn get_t_guess(mut tTry: i32, dirOffset: &mut usize) -> f64 {
    let mut t = 0.5;
    *dirOffset = (tTry & 1) as usize;
    let tBase = tTry >> 1;
    let mut tBits = 0;
    loop {
        tTry >>= 1;
        if tTry == 0 {
            break;
        }
        t /= 2.0;
        tBits += 1;
    }
    if tBits != 0 {
        let index = (tBase - 1) & ((1 << tBits) - 1);
        t += t * 2.0 * index as f64;
    }
    t
}
impl SkOpSpan {
    pub unsafe fn sortableTop(&mut self, contourHead: *mut SkOpContour) -> bool {
        let mut allocator = core::pin::pin!(SkSTArenaAlloc::<1024>::new(0));
        let mut dirOffset = 0;
        let t = get_t_guess(self.fTopTTry, &mut dirOffset);
        self.fTopTTry += 1;
        let mut base = SkOpRayHit::default();
        let dir = base.makeTestBase(self, t);
        if base.fSlope.fX == 0.0 && base.fSlope.fY == 0.0 {
            return false;
        }
        let mut hitHead = &mut base as *mut SkOpRayHit;
        let dir = match dir as usize + dirOffset {
            0 => SkOpRayDir::kLeft,
            1 => SkOpRayDir::kTop,
            2 => SkOpRayDir::kRight,
            _ => SkOpRayDir::kBottom,
        };
        if !base.fSpan.is_null()
            && ((*(*base.fSpan).segment()).verb() as u32) > (Verb::kLine_Verb as u32)
            && pt_dydx(&base.fSlope, dir) == 0.0
        {
            return false;
        }
        let mut contour = contourHead;
        loop {
            if (*contour).count() != 0 {
                (*contour).rayCheck(&base, dir, &mut hitHead, allocator.as_mut().arena());
            }
            contour = (*contour).next();
            if contour.is_null() {
                break;
            }
        }
        let mut sorted = Vec::new();
        let mut hit = hitHead;
        while !hit.is_null() {
            sorted.push(hit);
            hit = (*hit).fNext;
        }
        SkTQSort(&mut sorted, &|a, b| {
            let a = pt_xy(&(**a).fPt, dir);
            let b = pt_xy(&(**b).fPt, dir);
            if less_than(dir) {
                a < b
            } else {
                b < a
            }
        });
        let mut last: Option<SkPoint> = None;
        let mut wind = 0;
        let mut oppWind = 0;
        for index in 0..sorted.len() {
            hit = sorted[index];
            if !(*hit).fValid {
                return false;
            }
            let ccw = ccw_dxdy(&(*hit).fSlope, dir);
            let span = (*hit).fSpan;
            if span.is_null() {
                return false;
            }
            let seg = (*span).segment();
            if (*span).windValue() == 0 && (*span).oppValue() == 0 {
                continue;
            }
            if let Some(last) = last {
                if SkDPoint::ApproximatelyEqual(last, (*hit).fPt) {
                    return false;
                }
            }
            if index + 1 < sorted.len()
                && SkDPoint::ApproximatelyEqual((*sorted[index + 1]).fPt, (*hit).fPt)
            {
                return false;
            }
            let operand = (*seg).operand();
            if operand {
                core::mem::swap(&mut wind, &mut oppWind);
            }
            let lastWind = wind;
            let lastOpp = oppWind;
            let windValue = if ccw {
                -(*span).windValue()
            } else {
                (*span).windValue()
            };
            let oppValue = if ccw {
                -(*span).oppValue()
            } else {
                (*span).oppValue()
            };
            wind += windValue;
            oppWind += oppValue;
            let windSum = if SkOpSegment::UseInnerWinding(lastWind, wind) {
                wind
            } else {
                lastWind
            };
            let sumSet = (*span).windSum() == i32::MIN;
            if sumSet {
                (*span).setWindSum(windSum);
            }
            let oppSum = if SkOpSegment::UseInnerWinding(lastOpp, oppWind) {
                oppWind
            } else {
                lastOpp
            };
            if (*span).oppSum() == i32::MIN {
                (*span).setOppSum(oppSum);
            }
            if sumSet {
                if (*self.globalState()).phase() == SkOpPhase::kFixWinding {
                    (*(*seg).contour()).setCcw(ccw as i32);
                } else {
                    (*seg).markAndChaseWindingOp(
                        span.cast(),
                        (*span).next(),
                        windSum,
                        oppSum,
                        null_mut(),
                    );
                    (*seg).markAndChaseWindingOp(
                        (*span).next(),
                        span.cast(),
                        windSum,
                        oppSum,
                        null_mut(),
                    );
                }
            }
            if operand {
                core::mem::swap(&mut wind, &mut oppWind);
            }
            last = Some((*hit).fPt);
            (*self.globalState()).bumpNested();
        }
        true
    }
}
pub unsafe fn FindSortableTop(head: *mut SkOpContourHead) -> *mut SkOpSpan {
    for _ in 0..SkOpGlobalState::kMaxWindingTries {
        let mut contour = head;
        loop {
            if !(*contour).done() {
                let result = (*contour).findSortableTop(head);
                if !result.is_null() {
                    return result;
                }
            }
            contour = (*contour).next();
            if contour.is_null() {
                break;
            }
        }
    }
    null_mut()
}
