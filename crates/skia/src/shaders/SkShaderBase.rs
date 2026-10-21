// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2017 Google LLC. BSD-style license.
use crate::src::core::SkMatrix::SkMatrix;

// Source: src/shaders/SkShaderBase.h:59-59
impl Default for MatrixRec {
    fn default() -> Self {
        Self::from_parts(
            SkMatrix::default(),
            SkMatrix::default(),
            SkMatrix::default(),
            true,
            false,
        )
    }
}
impl MatrixRec {
    // Source: src/shaders/SkShaderBase.h:101-101
    pub fn markTotalMatrixInvalid(&mut self) {
        self.fTotalMatrixIsValid = false;
    }
    // Source: src/shaders/SkShaderBase.h:104-104
    pub fn markCTMApplied(&mut self) {
        self.fCTMApplied = true;
    }
    // Source: src/shaders/SkShaderBase.h:110-110
    pub fn totalMatrixIsValid(&self) -> bool {
        self.fTotalMatrixIsValid
    }
    // Source: src/shaders/SkShaderBase.h:117-117
    pub fn totalMatrix(&self) -> SkMatrix {
        concat_matrix(&self.fCTM, &self.fTotalLocalMatrix)
    }
    // Source: src/shaders/SkShaderBase.h:120-121
    pub fn totalInverse(&self) -> Option<SkMatrix> {
        self.totalMatrix().invert()
    }
    // Source: src/shaders/SkShaderBase.h:125-126
    pub fn hasPendingMatrix(&self) -> bool {
        (!self.fCTMApplied && !self.fCTM.isIdentity()) || !self.fPendingLocalMatrix.isIdentity()
    }
    // Source: src/shaders/SkShaderBase.h:130-130
    pub fn rasterPipelineCoordsAreSeeded(&self) -> bool {
        self.fCTMApplied
    }
    // Source: src/shaders/SkShaderBase.h:133-142
    fn from_parts(
        ctm: SkMatrix,
        total_local_matrix: SkMatrix,
        pending_local_matrix: SkMatrix,
        total_is_valid: bool,
        ctm_applied: bool,
    ) -> Self {
        Self {
            fCTM: ctm,
            fTotalLocalMatrix: total_local_matrix,
            fPendingLocalMatrix: pending_local_matrix,
            fTotalMatrixIsValid: total_is_valid,
            fCTMApplied: ctm_applied,
        }
    }
}
// Source: src/shaders/SkShaderBase.h:144-144
// Source: src/shaders/SkShaderBase.h:147-147
// Source: src/shaders/SkShaderBase.h:151-151
// Source: src/shaders/SkShaderBase.h:153-153
// Source: src/shaders/SkShaderBase.h:157-157
#[derive(Clone)]
pub struct MatrixRec {
    fCTM: SkMatrix,
    fTotalLocalMatrix: SkMatrix,
    fPendingLocalMatrix: SkMatrix,
    fTotalMatrixIsValid: bool,
    fCTMApplied: bool,
}

// Source: src/shaders/SkShaderBase.h:163-177
// Macro-expanded discriminants retain upstream order; Rust enums replace C++ enum classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderType {
    kBlend,
    kCTM,
    kColor,
    kColorFilter,
    kCoordClamp,
    kEmpty,
    kGradientBase,
    kImage,
    kLocalMatrix,
    kPerlinNoise,
    kPicture,
    kRuntime,
    kTransform,
    kTriColor,
    kWorkingColorSpace,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientType {
    kNone,
    kConical,
    kLinear,
    kRadial,
    kSweep,
}

// Source: src/shaders/SkShaderBase.h:197-197
fn default_is_constant() -> bool {
    false
}

// Source: src/shaders/SkShaderBase.h:243-244
// Source: src/shaders/SkShaderBase.h:248-253
pub struct GradientInfo {
    pub fColorCount: i32,
    pub fColors: *mut crate::src::core::SkColor::SkColor4f,
    pub fColorOffsets: *mut f32,
    pub fPoint: [crate::src::core::SkPoint::SkPoint; 2],
    pub fRadius: [f32; 2],
    pub fTileMode: crate::include::core::SkTileMode::SkTileMode,
    pub fPremulInterp: bool,
}

