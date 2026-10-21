// Copyright 2021 Google LLC. BSD-style license.
use crate::src::core::SkBlendMode::{
    SkBlendMode, SkBlendModeCoeff, SkBlendMode_AppendStages, SkBlendMode_AsCoeff,
};
use crate::src::core::SkBlenderBase::{BlenderType, SkBlenderBase};
use crate::src::core::SkEffectPriv::SkStageRec;
// Native retained CPU blender. Intrusive ownership is composed from the canonical base.
impl SkBlendModeBlender {
    // Source: src/core/SkBlendModeBlender.h:23-23
    pub fn new(mode: SkBlendMode) -> Self {
        Self {
            fRefCnt: crate::include::core::SkRefCnt::SkRefCnt::new(),
            fMode: mode,
        }
    }
    // Source: src/core/SkBlendModeBlender.h:25-25
    pub fn r#type(&self) -> BlenderType {
        BlenderType::kBlendMode
    }
    // Source: src/core/SkBlendModeBlender.h:26-26
    pub fn mode(&self) -> SkBlendMode {
        self.fMode
    }
    // Source: src/core/SkBlendModeBlender.h:28-28
    fn factory_impl(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    // Source: src/core/SkBlendModeBlender.h:33-33
    pub fn asBlendMode(&self) -> Option<SkBlendMode> {
        Some(self.fMode)
    }
}
// Source: src/core/SkBlendModeBlender.h:35-35
impl SkBlendModeBlender {
    fn flatten_dispatch(
        &self,
        writer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/core/SkBlendModeBlender.h:39-39
pub struct SkBlendModeBlender {
    fMode: SkBlendMode,
    // Native SkRefCnt inherited through SkFlattenable; one base counter and the selected mixin.
    fRefCnt: crate::include::core::SkRefCnt::SkRefCnt,
}
// Canonical virtual ownership/identity adapters for the inherited native bases.
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for SkBlendModeBlender {
    // Source: src/core/SkBlendModeBlender.h:21-21
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        self.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fRefCnt.ref_();
    }
}
impl crate::src::core::SkFlattenable::SkFlattenable for SkBlendModeBlender {
    unsafe fn project_path_effect_owner(
        &self,
        _: std::ptr::NonNull<()>,
    ) -> Option<std::ptr::NonNull<dyn crate::src::core::SkPathEffect::SkPathEffect>> {
        None
    }
}
impl crate::src::core::SkFlattenable::SkFlattenableView for SkBlendModeBlender {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        self.factory_impl()
    }
    // Source: src/core/SkBlendModeBlender.h:35-35
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }

    fn getTypeName(&self) -> Option<&str> {
        Some("SkBlendModeBlender")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        <Self as SkBlenderBase>::GetFlattenableType()
    }
}
impl crate::include::core::SkBlender::SkBlender for SkBlendModeBlender {}

