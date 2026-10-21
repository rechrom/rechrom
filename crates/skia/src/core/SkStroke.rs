use crate::src::core::SkPaint::{Cap, Join};
impl SkStroke {
    // Source: src/core/SkStroke.h:43-43
    pub fn getCap(&self) -> Cap {
        match self.fCap {
            0 => Cap::kButt_Cap,
            1 => Cap::kRound_Cap,
            2 => Cap::kSquare_Cap,
            _ => unreachable!(),
        }
    }
    // Source: src/core/SkStroke.h:46-46
    pub fn getJoin(&self) -> Join {
        match self.fJoin {
            0 => Join::kMiter_Join,
            1 => Join::kRound_Join,
            2 => Join::kBevel_Join,
            _ => unreachable!(),
        }
    }
    // Source: src/core/SkStroke.h:52-53
    pub fn getDoFill(&self) -> bool {
        self.fDoFill
    }
    pub fn setDoFill(&mut self, doFill: bool) {
        self.fDoFill = doFill;
    }
    // Source: src/core/SkStroke.h:63-67
    pub fn getResScale(&self) -> f32 {
        self.fResScale
    }
    pub fn setResScale(&mut self, rs: f32) {
        debug_assert!(rs > 0.0 && rs.is_finite());
        self.fResScale = rs;
    }
}
// Source: src/core/SkStroke.h:79-82
pub struct SkStroke {
    fWidth: f32,
    fMiterLimit: f32,
    fResScale: f32,
    fCap: u8,
    fJoin: u8,
    fDoFill: bool,
}
use crate::src::core::SkPoint::{SkPoint, SkVector};
use crate::src::core::SkPointPriv::SkPointPriv;
// Source: src/core/SkStroke.cpp:30-35
pub(crate) const kTangent_RecursiveLimit: usize = 0;
pub(crate) const kCubic_RecursiveLimit: usize = 1;
pub(crate) const kConic_RecursiveLimit: usize = 2;
pub(crate) const kQuad_RecursiveLimit: usize = 3;
// Source: src/core/SkStroke.cpp:44-49
pub(crate) const kRecursiveLimits: [i32; 4] = [5 * 3, 24, 11 * 3, 11 * 3];
const _: () = assert!(kTangent_RecursiveLimit == 0);
const _: () = assert!(kCubic_RecursiveLimit == 1);
const _: () = assert!(kRecursiveLimits.len() == kQuad_RecursiveLimit + 1);
// Source: src/core/SkStroke.cpp:101-103
pub(crate) fn degenerate_vector(v: SkVector) -> bool {
    !SkPointPriv::CanNormalize(v.fX, v.fY)
}
// Source: src/core/SkStroke.cpp:105-115
pub(crate) fn set_normal_unitnormal(
    before: SkPoint,
    after: SkPoint,
    scale: f32,
    radius: f32,
    normal: &mut SkVector,
    unitNormal: &mut SkVector,
) -> bool {
    if !unitNormal.set_normalize(
        (after.fX - before.fX) * scale,
        (after.fY - before.fY) * scale,
    ) {
        return false;
    }
    SkPointPriv::RotateCCW(unitNormal);
    unitNormal.scale_to(radius, normal);
    true
}
// Source: src/core/SkStroke.cpp:117-126
pub(crate) fn set_normal_unitnormal_vec(
    vec: SkVector,
    radius: f32,
    normal: &mut SkVector,
    unitNormal: &mut SkVector,
) -> bool {
    if !unitNormal.set_normalize(vec.fX, vec.fY) {
        return false;
    }
    SkPointPriv::RotateCCW(unitNormal);
    unitNormal.scale_to(radius, normal);
    true
}
// Source: src/core/SkStroke.cpp:130-139
pub(crate) struct SkQuadConstruct {
    pub fQuad: [SkPoint; 3],
    pub fTangentStart: SkVector,
    pub fTangentEnd: SkVector,
    pub fStartT: f32,
    pub fMidT: f32,
    pub fEndT: f32,
    pub fStartSet: bool,
    pub fEndSet: bool,
    pub fOppositeTangents: bool,
}
impl SkQuadConstruct {
    // Rust storage initialization replaces C++ automatic indeterminate fields. init/quadEnds
    // overwrite every value before the algorithm reads it; this helper has no ported source lines.
    fn storage() -> Self {
        let z = SkPoint::make(0.0, 0.0);
        Self {
            fQuad: [z; 3],
            fTangentStart: z,
            fTangentEnd: z,
            fStartT: 0.0,
            fMidT: 0.0,
            fEndT: 0.0,
            fStartSet: false,
            fEndSet: false,
            fOppositeTangents: false,
        }
    }
    // Source: src/core/SkStroke.cpp:142-148
    pub fn init(&mut self, start: f32, end: f32) -> bool {
        self.fStartT = start;
        self.fMidT = (start + end) * 0.5;
        self.fEndT = end;
        self.fEndSet = false;
        self.fStartSet = false;
        self.fStartT < self.fMidT && self.fMidT < self.fEndT
    }
    // Source: src/core/SkStroke.cpp:150-158
    pub fn initWithStart(&mut self, parent: &Self) -> bool {
        if !self.init(parent.fStartT, parent.fMidT) {
            return false;
        }
        self.fQuad[0] = parent.fQuad[0];
        self.fTangentStart = parent.fTangentStart;
        self.fStartSet = true;
        true
    }
    // Source: src/core/SkStroke.cpp:160-169
    pub fn initWithEnd(&mut self, parent: &Self) -> bool {
        if !self.init(parent.fMidT, parent.fEndT) {
            return false;
        }
        self.fQuad[2] = parent.fQuad[2];
        self.fTangentEnd = parent.fTangentEnd;
        self.fEndSet = true;
        true
    }
}
// Source: src/core/SkStroke.cpp:171-184
pub(crate) fn isZeroLengthSincePoint(span: &[SkPoint], startPtIndex: i32) -> bool {
    let count = span.len() as i32 - startPtIndex;
    if count < 2 {
        return true;
    }
    let pts = &span[startPtIndex as usize..];
    let first = pts[0];
    for index in 1..count as usize {
        if first != pts[index] {
            return false;
        }
    }
    true
}

