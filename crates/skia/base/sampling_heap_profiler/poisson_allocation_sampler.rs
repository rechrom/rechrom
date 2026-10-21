use super::lock_free_address_hash_set::{ContainsResult, LockFreeAddressHashSet};
use crate::base::allocator::dispatcher::{
    notification_data::{AllocationNotificationData, FreeNotificationData},
    reentry_guard::ReentryGuard,
    subsystem::AllocationSubsystem,
};
use crate::base::synchronization::lock::{AutoLock, Lock};
use core::{
    cell::UnsafeCell,
    ffi::{c_char, c_int, c_void},
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
    pin::Pin,
    ptr::{self, NonNull},
    sync::atomic::{AtomicI32, AtomicPtr, AtomicUsize, Ordering},
};
// Source: poisson_allocation_sampler.h: SamplesObserver is the actual pure-virtual interface.
// The native contract requires thread-safe observers and permits calls after removal.
// This is a nonowning observer: no Arc, retention, allocation or second count.
pub trait SamplesObserver: Sync {
    unsafe fn SampleAdded(
        &self,
        address: *mut c_void,
        size: usize,
        total: usize,
        type_: AllocationSubsystem,
        context: *const c_char,
    );
    unsafe fn SampleRemoved(&self, address: *mut c_void);
}
type ObserverRef = NonNull<dyn SamplesObserver>;
const kAccumulatedBytesOffset: isize = 1 << 29;
static mut g_deterministic: bool = false;
static g_sampled_addresses_set: AtomicPtr<LockFreeAddressHashSet> = AtomicPtr::new(ptr::null_mut());
static g_sampling_interval: AtomicUsize =
    AtomicUsize::new(PoissonAllocationSampler::kDefaultSamplingIntervalBytes);