// Rust Default corresponds to C++ GradientInfo{} value initialization, including zeroed unspecified fields.
impl Default for GradientInfo {
    fn default() -> Self {
        Self {
            fColorCount: 0,
            fColors: core::ptr::null_mut(),
            fColorOffsets: core::ptr::null_mut(),
            fPoint: [crate::src::core::SkPoint::SkPoint::default(); 2],
            fRadius: [0.0; 2],
            fTileMode: crate::include::core::SkTileMode::SkTileMode::kClamp,
            fPremulInterp: false,
        }
    }
}

// Source: src/shaders/SkShaderBase.h:256-258
fn default_as_gradient() -> GradientType {
    GradientType::kNone
}

// Source: src/shaders/SkShaderBase.h:261-261
// Source: src/shaders/SkShaderBase.h:263-263
pub const kOpaqueAlpha_Flag: u32 = 1 << 0;

impl<'a> ContextRec<'a> {
    // Source: src/shaders/SkShaderBase.h:270-279
    pub fn new(
        paint_alpha: crate::src::core::SkColor::SkAlpha,
        matrix_rec: &MatrixRec,
        dst_color_type: crate::include::core::SkColorType::SkColorType,
        dst_color_space: Option<&'a std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> Self {
        Self {
            fMatrixRec: matrix_rec.clone(),
            fDstColorType: dst_color_type,
            fDstColorSpace: dst_color_space,
            fProps: *props,
            fPaintAlpha: paint_alpha,
        }
    }
    // Source: src/shaders/SkShaderBase.h:281-286
    pub fn Concat(parent_rec: &Self, local_m: &SkMatrix) -> Self {
        Self::new(
            parent_rec.fPaintAlpha,
            &parent_rec.fMatrixRec.concat(local_m),
            parent_rec.fDstColorType,
            parent_rec.fDstColorSpace,
            &parent_rec.fProps,
        )
    }
}
// Source: src/shaders/SkShaderBase.h:289-293
#[derive(Clone)]
pub struct ContextRec<'a> {
    pub fMatrixRec: MatrixRec,
    pub fDstColorType: crate::include::core::SkColorType::SkColorType,
    pub fDstColorSpace: Option<&'a std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    pub fProps: crate::include::core::SkSurfaceProps::SkSurfaceProps,
    pub fPaintAlpha: crate::src::core::SkColor::SkAlpha,
}

pub trait Context {
    fn state(&self) -> &ContextState;
    // Source: src/shaders/SkShaderBase.h:311-311
    fn getFlags(&self) -> u32 {
        0
    }
    /// Safety: dst contains count writable pixels, exclusive for this call; count is positive.
    /// All shader, image and arena backing referenced by this context must remain live and stable.
    // Source: src/shaders/SkShaderBase.h:318-318
    unsafe fn shadeSpan(&mut self, x: i32, y: i32, dst: *mut u32, count: i32);
}
// Source: src/shaders/SkShaderBase.h:322-322
// Source: src/shaders/SkShaderBase.h:327-327
// Native non-owning state, never retained or deleted by the arena context.
pub struct ContextState {
    // Native non-owning identity only: Context never dereferences fShader.
    // Keep the original object's data address, without retaining its owner,
    // reconstructing trait metadata, or publishing a safe shader reference.
    pub fShader: core::ptr::NonNull<()>,
    fPaintAlpha: u8,
}
impl ContextState {
    // Source: src/shaders/SkShaderBase.h:324-324
    pub fn getPaintAlpha(&self) -> u8 {
        self.fPaintAlpha
    }
}

// Language adaptation of SkMatrix::Concat convenience factory through its translated setConcat.
fn concat_matrix(a: &SkMatrix, b: &SkMatrix) -> SkMatrix {
    let mut result = SkMatrix::default();
    result.setConcat(a, b);
    result
}

// Source: src/shaders/SkShaderBase.cpp:24-24
impl MatrixRec {
    pub fn new(ctm: &SkMatrix) -> Self {
        Self::from_parts(
            ctm.clone(),
            SkMatrix::default(),
            SkMatrix::default(),
            true,
            false,
        )
    }
}

