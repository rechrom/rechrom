// Copyright 2012 The Android Open Source Project. BSD-style license. UNVERIFIED.
use crate::{
    include::core::{
        SkSize::{SkISize, SkSize},
        SkTileMode::SkTileMode,
    },
    src::{
        core::{
            SkImageFilter::SkImageFilter,
            SkImageFilterTypes::{
                Builder, Context, FilterResult, LayerSpace, Mapping, ParameterSpace, ShaderFlags,
            },
            SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
            SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
            SkRect::{SkIRect, SkRect},
            SkRuntimeEffect::{ChildPtr, SkRuntimeEffectBuilder},
        },
        shaders::SkShader::SkShader,
    },
};
use std::sync::Arc;
type Filter = Option<Arc<dyn SkImageFilter>>;
type Shader = Option<Arc<dyn SkShader>>;
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:34-38
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum MorphType {
    kErode,
    kDilate,
}
impl MorphType {
    const kLastType: Self = Self::kDilate;
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:40-40
#[derive(Clone, Copy, PartialEq, Eq)]
enum MorphDirection {
    kX,
    kY,
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:42-42
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:102-103
// Rust contiguous field group. Native trivial fields precede the base so its
// actual retained input owners are released last, preserving C++ destruction.
struct SkMorphologyImageFilter {
    fType: MorphType,
    fRadii: ParameterSpace<SkSize>,
    base: SkImageFilter_Base,
}
impl SkMorphologyImageFilter {
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:44-47
    fn new(kind: MorphType, radii: SkSize, input: Filter) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[input], None),
            fType: kind,
            fRadii: ParameterSpace::new(radii),
        }
    }
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:57-57
impl crate::src::core::SkFlattenable::SkFlattenableView for SkMorphologyImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkMorphologyImageFilter")
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

impl SkMorphologyImageFilter {
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:70-78
    fn radii(&self, mapping: &Mapping) -> LayerSpace<SkISize> {
        let radii: LayerSpace<SkSize> = mapping.paramToLayer(&self.fRadii);
        let radii = radii.round();
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMorphologyImageFilter.cpp\0".as_ptr(),72; @condition b"radii.width() >= 0 && radii.height() >= 0\0".as_ptr(); radii.width()>=0&&radii.height()>=0);
        const kMaxRadii: i32 = 256;
        LayerSpace::new(SkISize::Make(
            radii.width().min(kMaxRadii),
            radii.height().min(kMaxRadii),
        ))
    }
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:80-85
    fn requiredInput(
        &self,
        mapping: &Mapping,
        mut bounds: LayerSpace<SkIRect>,
    ) -> LayerSpace<SkIRect> {
        bounds.outset(self.radii(mapping));
        bounds
    }
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:87-100
    fn kernelOutputBounds(
        &self,
        mapping: &Mapping,
        mut bounds: LayerSpace<SkIRect>,
    ) -> LayerSpace<SkIRect> {
        let radii = self.radii(mapping);
        if self.fType == MorphType::kDilate {
            bounds.outset(radii);
        } else {
            bounds.inset(radii);
        }
        bounds
    }
}

// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:106-124
fn make_morphology(kind: MorphType, radii: SkSize, input: Filter, crop: Option<SkRect>) -> Filter {
    if radii.width() < 0.0 || radii.height() < 0.0 {
        return None;
    }
    let mut filter = input;
    if radii.width() > 0.0 || radii.height() > 0.0 {
        filter = Some(Arc::new(SkMorphologyImageFilter::new(kind, radii, filter)));
    }
    if let Some(crop) = crop {
        filter = super::SkCropImageFilter::Crop(crop, SkTileMode::kDecal, filter);
    }
    filter
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:131-131
const kMaxLinearRadius: i32 = 14;
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:132-148
unsafe fn make_linear_morphology(
    input: Shader,
    kind: MorphType,
    direction: MorphDirection,
    radius: i32,
) -> Shader {
    crate::SkASSERT!(@source b"src/effects/imagefilters/SkMorphologyImageFilter.cpp\0".as_ptr(),136; @condition b"radius <= kMaxLinearRadius\0".as_ptr(); radius<=kMaxLinearRadius);
    let effect = GetKnownRuntimeEffect(StableKey::kLinearMorphology)
        .expect("native known linear morphology effect")
        .clone();
    let mut builder = SkRuntimeEffectBuilder::new(effect);
    builder
        .child(b"child")
        .assign(input.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
    builder
        .uniform(b"offset")
        .assign(&if direction == MorphDirection::kX {
            [1.0f32, 0.0]
        } else {
            [0.0f32, 1.0]
        });
    builder
        .uniform(b"flip")
        .assign(&if kind == MorphType::kDilate {
            1.0f32
        } else {
            -1.0f32
        });
    builder.uniform(b"radius").assign(&radius);
    builder.makeShader(None)
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:154-169
unsafe fn make_sparse_morphology(
    input: Shader,
    kind: MorphType,
    direction: MorphDirection,
    radius: i32,
) -> Shader {
    let effect = GetKnownRuntimeEffect(StableKey::kSparseMorphology)
        .expect("native known sparse morphology effect")
        .clone();
    let mut builder = SkRuntimeEffectBuilder::new(effect);
    builder
        .child(b"child")
        .assign(input.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
    builder
        .uniform(b"offset")
        .assign(&if direction == MorphDirection::kX {
            [radius as f32, 0.0f32]
        } else {
            [0.0f32, radius as f32]
        });
    builder
        .uniform(b"flip")
        .assign(&if kind == MorphType::kDilate {
            1.0f32
        } else {
            -1.0f32
        });
    builder.makeShader(None)
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:171-223
unsafe fn morphology_pass(
    ctx: &Context<'_>,
    input: &FilterResult,
    kind: MorphType,
    dir: MorphDirection,
    radius: i32,
) -> Result<FilterResult, crate::RasterError> {
    let axisDelta = |step| {
        LayerSpace::new(SkISize::Make(
            if dir == MorphDirection::kX { step } else { 0 },
            if dir == MorphDirection::kY { step } else { 0 },
        ))
    };
    let mut sampleBounds = *ctx.desiredOutput();
    sampleBounds.outset(axisDelta(radius));
    let mut childOutput = input.clone();
    let mut appliedRadius = 0;
    while radius > appliedRadius {
        if !childOutput.has_image() {
            return Ok(FilterResult::default());
        }
        let stepRadius = if appliedRadius == 0 {
            kMaxLinearRadius.min(radius)
        } else {
            (radius - appliedRadius).min(appliedRadius)
        };
        let mut stepCtx = ctx.clone();
        if appliedRadius + stepRadius < radius {
            let mut outputBounds = sampleBounds;
            outputBounds.inset(axisDelta(stepRadius));
            stepCtx = ctx.withNewDesiredOutput(outputBounds);
        }
        let mut builder = Builder::new(&stepCtx);
        builder.add_sampled(
            childOutput,
            Some(sampleBounds),
            ShaderFlags::kSampledRepeatedly,
            FilterResult::kDefaultSampling,
        );
        childOutput = builder.eval(
            |inputs| {
                Ok(if appliedRadius == 0 {
                    make_linear_morphology(inputs[0].clone(), kind, dir, stepRadius)
                } else {
                    make_sparse_morphology(inputs[0].clone(), kind, dir, stepRadius)
                })
            },
            None,
        )?;
        sampleBounds = *stepCtx.desiredOutput();
        appliedRadius += stepRadius;
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMorphologyImageFilter.cpp\0".as_ptr(),219; @condition b"appliedRadius <= radius\0".as_ptr(); appliedRadius<=radius);
    }
    Ok(childOutput)
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:227-231
pub fn Dilate(x: f32, y: f32, input: Filter, crop: Option<SkRect>) -> Filter {
    make_morphology(MorphType::kDilate, SkSize::Make(x, y), input, crop)
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:233-237
pub fn Erode(x: f32, y: f32, input: Filter, crop: Option<SkRect>) -> Filter {
    make_morphology(MorphType::kErode, SkSize::Make(x, y), input, crop)
}
// Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:239-243
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterMorphologyImageFilterFlattenables() {
    unsafe {
        use crate::src::core::SkFlattenable::Register;
        Register(
            b"SkMorphologyImageFilter",
            SkMorphologyImageFilter::CreateProc,
        );
        Register(
            b"SkMorphologyImageFilterImpl",
            SkMorphologyImageFilter::CreateProc,
        );
    }
}
impl SkMorphologyImageFilter {
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:245-259
    fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        crate::src::core::SkImageFilter_Base::SK_IMAGEFILTER_UNFLATTEN_COMMON!(common, buffer, 1);
        let width = buffer.readScalar();
        let height = buffer.readScalar();
        let kind = buffer.read32LE(MorphType::kLastType as u32);
        let filter = if kind == MorphType::kDilate as u32 {
            Dilate(width, height, common.getInput(0), common.cropRect())
        } else if kind == MorphType::kErode as u32 {
            Erode(width, height, common.getInput(0), common.cropRect())
        } else {
            None
        };
        Ok(filter.map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:261-266
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeScalar(self.fRadii.geometry().width());
        buffer.writeScalar(self.fRadii.geometry().height());
        buffer.writeInt(self.fType as i32);
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkMorphologyImageFilter {
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:270-294
    unsafe fn onFilterImage(&self, ctx: &Context<'_>) -> Result<FilterResult, crate::RasterError> {
        let required = self.requiredInput(ctx.mapping(), *ctx.desiredOutput());
        let mut childOutput = self.getChildOutput(0, &ctx.withNewDesiredOutput(required))?;
        let mut maxOutput = self.kernelOutputBounds(ctx.mapping(), childOutput.layerBounds());
        if !maxOutput.intersect(ctx.desiredOutput()) {
            return Ok(FilterResult::default());
        }
        let radii = self.radii(ctx.mapping());
        let mut maxOutputX = maxOutput;
        maxOutputX.outset(LayerSpace::new(SkISize::Make(0, radii.height())));
        childOutput = morphology_pass(
            &ctx.withNewDesiredOutput(maxOutputX),
            &childOutput,
            self.fType,
            MorphDirection::kX,
            radii.width(),
        )?;
        childOutput = morphology_pass(
            &ctx.withNewDesiredOutput(maxOutput),
            &childOutput,
            self.fType,
            MorphDirection::kY,
            radii.height(),
        )?;
        Ok(childOutput)
    }
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:296-302
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        let required = self.requiredInput(mapping, *desired);
        self.getChildInputLayerBounds(0, mapping, &required, content)
    }
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:304-313
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok(
            if let Some(child) = self.getChildOutputLayerBounds(0, mapping, content)? {
                Some(self.kernelOutputBounds(mapping, child))
            } else {
                LayerSpace::<SkIRect>::Unbounded()
            },
        )
    }
}
impl SkImageFilter for SkMorphologyImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkMorphologyImageFilter.cpp:315-324
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        let mut bounds = if let Some(child) = self.getInput(0) {
            child.computeFastBounds(src)?
        } else {
            *src
        };
        let radii = self.fRadii.geometry();
        if self.fType == MorphType::kDilate {
            bounds.outset(radii.width(), radii.height());
        } else {
            bounds.inset(radii.width(), radii.height());
        }
        Ok(bounds)
    }
}
