//! Native stable doubly linked blocks, inline head, cached scratch, and packed growth sequence.
//! Construction is staged until structural Pin binds the inline head. Raw allocation
//! pointers remain the caller's responsibility; reset/release invalidate their contents.
fn check_layout(align: usize, padding: usize) {
    let max = SkBlockAllocator::max_block_size(align, padding);
    let bound = max
        .checked_add(padding)
        .and_then(|n| n.checked_add(align - 1));
    assert!(bound.is_some_and(|n| n <= i32::MAX as usize));
    let aligned = max.checked_add(align - 1).map(|n| n & !(align - 1));
    assert!(aligned
        .and_then(|n| n.checked_add(K_MAX_ALLOCATION_SIZE))
        .is_some_and(|n| n <= i32::MAX as usize));
    assert!(SkBlockAllocator::block_overhead(align, padding)
        .checked_add(K_MAX_ALLOCATION_SIZE + 4095)
        .is_some_and(|n| n <= i32::MAX as usize));
}
use core::{marker::PhantomPinned, mem, pin::Pin, ptr};
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
// Source: src/core/SkBlockAllocator.h:60-70
pub const K_MAX_ALLOCATION_SIZE: usize = 1 << 29;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GrowthPolicy {
    kFixed,
    kLinear,
    kFibonacci,
    kExponential,
}
pub const K_GROWTH_POLICY_COUNT: usize = 4;
// Source: src/core/SkBlockAllocator.h:76-97
impl Block {
    pub fn avail(&self, align: usize, padding: usize) -> i32 {
        (self.size - self.cursor(align, padding)).max(0)
    }
    pub fn first_aligned_offset(&self, align: usize, padding: usize) -> i32 {
        self.aligned_offset(DATA_START, align, padding)
    }
    /// Safety: block retains the provenance of its complete allocation, including
    /// bytes after the native header; offset names allocated user storage.
    pub unsafe fn ptr(block: *mut Self, offset: i32) -> *mut u8 {
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 89; @condition b"offset >= kDataStart && offset < fSize\0".as_ptr(); offset >= DATA_START && offset < unsafe { (*block).size });
        unsafe { block.cast::<u8>().add(offset as usize) }
    }
    pub fn metadata(&self) -> i32 {
        self.metadata
    }
    pub fn set_metadata(&mut self, value: i32) {
        self.metadata = value;
    }
}
// Source: src/core/SkBlockAllocator.h:137-138
impl Block {
    fn cursor(&self, align: usize, padding: usize) -> i32 {
        self.aligned_offset(self.cursor, align, padding)
    }
}
// Source: src/core/SkBlockAllocator.h:143-143
impl Block {
    fn is_scratch(&self) -> bool {
        self.cursor < 0
    }
}
// Source: src/core/SkBlockAllocator.h:144-147
impl Block {
    fn mark_as_scratch(&mut self) {
        self.cursor = -1;
    }
}
impl GrowthPolicy {
    pub const kLast: Self = Self::kExponential;
}
// Source: src/core/SkBlockAllocator.h:149-165
#[repr(C)]
pub struct Block {
    #[cfg(sk_debug)]
    sentinel: u32,
    next: *mut Block,
    prev: *mut Block,
    size: i32,
    cursor: i32,
    metadata: i32,
    allocator_metadata: i32,
}
#[repr(C)]
#[cfg_attr(
    any(
        all(target_vendor = "apple", target_arch = "aarch64"),
        target_os = "windows"
    ),
    repr(align(8))
)]
#[cfg_attr(
    not(any(
        all(target_vendor = "apple", target_arch = "aarch64"),
        target_os = "windows"
    )),
    repr(align(16))
)]
struct HeadBlock {
    block: Block,
}
#[cfg(any(
    SK_FORCE_8_BYTE_ALIGNMENT,
    all(target_vendor = "apple", target_arch = "aarch64"),
    target_os = "windows"
))]
pub const ADDRESS_ALIGN: usize = 8;
#[cfg(not(any(
    SK_FORCE_8_BYTE_ALIGNMENT,
    all(target_vendor = "apple", target_arch = "aarch64"),
    target_os = "windows"
)))]
pub const ADDRESS_ALIGN: usize = 16;
pub // Source: src/core/SkBlockAllocator.h:417-417
const DATA_START: i32 = mem::size_of::<Block>() as i32;
// Source: src/core/SkBlockAllocator.h:407-407
const ASSIGNED_MARKER: u32 = 0xBEEFFACE;
// Source: src/core/SkBlockAllocator.h:408-408
const FREED_MARKER: u32 = 0xCAFEBABE;
// Source: src/core/SkBlockAllocator.h:169-174
#[derive(Clone, Copy)]
pub struct ByteRange {
    pub block: *mut Block,
    pub start: i32,
    pub aligned_offset: i32,
    pub end: i32,
}
// Source: src/core/SkBlockAllocator.h:187-188
impl Drop for SkBlockAllocator {
    fn drop(&mut self) {
        if !self.tail.is_null() {
            unsafe {
                let this = self as *mut Self;
                Self::bind_raw(this);
                Self::reset_impl(this);
            }
        }
    }
}
// Source: src/core/SkBlockAllocator.h:230-253
impl SkBlockAllocator {
    pub fn prealloc_size(&self) -> usize {
        mem::size_of::<Self>() + self.head.block.size as usize - Self::base_head_block_size()
    }
    pub fn prealloc_usable_space(&self) -> usize {
        (self.head.block.size - DATA_START) as usize
    }
    pub fn metadata(&self) -> i32 {
        self.head.block.allocator_metadata
    }
    pub fn set_metadata(self: Pin<&mut Self>, value: i32) {
        unsafe {
            (*self.bound()).head.block.allocator_metadata = value;
        }
    }
}
// Source: src/core/SkBlockAllocator.h:285-295
pub const IGNORE_GROWTH_POLICY: u32 = 1;
pub const IGNORE_EXISTING_BYTES: u32 = 2;
pub const NO_RESERVE_FLAGS: u32 = 0;
impl SkBlockAllocator {
    // Source: src/core/SkBlockAllocator.h:438-440
    pub const fn base_head_block_size() -> usize {
        mem::size_of::<Self>() - mem::offset_of!(Self, head)
    }
    fn head_ptr(&self) -> *mut Block {
        ptr::addr_of!(self.head.block).cast_mut()
    }
    // Refresh inline links from the current structural borrow before dereference.
    // Heap allocation addresses are stable and are never replaced by Rust containers.
    unsafe fn bound(self: Pin<&mut Self>) -> *mut Self {
        let this = unsafe { self.get_unchecked_mut() } as *mut Self;
        unsafe {
            Self::bind_raw(this);
        }
        this
    }
    unsafe fn bind_raw(this: *mut Self) {
        unsafe {
            let head = ptr::addr_of_mut!((*this).head.block);
            if (*this).tail.is_null() || ptr::eq((*this).tail, head) {
                (*this).tail = head;
            }
            if !(*head).next.is_null() {
                (*(*head).next).prev = head;
            }
        }
    }
    pub fn init(self: Pin<&mut Self>) {
        unsafe {
            self.bound();
        }
    }
    fn increment(&self) -> i32 {
        (self.growth & 0xffff) as i32
    }
    fn policy(&self) -> GrowthPolicy {
        match (self.growth >> 16) & 3 {
            0 => GrowthPolicy::kFixed,
            1 => GrowthPolicy::kLinear,
            2 => GrowthPolicy::kFibonacci,
            _ => GrowthPolicy::kExponential,
        }
    }
    fn n0(&self) -> i32 {
        ((self.growth >> 18) & 0x7fffff) as i32
    }
    fn n1(&self) -> i32 {
        ((self.growth >> 41) & 0x7fffff) as i32
    }
    fn set_sequence(&mut self, n0: i32, n1: i32) {
        self.growth = (self.growth & 0x3ffff) | ((n0 as u64) << 18) | ((n1 as u64) << 41);
    }
    pub fn new(policy: GrowthPolicy, block_increment_bytes: usize) -> Self {
        // No head pointer is published until init/first Pin mutation.
        unsafe { Self::with_prealloc(policy, block_increment_bytes, 0) }
    }
}
// Source: src/core/SkBlockAllocator.h:315-319
impl SkBlockAllocator {
    pub fn current_block(&self) -> *mut Block {
        if self.tail.is_null() || ptr::eq(self.tail, self.head_ptr()) {
            self.head_ptr()
        } else {
            self.tail
        }
    }
    pub fn head_block(&self) -> *mut Block {
        self.head_ptr()
    }
}
// Source: src/core/SkBlockAllocator.h:447-447
impl SkBlockAllocator {
    fn scratch_block_size(&self) -> i32 {
        if self.head.block.prev.is_null() {
            0
        } else {
            unsafe { (*self.head.block.prev).size }
        }
    }
}
impl SkBlockAllocator {
    /// Safety: this points into a stable live owner whose provenance covers the
    /// inline head and every additional_prealloc byte; no conflicting references
    /// to allocated contents remain across mutation.
    pub unsafe fn allocate_raw(
        this: *mut Self,
        size: usize,
        align: usize,
        padding: usize,
    ) -> ByteRange {
        unsafe {
            Self::bind_raw(this);
            Self::allocate_impl(this, size, align, padding)
        }
    }
    /// Safety: same stable-owner and alias requirements as allocate_raw.
    pub unsafe fn reserve_raw(
        this: *mut Self,
        size: usize,
        align: usize,
        padding: usize,
        flags: u32,
    ) {
        unsafe {
            Self::bind_raw(this);
            Self::reserve_impl(this, size, align, padding, flags);
        }
    }
    /// Safety: block belongs to this; all user contents have already been destroyed.
    pub unsafe fn release_block_raw(this: *mut Self, block: *mut Block) {
        unsafe {
            Self::bind_raw(this);
            Self::release_block_impl(this, block);
        }
    }
    /// Safety: both owners are distinct, pinned, and exclusively borrowed.
    pub unsafe fn steal_heap_blocks_raw(this: *mut Self, other: *mut Self) {
        unsafe {
            Self::bind_raw(this);
            Self::bind_raw(other);
            Self::steal_impl(this, other);
        }
    }
    /// Safety: all initialized values have already been destroyed.
    pub unsafe fn reset_raw(this: *mut Self) {
        unsafe {
            Self::bind_raw(this);
            Self::reset_impl(this);
        }
    }
    /// Safety: header is exclusively accessible; disjoint initialized user
    /// storage retains its complete enclosing allocation provenance.
    pub unsafe fn set_metadata_raw(this: *mut Self, value: i32) {
        unsafe {
            (*this).head.block.allocator_metadata = value;
        }
    }
}
// Source: src/core/SkBlockAllocator.h:449-476
#[repr(C)]
pub struct SkBlockAllocator {
    tail: *mut Block,
    growth: u64,
    head: HeadBlock,
    pinned: PhantomPinned,
}
// Source: src/core/SkBlockAllocator.h:477-513
// Movable constructor shell contains a valid unbound native allocator in raw
// inline bytes. Pin binds its head; all projection methods preserve its location.
#[repr(C)]
#[cfg_attr(
    any(
        all(target_vendor = "apple", target_arch = "aarch64"),
        target_os = "windows"
    ),
    repr(align(8))
)]
#[cfg_attr(
    not(any(
        all(target_vendor = "apple", target_arch = "aarch64"),
        target_os = "windows"
    )),
    repr(align(16))
)]
pub struct SkSBlockAllocator<const N: usize> {
    storage: [mem::MaybeUninit<u8>; N],
    pinned: PhantomPinned,
}
impl<const N: usize> SkSBlockAllocator<N> {
    pub fn new() -> Self {
        Self::with_policy(GrowthPolicy::kFixed)
    }
    pub fn with_policy(policy: GrowthPolicy) -> Self {
        Self::with_increment(policy, N)
    }
    pub fn with_increment(policy: GrowthPolicy, increment: usize) -> Self {
        assert!(N >= mem::size_of::<SkBlockAllocator>());
        let mut owner = Self {
            storage: [mem::MaybeUninit::uninit(); N],
            pinned: PhantomPinned,
        };
        let allocator = unsafe {
            SkBlockAllocator::with_prealloc(
                policy,
                increment,
                N - mem::size_of::<SkBlockAllocator>(),
            )
        };
        unsafe {
            owner
                .storage
                .as_mut_ptr()
                .cast::<SkBlockAllocator>()
                .write(allocator);
        }
        owner
    }
    pub fn allocator(&self) -> &SkBlockAllocator {
        unsafe { &*self.storage.as_ptr().cast::<SkBlockAllocator>() }
    }
    /// Safety: caller preserves structural Pin of this owner; initialized items
    /// in inline tail bytes must be destroyed before reset/Drop.
    pub unsafe fn allocator_raw(self: Pin<&mut Self>) -> *mut SkBlockAllocator {
        unsafe { self.get_unchecked_mut() }
            .storage
            .as_mut_ptr()
            .cast()
    }
}
impl<const N: usize> Drop for SkSBlockAllocator<N> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.storage.as_mut_ptr().cast::<SkBlockAllocator>());
        }
    }
}
impl SkBlockAllocator {
    /// Safety: this retains complete enclosing allocation provenance.
    pub unsafe fn head_block_raw(this: *mut Self) -> *mut Block {
        unsafe { ptr::addr_of_mut!((*this).head.block) }
    }
}
impl SkBlockAllocator {
    pub(crate) fn block_iter<const F: bool>(&self) -> BlockIter<F> {
        unsafe {
            BlockIter::from_block(
                if F {
                    self.head_ptr()
                } else {
                    self.current_block()
                },
                self.head_ptr(),
            )
        }
    }
}

