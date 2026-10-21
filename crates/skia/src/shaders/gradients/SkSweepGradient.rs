// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2012 Google Inc. BSD-style license.
use super::SkGradientBaseShader::SkGradientBaseShader;
use crate::include::effects::SkGradient::SkGradient;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkPoint;
impl SkSweepGradient {
    // Source: src/shaders/gradients/SkSweepGradient.h:27-27
    pub fn center(&self) -> &SkPoint {
        &self.fCenter
    }
    // Source: src/shaders/gradients/SkSweepGradient.h:28-29
    pub fn tBias(&self) -> f32 {
        self.fTBias
    }
    pub fn tScale(&self) -> f32 {
        self.fTScale
    }
}
// Source: src/shaders/gradients/SkSweepGradient.h:40-40
impl crate::src::core::SkFlattenable::SkFlattenableView for SkSweepGradient {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkSweepGradient")
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
// Source: src/shaders/gradients/SkSweepGradient.h:42-44
pub struct SkSweepGradient {
    base: SkGradientBaseShader,
    fCenter: SkPoint,
    fTBias: f32,
    fTScale: f32,
}
// Source: src/shaders/gradients/SkSweepGradient.cpp:33-42
impl SkSweepGradient {
    pub fn new(center: SkPoint, t0: f32, t1: f32, desc: &SkGradient<'_>) -> Self {
        let result = Self {
            base: SkGradientBaseShader::new(desc, &SkMatrix::Translate(-center.fX, -center.fY)),
            fCenter: center,
            fTBias: -t0,
            fTScale: 1.0 / (t1 - t0),
        };
        crate::SkASSERT!(@source b"src/shaders/gradients/SkSweepGradient.cpp\0".as_ptr(),41; @condition b"t0 < t1\0".as_ptr(); t0 < t1);
        result
    }
}
// Source: src/shaders/gradients/SkSweepGradient.cpp:44-56
impl SkSweepGradient {
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
            info.fPoint[1].fX = self.fTScale;
            info.fPoint[1].fY = self.fTBias;
        }
        if let Some(matrix) = local_matrix {
            *matrix = SkMatrix::default();
        }
        crate::src::shaders::SkShaderBase::GradientType::kSweep
    }
}
// Source: src/shaders/gradients/SkSweepGradient.cpp:58-60
pub fn angles_from_t_coeff(tBias: f32, tScale: f32) -> (f32, f32) {
    (-tBias * 360.0, (1.0 / tScale - tBias) * 360.0)
}
impl SkSweepGradient {
    // Source: src/shaders/gradients/SkSweepGradient.cpp:62-79
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
        let bias = buffer.readScalar();
        let scale = buffer.readScalar();
        let (start, end) = angles_from_t_coeff(bias, scale);
        let Some(gradient) = descriptor.typed() else {
            return Ok(None);
        };
        Ok(SweepGradient(center, start, end, &gradient, local)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/gradients/SkSweepGradient.h:32-32
    // Source: src/shaders/gradients/SkSweepGradient.cpp:81-86
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writePoint(&self.fCenter);
        buffer.writeScalar(self.fTBias);
        buffer.writeScalar(self.fTScale);

        Ok(())
    }
}
// Source: src/shaders/gradients/SkSweepGradient.cpp:88-92
impl SkSweepGradient {
    /// Safety: arena and pipeline must remain valid until appended stage execution finishes.
    pub unsafe fn appendGradientStages(
        &self,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        p: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
        _: *mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    ) {
        unsafe {
            (*p).append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::xy_to_unit_angle,
                std::ptr::null_mut(),
            );
            let mut matrix = SkMatrix::default();
            matrix.setConcat(
                &SkMatrix::Scale(self.fTScale, 1.0),
                &SkMatrix::Translate(self.fTBias, 0.0),
            );
            (*p).appendMatrix(alloc, &matrix);
        }
    }
}

// Source: src/shaders/gradients/SkSweepGradient.cpp:94-142
pub fn SweepGradient(
    center: SkPoint,
    startAngle: f32,
    endAngle: f32,
    grad: &SkGradient<'_>,
    lm: Option<&SkMatrix>,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    use crate::include::core::SkTileMode::SkTileMode;
    use crate::include::effects::SkGradient::Colors;
    if !startAngle.is_finite() || !endAngle.is_finite() || startAngle > endAngle {
        return None;
    }
    super::SkGradientBaseShader::gradient_factory_early_exit!(grad, lm);
    let colors = grad.colors();
    let interpolation = *grad.interpolation();
    let mut mode = colors.tileMode();
    if (startAngle - endAngle).abs() <= SkGradientBaseShader::kDegenerateThreshold {
        if mode == SkTileMode::kClamp && endAngle > SkGradientBaseShader::kDegenerateThreshold {
            let source = colors.colors();
            let repeated = [source[0], source[0], source[source.len() - 1]];
            let positions = [0.0, 1.0, 1.0];
            let colors = Colors::new(&repeated, &positions, mode, colors.colorSpace().clone());
            return SweepGradient(
                center,
                0.0,
                endAngle,
                &SkGradient::new(colors, interpolation),
                lm,
            );
        } else {
            return SkGradientBaseShader::MakeDegenerateGradient(colors);
        }
    }
    if startAngle <= 0.0 && endAngle >= 360.0 {
        mode = SkTileMode::kClamp;
    }
    let new_gradient = SkGradient::new(
        Colors::new(
            colors.colors(),
            colors.positions(),
            mode,
            colors.colorSpace().clone(),
        ),
        interpolation,
    );
    let t0 = startAngle / 360.0;
    let t1 = endAngle / 360.0;
    let shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> =
        std::sync::Arc::new(SkSweepGradient::new(center, t0, t1, &new_gradient));
    Some(shader.makeWithLocalMatrix(lm.unwrap_or(SkMatrix::I())))
}

// Source: src/shaders/gradients/SkSweepGradient.cpp:144-146
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterSweepGradientShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(b"SkSweepGradient", SkSweepGradient::CreateProc);
    }
}

// Rust delegation of the native inherited shader interfaces; the algorithms
// remain in the existing concrete methods and base pipeline implementation.
impl super::SkGradientBaseShader::SkGradientShader for SkSweepGradient {
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
impl crate::src::shaders::SkShader::SkShader for SkSweepGradient {
    fn isOpaque(&self) -> bool {
        self.base.isOpaque()
    }
}
impl crate::src::shaders::SkShaderBase::SkShaderBase for SkSweepGradient {
    fn r#type(&self) -> crate::src::shaders::SkShaderBase::ShaderType {
        self.base.r#type()
    }
    unsafe fn asGradient(
        &self,
        info: Option<&mut crate::src::shaders::SkShaderBase::GradientInfo>,
        matrix: Option<&mut SkMatrix>,
    ) -> crate::src::shaders::SkShaderBase::GradientType {
        unsafe { SkSweepGradient::asGradient(self, info, matrix) }
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
