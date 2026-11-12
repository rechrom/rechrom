// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkDQuadLineIntersection.cpp.
use super::{
    SkIntersections::SkIntersections, SkPathOpsCurve::SkDCurve, SkPathOpsLine::SkDLine,
    SkPathOpsPoint::SkDPoint, SkPathOpsQuad::SkDQuad, SkPathOpsTypes::*,
};
use crate::src::core::SkPath::Verb;

struct LineQuadraticIntersections<'a> {
    fQuad: &'a SkDQuad,
    fLine: *const SkDLine,
    fIntersections: *mut SkIntersections,
    fAllowNear: bool,
}
impl<'a> LineQuadraticIntersections<'a> {
    fn new(quad: &'a SkDQuad, line: &'a SkDLine, i: *mut SkIntersections) -> Self {
        unsafe {
            (*i).setMax(5);
        }
        Self {
            fQuad: quad,
            fLine: line,
            fIntersections: i,
            fAllowNear: true,
        }
    }
    fn quad(quad: &'a SkDQuad) -> Self {
        Self {
            fQuad: quad,
            fLine: core::ptr::null(),
            fIntersections: core::ptr::null_mut(),
            fAllowNear: false,
        }
    }
    unsafe fn checkCoincident(&mut self) {
        let i = &mut *self.fIntersections;
        let mut last = i.used() as isize - 1;
        let mut index = 0;
        while (index as isize) < last {
            let mid = (i[0][index] + i[0][index + 1]) / 2.0;
            let point = self.fQuad.ptAtT(mid);
            let t = (*self.fLine).nearPoint(&point, None);
            if t < 0.0 {
                index += 1;
                continue;
            }
            if i.isCoincident(index) {
                i.removeOne(index);
                last -= 1;
            } else if i.isCoincident(index + 1) {
                i.removeOne(index + 1);
                last -= 1;
            } else {
                i.setCoincident(index);
                index += 1;
            }
            i.setCoincident(index);
        }
    }
    unsafe fn intersectRay(&self, roots: &mut [f64]) -> usize {
        let line = &*self.fLine;
        let adj = line[1].fX - line[0].fX;
        let opp = line[1].fY - line[0].fY;
        let mut r = [0.0; 3];
        for n in 0..3 {
            r[n] = (self.fQuad[n].fY - line[0].fY) * adj - (self.fQuad[n].fX - line[0].fX) * opp;
        }
        let (mut a, mut b, c) = (r[2], r[1], r[0]);
        a += c - 2.0 * b;
        b -= c;
        SkDQuad::RootsValidT(a, 2.0 * b, c, roots)
    }
    unsafe fn intersect(&mut self) -> usize {
        self.addExactEndPoints();
        if self.fAllowNear {
            self.addNearEndPoints();
        }
        let mut roots = [0.0; 2];
        let count = self.intersectRay(&mut roots);
        for &root in &roots[..count] {
            let mut qt = root;
            let mut lt = self.findLineT(qt);
            let mut pt = SkDPoint::default();
            if self.pinTs(&mut qt, &mut lt, &mut pt, false) && self.uniqueAnswer(qt, &pt) {
                (*self.fIntersections).insert(qt, lt, pt);
            }
        }
        self.checkCoincident();
        (*self.fIntersections).used()
    }
    fn horizontalRoots(&self, axis: f64, roots: &mut [f64]) -> usize {
        let (mut d, mut e, mut f) = (self.fQuad[2].fY, self.fQuad[1].fY, self.fQuad[0].fY);
        d += f - 2.0 * e;
        e -= f;
        f -= axis;
        SkDQuad::RootsValidT(d, 2.0 * e, f, roots)
    }
    fn verticalRoots(&self, axis: f64, roots: &mut [f64]) -> usize {
        let (mut d, mut e, mut f) = (self.fQuad[2].fX, self.fQuad[1].fX, self.fQuad[0].fX);
        d += f - 2.0 * e;
        e -= f;
        f -= axis;
        SkDQuad::RootsValidT(d, 2.0 * e, f, roots)
    }
    unsafe fn horizontalIntersect(
        &mut self,
        axis: f64,
        left: f64,
        right: f64,
        flipped: bool,
    ) -> usize {
        self.addExactHorizontalEndPoints(left, right, axis);
        if self.fAllowNear {
            self.addNearHorizontalEndPoints(left, right, axis);
        }
        let mut roots = [0.0; 2];
        let count = self.horizontalRoots(axis, &mut roots);
        for &root in &roots[..count] {
            let mut qt = root;
            let mut pt = self.fQuad.ptAtT(qt);
            let mut lt = (pt.fX - left) / (right - left);
            if self.pinTs(&mut qt, &mut lt, &mut pt, true) && self.uniqueAnswer(qt, &pt) {
                (*self.fIntersections).insert(qt, lt, pt);
            }
        }
        if flipped {
            (*self.fIntersections).flip();
        }
        self.checkCoincident();
        (*self.fIntersections).used()
    }
    unsafe fn verticalIntersect(
        &mut self,
        axis: f64,
        top: f64,
        bottom: f64,
        flipped: bool,
    ) -> usize {
        self.addExactVerticalEndPoints(top, bottom, axis);
        if self.fAllowNear {
            self.addNearVerticalEndPoints(top, bottom, axis);
        }
        let mut roots = [0.0; 2];
        let count = self.verticalRoots(axis, &mut roots);
        for &root in &roots[..count] {
            let mut qt = root;
            let mut pt = self.fQuad.ptAtT(qt);
            let mut lt = (pt.fY - top) / (bottom - top);
            if self.pinTs(&mut qt, &mut lt, &mut pt, true) && self.uniqueAnswer(qt, &pt) {
                (*self.fIntersections).insert(qt, lt, pt);
            }
        }
        if flipped {
            (*self.fIntersections).flip();
        }
        self.checkCoincident();
        (*self.fIntersections).used()
    }
    unsafe fn uniqueAnswer(&self, qt: f64, point: &SkDPoint) -> bool {
        let i = &*self.fIntersections;
        for n in 0..i.used() {
            if i.pt(n) != point {
                continue;
            }
            let existing = i[0][n];
            if qt == existing {
                return false;
            }
            let mid = self.fQuad.ptAtT((existing + qt) / 2.0);
            if mid.approximatelyEqual(point) {
                return false;
            }
        }
        true
    }
    unsafe fn addExactEndPoints(&mut self) {
        for n in (0..3).step_by(2) {
            let t = (*self.fLine).exactPoint(&self.fQuad[n]);
            if t < 0.0 {
                continue;
            }
            (*self.fIntersections).insert((n >> 1) as f64, t, self.fQuad[n]);
        }
    }
    unsafe fn addNearEndPoints(&mut self) {
        for n in (0..3).step_by(2) {
            let qt = (n >> 1) as f64;
            if (*self.fIntersections).hasT(qt) {
                continue;
            }
            let lt = (*self.fLine).nearPoint(&self.fQuad[n], None);
            if lt < 0.0 {
                continue;
            }
            (*self.fIntersections).insert(qt, lt, self.fQuad[n]);
        }
        self.addLineNearEndPoints();
    }
    unsafe fn addLineNearEndPoints(&mut self) {
        for n in 0..2 {
            let lt = n as f64;
            if (*self.fIntersections).hasOppT(lt) {
                continue;
            }
            let curve = SkDCurve { fQuad: *self.fQuad };
            let qt = curve.nearPoint(Verb::kQuad_Verb, &(&*self.fLine)[n], &(&*self.fLine)[1 - n]);
            if qt < 0.0 {
                continue;
            }
            (*self.fIntersections).insert(qt, lt, (&*self.fLine)[n]);
        }
    }
    unsafe fn addExactHorizontalEndPoints(&mut self, left: f64, right: f64, y: f64) {
        for n in (0..3).step_by(2) {
            let lt = SkDLine::ExactPointH(&self.fQuad[n], left, right, y);
            if lt < 0.0 {
                continue;
            }
            (*self.fIntersections).insert((n >> 1) as f64, lt, self.fQuad[n]);
        }
    }
    unsafe fn addNearHorizontalEndPoints(&mut self, left: f64, right: f64, y: f64) {
        for n in (0..3).step_by(2) {
            let qt = (n >> 1) as f64;
            if (*self.fIntersections).hasT(qt) {
                continue;
            }
            let lt = SkDLine::NearPointH(&self.fQuad[n], left, right, y);
            if lt < 0.0 {
                continue;
            }
            (*self.fIntersections).insert(qt, lt, self.fQuad[n]);
        }
        self.addLineNearEndPoints();
    }
    unsafe fn addExactVerticalEndPoints(&mut self, top: f64, bottom: f64, x: f64) {
        for n in (0..3).step_by(2) {
            let lt = SkDLine::ExactPointV(&self.fQuad[n], top, bottom, x);
            if lt < 0.0 {
                continue;
            }
            (*self.fIntersections).insert((n >> 1) as f64, lt, self.fQuad[n]);
        }
    }
    unsafe fn addNearVerticalEndPoints(&mut self, top: f64, bottom: f64, x: f64) {
        for n in (0..3).step_by(2) {
            let qt = (n >> 1) as f64;
            if (*self.fIntersections).hasT(qt) {
                continue;
            }
            let lt = SkDLine::NearPointV(&self.fQuad[n], top, bottom, x);
            if lt < 0.0 {
                continue;
            }
            (*self.fIntersections).insert(qt, lt, self.fQuad[n]);
        }
        self.addLineNearEndPoints();
    }
    unsafe fn findLineT(&self, t: f64) -> f64 {
        let xy = self.fQuad.ptAtT(t);
        let line = &*self.fLine;
        let dx = line[1].fX - line[0].fX;
        let dy = line[1].fY - line[0].fY;
        if dx.abs() > dy.abs() {
            (xy.fX - line[0].fX) / dx
        } else {
            (xy.fY - line[0].fY) / dy
        }
    }
    unsafe fn pinTs(
        &self,
        qt: &mut f64,
        lt: &mut f64,
        pt: &mut SkDPoint,
        initialized: bool,
    ) -> bool {
        if !approximately_one_or_less_double(*lt) || !approximately_zero_or_more_double(*lt) {
            return false;
        }
        *qt = SkPinT(*qt);
        *lt = SkPinT(*lt);
        let q = *qt;
        let l = *lt;
        let line = &*self.fLine;
        if l == 0.0 || l == 1.0 || (!initialized && q != 0.0 && q != 1.0) {
            *pt = line.ptAtT(l);
        } else if !initialized {
            *pt = self.fQuad.ptAtT(q);
        }
        let grid = pt.asSkPoint();
        if SkDPoint::ApproximatelyEqual(grid, line[0].asSkPoint()) {
            *pt = line[0];
            *lt = 0.0;
        } else if SkDPoint::ApproximatelyEqual(grid, line[1].asSkPoint()) {
            *pt = line[1];
            *lt = 1.0;
        }
        let i = &*self.fIntersections;
        if i.used() > 0 && approximately_equal(i[1][0], *lt) {
            return false;
        }
        if grid == self.fQuad[0].asSkPoint() {
            *pt = self.fQuad[0];
            *qt = 0.0;
        } else if grid == self.fQuad[2].asSkPoint() {
            *pt = self.fQuad[2];
            *qt = 1.0;
        }
        true
    }
}
impl SkIntersections {
    pub fn horizontalQuad(
        &mut self,
        quad: &SkDQuad,
        left: f64,
        right: f64,
        y: f64,
        flipped: bool,
    ) -> usize {
        let line = SkDLine {
            fPts: [SkDPoint { fX: left, fY: y }, SkDPoint { fX: right, fY: y }],
        };
        unsafe {
            LineQuadraticIntersections::new(quad, &line, self)
                .horizontalIntersect(y, left, right, flipped)
        }
    }
    pub fn verticalQuad(
        &mut self,
        quad: &SkDQuad,
        top: f64,
        bottom: f64,
        x: f64,
        flipped: bool,
    ) -> usize {
        let line = SkDLine {
            fPts: [SkDPoint { fX: x, fY: top }, SkDPoint { fX: x, fY: bottom }],
        };
        unsafe {
            LineQuadraticIntersections::new(quad, &line, self)
                .verticalIntersect(x, top, bottom, flipped)
        }
    }
    pub fn intersectQuadLine(&mut self, quad: &SkDQuad, line: &SkDLine) -> usize {
        let mut q = LineQuadraticIntersections::new(quad, line, self);
        q.fAllowNear = self.fAllowNear;
        unsafe { q.intersect() }
    }
    pub fn intersectRayQuad(&mut self, quad: &SkDQuad, line: &SkDLine) -> usize {
        let q = LineQuadraticIntersections::new(quad, line, self);
        self.fUsed = unsafe { q.intersectRay(&mut self.fT[0]) };
        for n in 0..self.fUsed {
            self.fPt[n] = quad.ptAtT(self.fT[0][n]);
        }
        self.fUsed
    }
    pub fn HorizontalInterceptQuad(quad: &SkDQuad, y: f32, roots: &mut [f64]) -> usize {
        LineQuadraticIntersections::quad(quad).horizontalRoots(y as f64, roots)
    }
    pub fn VerticalInterceptQuad(quad: &SkDQuad, x: f32, roots: &mut [f64]) -> usize {
        LineQuadraticIntersections::quad(quad).verticalRoots(x as f64, roots)
    }
}
impl SkDQuad {
    pub fn horizontalIntersect(&self, y: f64, roots: &mut [f64]) -> usize {
        SkIntersections::HorizontalInterceptQuad(self, y as f32, roots)
    }
    pub fn verticalIntersect(&self, x: f64, roots: &mut [f64]) -> usize {
        SkIntersections::VerticalInterceptQuad(self, x as f32, roots)
    }
}
