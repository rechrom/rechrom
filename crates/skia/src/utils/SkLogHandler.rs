use crate::include::core::SkRefCnt::{sk_ref_sp, sk_sp, SkRefCntObject, SkSafeUnref};
use crate::include::private::SkLogPriority::SkLogPriority;
use core::ffi::VaList;
use core::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
// Source: include/utils/SkLogHandler.h:21-23
// Source: include/utils/SkLogHandler.h:32-32
// Actual retained abstract handler, not a concrete provider. Implementors retain
// their own native intrusive counter and destruction through SkRefCntObject.
pub trait SkLogHandler: SkRefCntObject + Send + Sync {
    /// Safety: format/pointers and the copied va_list are valid only through
    /// this synchronous callback. Callback is thread-safe and may consume its copy.
    unsafe fn onLog(&self, priority: SkLogPriority, format: *const u8, args: VaList<'_>);
}
// Source: src/utils/SkLogHandler.cpp:14-14
// Rust trait pointers carry data+vtable metadata. AtomicPtr needs a thin pointer,
// so this immutable, intentionally leaked two-word descriptor holds only that
// actual native pointer. It is not another handler/refcount/provider. A failed
// CAS releases the actual handler and frees only its unpublished descriptor.
static gUserLogHandler: AtomicPtr<NonNull<dyn SkLogHandler>> =
    AtomicPtr::new(core::ptr::null_mut());
impl dyn SkLogHandler {
    // Source: include/utils/SkLogHandler.h:38-38
    // Source: src/utils/SkLogHandler.cpp:16-25
    pub fn SetInstance(mut handler: sk_sp<dyn SkLogHandler>) -> bool {
        let h = handler.release();
        let candidate = h.map_or(core::ptr::null_mut(), |pointer| {
            Box::into_raw(Box::new(pointer))
        });
        if gUserLogHandler
            .compare_exchange(
                core::ptr::null_mut(),
                candidate,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            if !candidate.is_null() {
                unsafe {
                    drop(Box::from_raw(candidate));
                }
            }
            unsafe {
                SkSafeUnref(h);
            }
            return false;
        }
        // SK_INTENTIONALLY_LEAKED: the installed actual intrusive reference and
        // its immutable dispatch metadata remain live for the process lifetime.
        true
    }
    // Source: include/utils/SkLogHandler.h:43-43
    // Source: src/utils/SkLogHandler.cpp:27-29
    pub fn GetInstance() -> sk_sp<dyn SkLogHandler> {
        let descriptor = gUserLogHandler.load(Ordering::Acquire);
        let pointer = if descriptor.is_null() {
            None
        } else {
            Some(unsafe { *descriptor })
        };
        unsafe { sk_ref_sp(pointer) }
    }
}
// Source: include/utils/SkLogHandler.h:46-50
// Abstract protected construction and
// deleted copy/assignment have no concrete producer; trait objects cannot be copied.
