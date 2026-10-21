// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::core::SkPathTypes::SkPathVerb;
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::{
    SkFlattenable::{Factory, FactoryResult, SkFlattenable, SkFlattenableView, Type},
    SkMatrix::SkMatrix,
    SkPath::{AddPathMode, Iter, SkPath},
    SkPathBuilder::SkPathBuilder,
    SkPathEffect::SkPathEffect,
    SkPathEffectBase::SkPathEffectBase,
    SkPathMeasure::{kGetPosAndTan_MatrixFlag, SkPathMeasure},
    SkPoint::SkPoint,
    SkReadBuffer::SkReadBuffer,
    SkRect::SkRect,
    SkStrokeRec::SkStrokeRec,
    SkWriteBuffer::SkWriteBuffer,
};
// Source: include/effects/Sk1DPathEffect.h:18-37
pub struct SkPath1DPathEffect;
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    kTranslate_Style,
    kRotate_Style,
    kMorph_Style,
}
pub const kLastEnum_Style: Style = Style::kMorph_Style;
// Source: src/effects/Sk1DPathEffect.cpp:29-70
const MAX_REASONABLE_ITERATIONS: i32 = 100000;
// The actual abstract native base has two required stepping virtuals. Its
// qualified base filter is shared by the concrete owner, not a second provider.
trait Sk1DPathEffect: SkPathEffectBase {
    fn begin(&self, contourLength: f32) -> f32;
    fn next(&self, dst: &mut SkPathBuilder, dist: f32, meas: &mut SkPathMeasure) -> f32;
    fn filter_contours(&self, builder: &mut SkPathBuilder, src: &SkPath) -> bool {
        let mut meas = SkPathMeasure::new(src, false, 1.0);
        loop {
            let mut governor = MAX_REASONABLE_ITERATIONS;
            let length = meas.getLength();
            let mut distance = self.begin(length);
            while distance < length {
                governor -= 1;
                if governor < 0 {
                    break;
                }
                let delta = self.next(builder, distance, &mut meas);
                if delta <= 0. {
                    break;
                }
                distance += delta;
            }
            if governor < 0 {
                return false;
            }
            if !meas.nextContour() {
                break;
            }
        }
        true
    }
}
// Source: src/effects/Sk1DPathEffect.cpp:74-147
struct SkPath1DPathEffectImpl {
    fPath: SkPath,
    fAdvance: f32,
    fInitialOffset: f32,
    fStyle: Style,
    base: SkRefCnt,
}
impl SkPath1DPathEffectImpl {
    // Source: src/effects/Sk1DPathEffect.cpp:76-102
    fn new(path: &SkPath, advance: f32, mut phase: f32, style: Style) -> Self {
        crate::SkASSERT!(@source b"src/effects/Sk1DPathEffect.cpp\0".as_ptr(),78; @condition b"advance > 0 && !path.isEmpty()\0".as_ptr(); advance>0.&&!path.isEmpty());
        if phase < 0. {
            phase = -phase;
            if phase > advance {
                phase %= advance;
            }
        } else {
            if phase > advance {
                phase %= advance;
            }
            phase = advance - phase;
        }
        if phase >= advance {
            phase = 0.;
        }
        crate::SkASSERT!(@source b"src/effects/Sk1DPathEffect.cpp\0".as_ptr(),97; @condition b"phase >= 0\0".as_ptr(); phase>=0.);
        Self {
            fPath: path.clone(),
            fAdvance: advance,
            fInitialOffset: phase,
            fStyle: style,
            base: SkRefCnt::new(),
        }
    }
    // Source: src/effects/Sk1DPathEffect.cpp:116-128
    fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let advance = buffer.readScalar();
        let mut result = sk_sp::default();
        if let Some(path) = buffer.readPath() {
            let phase = buffer.readScalar();
            let style = match buffer.read32LE(kLastEnum_Style as u32) {
                0 => Style::kTranslate_Style,
                1 => Style::kRotate_Style,
                _ => Style::kMorph_Style,
            };
            if buffer.isValid() {
                result = SkPath1DPathEffect::Make(&path, advance, phase, style);
            }
        }
        Ok(crate::src::core::SkPathEffect::into_flattenable(result))
    }
}
impl SkPathEffectBase for SkPath1DPathEffectImpl {
    // Source: src/effects/Sk1DPathEffect.cpp:104-108
    fn onFilterPath(
        &self,
        builder: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        _: Option<&SkRect>,
        _: &SkMatrix,
    ) -> bool {
        rec.setFillStyle();
        self.filter_contours(builder, src)
    }
    fn computeFastBounds(&self, _: Option<&mut SkRect>) -> bool {
        false
    }
}
// Source: src/effects/Sk1DPathEffect.cpp:149-173
fn morphpoints(dst: &mut [SkPoint], src: &[SkPoint], meas: &mut SkPathMeasure, dist: f32) -> bool {
    crate::SkASSERT!(@source b"src/effects/Sk1DPathEffect.cpp\0".as_ptr(),151; @condition b"dst.size() >= src.size()\0".as_ptr(); dst.len()>=src.len());
    for i in 0..src.len() {
        let mut pos = SkPoint::make(0., 0.);
        let mut tangent = SkPoint::make(0., 0.);
        let sx = src[i].fX;
        let sy = src[i].fY;
        if !meas.getPosTan(dist + sx, Some(&mut pos), Some(&mut tangent)) {
            return false;
        }
        let mut matrix = SkMatrix::default();
        let pt = SkPoint::make(sx, sy);
        matrix.setSinCosPivot(tangent.fY, tangent.fX, 0., 0.);
        matrix.preTranslate(-sx, 0.);
        matrix.postTranslate(pos.fX, pos.fY);
        dst[i] = matrix.mapPoint(pt);
    }
    true
}
// Source: src/effects/Sk1DPathEffect.cpp:181-221
fn morphpath(dst: &mut SkPathBuilder, src: &SkPath, meas: &mut SkPathMeasure, dist: f32) {
    let mut iter = Iter::new(src, false);
    let mut dstP = [SkPoint::make(0., 0.); 3];
    while let Some(rec) = iter.next() {
        let srcP = rec.fPoints;
        match rec.fVerb {
            SkPathVerb::kMove => {
                if morphpoints(&mut dstP, srcP, meas, dist) {
                    dst.moveTo(dstP[0]);
                }
            }
            SkPathVerb::kLine => {
                let scratch = [
                    srcP[0],
                    SkPoint::make(
                        crate::src::core::SkFloatingPoint::sk_float_midpoint(
                            srcP[0].fX, srcP[1].fX,
                        ),
                        crate::src::core::SkFloatingPoint::sk_float_midpoint(
                            srcP[0].fY, srcP[1].fY,
                        ),
                    ),
                    srcP[1],
                ];
                if morphpoints(&mut dstP, &scratch[1..], meas, dist) {
                    dst.quadTo(dstP[0], dstP[1]);
                }
            }
            SkPathVerb::kQuad => {
                if morphpoints(&mut dstP, &srcP[1..], meas, dist) {
                    dst.quadTo(dstP[0], dstP[1]);
                }
            }
            SkPathVerb::kConic => {
                if morphpoints(&mut dstP, &srcP[1..], meas, dist) {
                    dst.conicTo(dstP[0], dstP[1], rec.conicWeight());
                }
            }
            SkPathVerb::kCubic => {
                if morphpoints(&mut dstP, &srcP[1..], meas, dist) {
                    dst.cubicTo(dstP[0], dstP[1], dstP[2]);
                }
            }
            SkPathVerb::kClose => {
                dst.close();
            }
        }
    }
}
// Source: src/effects/Sk1DPathEffect.cpp:223-248
impl Sk1DPathEffect for SkPath1DPathEffectImpl {
    // Source: src/effects/Sk1DPathEffect.cpp:110-112
    fn begin(&self, _: f32) -> f32 {
        self.fInitialOffset
    }
    fn next(&self, builder: &mut SkPathBuilder, distance: f32, meas: &mut SkPathMeasure) -> f32 {
        #[cfg(sk_build_for_fuzzer)]
        if builder.countPoints() > 100000 {
            return self.fAdvance;
        }
        match self.fStyle {
            Style::kTranslate_Style => {
                let mut pos = SkPoint::make(0., 0.);
                if meas.getPosTan(distance, Some(&mut pos), None) {
                    builder.addPathOffset(
                        &self.fPath,
                        pos.fX,
                        pos.fY,
                        AddPathMode::kAppend_AddPathMode,
                    );
                }
            }
            Style::kRotate_Style => {
                let mut matrix = SkMatrix::default();
                if meas.getMatrix(distance, Some(&mut matrix), kGetPosAndTan_MatrixFlag) {
                    builder.addPath(&self.fPath, &matrix, AddPathMode::kAppend_AddPathMode);
                }
            }
            Style::kMorph_Style => morphpath(builder, &self.fPath, meas, distance),
        }
        self.fAdvance
    }
}
// Source: src/effects/Sk1DPathEffect.cpp:252-262
impl SkPath1DPathEffect {
    pub fn Make(path: &SkPath, advance: f32, phase: f32, style: Style) -> sk_sp<dyn SkPathEffect> {
        if advance <= 0. || !advance.is_finite() || !phase.is_finite() || path.isEmpty() {
            return sk_sp::default();
        }
        let object: Box<dyn SkPathEffect> =
            Box::new(SkPath1DPathEffectImpl::new(path, advance, phase, style));
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    }
    /// Safety: exclusive native registration phase, before once publication;
    /// no concurrent lookup/registration, with space in the fixed factory table.
    pub unsafe fn RegisterFlattenables() {
        unsafe {
            #[cfg(not(sk_disable_effect_deserialization))]
            crate::src::core::SkFlattenable::Register(
                b"SkPath1DPathEffectImpl",
                SkPath1DPathEffectImpl::CreateProc,
            );
        }
    }
}
// Structural native intrusive-base/virtual metadata projection; no second owner.
unsafe impl SkRefCntObject for SkPath1DPathEffectImpl {
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    fn ref_(&self) {
        SkRefCnt::ref_(&self.base);
    }
}
impl SkFlattenable for SkPath1DPathEffectImpl {
    unsafe fn project_path_effect_owner(
        &self,
        allocation: core::ptr::NonNull<()>,
    ) -> Option<core::ptr::NonNull<dyn SkPathEffect>> {
        let concrete: *mut Self = allocation.as_ptr().cast();
        Some(unsafe { core::ptr::NonNull::new_unchecked(concrete as *mut dyn SkPathEffect) })
    }
}
impl SkFlattenableView for SkPath1DPathEffectImpl {
    fn getFactory(&self) -> Option<Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkPath1DPathEffect")
    }
    fn getFlattenableType(&self) -> Type {
        Type::kSkPathEffect_Type
    }
    // Source: src/effects/Sk1DPathEffect.cpp:130-135
    fn flatten(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        buffer.writeScalar(self.fAdvance);
        buffer.writePath(&self.fPath);
        buffer.writeScalar(self.fInitialOffset);
        buffer.writeUInt(self.fStyle as u32);
        Ok(())
    }
}
