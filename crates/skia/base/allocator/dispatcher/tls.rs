// Source: base/allocator/dispatcher/tls.h
// POSIX-only native template. Default capacity uses the actual Rust-internal
// representation; no C++ ABI identity is claimed for these private chunks.
#![cfg(unix)]
use core::{
    cell::UnsafeCell,
    ffi::c_void,
    marker::PhantomPinned,
    mem::MaybeUninit,
    pin::Pin,
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};
// Source: tls.h:72-97. Ordinary methods are declared in C++ but defined only in tls.cc.
pub struct MMapAllocator;
impl MMapAllocator {
    #[cfg(pa_page_allocator_constants_are_constexpr)]
    pub const AllocationChunkSize: usize =
        crate::partition_alloc::partition_alloc_constants::PartitionPageSize();
    #[cfg(all(
        not(pa_page_allocator_constants_are_constexpr),
        any(
            target_vendor = "apple",
            all(target_os = "android", target_pointer_width = "64"),
            all(target_os = "linux", target_arch = "aarch64")
        )
    ))]
    pub const AllocationChunkSize: usize = 16384;
    #[cfg(all(
        not(pa_page_allocator_constants_are_constexpr),
        not(any(
            target_vendor = "apple",
            all(target_os = "android", target_pointer_width = "64"),
            all(target_os = "linux", target_arch = "aarch64")
        ))
    ))]
    pub const AllocationChunkSize: usize = 4096;
}
pub type DefaultAllocator = MMapAllocator;
pub type OnThreadTerminationFunction = unsafe extern "C" fn(*mut c_void);
// Source: tls.h:106-153. No invented constructor, move or OS method bodies.
// Source: tls.h:106-153, tls.cc:78-189.
// Internal unique_ptr<string> maps an owned string object and its NUL-terminated
// byte buffer. No C++ std::string ABI is exposed across FFI.
pub struct PThreadTLSSystem {
    crash_key_name_: UnsafeCell<Option<Box<Vec<u8>>>>,
    crash_key_: UnsafeCell<*mut crate::base::debug::crash_logging::CrashKeyString>,
    data_access_key_: UnsafeCell<libc::pthread_key_t>,
    #[cfg(base_dcheck_is_on)]
    initialized_: AtomicBool,
}
pub type DefaultTLSSystem = PThreadTLSSystem;
impl PThreadTLSSystem {
    pub fn new() -> Self {
        Self {
            crash_key_name_: UnsafeCell::new(None),
            crash_key_: UnsafeCell::new(ptr::null_mut()),
            data_access_key_: UnsafeCell::new(0),
            #[cfg(base_dcheck_is_on)]
            initialized_: AtomicBool::new(false),
        }
    }
    pub fn move_from(other: &mut Self) -> Self {
        let mut out = Self::new();
        out.move_assign(other);
        out
    }
    pub fn move_assign(&mut self, other: &mut Self) -> &mut Self {
        core::mem::swap(
            self.crash_key_name_.get_mut(),
            other.crash_key_name_.get_mut(),
        );
        core::mem::swap(self.crash_key_.get_mut(), other.crash_key_.get_mut());
        core::mem::swap(
            self.data_access_key_.get_mut(),
            other.data_access_key_.get_mut(),
        );
        #[cfg(base_dcheck_is_on)]
        {
            let l = self.initialized_.load(Ordering::Relaxed);
            let r = other.initialized_.load(Ordering::Relaxed);
            crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"lh_op.compare_exchange_strong(lh_op_value, rh_op_value)\0".as_ptr().cast();self.initialized_.compare_exchange(l,r,Ordering::SeqCst,Ordering::SeqCst).is_ok());
            crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"rh_op.compare_exchange_strong(rh_op_value, lh_op_value)\0".as_ptr().cast();other.initialized_.compare_exchange(r,l,Ordering::SeqCst,Ordering::SeqCst).is_ok());
        }
        self
    }
}
// Native default destructor does not delete the pthread key. Only explicit test
// teardown does; dropping the string owner retains the same native distinction.
unsafe impl TLSSystemInterface for PThreadTLSSystem {
    unsafe fn Setup(&self, on_exit: OnThreadTerminationFunction, id: &str) -> bool {
        #[cfg(base_dcheck_is_on)]
        crate::DCHECK!(@location crate::base::location::Location::Current(); @condition b"!initialized_.exchange(true, std::memory_order_acq_rel)\0".as_ptr().cast();!self.initialized_.swap(true,Ordering::AcqRel));
        let result = libc::pthread_key_create(self.data_access_key_.get(), Some(on_exit));
        // Original make_unique precedes append and crash-key publication. String remains
        // owned until explicit teardown because crashpad retains the name pointer.
        *self.crash_key_name_.get() = Some(Box::new(b"tls_system-".to_vec()));
        let name = (*self.crash_key_name_.get()).as_mut().unwrap();
        name.extend_from_slice(id.as_bytes());
        let len = name.len();
        name.push(0);
        use crate::base::debug::crash_logging::{
            AllocateCrashKeyString, CrashKeySize, SetCrashKeyString,
        };
        let size = if len <= 32 {
            CrashKeySize::Size32
        } else if len <= 64 {
            CrashKeySize::Size64
        } else if len <= 256 {
            CrashKeySize::Size256
        } else {
            crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"std::size(crash_key_name) <= 1024ul\0".as_ptr().cast();len<=1024);
            CrashKeySize::Size1024
        };
        *self.crash_key_.get() = AllocateCrashKeyString(name.as_ptr().cast(), size);
        SetCrashKeyString(
            *self.crash_key_.get(),
            crate::base::strings::string_number_conversions::NumberToString(
                *self.data_access_key_.get(),
            ),
        );
        result == 0
    }
    unsafe fn TearDownForTesting(&self) -> bool {
        #[cfg(base_dcheck_is_on)]
        crate::DCHECK!(@location crate::base::location::Location::Current(); @condition b"initialized_.exchange(false, std::memory_order_acq_rel)\0".as_ptr().cast();self.initialized_.swap(false,Ordering::AcqRel));
        crate::base::debug::crash_logging::ClearCrashKeyString(*self.crash_key_.get());
        *self.crash_key_.get() = ptr::null_mut();
        *self.crash_key_name_.get() = None;
        libc::pthread_key_delete(*self.data_access_key_.get()) == 0
    }
    unsafe fn GetThreadSpecificData(&self) -> *mut c_void {
        #[cfg(base_dcheck_is_on)]
        if !self.initialized_.load(Ordering::Acquire) {
            return ptr::null_mut();
        }
        libc::pthread_getspecific(*self.data_access_key_.get())
    }
    unsafe fn SetThreadSpecificData(&self, data: *mut c_void) -> bool {
        #[cfg(base_dcheck_is_on)]
        if !self.initialized_.load(Ordering::Acquire) {
            return false;
        }
        libc::pthread_setspecific(*self.data_access_key_.get(), data) == 0
    }
}
unsafe impl AllocatorInterface for MMapAllocator {
    unsafe fn AllocateMemory(&self, size: usize) -> *mut c_void {
        let result = libc::mmap(
            ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        #[cfg(all(
            any(target_os = "android", target_os = "linux"),
            native_pr_set_vma_defined,
            native_pr_set_vma_anon_name_defined
        ))]
        if result != libc::MAP_FAILED {
            libc::prctl(
                libc::PR_SET_VMA,
                libc::PR_SET_VMA_ANON_NAME,
                result,
                size,
                b"tls-mmap-allocator\0".as_ptr(),
            );
        }
        if result != libc::MAP_FAILED {
            result
        } else {
            ptr::null_mut()
        }
    }
    unsafe fn FreeMemoryForTesting(&self, p: *mut c_void, size: usize) -> bool {
        libc::munmap(p, size) == 0
    }
}

