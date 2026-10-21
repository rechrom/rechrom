// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2014 Google Inc. BSD-style license.
use crate::src::core::{SkColor::SkColor4f, SkEffectPriv::SkStageRec, SkMatrix::SkMatrix};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{
        ConcatLocalMatrices, GradientInfo, GradientType, MatrixRec, ShaderType, SkShaderBase,
    },
};
use std::sync::Arc;
impl SkLocalMatrixShader {
    // Source: src/shaders/SkLocalMatrixShader.h:31-38
    // UNVERIFIED: FnOnce captures the native variadic argument pack, preserving
    // its actual moves/borrows and calling the concrete constructor exactly once.
    // The SkShader bound is the original is_base_of constraint. Arc is the
    // canonical retained shader owner; a null matrix returns this same allocation.
    pub fn MakeWrapped<T: SkShader + 'static>(
        localMatrix: Option<&SkMatrix>,
        construct: impl FnOnce() -> T,
    ) -> Arc<dyn SkShader> {
        let shader: Arc<dyn SkShader> = Arc::new(construct());
        if let Some(matrix) = localMatrix {
            return shader.makeWithLocalMatrix(matrix);
        }
        shader
    }
    // Source: src/shaders/SkLocalMatrixShader.h:41-42
    pub fn new(wrapped: Arc<dyn SkShader>, localMatrix: &SkMatrix) -> Self {
        Self {
            fLocalMatrix: localMatrix.clone(),
            fWrappedShader: wrapped,
        }
    }
    // Source: src/shaders/SkLocalMatrixShader.h:44-44
    fn header_isOpaque(&self) -> bool {
        self.fWrappedShader.isOpaque()
    }
    // Source: src/shaders/SkLocalMatrixShader.h:48-48
    fn header_type(&self) -> ShaderType {
        ShaderType::kLocalMatrix
    }
    // Source: src/shaders/SkLocalMatrixShader.h:50-52
    // Source: src/shaders/SkLocalMatrixShader.h:54-54
    fn header_makeAsALocalMatrixShader(
        &self,
        localMatrix: Option<&mut SkMatrix>,
    ) -> Option<Arc<dyn SkShader>> {
        if let Some(localMatrix) = localMatrix {
            *localMatrix = self.fLocalMatrix.clone();
        }
        Some(self.fWrappedShader.clone())
    }
    // Source: src/shaders/SkLocalMatrixShader.h:57-58
    pub fn localMatrix(&self) -> &SkMatrix {
        &self.fLocalMatrix
    }
    pub fn wrappedShader(&self) -> Arc<dyn SkShader> {
        self.fWrappedShader.clone()
    }
}
// Source: src/shaders/SkLocalMatrixShader.h:76-77
pub struct SkLocalMatrixShader {
    fLocalMatrix: SkMatrix,
    fWrappedShader: Arc<dyn SkShader>,
}
impl SkCTMShader {
    // Source: src/shaders/SkLocalMatrixShader.h:89-89
    fn header_isOpaque(&self) -> bool {
        self.fProxyShader.isOpaque()
    }
    // Source: src/shaders/SkLocalMatrixShader.h:94-94
    fn header_type(&self) -> ShaderType {
        ShaderType::kCTM
    }
    // Source: src/shaders/SkLocalMatrixShader.h:96-97
    pub fn ctm(&self) -> &SkMatrix {
        &self.fCTM
    }
    pub fn proxyShader(&self) -> Arc<dyn SkShader> {
        self.fProxyShader.clone()
    }
}
// Source: src/shaders/SkLocalMatrixShader.h:107-108
pub struct SkCTMShader {
    fProxyShader: Arc<dyn SkShader>,
    fCTM: SkMatrix,
}
// Required virtual implementations delegate to their original header algorithms above.
impl SkShader for SkLocalMatrixShader {
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
// Source: src/shaders/SkLocalMatrixShader.h:74-74
impl crate::src::core::SkFlattenable::SkFlattenableView for SkLocalMatrixShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkLocalMatrixShader")
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
impl SkLocalMatrixShader {
    // Source: src/shaders/SkLocalMatrixShader.cpp:16-17
    fn isConstant_impl(&self, color: Option<&mut SkColor4f>) -> bool {
        self.fWrappedShader.isConstant(color)
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:20-24
    // Source: src/shaders/SkLocalMatrixShader.cpp:26-26
    unsafe fn asGradient_impl(
        &self,
        info: Option<&mut GradientInfo>,
        localMatrix: Option<&mut SkMatrix>,
    ) -> GradientType {
        let mut localMatrix = localMatrix;
        let kind = self
            .fWrappedShader
            .asGradient(info, localMatrix.as_deref_mut());
        if kind != GradientType::kNone {
            if let Some(localMatrix) = localMatrix {
                *localMatrix = ConcatLocalMatrices(&self.fLocalMatrix, localMatrix);
            }
        }
        kind
    }
    fn type_impl(&self) -> ShaderType {
        self.header_type()
    }
    fn makeAsALocalMatrixShader_impl(
        &self,
        localMatrix: Option<&mut SkMatrix>,
    ) -> Option<Arc<dyn SkShader>> {
        self.header_makeAsALocalMatrixShader(localMatrix)
    }
}
// Source: src/shaders/SkLocalMatrixShader.cpp:29-37
impl SkLocalMatrixShader {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut lm = SkMatrix::default();
        buffer.readMatrix(&mut lm);
        let Some(base) = buffer.readShader()? else {
            return Ok(None);
        };
        Ok(Some(
            crate::src::core::SkFlattenable::FlattenableOwner::Shader(
                base.makeWithLocalMatrix(&lm),
            ),
        ))
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:39-42
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeMatrix(&self.fLocalMatrix);
        buffer.writeFlattenable(Some(&*self.fWrappedShader))?;
        Ok(())
    }
}
impl SkShaderBase for SkLocalMatrixShader {
    fn isConstant(&self, color: Option<&mut SkColor4f>) -> bool {
        self.isConstant_impl(color)
    }
    unsafe fn asGradient(
        &self,
        info: Option<&mut GradientInfo>,
        localMatrix: Option<&mut SkMatrix>,
    ) -> GradientType {
        unsafe { self.asGradient_impl(info, localMatrix) }
    }
    fn r#type(&self) -> ShaderType {
        self.type_impl()
    }
    fn makeAsALocalMatrixShader(&self, lm: Option<&mut SkMatrix>) -> Option<Arc<dyn SkShader>> {
        self.makeAsALocalMatrixShader_impl(lm)
    }

