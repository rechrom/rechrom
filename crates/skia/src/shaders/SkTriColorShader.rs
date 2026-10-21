use crate::src::core::{
    SkColorData::SkPMColor4f, SkEffectPriv::SkStageRec, SkMatrix::SkMatrix, SkPoint::SkPoint,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{MatrixRec, ShaderType, SkShaderBase},
};
use core::{cell::UnsafeCell, mem::MaybeUninit};
// Source: src/shaders/SkTriColorShader.h:41-42
#[derive(Clone, Copy)]
struct Matrix43 {
    fMat: [f32; 12],
}
// Source: src/shaders/SkTriColorShader.h:45-62
impl Matrix43 {
    fn setConcat(&mut self, a: Self, b: &SkMatrix) {
        crate::SkASSERT!(@source b"src/shaders/SkTriColorShader.h\0".as_ptr(),46; @condition b"!b.hasPerspective()\0".as_ptr(); !b.hasPerspective());
        self.fMat[0] = a.dot(0, b.getScaleX(), b.getSkewY());
        self.fMat[1] = a.dot(1, b.getScaleX(), b.getSkewY());
        self.fMat[2] = a.dot(2, b.getScaleX(), b.getSkewY());
        self.fMat[3] = a.dot(3, b.getScaleX(), b.getSkewY());
        self.fMat[4] = a.dot(0, b.getSkewX(), b.getScaleY());
        self.fMat[5] = a.dot(1, b.getSkewX(), b.getScaleY());
        self.fMat[6] = a.dot(2, b.getSkewX(), b.getScaleY());
        self.fMat[7] = a.dot(3, b.getSkewX(), b.getScaleY());
        self.fMat[8] = a.dot(0, b.getTranslateX(), b.getTranslateY()) + a.fMat[8];
        self.fMat[9] = a.dot(1, b.getTranslateX(), b.getTranslateY()) + a.fMat[9];
        self.fMat[10] = a.dot(2, b.getTranslateX(), b.getTranslateY()) + a.fMat[10];
        self.fMat[11] = a.dot(3, b.getTranslateX(), b.getTranslateY()) + a.fMat[11];
    }
    // Source: src/shaders/SkTriColorShader.h:65-67
    fn dot(&self, index: usize, x: f32, y: f32) -> f32 {
        self.fMat[index + 0] * x + self.fMat[index + 4] * y
    }
}
// Source: src/shaders/SkTriColorShader.h:18-18
// Source: src/shaders/SkTriColorShader.h:73-76
// Exact matrix payloads remain at stable owner addresses. UnsafeCell expresses
// native between-triangle mutation; it is not a mutex or an additional owner.
pub struct SkTriColorShader {
    // Source: src/shaders/SkTriColorShader.h:73-73
    fM43: UnsafeCell<MaybeUninit<Matrix43>>,
    // Source: src/shaders/SkTriColorShader.h:74-74
    fM33: UnsafeCell<SkMatrix>,
    // Source: src/shaders/SkTriColorShader.h:75-75
    fIsOpaque: bool,
    // Source: src/shaders/SkTriColorShader.h:76-76
    fUsePersp: bool,
}
// Safe methods touch only immutable flags. All matrix access and context
// publication is unsafe and requires the exclusive operation lease below.
// This does not allow safe callers to race update against pipeline execution.
unsafe impl Send for SkTriColorShader {}
unsafe impl Sync for SkTriColorShader {}
// Source: src/shaders/SkTriColorShader.h:20-20
impl SkTriColorShader {
    pub fn new(is_opaque: bool, use_persp: bool) -> Self {
        Self {
            fM43: UnsafeCell::new(MaybeUninit::uninit()),
            fM33: UnsafeCell::new(SkMatrix::default()),
            fIsOpaque: is_opaque,
            fUsePersp: use_persp,
        }
    }
}
// Source: src/shaders/SkTriColorShader.h:22-22
impl SkShaderBase for SkTriColorShader {
    fn r#type(&self) -> ShaderType {
        ShaderType::kTriColor
    }
    // Source: src/shaders/SkTriColorShader.h:33-33
    // Source: src/shaders/SkTriColorShader.cpp:19-26
    /// Safety: this exact shader remains live/pinned while its contexts execute;
    /// successful update initializes all matrices before first execution. No concurrent update,
    /// append, execution or matrix reference may overlap these matrix leases.
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        _: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            (*rec.fPipeline).append(Op::seed_shader, core::ptr::null_mut());
        }
        if self.fUsePersp {
            // Native &SkMatrix denotes its nine scalar payload values. Project
            // the actual Rust field, without assuming native or Rust ABI offsets.
            let matrix = unsafe { core::ptr::addr_of!((*self.fM33.get()).fMat) };
            unsafe {
                (*rec.fPipeline).append_const(Op::matrix_perspective, matrix.cast());
            }
        }
        let matrix43 = self.fM43.get().cast::<Matrix43>();
        let values = unsafe { core::ptr::addr_of!((*matrix43).fMat) };
        unsafe {
            (*rec.fPipeline).append_const(Op::matrix_4x3, values.cast());
        }
        Ok(true)
    }
}
// Source: src/shaders/SkTriColorShader.h:36-36
impl SkShader for SkTriColorShader {
    fn isOpaque(&self) -> bool {
        self.fIsOpaque
    }
}
// Source: src/shaders/SkTriColorShader.h:38-39
impl crate::src::core::SkFlattenable::SkFlattenableView for SkTriColorShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        None
    }
    fn getTypeName(&self) -> Option<&str> {
        None
    }
    // Inherited native flatten writes no additional fields; no new source credit.
    fn flatten(
        &self,
        _writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
}
// Source: src/shaders/SkTriColorShader.h:25-30
// Source: src/shaders/SkTriColorShader.cpp:28-59
impl SkTriColorShader {
    /// Safety: all indices address initialized positions/colors; matrix storage
    /// is exclusively leased between complete blits, with no concurrent update,
    /// stage append, execution or matrix access. Caller pins this actual owner
    /// before stages retain its matrix addresses, and keeps it live through use.
    pub unsafe fn update(
        &self,
        ctm_inv: &SkMatrix,
        pts: &[SkPoint],
        colors: &[SkPMColor4f],
        index0: i32,
        index1: i32,
        index2: i32,
    ) -> bool {
        let (i0, i1, i2) = (index0 as usize, index1 as usize, index2 as usize);
        let mut m = SkMatrix::default();
        m.set(0, pts[i1].fX - pts[i0].fX);
        m.set(1, pts[i2].fX - pts[i0].fX);
        m.set(2, pts[i0].fX);
        m.set(3, pts[i1].fY - pts[i0].fY);
        m.set(4, pts[i2].fY - pts[i0].fY);
        m.set(5, pts[i0].fY);
        let Some(im) = m.invert() else {
            return false;
        };
        let m33 = SkMatrix::Concat(&im, ctm_inv);
        let (c0, c1, c2) = (colors[i0].vec(), colors[i1].vec(), colors[i2].vec());
        let mut m43 = Matrix43 { fMat: [0.0; 12] };
        // Pure lane load/subtract/store substitution for skvx float4; each
        // channel preserves original arithmetic and column-major layout.
        for lane in 0..4 {
            m43.fMat[lane] = c1[lane] - c0[lane];
            m43.fMat[4 + lane] = c2[lane] - c0[lane];
            m43.fMat[8 + lane] = c0[lane];
        }
        if !self.fUsePersp {
            m43.setConcat(m43, &m33);
        }
        unsafe {
            self.fM33.get().write(m33);
            (*self.fM43.get()).write(m43);
        }
        true
    }
}
