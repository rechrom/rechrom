use crate::include::core::SkPathTypes::SkPathVerb;
use crate::include::core::SkRefCnt::{sk_make_sp, sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::SkGeometry::SkConic;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::{RangeIter, SkPath};
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPathPriv::SkPathPriv;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkTDArray::SkTDArray;
// Source: include/core/SkContourMeasure.h:26-26
unsafe impl SkRefCntObject for SkContourMeasure {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fRefCnt.ref_();
    }
}
impl SkContourMeasure {
    // Source: include/core/SkContourMeasure.h:30-30
    pub fn length(&self) -> f32 {
        self.fLength
    }
}
// Source: include/core/SkContourMeasure.h:37-41
pub type MatrixFlags = u32;
pub const kGetPosition_MatrixFlag: MatrixFlags = 0x01;
pub const kGetTangent_MatrixFlag: MatrixFlags = 0x02;
pub const kGetPosAndTan_MatrixFlag: MatrixFlags = kGetPosition_MatrixFlag | kGetTangent_MatrixFlag;
impl SkContourMeasure {
    // Source: include/core/SkContourMeasure.h:62-62
    pub fn isClosed(&self) -> bool {
        self.fIsClosed
    }
}
// Source: include/core/SkContourMeasure.h:66-70
pub struct VerbMeasure<'a> {
    pub fDistance: f32,
    pub fVerb: SkPathVerb,
    pub fPts: &'a [SkPoint],
}
impl<'a> ForwardVerbIterator<'a> {
    // Source: include/core/SkContourMeasure.h:86-92
    pub fn advance(&mut self) -> &mut Self {
        crate::SkASSERT!(@source b"include/core/SkContourMeasure.h\0".as_ptr(),87; @condition b"!fSegments.empty()\0".as_ptr(); !self.fSegments.is_empty());
        self.fSegments = Self::LastSegForCurrentVerb(&self.fSegments[1..]);
        self
    }
}
// Source: include/core/SkContourMeasure.h:94-102
impl PartialEq for ForwardVerbIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert!(
            self.fSegments.as_ptr() != other.fSegments.as_ptr()
                || self.fSegments.len() == other.fSegments.len()
        );
        self.fSegments.as_ptr() == other.fSegments.as_ptr()
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl<'a> ForwardVerbIterator<'a> {
    // Source: include/core/SkContourMeasure.h:107-109
    pub(crate) fn new(segs: &'a [Segment], pts: &'a [SkPoint]) -> Self {
        Self {
            fSegments: Self::LastSegForCurrentVerb(segs),
            fPts: pts,
        }
    }
    // Source: include/core/SkContourMeasure.h:111-118
    fn LastSegForCurrentVerb(segs: &'a [Segment]) -> &'a [Segment] {
        let mut i = 1;
        while i < segs.len() && segs[0].fPtIndex == segs[i].fPtIndex {
            i += 1;
        }
        &segs[i - 1..]
    }
}
// Source: include/core/SkContourMeasure.h:123-126
pub struct ForwardVerbIterator<'a> {
    fSegments: &'a [Segment],
    fPts: &'a [SkPoint],
}
// Source: include/core/SkContourMeasure.h:129-134
impl SkContourMeasure {
    pub fn begin(&self) -> ForwardVerbIterator<'_> {
        ForwardVerbIterator::new(self.fSegments.as_slice(), self.fPts.as_slice())
    }
    pub fn end(&self) -> ForwardVerbIterator<'_> {
        ForwardVerbIterator::new(
            &self.fSegments.as_slice()[self.fSegments.size() as usize..],
            self.fPts.as_slice(),
        )
    }
}
// Source: include/core/SkContourMeasure.h:137-141
// The native 30/2 bitfields share one u32. Accessors preserve their truncation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Segment {
    pub fDistance: f32,
    pub fPtIndex: u32,
    packed: u32,
}
impl Segment {
    pub(crate) fn t_value(&self) -> u32 {
        self.packed & 0x3fffffff
    }
    pub(crate) fn seg_type(&self) -> u32 {
        self.packed >> 30
    }
}
// Source: include/core/SkContourMeasure.h:146-152
impl Segment {
    pub(crate) fn Next(segs: &[Self]) -> &[Self] {
        let ptIndex = segs[0].fPtIndex;
        let mut index = 1;
        while segs[index].fPtIndex == ptIndex {
            index += 1;
        }
        &segs[index..]
    }
}
// Source: include/core/SkContourMeasure.h:156-160
// Field drop order is the C++ reverse declaration order: point storage, segment storage, base.
pub struct SkContourMeasure {
    fPts: SkTDArray<SkPoint>,
    pub(crate) fSegments: SkTDArray<Segment>,
    fLength: f32,
    fIsClosed: bool,
    fRefCnt: SkRefCnt,
}
// Source: include/core/SkContourMeasure.h:212-212
pub struct SkContourMeasureIter {
    fImpl: Option<Box<MeasureIterImpl>>,
}
// Source: src/core/SkContourMeasure.cpp:27-34
pub(crate) const kMaxTValue: i32 = 0x3fffffff;
pub(crate) fn tValue2Scalar(t: i32) -> f32 {
    debug_assert!((t as u32) <= kMaxTValue as u32);
    let kMaxTReciprocal = 1.0_f32 / kMaxTValue as f32;
    t as f32 * kMaxTReciprocal
}
// Source: src/core/SkContourMeasure.cpp:36-37
const _: () = assert!(0.0 == 0.0_f32 * (1.0_f32 / kMaxTValue as f32));
const _: () = assert!(1.0 == kMaxTValue as f32 * (1.0_f32 / kMaxTValue as f32));
// Source: src/core/SkContourMeasure.cpp:39-41
impl Segment {
    pub(crate) fn getScalarT(&self) -> f32 {
        tValue2Scalar(self.t_value() as i32)
    }
}
// Source: src/core/SkContourMeasure.cpp:43-135
pub(crate) fn SkContourMeasure_segTo(
    pts: &[SkPoint],
    segType: u32,
    startT: f32,
    stopT: f32,
    dst: &mut SkPathBuilder,
) {
    use crate::include::core::SkScalar::SkScalarInterp;
    use crate::src::core::SkGeometry::{SkChopCubicAt, SkChopQuadAt};
    use crate::src::core::SkPathMeasurePriv::SkSegType;
    debug_assert!(startT >= 0.0 && startT <= 1.0);
    debug_assert!(stopT >= 0.0 && stopT <= 1.0);
    crate::SkASSERT!(@source b"src/core/SkContourMeasure.cpp\0".as_ptr(),47; @condition b"startT <= stopT\0".as_ptr(); startT<=stopT);
    if startT == stopT {
        if !dst.isEmpty() {
            let last = dst.getLastPt().expect("nonempty builder has last point");
            dst.lineTo(last);
        }
        return;
    }
    let mut tmp0 = [SkPoint::make(0.0, 0.0); 7];
    let mut tmp1 = tmp0;
    match segType {
        v if v == SkSegType::kLine_SegType as u32 => {
            if stopT == 1.0 {
                dst.lineTo(pts[1]);
            } else {
                dst.lineTo(SkPoint::make(
                    SkScalarInterp(pts[0].fX, pts[1].fX, stopT),
                    SkScalarInterp(pts[0].fY, pts[1].fY, stopT),
                ));
            }
        }
        v if v == SkSegType::kQuad_SegType as u32 => {
            if startT == 0.0 {
                if stopT == 1.0 {
                    dst.quadTo(pts[1], pts[2]);
                } else {
                    SkChopQuadAt(
                        pts[..3].try_into().unwrap(),
                        (&mut tmp0[..5]).try_into().unwrap(),
                        stopT,
                    );
                    dst.quadTo(tmp0[1], tmp0[2]);
                }
            } else {
                SkChopQuadAt(
                    pts[..3].try_into().unwrap(),
                    (&mut tmp0[..5]).try_into().unwrap(),
                    startT,
                );
                if stopT == 1.0 {
                    dst.quadTo(tmp0[3], tmp0[4]);
                } else {
                    SkChopQuadAt(
                        tmp0[2..5].try_into().unwrap(),
                        (&mut tmp1[..5]).try_into().unwrap(),
                        (stopT - startT) / (1.0 - startT),
                    );
                    dst.quadTo(tmp1[1], tmp1[2]);
                }
            }
        }
        v if v == SkSegType::kConic_SegType as u32 => {
            let conic = SkConic::new(&[pts[0], pts[2], pts[3]], pts[1].fX);
            if startT == 0.0 {
                if stopT == 1.0 {
                    dst.conicTo(conic.fPts[1], conic.fPts[2], conic.fW);
                } else {
                    let mut tmp = [SkConic::default(); 2];
                    if conic.chopAt(stopT, &mut tmp) {
                        dst.conicTo(tmp[0].fPts[1], tmp[0].fPts[2], tmp[0].fW);
                    }
                }
            } else if stopT == 1.0 {
                let mut tmp = [SkConic::default(); 2];
                if conic.chopAt(startT, &mut tmp) {
                    dst.conicTo(tmp[1].fPts[1], tmp[1].fPts[2], tmp[1].fW);
                }
            } else {
                let mut tmp = SkConic::default();
                conic.chopAtRange(startT, stopT, &mut tmp);
                dst.conicTo(tmp.fPts[1], tmp.fPts[2], tmp.fW);
            }
        }
        v if v == SkSegType::kCubic_SegType as u32 => {
            if startT == 0.0 {
                if stopT == 1.0 {
                    dst.cubicTo(pts[1], pts[2], pts[3]);
                } else {
                    SkChopCubicAt(pts[..4].try_into().unwrap(), &mut tmp0, stopT);
                    dst.cubicTo(tmp0[1], tmp0[2], tmp0[3]);
                }
            } else {
                SkChopCubicAt(pts[..4].try_into().unwrap(), &mut tmp0, startT);
                if stopT == 1.0 {
                    dst.cubicTo(tmp0[4], tmp0[5], tmp0[6]);
                } else {
                    SkChopCubicAt(
                        tmp0[3..].try_into().unwrap(),
                        &mut tmp1,
                        (stopT - startT) / (1.0 - startT),
                    );
                    dst.cubicTo(tmp1[1], tmp1[2], tmp1[3]);
                }
            }
        }
        _ => std::process::abort(),
    }
}
// Source: src/core/SkContourMeasure.cpp:139-142
pub(crate) fn tspan_big_enough(tspan: i32) -> i32 {
    debug_assert!((tspan as u32) <= kMaxTValue as u32);
    tspan >> 10
}
// Source: src/core/SkContourMeasure.cpp:147-147
pub(crate) const CHEAP_DIST_LIMIT: f32 = 1.0 / 2.0;
// Source: src/core/SkContourMeasure.cpp:149-159
pub(crate) fn quad_too_curvy(pts: &[SkPoint; 3], tolerance: f32) -> bool {
    let dx = pts[1].fX / 2.0 - ((pts[0].fX + pts[2].fX) / 2.0) / 2.0;
    let dy = pts[1].fY / 2.0 - ((pts[0].fY + pts[2].fY) / 2.0) / 2.0;
    let (a, b) = (dx.abs(), dy.abs());
    let dist = if a < b { b } else { a };
    dist > tolerance
}
// Source: src/core/SkContourMeasure.cpp:161-168
pub(crate) fn conic_too_curvy(
    firstPt: SkPoint,
    midTPt: SkPoint,
    lastPt: SkPoint,
    tolerance: f32,
) -> bool {
    let mut midEnds = firstPt + lastPt;
    midEnds *= 0.5;
    let dxy = midTPt - midEnds;
    let (a, b) = (dxy.fX.abs(), dxy.fY.abs());
    let dist = if a < b { b } else { a };
    dist > tolerance
}
// Source: src/core/SkContourMeasure.cpp:170-175
pub(crate) fn cheap_dist_exceeds_limit(pt: SkPoint, x: f32, y: f32, tolerance: f32) -> bool {
    let (a, b) = ((x - pt.fX).abs(), (y - pt.fY).abs());
    let dist = if a < b { b } else { a };
    dist > tolerance
}
// Source: src/core/SkContourMeasure.cpp:177-185
pub(crate) fn cubic_too_curvy(pts: &[SkPoint; 4], tolerance: f32) -> bool {
    use crate::include::core::SkScalar::SkScalarInterp;
    cheap_dist_exceeds_limit(
        pts[1],
        SkScalarInterp(pts[0].fX, pts[3].fX, 1.0 / 3.0),
        SkScalarInterp(pts[0].fY, pts[3].fY, 1.0 / 3.0),
        tolerance,
    ) || cheap_dist_exceeds_limit(
        pts[2],
        SkScalarInterp(pts[0].fX, pts[3].fX, 1.0 * 2.0 / 3.0),
        SkScalarInterp(pts[0].fY, pts[3].fY, 1.0 * 2.0 / 3.0),
        tolerance,
    )
}
// Source: src/core/SkContourMeasure.cpp:189-189
pub(crate) const kMaxRecursionDepth: i32 = 8;