// These traits express the *native template parameter requirements*, not replacement
// implementations of the ordinary MMapAllocator/PThreadTLSSystem methods above.
pub unsafe trait AllocatorInterface {
    unsafe fn AllocateMemory(&self, size: usize) -> *mut c_void;
    unsafe fn FreeMemoryForTesting(&self, p: *mut c_void, size: usize) -> bool;
}
pub unsafe trait TLSSystemInterface {
    unsafe fn Setup(&self, on_exit: OnThreadTerminationFunction, id: &str) -> bool;
    unsafe fn TearDownForTesting(&self) -> bool;
    unsafe fn GetThreadSpecificData(&self) -> *mut c_void;
    unsafe fn SetThreadSpecificData(&self, p: *mut c_void) -> bool;
}
// Actual dereference overloads use references, without erasing their lifetime.
pub fn dereference<T>(value: &T) -> &T {
    value
}
pub fn dereference_wrapper<'a, T>(value: &'a &'a T) -> &'a T {
    *value
}
#[repr(C)]
struct SingleSlot<T> {
    item: UnsafeCell<T>,
    is_used: AtomicBool,
}
#[repr(C)]
struct Chunk<T, const N: usize> {
    slots: [SingleSlot<T>; N],
    next_chunk: AtomicPtr<Chunk<T, N>>,
    create_next_chunk_flag: std::sync::Once,
}
// Source: tls.h:298-352; sizeof callback is the actual selected representation's sizeof.
// Native layout metadata must supply this callback for native capacity equivalence.
pub fn CalculateEffectiveNumberOfItemsBinSearch(
    mut lower: usize,
    mut upper: usize,
    bytes: usize,
    size_of_chunk: impl Fn(usize) -> usize,
) -> usize {
    loop {
        if lower == upper {
            return lower;
        }
        let current = (upper - lower) / 2 + lower;
        if size_of_chunk(current) > bytes {
            upper = current;
            continue;
        }
        if size_of_chunk(current + 1) < bytes {
            lower = current + 1;
            continue;
        }
        return current;
    }
}
pub fn CalculateEffectiveNumberOfItems(
    minimum: usize,
    bytes: usize,
    payload_size: usize,
    size_of_chunk: impl Fn(usize) -> usize,
) -> usize {
    if size_of_chunk(minimum) < bytes {
        CalculateEffectiveNumberOfItemsBinSearch(
            minimum,
            bytes / payload_size + 1,
            bytes,
            size_of_chunk,
        )
    } else {
        minimum
    }
}
// Copy+Default is the actual trivially assignable Poisson ThreadLocalData domain.
// Nontrivial assignment/exception cleanup and full generic destructor domain remain pending.
pub struct Storage<
    T: Copy + Default,
    A: AllocatorInterface,
    S: TLSSystemInterface,
    const N: usize,
    const TEST: bool = false,
