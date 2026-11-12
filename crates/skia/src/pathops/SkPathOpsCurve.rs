// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsCurve.h/.cpp.
use super::{
    SkIntersections::SkIntersections,
    SkPathOpsBounds::SkPathOpsBounds,
    SkPathOpsConic::SkDConic,
    SkPathOpsCubic::SkDCubic,
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsQuad::SkDQuad,
    SkPathOpsRect::SkDRect,
    SkPathOpsTypes::*,
};
use crate::src::core::{SkPath::Verb, SkPoint::SkPoint};
use core::ops::{Index, IndexMut};
#[repr(C)]
#[derive(Clone, Copy)]
pub union SkDCurve {
    pub fLine: SkDLine,
    pub fQuad: SkDQuad,
    pub fConic: SkDConic,
    pub fCubic: SkDCubic,
}
impl Default for SkDCurve {
    fn default() -> Self {
        Self {
            fCubic: SkDCubic::default(),
        }
    }
}
impl Index<usize> for SkDCurve {
    type Output = SkDPoint;
    fn index(&self, n: usize) -> &SkDPoint {
        unsafe { &self.fCubic[n] }
    }
}
impl IndexMut<usize> for SkDCurve {
    fn index_mut(&mut self, n: usize) -> &mut SkDPoint {
        unsafe { &mut self.fCubic[n] }
    }
}
#[derive(Clone, Copy, Default)]
pub struct SkOpCurve {
    pub fPts: [SkPoint; 4],
    pub fWeight: f32,
}
impl Index<usize> for SkOpCurve {
    type Output = SkPoint;
    fn index(&self, n: usize) -> &SkPoint {
        &self.fPts[n]
    }
}
impl SkOpCurve {
    pub fn setQuad(&mut self, q: &SkDQuad) {
        for i in 0..3 {
            self.fPts[i] = q[i].asSkPoint();
        }
    }
    pub fn setCubic(&mut self, c: &SkDCubic) {
        for i in 0..4 {
            self.fPts[i] = c[i].asSkPoint();
        }
    }
}
#[derive(Clone, Copy, Default)]
pub struct SkDCurveSweep {
    pub fCurve: SkDCurve,
    pub fSweep: [SkDVector; 2],
    fIsCurve: bool,
    fOrdered: bool,
}
impl SkDCurveSweep {
    pub fn isCurve(&self) -> bool {
        self.fIsCurve
    }
    pub fn isOrdered(&self) -> bool {
        self.fOrdered
    }
    pub fn setCurveHullSweep(&mut self, verb: Verb) {
        self.fOrdered = true;
        self.fSweep[0] = self.fCurve[1] - self.fCurve[0];
        if verb == Verb::kLine_Verb {
            self.fSweep[1] = self.fSweep[0];
            self.fIsCurve = false;
            return;
        }
        self.fSweep[1] = self.fCurve[2] - self.fCurve[0];
        let mut max: f64 = 0.0;
        for i in 0..=SkPathOpsVerbToPoints(verb) as usize {
            max = max.max(self.fCurve[i].fX.abs().max(self.fCurve[i].fY.abs()));
        }
        if verb != Verb::kCubic_Verb {
            if roughly_zero_when_compared_to(self.fSweep[0].fX, max)
                && roughly_zero_when_compared_to(self.fSweep[0].fY, max)
            {
                self.fSweep[0] = self.fSweep[1];
            }
        } else {
            let third = self.fCurve[3] - self.fCurve[0];
            if self.fSweep[0].fX == 0.0 && self.fSweep[0].fY == 0.0 {
                self.fSweep[0] = self.fSweep[1];
                self.fSweep[1] = third;
                if roughly_zero_when_compared_to(self.fSweep[0].fX, max)
                    && roughly_zero_when_compared_to(self.fSweep[0].fY, max)
                {
                    self.fSweep[0] = self.fSweep[1];
                    self.fCurve[1] = self.fCurve[3];
                }
            } else {
                let s1x3 = self.fSweep[0].crossCheck(&third);
                let s3x2 = third.crossCheck(&self.fSweep[1]);
                if s1x3 * s3x2 < 0.0 {
                    let s2x1 = self.fSweep[1].crossCheck(&self.fSweep[0]);

                    if s3x2 * s2x1 < 0.0 {
                        self.fSweep[0] = self.fSweep[1];
                        self.fOrdered = false;
                    }
                    self.fSweep[1] = third;
                }
            }
        }
        self.fIsCurve = self.fSweep[0].crossCheck(&self.fSweep[1]) != 0.0;
    }
}
impl SkDCurve {
    pub fn nearPoint(&self, verb: Verb, xy: &SkDPoint, opp: &SkDPoint) -> f64 {
        let count = SkPathOpsVerbToPoints(verb) as usize;
        let mut minx = self[0].fX;
        let mut maxx = minx;
        let mut miny = self[0].fY;
        let mut maxy = miny;
        for i in 1..=count {
            minx = minx.min(self[i].fX);
            maxx = maxx.max(self[i].fX);
        }
        if !AlmostBetweenUlps(minx, xy.fX, maxx) {
            return -1.0;
        }
        for i in 1..=count {
            miny = miny.min(self[i].fY);
            maxy = maxy.max(self[i].fY);
        }
        if !AlmostBetweenUlps(miny, xy.fY, maxy) {
            return -1.0;
        }
        let mut intersections = SkIntersections::default();
        let perp = SkDLine {
            fPts: [
                *xy,
                SkDPoint {
                    fX: xy.fX + opp.fY - xy.fY,
                    fY: xy.fY + xy.fX - opp.fX,
                },
            ],
        };
        unsafe {
            CurveDIntersectRay(verb, self, &perp, &mut intersections);
        }
        let mut minIndex = None;
        let mut minDist = f32::MAX as f64;
        for i in 0..intersections.used() {
            let dist = xy.distance(intersections.pt(i));
            if minDist > dist {
                minDist = dist;
                minIndex = Some(i);
            }
        }
        let Some(index) = minIndex else {
            return -1.0;
        };
        let largest = maxx.max(maxy).max(-minx.min(miny));
        if !AlmostEqualUlps_Pin(largest, largest + minDist) {
            return -1.0;
        }
        SkPinT(intersections[0][index])
    }
    pub unsafe fn setConicBounds(
        &self,
        curve: &[SkPoint; 3],
        weight: f32,
        start: f64,
        end: f64,
        bounds: &mut SkPathOpsBounds,
    ) {
        let mut d = SkDConic::default();
        d.set(curve, weight);
        let mut rect = SkDRect::default();
        rect.setBoundsConicSub(&d, &self.fConic, start, end);
        bounds.setLTRB(
            rect.fLeft as f32,
            rect.fTop as f32,
            rect.fRight as f32,
            rect.fBottom as f32,
        );
    }
    pub unsafe fn setCubicBounds(
        &self,
        curve: &[SkPoint; 4],
        _weight: f32,
        start: f64,
        end: f64,
        bounds: &mut SkPathOpsBounds,
    ) {
        let mut d = SkDCubic::default();
        d.set(curve);
        let mut rect = SkDRect::default();
        rect.setBoundsCubicSub(&d, &self.fCubic, start, end);
        bounds.setLTRB(
            rect.fLeft as f32,
            rect.fTop as f32,
            rect.fRight as f32,
            rect.fBottom as f32,
        );
    }
    pub unsafe fn setQuadBounds(
        &self,
        curve: &[SkPoint; 3],
        _weight: f32,
        start: f64,
        end: f64,
        bounds: &mut SkPathOpsBounds,
    ) {
        let mut d = SkDQuad::default();
        d.set(curve);
        let mut rect = SkDRect::default();
        rect.setBoundsQuadSub(&d, &self.fQuad, start, end);
        bounds.setLTRB(
            rect.fLeft as f32,
            rect.fTop as f32,
            rect.fRight as f32,
            rect.fBottom as f32,
        );
    }
}
pub unsafe fn CurveDPointAtT(verb: Verb, p: *const SkPoint, w: f32, t: f64) -> SkDPoint {
    let c = curveFromPoints(verb, p, w);
    CurveDDPointAtT(verb, &c, t)
}
pub unsafe fn CurveDDPointAtT(verb: Verb, c: &SkDCurve, t: f64) -> SkDPoint {
    match verb {
        Verb::kLine_Verb => c.fLine.ptAtT(t),
        Verb::kQuad_Verb => c.fQuad.ptAtT(t),
        Verb::kConic_Verb => c.fConic.ptAtT(t),
        Verb::kCubic_Verb => c.fCubic.ptAtT(t),
        _ => panic!("invalid curve verb"),
    }
}
pub unsafe fn CurvePointAtT(verb: Verb, p: *const SkPoint, w: f32, t: f64) -> SkPoint {
    CurveDPointAtT(verb, p, w, t).asSkPoint()
}
pub unsafe fn CurveDSlopeAtT(verb: Verb, p: *const SkPoint, w: f32, t: f64) -> SkDVector {
    let c = curveFromPoints(verb, p, w);
    CurveDDSlopeAtT(verb, &c, t)
}
pub unsafe fn CurveDDSlopeAtT(verb: Verb, c: &SkDCurve, t: f64) -> SkDVector {
    match verb {
        Verb::kLine_Verb => c.fLine[1] - c.fLine[0],
        Verb::kQuad_Verb => c.fQuad.dxdyAtT(t),
        Verb::kConic_Verb => c.fConic.dxdyAtT(t),
        Verb::kCubic_Verb => c.fCubic.dxdyAtT(t),
        _ => panic!("invalid curve verb"),
    }
}
pub unsafe fn CurveSlopeAtT(verb: Verb, p: *const SkPoint, w: f32, t: f64) -> SkPoint {
    if verb == Verb::kLine_Verb {
        return *p.add(1) - *p;
    }
    CurveDSlopeAtT(verb, p, w, t).asSkVector()
}
pub unsafe fn CurveIsVertical(verb: Verb, p: *const SkPoint, w: f32, start: f64, end: f64) -> bool {
    let c = curveFromPoints(verb, p, w);
    match verb {
        Verb::kLine_Verb => AlmostEqualUlps(c.fLine.ptAtT(start).fX, c.fLine.ptAtT(end).fX),
        Verb::kQuad_Verb => {
            let dst = c.fQuad.subDivide(start, end);
            AlmostEqualUlps(dst[0].fX, dst[1].fX) && AlmostEqualUlps(dst[1].fX, dst[2].fX)
        }
        Verb::kConic_Verb => {
            let dst = c.fConic.subDivide(start, end);
            AlmostEqualUlps(dst[0].fX, dst[1].fX) && AlmostEqualUlps(dst[1].fX, dst[2].fX)
        }
        Verb::kCubic_Verb => {
            let dst = c.fCubic.subDivide(start, end);
            AlmostEqualUlps(dst[0].fX, dst[1].fX)
                && AlmostEqualUlps(dst[1].fX, dst[2].fX)
                && AlmostEqualUlps(dst[2].fX, dst[3].fX)
        }
        _ => panic!("invalid curve verb"),
    }
}
pub unsafe fn CurveIntersectRay(
    verb: Verb,
    p: *const SkPoint,
    w: f32,
    ray: &SkDLine,
    i: &mut SkIntersections,
) -> usize {
    let c = curveFromPoints(verb, p, w);
    CurveDIntersectRay(verb, &c, ray, i)
}
pub unsafe fn CurveDIntersectRay(
    verb: Verb,
    c: &SkDCurve,
    ray: &SkDLine,
    i: &mut SkIntersections,
) -> usize {
    match verb {
        Verb::kLine_Verb => i.intersectRayLine(&c.fLine, ray),
        Verb::kQuad_Verb => i.intersectRayQuad(&c.fQuad, ray),
        Verb::kConic_Verb => i.intersectRayConic(&c.fConic, ray),
        Verb::kCubic_Verb => i.intersectRayCubic(&c.fCubic, ray),
        _ => panic!("invalid curve verb"),
    }
}
pub unsafe fn curveFromPoints(verb: Verb, p: *const SkPoint, w: f32) -> SkDCurve {
    let mut c = SkDCurve::default();
    match verb {
        Verb::kLine_Verb => {
            c.fLine.set(&*p.cast::<[SkPoint; 2]>());
        }
        Verb::kQuad_Verb => {
            c.fQuad.set(&*p.cast::<[SkPoint; 3]>());
        }
        Verb::kConic_Verb => {
            c.fConic.set(&*p.cast::<[SkPoint; 3]>(), w);
        }
        Verb::kCubic_Verb => {
            c.fCubic.set(&*p.cast::<[SkPoint; 4]>());
        }
        _ => panic!("invalid curve verb"),
    };
    c
}
pub unsafe fn curve_intercept(
    verb: Verb,
    axis: usize,
    p: *const SkPoint,
    w: f32,
    value: f32,
    roots: &mut [f64; 3],
) -> usize {
    let c = curveFromPoints(verb, p, w);
    match verb {
        Verb::kLine_Verb => {
            let line = c.fLine;
            let a = if axis == 0 { line[0].fY } else { line[0].fX };
            let b = if axis == 0 { line[1].fY } else { line[1].fX };
            if a == b {
                return 0;
            }
            roots[0] = if axis == 0 {
                SkIntersections::HorizontalInterceptLine(&line, value as f64)
            } else {
                SkIntersections::VerticalInterceptLine(&line, value as f64)
            };
            between(0.0, roots[0], 1.0) as usize
        }
        Verb::kQuad_Verb => {
            if axis == 0 {
                c.fQuad.horizontalIntersect(value as f64, roots)
            } else {
                c.fQuad.verticalIntersect(value as f64, roots)
            }
        }
        Verb::kConic_Verb => {
            if axis == 0 {
                SkIntersections::HorizontalInterceptConic(&c.fConic, value, roots)
            } else {
                SkIntersections::VerticalInterceptConic(&c.fConic, value, roots)
            }
        }
        Verb::kCubic_Verb => {
            if axis == 0 {
                c.fCubic.horizontalIntersect(value as f64, roots)
            } else {
                c.fCubic.verticalIntersect(value as f64, roots)
            }
        }
        _ => panic!("invalid curve verb"),
    }
}
