#![allow(non_snake_case)]

use crate::include::core::SkPathTypes::*;
use crate::include::private::SkPathRef::*;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPathEnums::*;
use crate::src::core::SkPathIter::SkPathIter;
use crate::src::core::SkPathPriv::SkPathPriv;
use crate::src::core::SkPathRaw::SkPathRaw;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;

// Source: include/core/SkPathBuilder.h:1002-1005
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpFormat {
    kDecimal,
    kHex,
}

// Source: include/core/SkPathBuilder.h:87-87
impl SkPathBuilder {
    #[inline]
    pub fn fillType(&self) -> SkPathFillType {
        self.fFillType
    }
}

// Source: include/core/SkPathBuilder.h:108-113
impl SkPathBuilder {
    #[inline]
    pub fn computeBounds(&self) -> SkRect {
        self.computeFiniteBounds().unwrap_or_else(SkRect::MakeEmpty)
    }
}

// Source: include/core/SkPathBuilder.h:140-140
impl SkPathBuilder {
    #[inline]
    pub fn setFillType(&mut self, ft: SkPathFillType) -> &mut Self {
        self.fFillType = ft;
        self
    }
}

// Source: include/core/SkPathBuilder.h:158-158
impl SkPathBuilder {
    #[inline]
    pub fn setIsVolatile(&mut self, v: bool) -> &mut Self {
        self.fIsVolatile = v;
        self
    }
}

