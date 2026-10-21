// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2023 Google LLC. BSD-style license.
use crate::src::core::{
    SkEffectPriv::SkStageRec, SkMatrix::SkMatrix, SkRasterPipelineOpContexts::CoordClampCtx,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op, SkRect::SkRect,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;
impl SkCoordClampShader {
    // Source: src/shaders/SkCoordClampShader.h:25-26
    pub fn new(shader: Arc<dyn SkShader>, subset: &SkRect) -> Self {
        Self {
            fShader: shader,
            fSubset: *subset,
        }
    }
    // Source: src/shaders/SkCoordClampShader.h:28-28
    fn header_type(&self) -> ShaderType {
        ShaderType::kCoordClamp
    }
    // Source: src/shaders/SkCoordClampShader.h:30-31
    pub fn shader(&self) -> Arc<dyn SkShader> {
        self.fShader.clone()
    }
    pub fn subset(&self) -> SkRect {
        self.fSubset
    }
    // Source: src/shaders/SkCoordClampShader.h:33-33
    fn header_isOpaque(&self) -> bool {
        self.fShader.isOpaque()
    }
}
// Source: src/shaders/SkCoordClampShader.h:44-45
pub struct SkCoordClampShader {
    fShader: Arc<dyn SkShader>,
    fSubset: SkRect,
}
impl SkShader for SkCoordClampShader {
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
// Source: src/shaders/SkCoordClampShader.h:42-42
impl crate::src::core::SkFlattenable::SkFlattenableView for SkCoordClampShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkCoordClampShader")
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
// Source: src/shaders/SkCoordClampShader.cpp:23-30
impl SkCoordClampShader {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let shader = buffer.readShader()?;
        let subset = buffer.readRect();
        if !buffer.validate(shader.is_some()) {
            return Ok(None);
        }
        Ok(CoordClamp(shader, &subset)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/SkCoordClampShader.cpp:32-35
    // Source: src/shaders/SkCoordClampShader.h:37
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(Some(&*self.fShader))?;
        buffer.writeRect(&self.fSubset);
        Ok(())
    }
}
impl SkShaderBase for SkCoordClampShader {
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkCoordClampShader.cpp:37-41
    // Source: src/shaders/SkCoordClampShader.cpp:46-49
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        let Some(childMRec) = (unsafe { mrec.apply(rec, &SkMatrix::default()) }) else {
            return Ok(false);
        };
        let clampCtx = unsafe {
            (*rec.fAlloc).make(CoordClampCtx {
                min_x: self.fSubset.fLeft,
                min_y: self.fSubset.fTop,
                max_x: self.fSubset.fRight,
                max_y: self.fSubset.fBottom,
            })
        };
        unsafe {
            (*rec.fPipeline).append(Op::clamp_x_and_y, clampCtx.cast());
            self.fShader.appendStages(rec, &childMRec)
        }
    }
}
// Source: src/shaders/SkCoordClampShader.cpp:52-57
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterCoordClampShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkCoordClampShader",
            SkCoordClampShader::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkShader_CoordClamp",
            SkCoordClampShader::CreateProc,
        );
    }
}
// Source: src/shaders/SkCoordClampShader.cpp:59-61
// Source: src/shaders/SkCoordClampShader.cpp:63-64
// Source: src/shaders/SkCoordClampShader.cpp:66-66
pub fn CoordClamp(shader: Option<Arc<dyn SkShader>>, subset: &SkRect) -> Option<Arc<dyn SkShader>> {
    let Some(shader) = shader else {
        return None;
    };
    if !subset.isSorted() {
        return None;
    }
    Some(Arc::new(SkCoordClampShader::new(shader, subset)))
}

// Source: src/shaders/SkCoordClampShader.h:36
// Native protected ordinary declaration: SkCoordClampShader(SkReadBuffer&).
// No definition occurs in src/shaders/SkCoordClampShader.cpp. Rust has no bodyless ordinary
// inherent method with this adapted object representation; the declaration remains
// pending rather than inventing an extern ABI, pure-virtual trait, or constructor body.
