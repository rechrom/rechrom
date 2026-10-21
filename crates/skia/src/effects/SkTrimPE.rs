// Copyright 2018 Google LLC. BSD-style license.
use super::SkTrimPathEffect::Mode;
use crate::include::core::SkRefCnt::{SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::{
    SkFlattenable::{Factory, SkFlattenable, SkFlattenableView, Type},
    SkPathEffect::SkPathEffect,
    SkWriteBuffer::SkWriteBuffer,
};
// Source: src/effects/SkTrimPE.h:14-37
// This is the actual private native class; its implementation source remains
// SkTrimPathEffect.cpp. No synthetic SkTrimPE.cpp or separate owner is created.
pub struct SkTrimPE {
    pub(crate) fStartT: f32,
    pub(crate) fStopT: f32,
    pub(crate) fMode: Mode,
    pub(crate) base: SkRefCnt,
}
// Source: src/effects/SkTrimPE.h:24-24
impl SkTrimPE {
    // Source: src/effects/SkTrimPE.h:26-30
    pub(crate) fn computeFastBounds_impl(
        &self,
        _: Option<&mut crate::src::core::SkRect::SkRect>,
    ) -> bool {
        true
    }

    pub fn RegisterFlattenables() {
        #[cfg(not(sk_disable_effect_deserialization))]
        unsafe {
            crate::src::core::SkFlattenable::Register(b"SkTrimPE", Self::CreateProc);
        }
    }
}
// Structural projection of the original complete intrusive object and hooks.
unsafe impl SkRefCntObject for SkTrimPE {
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    fn ref_(&self) {
        SkRefCnt::ref_(&self.base);
    }
}
impl SkFlattenable for SkTrimPE {
    unsafe fn project_path_effect_owner(
        &self,
        allocation: core::ptr::NonNull<()>,
    ) -> Option<core::ptr::NonNull<dyn SkPathEffect>> {
        let concrete: *mut Self = allocation.as_ptr().cast();
        Some(unsafe { core::ptr::NonNull::new_unchecked(concrete as *mut dyn SkPathEffect) })
    }
}
impl SkFlattenableView for SkTrimPE {
    fn getFactory(&self) -> Option<Factory> {
        if cfg!(sk_disable_effect_deserialization) {
            None
        } else {
            Some(Self::CreateProc)
        }
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkTrimPE")
    }
    fn getFlattenableType(&self) -> Type {
        Type::kSkPathEffect_Type
    }
    fn flatten(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)?;
        Ok(())
    }
}
