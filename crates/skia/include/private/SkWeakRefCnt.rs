//! Native weak intrusive parent. Both counters remain in the complete object;
//! virtual glue records no owner, allocation, registry or additional counter.
use crate::include::core::SkRefCnt::{
    SkRawDisposeProc, SkRefCnt, SkRefCntBase, SkRefCntBaseDispose, SkRefCntObject,
};
use core::sync::atomic::{AtomicI32, Ordering};
use core::{ops::Deref, ptr::NonNull};

// Source: include/private/SkWeakRefCnt.h:55-56,168,170-171
// Private Rust field order destroys the actual weak member before its parent.
// This representation supplies inherited projection, not C++ object ABI.
pub struct SkWeakRefCnt {
    // Source: include/private/SkWeakRefCnt.h:167-168
    fWeakCnt: AtomicI32,
    // Source: include/private/SkWeakRefCnt.h:55,170
    parent: SkRefCnt,
}
impl SkWeakRefCnt {
    // Source: include/private/SkWeakRefCnt.h:62
    pub const fn new() -> Self {
        // Native parent construction precedes the member initialization.
        Self {
            parent: SkRefCnt::new(),
            fWeakCnt: AtomicI32::new(1),
        }
    }
}
// Source: include/private/SkWeakRefCnt.h:66-71
impl Drop for SkWeakRefCnt {
    fn drop(&mut self) {
        #[cfg(sk_debug)]
        {
            crate::SkASSERT!(@source b"include/private/SkWeakRefCnt.h\0".as_ptr(),68; @condition b"getWeakCnt() == 1\0".as_ptr(); self.getWeakCnt()==1);
            self.fWeakCnt.store(0, Ordering::Relaxed);
        }
    }
}
impl SkWeakRefCnt {
    // Source: include/private/SkWeakRefCnt.h:73-78
    #[cfg(sk_debug)]
    pub fn getWeakCnt(&self) -> i32 {
        self.fWeakCnt.load(Ordering::Relaxed)
    }

    // Source: include/private/SkWeakRefCnt.h:84-93
    fn atomic_conditional_acquire_strong_ref(&self) -> i32 {
        let strong = &self.parent.ref_cnt_base().fRefCnt;
        let mut prev = strong.load(Ordering::Relaxed);
        loop {
            if prev == 0 {
                break;
            }
            match strong.compare_exchange_weak(
                prev,
                prev.wrapping_add(1),
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => prev = actual,
            }
        }
        prev
    }

    // Source: include/private/SkWeakRefCnt.h:103-110
    #[must_use]
    pub fn try_ref(&self) -> bool {
        if self.atomic_conditional_acquire_strong_ref() != 0 {
            return true;
        }
        false
    }

    // Source: include/private/SkWeakRefCnt.h:115-120
    pub fn weak_ref(&self) {
        crate::SkASSERT!(@source b"include/private/SkWeakRefCnt.h\0".as_ptr(),116; @condition b"getRefCnt() > 0\0".as_ptr(); self.parent.ref_cnt_base().fRefCnt.load(Ordering::Relaxed)>0);
        crate::SkASSERT!(@source b"include/private/SkWeakRefCnt.h\0".as_ptr(),117; @condition b"getWeakCnt() > 0\0".as_ptr(); self.getWeakCnt()>0);
        self.fWeakCnt.fetch_add(1, Ordering::Relaxed);
    }

    // Source: include/private/SkWeakRefCnt.h:127-139
    /// Safety: object transfers one real weak reference on its original complete
    /// allocation. It stays stationary; final concrete disposal must match that
    /// allocation. Initial stack/static ownership cannot be passed for deletion.
    pub unsafe fn weak_unref<T: SkWeakRefCntObject + ?Sized>(object: NonNull<T>) {
        unsafe {
            SkWeakUnref(object);
        }
    }

    // Source: include/private/SkWeakRefCnt.h:144-146
    pub fn weak_expired(&self) -> bool {
        self.parent.ref_cnt_base().fRefCnt.load(Ordering::Relaxed) == 0
    }
}