// Source: src/core/SkBlendModeBlender.cpp:20-63
// Each macro expansion has its own function-local OnceLock, retaining the
// original thread-safe local initialization and process-lifetime inline object.
pub fn GetBlendModeSingleton(
    mode: SkBlendMode,
) -> &'static dyn crate::include::core::SkBlender::SkBlender {
    macro_rules! singleton {
        ($mode:ident) => {{
            static BLENDER: std::sync::OnceLock<
                crate::src::core::SkNoDestructor::SkNoDestructor<SkBlendModeBlender>,
            > = std::sync::OnceLock::new();
            BLENDER
                .get_or_init(|| {
                    crate::src::core::SkNoDestructor::SkNoDestructor::new(SkBlendModeBlender::new(
                        SkBlendMode::$mode,
                    ))
                })
                .get()
        }};
    }
    match mode {
        SkBlendMode::kClear => singleton!(kClear),
        SkBlendMode::kSrc => singleton!(kSrc),
        SkBlendMode::kDst => singleton!(kDst),
        SkBlendMode::kSrcOver => singleton!(kSrcOver),
        SkBlendMode::kDstOver => singleton!(kDstOver),
        SkBlendMode::kSrcIn => singleton!(kSrcIn),
        SkBlendMode::kDstIn => singleton!(kDstIn),
        SkBlendMode::kSrcOut => singleton!(kSrcOut),
        SkBlendMode::kDstOut => singleton!(kDstOut),
        SkBlendMode::kSrcATop => singleton!(kSrcATop),
        SkBlendMode::kDstATop => singleton!(kDstATop),
        SkBlendMode::kXor => singleton!(kXor),
        SkBlendMode::kPlus => singleton!(kPlus),
        SkBlendMode::kModulate => singleton!(kModulate),
        SkBlendMode::kScreen => singleton!(kScreen),
        SkBlendMode::kOverlay => singleton!(kOverlay),
        SkBlendMode::kDarken => singleton!(kDarken),
        SkBlendMode::kLighten => singleton!(kLighten),
        SkBlendMode::kColorDodge => singleton!(kColorDodge),
        SkBlendMode::kColorBurn => singleton!(kColorBurn),
        SkBlendMode::kHardLight => singleton!(kHardLight),
        SkBlendMode::kSoftLight => singleton!(kSoftLight),
        SkBlendMode::kDifference => singleton!(kDifference),
        SkBlendMode::kExclusion => singleton!(kExclusion),
        SkBlendMode::kMultiply => singleton!(kMultiply),
        SkBlendMode::kHue => singleton!(kHue),
        SkBlendMode::kSaturation => singleton!(kSaturation),
        SkBlendMode::kColor => singleton!(kColor),
        SkBlendMode::kLuminosity => singleton!(kLuminosity),
    }
}
// Source: src/core/SkBlendModeBlender.cpp:65-67
pub fn BlenderMode(
    mode: SkBlendMode,
) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::include::core::SkBlender::SkBlender> {
    unsafe {
        crate::include::core::SkRefCnt::sk_ref_sp(Some(core::ptr::NonNull::from(
            GetBlendModeSingleton(mode),
        )))
    }
}

// Source: src/core/SkBlendModeBlender.cpp:69-72
impl SkBlendModeBlender {
    pub fn CreateProc(
        buffer: &mut super::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut mode = buffer.readUInt();
        if !buffer.validate(mode <= SkBlendMode::kLastMode as u32) {
            mode = 0;
        }
        // Every validated u8 discriminant through kLastMode is present, contiguous in native order.
        let mode: SkBlendMode = unsafe { std::mem::transmute(mode as u8) };
        Ok(Some(super::SkFlattenable::FlattenableOwner::Blender(
            BlenderMode(mode),
        )))
    }
    // Source: src/core/SkBlendModeBlender.cpp:74-76
    fn flatten_impl(
        &self,
        buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeInt(self.fMode as i32);
        Ok(())
    }
}

// Rust dispatch adapters only; original method bodies are mapped above.
impl SkBlenderBase for SkBlendModeBlender {
    fn r#type(&self) -> BlenderType {
        SkBlendModeBlender::r#type(self)
    }
    fn asBlendMode(&self) -> Option<SkBlendMode> {
        SkBlendModeBlender::asBlendMode(self)
    }
    // Source: src/core/SkBlendModeBlender.cpp:78-81
    unsafe fn onAppendStages(&self, rec: &SkStageRec<'_>) -> Result<bool, crate::RasterError> {
        SkBlendMode_AppendStages(self.fMode, &mut *rec.fPipeline);
        Ok(true)
    }
}
// Source: src/core/SkBlendModeBlender.cpp:83-101
pub fn affectsTransparentBlack<B: SkBlenderBase + ?Sized>(blender: &B) -> bool {
    if let Some(blendMode) = blender.asBlendMode() {
        if let Some((_src, dst)) = SkBlendMode_AsCoeff(blendMode) {
            dst != SkBlendModeCoeff::kOne
                && dst != SkBlendModeCoeff::kISA
                && dst != SkBlendModeCoeff::kISC
        } else {
            false
        }
    } else {
        true
    }
}