    // Source: src/shaders/SkLocalMatrixShader.cpp:45-47
    unsafe fn onMakeContext(
        &self,
        rec: &crate::src::shaders::SkShaderBase::ContextRec<'_>,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<Option<*mut dyn crate::src::shaders::SkShaderBase::Context>, crate::RasterError>
    {
        unsafe {
            self.fWrappedShader.makeContext(
                &crate::src::shaders::SkShaderBase::ContextRec::Concat(rec, &self.fLocalMatrix),
                alloc,
            )
        }
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:51-55
    // Source: src/shaders/SkLocalMatrixShader.cpp:58-58
    fn onIsAImage(
        &self,
        outMatrix: Option<&mut SkMatrix>,
        mode: Option<&mut [crate::include::core::SkTileMode::SkTileMode; 2]>,
    ) -> Option<&dyn crate::src::image::SkImage::SkImage> {
        let mut imageMatrix = SkMatrix::default();
        let image = self.fWrappedShader.isAImage(Some(&mut imageMatrix), mode);
        if image.is_some() {
            if let Some(outMatrix) = outMatrix {
                *outMatrix = ConcatLocalMatrices(&self.fLocalMatrix, &imageMatrix);
            }
        }
        image
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:61-62
    fn onAsLuminanceColor(&self, color: &mut SkColor4f) -> bool {
        self.fWrappedShader.asLuminanceColor(Some(color))
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:65-67
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            self.fWrappedShader
                .appendStages(rec, &mrec.concat(&self.fLocalMatrix))
        }
    }
}
impl SkCTMShader {
    // Source: src/shaders/SkLocalMatrixShader.cpp:72-73
    pub fn new(proxy: Arc<dyn SkShader>, ctm: &SkMatrix) -> Self {
        Self {
            fProxyShader: proxy,
            fCTM: ctm.clone(),
        }
    }
}
impl SkShader for SkCTMShader {
    fn isOpaque(&self) -> bool {
        self.header_isOpaque()
    }
}
impl SkShaderBase for SkCTMShader {
    // Source: src/shaders/SkLocalMatrixShader.cpp:75-76
    fn isConstant(&self, color: Option<&mut SkColor4f>) -> bool {
        self.fProxyShader.isConstant(color)
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:79-81
    unsafe fn asGradient(
        &self,
        info: Option<&mut GradientInfo>,
        localMatrix: Option<&mut SkMatrix>,
    ) -> GradientType {
        self.fProxyShader.asGradient(info, localMatrix)
    }
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkLocalMatrixShader.cpp:84-85
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        _mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.fProxyShader.appendRootStages(rec, &self.fCTM) }
    }
}
// Source: src/shaders/SkLocalMatrixShader.cpp:88-91
impl SkCTMShader {
    pub fn CreateProc(
        _buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        crate::SkASSERT!(@source b"src/shaders/SkLocalMatrixShader.cpp\0".as_ptr(),89; @condition b"false\0".as_ptr(); false);
        Ok(None)
    }
}
// Source: src/shaders/SkLocalMatrixShader.h:100-100
// Source: src/shaders/SkLocalMatrixShader.h:105-105
impl crate::src::core::SkFlattenable::SkFlattenableView for SkCTMShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkCTMShader")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
    fn flatten(
        &self,
        _writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/shaders/SkLocalMatrixShader.h\0".as_ptr(),100; @condition b"SkWriteBuffer&\0".as_ptr(); false);
        Ok(())
    }
}
