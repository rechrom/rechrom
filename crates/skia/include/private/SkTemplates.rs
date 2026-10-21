use core::{
    mem::{self, MaybeUninit},
    ptr,
};

// Source: include/private/SkTemplates.h:37-37
pub fn sk_ignore_unused_variable<T: ?Sized>(_: &T) {}

// Source: include/private/SkTemplates.h:43-48
pub fn SkTAbs<T>(mut value: T) -> T
where
    T: PartialOrd + core::ops::Neg<Output = T> + From<i8>,
{
    if value < T::from(0) {
        value = -value;
    }
    value
}

// Source: include/private/SkTemplates.h:53-55
/// Safety: ptr/count stay within the same allocated S array (or one past it).
pub unsafe fn SkTAfter<D, S>(ptr: *const S, count: usize) -> *const D {
    unsafe { ptr.add(count).cast() }
}
/// Safety: same array/provenance contract as SkTAfter, with exclusive destination access.
pub unsafe fn SkTAfterMut<D, S>(ptr: *mut S, count: usize) -> *mut D {
    unsafe { ptr.add(count).cast() }
}

// Source: include/private/SkTemplates.h:60-64
/// Safety: byte_offset remains within ptr's allocated object (or one past it).
pub unsafe fn SkTAddOffset<D, S>(ptr: *const S, byte_offset: isize) -> *const D {
    unsafe { ptr.cast::<u8>().offset(byte_offset).cast() }
}
/// Safety: same object/provenance contract as SkTAddOffset, retaining exclusive access.
pub unsafe fn SkTAddOffsetMut<D, S>(ptr: *mut S, byte_offset: isize) -> *mut D {
    unsafe { ptr.cast::<u8>().offset(byte_offset).cast() }
}

// Source: include/private/SkTemplates.h:66-74
// Rust function items preserve the native statically selected callable. Tuple
// arguments replace C++ variadic forwarding; arguments are moved exactly once.
pub struct SkOverloadedFunctionObject<F>(pub F);
impl<F> SkOverloadedFunctionObject<F> {
    pub fn call<Args, R>(&mut self, args: Args) -> R
    where
        F: FnMut(Args) -> R,
    {
        (self.0)(args)
    }
}
pub type SkFunctionObject<F> = SkOverloadedFunctionObject<F>;

// Source: include/private/SkTemplates.h:84-93
// Native unique_ptr's inherited state/reset/release become an explicit pointer
// owner. Copyable function items replace the native compile-time deleter value.
pub struct SkAutoTCallVProc<T, F: FnMut(*mut T)> {
    pointer: *mut T,
    deleter: F,
}
impl<T, F: FnMut(*mut T)> SkAutoTCallVProc<T, F> {
    /// Safety: pointer is null or uniquely owned and must be released with exactly deleter.
    /// No other owner may destroy it or access it after this owner's disposal.
    pub unsafe fn new(pointer: *mut T, deleter: F) -> Self {
        Self { pointer, deleter }
    }
    pub fn get(&self) -> *mut T {
        self.pointer
    }
    pub fn release(&mut self) -> *mut T {
        core::mem::replace(&mut self.pointer, core::ptr::null_mut())
    }
    /// Safety: pointer transfers a new sole ownership matching this object's deleter.
    pub unsafe fn reset(&mut self, pointer: *mut T) {
        let previous = core::mem::replace(&mut self.pointer, pointer);
        if !previous.is_null() {
            (self.deleter)(previous);
        }
    }
    pub fn take_from(other: &mut Self) -> Self
    where
        F: Copy,
    {
        Self {
            pointer: other.release(),
            deleter: other.deleter,
        }
    }
}
impl<T, F: FnMut(*mut T)> Drop for SkAutoTCallVProc<T, F> {
    fn drop(&mut self) {
        let pointer = self.release();
        if !pointer.is_null() {
            (self.deleter)(pointer);
        }
    }
}

