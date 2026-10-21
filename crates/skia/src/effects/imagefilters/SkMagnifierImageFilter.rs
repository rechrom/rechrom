// Copyright 2012 The Android Open Source Project. BSD-style license. UNVERIFIED.
use crate::{
    include::core::{SkSamplingOptions::SkSamplingOptions, SkSize::SkSize, SkTileMode::SkTileMode},
    src::{
        core::{
            SkImageFilter::SkImageFilter,
            SkImageFilterTypes::{
                Builder, Context, FilterResult, LayerSpace, Mapping, MatrixCapability,
                ParameterSpace, ShaderFlags,
            },
            SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
            SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
            SkMatrix::SkMatrix,
            SkRect::{SkIRect, SkRect},
            SkRuntimeEffect::{ChildPtr, SkRuntimeEffectBuilder},
        },
        shaders::SkShader::SkShader,
    },
};
use std::sync::Arc;
type Filter = Option<Arc<dyn SkImageFilter>>;
type Shader = Option<Arc<dyn SkShader>>;
// Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:38-38
// Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:71-77
struct SkMagnifierImageFilter {
    fLensBounds: ParameterSpace<SkRect>,
    fZoomAmount: f32,
    fInset: f32,
    fSampling: SkSamplingOptions,
    base: SkImageFilter_Base,
}
impl SkMagnifierImageFilter {
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:40-49
    fn new(
        lens: &SkRect,
        zoom: f32,
        inset: f32,
        sampling: &SkSamplingOptions,
        input: Filter,
    ) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[input], None),
            fLensBounds: ParameterSpace::new(*lens),
            fZoomAmount: zoom,
            fInset: inset,
            fSampling: *sampling,
        }
    }
}
// Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:58-58
impl crate::src::core::SkFlattenable::SkFlattenableView for SkMagnifierImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkMagnifierImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:82-108
pub fn Magnifier(
    lens: &SkRect,
    zoom: f32,
    inset: f32,
    sampling: &SkSamplingOptions,
    mut input: Filter,
    crop: Option<SkRect>,
) -> Filter {
    if lens.isEmpty()
        || !lens.isFinite()
        || zoom <= 0.0
        || inset < 0.0
        || !zoom.is_finite()
        || !inset.is_finite()
    {
        return None;
    }
    if let Some(crop) = crop {
        input = super::SkCropImageFilter::Crop(crop, SkTileMode::kDecal, input);
    }
    if zoom > 1.0 {
        Some(Arc::new(SkMagnifierImageFilter::new(
            lens, zoom, inset, sampling, input,
        )))
    } else {
        input
    }
}
// Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:110-112
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterMagnifierImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkMagnifierImageFilter",
            SkMagnifierImageFilter::CreateProc,
        );
    }
}
impl SkMagnifierImageFilter {
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:114-130
    fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        if buffer.isVersionLT(crate::src::core::SkPicturePriv::kRevampMagnifierFilter) {
            return Ok(None);
        }
        crate::src::core::SkImageFilter_Base::SK_IMAGEFILTER_UNFLATTEN_COMMON!(common, buffer, 1);
        let lens = buffer.readRect();
        let zoom = buffer.readScalar();
        let inset = buffer.readScalar();
        let sampling = buffer.readSampling();
        Ok(
            Magnifier(&lens, zoom, inset, &sampling, common.getInput(0), None)
                .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter),
        )
    }
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:132-138
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeRect(self.fLensBounds.geometry());
        buffer.writeScalar(self.fZoomAmount);
        buffer.writeScalar(self.fInset);
        buffer.writeSampling(&self.fSampling);
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:142-161
unsafe fn make_magnifier_shader(
    input: Shader,
    lens: &LayerSpace<SkRect>,
    zoom: &LayerSpace<SkMatrix>,
    inset: &LayerSpace<SkSize>,
) -> Shader {
    let effect = GetKnownRuntimeEffect(StableKey::kMagnifier)
        .expect("native known magnifier effect")
        .clone();
    let mut builder = SkRuntimeEffectBuilder::new(effect);
    builder
        .child(b"src")
        .assign(input.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
    crate::SkASSERT!(@source b"src/effects/imagefilters/SkMagnifierImageFilter.cpp\0".as_ptr(),153; @condition b"inset.width() > 0.f && inset.height() > 0.f\0".as_ptr(); inset.width()>0.0&&inset.height()>0.0);
    builder.uniform(b"lensBounds").assign(lens.geometry());
    builder.uniform(b"zoomXform").assign(&[
        zoom.rc(0, 2),
        zoom.rc(1, 2),
        zoom.rc(0, 0),
        zoom.rc(1, 1),
    ]);
    builder
        .uniform(b"invInset")
        .assign(&[1.0f32 / inset.width(), 1.0f32 / inset.height()]);
    builder.makeShader(None)
}
impl SkImageFilterBaseHooks for SkMagnifierImageFilter {
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:165-277
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:60-60
    unsafe fn onFilterImage(&self, ctx: &Context<'_>) -> Result<FilterResult, crate::RasterError> {
        let lens: LayerSpace<SkRect> = ctx.mapping().paramToLayer(&self.fLensBounds);
        let mut center = lens.center();
        let mut visible = lens;
        if !visible.intersect(&LayerSpace::<SkRect>::from_irect(*ctx.desiredOutput())) {
            return Ok(FilterResult::default());
        }
        let mut expected = lens;
        if let Some(output) =
            self.getChildOutputLayerBounds(0, ctx.mapping(), Some(ctx.source().layerBounds()))?
        {
            expected = LayerSpace::<SkRect>::from_irect(output);
        }
        center = expected.clamp(center);
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMagnifierImageFilter.cpp\0".as_ptr(),195; @condition b"this->getCTMCapability() == MatrixCapability::kScaleTranslate\0".as_ptr(); self.getCTMCapability()==MatrixCapability::kScaleTranslate);
        // std::max/min preserve their first operand for unordered comparisons.
        let maxDimension = if lens.width() < lens.height() {
            lens.height()
        } else {
            lens.width()
        };
        let maxLens = if 1.0 < maxDimension {
            maxDimension
        } else {
            1.0
        };
        let maxZoom = 2.0 * maxLens;
        let invZoom = 1.0
            / if maxZoom < self.fZoomAmount {
                maxZoom
            } else {
                self.fZoomAmount
            };
        let mut src = LayerSpace::new(SkRect::MakeLTRB(
            lens.left() * invZoom + center.x() * (1.0 - invZoom),
            lens.top() * invZoom + center.y() * (1.0 - invZoom),
            lens.right() * invZoom + center.x() * (1.0 - invZoom),
            lens.bottom() * invZoom + center.y() * (1.0 - invZoom),
        ));
        let mut zoom = LayerSpace::<SkMatrix>::RectToRect(&lens, &src);
        if !expected.contains(&visible) {
            src = zoom.mapRect(&visible);
            if expected.width() >= src.width() && expected.height() >= src.height() {
                let right = if expected.right() < src.right() {
                    expected.right()
                } else {
                    src.right()
                };
                let bottom = if expected.bottom() < src.bottom() {
                    expected.bottom()
                } else {
                    src.bottom()
                };
                let left = if src.left() < expected.left() {
                    expected.left()
                } else {
                    right - src.width()
                };
                let top = if src.top() < expected.top() {
                    expected.top()
                } else {
                    bottom - src.height()
                };
                src = LayerSpace::new(SkRect::MakeXYWH(left, top, src.width(), src.height()));
                zoom = LayerSpace::<SkMatrix>::RectToRect(&visible, &src);
            }
        }
        let inset: LayerSpace<SkSize> = ctx
            .mapping()
            .paramToLayer(&ParameterSpace::new(SkSize::Make(self.fInset, self.fInset)));
        if inset.width() <= 0.0 || inset.height() <= 0.0 {
            let mut inverse = LayerSpace::new(SkMatrix::default());
            if !zoom.invert(Some(&mut inverse)) {
                return Ok(FilterResult::default());
            }
            let child = self.getChildOutput(0, &ctx.withNewDesiredOutput(src.roundOut()))?;
            return child
                .applyTransform(ctx, inverse, self.fSampling)?
                .applyCrop(ctx, lens.roundOut(), SkTileMode::kDecal);
        }
        let mut builder = Builder::new(ctx);
        let actual = self.getChildOutput(0, &ctx.withNewDesiredOutput(expected.roundOut()))?;
        builder.add_sampled(
            actual,
            None,
            ShaderFlags::kNonTrivialSampling,
            self.fSampling,
        );
        builder.eval(
            |inputs| {
                Ok(if inputs[0].is_some() {
                    make_magnifier_shader(inputs[0].clone(), &lens, &zoom, &inset)
                } else {
                    None
                })
            },
            Some(lens.roundOut()),
        )
    }
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:279-292
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:62-65
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        _desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        let lens: LayerSpace<SkRect> = mapping.paramToLayer(&self.fLensBounds);
        self.getChildInputLayerBounds(0, mapping, &lens.roundOut(), content)
    }
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:294-306
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:67-69
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        let output = self.getChildOutputLayerBounds(0, mapping, content)?;
        let lens: LayerSpace<SkRect> = mapping.paramToLayer(&self.fLensBounds);
        let mut lens = lens.roundOut();
        if output.is_none() || lens.intersect(&output.unwrap()) {
            Ok(Some(lens))
        } else {
            Ok(Some(LayerSpace::<SkIRect>::Empty()))
        }
    }
}
impl SkImageFilter for SkMagnifierImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkMagnifierImageFilter.cpp:308-315
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        let mut bounds = if let Some(child) = self.getInput(0) {
            child.computeFastBounds(src)?
        } else {
            *src
        };
        if bounds.intersect(self.fLensBounds.geometry()) {
            Ok(bounds)
        } else {
            Ok(SkRect::MakeEmpty())
        }
    }
}
