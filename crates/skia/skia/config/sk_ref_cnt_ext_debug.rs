//! Chromium's selected debug SkRefCnt mixin; external embedding source, no Skia credit.
use crate::include::core::SkRefCnt::{
    SkRawDisposeProc, SkRefCntBase, SkRefCntDispose, SkRefCntObject, SkUnref,
};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI32, Ordering};
// Source: skia/config/sk_ref_cnt_ext_debug.h:22-41
// Adapted field representation: private Rust projection, not a C++ base-offset ABI.
// The empty derived destructor runs first, its own flags then the sole parent last.
pub struct SkRefCnt {
    // Source: skia/config/sk_ref_cnt_ext_debug.h:37-37
    flags_: AtomicI32,
    // Source: skia/config/sk_ref_cnt_ext_debug.h:22-22
    base: SkRefCntBase,
}
impl SkRefCnt {
    // Source: skia/config/sk_ref_cnt_ext_debug.h:43-43
    pub const fn new() -> Self {
        Self {
            base: SkRefCntBase::new(),
            flags_: AtomicI32::new(0),
        }
    }
    // Native public base-class projection, not a second counter.
    pub fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base
    }
    // Source: skia/config/sk_ref_cnt_ext_debug.h:26-26
    pub fn ref_(&self) {
        crate::SkASSERT!(@source b"skia/config/sk_ref_cnt_ext_debug.h\0".as_ptr(),26; @condition b"flags_.load() != AdoptionRequired_Flag\0".as_ptr(); self.flags_.load(Ordering::SeqCst)!=Self::AdoptionRequired_Flag);
        self.base.ref_();
    }
    // Source: skia/config/sk_ref_cnt_ext_debug.h:27-27
    /// # Safety
    /// Transfers one owned reference to the complete object containing this parent.
    /// The complete-object pointer and destructor metadata must satisfy SkUnref.
    pub unsafe fn deref<T: SkRefCntObject + ?Sized>(object: NonNull<T>) {
        unsafe {
            SkUnref(object);
        }
    }
    // Source: skia/config/sk_ref_cnt_ext_debug.h:29-30
    fn adopted(&self) {
        self.flags_.fetch_or(Self::Adopted_Flag, Ordering::SeqCst);
    }
    fn requireAdoption(&self) {
        self.flags_
            .fetch_or(Self::AdoptionRequired_Flag, Ordering::SeqCst);
    }
    // Source: skia/config/sk_ref_cnt_ext_debug.h:32-35
    const Adopted_Flag: i32 = 0x1;
    const AdoptionRequired_Flag: i32 = 0x2;
}
// Source: skia/config/sk_ref_cnt_ext_debug.h:45-45
impl Drop for SkRefCnt {
    fn drop(&mut self) {}
}
// Source: skia/config/sk_ref_cnt_ext_debug.h:22-22
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
// Source: skia/config/sk_ref_cnt_ext_debug.h:49-60
pub mod WTF {
    use super::SkRefCnt;
    // Source: skia/config/sk_ref_cnt_ext_debug.h:50-54
    pub fn adopted(object: Option<&SkRefCnt>) {
        let Some(object) = object else {
            return;
        };
        object.adopted();
    }
    // Source: skia/config/sk_ref_cnt_ext_debug.h:55-59
    pub fn requireAdoption(object: Option<&SkRefCnt>) {
        let Some(object) = object else {
            return;
        };
        object.requireAdoption();
    }
}
// Source: skia/config/sk_ref_cnt_ext_debug.h:62-63
pub use WTF::{adopted, requireAdoption};
