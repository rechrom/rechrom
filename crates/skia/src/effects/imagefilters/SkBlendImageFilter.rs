#![allow(non_snake_case)]
use crate::include::core::{SkBlender::SkBlender, SkRefCnt::sk_sp, SkTileMode::SkTileMode};
use crate::src::core::{
    SkBlendMode::{SkBlendMode, SkBlendModeCoeff, SkBlendMode_AsCoeff},
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkRect::{SkIRect, SkRect},
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:91-96
// Arithmetic factory stays pending its canonical runtime blender; this field retains original geometry support.
struct SkBlendImageFilter {
    fEnforcePremul: bool,
    fArithmeticCoefficients: Option<[f32; 4]>,
    fBlender: sk_sp<dyn SkBlender>,
    base: SkImageFilter_Base,
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:45-55
impl SkBlendImageFilter {
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:41-42
    const kBackground: usize = 0;
    const kForeground: usize = 1;
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:45-55
    fn new(
        blender: sk_sp<dyn SkBlender>,
        coeff: Option<[f32; 4]>,
        enforce: bool,
        inputs: [Option<Arc<dyn SkImageFilter>>; 2],
    ) -> Self {
        let filter = Self {
            base: SkImageFilter_Base::new(&inputs, None),
            fBlender: blender,
            fArithmeticCoefficients: coeff,
            fEnforcePremul: enforce,
        };
        // Native assertion follows construction of the retained blender/base.
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlendImageFilter.cpp\0".as_ptr(),54; @condition b"fBlender\0".as_ptr(); filter.fBlender.is_some());
        filter
    }
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:99-130
fn make_blend(
    blender: Option<sk_sp<dyn SkBlender>>,
    bg: Option<Arc<dyn SkImageFilter>>,
    fg: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
    coeff: Option<[f32; 4]>,
    enforce: bool,
) -> Option<Arc<dyn SkImageFilter>> {
    // Outer Option and the native nullable intrusive owner represent the same
    // native null. Normalize both before coefficient inspection/construction.
    let blender = match blender {
        Some(blender) if blender.is_some() => blender,
        _ => <dyn SkBlender>::Mode(SkBlendMode::kSrcOver),
    };
    let cropped = |filter| {
        if let Some(rect) = crop {
            super::SkCropImageFilter::Crop(rect, SkTileMode::kDecal, filter)
        } else {
            filter
        }
    };
    if let Some(mode) = blender.asBlendMode() {
        if mode == SkBlendMode::kSrc {
            return cropped(fg);
        } else if mode == SkBlendMode::kDst {
            return cropped(bg);
        } else if mode == SkBlendMode::kClear {
            return super::SkCropImageFilter::Empty();
        }
    }
    cropped(Some(Arc::new(SkBlendImageFilter::new(
        blender,
        coeff,
        enforce,
        [bg, fg],
    ))))
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:134-142
pub fn Blend(
    mode: SkBlendMode,
    bg: Option<Arc<dyn SkImageFilter>>,
    fg: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_blend(Some(<dyn SkBlender>::Mode(mode)), bg, fg, crop, None, false)
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:144-149
pub fn BlendWithBlender(
    blender: Option<sk_sp<dyn SkBlender>>,
    bg: Option<Arc<dyn SkImageFilter>>,
    fg: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_blend(blender, bg, fg, crop, None, false)
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:63-63
const kArithmetic_SkBlendMode: u32 =
    crate::src::core::SkBlendModePriv::kCustom_SkBlendMode as u32 + 1;
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:66-66
impl crate::src::core::SkFlattenable::SkFlattenableView for SkBlendImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkBlendImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:151-172
pub fn Arithmetic(
    k1: f32,
    k2: f32,
    k3: f32,
    k4: f32,
    enforce: bool,
    bg: Option<Arc<dyn SkImageFilter>>,
    fg: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    let blender = crate::src::effects::SkBlenders::Arithmetic(k1, k2, k3, k4, enforce);
    if !blender.is_some() {
        return None;
    }
    make_blend(Some(blender), bg, fg, crop, Some([k1, k2, k3, k4]), enforce)
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:174-183
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterBlendImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkBlendImageFilter",
            SkBlendImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkXfermodeImageFilter_Base",
            SkBlendImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkXfermodeImageFilterImpl",
            SkBlendImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"ArithmeticImageFilterImpl",
            SkBlendImageFilter::LegacyArithmeticCreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkArithmeticImageFilter",
            SkBlendImageFilter::LegacyArithmeticCreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkBlendImageFilter.cpp:185-200
impl SkBlendImageFilter {
    pub fn LegacyArithmeticCreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let old =
            buffer.isVersionLT(crate::src::core::SkPicturePriv::kCombineBlendArithmeticFilters);
        if !buffer.validate(old) {
            crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlendImageFilter.cpp\0".as_ptr(),188; @condition b"false\0".as_ptr(); false);
            return Ok(None);
        }
        crate::src::core::SkImageFilter_Base::SK_IMAGEFILTER_UNFLATTEN_COMMON!(common, buffer, 2);
        let mut k = [0.0; 4];
        for value in &mut k {
            *value = buffer.readScalar();
        }
        let enforce = buffer.readBool();
        Ok(Arithmetic(
            k[0],
            k[1],
            k[2],
            k[3],
            enforce,
            common.getInput(0),
            common.getInput(1),
            common.cropRect(),
        )
        .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:202-239
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 2)? {
            return Ok(None);
        }
        let mut blender = sk_sp::default();
        let mut coefficients = None;
        let mut enforce = false;
        let mode = buffer.read32() as u32;
        if mode == kArithmetic_SkBlendMode {
            let current = !buffer
                .isVersionLT(crate::src::core::SkPicturePriv::kCombineBlendArithmeticFilters);
            if buffer.validate(current) {
                let mut k = [0.0; 4];
                for value in &mut k {
                    *value = buffer.readScalar();
                }
                coefficients = Some(k);
                enforce = buffer.readBool();
                blender =
                    crate::src::effects::SkBlenders::Arithmetic(k[0], k[1], k[2], k[3], enforce);
                if !buffer.validate(blender.is_some()) {
                    return Ok(None);
                }
            }
        } else if mode == crate::src::core::SkBlendModePriv::kCustom_SkBlendMode as u32 {
            blender = buffer.readBlender()?;
        } else {
            if !buffer.validate(mode <= SkBlendMode::kLastMode as u32) {
                return Ok(None);
            }
            let mode: SkBlendMode = unsafe { std::mem::transmute(mode as u8) };
            blender = crate::src::core::SkBlendModeBlender::BlenderMode(mode);
        }
        Ok(make_blend(
            Some(blender),
            common.getInput(0),
            common.getInput(1),
            common.cropRect(),
            coefficients,
            enforce,
        )
        .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:241-258
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        if let Some(k) = self.fArithmeticCoefficients {
            buffer.write32(kArithmetic_SkBlendMode as i32);
            for value in k {
                buffer.writeScalar(value);
            }
            buffer.writeBool(self.fEnforcePremul);
        } else if let Some(mode) = self.fBlender.get().unwrap().asBlendMode() {
            buffer.write32(mode as i32);
        } else {
            buffer.write32(crate::src::core::SkBlendModePriv::kCustom_SkBlendMode as i32);
            buffer.writeFlattenable(
                self.fBlender
                    .get()
                    .map(|b| b as &dyn crate::src::core::SkFlattenable::SkFlattenableView),
            )?;
        }

        Ok(())
    }
}

impl SkBlendImageFilter {
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:262-293
    fn makeBlendShader(
        &self,
        mut bg: Option<Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        mut fg: Option<Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) -> Result<Option<Arc<dyn crate::src::shaders::SkShader::SkShader>>, crate::RasterError> {
        if bg.is_none() || fg.is_none() {
            if !self.onAffectsTransparentBlack()? && bg.is_none() && fg.is_none() {
                return Ok(None);
            }
            if let Some(mode) = self.fBlender.asBlendMode() {
                if let Some((src, dst)) = SkBlendMode_AsCoeff(mode) {
                    if bg.is_some()
                        && (dst == SkBlendModeCoeff::kOne
                            || dst == SkBlendModeCoeff::kISA
                            || dst == SkBlendModeCoeff::kISC)
                    {
                        return Ok(bg);
                    }
                    if fg.is_some()
                        && (src == SkBlendModeCoeff::kOne || src == SkBlendModeCoeff::kIDA)
                    {
                        return Ok(fg);
                    }
                }
            }
            if bg.is_none() {
                bg = crate::src::shaders::SkColorShader::Color(0);
            }
            if fg.is_none() {
                fg = crate::src::shaders::SkColorShader::Color(0);
            }
        }
        Ok(crate::src::shaders::SkBlendShader::BlendWithBlender(
            self.fBlender.clone(),
            bg,
            fg,
        ))
    }
}

impl SkImageFilterBaseHooks for SkBlendImageFilter {
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:69-69
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:71-76
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok({
            self.fBlender.asBlendMode().is_none()
                && self.fArithmeticCoefficients.map_or(true, |k| k[3] != 0.0)
        })
    }
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:295-318
    unsafe fn onFilterImage(
        &self,
        ctx: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let mut required =
            self.onGetOutputLayerBounds(ctx.mapping(), Some(ctx.source().layerBounds()))?;
        if let Some(bounds) = required.as_mut() {
            if !bounds.intersect(ctx.desiredOutput()) {
                return Ok(crate::src::core::SkImageFilterTypes::FilterResult::default());
            }
        } else {
            required = Some(*ctx.desiredOutput());
        }
        let inputCtx = ctx.withNewDesiredOutput(required.unwrap());
        let mut builder = crate::src::core::SkImageFilterTypes::Builder::new(ctx);
        builder.add(self.getChildOutput(Self::kBackground, &inputCtx)?);
        builder.add(self.getChildOutput(Self::kForeground, &inputCtx)?);
        builder.eval(
            |inputs| {
                self.makeBlendShader(
                    inputs[Self::kBackground].clone(),
                    inputs[Self::kForeground].clone(),
                )
            },
            required,
        )
    }

    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:320-348
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            let mut required = *desired;
            if let Some(content) = content {
                if let Some(mut maximum) = self.onGetOutputLayerBounds(mapping, Some(content))? {
                    if !maximum.intersect(desired) {
                        return Ok(LayerSpace::<SkIRect>::Empty());
                    }
                    required = maximum;
                }
            }
            let mut bg = self.getChildInputLayerBounds(0, mapping, &required, content)?;
            let fg = self.getChildInputLayerBounds(1, mapping, &required, content)?;
            bg.join(&fg);
            bg
        })
    }
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:350-428
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            use SkBlendModeCoeff::*;
            let (mut outside_fg, mut outside_bg) = (false, false);
            if let Some(mode) = self.fBlender.asBlendMode() {
                crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlendImageFilter.cpp\0".as_ptr(),368; @condition b"*bm != SkBlendMode::kClear\0".as_ptr(); mode != SkBlendMode::kClear);
                if let Some((src, dst)) = SkBlendMode_AsCoeff(mode) {
                    outside_fg = dst == kZero || dst == kSA || dst == kSC;
                    outside_bg = src == kZero || src == kDA;
                }
            } else if let Some(k) = self.fArithmeticCoefficients {
                crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlendImageFilter.cpp\0".as_ptr(),384; @condition b"k != kClearCoeff\0".as_ptr(); k != [0.; 4]);
                if k[3] != 0.0 {
                    return Ok(LayerSpace::<SkIRect>::Unbounded());
                }
                outside_fg = k[2] == 0.;
                outside_bg = k[1] == 0.;
            } else {
                return Ok(LayerSpace::<SkIRect>::Unbounded());
            }
            let mut fg = self.getChildOutputLayerBounds(1, mapping, content)?;
            let mut bg = self.getChildOutputLayerBounds(0, mapping, content)?;
            if outside_fg {
                if outside_bg {
                    if fg.is_none() && bg.is_some() {
                        fg = bg;
                    } else if let (Some(f), Some(b)) = (&mut fg, bg) {
                        if !f.intersect(&b) {
                            return Ok(Some(LayerSpace::<SkIRect>::Empty()));
                        }
                    }
                }
                fg
            } else {
                if !outside_bg {
                    if let (Some(f), Some(b)) = (fg, &mut bg) {
                        b.join(&f);
                    } else {
                        bg = None;
                    }
                }
                bg
            }
        })
    }
}
impl SkImageFilter for SkBlendImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkBlendImageFilter.cpp:430-485
    fn computeFastBounds(&self, bounds: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({
            use SkBlendModeCoeff::*;
            let (mut outside_fg, mut outside_bg) = (false, false);
            if let Some(mode) = self.fBlender.asBlendMode() {
                crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlendImageFilter.cpp\0".as_ptr(),436; @condition b"*bm != SkBlendMode::kClear\0".as_ptr(); mode != SkBlendMode::kClear);
                if let Some((src, dst)) = SkBlendMode_AsCoeff(mode) {
                    outside_fg = dst == kZero || dst == kSA;
                    outside_bg = src == kZero || src == kDA;
                }
            } else if let Some(k) = self.fArithmeticCoefficients {
                crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlendImageFilter.cpp\0".as_ptr(),448; @condition b"k != kClearCoeff\0".as_ptr(); k != [0.; 4]);
                if k[3] != 0.0 {
                    return Ok(crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32());
                }
                outside_fg = k[2] == 0.;
                outside_bg = k[1] == 0.;
            } else {
                return Ok(crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32());
            }
            let mut fg = match self.getInput(1) {
                Some(child) => child.computeFastBounds(bounds)?,
                None => *bounds,
            };
            let mut bg = match self.getInput(0) {
                Some(child) => child.computeFastBounds(bounds)?,
                None => *bounds,
            };
            if outside_fg {
                if outside_bg && !fg.intersect(&bg) {
                    return Ok(SkRect::MakeEmpty());
                }
                fg
            } else {
                if !outside_bg {
                    bg.join(&fg);
                }
                bg
            }
        })
    }
}