// Source: include/private/SkTemplates.h:100-102
impl<T> Default for AutoTArray<T> {
    fn default() -> Self {
        Self {
            fData: None,
            fSize: 0,
            initialized: 0,
        }
    }
}
impl<T> AutoTArray<T> {
    pub fn new() -> Self {
        Self::default()
    }
}
// Source: include/private/SkTemplates.h:104-109
// Rust has no universal C++ default constructor: scalar default-initialization
// may leave bytes uninitialized. An explicit constructor implements the native
// T policy; trivial scratch arrays instead use the unsafe uninitialized entry.
impl<T> AutoTArray<T> {
    pub fn with_constructor(size: usize, mut construct: impl FnMut() -> T) -> Self {
        check_array_bytes::<T>(size);
        let mut array = Self {
            fData: if size > 0 {
                Some(Box::<[T]>::new_uninit_slice(size))
            } else {
                None
            },
            fSize: size,
            initialized: 0,
        };
        for i in 0..size {
            unsafe {
                array.data_mut().add(i).write(construct());
            }
            array.initialized += 1;
        }
        array
    }
    pub fn with_constructor_i32(size: i32, construct: impl FnMut() -> T) -> Self {
        if size < 0 {
            std::process::abort();
        }
        Self::with_constructor(size as usize, construct)
    }
    /// # Safety
    /// T must match a native trivially default-constructible/destructible type.
    /// Its bytes remain uninitialized, as native `new T[count]` for a scalar.
    /// Write every element before assume_init()/forming a T reference or reading it.
    pub unsafe fn new_uninit(size: usize) -> Self
    where
        T: Copy,
    {
        check_array_bytes::<T>(size);
        Self {
            fData: if size > 0 {
                Some(Box::<[T]>::new_uninit_slice(size))
            } else {
                None
            },
            fSize: size,
            initialized: 0,
        }
    }
}
// Source: include/private/SkTemplates.h:111-121
impl<T> AutoTArray<T> {
    pub fn take_from(other: &mut Self) -> Self {
        Self {
            fData: other.fData.take(),
            fSize: mem::replace(&mut other.fSize, 0),
            initialized: mem::replace(&mut other.initialized, 0),
        }
    }
    pub fn move_assign(&mut self, other: &mut Self) {
        if !ptr::eq(self, other) {
            // unique_ptr releases the source pointer and installs it before deleting
            // the old array; source size is exchanged only after old destruction.
            let incoming = other.fData.take();
            let incoming_init = mem::replace(&mut other.initialized, 0);
            let old = Self {
                fData: mem::replace(&mut self.fData, incoming),
                fSize: self.fSize,
                initialized: mem::replace(&mut self.initialized, incoming_init),
            };
            drop(old);
            self.fSize = mem::replace(&mut other.fSize, 0);
        }
    }
}
// Source: include/private/SkTemplates.h:125-127
impl<T> AutoTArray<T> {
    // Native AutoTArray allocates/constructs the replacement before releasing old data,
    // even when the new count equals the old count. Do not optimize to buffer reuse.
    pub fn reset_with_constructor(&mut self, count: usize, construct: impl FnMut() -> T) {
        let mut next = Self::with_constructor(count, construct);
        self.move_assign(&mut next);
    }
    /// # Safety
    /// Same native trivial-type and initialization contract as new_uninit().
    pub unsafe fn reset_uninit(&mut self, count: usize)
    where
        T: Copy,
    {
        let mut next = unsafe { Self::new_uninit(count) };
        self.move_assign(&mut next);
    }
    pub fn reset_empty(&mut self) {
        let mut next = Self::new();
        self.move_assign(&mut next);
    }
}
// Source: include/private/SkTemplates.h:129-162
impl<T> AutoTArray<T> {
    pub fn get(&self) -> *const T {
        self.fData
            .as_ref()
            .map_or(ptr::null(), |b| b.as_ptr().cast())
    }
    pub fn at(&self, index: usize) -> &T {
        check_initialized_index(index, self.fSize, self.initialized);
        unsafe { &*self.get().add(index) }
    }
    pub fn at_mut(&mut self, index: usize) -> &mut T {
        check_initialized_index(index, self.fSize, self.initialized);
        unsafe { &mut *self.data_mut().add(index) }
    }
    pub fn data(&self) -> *const T {
        self.get()
    }
    pub fn data_mut(&mut self) -> *mut T {
        self.fData
            .as_deref_mut()
            .map_or(ptr::null_mut(), |b| b.as_mut_ptr().cast())
    }
    pub fn size(&self) -> usize {
        self.fSize
    }
    pub fn empty(&self) -> bool {
        self.fSize == 0
    }
    pub fn size_bytes(&self) -> usize {
        mem::size_of::<T>() * self.fSize
    }
    pub fn begin(&self) -> *const T {
        self.get()
    }
    pub fn begin_mut(&mut self) -> *mut T {
        self.data_mut()
    }
    pub fn end(&self) -> *const T {
        if self.get().is_null() {
            crate::SkASSERT!(@source b"include/private/SkTemplates.h\0".as_ptr(), 153; @condition b"fSize == 0\0".as_ptr(); (self.fSize) == (0));
        }
        self.get().wrapping_add(self.fSize)
    }
    pub fn end_mut(&mut self) -> *mut T {
        let size = self.fSize;
        let data = self.data_mut();
        if data.is_null() {
            crate::SkASSERT!(@source b"include/private/SkTemplates.h\0".as_ptr(), 159; @condition b"fSize == 0\0".as_ptr(); (size) == (0));
        }
        data.wrapping_add(size)
    }
    /// # Safety
    /// Every one of size() slots contains a valid initialized T; no overlapping
    /// element references exist. This does not write or default-fill any bytes.
    pub unsafe fn assume_init(&mut self) {
        self.initialized = self.fSize;
    }
    pub fn as_slice(&self) -> Option<&[T]> {
        if self.initialized != self.fSize {
            return None;
        }
        Some(if self.fSize == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.get(), self.fSize) }
        })
    }
    pub fn as_mut_slice(&mut self) -> Option<&mut [T]> {
        if self.initialized != self.fSize {
            return None;
        }
        Some(if self.fSize == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(self.data_mut(), self.fSize) }
        })
    }
}
// Source: include/private/SkTemplates.h:165-167
// Box<[MaybeUninit<T>]> is an exact heap-array allocation, never Vec capacity.
// Initialized-prefix tracking reproduces new[] partial-construction cleanup and
// prohibits safe T references into native uninitialized scalar scratch memory.
pub struct AutoTArray<T> {
    fData: Option<Box<[MaybeUninit<T>]>>,
    fSize: usize,
    initialized: usize,
}
impl<T> Drop for AutoTArray<T> {
    fn drop(&mut self) {
        let mut guard = ReverseDropGuard {
            data: self.data_mut(),
            remaining: self.initialized,
        };
        self.initialized = 0;
        unsafe {
            guard.finish();
        }
    }
}

