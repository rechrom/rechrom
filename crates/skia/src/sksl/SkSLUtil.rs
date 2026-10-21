// UNVERIFIED: user explicitly prohibited all validation.
use crate::include::sksl::SkSLVersion::Version;
// Actual pinned GLSLGeneration enum translated in v54.
use super::SkSLGLSL::GLSLGeneration;
use core::ffi::c_char;
// Source: src/sksl/SkSLUtil.h:31-37
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdvBlendEqInteraction {
    kNotSupported_AdvBlendEqInteraction,
    kAutomatic_AdvBlendEqInteraction,
    kGeneralEnable_AdvBlendEqInteraction,
}
impl AdvBlendEqInteraction {
    pub const kLast_AdvBlendEqInteraction: Self = Self::kGeneralEnable_AdvBlendEqInteraction;
}
// Source: src/sksl/SkSLUtil.h:26-26
#[repr(C)]
pub struct ShaderCaps {
    // Source: src/sksl/SkSLUtil.h:96-96
    pub fFloatIs32Bits: bool,
    // Source: src/sksl/SkSLUtil.h:102-102
    pub fGLSLGeneration: GLSLGeneration,
    // Source: src/sksl/SkSLUtil.h:103-103
    pub fFlatInterpolationSupport: bool,
    // Source: src/sksl/SkSLUtil.h:104-104
    pub fUsesPrecisionModifiers: bool,
    // Source: src/sksl/SkSLUtil.h:106-106
    pub fDualSourceBlendingSupport: bool,
    // Source: src/sksl/SkSLUtil.h:107-107
    pub fShaderDerivativeSupport: bool,
    // Source: src/sksl/SkSLUtil.h:109-109
    pub fExplicitTextureLodSupport: bool,
    // Source: src/sksl/SkSLUtil.h:111-111
    pub fIntegerSupport: bool,
    // Source: src/sksl/SkSLUtil.h:112-112
    pub fNonsquareMatrixSupport: bool,
    // Source: src/sksl/SkSLUtil.h:114-114
    pub fInverseHyperbolicSupport: bool,
    // Source: src/sksl/SkSLUtil.h:115-115
    pub fFBFetchSupport: bool,
    // Source: src/sksl/SkSLUtil.h:116-116
    pub fFBFetchNeedsCustomOutput: bool,
    // Source: src/sksl/SkSLUtil.h:117-117
    pub fNoPerspectiveInterpolationSupport: bool,
    // Source: src/sksl/SkSLUtil.h:118-118
    pub fSampleMaskSupport: bool,
    // Source: src/sksl/SkSLUtil.h:119-119
    pub fExternalTextureSupport: bool,
    // Source: src/sksl/SkSLUtil.h:122-122
    pub fInfinitySupport: bool,
    // Source: src/sksl/SkSLUtil.h:125-125
    pub fBuiltinFMASupport: bool,
    // Source: src/sksl/SkSLUtil.h:126-126
    pub fBuiltinDeterminantSupport: bool,
    // Source: src/sksl/SkSLUtil.h:129-129
    pub fCanUseVoidInSequenceExpressions: bool,
    // Source: src/sksl/SkSLUtil.h:130-130
    pub fCanUseMinAndAbsTogether: bool,
    // Source: src/sksl/SkSLUtil.h:131-131
    pub fCanUseFractForNegativeValues: bool,
    // Source: src/sksl/SkSLUtil.h:132-132
    pub fMustForceNegatedAtanParamToFloat: bool,
    // Source: src/sksl/SkSLUtil.h:133-133
    pub fMustForceNegatedLdexpParamToMultiply: bool,
    // Source: src/sksl/SkSLUtil.h:135-135
    pub fAtan2ImplementedAsAtanYOverX: bool,
    // Source: src/sksl/SkSLUtil.h:141-141
    pub fMustDoOpBetweenFloorAndAbs: bool,
    // Source: src/sksl/SkSLUtil.h:146-146
    pub fMustGuardDivisionEvenAfterExplicitZeroCheck: bool,
    // Source: src/sksl/SkSLUtil.h:148-148
    pub fCanUseFragCoord: bool,
    // Source: src/sksl/SkSLUtil.h:150-150
    pub fAddAndTrueToLoopCondition: bool,
    // Source: src/sksl/SkSLUtil.h:155-155
    pub fUnfoldShortCircuitAsTernary: bool,
    // Source: src/sksl/SkSLUtil.h:156-156
    pub fEmulateAbsIntFunction: bool,
    // Source: src/sksl/SkSLUtil.h:157-157
    pub fRewriteDoWhileLoops: bool,
    // Source: src/sksl/SkSLUtil.h:158-158
    pub fRewriteSwitchStatements: bool,
    // Source: src/sksl/SkSLUtil.h:159-159
    pub fRemovePowWithConstantExponent: bool,
    // Source: src/sksl/SkSLUtil.h:164-164
    pub fNoDefaultPrecisionForExternalSamplers: bool,
    // Source: src/sksl/SkSLUtil.h:170-170
    pub fRewriteMatrixVectorMultiply: bool,
    // Source: src/sksl/SkSLUtil.h:172-172
    pub fRewriteMatrixComparisons: bool,
    // Source: src/sksl/SkSLUtil.h:174-174
    pub fRemoveConstFromFunctionParameters: bool,
    // Source: src/sksl/SkSLUtil.h:179-179
    pub fPerlinNoiseRoundingFix: bool,
    // Source: src/sksl/SkSLUtil.h:185-185
    pub fMustDeclareFragmentFrontFacing: bool,
    // Source: src/sksl/SkSLUtil.h:191-191
    pub fForceStd430ArrayLayout: bool,
    // Source: src/sksl/SkSLUtil.h:197-197
    pub fCannotUseRelaxedPrecisionOnImageSample: bool,
    // Source: src/sksl/SkSLUtil.h:202-202
    pub fVectorClampMinMaxSupport: bool,
    // Source: src/sksl/SkSLUtil.h:204-204
    pub fVersionDeclString: *const c_char,
    // Source: src/sksl/SkSLUtil.h:206-206
    pub fShaderDerivativeExtensionString: *const c_char,
    // Source: src/sksl/SkSLUtil.h:207-207
    pub fExternalTextureExtensionString: *const c_char,
    // Source: src/sksl/SkSLUtil.h:208-208
    pub fSecondExternalTextureExtensionString: *const c_char,
    // Source: src/sksl/SkSLUtil.h:209-209
    pub fFBFetchColorName: *const c_char,
    // Source: src/sksl/SkSLUtil.h:211-211
    pub fAdvBlendEqInteraction: AdvBlendEqInteraction,
}
impl ShaderCaps {
    // Source: src/sksl/SkSLUtil.h:39-41
    pub fn mustEnableAdvBlendEqs(&self) -> bool {
        self.fAdvBlendEqInteraction >= AdvBlendEqInteraction::kGeneralEnable_AdvBlendEqInteraction
    }
    // Source: src/sksl/SkSLUtil.h:43-45
    pub fn mustDeclareFragmentShaderOutput(&self) -> bool {
        self.fGLSLGeneration > GLSLGeneration::k110
    }
    // Source: src/sksl/SkSLUtil.h:52-55
    pub fn shaderDerivativeExtensionString(&self) -> *const c_char {
        crate::SkASSERT!(@source b"src/sksl/SkSLUtil.h\0".as_ptr(),53; @condition b"this->fShaderDerivativeSupport\0".as_ptr(); self.fShaderDerivativeSupport);
        self.fShaderDerivativeExtensionString
    }
    // Source: src/sksl/SkSLUtil.h:63-66
    pub fn externalTextureExtensionString(&self) -> *const c_char {
        crate::SkASSERT!(@source b"src/sksl/SkSLUtil.h\0".as_ptr(),64; @condition b"this->fExternalTextureSupport\0".as_ptr(); self.fExternalTextureSupport);
        self.fExternalTextureExtensionString
    }
    // Source: src/sksl/SkSLUtil.h:60-66
    pub fn secondExternalTextureExtensionString(&self) -> *const c_char {
        crate::SkASSERT!(@source b"src/sksl/SkSLUtil.h\0".as_ptr(),64; @condition b"this->fExternalTextureSupport\0".as_ptr(); self.fExternalTextureSupport);
        self.fSecondExternalTextureExtensionString
    }
    // Source: src/sksl/SkSLUtil.h:76-82
    pub fn supportedSkSLVerion(&self) -> Version {
        if self.fShaderDerivativeSupport
            && self.fNonsquareMatrixSupport
            && self.fIntegerSupport
            && self.fGLSLGeneration >= GLSLGeneration::k330
        {
            Version::k300
        } else {
            Version::k100
        }
    }
    // Source: src/sksl/SkSLUtil.h:84-84
    pub fn supportsDistanceFieldText(&self) -> bool {
        self.fShaderDerivativeSupport
    }
}
// Adapted aggregate initialization executes only original in-class defaults.
impl Default for ShaderCaps {
    fn default() -> Self {
        Self {
            fFloatIs32Bits: true,
            fGLSLGeneration: GLSLGeneration::k330,
            fFlatInterpolationSupport: false,
            fUsesPrecisionModifiers: false,
            fDualSourceBlendingSupport: false,
            fShaderDerivativeSupport: false,
            fExplicitTextureLodSupport: false,
            fIntegerSupport: false,
            fNonsquareMatrixSupport: false,
            fInverseHyperbolicSupport: false,
            fFBFetchSupport: false,
            fFBFetchNeedsCustomOutput: false,
            fNoPerspectiveInterpolationSupport: false,
            fSampleMaskSupport: false,
            fExternalTextureSupport: false,
            fInfinitySupport: false,
            fBuiltinFMASupport: true,
            fBuiltinDeterminantSupport: true,
            fCanUseVoidInSequenceExpressions: true,
            fCanUseMinAndAbsTogether: true,
            fCanUseFractForNegativeValues: true,
            fMustForceNegatedAtanParamToFloat: false,
            fMustForceNegatedLdexpParamToMultiply: false,
            fAtan2ImplementedAsAtanYOverX: false,
            fMustDoOpBetweenFloorAndAbs: false,
            fMustGuardDivisionEvenAfterExplicitZeroCheck: false,
            fCanUseFragCoord: true,
            fAddAndTrueToLoopCondition: false,
            fUnfoldShortCircuitAsTernary: false,
            fEmulateAbsIntFunction: false,
            fRewriteDoWhileLoops: false,
            fRewriteSwitchStatements: false,
            fRemovePowWithConstantExponent: false,
            fNoDefaultPrecisionForExternalSamplers: false,
            fRewriteMatrixVectorMultiply: false,
            fRewriteMatrixComparisons: false,
            fRemoveConstFromFunctionParameters: false,
            fPerlinNoiseRoundingFix: false,
            fMustDeclareFragmentFrontFacing: false,
            fForceStd430ArrayLayout: false,
            fCannotUseRelaxedPrecisionOnImageSample: false,
            fVectorClampMinMaxSupport: true,
            fVersionDeclString: b"\0".as_ptr().cast(),
            fShaderDerivativeExtensionString: core::ptr::null(),
            fExternalTextureExtensionString: core::ptr::null(),
            fSecondExternalTextureExtensionString: core::ptr::null(),
            fFBFetchColorName: core::ptr::null(),
            fAdvBlendEqInteraction: AdvBlendEqInteraction::kNotSupported_AdvBlendEqInteraction,
        }
    }
}
// Source: src/sksl/SkSLUtil.h:215-215
pub struct ShaderCapsFactory;
impl ShaderCapsFactory {
    // Source: src/sksl/SkSLUtil.h:217-225
    // Native owner deliberately releases allocation for static lifetime. Actual MakeShaderCaps
    // is now translated below from the actual selected CPU build branch.
    pub fn Default() -> &'static ShaderCaps {
        static CAPS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let p = *CAPS.get_or_init(|| {
            let mut caps = Self::MakeShaderCaps();
            caps.fVersionDeclString = b"#version 400\0".as_ptr().cast();
            caps.fShaderDerivativeSupport = true;
            Box::into_raw(caps) as usize
        });
        unsafe { &*(p as *const ShaderCaps) }
    }
    // Source: src/sksl/SkSLUtil.h:227-230
    pub fn Standalone() -> &'static ShaderCaps {
        static CAPS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let p = *CAPS.get_or_init(|| Box::into_raw(Self::MakeShaderCaps()) as usize);
        unsafe { &*(p as *const ShaderCaps) }
    }
}
// Native external declarations type_to_sksltype/write_stringstream require their real
// definitions and stream/type dependencies; no substitute implementations added.
impl ShaderCapsFactory {
    // Source: src/sksl/SkSLUtil.cpp:25-34
    // CPU build selects !SK_GANESH; these six capability assignments are actual native values.
    pub fn MakeShaderCaps() -> Box<ShaderCaps> {
        let mut standalone = Box::new(ShaderCaps::default());
        standalone.fShaderDerivativeSupport = true;
        standalone.fExplicitTextureLodSupport = true;
        standalone.fFlatInterpolationSupport = true;
        standalone.fNoPerspectiveInterpolationSupport = true;
        standalone.fSampleMaskSupport = true;
        standalone.fExternalTextureSupport = true;
        standalone
    }
}
// Source: src/sksl/SkSLUtil.cpp:41-43
// Actual StringStream cache and virtual OutputStream are now translated. Borrow guards
// end between the two native str() calls, preserving empty-cache detach semantics.
pub fn write_stringstream(
    s: &super::SkSLStringStream::StringStream,
    out: &mut dyn super::SkSLOutputStream::OutputStream,
) {
    let data = {
        let value = s.str();
        value.as_ptr()
    };
    let size = s.str().len();
    unsafe {
        out.write(data.cast(), size);
    }
}
// Source: src/sksl/SkSLUtil.cpp:45-85
// Actual pinned shared reflection enum is translated in v56.
pub unsafe fn type_to_sksltype(
    context: &super::SkSLContext::Context,
    ty: &dyn super::ir::SkSLType::Type,
    outType: *mut crate::src::core::SkSLTypeShared::SkSLType,
) -> bool {
    use crate::src::core::SkSLTypeShared::{kSkSLTypeCount, SkSLType};
    const _: () = assert!(kSkSLTypeCount == 41);
    if ty.matches(&**context.fTypes.as_ref().fVoid) {
        *outType = SkSLType::kVoid;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fBool) {
        *outType = SkSLType::kBool;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fBool2) {
        *outType = SkSLType::kBool2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fBool3) {
        *outType = SkSLType::kBool3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fBool4) {
        *outType = SkSLType::kBool4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fShort) {
        *outType = SkSLType::kShort;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fShort2) {
        *outType = SkSLType::kShort2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fShort3) {
        *outType = SkSLType::kShort3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fShort4) {
        *outType = SkSLType::kShort4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUShort) {
        *outType = SkSLType::kUShort;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUShort2) {
        *outType = SkSLType::kUShort2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUShort3) {
        *outType = SkSLType::kUShort3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUShort4) {
        *outType = SkSLType::kUShort4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat) {
        *outType = SkSLType::kFloat;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat2) {
        *outType = SkSLType::kFloat2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat3) {
        *outType = SkSLType::kFloat3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat4) {
        *outType = SkSLType::kFloat4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat2x2) {
        *outType = SkSLType::kFloat2x2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat3x3) {
        *outType = SkSLType::kFloat3x3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fFloat4x4) {
        *outType = SkSLType::kFloat4x4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf) {
        *outType = SkSLType::kHalf;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf2) {
        *outType = SkSLType::kHalf2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf3) {
        *outType = SkSLType::kHalf3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf4) {
        *outType = SkSLType::kHalf4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf2x2) {
        *outType = SkSLType::kHalf2x2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf3x3) {
        *outType = SkSLType::kHalf3x3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fHalf4x4) {
        *outType = SkSLType::kHalf4x4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fInt) {
        *outType = SkSLType::kInt;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fInt2) {
        *outType = SkSLType::kInt2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fInt3) {
        *outType = SkSLType::kInt3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fInt4) {
        *outType = SkSLType::kInt4;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUInt) {
        *outType = SkSLType::kUInt;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUInt2) {
        *outType = SkSLType::kUInt2;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUInt3) {
        *outType = SkSLType::kUInt3;
        return true;
    }
    if ty.matches(&**context.fTypes.as_ref().fUInt4) {
        *outType = SkSLType::kUInt4;
        return true;
    }
    false
}
