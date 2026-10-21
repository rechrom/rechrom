use super::thread_local_storage::Slot;
use core::{
    ffi::c_void,
    marker::PhantomPinned,
    pin::Pin,
    ptr,
    sync::atomic::{AtomicI32, Ordering},
};
// Source: base/threading/thread_local_internal.h: actual checked unique-pointer tracker.
// num_assigned_threads_ is the native diagnostic count, not a second T reference count.
pub struct CheckedThreadLocalOwnedPointer<T> {
    num_assigned_threads_: AtomicI32,
    slot_: Slot,
    _type: core::marker::PhantomData<T>,
    _pin: PhantomPinned,
}
struct PtrTracker<T> {
    ptr_: Option<Box<T>>,
    outer_: *const CheckedThreadLocalOwnedPointer<T>,
}
impl<T> PtrTracker<T> {
    unsafe fn init_at(
        out: *mut Self,
        outer: *const CheckedThreadLocalOwnedPointer<T>,
        value: Box<T>,
    ) {
        core::ptr::addr_of_mut!((*out).outer_).write(outer);
        core::ptr::addr_of_mut!((*out).ptr_).write(Some(value));
        (*outer)
            .num_assigned_threads_
            .fetch_add(1, Ordering::Relaxed);
    }
}
impl<T> Drop for PtrTracker<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.outer_)
                .num_assigned_threads_
                .fetch_sub(1, Ordering::Relaxed);
        }
    }
}
impl<T> CheckedThreadLocalOwnedPointer<T> {
    pub unsafe fn init_at(out: *mut Self) {
        core::ptr::addr_of_mut!((*out).slot_).write(Slot::new(Some(Self::DeleteTlsPtr)));
        core::ptr::addr_of_mut!((*out).num_assigned_threads_).write(AtomicI32::new(0));
        core::ptr::addr_of_mut!((*out)._type).write(core::marker::PhantomData);
        core::ptr::addr_of_mut!((*out)._pin).write(PhantomPinned);
    }
    pub unsafe fn Get(self: Pin<&Self>) -> *mut T {
        let tracker = self.slot_.Get().cast::<PtrTracker<T>>();
        if tracker.is_null() {
            ptr::null_mut()
        } else {
            (*tracker)
                .ptr_
                .as_ref()
                .map_or(ptr::null_mut(), |p| core::ptr::from_ref(&**p).cast_mut())
        }
    }
    // Safety: the pinned outer remains alive until all assigned thread values are gone.
    pub unsafe fn Set(self: Pin<&Self>, value: Option<Box<T>>) -> Option<Box<T>> {
        let old = self.slot_.Get().cast::<PtrTracker<T>>();
        let mut existing = None;
        if !old.is_null() {
            let mut tracker = Box::from_raw(old);
            existing = tracker.ptr_.take();
            drop(tracker);
        }
        if let Some(value) = value {
            let owner = Box::<PtrTracker<T>>::new_uninit();
            let tracker = Box::into_raw(owner).cast::<PtrTracker<T>>();
            PtrTracker::init_at(tracker, &*self, value);
            self.slot_.Set(tracker.cast());
        } else {
            self.slot_.Set(ptr::null_mut());
        }
        existing
    }
    unsafe extern "C-unwind" fn DeleteTlsPtr(value: *mut c_void) {
        if !value.is_null() {
            drop(Box::from_raw(value.cast::<PtrTracker<T>>()));
        }
    }
}
impl<T> Drop for CheckedThreadLocalOwnedPointer<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Pin::new_unchecked(&*self).Set(None));
            // Actual streamed DCHECK_EQ failure text needs the genuine base CheckError stream protocol.
            crate::DCHECK_EQ!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/threading/thread_local_internal.h\0".as_ptr().cast(),41);self.num_assigned_threads_.load(Ordering::Relaxed),0;@stream b"Memory leak: Must join all threads or release all associated thread-local slots before ~ThreadLocalOwnedPointer\0".as_ptr());
        }
    }
}
