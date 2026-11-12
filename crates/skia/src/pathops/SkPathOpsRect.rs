// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsRect.h/.cpp.
use super::{
    SkPathOpsConic::SkDConic, SkPathOpsCubic::SkDCubic, SkPathOpsPoint::SkDPoint,
    SkPathOpsQuad::SkDQuad, SkPathOpsTypes::approximately_between,
};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkDRect {
    pub fLeft: f64,
    pub fTop: f64,
    pub fRight: f64,
    pub fBottom: f64,
}
pub trait SkDRectCurve {
    fn boundsInto(&self, rect: &mut SkDRect);
}
impl SkDRectCurve for SkDQuad {
    fn boundsInto(&self, r: &mut SkDRect) {
        r.setBoundsQuadSub(self, self, 0.0, 1.0)
    }
}
impl SkDRectCurve for SkDConic {
    fn boundsInto(&self, r: &mut SkDRect) {
        r.setBoundsConicSub(self, self, 0.0, 1.0)
    }
}
impl SkDRectCurve for SkDCubic {
    fn boundsInto(&self, r: &mut SkDRect) {
        r.setBoundsCubicSub(self, self, 0.0, 1.0)
    }
}
impl SkDRect {
    pub fn add(&mut self, p: &SkDPoint) {
        self.fLeft = self.fLeft.min(p.fX);
        self.fTop = self.fTop.min(p.fY);
        self.fRight = self.fRight.max(p.fX);
        self.fBottom = self.fBottom.max(p.fY);
    }
    pub fn contains(&self, p: &SkDPoint) -> bool {
        approximately_between(self.fLeft, p.fX, self.fRight)
            && approximately_between(self.fTop, p.fY, self.fBottom)
    }
    pub fn intersects(&self, r: &Self) -> bool {
        r.fLeft <= self.fRight
            && self.fLeft <= r.fRight
            && r.fTop <= self.fBottom
            && self.fTop <= r.fBottom
    }
    pub fn set(&mut self, p: &SkDPoint) {
        self.fLeft = p.fX;
        self.fRight = p.fX;
        self.fTop = p.fY;
        self.fBottom = p.fY;
    }
    pub fn width(&self) -> f64 {
        self.fRight - self.fLeft
    }
    pub fn height(&self) -> f64 {
        self.fBottom - self.fTop
    }
    pub fn valid(&self) -> bool {
        self.fLeft <= self.fRight && self.fTop <= self.fBottom
    }
    pub fn setBounds<C: SkDRectCurve + ?Sized>(&mut self, c: &C) {
        c.boundsInto(self)
    }
    pub fn setBoundsQuadSub(&mut self, c: &SkDQuad, s: &SkDQuad, start: f64, end: f64) {
        self.set(&s[0]);
        self.add(&s[2]);
        let mut ts = [0.0; 2];
        let mut roots = 0;
        if !s.monotonicInX() {
            roots = SkDQuad::FindExtrema(&s.fPts.map(|p| p.fX), &mut ts);
        }
        if !s.monotonicInY() {
            roots += SkDQuad::FindExtrema(&s.fPts.map(|p| p.fY), &mut ts[roots..]);
        }
        for &v in &ts[..roots] {
            self.add(&c.ptAtT(start + (end - start) * v));
        }
    }
    pub fn setBoundsConicSub(&mut self, c: &SkDConic, s: &SkDConic, start: f64, end: f64) {
        self.set(&s[0]);
        self.add(&s[2]);
        let mut ts = [0.0; 2];
        let mut roots = 0;
        if !s.monotonicInX() {
            roots = SkDConic::FindExtrema(&s.fPts.fPts.map(|p| p.fX), s.fWeight, &mut ts);
        }
        if !s.monotonicInY() {
            roots += SkDConic::FindExtrema(&s.fPts.fPts.map(|p| p.fY), s.fWeight, &mut ts[roots..]);
        }
        for &v in &ts[..roots] {
            self.add(&c.ptAtT(start + (end - start) * v));
        }
    }
    pub fn setBoundsCubicSub(&mut self, c: &SkDCubic, s: &SkDCubic, start: f64, end: f64) {
        self.set(&s[0]);
        self.add(&s[3]);
        let mut ts = [0.0; 4];
        let mut roots = 0;
        if !s.monotonicInX() {
            roots = SkDCubic::FindExtrema(&s.fPts.map(|p| p.fX), &mut ts);
        }
        if !s.monotonicInY() {
            roots += SkDCubic::FindExtrema(&s.fPts.map(|p| p.fY), &mut ts[roots..]);
        }
        for &v in &ts[..roots] {
            self.add(&c.ptAtT(start + (end - start) * v));
        }
    }
}