// Rust-only validity and unwind support. No source from unassigned malloc/new
// implementations is counted; Box owns the array allocation, not its T values.
fn check_array_bytes<T>(count: usize) {
    if mem::size_of::<T>() == 0
        || count
            .checked_mul(mem::size_of::<T>())
            .is_none_or(|n| n > isize::MAX as usize)
    {
        std::process::abort();
    }
}
fn check_initialized_index(index: usize, count: usize, initialized: usize) {
    if index >= count || index >= initialized {
        std::process::abort();
    }
}
struct ReverseDropGuard<T> {
    data: *mut T,
    remaining: usize,
}
impl<T> ReverseDropGuard<T> {
    unsafe fn finish(&mut self) {
        while self.remaining > 0 {
            self.remaining -= 1;
            unsafe {
                ptr::drop_in_place(self.data.add(self.remaining));
            }
        }
    }
}
impl<T> Drop for ReverseDropGuard<T> {
    fn drop(&mut self) {
        unsafe {
            self.finish();
        }
    }
}

// Source: include/private/SkTemplates.h:174-177
// No Clone, move-out projection, or mutable Deref is exposed. A default empty
// shell may move before Pin; every operation that binds inline storage is pinned.
impl<T, const N: usize> AutoSTArray<T, N> {
    fn inline(&self) -> *const T {
        ptr::addr_of!(self.storage).cast()
    }
    unsafe fn bind_inline(&mut self) {
        let fresh = ptr::addr_of_mut!(self.storage).cast::<T>();
        if !self.fArray.is_null() && self.fArray.addr() == fresh.addr() {
            self.fArray = fresh;
        }
    }
}
// Source: include/private/SkTemplates.h:179-192
impl<T, const N: usize> AutoSTArray<T, N> {
    /// # Safety
    /// Target is a newly empty pinned shell. Source's initialized inline objects
    /// implement the actual native T move constructor via move_constructor, which
    /// leaves a valid moved-from T in every source slot. Both arrays stay pinned.
    /// Heap ownership transfers; inline source retains its native count and objects.
    pub unsafe fn init_move_with(
        self: core::pin::Pin<&mut Self>,
        other: core::pin::Pin<&mut Self>,
        mut move_constructor: impl FnMut(&mut T) -> T,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        let other = unsafe { other.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
            other.bind_inline();
        }
        assert!(
            this.fArray.is_null() && this.fCount == 0,
            "native move constructor requires empty target"
        );
        if other.fArray.is_null() {
            this.fArray = ptr::null_mut();
            this.fCount = 0;
            this.initialized = 0;
        } else if other.fArray.addr() == other.inline().addr() {
            if other.initialized != other.fCount as usize {
                std::process::abort();
            }
            this.fArray = ptr::addr_of_mut!(this.storage).cast();
            this.fCount = other.fCount;
            for i in 0..this.fCount as usize {
                let value = move_constructor(unsafe { &mut *other.fArray.add(i) });
                unsafe {
                    this.fArray.add(i).write(value);
                }
                this.initialized += 1;
            }
        } else {
            this.fArray = mem::replace(&mut other.fArray, ptr::null_mut());
            this.fCount = mem::replace(&mut other.fCount, 0);
            this.initialized = mem::replace(&mut other.initialized, 0);
        }
    }
    /// # Safety
    /// Native T's move is a trivial value copy, and the source is fully initialized.
    pub unsafe fn init_move_copy(self: core::pin::Pin<&mut Self>, other: core::pin::Pin<&mut Self>)
    where
        T: Copy,
    {
        unsafe {
            self.init_move_with(other, |v| *v);
        }
    }
}
// Source: include/private/SkTemplates.h:195-198
impl<T, const N: usize> AutoSTArray<T, N> {
    pub fn new() -> Self {
        check_array_bytes::<T>(0);
        if mem::align_of::<T>() > 16 {
            std::process::abort();
        }
        Self {
            fArray: ptr::null_mut(),
            storage: AutoStorage {
                requested: [const { MaybeUninit::uninit() }; N],
                padding: MaybeUninit::uninit(),
            },
            fCount: 0,
            initialized: 0,
            pinned: core::marker::PhantomPinned,
        }
    }
}
impl<T, const N: usize> Default for AutoSTArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
// Source: include/private/SkTemplates.h:201-205
impl<T, const N: usize> AutoSTArray<T, N> {
    pub fn init_with_constructor(
        self: core::pin::Pin<&mut Self>,
        count: i32,
        construct: impl FnMut() -> T,
    ) {
        assert!(
            self.fArray.is_null() && self.fCount == 0,
            "native initial constructor requires empty shell"
        );
        self.reset_with_constructor(count, construct);
    }
    /// # Safety
    /// Native T is trivially default-constructible/destructible. Slots remain
    /// uninitialized; initialize every slot before assume_init or any typed read.
    pub unsafe fn init_uninit(self: core::pin::Pin<&mut Self>, count: i32)
    where
        T: Copy,
    {
        assert!(
            self.fArray.is_null() && self.fCount == 0,
            "native initial constructor requires empty shell"
        );
        unsafe {
            self.reset_uninit(count);
        }
    }
}
// Source: include/private/SkTemplates.h:207-209
impl<T, const N: usize> Drop for AutoSTArray<T, N> {
    fn drop(&mut self) {
        unsafe {
            self.prepare_reset(0);
        }
    }
}
// Source: include/private/SkTemplates.h:213-246
impl<T, const N: usize> AutoSTArray<T, N> {
    unsafe fn prepare_reset(&mut self, count: i32) {
        unsafe {
            self.bind_inline();
        }
        let mut guard = ReverseDropGuard {
            data: self.fArray,
            remaining: self.initialized,
        };
        self.initialized = 0;
        unsafe {
            guard.finish();
        }
        // Native tests count only after destroying the old objects.
        crate::SkASSERT!(@source b"include/private/SkTemplates.h\0".as_ptr(), 220; @condition b"count >= 0\0".as_ptr(); count >= 0);
        #[cfg(not(sk_debug))]
        if count < 0 {
            std::process::abort();
        }
        if self.fCount != count {
            if self.fArray.addr() != self.inline().addr() {
                unsafe {
                    crate::src::core::SkContainers::sk_free(self.fArray.cast());
                }
            }
            self.fArray = if count as usize > Self::kCount {
                let bytes = (count as usize)
                    .checked_mul(mem::size_of::<T>())
                    .unwrap_or_else(|| std::process::abort());
                crate::src::core::SkContainers::sk_malloc_throw(bytes).cast()
            } else if count > 0 {
                ptr::addr_of_mut!(self.storage).cast()
            } else {
                ptr::null_mut()
            };
            self.fCount = count;
        }
    }
    // T's native default constructor is explicit rather than incorrectly mapping
    // C++ uninitialized scalars to Rust Default's zero value.
    pub fn reset_with_constructor(
        self: core::pin::Pin<&mut Self>,
        count: i32,
        mut construct: impl FnMut() -> T,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.prepare_reset(count);
        }
        for i in 0..count as usize {
            unsafe {
                this.fArray.add(i).write(construct());
            }
            this.initialized += 1;
        }
    }
    /// # Safety
    /// Same native trivial-type and initialize-before-reference contract as init_uninit.
    pub unsafe fn reset_uninit(self: core::pin::Pin<&mut Self>, count: i32)
    where
        T: Copy,
    {
        unsafe {
            self.get_unchecked_mut().prepare_reset(count);
        }
    }
}
// Source: include/private/SkTemplates.h:245-256
impl<T, const N: usize> AutoSTArray<T, N> {
    pub fn trimTo(self: core::pin::Pin<&mut Self>, count: i32) {
        crate::SkASSERT!(@source b"include/private/SkTemplates.h\0".as_ptr(), 246; @condition b"count >= 0\0".as_ptr(); count >= 0);
        #[cfg(not(sk_debug))]
        if count < 0 {
            std::process::abort();
        }
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        if count >= this.fCount {
            return;
        }
        let keep = (count as usize).min(this.initialized);
        let mut guard = ReverseDropGuard {
            data: this.fArray.wrapping_add(keep),
            remaining: this.initialized - keep,
        };
        this.initialized = keep;
        unsafe {
            guard.finish();
        }
        this.fCount = count;
    }
}
// Source: include/private/SkTemplates.h:259-280
impl<T, const N: usize> AutoSTArray<T, N> {
    pub fn count(&self) -> i32 {
        self.fCount
    }
    pub fn get(&self) -> *const T {
        self.fArray
    }
    pub fn begin(&self) -> *const T {
        self.fArray
    }
    pub fn end(&self) -> *const T {
        self.fArray.wrapping_add(self.fCount as usize)
    }
    pub fn data(&self) -> *const T {
        self.fArray
    }
    pub fn size(&self) -> usize {
        self.fCount as usize
    }
    pub fn data_mut(self: core::pin::Pin<&mut Self>) -> *mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.fArray
    }
    pub fn at(&self, index: i32) -> &T {
        if index < 0 {
            std::process::abort();
        }
        check_initialized_index(index as usize, self.fCount as usize, self.initialized);
        unsafe { &*self.fArray.add(index as usize) }
    }
    pub fn at_mut(self: core::pin::Pin<&mut Self>, index: i32) -> &mut T {
        if index < 0 {
            std::process::abort();
        }
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        check_initialized_index(index as usize, this.fCount as usize, this.initialized);
        unsafe { &mut *this.fArray.add(index as usize) }
    }
    /// # Safety
    /// Every count() slot was written with a valid T, with no overlapping element references.
    pub unsafe fn assume_init(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.initialized = this.fCount as usize;
    }
    pub fn as_slice(&self) -> Option<&[T]> {
        if self.initialized != self.fCount as usize {
            return None;
        }
        Some(if self.fCount == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.fArray, self.fCount as usize) }
        })
    }
    pub fn as_mut_slice(self: core::pin::Pin<&mut Self>) -> Option<&mut [T]> {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        if this.initialized != this.fCount as usize {
            return None;
        }
        Some(if this.fCount == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(this.fArray, this.fCount as usize) }
        })
    }
}
// Source: include/private/SkTemplates.h:283-292
// Selected native non-Google3 CPU configuration; stack-limit branch is inactive.
impl<T, const N: usize> AutoSTArray<T, N> {
    const kMinCount: usize = N;
}
// Source: include/private/SkTemplates.h:300-301
impl<T, const N: usize> AutoSTArray<T, N> {
    pub const kCount: usize = {
        let size = mem::size_of::<T>();
        let align = if mem::align_of::<T>() > mem::align_of::<*mut T>() {
            mem::align_of::<T>()
        } else {
            mem::align_of::<*mut T>()
        };
        ((Self::kMinCount * size + mem::size_of::<i32>() + align - 1) & !(align - 1)) / size
    };
}
// Source: include/private/SkTemplates.h:304-307
// Stable Rust cannot express [u8; kCount*sizeof(T)] using generic const math.
// This contiguous storage supplies at least that exact logical capacity; a few
// unused padding bytes may differ physically. No bytes beyond kCount are exposed.
// Native malloc alignment supports T <= max_align_t (16) in this selected CPU build.
#[repr(C)]
struct AutoStorage<T, const N: usize> {
    requested: [MaybeUninit<T>; N],
    padding: MaybeUninit<usize>,
}
pub struct AutoSTArray<T, const N: usize> {
    fArray: *mut T,
    storage: AutoStorage<T, N>,
    fCount: i32,
    initialized: usize,
    pinned: core::marker::PhantomPinned,
}
// Native trimTo(0) retains the buffer. If fCount is already0, native reset(0)
// (including its destructor) skips the free branch. Preserve that source
// behavior rather than silently repairing its retained-heap leak here.

