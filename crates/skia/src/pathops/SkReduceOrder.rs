// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkReduceOrder.h/.cpp. Native union storage is retained.
use super::{
    SkPathOpsCubic::SkDCubic, SkPathOpsLine::SkDLine, SkPathOpsPoint::SkDPoint,
    SkPathOpsQuad::SkDQuad, SkPathOpsTypes::*,
};
use crate::src::core::{SkGeometry::SkConic, SkPath::Verb, SkPoint::SkPoint};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quadratics {
    kNo_Quadratics,
    kAllow_Quadratics,
}
pub use Quadratics::*;
#[repr(C)]
pub union SkReduceOrder {
    pub fLine: SkDLine,
    pub fQuad: SkDQuad,
    pub fCubic: SkDCubic,
}
impl Default for SkReduceOrder {
    fn default() -> Self {
        Self {
            fCubic: SkDCubic::default(),
        }
    }
}
fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}
fn line_count(a: SkDPoint, b: SkDPoint) -> i32 {
    1 + (!a.approximatelyEqual(b)) as i32
}
impl SkReduceOrder {
    pub unsafe fn reduceLine(&mut self, line: &SkDLine) -> i32 {
        self.fLine[0] = line[0];
        let different = (line[0] != line[1]) as usize;
        self.fLine[1] = line[different];
        1 + different as i32
    }
    pub unsafe fn reduceQuad(&mut self, quad: &SkDQuad) -> i32 {
        let (mut minX, mut maxX, mut minY, mut maxY) = (0, 0, 0, 0);
        for i in 1..3 {
            if quad[minX].fX > quad[i].fX {
                minX = i;
            }
            if quad[minY].fY > quad[i].fY {
                minY = i;
            }
            if quad[maxX].fX < quad[i].fX {
                maxX = i;
            }
            if quad[maxY].fY < quad[i].fY {
                maxY = i;
            }
        }
        let (mut minXSet, mut minYSet) = (0, 0);
        for i in 0..3 {
            if AlmostEqualUlps(quad[i].fX, quad[minX].fX) {
                minXSet |= 1 << i;
            }
            if AlmostEqualUlps(quad[i].fY, quad[minY].fY) {
                minYSet |= 1 << i;
            }
        }
        if minXSet & 0x5 == 0x5 && minYSet & 0x5 == 0x5 {
            self.fQuad[0] = quad[0];
            self.fQuad[1] = quad[0];
            return 1;
        }
        if minXSet == 0x7 || minYSet == 0x7 || quad.isLinear(0, 2) {
            self.fQuad[0] = quad[0];
            self.fQuad[1] = quad[2];
            return line_count(self.fQuad[0], self.fQuad[1]);
        }
        self.fQuad = *quad;
        3
    }
    pub unsafe fn reduceCubic(&mut self, cubic: &SkDCubic, allow: Quadratics) -> i32 {
        let (mut minX, mut maxX, mut minY, mut maxY) = (0, 0, 0, 0);
        for i in 1..4 {
            if cubic[minX].fX > cubic[i].fX {
                minX = i;
            }
            if cubic[minY].fY > cubic[i].fY {
                minY = i;
            }
            if cubic[maxX].fX < cubic[i].fX {
                maxX = i;
            }
            if cubic[maxY].fY < cubic[i].fY {
                maxY = i;
            }
        }
        let (mut minXSet, mut minYSet) = (0, 0);
        for i in 0..4 {
            let cx = cubic[i].fX;
            let cy = cubic[i].fY;
            let denom = cpp_max(
                cx.abs(),
                cpp_max(
                    cy.abs(),
                    cpp_max(cubic[minX].fX.abs(), cubic[minY].fY.abs()),
                ),
            );
            if denom == 0.0 {
                minXSet |= 1 << i;
                minYSet |= 1 << i;
                continue;
            }
            let inv = 1.0 / denom;
            if approximately_equal_half(cx * inv, cubic[minX].fX * inv) {
                minXSet |= 1 << i;
            }
            if approximately_equal_half(cy * inv, cubic[minY].fY * inv) {
                minYSet |= 1 << i;
            }
        }
        if minXSet == 0xf && minYSet == 0xf {
            self.fCubic[0] = cubic[0];
            self.fCubic[1] = cubic[0];
            return 1;
        }
        if minXSet == 0xf || minYSet == 0xf || cubic.isLinear(0, 3) {
            self.fCubic[0] = cubic[0];
            self.fCubic[1] = cubic[3];
            return line_count(self.fCubic[0], self.fCubic[1]);
        }
        if allow == kAllow_Quadratics {
            let dx10 = cubic[1].fX - cubic[0].fX;
            let dx23 = cubic[2].fX - cubic[3].fX;
            let midX = cubic[0].fX + dx10 * 3.0 / 2.0;
            let sideAx = midX - cubic[3].fX;
            let sideBx = dx23 * 3.0 / 2.0;
            let xEqual = if approximately_zero(sideAx) {
                approximately_equal(sideAx, sideBx)
            } else {
                AlmostEqualUlps_Pin(sideAx, sideBx)
            };
            if xEqual {
                let dy10 = cubic[1].fY - cubic[0].fY;
                let dy23 = cubic[2].fY - cubic[3].fY;
                let midY = cubic[0].fY + dy10 * 3.0 / 2.0;
                let sideAy = midY - cubic[3].fY;
                let sideBy = dy23 * 3.0 / 2.0;
                let yEqual = if approximately_zero(sideAy) {
                    approximately_equal(sideAy, sideBy)
                } else {
                    AlmostEqualUlps_Pin(sideAy, sideBy)
                };
                if yEqual {
                    self.fCubic[0] = cubic[0];
                    self.fCubic[1] = SkDPoint { fX: midX, fY: midY };
                    self.fCubic[2] = cubic[3];
                    return 3;
                }
            }
        }
        self.fCubic = *cubic;
        4
    }
    pub fn Quad(pts: &[SkPoint; 3], reduced: &mut [SkPoint]) -> Verb {
        let mut quad = SkDQuad::default();
        quad.set(pts);
        let mut reducer = Self::default();
        unsafe {
            let order = reducer.reduceQuad(&quad);
            if order == 2 {
                for i in 0..order as usize {
                    reduced[i] = reducer.fLine[i].asSkPoint();
                }
            }
            SkPathOpsPointsToVerb(order - 1)
        }
    }
    pub fn Conic(c: &SkConic, reduced: &mut [SkPoint]) -> Verb {
        let verb = Self::Quad(&c.fPts, reduced);
        if (verb as u32) > (Verb::kLine_Verb as u32) && c.fW == 1.0 {
            return Verb::kQuad_Verb;
        }
        if verb == Verb::kQuad_Verb {
            Verb::kConic_Verb
        } else {
            verb
        }
    }
    pub fn Cubic(pts: &[SkPoint; 4], reduced: &mut [SkPoint]) -> Verb {
        if SkDPoint::ApproximatelyEqual(pts[0], pts[1])
            && SkDPoint::ApproximatelyEqual(pts[0], pts[2])
            && SkDPoint::ApproximatelyEqual(pts[0], pts[3])
        {
            reduced[0] = pts[0];
            return Verb::kMove_Verb;
        }
        let mut cubic = SkDCubic::default();
        cubic.set(pts);
        let mut reducer = Self::default();
        unsafe {
            let order = reducer.reduceCubic(&cubic, kAllow_Quadratics);
            if order == 2 || order == 3 {
                for i in 0..order as usize {
                    reduced[i] = reducer.fQuad[i].asSkPoint();
                }
            }
            SkPathOpsPointsToVerb(order - 1)
        }
    }
}