// Source: src/shaders/SkShaderBase.cpp:26-29
// Source: src/shaders/SkShaderBase.cpp:31-34
// Source: src/shaders/SkShaderBase.cpp:36-37
// Source: src/shaders/SkShaderBase.cpp:40-45
impl MatrixRec {
    /// Safety: rec's arena and pipeline pointers must be valid for all appended stage contexts.
    pub unsafe fn apply(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        post_inv: &SkMatrix,
    ) -> Option<Self> {
        let mut total = self.fPendingLocalMatrix.clone();
        if !self.fCTMApplied {
            total = concat_matrix(&self.fCTM, &total);
        }
        if let Some(inv) = total.invert() {
            total = concat_matrix(post_inv, &inv);
        } else {
            return None;
        }
        if !self.fCTMApplied {
            unsafe {
                (*rec.fPipeline).append(
                    crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::seed_shader,
                    core::ptr::null_mut(),
                );
            }
        }
        unsafe {
            (*rec.fPipeline).appendMatrix(rec.fAlloc, &total);
        }
        Some(Self::from_parts(
            self.fCTM.clone(),
            self.fTotalLocalMatrix.clone(),
            SkMatrix::default(),
            self.fTotalMatrixIsValid,
            true,
        ))
    }
}

// Source: src/shaders/SkShaderBase.cpp:67-72
impl MatrixRec {
    pub fn concat(&self, m: &SkMatrix) -> Self {
        Self::from_parts(
            self.fCTM.clone(),
            ConcatLocalMatrices(&self.fTotalLocalMatrix, m),
            ConcatLocalMatrices(&self.fPendingLocalMatrix, m),
            self.fTotalMatrixIsValid,
            self.fCTMApplied,
        )
    }
}

// Source: src/shaders/SkShaderBase.cpp:85-88
// Source: src/shaders/SkShaderBase.cpp:90-92
// Source: src/shaders/SkShaderBase.cpp:94-94
fn as_luminance_color(
    shader: &(impl SkShaderBase + ?Sized),
    color: Option<&mut crate::src::core::SkColor::SkColor4f>,
) -> bool {
    let mut storage = crate::src::core::SkColor::SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 0.0,
        fA: 0.0,
    };
    let color = color.unwrap_or(&mut storage);
    if shader.onAsLuminanceColor(color) {
        color.fA = 1.0;
        return true;
    }
    false
}

// Source: src/shaders/SkShaderBase.cpp:97-97
// Source: src/shaders/SkShaderBase.cpp:100-102
// Source: src/shaders/SkShaderBase.cpp:105-105
// Native configuration selects the exact enabled/disabled body.
unsafe fn make_context<S: SkShaderBase + ?Sized>(
    shader: &S,
    rec: &ContextRec<'_>,
    alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
) -> Result<Option<*mut dyn Context>, crate::RasterError> {
    #[cfg(SK_ENABLE_LEGACY_SHADERCONTEXT)]
    {
        let totalMatrix = rec.fMatrixRec.totalMatrix();
        if totalMatrix.hasPerspective() || totalMatrix.invert().is_none() {
            return Ok(None);
        }
        unsafe { shader.onMakeContext(rec, alloc) }
    }
    #[cfg(not(SK_ENABLE_LEGACY_SHADERCONTEXT))]
    {
        // Source: src/shaders/SkShaderBase.cpp:106-108
        Ok(None)
    }
}
// Source: src/shaders/SkShaderBase.cpp:111-112
// Source: src/shaders/SkShaderBase.cpp:114-114
// Source: src/shaders/SkShaderBase.cpp:116-116
// Source: src/shaders/SkShaderBase.cpp:119-119
impl ContextState {
    pub fn new(shader: &dyn SkShaderBase, rec: &ContextRec<'_>) -> Self {
        crate::SkASSERT!(@source b"src/shaders/SkShaderBase.cpp\0".as_ptr(),114; @condition b"!rec.fMatrixRec.totalMatrix().hasPerspective()\0".as_ptr(); !rec.fMatrixRec.totalMatrix().hasPerspective());
        Self {
            fShader: core::ptr::NonNull::from(shader).cast(),
            fPaintAlpha: rec.fPaintAlpha,
        }
    }
}

// Source: src/shaders/SkShaderBase.cpp:121-121
// Source: src/shaders/SkShaderBase.cpp:124-126
impl ContextRec<'_> {
    pub fn isLegacyCompatible(
        &self,
        shader_color_space: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
    ) -> bool {
        use crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType;
        crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
            shader_color_space,
            kPremul_SkAlphaType,
            self.fDstColorSpace.map(std::sync::Arc::as_ref),
            kPremul_SkAlphaType,
        )
        .fFlags
        .mask()
            == 0
    }
}

