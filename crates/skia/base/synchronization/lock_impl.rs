use super::lock_metrics_recorder_tags::LockMetricTagList;
use super::lock_subtle::LockTracking;
use core::{cell::UnsafeCell, marker::PhantomPinned, mem::MaybeUninit, pin::Pin};
// Source: base/synchronization/lock_impl.h:45-122. Actual mutex stays at its final address.
#[cfg(any(base_is_posix, base_is_fuchsia))]
pub struct LockImpl {
    pub(crate) native_handle_: UnsafeCell<MaybeUninit<libc::pthread_mutex_t>>,
    pub(crate) _pin: PhantomPinned,
}
#[cfg(any(base_is_posix, base_is_fuchsia))]
unsafe impl Send for LockImpl {}
#[cfg(any(base_is_posix, base_is_fuchsia))]
unsafe impl Sync for LockImpl {}
#[cfg(any(base_is_posix, base_is_fuchsia))]
impl LockImpl {
    pub(crate) unsafe fn native_handle(&self) -> *mut libc::pthread_mutex_t {
        (*self.native_handle_.get()).as_mut_ptr()
    }
    // Source: base/synchronization/lock_impl.h:125-138
    pub(crate) unsafe fn Lock(&self, tags: &LockMetricTagList) {
        if self.Try() {
            return;
        }
        self.LockInternal(tags);
    }
    // Source: base/synchronization/lock_impl.h:162-180
    pub(crate) unsafe fn Try(&self) -> bool {
        let rv = libc::pthread_mutex_trylock(self.native_handle());
        #[cfg(base_dcheck_is_on)]
        super::lock_impl_posix::dcheck_trylock_result(rv);
        rv == 0
    }
    pub(crate) unsafe fn Unlock(&self) {
        let rv = libc::pthread_mutex_unlock(self.native_handle());
        #[cfg(base_dcheck_is_on)]
        super::lock_impl_posix::dcheck_unlock_result(rv);
    }
}
// Source: base/synchronization/lock_impl.h:183-418. A template capability maps only real Lock methods.
// Implementors must obey native same-thread exclusive-acquisition and nonmovable identity contracts.
pub unsafe trait LockType {
    unsafe fn Acquire(self: Pin<&Self>, tracking: LockTracking);
    unsafe fn Release(self: Pin<&Self>);
    unsafe fn Try(self: Pin<&Self>, tracking: LockTracking) -> bool;
    unsafe fn AssertAcquired(self: Pin<&Self>);
}
pub struct AlreadyAcquired;
#[must_use]
pub struct BasicAutoLock<'a, L: LockType + ?Sized> {
    lock_: Pin<&'a L>,
    _thread: core::marker::PhantomData<*mut ()>,
}
impl<'a, L: LockType + ?Sized> BasicAutoLock<'a, L> {
    pub unsafe fn new(lock: Pin<&'a L>) -> Self {
        Self::with_tracking(lock, LockTracking::kDisabled)
    }
    pub unsafe fn with_tracking(lock: Pin<&'a L>, tracking: LockTracking) -> Self {
        lock.Acquire(tracking);
        Self {
            lock_: lock,
            _thread: core::marker::PhantomData,
        }
    }
    pub unsafe fn already_acquired(lock: Pin<&'a L>, _: AlreadyAcquired) -> Self {
        lock.AssertAcquired();
        Self {
            lock_: lock,
            _thread: core::marker::PhantomData,
        }
    }
}
impl<L: LockType + ?Sized> Drop for BasicAutoLock<'_, L> {
    fn drop(&mut self) {
        unsafe {
            self.lock_.AssertAcquired();
            self.lock_.Release();
        }
    }
}
#[must_use]
pub struct BasicMovableAutoLock<'a, L: LockType + ?Sized> {
    lock_: Option<Pin<&'a L>>,
    _thread: core::marker::PhantomData<*mut ()>,
}
impl<'a, L: LockType + ?Sized> BasicMovableAutoLock<'a, L> {
    pub unsafe fn new(lock: Pin<&'a L>, tracking: LockTracking) -> Self {
        lock.Acquire(tracking);
        Self {
            lock_: Some(lock),
            _thread: core::marker::PhantomData,
        }
    }
    pub fn move_from(other: &mut Self) -> Self {
        Self {
            lock_: other.lock_.take(),
            _thread: core::marker::PhantomData,
        }
    }
}
impl<L: LockType + ?Sized> Drop for BasicMovableAutoLock<'_, L> {
    fn drop(&mut self) {
        if let Some(lock) = self.lock_ {
            unsafe {
                lock.AssertAcquired();
                lock.Release();
            }
        }
    }
}
#[must_use]
pub struct BasicAutoTryLock<'a, L: LockType + ?Sized> {
    lock_: Pin<&'a L>,
    is_acquired_: bool,
    _thread: core::marker::PhantomData<*mut ()>,
}
impl<'a, L: LockType + ?Sized> BasicAutoTryLock<'a, L> {
    pub unsafe fn new(lock: Pin<&'a L>, tracking: LockTracking) -> Self {
        let is_acquired_ = lock.Try(tracking);
        Self {
            lock_: lock,
            is_acquired_,
            _thread: core::marker::PhantomData,
        }
    }
    pub fn is_acquired(&self) -> bool {
        self.is_acquired_
    }
}
impl<L: LockType + ?Sized> Drop for BasicAutoTryLock<'_, L> {
    fn drop(&mut self) {
        if self.is_acquired_ {
            unsafe {
                self.lock_.AssertAcquired();
                self.lock_.Release();
            }
        }
    }
}
#[must_use]
pub struct BasicAutoUnlock<'a, L: LockType + ?Sized> {
    lock_: Pin<&'a L>,
    _thread: core::marker::PhantomData<*mut ()>,
}
impl<'a, L: LockType + ?Sized> BasicAutoUnlock<'a, L> {
    pub unsafe fn new(lock: Pin<&'a L>) -> Self {
        lock.AssertAcquired();
        lock.Release();
        Self {
            lock_: lock,
            _thread: core::marker::PhantomData,
        }
    }
}
impl<L: LockType + ?Sized> Drop for BasicAutoUnlock<'_, L> {
    fn drop(&mut self) {
        unsafe {
            self.lock_.Acquire(LockTracking::kDisabled);
        }
    }
}
#[must_use]
pub struct BasicAutoLockMaybe<'a, L: LockType + ?Sized> {
    lock_: Option<Pin<&'a L>>,
    _thread: core::marker::PhantomData<*mut ()>,
}
impl<'a, L: LockType + ?Sized> BasicAutoLockMaybe<'a, L> {
    pub unsafe fn new(lock: Option<Pin<&'a L>>, tracking: LockTracking) -> Self {
        if let Some(l) = lock {
            l.Acquire(tracking);
        }
        Self {
            lock_: lock,
            _thread: core::marker::PhantomData,
        }
    }
}
impl<L: LockType + ?Sized> Drop for BasicAutoLockMaybe<'_, L> {
    fn drop(&mut self) {
        if let Some(l) = self.lock_ {
            unsafe {
                l.AssertAcquired();
                l.Release();
            }
        }
    }
}
#[must_use]
pub struct BasicReleasableAutoLock<'a, L: LockType + ?Sized> {
    lock_: Option<Pin<&'a L>>,
    _thread: core::marker::PhantomData<*mut ()>,
}
impl<'a, L: LockType + ?Sized> BasicReleasableAutoLock<'a, L> {
    pub unsafe fn new(lock: Pin<&'a L>, tracking: LockTracking) -> Self {
        lock.Acquire(tracking);
        Self {
            lock_: Some(lock),
            _thread: core::marker::PhantomData,
        }
    }
    pub unsafe fn Release(&mut self) {
        let l = self.lock_.unwrap_unchecked();
        l.AssertAcquired();
        l.Release();
        self.lock_ = None;
    }
}
impl<L: LockType + ?Sized> Drop for BasicReleasableAutoLock<'_, L> {
    fn drop(&mut self) {
        if let Some(l) = self.lock_ {
            unsafe {
                l.AssertAcquired();
                l.Release();
            }
        }
    }
}
