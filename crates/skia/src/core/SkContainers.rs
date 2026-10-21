use core::ffi::c_void;
use core::ptr;

// Rust representation of the native non-owning SkSpan<std::byte> result. Raw bytes
// are uninitialized; ownership transfers to the caller and is released with free.
// This tuple is a language adaptation, not a replacement storage container.
pub type ByteSpan = (*mut u8, usize);
unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    #[cfg(target_vendor = "apple")]
    fn malloc_size(ptr: *const c_void) -> usize;
    #[cfg(target_os = "linux")]
    fn malloc_usable_size(ptr: *mut c_void) -> usize;
}

// Selected native allocator ABI adaptation. It retains the exact Apple/Unix
// usable-size branches of sk_malloc_size, without claiming those dependency lines.
// Unsupported platform branches are not silently replaced with requested capacity.
#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
compile_error!(
    "SkContainers native allocator usable-size branch remains untranslated for this target"
);
unsafe fn allocation_size(p: *mut u8, requested: usize) -> usize {
    #[cfg(target_vendor = "apple")]
    {
        unsafe { malloc_size(p.cast()).max(requested) }
    }
    #[cfg(target_os = "linux")]
    {
        let actual = unsafe { malloc_usable_size(p.cast()) };
        debug_assert!(actual >= requested);
        actual
    }
}
/// Safety: p is null or a live C allocation uniquely transferred to this call.
pub(crate) unsafe fn sk_free(p: *mut u8) {
    unsafe {
        free(p.cast());
    }
}
pub(crate) fn sk_malloc_throw(size: usize) -> *mut u8 {
    let p = unsafe { malloc(size) }.cast::<u8>();
    if p.is_null() && size != 0 {
        std::process::abort();
    }
    p
}

// Native selected C allocator ABI adaptation, not additional SkMalloc source
// credit. The incoming allocation is transferred exactly once; failure aborts.
/// Safety: p is null or the uniquely owned live C allocation being resized.
pub(crate) unsafe fn sk_realloc_throw(p: *mut u8, size: usize) -> *mut u8 {
    let result = unsafe { realloc(p.cast(), size) }.cast::<u8>();
    if result.is_null() && size != 0 {
        std::process::abort();
    }
    result
}

// Source: include/private/SkContainers.h:14-18
impl SkContainerAllocator {
    pub fn new(sizeOfT: usize, maxCapacity: i32) -> Self {
        Self {
            fSizeOfT: sizeOfT,
            fMaxCapacity: maxCapacity as i64,
        }
    }
}
// Source: include/private/SkContainers.h:25-28
impl SkContainerAllocator {
    pub const fn RoundUp<T>(capacity: usize) -> usize {
        let bytes = capacity.wrapping_mul(core::mem::size_of::<T>());
        (bytes.wrapping_add(Self::kCapacityMultiple as usize - 1)
            & !(Self::kCapacityMultiple as usize - 1))
            / core::mem::size_of::<T>()
    }
}
// Source: include/private/SkContainers.h:35-35
impl SkContainerAllocator {
    const kCapacityMultiple: i64 = 8;
}

// Source: include/private/SkContainers.h:43-45
pub struct SkContainerAllocator {
    fSizeOfT: usize,
    fMaxCapacity: i64,
}

// Source: src/core/SkContainers.cpp:16-16
// Current selected Apple/Linux 64-bit ABI max_align_t alignment is 16, as u128.
const kMinBytes: usize = core::mem::align_of::<u128>();

// Source: src/core/SkContainers.cpp:18-24
unsafe fn complete_size(p: *mut u8, size: usize) -> ByteSpan {
    if p.is_null() {
        return (ptr::null_mut(), 0);
    }
    (p, unsafe { allocation_size(p, size) })
}
// Source: src/core/SkContainers.cpp:27-37
impl SkContainerAllocator {
    pub fn allocate(&self, mut capacity: i32, growthFactor: f64) -> ByteSpan {
        crate::SkASSERT!(@source b"src/core/SkContainers.cpp\0".as_ptr(), 28; @condition b"capacity >= 0\0".as_ptr(); capacity >= 0);
        crate::SkASSERT!(@source b"src/core/SkContainers.cpp\0".as_ptr(), 29; @condition b"growthFactor >= 1.0\0".as_ptr(); growthFactor >= 1.0);
        // Native SK_ASSERT_RELEASE terminates rather than unwinding.
        if capacity as i64 > self.fMaxCapacity {
            std::process::abort();
        }
        if growthFactor > 1.0 && capacity > 0 {
            capacity = self.growthFactorCapacity(capacity, growthFactor) as i32;
        }
        sk_allocate_throw((capacity as usize).wrapping_mul(self.fSizeOfT))
    }
}
// Source: src/core/SkContainers.cpp:39-48
impl SkContainerAllocator {
    pub fn roundUpCapacity(&self, capacity: i64) -> usize {
        crate::SkASSERT!(@source b"src/core/SkContainers.cpp\0".as_ptr(), 40; @condition b"capacity >= 0\0".as_ptr(); capacity >= 0);
        if capacity < self.fMaxCapacity - Self::kCapacityMultiple {
            ((capacity + Self::kCapacityMultiple - 1) & !(Self::kCapacityMultiple - 1)) as usize
        } else {
            self.fMaxCapacity as usize
        }
    }
}
// Source: src/core/SkContainers.cpp:50-59
impl SkContainerAllocator {
    pub fn growthFactorCapacity(&self, capacity: i32, growthFactor: f64) -> usize {
        crate::SkASSERT!(@source b"src/core/SkContainers.cpp\0".as_ptr(), 51; @condition b"capacity >= 0\0".as_ptr(); capacity >= 0);
        crate::SkASSERT!(@source b"src/core/SkContainers.cpp\0".as_ptr(), 52; @condition b"growthFactor >= 1.0\0".as_ptr(); growthFactor >= 1.0);
        let capacityGrowth = (capacity as f64 * growthFactor) as i64;
        self.roundUpCapacity(capacityGrowth)
    }
}
// Source: src/core/SkContainers.cpp:62-67
pub fn sk_allocate_canfail(size: usize) -> ByteSpan {
    let adjustedSize = size.max(kMinBytes);
    let p = unsafe { malloc(adjustedSize) }.cast::<u8>();
    unsafe { complete_size(p, adjustedSize) }
}
// Source: src/core/SkContainers.cpp:69-77
pub fn sk_allocate_throw(size: usize) -> ByteSpan {
    if size == 0 {
        return (ptr::null_mut(), 0);
    }
    let adjustedSize = size.max(kMinBytes);
    let p = sk_malloc_throw(adjustedSize);
    unsafe { complete_size(p, adjustedSize) }
}
// Source: src/core/SkContainers.cpp:79-81
pub fn sk_report_container_overflow_and_die() -> ! {
    crate::SK_ABORT!(@source b"src/core/SkContainers.cpp\0".as_ptr(), 80; "Requested capacity is too large.")
}
