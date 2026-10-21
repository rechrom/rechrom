// Copyright 2023 Google LLC. BSD-style license.
use super::SkColorFilterBase::{SkColorFilterBase, Type};
use crate::src::core::{
    SkColorFilter::SkColorFilter, SkEffectPriv::SkStageRec,
    SkRasterPipelineOpList::SkRasterPipelineOp,
};
use std::sync::Arc;

// Source: src/effects/colorfilters/SkGaussianColorFilter.h:22-24
pub struct SkGaussianColorFilter;
impl SkGaussianColorFilter {
    // Source: src/effects/colorfilters/SkGaussianColorFilter.cpp:20-20
    pub fn new() -> Self {
        Self
    }
}
impl SkColorFilterBase for SkGaussianColorFilter {
    // Source: src/effects/colorfilters/SkGaussianColorFilter.h:26-26
    // Source: src/effects/colorfilters/SkGaussianColorFilter.cpp:22-25
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        _shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError> {
        // Caller owns and synchronizes pipeline/arena through execution, as for the base hook.
        unsafe {
            (*rec.fPipeline).append(SkRasterPipelineOp::gauss_a_to_rgba, std::ptr::null_mut());
        }
        Ok(true)
    }
    // Source: src/effects/colorfilters/SkGaussianColorFilter.h:28-28
    fn r#type(&self) -> Type {
        Type::kGaussian
    }
}
// Source: src/effects/colorfilters/SkGaussianColorFilter.h:34-34
impl crate::src::core::SkFlattenable::SkFlattenableView for SkGaussianColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkGaussianColorFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkColorFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/effects/colorfilters/SkGaussianColorFilter.h:31-31
impl SkGaussianColorFilter {
    fn flatten_impl(
        &self,
        _writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
}
// Source: src/effects/colorfilters/SkGaussianColorFilter.cpp:27-29
impl SkGaussianColorFilter {
    pub fn CreateProc(
        _buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        Ok(Some(
            crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter(MakeGaussian()),
        ))
    }
}
// Source: src/effects/colorfilters/SkGaussianColorFilter.cpp:31-33
pub fn MakeGaussian() -> Arc<dyn SkColorFilter> {
    Arc::new(SkGaussianColorFilter::new())
}
