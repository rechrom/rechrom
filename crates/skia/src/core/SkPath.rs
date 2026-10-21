use crate::include::core::SkPathTypes::*;
use crate::src::core::SkPathData::{SkPathData, SkPathDataRef};
use crate::src::core::SkPathEnums::*;
use crate::src::core::SkPathPriv::{RRectAsEnum, SkPathPriv};
use crate::src::core::SkPathRaw::SkPathRaw;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::SkRect;
use std::sync::OnceLock;

use crate::src::core::SkPathPriv::SkPathEdgeIter;
// Source: include/core/SkPath.h:89-92
impl SkPath {
    #[inline]
    pub fn RectDefault(r: &SkRect, dir: SkPathDirection, index: u32) -> Self {
        Self::Rect(r, SkPathFillType::kDefault, dir, index)
    }
}
// Source: include/core/SkPath.h:106-108
impl SkPath {
    #[inline]
    pub fn Line(a: SkPoint, b: SkPoint) -> Self {
        Self::Polygon(&[a, b], false, SkPathFillType::kDefault, false)
    }
}
// Source: include/core/SkPath.h:111-118
// Adapted Source: include/core/SkPath.h:115-117
impl SkPath {
    #[inline]
    pub fn Make(
        pts: &[SkPoint],
        verbs: &[u8],
        conics: &[f32],
        fill: SkPathFillType,
        is_volatile: bool,
    ) -> Self {
        // Rust enums cannot borrow invalid discriminants. Native PathData::Make rejects the same bytes;
        // this extra validation pass must happen before forming a typed borrowed span.
        if verbs.iter().any(|&v| v > SkPathVerb::kClose as u8) {
            return Self::MakeNullCheck(None, fill, is_volatile);
        }
        let typed = unsafe {
            core::slice::from_raw_parts(verbs.as_ptr().cast::<SkPathVerb>(), verbs.len())
        };
        Self::Raw(pts, typed, conics, fill, is_volatile)
    }
}
// Source: include/core/SkPath.h:261-261
impl SkPath {
    #[inline]
    pub fn getFillType(&self) -> SkPathFillType {
        self.fFillType
    }
}

// Source: include/core/SkPath.h:273-273
impl SkPath {
    #[inline]
    pub fn isInverseFillType(&self) -> bool {
        SkPathFillType_IsInverse(self.getFillType())
    }
}

// Source: include/core/SkPath.h:345-347
impl SkPath {
    #[inline]
    pub fn isVolatile(&self) -> bool {
        self.fIsVolatile
    }
}

// Source: include/core/SkPath.h:440-440
impl SkPath {
    #[inline]
    pub fn countPoints(&self) -> usize {
        self.points().len()
    }
}

// Source: include/core/SkPath.h:595-606
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AddPathMode {
    kAppend_AddPathMode,
    kExtend_AddPathMode,
}

// Source: include/core/SkPath.h:618-620
impl SkPath {
    #[inline]
    pub fn tryMakeOffset(&self, dx: f32, dy: f32) -> Option<Self> {
        self.tryMakeTransform(&crate::src::core::SkMatrix::SkMatrix::Translate(dx, dy))
    }
}

// Source: include/core/SkPath.h:622-624
impl SkPath {
    #[inline]
    pub fn tryMakeScale(&self, sx: f32, sy: f32) -> Option<Self> {
        self.tryMakeTransform(&crate::src::core::SkMatrix::SkMatrix::Scale(sx, sy))
    }
}

// Source: include/core/SkPath.h:645-647
impl SkPath {
    #[inline]
    pub fn makeOffset(&self, dx: f32, dy: f32) -> Self {
        self.makeTransform(&crate::src::core::SkMatrix::SkMatrix::Translate(dx, dy))
    }
}

// Source: include/core/SkPath.h:649-651
impl SkPath {
    #[inline]
    pub fn makeScale(&self, sx: f32, sy: f32) -> Self {
        self.makeTransform(&crate::src::core::SkMatrix::SkMatrix::Scale(sx, sy))
    }
}

// Source: include/core/SkPath.h:704-707
impl SkPath {
    #[inline]
    pub fn setIsVolatile(&mut self, is_volatile: bool) -> &mut Self {
        self.fIsVolatile = is_volatile;
        self
    }
}

// Source: include/core/SkPath.h:726-728
impl SkPath {
    #[inline]
    pub fn setFillType(&mut self, ft: SkPathFillType) {
        self.fFillType = ft;
    }
}

// Source: include/core/SkPath.h:733-735
impl SkPath {
    #[inline]
    pub fn toggleInverseFillType(&mut self) {
        self.fFillType = SkPathFillType_ToggleInverse(self.fFillType);
    }
}

// Source: include/core/SkPath.h:749-758
pub struct IterRec<'a> {
    pub fVerb: SkPathVerb,
    pub fPoints: &'a [SkPoint],
    pub fConicWeight: f32,
}
impl IterRec<'_> {
    #[inline]
    pub fn conicWeight(&self) -> f32 {
        crate::SkASSERT!(@source b"include/core/SkPath.h\0".as_ptr(),755; @condition b"fVerb == SkPathVerb::kConic\0".as_ptr(); (self.fVerb)==(SkPathVerb::kConic));
        self.fConicWeight
    }
}

// Source: include/core/SkPath.h:822-822
impl Iter<'_> {
    #[inline]
    pub fn conicWeight(&self) -> f32 {
        self.state.fConicWeights[self.state.conic_index as usize]
    }
}

// Source: include/core/SkPath.h:833-833
impl Iter<'_> {
    #[inline]
    pub fn isCloseLine(&self) -> bool {
        self.state.fCloseLine
    }
}

// Source: include/core/SkPath.h:846-858
#[derive(Clone)]
pub struct Iter<'a> {
    state: IterState<'a>,
    fStorage: [SkPoint; 4],
}
#[derive(Clone)]
struct IterState<'a> {
    fPts: &'a [SkPoint],
    fVerbs: &'a [SkPathVerb],
    fConicWeights: &'a [f32],
    point_index: usize,
    verb_index: usize,
    conic_index: isize,
    fMoveTo: SkPoint,
    fLastPt: SkPoint,
    fForceClose: bool,
    fNeedClose: bool,
    fCloseLine: bool,
}

// Source: include/core/SkPath.h:873-877
impl<'a> RangeIter<'a> {
    #[inline]
    pub fn new(verbs: &'a [SkPathVerb], points: &'a [SkPoint], weights: &'a [f32]) -> Self {
        Self {
            verbs,
            points,
            weights,
            verb_index: 0,
            point_index: 0,
            weight_index: 0,
        }
    }
}

