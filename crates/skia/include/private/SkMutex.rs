// Copyright 2015 Google Inc. BSD-style license.
use crate::src::core::SkSemaphore::SkSemaphore;
#[cfg(sk_debug)]
use crate::src::core::SkThreadID::{kIllegalThreadID, SkGetThreadID, SkThreadID};
#[cfg(sk_debug)]
use core::cell::UnsafeCell;
use core::{
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
    ptr::NonNull,
};

// Source: include/private/SkMutex.h:25-25
// Source: include/private/SkMutex.h:53-54
pub struct SkMutex {
    fSemaphore: SkSemaphore,
    #[cfg(sk_debug)]
    fOwner: UnsafeCell<SkThreadID>,
    // Native semaphore/atomic members cannot be copied or moved after publication.
    _pin: PhantomPinned,
}
// The real semaphore owns exclusion and its native acquire/release ordering.
// These bounds do not establish safety for deleting a contended OS backend.
unsafe impl Send for SkMutex {}
unsafe impl Sync for SkMutex {}
impl SkMutex {
    // Source: include/private/SkMutex.h:27-27
    pub const fn new() -> Self {
        Self {
            fSemaphore: SkSemaphore::new(1),
            #[cfg(sk_debug)]
            fOwner: UnsafeCell::new(kIllegalThreadID),
            _pin: PhantomPinned,
        }
    }
    // Source: include/private/SkMutex.h:33-36
    /// Safety: this is the original stable mutex allocation, alive throughout
    /// wait and subsequent ownership. Its OS semaphore stays alive throughout
    /// every in-flight wait/signal. A matching release is performed by this thread.
    pub unsafe fn acquire_raw(this: NonNull<Self>) {
        let semaphore = unsafe { core::ptr::addr_of!((*this.as_ptr()).fSemaphore) };
        unsafe {
            SkSemaphore::wait(semaphore);
        }
        #[cfg(sk_debug)]
        unsafe {
            (*this.as_ptr()).fOwner.get().write(SkGetThreadID());
        }
    }
    // Source: include/private/SkMutex.h:38-42
    /// Safety: the current thread owns this mutex; allocation and underlying OS
    /// semaphore survive the entire signal call. In particular this contract
    /// does NOT permit a newly awakened thread to delete a contended backend
    /// before its final OS signal returns. No field access follows publication here.
    pub unsafe fn release_raw(this: NonNull<Self>) {
        unsafe {
            Self::assertHeld_raw(this);
        }
        #[cfg(sk_debug)]
        unsafe {
            (*this.as_ptr()).fOwner.get().write(kIllegalThreadID);
        }
        let semaphore = unsafe { core::ptr::addr_of!((*this.as_ptr()).fSemaphore) };
        // Last operation: signal can publish the mutex to another thread.
        unsafe {
            SkSemaphore::signal(semaphore, 1);
        }
    }
    // Source: include/private/SkMutex.h:44-46
    /// Safety: the original allocation is live, and this thread owns the mutex.
    /// No concurrent owner-field mutation may race this native debug assertion.
    pub unsafe fn assertHeld_raw(this: NonNull<Self>) {
        #[cfg(sk_debug)]
        unsafe {
            crate::SkASSERT!(@source b"include/private/SkMutex.h\0".as_ptr(), 45; @condition b"fOwner == SkGetThreadID()\0".as_ptr(); ((*this.as_ptr()).fOwner.get().read()) == (SkGetThreadID()));
        }
        #[cfg(not(sk_debug))]
        let _ = this;
    }
    // Source: include/private/SkMutex.h:48-50
    /// Safety: the original allocation is live and no concurrent acquire/release
    /// mutates its native debug owner during this observation.
    pub unsafe fn assertNotHeld_raw(this: NonNull<Self>) {
        #[cfg(sk_debug)]
        unsafe {
            crate::SkASSERT!(@source b"include/private/SkMutex.h\0".as_ptr(), 49; @condition b"fOwner == kIllegalThreadID\0".as_ptr(); ((*this.as_ptr()).fOwner.get().read()) == (kIllegalThreadID));
        }
        #[cfg(not(sk_debug))]
        let _ = this;
    }
}
// Source: include/private/SkMutex.h:29-31
impl Drop for SkMutex {
    fn drop(&mut self) {
        // Exclusive final destruction excludes all outstanding raw operations.
        unsafe {
            Self::assertNotHeld_raw(NonNull::from(&mut *self));
        }
        // Rust then destroys the actual embedded native semaphore.
    }
}

// Source: include/private/SkMutex.h:57-57
// Source: include/private/SkMutex.h:62-66
// Source: include/private/SkMutex.h:69-69
// The native guard is constructed in caller-owned pinned storage, without a
// separate heap allocation. Neither the storage nor its live guard is movable.
#[must_use]
pub struct SkAutoMutexExclusive<'mutex> {
    fMutex: NonNull<SkMutex>,
    _lease: PhantomData<&'mutex SkMutex>,
    _pin: PhantomPinned,
}
// Unconstructed storage is Rust placement-construction glue. Option records
// only whether the native object exists; it adds no mutex owner or lock count.
#[must_use]
pub struct SkAutoMutexExclusiveStorage<'mutex> {
    guard: Option<SkAutoMutexExclusive<'mutex>>,
    _pin: PhantomPinned,
}
impl<'mutex> SkAutoMutexExclusiveStorage<'mutex> {
    pub const fn new() -> Self {
        Self {
            guard: None,
            _pin: PhantomPinned,
        }
    }
}
impl<'mutex> SkAutoMutexExclusive<'mutex> {
    // Source: include/private/SkMutex.h:59-59
    /// Safety: storage has not previously been initialized. It remains pinned
    /// until destruction and the original mutex allocation remains alive through
    /// release's final signal. The same OS wait/signal lifetime preconditions as
    /// acquire_raw/release_raw apply. The storage automatically drops this guard.
    pub unsafe fn init<'slot>(
        mut storage: Pin<&'slot mut SkAutoMutexExclusiveStorage<'mutex>>,
        mutex: Pin<&'mutex SkMutex>,
    ) -> Pin<&'slot mut Self> {
        let mutex = NonNull::from(mutex.get_ref());
        unsafe {
            SkMutex::acquire_raw(mutex);
        }
        let slot = unsafe { storage.as_mut().get_unchecked_mut() };
        slot.guard = Some(Self {
            fMutex: mutex,
            _lease: PhantomData,
            _pin: PhantomPinned,
        });
        unsafe { storage.map_unchecked_mut(|s| s.guard.as_mut().unwrap_unchecked()) }
    }
}
// Source: include/private/SkMutex.h:60-60
impl Drop for SkAutoMutexExclusive<'_> {
    fn drop(&mut self) {
        // The original borrowed owner cannot be destroyed until this signal
        // has returned. No guard/mutex field is read after release publishes it.
        let mutex = self.fMutex;
        unsafe {
            SkMutex::release_raw(mutex);
        }
    }
}
