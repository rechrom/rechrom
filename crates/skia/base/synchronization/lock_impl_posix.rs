use super::{lock_impl::LockImpl, lock_metrics_recorder_tags::LockMetricTagList};
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicI32, Ordering},
};
// Source: base/synchronization/lock_impl_posix.cc:47-50
#[cfg(base_is_posix)]
static g_spin_count: AtomicI32 = AtomicI32::new(0);
// Native streamed DCHECK diagnostics require base/check's CheckError/ostream, not a Skia assert.
#[cfg(base_dcheck_is_on)]
compile_error!("LockImpl debug streamed DCHECK producer is pending: base/check CheckError/ostream + safe_strerror");
#[cfg(any(base_is_posix, base_is_fuchsia))]
impl LockImpl {
    // Source: base/synchronization/lock_impl_posix.cc:107-129
    // Caller provides final-address, exclusive uninitialized storage and must not move it thereafter.
    pub(crate) unsafe fn init_at(out: *mut Self) {
        out.write(Self {
            native_handle_: core::cell::UnsafeCell::new(MaybeUninit::uninit()),
            _pin: core::marker::PhantomPinned,
        });
        let mut mta = MaybeUninit::<libc::pthread_mutexattr_t>::uninit();
        let rv = libc::pthread_mutexattr_init(mta.as_mut_ptr());
        #[cfg(base_dcheck_is_on)]
        dcheck_system_result(rv);
        #[cfg(not(base_is_fuchsia))]
        if Self::PriorityInheritanceAvailable() {
            let rv =
                libc::pthread_mutexattr_setprotocol(mta.as_mut_ptr(), libc::PTHREAD_PRIO_INHERIT);
            #[cfg(base_dcheck_is_on)]
            dcheck_system_result(rv);
        }
        #[cfg(not(base_native_ndebug))]
        {
            let rv =
                libc::pthread_mutexattr_settype(mta.as_mut_ptr(), libc::PTHREAD_MUTEX_ERRORCHECK);
            #[cfg(base_dcheck_is_on)]
            dcheck_system_result(rv);
        }
        let rv = libc::pthread_mutex_init((*out).native_handle(), mta.as_ptr());
        #[cfg(base_dcheck_is_on)]
        dcheck_system_result(rv);
        let rv = libc::pthread_mutexattr_destroy(mta.as_mut_ptr());
        #[cfg(base_dcheck_is_on)]
        dcheck_system_result(rv);
    }
    // Source: base/synchronization/lock_impl_posix.cc:136-146
    pub(crate) unsafe fn LockInternal(&self, tags: &LockMetricTagList) {
        #[cfg(base_is_posix)]
        if self.TrySpin() {
            return;
        }
        let _timer = super::lock_metrics_recorder::ScopedLockAcquisitionTimer::new(tags);
        let rv = libc::pthread_mutex_lock(self.native_handle());
        #[cfg(base_dcheck_is_on)]
        dcheck_system_result(rv);
    }
    // Source: base/synchronization/lock_impl_posix.cc:149-175
    #[cfg(base_is_posix)]
    unsafe fn TrySpin(&self) -> bool {
        let mut tries = 0;
        let mut backoff = 1;
        let spin_count = g_spin_count.load(Ordering::Relaxed);
        while tries < spin_count {
            if self.Try() {
                return true;
            }
            for _ in 0..backoff {
                super::lock_subtle::YieldProcessor();
            }
            tries += backoff;
            backoff = 16.min(backoff << 1);
        }
        false
    }
    #[cfg(base_is_posix)]
    pub fn SetTrySpinCount(spin_count: i32) {
        g_spin_count.store(spin_count, Ordering::Relaxed);
    }
    // Source: base/synchronization/lock_impl_posix.cc:179-202
    pub fn PriorityInheritanceAvailable() -> bool {
        #[cfg(all(base_enable_mutex_priority_inheritance, base_is_android))]
        {
            compile_error!("actual Android weak pthread_mutexattr_setprotocol and BackgroundThreadPoolFieldTrial producer pending");
        }
        #[cfg(all(
            not(all(base_enable_mutex_priority_inheritance, base_is_android)),
            not(base_is_fuchsia),
            base_is_apple
        ))]
        {
            return true;
        }
        #[cfg(all(
            not(all(base_enable_mutex_priority_inheritance, base_is_android)),
            not(all(not(base_is_fuchsia), base_is_apple))
        ))]
        {
            return false;
        }
    }
}
#[cfg(any(base_is_posix, base_is_fuchsia))]
impl Drop for LockImpl {
    // Source: base/synchronization/lock_impl_posix.cc:131-134
    fn drop(&mut self) {
        let rv = unsafe { libc::pthread_mutex_destroy(self.native_handle()) };
        #[cfg(base_dcheck_is_on)]
        dcheck_system_result(rv);
    }
}
// Source: base/synchronization/lock_impl_posix.cc:206-229
pub fn KernelSupportsPriorityInheritanceFutex() -> bool {
    #[cfg(base_is_android)]
    {
        compile_error!("actual SysInfo::KernelVersionNumber::Current producer pending; native strict version comparisons retained in receipt");
    }
    #[cfg(not(base_is_android))]
    {
        false
    }
}