// Source: include/core/SkPath.h:878-883
impl PartialEq for RangeIter<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.verbs.as_ptr().wrapping_add(self.verb_index)
            == other.verbs.as_ptr().wrapping_add(other.verb_index)
    }
}

// Source: include/core/SkPath.h:884-891
impl RangeIter<'_> {
    #[inline]
    pub fn advance(&mut self) {
        let verb = self.verbs[self.verb_index];
        self.verb_index += 1;
        self.point_index += Self::pts_advance_after_verb(verb);
        if verb == SkPathVerb::kConic {
            self.weight_index += 1;
        }
    }
}

// Source: include/core/SkPath.h:892-896
impl RangeIter<'_> {
    #[inline]
    pub fn post_advance(&mut self) -> Self {
        let copy = self.clone();
        self.advance();
        copy
    }
}

// Source: include/core/SkPath.h:897-899
impl RangeIter<'_> {
    #[inline]
    pub fn peekVerb(&self) -> SkPathVerb {
        self.verbs[self.verb_index]
    }
}

// Source: include/core/SkPath.h:900-908
impl<'a> RangeIter<'a> {
    #[inline]
    pub fn record(&self) -> RangeRec<'a> {
        let verb = self.peekVerb();
        let back = Self::pts_backset_for_verb(verb);
        let start = self.point_index.checked_add_signed(back).unwrap();
        let count = if verb == SkPathVerb::kMove {
            1
        } else {
            Self::pts_advance_after_verb(verb) + 1
        };
        RangeRec {
            fVerb: verb,
            fPoints: &self.points[start..start + count],
            fConicWeight: if verb == SkPathVerb::kConic {
                self.weights[self.weight_index]
            } else {
                1.0
            },
        }
    }
}

// Source: include/core/SkPath.h:910-920
impl RangeIter<'_> {
    #[inline]
    fn pts_advance_after_verb(verb: SkPathVerb) -> usize {
        match verb {
            SkPathVerb::kMove | SkPathVerb::kLine => 1,
            SkPathVerb::kQuad | SkPathVerb::kConic => 2,
            SkPathVerb::kCubic => 3,
            SkPathVerb::kClose => 0,
        }
    }
}

// Source: include/core/SkPath.h:921-931
impl RangeIter<'_> {
    #[inline]
    fn pts_backset_for_verb(verb: SkPathVerb) -> isize {
        if verb == SkPathVerb::kMove {
            0
        } else {
            -1
        }
    }
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = RangeRec<'a>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.verb_index == self.verbs.len() {
            return None;
        }
        let rec = self.record();
        self.advance();
        Some(rec)
    }
}

// Source: include/core/SkPath.h:932-936
#[derive(Clone, Default)]
pub struct RangeIter<'a> {
    verbs: &'a [SkPathVerb],
    points: &'a [SkPoint],
    weights: &'a [f32],
    verb_index: usize,
    point_index: usize,
    weight_index: usize,
}
// Rust record borrows the original arrays directly; no trailing Move trim or synthesized Close points.
pub struct RangeRec<'a> {
    pub fVerb: SkPathVerb,
    pub fPoints: &'a [SkPoint],
    pub fConicWeight: f32,
}

// Source: include/core/SkPath.h:957-959
impl<'a> RawIter<'a> {
    #[inline]
    pub fn new(path: &'a SkPath) -> Self {
        let mut iter = Self::default();
        iter.setPath(path);
        iter
    }
}
// Source: include/core/SkPath.h:983-985
impl RawIter<'_> {
    #[inline]
    pub fn peek(&self) -> Option<SkPathVerb> {
        (self.fIter != self.fEnd).then(|| self.fIter.peekVerb())
    }
}
// Source: include/core/SkPath.h:994-996
impl RawIter<'_> {
    #[inline]
    pub fn conicWeight(&self) -> f32 {
        self.fConicWeight
    }
}
// Source: include/core/SkPath.h:999-1001
pub struct RawIter<'a> {
    fIter: RangeIter<'a>,
    fEnd: RangeIter<'a>,
    fConicWeight: f32,
}
// Source: include/core/SkPath.h:1109-1111
#[derive(Clone)]
pub struct SkPath {
    pub(crate) fPathData: SkPathDataRef,
    fFillType: SkPathFillType,
    fIsVolatile: bool,
}

// Source: src/core/SkPath.cpp:31-37
impl SkPath {
    pub(crate) fn from_data(pd: SkPathDataRef, ft: SkPathFillType, is_volatile: bool) -> Self {
        Self {
            fPathData: pd,
            fFillType: ft,
            fIsVolatile: is_volatile,
        }
    }
}

// Source: src/core/SkPath.cpp:39-43
impl SkPath {
    pub fn new(ft: SkPathFillType) -> Self {
        Self {
            fPathData: SkPathData::Empty(),
            fFillType: ft,
            fIsVolatile: false,
        }
    }
}
// Source: include/core/SkPath.h:128-128
impl Default for SkPath {
    #[inline]
    fn default() -> Self {
        Self::new(SkPathFillType::kDefault)
    }
}

// Source: src/core/SkPath.cpp:54-56
impl SkPath {
    pub(crate) fn setConvexity(&self, c: SkPathConvexity) {
        self.fPathData.setConvexity(c);
    }
}

// Source: src/core/SkPath.cpp:58-60
impl SkPath {
    pub(crate) fn getConvexityOrUnknown(&self) -> SkPathConvexity {
        self.fPathData.getConvexityOrUnknown()
    }
}

// Source: src/core/SkPath.cpp:62-65
impl PartialEq for SkPath {
    fn eq(&self, b: &Self) -> bool {
        core::ptr::eq(self, b) || (self.fFillType == b.fFillType && *self.fPathData == *b.fPathData)
    }

    // Source: include/core/SkPath.h:191-193
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !self.eq(b)
    }
}

// Source: src/core/SkPath.cpp:67-73
impl SkPath {
    pub fn swap(&mut self, that: &mut Self) {
        if !core::ptr::eq(self, that) {
            core::mem::swap(&mut self.fPathData, &mut that.fPathData);
            core::mem::swap(&mut self.fFillType, &mut that.fFillType);
            core::mem::swap(&mut self.fIsVolatile, &mut that.fIsVolatile);
        }
    }
}

