// Source: src/core/SkTSearch.h:39-71
/// Safety: base addresses count records separated by elem_size bytes, correctly aligned
/// and initialized as T. Comparators are a strict weak ordering across T and K.
pub unsafe fn SkTSearch<T, K: ?Sized>(
    base: *const T,
    count: i32,
    key: &K,
    elem_size: usize,
    less: impl Fn(&T, &K) -> bool,
    less_reverse: impl Fn(&K, &T) -> bool,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkTSearch.h\0".as_ptr(),43; @condition b"count >= 0\0".as_ptr(); count >= 0);
    if count <= 0 {
        return !0;
    }
    crate::SkASSERT!(@source b"src/core/SkTSearch.h\0".as_ptr(),48; @condition b"base != nullptr\0".as_ptr(); !base.is_null());
    let mut lo = 0;
    let mut hi = count - 1;
    while lo < hi {
        let mid = lo + ((hi - lo) >> 1);
        let elem = unsafe { &*base.cast::<u8>().add(mid as usize * elem_size).cast::<T>() };
        if less(elem, key) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let elem = unsafe { &*base.cast::<u8>().add(hi as usize * elem_size).cast::<T>() };
    if less(elem, key) {
        hi += 1;
        hi = !hi;
    } else if less_reverse(key, elem) {
        hi = !hi;
    }
    hi
}
// Source: src/core/SkTSearch.h:74-78
pub unsafe fn SkTSearchWith<T>(
    base: *const T,
    count: i32,
    target: &T,
    elem_size: usize,
    less: impl Fn(&T, &T) -> bool,
) -> i32 {
    unsafe { SkTSearch(base, count, target, elem_size, &less, &less) }
}
// Source: src/core/SkTSearch.h:81-84
pub unsafe fn SkTSearchOrd<T: PartialOrd>(
    base: *const T,
    count: i32,
    target: &T,
    elem_size: usize,
) -> i32 {
    unsafe { SkTSearchWith(base, count, target, elem_size, |a, b| a < b) }
}
// Source: src/core/SkTSearch.h:88-92
pub unsafe fn SkTSearchPointers<T>(
    base: *const *const T,
    count: i32,
    target: *const T,
    elem_size: usize,
    less: impl Fn(&T, &T) -> bool,
) -> i32 {
    unsafe { SkTSearchWith(base, count, &target, elem_size, |a, b| less(&**a, &**b)) }
}

// Source: src/core/SkTSearch.h:112-118
// Source: src/core/SkTSearch.h:121-126
// This is the actual native storage/protocol, not a replacement lowercase
// algorithm. Constructor placement requires Pin so fStorage cannot move after
// fLC points into it. Its actual out-of-line constructor/destructor follow below.
#[must_use]
#[repr(C)]
pub struct SkAutoAsciiToLC {
    // Source: src/core/SkTSearch.h:121-121
    fLC: *mut core::ffi::c_char,
    // Source: src/core/SkTSearch.h:122-122
    fLength: usize,
    // Source: src/core/SkTSearch.h:126-126
    fStorage: [core::mem::MaybeUninit<core::ffi::c_char>; SkAutoAsciiToLC::STORAGE + 1],
    _pin: core::marker::PhantomPinned,
}
impl SkAutoAsciiToLC {
    // Source: src/core/SkTSearch.h:117-117
    pub fn lc(&self) -> *const core::ffi::c_char {
        self.fLC
    }
    // Source: src/core/SkTSearch.h:118-118
    pub fn length(&self) -> usize {
        self.fLength
    }
    // Source: src/core/SkTSearch.h:123-125
    const STORAGE: usize = 64;
}
// Source: src/core/SkTSearch.h:130-130
/// Safety: comparator uses the C qsort calling convention and only receives
/// correctly aligned initialized T records while called through this pointer.
/// Its pointer arguments have the same representation as const void*, exactly
/// the original reinterpret_cast. Rust-ABI callbacks are deliberately rejected.
pub unsafe fn SkCastForQSort<T>(
    compare: unsafe extern "C" fn(*const T, *const T) -> core::ffi::c_int,
) -> unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> core::ffi::c_int {
    core::mem::transmute(compare)
}

// Standard C runtime operations are primitive ABI substitutions, not alternate
// Skia algorithms. Throwing allocation maps allocation failure to Rust's fatal
// allocator handler; malloc/free retain native unsized deallocation.
unsafe extern "C" {
    fn strlen(s: *const core::ffi::c_char) -> usize;
    fn strncmp(
        a: *const core::ffi::c_char,
        b: *const core::ffi::c_char,
        n: usize,
    ) -> core::ffi::c_int;
    fn tolower(c: core::ffi::c_int) -> core::ffi::c_int;
    fn malloc(n: usize) -> *mut core::ffi::c_void;
    fn free(p: *mut core::ffi::c_void);
}
// Source: src/core/SkTSearch.cpp:16-20
unsafe fn index_into_base(
    base: *const *const core::ffi::c_char,
    index: i32,
    elem_size: usize,
) -> *const core::ffi::c_char {
    *base
        .cast::<u8>()
        .add((index as usize).wrapping_mul(elem_size))
        .cast::<*const core::ffi::c_char>()
}
// Source: src/core/SkTSearch.h:94-95
// Source: src/core/SkTSearch.cpp:22-56
pub unsafe fn SkStrSearch(
    base: *const *const core::ffi::c_char,
    count: i32,
    target: *const core::ffi::c_char,
    target_len: usize,
    elem_size: usize,
) -> i32 {
    if count <= 0 {
        return !0;
    }
    crate::SkASSERT!(@source b"src/core/SkTSearch.cpp\0".as_ptr(),28; @condition b"base != nullptr\0".as_ptr(); !base.is_null());
    let mut lo = 0;
    let mut hi = count - 1;
    while lo < hi {
        let mid = (hi + lo) >> 1;
        let elem = index_into_base(base, mid, elem_size);
        let cmp = strncmp(elem, target, target_len);
        if cmp < 0 {
            lo = mid + 1;
        } else if cmp > 0 || strlen(elem) > target_len {
            hi = mid;
        } else {
            return mid;
        }
    }
    let elem = index_into_base(base, hi, elem_size);
    let cmp = strncmp(elem, target, target_len);
    if cmp != 0 || strlen(elem) > target_len {
        if cmp < 0 {
            hi += 1;
        }
        hi = !hi;
    }
    hi
}
// Source: src/core/SkTSearch.h:96-97
// Source: src/core/SkTSearch.cpp:58-62
pub unsafe fn SkStrSearchNullTerminated(
    base: *const *const core::ffi::c_char,
    count: i32,
    target: *const core::ffi::c_char,
    elem_size: usize,
) -> i32 {
    SkStrSearch(base, count, target, strlen(target), elem_size)
}
// Source: src/core/SkTSearch.h:102-103
// Source: src/core/SkTSearch.cpp:64-72
pub unsafe fn SkStrLCSearch(
    base: *const *const core::ffi::c_char,
    count: i32,
    target: *const core::ffi::c_char,
    len: usize,
    elem_size: usize,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkTSearch.cpp\0".as_ptr(),67; @condition b"target\0".as_ptr(); !target.is_null());
    // Native local placement storage stays on the stack; only long strings allocate.
    let mut storage = core::pin::pin!(core::mem::MaybeUninit::<SkAutoAsciiToLC>::uninit());
    let mut tolc = SkAutoAsciiToLC::new_in(storage.as_mut(), target, len);
    let result = SkStrSearch(base, count, tolc.lc(), len, elem_size);
    core::ptr::drop_in_place(core::pin::Pin::get_unchecked_mut(tolc.as_mut()));
    result
}
// Source: src/core/SkTSearch.h:104-105
// Source: src/core/SkTSearch.cpp:74-78
pub unsafe fn SkStrLCSearchNullTerminated(
    base: *const *const core::ffi::c_char,
    count: i32,
    target: *const core::ffi::c_char,
    elem_size: usize,
) -> i32 {
    SkStrLCSearch(base, count, target, strlen(target), elem_size)
}
impl SkAutoAsciiToLC {
    // Source: src/core/SkTSearch.h:114-114
    // Source: src/core/SkTSearch.cpp:82-110
    /// Safety: destination is exclusive pinned uninitialized storage. The caller
    /// must destroy the returned initialized object exactly once before reusing or
    /// releasing that storage. str is readable for len (or NUL terminated for MAX).
    pub unsafe fn new_in<'a>(
        mut destination: core::pin::Pin<&'a mut core::mem::MaybeUninit<Self>>,
        str: *const core::ffi::c_char,
        mut len: usize,
    ) -> core::pin::Pin<&'a mut Self> {
        let ptr = destination.as_mut().get_unchecked_mut().as_mut_ptr();
        if (len as core::ffi::c_long) < 0 {
            len = strlen(str);
        }
        core::ptr::addr_of_mut!((*ptr).fLength).write(len);
        let lc = if len <= Self::STORAGE {
            core::ptr::addr_of_mut!((*ptr).fStorage).cast::<core::ffi::c_char>()
        } else {
            let allocation = malloc(len.wrapping_add(1)).cast::<core::ffi::c_char>();
            if allocation.is_null() {
                std::alloc::handle_alloc_error(std::alloc::Layout::from_size_align_unchecked(
                    len.wrapping_add(1),
                    1,
                ));
            }
            allocation
        };
        core::ptr::addr_of_mut!((*ptr).fLC).write(lc);
        core::ptr::addr_of_mut!((*ptr)._pin).write(core::marker::PhantomPinned);
        let mut i = len.wrapping_sub(1) as i32;
        while i >= 0 {
            let mut c = *str.add(i as usize) as core::ffi::c_int;
            if c & 0x80 == 0 {
                c = tolower(c);
            }
            *lc.add(i as usize) = c as core::ffi::c_char;
            i -= 1;
        }
        *lc.add(len) = 0;
        core::pin::Pin::new_unchecked(&mut *ptr)
    }
}
// Source: src/core/SkTSearch.h:115-115
// Source: src/core/SkTSearch.cpp:112-117
impl Drop for SkAutoAsciiToLC {
    fn drop(&mut self) {
        let storage = self.fStorage.as_mut_ptr().cast::<core::ffi::c_char>();
        if self.fLC != storage {
            unsafe { free(self.fLC.cast()) }
        }
    }
}
