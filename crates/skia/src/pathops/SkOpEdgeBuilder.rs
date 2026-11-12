// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: src/pathops/SkOpEdgeBuilder.h/.cpp. Native segment admission order.
use super::{
    SkOpContour::{SkOpContour, SkOpContourBuilder, SkOpContourHead},
    SkOpGlobalState::SkOpGlobalState,
    SkPathOpsCubic::SkDCubic,
    SkPathOpsPoint::SkDPoint,
    SkPathOpsTypes::{
        SkPathOpsMask, SkPathOpsMask::*, SkPathOpsVerbToPoints, FLT_EPSILON_ORDERABLE_ERR,
    },
    SkReduceOrder::SkReduceOrder,
};
use crate::src::core::{
    SkGeometry::{SkChopQuadAtMaxCurvature, SkConic, SkFindQuadMaxCurvature},
    SkPath::{SkPath, Verb},
    SkPathPriv::SkPathPriv,
    SkPoint::SkPoint,
    SkTSort::SkTQSortNatural,
};
use core::ptr::null_mut;

pub struct SkOpEdgeBuilder {
    fGlobalState: *mut SkOpGlobalState,
    fPath: *const SkPath,
    fPathPts: Vec<SkPoint>,
    fWeights: Vec<f32>,
    fPathVerbs: Vec<Verb>,
    fContourBuilder: SkOpContourBuilder,
    fContoursHead: *mut SkOpContourHead,
    fXorMask: [SkPathOpsMask; 2],
    fSecondHalf: usize,
    fOperand: bool,
    fAllowOpenContours: bool,
    fUnparseable: bool,
}
fn force_small_to_zero(mut pt: SkPoint) -> SkPoint {
    if pt.fX.abs() < FLT_EPSILON_ORDERABLE_ERR as f32 {
        pt.fX = 0.0;
    }
    if pt.fY.abs() < FLT_EPSILON_ORDERABLE_ERR as f32 {
        pt.fY = 0.0;
    }
    pt
}
fn can_add_curve(verb: Verb, curve: &mut [SkPoint]) -> bool {
    if verb == Verb::kMove_Verb {
        return false;
    }
    for pt in &mut curve[..=SkPathOpsVerbToPoints(verb) as usize] {
        *pt = force_small_to_zero(*pt);
    }
    verb != Verb::kLine_Verb || !SkDPoint::ApproximatelyEqual(curve[0], curve[1])
}
impl SkOpEdgeBuilder {
    pub unsafe fn new(
        path: &SkPath,
        contours: *mut SkOpContourHead,
        global: *mut SkOpGlobalState,
    ) -> Self {
        let mut result = Self {
            fGlobalState: global,
            fPath: path,
            fPathPts: Vec::new(),
            fWeights: Vec::new(),
            fPathVerbs: Vec::new(),
            fContourBuilder: SkOpContourBuilder::new(contours.cast::<SkOpContour>()),
            fContoursHead: contours,
            fXorMask: [kNo_PathOpsMask; 2],
            fSecondHalf: 0,
            fOperand: false,
            fAllowOpenContours: false,
            fUnparseable: false,
        };
        result.init();
        result
    }
    pub unsafe fn init(&mut self) {
        self.fOperand = false;
        let mask = if (*self.fPath).getFillType() as u32 & 1 != 0 {
            kEvenOdd_PathOpsMask
        } else {
            kWinding_PathOpsMask
        };
        self.fXorMask = [mask; 2];
        self.fUnparseable = false;
        self.fSecondHalf = self.preFetch();
    }
    pub fn unparseable(&self) -> bool {
        self.fUnparseable
    }
    pub fn xorMask(&self) -> SkPathOpsMask {
        self.fXorMask[self.fOperand as usize]
    }
    pub unsafe fn head(&self) -> *const SkOpContour {
        self.fContoursHead.cast()
    }
    pub unsafe fn complete(&mut self) {
        self.fContourBuilder.flush();
        let contour = self.fContourBuilder.contour();
        if !contour.is_null() && (*contour).count() != 0 {
            (*contour).complete();
            self.fContourBuilder.setContour(null_mut());
        }
    }
    pub unsafe fn addOperand(&mut self, path: &SkPath) {
        self.fPathVerbs.pop();
        self.fPath = path;
        self.fXorMask[1] = if path.getFillType() as u32 & 1 != 0 {
            kEvenOdd_PathOpsMask
        } else {
            kWinding_PathOpsMask
        };
        self.preFetch();
    }
    pub unsafe fn finish(&mut self) -> bool {
        self.fOperand = false;
        if self.fUnparseable || !self.walk() {
            return false;
        }
        self.complete();
        let contour = self.fContourBuilder.contour();
        if !contour.is_null() && (*contour).count() == 0 {
            (*self.fContoursHead).remove(contour);
        }
        true
    }
    fn closeContour(&mut self, curveEnd: SkPoint, curveStart: SkPoint) {
        if !SkDPoint::ApproximatelyEqual(curveEnd, curveStart) {
            self.fPathVerbs.push(Verb::kLine_Verb);
            self.fPathPts.push(curveStart);
        } else {
            let verbs = self.fPathVerbs.len();
            let pts = self.fPathPts.len();
            if self.fPathVerbs[verbs - 1] == Verb::kLine_Verb
                && self.fPathPts[pts - 2] == curveStart
            {
                self.fPathVerbs.pop();
                self.fPathPts.pop();
            } else {
                self.fPathPts[pts - 1] = curveStart;
            }
        }
        self.fPathVerbs.push(Verb::kClose_Verb);
    }
    unsafe fn preFetch(&mut self) -> usize {
        if !(*self.fPath).isFinite() {
            self.fUnparseable = true;
            return 0;
        }
        let mut curveStart = SkPoint::default();
        let mut curve = [SkPoint::default(); 4];
        let mut lastCurve = false;
        for rec in SkPathPriv::Iterate(&*self.fPath) {
            let mut verb = match rec.fVerb {
                crate::include::core::SkPathTypes::SkPathVerb::kMove => Verb::kMove_Verb,
                crate::include::core::SkPathTypes::SkPathVerb::kLine => Verb::kLine_Verb,
                crate::include::core::SkPathTypes::SkPathVerb::kQuad => Verb::kQuad_Verb,
                crate::include::core::SkPathTypes::SkPathVerb::kConic => Verb::kConic_Verb,
                crate::include::core::SkPathTypes::SkPathVerb::kCubic => Verb::kCubic_Verb,
                crate::include::core::SkPathTypes::SkPathVerb::kClose => Verb::kClose_Verb,
            };
            let pts = rec.fPoints;
            let w = rec.fConicWeight;
            match verb {
                Verb::kMove_Verb => {
                    if !self.fAllowOpenContours && lastCurve {
                        self.closeContour(curve[0], curveStart);
                    }
                    self.fPathVerbs.push(verb);
                    curve[0] = force_small_to_zero(pts[0]);
                    self.fPathPts.push(curve[0]);
                    curveStart = curve[0];
                    lastCurve = false;
                    continue;
                }
                Verb::kLine_Verb => {
                    curve[1] = force_small_to_zero(pts[1]);
                    if SkDPoint::ApproximatelyEqual(curve[0], curve[1]) {
                        let last = *self.fPathVerbs.last().unwrap();
                        if last != Verb::kLine_Verb && last != Verb::kMove_Verb {
                            curve[0] = curve[1];
                            *self.fPathPts.last_mut().unwrap() = curve[0];
                        }
                        continue;
                    }
                }
                Verb::kQuad_Verb | Verb::kConic_Verb => {
                    curve[1] = force_small_to_zero(pts[1]);
                    curve[2] = force_small_to_zero(pts[2]);
                    let source = [curve[0], curve[1], curve[2]];
                    verb = SkReduceOrder::Quad(&source, &mut curve);
                    if rec.fVerb == crate::include::core::SkPathTypes::SkPathVerb::kConic
                        && verb == Verb::kQuad_Verb
                        && w != 1.0
                    {
                        verb = Verb::kConic_Verb;
                    } else if verb == Verb::kMove_Verb {
                        continue;
                    }
                }
                Verb::kCubic_Verb => {
                    for i in 1..4 {
                        curve[i] = force_small_to_zero(pts[i]);
                    }
                    let source = curve;
                    verb = SkReduceOrder::Cubic(&source, &mut curve);
                    if verb == Verb::kMove_Verb {
                        continue;
                    }
                }
                Verb::kClose_Verb => {
                    self.closeContour(curve[0], curveStart);
                    lastCurve = false;
                    continue;
                }
                Verb::kDone_Verb => continue,
            }
            self.fPathVerbs.push(verb);
            let count = SkPathOpsVerbToPoints(verb) as usize;
            self.fPathPts.extend_from_slice(&curve[1..=count]);
            if verb == Verb::kConic_Verb {
                self.fWeights.push(w);
            }
            curve[0] = curve[count];
            lastCurve = true;
        }
        if !self.fAllowOpenContours && lastCurve {
            self.closeContour(curve[0], curveStart);
        }
        self.fPathVerbs.push(Verb::kDone_Verb);
        self.fPathVerbs.len() - 1
    }
    unsafe fn close(&mut self) -> bool {
        self.complete();
        true
    }
    unsafe fn walk(&mut self) -> bool {
        let mut vi = 0;
        let mut pi = 0;
        let mut wi = 0;
        let mut contour = self.fContourBuilder.contour();
        let mut moveToPtrBump = 0;
        loop {
            let verb = self.fPathVerbs[vi];
            if verb == Verb::kDone_Verb {
                break;
            }
            if vi == self.fSecondHalf {
                self.fOperand = true;
            }
            vi += 1;
            let points = self.fPathPts.as_mut_ptr().add(pi);
            match verb {
                Verb::kMove_Verb => {
                    if !contour.is_null() && (*contour).count() != 0 {
                        if self.fAllowOpenContours {
                            self.complete();
                        } else if !self.close() {
                            return false;
                        }
                    }
                    if contour.is_null() {
                        contour = (*self.fContoursHead).appendContour();
                        self.fContourBuilder.setContour(contour);
                    }
                    (*contour).init(
                        self.fGlobalState,
                        self.fOperand,
                        self.fXorMask[self.fOperand as usize] == kEvenOdd_PathOpsMask,
                    );
                    pi += moveToPtrBump;
                    moveToPtrBump = 1;
                    continue;
                }
                Verb::kLine_Verb => {
                    self.fContourBuilder.addLine(points);
                }
                Verb::kQuad_Verb => {
                    let pts = &*points.cast::<[SkPoint; 3]>();
                    let vec1 = pts[1] - pts[0];
                    let vec2 = pts[2] - pts[1];
                    let mut added = false;
                    if vec1.dot(vec2) < 0.0 {
                        let mut pair = [SkPoint::default(); 5];
                        if SkChopQuadAtMaxCurvature(pts, &mut pair) != 1 {
                            if pair.iter().any(|p| !p.fX.is_finite() || !p.fY.is_finite()) {
                                return false;
                            }
                            for pt in &mut pair {
                                *pt = force_small_to_zero(*pt);
                            }
                            let mut c1 = [SkPoint::default(); 3];
                            let mut c2 = [SkPoint::default(); 3];
                            let v1 = SkReduceOrder::Quad((&pair[..3]).try_into().unwrap(), &mut c1);
                            let v2 = SkReduceOrder::Quad((&pair[2..]).try_into().unwrap(), &mut c2);
                            let mut curve1 = if v1 == Verb::kLine_Verb {
                                c1
                            } else {
                                [pair[0], pair[1], pair[2]]
                            };
                            let mut curve2 = if v2 == Verb::kLine_Verb {
                                c2
                            } else {
                                [pair[2], pair[3], pair[4]]
                            };
                            if can_add_curve(v1, &mut curve1) && can_add_curve(v2, &mut curve2) {
                                self.fContourBuilder.addCurve(v1, curve1.as_ptr(), 1.0);
                                self.fContourBuilder.addCurve(v2, curve2.as_ptr(), 1.0);
                                added = true;
                            }
                        }
                    }
                    if !added {
                        self.fContourBuilder.addQuad(points);
                    }
                }
                Verb::kConic_Verb => {
                    let pts = &*points.cast::<[SkPoint; 3]>();
                    let weight = self.fWeights[wi];
                    wi += 1;
                    let vec1 = pts[1] - pts[0];
                    let vec2 = pts[2] - pts[1];
                    let mut added = false;
                    if vec1.dot(vec2) < 0.0 {
                        let t = SkFindQuadMaxCurvature(pts);
                        if t > 0.0 && t < 1.0 {
                            let conic = SkConic::new(pts, weight);
                            let mut pair = [SkConic::default(); 2];
                            if !conic.chopAt(t, &mut pair) {
                                self.fContourBuilder.addConic(points, weight);
                                added = true;
                            } else {
                                let mut c1 = [SkPoint::default(); 3];
                                let mut c2 = [SkPoint::default(); 3];
                                let v1 = SkReduceOrder::Conic(&pair[0], &mut c1);
                                let v2 = SkReduceOrder::Conic(&pair[1], &mut c2);
                                let mut curve1 = if v1 == Verb::kLine_Verb {
                                    c1
                                } else {
                                    pair[0].fPts
                                };
                                let mut curve2 = if v2 == Verb::kLine_Verb {
                                    c2
                                } else {
                                    pair[1].fPts
                                };
                                if can_add_curve(v1, &mut curve1) && can_add_curve(v2, &mut curve2)
                                {
                                    self.fContourBuilder
                                        .addCurve(v1, curve1.as_ptr(), pair[0].fW);
                                    self.fContourBuilder
                                        .addCurve(v2, curve2.as_ptr(), pair[1].fW);
                                    added = true;
                                }
                            }
                        }
                    }
                    if !added {
                        self.fContourBuilder.addConic(points, weight);
                    }
                }
                Verb::kCubic_Verb => {
                    let pts = &*points.cast::<[SkPoint; 4]>();
                    let mut splitT = [0.0; 3];
                    let breaks = SkDCubic::ComplexBreak(pts, &mut splitT);
                    if breaks == 0 {
                        self.fContourBuilder.addCubic(points);
                    } else if !self.addSplitCubics(pts, &mut splitT, breaks) {
                        return false;
                    }
                }
                Verb::kClose_Verb => {
                    if !self.close() {
                        return false;
                    }
                    contour = null_mut();
                    continue;
                }
                _ => return false,
            }

            pi += SkPathOpsVerbToPoints(verb) as usize;
        }
        self.fContourBuilder.flush();
        if !contour.is_null()
            && (*contour).count() != 0
            && !self.fAllowOpenContours
            && !self.close()
        {
            return false;
        }
        true
    }
    unsafe fn addSplitCubics(
        &mut self,
        pts: &[SkPoint; 4],
        splitT: &mut [f32; 3],
        breaks: usize,
    ) -> bool {
        #[derive(Clone, Copy)]
        struct Splitsville {
            t: [f64; 2],
            pts: [SkPoint; 4],
            reduced: [SkPoint; 4],
            verb: Verb,
            canAdd: bool,
        }

        SkTQSortNatural(&mut splitT[..breaks]);
        let mut splits = [Splitsville {
            t: [0.0; 2],
            pts: [SkPoint::default(); 4],
            reduced: [SkPoint::default(); 4],
            verb: Verb::kMove_Verb,
            canAdd: false,
        }; 4];
        for index in 0..=breaks {
            let split = &mut splits[index];
            split.t = [
                if index > 0 {
                    splitT[index - 1] as f64
                } else {
                    0.0
                },
                if index < breaks {
                    splitT[index] as f64
                } else {
                    1.0
                },
            ];
            let part = SkDCubic::SubDivide(pts, split.t[0], split.t[1]);
            if !part.toFloatPoints(&mut split.pts) {
                return false;
            }
            split.verb = SkReduceOrder::Cubic(&split.pts, &mut split.reduced);
            let curve = if split.verb == Verb::kCubic_Verb {
                &mut split.pts
            } else {
                &mut split.reduced
            };
            split.canAdd = can_add_curve(split.verb, curve);
        }
        for index in 0..=breaks {
            if !splits[index].canAdd {
                continue;
            }
            let mut prior = index;
            while prior > 0 && !splits[prior - 1].canAdd {
                prior -= 1;
            }
            if prior < index {
                splits[index].t[0] = splits[prior].t[0];
                splits[index].pts[0] = splits[prior].pts[0];
            }
            let mut next = index;
            let limit = breaks.min(splits.len() - 1);
            while next < limit && !splits[next + 1].canAdd {
                next += 1;
            }
            if next > index {
                splits[index].t[1] = splits[next].t[1];
                splits[index].pts[3] = splits[next].pts[3];
            }
            let split = &mut splits[index];
            if prior < index || next > index {
                split.verb = SkReduceOrder::Cubic(&split.pts, &mut split.reduced);
            }
            let curve = if split.verb == Verb::kCubic_Verb {
                &mut split.pts
            } else {
                &mut split.reduced
            };
            if !can_add_curve(split.verb, curve) {
                return false;
            }
            self.fContourBuilder
                .addCurve(split.verb, curve.as_ptr(), 1.0);
        }
        true
    }
}
