// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2021 Google LLC. BSD-style license.
use crate::src::core::{
    SkEffectPriv::SkStageRec, SkMatrix::SkMatrix, SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use core::cell::UnsafeCell;
impl SkTransformShader<'_> {
    // Source: src/shaders/SkTransformShader.h:34-34
    fn header_type(&self) -> ShaderType {
        ShaderType::kTransform
    }
    // Source: src/shaders/SkTransformShader.h:46-46
    fn header_isOpaque(&self) -> bool {
        self.fShader.isOpaque()
    }
}
// Source: src/shaders/SkTransformShader.h:49-51
// The source retains only a borrowed child. Pipeline contexts borrow this exact inline storage;
// UnsafeCell permits native updates through the stable shared pipeline context and keeps this
// matrix accesses subject to an unsafe exclusive lease. Caller keeps shader/borrowed child alive and unmoved until all pipeline
// executions finish, and must not update during execution.
pub struct SkTransformShader<'a> {
    fShader: &'a dyn SkShader,
    fMatrixStorage: UnsafeCell<[f32; 9]>,
    fAllowPerspective: bool,
}
impl<'a> SkTransformShader<'a> {
    // Source: src/shaders/SkTransformShader.cpp:17-19
    pub fn new(shader: &'a dyn SkShader, allowPerspective: bool) -> Self {
        let mut storage = [0.0; 9];
        SkMatrix::default().get9(&mut storage);
        Self {
            fShader: shader,
            fAllowPerspective: allowPerspective,
            fMatrixStorage: UnsafeCell::new(storage),
        }
    }
    // Source: src/shaders/SkTransformShader.cpp:22-24
    // Source: src/shaders/SkTransformShader.cpp:27-28
    /// Safety: exclusive native operation lease; no update overlaps append,
    /// execution or matrix access. This exact owner remains pinned/live while
    /// stages hold its matrix address. No safe method mutates shared state.
    pub unsafe fn update(&self, matrix: &SkMatrix) -> bool {
        if !self.fAllowPerspective && matrix.hasPerspective() {
            return false;
        }
        let mut storage = [0.0; 9];
        matrix.get9(&mut storage);
        unsafe {
            self.fMatrixStorage.get().write(storage);
        }
        true
    }
}
// Source: src/shaders/SkTransformShader.h:37-44
impl crate::src::core::SkFlattenable::SkFlattenableView for SkTransformShader<'_> {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        crate::SkDEBUGFAIL!(@source b"src/shaders/SkTransformShader.h\0".as_ptr(),38; b"SkTransformShader shouldn't be serialized.\0".as_ptr());
        None
    }
    fn getTypeName(&self) -> Option<&str> {
        crate::SkDEBUGFAIL!(@source b"src/shaders/SkTransformShader.h\0".as_ptr(),42; b"SkTransformShader shouldn't be serialized.\0".as_ptr());
        None
    }
    // Inherited SkShaderBase/SkFlattenable base protocol, no additional native
    // algorithm credit: the base flatten writes no fields and type is the shader type.
    fn flatten(
        &self,
        _writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
}
impl SkShader for SkTransformShader<'_> {
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
impl SkShaderBase for SkTransformShader<'_> {
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkTransformShader.cpp:31-32
    // Source: src/shaders/SkTransformShader.cpp:39-42
    // Source: src/shaders/SkTransformShader.cpp:47-47
    // Source: src/shaders/SkTransformShader.cpp:49-51
    // Source: src/shaders/SkTransformShader.cpp:53-54
    /// Safety: this shader and child remain alive/unmoved through pipeline execution;
    /// updates may occur only between executions, never concurrently.
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        crate::SkASSERT!(@source b"src/shaders/SkTransformShader.cpp\0".as_ptr(),39; @condition b"!mRec.hasPendingMatrix()\0".as_ptr(); !mrec.hasPendingMatrix());
        let Some(mut childMRec) = (unsafe { mrec.apply(rec, &SkMatrix::default()) }) else {
            return Ok(false);
        };
        childMRec.markTotalMatrixInvalid();
        let kind = if self.fAllowPerspective {
            Op::matrix_perspective
        } else {
            Op::matrix_2x3
        };
        unsafe {
            (*rec.fPipeline).append(kind, self.fMatrixStorage.get().cast());
            let _ = self.fShader.appendStages(rec, &childMRec)?;
        }
        Ok(true)
    }
}

// Immutable queries access only the borrowed immutable child/allow flag. Matrix
// access/publication is exclusively through unsafe update/append contracts.
unsafe impl Send for SkTransformShader<'_> {}
unsafe impl Sync for SkTransformShader<'_> {}
