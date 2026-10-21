//! Chromium's selected release SkRefCnt mixin; external embedding source, no Skia credit.
use crate::include::core::SkRefCnt::{
    SkRawDisposeProc, SkRefCntBase, SkRefCntDispose, SkRefCntObject, SkUnref,
};
use std::ptr::NonNull;
// Source: skia/config/sk_ref_cnt_ext_release.h:13-16
pub struct SkRefCnt {
    base: SkRefCntBase,
}
impl SkRefCnt {
    // Implicit native construction initializes only the actual SkRefCntBase subobject.
    pub const fn new() -> Self {
        Self {
            base: SkRefCntBase::new(),
        }
    }
    pub fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    pub fn ref_(&self) {
        self.base.ref_();
    }
    // Source: skia/config/sk_ref_cnt_ext_release.h:15-15
    /// # Safety
    /// Transfers one owned reference to the complete object containing this parent.
    /// The complete-object pointer and destructor metadata must satisfy SkUnref.
    pub unsafe fn deref<T: SkRefCntObject + ?Sized>(object: NonNull<T>) {
        unsafe {
            SkUnref(object);
        }
    }
}
// Source: skia/config/sk_ref_cnt_ext_release.h:13-13
impl std::ops::Deref for SkRefCnt {
    type Target = SkRefCntBase;
    fn deref(&self) -> &SkRefCntBase {
        &self.base
    }
}
unsafe impl SkRefCntObject for SkRefCnt {
    fn internal_dispose_raw_proc(&self) -> SkRawDisposeProc {
        SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    fn ref_(&self) {
        SkRefCnt::ref_(self);
    }
}
// Source: skia/config/sk_ref_cnt_ext_release.h:18-21
pub mod WTF {
    use super::SkRefCnt;
    // Source: skia/config/sk_ref_cnt_ext_release.h:19-20
    pub fn Adopted(_object: Option<&SkRefCnt>) {}
    pub fn RequireAdoption(_object: Option<&SkRefCnt>) {}
}
// Source: skia/config/sk_ref_cnt_ext_release.h:23-24
pub use WTF::{Adopted, RequireAdoption};
