use super::{
    lock_impl::{LockImpl, LockType},
    lock_metrics_recorder_tags::{LockMetricTag, LockMetricTagList},
    lock_subtle::LockTracking,
};
use core::{mem::MaybeUninit, pin::Pin};
// Source: base/synchronization/lock.cc: actual static constexpr BaseLock tag/list.
static BASE_LOCK_TAG: LockMetricTag = LockMetricTag::new("BaseLock");
static BASE_LOCK_TAGS: LockMetricTagList = LockMetricTagList::new(&BASE_LOCK_TAG);
// Source: base/synchronization/lock.h:29-155. Native OS storage is never moved after initialization.
pub struct Lock {
    #[cfg(base_dcheck_is_on)]
    owning_thread_ref_:
        core::cell::UnsafeCell<crate::base::threading::platform_thread_ref::PlatformThreadRef>,
    #[cfg(base_dcheck_is_on)]
    in_tracked_locks_held_by_current_thread_: core::cell::UnsafeCell<bool>,
    lock_: LockImpl,
}
unsafe impl Send for Lock {}
unsafe impl Sync for Lock {}
impl Lock {
    pub fn GetBaseLockMetricTag() -> &'static LockMetricTag {
        &BASE_LOCK_TAG
    }
    pub fn GetBaseLockMetricTagList() -> &'static LockMetricTagList {
        &BASE_LOCK_TAGS
    }
    // Adapted native default construction: caller's existing storage, no extra allocation/owner.
    // Safety: storage remains pinned until drop; native constructor's pthread preconditions apply.
    pub unsafe fn new(storage: Pin<&mut MaybeUninit<Self>>) -> Pin<&mut Self> {
        let p = Pin::get_unchecked_mut(storage).as_mut_ptr();
        #[cfg(base_dcheck_is_on)]
        core::ptr::addr_of_mut!((*p).owning_thread_ref_).write(core::cell::UnsafeCell::new(
            crate::base::threading::platform_thread_ref::PlatformThreadRef::new(),
        ));
        #[cfg(base_dcheck_is_on)]
        core::ptr::addr_of_mut!((*p).in_tracked_locks_held_by_current_thread_)
            .write(core::cell::UnsafeCell::new(false));
        LockImpl::init_at(core::ptr::addr_of_mut!((*p).lock_));
        Pin::new_unchecked(&mut *p)
    }
    pub unsafe fn Acquire(self: Pin<&Self>) {
        self.AcquireWithTracking(LockTracking::kDisabled);
    }
    pub unsafe fn AcquireWithTracking(self: Pin<&Self>, tracking: LockTracking) {
        self.AcquireWithTags(Self::GetBaseLockMetricTagList(), tracking);
    }
    pub unsafe fn AcquireWithTags(
        self: Pin<&Self>,
        tags: &LockMetricTagList,
        tracking: LockTracking,
    ) {
        self.lock_.Lock(tags);
        #[cfg(base_dcheck_is_on)]
        {
            if tracking == LockTracking::kEnabled {
                self.AddToLocksHeldOnCurrentThread();
            }
            self.CheckUnheldAndMark();
        }
    }
    // Safety: caller holds this exact lock on this thread, with no still-live protected-data lease.
    pub unsafe fn Release(self: Pin<&Self>) {
        #[cfg(base_dcheck_is_on)]
        {
            self.CheckHeldAndUnmark();
            if *self.in_tracked_locks_held_by_current_thread_.get() {
                self.RemoveFromLocksHeldOnCurrentThread();
            }
        }
        self.lock_.Unlock();
    }
    pub unsafe fn Try(self: Pin<&Self>, tracking: LockTracking) -> bool {
        let acquired = self.lock_.Try();
        #[cfg(base_dcheck_is_on)]
        if acquired {
            if tracking == LockTracking::kEnabled {
                self.AddToLocksHeldOnCurrentThread();
            }
            self.CheckUnheldAndMark();
        }
        acquired
    }
    #[cfg(not(base_dcheck_is_on))]
    pub fn AssertAcquired(self: Pin<&Self>) {}
    #[cfg(not(base_dcheck_is_on))]
    pub fn AssertNotHeld(self: Pin<&Self>) {}
    pub fn HandlesMultipleThreadPriorities() -> bool {
        #[cfg(base_is_win)]
        {
            true
        }
        #[cfg(any(base_is_posix, base_is_fuchsia))]
        {
            LockImpl::PriorityInheritanceAvailable()
        }
    }
}
unsafe impl LockType for Lock {
    unsafe fn Acquire(self: Pin<&Self>, tracking: LockTracking) {
        self.AcquireWithTracking(tracking);
    }
    unsafe fn Release(self: Pin<&Self>) {
        self.Release();
    }
    unsafe fn Try(self: Pin<&Self>, tracking: LockTracking) -> bool {
        self.Try(tracking)
    }
    unsafe fn AssertAcquired(self: Pin<&Self>) {
        self.AssertAcquired();
    }
}
pub type AutoLock<'a> = super::lock_impl::BasicAutoLock<'a, Lock>;
pub type MovableAutoLock<'a> = super::lock_impl::BasicMovableAutoLock<'a, Lock>;
pub type AutoTryLock<'a> = super::lock_impl::BasicAutoTryLock<'a, Lock>;
pub type AutoUnlock<'a> = super::lock_impl::BasicAutoUnlock<'a, Lock>;
pub type AutoLockMaybe<'a> = super::lock_impl::BasicAutoLockMaybe<'a, Lock>;
pub type ReleasableAutoLock<'a> = super::lock_impl::BasicReleasableAutoLock<'a, Lock>;
// Debug owner checks/tracking and InitializeFeatures are exact pending native CheckError/feature dependencies.
#[cfg(base_dcheck_is_on)]
compile_error!("base Lock debug owner/tracking streamed CHECK/ DCHECK producer pending; no diagnostic substitution");
