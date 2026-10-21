#![allow(non_snake_case)]
use super::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkMatrix::SkMatrix,
    SkRect::{SkIRect, SkRect},
};
use std::sync::Arc;
// Source: src/core/SkLocalMatrixImageFilter.h:40-45
// Source: src/core/SkLocalMatrixImageFilter.h:66-68
// Geometric local-matrix ownership slice; filtering requires the real Context execution path.
pub(crate) struct SkLocalMatrixImageFilter {
    fInvLocalMatrix: SkMatrix,
    fLocalMatrix: SkMatrix,
    base: SkImageFilter_Base,
}
impl SkLocalMatrixImageFilter {
    fn new(local: &SkMatrix, inverse: SkMatrix, input: Arc<dyn SkImageFilter>) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[Some(input)], None),
            fLocalMatrix: local.clone(),
            fInvLocalMatrix: inverse,
        }
    }
}
// Source: src/core/SkLocalMatrixImageFilter.h:30-30
// Source: src/core/SkLocalMatrixImageFilter.cpp:14-34
pub fn Make(
    local: &SkMatrix,
    input: Option<Arc<dyn SkImageFilter>>,
) -> Option<Arc<dyn SkImageFilter>> {
    let input = input?;
    if local.isIdentity() {
        return Some(input);
    }
    let capability = input.getCTMCapability();
    if (capability == MatrixCapability::kTranslate && !local.isTranslate())
        || (capability == MatrixCapability::kScaleTranslate && !local.isScaleTranslate())
    {
        return None;
    }
    if let Some(inverse) = local.invert() {
        Some(Arc::new(SkLocalMatrixImageFilter::new(
            local, inverse, input,
        )))
    } else {
        None
    }
}
// Source: src/core/SkLocalMatrixImageFilter.cpp:51-55
impl SkLocalMatrixImageFilter {
    fn localMapping(&self, mapping: &Mapping) -> Mapping {
        let mut local = *mapping;
        local.concatLocal(&self.fLocalMatrix);
        local
    }
}
// Source: src/core/SkLocalMatrixImageFilter.h:38-38
impl crate::src::core::SkFlattenable::SkFlattenableView for SkLocalMatrixImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkLocalMatrixImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)
    }
}
// Source: src/core/SkLocalMatrixImageFilter.cpp:35-40
impl SkLocalMatrixImageFilter {
    pub(crate) fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let mut lm = SkMatrix::default();
        buffer.readMatrix(&mut lm);
        Ok(Make(&lm, common.getInput(0))
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/core/SkLocalMatrixImageFilter.cpp:42-46
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeMatrix(&self.fLocalMatrix);
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkLocalMatrixImageFilter {
    // Source: src/core/SkLocalMatrixImageFilter.cpp:57-60
    unsafe fn onFilterImage(
        &self,
        ctx: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let local = self.localMapping(ctx.mapping());
        self.getChildOutput(0, &ctx.withNewMapping(&local))
    }

    // Source: src/core/SkLocalMatrixImageFilter.h:47-47
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/core/SkLocalMatrixImageFilter.cpp:62-71
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({ self.getChildInputLayerBounds(0, &self.localMapping(mapping), desired, content)? })
    }
    // Source: src/core/SkLocalMatrixImageFilter.cpp:73-77
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({ self.getChildOutputLayerBounds(0, &self.localMapping(mapping), content)? })
    }
}
impl SkImageFilter for SkLocalMatrixImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/core/SkLocalMatrixImageFilter.h:32-32
    // Source: src/core/SkLocalMatrixImageFilter.cpp:79-90
    fn computeFastBounds(&self, bounds: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({
            let mut local = SkRect::MakeEmpty();
            self.fInvLocalMatrix.mapRect(&mut local, bounds);
            let child = self.getInput(0).unwrap().computeFastBounds(&local)?;
            let mut result = SkRect::MakeEmpty();
            self.fLocalMatrix.mapRect(&mut result, &child);
            result
        })
    }
}