> {
    allocator_: A,
    tls_system_: S,
    root_: AtomicPtr<Chunk<T, N>>,
    _pin: PhantomPinned,
}
impl<
        T: Copy + Default,
        A: AllocatorInterface,
        S: TLSSystemInterface,
        const N: usize,
        const TEST: bool,
    > Storage<T, A, S, N, TEST>
{
    pub const MinimumNumberOfItemsPerChunk: usize = 75;
    pub const ItemsPerChunk: usize = N;
    /// All dependencies must have their original lifetime; this object may not move.
    /// N is the selected internal representation's source-order capacity computation.
    /// Restricted current payload/default and allocator/TLS setup operations must not unwind;
    /// generic exceptional partial-construction cleanup is not yet mapped.
    pub unsafe fn init_with_at<'a>(
        out: Pin<&'a mut MaybeUninit<Self>>,
        id: &str,
        allocator: A,
        tls_system: S,
    ) -> Pin<&'a mut Self> {
        assert!(N >= 75);
        let p = out.get_unchecked_mut().as_mut_ptr();
        ptr::addr_of_mut!((*p).allocator_).write(allocator);
        ptr::addr_of_mut!((*p).tls_system_).write(tls_system);
        // root is initialized after both parameter objects, before Setup.
        let root = Self::AllocateAndInitializeChunkRaw(ptr::addr_of!((*p).allocator_));
        ptr::addr_of_mut!((*p).root_).write(AtomicPtr::new(root));
        ptr::addr_of_mut!((*p)._pin).write(PhantomPinned);
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"dereference(tls_system_).Setup(&MarkSlotAsFree, instance_id)\0".as_ptr().cast();(*p).tls_system_.Setup(Self::MarkSlotAsFree,id));
        Pin::new_unchecked(&mut *p)
    }
    pub unsafe fn GetThreadLocalData(self: Pin<&Self>) -> *mut T {
        let this = self.get_ref();
        let mut slot = this
            .tls_system_
            .GetThreadSpecificData()
            .cast::<SingleSlot<T>>();
        if core::intrinsics::unlikely(slot.is_null()) {
            slot = this.FindAndAllocateFreeSlot(this.root_.load(Ordering::Relaxed));
            if !this.tls_system_.SetThreadSpecificData(slot.cast()) {
                crate::base::check::RawCheckFailure(b"TLS System: Failed to set thread specific data. Failed condition 'tls_system.SetThreadSpecificData(slot)' in (base/allocator/dispatcher/tls.h@260).\n\0".as_ptr().cast());
            }
            // Native reset occurs only after TLS registration succeeds.
            (*slot).item.get().write(T::default());
        }
        (*slot).item.get()
    }
    unsafe extern "C" fn MarkSlotAsFree(data: *mut c_void) {
        let slot = data.cast::<SingleSlot<T>>();
        if slot.is_null() || !(*slot).is_used.swap(true, Ordering::SeqCst) {
            crate::base::check::RawCheckFailure(b"TLS System: Received an invalid slot. Failed condition 'slot && slot->is_used.test_and_set()' in (base/allocator/dispatcher/tls.h@364).\n\0".as_ptr().cast());
        }
        (*slot).is_used.store(false, Ordering::Relaxed);
    }
    unsafe fn AllocateAndInitializeChunkRaw(allocator: *const A) -> *mut Chunk<T, N> {
        let raw = (*allocator)
            .AllocateMemory(core::mem::size_of::<Chunk<T, N>>())
            .cast::<Chunk<T, N>>();
        if raw.is_null() {
            crate::base::check::RawCheckFailure(b"TLS System: Failed to allocate memory for new chunk. Failed condition 'uninitialized_memory != nullptr' in (base/allocator/dispatcher/tls.h@387).\n\0".as_ptr().cast());
        }
        // Value construction, never zero-fill arbitrary payloads.
        let slots = ptr::addr_of_mut!((*raw).slots).cast::<SingleSlot<T>>();
        for i in 0..N {
            slots.add(i).write(SingleSlot {
                item: UnsafeCell::new(T::default()),
                is_used: AtomicBool::new(false),
            });
        }
        ptr::addr_of_mut!((*raw).next_chunk).write(AtomicPtr::new(ptr::null_mut()));
        ptr::addr_of_mut!((*raw).create_next_chunk_flag).write(std::sync::Once::new());
        raw
    }
    unsafe fn FindAndAllocateFreeSlot(&self, chunk: *mut Chunk<T, N>) -> *mut SingleSlot<T> {
        for i in 0..N {
            let slot = ptr::addr_of_mut!((*chunk).slots)
                .cast::<SingleSlot<T>>()
                .add(i);
            if !(*slot).is_used.swap(true, Ordering::Relaxed) {
                return slot;
            }
        }
        (*chunk).create_next_chunk_flag.call_once(|| {
            (*chunk).next_chunk.store(
                Self::AllocateAndInitializeChunkRaw(&self.allocator_),
                Ordering::Relaxed,
            );
        });
        self.FindAndAllocateFreeSlot((*chunk).next_chunk.load(Ordering::SeqCst))
    }
    unsafe fn TearDownForTesting(&mut self) {
        crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"dereference(tls_system_).TearDownForTesting()\0".as_ptr().cast();self.tls_system_.TearDownForTesting());
        let mut chunk = self.root_.load(Ordering::SeqCst);
        while !chunk.is_null() {
            let next = (*chunk).next_chunk.load(Ordering::SeqCst);
            ptr::drop_in_place(chunk);
            crate::CHECK!(@location crate::base::location::Location::Current(); @condition b"dereference(allocator_).FreeMemoryForTesting(chunk_to_erase, sizeof(Chunk))\0".as_ptr().cast();self.allocator_.FreeMemoryForTesting(chunk.cast(),core::mem::size_of::<Chunk<T,N>>()));
            chunk = next;
        }
    }
}
impl<
        T: Copy + Default,
        A: AllocatorInterface,
        S: TLSSystemInterface,
        const N: usize,
        const TEST: bool,
    > Drop for Storage<T, A, S, N, TEST>
{
    fn drop(&mut self) {
        const {
            assert!(
                TEST,
                "ThreadLocalStorage cannot be destructed outside of test code."
            );
        }
        unsafe {
            self.TearDownForTesting();
        }
    }
}

