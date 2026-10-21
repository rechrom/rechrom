// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2023 Google LLC. BSD-style license.
use crate::src::core::SkEffectPriv::SkStageRec;
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;

// Source: src/shaders/SkEmptyShader.h:22-22
// Source: src/shaders/SkEmptyShader.h:24-24
pub struct SkEmptyShader;
impl SkEmptyShader {
    pub fn new() -> Self {
        Self
    }
}

// Source: src/shaders/SkEmptyShader.h:33-34
// Source: src/shaders/SkEmptyShader.h:37-38
impl SkShaderBase for SkEmptyShader {
    // This false return is the original empty shader's intentional no-draw behavior.
    unsafe fn appendStages(
        &self,
        _rec: &SkStageRec<'_>,
        _mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        Ok(false)
    }
    fn r#type(&self) -> ShaderType {
        ShaderType::kEmpty
    }
}
impl SkShader for SkEmptyShader {
    fn isOpaque(&self) -> bool {
        false
    }
}

// Source: src/shaders/SkEmptyShader.h:42-42
impl crate::src::core::SkFlattenable::SkFlattenableView for SkEmptyShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkEmptyShader")
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
// Source: src/shaders/SkEmptyShader.h:27-31
impl SkEmptyShader {
    fn flatten_impl(
        &self,
        _writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
}
// Source: src/shaders/SkEmptyShader.cpp:16-18
impl SkEmptyShader {
    pub fn CreateProc(
        _buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        Ok(Some(
            crate::src::core::SkFlattenable::FlattenableOwner::Shader(Empty()),
        ))
    }
}
// Source: src/shaders/SkEmptyShader.cpp:20-20
pub fn Empty() -> Arc<dyn SkShader> {
    Arc::new(SkEmptyShader::new())
}
// Source: src/shaders/SkEmptyShader.cpp:22-24
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterEmptyShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(b"SkEmptyShader", SkEmptyShader::CreateProc);
    }
}
