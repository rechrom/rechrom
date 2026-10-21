// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2023 Google LLC. BSD-style license.
use crate::src::core::{
    SkBlendMode::{SkBlendMode, SkBlendModeCoeff, SkBlendMode_AsCoeff},
    SkEffectPriv::SkStageRec,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;

impl SkBlendShader {
    // Source: src/shaders/SkBlendShader.h:25-26
    pub fn new(mode: SkBlendMode, dst: Arc<dyn SkShader>, src: Arc<dyn SkShader>) -> Self {
        Self {
            fDst: dst,
            fSrc: src,
            fMode: mode,
        }
    }
    // Source: src/shaders/SkBlendShader.h:32-34
    pub fn dst(&self) -> Arc<dyn SkShader> {
        self.fDst.clone()
    }
    pub fn src(&self) -> Arc<dyn SkShader> {
        self.fSrc.clone()
    }
    pub fn mode(&self) -> SkBlendMode {
        self.fMode
    }
}
// Source: src/shaders/SkBlendShader.h:45-47
// Source: src/shaders/SkBlendShader.h:23-23
pub struct SkBlendShader {
    // Source: src/shaders/SkBlendShader.h:47-47
    fMode: SkBlendMode,
    // Source: src/shaders/SkBlendShader.h:46-46
    fSrc: Arc<dyn SkShader>,
    // Source: src/shaders/SkBlendShader.h:45-45
    fDst: Arc<dyn SkShader>,
}

// Source: src/shaders/SkBlendShader.h:43-43
impl crate::src::core::SkFlattenable::SkFlattenableView for SkBlendShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkBlendShader")
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
// Source: src/shaders/SkBlendShader.cpp:29-49
impl SkBlendShader {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let dst = buffer.readShader()?;
        let src = buffer.readShader()?;
        if !buffer.validate(dst.is_some() && src.is_some()) {
            return Ok(None);
        }
        let mode = buffer.read32() as u32;
        let shader = if mode == crate::src::core::SkBlendModePriv::kCustom_SkBlendMode as u32 {
            let blender = buffer.readBlender()?;
            if !buffer.validate(blender.is_some()) {
                return Ok(None);
            }
            BlendWithBlender(blender, dst, src)
        } else {
            if !buffer.validate(mode <= SkBlendMode::kLastMode as u32) {
                return Ok(None);
            }
            let mode: SkBlendMode = unsafe { std::mem::transmute(mode as u8) };
            Blend(mode, dst, src)
        };
        Ok(shader.map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
}
// Source: src/shaders/SkBlendShader.cpp:51-54
// Source: src/shaders/SkBlendShader.cpp:58-65
// Source: src/shaders/SkBlendShader.cpp:67-67
impl SkShader for SkBlendShader {
    // Source: src/shaders/SkBlendShader.h:30-30
    fn isOpaque(&self) -> bool {
        blend_is_opaque(self.fMode, self.fSrc.isOpaque(), self.fDst.isOpaque())
    }
}

// Source: src/shaders/SkBlendShader.cpp:70-74
// Source: src/shaders/SkBlendShader.h:38-38
// Source: src/shaders/SkBlendShader.h:38
impl SkBlendShader {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(Some(&*self.fDst))?;
        buffer.writeFlattenable(Some(&*self.fSrc))?;
        buffer.write32(self.fMode as i32);
        Ok(())
    }
}
// Source: src/shaders/SkBlendShader.cpp:77-83
// Source: src/shaders/SkBlendShader.cpp:85-85
// Source: src/shaders/SkBlendShader.cpp:91-92
// Source: src/shaders/SkBlendShader.cpp:94-95
// Source: src/shaders/SkBlendShader.cpp:97-97
// Source: src/shaders/SkBlendShader.cpp:99-100
// Source: src/shaders/SkBlendShader.cpp:102-103
// Source: src/shaders/SkBlendShader.cpp:105-105
unsafe fn append_two_shaders(
    rec: &SkStageRec<'_>,
    mrec: &MatrixRec,
    s0: &dyn SkShader,
    s1: &dyn SkShader,
) -> Result<Option<*mut f32>, crate::RasterError> {
    use crate::src::core::SkRasterPipelineOpContexts::kMaxStride;
    struct Storage {
        fCoords: [f32; 2 * kMaxStride],
        fRes0: [f32; 4 * kMaxStride],
    }
    let storage = unsafe {
        (*rec.fAlloc).make(Storage {
            fCoords: [0.0; 2 * kMaxStride],
            fRes0: [0.0; 4 * kMaxStride],
        })
    };
    let coords = unsafe { core::ptr::addr_of_mut!((*storage).fCoords).cast::<f32>() };
    let res0 = unsafe { core::ptr::addr_of_mut!((*storage).fRes0).cast::<f32>() };
    if mrec.rasterPipelineCoordsAreSeeded() {
        unsafe {
            (*rec.fPipeline).append(Op::store_src_rg, coords.cast());
        }
    }
    if !unsafe { s0.appendStages(rec, mrec) }? {
        return Ok(None);
    }
    unsafe {
        (*rec.fPipeline).append(Op::store_src, res0.cast());
    }
    if mrec.rasterPipelineCoordsAreSeeded() {
        unsafe {
            (*rec.fPipeline).append(Op::load_src_rg, coords.cast());
        }
    }
    if !unsafe { s1.appendStages(rec, mrec) }? {
        return Ok(None);
    }
    Ok(Some(res0))
}

// Source: src/shaders/SkBlendShader.cpp:108-111
// Source: src/shaders/SkBlendShader.cpp:114-116
impl SkShaderBase for SkBlendShader {
    // Source: src/shaders/SkBlendShader.h:28-28
    fn r#type(&self) -> ShaderType {
        ShaderType::kBlend
    }
    // Source: src/shaders/SkBlendShader.h:39-39
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        let Some(res0) =
            (unsafe { append_two_shaders(rec, mrec, self.fDst.as_ref(), self.fSrc.as_ref()) })?
        else {
            return Ok(false);
        };
        unsafe {
            (*rec.fPipeline).append(Op::load_dst, res0.cast());
            crate::src::core::SkBlendMode::SkBlendMode_AppendStages(
                self.fMode,
                &mut *rec.fPipeline,
            );
        }
        Ok(true)
    }
}

