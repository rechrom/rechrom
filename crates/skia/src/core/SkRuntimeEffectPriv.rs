// Copyright 2020 Google LLC. BSD-style license.
// Actual reflection/transform implementations remain in their native .cpp target.
// Borrowed projections use the actual retained compiled-effect owner.
// Source: src/core/SkRuntimeEffectPriv.h:118-122
pub fn ES3Options() -> super::SkRuntimeEffect::Options<'static> {
    let mut options = super::SkRuntimeEffect::Options::default();
    options.maxVersionAllowed = crate::include::sksl::SkSLVersion::Version::k300;
    options
}
// Source: src/core/SkRuntimeEffectPriv.h:124-126
pub fn AllowPrivateAccess(options: &mut super::SkRuntimeEffect::Options<'_>) {
    options.allowPrivateAccess = true;
}
// Source: src/core/SkRuntimeEffectPriv.h:128-135
pub use super::SkRuntimeEffect::{ChildTypeToStr, VarAsChild, VarAsUniform};
// Source: src/core/SkRuntimeEffectPriv.h:139-144
pub use super::SkRuntimeEffect::{TransformUniforms, TransformUniformsWithSteps};

// Source: src/core/SkRuntimeEffectPriv.h:222-259
// Concrete native callback state. The SkSL::RP::Callbacks base protocol lives in
// the actual admitted SkSLRasterPipelineBuilder.h. The methods below bridge
// directly to that real callback protocol without any evaluator adapter.
pub struct RuntimeEffectRPCallbacks<'stage, 'children> {
    pub(super) fStage: super::SkEffectPriv::SkStageRec<'stage>,
    pub(super) fMatrix: &'children crate::src::shaders::SkShaderBase::MatrixRec,
    pub(super) fChildren: &'children [super::SkRuntimeEffect::ChildPtr],
    pub(super) fSampleUsages: &'children [crate::src::sksl::SkSLSampleUsage::SampleUsage],
}
impl<'stage, 'children> RuntimeEffectRPCallbacks<'stage, 'children> {
    // Source: src/core/SkRuntimeEffectPriv.h:227-240
    pub fn new(
        stage: &super::SkEffectPriv::SkStageRec<'stage>,
        matrix: &'children crate::src::shaders::SkShaderBase::MatrixRec,
        children: &'children [super::SkRuntimeEffect::ChildPtr],
        usages: &'children [crate::src::sksl::SkSLSampleUsage::SampleUsage],
    ) -> Self {
        Self {
            fStage: super::SkEffectPriv::SkStageRec {
                fPipeline: stage.fPipeline,
                fAlloc: stage.fAlloc,
                fDstColorType: stage.fDstColorType,
                fDstCS: stage.fDstCS,
                fPaintColor: super::SkColor::SkColors::kTransparent,
                fSurfaceProps: stage.fSurfaceProps,
                fDstBounds: stage.fDstBounds,
            },
            fMatrix: matrix,
            fChildren: children,
            fSampleUsages: usages,
        }
    }
}

// Source: src/core/SkRuntimeEffectPriv.h:54-64
pub struct UniformsCallbackContext<'a> {
    pub fDstColorSpace: Option<&'a std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
}
pub type UniformsCallback = std::sync::Arc<
    dyn for<'a> Fn(&UniformsCallbackContext<'a>) -> Option<std::sync::Arc<super::SkData::SkData>>
        + Send
        + Sync,
>;
// Source: src/core/SkRuntimeEffectPriv.h:146-153
pub use super::SkRuntimeEffect::UniformsAsSpan;

// Source: src/core/SkRuntimeEffectPriv.h:222-259
impl crate::src::sksl::codegen::SkSLRasterPipelineBuilder::Callbacks
    for RuntimeEffectRPCallbacks<'_, '_>
{
    fn appendShader(&mut self, index: i32) -> Result<bool, crate::RasterError> {
        unsafe { RuntimeEffectRPCallbacks::appendShader(self, index) }
    }
    fn appendColorFilter(&mut self, index: i32) -> Result<bool, crate::RasterError> {
        unsafe { RuntimeEffectRPCallbacks::appendColorFilter(self, index) }
    }
    fn appendBlender(&mut self, index: i32) -> Result<bool, crate::RasterError> {
        unsafe { RuntimeEffectRPCallbacks::appendBlender(self, index) }
    }
    fn toLinearSrgb(&mut self, color: *const core::ffi::c_void) -> Result<(), crate::RasterError> {
        unsafe {
            RuntimeEffectRPCallbacks::toLinearSrgb(self, color);
        }
        Ok(())
    }
    fn fromLinearSrgb(
        &mut self,
        color: *const core::ffi::c_void,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            RuntimeEffectRPCallbacks::fromLinearSrgb(self, color);
        }
        Ok(())
    }
}
// Source: src/core/SkRuntimeEffectPriv.h:72-90
pub fn SupportsConstantOutputForConstantInput(
    effect: &super::SkRuntimeEffect::SkRuntimeEffect,
) -> bool {
    effect.allowColorFilter() && effect.children().is_empty()
}
pub fn Hash(effect: &super::SkRuntimeEffect::SkRuntimeEffect) -> u32 {
    effect.hash()
}
pub fn HasName(effect: &super::SkRuntimeEffect::SkRuntimeEffect) -> bool {
    effect.fName.len() > 1
}
pub fn GetName(effect: &super::SkRuntimeEffect::SkRuntimeEffect) -> *const core::ffi::c_char {
    effect.fName.as_ptr().cast()
}
pub fn StableKey(effect: &super::SkRuntimeEffect::SkRuntimeEffect) -> u32 {
    effect.fStableKey
}
// Source: src/core/SkRuntimeEffectPriv.h:104-108
pub fn SetStableKeyOnOptions(options: &mut super::SkRuntimeEffect::Options<'_>, key: u32) {
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffectPriv.h\0".as_ptr(),105; @condition b"!options->fStableKey\0".as_ptr(); options.fStableKey==0);
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffectPriv.h\0".as_ptr(),106; @condition b"SkKnownRuntimeEffects::IsSkiaKnownRuntimeEffect(stableKey)\0".as_ptr(); super::SkKnownRuntimeEffects::IsSkiaKnownRuntimeEffect(key as i32));
    options.fStableKey = key;
}

