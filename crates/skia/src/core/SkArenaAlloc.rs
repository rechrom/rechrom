// Copyright 2016 Google Inc. BSD-style license.
// Blocks use the platform C allocator, matching sk_malloc/sk_free ownership.
// Objects and destructor footers remain inside Fibonacci-sized blocks; no
// per-object heap allocation or separate destructor vector is introduced.
use core::ffi::c_void;
use core::mem::{align_of, needs_drop, size_of, MaybeUninit};
use core::ptr;
unsafe extern "C" {
    #[link_name = "malloc"]
    fn arena_malloc(size: usize) -> *mut c_void;
    #[link_name = "free"]
    fn arena_free(ptr: *mut c_void);
}
type FooterAction = unsafe fn(*mut u8) -> *mut u8;
const FOOTER_SIZE: usize = size_of::<FooterAction>() + 1;
fn assert_release(cond: bool) {
    if !cond {
        std::process::abort();
    }
}
// Source: src/core/SkArenaAlloc.cpp:16-16
unsafe fn end_chain(_end: *mut u8) -> *mut u8 {
    ptr::null_mut()
}
// Source: src/core/SkArenaAlloc.h:33-65
pub struct SkFibBlockSizes<const MAX: u32> {
    fIndex: usize,
    fBlockUnitSize: u32,
}
impl<const MAX: u32> SkFibBlockSizes<MAX> {
    pub fn new(static_block_size: u32, first_allocation_size: u32) -> Self {
        let unit = if first_allocation_size > 0 {
            first_allocation_size
        } else if static_block_size > 0 {
            static_block_size
        } else {
            1024
        };
        crate::SkASSERT_RELEASE!(@source b"src/core/SkArenaAlloc.h\0".as_ptr(), 45; @condition b"0 < fBlockUnitSize\0".as_ptr(); unit > 0);
        crate::SkASSERT_RELEASE!(@source b"src/core/SkArenaAlloc.h\0".as_ptr(), 46; @condition b"fBlockUnitSize < std::min(kMaxSize, (1u << 26) - 1)\0".as_ptr(); unit < MAX.min((1 << 26) - 1));
        Self {
            fIndex: 0,
            fBlockUnitSize: unit,
        }
    }
    pub fn nextBlockSize(&mut self) -> u32 {
        let result = SkFibonacci47[self.fIndex].wrapping_mul(self.fBlockUnitSize);
        if self.fIndex + 1 < SkFibonacci47.len()
            && SkFibonacci47[self.fIndex + 1] < MAX / self.fBlockUnitSize
        {
            self.fIndex += 1;
        }
        result
    }
}
impl SkArenaAlloc {
    // Source: src/core/SkArenaAlloc.h:110-111
    pub fn new(first_heap_allocation: usize) -> Self {
        unsafe { Self::from_block(ptr::null_mut(), 0, first_heap_allocation) }
    }
    // Rust stable-owner constructor; native allocation/footer algorithms unchanged.
    pub fn new_pinned(first_heap_allocation: usize) -> core::pin::Pin<Box<Self>> {
        Box::pin(Self::new(first_heap_allocation))
    }
    /// Allocates an owned object and publishes it only as a pinned borrow.
    /// The arena itself is pinned, and its real heap/inline allocation never
    /// relocates. Arena destruction runs the registered footer in native order.
    pub fn make_pinned<T: 'static>(
        self: core::pin::Pin<&mut Self>,
        value: T,
    ) -> core::pin::Pin<&mut T> {
        unsafe {
            let object = self.get_unchecked_mut().make(value);
            core::pin::Pin::new_unchecked(&mut *object)
        }
    }
    /// Safety: a non-null caller block must remain valid and unmoved until this
    /// arena is dropped, and no other owner may read or overwrite occupied bytes.
    // Source: src/core/SkArenaAlloc.h:108-108
    // Source: src/core/SkArenaAlloc.cpp:18-32
    pub unsafe fn from_block(block: *mut u8, size: usize, first_heap_allocation: usize) -> Self {
        assert_release(size <= u32::MAX as usize && first_heap_allocation <= u32::MAX as usize);
        let mut arena = Self {
            fDtorCursor: block,
            fCursor: block,
            fEnd: block.wrapping_add(size),
            fFibonacciProgression: SkFibBlockSizes::new(size as u32, first_heap_allocation as u32),
            _pin: core::marker::PhantomPinned,
        };
        if size < FOOTER_SIZE {
            arena.fEnd = ptr::null_mut();
            arena.fCursor = ptr::null_mut();
            arena.fDtorCursor = ptr::null_mut();
        }
        if !arena.fCursor.is_null() {
            arena.installFooter(end_chain, 0);
        }
        arena
    }
    // Source: src/core/SkArenaAlloc.h:120-149
    /// Safety: constructor must initialize its returned T without unwinding. The
    /// footer is installed first so nested arena allocations see the same chain.
    /// Any data borrowed by T or accessed by its destructor must remain valid
    /// until this arena is reset or destroyed.
    pub unsafe fn make_with<T>(
        &mut self,
        constructor: impl FnOnce(*mut T, &mut Self) -> T,
    ) -> *mut T {
        let size = size_of::<T>();
        let alignment = align_of::<T>();
        assert_release(size <= u32::MAX as usize && alignment <= u32::MAX as usize);
        let object;
        if !needs_drop::<T>() {
            object = self.allocObject(size as u32, alignment as u32);
            self.fCursor = object.add(size);
        } else {
            assert_release(size <= u32::MAX as usize - FOOTER_SIZE);
            object = self.allocObjectWithFooter((size + FOOTER_SIZE) as u32, alignment as u32);
            let padding = object.offset_from(self.fCursor) as u32;
            self.fCursor = object.add(size);
            self.installFooter(drop_object::<T>, padding);
        }
        let typed = object as *mut T;
        typed.write(constructor(typed, self));
        typed
    }
    // Source: src/core/SkArenaAlloc.h:151-156
    pub fn make<T: 'static>(&mut self, value: T) -> *mut T {
        unsafe { self.make_with(|_, _| value) }
    }
    // Source: src/core/SkArenaAlloc.h:158-169
    // Explicit Rust entrypoints select the original POD/uninitialized or
    // nontrivial/default-constructor branch, without guessing C++ traits.
    pub fn makeUninit<T: Copy>(&mut self) -> *mut T {
        self.makeBytesAlignedTo(size_of::<T>(), align_of::<T>()) as *mut T
    }
    /// Safety: the nontrivial default constructor must not unwind.
    /// Borrowed data accessed by elements or their destructors must remain valid
    /// until the arena is reset or destroyed.
    pub unsafe fn makeDefault<T: Default>(&mut self) -> *mut T {
        self.make_with(|_, _| T::default())
    }
    // Source: src/core/SkArenaAlloc.h:171-179
    pub fn makeArrayDefault<T: Copy>(&mut self, count: usize) -> *mut T {
        // C++ primitive default initialization leaves the elements uninitialized;
        // return raw typed storage, never a Rust reference to uninitialized values.
        unsafe { self.allocUninitializedArray::<T>(count) }
    }
    /// Nontrivial default-initialization branch of makeArrayDefault.
    /// Safety: T::default() must not unwind; primitive callers use raw storage.
    /// Borrowed data accessed by elements or their destructors must remain valid
    /// until the arena is reset or destroyed.
    pub unsafe fn makeArrayDefaultConstructed<T: Default>(&mut self, count: usize) -> *mut T {
        self.makeArray(count)
    }
    // Source: src/core/SkArenaAlloc.h:181-189
    /// Safety: T::default() must not unwind after arena footer installation.
    /// Borrowed data accessed by elements or their destructors must remain valid
    /// until the arena is reset or destroyed.
    pub unsafe fn makeArray<T: Default>(&mut self, count: usize) -> *mut T {
        let array = self.allocUninitializedArray::<T>(count);
        for i in 0..count {
            array.add(i).write(T::default());
        }
        array
    }
    // Source: src/core/SkArenaAlloc.h:191-198
    /// Safety: initializer must not unwind after footer installation.
    /// Borrowed data accessed by elements or their destructors must remain valid
    /// until the arena is reset or destroyed.
    pub unsafe fn makeInitializedArray<T>(
        &mut self,
        count: usize,
        mut initializer: impl FnMut(usize) -> T,
    ) -> *mut T {
        let array = self.allocUninitializedArray::<T>(count);
        for i in 0..count {
            array.add(i).write(initializer(i));
        }
        array
    }
    // Source: src/core/SkArenaAlloc.h:200-211
    pub fn makeArrayCopy<T: Copy>(&mut self, source: &[T]) -> *mut T {
        unsafe {
            let array = self.allocUninitializedArray::<T>(source.len());
            ptr::copy_nonoverlapping(source.as_ptr(), array, source.len());
            array
        }
    }
    /// Safety: cloning must not unwind after footer installation.
    /// Borrowed data accessed by elements or their destructors must remain valid
    /// until the arena is reset or destroyed.
    pub unsafe fn makeArrayCloned<T: Clone>(&mut self, source: &[T]) -> *mut T {
        let array = self.allocUninitializedArray::<T>(source.len());
        for (i, value) in source.iter().enumerate() {
            array.add(i).write(value.clone());
        }
        array
    }
    // Source: src/core/SkArenaAlloc.h:214-220
    pub fn makeBytesAlignedTo(&mut self, size: usize, align: usize) -> *mut u8 {
        assert_release(
            size <= u32::MAX as usize && align <= u32::MAX as usize && align.is_power_of_two(),
        );
        unsafe {
            let object = self.allocObject(size as u32, align as u32);
            self.fCursor = object.add(size);
            object
        }
    }
    // Source: src/core/SkArenaAlloc.h:229-230
    pub fn cursor(&self) -> *mut u8 {
        self.fCursor
    }
    pub fn end(&self) -> *mut u8 {
        self.fEnd
    }
    // Source: src/core/SkArenaAlloc.h:239-244
    unsafe fn installRaw<T: Copy>(&mut self, value: T) {
        (self.fCursor as *mut T).write_unaligned(value);
        self.fCursor = self.fCursor.add(size_of::<T>());
    }
    // Source: src/core/SkArenaAlloc.h:249-265
    unsafe fn allocObject(&mut self, size: u32, alignment: u32) -> *mut u8 {
        let mask = alignment as usize - 1;
        let mut aligned_offset = (self.fCursor as usize).wrapping_neg() & mask;
        let total_size = (size as usize).wrapping_add(aligned_offset);
        assert_release(total_size >= size as usize);
        if total_size > (self.fEnd as usize).wrapping_sub(self.fCursor as usize) {
            self.ensureSpace(size, alignment);
            aligned_offset = (self.fCursor as usize).wrapping_neg() & mask;
        }
        let object = self.fCursor.add(aligned_offset);
        crate::SkASSERT!(@source b"src/core/SkArenaAlloc.h\0".as_ptr(), 261; @condition b"(reinterpret_cast<uintptr_t>(object) & (alignment - 1)) == 0\0".as_ptr(); (object as usize & mask) == ( 0));
        crate::SkASSERT!(@source b"src/core/SkArenaAlloc.h\0".as_ptr(), 262; @condition b"object + size <= fEnd\0".as_ptr(); object.add(size as usize) <= self.fEnd);
        object
    }
    // Source: src/core/SkArenaAlloc.h:269-313
    unsafe fn allocUninitializedArray<T>(&mut self, count_z: usize) -> *mut T {
        assert_release(count_z <= u32::MAX as usize);
        let count = count_z as u32;
        let element_size = size_of::<T>();
        assert_release(element_size == 0 || count_z <= u32::MAX as usize / element_size);
        let array_size = count_z * element_size;
        let alignment = align_of::<T>() as u32;
        let object;
        if !needs_drop::<T>() {
            object = self.allocObject(array_size as u32, alignment);
            self.fCursor = object.add(array_size);
        } else {
            let overhead = FOOTER_SIZE + size_of::<u32>();
            assert_release(array_size <= u32::MAX as usize - overhead);
            object = self.allocObjectWithFooter((array_size + overhead) as u32, alignment);
            let padding = object.offset_from(self.fCursor) as u32;
            self.fCursor = object.add(array_size);
            self.installRaw(count);
            self.installFooter(drop_array::<T>, padding);
        }
        object as *mut T
    }
}
// Source: src/core/SkArenaAlloc.h:315-320
pub struct SkArenaAlloc {
    fDtorCursor: *mut u8,
    fCursor: *mut u8,
    fEnd: *mut u8,
    fFibonacciProgression: SkFibBlockSizes<{ u32::MAX }>,
    // Native deleted-move adaptation used by actual pinned arena publication.
    // Source: src/core/SkArenaAlloc.h:115-116
    // Existing !Unpin marker enforces the no-move boundary once published pinned.
    _pin: core::marker::PhantomPinned,
}
// Source: src/core/SkArenaAlloc.h:118-118
// Source: src/core/SkArenaAlloc.cpp:34-36
impl Drop for SkArenaAlloc {
    fn drop(&mut self) {
        unsafe {
            Self::RunDtorsOnBlock(self.fDtorCursor);
        }
    }
}
impl SkArenaAlloc {
    // Source: src/core/SkArenaAlloc.cpp:38-43
    unsafe fn installFooter(&mut self, action: FooterAction, padding: u32) {
        assert_release(padding <= u8::MAX as u32);
        self.installRaw(action);
        self.installRaw(padding as u8);
        self.fDtorCursor = self.fCursor;
    }
    // Source: src/core/SkArenaAlloc.cpp:45-50
    unsafe fn SkipPod(footer_end: *mut u8) -> *mut u8 {
        let object_end = footer_end.sub(FOOTER_SIZE + size_of::<u32>());
        let skip = (object_end as *const u32).read_unaligned();
        object_end.sub(skip as usize)
    }
    // Source: src/core/SkArenaAlloc.cpp:52-62
    unsafe fn RunDtorsOnBlock(mut footer_end: *mut u8) {
        while !footer_end.is_null() {
            let action = (footer_end.sub(FOOTER_SIZE) as *const FooterAction).read_unaligned();
            let padding = footer_end.sub(1).read();
            footer_end = action(footer_end).wrapping_sub(padding as usize);
        }
    }
    // Source: src/core/SkArenaAlloc.cpp:64-71
    unsafe fn NextBlock(footer_end: *mut u8) -> *mut u8 {
        let object_end = footer_end.sub(size_of::<*mut u8>() + FOOTER_SIZE);
        let next = (object_end as *const *mut u8).read_unaligned();
        Self::RunDtorsOnBlock(next);
        arena_free(object_end as *mut c_void);
        ptr::null_mut()
    }
    // Source: src/core/SkArenaAlloc.cpp:73-107
    unsafe fn ensureSpace(&mut self, size: u32, alignment: u32) {
        let header_size = FOOTER_SIZE + size_of::<isize>();
        let overhead = header_size + FOOTER_SIZE;
        assert_release(size <= u32::MAX - overhead as u32);
        let mut object_size_and_overhead = size + overhead as u32;
        let alignment_overhead = alignment - 1;
        assert_release(object_size_and_overhead <= u32::MAX - alignment_overhead);
        object_size_and_overhead += alignment_overhead;
        let minimum = self.fFibonacciProgression.nextBlockSize();
        let mut allocation_size = object_size_and_overhead.max(minimum);
        let mask = if allocation_size > 1 << 15 {
            (1 << 12) - 1
        } else {
            16 - 1
        };
        assert_release(allocation_size <= u32::MAX - mask);
        allocation_size = (allocation_size + mask) & !mask;
        let new_block = arena_malloc(allocation_size as usize) as *mut u8;
        if new_block.is_null() {
            std::alloc::handle_alloc_error(std::alloc::Layout::from_size_align_unchecked(
                allocation_size as usize,
                16,
            ));
        }
        let previous_dtor = self.fDtorCursor;
        self.fCursor = new_block;
        self.fDtorCursor = new_block;
        self.fEnd = new_block.add(allocation_size as usize);
        self.installRaw(previous_dtor);
        self.installFooter(Self::NextBlock, 0);
    }
    // Source: src/core/SkArenaAlloc.cpp:109-144
    unsafe fn allocObjectWithFooter(
        &mut self,
        size_including_footer: u32,
        alignment: u32,
    ) -> *mut u8 {
        let mask = alignment as usize - 1;
        loop {
            let needs_skip_footer = self.fCursor != self.fDtorCursor;
            let skip_overhead = if needs_skip_footer {
                FOOTER_SIZE + size_of::<u32>()
            } else {
                0
            };
            let total_size = size_including_footer + skip_overhead as u32;
            if self.fCursor.is_null() {
                self.ensureSpace(total_size, alignment);
                continue;
            }
            debug_assert!(!self.fEnd.is_null());
            let object_start = ((self.fCursor as usize + skip_overhead + mask) & !mask) as *mut u8;
            if (total_size as isize) > (self.fEnd as isize).wrapping_sub(object_start as isize) {
                self.ensureSpace(total_size, alignment);
                continue;
            }
            assert_release(
                (total_size as isize) <= (self.fEnd as isize).wrapping_sub(object_start as isize),
            );
            if needs_skip_footer {
                self.installRaw(self.fCursor.offset_from(self.fDtorCursor) as u32);
                self.installFooter(Self::SkipPod, 0);
            }
            return object_start;
        }
    }
}

