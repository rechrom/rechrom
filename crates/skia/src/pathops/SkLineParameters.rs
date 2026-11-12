// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkLineParameters.h.
use super::{
    SkPathOpsCubic::SkDCubic, SkPathOpsLine::SkDLine, SkPathOpsPoint::SkDPoint,
    SkPathOpsQuad::SkDQuad, SkPathOpsTypes::*,
};
#[derive(Clone, Copy, Debug, Default)]
pub struct SkLineParameters {
    fA: f64,
    fB: f64,
    fC: f64,
}
impl SkLineParameters {
    pub fn cubicEndPoints(&mut self, p: &SkDCubic) -> bool {
        let mut end = 1;
        self.cubicEndPointsRange(p, 0, end);
        if self.dy() != 0.0 {
            return true;
        }
        if self.dx() == 0.0 {
            end += 1;
            self.cubicEndPointsRange(p, 0, end);

            if self.dy() != 0.0 {
                return true;
            }
            if self.dx() == 0.0 {
                end += 1;
                self.cubicEndPointsRange(p, 0, end);

                return false;
            }
        }
        if self.dx() < 0.0 {
            return true;
        }
        end += 1;
        if NotAlmostEqualUlps(p[0].fY, p[end].fY) {
            if p[0].fY > p[end].fY {
                self.fA = DBL_EPSILON;
            }
            return true;
        }
        if end == 3 {
            return true;
        }

        if p[0].fY > p[3].fY {
            self.fA = DBL_EPSILON;
        }
        true
    }
    pub fn cubicEndPointsRange(&mut self, p: &SkDCubic, s: usize, e: usize) {
        self.fA = p[s].fY - p[e].fY;
        self.fB = p[e].fX - p[s].fX;
        self.fC = p[s].fX * p[e].fY - p[e].fX * p[s].fY;
    }
    pub fn cubicPart(&mut self, p: &SkDCubic) -> f64 {
        self.cubicEndPoints(p);
        if p[0] == p[1] || (SkDLine { fPts: [p[0], p[1]] }).nearRay(&p[2]) {
            self.pointDistance(&p[3])
        } else {
            self.pointDistance(&p[2])
        }
    }
    pub fn lineEndPoints(&mut self, p: &SkDLine) {
        self.fA = p[0].fY - p[1].fY;
        self.fB = p[1].fX - p[0].fX;
        self.fC = p[0].fX * p[1].fY - p[1].fX * p[0].fY;
    }
    pub fn quadEndPoints(&mut self, p: &SkDQuad) -> bool {
        self.quadEndPointsRange(p, 0, 1);
        if self.dy() != 0.0 {
            return true;
        }
        if self.dx() == 0.0 {
            self.quadEndPointsRange(p, 0, 2);
            return false;
        }
        if self.dx() < 0.0 {
            return true;
        }
        if p[0].fY > p[2].fY {
            self.fA = DBL_EPSILON;
        }
        true
    }
    pub fn quadEndPointsRange(&mut self, p: &SkDQuad, s: usize, e: usize) {
        self.fA = p[s].fY - p[e].fY;
        self.fB = p[e].fX - p[s].fX;
        self.fC = p[s].fX * p[e].fY - p[e].fX * p[s].fY;
    }
    pub fn quadPart(&mut self, p: &SkDQuad) -> f64 {
        self.quadEndPoints(p);
        self.pointDistance(&p[2])
    }
    pub fn normalSquared(&self) -> f64 {
        self.fA * self.fA + self.fB * self.fB
    }
    pub fn normalize(&mut self) -> bool {
        let normal = self.normalSquared().sqrt();
        if approximately_zero(normal) {
            self.fA = 0.0;
            self.fB = 0.0;
            self.fC = 0.0;
            return false;
        }
        let inv = 1.0 / normal;
        self.fA *= inv;
        self.fB *= inv;
        self.fC *= inv;
        true
    }
    pub fn cubicDistanceY(&self, p: &SkDCubic, d: &mut SkDCubic) {
        let one = 1.0 / 3.0;
        for i in 0..4 {
            d[i].fX = i as f64 * one;
            d[i].fY = self.fA * p[i].fX + self.fB * p[i].fY + self.fC;
        }
    }
    pub fn quadDistanceY(&self, p: &SkDQuad, d: &mut SkDQuad) {
        let one = 1.0 / 2.0;
        for i in 0..3 {
            d[i].fX = i as f64 * one;
            d[i].fY = self.fA * p[i].fX + self.fB * p[i].fY + self.fC;
        }
    }
    pub fn controlPtDistanceCubic(&self, p: &SkDCubic, i: usize) -> f64 {
        self.fA * p[i].fX + self.fB * p[i].fY + self.fC
    }
    pub fn controlPtDistanceQuad(&self, p: &SkDQuad) -> f64 {
        self.fA * p[1].fX + self.fB * p[1].fY + self.fC
    }
    pub fn pointDistance(&self, p: &SkDPoint) -> f64 {
        self.fA * p.fX + self.fB * p.fY + self.fC
    }
    pub fn dx(&self) -> f64 {
        self.fB
    }
    pub fn dy(&self) -> f64 {
        -self.fA
    }
}
