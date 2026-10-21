// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2023 Google LLC. BSD-style license.
use crate::include::core::SkAlphaType::SkAlphaType::{kPremul_SkAlphaType, kUnpremul_SkAlphaType};
use crate::src::core::{
    SkColor::{SkColor, SkColor4f},
    SkColorSpace::SkColorSpace,
    SkColorSpaceXformSteps::SkColorSpaceXformSteps,
    SkEffectPriv::SkStageRec,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;

impl SkColorShader {
    // Source: src/shaders/SkColorShader.h:29-29
    pub fn new(color: &SkColor4f) -> Self {
        Self { fColor: *color }
    }
    // Private helpers hold native inline bodies in source order. Required trait dispatch below
    // forwards directly, without bypassing any child's virtual method or changing ownership.
    // Source: src/shaders/SkColorShader.h:31-31
    fn header_isOpaque(&self) -> bool {
        self.fColor.isOpaque()
    }
    // Source: src/shaders/SkColorShader.h:32-34
    // Source: src/shaders/SkColorShader.h:36-36
    fn header_isConstant(&self, color: Option<&mut SkColor4f>) -> bool {
        if let Some(color) = color {
            *color = self.fColor;
        }
        true
    }
    // Source: src/shaders/SkColorShader.h:39-39
    fn header_type(&self) -> ShaderType {
        ShaderType::kColor
    }
    // Source: src/shaders/SkColorShader.h:41-41
    pub fn color(&self) -> &SkColor4f {
        &self.fColor
    }
    // Source: src/shaders/SkColorShader.h:49-51
    fn header_onAsLuminanceColor(&self, color: &mut SkColor4f) -> bool {
        *color = self.fColor;
        true
    }
}
// Source: src/shaders/SkColorShader.h:58-58
// Source: src/shaders/SkColorShader.h:25-25
// Source: src/shaders/SkColorShader.h:25
pub struct SkColorShader {
    fColor: SkColor4f,
}
impl SkShader for SkColorShader {
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
// Source: src/shaders/SkColorShader.h:45-45
impl crate::src::core::SkFlattenable::SkFlattenableView for SkColorShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkColorShader")
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
// Source: src/shaders/SkColorShader.cpp:30-44
fn legacy_color4_create_proc(
    buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
) -> crate::src::core::SkFlattenable::FactoryResult {
    let old = buffer.isVersionLT(crate::src::core::SkPicturePriv::kCombineColorShaders);
    if buffer.validate(old) {
        let mut color = SkColor4f {
            fR: 0.0,
            fG: 0.0,
            fB: 0.0,
            fA: 0.0,
        };
        let mut color_space = None;
        buffer.readColor4f(&mut color);
        if buffer.readBool() {
            if let Some(data) = buffer.readByteArrayAsData() {
                color_space = SkColorSpace::Deserialize(unsafe {
                    std::slice::from_raw_parts(data.data(), data.size())
                });
            }
        }
        return Ok(Color4f(&color, color_space)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader));
    }
    Ok(None)
}
// Source: src/shaders/SkColorShader.cpp:48-58
impl SkColorShader {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let shader = if buffer.isVersionLT(crate::src::core::SkPicturePriv::kCombineColorShaders) {
            Color(buffer.readColor())
        } else {
            let mut color = SkColor4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            };
            buffer.readColor4f(&mut color);
            Color4f(&color, Some(SkColorSpace::MakeSRGB()))
        };
        Ok(shader.map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/SkColorShader.cpp:60-62
    // Source: src/shaders/SkColorShader.h:47-47
    // Source: src/shaders/SkColorShader.h:47
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeColor4f(&self.fColor);
        Ok(())
    }
}
impl SkShaderBase for SkColorShader {
    fn isConstant(&self, color: Option<&mut SkColor4f>) -> bool {
        self.header_isConstant(color)
    }
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    fn onAsLuminanceColor(&self, color: &mut SkColor4f) -> bool {
        self.header_onAsLuminanceColor(color)
    }

    // Source: src/shaders/SkColorShader.cpp:64-69
    // Source: src/shaders/SkColorShader.h:54-54
    // Source: src/shaders/SkColorShader.h:54
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        _mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        let mut color = self.fColor;
        // MakeSRGB retains the canonical process singleton while borrowed by the source constructor;
        // XformSteps stores only derived numbers, so no extra color-space owner survives this call.
        let srgb = SkColorSpace::MakeSRGB();
        SkColorSpaceXformSteps::new(
            Some(srgb.as_ref()),
            kUnpremul_SkAlphaType,
            rec.fDstCS.map(Arc::as_ref),
            kPremul_SkAlphaType,
        )
        .apply(color.vec_mut());
        unsafe {
            (*rec.fPipeline).appendConstantColor(rec.fAlloc, color.vec());
        }
        Ok(true)
    }
}

// Source: src/shaders/SkColorShader.cpp:74-77
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterColorShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(b"SkColorShader", SkColorShader::CreateProc);
        crate::src::core::SkFlattenable::Register(b"SkColorShader4", legacy_color4_create_proc);
    }
}
// Source: src/shaders/SkColorShader.cpp:80-81
pub fn Color(color: SkColor) -> Option<Arc<dyn SkShader>> {
    Color4f(&SkColor4f::FromColor(color), Some(SkColorSpace::MakeSRGB()))
}

// Source: src/shaders/SkColorShader.cpp:84-86
// Source: src/shaders/SkColorShader.cpp:91-93
// Source: src/shaders/SkColorShader.cpp:95-95
pub fn Color4f(color: &SkColor4f, space: Option<Arc<SkColorSpace>>) -> Option<Arc<dyn SkShader>> {
    if !crate::src::core::SkFloatingPoint::SkIsFinite(color.vec()) {
        return None;
    }
    let mut srgb = color.pinAlpha();
    let srgb_space = SkColorSpace::MakeSRGB();
    SkColorSpaceXformSteps::new(
        space.as_deref(),
        kUnpremul_SkAlphaType,
        Some(srgb_space.as_ref()),
        kUnpremul_SkAlphaType,
    )
    .apply(srgb.vec_mut());
    Some(Arc::new(SkColorShader::new(&srgb)))
}