// Source: src/core/SkBlockAllocator.h:520-540
impl SkBlockAllocator {
    pub const fn block_overhead(align: usize, padding: usize) -> usize {
        assert!(align.is_power_of_two());
        let v = match (DATA_START as usize).checked_add(padding) {
            Some(v) => v,
            None => panic!("block padding overflow"),
        };
        let v = match v.checked_add(align - 1) {
            Some(v) => v,
            None => panic!("block alignment overflow"),
        };
        v & !(align - 1)
    }
    pub const fn overhead(align: usize, padding: usize) -> usize {
        let aligned = mem::offset_of!(Self, head) + Self::block_overhead(align, padding);
        if mem::size_of::<Self>() > aligned {
            mem::size_of::<Self>()
        } else {
            aligned
        }
    }
    pub const fn max_block_size(align: usize, padding: usize) -> usize {
        match Self::block_overhead(align, padding).checked_add(K_MAX_ALLOCATION_SIZE) {
            Some(v) => v,
            None => panic!("block size overflow"),
        }
    }
}
// Source: src/core/SkBlockAllocator.h:542-563
impl SkBlockAllocator {
    pub fn reserve(self: Pin<&mut Self>, size: usize, align: usize, padding: usize, flags: u32) {
        unsafe {
            Self::reserve_impl(self.bound(), size, align, padding, flags);
        }
    }
    unsafe fn reserve_impl(this: *mut Self, size: usize, align: usize, padding: usize, flags: u32) {
        unsafe {
            check_layout(align, padding);
            if size > K_MAX_ALLOCATION_SIZE {
                crate::SK_ABORT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 545; "Allocation too large (%zu bytes requested)", size);
            }
            if flags & IGNORE_EXISTING_BYTES != 0
                || (*(*this).tail).avail(align, padding) < size as i32
            {
                let block_size = Self::block_overhead(align, padding) + size;
                let max_size = if flags & IGNORE_GROWTH_POLICY != 0 {
                    block_size
                } else {
                    Self::max_block_size(align, padding)
                };
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 554; @condition b"(size_t) maxSize <= (MaxBlockSize<Align, Padding>())\0".as_ptr(); max_size <= Self::max_block_size(align, padding));
                #[cfg(sk_debug)]
                let old_tail = (*this).tail;
                Self::add_block(this, block_size as i32, max_size as i32);
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 558; @condition b"fTail != oldTail\0".as_ptr(); ((*this).tail) != ( old_tail));
                Self::release_block_impl(this, (*this).tail);
            }
        }
    }
}
// Source: src/core/SkBlockAllocator.h:565-603
impl SkBlockAllocator {
    pub fn allocate(self: Pin<&mut Self>, size: usize, align: usize, padding: usize) -> ByteRange {
        unsafe { Self::allocate_impl(self.bound(), size, align, padding) }
    }
    unsafe fn allocate_impl(
        this: *mut Self,
        size: usize,
        align: usize,
        padding: usize,
    ) -> ByteRange {
        unsafe {
            check_layout(align, padding);
            if size > K_MAX_ALLOCATION_SIZE {
                crate::SK_ABORT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 578; "Allocation too large (%zu bytes requested)", size);
            }
            let mut offset = (*(*this).tail).aligned_offset((*(*this).tail).cursor, align, padding);
            let mut end = offset + size as i32;
            if end > (*(*this).tail).size {
                Self::add_block(
                    this,
                    (size + Self::block_overhead(align, padding)) as i32,
                    Self::max_block_size(align, padding) as i32,
                );
                offset = (*(*this).tail).aligned_offset((*(*this).tail).cursor, align, padding);
                end = offset + size as i32;
            }
            assert!(
                end <= (*(*this).tail).size,
                "native allocation must fit block"
            );
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 592; @condition b"end - offset == iSize\0".as_ptr(); (end - offset) == ( size as i32));
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 593; @condition b"offset - fTail->fCursor >= (int) Padding && offset - fTail->fCursor <= (int) (Padding + Align - 1)\0".as_ptr(); (offset - (*(*this).tail).cursor) >= padding as i32
                    && (offset - (*(*this).tail).cursor) <= (padding + align - 1) as i32);
            debug_assert!(offset >= DATA_START && offset < (*(*this).tail).size);
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 595; @condition b"reinterpret_cast<uintptr_t>(fTail->ptr(offset)) % Align == 0\0".as_ptr(); (
                ((*this).tail.cast::<u8>().add(offset as usize) as usize) % align) == (
                0
            ));
            let start = (*(*this).tail).cursor;
            (*(*this).tail).cursor = end;
            ByteRange {
                block: (*this).tail,
                start,
                aligned_offset: offset,
                end,
            }
        }
    }
}
// Source: src/core/SkBlockAllocator.h:605-624
impl SkBlockAllocator {
    /// Safety: p/start describe a live allocation made by this allocator.
    pub unsafe fn owning_block(
        &self,
        p: *const u8,
        start: i32,
        align: usize,
        padding: usize,
    ) -> *mut Block {
        assert!(align.is_power_of_two());
        if align <= ADDRESS_ALIGN {
            let block = ((p as usize - start as usize - padding) & !(align - 1)) as *mut Block;
            #[cfg(sk_debug)]
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 617; @condition b"block->fSentinel == kAssignedMarker\0".as_ptr(); (unsafe { (*block).sentinel }) == ( ASSIGNED_MARKER));
            block
        } else {
            self.find_owning_block(p)
        }
    }
}
// Source: src/core/SkBlockAllocator.h:626-644
impl Block {
    pub fn aligned_offset(&self, offset: i32, align: usize, padding: usize) -> i32 {
        check_layout(align, padding);
        assert!(
            SkBlockAllocator::max_block_size(align, padding) + padding + align - 1
                <= i32::MAX as usize
        );
        if align <= ADDRESS_ALIGN {
            ((offset as usize + padding + align - 1) & !(align - 1)) as i32
        } else {
            let block = self as *const Self as usize;
            let aligned = (block + offset as usize + padding + align - 1) & !(align - 1);
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 641; @condition b"alignedPtr - blockPtr <= (uintptr_t) std::numeric_limits<int32_t>::max()\0".as_ptr(); aligned - block <= i32::MAX as usize);
            (aligned - block) as i32
        }
    }
}
// Source: src/core/SkBlockAllocator.h:646-672
impl Block {
    pub fn resize(&mut self, start: i32, end: i32, delta: i32) -> bool {
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 647; @condition b"fSentinel == kAssignedMarker\0".as_ptr(); (self.sentinel) == ( ASSIGNED_MARKER));
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 648; @condition b"start >= kDataStart && end <= fSize && start < end\0".as_ptr(); start >= DATA_START && end <= self.size && start < end);
        if delta > K_MAX_ALLOCATION_SIZE as i32 || delta < -(K_MAX_ALLOCATION_SIZE as i32) {
            return false;
        }
        if self.cursor == end {
            let next = end + delta;
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 656; @condition b"nextCursor >= start\0".as_ptr(); next >= start);
            if next <= self.size && next >= start {
                self.cursor = next;
                return true;
            }
        }
        false
    }
}
// Source: src/core/SkBlockAllocator.h:677-689
impl Block {
    pub fn release(&mut self, start: i32, end: i32) -> bool {
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 678; @condition b"fSentinel == kAssignedMarker\0".as_ptr(); (self.sentinel) == ( ASSIGNED_MARKER));
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 679; @condition b"start >= kDataStart && end <= fSize && start < end\0".as_ptr(); start >= DATA_START && end <= self.size && start < end);
        if self.cursor == end {
            self.cursor = start;
            true
        } else {
            false
        }
    }
}
// Source: src/core/SkBlockAllocator.h:691-739
pub struct BlockIter<const FORWARD: bool> {
    head: *mut Block,
    current: *mut Block,
    next: *mut Block,
}
impl<const F: bool> BlockIter<F> {
    unsafe fn from_block(block: *mut Block, head: *mut Block) -> Self {
        let mut iter = Self {
            head,
            current: ptr::null_mut(),
            next: ptr::null_mut(),
        };
        unsafe {
            iter.advance(block);
        }
        iter
    }
    unsafe fn advance(&mut self, mut block: *mut Block) {
        if ptr::eq(block, self.head) {
            block = self.head;
        }
        self.current = block;
        self.next = if block.is_null() {
            ptr::null_mut()
        } else {
            unsafe {
                if F {
                    (*block).next
                } else {
                    (*block).prev
                }
            }
        };
        if ptr::eq(self.next, self.head) {
            self.next = self.head;
        }
        if !F && !self.next.is_null() && unsafe { (*self.next).is_scratch() } {
            self.next = ptr::null_mut();
        }
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.h\0".as_ptr(), 726; @condition b"!fNext || !fNext->isScratch()\0".as_ptr(); self.next.is_null() || !unsafe { (*self.next).is_scratch() });
    }
    /// Safety: blocks cannot be freed except the block just returned; its successor
    /// was cached before return. Allocator itself must remain alive and pinned.
    pub unsafe fn next_raw(&mut self) -> Option<*mut Block> {
        let current = self.current;
        if current.is_null() {
            return None;
        }
        unsafe {
            self.advance(self.next);
        }
        Some(current)
    }
}
// Source: src/core/SkBlockAllocator.h:741-752
impl SkBlockAllocator {
    pub fn blocks(&self) -> BlockIter<true> {
        unsafe { BlockIter::from_block(self.head_ptr(), self.head_ptr()) }
    }
    pub fn rblocks(&self) -> BlockIter<false> {
        unsafe { BlockIter::from_block(self.current_block(), self.head_ptr()) }
    }
}
// Source: src/core/SkBlockAllocator.cpp:17-33
impl SkBlockAllocator {
    /// Safety: after structural Pin, additional_prealloc_bytes writable bytes must
    /// follow Self and belong to this allocation until Drop, with no typed owners.
    pub unsafe fn with_prealloc(
        policy: GrowthPolicy,
        increment: usize,
        additional_prealloc_bytes: usize,
    ) -> Self {
        let units = (increment
            .checked_add(ADDRESS_ALIGN - 1)
            .expect("block increment overflow")
            & !(ADDRESS_ALIGN - 1))
            / ADDRESS_ALIGN;
        let units = units.min(u16::MAX as usize);
        assert!(units >= 1);
        assert!(additional_prealloc_bytes <= K_MAX_ALLOCATION_SIZE);
        let n0 = if policy == GrowthPolicy::kLinear || policy == GrowthPolicy::kExponential {
            1
        } else {
            0
        };
        Self {
            tail: ptr::null_mut(),
            growth: units as u64 | ((policy as u64) << 16) | ((n0 as u64) << 18) | (1u64 << 41),
            head: HeadBlock {
                block: Block::new(
                    ptr::null_mut(),
                    (additional_prealloc_bytes + Self::base_head_block_size()) as i32,
                ),
            },
            pinned: PhantomPinned,
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:35-46
impl Block {
    fn new(prev: *mut Block, allocation_size: i32) -> Self {
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 42; @condition b"allocationSize >= (int) sizeof(Block)\0".as_ptr(); allocation_size >= mem::size_of::<Self>() as i32);
        Self {
            #[cfg(sk_debug)]
            sentinel: ASSIGNED_MARKER,
            next: ptr::null_mut(),
            prev,
            size: allocation_size,
            cursor: DATA_START,
            metadata: 0,
            allocator_metadata: 0,
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:48-53
impl Drop for Block {
    fn drop(&mut self) {
        #[cfg(sk_debug)]
        {
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 51; @condition b"fSentinel == kAssignedMarker\0".as_ptr(); (self.sentinel) == ( ASSIGNED_MARKER));
            self.sentinel = FREED_MARKER;
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:55-63
impl SkBlockAllocator {
    pub fn total_size(&self) -> usize {
        let mut size = mem::offset_of!(Self, head) + self.scratch_block_size() as usize;
        let mut iter = self.blocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            size += unsafe { (*block).size as usize };
        }
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 61; @condition b"size >= this->preallocSize()\0".as_ptr(); size >= self.prealloc_size());
        size
    }
}
// Source: src/core/SkBlockAllocator.cpp:65-75
impl SkBlockAllocator {
    pub fn total_usable_space(&self) -> usize {
        let mut size = self.scratch_block_size() as usize;
        if size > 0 {
            size -= DATA_START as usize;
        }
        let mut iter = self.blocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            size += unsafe { ((*block).size - DATA_START) as usize };
        }
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 73; @condition b"size >= this->preallocUsableSpace()\0".as_ptr(); size >= self.prealloc_usable_space());
        size
    }
}
// Source: src/core/SkBlockAllocator.cpp:77-84
impl SkBlockAllocator {
    pub fn total_space_in_use(&self) -> usize {
        let mut size = 0;
        let mut iter = self.blocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            size += unsafe { ((*block).cursor - DATA_START) as usize };
        }
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 82; @condition b"size <= this->totalUsableSpace()\0".as_ptr(); size <= self.total_usable_space());
        size
    }
}
// Source: src/core/SkBlockAllocator.cpp:86-98
impl SkBlockAllocator {
    pub fn find_owning_block(&self, p: *const u8) -> *mut Block {
        let address = p as usize;
        let mut iter = self.rblocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            unsafe {
                if block as usize + DATA_START as usize <= address
                    && address < block as usize + (*block).size as usize
                {
                    #[cfg(sk_debug)]
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 93; @condition b"b->fSentinel == kAssignedMarker\0".as_ptr(); ((*block).sentinel) == ( ASSIGNED_MARKER));
                    return block;
                }
            }
        }
        ptr::null_mut()
    }
}
// Source: src/core/SkBlockAllocator.cpp:100-152
impl SkBlockAllocator {
    /// Safety: block belongs to this allocator and its user contents may be discarded.
    pub unsafe fn release_block(self: Pin<&mut Self>, block: *mut Block) {
        unsafe {
            Self::release_block_impl(self.bound(), block);
        }
    }
    unsafe fn release_block_impl(this: *mut Self, mut block: *mut Block) {
        unsafe {
            let head = ptr::addr_of_mut!((*this).head.block);
            if ptr::eq(block, head) {
                block = head;
                (*block).cursor = DATA_START;
                (*block).metadata = 0;
            } else {
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 109; @condition b"block->fPrev\0".as_ptr(); !(*block).prev.is_null());
                (*(*block).prev).next = (*block).next;
                if !(*block).next.is_null() {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 112; @condition b"fTail != block\0".as_ptr(); ((*this).tail) != ( block));
                    (*(*block).next).prev = (*block).prev;
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 115; @condition b"fTail == block\0".as_ptr(); ((*this).tail) == ( block));
                    (*this).tail = (*block).prev;
                }
                if (*this).scratch_block_size() < (*block).size {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 121; @condition b"block != fHead.fPrev\0".as_ptr(); (block) != ((*head).prev));
                    if !(*head).prev.is_null() {
                        free_block((*head).prev);
                    }
                    (*block).mark_as_scratch();
                    (*head).prev = block;
                } else {
                    free_block(block);
                }
            }
            let gp = (*this).policy();
            let (mut n0, mut n1) = ((*this).n0(), (*this).n1());
            if n0 > 0 && (n1 > 1 || gp == GrowthPolicy::kFibonacci) {
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 135; @condition b"gp != GrowthPolicy::kFixed\0".as_ptr(); (gp) != (GrowthPolicy::kFixed));
                if gp == GrowthPolicy::kLinear {
                    n1 -= n0;
                } else if gp == GrowthPolicy::kFibonacci {
                    let temp = n1 - n0;
                    n1 -= temp;
                    n0 = temp;
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 144; @condition b"gp == GrowthPolicy::kExponential\0".as_ptr(); (gp) == (GrowthPolicy::kExponential));
                    n1 >>= 1;
                    n0 = n1;
                }
            }
            (*this).set_sequence(n0, n1);
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 151; @condition b"fN1 >= 1 && fN0 >= 0\0".as_ptr(); n1 >= 1 && n0 >= 0);
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:154-167
impl SkBlockAllocator {
    pub fn steal_heap_blocks(self: Pin<&mut Self>, other: Pin<&mut Self>) {
        unsafe {
            Self::steal_impl(self.bound(), other.bound());
        }
    }
    unsafe fn steal_impl(this: *mut Self, other: *mut Self) {
        unsafe {
            let to_steal = (*other).head.block.next;
            if !to_steal.is_null() {
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 159; @condition b"other->fTail != &other->fHead\0".as_ptr(); ((*other).tail) != ( ptr::addr_of_mut!((*other).head.block)));
                (*to_steal).prev = (*this).tail;
                (*(*this).tail).next = to_steal;
                (*this).tail = (*other).tail;
                (*other).tail = ptr::addr_of_mut!((*other).head.block);
                (*other).head.block.next = ptr::null_mut();
            }
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:169-192
impl SkBlockAllocator {
    pub fn reset(self: Pin<&mut Self>) {
        unsafe {
            Self::reset_impl(self.bound());
        }
    }
    unsafe fn reset_impl(this: *mut Self) {
        unsafe {
            let head = ptr::addr_of_mut!((*this).head.block);
            let mut iter = BlockIter::<false>::from_block((*this).tail, head);
            while let Some(block) = iter.next_raw() {
                if ptr::eq(block, head) {
                    (*this).tail = head;
                    (*head).next = ptr::null_mut();
                    (*head).cursor = DATA_START;
                    (*head).metadata = 0;
                    (*head).allocator_metadata = 0;
                    Self::reset_scratch_impl(this);
                } else {
                    free_block(block);
                }
            }
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 186; @condition b"fTail == &fHead && fHead.fNext == nullptr && fHead.fPrev == nullptr && fHead.metadata() == 0 && fHead.fCursor == kDataStart\0".as_ptr(); ptr::eq((*this).tail, head)
                    && (*head).next.is_null()
                    && (*head).prev.is_null()
                    && (*head).metadata == 0
                    && (*head).cursor == DATA_START);
            let gp = (*this).policy();
            (*this).set_sequence(
                if gp == GrowthPolicy::kLinear || gp == GrowthPolicy::kExponential {
                    1
                } else {
                    0
                },
                1,
            );
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:194-199
impl SkBlockAllocator {
    pub fn reset_scratch_space(self: Pin<&mut Self>) {
        unsafe {
            Self::reset_scratch_impl(self.bound());
        }
    }
    unsafe fn reset_scratch_impl(this: *mut Self) {
        unsafe {
            let head = ptr::addr_of_mut!((*this).head.block);
            if !(*head).prev.is_null() {
                free_block((*head).prev);
                (*head).prev = ptr::null_mut();
            }
        }
    }
}
// Source: src/core/SkBlockAllocator.cpp:201-263
impl SkBlockAllocator {
    unsafe fn add_block(this: *mut Self, min_size: i32, max_size: i32) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 202; @condition b"minSize > (int) sizeof(Block) && minSize <= maxSize\0".as_ptr(); min_size > mem::size_of::<Block>() as i32 && min_size <= max_size);
            const MAX_N: i32 = (1 << 23) - 1;
            let align_alloc = |size: i32| {
                let mask = if size > (1 << 15) {
                    (1 << 12) - 1
                } else {
                    ADDRESS_ALIGN as i32 - 1
                };
                (size + mask) & !mask
            };
            let alloc_size;
            let mut memory = ptr::null_mut::<Block>();
            if (*this).scratch_block_size() >= min_size {
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 220; @condition b"fHead.fPrev->isScratch()\0".as_ptr(); (*(*this).head.block.prev).is_scratch());
                alloc_size = (*(*this).head.block.prev).size;
                memory = (*this).head.block.prev;
                (*this).head.block.prev = ptr::null_mut();
                // Placement-new replaces a trivially-owned block header. Do not run its destructor
                // before replacement, just as the original placement-new path does not.
            } else if min_size < max_size {
                let gp = (*this).policy();
                let next_n1 = (*this).n0() + (*this).n1();
                let next_n0 = if gp == GrowthPolicy::kFixed || gp == GrowthPolicy::kLinear {
                    (*this).n0()
                } else if gp == GrowthPolicy::kFibonacci {
                    (*this).n1()
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 234; @condition b"gp == GrowthPolicy::kExponential\0".as_ptr(); (gp) == (GrowthPolicy::kExponential));
                    next_n1
                };
                (*this).set_sequence(next_n0.min(MAX_N), next_n1.min(MAX_N));
                let increment = (*this).increment() * ADDRESS_ALIGN as i32;
                alloc_size = if max_size / increment < next_n1 {
                    max_size
                } else {
                    align_alloc(min_size.max(increment * next_n1)).min(max_size)
                };
            } else {
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 252; @condition b"minSize == maxSize\0".as_ptr(); (min_size) == (max_size));
                alloc_size = align_alloc(min_size);
            }
            if memory.is_null() {
                let layout = block_layout(alloc_size);
                memory = alloc(layout).cast();
                if memory.is_null() {
                    handle_alloc_error(layout);
                }
            }
            memory.write(Block::new((*this).tail, alloc_size));
            (*(*this).tail).next = memory;
            (*this).tail = memory;
        }
    }
}
// Allocation uses Rust's system allocator with the native max alignment and exact block size.
// ASAN poison/unpoison calls are inactive in the selected production CPU configuration;
// this translation does not claim instrumentation support.
fn block_layout(size: i32) -> Layout {
    Layout::from_size_align(size as usize, ADDRESS_ALIGN).expect("block allocation layout")
}
unsafe fn free_block(block: *mut Block) {
    let size = unsafe { (*block).size };
    unsafe {
        ptr::drop_in_place(block);
        dealloc(block.cast(), block_layout(size));
    }
}
// Source: src/core/SkBlockAllocator.cpp:266-301
#[cfg(sk_debug)]
impl SkBlockAllocator {
    pub fn validate(&self) {
        let mut blocks = Vec::new();
        let mut prev: *mut Block = ptr::null_mut();
        let mut iter = self.blocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            unsafe {
                blocks.push(block);
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 272; @condition b"kAssignedMarker == block->fSentinel\0".as_ptr(); ((*block).sentinel) == ( ASSIGNED_MARKER));
                if ptr::eq(block, self.head_ptr()) {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 276; @condition b"!prev && (!fHead.fPrev || fHead.fPrev->isScratch())\0".as_ptr(); prev.is_null()
                            && (self.head.block.prev.is_null()
                                || (*self.head.block.prev).is_scratch()));
                } else {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 278; @condition b"prev == block->fPrev\0".as_ptr(); (prev) == ((*block).prev));
                }
                if !prev.is_null() {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 281; @condition b"prev->fNext == block\0".as_ptr(); ((*prev).next) == (block));
                }
                crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 284; @condition b"block->fSize >= (int) sizeof(Block)\0".as_ptr(); (*block).size >= mem::size_of::<Block>() as i32);
                {
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 285; @condition b"block->fCursor >= kDataStart\0".as_ptr(); (*block).cursor >= DATA_START);
                    crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 286; @condition b"block->fCursor <= block->fSize\0".as_ptr(); (*block).cursor <= (*block).size);
                };
                prev = block;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 290; @condition b"prev == fTail\0".as_ptr(); (prev) == ( self.current_block()));
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 291; @condition b"!blocks.empty()\0".as_ptr(); !blocks.is_empty());
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 292; @condition b"blocks[0] == &fHead\0".as_ptr(); (blocks[0]) == ( self.head_ptr()));
        let mut j = blocks.len();
        let mut iter = self.rblocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 297; @condition b"b == blocks[j - 1]\0".as_ptr(); (block) == ( blocks[j - 1]));
            j -= 1;
        }
        crate::SkASSERT!(@source b"src/core/SkBlockAllocator.cpp\0".as_ptr(), 300; @condition b"j == 0\0".as_ptr(); (j) == (0));
    }
}