// Source: include/private/SkWeakRefCnt.h:154-155,162-165
/// Actual virtual projection of the weak parent in the same complete object.
/// Safety: the parent is unique, initially strong=weak=1. Raw slots pair the
/// original data pointer with that complete concrete allocation's real type.
/// The weak-disposed state retains the object until the final weak release.
pub unsafe trait SkWeakRefCntObject: SkRefCntObject {
    fn weak_ref_cnt(&self) -> &SkWeakRefCnt;
    // Source: include/private/SkWeakRefCnt.h:154-155
    // This empty method is the actual native default virtual implementation.
    fn weak_dispose(&self) {}
    fn weak_final_dispose_raw_proc(&self) -> SkRawDisposeProc;
}

// Source: include/private/SkWeakRefCnt.h:127-139
/// Safety: same original complete allocation and one owned weak reference as
/// SkWeakRefCnt::weak_unref. No whole-object borrow survives into final disposal.
pub unsafe fn SkWeakUnref<T: SkWeakRefCntObject + ?Sized>(object: NonNull<T>) {
    let dispose = {
        let complete = unsafe { object.as_ref() };
        let weak = complete.weak_ref_cnt();
        crate::SkASSERT!(@source b"include/private/SkWeakRefCnt.h\0".as_ptr(),128; @condition b"getWeakCnt() > 0\0".as_ptr(); weak.getWeakCnt()>0);
        if weak.fWeakCnt.fetch_add(-1, Ordering::AcqRel) == 1 {
            #[cfg(sk_debug)]
            {
                // Source: include/private/SkWeakRefCnt.h:133-136
                weak.fWeakCnt.store(1, Ordering::Relaxed);
            }
            // Source: include/private/SkWeakRefCnt.h:137
            Some(complete.weak_final_dispose_raw_proc())
        } else {
            None
        }
    };
    if let Some(dispose) = dispose {
        unsafe {
            dispose(object.cast::<()>());
        }
    }
}

// Source: include/private/SkWeakRefCnt.h:162-165
/// Safety: selected only after the sole strong counter reaches zero. This is
/// raw virtual glue for T's original complete allocation, not Box ownership.
pub unsafe fn SkWeakRefCntStrongDispose<T: SkWeakRefCntObject>(object: NonNull<()>) {
    let complete = object.cast::<T>();
    // The actual virtual receiver borrow ends at the end of this statement;
    // no whole-object borrow crosses the subsequent possibly-final weak_unref.
    unsafe { complete.as_ref() }.weak_dispose();
    unsafe {
        SkWeakUnref(complete);
    }
}

// Source: include/private/SkWeakRefCnt.h:137
/// Safety: selected only by the last weak reference. This qualified native base
/// call bypasses strong-dispose and destroys/frees the same complete object.
pub unsafe fn SkWeakRefCntFinalDispose<T: SkWeakRefCntObject>(object: NonNull<()>) {
    unsafe {
        SkRefCntBaseDispose(object.cast::<T>());
    }
}

// Source: include/private/SkWeakRefCnt.h:55,170
impl Deref for SkWeakRefCnt {
    type Target = SkRefCnt;
    fn deref(&self) -> &SkRefCnt {
        &self.parent
    }
}
unsafe impl SkRefCntObject for SkWeakRefCnt {
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.parent.ref_cnt_base()
    }
    fn ref_(&self) {
        self.parent.ref_();
    }
    fn internal_dispose_raw_proc(&self) -> SkRawDisposeProc {
        SkWeakRefCntStrongDispose::<Self>
    }
}
unsafe impl SkWeakRefCntObject for SkWeakRefCnt {
    fn weak_ref_cnt(&self) -> &SkWeakRefCnt {
        self
    }
    fn weak_final_dispose_raw_proc(&self) -> SkRawDisposeProc {
        SkWeakRefCntFinalDispose::<Self>
    }
}