// Source: tls.h:298-367,490-507. Rust-internal repr(C) Chunk sizing uses
// the actual Rust field sizes/alignments; this is adapted, not C++ ABI identity.
const fn aligned(n: usize, a: usize) -> usize {
    (n + a - 1) & !(a - 1)
}
const fn chunk_size<T>(n: usize) -> usize {
    let sa = core::mem::align_of::<SingleSlot<T>>();
    let pa = core::mem::align_of::<AtomicPtr<()>>();
    let oa = core::mem::align_of::<std::sync::Once>();
    let a = if sa > pa { sa } else { pa };
    let a = if a > oa { a } else { oa };
    let p = aligned(core::mem::size_of::<SingleSlot<T>>() * n, pa)
        + core::mem::size_of::<AtomicPtr<()>>();
    aligned(aligned(p, oa) + core::mem::size_of::<std::sync::Once>(), a)
}
pub const fn items_per_chunk<T>() -> usize {
    let bytes = MMapAllocator::AllocationChunkSize;
    let mut lower = 75;
    let mut upper = bytes / core::mem::size_of::<T>() + 1;
    if chunk_size::<T>(75) >= bytes {
        return 75;
    }
    loop {
        if lower == upper {
            return lower;
        }
        let cur = (upper - lower) / 2 + lower;
        if chunk_size::<T>(cur) > bytes {
            upper = cur;
            continue;
        }
        if chunk_size::<T>(cur + 1) < bytes {
            lower = cur + 1;
            continue;
        }
        return cur;
    }
}
pub type ThreadLocalStorage<T> =
    Storage<T, MMapAllocator, PThreadTLSSystem, { items_per_chunk::<T>() }, false>;
impl<T: Copy + Default, const N: usize, const TEST: bool>
    Storage<T, MMapAllocator, PThreadTLSSystem, N, TEST>
{
    pub unsafe fn init_at<'a>(out: Pin<&'a mut MaybeUninit<Self>>, id: &str) -> Pin<&'a mut Self> {
        Self::init_with_at(out, id, MMapAllocator, PThreadTLSSystem::new())
    }
}

// Setup/move/teardown require the original exclusive lifecycle preconditions.
// After setup, pthread get/set operate per-thread and initialized_ is atomic.
unsafe impl Sync for PThreadTLSSystem {}
unsafe impl Send for PThreadTLSSystem {}
unsafe impl<
        T: Copy + Default + Send,
        A: AllocatorInterface + Sync,
        S: TLSSystemInterface + Sync,
        const N: usize,
        const TEST: bool,
    > Sync for Storage<T, A, S, N, TEST>
{
}