// Source: src/core/SkRuntimeEffectPriv.h:203-219
pub fn SkMakeRuntimeEffect(
    make: super::SkRuntimeEffect::MakeEffect,
    sksl: &[u8],
    mut options: super::SkRuntimeEffect::Options<'_>,
) -> std::sync::Arc<super::SkRuntimeEffect::SkRuntimeEffect> {
    #[cfg(sk_debug)]
    {
        if sksl.windows(2).any(|s| s == b"//") || sksl.windows(4).any(|s| s == b"    ") {
            let mut nul_sksl = sksl.to_vec();
            nul_sksl.push(0);
            crate::SkDEBUGFAILF!(@source b"src/core/SkRuntimeEffectPriv.h\0".as_ptr(),211; "Found SkSL snippet that can be minified: \n %s\n",nul_sksl.as_ptr());
        }
    }
    AllowPrivateAccess(&mut options);
    let result = make(sksl.to_vec(), &options);
    result.effect.unwrap_or_else(||{let mut error_text=result.errorText;error_text.push(0);crate::SK_ABORT!(@source b"src/core/SkRuntimeEffectPriv.h\0".as_ptr(),217; "%s",error_text.as_ptr())})
}
// Source: src/core/SkRuntimeEffectPriv.h:161-180
pub fn UsesSampleCoords(e: &super::SkRuntimeEffect::SkRuntimeEffect) -> bool {
    e.usesSampleCoords()
}
pub fn SamplesOutsideMain(e: &super::SkRuntimeEffect::SkRuntimeEffect) -> bool {
    e.samplesOutsideMain()
}
pub fn UsesColorTransform(e: &super::SkRuntimeEffect::SkRuntimeEffect) -> bool {
    e.usesColorTransform()
}
pub fn AlwaysOpaque(e: &super::SkRuntimeEffect::SkRuntimeEffect) -> bool {
    e.alwaysOpaque()
}
pub fn IsAlphaUnchanged(e: &super::SkRuntimeEffect::SkRuntimeEffect) -> bool {
    e.isAlphaUnchanged()
}
pub fn ChildSampleUsage(
    e: &super::SkRuntimeEffect::SkRuntimeEffect,
    i: usize,
) -> crate::src::sksl::SkSLSampleUsage::SampleUsage {
    e.fSampleUsages[i]
}

// Source: src/core/SkRuntimeEffectPriv.h:97-101
/// Safety: effect is its original owner under an exclusive metadata lease;
/// no shared access or lazy compilation may race the stable-key mutation.
pub unsafe fn SetStableKey(
    mut effect: core::ptr::NonNull<super::SkRuntimeEffect::SkRuntimeEffect>,
    key: u32,
) {
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffectPriv.h\0".as_ptr(),98; @condition b"!effect->fStableKey\0".as_ptr(); effect.as_ref().fStableKey==0);
    crate::SkASSERT!(@source b"src/core/SkRuntimeEffectPriv.h\0".as_ptr(),99; @condition b"SkKnownRuntimeEffects::IsViableUserDefinedKnownRuntimeEffect(stableKey)\0".as_ptr(); super::SkKnownRuntimeEffects::IsViableUserDefinedKnownRuntimeEffect(key as i32));
    effect.as_mut().fStableKey = key;
}
// Source: src/core/SkRuntimeEffectPriv.h:110-112
/// Safety: same original exclusive metadata lease as SetStableKey.
pub unsafe fn ResetStableKey(
    mut effect: core::ptr::NonNull<super::SkRuntimeEffect::SkRuntimeEffect>,
) {
    effect.as_mut().fStableKey = 0;
}
// Source: src/core/SkRuntimeEffectPriv.h:114-116
pub fn Program(
    effect: &super::SkRuntimeEffect::SkRuntimeEffect,
) -> &crate::src::sksl::ir::SkSLProgram::Program {
    &effect.fBaseProgram
}

// Source: src/core/SkRuntimeEffectPriv.h:65-69
// Source: src/core/SkRuntimeEffectPriv.h:152-153
pub use super::SkRuntimeEffect::{CanDraw, CanDrawProgram, MakeDeferredShader};

// Source: src/core/SkRuntimeEffectPriv.h:194-198
// CStr retains native first-NUL semantics; the real cache takes the resulting
// owned source bytes and preserves its canonical effect owner.
pub fn SkMakeCachedRuntimeEffectCStr(
    make: super::SkRuntimeEffect::MakeEffect,
    sksl: &core::ffi::CStr,
) -> Option<std::sync::Arc<super::SkRuntimeEffect::SkRuntimeEffect>> {
    super::SkRuntimeEffect::SkMakeCachedRuntimeEffect(make, sksl.to_bytes().to_vec())
}

// Source: src/core/SkRuntimeEffectPriv.h:190-192
pub use super::SkRuntimeEffect::SkMakeCachedRuntimeEffect;

// Native header declarations project the existing canonical implementations.
// These exports add no object, owner, counter, fallback or native body.
// Source: src/core/SkRuntimeEffectPriv.h:155-159
pub use super::SkRuntimeEffect::{ReadChildEffects, WriteChildEffects};