// Source: src/core/SkPath.cpp:75-78
impl SkPath {
    pub fn reset(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

// Source: src/core/SkPath.cpp:80-82
impl SkPath {
    pub fn getBounds(&self) -> &SkRect {
        self.fPathData.bounds()
    }
}

// Source: src/core/SkPath.cpp:84-86
impl SkPath {
    pub fn getSegmentMasks(&self) -> u32 {
        self.fPathData.segmentMask() as u32
    }
}

// Source: src/core/SkPath.cpp:88-90
impl SkPath {
    pub fn isFinite(&self) -> bool {
        SkPathDataRef::as_ptr(&self.fPathData) != SkPathDataRef::as_ptr(Self::PeekErrorSingleton())
    }
}

// Source: src/core/SkPath.cpp:92-92
impl SkPath {
    pub fn isValid(&self) -> bool {
        self.isFinite()
    }
}

// Source: src/core/SkPath.cpp:94-94
impl SkPath {
    pub fn getGenerationID(&self) -> u32 {
        self.fPathData.uniqueID()
    }
}

// Source: src/core/SkPath.cpp:97-97
impl SkPath {
    #[cfg(sk_debug)]
    pub(crate) fn validate(&self) {}
}

// Source: src/core/SkPath.cpp:100-100
impl SkPath {
    pub fn getOvalInfo(&self) -> Option<crate::include::private::SkPathRef::SkPathOvalInfo> {
        self.fPathData.asOval()
    }
}

// Source: src/core/SkPath.cpp:101-101
impl SkPath {
    pub fn getRRectInfo(&self) -> Option<crate::include::private::SkPathRef::SkPathRRectInfo> {
        self.fPathData.asRRect()
    }
}

// Source: src/core/SkPath.cpp:103-103
impl SkPath {
    pub fn points(&self) -> &[SkPoint] {
        self.fPathData.points()
    }
}

// Source: src/core/SkPath.cpp:104-104
impl SkPath {
    pub fn verbs(&self) -> &[SkPathVerb] {
        self.fPathData.verbs()
    }
}

// Source: src/core/SkPath.cpp:105-105
impl SkPath {
    pub fn conicWeights(&self) -> &[f32] {
        self.fPathData.conics()
    }
}

// Source: src/core/SkPath.cpp:107-110
impl SkPath {
    pub fn Raw(
        pts: &[SkPoint],
        vbs: &[SkPathVerb],
        ws: &[f32],
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> Self {
        Self::MakeNullCheck(SkPathData::Make(pts, vbs, ws), ft, is_volatile)
    }
}

// Source: src/core/SkPath.cpp:112-115
impl SkPath {
    pub fn Rect(
        r: &SkRect,
        ft: SkPathFillType,
        dir: SkPathDirection,
        mut start_index: u32,
    ) -> Self {
        start_index &= 3;
        Self::MakeNullCheck(SkPathData::Rect(r, dir, start_index), ft, false)
    }
}

// Source: src/core/SkPath.cpp:117-120
impl SkPath {
    pub fn Oval(r: &SkRect, dir: SkPathDirection, mut start_index: u32) -> Self {
        start_index &= 3;
        Self::MakeNullCheck(
            SkPathData::Oval(r, dir, start_index),
            SkPathFillType::kDefault,
            false,
        )
    }
}

// Source: src/core/SkPath.cpp:122-140
impl SkPath {
    pub fn RRect(rr: &SkRRect, dir: SkPathDirection, mut start_index: u32) -> Self {
        start_index &= 7;
        let bounds = rr.getBounds();
        let (as_type, new_index) = SkPathPriv::SimplifyRRect(rr, start_index);
        match as_type {
            RRectAsEnum::kRect => {
                return Self::Rect(bounds, SkPathFillType::kDefault, dir, new_index);
            }
            RRectAsEnum::kOval => return Self::Oval(bounds, dir, new_index),
            RRectAsEnum::kRRect => {}
        }
        Self::MakeNullCheck(
            SkPathData::RRect(rr, dir, new_index),
            SkPathFillType::kDefault,
            false,
        )
    }
}

// Source: src/core/SkPath.cpp:142-145
impl SkPath {
    pub fn Polygon(
        pts: &[SkPoint],
        is_closed: bool,
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> Self {
        Self::MakeNullCheck(SkPathData::Polygon(pts, is_closed), ft, is_volatile)
    }
}

// Source: src/core/SkPath.cpp:147-152
impl SkPath {
    pub fn tryMakeTransform(&self, matrix: &crate::src::core::SkMatrix::SkMatrix) -> Option<Self> {
        self.fPathData
            .makeTransform(matrix)
            .map(|data| Self::from_data(data, self.fFillType, self.fIsVolatile))
    }
}

// Source: src/core/SkPath.cpp:154-162
impl SkPath {
    pub fn makeTransform(&self, matrix: &crate::src::core::SkMatrix::SkMatrix) -> Self {
        if !self.isFinite() {
            return self.clone();
        }
        if let Some(path) = self.tryMakeTransform(matrix) {
            return path;
        }
        Self::from_data(Self::PeekErrorSingleton().clone(), self.fFillType, false)
    }
}

// Source: src/core/SkPath.cpp:164-166
impl SkPath {
    pub fn raw(&self, rc: SkResolveConvexity) -> Option<SkPathRaw<'_>> {
        Some(self.fPathData.raw(self.fFillType, rc))
    }
}

// Source: src/core/SkPath.cpp:168-192
fn check_edge_against_rect(p0: SkPoint, p1: SkPoint, rect: &SkRect, dir: SkPathDirection) -> bool {
    let (v, edge) = if dir == SkPathDirection::kCW {
        (p1 - p0, p0)
    } else {
        (p0 - p1, p1)
    };
    if v.fX != 0.0 || v.fY != 0.0 {
        let yl = v.fY * (rect.fLeft - edge.fX);
        let xt = v.fX * (rect.fTop - edge.fY);
        let yr = v.fY * (rect.fRight - edge.fX);
        let xb = v.fX * (rect.fBottom - edge.fY);
        if xt < yl || xt < yr || xb < yl || xb < yr {
            return false;
        }
    }
    true
}

// Source: src/core/SkPath.cpp:194-267
impl SkPath {
    pub fn conservativelyContainsRect(&self, rect: &SkRect) -> bool {
        let convexity = self.getConvexity();
        if !SkPathConvexity_IsConvex(convexity) {
            return false;
        }
        let Some(direction) = SkPathConvexity_ToDirection(convexity) else {
            return false;
        };
        let mut first = SkPoint::default();
        let mut prev = SkPoint::default();
        let mut segments = 0;
        let mut moves = 0;
        let mut iter = SkPathPriv::Iterate(self).begin();
        while let Some(rec) = iter.next() {
            let verb = rec.fVerb;
            let pts = rec.fPoints;
            if verb == SkPathVerb::kClose || (segments > 0 && verb == SkPathVerb::kMove) {
                debug_assert!(moves > 0);
                segments += 1;
                break;
            } else if verb == SkPathVerb::kMove {
                debug_assert_eq!(segments, 0);
                moves += 1;
                first = pts[0];
                prev = pts[0];
            } else {
                let count = SkPathPriv::PtsInVerb(verb as u32);
                debug_assert!(count > 0);
                if !SkPathPriv::AllPointsEq(&pts[..count + 1]) {
                    debug_assert!(moves > 0);
                    segments += 1;
                    if prev == pts[count] {
                        return false;
                    } else if verb == SkPathVerb::kConic {
                        let mut orig = crate::src::core::SkGeometry::SkConic::default();
                        orig.set(pts.try_into().unwrap(), rec.fConicWeight);
                        let mut quad = [SkPoint::default(); 5];
                        let count = orig.chopIntoQuadsPOW2(&mut quad, 1);
                        crate::SkASSERT_RELEASE!(@source b"src/core/SkPath.cpp\0".as_ptr(),245; @condition b"2 == count\0".as_ptr(); (count)==(2));
                        if !check_edge_against_rect(quad[0], quad[2], rect, direction) {
                            return false;
                        }
                        if !check_edge_against_rect(quad[2], quad[4], rect, direction) {
                            return false;
                        }
                    } else if !check_edge_against_rect(prev, pts[count], rect, direction) {
                        return false;
                    }
                    prev = pts[count];
                }
            }
        }
        if segments > 0 {
            return check_edge_against_rect(prev, first, rect, direction);
        }
        false
    }
}

// Source: src/core/SkPath.cpp:269-272
impl SkPath {
    pub fn isLastContourClosed(&self) -> bool {
        let verbs = self.verbs();
        !verbs.is_empty() && verbs.last() == Some(&SkPathVerb::kClose)
    }
}

// Source: src/core/SkPath.cpp:274-287
impl SkPath {
    pub fn isLine(&self, line: Option<&mut [SkPoint; 2]>) -> bool {
        let verbs = self.verbs();
        if verbs.len() == 2 && verbs[1] == SkPathVerb::kLine {
            crate::SkASSERT!(@source b"src/core/SkPath.cpp\0".as_ptr(),277; @condition b"verbs[0] == SkPathVerb::kMove\0".as_ptr(); (verbs[0])==(SkPathVerb::kMove));
            let pts = self.points();
            crate::SkASSERT!(@source b"src/core/SkPath.cpp\0".as_ptr(),279; @condition b"pts.size() == 2\0".as_ptr(); (pts.len())==(2));
            if let Some(line) = line {
                line[0] = pts[0];
                line[1] = pts[1];
            }
            return true;
        }
        false
    }
}

// Source: src/core/SkPath.cpp:289-292
impl SkPath {
    pub fn isEmpty(&self) -> bool {
        #[cfg(sk_debug)]
        self.validate();
        self.verbs().is_empty()
    }
}

// Source: src/core/SkPath.cpp:294-296
impl SkPath {
    pub fn isConvex(&self) -> bool {
        SkPathConvexity_IsConvex(self.getConvexity())
    }
}

// Source: src/core/SkPath.cpp:298-315
impl SkPath {
    pub fn isRect(
        &self,
        rect: Option<&mut SkRect>,
        is_closed: Option<&mut bool>,
        direction: Option<&mut SkPathDirection>,
    ) -> bool {
        #[cfg(sk_debug)]
        self.validate();
        if let Some(rc) =
            SkPathPriv::IsRectContour(self.points(), self.verbs(), self.getSegmentMasks(), false)
        {
            if let Some(rect) = rect {
                *rect = rc.fRect;
            }
            if let Some(closed) = is_closed {
                *closed = rc.fIsClosed;
            }
            if let Some(dir) = direction {
                *dir = rc.fDirection;
            }
            return true;
        }
        false
    }
}

// Source: src/core/SkPath.cpp:317-325
impl SkPath {
    pub fn isOval(&self, bounds: Option<&mut SkRect>) -> bool {
        if let Some(info) = self.getOvalInfo() {
            if let Some(bounds) = bounds {
                *bounds = info.fBounds;
            }
            return true;
        }
        false
    }
}

// Source: src/core/SkPath.cpp:327-335
impl SkPath {
    pub fn isRRect(&self, rrect: Option<&mut SkRRect>) -> bool {
        if let Some(info) = self.getRRectInfo() {
            if let Some(rrect) = rrect {
                *rrect = info.fRRect;
            }
            return true;
        }
        false
    }
}

// Source: src/core/SkPath.cpp:338-345
impl SkPath {
    pub fn getPoints(&self, dst: &mut [SkPoint]) -> usize {
        let src = self.points();
        let n = dst.len().min(src.len());
        dst[..n].copy_from_slice(&src[..n]);
        src.len()
    }
}
// Source: src/core/SkPath.cpp:347-353
impl SkPath {
    pub fn getPoint(&self, index: i32) -> SkPoint {
        let pts = self.points();
        if (index as u32) < pts.len() as u32 {
            pts[index as usize]
        } else {
            SkPoint::make(0.0, 0.0)
        }
    }
}
// Source: src/core/SkPath.cpp:355-362
impl SkPath {
    pub fn getVerbs(&self, dst: &mut [u8]) -> usize {
        let src = self.verbs();
        let n = dst.len().min(src.len());
        for i in 0..n {
            dst[i] = src[i] as u8;
        }
        src.len()
    }
}
// Source: src/core/SkPath.cpp:365-370
impl SkPath {
    pub fn approximateBytesUsed(&self) -> usize {
        core::mem::size_of::<Self>()
            + core::mem::size_of_val(self.points())
            + core::mem::size_of_val(self.verbs())
            + core::mem::size_of_val(self.conicWeights())
    }
}
// Source: src/core/SkPath.cpp:372-379
impl SkPath {
    pub fn getLastPt(&self) -> Option<SkPoint> {
        self.points().last().copied()
    }
}
// Source: src/core/SkPath.cpp:381-390
impl SkPath {
    pub fn getConvexity(&self) -> SkPathConvexity {
        let mut convexity = self.getConvexityOrUnknown();
        if convexity == SkPathConvexity::kUnknown {
            convexity = self.computeConvexity();
        }
        crate::SkASSERT!(@source b"src/core/SkPath.cpp\0".as_ptr(),388; @condition b"convexity != SkPathConvexity::kUnknown\0".as_ptr(); convexity != SkPathConvexity::kUnknown);
        convexity
    }
}

// Source: src/core/SkPath.cpp:392-394
impl SkPath {
    pub fn iter(&self) -> crate::src::core::SkPathIter::SkPathIter<'_> {
        crate::src::core::SkPathIter::SkPathIter::new(
            self.points(),
            self.verbs(),
            self.conicWeights(),
        )
    }
}
// Source: src/core/SkPath.cpp:398-409
impl Default for Iter<'_> {
    fn default() -> Self {
        Self {
            state: IterState {
                fPts: &[],
                fVerbs: &[],
                fConicWeights: &[],
                point_index: 0,
                verb_index: 0,
                conic_index: -1,
                fMoveTo: SkPoint::make(0.0, 0.0),
                fLastPt: SkPoint::make(0.0, 0.0),
                fForceClose: false,
                fNeedClose: false,
                fCloseLine: false,
            },
            fStorage: [SkPoint::default(); 4],
        }
    }
}

// Source: src/core/SkPath.cpp:411-413
impl<'a> Iter<'a> {
    pub fn new(path: &'a SkPath, force_close: bool) -> Self {
        let mut iter = Self::default();
        iter.setPath(path, force_close);
        iter
    }
}

// Source: src/core/SkPath.cpp:415-432
impl<'a> Iter<'a> {
    pub fn setPath(&mut self, path: &'a SkPath, force_close: bool) {
        let s = &mut self.state;
        s.fPts = path.points();
        s.fVerbs = path.verbs();
        s.point_index = 0;
        s.verb_index = 0;
        s.fConicWeights = path.conicWeights();
        s.conic_index = -1;
        s.fLastPt = SkPoint::make(0.0, 0.0);
        s.fMoveTo = SkPoint::make(0.0, 0.0);
        s.fForceClose = force_close;
        s.fNeedClose = false;
    }
}

// Source: src/core/SkPath.cpp:434-460
impl Iter<'_> {
    pub fn isClosedContour(&self) -> bool {
        let s = &self.state;
        if s.verb_index == s.fVerbs.len() {
            return false;
        }
        if s.fForceClose {
            return true;
        }
        let mut i = s.verb_index;
        if s.fVerbs[i] == SkPathVerb::kMove {
            i += 1;
        }
        while i < s.fVerbs.len() {
            let verb = s.fVerbs[i];
            i += 1;
            if verb == SkPathVerb::kMove {
                break;
            }
            if verb == SkPathVerb::kClose {
                return true;
            }
        }
        false
    }
}