// C++'s public/base split becomes a required supertrait of SkShader so its virtual CPU methods
// remain available on every retained shader object. These default returns are upstream definitions.
// Source: src/shaders/SkShaderBase.h:185-185
pub trait SkShaderBase: crate::src::core::SkFlattenable::SkFlattenableView {
    // Source: src/shaders/SkShaderBase.h:365-365
    fn asRuntimeEffect(&self) -> Option<&crate::src::core::SkRuntimeEffect::SkRuntimeEffect> {
        None
    }
    // Rust typed projection of the same native RuntimeShader subclass, not an owner.
    fn asRuntimeShader(&self) -> Option<&crate::src::shaders::SkRuntimeShader::SkRuntimeShader> {
        None
    }
    /// Safety: alloc is a live arena; all actually captured image/pixel backing must
    /// remain live through context use. The base's opaque shader identity is never dereferenced.
    /// No arena reset/destruction or movement of retained context backing is allowed during use.
    // Source: src/shaders/SkShaderBase.h:335-335
    unsafe fn makeContext(
        &self,
        rec: &ContextRec<'_>,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<Option<*mut dyn Context>, crate::RasterError> {
        unsafe { make_context(self, rec, alloc) }
    }
    unsafe fn onMakeContext(
        &self,
        _rec: &ContextRec<'_>,
        _alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<Option<*mut dyn Context>, crate::RasterError> {
        default_make_context()
    }
    fn isConstant(&self, _color: Option<&mut crate::src::core::SkColor::SkColor4f>) -> bool {
        default_is_constant()
    }
    // Source: src/shaders/SkShaderBase.h:205-205
    fn r#type(&self) -> ShaderType;
    /// Safety: every nonnull GradientInfo output pointer must cover fColorCount
    /// writable elements for the synchronous native gradient query.
    unsafe fn asGradient(
        &self,
        _info: Option<&mut GradientInfo>,
        _local_matrix: Option<&mut SkMatrix>,
    ) -> GradientType {
        default_as_gradient()
    }
    /// Safety: rec's pipeline/arena pointers must reference live objects throughout stage construction.
    // Source: src/shaders/SkShaderBase.h:359-359
    unsafe fn appendStages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        matrix_rec: &MatrixRec,
    ) -> Result<bool, crate::RasterError>;

    // Source: src/shaders/SkShaderBase.h:361-362
    fn onIsAImage(
        &self,
        _localMatrix: Option<&mut SkMatrix>,
        _xy: Option<&mut [crate::include::core::SkTileMode::SkTileMode; 2]>,
    ) -> Option<&dyn crate::src::image::SkImage::SkImage> {
        None
    }

    fn onAsLuminanceColor(&self, _color: &mut crate::src::core::SkColor::SkColor4f) -> bool {
        default_on_luminance()
    }

    // Language delegation keeps the original algorithm in source order above.
    // Source: src/shaders/SkShaderBase.h:345-345
    fn asLuminanceColor(&self, color: Option<&mut crate::src::core::SkColor::SkColor4f>) -> bool {
        as_luminance_color(self, color)
    }
    // Source: src/shaders/SkShaderBase.cpp:129-129
    // Source: src/shaders/SkShaderBase.h:381-381
    fn makeAsALocalMatrixShader(
        &self,
        _local_matrix: Option<&mut SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        None
    }
    // Source: src/shaders/SkShaderBase.cpp:131-132
    unsafe fn appendRootStages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        ctm: &SkMatrix,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.appendStages(rec, &MatrixRec::new(ctm)) }
    }
}

// Source: src/shaders/SkShaderBase.h:367-367
pub fn GetFlattenableType() -> crate::src::core::SkFlattenable::Type {
    crate::src::core::SkFlattenable::Type::kSkShader_Type
}

