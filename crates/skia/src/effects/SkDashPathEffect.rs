use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt};
use crate::include::core::SkScalar::SkScalarMod;
use crate::include::core::SkScalar::{
    SK_ScalarNearlyZero, SkScalarFloorToInt, SkScalarInvert, SkScalarIsInt, SkScalarNearlyEqual,
};
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPaint::Cap;
use crate::src::core::SkPaint::Join;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPathEffectBase::{DashInfo, PointData, SkPathEffectBase};
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkStrokeRec::SkStrokeRec;
use crate::src::effects::SkDashImpl::SkDashImpl;
// Source: src/effects/SkDashPathEffect.cpp:36-50
impl SkDashImpl {
    pub(crate) fn new(intervals: &[f32], phase: f32) -> Self {
        let mut result = Self {
            fRefCnt: SkRefCnt::new(),
            fIntervals: unsafe {
                crate::include::private::SkTemplates::AutoTArray::new_uninit(intervals.len())
            },
            fPhase: 0.0,
            fInitialDashLength: -1.0,
            fIntervalLength: 0.0,
            fInitialDashIndex: 0,
        };
        debug_assert!(intervals.len() > 1 && intervals.len() & 1 == 0);
        // A valid Dash has >1 intervals. The extra empty guard keeps Rust's zero-count
        // copy pointer validity even for an invalid direct internal constructor call.
        if !intervals.is_empty() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    intervals.as_ptr(),
                    result.fIntervals.data_mut(),
                    intervals.len(),
                );
            }
        }
        unsafe {
            result.fIntervals.assume_init();
        }
        crate::src::utils::SkDashPath::CalcDashParameters(
            phase,
            result.fIntervals.as_slice().unwrap(),
            &mut result.fInitialDashLength,
            &mut result.fInitialDashIndex,
            &mut result.fIntervalLength,
            Some(&mut result.fPhase),
        );
        result
    }
    // Source: src/effects/SkDashPathEffect.cpp:52-57
    fn onFilterPath_impl(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cullRect: Option<&SkRect>,
        _: &SkMatrix,
    ) -> bool {
        crate::src::utils::SkDashPath::InternalFilter(
            builder,
            src,
            rec,
            cullRect,
            self.fIntervals.as_slice().unwrap(),
            self.fInitialDashLength,
            self.fInitialDashIndex as i32,
            self.fIntervalLength,
            self.fPhase,
            crate::src::utils::SkDashPathPriv::StrokeRecApplication::kAllow,
        )
    }
}
// Source: src/effects/SkDashPathEffect.cpp:59-68
pub(crate) fn outset_for_stroke(rect: &mut SkRect, rec: &SkStrokeRec) {
    let mut radius = rec.getWidth() / 2.0_f32;
    if radius == 0.0 {
        radius = 1.0;
    }
    if rec.getJoin() == Join::kMiter_Join {
        radius *= rec.getMiter();
    }
    rect.outset(radius, radius);
}
// Source: src/effects/SkDashPathEffect.cpp:73-171
pub(crate) fn cull_line(
    pts: &mut [SkPoint; 2],
    rec: &SkStrokeRec,
    ctm: &SkMatrix,
    cullRect: Option<&SkRect>,
    intervalLength: f32,
) -> bool {
    let Some(cullRect) = cullRect else {
        crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),77; @condition b"false\0".as_ptr(); false);
        return false;
    };
    let dx = pts[1].x() - pts[0].x();
    let dy = pts[1].y() - pts[0].y();
    if (dx != 0.0 && dy != 0.0) || (dx == 0.0 && dy == 0.0) {
        return false;
    }
    let mut bounds = *cullRect;
    outset_for_stroke(&mut bounds, rec);
    crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),94; @condition b"ctm.rectStaysRect()\0".as_ptr(); ctm.rectStaysRect());
    let Some(inv) = ctm.invert() else {
        return false;
    };
    let original_bounds = bounds;
    inv.mapRect(&mut bounds, &original_bounds);
    if dx != 0.0 {
        debug_assert!(dx != 0.0 && dy == 0.0);
        let mut minX = pts[0].fX;
        let mut maxX = pts[1].fX;
        if dx < 0.0 {
            std::mem::swap(&mut minX, &mut maxX);
        }
        crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),112; @condition b"minX < maxX\0".as_ptr(); minX < maxX);
        if maxX <= bounds.fLeft || minX >= bounds.fRight {
            return false;
        }
        if minX < bounds.fLeft {
            minX = bounds.fLeft - SkScalarMod(bounds.fLeft - minX, intervalLength);
        }
        if maxX > bounds.fRight {
            maxX = bounds.fRight + SkScalarMod(maxX - bounds.fRight, intervalLength);
        }
        crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),128; @condition b"maxX > minX\0".as_ptr(); maxX > minX);
        if dx < 0.0 {
            std::mem::swap(&mut minX, &mut maxX);
        }
        pts[0].fX = minX;
        pts[1].fX = maxX;
    } else {
        debug_assert!(dy != 0.0 && dx == 0.0);
        let mut minY = pts[0].fY;
        let mut maxY = pts[1].fY;
        if dy < 0.0 {
            std::mem::swap(&mut minY, &mut maxY);
        }
        crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),145; @condition b"minY < maxY\0".as_ptr(); minY < maxY);
        if maxY <= bounds.fTop || minY >= bounds.fBottom {
            return false;
        }
        if minY < bounds.fTop {
            minY = bounds.fTop - SkScalarMod(bounds.fTop - minY, intervalLength);
        }
        if maxY > bounds.fBottom {
            maxY = bounds.fBottom + SkScalarMod(maxY - bounds.fBottom, intervalLength);
        }
        crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),161; @condition b"maxY > minY\0".as_ptr(); maxY > minY);
        if dy < 0.0 {
            std::mem::swap(&mut minY, &mut maxY);
        }
        pts[0].fY = minY;
        pts[1].fY = maxY;
    }
    true
}