// Source: include/core/SkPathBuilder.h:176-178
impl SkPathBuilder {
    #[inline]
    pub fn moveToXY(&mut self, x: f32, y: f32) -> &mut Self {
        self.moveTo(SkPoint::make(x, y))
    }
}
// Source: include/core/SkPathBuilder.h:201-201
impl SkPathBuilder {
    #[inline]
    pub fn lineToXY(&mut self, x: f32, y: f32) -> &mut Self {
        self.lineTo(SkPoint::make(x, y))
    }
}
// Source: include/core/SkPathBuilder.h:233-235
impl SkPathBuilder {
    #[inline]
    pub fn quadToXY(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> &mut Self {
        self.quadTo(SkPoint::make(x1, y1), SkPoint::make(x2, y2))
    }
}
// Source: include/core/SkPathBuilder.h:247-247
impl SkPathBuilder {
    pub fn quadToArray(&mut self, pts: &[SkPoint; 2]) -> &mut Self {
        self.quadTo(pts[0], pts[1])
    }
}
// Source: include/core/SkPathBuilder.h:295-297
impl SkPathBuilder {
    #[inline]
    pub fn conicToXY(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, w: f32) -> &mut Self {
        self.conicTo(SkPoint::make(x1, y1), SkPoint::make(x2, y2), w)
    }
}
// Source: include/core/SkPathBuilder.h:319-321
impl SkPathBuilder {
    pub fn conicToArray(&mut self, pts: &[SkPoint; 2], w: f32) -> &mut Self {
        self.conicTo(pts[0], pts[1], w)
    }
}
// Source: include/core/SkPathBuilder.h:354-356
impl SkPathBuilder {
    #[inline]
    pub fn cubicToXY(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> &mut Self {
        self.cubicTo(
            SkPoint::make(x1, y1),
            SkPoint::make(x2, y2),
            SkPoint::make(x3, y3),
        )
    }
}
// Source: include/core/SkPathBuilder.h:369-371
impl SkPathBuilder {
    pub fn cubicToArray(&mut self, pts: &[SkPoint; 3]) -> &mut Self {
        self.cubicTo(pts[0], pts[1], pts[2])
    }
}
// Source: include/core/SkPathBuilder.h:417-417
impl SkPathBuilder {
    #[inline]
    pub fn rMoveToXY(&mut self, dx: f32, dy: f32) -> &mut Self {
        self.rMoveTo(SkPoint::make(dx, dy))
    }
}
// Source: include/core/SkPathBuilder.h:444-444
impl SkPathBuilder {
    #[inline]
    pub fn rLineToXY(&mut self, dx: f32, dy: f32) -> &mut Self {
        self.rLineTo(SkPoint::make(dx, dy))
    }
}
// Source: include/core/SkPathBuilder.h:480-482
impl SkPathBuilder {
    #[inline]
    pub fn rQuadToXY(&mut self, dx1: f32, dy1: f32, dx2: f32, dy2: f32) -> &mut Self {
        self.rQuadTo(SkPoint::make(dx1, dy1), SkPoint::make(dx2, dy2))
    }
}
// Source: include/core/SkPathBuilder.h:534-536
impl SkPathBuilder {
    #[inline]
    pub fn rConicToXY(&mut self, dx1: f32, dy1: f32, dx2: f32, dy2: f32, w: f32) -> &mut Self {
        self.rConicTo(SkPoint::make(dx1, dy1), SkPoint::make(dx2, dy2), w)
    }
}
// Source: include/core/SkPathBuilder.h:577-581
impl SkPathBuilder {
    #[inline]
    pub fn rCubicToXY(
        &mut self,
        dx1: f32,
        dy1: f32,
        dx2: f32,
        dy2: f32,
        dx3: f32,
        dy3: f32,
    ) -> &mut Self {
        self.rCubicTo(
            SkPoint::make(dx1, dy1),
            SkPoint::make(dx2, dy2),
            SkPoint::make(dx3, dy3),
        )
    }
}
// Source: include/core/SkPathBuilder.h:585-588
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArcSize {
    kSmall_ArcSize = 0,
    kLarge_ArcSize = 1,
}

// Source: include/core/SkPathBuilder.h:698-700
impl SkPathBuilder {
    #[inline]
    pub fn addLine(&mut self, a: SkPoint, b: SkPoint) -> &mut Self {
        self.moveTo(a).lineTo(b)
    }
}
// Source: include/core/SkPathBuilder.h:743-745
impl SkPathBuilder {
    #[inline]
    pub fn addRectDefault(&mut self, rect: &SkRect, dir: SkPathDirection) -> &mut Self {
        self.addRect(rect, dir, 0)
    }
}
// Source: include/core/SkPathBuilder.h:781-784
impl SkPathBuilder {
    #[inline]
    pub fn addRRectDefault(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        dir: SkPathDirection,
    ) -> &mut Self {
        self.addRRect(rrect, dir, if dir == SkPathDirection::kCW { 6 } else { 7 })
    }
}
// Source: include/core/SkPathBuilder.h:797-800
impl SkPathBuilder {
    #[inline]
    pub fn addOvalDefault(&mut self, oval: &SkRect, dir: SkPathDirection) -> &mut Self {
        self.addOval(oval, dir, 1)
    }
}
// Source: include/core/SkPathBuilder.h:814-817
impl SkPathBuilder {
    #[inline]
    pub fn addCircleXY(&mut self, x: f32, y: f32, radius: f32, dir: SkPathDirection) -> &mut Self {
        self.addCircle(SkPoint::make(x, y), radius, dir)
    }
}
// Source: include/core/SkPathBuilder.h:855-858
impl SkPathBuilder {
    #[inline]
    pub fn addPathUntransformed(
        &mut self,
        src: &SkPath,
        mode: crate::src::core::SkPath::AddPathMode,
    ) -> &mut Self {
        self.addPath(src, crate::src::core::SkMatrix::SkMatrix::I(), mode)
    }
}
// Source: include/core/SkPathBuilder.h:893-895
impl SkPathBuilder {
    #[inline]
    pub fn incReservePoints(&mut self, extraPtCount: usize) {
        self.incReserve(extraPtCount, extraPtCount, 0);
    }
}
// Source: include/core/SkPathBuilder.h:920-923
impl SkPathBuilder {
    #[inline]
    pub fn toggleInverseFillType(&mut self) -> &mut Self {
        self.fFillType = SkPathFillType_ToggleInverse(self.fFillType);
        self
    }
}

// Source: include/core/SkPathBuilder.h:931-931
impl SkPathBuilder {
    #[inline]
    pub fn isEmpty(&self) -> bool {
        self.fVerbs.is_empty()
    }
}

// Source: include/core/SkPathBuilder.h:954-956
impl SkPathBuilder {
    #[inline]
    pub fn setLastPoint(&mut self, p: SkPoint) {
        self.setPoint(self.points().len().wrapping_sub(1), p);
    }
}

// Source: include/core/SkPathBuilder.h:965-965
impl SkPathBuilder {
    #[inline]
    pub fn setLastPtXY(&mut self, x: f32, y: f32) {
        self.setLastPt(SkPoint::make(x, y));
    }
}
// Source: include/core/SkPathBuilder.h:973-973
impl SkPathBuilder {
    #[inline]
    pub fn countPoints(&self) -> usize {
        self.fPts.len()
    }
}

// Source: include/core/SkPathBuilder.h:980-980
impl SkPathBuilder {
    #[inline]
    pub fn isInverseFillType(&self) -> bool {
        SkPathFillType_IsInverse(self.fFillType)
    }
}

// Source: include/core/SkPathBuilder.h:982-984
impl SkPathBuilder {
    #[inline]
    pub fn points(&self) -> &[SkPoint] {
        &self.fPts
    }
}

// Source: include/core/SkPathBuilder.h:985-987
impl SkPathBuilder {
    #[inline]
    pub fn verbs(&self) -> &[SkPathVerb] {
        &self.fVerbs
    }
}

// Source: include/core/SkPathBuilder.h:988-990
impl SkPathBuilder {
    #[inline]
    pub fn conicWeights(&self) -> &[f32] {
        &self.fConicWeights
    }
}

// Source: include/core/SkPathBuilder.h:991-997
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reserve {
    kExact,
    kGrow,
}

// Source: include/core/SkPathBuilder.h:1015-1027
// Adapted Source: include/core/SkPathBuilder.h:1015-1017
// Adapted private field order: native owned arrays die weights, verbs, points.
// The declaration is not a serialized/payload/FFI layout contract.
pub struct SkPathBuilder {
    // Source: include/core/SkPathBuilder.h:1017-1017
    pub(crate) fConicWeights: Vec<f32>,
    // Source: include/core/SkPathBuilder.h:1016-1016
    pub(crate) fVerbs: Vec<SkPathVerb>,
    // Source: include/core/SkPathBuilder.h:1015-1015
    pub(crate) fPts: Vec<SkPoint>,
    pub(crate) fFillType: SkPathFillType,
    pub(crate) fIsVolatile: bool,
    pub(crate) fConvexity: SkPathConvexity,
    pub(crate) fSegmentMask: u32,
    pub(crate) fLastMoveIndex: i32,
    pub(crate) fType: SkPathIsAType,
    pub(crate) fIsA: SkPathIsAData,
}

// The pre-existing native member copy stays in forward native declaration
// order despite the private Rust drop-order representation above.
impl Clone for SkPathBuilder {
    fn clone(&self) -> Self {
        Self {
            fPts: self.fPts.clone(),
            fVerbs: self.fVerbs.clone(),
            fConicWeights: self.fConicWeights.clone(),
            fFillType: self.fFillType,
            fIsVolatile: self.fIsVolatile,
            fConvexity: self.fConvexity,
            fSegmentMask: self.fSegmentMask,
            fLastMoveIndex: self.fLastMoveIndex,
            fType: self.fType,
            fIsA: self.fIsA,
        }
    }
}

// Source: include/core/SkPathBuilder.h:1030-1037
impl SkPathBuilder {
    #[inline]
    fn ensureMove(&mut self) {
        self.fType = SkPathIsAType::kGeneral;
        if self.fVerbs.is_empty() {
            self.moveTo(SkPoint::make(0.0, 0.0));
        } else if self.fVerbs.last() == Some(&SkPathVerb::kClose) {
            self.moveTo(self.fPts[self.fLastMoveIndex as usize]);
        }
    }
}

// Source: src/core/SkPathBuilder.cpp:37-47
fn subdivide_cubic_to(path: &mut SkPathBuilder, pts: &[SkPoint; 4], level: i32) {
    let level = level - 1;
    if level >= 0 {
        let mut tmp = [SkPoint::default(); 7];
        crate::src::core::SkGeometry::SkChopCubicAtHalf(pts, &mut tmp);
        subdivide_cubic_to(path, (&tmp[..4]).try_into().unwrap(), level);
        subdivide_cubic_to(path, (&tmp[3..]).try_into().unwrap(), level);
    } else {
        path.cubicTo(pts[1], pts[2], pts[3]);
    }
}

// Source: src/core/SkPathBuilder.cpp:51-53
impl Default for SkPathBuilder {
    fn default() -> Self {
        let mut b = Self {
            fPts: Vec::new(),
            fVerbs: Vec::new(),
            fConicWeights: Vec::new(),
            fFillType: SkPathFillType::kWinding,
            fIsVolatile: false,
            fConvexity: SkPathConvexity::kUnknown,
            fSegmentMask: 0,
            fLastMoveIndex: -1,
            fType: SkPathIsAType::kGeneral,
            fIsA: SkPathIsAData {
                fStartIndex: 0,
                fDirection: SkPathDirection::kCW,
            },
        };
        b.reset();
        b
    }
}
impl SkPathBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

// Source: src/core/SkPathBuilder.cpp:61-64
impl SkPathBuilder {
    pub fn with_fill_type(ft: SkPathFillType) -> Self {
        let mut b = Self::default();
        b.fFillType = ft;
        b
    }
}

// Source: src/core/SkPathBuilder.cpp:66-68
impl SkPathBuilder {
    pub fn from_path(src: &SkPath) -> Self {
        let mut b = Self::default();
        b.assign_path(src);
        b
    }
}

// Source: src/core/SkPathBuilder.cpp:70-87
impl SkPathBuilder {
    pub fn assign_path(&mut self, src: &SkPath) -> &mut Self {
        self.reset().setFillType(src.getFillType());
        self.setIsVolatile(src.isVolatile());
        if src.isEmpty() {
            return self;
        }
        self.addRaw(
            &src.fPathData
                .raw(src.getFillType(), SkResolveConvexity::kYes),
            Reserve::kExact,
        );
        self.fLastMoveIndex = SkPathPriv::FindLastMoveToIndex(&self.fVerbs, self.fPts.len());
        debug_assert!(self.fLastMoveIndex < (self.fPts.len() as i32));
        self.fType = src.fPathData.fType;
        self.fIsA = src.fPathData.fIsA;
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:89-105
impl SkPathBuilder {
    pub fn reset(&mut self) -> &mut Self {
        self.fPts.clear();
        self.fVerbs.clear();
        self.fConicWeights.clear();
        self.fFillType = SkPathFillType::kWinding;
        self.fIsVolatile = false;
        self.fSegmentMask = 0;
        self.fLastMoveIndex = -1;
        self.fType = SkPathIsAType::kGeneral;
        self.fConvexity = SkPathConvexity::kUnknown;
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:107-118
impl PartialEq for SkPathBuilder {
    fn eq(&self, o: &Self) -> bool {
        if core::ptr::eq(self, o) {
            return true;
        }
        if self.fSegmentMask != o.fSegmentMask || self.fFillType != o.fFillType {
            return false;
        }
        self.fVerbs == o.fVerbs && self.fPts == o.fPts && self.fConicWeights == o.fConicWeights
    }

    // Source: include/core/SkPathBuilder.h:81-81
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !self.eq(b)
    }
}

// Source: src/core/SkPathBuilder.cpp:120-124
impl SkPathBuilder {
    pub fn incReserve(&mut self, points: usize, verbs: usize, conics: usize) {
        self.fPts
            .reserve(points.min((i32::MAX as usize).saturating_sub(self.fPts.len())));
        self.fVerbs
            .reserve(verbs.min((i32::MAX as usize).saturating_sub(self.fVerbs.len())));
        self.fConicWeights
            .reserve(conics.min((i32::MAX as usize).saturating_sub(self.fConicWeights.len())));
    }
}

// Source: src/core/SkPathBuilder.cpp:126-148
impl SkPathBuilder {
    fn growForVerbsInPath(&mut self, path: &SkPath) -> (usize, usize) {
        self.fSegmentMask |= path.getSegmentMasks();
        let verbs = path.verbs();
        if !verbs.is_empty() {
            self.fType = SkPathIsAType::kGeneral;
            self.fVerbs.extend_from_slice(verbs);
        }
        // Rust reserves uninitialized POD capacity here; the sole caller publishes lengths after writing.
        let points = self.fPts.len();
        if path.countPoints() != 0 {
            self.fPts.reserve(path.countPoints());
        }
        let weights = self.fConicWeights.len();
        if !path.conicWeights().is_empty() {
            self.fConicWeights.reserve(path.conicWeights().len());
        }
        (points, weights)
    }
}

// Source: src/core/SkPathBuilder.cpp:158-178
impl SkPathBuilder {
    pub fn moveTo(&mut self, pt: SkPoint) -> &mut Self {
        if self.fVerbs.last() == Some(&SkPathVerb::kMove) {
            *self.fPts.last_mut().unwrap() = pt;
            crate::SkASSERT!(@source b"src/core/SkPathBuilder.cpp\0".as_ptr(),162; @condition b"fType != SkPathIsAType::kOval && fType != SkPathIsAType::kRRect\0".as_ptr(); self.fType != SkPathIsAType::kOval && self.fType != SkPathIsAType::kRRect);
            crate::SkASSERT!(@source b"src/core/SkPathBuilder.cpp\0".as_ptr(),163; @condition b"fConvexity == SkPathConvexity::kUnknown\0".as_ptr(); (self.fConvexity)==(SkPathConvexity::kUnknown));
            debug_assert_eq!(self.fLastMoveIndex, self.fPts.len() as i32 - 1);
        } else {
            self.fLastMoveIndex = self.fPts.len() as i32;
            self.fPts.push(pt);
            self.fVerbs.push(SkPathVerb::kMove);
            if self.fType == SkPathIsAType::kOval || self.fType == SkPathIsAType::kRRect {
                self.fType = SkPathIsAType::kGeneral;
            }
            self.fConvexity = SkPathConvexity::kUnknown;
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:180-188
impl SkPathBuilder {
    pub fn lineTo(&mut self, pt: SkPoint) -> &mut Self {
        self.ensureMove();
        self.fPts.push(pt);
        self.fVerbs.push(SkPathVerb::kLine);
        self.fSegmentMask |= 1;
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:190-200
impl SkPathBuilder {
    pub fn quadTo(&mut self, p1: SkPoint, p2: SkPoint) -> &mut Self {
        self.ensureMove();
        self.fPts.extend_from_slice(&[p1, p2]);
        self.fVerbs.push(SkPathVerb::kQuad);
        self.fSegmentMask |= 2;
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:202-225
impl SkPathBuilder {
    pub fn conicTo(&mut self, p1: SkPoint, p2: SkPoint, w: f32) -> &mut Self {
        self.ensureMove();
        if w <= 0.0 {
            return self.lineTo(p2);
        }
        self.fPts.extend_from_slice(&[p1, p2]);
        if w == 1.0 {
            self.fVerbs.push(SkPathVerb::kQuad);
            self.fSegmentMask |= 2;
        } else if w.is_finite() {
            self.fVerbs.push(SkPathVerb::kConic);
            self.fConicWeights.push(w);
            self.fSegmentMask |= 4;
        } else {
            self.fVerbs
                .extend_from_slice(&[SkPathVerb::kLine, SkPathVerb::kLine]);
            self.fSegmentMask |= 1;
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:227-238
impl SkPathBuilder {
    pub fn cubicTo(&mut self, p1: SkPoint, p2: SkPoint, p3: SkPoint) -> &mut Self {
        self.ensureMove();
        self.fPts.extend_from_slice(&[p1, p2, p3]);
        self.fVerbs.push(SkPathVerb::kCubic);
        self.fSegmentMask |= 8;
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:240-247
impl SkPathBuilder {
    pub fn close(&mut self) -> &mut Self {
        if !self.fVerbs.is_empty() && self.fVerbs.last() != Some(&SkPathVerb::kClose) {
            self.ensureMove();
            self.fVerbs.push(SkPathVerb::kClose);
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:251-262
impl SkPathBuilder {
    pub fn rMoveTo(&mut self, pt: SkPoint) -> &mut Self {
        let mut last = SkPoint::make(0.0, 0.0);
        if !self.fPts.is_empty() {
            crate::SkASSERT!(@source b"src/core/SkPathBuilder.cpp\0".as_ptr(),254; @condition b"fLastMoveIndex >= 0\0".as_ptr(); self.fLastMoveIndex >= 0);
            last = if self.fVerbs.last() == Some(&SkPathVerb::kClose) {
                self.fPts[self.fLastMoveIndex as usize]
            } else {
                *self.fPts.last().unwrap()
            };
        }
        self.moveTo(last + pt)
    }
}

// Source: src/core/SkPathBuilder.cpp:264-267
impl SkPathBuilder {
    pub fn rLineTo(&mut self, p1: SkPoint) -> &mut Self {
        self.ensureMove();
        let base = *self.fPts.last().unwrap();
        self.lineTo(base + p1)
    }
}

// Source: src/core/SkPathBuilder.cpp:269-273
impl SkPathBuilder {
    pub fn rQuadTo(&mut self, p1: SkPoint, p2: SkPoint) -> &mut Self {
        self.ensureMove();
        let base = *self.fPts.last().unwrap();
        self.quadTo(base + p1, base + p2)
    }
}

// Source: src/core/SkPathBuilder.cpp:275-279
impl SkPathBuilder {
    pub fn rConicTo(&mut self, p1: SkPoint, p2: SkPoint, w: f32) -> &mut Self {
        self.ensureMove();
        let base = *self.fPts.last().unwrap();
        self.conicTo(base + p1, base + p2, w)
    }
}

// Source: src/core/SkPathBuilder.cpp:281-285
impl SkPathBuilder {
    pub fn rCubicTo(&mut self, p1: SkPoint, p2: SkPoint, p3: SkPoint) -> &mut Self {
        self.ensureMove();
        let base = *self.fPts.last().unwrap();
        self.cubicTo(base + p1, base + p2, base + p3)
    }
}

// Source: src/core/SkPathBuilder.cpp:289-307
impl SkPathBuilder {
    pub fn snapshot(&self, mx: Option<&crate::src::core::SkMatrix::SkMatrix>) -> SkPath {
        use crate::src::core::SkPathData::{SkPathData, SkPathDataRef};
        let identity = crate::src::core::SkMatrix::SkMatrix::default();
        let mx = mx.unwrap_or(&identity);
        let mut pdata = SkPathPriv::RawBuilder(self, SkResolveConvexity::kNo)
            .and_then(|raw| SkPathData::MakeTransform(&raw, mx));
        if let Some(data) = pdata.as_mut() {
            if self.fType != SkPathIsAType::kGeneral {
                debug_assert!(SkPathPriv::IsAxisAligned(&self.fPts));
                if mx.rectStaysRect() {
                    let (dir, start) = SkPathPriv::TransformDirAndStart(
                        mx,
                        self.fType == SkPathIsAType::kRRect,
                        self.fIsA.fDirection,
                        self.fIsA.fStartIndex as u32,
                    );
                    SkPathDataRef::get_mut(data)
                        .unwrap()
                        .setupIsA(self.fType, dir, start);
                }
            }
        }
        SkPath::MakeNullCheck(pdata, self.fFillType, self.fIsVolatile)
    }
}

// Source: src/core/SkPathBuilder.cpp:309-313
impl SkPathBuilder {
    pub fn detach(&mut self, mx: Option<&crate::src::core::SkMatrix::SkMatrix>) -> SkPath {
        let path = self.snapshot(mx);
        self.reset();
        path
    }
}

// Source: src/core/SkPathBuilder.cpp:317-340
impl SkPathBuilder {
    pub fn snapshotData(&self) -> Option<crate::src::core::SkPathData::SkPathDataRef> {
        use crate::src::core::SkPathData::SkPathData;
        if self.fVerbs.len() <= 1 {
            return Some(SkPathData::Empty());
        }
        match self.fType {
            SkPathIsAType::kGeneral => {}
            SkPathIsAType::kOval => {
                if let Some(r) = SkRect::Bounds(&self.fPts) {
                    return SkPathData::Oval(
                        &r,
                        self.fIsA.fDirection,
                        self.fIsA.fStartIndex as u32,
                    );
                }
                return None;
            }
            SkPathIsAType::kRRect => {
                if let Some(r) = SkRect::Bounds(&self.fPts) {
                    return SkPathData::RRect(
                        &SkPathPriv::DeduceRRectFromContour(&r, &self.fPts, &self.fVerbs),
                        self.fIsA.fDirection,
                        self.fIsA.fStartIndex as u32,
                    );
                }
                return None;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkPathBuilder.cpp\0".as_ptr(),337; @condition b"fType == SkPathIsAType::kGeneral\0".as_ptr(); (self.fType)==(SkPathIsAType::kGeneral));
        SkPathData::Make(&self.fPts, &self.fVerbs, &self.fConicWeights)
    }
}

// Source: src/core/SkPathBuilder.cpp:342-346
impl SkPathBuilder {
    pub fn detachData(&mut self) -> Option<crate::src::core::SkPathData::SkPathDataRef> {
        let data = self.snapshotData();
        self.reset();
        data
    }
}

// Source: src/core/SkPathBuilder.cpp:350-369
fn arc_is_lone_point(oval: &SkRect, start: f32, sweep: f32, pt: &mut SkPoint) -> bool {
    if sweep == 0.0 && (start == 0.0 || start == 360.0) {
        pt.set(oval.fRight, oval.centerY());
        true
    } else if oval.width() == 0.0 && oval.height() == 0.0 {
        pt.set(oval.fRight, oval.fTop);
        true
    } else {
        false
    }
}

// Source: src/core/SkPathBuilder.cpp:373-405
fn angles_to_unit_vectors(
    start: f32,
    sweep: f32,
    start_v: &mut SkPoint,
    stop_v: &mut SkPoint,
) -> SkPathDirection {
    let start_rad = SkDegreesToRadians(start);
    let mut stop_rad = SkDegreesToRadians(start + sweep);
    start_v.fY = SkScalarSinSnapToZero(start_rad);
    start_v.fX = SkScalarCosSnapToZero(start_rad);
    stop_v.fY = SkScalarSinSnapToZero(stop_rad);
    stop_v.fX = SkScalarCosSnapToZero(stop_rad);
    if start_v == stop_v {
        let sw = sweep.abs();
        if sw < 360.0 && sw > 359.0 {
            let delta = (1.0f32 / 512.0).copysign(sweep);
            loop {
                stop_rad -= delta;
                stop_v.fY = SkScalarSinSnapToZero(stop_rad);
                stop_v.fX = SkScalarCosSnapToZero(stop_rad);
                if start_v != stop_v {
                    break;
                }
            }
        }
    }
    if sweep > 0.0 {
        SkPathDirection::kCW
    } else {
        SkPathDirection::kCCW
    }
}

// Source: src/core/SkPathBuilder.cpp:411-424
fn build_arc_conics(
    oval: &SkRect,
    start: &SkPoint,
    stop: &SkPoint,
    dir: SkPathDirection,
    conics: &mut [SkConic; 5],
    single: &mut SkPoint,
) -> usize {
    let mut matrix = SkMatrix::default();
    matrix.setScale(oval.width() / 2.0, oval.height() / 2.0);
    matrix.postTranslate(oval.centerX(), oval.centerY());
    let count = SkConic::BuildUnitArc(start, stop, dir, Some(&matrix), conics);
    if count == 0 {
        *single = matrix.mapPoint(*stop);
    }
    count
}

// Source: src/core/SkPathBuilder.cpp:426-429
fn nearly_equal(a: SkPoint, b: SkPoint) -> bool {
    crate::include::core::SkScalar::SkScalarNearlyEqual(a.fX, b.fX, SK_ScalarNearlyZero)
        && crate::include::core::SkScalar::SkScalarNearlyEqual(a.fY, b.fY, SK_ScalarNearlyZero)
}

// Source: src/core/SkPathBuilder.cpp:431-494
impl SkPathBuilder {
    pub fn arcTo(
        &mut self,
        oval: &SkRect,
        mut start: f32,
        sweep: f32,
        mut force: bool,
    ) -> &mut Self {
        if oval.width() < 0.0 || oval.height() < 0.0 {
            return self;
        }
        start %= 360.0;
        if self.fVerbs.is_empty() {
            force = true;
        }
        let mut lone = SkPoint::default();
        if arc_is_lone_point(oval, start, sweep, &mut lone) {
            return if force {
                self.moveTo(lone)
            } else {
                self.lineTo(lone)
            };
        }
        let mut start_v = SkPoint::default();
        let mut stop_v = SkPoint::default();
        let dir = angles_to_unit_vectors(start, sweep, &mut start_v, &mut stop_v);
        let mut single = SkPoint::default();
        let add_pt = |builder: &mut Self, pt: SkPoint| {
            if force {
                builder.moveTo(pt);
            } else if !nearly_equal(*builder.fPts.last().unwrap(), pt) {
                builder.lineTo(pt);
            }
        };
        if start_v == stop_v {
            let end = SkDegreesToRadians(start + sweep);
            let rx = oval.width() / 2.0;
            let ry = oval.height() / 2.0;
            single.set(
                oval.centerX() + rx * end.cos(),
                oval.centerY() + ry * end.sin(),
            );
            add_pt(self, single);
            return self;
        }
        let mut conics = [SkConic::default(); 5];
        let count = build_arc_conics(oval, &start_v, &stop_v, dir, &mut conics, &mut single);
        if count > 0 {
            self.incReserve(count * 2 + 1, count * 2 + 1, 0);
            add_pt(self, conics[0].fPts[0]);
            for conic in &conics[..count] {
                self.conicTo(conic.fPts[1], conic.fPts[2], conic.fW);
            }
        } else {
            add_pt(self, single);
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:496-500
impl SkPathBuilder {
    pub fn rArcTo(
        &mut self,
        r: SkPoint,
        axis_rotate: f32,
        large: ArcSize,
        sweep: SkPathDirection,
        delta: SkPoint,
    ) -> &mut Self {
        let current = self.getLastPt().unwrap_or(SkPoint::make(0.0, 0.0));
        self.arcToSvg(r, axis_rotate, large, sweep, current + delta)
    }
}

// Source: src/core/SkPathBuilder.cpp:502-524
impl SkPathBuilder {
    pub fn addArc(&mut self, oval: &SkRect, start: f32, sweep: f32) -> &mut Self {
        if oval.isEmpty() || sweep == 0.0 {
            return self;
        }
        let full = 360.0;
        if sweep >= full || sweep <= -full {
            let over = start / 90.0;
            let rounded = SkScalarRoundToScalar(over);
            let error = over - rounded;
            if crate::include::core::SkScalar::SkScalarNearlyEqual(error, 0.0, SK_ScalarNearlyZero)
            {
                let mut index = (rounded + 1.0) % 4.0;
                if index < 0.0 {
                    index += 4.0;
                }
                return self.addOval(
                    oval,
                    if sweep > 0.0 {
                        SkPathDirection::kCW
                    } else {
                        SkPathDirection::kCCW
                    },
                    index as u32,
                );
            }
        }
        self.arcTo(oval, start, sweep, true)
    }
}

// Source: src/core/SkPathBuilder.cpp:526-560
impl SkPathBuilder {
    pub fn arcToPoints(&mut self, p1: SkPoint, p2: SkPoint, radius: f32) -> &mut Self {
        self.ensureMove();
        if radius == 0.0 {
            return self.lineTo(p1);
        }
        let start = *self.fPts.last().unwrap();
        let normalize = |p: SkPoint| {
            let x = p.fX as f64;
            let y = p.fY as f64;
            let inv = 1.0 / (x * x + y * y).sqrt();
            [x * inv, y * inv]
        };
        let before = normalize(p1 - start);
        let after = normalize(p2 - p1);
        let cosh = before[0] * after[0] + before[1] * after[1];
        let sinh = before[0] * after[1] - before[1] * after[0];
        if !before.iter().all(|x| x.is_finite())
            || !after.iter().all(|x| x.is_finite())
            || crate::include::core::SkScalar::SkScalarNearlyZero(sinh as f32, SK_ScalarNearlyZero)
        {
            return self.lineTo(p1);
        }
        let dist = ((radius as f64 * (1.0 - cosh) / sinh) as f32).abs();
        let xx = (p1.fX as f64 - dist as f64 * before[0]) as f32;
        let yy = (p1.fY as f64 - dist as f64 * before[1]) as f32;
        let mut after = SkPoint::make(after[0] as f32, after[1] as f32);
        after.set_length(dist);
        self.lineTo(SkPoint::make(xx, yy));
        let weight = ((0.5 + cosh * 0.5) as f32).sqrt();
        self.conicTo(p1, p1 + after, weight)
    }
}

// Source: src/core/SkPathBuilder.cpp:568-694
impl SkPathBuilder {
    pub fn arcToSvg(
        &mut self,
        rad: SkPoint,
        angle: f32,
        large: ArcSize,
        sweep: SkPathDirection,
        end: SkPoint,
    ) -> &mut Self {
        self.ensureMove();
        let src = [*self.fPts.last().unwrap(), end];
        if rad.fX == 0.0 || rad.fY == 0.0 {
            return self.lineTo(end);
        }
        if src[0] == src[1] {
            return self.lineTo(end);
        }
        let mut rx = rad.fX.abs();
        let mut ry = rad.fY.abs();
        let mut midpoint = src[0] - src[1];
        midpoint *= 0.5;
        let mut transform = SkMatrix::default();
        transform.setRotate(-angle);
        let transformed = transform.mapPoint(midpoint);
        let sq_rx = rx * rx;
        let sq_ry = ry * ry;
        let sq_x = transformed.fX * transformed.fX;
        let sq_y = transformed.fY * transformed.fY;
        let mut radii_scale = sq_x / sq_rx + sq_y / sq_ry;
        if radii_scale > 1.0 {
            radii_scale = radii_scale.sqrt();
            rx *= radii_scale;
            ry *= radii_scale;
        }
        transform.setScale(1.0 / rx, 1.0 / ry);
        transform.preRotate(-angle);
        let mut unit = [SkPoint::default(); 2];
        transform.mapPoints(&mut unit, &src);
        let mut delta = unit[1] - unit[0];
        let d = delta.fX * delta.fX + delta.fY * delta.fY;
        let candidate = 1.0 / d - 0.25;
        let factor2 = if candidate < 0.0 { 0.0 } else { candidate };
        let mut factor = factor2.sqrt();
        if (sweep == SkPathDirection::kCCW) != (large == ArcSize::kLarge_ArcSize) {
            factor = -factor;
        }
        delta.scale(factor);
        let mut center = unit[0] + unit[1];
        center *= 0.5;
        center.offset(-delta.fY, delta.fX);
        unit[0] -= center;
        unit[1] -= center;
        let theta1 = unit[0].fY.atan2(unit[0].fX);
        let theta2 = unit[1].fY.atan2(unit[1].fX);
        let mut arc = theta2 - theta1;
        if arc < 0.0 && sweep == SkPathDirection::kCW {
            arc += SK_ScalarPI * 2.0;
        } else if arc > 0.0 && sweep != SkPathDirection::kCW {
            arc -= SK_ScalarPI * 2.0;
        }
        if arc.abs() < SK_ScalarPI / 1_000_000.0 {
            return self.lineTo(end);
        }
        transform.setRotate(angle);
        transform.preScale(rx, ry);
        let segments = SkScalarCeilToInt((arc / (2.0 * SK_ScalarPI / 3.0)).abs());
        let width = arc / segments as f32;
        let t = (0.5 * width).tan();
        if !t.is_finite() {
            return self;
        }
        let mut start = theta1;
        let w = (0.5 + width.cos() * 0.5).sqrt();
        let integer = |s: f32| s == s.floor();
        let expect = crate::include::core::SkScalar::SkScalarNearlyZero(
            SK_ScalarPI / 2.0 - width.abs(),
            SK_ScalarNearlyZero,
        ) && integer(rx)
            && integer(ry)
            && integer(end.fX)
            && integer(end.fY);
        for _ in 0..segments {
            let end_theta = start + width;
            let sin = SkScalarSinSnapToZero(end_theta);
            let cos = SkScalarCosSnapToZero(end_theta);
            unit[1].set(cos, sin);
            unit[1] += center;
            unit[0] = unit[1];
            unit[0].offset(t * sin, -t * cos);
            let mut mapped = [SkPoint::default(); 2];
            transform.mapPoints(&mut mapped, &unit);
            if expect {
                for p in &mut mapped {
                    p.fX = SkScalarRoundToScalar(p.fX);
                    p.fY = SkScalarRoundToScalar(p.fY);
                }
            }
            self.conicTo(mapped[0], mapped[1], w);
            start = end_theta;
        }
        *self.fPts.last_mut().unwrap() = end;
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:698-700
impl SkPathBuilder {
    pub fn iter(&self) -> SkPathIter<'_> {
        SkPathIter::new(&self.fPts, &self.fVerbs, &self.fConicWeights)
    }
}

// Source: src/core/SkPathBuilder.cpp:702-733
impl SkPathBuilder {
    pub fn addRaw(&mut self, raw: &SkPathRaw<'_>, reserve: Reserve) -> &mut Self {
        if reserve == Reserve::kGrow {
            self.incReserve(raw.points().len(), raw.verbs().len(), raw.conics().len());
        } else {
            self.fPts.reserve_exact(
                raw.points()
                    .len()
                    .min((i32::MAX as usize).saturating_sub(self.fPts.len())),
            );
            self.fVerbs.reserve_exact(
                raw.verbs()
                    .len()
                    .min((i32::MAX as usize).saturating_sub(self.fVerbs.len())),
            );
            self.fConicWeights.reserve_exact(
                raw.conics()
                    .len()
                    .min((i32::MAX as usize).saturating_sub(self.fConicWeights.len())),
            );
        }
        let mut iter = raw.iter();
        while let Some(rec) = iter.next() {
            let pts = rec.fPoints;
            match rec.fVerb {
                SkPathVerb::kMove => {
                    self.moveTo(pts[0]);
                }
                SkPathVerb::kLine => {
                    self.lineTo(pts[1]);
                }
                SkPathVerb::kQuad => {
                    self.quadTo(pts[1], pts[2]);
                }
                SkPathVerb::kConic => {
                    self.conicTo(pts[1], pts[2], rec.fConicWeight);
                }
                SkPathVerb::kCubic => {
                    self.cubicTo(pts[1], pts[2], pts[3]);
                }
                SkPathVerb::kClose => {
                    self.close();
                }
            }
        }
        if raw.verbs().last() == Some(&SkPathVerb::kMove)
            && self.verbs().last() != Some(&SkPathVerb::kMove)
        {
            self.moveTo(*raw.points().last().unwrap());
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:737-739
fn is_empty_or_moves(v: &[SkPathVerb]) -> bool {
    v.is_empty() || (v.len() == 1 && v.last() == Some(&SkPathVerb::kMove))
}

// Source: src/core/SkPathBuilder.cpp:741-751
impl SkPathBuilder {
    pub fn addRect(&mut self, rect: &SkRect, dir: SkPathDirection, index: u32) -> &mut Self {
        let empty = is_empty_or_moves(&self.fVerbs);
        let shape = crate::src::core::SkPathRawShapes::Rect::new(rect, dir, index);
        self.addRaw(&shape.raw(), Reserve::kGrow);
        if empty {
            self.fConvexity = SkPathDirection_ToConvexity(dir);
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:753-766
impl SkPathBuilder {
    pub fn addOval(&mut self, oval: &SkRect, dir: SkPathDirection, index: u32) -> &mut Self {
        let empty = is_empty_or_moves(&self.fVerbs);
        let shape = crate::src::core::SkPathRawShapes::Oval::new(oval, dir, index);
        self.addRaw(&shape.raw(), Reserve::kGrow);
        if empty {
            self.fType = SkPathIsAType::kOval;
            self.fIsA.fDirection = dir;
            self.fIsA.fStartIndex = (index % 4) as u8;
            self.fConvexity = SkPathDirection_ToConvexity(dir);
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:768-793
impl SkPathBuilder {
    pub fn addRRect(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        dir: SkPathDirection,
        index: u32,
    ) -> &mut Self {
        use crate::src::core::SkPathPriv::RRectAsEnum;
        let bounds = rrect.getBounds();
        let (typ, new_index) = SkPathPriv::SimplifyRRect(rrect, index);
        match typ {
            RRectAsEnum::kRect => return self.addRect(bounds, dir, new_index),
            RRectAsEnum::kOval => return self.addOval(bounds, dir, new_index),
            RRectAsEnum::kRRect => {}
        }
        let empty = is_empty_or_moves(&self.fVerbs);
        let shape = crate::src::core::SkPathRawShapes::RRect::new(rrect, dir, index);
        self.addRaw(&shape.raw(), Reserve::kGrow);
        if empty {
            self.fType = SkPathIsAType::kRRect;
            self.fIsA.fDirection = dir;
            self.fIsA.fStartIndex = (index % 8) as u8;
            self.fConvexity = SkPathDirection_ToConvexity(dir);
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:795-801
impl SkPathBuilder {
    pub fn addCircle(&mut self, center: SkPoint, r: f32, dir: SkPathDirection) -> &mut Self {
        if r >= 0.0 {
            self.addOval(
                &SkRect::MakeLTRB(center.fX - r, center.fY - r, center.fX + r, center.fY + r),
                dir,
                1,
            );
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:803-814
impl SkPathBuilder {
    pub fn addPolygon(&mut self, pts: &[SkPoint], is_closed: bool) -> &mut Self {
        if pts.is_empty() {
            return self;
        }
        self.moveTo(pts[0]);
        self.polylineTo(&pts[1..]);
        if is_closed {
            self.close();
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:816-827
impl SkPathBuilder {
    pub fn polylineTo(&mut self, pts: &[SkPoint]) -> &mut Self {
        if !pts.is_empty() {
            self.ensureMove();
            let count = pts.len();
            self.incReserve(count, count, 0);
            self.fPts.extend_from_slice(pts);
            self.fVerbs
                .resize(self.fVerbs.len() + count, SkPathVerb::kLine);
            self.fSegmentMask |= 1;
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:831-836
impl SkPathBuilder {
    pub fn offset(&mut self, dx: f32, dy: f32) -> &mut Self {
        for p in &mut self.fPts {
            *p += SkPoint::make(dx, dy);
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:838-842
impl SkPathBuilder {
    pub fn addPathOffset(
        &mut self,
        path: &SkPath,
        dx: f32,
        dy: f32,
        mode: crate::src::core::SkPath::AddPathMode,
    ) -> &mut Self {
        let matrix = crate::src::core::SkMatrix::SkMatrix::Translate(dx, dy);
        self.addPath(path, &matrix, mode)
    }
}

// Source: src/core/SkPathBuilder.cpp:844-929
impl SkPathBuilder {
    pub fn addPath(
        &mut self,
        src: &SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        mode: crate::src::core::SkPath::AddPathMode,
    ) -> &mut Self {
        use crate::src::core::SkPath::AddPathMode;
        if src.isEmpty() {
            return self;
        }
        let replace = (mode == AddPathMode::kAppend_AddPathMode && self.verbs().len() <= 1)
            || self.verbs().is_empty();
        if replace && matrix.isIdentity() {
            let fill = self.fFillType;
            self.assign_path(src);
            self.fFillType = fill;
            return self;
        }
        self.fConvexity = SkPathConvexity::kUnknown;
        if mode == AddPathMode::kAppend_AddPathMode && !matrix.hasPerspective() {
            if self.fVerbs.last() == Some(&SkPathVerb::kMove) && !src.isEmpty() {
                debug_assert_eq!(src.verbs()[0], SkPathVerb::kMove);
                self.fVerbs.pop();
                self.fPts.pop();
                debug_assert!(self.fVerbs.last() != Some(&SkPathVerb::kMove));
            }
            let last = SkPathPriv::FindLastMoveToIndex(src.verbs(), src.points().len());
            debug_assert!(last >= 0);
            self.fLastMoveIndex = last + self.countPoints() as i32;
            let (p, c) = self.growForVerbsInPath(src);
            let count = src.points().len();
            let weights = src.conicWeights();
            // Native destination memory is uninitialized until the original map/copy operations complete.
            unsafe {
                let dest = self.fPts.as_mut_ptr().add(p);
                (matrix.getMapPtsProc())(matrix, dest, src.points().as_ptr(), count);
                if !weights.is_empty() {
                    core::ptr::copy_nonoverlapping(
                        weights.as_ptr(),
                        self.fConicWeights.as_mut_ptr().add(c),
                        weights.len(),
                    );
                }
                self.fPts.set_len(p + count);
                self.fConicWeights.set_len(c + weights.len());
            }
            return self;
        }
        let map = matrix.getMapPtsProc();
        let mut first = true;
        for rec in SkPathPriv::Iterate(src) {
            let pts = rec.fPoints;
            let mut mapped = [SkPoint::default(); 3];
            match rec.fVerb {
                SkPathVerb::kMove => {
                    unsafe {
                        map(matrix, mapped.as_mut_ptr(), pts.as_ptr(), 1);
                    }
                    if first && mode == AddPathMode::kExtend_AddPathMode && !self.isEmpty() {
                        self.ensureMove();
                        let last = self.getLastPt();
                        if last != Some(mapped[0]) {
                            self.lineTo(mapped[0]);
                        }
                    } else {
                        self.moveTo(mapped[0]);
                    }
                }
                SkPathVerb::kLine => {
                    unsafe {
                        map(matrix, mapped.as_mut_ptr(), pts.as_ptr().add(1), 1);
                    }
                    self.lineTo(mapped[0]);
                }
                SkPathVerb::kQuad => {
                    unsafe {
                        map(matrix, mapped.as_mut_ptr(), pts.as_ptr().add(1), 2);
                    }
                    self.quadTo(mapped[0], mapped[1]);
                }
                SkPathVerb::kConic => {
                    unsafe {
                        map(matrix, mapped.as_mut_ptr(), pts.as_ptr().add(1), 2);
                    }
                    self.conicTo(mapped[0], mapped[1], rec.fConicWeight);
                }
                SkPathVerb::kCubic => {
                    unsafe {
                        map(matrix, mapped.as_mut_ptr(), pts.as_ptr().add(1), 3);
                    }
                    self.cubicTo(mapped[0], mapped[1], mapped[2]);
                }
                SkPathVerb::kClose => {
                    self.close();
                }
            }
            first = false;
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:932-967
impl SkPathBuilder {
    pub fn privateReversePathTo(&mut self, path: &SkPath) -> &mut Self {
        let verbs = path.verbs();
        if verbs.is_empty() {
            return self;
        }
        let mut pi = path.points().len() as isize - 1;
        let pts = path.points();
        let weights = path.conicWeights();
        let mut wi = weights.len();
        for &v in verbs.iter().rev() {
            pi -= SkPathPriv::PtsInVerb(v as u32) as isize;
            match v {
                SkPathVerb::kMove => return self,
                SkPathVerb::kLine => {
                    self.lineTo(pts[pi as usize]);
                }
                SkPathVerb::kQuad => {
                    self.quadTo(pts[(pi + 1) as usize], pts[pi as usize]);
                }
                SkPathVerb::kConic => {
                    wi -= 1;
                    self.conicTo(pts[(pi + 1) as usize], pts[pi as usize], weights[wi]);
                }
                SkPathVerb::kCubic => {
                    self.cubicTo(
                        pts[(pi + 2) as usize],
                        pts[(pi + 1) as usize],
                        pts[pi as usize],
                    );
                }
                SkPathVerb::kClose => {}
            }
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:969-1019
impl SkPathBuilder {
    pub fn privateReverseAddPath(&mut self, src: &SkPath) -> &mut Self {
        let verbs = src.verbs();
        if verbs.is_empty() {
            return self;
        }
        let pts = src.points();
        let weights = src.conicWeights();
        let mut pi = pts.len() as isize;
        let mut wi = weights.len();
        let mut need_move = true;
        let mut need_close = false;
        for &v in verbs.iter().rev() {
            let n = SkPathPriv::PtsInVerb(v as u32);
            if need_move {
                pi -= 1;
                self.moveTo(pts[pi as usize]);
                need_move = false;
            }
            pi -= n as isize;
            match v {
                SkPathVerb::kMove => {
                    if need_close {
                        self.close();
                        need_close = false;
                    }
                    need_move = true;
                    pi += 1;
                }
                SkPathVerb::kLine => {
                    self.lineTo(pts[pi as usize]);
                }
                SkPathVerb::kQuad => {
                    self.quadTo(pts[(pi + 1) as usize], pts[pi as usize]);
                }
                SkPathVerb::kConic => {
                    wi -= 1;
                    self.conicTo(pts[(pi + 1) as usize], pts[pi as usize], weights[wi]);
                }
                SkPathVerb::kCubic => {
                    self.cubicTo(
                        pts[(pi + 2) as usize],
                        pts[(pi + 1) as usize],
                        pts[pi as usize],
                    );
                }
                SkPathVerb::kClose => need_close = true,
            }
        }
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:1021-1027
impl SkPathBuilder {
    pub fn getLastPt(&self) -> Option<SkPoint> {
        self.fPts.last().copied()
    }
}

// Source: src/core/SkPathBuilder.cpp:1030-1037
impl SkPathBuilder {
    pub fn setLastPt(&mut self, pt: SkPoint) {
        let count = self.fPts.len();
        if count == 0 {
            self.moveTo(pt);
        } else {
            self.setPoint(count - 1, pt);
        }
    }
}
// Source: src/core/SkPathBuilder.cpp:1040-1045
impl SkPathBuilder {
    pub fn setPoint(&mut self, index: usize, p: SkPoint) {
        if index < self.fPts.len() {
            self.fPts[index] = p;
            self.fType = SkPathIsAType::kGeneral;
        }
    }
}

// Source: src/core/SkPathBuilder.cpp:1047-1110
impl SkPathBuilder {
    pub fn transform(&mut self, matrix: &crate::src::core::SkMatrix::SkMatrix) -> &mut Self {
        use crate::src::core::SkGeometry::SkConic;
        if matrix.isIdentity() || self.isEmpty() {
            return self;
        }
        if matrix.hasPerspective() {
            let mut src = self.detach(None);
            self.setFillType(src.getFillType());
            let mut clipped = SkPath::default();
            if SkPathPriv::PerspectiveClip(&src, matrix, &mut clipped) {
                src = clipped;
            }
            let mut iter = SkPathPriv::Iterate(&src).begin();
            while let Some(rec) = iter.next() {
                let pts = rec.fPoints;
                match rec.fVerb {
                    SkPathVerb::kMove => {
                        self.moveTo(pts[0]);
                    }
                    SkPathVerb::kLine => {
                        self.lineTo(pts[1]);
                    }
                    SkPathVerb::kQuad => {
                        self.conicTo(
                            pts[1],
                            pts[2],
                            SkConic::TransformW(pts.try_into().unwrap(), 1.0, matrix),
                        );
                    }
                    SkPathVerb::kConic => {
                        self.conicTo(
                            pts[1],
                            pts[2],
                            SkConic::TransformW(pts.try_into().unwrap(), rec.fConicWeight, matrix),
                        );
                    }
                    SkPathVerb::kCubic => subdivide_cubic_to(self, pts.try_into().unwrap(), 2),
                    SkPathVerb::kClose => {
                        self.close();
                    }
                }
            }
        } else {
            if !matrix.rectStaysRect() || !SkPathPriv::IsAxisAligned(&self.fPts) {
                self.fType = SkPathIsAType::kGeneral;
                if SkPathConvexity_IsConvex(self.fConvexity) {
                    self.fConvexity = SkPathConvexity::kUnknown;
                }
            }
            if self.fType == SkPathIsAType::kOval || self.fType == SkPathIsAType::kRRect {
                let (dir, start) = SkPathPriv::TransformDirAndStart(
                    matrix,
                    self.fType == SkPathIsAType::kRRect,
                    self.fIsA.fDirection,
                    self.fIsA.fStartIndex as u32,
                );
                self.fIsA.fDirection = dir;
                self.fIsA.fStartIndex = start as u8;
            }
        }
        matrix.mapPointsInPlace(&mut self.fPts);
        self
    }
}

// Source: src/core/SkPathBuilder.cpp:1112-1114
impl SkPathBuilder {
    pub fn computeFiniteBounds(&self) -> Option<SkRect> {
        SkPathPriv::TrimmedBounds(self.points(), self.verbs())
    }
}

// Source: src/core/SkPathBuilder.cpp:1116-1121
impl SkPathBuilder {
    pub fn computeTightBounds(&self) -> Option<SkRect> {
        if !self.isFinite() {
            return None;
        }
        Some(SkPathPriv::ComputeTightBounds(
            self.points(),
            self.verbs(),
            self.conicWeights(),
        ))
    }
}

// Source: src/core/SkPathBuilder.cpp:1123-1130
impl SkPathBuilder {
    pub fn isFinite(&self) -> bool {
        self.fPts.iter().all(SkPoint::is_finite)
    }
}

// Source: src/core/SkPathBuilder.cpp:1132-1145
impl SkPathBuilder {
    pub fn isZeroLengthSincePoint(&self, start: i32) -> bool {
        let count = self.fPts.len() as i32 - start;
        if count < 2 {
            return true;
        }
        let pts = &self.fPts[start as usize..];
        let first = pts[0];
        for index in 1..count as usize {
            if first != pts[index] {
                return false;
            }
        }
        true
    }
}

// Source: src/core/SkPathBuilder.cpp:1147-1150
impl SkPathBuilder {
    pub fn contains(&self, p: SkPoint) -> bool {
        SkPathPriv::RawBuilder(self, SkResolveConvexity::kNo)
            .is_some_and(|raw| SkPathPriv::Contains(&raw, p))
    }
}

use crate::include::core::SkScalar::{
    SK_ScalarNearlyZero, SK_ScalarPI, SkDegreesToRadians, SkScalarCeilToInt, SkScalarCosSnapToZero,
    SkScalarRoundToScalar, SkScalarSinSnapToZero,
};
use crate::src::core::SkGeometry::SkConic;
use crate::src::core::SkMatrix::SkMatrix;

// Source: include/core/SkPathBuilder.h:1006-1007
// Diagnostic definitions live in the native SkPathDump implementation module.
impl SkPathBuilder {
    pub fn dumpToString_default(&self) -> crate::src::core::SkString::SkString {
        self.dumpToString(DumpFormat::kDecimal)
    }
    // Source: include/core/SkPathBuilder.h:1010-1010
    pub fn dump_default(&self) {
        self.dump(DumpFormat::kDecimal);
    }
}