// Source: src/core/SkPath.cpp:462-481
impl IterState<'_> {
    fn autoClose(&mut self, pts: &mut [SkPoint]) -> SkPathVerb {
        if self.fLastPt != self.fMoveTo {
            if self.fLastPt.fX.is_nan()
                || self.fLastPt.fY.is_nan()
                || self.fMoveTo.fX.is_nan()
                || self.fMoveTo.fY.is_nan()
            {
                return SkPathVerb::kClose;
            }
            pts[0] = self.fLastPt;
            pts[1] = self.fMoveTo;
            self.fLastPt = self.fMoveTo;
            self.fCloseLine = true;
            return SkPathVerb::kLine;
        }
        pts[0] = self.fMoveTo;
        SkPathVerb::kClose
    }
}

// Source: src/core/SkPath.cpp:483-556
impl IterState<'_> {
    fn next_into(&mut self, pts: &mut [SkPoint; 4]) -> Option<SkPathVerb> {
        if self.verb_index == self.fVerbs.len() {
            if self.fNeedClose {
                if self.autoClose(pts) == SkPathVerb::kLine {
                    return Some(SkPathVerb::kLine);
                }
                self.fNeedClose = false;
                return Some(SkPathVerb::kClose);
            }
            return None;
        }
        let mut verb = self.fVerbs[self.verb_index];
        self.verb_index += 1;
        let mut src = self.point_index;
        match verb {
            SkPathVerb::kMove => {
                if self.fNeedClose {
                    self.verb_index -= 1;
                    verb = self.autoClose(pts);
                    if verb == SkPathVerb::kClose {
                        self.fNeedClose = false;
                    }
                    return Some(verb);
                }
                if self.verb_index == self.fVerbs.len() {
                    return None;
                }
                self.fMoveTo = self.fPts[src];
                pts[0] = self.fPts[src];
                src += 1;
                self.fLastPt = self.fMoveTo;
                self.fNeedClose = self.fForceClose;
            }
            SkPathVerb::kLine => {
                pts[0] = self.fLastPt;
                pts[1] = self.fPts[src];
                self.fLastPt = self.fPts[src];
                self.fCloseLine = false;
                src += 1;
            }
            SkPathVerb::kConic | SkPathVerb::kQuad => {
                if verb == SkPathVerb::kConic {
                    self.conic_index += 1;
                }
                pts[0] = self.fLastPt;
                pts[1..3].copy_from_slice(&self.fPts[src..src + 2]);
                self.fLastPt = self.fPts[src + 1];
                src += 2;
            }
            SkPathVerb::kCubic => {
                pts[0] = self.fLastPt;
                pts[1..4].copy_from_slice(&self.fPts[src..src + 3]);
                self.fLastPt = self.fPts[src + 2];
                src += 3;
            }
            SkPathVerb::kClose => {
                verb = self.autoClose(pts);
                if verb == SkPathVerb::kLine {
                    self.verb_index -= 1;
                } else {
                    self.fNeedClose = false;
                }
                self.fLastPt = self.fMoveTo;
            }
        }
        self.point_index = src;
        Some(verb)
    }
}
impl Iter<'_> {
    pub fn next_into(&mut self, pts: &mut [SkPoint; 4]) -> Option<SkPathVerb> {
        self.state.next_into(pts)
    }
}