// Source: src/shaders/SkBlendShader.cpp:119-121
// Source: src/shaders/SkBlendShader.cpp:123-131
// Source: src/shaders/SkBlendShader.cpp:133-133
pub fn Blend(
    mode: SkBlendMode,
    dst: Option<Arc<dyn SkShader>>,
    src: Option<Arc<dyn SkShader>>,
) -> Option<Arc<dyn SkShader>> {
    if src.is_none() || dst.is_none() {
        return None;
    }
    match mode {
        SkBlendMode::kClear => crate::src::shaders::SkColorShader::Color(0),
        SkBlendMode::kDst => dst,
        SkBlendMode::kSrc => src,
        _ => Some(Arc::new(SkBlendShader::new(
            mode,
            dst.unwrap(),
            src.unwrap(),
        ))),
    }
}
// Source: src/shaders/SkBlendShader.cpp:136-156
pub fn BlendWithBlender(
    blender: crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
    dst: Option<Arc<dyn SkShader>>,
    src: Option<Arc<dyn SkShader>>,
) -> Option<Arc<dyn SkShader>> {
    if src.is_none() || dst.is_none() {
        return None;
    }
    if !blender.is_some() {
        return Blend(SkBlendMode::kSrcOver, dst, src);
    }
    if let Some(mode) = blender.get().unwrap().asBlendMode() {
        return Blend(mode, dst, src);
    }
    // Preserve the actual runtime-effect dependency; no approximate blend fallback.
    let effect = crate::src::core::SkKnownRuntimeEffects::GetKnownRuntimeEffect(
        crate::src::core::SkKnownRuntimeEffects::StableKey::kBlend,
    )
    .expect("native known blend effect");
    let children = [
        crate::src::core::SkRuntimeEffect::ChildPtr::Shader(src.unwrap()),
        crate::src::core::SkRuntimeEffect::ChildPtr::Shader(dst.unwrap()),
        crate::src::core::SkRuntimeEffect::ChildPtr::Blender(blender),
    ];
    effect.makeShader(None, &children, None)
}
// Source: src/shaders/SkBlendShader.cpp:158-162
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterBlendShaderFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(b"SkBlendShader", SkBlendShader::CreateProc);
        crate::src::core::SkFlattenable::Register(b"SkShader_Blend", SkBlendShader::CreateProc);
    }
}

// Existing native opacity algorithm factored for the operation-borrowed view;
// no new source credit and no second implementation of the blend equations.
fn blend_is_opaque(mode: SkBlendMode, src_is_opaque: bool, dst_is_opaque: bool) -> bool {
    let Some((src_coeff, dst_coeff)) = SkBlendMode_AsCoeff(mode) else {
        return false;
    };
    let coeff_is_opaque = |coeff: SkBlendModeCoeff| {
        let src_alpha = matches!(coeff, SkBlendModeCoeff::kSA | SkBlendModeCoeff::kSC);
        let dst_alpha = matches!(coeff, SkBlendModeCoeff::kDA | SkBlendModeCoeff::kDC);
        coeff == SkBlendModeCoeff::kOne
            || (src_alpha && src_is_opaque)
            || (dst_alpha && dst_is_opaque)
    };
    (src_is_opaque && coeff_is_opaque(src_coeff)) || (dst_is_opaque && coeff_is_opaque(dst_coeff))
}