// Source: src/core/SkContourMeasure.cpp:191-210
// fIter only borrows immutable tail arrays owned by fPath's retained PathData. It never points
// into the movable SkPath value. No borrowed record escapes Impl; fPath is never mutated.
// Private lifetime extension represents the native retained-owner/raw-range relationship.
// Rust fields drop in declaration order: scratch arrays, iterator, then its retained path.
struct MeasureIterImpl {
    fPts: SkTDArray<SkPoint>,
    fSegments: SkTDArray<Segment>,
    fIter: RangeIter<'static>,
    fPath: SkPath,
    fTolerance: f32,
    fForceClosed: bool,
}
impl MeasureIterImpl {
    fn new(path: &SkPath, forceClosed: bool, resScale: f32) -> Self {
        let owner = path.clone();
        let iter = SkPathPriv::Iterate(&owner).begin();
        let iter = unsafe { std::mem::transmute::<RangeIter<'_>, RangeIter<'static>>(iter) };
        Self {
            fPath: owner,
            fIter: iter,
            fTolerance: CHEAP_DIST_LIMIT * (1.0_f32 / resScale),
            fForceClosed: forceClosed,
            fSegments: SkTDArray::new(),
            fPts: SkTDArray::new(),
        }
    }
    fn hasNextSegments(&self) -> bool {
        self.fIter.clone().next().is_some()
    }
    // Source: src/core/SkContourMeasure.cpp:224-250
    fn compute_quad_segs(
        &mut self,
        pts: &[SkPoint; 3],
        mut distance: f32,
        mint: i32,
        maxt: i32,
        ptIndex: u32,
        mut recursionDepth: i32,
    ) -> f32 {
        if recursionDepth < kMaxRecursionDepth
            && tspan_big_enough(maxt - mint) != 0
            && quad_too_curvy(pts, self.fTolerance)
        {
            let mut tmp = [SkPoint::make(0.0, 0.0); 5];
            let halft = (mint + maxt) >> 1;
            crate::src::core::SkGeometry::SkChopQuadAtHalf(pts, &mut tmp);
            recursionDepth += 1;
            distance = self.compute_quad_segs(
                tmp[..3].try_into().unwrap(),
                distance,
                mint,
                halft,
                ptIndex,
                recursionDepth,
            );
            distance = self.compute_quad_segs(
                tmp[2..].try_into().unwrap(),
                distance,
                halft,
                maxt,
                ptIndex,
                recursionDepth,
            );
        } else {
            let d = SkPoint::distance(pts[0], pts[2]);
            let prevD = distance;
            distance += d;
            if distance > prevD {
                debug_assert!(ptIndex < self.fPts.size() as u32);
                self.fSegments.push_back(Segment {
                    fDistance: distance,
                    fPtIndex: ptIndex,
                    packed: (maxt as u32 & 0x3fffffff)
                        | ((crate::src::core::SkPathMeasurePriv::SkSegType::kQuad_SegType as u32)
                            << 30),
                });
            }
        }
        distance
    }
    // Source: src/core/SkContourMeasure.cpp:252-283
    fn compute_conic_segs(
        &mut self,
        conic: &SkConic,
        mut distance: f32,
        mint: i32,
        minPt: SkPoint,
        maxt: i32,
        maxPt: SkPoint,
        ptIndex: u32,
        mut recursionDepth: i32,
    ) -> f32 {
        let halft = (mint + maxt) >> 1;
        let halfPt = conic.evalAt(tValue2Scalar(halft));
        if !halfPt.is_finite() {
            return distance;
        }
        if recursionDepth < kMaxRecursionDepth
            && tspan_big_enough(maxt - mint) != 0
            && conic_too_curvy(minPt, halfPt, maxPt, self.fTolerance)
        {
            recursionDepth += 1;
            distance = self.compute_conic_segs(
                conic,
                distance,
                mint,
                minPt,
                halft,
                halfPt,
                ptIndex,
                recursionDepth,
            );
            distance = self.compute_conic_segs(
                conic,
                distance,
                halft,
                halfPt,
                maxt,
                maxPt,
                ptIndex,
                recursionDepth,
            );
        } else {
            let d = SkPoint::distance(minPt, maxPt);
            let prevD = distance;
            distance += d;
            if distance > prevD {
                debug_assert!(ptIndex < self.fPts.size() as u32);
                self.fSegments.push_back(Segment {
                    fDistance: distance,
                    fPtIndex: ptIndex,
                    packed: (maxt as u32 & 0x3fffffff)
                        | ((crate::src::core::SkPathMeasurePriv::SkSegType::kConic_SegType as u32)
                            << 30),
                });
            }
        }
        distance
    }
    // Source: src/core/SkContourMeasure.cpp:285-314
    fn compute_cubic_segs(
        &mut self,
        pts: &[SkPoint; 4],
        mut distance: f32,
        mint: i32,
        maxt: i32,
        ptIndex: u32,
        mut recursionDepth: i32,
    ) -> f32 {
        if recursionDepth < kMaxRecursionDepth
            && tspan_big_enough(maxt - mint) != 0
            && cubic_too_curvy(pts, self.fTolerance)
        {
            let mut tmp = [SkPoint::make(0.0, 0.0); 7];
            let halft = (mint + maxt) >> 1;
            crate::src::core::SkGeometry::SkChopCubicAtHalf(pts, &mut tmp);
            recursionDepth += 1;
            distance = self.compute_cubic_segs(
                tmp[..4].try_into().unwrap(),
                distance,
                mint,
                halft,
                ptIndex,
                recursionDepth,
            );
            distance = self.compute_cubic_segs(
                tmp[3..].try_into().unwrap(),
                distance,
                halft,
                maxt,
                ptIndex,
                recursionDepth,
            );
        } else {
            let d = SkPoint::distance(pts[0], pts[3]);
            let prevD = distance;
            distance += d;
            if distance > prevD {
                debug_assert!(ptIndex < self.fPts.size() as u32);
                self.fSegments.push_back(Segment {
                    fDistance: distance,
                    fPtIndex: ptIndex,
                    packed: (maxt as u32 & 0x3fffffff)
                        | ((crate::src::core::SkPathMeasurePriv::SkSegType::kCubic_SegType as u32)
                            << 30),
                });
            }
        }
        distance
    }
    // Source: src/core/SkContourMeasure.cpp:316-331
    fn compute_line_seg(
        &mut self,
        p0: SkPoint,
        p1: SkPoint,
        mut distance: f32,
        ptIndex: u32,
    ) -> f32 {
        let d = SkPoint::distance(p0, p1);
        debug_assert!(d >= 0.0);
        let prevD = distance;
        distance += d;
        if distance > prevD {
            debug_assert!(ptIndex < self.fPts.size() as u32);
            self.fSegments.push_back(Segment {
                fDistance: distance,
                fPtIndex: ptIndex,
                packed: kMaxTValue as u32,
            });
        }
        distance
    }
    // Source: src/core/SkContourMeasure.cpp:334-357
    #[cfg(sk_debug)]
    fn validate(&self) {
        let segs = self.fSegments.as_slice();
        let mut seg = 0;
        let stop = segs.len();
        let mut ptIndex = 0;
        let mut distance = 0.0;
        let mut maxChecks = 10000000_i32;
        while seg < stop {
            debug_assert!(segs[seg].fDistance > distance);
            debug_assert!(segs[seg].fPtIndex >= ptIndex);
            debug_assert!(segs[seg].t_value() > 0);
            let mut s = seg;
            while s < stop - 1 && segs[s].fPtIndex == segs[s + 1].fPtIndex {
                maxChecks -= 1;
                if maxChecks <= 0 {
                    break;
                }
                debug_assert_eq!(segs[s].seg_type(), segs[s + 1].seg_type());
                debug_assert!(segs[s].t_value() < segs[s + 1].t_value());
                s += 1;
            }
            distance = segs[seg].fDistance;
            ptIndex = segs[seg].fPtIndex;
            seg += 1;
        }
    }
    // Source: src/core/SkContourMeasure.cpp:360-463
    fn buildSegments(&mut self) -> sk_sp<SkContourMeasure> {
        let mut ptIndex = -1_i32;
        let mut distance = 0.0;
        let mut haveSeenClose = self.fForceClosed;
        let mut haveSeenMoveTo = false;
        self.fSegments.reset();
        self.fPts.reset();
        while self.hasNextSegments() {
            let rec = self.fIter.record();
            let verb = rec.fVerb;
            let pts = rec.fPoints;
            let w = rec.fConicWeight;
            if haveSeenMoveTo && verb == SkPathVerb::kMove {
                break;
            }
            match verb {
                SkPathVerb::kMove => {
                    ptIndex += 1;
                    self.fPts.append(&pts[..1]);
                    crate::SkASSERT!(@source b"src/core/SkContourMeasure.cpp\0".as_ptr(),387; @condition b"!haveSeenMoveTo\0".as_ptr(); !haveSeenMoveTo);
                    haveSeenMoveTo = true;
                }
                SkPathVerb::kLine => {
                    debug_assert!(haveSeenMoveTo);
                    let prevD = distance;
                    distance = self.compute_line_seg(pts[0], pts[1], distance, ptIndex as u32);
                    if distance > prevD {
                        self.fPts.append(&pts[1..2]);
                        ptIndex += 1;
                    }
                }
                SkPathVerb::kQuad => {
                    debug_assert!(haveSeenMoveTo);
                    let prevD = distance;
                    distance = self.compute_quad_segs(
                        pts[..3].try_into().unwrap(),
                        distance,
                        0,
                        kMaxTValue,
                        ptIndex as u32,
                        0,
                    );
                    if distance > prevD {
                        self.fPts.append(&pts[1..3]);
                        ptIndex += 2;
                    }
                }
                SkPathVerb::kConic => {
                    debug_assert!(haveSeenMoveTo);
                    let conic = SkConic::new(pts[..3].try_into().unwrap(), w);
                    let prevD = distance;
                    distance = self.compute_conic_segs(
                        &conic,
                        distance,
                        0,
                        conic.fPts[0],
                        kMaxTValue,
                        conic.fPts[2],
                        ptIndex as u32,
                        0,
                    );
                    if distance > prevD {
                        self.fPts.push_back(SkPoint::make(conic.fW, 0.0));
                        self.fPts.append(&pts[1..3]);
                        ptIndex += 3;
                    }
                }
                SkPathVerb::kCubic => {
                    debug_assert!(haveSeenMoveTo);
                    let prevD = distance;
                    distance = self.compute_cubic_segs(
                        pts[..4].try_into().unwrap(),
                        distance,
                        0,
                        kMaxTValue,
                        ptIndex as u32,
                        0,
                    );
                    if distance > prevD {
                        self.fPts.append(&pts[1..4]);
                        ptIndex += 3;
                    }
                }
                SkPathVerb::kClose => haveSeenClose = true,
            }
            self.fIter.advance();
        }
        if !distance.is_finite() {
            return sk_sp::default();
        }
        if self.fSegments.empty() {
            return sk_sp::default();
        }
        if haveSeenClose {
            let prevD = distance;
            let firstPt = self.fPts[0];
            distance = self.compute_line_seg(
                self.fPts[ptIndex as usize],
                firstPt,
                distance,
                ptIndex as u32,
            );
            if distance > prevD {
                self.fPts.push_back(firstPt);
            }
        }
        #[cfg(sk_debug)]
        self.validate();
        sk_make_sp(SkContourMeasure::new(
            std::mem::take(&mut self.fSegments),
            std::mem::take(&mut self.fPts),
            distance,
            haveSeenClose,
        ))
    }
}
// Source: src/core/SkContourMeasure.cpp:465-498
pub(crate) fn compute_pos_tan(
    pts: &[SkPoint],
    segType: u32,
    t: f32,
    pos: Option<&mut SkPoint>,
    mut tangent: Option<&mut SkPoint>,
) {
    use crate::include::core::SkScalar::SkScalarInterp;
    use crate::src::core::SkGeometry::{SkConic, SkEvalCubicAt, SkEvalQuadAtWithTangent};
    use crate::src::core::SkPathMeasurePriv::SkSegType;
    match segType {
        v if v == SkSegType::kLine_SegType as u32 => {
            if let Some(pos) = pos {
                pos.set(
                    SkScalarInterp(pts[0].fX, pts[1].fX, t),
                    SkScalarInterp(pts[0].fY, pts[1].fY, t),
                );
            }
            if let Some(tangent) = tangent {
                tangent.set_normalize(pts[1].fX - pts[0].fX, pts[1].fY - pts[0].fY);
            }
        }
        v if v == SkSegType::kQuad_SegType as u32 => {
            SkEvalQuadAtWithTangent(pts[..3].try_into().unwrap(), t, pos, tangent.as_deref_mut());
            if let Some(tangent) = tangent {
                tangent.normalize();
            }
        }
        v if v == SkSegType::kConic_SegType as u32 => {
            SkConic::new(&[pts[0], pts[2], pts[3]], pts[1].fX).evalAtWithTangent(
                t,
                pos,
                tangent.as_deref_mut(),
            );
            if let Some(tangent) = tangent {
                tangent.normalize();
            }
        }
        v if v == SkSegType::kCubic_SegType as u32 => {
            SkEvalCubicAt(
                pts[..4].try_into().unwrap(),
                t,
                pos,
                tangent.as_deref_mut(),
                None,
            );
            if let Some(tangent) = tangent {
                tangent.normalize();
            }
        }
        _ => debug_assert!(false, "unknown segType"),
    }
}
// Source: src/core/SkContourMeasure.cpp:504-505
impl Default for SkContourMeasureIter {
    fn default() -> Self {
        Self { fImpl: None }
    }
}
impl SkContourMeasureIter {
    // Source: src/core/SkContourMeasure.cpp:507-510
    pub fn new(path: &SkPath, forceClosed: bool, resScale: f32) -> Self {
        let mut out = Self::default();
        out.reset(path, forceClosed, resScale);
        out
    }
    // Source: src/core/SkContourMeasure.cpp:519-525
    pub fn reset(&mut self, path: &SkPath, forceClosed: bool, resScale: f32) {
        if path.isFinite() {
            self.fImpl = Some(Box::new(MeasureIterImpl::new(path, forceClosed, resScale)));
        } else {
            self.fImpl = None;
        }
    }
    // Source: src/core/SkContourMeasure.cpp:527-538
    pub fn next(&mut self) -> sk_sp<SkContourMeasure> {
        let Some(implementation) = self.fImpl.as_mut() else {
            return sk_sp::default();
        };
        while implementation.hasNextSegments() {
            let cm = implementation.buildSegments();
            if cm.is_some() {
                return cm;
            }
        }
        sk_sp::default()
    }
}
// Source: src/core/SkContourMeasure.cpp:542-547
impl SkContourMeasure {
    fn new(segs: SkTDArray<Segment>, pts: SkTDArray<SkPoint>, length: f32, isClosed: bool) -> Self {
        Self {
            fSegments: segs,
            fPts: pts,
            fLength: length,
            fIsClosed: isClosed,
            fRefCnt: SkRefCnt::new(),
        }
    }
}
// Source: src/core/SkContourMeasure.cpp:549-577
// The only source instantiation uses Segment/f32; a borrowed slice carries base/count validity.
pub(crate) fn SkTKSearch(base: &[Segment], count: i32, key: f32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkContourMeasure.cpp\0".as_ptr(),551; @condition b"count >= 0\0".as_ptr(); count >= 0);
    if count <= 0 {
        return !0;
    }
    let mut lo = 0_u32;
    let mut hi = (count - 1) as u32;
    while lo < hi {
        let mid = (hi + lo) >> 1;
        if base[mid as usize].fDistance < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if base[hi as usize].fDistance < key {
        hi += 1;
        hi = !hi;
    } else if key < base[hi as usize].fDistance {
        hi = !hi;
    }
    hi as i32
}

impl SkContourMeasure {
    // Source: src/core/SkContourMeasure.cpp:579-609
    fn distanceToSegment(&self, distance: f32, t: &mut f32) -> usize {
        debug_assert!(distance >= 0.0 && distance <= self.length());
        let seg = self.fSegments.as_slice();
        let count = self.fSegments.size();
        let mut index = SkTKSearch(seg, count, distance);
        index ^= index >> 31;
        let index = index as usize;
        let mut startT = 0.0;
        let mut startD = 0.0;
        if index > 0 {
            startD = seg[index - 1].fDistance;
            if seg[index - 1].fPtIndex == seg[index].fPtIndex {
                debug_assert_eq!(seg[index - 1].seg_type(), seg[index].seg_type());
                startT = seg[index - 1].getScalarT();
            }
        }
        debug_assert!(seg[index].getScalarT() > startT);
        crate::SkASSERT!(@source b"src/core/SkContourMeasure.cpp\0".as_ptr(),604; @condition b"distance >= startD\0".as_ptr(); distance>=startD);
        debug_assert!(seg[index].fDistance > startD);
        *t = startT
            + (seg[index].getScalarT() - startT) * (distance - startD)
                / (seg[index].fDistance - startD);
        index
    }
    // Source: src/core/SkContourMeasure.cpp:611-635
    pub fn getPosTan(
        &self,
        mut distance: f32,
        pos: Option<&mut SkPoint>,
        tangent: Option<&mut SkPoint>,
    ) -> bool {
        if distance.is_nan() {
            return false;
        }
        let length = self.length();
        debug_assert!(length > 0.0 && !self.fSegments.empty());
        if distance < 0.0 {
            distance = 0.0;
        } else if distance > length {
            distance = length;
        }
        let mut t = 0.0;
        let index = self.distanceToSegment(distance, &mut t);
        let seg = &self.fSegments[index];
        if t.is_nan() {
            return false;
        }
        debug_assert!(seg.fPtIndex < (self.fPts.size() as u32));
        compute_pos_tan(
            &self.fPts.as_slice()[seg.fPtIndex as usize..],
            seg.seg_type(),
            t,
            pos,
            tangent,
        );
        true
    }
    // Source: src/core/SkContourMeasure.cpp:637-655
    pub fn getMatrix(
        &self,
        distance: f32,
        matrix: Option<&mut SkMatrix>,
        flags: MatrixFlags,
    ) -> bool {
        let mut position = SkPoint::make(0.0, 0.0);
        let mut tangent = position;
        if self.getPosTan(distance, Some(&mut position), Some(&mut tangent)) {
            if let Some(matrix) = matrix {
                if flags & kGetTangent_MatrixFlag != 0 {
                    matrix.setSinCosPivot(tangent.fY, tangent.fX, 0.0, 0.0);
                } else {
                    matrix.reset();
                }
                if flags & kGetPosition_MatrixFlag != 0 {
                    matrix.postTranslate(position.fX, position.fY);
                }
            }
            return true;
        }
        false
    }
}
// Source: src/core/SkContourMeasure.cpp:657-704
impl SkContourMeasure {
    pub fn getSegment(
        &self,
        mut startD: f32,
        mut stopD: f32,
        dst: &mut SkPathBuilder,
        startWithMoveTo: bool,
    ) -> bool {
        let length = self.length();
        if startD < 0.0 {
            startD = 0.0;
        }
        if stopD > length {
            stopD = length;
        }
        if !(startD <= stopD) {
            return false;
        }
        if self.fSegments.empty() {
            return false;
        }
        let mut p = SkPoint::make(0.0, 0.0);
        let mut startT = 0.0;
        let mut stopT = 0.0;
        let mut index = self.distanceToSegment(startD, &mut startT);
        if !startT.is_finite() {
            return false;
        }
        let stopIndex = self.distanceToSegment(stopD, &mut stopT);
        if !stopT.is_finite() {
            return false;
        }
        debug_assert!(index <= stopIndex);
        if startWithMoveTo {
            let seg = &self.fSegments[index];
            compute_pos_tan(
                &self.fPts.as_slice()[seg.fPtIndex as usize..],
                seg.seg_type(),
                startT,
                Some(&mut p),
                None,
            );
            dst.moveTo(p);
        }
        if self.fSegments[index].fPtIndex == self.fSegments[stopIndex].fPtIndex {
            let seg = &self.fSegments[index];
            SkContourMeasure_segTo(
                &self.fPts.as_slice()[seg.fPtIndex as usize..],
                seg.seg_type(),
                startT,
                stopT,
                dst,
            );
        } else {
            loop {
                let seg = &self.fSegments[index];
                SkContourMeasure_segTo(
                    &self.fPts.as_slice()[seg.fPtIndex as usize..],
                    seg.seg_type(),
                    startT,
                    1.0,
                    dst,
                );
                index = self.fSegments.size() as usize
                    - Segment::Next(&self.fSegments.as_slice()[index..]).len();
                startT = 0.0;
                if self.fSegments[index].fPtIndex >= self.fSegments[stopIndex].fPtIndex {
                    break;
                }
            }
            let seg = &self.fSegments[index];
            SkContourMeasure_segTo(
                &self.fPts.as_slice()[seg.fPtIndex as usize..],
                seg.seg_type(),
                0.0,
                stopT,
                dst,
            );
        }
        true
    }
}
// Source: src/core/SkContourMeasure.cpp:706-733
impl<'a> ForwardVerbIterator<'a> {
    pub fn value(&self) -> VerbMeasure<'a> {
        use crate::src::core::SkPathMeasurePriv::SkSegType;
        const seg_pt_count: [usize; 4] = [2, 3, 4, 4];
        const seg_verb: [SkPathVerb; 4] = [
            SkPathVerb::kLine,
            SkPathVerb::kQuad,
            SkPathVerb::kCubic,
            SkPathVerb::kConic,
        ];
        const _: () = assert!(seg_pt_count.len() == seg_verb.len());
        const _: () = assert!((SkSegType::kLine_SegType as usize) < seg_pt_count.len());
        const _: () = assert!((SkSegType::kQuad_SegType as usize) < seg_pt_count.len());
        const _: () = assert!((SkSegType::kCubic_SegType as usize) < seg_pt_count.len());
        const _: () = assert!((SkSegType::kConic_SegType as usize) < seg_pt_count.len());
        let segment = &self.fSegments[0];
        let typ = segment.seg_type() as usize;
        debug_assert!(typ < seg_pt_count.len());
        debug_assert!(segment.fPtIndex as usize + seg_pt_count[typ] <= self.fPts.len());
        VerbMeasure {
            fDistance: segment.fDistance,
            fVerb: seg_verb[typ],
            fPts: &self.fPts
                [segment.fPtIndex as usize..segment.fPtIndex as usize + seg_pt_count[typ]],
        }
    }
}