impl SkPathEffectBase for SkDashImpl {
    // Rust trait receiver forwards to the original cpp52-57 method above.
    fn onFilterPath(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        cullRect: Option<&SkRect>,
        ctm: &SkMatrix,
    ) -> bool {
        self.onFilterPath_impl(builder, src, rec, cullRect, ctm)
    }
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool {
        self.computeFastBounds_impl(bounds)
    }
    // Source: src/effects/SkDashPathEffect.cpp:177-368
    fn onAsPoints(
        &self,
        mut results: Option<&mut PointData>,
        src: &SkPath,
        rec: &SkStrokeRec,
        matrix: &SkMatrix,
        cullRect: Option<&SkRect>,
    ) -> bool {
        if 0.0 >= rec.getWidth() {
            return false;
        }
        let intervals = self.fIntervals.as_slice().unwrap();
        if self.fIntervals.size() != 2
            || !SkScalarNearlyEqual(intervals[0], intervals[1], SK_ScalarNearlyZero)
            || !SkScalarIsInt(intervals[0])
            || !SkScalarIsInt(intervals[1])
        {
            return false;
        }
        let mut pts = [SkPoint::make(0.0, 0.0); 2];
        if !src.isLine(Some(&mut pts)) {
            return false;
        }
        if rec.getCap() != Cap::kButt_Cap {
            return false;
        }
        if !matrix.rectStaysRect() {
            return false;
        }
        if !cull_line(&mut pts, rec, matrix, cullRect, self.fIntervalLength) {
            return false;
        }
        let length = SkPoint::distance(pts[1], pts[0]);
        let mut tangent = pts[1] - pts[0];
        if tangent.is_zero() {
            return false;
        }
        tangent.scale(SkScalarInvert(length));
        let mut isXAxis = true;
        if SkScalarNearlyEqual(1.0, tangent.fX, SK_ScalarNearlyZero)
            || SkScalarNearlyEqual(-1.0, tangent.fX, SK_ScalarNearlyZero)
        {
            // Native writes fSize before its later `if(results)` guard. Null here is an
            // invalid native call; abort rather than introduce a fake successful probe.
            let Some(output) = results.as_deref_mut() else {
                std::process::abort();
            };
            output.fSize.set(intervals[0] / 2.0, rec.getWidth() / 2.0);
        } else if SkScalarNearlyEqual(1.0, tangent.fY, SK_ScalarNearlyZero)
            || SkScalarNearlyEqual(-1.0, tangent.fY, SK_ScalarNearlyZero)
        {
            let Some(output) = results.as_deref_mut() else {
                std::process::abort();
            };
            output.fSize.set(rec.getWidth() / 2.0, intervals[0] / 2.0);
            isXAxis = false;
        } else if rec.getCap() != Cap::kRound_Cap {
            return false;
        }
        if let Some(results) = results {
            results.fFlags = 0;
            // std::min retains its first operand when comparison is false (including NaN).
            let clampedInitialDashLength = if self.fInitialDashLength < length {
                self.fInitialDashLength
            } else {
                length
            };
            if rec.getCap() == Cap::kRound_Cap {
                results.fFlags |= crate::src::core::SkPathEffectBase::kCircles_PointFlag;
            }
            results.fNumPoints = 0;
            let mut len2 = length;
            if clampedInitialDashLength > 0.0 || self.fInitialDashIndex == 0 {
                crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),251; @condition b"len2 >= clampedInitialDashLength\0".as_ptr(); len2>=clampedInitialDashLength);
                if self.fInitialDashIndex == 0 {
                    if clampedInitialDashLength > 0.0 {
                        if clampedInitialDashLength >= intervals[0] {
                            results.fNumPoints += 1;
                        }
                        len2 -= clampedInitialDashLength;
                    }
                    len2 -= intervals[1];
                    if len2 < 0.0 {
                        len2 = 0.0;
                    }
                } else {
                    len2 -= clampedInitialDashLength;
                }
            }
            let numIntervals = len2 / self.fIntervalLength;
            if !numIntervals.is_finite()
                || numIntervals > crate::src::utils::SkDashPathPriv::kMaxDashCount
            {
                return false;
            }
            let mut numMidPoints = SkScalarFloorToInt(numIntervals);
            results.fNumPoints += numMidPoints;
            len2 -= numMidPoints as f32 * self.fIntervalLength;
            let mut partialLast = false;
            if len2 > 0.0 {
                if len2 < intervals[0] {
                    partialLast = true;
                } else {
                    numMidPoints += 1;
                    results.fNumPoints += 1;
                }
            }
            // Native new Point[count] allocates uninitialized POD here. Rust stages that exact
            // allocation until all slots are written, then publishes the valid Box slice.
            let mut pointStorage = Box::<[SkPoint]>::new_uninit_slice(results.fNumPoints as usize);
            // Raw native assignment overwrites the prior pointer without deleting it.
            if let Some(old) = results.fPoints.take() {
                let _ = Box::into_raw(old);
            }
            let mut distance = 0.0;
            let mut curPt = 0_i32;
            if clampedInitialDashLength > 0.0 || self.fInitialDashIndex == 0 {
                crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),293; @condition b"clampedInitialDashLength <= length\0".as_ptr(); clampedInitialDashLength<=length);
                if self.fInitialDashIndex == 0 {
                    if clampedInitialDashLength > 0.0 {
                        debug_assert!(rec.getCap() != Cap::kRound_Cap);
                        let x = pts[0].fX + tangent.fX * (clampedInitialDashLength / 2.0);
                        let y = pts[0].fY + tangent.fY * (clampedInitialDashLength / 2.0);
                        let (halfWidth, halfHeight) = if isXAxis {
                            (clampedInitialDashLength / 2.0, rec.getWidth() / 2.0)
                        } else {
                            (rec.getWidth() / 2.0, clampedInitialDashLength / 2.0)
                        };
                        if clampedInitialDashLength < intervals[0] {
                            results.fFirst = SkPath::RectDefault(
                                &SkRect::MakeLTRB(
                                    x - halfWidth,
                                    y - halfHeight,
                                    x + halfWidth,
                                    y + halfHeight,
                                ),
                                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                                0,
                            );
                        } else {
                            debug_assert!(curPt < results.fNumPoints);
                            pointStorage[curPt as usize].write(SkPoint::make(x, y));
                            curPt += 1;
                        }
                        distance += clampedInitialDashLength;
                    }
                    distance += intervals[1];
                } else {
                    distance += clampedInitialDashLength;
                }
            }
            if numMidPoints != 0 {
                distance += intervals[0] / 2.0;
                for _ in 0..numMidPoints {
                    let x = pts[0].fX + tangent.fX * distance;
                    let y = pts[0].fY + tangent.fY * distance;
                    debug_assert!(curPt < results.fNumPoints);
                    pointStorage[curPt as usize].write(SkPoint::make(x, y));
                    curPt += 1;
                    distance += self.fIntervalLength;
                }
                distance -= intervals[0] / 2.0;
            }
            if partialLast {
                debug_assert!(rec.getCap() != Cap::kRound_Cap);
                let temp = length - distance;
                debug_assert!(temp < intervals[0]);
                let x = pts[0].fX + tangent.fX * (distance + (temp / 2.0));
                let y = pts[0].fY + tangent.fY * (distance + (temp / 2.0));
                let (halfWidth, halfHeight) = if isXAxis {
                    (temp / 2.0, rec.getWidth() / 2.0)
                } else {
                    (rec.getWidth() / 2.0, temp / 2.0)
                };
                results.fLast = SkPath::RectDefault(
                    &SkRect::MakeLTRB(x - halfWidth, y - halfHeight, x + halfWidth, y + halfHeight),
                    crate::include::core::SkPathTypes::SkPathDirection::kCW,
                    0,
                );
            }
            crate::SkASSERT!(@source b"src/effects/SkDashPathEffect.cpp\0".as_ptr(),364; @condition b"curPt == results->fNumPoints\0".as_ptr(); (curPt)==(results.fNumPoints));
            // Rust validity guard for converting MaybeUninit<Point>[] into Point[].
            if curPt as usize != pointStorage.len() {
                std::process::abort();
            }
            results.fPoints = Some(unsafe { pointStorage.assume_init() });
        }
        true
    }
    // Source: src/effects/SkDashPathEffect.cpp:370-372
    fn asADash(&self) -> Option<DashInfo<'_>> {
        Some(DashInfo {
            fIntervals: self.fIntervals.as_slice().unwrap(),
            fPhase: self.fPhase,
        })
    }
}
// Source: src/effects/SkDashPathEffect.cpp:374-377
impl SkDashImpl {
    pub(crate) fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeScalar(self.fPhase);
        buffer.writeScalarArray(self.fIntervals.as_slice().unwrap());

        Ok(())
    }
    // Source: src/effects/SkDashPathEffect.cpp:379-393
    pub(crate) fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let phase = buffer.readScalar();
        let count = buffer.getArrayCount() as usize;
        if !buffer.validateCanReadN::<f32>(count) {
            return Ok(None);
        }
        let mut intervals = vec![0.0; count];
        if buffer.readScalarArray(&mut intervals) {
            return Ok(crate::src::core::SkPathEffect::into_flattenable(Make(
                &intervals, phase,
            )));
        }
        Ok(None)
    }
}
// Source: src/effects/SkDashPathEffect.cpp:397-402
pub fn Make(
    intervals: &[f32],
    phase: f32,
) -> sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect> {
    if !crate::src::utils::SkDashPath::ValidDashPath(phase, intervals) {
        return sk_sp::default();
    }
    let object: Box<dyn crate::src::core::SkPathEffect::SkPathEffect> =
        Box::new(SkDashImpl::new(intervals, phase));
    unsafe { sk_sp::from_raw(Box::into_raw(object)) }
}