// Operation-only stage projection. This is not a retained global shader owner
// and has no fabricated flatten/serialization methods or Arc/static lifetime.
pub trait ShaderStageView {
    fn stage_is_opaque(&self) -> bool;
    fn stage_is_constant(&self, color: Option<&mut crate::src::core::SkColor::SkColor4f>) -> bool;
    unsafe fn append_stage_view(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError>;
    unsafe fn append_root_stage_view(
        &self,
        rec: &SkStageRec<'_>,
        ctm: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.append_stage_view(rec, &MatrixRec::new(ctm)) }
    }
}
pub struct BorrowedBlendStage<'shader> {
    pub blender: &'shader dyn crate::include::core::SkBlender::SkBlender,
    pub dst: &'shader dyn SkShader,
    pub src: &'shader dyn SkShader,
}
pub enum BorrowedShaderStage<'shader> {
    Shader(&'shader dyn SkShader),
    Blend(BorrowedBlendStage<'shader>),
}
impl ShaderStageView for BorrowedShaderStage<'_> {
    fn stage_is_opaque(&self) -> bool {
        match self {
            Self::Shader(shader) => shader.isOpaque(),
            Self::Blend(blend) => match blend.blender.asBlendMode() {
                Some(mode) => blend_is_opaque(mode, blend.src.isOpaque(), blend.dst.isOpaque()),
                None => false, // Native runtime blend Shader is not flagged opaque.
            },
        }
    }
    fn stage_is_constant(&self, color: Option<&mut crate::src::core::SkColor::SkColor4f>) -> bool {
        match self {
            Self::Shader(shader) => shader.isConstant(color),
            Self::Blend(blend) => match blend.blender.asBlendMode() {
                Some(SkBlendMode::kClear) => {
                    crate::src::shaders::SkColorShader::SkColorShader::new(
                        &crate::src::core::SkColor::SkColors::kTransparent,
                    )
                    .isConstant(color)
                }
                Some(SkBlendMode::kDst) => blend.dst.isConstant(color),
                Some(SkBlendMode::kSrc) => blend.src.isConstant(color),
                _ => false,
            },
        }
    }
    /// Safety: all actual child/blender owners and arena contexts outlive every
    /// blit; the view is used synchronously for stage assembly. No mutable shader
    /// update may overlap append or execution. Only the actual native children
    /// and blender append algorithms emit stages; no interpolation is duplicated.
    unsafe fn append_stage_view(
        &self,
        rec: &SkStageRec<'_>,
        mrec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        let Self::Blend(blend) = self else {
            if let Self::Shader(shader) = self {
                return unsafe { shader.appendStages(rec, mrec) };
            }
            unreachable!();
        };
        let mode = blend.blender.asBlendMode();
        match mode {
            Some(SkBlendMode::kClear) => {
                return unsafe {
                    crate::src::shaders::SkColorShader::SkColorShader::new(
                        &crate::src::core::SkColor::SkColors::kTransparent,
                    )
                    .appendStages(rec, mrec)
                }
            }
            Some(SkBlendMode::kDst) => return unsafe { blend.dst.appendStages(rec, mrec) },
            Some(SkBlendMode::kSrc) => return unsafe { blend.src.appendStages(rec, mrec) },
            _ => {}
        }
        let Some(res0) = (unsafe { append_two_shaders(rec, mrec, blend.dst, blend.src) })? else {
            return Ok(false);
        };
        unsafe {
            (*rec.fPipeline).append(Op::load_dst, res0.cast());
        }
        if let Some(mode) = mode {
            unsafe {
                crate::src::core::SkBlendMode::SkBlendMode_AppendStages(mode, &mut *rec.fPipeline);
            }
            Ok(true)
        } else {
            // Same actual SkBlender CPU append contract used by the native
            // runtime blend child; the two shader results are in src/dst slots.
            unsafe { blend.blender.appendStages(rec) }
        }
    }
}

// Source: src/shaders/SkBlendShader.h:37
// Native protected ordinary declaration: SkBlendShader(SkReadBuffer&).
// No definition occurs in src/shaders/SkBlendShader.cpp. Rust has no bodyless ordinary
// inherent method with this adapted object representation; the declaration remains
// pending rather than inventing an extern ABI, pure-virtual trait, or constructor body.