use crate::include::core::SkPathTypes::SkPathVerb;
use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SkScalarInvert};
use crate::src::core::SkPath::{Iter, SkPath};
use crate::src::core::SkStrokerPriv::{CapFactory, CapProc, JoinFactory, JoinProc};
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:193-194
    pub fn hasOnlyMoveTo(&self) -> bool {
        self.fSegmentCount == 0
    }
    pub fn moveToPt(&self) -> SkPoint {
        self.fFirstPt
    }
    // Source: src/core/SkStroke.cpp:201-207
    pub fn close(&mut self, isLine: bool) {
        self.finishContour(true, isLine);
    }
    pub fn done(&mut self, dst: &mut SkPathBuilder, isLine: bool) {
        self.finishContour(false, isLine);
        *dst = self.fOuter.clone();
        self.fOuter.reset();
    }
    // Source: src/core/SkStroke.cpp:209-214
    pub fn getResScale(&self) -> f32 {
        self.fResScale
    }
    pub fn isCurrentContourEmpty(&self) -> bool {
        isZeroLengthSincePoint(self.fInner.points(), 0)
            && isZeroLengthSincePoint(self.fOuter.points(), self.fFirstOuterPtIndexInContour)
    }
}
// Source: src/core/SkStroke.cpp:217-263
pub(crate) struct SkPathStroker {
    fRadius: f32,
    fInvMiterLimit: f32,
    fResScale: f32,
    fInvResScale: f32,
    fInvResScaleSquared: f32,
    fFirstNormal: SkVector,
    fPrevNormal: SkVector,
    fFirstUnitNormal: SkVector,
    fPrevUnitNormal: SkVector,
    fFirstPt: SkPoint,
    fPrevPt: SkPoint,
    fFirstOuterPt: SkPoint,
    fFirstOuterPtIndexInContour: i32,
    fSegmentCount: i32,
    fPrevIsLine: bool,
    fCanIgnoreCenter: bool,
    fCapper: CapProc,
    fJoiner: JoinProc,
    // Native member order is inner, outer, cusper; Rust drops fields forward.
    fCusper: SkPathBuilder,
    fOuter: SkPathBuilder,
    fInner: SkPathBuilder,
    fStrokeType: StrokeType,
    // Remaining members are declared in the original class below its nested enums (263-265).
    fRecursionDepth: i32,
    fFoundTangents: bool,
    fJoinCompleted: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum StrokeType {
    kOuter_StrokeType = 1,
    kInner_StrokeType = -1,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ResultType {
    kSplit_ResultType,
    kDegenerate_ResultType,
    kQuad_ResultType,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
enum ReductionType {
    kPoint_ReductionType,
    kLine_ReductionType,
    kQuad_ReductionType,
    kDegenerate_ReductionType,
    kDegenerate2_ReductionType,
    kDegenerate3_ReductionType,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum IntersectRayType {
    kCtrlPt_RayType,
    kResultType_RayType,
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:315-343
    fn preJoinTo(
        &mut self,
        currPt: SkPoint,
        normal: &mut SkVector,
        unitNormal: &mut SkVector,
        currIsLine: bool,
    ) -> bool {
        crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),317; @condition b"fSegmentCount >= 0\0".as_ptr(); self.fSegmentCount>=0);
        if !set_normal_unitnormal(
            self.fPrevPt,
            currPt,
            self.fResScale,
            self.fRadius,
            normal,
            unitNormal,
        ) {
            if std::ptr::fn_addr_eq(CapFactory(Cap::kButt_Cap), self.fCapper) {
                return false;
            }
            normal.set(self.fRadius, 0.0);
            unitNormal.set(1.0, 0.0);
        }
        if self.fSegmentCount == 0 {
            self.fFirstNormal = *normal;
            self.fFirstUnitNormal = *unitNormal;
            self.fFirstOuterPt = self.fPrevPt + *normal;
            self.fOuter.moveTo(self.fFirstOuterPt);
            self.fInner.moveTo(self.fPrevPt - *normal);
        } else {
            (self.fJoiner)(
                &mut self.fOuter,
                &mut self.fInner,
                self.fPrevUnitNormal,
                self.fPrevPt,
                *unitNormal,
                self.fRadius,
                self.fInvMiterLimit,
                self.fPrevIsLine,
                currIsLine,
            );
        }
        self.fPrevIsLine = currIsLine;
        true
    }
    // Source: src/core/SkStroke.cpp:345-352
    fn postJoinTo(&mut self, currPt: SkPoint, normal: SkVector, unitNormal: SkVector) {
        self.fJoinCompleted = true;
        self.fPrevPt = currPt;
        self.fPrevUnitNormal = unitNormal;
        self.fPrevNormal = normal;
        self.fSegmentCount += 1;
    }
    // Source: src/core/SkStroke.cpp:354-393
    fn finishContour(&mut self, close: bool, currIsLine: bool) {
        if self.fSegmentCount > 0 {
            if close {
                (self.fJoiner)(
                    &mut self.fOuter,
                    &mut self.fInner,
                    self.fPrevUnitNormal,
                    self.fPrevPt,
                    self.fFirstUnitNormal,
                    self.fRadius,
                    self.fInvMiterLimit,
                    self.fPrevIsLine,
                    currIsLine,
                );
                self.fOuter.close();
                if self.fCanIgnoreCenter {
                    if self
                        .fInner
                        .computeBounds()
                        .contains(&self.fOuter.computeBounds())
                    {
                        self.fOuter = self.fInner.clone();
                    }
                } else if let Some(pt) = self.fInner.getLastPt() {
                    self.fOuter.moveTo(pt);
                    self.fOuter.privateReversePathTo(&self.fInner.detach(None));
                    self.fOuter.close();
                }
            } else if let Some(pt) = self.fInner.getLastPt() {
                (self.fCapper)(
                    &mut self.fOuter,
                    self.fPrevPt,
                    self.fPrevNormal,
                    pt,
                    currIsLine,
                );
                self.fOuter.privateReversePathTo(&self.fInner.detach(None));
                (self.fCapper)(
                    &mut self.fOuter,
                    self.fFirstPt,
                    -self.fFirstNormal,
                    self.fFirstOuterPt,
                    self.fPrevIsLine,
                );
                self.fOuter.close();
            }
            if !self.fCusper.isEmpty() {
                self.fOuter.addPath(
                    &self.fCusper.detach(None),
                    &crate::src::core::SkMatrix::SkMatrix::default(),
                    crate::src::core::SkPath::AddPathMode::kAppend_AddPathMode,
                );
            }
        }
        self.fInner.reset();
        self.fSegmentCount = -1;
        self.fFirstOuterPtIndexInContour = self.fOuter.countPoints() as i32;
    }
    // Source: src/core/SkStroke.cpp:397-438
    pub fn new(
        src: &SkPath,
        radius: f32,
        miterLimit: f32,
        cap: Cap,
        mut join: Join,
        resScale: f32,
        canIgnoreCenter: bool,
    ) -> Self {
        let mut invMiterLimit = 0.0;
        if join == Join::kMiter_Join {
            if miterLimit <= 1.0 {
                join = Join::kBevel_Join;
            } else {
                invMiterLimit = SkScalarInvert(miterLimit);
            }
        }
        let capper = CapFactory(cap);
        let joiner = JoinFactory(join);
        let mut outer = SkPathBuilder::new();
        let mut inner = SkPathBuilder::new();
        outer.incReserve(src.countPoints() * 3, src.countPoints() * 3, 0);
        outer.setIsVolatile(true);
        inner.incReserve(src.countPoints(), src.countPoints(), 0);
        inner.setIsVolatile(true);
        let invResScale = SkScalarInvert(resScale * 4.0);
        // Rust initializes otherwise indeterminate, unread fields; these initializers are adapted.
        let zero = SkPoint::make(0.0, 0.0);
        Self {
            fRadius: radius,
            fResScale: resScale,
            fCanIgnoreCenter: canIgnoreCenter,
            fInvMiterLimit: invMiterLimit,
            fCapper: capper,
            fJoiner: joiner,
            fSegmentCount: -1,
            fFirstOuterPtIndexInContour: 0,
            fPrevIsLine: false,
            fOuter: outer,
            fInner: inner,
            fCusper: SkPathBuilder::new(),
            fInvResScale: invResScale,
            fInvResScaleSquared: invResScale * invResScale,
            fRecursionDepth: 0,
            fFirstNormal: zero,
            fPrevNormal: zero,
            fFirstUnitNormal: zero,
            fPrevUnitNormal: zero,
            fFirstPt: zero,
            fPrevPt: zero,
            fFirstOuterPt: zero,
            fStrokeType: StrokeType::kOuter_StrokeType,
            fFoundTangents: false,
            fJoinCompleted: false,
        }
    }
    // Source: src/core/SkStroke.cpp:440-447
    pub fn moveTo(&mut self, pt: SkPoint) {
        if self.fSegmentCount > 0 {
            self.finishContour(false, false);
        }
        self.fSegmentCount = 0;
        self.fPrevPt = pt;
        self.fFirstPt = pt;
        self.fJoinCompleted = false;
    }
    // Source: src/core/SkStroke.cpp:449-452
    fn line_to(&mut self, currPt: SkPoint, normal: SkVector) {
        self.fOuter.lineTo(currPt + normal);
        self.fInner.lineTo(currPt - normal);
    }
}
// Source: src/core/SkStroke.cpp:454-482
fn has_valid_tangent(iter: &Iter<'_>) -> bool {
    let mut copy: Iter<'_> = iter.clone();
    while let Some(rec) = copy.next() {
        let pts = rec.fPoints;
        match rec.fVerb {
            SkPathVerb::kMove => return false,
            SkPathVerb::kLine => {
                if pts[0] == pts[1] {
                    continue;
                }
                return true;
            }
            SkPathVerb::kQuad | SkPathVerb::kConic => {
                if pts[0] == pts[1] && pts[0] == pts[2] {
                    continue;
                }
                return true;
            }
            SkPathVerb::kCubic => {
                if pts[0] == pts[1] && pts[0] == pts[2] && pts[0] == pts[3] {
                    continue;
                }
                return true;
            }
            SkPathVerb::kClose => return false,
        }
    }
    false
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:484-499
    pub fn lineTo(&mut self, currPt: SkPoint, iter: Option<&Iter<'_>>) {
        let teenyLine = SkPointPriv::EqualsWithinToleranceWith(
            self.fPrevPt,
            currPt,
            SK_ScalarNearlyZero * self.fInvResScale,
        );
        if std::ptr::fn_addr_eq(CapFactory(Cap::kButt_Cap), self.fCapper) && teenyLine {
            return;
        }
        if teenyLine && (self.fJoinCompleted || iter.is_some_and(has_valid_tangent)) {
            return;
        }
        let mut normal = SkPoint::make(0.0, 0.0);
        let mut unitNormal = SkPoint::make(0.0, 0.0);
        if !self.preJoinTo(currPt, &mut normal, &mut unitNormal, true) {
            return;
        }
        self.line_to(currPt, normal);
        self.postJoinTo(currPt, normal, unitNormal);
    }
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:501-507
    fn setQuadEndNormal(
        &self,
        quad: &[SkPoint; 3],
        normalAB: SkVector,
        unitNormalAB: SkVector,
        normalBC: &mut SkVector,
        unitNormalBC: &mut SkVector,
    ) {
        if !set_normal_unitnormal(
            quad[1],
            quad[2],
            self.fResScale,
            self.fRadius,
            normalBC,
            unitNormalBC,
        ) {
            *normalBC = normalAB;
            *unitNormalBC = unitNormalAB;
        }
    }
    // Source: src/core/SkStroke.cpp:509-512
    fn setConicEndNormal(
        &self,
        conic: &crate::src::core::SkGeometry::SkConic,
        normalAB: SkVector,
        unitNormalAB: SkVector,
        normalBC: &mut SkVector,
        unitNormalBC: &mut SkVector,
    ) {
        self.setQuadEndNormal(&conic.fPts, normalAB, unitNormalAB, normalBC, unitNormalBC);
    }
    // Source: src/core/SkStroke.cpp:514-541
    fn setCubicEndNormal(
        &self,
        cubic: &[SkPoint; 4],
        normalAB: SkVector,
        unitNormalAB: SkVector,
        normalCD: &mut SkVector,
        unitNormalCD: &mut SkVector,
    ) {
        let mut ab = cubic[1] - cubic[0];
        let mut cd = cubic[3] - cubic[2];
        let mut degenerateAB = degenerate_vector(ab);
        let mut degenerateCD = degenerate_vector(cd);
        // The source forward jump skips the fallback tests when both first tangents degenerate.
        if !(degenerateAB && degenerateCD) {
            if degenerateAB {
                ab = cubic[2] - cubic[0];
                degenerateAB = degenerate_vector(ab);
            }
            if degenerateCD {
                cd = cubic[3] - cubic[1];
                degenerateCD = degenerate_vector(cd);
            }
        }
        if degenerateAB || degenerateCD {
            *normalCD = normalAB;
            *unitNormalCD = unitNormalAB;
            return;
        }
        let success = set_normal_unitnormal_vec(cd, self.fRadius, normalCD, unitNormalCD);
        debug_assert!(success);
    }
    // Source: src/core/SkStroke.cpp:543-548
    fn init(
        &mut self,
        strokeType: StrokeType,
        quadPts: &mut SkQuadConstruct,
        tStart: f32,
        tEnd: f32,
    ) {
        self.fStrokeType = strokeType;
        self.fFoundTangents = false;
        quadPts.init(tStart, tEnd);
    }
}
// Source: src/core/SkStroke.cpp:551-563
fn pt_to_line(pt: SkPoint, lineStart: SkPoint, lineEnd: SkPoint) -> f32 {
    let dxy = lineEnd - lineStart;
    let ab0 = pt - lineStart;
    let numer = dxy.dot(ab0);
    let denom = dxy.dot(dxy);
    let t = numer / denom;
    if t >= 0.0 && t <= 1.0 {
        let hit = lineStart * (1.0 - t) + lineEnd * t;
        SkPointPriv::DistanceToSqd(hit, pt)
    } else {
        SkPointPriv::DistanceToSqd(pt, lineStart)
    }
}
// Source: src/core/SkStroke.cpp:566-580
fn pt_to_tangent_line(pt: SkPoint, lineStart: SkPoint, tangent: SkVector) -> f32 {
    let dxy = tangent;
    let ab0 = pt - lineStart;
    let numer = dxy.dot(ab0);
    let denom = dxy.dot(dxy);
    let t = numer / denom;
    if t >= 0.0 && t <= 1.0 {
        let hit = lineStart + tangent * t;
        SkPointPriv::DistanceToSqd(hit, pt)
    } else {
        SkPointPriv::DistanceToSqd(pt, lineStart)
    }
}
// Source: src/core/SkStroke.cpp:605-633
fn cubic_in_line(cubic: &[SkPoint; 4]) -> bool {
    let mut ptMax = -1.0;
    let mut outer1 = 0_usize;
    let mut outer2 = 0_usize;
    for index in 0..3 {
        for inner in index + 1..4 {
            let testDiff = cubic[inner] - cubic[index];
            let a = testDiff.fX.abs();
            let b = testDiff.fY.abs();
            let testMax = if a < b { b } else { a };
            if ptMax < testMax {
                outer1 = index;
                outer2 = inner;
                ptMax = testMax;
            }
        }
    }
    debug_assert!(outer1 <= 2);
    debug_assert!((1..=3).contains(&outer2));
    crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),622; @condition b"outer1 < outer2\0".as_ptr(); outer1<outer2);
    let mid1 = (1 + (2 >> outer2)) >> outer1;
    debug_assert!(mid1 <= 2);
    crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),625; @condition b"outer1 != mid1 && outer2 != mid1\0".as_ptr(); outer1!=mid1&&outer2!=mid1);
    let mid2 = outer1 ^ outer2 ^ mid1;
    debug_assert!((1..=3).contains(&mid2));
    crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),628; @condition b"mid2 != outer1 && mid2 != outer2 && mid2 != mid1\0".as_ptr(); mid2!=outer1&&mid2!=outer2&&mid2!=mid1);
    crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),629; @condition b"((1 << outer1) | (1 << outer2) | (1 << mid1) | (1 << mid2)) == 0x0f\0".as_ptr(); ((1<<outer1)|(1<<outer2)|(1<<mid1)|(1<<mid2))==0x0f);
    let lineSlop = ptMax * ptMax * 0.00001_f32;
    pt_to_line(cubic[mid1], cubic[outer1], cubic[outer2]) <= lineSlop
        && pt_to_line(cubic[mid2], cubic[outer1], cubic[outer2]) <= lineSlop
}
// Source: src/core/SkStroke.cpp:642-664
fn quad_in_line(quad: &[SkPoint; 3]) -> bool {
    let mut ptMax = -1.0;
    let mut outer1 = 0_usize;
    let mut outer2 = 0_usize;
    for index in 0..2 {
        for inner in index + 1..3 {
            let testDiff = quad[inner] - quad[index];
            let a = testDiff.fX.abs();
            let b = testDiff.fY.abs();
            let testMax = if a < b { b } else { a };
            if ptMax < testMax {
                outer1 = index;
                outer2 = inner;
                ptMax = testMax;
            }
        }
    }
    debug_assert!(outer1 <= 1);
    debug_assert!((1..=2).contains(&outer2));
    crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),659; @condition b"outer1 < outer2\0".as_ptr(); outer1<outer2);
    let mid = outer1 ^ outer2 ^ 3;
    let kCurvatureSlop = 0.000005_f32;
    let lineSlop = ptMax * ptMax * kCurvatureSlop;
    pt_to_line(quad[mid], quad[outer1], quad[outer2]) <= lineSlop
}
// Source: src/core/SkStroke.cpp:666-668
fn conic_in_line(conic: &crate::src::core::SkGeometry::SkConic) -> bool {
    quad_in_line(&conic.fPts)
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:670-707
    fn CheckCubicLinear(
        cubic: &[SkPoint; 4],
        reduction: &mut [SkPoint; 3],
        tangentPt: &mut SkPoint,
    ) -> ReductionType {
        use crate::src::core::SkGeometry::{SkEvalCubicAt, SkFindCubicMaxCurvature};
        let degenerateAB = degenerate_vector(cubic[1] - cubic[0]);
        let degenerateBC = degenerate_vector(cubic[2] - cubic[1]);
        let degenerateCD = degenerate_vector(cubic[3] - cubic[2]);
        if degenerateAB & degenerateBC & degenerateCD {
            return ReductionType::kPoint_ReductionType;
        }
        if degenerateAB as u8 + degenerateBC as u8 + degenerateCD as u8 == 2 {
            return ReductionType::kLine_ReductionType;
        }
        if !cubic_in_line(cubic) {
            *tangentPt = if degenerateAB { cubic[2] } else { cubic[1] };
            return ReductionType::kQuad_ReductionType;
        }
        let mut tValues = [0.0; 3];
        let count = SkFindCubicMaxCurvature(cubic, &mut tValues);
        let mut rCount = 0;
        for index in 0..count {
            let t = tValues[index];
            if t <= 0.0 || t >= 1.0 {
                continue;
            }
            SkEvalCubicAt(cubic, t, Some(&mut reduction[rCount]), None, None);
            if reduction[rCount] != cubic[0] && reduction[rCount] != cubic[3] {
                rCount += 1;
            }
        }
        if rCount == 0 {
            return ReductionType::kLine_ReductionType;
        }
        const _: () = assert!(
            ReductionType::kQuad_ReductionType as i32 + 1
                == ReductionType::kDegenerate_ReductionType as i32
        );
        const _: () = assert!(
            ReductionType::kQuad_ReductionType as i32 + 2
                == ReductionType::kDegenerate2_ReductionType as i32
        );
        const _: () = assert!(
            ReductionType::kQuad_ReductionType as i32 + 3
                == ReductionType::kDegenerate3_ReductionType as i32
        );
        match rCount {
            1 => ReductionType::kDegenerate_ReductionType,
            2 => ReductionType::kDegenerate2_ReductionType,
            3 => ReductionType::kDegenerate3_ReductionType,
            _ => unreachable!(),
        }
    }
    // Source: src/core/SkStroke.cpp:709-730
    fn CheckConicLinear(
        conic: &crate::src::core::SkGeometry::SkConic,
        reduction: &mut SkPoint,
    ) -> ReductionType {
        let degenerateAB = degenerate_vector(conic.fPts[1] - conic.fPts[0]);
        let degenerateBC = degenerate_vector(conic.fPts[2] - conic.fPts[1]);
        if degenerateAB & degenerateBC {
            return ReductionType::kPoint_ReductionType;
        }
        if degenerateAB | degenerateBC {
            return ReductionType::kLine_ReductionType;
        }
        if !conic_in_line(conic) {
            return ReductionType::kQuad_ReductionType;
        }
        let t = crate::src::core::SkGeometry::SkFindQuadMaxCurvature(&conic.fPts);
        if t == 0.0 || t.is_nan() {
            return ReductionType::kLine_ReductionType;
        }
        conic.evalAtWithTangent(t, Some(reduction), None);
        ReductionType::kDegenerate_ReductionType
    }
    // Source: src/core/SkStroke.cpp:732-751
    fn CheckQuadLinear(quad: &[SkPoint; 3], reduction: &mut SkPoint) -> ReductionType {
        let degenerateAB = degenerate_vector(quad[1] - quad[0]);
        let degenerateBC = degenerate_vector(quad[2] - quad[1]);
        if degenerateAB & degenerateBC {
            return ReductionType::kPoint_ReductionType;
        }
        if degenerateAB | degenerateBC {
            return ReductionType::kLine_ReductionType;
        }
        if !quad_in_line(quad) {
            return ReductionType::kQuad_ReductionType;
        }
        let t = crate::src::core::SkGeometry::SkFindQuadMaxCurvature(quad);
        if t == 0.0 || t == 1.0 {
            return ReductionType::kLine_ReductionType;
        }
        *reduction = crate::src::core::SkGeometry::SkEvalQuadAt(quad, t);
        ReductionType::kDegenerate_ReductionType
    }
    // Source: src/core/SkStroke.cpp:753-789
    pub fn conicTo(&mut self, pt1: SkPoint, pt2: SkPoint, weight: f32) {
        let conic = crate::src::core::SkGeometry::SkConic::new(&[self.fPrevPt, pt1, pt2], weight);
        let mut reduction = SkPoint::make(0.0, 0.0);
        let reductionType = Self::CheckConicLinear(&conic, &mut reduction);
        if reductionType == ReductionType::kPoint_ReductionType {
            self.lineTo(pt2, None);
            return;
        }
        if reductionType == ReductionType::kLine_ReductionType {
            self.lineTo(pt2, None);
            return;
        }
        if reductionType == ReductionType::kDegenerate_ReductionType {
            self.lineTo(reduction, None);
            let saveJoiner = self.fJoiner;
            self.fJoiner = JoinFactory(Join::kRound_Join);
            self.lineTo(pt2, None);
            self.fJoiner = saveJoiner;
            return;
        }
        debug_assert!(reductionType == ReductionType::kQuad_ReductionType);
        let mut normalAB = SkPoint::make(0.0, 0.0);
        let mut unitAB = normalAB;
        let mut normalBC = normalAB;
        let mut unitBC = normalAB;
        if !self.preJoinTo(pt1, &mut normalAB, &mut unitAB, false) {
            self.lineTo(pt2, None);
            return;
        }
        let mut quadPts = SkQuadConstruct::storage();
        self.init(StrokeType::kOuter_StrokeType, &mut quadPts, 0.0, 1.0);
        self.conicStroke(&conic, &mut quadPts);
        self.init(StrokeType::kInner_StrokeType, &mut quadPts, 0.0, 1.0);
        self.conicStroke(&conic, &mut quadPts);
        self.setConicEndNormal(&conic, normalAB, unitAB, &mut normalBC, &mut unitBC);
        self.postJoinTo(pt2, normalBC, unitBC);
    }
    // Source: src/core/SkStroke.cpp:791-828
    pub fn quadTo(&mut self, pt1: SkPoint, pt2: SkPoint) {
        let quad = [self.fPrevPt, pt1, pt2];
        let mut reduction = SkPoint::make(0.0, 0.0);
        let reductionType = Self::CheckQuadLinear(&quad, &mut reduction);
        if reductionType == ReductionType::kPoint_ReductionType {
            self.lineTo(pt2, None);
            return;
        }
        if reductionType == ReductionType::kLine_ReductionType {
            self.lineTo(pt2, None);
            return;
        }
        if reductionType == ReductionType::kDegenerate_ReductionType {
            self.lineTo(reduction, None);
            let saveJoiner = self.fJoiner;
            self.fJoiner = JoinFactory(Join::kRound_Join);
            self.lineTo(pt2, None);
            self.fJoiner = saveJoiner;
            return;
        }
        debug_assert!(reductionType == ReductionType::kQuad_ReductionType);
        let mut normalAB = SkPoint::make(0.0, 0.0);
        let mut unitAB = normalAB;
        let mut normalBC = normalAB;
        let mut unitBC = normalAB;
        if !self.preJoinTo(pt1, &mut normalAB, &mut unitAB, false) {
            self.lineTo(pt2, None);
            return;
        }
        let mut quadPts = SkQuadConstruct::storage();
        self.init(StrokeType::kOuter_StrokeType, &mut quadPts, 0.0, 1.0);
        self.quadStroke(&quad, &mut quadPts);
        self.init(StrokeType::kInner_StrokeType, &mut quadPts, 0.0, 1.0);
        self.quadStroke(&quad, &mut quadPts);
        self.setQuadEndNormal(&quad, normalAB, unitAB, &mut normalBC, &mut unitBC);
        self.postJoinTo(pt2, normalBC, unitBC);
    }
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:832-843
    fn setRayPts(
        &self,
        tPt: SkPoint,
        dxy: &mut SkVector,
        onPt: &mut SkPoint,
        tangent: Option<&mut SkVector>,
    ) {
        if !dxy.set_length(self.fRadius) {
            dxy.set(self.fRadius, 0.0);
        }
        let axisFlip = self.fStrokeType as i32 as f32;
        onPt.fX = tPt.fX + axisFlip * dxy.fY;
        onPt.fY = tPt.fY - axisFlip * dxy.fX;
        if let Some(tangent) = tangent {
            *tangent = *dxy;
        }
    }
    // Source: src/core/SkStroke.cpp:847-855
    fn conicPerpRay(
        &self,
        conic: &crate::src::core::SkGeometry::SkConic,
        t: f32,
        tPt: &mut SkPoint,
        onPt: &mut SkPoint,
        tangent: Option<&mut SkVector>,
    ) {
        let mut dxy = SkPoint::make(0.0, 0.0);
        conic.evalAtWithTangent(t, Some(tPt), Some(&mut dxy));
        if dxy.is_zero() {
            dxy = conic.fPts[2] - conic.fPts[0];
        }
        self.setRayPts(*tPt, &mut dxy, onPt, tangent);
    }
    // Source: src/core/SkStroke.cpp:858-871
    fn conicQuadEnds(
        &self,
        conic: &crate::src::core::SkGeometry::SkConic,
        quadPts: &mut SkQuadConstruct,
    ) {
        if !quadPts.fStartSet {
            let mut pt = SkPoint::make(0.0, 0.0);
            self.conicPerpRay(
                conic,
                quadPts.fStartT,
                &mut pt,
                &mut quadPts.fQuad[0],
                Some(&mut quadPts.fTangentStart),
            );
            quadPts.fStartSet = true;
        }
        if !quadPts.fEndSet {
            let mut pt = SkPoint::make(0.0, 0.0);
            self.conicPerpRay(
                conic,
                quadPts.fEndT,
                &mut pt,
                &mut quadPts.fQuad[2],
                Some(&mut quadPts.fTangentEnd),
            );
            quadPts.fEndSet = true;
        }
    }
    // Source: src/core/SkStroke.cpp:875-901
    fn cubicPerpRay(
        &self,
        cubic: &[SkPoint; 4],
        t: f32,
        tPt: &mut SkPoint,
        onPt: &mut SkPoint,
        tangent: Option<&mut SkVector>,
    ) {
        use crate::include::core::SkScalar::SkScalarNearlyZero;
        use crate::src::core::SkGeometry::{SkChopCubicAt, SkEvalCubicAt};
        let mut dxy = SkPoint::make(0.0, 0.0);
        let mut chopped = [SkPoint::make(0.0, 0.0); 7];
        SkEvalCubicAt(cubic, t, Some(tPt), Some(&mut dxy), None);
        if dxy.is_zero() {
            let mut cPts: &[SkPoint] = cubic;
            if SkScalarNearlyZero(t, SK_ScalarNearlyZero) {
                dxy = cubic[2] - cubic[0];
            } else if SkScalarNearlyZero(1.0 - t, SK_ScalarNearlyZero) {
                dxy = cubic[3] - cubic[1];
            } else {
                SkChopCubicAt(cubic, &mut chopped, t);
                dxy = chopped[3] - chopped[2];
                if dxy.is_zero() {
                    dxy = chopped[3] - chopped[1];
                    cPts = &chopped;
                }
            }
            if dxy.is_zero() {
                dxy = cPts[3] - cPts[0];
            }
        }
        self.setRayPts(*tPt, &mut dxy, onPt, tangent);
    }
    // Source: src/core/SkStroke.cpp:904-917
    fn cubicQuadEnds(&self, cubic: &[SkPoint; 4], quadPts: &mut SkQuadConstruct) {
        if !quadPts.fStartSet {
            let mut pt = SkPoint::make(0.0, 0.0);
            self.cubicPerpRay(
                cubic,
                quadPts.fStartT,
                &mut pt,
                &mut quadPts.fQuad[0],
                Some(&mut quadPts.fTangentStart),
            );
            quadPts.fStartSet = true;
        }
        if !quadPts.fEndSet {
            let mut pt = SkPoint::make(0.0, 0.0);
            self.cubicPerpRay(
                cubic,
                quadPts.fEndT,
                &mut pt,
                &mut quadPts.fQuad[2],
                Some(&mut quadPts.fTangentEnd),
            );
            quadPts.fEndSet = true;
        }
    }
    // Source: src/core/SkStroke.cpp:919-923
    fn cubicQuadMid(&self, cubic: &[SkPoint; 4], quadPts: &SkQuadConstruct, mid: &mut SkPoint) {
        let mut pt = SkPoint::make(0.0, 0.0);
        self.cubicPerpRay(cubic, quadPts.fMidT, &mut pt, mid, None);
    }
    // Source: src/core/SkStroke.cpp:926-934
    fn quadPerpRay(
        &self,
        quad: &[SkPoint; 3],
        t: f32,
        tPt: &mut SkPoint,
        onPt: &mut SkPoint,
        tangent: Option<&mut SkVector>,
    ) {
        let mut dxy = SkPoint::make(0.0, 0.0);
        crate::src::core::SkGeometry::SkEvalQuadAtWithTangent(quad, t, Some(tPt), Some(&mut dxy));
        if dxy.is_zero() {
            dxy = quad[2] - quad[0];
        }
        self.setRayPts(*tPt, &mut dxy, onPt, tangent);
    }
    // Source: src/core/SkStroke.cpp:939-990
    fn intersectRay(
        &self,
        quadPts: &mut SkQuadConstruct,
        intersectRayType: IntersectRayType,
    ) -> ResultType {
        let start = quadPts.fQuad[0];
        let end = quadPts.fQuad[2];
        let aLen = quadPts.fTangentStart;
        let bLen = quadPts.fTangentEnd;
        let denom = aLen.cross(bLen);
        if denom == 0.0 || !denom.is_finite() {
            quadPts.fOppositeTangents = aLen.dot(bLen) < 0.0;
            return ResultType::kDegenerate_ResultType;
        }
        quadPts.fOppositeTangents = false;
        let ab0 = start - end;
        let mut numerA = bLen.cross(ab0);
        let numerB = aLen.cross(ab0);
        if (numerA >= 0.0) == (numerB >= 0.0) {
            let dist1 = pt_to_tangent_line(start, end, quadPts.fTangentEnd);
            let dist2 = pt_to_tangent_line(end, start, quadPts.fTangentStart);
            let max_dist = if dist1 < dist2 { dist2 } else { dist1 };
            if max_dist <= self.fInvResScaleSquared {
                return ResultType::kDegenerate_ResultType;
            }
            return ResultType::kSplit_ResultType;
        }
        numerA /= denom;
        let validDivide = numerA > numerA - 1.0;
        if validDivide {
            if intersectRayType == IntersectRayType::kCtrlPt_RayType {
                quadPts.fQuad[1] = start + quadPts.fTangentStart * numerA;
            }
            return ResultType::kQuad_ResultType;
        }
        quadPts.fOppositeTangents = aLen.dot(bLen) < 0.0;
        ResultType::kDegenerate_ResultType
    }
}
// Source: src/core/SkStroke.cpp:993-997
impl SkPathStroker {
    fn tangentsMeet(&self, cubic: &[SkPoint; 4], quadPts: &mut SkQuadConstruct) -> ResultType {
        self.cubicQuadEnds(cubic, quadPts);
        self.intersectRay(quadPts, IntersectRayType::kResultType_RayType)
    }
}
// Source: src/core/SkStroke.cpp:1000-1012
fn intersect_quad_ray(line: &[SkPoint; 2], quad: &[SkPoint; 3], roots: &mut [f32; 2]) -> usize {
    let vec = line[1] - line[0];
    let mut r = [0.0; 3];
    for n in 0..3 {
        r[n] = vec.cross(quad[n] - line[0]);
    }
    let mut A = r[2];
    let mut B = r[1];
    let C = r[0];
    A += C - 2.0 * B;
    B -= C;
    crate::src::core::SkGeometry::SkFindUnitQuadRoots(A, 2.0 * B, C, roots)
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:1015-1033
    fn ptInQuadBounds(&self, quad: &[SkPoint; 3], pt: SkPoint) -> bool {
        let mut xMin = quad[0].fX;
        for p in &quad[1..] {
            if p.fX < xMin {
                xMin = p.fX;
            }
        }
        if pt.fX + self.fInvResScale < xMin {
            return false;
        }
        let mut xMax = quad[0].fX;
        for p in &quad[1..] {
            if xMax < p.fX {
                xMax = p.fX;
            }
        }
        if pt.fX - self.fInvResScale > xMax {
            return false;
        }
        let mut yMin = quad[0].fY;
        for p in &quad[1..] {
            if p.fY < yMin {
                yMin = p.fY;
            }
        }
        if pt.fY + self.fInvResScale < yMin {
            return false;
        }
        let mut yMax = quad[0].fY;
        for p in &quad[1..] {
            if yMax < p.fY {
                yMax = p.fY;
            }
        }
        if pt.fY - self.fInvResScale > yMax {
            return false;
        }
        true
    }
}
// Source: src/core/SkStroke.cpp:1035-1037
fn points_within_dist(nearPt: SkPoint, farPt: SkPoint, limit: f32) -> bool {
    SkPointPriv::DistanceToSqd(nearPt, farPt) <= limit * limit
}
// Source: src/core/SkStroke.cpp:1039-1054
fn sharp_angle(quad: &[SkPoint; 3]) -> bool {
    let mut smaller = quad[1] - quad[0];
    let mut larger = quad[1] - quad[2];
    let smallerLen = SkPointPriv::LengthSqd(smaller);
    let mut largerLen = SkPointPriv::LengthSqd(larger);
    if smallerLen > largerLen {
        std::mem::swap(&mut smaller, &mut larger);
        largerLen = smallerLen;
    }
    if !smaller.set_length(largerLen) {
        return false;
    }
    let dot = smaller.dot(larger);
    dot > 0.0
}
impl SkPathStroker {
    // Source: src/core/SkStroke.cpp:1056-1103
    fn strokeCloseEnough(
        &self,
        stroke: &[SkPoint; 3],
        ray: &[SkPoint; 2],
        quadPts: &SkQuadConstruct,
    ) -> ResultType {
        use crate::src::core::SkGeometry::SkEvalQuadAt;
        let strokeMid = SkEvalQuadAt(stroke, 0.5);
        if points_within_dist(ray[0], strokeMid, self.fInvResScale) {
            if sharp_angle(&quadPts.fQuad) {
                return ResultType::kSplit_ResultType;
            }
            return ResultType::kQuad_ResultType;
        }
        if !self.ptInQuadBounds(stroke, ray[0]) {
            return ResultType::kSplit_ResultType;
        }
        let mut roots = [0.0; 2];
        let rootCount = intersect_quad_ray(ray, stroke, &mut roots);
        if rootCount != 1 {
            return ResultType::kSplit_ResultType;
        }
        let quadPt = SkEvalQuadAt(stroke, roots[0]);
        let error = self.fInvResScale * (1.0 - (roots[0] - 0.5_f32).abs() * 2.0);
        if points_within_dist(ray[0], quadPt, error) {
            if sharp_angle(&quadPts.fQuad) {
                return ResultType::kSplit_ResultType;
            }
            return ResultType::kQuad_ResultType;
        }
        ResultType::kSplit_ResultType
    }
    // Source: src/core/SkStroke.cpp:1105-1119
    fn compareQuadCubic(&self, cubic: &[SkPoint; 4], quadPts: &mut SkQuadConstruct) -> ResultType {
        self.cubicQuadEnds(cubic, quadPts);
        let resultType = self.intersectRay(quadPts, IntersectRayType::kCtrlPt_RayType);
        if resultType != ResultType::kQuad_ResultType {
            return resultType;
        }
        let mut ray0 = SkPoint::make(0.0, 0.0);
        let mut ray1 = ray0;
        self.cubicPerpRay(cubic, quadPts.fMidT, &mut ray1, &mut ray0, None);
        self.strokeCloseEnough(&quadPts.fQuad, &[ray0, ray1], quadPts)
    }
    // Source: src/core/SkStroke.cpp:1121-1135
    fn compareQuadConic(
        &self,
        conic: &crate::src::core::SkGeometry::SkConic,
        quadPts: &mut SkQuadConstruct,
    ) -> ResultType {
        self.conicQuadEnds(conic, quadPts);
        let resultType = self.intersectRay(quadPts, IntersectRayType::kCtrlPt_RayType);
        if resultType != ResultType::kQuad_ResultType {
            return resultType;
        }
        let mut ray0 = SkPoint::make(0.0, 0.0);
        let mut ray1 = ray0;
        self.conicPerpRay(conic, quadPts.fMidT, &mut ray1, &mut ray0, None);
        self.strokeCloseEnough(&quadPts.fQuad, &[ray0, ray1], quadPts)
    }
    // Source: src/core/SkStroke.cpp:1137-1162
    fn compareQuadQuad(&self, quad: &[SkPoint; 3], quadPts: &mut SkQuadConstruct) -> ResultType {
        if !quadPts.fStartSet {
            let mut pt = SkPoint::make(0.0, 0.0);
            self.quadPerpRay(
                quad,
                quadPts.fStartT,
                &mut pt,
                &mut quadPts.fQuad[0],
                Some(&mut quadPts.fTangentStart),
            );
            quadPts.fStartSet = true;
        }
        if !quadPts.fEndSet {
            let mut pt = SkPoint::make(0.0, 0.0);
            self.quadPerpRay(
                quad,
                quadPts.fEndT,
                &mut pt,
                &mut quadPts.fQuad[2],
                Some(&mut quadPts.fTangentEnd),
            );
            quadPts.fEndSet = true;
        }
        let resultType = self.intersectRay(quadPts, IntersectRayType::kCtrlPt_RayType);
        if resultType != ResultType::kQuad_ResultType {
            return resultType;
        }
        let mut ray0 = SkPoint::make(0.0, 0.0);
        let mut ray1 = ray0;
        self.quadPerpRay(quad, quadPts.fMidT, &mut ray1, &mut ray0, None);
        self.strokeCloseEnough(&quadPts.fQuad, &[ray0, ray1], quadPts)
    }
    // Source: src/core/SkStroke.cpp:1164-1168
    fn addDegenerateLine(&mut self, quadPts: &SkQuadConstruct) {
        let sink = if self.fStrokeType == StrokeType::kOuter_StrokeType {
            &mut self.fOuter
        } else {
            &mut self.fInner
        };
        sink.lineTo(quadPts.fQuad[2]);
    }
    // Source: src/core/SkStroke.cpp:1170-1175
    fn cubicMidOnLine(&self, cubic: &[SkPoint; 4], quadPts: &SkQuadConstruct) -> bool {
        let mut strokeMid = SkPoint::make(0.0, 0.0);
        self.cubicQuadMid(cubic, quadPts, &mut strokeMid);
        let dist = pt_to_line(strokeMid, quadPts.fQuad[0], quadPts.fQuad[2]);
        dist < self.fInvResScaleSquared
    }
    // Source: src/core/SkStroke.cpp:1177-1244
    fn cubicStroke(&mut self, cubic: &[SkPoint; 4], quadPts: &mut SkQuadConstruct) -> bool {
        if !self.fFoundTangents {
            let resultType = self.tangentsMeet(cubic, quadPts);
            if resultType != ResultType::kQuad_ResultType {
                if (resultType == ResultType::kDegenerate_ResultType
                    || points_within_dist(quadPts.fQuad[0], quadPts.fQuad[2], self.fInvResScale))
                    && self.cubicMidOnLine(cubic, quadPts)
                {
                    self.addDegenerateLine(quadPts);
                    return true;
                }
            } else {
                self.fFoundTangents = true;
            }
        }
        if self.fFoundTangents {
            let resultType = self.compareQuadCubic(cubic, quadPts);
            if resultType == ResultType::kQuad_ResultType {
                let sink = if self.fStrokeType == StrokeType::kOuter_StrokeType {
                    &mut self.fOuter
                } else {
                    &mut self.fInner
                };
                sink.quadTo(quadPts.fQuad[1], quadPts.fQuad[2]);
                return true;
            }
            if resultType == ResultType::kDegenerate_ResultType && !quadPts.fOppositeTangents {
                self.addDegenerateLine(quadPts);
                return true;
            }
        }
        if !quadPts.fQuad[2].is_finite() {
            return false;
        }
        self.fRecursionDepth += 1;
        if self.fRecursionDepth > kRecursiveLimits[self.fFoundTangents as usize] {
            self.addDegenerateLine(quadPts);
            return true;
        }
        let mut half = SkQuadConstruct::storage();
        if !half.initWithStart(quadPts) {
            self.addDegenerateLine(quadPts);
            self.fRecursionDepth -= 1;
            return true;
        }
        if !self.cubicStroke(cubic, &mut half) {
            return false;
        }
        if !half.initWithEnd(quadPts) {
            self.addDegenerateLine(quadPts);
            self.fRecursionDepth -= 1;
            return true;
        }
        if !self.cubicStroke(cubic, &mut half) {
            return false;
        }
        self.fRecursionDepth -= 1;
        true
    }
    // Source: src/core/SkStroke.cpp:1246-1278
    fn conicStroke(
        &mut self,
        conic: &crate::src::core::SkGeometry::SkConic,
        quadPts: &mut SkQuadConstruct,
    ) -> bool {
        let resultType = self.compareQuadConic(conic, quadPts);
        if resultType == ResultType::kQuad_ResultType {
            let sink = if self.fStrokeType == StrokeType::kOuter_StrokeType {
                &mut self.fOuter
            } else {
                &mut self.fInner
            };
            sink.quadTo(quadPts.fQuad[1], quadPts.fQuad[2]);
            return true;
        }
        if resultType == ResultType::kDegenerate_ResultType {
            self.addDegenerateLine(quadPts);
            return true;
        }
        self.fRecursionDepth += 1;
        if self.fRecursionDepth > kRecursiveLimits[kConic_RecursiveLimit] {
            self.addDegenerateLine(quadPts);
            return true;
        }
        let mut half = SkQuadConstruct::storage();
        half.initWithStart(quadPts);
        if !self.conicStroke(conic, &mut half) {
            return false;
        }
        half.initWithEnd(quadPts);
        if !self.conicStroke(conic, &mut half) {
            return false;
        }
        self.fRecursionDepth -= 1;
        true
    }
    // Source: src/core/SkStroke.cpp:1280-1312
    fn quadStroke(&mut self, quad: &[SkPoint; 3], quadPts: &mut SkQuadConstruct) -> bool {
        let resultType = self.compareQuadQuad(quad, quadPts);
        if resultType == ResultType::kQuad_ResultType {
            let sink = if self.fStrokeType == StrokeType::kOuter_StrokeType {
                &mut self.fOuter
            } else {
                &mut self.fInner
            };
            sink.quadTo(quadPts.fQuad[1], quadPts.fQuad[2]);
            return true;
        }
        if resultType == ResultType::kDegenerate_ResultType {
            self.addDegenerateLine(quadPts);
            return true;
        }
        self.fRecursionDepth += 1;
        if self.fRecursionDepth > kRecursiveLimits[kQuad_RecursiveLimit] {
            self.addDegenerateLine(quadPts);
            return true;
        }
        let mut half = SkQuadConstruct::storage();
        half.initWithStart(quadPts);
        if !self.quadStroke(quad, &mut half) {
            return false;
        }
        half.initWithEnd(quadPts);
        if !self.quadStroke(quad, &mut half) {
            return false;
        }
        self.fRecursionDepth -= 1;
        true
    }
}
// Source: src/core/SkStroke.cpp:1314-1374
impl SkPathStroker {
    pub fn cubicTo(&mut self, pt1: SkPoint, pt2: SkPoint, pt3: SkPoint) {
        use crate::src::core::SkGeometry::{
            SkEvalCubicAt, SkFindCubicCusp, SkFindCubicInflections,
        };
        let cubic = [self.fPrevPt, pt1, pt2, pt3];
        let zero = SkPoint::make(0.0, 0.0);
        let mut reduction = [zero; 3];
        let mut tangentPt = zero;
        let reductionType = Self::CheckCubicLinear(&cubic, &mut reduction, &mut tangentPt);
        if reductionType == ReductionType::kPoint_ReductionType {
            self.lineTo(pt3, None);
            return;
        }
        if reductionType == ReductionType::kLine_ReductionType {
            self.lineTo(pt3, None);
            return;
        }
        if reductionType >= ReductionType::kDegenerate_ReductionType
            && reductionType <= ReductionType::kDegenerate3_ReductionType
        {
            self.lineTo(reduction[0], None);
            let saveJoiner = self.fJoiner;
            self.fJoiner = JoinFactory(Join::kRound_Join);
            if reductionType >= ReductionType::kDegenerate2_ReductionType {
                self.lineTo(reduction[1], None);
            }
            if reductionType == ReductionType::kDegenerate3_ReductionType {
                self.lineTo(reduction[2], None);
            }
            self.lineTo(pt3, None);
            self.fJoiner = saveJoiner;
            return;
        }
        debug_assert!(reductionType == ReductionType::kQuad_ReductionType);
        let mut normalAB = zero;
        let mut unitAB = zero;
        let mut normalCD = zero;
        let mut unitCD = zero;
        if !self.preJoinTo(tangentPt, &mut normalAB, &mut unitAB, false) {
            self.lineTo(pt3, None);
            return;
        }
        let mut tValues = [0.0; 2];
        let count = SkFindCubicInflections(&cubic, &mut tValues);
        let mut lastT = 0.0;
        for index in 0..=count {
            let nextT = if index < count { tValues[index] } else { 1.0 };
            let mut quadPts = SkQuadConstruct::storage();
            self.init(StrokeType::kOuter_StrokeType, &mut quadPts, lastT, nextT);
            self.cubicStroke(&cubic, &mut quadPts);
            self.init(StrokeType::kInner_StrokeType, &mut quadPts, lastT, nextT);
            self.cubicStroke(&cubic, &mut quadPts);
            lastT = nextT;
        }
        let cusp = SkFindCubicCusp(&cubic);
        if cusp > 0.0 {
            let mut cuspLoc = zero;
            SkEvalCubicAt(&cubic, cusp, Some(&mut cuspLoc), None, None);
            self.fCusper
                .addCircle(cuspLoc, self.fRadius, SkPathDirection::kCW);
        }
        self.setCubicEndNormal(&cubic, normalAB, unitAB, &mut normalCD, &mut unitCD);
        self.postJoinTo(pt3, normalCD, unitCD);
    }
}
impl SkStroke {
    // Source: src/core/SkStroke.cpp:1381-1388
    pub fn new() -> Self {
        Self {
            fWidth: 1.0,
            fMiterLimit: crate::src::core::SkPaintDefaults::SkPaintDefaults_MiterLimit,
            fResScale: 1.0,
            fCap: Cap::kDefault_Cap as u8,
            fJoin: Join::kDefault_Join as u8,
            fDoFill: false,
        }
    }
    // Source: src/core/SkStroke.cpp:1390-1397
    pub fn from_paint(p: &crate::src::core::SkPaint::SkPaint) -> Self {
        Self {
            fWidth: p.getStrokeWidth(),
            fMiterLimit: p.getStrokeMiter(),
            fResScale: 1.0,
            fCap: p.getStrokeCap() as u8,
            fJoin: p.getStrokeJoin() as u8,
            fDoFill: p.getStyle() == crate::src::core::SkPaint::Style::kStrokeAndFill_Style,
        }
    }
    // Source: src/core/SkStroke.cpp:1399-1406
    pub fn from_paint_width(p: &crate::src::core::SkPaint::SkPaint, width: f32) -> Self {
        Self {
            fWidth: width,
            fMiterLimit: p.getStrokeMiter(),
            fResScale: 1.0,
            fCap: p.getStrokeCap() as u8,
            fJoin: p.getStrokeJoin() as u8,
            fDoFill: p.getStyle() == crate::src::core::SkPaint::Style::kStrokeAndFill_Style,
        }
    }
    // Source: src/core/SkStroke.cpp:1408-1411
    pub fn setWidth(&mut self, width: f32) {
        debug_assert!(width >= 0.0);
        self.fWidth = width;
    }
    // Source: src/core/SkStroke.cpp:1413-1416
    pub fn setMiterLimit(&mut self, miterLimit: f32) {
        debug_assert!(miterLimit >= 0.0);
        self.fMiterLimit = miterLimit;
    }
    // Source: src/core/SkStroke.cpp:1418-1421
    pub fn setCap(&mut self, cap: Cap) {
        debug_assert!((cap as u32) < crate::src::core::SkPaint::kCapCount as u32);
        self.fCap = cap as u8;
    }
    // Source: src/core/SkStroke.cpp:1423-1426
    pub fn setJoin(&mut self, join: Join) {
        debug_assert!((join as u32) < crate::src::core::SkPaint::kJoinCount as u32);
        self.fJoin = join as u8;
    }
}

