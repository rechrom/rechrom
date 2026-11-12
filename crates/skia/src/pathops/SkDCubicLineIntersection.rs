// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkDCubicLineIntersection.cpp.
use super::{
    SkIntersections::SkIntersections,
    SkPathOpsCubic::{SearchAxis, SkDCubic},
    SkPathOpsCurve::SkDCurve,
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::SkDPoint,
    SkPathOpsTypes::*,
};
use crate::src::core::SkPath::Verb;
struct LineCubicIntersections<'a> {
    c: &'a SkDCubic,
    line: &'a SkDLine,
    i: &'a mut SkIntersections,
    allow: bool,
}
fn axis_roots(c: &SkDCubic, vertical: bool, axis: f64, roots: &mut [f64]) -> usize {
    let coords = c.fPts.map(|p| if vertical { p.fX } else { p.fY });
    let (a, b, cc, mut d) = SkDCubic::Coefficients(&coords);
    d -= axis;
    let mut count = SkDCubic::RootsValidT(a, b, cc, d, roots);
    for index in 0..count {
        let pt = c.ptAtT(roots[index]);
        let v = if vertical { pt.fX } else { pt.fY };
        if !approximately_equal(v, axis) {
            let mut extremes = [0.0; 6];
            let n = SkDCubic::FindExtrema(&coords, &mut extremes);
            count = c.searchRoots(
                &mut extremes,
                n,
                axis,
                if vertical {
                    SearchAxis::kXAxis
                } else {
                    SearchAxis::kYAxis
                },
                roots,
            );
            break;
        }
    }
    count
}
fn ray_roots(c: &SkDCubic, line: &SkDLine, roots: &mut [f64]) -> usize {
    let adj = line[1].fX - line[0].fX;
    let opp = line[1].fY - line[0].fY;
    let mut rotated = SkDCubic::default();
    for n in 0..4 {
        rotated[n].fX = (c[n].fY - line[0].fY) * adj - (c[n].fX - line[0].fX) * opp;
    }
    let (a, b, cc, d) = SkDCubic::Coefficients(&rotated.fPts.map(|p| p.fX));
    let mut count = SkDCubic::RootsValidT(a, b, cc, d, roots);
    for index in 0..count {
        if !approximately_zero(rotated.ptAtT(roots[index]).fX) {
            for n in 0..4 {
                rotated[n].fY = (c[n].fY - line[0].fY) * opp + (c[n].fX - line[0].fX) * adj;
            }
            let mut extremes = [0.0; 6];
            let n = SkDCubic::FindExtrema(&rotated.fPts.map(|p| p.fX), &mut extremes);
            count = rotated.searchRoots(&mut extremes, n, 0.0, SearchAxis::kXAxis, roots);
            break;
        }
    }
    count
}
impl<'a> LineCubicIntersections<'a> {
    fn new(c: &'a SkDCubic, line: &'a SkDLine, i: &'a mut SkIntersections) -> Self {
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
        for index in [0, 3] {
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
            curve.fCubic = *self.c;
            let t = curve.nearPoint(Verb::kCubic_Verb, &self.line[index], &self.line[1 - index]);
            if t < 0.0 {
                continue;
            }
            self.i.insert(t, lineT, self.line[index]);
        }
    }
    fn addNearEndPoints(&mut self) {
        for index in [0, 3] {
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
        for index in [0, 3] {
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
        cubicT: &mut f64,
        lineT: &mut f64,
        pt: &mut SkDPoint,
        initialized: bool,
    ) -> bool {
        if !approximately_one_or_less(*lineT) || !approximately_zero_or_more(*lineT) {
            return false;
        }
        *cubicT = SkPinT(*cubicT);
        *lineT = SkPinT(*lineT);
        let ct = *cubicT;
        let lt = *lineT;
        let lp = self.line.ptAtT(lt);
        let cp = self.c.ptAtT(ct);
        if !lp.roughlyEqual(&cp) {
            return false;
        }
        if lt == 0.0 || lt == 1.0 || (!initialized && ct != 0.0 && ct != 1.0) {
            *pt = lp;
        } else if !initialized {
            *pt = cp;
        }
        let grid = pt.asSkPoint();
        if grid == self.line[0].asSkPoint() {
            *lineT = 0.0;
        } else if grid == self.line[1].asSkPoint() {
            *lineT = 1.0;
        }
        if grid == self.c[0].asSkPoint() && approximately_equal(*cubicT, 0.0) {
            *cubicT = 0.0;
        } else if grid == self.c[3].asSkPoint() && approximately_equal(*cubicT, 1.0) {
            *cubicT = 1.0;
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
        let mut roots = [0.0; 3];
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
        let mut roots = [0.0; 3];
        let count = axis_roots(self.c, vertical, value, &mut roots);
        for &root in &roots[..count] {
            let mut t = root;
            let mut pt = if vertical {
                SkDPoint {
                    fX: value,
                    fY: self.c.ptAtT(t).fY,
                }
            } else {
                SkDPoint {
                    fX: self.c.ptAtT(t).fX,
                    fY: value,
                }
            };
            let mut lineT = ((if vertical { pt.fY } else { pt.fX }) - start) / (end - start);
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
    pub fn horizontalCubic(
        &mut self,
        c: &SkDCubic,
        left: f64,
        right: f64,
        y: f64,
        flipped: bool,
    ) -> usize {
        let line = SkDLine {
            fPts: [SkDPoint { fX: left, fY: y }, SkDPoint { fX: right, fY: y }],
        };
        LineCubicIntersections::new(c, &line, self).axisIntersect(false, y, left, right, flipped)
    }
    pub fn verticalCubic(
        &mut self,
        c: &SkDCubic,
        top: f64,
        bottom: f64,
        x: f64,
        flipped: bool,
    ) -> usize {
        let line = SkDLine {
            fPts: [SkDPoint { fX: x, fY: top }, SkDPoint { fX: x, fY: bottom }],
        };
        LineCubicIntersections::new(c, &line, self).axisIntersect(true, x, top, bottom, flipped)
    }
    pub fn intersectCubicLine(&mut self, c: &SkDCubic, l: &SkDLine) -> usize {
        let allow = self.fAllowNear;
        let mut inter = LineCubicIntersections::new(c, l, self);
        inter.allow = allow;
        inter.intersect()
    }
    pub fn intersectRayCubic(&mut self, c: &SkDCubic, l: &SkDLine) -> usize {
        self.setMax(4);
        self.fUsed = ray_roots(c, l, &mut self.fT[0]);
        for i in 0..self.fUsed {
            self.fPt[i] = c.ptAtT(self.fT[0][i]);
        }
        self.fUsed
    }
}
impl SkDCubic {
    pub fn horizontalIntersect(&self, y: f64, roots: &mut [f64]) -> usize {
        axis_roots(self, false, y, roots)
    }
    pub fn verticalIntersect(&self, x: f64, roots: &mut [f64]) -> usize {
        axis_roots(self, true, x, roots)
    }
}
