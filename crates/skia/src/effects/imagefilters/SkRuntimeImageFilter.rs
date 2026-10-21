// Copyright 2021 Google LLC. BSD-style license. UNVERIFIED native translation.
use crate::src::core::SkSpinlock::{SkAutoSpinlock, SkSpinlock};
use crate::{
    include::core::SkSize::SkSize,
    src::core::{
        SkImageFilter::{compute_fast_bounds_base, SkImageFilter},
        SkImageFilterTypes::{
            Builder, Context, FilterResult, LayerSpace, Mapping, MatrixCapability, ParameterSpace,
            ShaderFlags,
        },
        SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
        SkRect::{SkIRect, SkRect},
        SkRectPriv::SkRectPriv,
        SkRuntimeEffect::{
            ChildPtr, ChildType, SkMakeCachedRuntimeEffect, SkRuntimeEffect, SkRuntimeEffectBuilder,
        },
    },
};
use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use std::sync::Arc;
type Filter = Option<Arc<dyn SkImageFilter>>;
// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:41-41
// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:95-99
// Native spinlock protects the actual mutable builder; UnsafeCell only maps
// native mutable storage. Child names and builder retain native release order.
pub struct SkRuntimeImageFilter {
    fRuntimeEffectLock: SkSpinlock,
    fRuntimeEffectBuilder: ManuallyDrop<UnsafeCell<SkRuntimeEffectBuilder>>,
    fChildShaderNames: ManuallyDrop<Vec<Vec<u8>>>,
    fMaxSampleRadius: f32,
    fRestrictOutputToInputBounds: bool,
    base: SkImageFilter_Base,
}
impl Drop for SkRuntimeImageFilter {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fChildShaderNames);
            ManuallyDrop::drop(&mut self.fRuntimeEffectBuilder);
        }
    }
}
impl SkRuntimeImageFilter {
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:43-58
    fn new(
        builder: &SkRuntimeEffectBuilder,
        radius: f32,
        names: &[&[u8]],
        inputs: &[Filter],
        restrict: bool,
    ) -> Self {
        let mut result = Self {
            base: SkImageFilter_Base::new(inputs, None),
            fRuntimeEffectBuilder: ManuallyDrop::new(UnsafeCell::new(builder.clone())),
            fRuntimeEffectLock: SkSpinlock::new(),
            fMaxSampleRadius: radius,
            fRestrictOutputToInputBounds: restrict,
            fChildShaderNames: ManuallyDrop::new(Vec::new()),
        };
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkRuntimeImageFilter.cpp\0".as_ptr(),53; @condition b"maxSampleRadius >= 0.f\0".as_ptr(); radius>=0.0);
        result.fChildShaderNames.reserve_exact(inputs.len());
        for name in names {
            result.fChildShaderNames.push(name.to_vec());
        }
        result
    }
}
// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:67-67
impl crate::src::core::SkFlattenable::SkFlattenableView for SkRuntimeImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkRuntimeImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)
    }
}
impl SkRuntimeImageFilter {
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:69-69
    fn header_onAffectsTransparentBlack(&self) -> bool {
        !self.fRestrictOutputToInputBounds
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:73-73
    fn header_onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kTranslate
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:86-93
    fn applyMaxSampleRadius(
        &self,
        mapping: &Mapping,
        mut bounds: LayerSpace<SkIRect>,
    ) -> LayerSpace<SkIRect> {
        let radius: LayerSpace<SkSize> = mapping.paramToLayer(&ParameterSpace::new(SkSize::Make(
            self.fMaxSampleRadius,
            self.fMaxSampleRadius,
        )));
        bounds.outset(radius.ceil());
        bounds
    }
}
// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:102-119
pub fn RuntimeShader(
    builder: &SkRuntimeEffectBuilder,
    radius: f32,
    name: &[u8],
    input: Filter,
    restrict: bool,
) -> Filter {
    if name.is_empty() {
        let children = builder.effect().children();
        if children.len() != 1 {
            return None;
        }
        return RuntimeShaderInputs(builder, radius, &[children[0].name], &[input], restrict);
    }
    RuntimeShaderInputs(builder, radius, &[name], &[input], restrict)
}
// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:121-153
// Slice length projects the native explicit inputCount; arrays describe exactly
// the same count of native input and child-name slots.
pub fn RuntimeShaderInputs(
    builder: &SkRuntimeEffectBuilder,
    radius: f32,
    names: &[&[u8]],
    inputs: &[Filter],
    restrict: bool,
) -> Filter {
    debug_assert!(names.len() == inputs.len());
    if radius < 0.0 {
        return None;
    }
    let child_is_shader = |child: Option<&crate::src::core::SkRuntimeEffect::Child<'_>>| {
        child.is_some_and(|child| child.r#type == ChildType::kShader)
    };
    for i in 0..inputs.len() {
        let name = names[i];
        if name.is_empty() || !child_is_shader(builder.effect().findChild(name)) {
            return None;
        }
        for j in 0..i {
            if name == names[j] {
                return None;
            }
        }
    }
    Some(Arc::new(SkRuntimeImageFilter::new(
        builder, radius, names, inputs, restrict,
    )))
}
// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:155-157
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterRuntimeImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkRuntimeImageFilter",
            SkRuntimeImageFilter::CreateProc,
        );
    }
}
impl SkRuntimeImageFilter {
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:159-232
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        crate::src::core::SkImageFilter_Base::SK_IMAGEFILTER_UNFLATTEN_COMMON!(common, buffer, -1);
        if common.cropRect().is_some() {
            return Ok(None);
        }
        let allowed = buffer.allowSkSL();
        if !buffer.validate(allowed) {
            return Ok(None);
        }
        let mut sksl = Vec::new();
        buffer.readString(&mut sksl);
        let effect = SkMakeCachedRuntimeEffect(SkRuntimeEffect::MakeForShader, sksl);
        if !buffer.validate(effect.is_some()) {
            return Ok(None);
        }
        let effect = effect.unwrap();
        let uniforms = buffer.readByteArrayAsData();
        if !buffer.validate(
            uniforms
                .as_ref()
                .is_some_and(|u| u.size() == effect.uniformSize()),
        ) {
            return Ok(None);
        }
        let uniforms = uniforms.unwrap();
        let mut strings = vec![Vec::new(); common.inputCount()];
        for name in &mut strings {
            buffer.readString(name);
        }
        let names: Vec<&[u8]> = strings
            .iter()
            .map(|s| &s[..s.iter().position(|c| *c == 0).unwrap_or(s.len())])
            .collect();
        let mut builder = SkRuntimeEffectBuilder::with_uniforms(effect, uniforms);
        // Metadata remains owned by the same immutable effect; byte copies only end
        // the Rust shared borrow before mutating the builder's actual child slots.
        let children: Vec<_> = builder
            .effect()
            .children()
            .iter()
            .map(|c| (c.name.to_vec(), c.r#type))
            .collect();
        for (name, kind) in children {
            let child = match kind {
                ChildType::kBlender => ChildPtr::Blender(buffer.readBlender()?),
                ChildType::kColorFilter => buffer
                    .readColorFilter()?
                    .map(ChildPtr::ColorFilter)
                    .unwrap_or(ChildPtr::Empty),
                ChildType::kShader => buffer
                    .readShader()?
                    .map(ChildPtr::Shader)
                    .unwrap_or(ChildPtr::Empty),
            };
            builder.child(&name).assign(child);
        }
        let mut radius = 0.0;
        if !buffer.isVersionLT(crate::src::core::SkPicturePriv::kRuntimeImageFilterSampleRadius) {
            radius = buffer.readScalar();
        }
        let mut restrict = false;
        if !buffer.isVersionLT(crate::src::core::SkPicturePriv::kRuntimeImageFilterRestrictedOutput)
        {
            restrict = buffer.readBool();
        }
        if !buffer.isValid() {
            return Ok(None);
        }
        Ok(
            RuntimeShaderInputs(&builder, radius, &names, common.inputs(), restrict)
                .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter),
        )
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:234-249
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        {
            let _lock = SkAutoSpinlock::new(&self.fRuntimeEffectLock);
            let builder = unsafe { &*self.fRuntimeEffectBuilder.get() };
            fn cstr(v: &[u8]) -> &[u8] {
                &v[..v.iter().position(|c| *c == 0).unwrap_or(v.len())]
            }
            buffer.writeStringBytes(cstr(builder.effect().source()));
            buffer.writeDataAsByteArray(Some(&builder.uniforms()));
            for name in self.fChildShaderNames.iter() {
                buffer.writeStringBytes(cstr(name));
            }
            for child in builder.children() {
                buffer.writeFlattenable(child.flattenable())?;
            }
        }
        buffer.writeScalar(self.fMaxSampleRadius);
        buffer.writeBool(self.fRestrictOutputToInputBounds);
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkRuntimeImageFilter {
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok(self.header_onAffectsTransparentBlack())
    }
    fn onGetCTMCapability(&self) -> MatrixCapability {
        self.header_onGetCTMCapability()
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:253-297
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:75-75
    unsafe fn onFilterImage(&self, ctx: &Context<'_>) -> Result<FilterResult, crate::RasterError> {
        let count = self.countInputs();
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkRuntimeImageFilter.cpp\0".as_ptr(),257; @condition b"inputCount == fChildShaderNames.size()\0".as_ptr(); count==self.fChildShaderNames.len());
        let inputCtx = ctx
            .withNewDesiredOutput(self.applyMaxSampleRadius(ctx.mapping(), *ctx.desiredOutput()));
        let mut builder = Builder::new(ctx);
        let mut actual = LayerSpace::<SkIRect>::Empty();
        for i in 0..count {
            let child = self.getChildOutput(i, &inputCtx)?;
            actual.join(&child.layerBounds());
            builder.add_sampled(
                child,
                Some(*inputCtx.desiredOutput()),
                ShaderFlags::kNonTrivialSampling,
                FilterResult::kDefaultSampling,
            );
        }
        let explicit = if self.fRestrictOutputToInputBounds {
            Some(self.applyMaxSampleRadius(ctx.mapping(), actual))
        } else {
            None
        };
        builder.eval_in_space(
            |inputs| {
                let lock = SkAutoSpinlock::new(&self.fRuntimeEffectLock);
                let rt = unsafe { &mut *self.fRuntimeEffectBuilder.get() };
                for i in 0..count {
                    rt.child(&self.fChildShaderNames[i]).assign(
                        inputs[i]
                            .clone()
                            .map(ChildPtr::Shader)
                            .unwrap_or(ChildPtr::Empty),
                    );
                }
                let shader = rt.makeShader(None);
                for i in 0..count {
                    rt.child(&self.fChildShaderNames[i]).assign_null();
                }
                drop(lock);
                Ok(shader)
            },
            explicit,
            true,
        )
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:299-318
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:77-80
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        let count = self.countInputs();
        if count == 0 {
            return Ok(LayerSpace::<SkIRect>::Empty());
        }
        let required = self.applyMaxSampleRadius(mapping, *desired);
        let mut result = self.getChildInputLayerBounds(0, mapping, &required, content)?;
        for i in 1..count {
            result.join(&self.getChildInputLayerBounds(i, mapping, &required, content)?);
        }
        Ok(result)
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:320-344
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:82-84
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        if self.fRestrictOutputToInputBounds {
            let mut unbounded = false;
            let mut combined = LayerSpace::<SkIRect>::Empty();
            for i in 0..self.countInputs() {
                let output = self.getChildOutputLayerBounds(i, mapping, content)?;
                let bounds = if let Some(o) = output {
                    o
                } else {
                    unbounded = true;
                    LayerSpace::<SkIRect>::Empty()
                };
                if i == 0 {
                    combined = bounds;
                } else {
                    combined.join(&bounds);
                }
            }
            if !unbounded {
                return Ok(Some(self.applyMaxSampleRadius(mapping, combined)));
            }
        }
        Ok(LayerSpace::<SkIRect>::Unbounded())
    }
}
impl SkImageFilter for SkRuntimeImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:346-354
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        if self.fRestrictOutputToInputBounds {
            let mut bounds = compute_fast_bounds_base(self, src)?;
            bounds.outset(self.fMaxSampleRadius, self.fMaxSampleRadius);
            return Ok(bounds);
        }
        Ok(SkRectPriv::MakeLargeS32())
    }
}

// Source: src/effects/imagefilters/SkRuntimeImageFilter.cpp:95-99,236-245,282-293
// Native synchronization projection: all shared builder mutation/read intervals
// hold fRuntimeEffectLock. As in the native CPU owner, retained effect/child
// resource graphs must support the same thread transfer and synchronous leases;
// these impls do not establish safety of unproved external resource lifetimes.
unsafe impl Send for SkRuntimeImageFilter {}
unsafe impl Sync for SkRuntimeImageFilter {}
