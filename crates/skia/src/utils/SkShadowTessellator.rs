// Copyright 2017 Google LLC. BSD-style license.
use crate::include::core::{SkPathTypes::SkPathVerb, SkRefCnt::sk_sp};
use crate::src::core::SkDrawShadowInfo::SkDrawShadowMetrics;
use crate::src::core::SkPointPriv::SkPointPriv;
use crate::src::core::{
    SkColor::*,
    SkGeometry::{SkAutoConicToQuads, SkConic},
    SkMatrix::SkMatrix,
    SkPath::{Iter, SkPath},
    SkPoint::SkPoint,
    SkPoint3::SkPoint3,
    SkRect::SkRect,
    SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyZero},
    SkVertices::{SkVertices, VertexMode},
};
use core::mem::ManuallyDrop;
// The real polygon producers remain an explicitly requested dependency. No
// substitute offsetter, simplifier, or triangulator is provided here.
use crate::src::utils::SkPolyUtils::{
    SkComputeRadialSteps, SkInsetConvexPolygon, SkIsSimplePolygon, SkOffsetSimplePolygon,
    SkTriangulateSimplePolygon,
};
// Source: src/utils/SkShadowTessellator.h:25-25
pub type HeightFunc<'a> = Box<dyn Fn(f32, f32) -> f32 + 'a>;
// Source: src/utils/SkShadowTessellator.cpp:107-142
// Vec replaces SkTDArray. Scratch POD fields not initialized by C++ are initialized
// here, but are written before use on the same native branches. Signed sentinel
// indices retain i32, while known nonnegative ring cursors use usize.
// Source: src/utils/SkShadowTessellator.cpp:44-44
// Source: src/utils/SkShadowTessellator.cpp:43-43
struct SkBaseShadowTessellator {
    fZPlaneParams: SkPoint3,
    fPointBuffer: ManuallyDrop<Vec<SkPoint>>,
    fPositions: ManuallyDrop<Vec<SkPoint>>,
    fColors: ManuallyDrop<Vec<SkColor>>,
    fIndices: ManuallyDrop<Vec<u16>>,
    fPathPolygon: ManuallyDrop<Vec<SkPoint>>,
    fClipPolygon: ManuallyDrop<Vec<SkPoint>>,
    fClipVectors: ManuallyDrop<Vec<SkPoint>>,
    fPathBounds: SkRect,
    fCentroid: SkPoint,
    fArea: f32,
    fLastArea: f32,
    fLastCross: f32,
    fFirstVertexIndex: i32,
    fFirstOutset: SkPoint,
    fFirstPoint: SkPoint,
    fSucceeded: bool,
    fTransparent: bool,
    fIsConvex: bool,
    fValidUmbra: bool,
    fDirection: f32,
    fPrevUmbraIndex: i32,
    fCurrUmbraIndex: usize,
    fCurrClipIndex: usize,
    fPrevUmbraOutside: bool,
    fFirstUmbraOutside: bool,
    fPrevOutset: SkPoint,
    fPrevPoint: SkPoint,
}
// Native derived/base destruction releases SkTDArray members in reverse field
// order. ManuallyDrop retains that order without moving the field declarations.
// Source: src/utils/SkShadowTessellator.cpp:47-47
impl Drop for SkBaseShadowTessellator {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fClipVectors);
            ManuallyDrop::drop(&mut self.fClipPolygon);
            ManuallyDrop::drop(&mut self.fPathPolygon);
            ManuallyDrop::drop(&mut self.fIndices);
            ManuallyDrop::drop(&mut self.fColors);
            ManuallyDrop::drop(&mut self.fPositions);
            ManuallyDrop::drop(&mut self.fPointBuffer);
        }
    }
}
impl SkBaseShadowTessellator {
    // Source: src/utils/SkShadowTessellator.cpp:49-56
    fn releaseVertices(&self) -> sk_sp<SkVertices> {
        if !self.fSucceeded {
            return Default::default();
        }
        unsafe {
            SkVertices::MakeCopy(
                VertexMode::kTriangles_VertexMode,
                self.fPositions.len() as i32,
                self.fPositions.as_ptr(),
                core::ptr::null(),
                self.fColors.as_ptr(),
                self.fIndices.len() as i32,
                self.fIndices.as_ptr(),
            )
        }
    }
    // Source: src/utils/SkShadowTessellator.cpp:59-61
    const kMinHeight: f32 = 0.1;
    const kPenumbraColor: SkColor = SK_ColorTRANSPARENT;
    const kUmbraColor: SkColor = SK_ColorBLACK;
    // Source: src/utils/SkShadowTessellator.cpp:63-64
    fn vertexCount(&self) -> usize {
        self.fPositions.len()
    }
    fn indexCount(&self) -> usize {
        self.fIndices.len()
    }
    // Source: src/utils/SkShadowTessellator.cpp:103-105
    fn heightFunc(&self, x: f32, y: f32) -> f32 {
        self.fZPlaneParams.fX * x + self.fZPlaneParams.fY * y + self.fZPlaneParams.fZ
    }
}
// Source: src/utils/SkShadowTessellator.cpp:145-157
fn compute_normal(p0: SkPoint, p1: SkPoint, dir: f32, newNormal: &mut SkPoint) -> bool {
    let mut normal = SkPoint::make((p0.fY - p1.fY) * dir, (p1.fX - p0.fX) * dir);
    if !normal.normalize() {
        return false;
    }
    *newNormal = normal;
    true
}
// Source: src/utils/SkShadowTessellator.cpp:159-165
fn duplicate_pt(p0: SkPoint, p1: SkPoint) -> bool {
    let kClose = 1.0 / 16.0;
    SkPointPriv::DistanceToSqd(p0, p1) < kClose * kClose
}
// Source: src/utils/SkShadowTessellator.cpp:167-171
fn perp_dot(p0: SkPoint, p1: SkPoint, p2: SkPoint) -> f32 {
    (p1 - p0).cross(p2 - p1)
}
impl SkBaseShadowTessellator {
    // Source: src/utils/SkShadowTessellator.cpp:46-46
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:173-193
    fn new(z: SkPoint3, bounds: SkRect, transparent: bool) -> Self {
        Self {
            fZPlaneParams: z,
            fPointBuffer: ManuallyDrop::new(Vec::new()),
            fPositions: ManuallyDrop::new(Vec::new()),
            fColors: ManuallyDrop::new(Vec::new()),
            fIndices: ManuallyDrop::new(Vec::new()),
            fPathPolygon: ManuallyDrop::new(Vec::new()),
            fClipPolygon: ManuallyDrop::new(Vec::new()),
            fClipVectors: ManuallyDrop::new(Vec::new()),
            fPathBounds: bounds,
            fCentroid: SkPoint::make(0.0, 0.0),
            fArea: 0.0,
            fLastArea: 0.0,
            fLastCross: 0.0,
            fFirstVertexIndex: -1,
            fFirstOutset: SkPoint::default(),
            fFirstPoint: SkPoint::default(),
            fSucceeded: false,
            fTransparent: transparent,
            fIsConvex: true,
            fValidUmbra: true,
            fDirection: 1.0,
            fPrevUmbraIndex: -1,
            fCurrUmbraIndex: 0,
            fCurrClipIndex: 0,
            fPrevUmbraOutside: false,
            fFirstUmbraOutside: false,
            fPrevOutset: SkPoint::default(),
            fPrevPoint: SkPoint::default(),
        }
    }
    // Source: src/utils/SkShadowTessellator.cpp:67-67
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:195-216
    fn accumulateCentroid(&mut self, curr: SkPoint, next: SkPoint) -> bool {
        if duplicate_pt(curr, next) {
            return false;
        }
        crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 200; @condition b"!fPathPolygon.empty()\0".as_ptr(); !self.fPathPolygon.is_empty());
        let v0 = curr - self.fPathPolygon[0];
        let v1 = next - self.fPathPolygon[0];
        let quadArea = v0.cross(v1);
        self.fCentroid.fX += (v0.fX + v1.fX) * quadArea;
        self.fCentroid.fY += (v0.fY + v1.fY) * quadArea;
        self.fArea += quadArea;
        if quadArea * self.fLastArea < 0.0 {
            self.fIsConvex = false;
        }
        if quadArea != 0.0 {
            self.fLastArea = quadArea;
        }
        true
    }
    // Source: src/utils/SkShadowTessellator.cpp:68-68
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:218-236
    fn checkConvexity(&mut self, p0: SkPoint, p1: SkPoint, p2: SkPoint) -> bool {
        let cross = perp_dot(p0, p1, p2);
        if SkScalarNearlyZero(cross, SK_ScalarNearlyZero) {
            return false;
        }
        if self.fLastCross * cross < 0.0 {
            self.fIsConvex = false;
        }
        if cross != 0.0 {
            self.fLastCross = cross;
        }
        true
    }
    // Source: src/utils/SkShadowTessellator.cpp:69-69
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:238-261
    fn finishPathPolygon(&mut self) {
        let n = self.fPathPolygon.len();
        if n > 1 && !self.accumulateCentroid(self.fPathPolygon[n - 1], self.fPathPolygon[0]) {
            self.fPathPolygon.pop();
        }
        let n = self.fPathPolygon.len();
        if n > 2 {
            self.fCentroid *=
                crate::src::core::SkFloatingPoint::sk_ieee_float_divide(1.0, 3.0 * self.fArea);
            self.fCentroid += self.fPathPolygon[0];
            if !self.checkConvexity(
                self.fPathPolygon[n - 2],
                self.fPathPolygon[n - 1],
                self.fPathPolygon[0],
            ) {
                self.fPathPolygon[0] = self.fPathPolygon[n - 1];
                self.fPathPolygon.pop();
            }
        }
        self.fDirection = if self.fArea > 0.0 { -1.0 } else { 1.0 };
    }
    // Source: src/utils/SkShadowTessellator.cpp:72-72
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:263-383
    fn computeConvexShadow(&mut self, mut inset: f32, outset: f32, doClip: bool) -> bool {
        if doClip {
            self.computeClipVectorsAndTestCentroid();
        }
        let mut umbraColor = Self::kUmbraColor;
        let mut minDistSq = SkPointPriv::DistanceToLineSegmentBetweenSqd(
            self.fCentroid,
            self.fPathPolygon[0],
            self.fPathPolygon[1],
        );
        for i in 1..self.fPathPolygon.len() {
            let j = if i == self.fPathPolygon.len() - 1 {
                0
            } else {
                i + 1
            };
            let distSq = SkPointPriv::DistanceToLineSegmentBetweenSqd(
                self.fCentroid,
                self.fPathPolygon[i],
                self.fPathPolygon[j],
            );
            if distSq < minDistSq {
                minDistSq = distSq;
            }
        }
        let mut insetPolygon = Vec::new();
        if inset > SK_ScalarNearlyZero {
            let kTolerance = 1.0e-2;
            if minDistSq < (inset + kTolerance) * (inset + kTolerance) {
                let newInset = minDistSq.sqrt() - kTolerance;
                let ratio = 128.0 * (newInset / inset + 1.0);
                crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 294; @condition b"SkIsFinite(ratio)\0".as_ptr(); ratio.is_finite());
                umbraColor = crate::src::core::SkColorData::SkPMLerp(
                    Self::kUmbraColor,
                    Self::kPenumbraColor,
                    ratio as u32,
                );
                inset = newInset;
            }
            if !SkInsetConvexPolygon(&self.fPathPolygon, inset, &mut insetPolygon) {
                self.fValidUmbra = false;
            }
        }
        // This raw slice keeps the native borrowed polygon without allocating a copy.
        // The remaining methods mutate output arrays/cursors, never fPathPolygon.
        let umbraPolygon = if inset > SK_ScalarNearlyZero {
            insetPolygon.as_slice()
        } else {
            unsafe {
                core::slice::from_raw_parts(self.fPathPolygon.as_ptr(), self.fPathPolygon.len())
            }
        };
        if self.fTransparent {
            self.fPositions.push(self.fCentroid);
            self.fColors.push(umbraColor);
        }
        self.fCurrUmbraIndex = 0;
        let polyCount = self.fPathPolygon.len();
        let mut firstOutset = SkPoint::default();
        if !compute_normal(
            self.fPathPolygon[polyCount - 1],
            self.fPathPolygon[0],
            self.fDirection,
            &mut firstOutset,
        ) {
            return false;
        }
        self.fFirstOutset = firstOutset * outset;
        self.fFirstPoint = self.fPathPolygon[polyCount - 1];
        self.fFirstVertexIndex = self.fPositions.len() as i32;
        self.fPrevOutset = self.fFirstOutset;
        self.fPrevPoint = self.fFirstPoint;
        self.fPrevUmbraIndex = -1;
        let (_, index) = self.addInnerPoint(self.fFirstPoint, umbraColor, umbraPolygon);
        self.fPrevUmbraIndex = index;
        if !self.fTransparent && doClip {
            let mut clipPoint = SkPoint::default();
            let outside = self.clipUmbraPoint(
                self.fPositions[self.fFirstVertexIndex as usize],
                self.fCentroid,
                &mut clipPoint,
            );
            if outside {
                self.fPositions.push(clipPoint);
                self.fColors.push(umbraColor);
            }
            self.fPrevUmbraOutside = outside;
            self.fFirstUmbraOutside = outside;
        }
        self.fPositions.push(self.fFirstPoint + self.fFirstOutset);
        self.fColors.push(Self::kPenumbraColor);
        self.addEdge(
            self.fPathPolygon[0],
            self.fFirstOutset,
            umbraColor,
            umbraPolygon,
            false,
            doClip,
        );
        for i in 1..polyCount {
            let mut normal = SkPoint::default();
            if !compute_normal(
                self.fPrevPoint,
                self.fPathPolygon[i],
                self.fDirection,
                &mut normal,
            ) {
                return false;
            }
            normal *= outset;
            self.addArc(normal, outset, true);
            self.addEdge(
                self.fPathPolygon[i],
                normal,
                umbraColor,
                umbraPolygon,
                i == polyCount - 1,
                doClip,
            );
        }
        crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 361; @condition b"this->indexCount()\0".as_ptr(); self.indexCount()!=0);
        crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 364; @condition b"fPositions.size() >= 3\0".as_ptr(); self.fPositions.len()>=3);
        if self.addArc(self.fFirstOutset, outset, false) {
            let offset = if self.fFirstUmbraOutside { 2 } else { 1 };
            self.appendTriangle(
                self.fFirstVertexIndex,
                self.fPositions.len() as i32 - 1,
                self.fFirstVertexIndex + offset,
            );
        } else {
            let offset = if self.fFirstUmbraOutside { 2 } else { 1 };
            let last = *self.fPositions.last().unwrap();
            self.fPositions[self.fFirstVertexIndex as usize + offset] = last;
        }
        true
    }
    // Source: src/utils/SkShadowTessellator.cpp:73-73
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:385-412
    fn computeClipVectorsAndTestCentroid(&mut self) {
        crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 386; @condition b"fClipPolygon.size() >= 3\0".as_ptr(); self.fClipPolygon.len()>=3);
        self.fCurrClipIndex = self.fClipPolygon.len() - 1;
        let mut v0 = self.fClipPolygon[1] - self.fClipPolygon[0];
        self.fClipVectors.push(v0);
        let mut hiddenCentroid = true;
        let mut v1 = self.fCentroid - self.fClipPolygon[0];
        let initCross = v0.cross(v1);
        for p in 1..self.fClipPolygon.len() {
            v0 = self.fClipPolygon[(p + 1) % self.fClipPolygon.len()] - self.fClipPolygon[p];
            self.fClipVectors.push(v0);
            v1 = self.fCentroid - self.fClipPolygon[p];
            if initCross * v0.cross(v1) <= 0.0 {
                hiddenCentroid = false;
            }
        }
        crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 409; @condition b"fClipVectors.size() == fClipPolygon.size()\0".as_ptr(); (self.fClipVectors.len()) == (self.fClipPolygon.len()));
        self.fTransparent = self.fTransparent || !hiddenCentroid;
    }
    // Source: src/utils/SkShadowTessellator.cpp:75-76
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:414-472
    fn addEdge(
        &mut self,
        nextPoint: SkPoint,
        nextNormal: SkPoint,
        umbraColor: SkColor,
        umbraPolygon: &[SkPoint],
        lastEdge: bool,
        doClip: bool,
    ) {
        let (duplicate, currUmbraIndex) = if lastEdge {
            self.fPrevPoint = nextPoint;
            (false, self.fFirstVertexIndex)
        } else {
            self.addInnerPoint(nextPoint, umbraColor, umbraPolygon)
        };
        let prevPenumbraIndex = self.fPositions.len() as i32
            - if duplicate || currUmbraIndex == self.fFirstVertexIndex {
                1
            } else {
                2
            };
        if !duplicate {
            if self.fTransparent {
                self.appendTriangle(0, self.fPrevUmbraIndex, currUmbraIndex);
            } else if doClip {
                let mut clipPoint = SkPoint::default();
                let outside = if lastEdge {
                    self.fFirstUmbraOutside
                } else {
                    self.clipUmbraPoint(
                        self.fPositions[currUmbraIndex as usize],
                        self.fCentroid,
                        &mut clipPoint,
                    )
                };
                if outside {
                    if !lastEdge {
                        self.fPositions.push(clipPoint);
                        self.fColors.push(umbraColor);
                    }
                    self.appendTriangle(self.fPrevUmbraIndex, currUmbraIndex, currUmbraIndex + 1);
                    if self.fPrevUmbraOutside {
                        self.appendTriangle(
                            self.fPrevUmbraIndex,
                            currUmbraIndex + 1,
                            self.fPrevUmbraIndex + 1,
                        );
                    }
                } else if self.fPrevUmbraOutside {
                    self.appendTriangle(
                        self.fPrevUmbraIndex,
                        currUmbraIndex,
                        self.fPrevUmbraIndex + 1,
                    );
                }
                self.fPrevUmbraOutside = outside;
            }
        }
        self.fPositions.push(nextPoint + nextNormal);
        self.fColors.push(Self::kPenumbraColor);
        if !duplicate {
            self.appendTriangle(self.fPrevUmbraIndex, prevPenumbraIndex, currUmbraIndex);
        }
        self.appendTriangle(
            prevPenumbraIndex,
            self.fPositions.len() as i32 - 1,
            currUmbraIndex,
        );
        self.fPrevUmbraIndex = currUmbraIndex;
        self.fPrevOutset = nextNormal;
    }
    // Source: src/utils/SkShadowTessellator.cpp:74-74
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:474-504
    fn clipUmbraPoint(
        &mut self,
        umbraPoint: SkPoint,
        centroid: SkPoint,
        clipPoint: &mut SkPoint,
    ) -> bool {
        let mut segmentVector = centroid - umbraPoint;
        let startClipPoint = self.fCurrClipIndex;
        loop {
            let dp = umbraPoint - self.fClipPolygon[self.fCurrClipIndex];
            let denom = self.fClipVectors[self.fCurrClipIndex].cross(segmentVector);
            let t_num = dp.cross(segmentVector);
            if SkScalarNearlyZero(denom, SK_ScalarNearlyZero) {
                if SkScalarNearlyZero(t_num, SK_ScalarNearlyZero) {
                    return false;
                }
            } else if t_num >= 0.0 && t_num <= denom {
                let s_num = dp.cross(self.fClipVectors[self.fCurrClipIndex]);
                if s_num >= 0.0 && s_num <= denom {
                    segmentVector *= s_num / denom;
                    *clipPoint = umbraPoint + segmentVector;
                    return true;
                }
            }
            self.fCurrClipIndex = (self.fCurrClipIndex + 1) % self.fClipPolygon.len();
            if self.fCurrClipIndex == startClipPoint {
                break;
            }
        }
        false
    }
    // Source: src/utils/SkShadowTessellator.cpp:77-78
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:506-536
    // Tuple preserves the native bool plus integer output without an aliasing borrow.
    fn addInnerPoint(
        &mut self,
        pathPoint: SkPoint,
        umbraColor: SkColor,
        umbraPolygon: &[SkPoint],
    ) -> (bool, i32) {
        let umbraPoint = if !self.fValidUmbra {
            pathPoint + (self.fCentroid - pathPoint) * 0.95
        } else {
            umbraPolygon[self.getClosestUmbraIndex(pathPoint, umbraPolygon)]
        };
        self.fPrevPoint = pathPoint;
        if self.fPrevUmbraIndex == -1
            || !duplicate_pt(umbraPoint, self.fPositions[self.fPrevUmbraIndex as usize])
        {
            let index = if self.fPrevUmbraIndex >= 0
                && duplicate_pt(umbraPoint, self.fPositions[self.fFirstVertexIndex as usize])
            {
                self.fFirstVertexIndex
            } else {
                let index = self.fPositions.len() as i32;
                self.fPositions.push(umbraPoint);
                self.fColors.push(umbraColor);
                index
            };
            (false, index)
        } else {
            (true, self.fPrevUmbraIndex)
        }
    }
    // Source: src/utils/SkShadowTessellator.cpp:79-79
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:538-566
    fn getClosestUmbraIndex(&mut self, p: SkPoint, umbraPolygon: &[SkPoint]) -> usize {
        let mut minDistance = SkPointPriv::DistanceToSqd(p, umbraPolygon[self.fCurrUmbraIndex]);
        let mut index = self.fCurrUmbraIndex;
        let mut dir = 1;
        let mut next = (index + dir) % umbraPolygon.len();
        let mut distance = SkPointPriv::DistanceToSqd(p, umbraPolygon[next]);
        if distance < minDistance {
            index = next;
            minDistance = distance;
        } else {
            dir = umbraPolygon.len() - 1;
        }
        next = (index + dir) % umbraPolygon.len();
        distance = SkPointPriv::DistanceToSqd(p, umbraPolygon[next]);
        while distance < minDistance {
            index = next;
            minDistance = distance;
            next = (index + dir) % umbraPolygon.len();
            distance = SkPointPriv::DistanceToSqd(p, umbraPolygon[next]);
        }
        self.fCurrUmbraIndex = index;
        index
    }
    // Source: src/utils/SkShadowTessellator.cpp:82-82
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:568-605
    fn computeConcaveShadow(&mut self, mut inset: f32, outset: f32) -> bool {
        if !SkIsSimplePolygon(&self.fPathPolygon) {
            return false;
        }
        fn source_min(a: f32, b: f32) -> f32 {
            if b < a {
                b
            } else {
                a
            }
        }
        inset = source_min(
            inset,
            source_min(
                crate::src::core::SkRectPriv::SkRectPriv::HalfWidth(&self.fPathBounds).abs(),
                crate::src::core::SkRectPriv::SkRectPriv::HalfHeight(&self.fPathBounds).abs(),
            ),
        );
        let mut umbraPolygon = Vec::new();
        let mut umbraIndices = Vec::new();
        umbraIndices.reserve(self.fPathPolygon.len());
        if !SkOffsetSimplePolygon(
            &self.fPathPolygon,
            &self.fPathBounds,
            inset,
            &mut umbraPolygon,
            Some(&mut umbraIndices),
        ) {
            return false;
        }
        let mut penumbraPolygon = Vec::with_capacity(umbraPolygon.len());
        let mut penumbraIndices = Vec::with_capacity(umbraPolygon.len());
        if !SkOffsetSimplePolygon(
            &self.fPathPolygon,
            &self.fPathBounds,
            -outset,
            &mut penumbraPolygon,
            Some(&mut penumbraIndices),
        ) {
            return false;
        }
        if umbraPolygon.is_empty() || penumbraPolygon.is_empty() {
            return false;
        }
        self.stitchConcaveRings(
            &umbraPolygon,
            &mut umbraIndices,
            &penumbraPolygon,
            &mut penumbraIndices,
        );
        true
    }
    // Source: src/utils/SkShadowTessellator.cpp:83-86
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:607-738
    fn stitchConcaveRings(
        &mut self,
        umbraPolygon: &[SkPoint],
        umbraIndices: &mut [i32],
        penumbraPolygon: &[SkPoint],
        penumbraIndices: &mut [i32],
    ) {
        // Native AutoSTMalloc has 64 inline entries; preserve that allocation threshold.
        let mut inline = [0u16; 64];
        let mut heap = Vec::new();
        let indexMap: &mut [u16] = if umbraPolygon.len() <= 64 {
            &mut inline[..umbraPolygon.len()]
        } else {
            heap.resize(umbraPolygon.len(), 0);
            heap.as_mut_slice()
        };
        let mut minIndex = 0;
        let mut min = penumbraIndices[0];
        for i in 1..penumbraIndices.len() {
            if penumbraIndices[i] < min {
                min = penumbraIndices[i];
                minIndex = i;
            }
        }
        let mut currPenumbra = minIndex;
        minIndex = 0;
        min = umbraIndices[0];
        for i in 1..umbraIndices.len() {
            if umbraIndices[i] < min {
                min = umbraIndices[i];
                minIndex = i;
            }
        }
        let mut currUmbra = minIndex;
        let pathCount = self.fPathPolygon.len() as i32;
        let mut maxPenumbraIndex = pathCount - 1;
        let mut maxUmbraIndex = pathCount - 1;
        while penumbraIndices[currPenumbra] != umbraIndices[currUmbra] {
            if penumbraIndices[currPenumbra] < umbraIndices[currUmbra] {
                penumbraIndices[currPenumbra] += pathCount;
                maxPenumbraIndex = penumbraIndices[currPenumbra];
                currPenumbra = (currPenumbra + 1) % penumbraPolygon.len();
            } else {
                umbraIndices[currUmbra] += pathCount;
                maxUmbraIndex = umbraIndices[currUmbra];
                currUmbra = (currUmbra + 1) % umbraPolygon.len();
            }
        }
        self.fPositions.push(penumbraPolygon[currPenumbra]);
        self.fColors.push(Self::kPenumbraColor);
        let mut prevPenumbraIndex = 0;
        self.fPositions.push(umbraPolygon[currUmbra]);
        self.fColors.push(Self::kUmbraColor);
        self.fPrevUmbraIndex = 1;
        indexMap[currUmbra] = 1;
        let mut nextPenumbra = (currPenumbra + 1) % penumbraPolygon.len();
        let mut nextUmbra = (currUmbra + 1) % umbraPolygon.len();
        while penumbraIndices[nextPenumbra] <= maxPenumbraIndex
            || umbraIndices[nextUmbra] <= maxUmbraIndex
        {
            if umbraIndices[nextUmbra] == penumbraIndices[nextPenumbra] {
                self.fPositions.push(penumbraPolygon[nextPenumbra]);
                self.fColors.push(Self::kPenumbraColor);
                let currPenumbraIndex = self.fPositions.len() as i32 - 1;
                self.fPositions.push(umbraPolygon[nextUmbra]);
                self.fColors.push(Self::kUmbraColor);
                let currUmbraIndex = self.fPositions.len() as i32 - 1;
                indexMap[nextUmbra] = currUmbraIndex as u16;
                self.appendQuad(
                    prevPenumbraIndex,
                    currPenumbraIndex,
                    self.fPrevUmbraIndex,
                    currUmbraIndex,
                );
                prevPenumbraIndex = currPenumbraIndex;
                penumbraIndices[currPenumbra] += pathCount;
                currPenumbra = nextPenumbra;
                nextPenumbra = (currPenumbra + 1) % penumbraPolygon.len();
                self.fPrevUmbraIndex = currUmbraIndex;
                umbraIndices[currUmbra] += pathCount;
                currUmbra = nextUmbra;
                nextUmbra = (currUmbra + 1) % umbraPolygon.len();
            }
            while penumbraIndices[nextPenumbra] < umbraIndices[nextUmbra]
                && penumbraIndices[nextPenumbra] <= maxPenumbraIndex
            {
                self.fPositions.push(penumbraPolygon[nextPenumbra]);
                self.fColors.push(Self::kPenumbraColor);
                let currPenumbraIndex = self.fPositions.len() as i32 - 1;
                self.appendTriangle(prevPenumbraIndex, currPenumbraIndex, self.fPrevUmbraIndex);
                prevPenumbraIndex = currPenumbraIndex;
                penumbraIndices[currPenumbra] += pathCount;
                currPenumbra = nextPenumbra;
                nextPenumbra = (currPenumbra + 1) % penumbraPolygon.len();
            }
            while umbraIndices[nextUmbra] < penumbraIndices[nextPenumbra]
                && umbraIndices[nextUmbra] <= maxUmbraIndex
            {
                self.fPositions.push(umbraPolygon[nextUmbra]);
                self.fColors.push(Self::kUmbraColor);
                let currUmbraIndex = self.fPositions.len() as i32 - 1;
                indexMap[nextUmbra] = currUmbraIndex as u16;
                self.appendTriangle(self.fPrevUmbraIndex, prevPenumbraIndex, currUmbraIndex);
                self.fPrevUmbraIndex = currUmbraIndex;
                umbraIndices[currUmbra] += pathCount;
                currUmbra = nextUmbra;
                nextUmbra = (currUmbra + 1) % umbraPolygon.len();
            }
        }
        self.fPositions.push(penumbraPolygon[nextPenumbra]);
        self.fColors.push(Self::kPenumbraColor);
        let currPenumbraIndex = self.fPositions.len() as i32 - 1;
        self.fPositions.push(umbraPolygon[nextUmbra]);
        self.fColors.push(Self::kUmbraColor);
        let currUmbraIndex = self.fPositions.len() as i32 - 1;
        indexMap[nextUmbra] = currUmbraIndex as u16;
        self.appendQuad(
            prevPenumbraIndex,
            currPenumbraIndex,
            self.fPrevUmbraIndex,
            currUmbraIndex,
        );
        if self.fTransparent {
            SkTriangulateSimplePolygon(umbraPolygon, indexMap, &mut self.fIndices);
        }
    }
}
// Source: src/utils/SkShadowTessellator.cpp:748-748
const kConicTolerance: f32 = 0.25;
// Source: src/utils/SkShadowTessellator.cpp:751-754
fn sanitize_point(input: SkPoint) -> SkPoint {
    SkPoint::make(
        crate::src::core::SkScalar::SkScalarRoundToScalar(16.0 * input.fX) * 0.0625,
        crate::src::core::SkScalar::SkScalarRoundToScalar(16.0 * input.fY) * 0.0625,
    )
}
impl SkBaseShadowTessellator {
    // Source: src/utils/SkShadowTessellator.cpp:88-88
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:756-781
    fn handleLine(&mut self, p: SkPoint) {
        let pSanitized = sanitize_point(p);
        let n = self.fPathPolygon.len();
        if n != 0 && !self.accumulateCentroid(self.fPathPolygon[n - 1], pSanitized) {
            return;
        }
        if n > 1
            && !self.checkConvexity(
                self.fPathPolygon[n - 2],
                self.fPathPolygon[n - 1],
                pSanitized,
            )
        {
            self.fPathPolygon.pop();
            if duplicate_pt(*self.fPathPolygon.last().unwrap(), pSanitized) {
                self.fPathPolygon.pop();
            }
        }
        self.fPathPolygon.push(pSanitized);
    }
    // Source: src/utils/SkShadowTessellator.cpp:89-89
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:783-787
    fn handleLine_matrix(&mut self, m: &SkMatrix, p: &mut SkPoint) {
        *p = m.mapPoint(*p);
        self.handleLine(*p);
    }
    // Source: src/utils/SkShadowTessellator.cpp:91-91
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:789-812
    fn handleQuad(&mut self, pts: &[SkPoint; 3]) {
        // Native non-Ganesh branch deliberately emits the control and end points.
        self.handleLine(pts[1]);
        self.handleLine(pts[2]);
    }
    // Source: src/utils/SkShadowTessellator.cpp:92-92
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:814-817
    fn handleQuad_matrix(&mut self, m: &SkMatrix, pts: &mut [SkPoint; 3]) {
        for p in pts.iter_mut() {
            *p = m.mapPoint(*p);
        }
        self.handleQuad(pts);
    }
    // Source: src/utils/SkShadowTessellator.cpp:94-94
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:819-838
    fn handleCubic(&mut self, m: &SkMatrix, pts: &mut [SkPoint; 4]) {
        for p in pts.iter_mut() {
            *p = m.mapPoint(*p);
        }
        self.handleLine(pts[1]);
        self.handleLine(pts[2]);
        self.handleLine(pts[3]);
    }
    // Source: src/utils/SkShadowTessellator.cpp:96-96
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:840-858
    fn handleConic(&mut self, m: &SkMatrix, pts: &mut [SkPoint; 3], mut w: f32) {
        if m.hasPerspective() {
            w = SkConic::TransformW(pts, w, m);
        }
        for p in pts.iter_mut() {
            *p = m.mapPoint(*p);
        }
        let mut quadder = SkAutoConicToQuads::default();
        let quads = quadder.computeQuads(pts, w, kConicTolerance);
        let mut lastPoint = quads[0];
        let count = (quads.len() - 1) / 2;
        for i in 0..count {
            let quadPts = [
                lastPoint,
                quads[1 + 2 * i],
                if i == count - 1 {
                    pts[2]
                } else {
                    quads[2 + 2 * i]
                },
            ];
            self.handleQuad(&quadPts);
            lastPoint = quadPts[2];
        }
    }
    // Source: src/utils/SkShadowTessellator.cpp:98-98
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:860-887
    fn addArc(&mut self, nextNormal: SkPoint, offset: f32, finishArc: bool) -> bool {
        let (mut rotSin, mut rotCos, mut numSteps) = (0.0, 0.0, 0);
        if !SkComputeRadialSteps(
            &self.fPrevOutset,
            &nextNormal,
            offset,
            &mut rotSin,
            &mut rotCos,
            &mut numSteps,
        ) {
            numSteps = 0;
        }
        let mut prevNormal = self.fPrevOutset;
        for _ in 0..numSteps - 1 {
            let currNormal = SkPoint::make(
                prevNormal.fX * rotCos - prevNormal.fY * rotSin,
                prevNormal.fY * rotCos + prevNormal.fX * rotSin,
            );
            self.fPositions.push(self.fPrevPoint + currNormal);
            self.fColors.push(Self::kPenumbraColor);
            self.appendTriangle(
                self.fPrevUmbraIndex,
                self.fPositions.len() as i32 - 1,
                self.fPositions.len() as i32 - 2,
            );
            prevNormal = currNormal;
        }
        if finishArc && numSteps != 0 {
            self.fPositions.push(self.fPrevPoint + nextNormal);
            self.fColors.push(Self::kPenumbraColor);
            self.appendTriangle(
                self.fPrevUmbraIndex,
                self.fPositions.len() as i32 - 1,
                self.fPositions.len() as i32 - 2,
            );
        }
        self.fPrevOutset = nextNormal;
        numSteps > 0
    }
    // Source: src/utils/SkShadowTessellator.cpp:100-100
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:889-895
    fn appendTriangle(&mut self, index0: i32, index1: i32, index2: i32) {
        self.fIndices
            .extend_from_slice(&[index0 as u16, index1 as u16, index2 as u16]);
    }
    // Source: src/utils/SkShadowTessellator.cpp:101-101
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:897-908
    fn appendQuad(&mut self, index0: i32, index1: i32, index2: i32, index3: i32) {
        self.fIndices.extend_from_slice(&[
            index0 as u16,
            index1 as u16,
            index2 as u16,
            index2 as u16,
            index1 as u16,
            index3 as u16,
        ]);
    }
}
// Source: src/utils/SkShadowTessellator.cpp:912-912
struct SkAmbientShadowTessellator {
    base: SkBaseShadowTessellator,
}
impl SkAmbientShadowTessellator {
    // Source: src/utils/SkShadowTessellator.cpp:914-915
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:923-956
    fn new(path: &SkPath, ctm: &SkMatrix, zPlaneParams: SkPoint3, transparent: bool) -> Self {
        let mut this = Self {
            base: SkBaseShadowTessellator::new(zPlaneParams, *path.getBounds(), transparent),
        };
        let baseZ = this.base.heightFunc(
            this.base.fPathBounds.centerX(),
            this.base.fPathBounds.centerY(),
        );
        let outset = SkDrawShadowMetrics::AmbientBlurRadius(baseZ);
        let inset = outset * SkDrawShadowMetrics::AmbientRecipAlpha(baseZ) - outset;
        if !this.computePathPolygon(path, ctm) {
            return this;
        }
        if this.base.fPathPolygon.len() < 3 || !this.base.fArea.is_finite() {
            this.base.fSucceeded = true;
            return this;
        }
        this.base.fPositions.reserve(4 * path.countPoints());
        this.base.fColors.reserve(4 * path.countPoints());
        this.base.fIndices.reserve(12 * path.countPoints());
        this.base.fSucceeded = if this.base.fIsConvex {
            this.base.computeConvexShadow(inset, outset, false)
        } else {
            this.base.computeConcaveShadow(inset, outset)
        };
        this
    }
}
// Source: src/utils/SkShadowTessellator.cpp:958-963
fn spancpy(dst: &mut [SkPoint], src: &[SkPoint]) {
    crate::SkASSERT!(@source b"src/utils/SkShadowTessellator.cpp\0".as_ptr(), 959; @condition b"dst.size() >= src.size()\0".as_ptr(); dst.len()>=src.len());
    for i in 0..src.len() {
        dst[i] = src[i];
    }
}
impl SkAmbientShadowTessellator {
    // Source: src/utils/SkShadowTessellator.cpp:918-918
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:965-1006
    fn computePathPolygon(&mut self, path: &SkPath, ctm: &SkMatrix) -> bool {
        self.base.fPathPolygon.reserve(path.countPoints());
        let mut iter = Iter::new(path, true);
        let mut verbSeen = false;
        let mut closeSeen = false;
        while let Some(rec) = iter.next() {
            if closeSeen {
                return false;
            }
            let mut pts = [SkPoint::default(); 4];
            spancpy(&mut pts, rec.fPoints);
            match rec.fVerb {
                SkPathVerb::kLine => self.base.handleLine_matrix(ctm, &mut pts[1]),
                SkPathVerb::kQuad => self
                    .base
                    .handleQuad_matrix(ctm, (&mut pts[..3]).try_into().unwrap()),
                SkPathVerb::kCubic => self.base.handleCubic(ctm, &mut pts),
                SkPathVerb::kConic => self.base.handleConic(
                    ctm,
                    (&mut pts[..3]).try_into().unwrap(),
                    rec.conicWeight(),
                ),
                SkPathVerb::kMove => {
                    if verbSeen {
                        return false;
                    }
                }
                SkPathVerb::kClose => closeSeen = true,
            }
            verbSeen = true;
        }
        self.base.finishPathPolygon();
        true
    }
}
// Source: src/utils/SkShadowTessellator.cpp:1010-1010
struct SkSpotShadowTessellator {
    base: SkBaseShadowTessellator,
}
impl SkSpotShadowTessellator {
    // Source: src/utils/SkShadowTessellator.cpp:1012-1014
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:1024-1071
    fn new(
        path: &SkPath,
        ctm: &SkMatrix,
        zPlaneParams: SkPoint3,
        lightPos: SkPoint3,
        lightRadius: f32,
        transparent: bool,
        directional: bool,
    ) -> Self {
        let mut this = Self {
            base: SkBaseShadowTessellator::new(zPlaneParams, *path.getBounds(), transparent),
        };
        let mut shadowTransform = SkMatrix::default();
        let mut outset = 0.0;
        if !SkDrawShadowMetrics::GetSpotShadowTransform(
            &lightPos,
            lightRadius,
            ctm,
            &zPlaneParams,
            path.getBounds(),
            directional,
            &mut shadowTransform,
            &mut outset,
        ) {
            return this;
        }
        let inset = outset;
        if !this.computeClipAndPathPolygons(path, ctm, &shadowTransform) {
            return this;
        }
        if this.base.fClipPolygon.len() < 3
            || this.base.fPathPolygon.len() < 3
            || !this.base.fArea.is_finite()
        {
            this.base.fSucceeded = true;
            return this;
        }
        this.base.fPositions.reserve(5 * path.countPoints());
        this.base.fColors.reserve(5 * path.countPoints());
        this.base.fIndices.reserve(15 * path.countPoints());
        this.base.fSucceeded = if this.base.fIsConvex {
            this.base.computeConvexShadow(inset, outset, true)
        } else {
            this.base.computeConcaveShadow(inset, outset)
        };
        if !this.base.fSucceeded {
            return this;
        }
        this.base.fSucceeded = true;
        this
    }
    // Source: src/utils/SkShadowTessellator.cpp:1017-1018
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:1073-1157
    fn computeClipAndPathPolygons(
        &mut self,
        path: &SkPath,
        ctm: &SkMatrix,
        shadowTransform: &SkMatrix,
    ) -> bool {
        self.base.fPathPolygon.reserve(path.countPoints());
        self.base.fClipPolygon.reserve(path.countPoints());
        let mut iter = Iter::new(path, true);
        let mut clipPts = [SkPoint::default(); 4];
        const kA: f32 = 0.32495117187;
        const kB: f32 = 0.44311523437;
        const kC: f32 = 0.20141601562;
        const kD: f32 = 0.03051757812;
        let mut closeSeen = false;
        let mut verbSeen = false;
        while let Some(rec) = iter.next() {
            if closeSeen {
                return false;
            }
            let mut pts = [SkPoint::default(); 4];
            spancpy(&mut pts, rec.fPoints);
            match rec.fVerb {
                SkPathVerb::kLine => {
                    clipPts[0] = ctm.mapPoint(pts[1]);
                    self.addToClip(clipPts[0]);
                    self.base.handleLine_matrix(shadowTransform, &mut pts[1]);
                }
                SkPathVerb::kQuad => {
                    for i in 0..3 {
                        clipPts[i] = ctm.mapPoint(pts[i]);
                    }
                    let curvePoint = SkPoint::make(
                        0.25 * clipPts[0].fX + 0.5 * clipPts[1].fX + 0.25 * clipPts[2].fX,
                        0.25 * clipPts[0].fY + 0.5 * clipPts[1].fY + 0.25 * clipPts[2].fY,
                    );
                    self.addToClip(curvePoint);
                    self.addToClip(clipPts[2]);
                    self.base
                        .handleQuad_matrix(shadowTransform, (&mut pts[..3]).try_into().unwrap());
                }
                SkPathVerb::kConic => {
                    for i in 0..3 {
                        clipPts[i] = ctm.mapPoint(pts[i]);
                    }
                    let w = rec.conicWeight();
                    let mut curvePoint = SkPoint::make(
                        0.25 * clipPts[0].fX + w * 0.5 * clipPts[1].fX + 0.25 * clipPts[2].fX,
                        0.25 * clipPts[0].fY + w * 0.5 * clipPts[1].fY + 0.25 * clipPts[2].fY,
                    );
                    curvePoint *= crate::src::core::SkScalar::SkScalarInvert(0.5 + 0.5 * w);
                    self.addToClip(curvePoint);
                    self.addToClip(clipPts[2]);
                    self.base
                        .handleConic(shadowTransform, (&mut pts[..3]).try_into().unwrap(), w);
                }
                SkPathVerb::kCubic => {
                    for i in 0..4 {
                        clipPts[i] = ctm.mapPoint(pts[i]);
                    }
                    let mut curvePoint = SkPoint::make(
                        kA * clipPts[0].fX
                            + kB * clipPts[1].fX
                            + kC * clipPts[2].fX
                            + kD * clipPts[3].fX,
                        kA * clipPts[0].fY
                            + kB * clipPts[1].fY
                            + kC * clipPts[2].fY
                            + kD * clipPts[3].fY,
                    );
                    self.addToClip(curvePoint);
                    curvePoint = SkPoint::make(
                        kD * clipPts[0].fX
                            + kC * clipPts[1].fX
                            + kB * clipPts[2].fX
                            + kA * clipPts[3].fX,
                        kD * clipPts[0].fY
                            + kC * clipPts[1].fY
                            + kB * clipPts[2].fY
                            + kA * clipPts[3].fY,
                    );
                    self.addToClip(curvePoint);
                    self.addToClip(clipPts[3]);
                    self.base.handleCubic(shadowTransform, &mut pts);
                }
                SkPathVerb::kMove => {
                    if verbSeen {
                        return false;
                    }
                }
                SkPathVerb::kClose => closeSeen = true,
            }
            verbSeen = true;
        }
        self.base.finishPathPolygon();
        true
    }
    // Source: src/utils/SkShadowTessellator.cpp:1019-1019
    // Declaration/overload maps to the existing concrete method body below.
    // Source: src/utils/SkShadowTessellator.cpp:1159-1163
    fn addToClip(&mut self, point: SkPoint) {
        if self.base.fClipPolygon.is_empty()
            || !duplicate_pt(point, *self.base.fClipPolygon.last().unwrap())
        {
            self.base.fClipPolygon.push(point);
        }
    }
}
// Source: src/utils/SkShadowTessellator.h:32-33
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkShadowTessellator.cpp:1167-1174
pub fn MakeAmbient(
    path: &SkPath,
    ctm: &SkMatrix,
    zPlane: &SkPoint3,
    transparent: bool,
) -> sk_sp<SkVertices> {
    if !ctm.mapRect_value(path.getBounds()).isFinite() || !zPlane.isFinite() {
        return Default::default();
    }
    SkAmbientShadowTessellator::new(path, ctm, *zPlane, transparent)
        .base
        .releaseVertices()
}
// Source: src/utils/SkShadowTessellator.h:40-42
// Declaration/overload maps to the existing concrete method body below.
// Source: src/utils/SkShadowTessellator.cpp:1176-1188
pub fn MakeSpot(
    path: &SkPath,
    ctm: &SkMatrix,
    zPlane: &SkPoint3,
    lightPos: &SkPoint3,
    lightRadius: f32,
    transparent: bool,
    directional: bool,
) -> sk_sp<SkVertices> {
    if !ctm.mapRect_value(path.getBounds()).isFinite()
        || !zPlane.isFinite()
        || !lightPos.isFinite()
        || !(lightPos.fZ >= SK_ScalarNearlyZero)
        || !lightRadius.is_finite()
        || !(lightRadius >= SK_ScalarNearlyZero)
    {
        return Default::default();
    }
    SkSpotShadowTessellator::new(
        path,
        ctm,
        *zPlane,
        *lightPos,
        lightRadius,
        transparent,
        directional,
    )
    .base
    .releaseVertices()
}
