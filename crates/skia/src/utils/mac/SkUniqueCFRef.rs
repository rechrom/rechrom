#![cfg(target_vendor = "apple")]
use core::{ffi::c_void, ptr::NonNull};
#[cfg(target_vendor = "apple")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: *const c_void);
}
// Source: src/utils/mac/SkUniqueCFRef.h:20-21
// The parameter is the opaque CF pointee rather than the C++ pointer typedef.
// This is the unique +1 CF owner, with no second reference counter or Clone.
#[repr(transparent)]
pub struct SkUniqueCFRef<T> {
    value: Option<NonNull<T>>,
}
impl<T> SkUniqueCFRef<T> {
    /// Safety: a nonnull pointer transfers exactly one CFRelease-compatible +1
    /// native ownership reference; no other unique owner consumes that reference.
    pub unsafe fn from_raw(value: *mut T) -> Self {
        Self {
            value: NonNull::new(value),
        }
    }
    pub fn get(&self) -> *mut T {
        self.value.map_or(core::ptr::null_mut(), NonNull::as_ptr)
    }
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }
    pub fn release(&mut self) -> *mut T {
        self.value
            .take()
            .map_or(core::ptr::null_mut(), NonNull::as_ptr)
    }
    /// Safety: the replacement transfers its own native +1 ownership reference.
    pub unsafe fn reset(&mut self, value: *mut T) {
        let old = core::mem::replace(self, unsafe { Self::from_raw(value) });
        drop(old);
    }
}
impl<T> Default for SkUniqueCFRef<T> {
    fn default() -> Self {
        Self { value: None }
    }
}
#[cfg(target_vendor = "apple")]
impl<T> Drop for SkUniqueCFRef<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            unsafe {
                CFRelease(value.as_ptr().cast());
            }
        }
    }
}
