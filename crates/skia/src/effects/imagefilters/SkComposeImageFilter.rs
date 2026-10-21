#![allow(non_snake_case)]
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkRect::{SkIRect, SkRect},
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkComposeImageFilter.cpp:25-38
// Actual input DAG; inner evaluation becomes the outer context source.
struct SkComposeImageFilter {
    base: SkImageFilter_Base,
}
const kOuter: usize = 0;
const kInner: usize = 1;
impl SkComposeImageFilter {
    fn new(inputs: [Option<Arc<dyn SkImageFilter>>; 2]) -> Self {
        let usesSrc = inputs[kInner]
            .as_ref()
            .map_or(false, |i| i.base().usesSource());
        let value = Self {
            base: SkImageFilter_Base::new(&inputs, Some(usesSrc)),
        };
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkComposeImageFilter.cpp\0".as_ptr(),36; @condition b"inputs[kOuter].get()\0".as_ptr(); inputs[kOuter].is_some());
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkComposeImageFilter.cpp\0".as_ptr(),37; @condition b"inputs[kInner].get()\0".as_ptr(); inputs[kInner].is_some());
        value
    }
}
// Source: src/effects/imagefilters/SkComposeImageFilter.cpp:66-76
pub fn Compose(
    outer: Option<Arc<dyn SkImageFilter>>,
    inner: Option<Arc<dyn SkImageFilter>>,
) -> Option<Arc<dyn SkImageFilter>> {
    if outer.is_none() {
        return inner;
    }
    if inner.is_none() {
        return outer;
    }
    Some(Arc::new(SkComposeImageFilter::new([outer, inner])))
}
// Source: src/effects/imagefilters/SkComposeImageFilter.cpp:48-48
impl crate::src::core::SkFlattenable::SkFlattenableView for SkComposeImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkComposeImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(writer)?;
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkComposeImageFilter.cpp:78-82
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterComposeImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkComposeImageFilter",
            SkComposeImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkComposeImageFilterImpl",
            SkComposeImageFilter::CreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkComposeImageFilter.cpp:84-87
impl SkComposeImageFilter {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 2)? {
            return Ok(None);
        }
        Ok(Compose(common.getInput(kOuter), common.getInput(kInner))
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
}
impl SkImageFilterBaseHooks for SkComposeImageFilter {
    // Source: src/effects/imagefilters/SkComposeImageFilter.cpp:50-50
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkComposeImageFilter.cpp:91-112
    // Adapted: the native synchronous CPU evaluation retains real child/image owners;
    // Result propagates failures from bounds, backend and raster operations.
    unsafe fn onFilterImage(
        &self,
        context: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let inner_output_bounds = self.getChildOutputLayerBounds(
            kInner,
            context.mapping(),
            Some(context.source().layerBounds()),
        )?;
        let outer_required_input = self.getChildInputLayerBounds(
            kOuter,
            context.mapping(),
            context.desiredOutput(),
            inner_output_bounds,
        )?;
        let inner_result =
            self.getChildOutput(kInner, &context.withNewDesiredOutput(outer_required_input))?;
        self.getChildOutput(kOuter, &context.withNewSource(&inner_result))
    }

    // Source: src/effects/imagefilters/SkComposeImageFilter.cpp:114-129
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            let outerContent = if content.is_some() {
                self.getChildOutputLayerBounds(kInner, mapping, content)?
            } else {
                None
            };
            let innerDesired =
                self.getChildInputLayerBounds(kOuter, mapping, desired, outerContent)?;
            self.getChildInputLayerBounds(kInner, mapping, &innerDesired, content)?
        })
    }
    // Source: src/effects/imagefilters/SkComposeImageFilter.cpp:131-140
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            let innerBounds = self.getChildOutputLayerBounds(kInner, mapping, content)?;
            self.getChildOutputLayerBounds(kOuter, mapping, innerBounds)?
        })
    }
}
impl SkImageFilter for SkComposeImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkComposeImageFilter.cpp:142-145
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({
            self.getInput(kOuter)
                .unwrap()
                .computeFastBounds(&self.getInput(kInner).unwrap().computeFastBounds(src)?)?
        })
    }
}
