// Copyright 2021 Google LLC. BSD-style license.
use crate::src::core::SkBlendMode::SkBlendMode;
use crate::src::core::SkEffectPriv::SkStageRec;
// Source: src/core/SkBlenderBase.h:39-58
// Rust trait dispatch is the borrowed CPU view. The public SkBlender trait
// combines it with canonical native Flattenable/RefCnt retained identity.
// Runtime-effect/serialization hooks remain pending their canonical owners.
pub trait SkBlenderBase {
    // Source: src/core/SkBlenderBase.h:55-55
    fn asRuntimeEffect(&self) -> Option<&crate::src::core::SkRuntimeEffect::SkRuntimeEffect> {
        None
    }
    // Source: src/core/SkBlenderBase.h:45-45
    fn asBlendMode(&self) -> Option<SkBlendMode> {
        None
    }
    // Source: src/core/SkBlenderBase.h:47-47
    fn affectsTransparentBlack(&self) -> bool {
        crate::src::core::SkBlendModeBlender::affectsTransparentBlack(self)
    }
    // Source: src/core/SkBlenderBase.h:49-51
    /// Safety: rec's pipeline and arena must be valid, exclusively accessible
    /// while appending, and all stored contexts must outlive pipeline execution.
    unsafe fn appendStages(&self, rec: &SkStageRec<'_>) -> Result<bool, crate::RasterError> {
        self.onAppendStages(rec)
    }
    // Source: src/core/SkBlenderBase.h:53-53
    /// Safety: same borrowed pipeline/arena contract as appendStages.
    unsafe fn onAppendStages(&self, rec: &SkStageRec<'_>) -> Result<bool, crate::RasterError>;
    // Source: src/core/SkBlenderBase.h:57-57
    fn GetFlattenableType() -> crate::src::core::SkFlattenable::Type
    where
        Self: Sized,
    {
        crate::src::core::SkFlattenable::Type::kSkBlender_Type
    }
    // Source: src/core/SkBlenderBase.h:58-58
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkBlender_Type
    }
    // The source line66 type requirement stays inside the Rust trait;
    // extracting its nested enum below is recorded as a language adaptation.
    fn r#type(&self) -> BlenderType;
}

// Source: src/core/SkBlenderBase.h:60-64
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlenderType {
    kBlendMode,
    kRuntime,
}

// Source: src/core/SkBlenderBase.h:69-79
// Shared/mutable C++ overloads project into one borrowed immutable CPU view;
// Rust trait upcasting replaces the native internal static casts.
pub fn as_BB(blend: &dyn crate::include::core::SkBlender::SkBlender) -> &dyn SkBlenderBase {
    blend
}