// Source: src/core/SkStroke.cpp:1430-1550
impl SkStroke {
    pub fn strokePath(&self, src: &SkPath, dst: &mut SkPathBuilder) {
        use crate::src::core::SkPathEnums::{SkPathFirstDirection, SkResolveConvexity};
        use crate::src::core::SkPathPriv::SkPathPriv;
        let radius = self.fWidth / 2.0_f32;
        if radius <= 0.0 {
            return;
        }
        let Some(raw) = SkPathPriv::Raw(src, SkResolveConvexity::kNo) else {
            return;
        };
        {
            let mut rect = SkRect::MakeEmpty();
            let mut isClosed = false;
            let mut dir = SkPathDirection::kCW;
            if src.isRect(Some(&mut rect), Some(&mut isClosed), Some(&mut dir)) && isClosed {
                self.strokeRect(&rect, dst, dir);
                if src.isInverseFillType() {
                    crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),1453; @condition b"!dst->isInverseFillType()\0".as_ptr(); !dst.isInverseFillType());
                    dst.toggleInverseFillType();
                }
                return;
            }
        }
        let ignoreCenter = self.fDoFill
            && src.getSegmentMasks() == crate::include::core::SkPathTypes::kLine_SkPathSegmentMask
            && src.isLastContourClosed()
            && src.isConvex();
        let mut stroker = SkPathStroker::new(
            src,
            radius,
            self.fMiterLimit,
            self.getCap(),
            self.getJoin(),
            self.fResScale,
            ignoreCenter,
        );
        let mut iter = Iter::new(src, false);
        let mut lastSegment = SkPathVerb::kMove;
        while let Some(rec) = iter.next() {
            let mut points = [SkPoint::make(0.0, 0.0); 4];
            points[..rec.fPoints.len()].copy_from_slice(rec.fPoints);
            let pts = &points;
            let verb = rec.fVerb;
            let weight = if verb == SkPathVerb::kConic {
                rec.conicWeight()
            } else {
                rec.fConicWeight
            };
            match verb {
                SkPathVerb::kMove => stroker.moveTo(pts[0]),
                SkPathVerb::kLine => {
                    stroker.lineTo(pts[1], Some(&iter));
                    lastSegment = SkPathVerb::kLine;
                }
                SkPathVerb::kQuad => {
                    stroker.quadTo(pts[1], pts[2]);
                    lastSegment = SkPathVerb::kQuad;
                }
                SkPathVerb::kConic => {
                    stroker.conicTo(pts[1], pts[2], weight);
                    lastSegment = SkPathVerb::kConic;
                }
                SkPathVerb::kCubic => {
                    stroker.cubicTo(pts[1], pts[2], pts[3]);
                    lastSegment = SkPathVerb::kCubic;
                }
                SkPathVerb::kClose => {
                    if self.getCap() != Cap::kButt_Cap {
                        if stroker.hasOnlyMoveTo() {
                            stroker.lineTo(stroker.moveToPt(), None);
                            lastSegment = SkPathVerb::kLine;
                            continue;
                        }
                        if stroker.isCurrentContourEmpty() {
                            lastSegment = SkPathVerb::kLine;
                            continue;
                        }
                    }
                    stroker.close(lastSegment == SkPathVerb::kLine);
                }
            }
        }
        stroker.done(dst, lastSegment == SkPathVerb::kLine);
        if self.fDoFill && !ignoreCenter {
            let d = SkPathPriv::ComputeFirstDirection(&raw);
            if d == SkPathFirstDirection::kCCW {
                dst.privateReverseAddPath(src);
            } else {
                dst.addPath(
                    src,
                    &crate::src::core::SkMatrix::SkMatrix::default(),
                    crate::src::core::SkPath::AddPathMode::kAppend_AddPathMode,
                );
            }
        }
        if src.isInverseFillType() {
            crate::SkASSERT!(@source b"src/core/SkStroke.cpp\0".as_ptr(),1453; @condition b"!dst->isInverseFillType()\0".as_ptr(); !dst.isInverseFillType());
            dst.toggleInverseFillType();
        }
    }
}
use crate::include::core::SkPathTypes::SkPathDirection;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::SkRect;
// Source: src/core/SkStroke.cpp:1552-1555
fn reverse_direction(dir: SkPathDirection) -> SkPathDirection {
    const gOpposite: [SkPathDirection; 2] = [SkPathDirection::kCCW, SkPathDirection::kCW];
    gOpposite[dir as usize]
}
// Source: src/core/SkStroke.cpp:1557-1581
fn addBevel(path: &mut SkPathBuilder, r: &SkRect, outer: &SkRect, dir: SkPathDirection) {
    // Initialization replaces the source uninitialized local array; every element is overwritten.
    let mut pts = [SkPoint::make(0.0, 0.0); 8];
    if dir == SkPathDirection::kCW {
        pts[0].set(r.fLeft, outer.fTop);
        pts[1].set(r.fRight, outer.fTop);
        pts[2].set(outer.fRight, r.fTop);
        pts[3].set(outer.fRight, r.fBottom);
        pts[4].set(r.fRight, outer.fBottom);
        pts[5].set(r.fLeft, outer.fBottom);
        pts[6].set(outer.fLeft, r.fBottom);
        pts[7].set(outer.fLeft, r.fTop);
    } else {
        pts[7].set(r.fLeft, outer.fTop);
        pts[6].set(r.fRight, outer.fTop);
        pts[5].set(outer.fRight, r.fTop);
        pts[4].set(outer.fRight, r.fBottom);
        pts[3].set(r.fRight, outer.fBottom);
        pts[2].set(r.fLeft, outer.fBottom);
        pts[1].set(outer.fLeft, r.fBottom);
        pts[0].set(outer.fLeft, r.fTop);
    }
    path.addPolygon(&pts, true);
}
impl SkStroke {
    // Source: src/core/SkStroke.cpp:1583-1631
    pub fn strokeRect(&self, origRect: &SkRect, dst: &mut SkPathBuilder, mut dir: SkPathDirection) {
        dst.reset();
        let radius = self.fWidth / 2.0_f32;
        if radius <= 0.0 {
            return;
        }
        let mut rw = origRect.width();
        let mut rh = origRect.height();
        if (rw < 0.0) ^ (rh < 0.0) {
            dir = reverse_direction(dir);
        }
        let mut rect = *origRect;
        rect.sort();
        rw = rect.width();
        rh = rect.height();
        let mut r = rect;
        r.outset(radius, radius);
        let mut join = self.getJoin();
        if join == Join::kMiter_Join && self.fMiterLimit < std::f32::consts::SQRT_2 {
            join = Join::kBevel_Join;
        }
        match join {
            Join::kMiter_Join => {
                dst.addRect(&r, dir, 0);
            }
            Join::kBevel_Join => {
                addBevel(dst, &rect, &r, dir);
            }
            Join::kRound_Join => {
                // SkPathBuilder.h:781-784 defines these direction-dependent legacy start indices.
                dst.addRRect(
                    &SkRRect::MakeRectXY(&r, radius, radius),
                    dir,
                    if dir == SkPathDirection::kCW { 6 } else { 7 },
                );
            }
        }
        let min_side = if rh < rw { rh } else { rw };
        if self.fWidth < min_side && !self.fDoFill {
            r = rect;
            r.inset(radius, radius);
            dst.addRect(&r, reverse_direction(dir), 0);
        }
    }
}