// Source: include/private/SkTemplates.h:312-318
// Copy excludes element destruction; no T references are formed from untouched
// POD storage. The canonical native malloc alignment bound is checked explicitly.
pub struct AutoTMalloc<T: Copy> {
    fPtr: *mut T,
}
impl<T: Copy> Default for AutoTMalloc<T> {
    fn default() -> Self {
        check_malloc_type::<T>();
        Self {
            fPtr: ptr::null_mut(),
        }
    }
}
impl<T: Copy> AutoTMalloc<T> {
    pub fn new() -> Self {
        Self::default()
    }
    /// # Safety
    /// Pointer is null or sole ownership of a live allocation accepted by sk_free.
    /// Its alignment and any later initialized reads must satisfy T's requirements.
    pub unsafe fn from_raw(pointer: *mut T) -> Self {
        check_malloc_type::<T>();
        Self { fPtr: pointer }
    }
}
// Source: include/private/SkTemplates.h:320-322
impl<T: Copy> AutoTMalloc<T> {
    pub fn with_count(count: usize) -> Self {
        check_malloc_type::<T>();
        Self {
            fPtr: malloc_pod::<T>(count),
        }
    }
}
// Source: include/private/SkTemplates.h:324-325
impl<T: Copy> AutoTMalloc<T> {
    pub fn take_from(other: &mut Self) -> Self {
        Self {
            fPtr: mem::replace(&mut other.fPtr, ptr::null_mut()),
        }
    }
    pub fn move_assign(&mut self, other: &mut Self) {
        if ptr::eq(self, other) {
            return;
        }
        let incoming = mem::replace(&mut other.fPtr, ptr::null_mut());
        let old = mem::replace(&mut self.fPtr, incoming);
        unsafe {
            crate::src::core::SkContainers::sk_free(old.cast());
        }
    }
}
// Source: include/private/SkTemplates.h:327-330
impl<T: Copy> AutoTMalloc<T> {
    pub fn realloc(&mut self, count: usize) {
        if count != 0 {
            let old = mem::replace(&mut self.fPtr, ptr::null_mut());
            // Native single-size overload multiplies in size_t before realloc.
            self.fPtr = unsafe {
                crate::src::core::SkContainers::sk_realloc_throw(
                    old.cast(),
                    count.wrapping_mul(mem::size_of::<T>()),
                )
            }
            .cast();
        } else {
            let old = mem::replace(&mut self.fPtr, ptr::null_mut());
            unsafe {
                crate::src::core::SkContainers::sk_free(old.cast());
            }
        }
    }
}
// Source: include/private/SkTemplates.h:332-337
impl<T: Copy> AutoTMalloc<T> {
    pub fn reset(&mut self, count: usize) -> *mut T {
        let incoming = malloc_pod::<T>(count);
        let old = mem::replace(&mut self.fPtr, incoming);
        unsafe {
            crate::src::core::SkContainers::sk_free(old.cast());
        }
        self.fPtr
    }
}
// Source: include/private/SkTemplates.h:339-351
impl<T: Copy> AutoTMalloc<T> {
    pub fn get(&self) -> *const T {
        self.fPtr
    }
    pub fn data(&self) -> *const T {
        self.fPtr
    }
    pub fn data_mut(&mut self) -> *mut T {
        self.fPtr
    }
    /// # Safety
    /// Index lies in the live allocation; reads require initialized valid T bytes.
    pub unsafe fn at_raw(&self, index: usize) -> *const T {
        unsafe { self.fPtr.add(index) }
    }
    /// # Safety
    /// Index lies in the live allocation and the resulting access is exclusive.
    pub unsafe fn at_raw_mut(&mut self, index: usize) -> *mut T {
        unsafe { self.fPtr.add(index) }
    }
}
// Source: include/private/SkTemplates.h:353-358
impl<T: Copy> AutoTMalloc<T> {
    pub fn release(&mut self) -> *mut T {
        mem::replace(&mut self.fPtr, ptr::null_mut())
    }
}
// Source: include/private/SkTemplates.h:360-362
impl<T: Copy> Drop for AutoTMalloc<T> {
    fn drop(&mut self) {
        unsafe {
            crate::src::core::SkContainers::sk_free(self.fPtr.cast());
        }
    }
}

