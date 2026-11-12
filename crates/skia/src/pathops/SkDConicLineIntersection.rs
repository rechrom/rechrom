// Copyright 2015 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkDConicLineIntersection.cpp.
use super::{
    SkIntersections::SkIntersections, SkPathOpsConic::SkDConic, SkPathOpsCurve::SkDCurve,
    SkPathOpsLine::SkDLine, SkPathOpsPoint::SkDPoint, SkPathOpsQuad::SkDQuad, SkPathOpsTypes::*,
};
use crate::src::core::SkPath::Verb;
struct LineConicIntersections<'a> {
    c: &'a SkDConic,
    line: &'a SkDLine,
    i: &'a mut SkIntersections,
    allow: bool,
}
fn valid_t(c: &SkDConic, r: [f64; 3], axis: f64, roots: &mut [f64]) -> usize {
    let mut a = r[2];
    let mut b = r[1] * c.fWeight as f64 - axis * c.fWeight as f64 + axis;
    let mut cc = r[0];
    a += cc - 2.0 * b;
    b -= cc;
    cc -= axis;
    SkDQuad::RootsValidT(a, 2.0 * b, cc, roots)
}
fn ray_roots(c: &SkDConic, line: &SkDLine, roots: &mut [f64]) -> usize {
    let adj = line[1].fX - line[0].fX;
    let opp = line[1].fY - line[0].fY;
    let r = c
        .fPts
        .fPts
        .map(|p| (p.fY - line[0].fY) * adj - (p.fX - line[0].fX) * opp);
    valid_t(c, r, 0.0, roots)
}
impl<'a> LineConicIntersections<'a> {
    fn new(c: &'a SkDConic, line: &'a SkDLine, i: &'a mut SkIntersections) -> Self {
        i.setMax(4);
        Self {
            c,
            line,
            i,
            allow: true,
        }
    }
    fn checkCoincident(&mut self) {
        let mut last = self.i.used() as isize - 1;
        let mut index = 0;
        while (index as isize) < last {
            let mid = (self.i[0][index] + self.i[0][index + 1]) / 2.0;
            let pt = self.c.ptAtT(mid);
            let t = self.line.nearPoint(&pt, None);
            if t < 0.0 {
                index += 1;
                continue;
            }
            if self.i.isCoincident(index) {
                self.i.removeOne(index);
                last -= 1;
            } else if self.i.isCoincident(index + 1) {
                self.i.removeOne(index + 1);
                last -= 1;
            } else {
                self.i.setCoincident(index);
                index += 1;
            }
            self.i.setCoincident(index);
        }
    }
    fn addExactEndPoints(&mut self) {
        for index in [0, 2] {
            let t = self.line.exactPoint(&self.c[index]);
            if t >= 0.0 {
                self.i.insert((index >> 1) as f64, t, self.c[index]);
            }
        }
    }
    fn addLineNearEndPoints(&mut self) {
        for index in 0..2 {
            let lineT = index as f64;
            if self.i.hasOppT(lineT) {
                continue;
            }
            let mut curve = SkDCurve::default();
            curve.fConic = *self.c;
            let t = curve.nearPoint(Verb::kConic_Verb, &self.line[index], &self.line[1 - index]);
            if t < 0.0 {
                continue;
            }
            self.i.insert(t, lineT, self.line[index]);
        }
    }
    fn addNearEndPoints(&mut self) {
        for index in [0, 2] {
            let t = (index >> 1) as f64;
            if self.i.hasT(t) {
                continue;
            }
            let lineT = self.line.nearPoint(&self.c[index], None);
            if lineT < 0.0 {
                continue;
            }
            self.i.insert(t, lineT, self.c[index]);
        }
        self.addLineNearEndPoints();
    }
    fn addAxisEnds(&mut self, vertical: bool, near: bool, start: f64, end: f64, intercept: f64) {
        for index in [0, 2] {
            let t = (index >> 1) as f64;
            if near && self.i.hasT(t) {
                continue;
            }
            let pt = &self.c[index];
            let lineT = match (vertical, near) {
                (false, false) => SkDLine::ExactPointH(pt, start, end, intercept),
                (false, true) => SkDLine::NearPointH(pt, start, end, intercept),
                (true, false) => SkDLine::ExactPointV(pt, start, end, intercept),
                (true, true) => SkDLine::NearPointV(pt, start, end, intercept),
            };
            if lineT < 0.0 {
                continue;
            }
            self.i.insert(t, lineT, *pt);
        }
        if near {
            self.addLineNearEndPoints();
        }
    }
    fn pinTs(
        &mut self,
        conicT: &mut f64,
        lineT: &mut f64,
        pt: &mut SkDPoint,
        initialized: bool,
    ) -> bool {
        if !approximately_one_or_less_double(*lineT) || !approximately_zero_or_more_double(*lineT) {
            return false;
        }
        *conicT = SkPinT(*conicT);
        *lineT = SkPinT(*lineT);
        let q = *conicT;
        let l = *lineT;
        if l == 0.0 || l == 1.0 || (!initialized && q != 0.0 && q != 1.0) {
            *pt = self.line.ptAtT(l);
        } else if !initialized {
            *pt = self.c.ptAtT(q);
        }
        let grid = pt.asSkPoint();
        if SkDPoint::ApproximatelyEqual(grid, self.line[0].asSkPoint()) {
            *pt = self.line[0];
            *lineT = 0.0;
        } else if SkDPoint::ApproximatelyEqual(grid, self.line[1].asSkPoint()) {
            *pt = self.line[1];
            *lineT = 1.0;
        }
        if self.i.used() > 0 && approximately_equal(self.i[1][0], *lineT) {
            return false;
        }
        if grid == self.c[0].asSkPoint() {
            *pt = self.c[0];
            *conicT = 0.0;
        } else if grid == self.c[2].asSkPoint() {
            *pt = self.c[2];
            *conicT = 1.0;
        }
        true
    }
    fn uniqueAnswer(&self, t: f64, pt: &SkDPoint) -> bool {
        for inner in 0..self.i.used() {
            if self.i.pt(inner) != pt {
                continue;
            }
            let old = self.i[0][inner];
            if t == old {
                return false;
            }
            if self.c.ptAtT((old + t) / 2.0).approximatelyEqual(pt) {
                return false;
            }
        }
        true
    }
    fn findLineT(&self, t: f64) -> f64 {
        let xy = self.c.ptAtT(t);
        let dx = self.line[1].fX - self.line[0].fX;
        let dy = self.line[1].fY - self.line[0].fY;
        if dx.abs() > dy.abs() {
            (xy.fX - self.line[0].fX) / dx
        } else {
            (xy.fY - self.line[0].fY) / dy
        }
    }
    fn intersect(&mut self) -> usize {
        self.addExactEndPoints();
        if self.allow {
            self.addNearEndPoints();
        }
        let mut roots = [0.0; 2];
        let count = ray_roots(self.c, self.line, &mut roots);
        for &root in &roots[..count] {
            let mut t = root;
            let mut lineT = self.findLineT(t);
            let mut pt = SkDPoint::default();
            if self.pinTs(&mut t, &mut lineT, &mut pt, false) && self.uniqueAnswer(t, &pt) {
                self.i.insert(t, lineT, pt);
            }
        }
        self.checkCoincident();
        self.i.used()
    }
    fn axisIntersect(
        &mut self,
        vertical: bool,
        value: f64,
        start: f64,
        end: f64,
        flipped: bool,
    ) -> usize {
        self.addAxisEnds(vertical, false, start, end, value);
        if self.allow {
            self.addAxisEnds(vertical, true, start, end, value);
        }
        let mut roots = [0.0; 2];
        let coords = self.c.fPts.fPts.map(|p| if vertical { p.fX } else { p.fY });
        let count = valid_t(self.c, coords, value, &mut roots);
        for &root in &roots[..count] {
            let mut t = root;
            let mut pt = self.c.ptAtT(t);
            let mut lineT = (if vertical { pt.fY } else { pt.fX } - start) / (end - start);
            if self.pinTs(&mut t, &mut lineT, &mut pt, true) && self.uniqueAnswer(t, &pt) {
                self.i.insert(t, lineT, pt);
            }
        }
        if flipped {
            self.i.flip();
        }
        self.checkCoincident();
        self.i.used()
    }
}
impl SkIntersections {
    pub fn horizontalConic(
        &mut self,
        c: &SkDConic,
        left: f64,
        right: f64,
        y: f64,
        flipped: bool,
    ) -> usize {
        let line = SkDLine {
            fPts: [SkDPoint { fX: left, fY: y }, SkDPoint { fX: right, fY: y }],
        };
        LineConicIntersections::new(c, &line, self).axisIntersect(false, y, left, right, flipped)
    }
    pub fn verticalConic(
        &mut self,
        c: &SkDConic,
        top: f64,
        bottom: f64,
        x: f64,
        flipped: bool,
    ) -> usize {
        let line = SkDLine {
            fPts: [SkDPoint { fX: x, fY: top }, SkDPoint { fX: x, fY: bottom }],
        };
        LineConicIntersections::new(c, &line, self).axisIntersect(true, x, top, bottom, flipped)
    }
    pub fn intersectConicLine(&mut self, c: &SkDConic, l: &SkDLine) -> usize {
        let allow = self.fAllowNear;
        let mut inter = LineConicIntersections::new(c, l, self);
        inter.allow = allow;
        inter.intersect()
    }
    pub fn intersectRayConic(&mut self, c: &SkDConic, l: &SkDLine) -> usize {
        self.setMax(4);
        self.fUsed = ray_roots(c, l, &mut self.fT[0]);
        for i in 0..self.fUsed {
            self.fPt[i] = c.ptAtT(self.fT[0][i]);
        }
        self.fUsed
    }
    pub fn HorizontalInterceptConic(c: &SkDConic, y: f32, roots: &mut [f64]) -> usize {
        valid_t(c, c.fPts.fPts.map(|p| p.fY), y as f64, roots)
    }
    pub fn VerticalInterceptConic(c: &SkDConic, x: f32, roots: &mut [f64]) -> usize {
        valid_t(c, c.fPts.fPts.map(|p| p.fX), x as f64, roots)
    }
}
