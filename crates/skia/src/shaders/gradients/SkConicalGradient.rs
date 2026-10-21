// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2012 Google Inc. BSD-style license.
use super::SkGradientBaseShader::SkGradientBaseShader;
use crate::include::effects::SkGradient::SkGradient;
use crate::src::core::SkArenaAlloc::SkArenaAlloc;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRasterPipeline::SkRasterPipeline;
use crate::src::core::SkRasterPipelineOpContexts::Conical2PtCtx;
use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;

// Source: src/shaders/gradients/SkConicalGradient.h:30-33
#[derive(Clone, Copy)]
pub struct FocalData {
    pub fR1: f32,
    pub fFocalX: f32,
    pub fIsSwapped: bool,
}
// Source: src/shaders/gradients/SkConicalGradient.h:46-50
impl FocalData {
    pub fn isFocalOnCircle(&self) -> bool {
        (1.0 - self.fR1).abs() <= 1.0 / 4096.0
    }
    pub fn isSwapped(&self) -> bool {
        self.fIsSwapped
    }
    pub fn isWellBehaved(&self) -> bool {
        !self.isFocalOnCircle() && self.fR1 > 1.0
    }
    pub fn isNativelyFocal(&self) -> bool {
        self.fFocalX.abs() <= 1.0 / 4096.0
    }
}
// Source: src/shaders/gradients/SkConicalGradient.h:53-53
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    kRadial,
    kStrip,
    kFocal,
}
// Source: src/shaders/gradients/SkConicalGradient.h:68-76
impl SkConicalGradient {
    pub fn getCenterX1(&self) -> f32 {
        (self.fCenter1 - self.fCenter2).length()
    }
    pub fn getStartRadius(&self) -> f32 {
        self.fRadius1
    }
    pub fn getDiffRadius(&self) -> f32 {
        self.fRadius2 - self.fRadius1
    }
    pub fn getStartCenter(&self) -> &SkPoint {
        &self.fCenter1
    }
    pub fn getEndCenter(&self) -> &SkPoint {
        &self.fCenter2
    }
    pub fn getEndRadius(&self) -> f32 {
        self.fRadius2
    }
    pub fn getType(&self) -> Type {
        self.fType
    }
    pub fn getFocalData(&self) -> Option<&FocalData> {
        self.fFocalData.as_ref()
    }
}
// Source: src/shaders/gradients/SkConicalGradient.h:96-96
impl crate::src::core::SkFlattenable::SkFlattenableView for SkConicalGradient {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkConicalGradient")
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
// Source: src/shaders/gradients/SkConicalGradient.h:98-104
pub struct SkConicalGradient {
    base: SkGradientBaseShader,
    fCenter1: SkPoint,
    fCenter2: SkPoint,
    fRadius1: f32,
    fRadius2: f32,
    fType: Type,
    // C++ leaves focal data uninitialized for nonfocal variants; Option prevents reading it there.
    fFocalData: Option<FocalData>,
}
// Source: src/shaders/gradients/SkConicalGradient.cpp:32-63
impl FocalData {
    pub fn set(&mut self, mut r0: f32, mut r1: f32, matrix: &mut SkMatrix) -> bool {
        self.fIsSwapped = false;
        self.fFocalX = r0 / (r0 - r1);
        if (self.fFocalX - 1.0).abs() <= 1.0 / 4096.0 {
            matrix.postTranslate(-1.0, 0.0);
            matrix.postScale(-1.0, 1.0);
            std::mem::swap(&mut r0, &mut r1);
            self.fFocalX = 0.0;
            self.fIsSwapped = true;
        }
        let from = [
            SkPoint {
                fX: self.fFocalX,
                fY: 0.0,
            },
            SkPoint { fX: 1.0, fY: 0.0 },
        ];
        let to = [SkPoint { fX: 0.0, fY: 0.0 }, SkPoint { fX: 1.0, fY: 0.0 }];
        let Some(focal_matrix) = SkMatrix::PolyToPoly(&from, &to) else {
            return false;
        };
        matrix.postConcat(&focal_matrix);
        self.fR1 = r1 / (1.0 - self.fFocalX).abs();
        if self.isFocalOnCircle() {
            matrix.postScale(0.5, 0.5);
        } else {
            matrix.postScale(
                self.fR1 / (self.fR1 * self.fR1 - 1.0),
                1.0 / (self.fR1 * self.fR1 - 1.0).abs().sqrt(),
            );
        }
        matrix.postScale((1.0 - self.fFocalX).abs(), (1.0 - self.fFocalX).abs());
        true
    }
}
// Source: src/shaders/gradients/SkConicalGradient.cpp:65-70
impl SkConicalGradient {
    pub fn MapToUnitX(start: &SkPoint, end: &SkPoint) -> Option<SkMatrix> {
        SkMatrix::PolyToPoly(
            &[*start, *end],
            &[SkPoint { fX: 0.0, fY: 0.0 }, SkPoint { fX: 1.0, fY: 0.0 }],
        )
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:72-110
    // Native Create geometry prefix, shared by the complete retained constructor below.
    pub fn prepare_geometry(
        c0: SkPoint,
        r0: f32,
        c1: SkPoint,
        r1: f32,
    ) -> Option<(SkMatrix, Type, Option<FocalData>)> {
        let mut matrix;
        let kind;
        if (c0 - c1).length().abs() <= 1.0 / 4096.0 {
            let max_r = if r0 < r1 { r1 } else { r0 };
            if max_r.abs() <= 1.0 / 4096.0 || (r0 - r1).abs() <= 1.0 / 4096.0 {
                return None;
            }
            let scale = 1.0 / max_r;
            matrix = SkMatrix::Translate(-c1.fX, -c1.fY);
            matrix.postScale(scale, scale);
            kind = Type::kRadial;
        } else {
            matrix = Self::MapToUnitX(&c0, &c1)?;
            kind = if (r1 - r0).abs() <= 1.0 / 4096.0 {
                Type::kStrip
            } else {
                Type::kFocal
            };
        }
        let focal = if kind == Type::kFocal {
            let distance = (c0 - c1).length();
            let mut focal = FocalData {
                fR1: 0.0,
                fFocalX: 0.0,
                fIsSwapped: false,
            };
            if !focal.set(r0 / distance, r1 / distance, &mut matrix) {
                return None;
            }
            Some(focal)
        } else {
            None
        };
        Some((matrix, kind, focal))
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:112-115
    pub fn Create(
        c0: SkPoint,
        r0: f32,
        c1: SkPoint,
        r1: f32,
        desc: &SkGradient<'_>,
        local_matrix: Option<&SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        let (matrix, kind, focal) = Self::prepare_geometry(c0, r0, c1, r1)?;
        let shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> =
            std::sync::Arc::new(Self::new(c0, r0, c1, r1, desc, kind, &matrix, focal));
        Some(shader.makeWithLocalMatrix(local_matrix.unwrap_or(SkMatrix::I())))
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:117-136
    pub fn new(
        start: SkPoint,
        startRadius: f32,
        end: SkPoint,
        endRadius: f32,
        desc: &SkGradient<'_>,
        kind: Type,
        matrix: &SkMatrix,
        data: Option<FocalData>,
    ) -> Self {
        crate::SkASSERT!(@source b"src/shaders/gradients/SkConicalGradient.cpp\0".as_ptr(),132; @condition b"fCenter1 != fCenter2 || fRadius1 != fRadius2\0".as_ptr(); start != end || startRadius != endRadius);
        debug_assert!(kind != Type::kFocal || data.is_some());
        Self {
            base: SkGradientBaseShader::new(desc, matrix),
            fCenter1: start,
            fCenter2: end,
            fRadius1: startRadius,
            fRadius2: endRadius,
            fType: kind,
            fFocalData: if kind == Type::kFocal { data } else { None },
        }
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:138-143
    pub fn isOpaque(&self) -> bool {
        false
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:146-159
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
            info.fPoint[0] = self.fCenter1;
            info.fPoint[1] = self.fCenter2;
            info.fRadius[0] = self.fRadius1;
            info.fRadius[1] = self.fRadius2;
        }
        if let Some(matrix) = local_matrix {
            *matrix = SkMatrix::default();
        }
        crate::src::shaders::SkShaderBase::GradientType::kConical
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:161-180
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
        let first = buffer.readPoint();
        let second = buffer.readPoint();
        let first_radius = buffer.readScalar();
        let second_radius = buffer.readScalar();
        if !buffer.isValid() {
            return Ok(None);
        }
        let Some(gradient) = descriptor.typed() else {
            return Ok(None);
        };
        Ok(
            TwoPointConicalGradient(first, first_radius, second, second_radius, &gradient, local)
                .map(crate::src::core::SkFlattenable::FlattenableOwner::Shader),
        )
    }
    // Source: src/shaders/gradients/SkConicalGradient.h:88-88
    // Source: src/shaders/gradients/SkConicalGradient.cpp:182-188
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writePoint(&self.fCenter1);
        buffer.writePoint(&self.fCenter2);
        buffer.writeScalar(self.fRadius1);
        buffer.writeScalar(self.fRadius2);

        Ok(())
    }
    // Source: src/shaders/gradients/SkConicalGradient.cpp:190-245
    /// Safety: alloc and both pipeline pointers must stay alive until execution finishes.
    pub unsafe fn appendGradientStages(
        &self,
        alloc: *mut SkArenaAlloc,
        p: *mut SkRasterPipeline,
        post: *mut SkRasterPipeline,
    ) {
        unsafe {
            let d_radius = self.fRadius2 - self.fRadius1;
            if self.fType == Type::kRadial {
                (*p).append(Op::xy_to_radius, std::ptr::null_mut());
                let max_r = if self.fRadius1 < self.fRadius2 {
                    self.fRadius2
                } else {
                    self.fRadius1
                };
                let scale = max_r / d_radius;
                let bias = -self.fRadius1 / d_radius;
                let mut matrix = SkMatrix::default();
                matrix.setConcat(
                    &SkMatrix::Translate(bias, 0.0),
                    &SkMatrix::Scale(scale, 1.0),
                );
                (*p).appendMatrix(alloc, &matrix);
                return;
            }
            let ctx = (*alloc)
                .makeBytesAlignedTo(
                    std::mem::size_of::<Conical2PtCtx>(),
                    std::mem::align_of::<Conical2PtCtx>(),
                )
                .cast::<Conical2PtCtx>();
            // Rust adaptation: upstream POD allocation leaves unused members unset.
            // Initialize the complete value before any stage forms a whole-context reference.
            ctx.write(Conical2PtCtx {
                fMask: [0; 16],
                fP0: 0.0,
                fP1: 0.0,
            });
            if self.fType == Type::kStrip {
                let scaled_r0 = self.fRadius1 / self.getCenterX1();
                (*ctx).fP0 = scaled_r0 * scaled_r0;
                (*p).append(Op::xy_to_2pt_conical_strip, ctx.cast());
                (*p).append(Op::mask_2pt_conical_nan, ctx.cast());
                (*post).append(
                    Op::apply_vector_mask,
                    std::ptr::addr_of_mut!((*ctx).fMask).cast(),
                );
                return;
            }
            let focal = self
                .fFocalData
                .as_ref()
                .expect("focal data belongs to focal variant");
            (*ctx).fP0 = 1.0 / focal.fR1;
            (*ctx).fP1 = focal.fFocalX;
            if focal.isFocalOnCircle() {
                (*p).append(Op::xy_to_2pt_conical_focal_on_circle, std::ptr::null_mut());
            } else if focal.isWellBehaved() {
                (*p).append(Op::xy_to_2pt_conical_well_behaved, ctx.cast());
            } else if focal.isSwapped() || 1.0 - focal.fFocalX < 0.0 {
                (*p).append(Op::xy_to_2pt_conical_smaller, ctx.cast());
            } else {
                (*p).append(Op::xy_to_2pt_conical_greater, ctx.cast());
            }
            if !focal.isWellBehaved() {
                (*p).append(Op::mask_2pt_conical_degenerates, ctx.cast());
            }
            if 1.0 - focal.fFocalX < 0.0 {
                (*p).append(Op::negate_x, std::ptr::null_mut());
            }
            if !focal.isNativelyFocal() {
                (*p).append(Op::alter_2pt_conical_compensate_focal, ctx.cast());
            }
            if focal.isSwapped() {
                (*p).append(Op::alter_2pt_conical_unswap, std::ptr::null_mut());
            }
            if !focal.isWellBehaved() {
                (*post).append(
                    Op::apply_vector_mask,
                    std::ptr::addr_of_mut!((*ctx).fMask).cast(),
                );
            }
        }
    }
}

// Source: src/shaders/gradients/SkConicalGradient.cpp:247-308
pub fn TwoPointConicalGradient(
    start: SkPoint,
    start_radius: f32,
    end: SkPoint,
    end_radius: f32,
    gradient: &SkGradient<'_>,
    local_matrix: Option<&SkMatrix>,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    use crate::include::core::SkTileMode::SkTileMode;
    use crate::include::effects::SkGradient::Colors;
    if start_radius < 0.0 || end_radius < 0.0 {
        return None;
    }
    let colors = gradient.colors();
    let interpolation = *gradient.interpolation();
    if !SkGradientBaseShader::ValidGradient(colors.colors(), colors.tileMode(), &interpolation) {
        return None;
    }
    if let Some(matrix) = local_matrix {
        if matrix.invert().is_none() {
            return None;
        }
    }
    let threshold = SkGradientBaseShader::kDegenerateThreshold;
    if (start - end).length().abs() <= threshold {
        if (start_radius - end_radius).abs() <= threshold {
            if colors.tileMode() == SkTileMode::kClamp && end_radius > threshold {
                let source = colors.colors();
                let repeated = [source[0], source[0], source[source.len() - 1]];
                let positions = [0.0, 1.0, 1.0];
                let colors = Colors::new(
                    &repeated,
                    &positions,
                    colors.tileMode(),
                    colors.colorSpace().clone(),
                );
                return super::SkRadialGradient::RadialGradient(
                    start,
                    end_radius,
                    &SkGradient::new(colors, interpolation),
                    local_matrix,
                );
            } else {
                return SkGradientBaseShader::MakeDegenerateGradient(colors);
            }
        } else if start_radius.abs() <= threshold {
            return super::SkRadialGradient::RadialGradient(
                start,
                end_radius,
                gradient,
                local_matrix,
            );
        }
    }
    let mut source = colors.colors();
    let mut positions = colors.positions();
    let repeated;
    if source.len() == 1 {
        repeated = [source[0], source[0]];
        source = &repeated;
        positions = &[];
    }
    let colors = Colors::new(
        source,
        positions,
        colors.tileMode(),
        colors.colorSpace().clone(),
    );
    SkConicalGradient::Create(
        start,
        start_radius,
        end,
        end_radius,
        &SkGradient::new(colors, interpolation),
        local_matrix,
    )
}

// Source: src/shaders/gradients/SkConicalGradient.cpp:311-315
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterConicalGradientShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkConicalGradient",
            SkConicalGradient::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkTwoPointConicalGradient",
            SkConicalGradient::CreateProc,
        );
    }
}

// Rust delegation of the native inherited shader interfaces; the algorithms
// remain in the existing concrete methods and base pipeline implementation.
impl super::SkGradientBaseShader::SkGradientShader for SkConicalGradient {
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
impl crate::src::shaders::SkShader::SkShader for SkConicalGradient {
    fn isOpaque(&self) -> bool {
        SkConicalGradient::isOpaque(self)
    }
}
impl crate::src::shaders::SkShaderBase::SkShaderBase for SkConicalGradient {
    fn r#type(&self) -> crate::src::shaders::SkShaderBase::ShaderType {
        self.base.r#type()
    }
    unsafe fn asGradient(
        &self,
        info: Option<&mut crate::src::shaders::SkShaderBase::GradientInfo>,
        matrix: Option<&mut SkMatrix>,
    ) -> crate::src::shaders::SkShaderBase::GradientType {
        unsafe { SkConicalGradient::asGradient(self, info, matrix) }
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