unsafe fn drop_object<T>(object_end: *mut u8) -> *mut u8 {
    let object_start = object_end.sub(size_of::<T>() + FOOTER_SIZE);
    ptr::drop_in_place(object_start as *mut T);
    object_start
}
unsafe fn drop_array<T>(footer_end: *mut u8) -> *mut u8 {
    let object_end = footer_end.sub(FOOTER_SIZE + size_of::<u32>());
    let count = (object_end as *const u32).read_unaligned();
    let object_start = object_end.sub(count as usize * size_of::<T>());
    let array = object_start as *mut T;
    for i in 0..count as usize {
        ptr::drop_in_place(array.add(i));
    }
    object_start
}
// Source: src/core/SkArenaAlloc.h:336-339
pub struct SkArenaAllocWithReset {
    arena: MaybeUninit<SkArenaAlloc>,
    fFirstBlock: *mut u8,
    fFirstSize: u32,
    fFirstHeapAllocationSize: u32,
}
impl SkArenaAllocWithReset {
    // Source: src/core/SkArenaAlloc.h:326-327
    pub fn new(first_heap: usize) -> Self {
        unsafe { Self::from_block(ptr::null_mut(), 0, first_heap) }
    }
    // Source: src/core/SkArenaAlloc.h:324-324
    // Source: src/core/SkArenaAlloc.cpp:146-152
    /// Safety: caller-provided storage must stay live and unmoved through drop.
    pub unsafe fn from_block(block: *mut u8, size: usize, first_heap: usize) -> Self {
        Self {
            arena: MaybeUninit::new(SkArenaAlloc::from_block(block, size, first_heap)),
            fFirstBlock: block,
            fFirstSize: size as u32,
            fFirstHeapAllocationSize: first_heap as u32,
        }
    }
    // Source: src/core/SkArenaAlloc.h:330-330
    // Source: src/core/SkArenaAlloc.cpp:154-160
    pub fn reset(&mut self) {
        let first_block = self.fFirstBlock;
        let first_size = self.fFirstSize;
        let first_heap = self.fFirstHeapAllocationSize;
        unsafe {
            self.arena.assume_init_drop();
            self.arena.write(SkArenaAlloc::from_block(
                first_block,
                first_size as usize,
                first_heap as usize,
            ));
        }
    }
    // Source: src/core/SkArenaAlloc.h:333-333
    // Source: src/core/SkArenaAlloc.cpp:162-165
    pub fn isEmpty(&self) -> bool {
        self.cursor().is_null() || self.cursor() == self.fFirstBlock.wrapping_add(FOOTER_SIZE)
    }
}
impl core::ops::Deref for SkArenaAllocWithReset {
    type Target = SkArenaAlloc;
    fn deref(&self) -> &SkArenaAlloc {
        unsafe { self.arena.assume_init_ref() }
    }
}
impl core::ops::DerefMut for SkArenaAllocWithReset {
    fn deref_mut(&mut self) -> &mut SkArenaAlloc {
        unsafe { self.arena.assume_init_mut() }
    }
}
impl Drop for SkArenaAllocWithReset {
    fn drop(&mut self) {
        unsafe {
            self.arena.assume_init_drop();
        }
    }
}
// Source: src/core/SkArenaAlloc.h:346-356
// Adapted: Pin establishes the final address before constructing the arena's
// pointer into embedded storage. The arena field drops before the storage.
pub struct SkSTArenaAlloc<const N: usize> {
    arena: Option<SkArenaAlloc>,
    storage: [MaybeUninit<u8>; N],
    first_heap: usize,
    _pin: core::marker::PhantomPinned,
}
impl<const N: usize> SkSTArenaAlloc<N> {
    pub fn new(first_heap: usize) -> Self {
        Self {
            arena: None,
            storage: [MaybeUninit::uninit(); N],
            first_heap,
            _pin: core::marker::PhantomPinned,
        }
    }
    /// Safety: do not move or replace the returned arena. Its allocations and
    /// destructor chain must not outlive this pinned inline storage.
    pub unsafe fn arena(self: core::pin::Pin<&mut Self>) -> &mut SkArenaAlloc {
        self.pinned_arena().get_unchecked_mut()
    }
    /// Safe pin projection into the inline arena; initialization occurs only
    /// after its backing storage has its final address. No mutable unpinned
    /// arena reference escapes this API.
    pub fn pinned_arena(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut SkArenaAlloc> {
        unsafe {
            let this = self.get_unchecked_mut();
            if this.arena.is_none() {
                this.arena = Some(SkArenaAlloc::from_block(
                    this.storage.as_mut_ptr().cast(),
                    N,
                    this.first_heap,
                ));
            }
            core::pin::Pin::new_unchecked(this.arena.as_mut().unwrap())
        }
    }
}
// Source: src/core/SkArenaAlloc.h:358-369
// Adapted: the same pinning requirement applies to resettable inline storage.
pub struct SkSTArenaAllocWithReset<const N: usize> {
    arena: Option<SkArenaAllocWithReset>,
    storage: [MaybeUninit<u8>; N],
    first_heap: usize,
    _pin: core::marker::PhantomPinned,
}
impl<const N: usize> SkSTArenaAllocWithReset<N> {
    pub fn new(first_heap: usize) -> Self {
        Self {
            arena: None,
            storage: [MaybeUninit::uninit(); N],
            first_heap,
            _pin: core::marker::PhantomPinned,
        }
    }
    /// Safety: do not move or replace the returned arena. Its allocations and
    /// destructor chain must not outlive this pinned inline storage.
    pub unsafe fn arena(self: core::pin::Pin<&mut Self>) -> &mut SkArenaAllocWithReset {
        self.pinned_arena().get_unchecked_mut()
    }
    pub fn pinned_arena(
        self: core::pin::Pin<&mut Self>,
    ) -> core::pin::Pin<&mut SkArenaAllocWithReset> {
        unsafe {
            let this = self.get_unchecked_mut();
            if this.arena.is_none() {
                this.arena = Some(SkArenaAllocWithReset::from_block(
                    this.storage.as_mut_ptr().cast(),
                    N,
                    this.first_heap,
                ));
            }
            core::pin::Pin::new_unchecked(this.arena.as_mut().unwrap())
        }
    }
}
// Source: src/core/SkArenaAlloc.cpp:169-178
pub const SkFibonacci47: [u32; 47] = [
    1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946,
    17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040, 1346269, 2178309, 3524578,
    5702887, 9227465, 14930352, 24157817, 39088169, 63245986, 102334155, 165580141, 267914296,
    433494437, 701408733, 1134903170, 1836311903, 2971215073,
];
