use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SkScalarCeilToInt, SkScalarMod};
use crate::src::core::SkPaint::{Cap, Join};
use crate::src::core::SkPath::{Iter, SkPath};
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPathMeasure::SkPathMeasure;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkPointPriv::SkPointPriv;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkStrokeRec::{SkStrokeRec, Style};
use crate::src::utils::SkDashPathPriv::{kMaxDashCount, StrokeRecApplication};
// Source: src/utils/SkDashPath.cpp:31-33
fn is_even(x: i32) -> bool {
    x & 1 == 0
}
// Source: src/utils/SkDashPath.cpp:35-52
fn find_first_interval(intervals: &[f32], mut phase: f32, index: &mut usize) -> f32 {
    for (i, &gap) in intervals.iter().enumerate() {
        if phase > gap || (phase == gap && gap != 0.0) {
            phase -= gap;
        } else {
            *index = i;
            return gap - phase;
        }
    }
    *index = 0;
    intervals[0]
}
// Source: src/utils/SkDashPath.cpp:54-90
pub fn CalcDashParameters(
    mut phase: f32,
    intervals: &[f32],
    initialDashLength: &mut f32,
    initialDashIndex: &mut usize,
    intervalLength: &mut f32,
    adjustedPhase: Option<&mut f32>,
) {
    let mut len = 0.0;
    for &interval in intervals {
        len += interval;
    }
    *intervalLength = len;
    if let Some(adjustedPhase) = adjustedPhase {
        if phase < 0.0 {
            phase = -phase;
            if phase > len {
                phase = SkScalarMod(phase, len);
            }
            phase = len - phase;
            crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),75; @condition b"phase <= len\0".as_ptr(); phase<=len);
            if phase == len {
                phase = 0.0;
            }
        } else if phase >= len {
            phase = SkScalarMod(phase, len);
        }
        *adjustedPhase = phase;
    }
    debug_assert!(phase >= 0.0 && phase < len);
    *initialDashLength = find_first_interval(intervals, phase, initialDashIndex);
    debug_assert!(*initialDashLength >= 0.0);
    crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),89; @condition b"*initialDashIndex < intervals.size()\0".as_ptr(); *initialDashIndex<intervals.len());
}
// Source: src/utils/SkDashPath.cpp:92-101
fn outset_for_stroke(rect: &mut SkRect, rec: &SkStrokeRec) {
    let mut radius = rec.getWidth() / 2.0;
    if radius == 0.0 {
        radius = 1.0;
    }
    if rec.getJoin() == Join::kMiter_Join {
        radius *= rec.getMiter();
    }
    rect.outset(radius, radius);
}
// Source: src/utils/SkDashPath.cpp:108-111
fn adjust_zero_length_line(pts: &mut [SkPoint; 2]) {
    crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),109; @condition b"pts[0] == pts[1]\0".as_ptr(); (pts[0])==(pts[1]));
    let multiplier = if 1.001_f32 < pts[1].fX {
        pts[1].fX
    } else {
        1.001_f32
    };
    pts[1].fX += multiplier * SK_ScalarNearlyZero;
}
// Source: src/utils/SkDashPath.cpp:113-167
fn clip_line(
    pts: &mut [SkPoint; 2],
    bounds: &SkRect,
    intervalLength: f32,
    priorPhase: f32,
) -> bool {
    let dxy = pts[1] - pts[0];
    if dxy.fX != 0.0 && dxy.fY != 0.0 {
        return false;
    }
    let vertical = dxy.fY != 0.0;
    let mut minXY = if vertical { pts[0].fY } else { pts[0].fX };
    let mut maxXY = if vertical { pts[1].fY } else { pts[1].fX };
    let swapped = maxXY < minXY;
    if swapped {
        std::mem::swap(&mut minXY, &mut maxXY);
    }
    crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),131; @condition b"minXY <= maxXY\0".as_ptr(); minXY<=maxXY);
    let leftTop = if vertical { bounds.fTop } else { bounds.fLeft };
    let rightBottom = if vertical {
        bounds.fBottom
    } else {
        bounds.fRight
    };
    if maxXY < leftTop || minXY > rightBottom {
        return false;
    }
    if minXY < leftTop {
        minXY = leftTop - SkScalarMod(leftTop - minXY, intervalLength);
        if !swapped {
            minXY -= priorPhase;
        }
    }
    if maxXY > rightBottom {
        maxXY = rightBottom + SkScalarMod(maxXY - rightBottom, intervalLength);
        if swapped {
            maxXY += priorPhase;
        }
    }
    crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),155; @condition b"maxXY >= minXY\0".as_ptr(); maxXY>=minXY);
    if swapped {
        std::mem::swap(&mut minXY, &mut maxXY);
    }
    if vertical {
        pts[0].fY = minXY;
        pts[1].fY = maxXY;
    } else {
        pts[0].fX = minXY;
        pts[1].fX = maxXY;
    }
    if minXY == maxXY {
        adjust_zero_length_line(pts);
    }
    true
}
// Source: src/utils/SkDashPath.cpp:172-233
fn cull_path(
    srcPath: &SkPath,
    rec: &SkStrokeRec,
    cullRect: Option<&SkRect>,
    intervalLength: f32,
    builder: &mut SkPathBuilder,
) -> bool {
    let z = SkPoint::make(0.0, 0.0);
    let Some(cullRect) = cullRect else {
        let mut pts = [z; 2];
        if srcPath.isLine(Some(&mut pts)) && pts[0] == pts[1] {
            adjust_zero_length_line(&mut pts);
            builder.moveTo(pts[0]);
            builder.lineTo(pts[1]);
            return true;
        }
        return false;
    };
    let mut bounds = *cullRect;
    outset_for_stroke(&mut bounds, rec);
    {
        let mut pts = [z; 2];
        if srcPath.isLine(Some(&mut pts)) {
            if clip_line(&mut pts, &bounds, intervalLength, 0.0) {
                builder.moveTo(pts[0]);
                builder.lineTo(pts[1]);
                return true;
            }
            return false;
        }
    }
    if srcPath.isRect(None, None, None) {
        let mut iter = Iter::new(srcPath, false);
        let first = iter.next();
        debug_assert!(
            first.is_some_and(|r| r.fVerb == crate::include::core::SkPathTypes::SkPathVerb::kMove)
        );
        let mut accum = 0.0_f64;
        while let Some(it) = iter.next() {
            if it.fVerb != crate::include::core::SkPathTypes::SkPathVerb::kLine {
                break;
            }
            let v = it.fPoints[1] - it.fPoints[0];
            let mut pts = [it.fPoints[0], it.fPoints[1]];
            if clip_line(
                &mut pts,
                &bounds,
                intervalLength,
                (accum % intervalLength as f64) as f32,
            ) {
                if builder.getLastPt().is_none_or(|last| last != pts[0]) {
                    builder.moveTo(pts[0]);
                }
                builder.lineTo(pts[1]);
            }
            debug_assert!(v.fX == 0.0 || v.fY == 0.0);
            accum += (v.fX + v.fY).abs() as f64;
        }
        return !builder.isEmpty();
    }
    false
}
// Source: src/utils/SkDashPath.cpp:235-306
struct SpecialLineRec {
    fPts: [SkPoint; 2],
    fTangent: SkPoint,
    fNormal: SkPoint,
    fPathLength: f32,
}
impl SpecialLineRec {
    // Language storage initialization; native init overwrites every field before addSegment.
    fn storage() -> Self {
        let z = SkPoint::make(0.0, 0.0);
        Self {
            fPts: [z; 2],
            fTangent: z,
            fNormal: z,
            fPathLength: 0.0,
        }
    }
    fn init(
        &mut self,
        src: &SkPath,
        dst: &mut SkPathBuilder,
        rec: &mut SkStrokeRec,
        intervalCount: i32,
        intervalLength: f32,
    ) -> bool {
        if rec.isHairlineStyle() || !src.isLine(Some(&mut self.fPts)) {
            return false;
        }
        if rec.getCap() != Cap::kButt_Cap {
            return false;
        }
        let pathLength = SkPoint::distance(self.fPts[0], self.fPts[1]);
        self.fTangent = self.fPts[1] - self.fPts[0];
        if self.fTangent.is_zero() {
            return false;
        }
        self.fPathLength = pathLength;
        self.fTangent.scale(1.0_f32 / pathLength);
        if !self.fTangent.fX.is_finite() || !self.fTangent.fY.is_finite() {
            return false;
        }
        self.fNormal = self.fTangent;
        SkPointPriv::RotateCCW(&mut self.fNormal);
        self.fNormal.scale(rec.getWidth() / 2.0);
        let mut ptCount = pathLength * intervalCount as f32 / intervalLength;
        if kMaxDashCount < ptCount {
            ptCount = kMaxDashCount;
        }
        if ptCount.is_nan() {
            return false;
        }
        let n = SkScalarCeilToInt(ptCount) << 2;
        dst.incReserve(n as usize, 0, 0);
        rec.setFillStyle();
        true
    }
    fn addSegment(&self, d0: f32, mut d1: f32, path: &mut SkPathBuilder) {
        crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),281; @condition b"d0 <= fPathLength\0".as_ptr(); d0<=self.fPathLength);
        if d1 > self.fPathLength {
            d1 = self.fPathLength;
        }
        let x0 = self.fPts[0].fX + self.fTangent.fX * d0;
        let x1 = self.fPts[0].fX + self.fTangent.fX * d1;
        let y0 = self.fPts[0].fY + self.fTangent.fY * d0;
        let y1 = self.fPts[0].fY + self.fTangent.fY * d1;
        let pts = [
            SkPoint::make(x0 + self.fNormal.fX, y0 + self.fNormal.fY),
            SkPoint::make(x1 + self.fNormal.fX, y1 + self.fNormal.fY),
            SkPoint::make(x1 - self.fNormal.fX, y1 - self.fNormal.fY),
            SkPoint::make(x0 - self.fNormal.fX, y0 - self.fNormal.fY),
        ];
        path.addPolygon(&pts, false);
    }
}
// Source: src/utils/SkDashPath.cpp:309-458
pub fn InternalFilter(
    dst: &mut SkPathBuilder,
    src: &SkPath,
    rec: &mut SkStrokeRec,
    cullRect: Option<&SkRect>,
    aIntervals: &[f32],
    initialDashLength: f32,
    initialDashIndex: i32,
    intervalLength: f32,
    startPhase: f32,
    strokeRecApplication: StrokeRecApplication,
) -> bool {
    let srcPts = src.points();
    if srcPts.is_empty() {
        return true;
    }
    let count = aIntervals.len();
    debug_assert!(is_even(count as i32));
    let style = rec.getStyle();
    if style == Style::kFill_Style || style == Style::kStrokeAndFill_Style {
        return false;
    }
    let intervals = aIntervals;
    let mut dashCount = 0.0;
    let mut builder = SkPathBuilder::new();
    let mut cullPathStorage = SkPath::default();
    let mut srcPtr = src;
    if cull_path(src, rec, cullRect, intervalLength, &mut builder) {
        if src.isRect(None, None, None) && src.isLastContourClosed() && is_even(initialDashIndex) {
            let pathLength = SkPathMeasure::new(src, false, rec.getResScale()).getLength();
            let mut endPhase = SkScalarMod(pathLength + startPhase, intervalLength);
            let mut index = 0;
            while endPhase > intervals[index] {
                endPhase -= intervals[index];
                index += 1;
                debug_assert!(index <= count);
                if index == count {
                    endPhase = 0.0;
                    break;
                }
            }
            if is_even(index as i32) == (endPhase > 0.0) {
                let midPoint = srcPts[0];
                let mut last = src.countPoints() as i32 - 1;
                while midPoint == srcPts[last as usize] {
                    last -= 1;
                    crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),360; @condition b"last >= 0\0".as_ptr(); last>=0);
                }
                let mut next = 1_i32;
                while midPoint == srcPts[next as usize] {
                    next += 1;
                    crate::SkASSERT!(@source b"src/utils/SkDashPath.cpp\0".as_ptr(),366; @condition b"next < last\0".as_ptr(); next<last);
                }
                let mut v = midPoint - srcPts[last as usize];
                let kTinyOffset = SK_ScalarNearlyZero;
                v *= kTinyOffset;
                builder.moveTo(midPoint - v);
                builder.lineTo(midPoint);
                v = midPoint - srcPts[next as usize];
                v *= kTinyOffset;
                builder.lineTo(midPoint - v);
            }
        }
        cullPathStorage = builder.detach(None);
        srcPtr = &cullPathStorage;
    }
    let mut lineRec = SpecialLineRec::storage();
    let specialLine = strokeRecApplication == StrokeRecApplication::kAllow
        && lineRec.init(srcPtr, dst, rec, (count >> 1) as i32, intervalLength);
    let mut meas = SkPathMeasure::new(srcPtr, false, rec.getResScale());
    loop {
        let mut skipFirstSegment = meas.isClosed();
        let mut addedSegment = false;
        let length = meas.getLength();
        let mut index = initialDashIndex as usize;
        dashCount += length * (count >> 1) as f32 / intervalLength;
        if dashCount > kMaxDashCount {
            dst.reset();
            return false;
        }
        let mut distance = 0.0_f64;
        let mut dlen = initialDashLength as f64;
        while distance < length as f64 {
            debug_assert!(dlen >= 0.0);
            addedSegment = false;
            if is_even(index as i32) && !skipFirstSegment {
                addedSegment = true;
                if specialLine {
                    lineRec.addSegment(distance as f32, (distance + dlen) as f32, dst);
                } else {
                    meas.getSegment(distance as f32, (distance + dlen) as f32, dst, true);
                }
            }
            distance += dlen;
            skipFirstSegment = false;
            index += 1;
            debug_assert!(index <= count);
            if index == count {
                index = 0;
            }
            dlen = intervals[index] as f64;
        }
        if meas.isClosed() && is_even(initialDashIndex) && initialDashLength >= 0.0 {
            meas.getSegment(0.0, initialDashLength, dst, !addedSegment);
        }
        if !meas.nextContour() {
            break;
        }
    }
    true
}
// Source: src/utils/SkDashPath.cpp:460-472
pub fn FilterDashPath(
    dst: &mut SkPathBuilder,
    src: &SkPath,
    rec: &mut SkStrokeRec,
    cullRect: Option<&SkRect>,
    info: &crate::src::core::SkPathEffectBase::DashInfo<'_>,
) -> bool {
    if !ValidDashPath(info.fPhase, info.fIntervals) {
        return false;
    }
    let mut initialDashLength = 0.0;
    let mut initialDashIndex = 0;
    let mut intervalLength = 0.0;
    CalcDashParameters(
        info.fPhase,
        info.fIntervals,
        &mut initialDashLength,
        &mut initialDashIndex,
        &mut intervalLength,
        None,
    );
    InternalFilter(
        dst,
        src,
        rec,
        cullRect,
        info.fIntervals,
        initialDashLength,
        initialDashIndex as i32,
        intervalLength,
        info.fPhase,
        StrokeRecApplication::kAllow,
    )
}
// Source: src/utils/SkDashPath.cpp:474-487
pub fn ValidDashPath(phase: f32, intervals: &[f32]) -> bool {
    if intervals.len() < 2 || intervals.len() & 1 != 0 {
        return false;
    }
    let mut length = 0.0;
    for &interval in intervals {
        if interval < 0.0 {
            return false;
        }
        length += interval;
    }
    length > 0.0 && phase.is_finite() && length.is_finite()
}