// Source: src/shaders/SkShaderBase.h:370-374
/// Return the real canonical shader allocation through its native base view.
/// Arc unsizing retains the original allocation, counter, and destructor.
pub fn Deserialize(
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> Result<Option<std::sync::Arc<dyn SkShaderBase>>, crate::RasterError> {
    match crate::src::core::SkFlattenable::Deserialize(GetFlattenableType(), data, procs)? {
        None => Ok(None),
        Some(crate::src::core::SkFlattenable::FlattenableOwner::Shader(owner)) => {
            let base: std::sync::Arc<dyn SkShaderBase> = owner;
            Ok(Some(base))
        }
        Some(_) => unreachable!("shader factory must return its canonical shader owner"),
    }
}

// Source: src/shaders/SkShaderBase.h:383-383
// Source: src/shaders/SkShaderBase.h:385-385
// Source: src/shaders/SkShaderBase.h:387-387
pub fn ConcatLocalMatrices(parent_lm: &SkMatrix, child_lm: &SkMatrix) -> SkMatrix {
    if cfg!(SK_BUILD_FOR_ANDROID_FRAMEWORK) {
        concat_matrix(child_lm, parent_lm)
    } else {
        concat_matrix(parent_lm, child_lm)
    }
}
// Source: src/shaders/SkShaderBase.h:400-401
fn default_make_context() -> Result<Option<*mut dyn Context>, crate::RasterError> {
    Ok(None)
}
// Source: src/shaders/SkShaderBase.h:405-406
fn default_on_luminance() -> bool {
    false
}

// Arc receiver adapts native sk_ref_sp(this) without an ownership cast or a second shader object.
impl dyn crate::src::shaders::SkShader::SkShader {
    // Source: src/shaders/SkShaderBase.cpp:135-136
    pub fn makeWithCTM(
        self: &std::sync::Arc<Self>,
        postM: &SkMatrix,
    ) -> std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader> {
        std::sync::Arc::new(crate::src::shaders::SkLocalMatrixShader::SkCTMShader::new(
            self.clone(),
            postM,
        ))
    }
    // Source: src/shaders/SkShaderBase.cpp:140-141
    pub fn makeInvertAlpha(
        self: &std::sync::Arc<Self>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        self.makeWithColorFilter(crate::src::core::SkColorFilter::BlendColor(
            0xFFFFFFFF,
            crate::src::core::SkBlendMode::SkBlendMode::kSrcOut,
        ))
    }
}

// Rust visibility adaptation of the native static method defined in SkImageShader.cpp.
pub use super::SkImageShader::RegisterFlattenables;

// Source: src/shaders/SkShaderBase.h:412-413
// UNVERIFIED language projection: this crate expresses the native public/base
// split as supertraits. No second object, pointer owner or refcount is created.
pub fn as_SB<'a>(
    shader: Option<&'a mut (dyn crate::src::shaders::SkShader::SkShader + 'a)>,
) -> Option<&'a mut (dyn SkShaderBase + 'a)> {
    shader.map(|shader| {
        let base: &'a mut (dyn SkShaderBase + 'a) = shader;
        base
    })
}
// Source: src/shaders/SkShaderBase.h:416-417
pub fn as_SB_const<'a>(
    shader: Option<&'a (dyn crate::src::shaders::SkShader::SkShader + 'a)>,
) -> Option<&'a (dyn SkShaderBase + 'a)> {
    shader.map(|shader| {
        let base: &'a (dyn SkShaderBase + 'a) = shader;
        base
    })
}
// Source: src/shaders/SkShaderBase.h:420-421
pub fn as_SB_sp(
    shader: Option<&std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
) -> Option<&dyn SkShaderBase> {
    shader.map(|shader| {
        let base: &dyn SkShaderBase = shader.as_ref();
        base
    })
}

// Native header declarations project the existing canonical implementations.
// These exports add no object, owner, counter, fallback or native body.
// Source: src/shaders/SkShaderBase.h:424-424
pub use super::SkBlendShader::SkRegisterBlendShaderFlattenable;
// Source: src/shaders/SkShaderBase.h:425-425
pub use super::SkColorShader::SkRegisterColorShaderFlattenable;
// Source: src/shaders/SkShaderBase.h:426-426
pub use super::SkCoordClampShader::SkRegisterCoordClampShaderFlattenable;
// Source: src/shaders/SkShaderBase.h:427-427
pub use super::SkEmptyShader::SkRegisterEmptyShaderFlattenable;
// Source: src/shaders/SkShaderBase.h:429-429
pub use super::SkWorkingColorSpaceShader::SkRegisterWorkingColorSpaceShaderFlattenable;

// Source: src/shaders/SkShaderBase.h:428-428
pub use super::SkPerlinNoiseShaderImpl::SkRegisterPerlinNoiseShaderFlattenable;

// Source: src/shaders/SkShaderBase.h:393
// Source: src/shaders/SkShaderBase.cpp:83
// Qualified inherited call: concrete shader serializers can invoke precisely
// the native base serialization without recursively dispatching their override.
pub fn flatten_base(
    buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
) -> Result<(), crate::RasterError> {
    crate::src::core::SkFlattenable::flatten_base(buffer)
}
