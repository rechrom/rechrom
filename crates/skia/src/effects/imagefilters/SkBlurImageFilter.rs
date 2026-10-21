#![allow(non_snake_case)]
use crate::include::core::{SkSize::SkSize, SkTileMode::SkTileMode};
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, ParameterSpace},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkRect::{SkIRect, SkRect},
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkBlurImageFilter.cpp:71-75
struct SkBlurImageFilter {
    fSigma: ParameterSpace<SkSize>,
    fLegacyTileMode: SkTileMode,
    base: SkImageFilter_Base,
}
impl SkBlurImageFilter {
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:33-35
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:75-75
    fn new(sigma: SkSize, input: Option<Arc<dyn SkImageFilter>>) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[input], None),
            fSigma: ParameterSpace::new(sigma),
            fLegacyTileMode: SkTileMode::kDecal,
        }
    }
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:37-40
    fn new_legacy(sigma: SkSize, tile: SkTileMode, input: Option<Arc<dyn SkImageFilter>>) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[input], None),
            fSigma: ParameterSpace::new(sigma),
            fLegacyTileMode: tile,
        }
    }
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:64-69
    fn kernelBounds(
        &self,
        mapping: &Mapping,
        mut bounds: LayerSpace<SkIRect>,
    ) -> LayerSpace<SkIRect> {
        let sigma = self.mapSigma(mapping);
        bounds.outset(
            LayerSpace::new(SkSize::Make(3.0 * sigma.width(), 3.0 * sigma.height())).ceil(),
        );
        bounds
    }
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:198-215
    fn mapSigma(&self, mapping: &Mapping) -> LayerSpace<SkSize> {
        let sigma = mapping.paramToLayer(&self.fSigma);
        // C++ std::min retains its first operand on unordered comparison, unlike f32::min.
        let mut x = if kMaxSigma < sigma.width() {
            kMaxSigma
        } else {
            sigma.width()
        };
        let mut y = if kMaxSigma < sigma.height() {
            kMaxSigma
        } else {
            sigma.height()
        };
        if !x.is_finite() || crate::src::core::SkBlurEngine::IsEffectivelyIdentity(x) {
            x = 0.0;
        }
        if !y.is_finite() || crate::src::core::SkBlurEngine::IsEffectivelyIdentity(y) {
            y = 0.0;
        }
        LayerSpace::new(SkSize::Make(x, y))
    }
}
// Source: src/effects/imagefilters/SkBlurImageFilter.cpp:151-151
const kMaxSigma: f32 = 532.0;
// Source: src/effects/imagefilters/SkBlurImageFilter.cpp:80-110
pub fn Blur(
    x: f32,
    y: f32,
    tile: SkTileMode,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    if tile != SkTileMode::kDecal && crop.is_none() {
        return Some(Arc::new(SkBlurImageFilter::new_legacy(
            SkSize::Make(x, y),
            tile,
            input,
        )));
    }
    let mut filter = input;
    if tile != SkTileMode::kDecal {
        if let Some(rect) = crop {
            filter = super::SkCropImageFilter::Crop(rect, tile, filter);
        }
    }
    filter = Some(Arc::new(SkBlurImageFilter::new(SkSize::Make(x, y), filter)));
    if let Some(rect) = crop {
        filter = super::SkCropImageFilter::Crop(rect, SkTileMode::kDecal, filter);
    }
    filter
}
impl SkImageFilterBaseHooks for SkBlurImageFilter {
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:155-196
    unsafe fn onFilterImage(
        &self,
        ctx: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let inputCtx =
            ctx.withNewDesiredOutput(self.kernelBounds(ctx.mapping(), *ctx.desiredOutput()));
        let mut child = self.getChildOutput(0, &inputCtx)?;
        let sigma = self.mapSigma(ctx.mapping());
        if sigma.width() == 0.0 && sigma.height() == 0.0 {
            return Ok(child);
        }
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkBlurImageFilter.cpp\0".as_ptr(),166; @condition b"sigma.width() >= 0.f && sigma.width() <= kMaxSigma && sigma.height() >= 0.f && sigma.height() <= kMaxSigma\0".as_ptr(); sigma.width()>=0.0 && sigma.width()<=kMaxSigma && sigma.height()>=0.0 && sigma.height()<=kMaxSigma);
        let mut maxOutput = *ctx.desiredOutput();
        if self.fLegacyTileMode != SkTileMode::kDecal {
            maxOutput = self.kernelBounds(ctx.mapping(), child.layerBounds());
            if !maxOutput.intersect(ctx.desiredOutput()) {
                return Ok(crate::src::core::SkImageFilterTypes::FilterResult::default());
            }
        }
        if self.fLegacyTileMode != SkTileMode::kDecal {
            child = child.applyCrop(&inputCtx, child.layerBounds(), self.fLegacyTileMode)?;
        }
        let cropped = ctx.withNewDesiredOutput(maxOutput);
        let mut builder = crate::src::core::SkImageFilterTypes::Builder::new(&cropped);
        builder.add(child);
        builder.blur(sigma)
    }

    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:217-224
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            let required = self.kernelBounds(mapping, *desired);
            self.getChildInputLayerBounds(0, mapping, &required, content)?
        })
    }
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:226-235
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            self.getChildOutputLayerBounds(0, mapping, content)?
                .map(|bounds| self.kernelBounds(mapping, bounds))
        })
    }
}
impl SkImageFilter for SkBlurImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:237-241
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({
            let mut bounds = match self.getInput(0) {
                Some(child) => child.computeFastBounds(src)?,
                None => *src,
            };
            bounds.outset(
                self.fSigma.geometry().fWidth * 3.0,
                self.fSigma.geometry().fHeight * 3.0,
            );
            bounds
        })
    }
}

// Source: src/effects/imagefilters/SkBlurImageFilter.cpp:49-49
impl crate::src::core::SkFlattenable::SkFlattenableView for SkBlurImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkBlurImageFilter")
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
// Source: src/effects/imagefilters/SkBlurImageFilter.cpp:113-116
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterBlurImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkBlurImageFilter",
            SkBlurImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkBlurImageFilterImpl",
            SkBlurImageFilter::CreateProc,
        );
    }
}
impl SkBlurImageFilter {
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:118-133
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        crate::src::core::SkImageFilter_Base::SK_IMAGEFILTER_UNFLATTEN_COMMON!(common, buffer, 1);
        let sigma_x = buffer.readScalar();
        let sigma_y = buffer.readScalar();
        let tile = match buffer.read32LE(SkTileMode::kLastTileMode as u32) {
            0 => SkTileMode::kClamp,
            1 => SkTileMode::kRepeat,
            2 => SkTileMode::kMirror,
            3 => SkTileMode::kDecal,
            _ => unreachable!("native validated tile mode"),
        };
        Ok(Blur(
            sigma_x,
            sigma_y,
            tile,
            common.getInput(0),
            common.cropRect(),
        )
        .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkBlurImageFilter.cpp:135-141
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeScalar(self.fSigma.geometry().fWidth);
        buffer.writeScalar(self.fSigma.geometry().fHeight);
        buffer.writeInt(self.fLegacyTileMode as i32);

        Ok(())
    }
}