// Source: src/core/SkPath.cpp:558-563
fn SkPathIterPointsPerVerb(verb: SkPathVerb) -> usize {
    const COUNTS: [usize; 6] = [1, 2, 3, 3, 4, 0];
    COUNTS[verb as usize]
}

// Source: src/core/SkPath.cpp:565-577
impl Iter<'_> {
    pub fn next(&mut self) -> Option<IterRec<'_>> {
        let verb = self.state.next_into(&mut self.fStorage)?;
        Some(IterRec {
            fVerb: verb,
            fPoints: &self.fStorage[..SkPathIterPointsPerVerb(verb)],
            fConicWeight: if verb == SkPathVerb::kConic {
                self.state.fConicWeights[self.state.conic_index as usize]
            } else {
                1.0
            },
        })
    }
}

// Source: src/core/SkPath.cpp:579-583
impl<'a> RawIter<'a> {
    pub fn setPath(&mut self, path: &'a SkPath) {
        let range = SkPathPriv::Iterate(path);
        self.fIter = range.begin();
        self.fEnd = range.end();
    }
}
// Source: src/core/SkPath.cpp:585-605
impl RawIter<'_> {
    pub fn next_into(&mut self, pts: &mut [SkPoint; 4]) -> Option<SkPathVerb> {
        if self.fIter == self.fEnd {
            return None;
        }
        let rec = self.fIter.record();
        let count = match rec.fVerb {
            SkPathVerb::kMove => 1,
            SkPathVerb::kLine => 2,
            SkPathVerb::kQuad => 3,
            SkPathVerb::kConic => {
                self.fConicWeight = rec.fConicWeight;
                3
            }
            SkPathVerb::kCubic => 4,
            SkPathVerb::kClose => 0,
        };
        pts[..count].copy_from_slice(&rec.fPoints[..count]);
        self.fIter.advance();
        Some(rec.fVerb)
    }
}
// Source: src/core/SkPath.cpp:607-618
impl<'a> RawIter<'a> {
    pub fn next(&mut self) -> Option<IterRec<'a>> {
        if self.fIter == self.fEnd {
            return None;
        }
        let rec = self.fIter.next().unwrap();
        Some(IterRec {
            fVerb: rec.fVerb,
            fPoints: if rec.fVerb == SkPathVerb::kClose {
                &rec.fPoints[..0]
            } else {
                rec.fPoints
            },
            fConicWeight: rec.fConicWeight,
        })
    }
}
// Source: src/core/SkPath.cpp:622-626
impl SkPath {
    pub fn makeFillType(&self, ft: SkPathFillType) -> Self {
        let mut copy = self.clone();
        copy.setFillType(ft);
        copy
    }
}

