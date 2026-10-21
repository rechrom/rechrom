// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2023 Google LLC. BSD-style license.
use crate::include::core::SkAlphaType::SkAlphaType::{
    self, kPremul_SkAlphaType, kUnpremul_SkAlphaType,
};
use crate::src::core::{
    SkColorSpace::SkColorSpace, SkColorSpaceXformSteps::SkColorSpaceXformSteps,
    SkEffectPriv::SkStageRec,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;
impl SkWorkingColorSpaceShader {
    // Source: src/shaders/SkWorkingColorSpaceShader.h:32-32
    fn header_type(&self) -> ShaderType {
        ShaderType::kWorkingColorSpace
    }
    // Source: src/shaders/SkWorkingColorSpaceShader.h:34-34
    fn header_isOpaque(&self) -> bool {
        self.fShader.isOpaque()
    }
    // Source: src/shaders/SkWorkingColorSpaceShader.h:36-36
    pub fn shader(&self) -> Arc<dyn SkShader> {
        self.fShader.clone()
    }
    // Source: src/shaders/SkWorkingColorSpaceShader.h:38-44
    pub fn workingSpace(
        &self,
        dstCS: Arc<SkColorSpace>,
        dstAT: SkAlphaType,
    ) -> (Arc<SkColorSpace>, Arc<SkColorSpace>, SkAlphaType) {
        let inputSpace = if let Some(input) = &self.fInputSpace {
            input.clone()
        } else {
            dstCS.clone()
        };
        let outputSpace = self
            .fOutputSpace
            .clone()
            .unwrap_or_else(|| inputSpace.clone());
        (
            inputSpace,
            outputSpace,
            if self.fWorkInUnpremul {
                kUnpremul_SkAlphaType
            } else {
                dstAT
            },
        )
    }
    // Source: src/shaders/SkWorkingColorSpaceShader.h:48-57
    fn new(
        shader: Arc<dyn SkShader>,
        inputCS: Option<Arc<SkColorSpace>>,
        outputCS: Option<Arc<SkColorSpace>>,
        workInUnpremul: bool,
    ) -> Self {
        crate::SkASSERT!(@source b"src/shaders/SkWorkingColorSpaceShader.h\0".as_ptr(),57; @condition b"fInputSpace || fOutputSpace || fWorkInUnpremul\0".as_ptr(); inputCS.is_some() || outputCS.is_some() || workInUnpremul);
        Self {
            fShader: shader,
            fInputSpace: inputCS,
            fOutputSpace: outputCS,
            fWorkInUnpremul: workInUnpremul,
        }
    }
}
// Source: src/shaders/SkWorkingColorSpaceShader.h:67-70
pub struct SkWorkingColorSpaceShader {
    // Source: src/shaders/SkWorkingColorSpaceShader.h:70-70
    fWorkInUnpremul: bool,
    // Source: src/shaders/SkWorkingColorSpaceShader.h:69-69
    fOutputSpace: Option<Arc<SkColorSpace>>,
    // Source: src/shaders/SkWorkingColorSpaceShader.h:68-68
    fInputSpace: Option<Arc<SkColorSpace>>,
    // Source: src/shaders/SkWorkingColorSpaceShader.h:67-67
    fShader: Arc<dyn SkShader>,
}
impl SkWorkingColorSpaceShader {
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:24-29
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:32-32
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:36-37
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:40-43
    pub fn Make(
        shader: Option<Arc<dyn SkShader>>,
        inputCS: Option<Arc<SkColorSpace>>,
        outputCS: Option<Arc<SkColorSpace>>,
        workInUnpremul: bool,
    ) -> Option<Arc<dyn SkShader>> {
        let Some(shader) = shader else {
            return None;
        };
        if inputCS.is_none() && outputCS.is_none() && !workInUnpremul {
            Some(shader)
        } else {
            Some(Arc::new(Self::new(
                shader,
                inputCS,
                outputCS,
                workInUnpremul,
            )))
        }
    }
}
impl SkShader for SkWorkingColorSpaceShader {
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
impl SkShaderBase for SkWorkingColorSpaceShader {
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:47-51
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:55-56
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:58-60
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:62-63
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:69-70
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:73-79
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:81-82
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:85-86
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        // StageRec's borrowed CS keeps the original sk_ref_sp temporary's pointee alive until
        // both numeric steps are derived. No color-space references survive in XformSteps.
        let fallback = rec.fDstCS.is_none().then(SkColorSpace::MakeSRGB);
        let dstCS = rec
            .fDstCS
            .map(Arc::as_ref)
            .unwrap_or_else(|| fallback.as_deref().unwrap());
        let dstAT = kPremul_SkAlphaType;
        let inputCS = self.fInputSpace.as_deref().unwrap_or(dstCS);
        let outputCS = self.fOutputSpace.as_deref().unwrap_or(inputCS);
        let workingAT = if self.fWorkInUnpremul {
            kUnpremul_SkAlphaType
        } else {
            dstAT
        };
        // Original ColorInfo temporaries supply only these CS/alpha values to the numeric
        // constructor; the source color type is identical in all three and is not inspected.
        let dstToInput = unsafe {
            (*rec.fAlloc).make(SkColorSpaceXformSteps::new(
                Some(dstCS),
                dstAT,
                Some(inputCS),
                workingAT,
            ))
        };
        let outputToDst = unsafe {
            (*rec.fAlloc).make(SkColorSpaceXformSteps::new(
                Some(outputCS),
                workingAT,
                Some(dstCS),
                dstAT,
            ))
        };
        let mut paintColorInWorkingSpace = rec.fPaintColor.makeOpaque();
        unsafe {
            (*dstToInput).apply(paintColorInWorkingSpace.vec_mut());
        }
        let workingRec = SkStageRec {
            fPipeline: rec.fPipeline,
            fAlloc: rec.fAlloc,
            fDstColorType: rec.fDstColorType,
            fDstCS: self.fInputSpace.as_ref(),
            fPaintColor: paintColorInWorkingSpace,
            fSurfaceProps: rec.fSurfaceProps,
            fDstBounds: rec.fDstBounds,
        };
        if !unsafe { self.fShader.appendStages(&workingRec, mrec) }? {
            return Ok(false);
        }
        unsafe {
            (*outputToDst).applyPipeline(&mut *rec.fPipeline);
        }
        Ok(true)
    }
}
// Source: src/shaders/SkWorkingColorSpaceShader.h:63-63
impl crate::src::core::SkFlattenable::SkFlattenableView for SkWorkingColorSpaceShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkWorkingColorSpaceShader")
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
// Source: src/shaders/SkWorkingColorSpaceShader.cpp:89-102
impl SkWorkingColorSpaceShader {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(Some(&*self.fShader))?;
        buffer.writeBool(self.fWorkInUnpremul);
        buffer.writeBool(self.fInputSpace.is_some());
        if let Some(cs) = &self.fInputSpace {
            let data = cs.serialize();
            buffer.writeDataAsByteArray(Some(&data));
        }
        buffer.writeBool(self.fOutputSpace.is_some());
        if let Some(cs) = &self.fOutputSpace {
            let data = cs.serialize();
            buffer.writeDataAsByteArray(Some(&data));
        }

        Ok(())
    }
    // Source: src/shaders/SkWorkingColorSpaceShader.cpp:104-140
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let shader = buffer.readShader()?;
        let legacy = buffer.isVersionLT(crate::src::core::SkPicturePriv::kWorkingColorSpaceOutput);
        let unpremul = !legacy && buffer.readBool();
        let mut input = None;
        if legacy || buffer.readBool() {
            let data = buffer.readByteArrayAsData();
            if !buffer.validate(data.is_some()) {
                return Ok(None);
            }
            let data = data.unwrap();
            input = SkColorSpace::Deserialize(unsafe {
                std::slice::from_raw_parts(data.data(), data.size())
            });
            if !buffer.validate(input.is_some()) {
                return Ok(None);
            }
        }
        let mut output = None;
        if !legacy && buffer.readBool() {
            let data = buffer.readByteArrayAsData();
            if !buffer.validate(data.is_some()) {
                return Ok(None);
            }
            let data = data.unwrap();
            output = SkColorSpace::Deserialize(unsafe {
                std::slice::from_raw_parts(data.data(), data.size())
            });
            if !buffer.validate(output.is_some()) {
                return Ok(None);
            }
        }
        Ok(Self::Make(shader, input, output, unpremul)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
}
// Source: src/shaders/SkWorkingColorSpaceShader.cpp:142-144
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterWorkingColorSpaceShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkWorkingColorSpaceShader",
            SkWorkingColorSpaceShader::CreateProc,
        );
    }
}
