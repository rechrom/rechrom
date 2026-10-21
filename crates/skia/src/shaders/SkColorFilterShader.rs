// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2013 Google Inc. BSD-style license.
use crate::src::core::{
    SkColorFilter::SkColorFilter, SkEffectPriv::SkStageRec,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;
impl SkColorFilterShader {
    // Source: src/shaders/SkColorFilterShader.h:26-26
    fn header_type(&self) -> ShaderType {
        ShaderType::kColorFilter
    }
    // Source: src/shaders/SkColorFilterShader.h:28-30
    pub fn shader(&self) -> Arc<dyn SkShader> {
        Arc::clone(&self.fShader)
    }
    pub fn filter(
        &self,
    ) -> Arc<dyn crate::src::effects::colorfilters::SkColorFilterBase::SkColorFilterBase> {
        let filter = Arc::clone(&self.fFilter);
        let filter: Arc<
            dyn crate::src::effects::colorfilters::SkColorFilterBase::SkColorFilterBase,
        > = filter;
        filter
    }
    pub fn alpha(&self) -> f32 {
        self.fAlpha
    }
}
// Source: src/shaders/SkColorFilterShader.h:41-43
pub struct SkColorFilterShader {
    fShader: core::mem::ManuallyDrop<Arc<dyn SkShader>>,
    fFilter: core::mem::ManuallyDrop<Arc<dyn SkColorFilter>>,
    fAlpha: f32,
}
impl SkColorFilterShader {
    // Source: src/shaders/SkColorFilterShader.cpp:23-28
    // Source: src/shaders/SkColorFilterShader.cpp:30-31
    fn new(shader: Arc<dyn SkShader>, alpha: f32, filter: Arc<dyn SkColorFilter>) -> Self {
        Self {
            fShader: core::mem::ManuallyDrop::new(shader),
            fFilter: core::mem::ManuallyDrop::new(filter),
            fAlpha: alpha,
        }
    }
    // Source: src/shaders/SkColorFilterShader.cpp:34-42
    pub fn Make(
        shader: Option<Arc<dyn SkShader>>,
        alpha: f32,
        filter: Option<Arc<dyn SkColorFilter>>,
    ) -> Option<Arc<dyn SkShader>> {
        if shader.is_none() {
            None
        } else if filter.is_none() {
            shader
        } else {
            Some(Arc::new(Self::new(shader.unwrap(), alpha, filter.unwrap())))
        }
    }
}
// Source: src/shaders/SkColorFilterShader.h:39-39
impl crate::src::core::SkFlattenable::SkFlattenableView for SkColorFilterShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkColorFilterShader")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/shaders/SkColorFilterShader.cpp:44-48
impl SkColorFilterShader {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let shader = buffer.readShader()?;
        let filter = buffer.readColorFilter()?;
        Ok(Self::Make(shader, 1.0, filter)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
}
impl SkShader for SkColorFilterShader {
    // Source: src/shaders/SkColorFilterShader.cpp:52-53
    fn isOpaque(&self) -> bool {
        self.fShader.isOpaque() && self.fAlpha == 1.0 && self.fFilter.isAlphaUnchanged()
    }
}
// Source: src/shaders/SkColorFilterShader.cpp:54-58
impl SkColorFilterShader {
    // Source: src/shaders/SkColorFilterShader.h:36
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(Some(&**self.fShader))?;
        crate::SkASSERT!(@source b"src/shaders/SkColorFilterShader.cpp\0".as_ptr(),58; @condition b"fAlpha == 1.0f\0".as_ptr(); self.fAlpha == 1.0);
        buffer.writeFlattenable(Some(&**self.fFilter))?;

        Ok(())
    }
}
impl SkShaderBase for SkColorFilterShader {
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkColorFilterShader.cpp:62-65
    // Source: src/shaders/SkColorFilterShader.cpp:67-68
    // Source: src/shaders/SkColorFilterShader.cpp:70-71
    // Source: src/shaders/SkColorFilterShader.cpp:73-73
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        if !unsafe { self.fShader.appendStages(rec, mrec) }? {
            return Ok(false);
        }
        if self.fAlpha != 1.0 {
            unsafe {
                let alpha = (*rec.fAlloc).make(self.fAlpha);
                (*rec.fPipeline).append(Op::scale_1_float, alpha.cast());
            }
        }
        if !unsafe {
            self.fFilter
                .appendStages(rec, self.fAlpha == 1.0 && self.fShader.isOpaque())
        }? {
            return Ok(false);
        }
        Ok(true)
    }
}

// Source: src/shaders/SkColorFilterShader.h:41-43
// Preserve native reverse member destruction, retaining the original Arc allocations.
impl Drop for SkColorFilterShader {
    fn drop(&mut self) {
        struct Release<'a, T>(&'a mut core::mem::ManuallyDrop<T>);
        impl<T> Drop for Release<'_, T> {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(self.0);
                }
            }
        }
        // Tuple elements drop in order; the shader guard survives filter unwinding.
        drop((Release(&mut self.fFilter), Release(&mut self.fShader)));
    }
}