// Source: src/core/SkPath.cpp:628-630
impl SkPath {
    pub fn makeToggleInverseFillType(&self) -> Self {
        self.makeFillType(SkPathFillType_ToggleInverse(self.fFillType))
    }
}

// Source: src/core/SkPath.cpp:632-636
impl SkPath {
    pub fn makeIsVolatile(&self, v: bool) -> Self {
        let mut copy = self.clone();
        copy.fIsVolatile = v;
        copy
    }
}

// Source: src/core/SkPath.cpp:638-654
impl SkPath {
    pub fn computeConvexity(&self) -> SkPathConvexity {
        let c = self.getConvexityOrUnknown();
        if c != SkPathConvexity::kUnknown {
            return c;
        }
        let mut cv = SkPathConvexity::kConcave;
        if self.isFinite() {
            cv = SkPathPriv::ComputeConvexity(self.points(), self.verbs(), self.conicWeights());
        }
        debug_assert!(cv != SkPathConvexity::kUnknown);
        self.setConvexity(cv);
        cv
    }
}

// Source: src/core/SkPath.cpp:656-659
impl SkPath {
    pub fn contains(&self, p: SkPoint) -> bool {
        SkPathPriv::Raw(self, SkResolveConvexity::kNo)
            .is_some_and(|raw| SkPathPriv::Contains(&raw, p))
    }
}

// Source: src/core/SkPath.cpp:661-665
impl SkPath {
    pub fn ConvertConicToQuads(
        p0: SkPoint,
        p1: SkPoint,
        p2: SkPoint,
        w: f32,
        pts: &mut [SkPoint],
        pow2: usize,
    ) -> usize {
        crate::src::core::SkGeometry::SkConic::new(&[p0, p1, p2], w).chopIntoQuadsPOW2(pts, pow2)
    }
}
// Source: src/core/SkPath.cpp:669-678
impl SkPath {
    pub fn computeTightBounds(&self) -> SkRect {
        if self.getSegmentMasks() == 1 {
            return *self.getBounds();
        }
        SkPathPriv::ComputeTightBounds(self.points(), self.verbs(), self.conicWeights())
    }
}

// Source: src/core/SkPath.cpp:680-682
impl SkPath {
    pub fn IsLineDegenerate(p1: SkPoint, p2: SkPoint, exact: bool) -> bool {
        if exact {
            p1 == p2
        } else {
            crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(p1, p2)
        }
    }
}
// Source: src/core/SkPath.cpp:684-688
impl SkPath {
    pub fn IsQuadDegenerate(p1: SkPoint, p2: SkPoint, p3: SkPoint, exact: bool) -> bool {
        if exact {
            p1 == p2 && p2 == p3
        } else {
            crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(p1, p2)
                && crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(p2, p3)
        }
    }
}
// Source: src/core/SkPath.cpp:690-696
impl SkPath {
    pub fn IsCubicDegenerate(
        p1: SkPoint,
        p2: SkPoint,
        p3: SkPoint,
        p4: SkPoint,
        exact: bool,
    ) -> bool {
        if exact {
            p1 == p2 && p2 == p3 && p3 == p4
        } else {
            crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(p1, p2)
                && crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(p2, p3)
                && crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(p3, p4)
        }
    }
}
// Source: src/core/SkPath.cpp:698-701
impl SkPath {
    pub fn RRectDefault(rr: &SkRRect, dir: SkPathDirection) -> Self {
        Self::RRect(rr, dir, if dir == SkPathDirection::kCW { 6 } else { 7 })
    }
}
// Source: src/core/SkPath.cpp:703-706
impl SkPath {
    pub fn OvalDefault(r: &SkRect, dir: SkPathDirection) -> Self {
        Self::Oval(r, dir, 1)
    }
}
// Source: src/core/SkPath.cpp:708-714
impl SkPath {
    pub fn Circle(x: f32, y: f32, r: f32, dir: SkPathDirection) -> Self {
        if r >= 0.0 {
            Self::OvalDefault(&SkRect::MakeLTRB(x - r, y - r, x + r, y + r), dir)
        } else {
            Self::default()
        }
    }
}
// Source: src/core/SkPath.cpp:716-718
impl SkPath {
    pub fn RRectXY(r: &SkRect, rx: f32, ry: f32, dir: SkPathDirection) -> Self {
        Self::RRectDefault(&SkRRect::MakeRectXY(r, rx, ry), dir)
    }
}
// Source: src/core/SkPath.cpp:720-735
impl SkPathPriv {
    pub fn ComputeFirstDirectionPath(path: &SkPath) -> SkPathFirstDirection {
        let convexity = path.getConvexityOrUnknown();
        if SkPathConvexity_IsConvex(convexity) {
            return SkPathConvexity_ToFirstDirection(convexity);
        }
        if let Some(raw) = Self::Raw(path, SkResolveConvexity::kNo) {
            Self::ComputeFirstDirection(&raw)
        } else {
            SkPathFirstDirection::kUnknown
        }
    }
}
// Source: src/core/SkPath.cpp:742-750
impl SkPath {
    pub(crate) fn PeekErrorSingleton() -> &'static SkPathDataRef {
        static ERROR: OnceLock<SkPathDataRef> = OnceLock::new();
        ERROR.get_or_init(|| {
            let error = SkPathData::MakeNoCheck(&[], &[], &[], None, None)
                .expect("empty error representation is valid");
            debug_assert!(!SkPathDataRef::ptr_eq(&error, &SkPathData::Empty()));
            error
        })
    }
}

