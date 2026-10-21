// Copyright 2013 Google Inc. BSD-style license. UNVERIFIED translation.
use crate::{
    include::core::{
        SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
        SkSize::SkSize,
        SkTileMode::SkTileMode,
    },
    src::{
        core::{
            SkColor::SkColorChannel,
            SkImageFilter::SkImageFilter,
            SkImageFilterTypes::{
                Builder, Context, FilterResult, LayerSpace, Mapping, ParameterSpace, ShaderFlags,
                Vector,
            },
            SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
            SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
            SkMatrix::SkMatrix,
            SkRect::{SkIRect, SkRect},
            SkRuntimeEffect::{ChildPtr, SkRuntimeEffectBuilder},
        },
        shaders::{SkColorShader, SkShader::SkShader},
    },
};
use std::sync::Arc;
type Filter = Option<Arc<dyn SkImageFilter>>;
type Shader = Option<Arc<dyn SkShader>>;
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:38-39
const kDisplacement: usize = 0;
const kColor: usize = 1;
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:43-43
const kDisplacementSampling: SkSamplingOptions =
    SkSamplingOptions::with_filter(SkFilterMode::kNearest);
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:36-36
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:87-91
// Native fields are contiguous in Rust; trivial channel/scale fields precede
// base so the base's retained child owners are destroyed last, as in C++.
struct SkDisplacementMapImageFilter {
    fXChannel: SkColorChannel,
    fYChannel: SkColorChannel,
    fScale: f32,
    base: SkImageFilter_Base,
}
impl SkDisplacementMapImageFilter {
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:46-56
    fn new(x: SkColorChannel, y: SkColorChannel, scale: f32, inputs: [Filter; 2]) -> Self {
        let result = Self {
            base: SkImageFilter_Base::new(&inputs, None),
            fXChannel: x,
            fYChannel: y,
            fScale: scale,
        };
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkDisplacementMapImageFilter.cpp\0".as_ptr(),54; @condition b"SkIsFinite(fScale)\0".as_ptr(); result.fScale.is_finite());
        result
    }
}
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:64-64
impl crate::src::core::SkFlattenable::SkFlattenableView for SkDisplacementMapImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkDisplacementMapImageFilter")
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

impl SkDisplacementMapImageFilter {
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:77-85
    fn outsetByMaxDisplacement(
        &self,
        mapping: &Mapping,
        mut bounds: LayerSpace<SkIRect>,
    ) -> LayerSpace<SkIRect> {
        let max: LayerSpace<SkSize> = mapping.paramToLayer(&ParameterSpace::new(SkSize::Make(
            0.5 * self.fScale,
            0.5 * self.fScale,
        )));
        bounds.outset(max.ceil());
        bounds
    }
}

// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:95-106
// Rust SkColorChannel admits exactly the four native valid channel values;
// invalid C++ integer-to-enum inputs are rejected by validated deserialization.
fn channel_selector_type_is_valid(c: SkColorChannel) -> bool {
    matches!(
        c,
        SkColorChannel::kR | SkColorChannel::kG | SkColorChannel::kB | SkColorChannel::kA
    )
}
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:108-143
unsafe fn make_displacement_shader(
    mut displacement: Shader,
    color: Shader,
    scale: LayerSpace<Vector>,
    x: SkColorChannel,
    y: SkColorChannel,
) -> Shader {
    if color.is_none() {
        return None;
    }
    if displacement.is_none() {
        displacement = SkColorShader::Color(crate::src::core::SkColor::SK_ColorTRANSPARENT);
    }
    let effect = GetKnownRuntimeEffect(StableKey::kDisplacement)
        .expect("native known displacement effect")
        .clone();
    let selector = |c| {
        [
            if c == SkColorChannel::kR { 1.0f32 } else { 0.0 },
            if c == SkColorChannel::kG { 1.0 } else { 0.0 },
            if c == SkColorChannel::kB { 1.0 } else { 0.0 },
            if c == SkColorChannel::kA { 1.0 } else { 0.0 },
        ]
    };
    let mut builder = SkRuntimeEffectBuilder::new(effect);
    builder.child(b"displMap").assign(
        displacement
            .map(ChildPtr::Shader)
            .unwrap_or(ChildPtr::Empty),
    );
    builder
        .child(b"colorMap")
        .assign(color.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
    builder.uniform(b"scale").assign(&[scale.x(), scale.y()]);
    builder.uniform(b"xSelect").assign(&selector(x));
    builder.uniform(b"ySelect").assign(&selector(y));
    builder.makeShader(None)
}
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:149-167
pub fn DisplacementMap(
    x: SkColorChannel,
    y: SkColorChannel,
    scale: f32,
    displacement: Filter,
    color: Filter,
    crop: Option<SkRect>,
) -> Filter {
    if !channel_selector_type_is_valid(x) || !channel_selector_type_is_valid(y) {
        return None;
    }
    if !scale.is_finite() {
        return None;
    }
    let inputs = [displacement, color];
    let mut filter: Filter = Some(Arc::new(SkDisplacementMapImageFilter::new(
        x, y, scale, inputs,
    )));
    if let Some(crop) = crop {
        filter = super::SkCropImageFilter::Crop(crop, SkTileMode::kDecal, filter);
    }
    filter
}
// Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:169-175
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterDisplacementMapImageFilterFlattenable() {
    unsafe {
        use crate::src::core::SkFlattenable::Register;
        Register(
            b"SkDisplacementMapImageFilter",
            SkDisplacementMapImageFilter::CreateProc,
        );
        Register(
            b"SkDisplacementMapEffect",
            SkDisplacementMapImageFilter::CreateProc,
        );
        Register(
            b"SkDisplacementMapEffectImpl",
            SkDisplacementMapImageFilter::CreateProc,
        );
    }
}
impl SkDisplacementMapImageFilter {
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:177-186
    fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        crate::src::core::SkImageFilter_Base::SK_IMAGEFILTER_UNFLATTEN_COMMON!(common, buffer, 2);
        fn channel(x: u32) -> SkColorChannel {
            match x {
                0 => SkColorChannel::kR,
                1 => SkColorChannel::kG,
                2 => SkColorChannel::kB,
                3 => SkColorChannel::kA,
                _ => unreachable!("native validated color channel"),
            }
        }
        let x = channel(buffer.read32LE(SkColorChannel::kLastEnum as u32));
        let y = channel(buffer.read32LE(SkColorChannel::kLastEnum as u32));
        let scale = buffer.readScalar();
        Ok(DisplacementMap(
            x,
            y,
            scale,
            common.getInput(0),
            common.getInput(1),
            common.cropRect(),
        )
        .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:188-193
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeInt(self.fXChannel as i32);
        buffer.writeInt(self.fYChannel as i32);
        buffer.writeScalar(self.fScale);
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkDisplacementMapImageFilter {
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:197-265
    unsafe fn onFilterImage(&self, ctx: &Context<'_>) -> Result<FilterResult, crate::RasterError> {
        let required = self.outsetByMaxDisplacement(ctx.mapping(), *ctx.desiredOutput());
        let color = self.getChildOutput(kColor, &ctx.withNewDesiredOutput(required))?;
        if !color.has_image() {
            return Ok(FilterResult::default());
        }
        let mut output = self.outsetByMaxDisplacement(ctx.mapping(), color.layerBounds());
        if !output.intersect(ctx.desiredOutput()) {
            return Ok(FilterResult::default());
        }
        let displacement = self.getChildOutput(
            kDisplacement,
            &ctx.withNewDesiredOutput(output).withNewColorSpace(None),
        )?;
        let scale: LayerSpace<Vector> = ctx
            .mapping()
            .paramToLayer(&ParameterSpace::new(Vector::new(self.fScale, self.fScale)));
        if !displacement.has_image() {
            let constant = LayerSpace::new(SkMatrix::Translate(-0.5 * scale.x(), -0.5 * scale.y()));
            return color.applyTransform(ctx, constant, kDisplacementSampling);
        }
        let mut builder = Builder::new(ctx);
        builder.add_sampled(
            displacement,
            Some(output),
            ShaderFlags::kNone,
            FilterResult::kDefaultSampling,
        );
        builder.add_sampled(
            color,
            Some(required),
            ShaderFlags::kNonTrivialSampling,
            kDisplacementSampling,
        );
        builder.eval(
            |inputs| {
                Ok(make_displacement_shader(
                    inputs[kDisplacement].clone(),
                    inputs[kColor].clone(),
                    scale,
                    self.fXChannel,
                    self.fYChannel,
                ))
            },
            Some(output),
        )
    }
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:267-281
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        let mut required = self.outsetByMaxDisplacement(mapping, *desired);
        required = self.getChildInputLayerBounds(kColor, mapping, &required, content)?;
        required.join(&self.getChildInputLayerBounds(kDisplacement, mapping, desired, content)?);
        Ok(required)
    }
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:283-292
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok(
            if let Some(color) = self.getChildOutputLayerBounds(kColor, mapping, content)? {
                Some(self.outsetByMaxDisplacement(mapping, color))
            } else {
                LayerSpace::<SkIRect>::Unbounded()
            },
        )
    }
}
impl SkImageFilter for SkDisplacementMapImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkDisplacementMapImageFilter.cpp:294-299
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        let bounds = if let Some(child) = self.getInput(kColor) {
            child.computeFastBounds(src)?
        } else {
            *src
        };
        let max = 0.5 * self.fScale.abs();
        Ok(bounds.makeOutset(max, max))
    }
}
