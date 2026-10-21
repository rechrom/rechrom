// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2012 Google Inc. BSD-style license.
use super::SkGradientBaseShader::SkGradientBaseShader;
use crate::include::effects::SkGradient::SkGradient;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkPoint;
// Source: src/shaders/gradients/SkLinearGradient.h:27-28
impl SkLinearGradient {
    pub fn start(&self) -> &SkPoint {
        &self.fStart
    }
    pub fn end(&self) -> &SkPoint {
        &self.fEnd
    }
}
// Source: src/shaders/gradients/SkLinearGradient.h:38-38
impl crate::src::core::SkFlattenable::SkFlattenableView for SkLinearGradient {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkLinearGradient")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
}
// Source: src/shaders/gradients/SkLinearGradient.h:44-45
pub struct SkLinearGradient {
    base: SkGradientBaseShader,
    fStart: SkPoint,
    fEnd: SkPoint,
}
// Source: src/shaders/gradients/SkLinearGradient.cpp:30-41
pub fn pts_to_unit_matrix(pts: &[SkPoint; 2]) -> SkMatrix {
    let mut vec = pts[1] - pts[0];
    let mag = vec.length();
    let inv = if mag != 0.0 { 1.0 / mag } else { 0.0 };
    vec.fX *= inv;
    vec.fY *= inv;
    let mut matrix = SkMatrix::default();
    matrix.setSinCosPivot(-vec.fY, vec.fX, pts[0].fX, pts[0].fY);
    matrix.postTranslate(-pts[0].fX, -pts[0].fY);
    matrix.postScale(inv, inv);
    matrix
}
// Source: src/shaders/gradients/SkLinearGradient.cpp:45-46
impl SkLinearGradient {
    pub fn new(pts: &[SkPoint; 2], desc: &SkGradient<'_>) -> Self {
        Self {
            base: SkGradientBaseShader::new(desc, &pts_to_unit_matrix(pts)),
            fStart: pts[0],
            fEnd: pts[1],
        }
    }
    // Source: src/shaders/gradients/SkLinearGradient.cpp:48-62
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut scope = super::SkGradientBaseShader::SkGradientScope::new();
        let mut legacy = SkMatrix::default();
        let Some(descriptor) = scope.unflatten(buffer, &mut legacy) else {
            return Ok(None);
        };
        let local = if legacy.isIdentity() {
            None
        } else {
            Some(&legacy)
        };
        let points = [buffer.readPoint(), buffer.readPoint()];
        let Some(gradient) = descriptor.typed() else {
            return Ok(None);
        };
        Ok(LinearGradient(Some(&points), &gradient, local)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/gradients/SkLinearGradient.h:31-31
    // Source: src/shaders/gradients/SkLinearGradient.cpp:64-68
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writePoint(&self.fStart);
        buffer.writePoint(&self.fEnd);

        Ok(())
    }
    // Source: src/shaders/gradients/SkLinearGradient.cpp:70-73
    pub fn appendGradientStages(
        &self,
        _: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        _: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        _: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    ) {
        // Upstream linear gradients require no additional stage.
    }
}
// Source: src/shaders/gradients/SkLinearGradient.cpp:75-87
impl SkLinearGradient {
    /// Safety: info's nonnull color/offset pointers must have capacity declared in fColorCount.
    pub unsafe fn asGradient(
        &self,
        info: Option<&mut crate::src::shaders::SkShaderBase::GradientInfo>,
        local_matrix: Option<&mut SkMatrix>,
    ) -> crate::src::shaders::SkShaderBase::GradientType {
        if let Some(info) = info {
            unsafe {
                self.base.commonAsAGradient(info);
            }
            info.fPoint[0] = self.fStart;
            info.fPoint[1] = self.fEnd;
        }
        if let Some(matrix) = local_matrix {
            *matrix = SkMatrix::default();
        }
        crate::src::shaders::SkShaderBase::GradientType::kLinear
    }
}

// Source: src/shaders/gradients/SkLinearGradient.cpp:88-107
pub fn LinearGradient(
    pts: Option<&[SkPoint; 2]>,
    grad: &SkGradient<'_>,
    lm: Option<&SkMatrix>,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    let pts = pts?;
    let length = (pts[1] - pts[0]).length();
    if !length.is_finite() {
        return None;
    }
    super::SkGradientBaseShader::gradient_factory_early_exit!(grad, lm);
    if length.abs() <= SkGradientBaseShader::kDegenerateThreshold {
        return SkGradientBaseShader::MakeDegenerateGradient(grad.colors());
    }
    let shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> =
        std::sync::Arc::new(SkLinearGradient::new(pts, grad));
    Some(shader.makeWithLocalMatrix(lm.unwrap_or(SkMatrix::I())))
}

// Source: src/shaders/gradients/SkLinearGradient.cpp:109-111
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterLinearGradientShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkLinearGradient",
            SkLinearGradient::CreateProc,
        );
    }
}

// Rust delegation of the native inherited shader interfaces; the algorithms
// remain in the existing concrete methods and base pipeline implementation.
impl super::SkGradientBaseShader::SkGradientShader for SkLinearGradient {
    fn gradient_base(&self) -> &SkGradientBaseShader {
        &self.base
    }
    unsafe fn append_gradient_stages(
        &self,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        pipeline: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        post: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    ) {
        self.appendGradientStages(alloc, pipeline, post);
    }
}
impl crate::src::shaders::SkShader::SkShader for SkLinearGradient {
    fn isOpaque(&self) -> bool {
        self.base.isOpaque()
    }
}
impl crate::src::shaders::SkShaderBase::SkShaderBase for SkLinearGradient {
    fn r#type(&self) -> crate::src::shaders::SkShaderBase::ShaderType {
        self.base.r#type()
    }
    unsafe fn asGradient(
        &self,
        info: Option<&mut crate::src::shaders::SkShaderBase::GradientInfo>,
        matrix: Option<&mut SkMatrix>,
    ) -> crate::src::shaders::SkShaderBase::GradientType {
        unsafe { SkLinearGradient::asGradient(self, info, matrix) }
    }
    fn onAsLuminanceColor(&self, color: &mut crate::src::core::SkColor::SkColor4f) -> bool {
        self.base.onAsLuminanceColor(color)
    }
    unsafe fn appendStages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        mrec: &crate::src::shaders::SkShaderBase::MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        super::SkGradientBaseShader::SkGradientShader::append_gradient_pipeline(self, rec, mrec)
    }
}