// Source: src/core/SkPath.cpp:752-757
impl SkPath {
    pub(crate) fn MakeNullCheck(
        pdata: Option<SkPathDataRef>,
        ft: SkPathFillType,
        is_volatile: bool,
    ) -> Self {
        let data = if let Some(data) = pdata {
            data
        } else {
            Self::PeekErrorSingleton().clone()
        };
        Self::from_data(data, ft, is_volatile)
    }
}

// Source: src/core/SkPath.cpp:761-831
struct SkHalfPlane {
    fA: f32,
    fB: f32,
    fC: f32,
}
#[derive(Clone, Copy)]
enum HalfPlaneResult {
    kAllNegative,
    kAllPositive,
    kMixed,
}
impl SkHalfPlane {
    fn eval(&self, x: f32, y: f32) -> f32 {
        self.fA * x + self.fB * y + self.fC
    }
    fn normalize(&mut self) -> bool {
        let mut a = self.fA as f64;
        let mut b = self.fB as f64;
        let mut c = self.fC as f64;
        let mag = (a * a + b * b).sqrt();
        if mag == 0.0 {
            self.fA = 0.0;
            self.fB = 0.0;
            self.fC = 1.0;
            return true;
        }
        let scale = 1.0 / mag;
        a *= scale;
        b *= scale;
        c *= scale;
        if !a.is_finite() || !b.is_finite() || !c.is_finite() || (a == 0.0 && b == 0.0) {
            self.fA = 0.0;
            self.fB = 0.0;
            self.fC = 1.0;
            return false;
        }
        self.fA = a as f32;
        self.fB = b as f32;
        self.fC = c as f32;
        true
    }
    fn test(&self, bounds: &SkRect) -> HalfPlaneResult {
        let (min_x, max_x) = if self.fA >= 0.0 {
            (bounds.fLeft, bounds.fRight)
        } else {
            (bounds.fRight, bounds.fLeft)
        };
        let (min_y, max_y) = if self.fB >= 0.0 {
            (bounds.fTop, bounds.fBottom)
        } else {
            (bounds.fBottom, bounds.fTop)
        };
        let test = self.eval(min_x, min_y);
        let sign = test * self.eval(max_x, max_y);
        if sign > 0.0 {
            if test < 0.0 {
                HalfPlaneResult::kAllNegative
            } else {
                HalfPlaneResult::kAllPositive
            }
        } else {
            HalfPlaneResult::kMixed
        }
    }
}

// Source: src/core/SkPath.cpp:834-905
fn clip(path: &SkPath, plane: &SkHalfPlane) -> Option<SkPath> {
    use crate::src::core::SkMatrix::SkMatrix;
    use crate::src::core::SkPathBuilder::SkPathBuilder;
    let mut mx = SkMatrix::default();
    let p0 = SkPoint::make(-plane.fA * plane.fC, -plane.fB * plane.fC);
    mx.setAll(
        plane.fB, plane.fA, p0.fX, -plane.fA, plane.fB, p0.fY, 0.0, 0.0, 1.0,
    );
    let inv = mx.invert()?;
    let rotated = path.tryMakeTransform(&inv)?;
    let raw = SkPathPriv::Raw(&rotated, SkResolveConvexity::kNo);
    let Some(raw) = raw else {
        crate::SkASSERT!(@source b"src/core/SkPath.cpp\0".as_ptr(),851; @condition b"false\0".as_ptr(); false);
        return None;
    };
    let clip_rect = SkRect::MakeLTRB(-f32::MAX, 0.0, f32::MAX, f32::MAX);
    let mut result = SkPathBuilder::new();
    let mut prev = SkPoint::make(0.0, 0.0);
    crate::src::core::SkEdgeClipper::SkEdgeClipper::ClipPath(
        &raw,
        &clip_rect,
        false,
        |clipper, mut new_ctr| {
            let mut add_line = false;
            let mut pts = [SkPoint::default(); 4];
            while let Some(verb) = clipper.next(&mut pts) {
                if new_ctr {
                    result.moveTo(pts[0]);
                    prev = pts[0];
                    new_ctr = false;
                }
                if add_line || pts[0] != prev {
                    result.lineTo(pts[0]);
                }
                match verb {
                    SkPathVerb::kLine => {
                        result.lineTo(pts[1]);
                        prev = pts[1];
                    }
                    SkPathVerb::kQuad => {
                        result.quadTo(pts[1], pts[2]);
                        prev = pts[2];
                    }
                    SkPathVerb::kCubic => {
                        result.cubicTo(pts[1], pts[2], pts[3]);
                        prev = pts[3];
                    }
                    _ => {}
                }
                add_line = true;
            }
        },
    );
    result.setFillType(path.getFillType());
    let path = result.detach(Some(&mx));
    if !path.isFinite() {
        return None;
    }
    Some(path)
}

// Source: src/core/SkPath.cpp:908-936
impl SkPathPriv {
    pub fn PerspectiveClip(
        path: &SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        clipped_path: &mut SkPath,
    ) -> bool {
        if !matrix.hasPerspective() {
            return false;
        }
        let mut plane = SkHalfPlane {
            fA: matrix.getPerspX(),
            fB: matrix.getPerspY(),
            fC: matrix.get(8) - Self::kW0PlaneDistance,
        };
        if plane.normalize() {
            match plane.test(path.getBounds()) {
                HalfPlaneResult::kAllPositive => return false,
                HalfPlaneResult::kMixed => {
                    *clipped_path = clip(path, &plane).unwrap_or_default();
                    return true;
                }
                _ => {}
            }
        }
        *clipped_path = SkPath::default();
        true
    }
}

