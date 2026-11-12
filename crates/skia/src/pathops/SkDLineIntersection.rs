// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkDLineIntersection.cpp. Native line/ray intersections.
use super::{
    SkIntersections::SkIntersections, SkPathOpsLine::SkDLine, SkPathOpsPoint::SkDPoint,
    SkPathOpsTypes::*,
};
impl SkIntersections {
    pub(super) fn cleanUpParallelLines(&mut self, parallel: bool) {
        while self.fUsed > 2 {
            self.removeOne(1);
        }
        if self.fUsed == 2 && !parallel {
            let start = self.fT[0][0] == 0.0 || zero_or_one(self.fT[1][0]);
            let end = self.fT[0][1] == 1.0 || zero_or_one(self.fT[1][1]);
            if (!start && !end) || approximately_equal(self.fT[0][0], self.fT[0][1]) {
                if start
                    && end
                    && (self.fT[0][0] != 0.0 || !zero_or_one(self.fT[1][0]))
                    && self.fT[0][1] == 1.0
                    && zero_or_one(self.fT[1][1])
                {
                    self.removeOne(0);
                } else {
                    self.removeOne(end as usize);
                }
            }
        }
        if self.fUsed == 2 {
            self.fIsCoincident = [0x03; 2];
        }
    }
    fn computePoints(&mut self, line: &SkDLine, used: usize) {
        self.fPt[0] = line.ptAtT(self.fT[0][0]);
        self.fUsed = used;
        if used == 2 {
            self.fPt[1] = line.ptAtT(self.fT[0][1]);
        }
    }
    pub fn intersectRayLine(&mut self, a: &SkDLine, b: &SkDLine) -> usize {
        self.fMax = 2;
        let al = a[1] - a[0];
        let bl = b[1] - b[0];
        let denom = bl.fY * al.fX - al.fY * bl.fX;
        let used;
        if !approximately_zero(denom) {
            let ab = a[0] - b[0];
            self.fT[0][0] = (ab.fY * bl.fX - bl.fY * ab.fX) / denom;
            self.fT[1][0] = (ab.fY * al.fX - al.fY * ab.fX) / denom;
            used = 1;
        } else {
            if !AlmostEqualUlps(
                al.fX * a[0].fY - al.fY * a[0].fX,
                al.fX * b[0].fY - al.fY * b[0].fX,
            ) {
                self.fUsed = 0;
                return 0;
            }
            self.fT[0][0] = 0.0;
            self.fT[1][0] = 0.0;
            self.fT[1][0] = 1.0;
            self.fT[1][1] = 1.0;
            used = 2;
        }
        self.computePoints(a, used);
        self.fUsed
    }
    pub fn intersectLineLine(&mut self, a: &SkDLine, b: &SkDLine) -> usize {
        self.fMax = 3;
        for ia in 0..2 {
            let t = b.exactPoint(&a[ia]);
            if t >= 0.0 {
                self.insert(ia as f64, t, &a[ia]);
            }
        }
        for ib in 0..2 {
            let t = a.exactPoint(&b[ib]);
            if t >= 0.0 {
                self.insert(t, ib as f64, &b[ib]);
            }
        }
        let ax = a[1].fX - a[0].fX;
        let ay = a[1].fY - a[0].fY;
        let bx = b[1].fX - b[0].fX;
        let by = b[1].fY - b[0].fY;
        let axby = ax * by;
        let aybx = ay * bx;
        let unparallel = if self.fAllowNear {
            NotAlmostEqualUlps_Pin(axby, aybx)
        } else {
            NotAlmostDequalUlps(axby, aybx)
        };
        if unparallel && self.fUsed == 0 {
            let y = a[0].fY - b[0].fY;
            let x = a[0].fX - b[0].fX;
            let na = y * bx - by * x;
            let nb = y * ax - ay * x;
            let denom = axby - aybx;
            if between(0.0, na, denom) && between(0.0, nb, denom) {
                self.fT[0][0] = na / denom;
                self.fT[1][0] = nb / denom;
                self.computePoints(a, 1);
            }
        }
        if self.fAllowNear || !unparallel {
            let mut an = [0.0; 2];
            let mut bn = [0.0; 2];
            let mut anot = [false; 2];
            let mut bnot = [false; 2];
            let mut count = 0;
            for idx in 0..2 {
                an[idx] = b.nearPoint(&a[idx], Some(&mut anot[idx]));
                count += (an[idx] >= 0.0) as i32;
                bn[idx] = a.nearPoint(&b[idx], Some(&mut bnot[idx]));
                count += (bn[idx] >= 0.0) as i32;
            }
            if count > 0 {
                if count != 2 || anot[0] == anot[1] {
                    for ia in 0..2 {
                        if !anot[ia] {
                            continue;
                        }
                        let nearer = (an[ia] > 0.5) as usize;
                        if !bnot[nearer] {
                            continue;
                        }

                        self.insertNear(ia as f64, nearer as f64, &a[ia], &b[nearer]);
                        an[ia] = -1.0;
                        bn[nearer] = -1.0;
                        count -= 2;
                    }
                }
                if count > 0 {
                    for ia in 0..2 {
                        if an[ia] >= 0.0 {
                            self.insert(ia as f64, an[ia], &a[ia]);
                        }
                    }
                    for ib in 0..2 {
                        if bn[ib] >= 0.0 {
                            self.insert(bn[ib], ib as f64, &b[ib]);
                        }
                    }
                }
            }
        }
        self.cleanUpParallelLines(!unparallel);

        self.fUsed
    }
    pub fn HorizontalInterceptLine(line: &SkDLine, y: f64) -> f64 {
        SkPinT((y - line[0].fY) / (line[1].fY - line[0].fY))
    }
    pub fn VerticalInterceptLine(line: &SkDLine, x: f64) -> f64 {
        SkPinT((x - line[0].fX) / (line[1].fX - line[0].fX))
    }
    pub fn horizontalLine(
        &mut self,
        line: &SkDLine,
        left: f64,
        right: f64,
        y: f64,
        flipped: bool,
    ) -> usize {
        self.fMax = 3;
        let lp = SkDPoint { fX: left, fY: y };
        let t = line.exactPoint(&lp);
        if t >= 0.0 {
            self.insert(t, flipped as u8 as f64, &lp);
        }
        if left != right {
            let rp = SkDPoint { fX: right, fY: y };
            let t = line.exactPoint(&rp);
            if t >= 0.0 {
                self.insert(t, !flipped as u8 as f64, &rp);
            }
            for idx in 0..2 {
                let t = SkDLine::ExactPointH(&line[idx], left, right, y);
                if t >= 0.0 {
                    self.insert(idx as f64, if flipped { 1.0 - t } else { t }, &line[idx]);
                }
            }
        }
        let result = horizontal_coincident(line, y);
        if result == 1 && self.fUsed == 0 {
            self.fT[0][0] = Self::HorizontalInterceptLine(line, y);
            let x = line[0].fX + self.fT[0][0] * (line[1].fX - line[0].fX);
            if between(left, x, right) {
                self.fT[1][0] = (x - left) / (right - left);
                if flipped {
                    for idx in 0..result {
                        self.fT[1][idx] = 1.0 - self.fT[1][idx];
                    }
                }
                self.fPt[0] = SkDPoint { fX: x, fY: y };
                self.fUsed = 1;
            }
        }
        if self.fAllowNear || result == 2 {
            let t = line.nearPoint(&lp, None);
            if t >= 0.0 {
                self.insert(t, flipped as u8 as f64, &lp);
            }
            if left != right {
                let rp = SkDPoint { fX: right, fY: y };
                let t = line.nearPoint(&rp, None);
                if t >= 0.0 {
                    self.insert(t, !flipped as u8 as f64, &rp);
                }
                for idx in 0..2 {
                    let t = SkDLine::NearPointH(&line[idx], left, right, y);
                    if t >= 0.0 {
                        self.insert(idx as f64, if flipped { 1.0 - t } else { t }, &line[idx]);
                    }
                }
            }
        }
        self.cleanUpParallelLines(result == 2);
        self.fUsed
    }
    pub fn verticalLine(
        &mut self,
        line: &SkDLine,
        top: f64,
        bottom: f64,
        x: f64,
        flipped: bool,
    ) -> usize {
        self.fMax = 3;
        let tp = SkDPoint { fX: x, fY: top };
        let t = line.exactPoint(&tp);
        if t >= 0.0 {
            self.insert(t, flipped as u8 as f64, &tp);
        }
        if top != bottom {
            let bp = SkDPoint { fX: x, fY: bottom };
            let t = line.exactPoint(&bp);
            if t >= 0.0 {
                self.insert(t, !flipped as u8 as f64, &bp);
            }
            for idx in 0..2 {
                let t = SkDLine::ExactPointV(&line[idx], top, bottom, x);
                if t >= 0.0 {
                    self.insert(idx as f64, if flipped { 1.0 - t } else { t }, &line[idx]);
                }
            }
        }
        let result = vertical_coincident(line, x);
        if result == 1 && self.fUsed == 0 {
            self.fT[0][0] = Self::VerticalInterceptLine(line, x);
            let y = line[0].fY + self.fT[0][0] * (line[1].fY - line[0].fY);
            if between(top, y, bottom) {
                self.fT[1][0] = (y - top) / (bottom - top);
                if flipped {
                    for idx in 0..result {
                        self.fT[1][idx] = 1.0 - self.fT[1][idx];
                    }
                }
                self.fPt[0] = SkDPoint { fX: x, fY: y };
                self.fUsed = 1;
            }
        }
        if self.fAllowNear || result == 2 {
            let t = line.nearPoint(&tp, None);
            if t >= 0.0 {
                self.insert(t, flipped as u8 as f64, &tp);
            }
            if top != bottom {
                let bp = SkDPoint { fX: x, fY: bottom };
                let t = line.nearPoint(&bp, None);
                if t >= 0.0 {
                    self.insert(t, !flipped as u8 as f64, &bp);
                }
                for idx in 0..2 {
                    let t = SkDLine::NearPointV(&line[idx], top, bottom, x);
                    if t >= 0.0 {
                        self.insert(idx as f64, if flipped { 1.0 - t } else { t }, &line[idx]);
                    }
                }
            }
        }
        self.cleanUpParallelLines(result == 2);

        self.fUsed
    }
}
fn horizontal_coincident(line: &SkDLine, y: f64) -> usize {
    let mut min = line[0].fY;
    let mut max = line[1].fY;
    if min > max {
        core::mem::swap(&mut min, &mut max);
    }
    if min > y || max < y {
        return 0;
    }
    if AlmostEqualUlps(min, max) && max - min < (line[0].fX - line[1].fX).abs() {
        2
    } else {
        1
    }
}
fn vertical_coincident(line: &SkDLine, x: f64) -> usize {
    let mut min = line[0].fX;
    let mut max = line[1].fX;
    if min > max {
        core::mem::swap(&mut min, &mut max);
    }
    if !precisely_between(min, x, max) {
        return 0;
    }
    if AlmostEqualUlps(min, max) {
        2
    } else {
        1
    }
}
