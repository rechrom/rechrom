#[cfg(base_dcheck_is_on)]
pub use super::thread_local_internal::CheckedThreadLocalOwnedPointer as ThreadLocalOwnedPointer;
use super::thread_local_storage::Slot;
use core::{
    ffi::c_void,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    ptr,
};
// Source: base/threading/thread_local.h: release actual owning pointer, one native TLS Slot.
#[cfg(not(base_dcheck_is_on))]
pub struct ThreadLocalOwnedPointer<T> {
    slot_: Slot,
    _type: PhantomData<T>,
    _pin: PhantomPinned,
}
#[cfg(not(base_dcheck_is_on))]
impl<T> ThreadLocalOwnedPointer<T> {
    pub unsafe fn init_at(out: *mut Self) {
        core::ptr::addr_of_mut!((*out).slot_).write(Slot::new(Some(Self::DeleteTlsPtr)));
        core::ptr::addr_of_mut!((*out)._type).write(PhantomData);
        core::ptr::addr_of_mut!((*out)._pin).write(PhantomPinned);
    }
    pub unsafe fn Get(self: Pin<&Self>) -> *mut T {
        self.slot_.Get().cast()
    }
    // Exact return of prior unique owner; replacing Set does not prematurely destroy it.
    pub unsafe fn Set(self: Pin<&Self>, value: Option<Box<T>>) -> Option<Box<T>> {
        let old = self.Get();
        let existing = if old.is_null() {
            None
        } else {
            Some(Box::from_raw(old))
        };
        self.slot_
            .Set(value.map_or(ptr::null_mut(), |p| Box::into_raw(p).cast()));
        existing
    }
    unsafe extern "C-unwind" fn DeleteTlsPtr(value: *mut c_void) {
        if !value.is_null() {
            drop(Box::from_raw(value.cast::<T>()));
        }
    }
}
#[cfg(not(base_dcheck_is_on))]
impl<T> Drop for ThreadLocalOwnedPointer<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Pin::new_unchecked(&*self).Set(None));
        }
    }
}