#[derive(Clone, Copy)]
pub struct ThreadLocalData {
    accumulated_bytes: isize,
    accumulated_bytes_snapshot: isize,
    internal_reentry_guard: bool,
    sampling_interval_initialized: bool,
}
impl ThreadLocalData {
    const fn new() -> Self {
        Self {
            accumulated_bytes: 0,
            accumulated_bytes_snapshot: 0,
            internal_reentry_guard: false,
            sampling_interval_initialized: false,
        }
    }
}
impl Default for ThreadLocalData {
    fn default() -> Self {
        Self::new()
    }
}
// Source: poisson_allocation_sampler.cc: GetThreadLocalData, exact USE_LOCAL_TLS_EMULATION = IS_POSIX.
unsafe fn GetThreadLocalData() -> *mut ThreadLocalData {
    #[cfg(base_is_posix)]
    {
        use crate::base::allocator::dispatcher::tls::ThreadLocalStorage;
        static INITIALIZED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        static mut DATA: MaybeUninit<ThreadLocalStorage<ThreadLocalData>> = MaybeUninit::uninit();
        INITIALIZED.get_or_init(|| unsafe {
            ThreadLocalStorage::init_at(
                core::ptr::addr_of_mut!(DATA).cast(),
                "poisson_allocation_sampler",
            );
        });
        return Pin::new_unchecked(
            &*core::ptr::addr_of!(DATA).cast::<ThreadLocalStorage<ThreadLocalData>>(),
        )
        .GetThreadLocalData();
    }
    #[cfg(not(base_is_posix))]
    {
        #[thread_local]
        static mut DATA: ThreadLocalData = ThreadLocalData::new();
        core::ptr::addr_of_mut!(DATA)
    }
}
// Source: h ScopedMuteThreadSamples; cpp ctor/dtor/IsMuted. Thread affinity is a zero-sized restriction.
pub struct ScopedMuteThreadSamples {
    was_muted_: bool,
    _thread: PhantomData<*mut ()>,
}
impl ScopedMuteThreadSamples {
    pub unsafe fn new() -> Self {
        let data = GetThreadLocalData();
        let was_muted_ = (*data).internal_reentry_guard;
        (*data).internal_reentry_guard = true;
        if !was_muted_ {
            (*data).accumulated_bytes_snapshot = (*data).accumulated_bytes;
            (*data).accumulated_bytes = (*data)
                .accumulated_bytes
                .wrapping_sub(kAccumulatedBytesOffset);
        }
        Self {
            was_muted_,
            _thread: PhantomData,
        }
    }
    pub unsafe fn IsMuted() -> bool {
        (*GetThreadLocalData()).internal_reentry_guard
    }
}
impl Drop for ScopedMuteThreadSamples {
    fn drop(&mut self) {
        unsafe {
            let data = GetThreadLocalData();
            crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),122);@condition b"thread_local_data->internal_reentry_guard\0".as_ptr();(*data).internal_reentry_guard);
            (*data).internal_reentry_guard = self.was_muted_;
            if !self.was_muted_ {
                (*data).accumulated_bytes = (*data).accumulated_bytes_snapshot;
            }
        }
    }
}
pub struct ScopedSuppressRandomnessForTesting {
    _thread: PhantomData<*mut ()>,
}
impl ScopedSuppressRandomnessForTesting {
    // Safety: native nonnested global test switch is externally synchronized with sampler reads.
    pub unsafe fn new() -> Self {
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),138);@condition b"!g_deterministic\0".as_ptr();!g_deterministic);
        g_deterministic = true;
        (*GetThreadLocalData()).accumulated_bytes = 0;
        Self {
            _thread: PhantomData,
        }
    }
    pub unsafe fn IsSuppressed() -> bool {
        g_deterministic
    }
}
impl Drop for ScopedSuppressRandomnessForTesting {
    fn drop(&mut self) {
        unsafe {
            crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),149);@condition b"g_deterministic\0".as_ptr();g_deterministic);
            g_deterministic = false;
        }
    }
}
pub struct ScopedMuteHookedSamplesForTesting {
    accumulated_bytes_snapshot_: isize,
    _thread: PhantomData<*mut ()>,
}
impl ScopedMuteHookedSamplesForTesting {
    pub unsafe fn new() -> Self {
        PoissonAllocationSampler::SetProfilingStateFlag(
            ProfilingStateFlag::kHookedSamplesMutedForTesting,
        );
        let data = GetThreadLocalData();
        let snapshot = (*data).accumulated_bytes;
        (*data).accumulated_bytes = 0;
        Self {
            accumulated_bytes_snapshot_: snapshot,
            _thread: PhantomData,
        }
    }
    // Native explicitly-defaulted move copies this scalar and leaves the source destructible.
    // It DOES NOT disarm either scope; no invented active flag fixes the native behavior.
    pub fn move_from(other: &mut Self) -> Self {
        Self {
            accumulated_bytes_snapshot_: other.accumulated_bytes_snapshot_,
            _thread: PhantomData,
        }
    }
    pub fn move_assign(&mut self, other: &mut Self) {
        self.accumulated_bytes_snapshot_ = other.accumulated_bytes_snapshot_;
    }
}
impl Drop for ScopedMuteHookedSamplesForTesting {
    fn drop(&mut self) {
        unsafe {
            (*GetThreadLocalData()).accumulated_bytes = self.accumulated_bytes_snapshot_;
            PoissonAllocationSampler::ResetProfilingStateFlag(
                ProfilingStateFlag::kHookedSamplesMutedForTesting,
            );
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum ProfilingStateFlag {
    kWasStarted = 1 << 0,
    kIsRunning = 1 << 1,
    kHookedSamplesMutedForTesting = 1 << 2,
}
type ProfilingStateFlagMask = c_int;
static profiling_state_: AtomicI32 = AtomicI32::new(0);
// Source: native private mutex/observer vector/load factor. No destructor is exposed: native dtor is deleted.
// The only supported instance is constructed at final inline immortal NoDestructor storage.
pub struct PoissonAllocationSampler {
    address_cache_target_load_factor_: UnsafeCell<f32>,
    observers_: UnsafeCell<Vec<ObserverRef>>,
    mutex_: Lock,
    _pin: PhantomPinned,
}
// Actual shared mutations are only native atomics, thread-local state or under mutex_.
// Observers themselves must provide their real Sync implementation and outlive all in-flight notifications.
unsafe impl Sync for PoissonAllocationSampler {}
impl PoissonAllocationSampler {
    pub const kDefaultSamplingIntervalBytes: usize = 128 * 1024;
    pub type SamplesObserver =
        dyn crate::base::sampling_heap_profiler::poisson_allocation_sampler::SamplesObserver;
    pub type ScopedMuteThreadSamples =
        crate::base::sampling_heap_profiler::poisson_allocation_sampler::ScopedMuteThreadSamples;
    pub type ScopedSuppressRandomnessForTesting=crate::base::sampling_heap_profiler::poisson_allocation_sampler::ScopedSuppressRandomnessForTesting;
    pub type ScopedMuteHookedSamplesForTesting=crate::base::sampling_heap_profiler::poisson_allocation_sampler::ScopedMuteHookedSamplesForTesting;
    unsafe fn mutex(&self) -> Pin<&Lock> {
        Pin::new_unchecked(&self.mutex_)
    }
    // Actual default member construction occurs before the native constructor body.
    unsafe fn init_at(out: *mut Self) {
        Lock::new(Pin::new_unchecked(
            &mut *core::ptr::addr_of_mut!((*out).mutex_).cast::<MaybeUninit<Lock>>(),
        ));
        core::ptr::addr_of_mut!((*out).observers_).write(UnsafeCell::new(Vec::new()));
        core::ptr::addr_of_mut!((*out).address_cache_target_load_factor_)
            .write(UnsafeCell::new(1.0));
        core::ptr::addr_of_mut!((*out)._pin).write(PhantomPinned);
        Self::Init();
        let allocation = Box::<LockFreeAddressHashSet>::new_uninit();
        let set = Box::into_raw(allocation).cast::<LockFreeAddressHashSet>();
        LockFreeAddressHashSet::init_at(set, 64, (*out).mutex());
        g_sampled_addresses_set.store(set, Ordering::Release);
    }
    pub unsafe fn Init() {
        static INIT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let _ = INIT.get_or_init(|| unsafe {
            GetThreadLocalData();
            ReentryGuard::InitTLSSlot();
            true
        });
    }
    pub fn SetSamplingInterval(&self, bytes: usize) {
        g_sampling_interval.store(bytes, Ordering::Relaxed);
    }
    pub fn SamplingInterval(&self) -> usize {
        g_sampling_interval.load(Ordering::Relaxed)
    }
    pub unsafe fn SetTargetHashSetLoadFactor(&self, factor: Option<f32>) {
        let _lock = AutoLock::new(self.mutex());
        *self.address_cache_target_load_factor_.get() = factor.unwrap_or(1.0);
    }
    unsafe fn GetNextSampleInterval(interval: usize) -> usize {
        if core::intrinsics::unlikely(g_deterministic) {
            return interval;
        }
        let uniform = crate::base::rand_util_posix::RandDoubleAvoidAllocation();
        let value = -log(1.0 - uniform) * (interval as f64);
        let min_value = core::mem::size_of::<isize>();
        let max_value = interval.wrapping_mul(20);
        if core::intrinsics::unlikely(value < (min_value as f64)) {
            return min_value;
        }
        if core::intrinsics::unlikely(value > (max_value as f64)) {
            return max_value;
        }
        value as usize
    }
    // Safety: initialized sampler/lock/hash set; valid sample intervals and original allocation-hook lifetime.
    #[inline(always)]
    pub unsafe fn OnAllocation(&self, data: &AllocationNotificationData) {
        let state = profiling_state_.load(Ordering::Relaxed);
        if core::intrinsics::likely(state & (ProfilingStateFlag::kWasStarted as i32) == 0) {
            return;
        }
        let type_ = data.allocation_subsystem();
        if core::intrinsics::unlikely(
            state & (ProfilingStateFlag::kHookedSamplesMutedForTesting as i32) != 0
                && type_ != AllocationSubsystem::kManualForTesting,
        ) {
            return;
        }
        let guard = ReentryGuard::new();
        if core::intrinsics::unlikely(!guard.is_allowed()) {
            return;
        }
        self.DoRecordAllocation(state, data.address(), data.size(), type_, data.type_name());
    }
    #[inline(always)]
    pub unsafe fn OnFree(&self, data: &FreeNotificationData) {
        let state = profiling_state_.load(Ordering::Relaxed);
        if core::intrinsics::likely(state & (ProfilingStateFlag::kWasStarted as i32) == 0) {
            return;
        }
        let address = data.address();
        if core::intrinsics::unlikely(address.is_null()) {
            return;
        }
        let cache = Self::sampled_addresses_set();
        match (*cache).Contains(address) {
            ContainsResult::kNotFound | ContainsResult::kNotFoundButMatchedInBloomFilter => return,
            ContainsResult::kFound => {}
        }
        if core::intrinsics::unlikely(ScopedMuteThreadSamples::IsMuted()) {
            return;
        }
        self.DoRecordFree(address);
    }
    unsafe fn DoRecordAllocation(
        &self,
        state: ProfilingStateFlagMask,
        address: *mut c_void,
        size: usize,
        type_: AllocationSubsystem,
        context: *const c_char,
    ) {
        let data = GetThreadLocalData();
        (*data).accumulated_bytes = (*data).accumulated_bytes.wrapping_add(size as isize);
        let mut accumulated = (*data).accumulated_bytes;
        if core::intrinsics::likely(accumulated < 0) {
            return;
        }
        if core::intrinsics::unlikely(state & (ProfilingStateFlag::kIsRunning as i32) == 0) {
            (*data).sampling_interval_initialized = false;
            (*data).accumulated_bytes = 0;
            return;
        }
        if core::intrinsics::unlikely(address.is_null()) {
            return;
        }
        let mean = g_sampling_interval.load(Ordering::Relaxed);
        if core::intrinsics::unlikely(!(*data).sampling_interval_initialized) {
            (*data).sampling_interval_initialized = true;
            accumulated = accumulated.wrapping_sub(Self::GetNextSampleInterval(mean) as isize);
            if accumulated < 0 {
                (*data).accumulated_bytes = accumulated;
                return;
            }
        }
        let mut samples = (accumulated as usize) / mean;
        accumulated = ((accumulated as usize) % mean) as isize;
        loop {
            accumulated = accumulated.wrapping_sub(Self::GetNextSampleInterval(mean) as isize);
            samples = samples.wrapping_add(1);
            if accumulated < 0 {
                break;
            }
        }
        (*data).accumulated_bytes = accumulated;
        if core::intrinsics::unlikely(ScopedMuteThreadSamples::IsMuted()) {
            return;
        }
        let _no_reentrancy = ScopedMuteThreadSamples::new();
        let mut observers_copy = Vec::new();
        {
            let _lock = AutoLock::new(self.mutex());
            let cache = Self::sampled_addresses_set();
            if (*cache).Contains(address) == ContainsResult::kFound {
                return;
            }
            (*cache).Insert(address);
            self.BalanceAddressesHashSet();
            observers_copy = (*self.observers_.get()).clone();
        }
        let total = mean.wrapping_mul(samples);
        for observer in observers_copy {
            observer
                .as_ref()
                .SampleAdded(address, size, total, type_, context);
        }
    }
    unsafe fn DoRecordFree(&self, address: *mut c_void) {
        let _no_reentrancy = ScopedMuteThreadSamples::new();
        let mut observers_copy = Vec::new();
        {
            let _lock = AutoLock::new(self.mutex());
            observers_copy = (*self.observers_.get()).clone();
            (*Self::sampled_addresses_set()).Remove(address);
        }
        for observer in observers_copy {
            observer.as_ref().SampleRemoved(address);
        }
    }
    // Safety: mutex_ is held. Old published sets intentionally remain allocated for concurrent readers.
    unsafe fn BalanceAddressesHashSet(&self) {
        let current = Self::sampled_addresses_set();
        if (*current).load_factor() < *self.address_cache_target_load_factor_.get() {
            return;
        }
        let mut allocation = Box::<LockFreeAddressHashSet>::new_uninit();
        LockFreeAddressHashSet::init_at(
            allocation.as_mut_ptr(),
            (*current).buckets_count().wrapping_mul(2),
            self.mutex(),
        );
        let owner = allocation.assume_init();
        owner.Copy(&*current);
        g_sampled_addresses_set.store(Box::into_raw(owner), Ordering::Release);
    }
    unsafe fn sampled_addresses_set() -> *mut LockFreeAddressHashSet {
        g_sampled_addresses_set.load(Ordering::Acquire)
    }
    pub unsafe fn Get() -> *mut Self {
        static READY: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        static mut INSTANCE: MaybeUninit<PoissonAllocationSampler> = MaybeUninit::uninit();
        READY.get_or_init(|| unsafe {
            Self::init_at(core::ptr::addr_of_mut!(INSTANCE).cast());
        });
        core::ptr::addr_of_mut!(INSTANCE).cast()
    }
    pub fn AreHookedSamplesMuted() -> bool {
        profiling_state_.load(Ordering::Relaxed)
            & (ProfilingStateFlag::kHookedSamplesMutedForTesting as i32)
            != 0
    }
    pub unsafe fn GetAccumulatedBytesForTesting() -> isize {
        (*GetThreadLocalData()).accumulated_bytes
    }
    unsafe fn SetProfilingStateFlag(flag: ProfilingStateFlag) {
        let mut flags = flag as i32;
        if flag == ProfilingStateFlag::kIsRunning {
            flags |= ProfilingStateFlag::kWasStarted as i32;
        }
        let old = profiling_state_.fetch_or(flags, Ordering::Relaxed);
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),404);@condition b"!(old_state & flag)\0".as_ptr();old&(flag as i32)==0);
    }
    unsafe fn ResetProfilingStateFlag(flag: ProfilingStateFlag) {
        crate::DCHECK_NE!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),410);flag,ProfilingStateFlag::kWasStarted);
        let old = profiling_state_.fetch_and(!(flag as i32), Ordering::Relaxed);
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),413);@condition b"old_state & flag\0".as_ptr();old&(flag as i32)!=0);
    }
    // Safety: observer has real thread-safe lifetime through every in-flight callback, including after removal.
    // NonNull is the defined notification domain; nullable/borrowed-field observer domains are not widened unsafely.
    pub unsafe fn AddSamplesObserver(&self, observer: ObserverRef) {
        let _guard = ReentryGuard::new();
        let _no_reentrancy = ScopedMuteThreadSamples::new();
        let _lock = AutoLock::new(self.mutex());
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),424);@condition b"std::ranges::find(observers_, observer) == observers_.end()\0".as_ptr();!(*self.observers_.get()).iter().any(|p|core::ptr::addr_eq(p.as_ptr(),observer.as_ptr())));
        let was_stopped = (*self.observers_.get()).is_empty();
        (*self.observers_.get()).push(observer);
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),430);@condition b"g_sampled_addresses_set.load(std::memory_order_relaxed)\0".as_ptr();!g_sampled_addresses_set.load(Ordering::Relaxed).is_null());
        if was_stopped {
            Self::SetProfilingStateFlag(ProfilingStateFlag::kIsRunning);
        }
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),438);@condition b"profiling_state_.load(std::memory_order_relaxed) & ProfilingStateFlag::kIsRunning\0".as_ptr();profiling_state_.load(Ordering::Relaxed)&(ProfilingStateFlag::kIsRunning as i32)!=0);
    }
    pub unsafe fn RemoveSamplesObserver(&self, observer: ObserverRef) {
        let _guard = ReentryGuard::new();
        let _no_reentrancy = ScopedMuteThreadSamples::new();
        let _lock = AutoLock::new(self.mutex());
        let it = (*self.observers_.get())
            .iter()
            .position(|p| core::ptr::addr_eq(p.as_ptr(), observer.as_ptr()));
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),452);@condition b"it != observers_.end()\0".as_ptr();it.is_some());
        // The native CHECK precedes erase; no unchecked unwrap manufactures a valid iterator.
        let index = it.expect("native CHECK guarantees a registered observer");
        (*self.observers_.get()).remove(index);
        crate::DCHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/sampling_heap_profiler/poisson_allocation_sampler.cc\0".as_ptr().cast(),438);@condition b"profiling_state_.load(std::memory_order_relaxed) & ProfilingStateFlag::kIsRunning\0".as_ptr();profiling_state_.load(Ordering::Relaxed)&(ProfilingStateFlag::kIsRunning as i32)!=0);
        if (*self.observers_.get()).is_empty() {
            Self::ResetProfilingStateFlag(ProfilingStateFlag::kIsRunning);
        }
    }
}
unsafe extern "C" {
    fn log(value: f64) -> f64;
}
