use crate::include::core::SkPathTypes::SkPathVerb;
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::{Iter, SkPath};
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkStrokeRec::SkStrokeRec;
// Source: src/effects/SkCornerPathEffect.cpp:27-39
fn ComputeStep(a: SkPoint, b: SkPoint, radius: f32, step: &mut SkPoint) -> bool {
    let dist = SkPoint::distance(a, b);
    *step = b - a;
    if dist <= radius * 2.0 {
        *step *= 0.5;
        false
    } else {
        *step *= radius / dist;
        true
    }
}
// Source: src/effects/SkCornerPathEffect.cpp:41-41
// Composition plus trait dispatch is the Rust inheritance adaptation; counter is native.
unsafe impl SkRefCntObject for SkCornerPathEffectImpl {
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
impl SkCornerPathEffectImpl {
    // Source: src/effects/SkCornerPathEffect.cpp:43-45
    pub fn new(radius: f32) -> Self {
        debug_assert!(radius > 0.0);
        Self {
            fRadius: radius,
            fRefCnt: SkRefCnt::new(),
        }
    }
}
impl crate::src::core::SkPathEffectBase::SkPathEffectBase for SkCornerPathEffectImpl {
    // Source: src/effects/SkCornerPathEffect.cpp:47-151
    fn onFilterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        _rec: &mut SkStrokeRec,
        _cull: Option<&SkRect>,
        _ctm: &SkMatrix,
    ) -> bool {
        if self.fRadius <= 0.0 {
            return false;
        }
        // Option represents the original invalid 0xff enum sentinel.
        let mut prevVerb = None;
        let mut moveTo = SkPoint::make(0.0, 0.0);
        let mut lastCorner = SkPoint::make(0.0, 0.0);
        let mut firstStep = SkPoint::make(0.0, 0.0);
        let mut step = SkPoint::make(0.0, 0.0);
        let mut prevIsValid = true;
        let mut iter = Iter::new(src, false);
        while let Some(rec) = iter.next() {
            // The native Iter record lends its internal point storage; copy only the four-value
            // record to release that borrow before the immutable isClosedContour query.
            let mut points = [SkPoint::make(0.0, 0.0); 4];
            points[..rec.fPoints.len()].copy_from_slice(rec.fPoints);
            let pts = &points;
            let verb = rec.fVerb;
            let weight = if rec.fVerb == SkPathVerb::kConic {
                rec.conicWeight()
            } else {
                rec.fConicWeight
            };
            match verb {
                SkPathVerb::kMove => {
                    if prevVerb == Some(SkPathVerb::kLine) {
                        dst.lineTo(lastCorner);
                    }
                    let closed = iter.isClosedContour();
                    if closed {
                        moveTo = pts[0];
                        prevIsValid = false;
                    } else {
                        dst.moveTo(pts[0]);
                        prevIsValid = true;
                    }
                }
                SkPathVerb::kLine => {
                    let drawSegment = ComputeStep(pts[0], pts[1], self.fRadius, &mut step);
                    if !prevIsValid {
                        dst.moveTo(moveTo + step);
                        prevIsValid = true;
                    } else {
                        dst.quadTo(
                            pts[0],
                            SkPoint::make(pts[0].fX + step.fX, pts[0].fY + step.fY),
                        );
                    }
                    if drawSegment {
                        dst.lineTo(SkPoint::make(pts[1].fX - step.fX, pts[1].fY - step.fY));
                    }
                    lastCorner = pts[1];
                    prevIsValid = true;
                }
                SkPathVerb::kQuad => {
                    if !prevIsValid {
                        dst.moveTo(pts[0]);
                        prevIsValid = true;
                    }
                    dst.quadTo(pts[1], pts[2]);
                    lastCorner = pts[2];
                    firstStep.set(0.0, 0.0);
                }
                SkPathVerb::kConic => {
                    if !prevIsValid {
                        dst.moveTo(pts[0]);
                        prevIsValid = true;
                    }
                    dst.conicTo(pts[1], pts[2], weight);
                    lastCorner = pts[2];
                    firstStep.set(0.0, 0.0);
                }
                SkPathVerb::kCubic => {
                    if !prevIsValid {
                        dst.moveTo(pts[0]);
                        prevIsValid = true;
                    }
                    dst.cubicTo(pts[1], pts[2], pts[3]);
                    lastCorner = pts[3];
                    firstStep.set(0.0, 0.0);
                }
                SkPathVerb::kClose => {
                    if firstStep.fX != 0.0 || firstStep.fY != 0.0 {
                        dst.quadTo(
                            lastCorner,
                            SkPoint::make(
                                lastCorner.fX + firstStep.fX,
                                lastCorner.fY + firstStep.fY,
                            ),
                        );
                    }
                    dst.close();
                    prevIsValid = false;
                }
            }
            if prevVerb == Some(SkPathVerb::kMove) {
                firstStep = step;
            }
            prevVerb = Some(verb);
        }
        if prevIsValid {
            dst.lineTo(lastCorner);
        }
        true
    }
    // Source: src/effects/SkCornerPathEffect.cpp:153-157
    fn computeFastBounds(&self, _bounds: Option<&mut SkRect>) -> bool {
        true
    }
}
// Source: src/effects/SkCornerPathEffect.cpp:159-161
impl SkCornerPathEffectImpl {
    pub(crate) fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        Ok(crate::src::core::SkPathEffect::into_flattenable(Make(
            buffer.readScalar(),
        )))
    }
}
impl crate::src::core::SkFlattenable::SkFlattenable for SkCornerPathEffectImpl {
    unsafe fn project_path_effect_owner(
        &self,
        allocation: std::ptr::NonNull<()>,
    ) -> Option<std::ptr::NonNull<dyn crate::src::core::SkPathEffect::SkPathEffect>> {
        let concrete: *mut Self = allocation.as_ptr().cast();
        Some(unsafe {
            std::ptr::NonNull::new_unchecked(
                concrete as *mut dyn crate::src::core::SkPathEffect::SkPathEffect,
            )
        })
    }
}
impl crate::src::core::SkFlattenable::SkFlattenableView for SkCornerPathEffectImpl {
    // Source: src/effects/SkCornerPathEffect.cpp:163-165
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeScalar(self.fRadius);
        Ok(())
    }
    // Source: src/effects/SkCornerPathEffect.cpp:167-167
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    // Source: src/effects/SkCornerPathEffect.cpp:168-168
    fn getTypeName(&self) -> Option<&str> {
        Some("SkCornerPathEffect")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkPathEffectBase::SkPathEffectBase::getFlattenableType(self)
    }
}
// Source: src/effects/SkCornerPathEffect.cpp:171-171
pub struct SkCornerPathEffectImpl {
    fRadius: f32,
    fRefCnt: SkRefCnt,
}
// Source: src/effects/SkCornerPathEffect.cpp:178-181
pub fn Make(radius: f32) -> sk_sp<dyn crate::src::core::SkPathEffect::SkPathEffect> {
    if radius.is_finite() && radius > 0.0 {
        let object: Box<dyn crate::src::core::SkPathEffect::SkPathEffect> =
            Box::new(SkCornerPathEffectImpl::new(radius));
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    } else {
        sk_sp::default()
    }
}
// Source: src/effects/SkCornerPathEffect.cpp:183-185
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn RegisterFlattenables() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkCornerPathEffect",
            SkCornerPathEffectImpl::CreateProc,
        );
    }
}