// Rust allocator/type validity adaptation, with no new SkMalloc source credit.
fn check_malloc_type<T>() {
    if mem::size_of::<T>() == 0 || mem::align_of::<T>() > 16 {
        std::process::abort();
    }
}
fn malloc_pod<T>(count: usize) -> *mut T {
    check_malloc_type::<T>();
    if count == 0 {
        return ptr::null_mut();
    }
    let bytes = count
        .checked_mul(mem::size_of::<T>())
        .unwrap_or_else(|| std::process::abort());
    crate::src::core::SkContainers::sk_malloc_throw(bytes).cast()
}
fn realloc_pod<T>(pointer: *mut T, count: usize) -> *mut T {
    let bytes = count
        .checked_mul(mem::size_of::<T>())
        .unwrap_or_else(|| std::process::abort());
    unsafe { crate::src::core::SkContainers::sk_realloc_throw(pointer.cast(), bytes) }.cast()
}

// Source: include/private/SkTemplates.h:364-370
// The inline address is bound only after pinning the entire owner. No move-out,
// DerefMut or replace-capable projection of an initialized owner is exposed.
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    /// Construction staging for an owner embedding this actual container. Pin the
    /// complete owner before initialize_default/initialize_count/init_move_from.
    /// # Safety
    /// Never expose or use an uninitialized shell except through its init methods.
    pub unsafe fn new_unbound() -> Self {
        check_malloc_type::<T>();
        Self {
            fPtr: ptr::null_mut(),
            fInline: false,
            initialized: false,
            storage: MallocInlineStorage {
                requested: [const { MaybeUninit::uninit() }; N],
                padding: MaybeUninit::uninit(),
            },
            pinned: core::marker::PhantomPinned,
        }
    }
    pub fn new() -> core::pin::Pin<Box<Self>> {
        let mut owner = Box::pin(unsafe { Self::new_unbound() });
        owner.as_mut().initialize_default();
        owner
    }
    pub fn initialize_default(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        assert!(!this.initialized, "native constructor runs once");
        this.fInline = true;
        this.fPtr = ptr::addr_of_mut!(this.storage).cast();
        this.initialized = true;
    }
}
// Source: include/private/SkTemplates.h:372-380
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    pub fn with_count(count: usize) -> core::pin::Pin<Box<Self>> {
        let mut owner = Box::pin(unsafe { Self::new_unbound() });
        owner.as_mut().initialize_count(count);
        owner
    }
    pub fn initialize_count(self: core::pin::Pin<&mut Self>, count: usize) {
        let this = unsafe { self.get_unchecked_mut() };
        assert!(!this.initialized, "native constructor runs once");
        this.select_storage(count);
        this.initialized = true;
    }
}
// Source: include/private/SkTemplates.h:382-395
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    pub fn take_from(mut other: core::pin::Pin<&mut Self>) -> core::pin::Pin<Box<Self>> {
        let mut owner = Box::pin(unsafe { Self::new_unbound() });
        owner.as_mut().init_move_from(other.as_mut());
        owner
    }
    pub fn init_move_from(self: core::pin::Pin<&mut Self>, other: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        let other = unsafe { other.get_unchecked_mut() };
        assert!(
            !this.initialized && other.initialized,
            "native move construction requires new destination and live source"
        );
        other.bind_inline();
        if other.fPtr.is_null() {
            this.fPtr = ptr::null_mut();
            this.fInline = false;
        } else if other.fInline {
            this.fInline = true;
            this.fPtr = ptr::addr_of_mut!(this.storage).cast();
            // Copy raw MaybeUninit bytes, including indeterminate bytes, without
            // loading T or asserting that the source prefix has been initialized.
            unsafe {
                ptr::copy_nonoverlapping(
                    other.fPtr.cast::<MaybeUninit<u8>>(),
                    this.fPtr.cast::<MaybeUninit<u8>>(),
                    Self::kCount * mem::size_of::<T>(),
                );
            }
        } else {
            this.fInline = false;
            this.fPtr = mem::replace(&mut other.fPtr, ptr::null_mut());
        }
        this.initialized = true;
    }
}
// Source: include/private/SkTemplates.h:397-401
impl<T: Copy, const N: usize> Drop for AutoSTMalloc<T, N> {
    fn drop(&mut self) {
        if self.initialized {
            self.bind_inline();
            if !self.fInline {
                unsafe {
                    crate::src::core::SkContainers::sk_free(self.fPtr.cast());
                }
            }
        }
    }
}
// Source: include/private/SkTemplates.h:403-417
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    pub fn reset(self: core::pin::Pin<&mut Self>, count: usize) -> *mut T {
        let this = unsafe { self.get_unchecked_mut() };
        assert!(this.initialized);
        this.bind_inline();
        if !this.fInline {
            unsafe {
                crate::src::core::SkContainers::sk_free(this.fPtr.cast());
            }
        }
        this.select_storage(count);
        this.fPtr
    }
}
// Source: include/private/SkTemplates.h:419-439
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    pub fn get(&self) -> *const T {
        assert!(self.initialized);
        self.fPtr
    }
    pub fn data(&self) -> *const T {
        self.get()
    }
    pub fn data_mut(self: core::pin::Pin<&mut Self>) -> *mut T {
        let this = unsafe { self.get_unchecked_mut() };
        assert!(this.initialized);
        this.bind_inline();
        this.fPtr
    }
    /// # Safety
    /// Index lies in the actual current allocation; read only initialized valid T.
    pub unsafe fn at_raw(&self, index: usize) -> *const T {
        unsafe { self.get().add(index) }
    }
    /// # Safety
    /// Index lies in the actual allocation, with exclusive access to the result.
    pub unsafe fn at_raw_mut(self: core::pin::Pin<&mut Self>, index: usize) -> *mut T {
        unsafe { self.data_mut().add(index) }
    }
}
// Source: include/private/SkTemplates.h:441-457
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    pub fn realloc(self: core::pin::Pin<&mut Self>, count: usize) {
        let this = unsafe { self.get_unchecked_mut() };
        assert!(this.initialized);
        this.bind_inline();
        if count > Self::kCount {
            if this.fInline {
                this.fPtr = malloc_pod::<T>(count);
                unsafe {
                    ptr::copy_nonoverlapping(
                        ptr::addr_of_mut!(this.storage).cast::<MaybeUninit<u8>>(),
                        this.fPtr.cast::<MaybeUninit<u8>>(),
                        Self::kCount * mem::size_of::<T>(),
                    );
                }
                this.fInline = false;
            } else {
                this.fPtr = realloc_pod(this.fPtr, count);
            }
        } else if count != 0 {
            if !this.fInline {
                this.fPtr = realloc_pod(this.fPtr, count);
            }
        } else {
            if !this.fInline {
                unsafe {
                    crate::src::core::SkContainers::sk_free(this.fPtr.cast());
                }
            }
            this.fPtr = ptr::null_mut();
            this.fInline = false;
        }
    }
}
// Source: include/private/SkTemplates.h:459-474
// Selected CPU configuration is not SK_BUILD_FOR_GOOGLE3.
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    pub const kCount: usize = ((N * mem::size_of::<T>() + 3) & !3) / mem::size_of::<T>();
}
// Source: include/private/SkTemplates.h:476-483
// Stable generic const expressions cannot name the exact padded union byte
// array. This aligned backing provides at least native kCount bytes; only that
// exact native logical capacity is ever exposed/copied. Additional bytes unused.
#[repr(C)]
struct MallocInlineStorage<T, const N: usize> {
    requested: [MaybeUninit<T>; N],
    padding: MaybeUninit<u32>,
}
pub struct AutoSTMalloc<T: Copy, const N: usize> {
    fPtr: *mut T,
    fInline: bool,
    initialized: bool,
    storage: MallocInlineStorage<T, N>,
    pinned: core::marker::PhantomPinned,
}
impl<T: Copy, const N: usize> AutoSTMalloc<T, N> {
    fn bind_inline(&mut self) {
        if self.fInline {
            self.fPtr = ptr::addr_of_mut!(self.storage).cast();
        }
    }
    fn select_storage(&mut self, count: usize) {
        if count > Self::kCount {
            self.fPtr = malloc_pod::<T>(count);
            self.fInline = false;
        } else if count != 0 {
            self.fPtr = ptr::addr_of_mut!(self.storage).cast();
            self.fInline = true;
        } else {
            self.fPtr = ptr::null_mut();
            self.fInline = false;
        }
    }
}

// Source: include/private/SkTemplates.h:485-485
pub type UniqueVoidPtr = SkAutoTCallVProc<core::ffi::c_void, fn(*mut core::ffi::c_void)>;
impl UniqueVoidPtr {
    /// Safety: pointer is null or transfers its sole malloc-compatible allocation owner.
    pub unsafe fn from_malloc(pointer: *mut core::ffi::c_void) -> Self {
        fn free_pointer(pointer: *mut core::ffi::c_void) {
            unsafe {
                crate::src::core::SkContainers::sk_free(pointer.cast());
            }
        }
        unsafe { Self::new(pointer, free_pointer) }
    }
}
// Source: include/private/SkTemplates.h:489-493
pub fn SkMakeArrayFromIndexSequence<T, const N: usize>(
    mut callable: impl FnMut(usize) -> T,
    indices: [usize; N],
) -> [T; N] {
    core::array::from_fn(|i| callable(indices[i]))
}
// Source: include/private/SkTemplates.h:495-498
pub fn SkMakeArray<T, const N: usize>(callable: impl FnMut(usize) -> T) -> [T; N] {
    SkMakeArrayFromIndexSequence(callable, core::array::from_fn(|i| i))
}
