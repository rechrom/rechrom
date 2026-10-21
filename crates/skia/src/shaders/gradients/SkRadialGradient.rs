// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2012 Google Inc. BSD-style license.
use super::SkGradientBaseShader::SkGradientBaseShader;
use crate::include::effects::SkGradient::SkGradient;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkPoint;
// Source: src/shaders/gradients/SkRadialGradient.h:27-28
impl SkRadialGradient {
    pub fn center(&self) -> &SkPoint {
        &self.fCenter
    }
    pub fn radius(&self) -> f32 {
        self.fRadius
    }
}
// Source: src/shaders/gradients/SkRadialGradient.h:38-38
impl crate::src::core::SkFlattenable::SkFlattenableView for SkRadialGradient {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkRadialGradient")
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
// Source: src/shaders/gradients/SkRadialGradient.h:40-41
pub struct SkRadialGradient {
    base: SkGradientBaseShader,
    fCenter: SkPoint,
    fRadius: f32,
}
// Source: src/shaders/gradients/SkRadialGradient.cpp:29-36
pub fn rad_to_unit_matrix(center: &SkPoint, radius: f32) -> SkMatrix {
    let inv = 1.0 / radius;
    let mut matrix = SkMatrix::default();
    matrix.setTranslate(-center.fX, -center.fY);
    matrix.postScale(inv, inv);
    matrix
}
// Source: src/shaders/gradients/SkRadialGradient.cpp:38-41
impl SkRadialGradient {
    pub fn new(center: SkPoint, radius: f32, desc: &SkGradient<'_>) -> Self {
        Self {
            base: SkGradientBaseShader::new(desc, &rad_to_unit_matrix(&center, radius)),
            fCenter: center,
            fRadius: radius,
        }
    }
    // Source: src/shaders/gradients/SkRadialGradient.cpp:43-55
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
            info.fPoint[0] = self.fCenter;
            info.fRadius[0] = self.fRadius;
        }
        if let Some(matrix) = local_matrix {
            *matrix = SkMatrix::default();
        }
        crate::src::shaders::SkShaderBase::GradientType::kRadial
    }
    // Source: src/shaders/gradients/SkRadialGradient.cpp:56-69
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
        let center = buffer.readPoint();
        let radius = buffer.readScalar();
        let Some(gradient) = descriptor.typed() else {
            return Ok(None);
        };
        Ok(RadialGradient(center, radius, &gradient, local)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/gradients/SkRadialGradient.h:31-31
    // Source: src/shaders/gradients/SkRadialGradient.cpp:71-75
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writePoint(&self.fCenter);
        buffer.writeScalar(self.fRadius);

        Ok(())
    }
    // Source: src/shaders/gradients/SkRadialGradient.cpp:77-80
    /// Safety: pipeline must be valid and mutable during stage construction.
    pub unsafe fn appendGradientStages(
        &self,
        _: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        p: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        _: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    ) {
        unsafe {
            (*p).append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::xy_to_radius,
                std::ptr::null_mut(),
            );
        }
    }
}

// Source: src/shaders/gradients/SkRadialGradient.cpp:82-97
pub fn RadialGradient(
    center: SkPoint,
    radius: f32,
    grad: &SkGradient<'_>,
    lm: Option<&SkMatrix>,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    if radius < 0.0 {
        return None;
    }
    super::SkGradientBaseShader::gradient_factory_early_exit!(grad, lm);
    if radius.abs() <= SkGradientBaseShader::kDegenerateThreshold {
        return SkGradientBaseShader::MakeDegenerateGradient(grad.colors());
    }
    let shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> =
        std::sync::Arc::new(SkRadialGradient::new(center, radius, grad));
    Some(shader.makeWithLocalMatrix(lm.unwrap_or(SkMatrix::I())))
}

// Source: src/shaders/gradients/SkRadialGradient.cpp:99-101
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterRadialGradientShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkRadialGradient",
            SkRadialGradient::CreateProc,
        );
    }
}

// Rust delegation of the native inherited shader interfaces; the algorithms
// remain in the existing concrete methods and base pipeline implementation.
impl super::SkGradientBaseShader::SkGradientShader for SkRadialGradient {
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
impl crate::src::shaders::SkShader::SkShader for SkRadialGradient {
    fn isOpaque(&self) -> bool {
        self.base.isOpaque()
    }
}
impl crate::src::shaders::SkShaderBase::SkShaderBase for SkRadialGradient {
    fn r#type(&self) -> crate::src::shaders::SkShaderBase::ShaderType {
        self.base.r#type()
    }
    unsafe fn asGradient(
        &self,
        info: Option<&mut crate::src::shaders::SkShaderBase::GradientInfo>,
        matrix: Option<&mut SkMatrix>,
    ) -> crate::src::shaders::SkShaderBase::GradientType {
        unsafe { SkRadialGradient::asGradient(self, info, matrix) }
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