// Source: src/core/SkPath.cpp:938-1034
impl SkPathPriv {
    pub fn IsSimpleRect(
        path: &SkPath,
        isSimpleFill: bool,
    ) -> Option<crate::include::private::SkPathRef::SkPathRectInfo> {
        if path.getSegmentMasks() != kLine_SkPathSegmentMask {
            return None;
        }
        let mut p = [SkPoint::default(); 5];
        let mut count = 0;
        let mut needs_close = !isSimpleFill;
        for rec in Self::Iterate(path) {
            match rec.fVerb {
                SkPathVerb::kMove => {
                    if count != 0 {
                        return None;
                    }
                    p[0] = rec.fPoints[0];
                    count += 1;
                }
                SkPathVerb::kLine => {
                    if count == 5 {
                        return None;
                    }
                    p[count] = rec.fPoints[1];
                    count += 1;
                }
                SkPathVerb::kClose => {
                    if count == 4 {
                        p[4] = p[0];
                        count = 5;
                    }
                    needs_close = false;
                }
                SkPathVerb::kQuad | SkPathVerb::kConic | SkPathVerb::kCubic => return None,
            }
        }
        if needs_close || count < 5 || p[0] != p[4] {
            return None;
        }
        let vertical;
        if p[0].fX == p[3].fX && p[1].fX == p[2].fX && p[0].fY == p[1].fY && p[3].fY == p[2].fY {
            if p[0].fX == p[1].fX || p[0].fY == p[3].fY {
                return None;
            }
            vertical = true;
        } else if p[0].fY == p[3].fY
            && p[1].fY == p[2].fY
            && p[0].fX == p[1].fX
            && p[3].fX == p[2].fX
        {
            if p[0].fY == p[1].fY || p[0].fX == p[3].fX {
                return None;
            }
            vertical = false;
        } else {
            return None;
        }
        let flags =
            (if p[0].fX < p[2].fX { 0 } else { 1 }) | (if p[0].fY < p[2].fY { 0 } else { 2 });
        let (rect, cw, start) = match flags {
            0 => (
                SkRect::MakeLTRB(p[0].fX, p[0].fY, p[2].fX, p[2].fY),
                vertical,
                0,
            ),
            1 => (
                SkRect::MakeLTRB(p[2].fX, p[0].fY, p[0].fX, p[2].fY),
                !vertical,
                1,
            ),
            2 => (
                SkRect::MakeLTRB(p[0].fX, p[2].fY, p[2].fX, p[0].fY),
                !vertical,
                3,
            ),
            _ => (
                SkRect::MakeLTRB(p[2].fX, p[2].fY, p[0].fX, p[0].fY),
                vertical,
                2,
            ),
        };
        Some(crate::include::private::SkPathRef::SkPathRectInfo {
            fRect: rect,
            fDirection: if cw {
                SkPathDirection::kCW
            } else {
                SkPathDirection::kCCW
            },
            fStartIndex: start,
        })
    }
}
// Source: src/core/SkPath.cpp:1038-1050
impl<'a> SkPathEdgeIter<'a> {
    pub fn new(raw: &crate::src::core::SkPathRaw::SkPathRaw<'a>) -> Self {
        Self {
            points: raw.fPoints,
            verbs: raw.fVerbs,
            conics: raw.fConics,
            verb_index: 0,
            point_index: 0,
            move_index: 0,
            conic_index: -1,
            scratch: [SkPoint::default(); 2],
            needs_close_line: false,
            next_is_new_contour: false,
            is_conic: false,
        }
    }
}

// Source: src/core/SkPath.cpp:1052-1054
impl<'a> SkPathEdgeIter<'a> {
    pub fn from_path(path: &'a crate::src::core::SkPath::SkPath) -> Self {
        Self::new(
            &SkPathPriv::Raw(path, SkResolveConvexity::kNo).unwrap_or_else(|| {
                crate::src::core::SkPathRaw::SkPathRaw::Empty(
                    crate::include::core::SkPathTypes::SkPathFillType::kWinding,
                )
            }),
        )
    }
}
// Source: include/core/SkPath.h:154-156
impl SkPath {
    #[inline]
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}
// Source: include/core/SkPath.h:441-441
impl SkPath {
    #[inline]
    pub fn countVerbs(&self) -> i32 {
        let count = self.verbs().len();
        debug_assert!(count <= i32::MAX as usize);
        count as i32
    }
}
// Source: include/core/SkPath.h:480-491
impl SkPath {
    #[inline]
    pub fn getLastPt_to(&self, lastPt: Option<&mut SkPoint>) -> bool {
        if let Some(last) = self.getLastPt() {
            if let Some(dst) = lastPt {
                *dst = last;
            }
            return true;
        }
        if let Some(dst) = lastPt {
            *dst = SkPoint::make(0.0, 0.0);
        }
        false
    }
}
// Source: include/core/SkPath.h:1015-1017
impl SkPath {
    #[inline]
    pub fn containsXY(&self, x: f32, y: f32) -> bool {
        self.contains(SkPoint::make(x, y))
    }
}
// Source: include/core/SkPath.h:950-950
impl Default for RawIter<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            fIter: RangeIter::default(),
            fEnd: RangeIter::default(),
            fConicWeight: 0.0,
        }
    }
}
// Source: include/core/SkPath.h:657-662
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentMask {
    kLine_SegmentMask = kLine_SkPathSegmentMask as u32,
    kQuad_SegmentMask = kQuad_SkPathSegmentMask as u32,
    kConic_SegmentMask = kConic_SkPathSegmentMask as u32,
    kCubic_SegmentMask = kCubic_SkPathSegmentMask as u32,
}
// Source: include/core/SkPath.h:678-686
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    kMove_Verb = SkPathVerb::kMove as u32,
    kLine_Verb = SkPathVerb::kLine as u32,
    kQuad_Verb = SkPathVerb::kQuad as u32,
    kConic_Verb = SkPathVerb::kConic as u32,
    kCubic_Verb = SkPathVerb::kCubic as u32,
    kClose_Verb = SkPathVerb::kClose as u32,
    kDone_Verb = SkPathVerb::kClose as u32 + 1,
}
