//! Stable inline-plus-linked-block owner. All content mutations require structural Pin;
//! no API exposes a movable allocator or mutable projection of its embedded storage.
use crate::src::core::SkBlockAllocator::{
    Block, BlockIter, GrowthPolicy, SkBlockAllocator, IGNORE_EXISTING_BYTES, IGNORE_GROWTH_POLICY,
};
use core::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
    pin::Pin,
    ptr,
};
// Source: src/core/SkTBlockList.h:21-27
/// Safety: callbacks resolve valid items, visit each slot once in the specified
/// direction, and return monotonically progressing indices within each block.
/// Rust's zero-sized policy type represents the native compile-time function arguments.
pub unsafe trait BlockIndexOps<T, const FORWARD: bool> {
    unsafe fn start(block: *const Block) -> i32;
    unsafe fn end(block: *const Block) -> i32;
    unsafe fn next(block: *const Block, index: i32) -> i32;
    unsafe fn resolve(block: *mut Block, index: i32) -> *mut T;
}
pub struct TBlockListIndex<T>(PhantomData<T>);
// Source: src/core/SkTBlockList.h:62-81
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn new() -> Self {
        Self::with_policy(GrowthPolicy::kFixed)
    }
    pub fn with_policy(policy: GrowthPolicy) -> Self {
        Self::with_increment(N as i32, policy)
    }
    pub fn with_increment(items_per_block: i32, policy: GrowthPolicy) -> Self {
        assert!(Self::ITEM_SIZE > 0 && items_per_block >= 0);
        let size = SkBlockAllocator::overhead(Self::ITEM_ALIGN, 0)
            .checked_add(N.checked_mul(Self::ITEM_SIZE).expect("inline item bytes"))
            .expect("inline storage size");
        assert!(size <= mem::size_of::<Self>());
        let increment = SkBlockAllocator::block_overhead(Self::ITEM_ALIGN, 0)
            .checked_add(
                Self::ITEM_SIZE
                    .checked_mul(items_per_block as usize)
                    .expect("block item bytes"),
            )
            .expect("block increment");
        let allocator = unsafe {
            SkBlockAllocator::with_prealloc(
                policy,
                increment,
                size - mem::size_of::<SkBlockAllocator>(),
            )
        };
        Self {
            allocator,
            inline: [const { MaybeUninit::uninit() }; N],
        }
    }
}
// Source: src/core/SkTBlockList.h:83-83
impl<T, const N: usize> Drop for SkTBlockList<T, N> {
    fn drop(&mut self) {
        unsafe {
            Self::reset_impl(self);
        }
    }
}
// Source: src/core/SkTBlockList.h:90-98
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn push_back(self: Pin<&mut Self>, value: T) -> &mut T {
        let this = self.raw();
        let item = unsafe { Self::push_item(this) };
        unsafe {
            item.write(value);
            &mut *item
        }
    }
}
impl<T: Default, const N: usize> SkTBlockList<T, N> {
    pub fn push_default(self: Pin<&mut Self>) -> &mut T {
        self.emplace_back(T::default)
    }
}
impl<T: Clone, const N: usize> SkTBlockList<T, N> {
    pub fn push_clone(self: Pin<&mut Self>, value: &T) -> &mut T {
        self.emplace_back(|| value.clone())
    }
}
// Source: src/core/SkTBlockList.h:100-103
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn emplace_back(self: Pin<&mut Self>, constructor: impl FnOnce() -> T) -> &mut T {
        let this = self.raw();
        let item = unsafe { Self::push_item(this) };
        // If a Rust constructor unwinds, roll back the unconstructed reservation
        // before exposing the owner again. Native builds do not use exceptions.
        let mut rollback = PushRollback::<T, N> {
            owner: this,
            item,
            active: true,
        };
        let value = constructor();
        unsafe {
            item.write(value);
        }
        rollback.active = false;
        unsafe { &mut *item }
    }
}
// Source: src/core/SkTBlockList.h:117-125
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn reserve(self: Pin<&mut Self>, n: i32) {
        assert!(n >= 0);
        let this = self.raw();
        let allocator = unsafe { Self::allocator_raw(this) };
        let avail = unsafe { (*(*allocator).current_block()).avail(Self::ITEM_ALIGN, 0) }
            / Self::ITEM_SIZE as i32;
        if n > avail {
            unsafe {
                SkBlockAllocator::reserve_raw(
                    allocator,
                    (n - avail) as usize * Self::ITEM_SIZE,
                    Self::ITEM_ALIGN,
                    0,
                    IGNORE_EXISTING_BYTES,
                );
            }
        }
    }
}
// Source: src/core/SkTBlockList.h:130-148
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn pop_back(self: Pin<&mut Self>) {
        let this = self.raw();
        {
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 131; @condition b"this->count() > 0\0".as_ptr(); unsafe { (*this).count() } > 0);
            #[cfg(not(sk_debug))]
            assert!(unsafe { (*this).count() } > 0);
        };
        let block = unsafe { Self::normalize(this, (*this).allocator.current_block()) };
        let index = unsafe { (*block).metadata() };
        let item = unsafe { Block::ptr(block, index).cast::<T>() };
        // Drop first, then release reservation and update global count, as native.
        let guard = PopRelease::<T, N> {
            owner: this,
            block,
            index,
        };
        unsafe {
            ptr::drop_in_place(item);
        }
        drop(guard);
    }
    unsafe fn release_popped(this: *mut Self, block: *mut Block, index: i32) {
        unsafe {
            let allocator = Self::allocator_raw(this);
            if index == (*block).first_aligned_offset(Self::ITEM_ALIGN, 0) {
                SkBlockAllocator::release_block_raw(allocator, block);
            } else {
                let result = (*block).release(index, index + Self::ITEM_SIZE as i32);
                crate::SkAssertResult!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 143; @condition b"block->release(releaseIndex, releaseIndex + sizeof(T))\0".as_ptr(); result);
                (*block).set_metadata(index - Self::ITEM_SIZE as i32);
            }
            SkBlockAllocator::set_metadata_raw(allocator, (*allocator).metadata() - 1);
        }
    }
}
// Source: src/core/SkTBlockList.h:153-162
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn reset(self: Pin<&mut Self>) {
        unsafe {
            Self::reset_impl(self.raw());
        }
    }
    unsafe fn reset_impl(this: *mut Self) {
        if mem::needs_drop::<T>() {
            let iter = unsafe { Self::raw_iter::<false>(this) };
            let mut guard = ResetDrop::<T, N> {
                owner: this,
                iter,
                active: true,
            };
            while let Some(item) = unsafe { guard.iter.next_raw() } {
                unsafe {
                    ptr::drop_in_place(item);
                }
            }
            guard.active = false;
        }
        unsafe {
            SkBlockAllocator::reset_raw(Self::allocator_raw(this));
        }
    }
}
// Source: src/core/SkTBlockList.h:167-180
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn count(&self) -> i32 {
        #[cfg(sk_debug)]
        {
            let mut count = 0;
            let mut iter = self.allocator.blocks();
            while let Some(block) = unsafe { iter.next_raw() } {
                unsafe {
                    if (*block).metadata() == 0 {
                        continue;
                    }
                    count += (Self::ITEM_SIZE as i32 + (*block).metadata()
                        - (*block).first_aligned_offset(Self::ITEM_ALIGN, 0))
                        / Self::ITEM_SIZE as i32;
                }
            }
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 177; @condition b"count == fAllocator->metadata()\0".as_ptr(); (count) == (self.allocator.metadata()));
        }
        self.allocator.metadata()
    }
}
// Source: src/core/SkTBlockList.h:185-185
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn empty(&self) -> bool {
        self.count() == 0
    }
}
// Source: src/core/SkTBlockList.h:190-199
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn front(&self) -> &T {
        assert!(N >= 1);
        {
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 193; @condition b"this->count() > 0 && fAllocator->headBlock()->metadata() > 0\0".as_ptr(); self.count() > 0 && unsafe { (*self.allocator.head_block()).metadata() } > 0);
            #[cfg(not(sk_debug))]
            assert!(self.count() > 0 && unsafe { (*self.allocator.head_block()).metadata() } > 0);
        };
        let block = unsafe { Self::head_raw(self) };
        unsafe {
            &*Block::ptr(block, (*block).first_aligned_offset(Self::ITEM_ALIGN, 0)).cast::<T>()
        }
    }
    pub fn front_mut(self: Pin<&mut Self>) -> &mut T {
        assert!(N >= 1);
        let this = self.raw();
        {
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 197; @condition b"this->count() > 0 && fAllocator->headBlock()->metadata() > 0\0".as_ptr(); unsafe { (*this).count() } > 0);
            #[cfg(not(sk_debug))]
            assert!(unsafe { (*this).count() } > 0);
        };
        let block = unsafe { Self::head_raw(this) };
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 197; @condition b"this->count() > 0 && fAllocator->headBlock()->metadata() > 0\0".as_ptr(); unsafe { (*block).metadata() } > 0);
        unsafe {
            &mut *Block::ptr(block, (*block).first_aligned_offset(Self::ITEM_ALIGN, 0)).cast::<T>()
        }
    }
}
// Source: src/core/SkTBlockList.h:204-211
impl<T, const N: usize> SkTBlockList<T, N> {
    pub fn back(&self) -> &T {
        {
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 205; @condition b"this->count() > 0 && fAllocator->currentBlock()->metadata() > 0\0".as_ptr(); self.count() > 0);
            #[cfg(not(sk_debug))]
            assert!(self.count() > 0);
        };
        let block = unsafe { Self::normalize(self, self.allocator.current_block()) };
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 205; @condition b"this->count() > 0 && fAllocator->currentBlock()->metadata() > 0\0".as_ptr(); unsafe { (*block).metadata() } > 0);
        unsafe { &*Block::ptr(block, (*block).metadata()).cast::<T>() }
    }
    pub fn back_mut(self: Pin<&mut Self>) -> &mut T {
        let this = self.raw();
        {
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 209; @condition b"this->count() > 0 && fAllocator->currentBlock()->metadata() > 0\0".as_ptr(); unsafe { (*this).count() } > 0);
            #[cfg(not(sk_debug))]
            assert!(unsafe { (*this).count() } > 0);
        };
        let block = unsafe { Self::normalize(this, (*this).allocator.current_block()) };
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 209; @condition b"this->count() > 0 && fAllocator->currentBlock()->metadata() > 0\0".as_ptr(); unsafe { (*block).metadata() } > 0);
        unsafe { &mut *Block::ptr(block, (*block).metadata()).cast::<T>() }
    }
}
// Source: src/core/SkTBlockList.h:217-239
impl<T, const N: usize> SkTBlockList<T, N> {
    unsafe fn item_ptr_raw(this: *mut Self, mut i: i32) -> *mut T {
        let this_ref = unsafe { &(*this).allocator };
        {
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 218; @condition b"i >= 0 && i < this->count()\0".as_ptr(); i >= 0 && i < this_ref.metadata());
            #[cfg(not(sk_debug))]
            assert!(i >= 0 && i < this_ref.metadata());
        };
        let mut iter = this_ref.blocks();
        while let Some(block) = unsafe { iter.next_raw() } {
            unsafe {
                if (*block).metadata() == 0 {
                    continue;
                }
                let block = Self::normalize(this, block);
                let start = (*block).first_aligned_offset(Self::ITEM_ALIGN, 0);
                let end = (*block).metadata() + Self::ITEM_SIZE as i32;
                let index = start + i * Self::ITEM_SIZE as i32;
                if index < end {
                    return Block::ptr(block, index).cast();
                } else {
                    i -= (end - start) / Self::ITEM_SIZE as i32;
                }
            }
        }
        crate::SkUNREACHABLE!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 235;)
    }
    pub fn item(&self, i: i32) -> &T {
        unsafe { &*Self::item_ptr_raw((self as *const Self).cast_mut(), i) }
    }
    pub fn item_mut(self: Pin<&mut Self>, i: i32) -> &mut T {
        let this = self.raw();
        let item = unsafe { Self::item_ptr_raw(this, i) };
        unsafe { &mut *item }
    }
}
// Source: src/core/SkTBlockList.h:250-255
impl<T, const N: usize> SkTBlockList<T, N> {
    unsafe fn get_item(block: *mut Block, index: i32) -> *mut T {
        unsafe { Block::ptr(block, index).cast() }
    }
}
// Source: src/core/SkTBlockList.h:256-267
unsafe impl<T, const F: bool> BlockIndexOps<T, F> for TBlockListIndex<T> {
    unsafe fn start(block: *const Block) -> i32 {
        unsafe {
            if F {
                (*block).first_aligned_offset(mem::align_of::<T>(), 0)
            } else {
                (*block).metadata()
            }
        }
    }
    unsafe fn end(block: *const Block) -> i32 {
        unsafe {
            if F {
                (*block).metadata()
            } else {
                (*block).first_aligned_offset(mem::align_of::<T>(), 0)
            }
        }
    }
    unsafe fn next(_block: *const Block, index: i32) -> i32 {
        if F {
            index + mem::size_of::<T>() as i32
        } else {
            index - mem::size_of::<T>() as i32
        }
    }
    unsafe fn resolve(block: *mut Block, index: i32) -> *mut T {
        unsafe { Block::ptr(block, index).cast() }
    }
}
impl<T, const N: usize> SkTBlockList<T, N> {
    // Alias-safe native emplace_back(back()) form: derive the source after the
    // owner borrow, then allocate only metadata/disjoint tail bytes before copy.
    pub fn emplace_from_back(self: Pin<&mut Self>, constructor: impl FnOnce(&T) -> T) -> &mut T {
        let this = self.raw();
        assert!(unsafe { (*this).count() } > 0);
        let block = unsafe { Self::normalize(this, (*this).allocator.current_block()) };
        let source = unsafe { Block::ptr(block, (*block).metadata()).cast::<T>() };
        let item = unsafe { Self::push_item(this) };
        let mut rollback = PushRollback::<T, N> {
            owner: this,
            item,
            active: true,
        };
        let value = constructor(unsafe { &*source });
        unsafe {
            item.write(value);
        }
        rollback.active = false;
        unsafe { &mut *item }
    }
}
// Source: src/core/SkTBlockList.h:269-277
impl<T, const N: usize> SkTBlockList<T, N> {
    unsafe fn push_item(this: *mut Self) -> *mut T {
        unsafe {
            let allocator = Self::allocator_raw(this);
            let br =
                SkBlockAllocator::allocate_raw(allocator, Self::ITEM_SIZE, Self::ITEM_ALIGN, 0);
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 272; @condition b"br.fStart == br.fAlignedOffset || br.fAlignedOffset == First(fAllocator->currentBlock())\0".as_ptr(); br.start == br.aligned_offset
                    || br.aligned_offset == (*br.block).first_aligned_offset(Self::ITEM_ALIGN, 0));
            (*br.block).set_metadata(br.aligned_offset);
            SkBlockAllocator::set_metadata_raw(allocator, (*allocator).metadata() + 1);
            Self::get_item(br.block, br.aligned_offset)
        }
    }
}
// Rust constructor/drop unwind guards keep the initialized prefix valid without
// adding a placeholder T or invoking a destructor on uninitialized bytes.
struct PushRollback<T, const N: usize> {
    owner: *mut SkTBlockList<T, N>,
    item: *mut T,
    active: bool,
}
impl<T, const N: usize> Drop for PushRollback<T, N> {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                let a = SkTBlockList::<T, N>::allocator_raw(self.owner);
                let b = SkTBlockList::<T, N>::normalize(
                    self.owner,
                    (*a).find_owning_block(self.item.cast()),
                );
                let index = self.item as usize - b as usize;
                SkTBlockList::<T, N>::release_popped(self.owner, b, index as i32);
            }
        }
    }
}
struct PopRelease<T, const N: usize> {
    owner: *mut SkTBlockList<T, N>,
    block: *mut Block,
    index: i32,
}
impl<T, const N: usize> Drop for PopRelease<T, N> {
    fn drop(&mut self) {
        unsafe {
            SkTBlockList::<T, N>::release_popped(self.owner, self.block, self.index);
        }
    }
}
// Source: src/core/SkTBlockList.h:279-284
// Generic const expressions cannot name SkSBlockAllocator<Overhead<T>+N*sizeof(T)>.
// The same allocator-prefix + inline bytes live in this one object. Native requested
// preallocation size ignores any extra Rust trailing alignment padding.
#[repr(C)]
pub struct SkTBlockList<T, const N: usize = 1> {
    allocator: SkBlockAllocator,
    inline: [MaybeUninit<T>; N],
}
impl<T, const N: usize> SkTBlockList<T, N> {
    const ITEM_SIZE: usize = mem::size_of::<T>();
    const ITEM_ALIGN: usize = mem::align_of::<T>();
    fn raw(self: Pin<&mut Self>) -> *mut Self {
        unsafe { self.get_unchecked_mut() }
    }
    unsafe fn allocator_raw(this: *mut Self) -> *mut SkBlockAllocator {
        unsafe { ptr::addr_of_mut!((*this).allocator) }
    }
    unsafe fn head_raw(this: *const Self) -> *mut Block {
        unsafe { SkBlockAllocator::head_block_raw(ptr::addr_of!((*this).allocator).cast_mut()) }
    }
    unsafe fn normalize(this: *const Self, block: *mut Block) -> *mut Block {
        let head = unsafe { Self::head_raw(this) };
        if ptr::eq(block, head) {
            head
        } else {
            block
        }
    }
    pub fn allocator(&self) -> &SkBlockAllocator {
        &self.allocator
    }
}
// Source: src/core/SkTBlockList.h:297-302
impl<T, const N: usize> SkTBlockList<T, N> {
    unsafe fn raw_iter<const F: bool>(this: *mut Self) -> BlockIndexIterator<T, F> {
        let blocks = unsafe { (*this).allocator.block_iter::<F>() };
        BlockIndexIterator::new(blocks, unsafe { Self::head_raw(this) })
    }
    fn items_raw(&self) -> BlockIndexIterator<T, true> {
        unsafe { Self::raw_iter((self as *const Self).cast_mut()) }
    }
    fn ritems_raw(&self) -> BlockIndexIterator<T, false> {
        unsafe { Self::raw_iter((self as *const Self).cast_mut()) }
    }
    pub fn items(&self) -> Items<'_, T, true> {
        Items {
            raw: self.items_raw(),
            lifetime: PhantomData,
        }
    }
    pub fn ritems(&self) -> Items<'_, T, false> {
        Items {
            raw: self.ritems_raw(),
            lifetime: PhantomData,
        }
    }
    pub fn items_mut(self: Pin<&mut Self>) -> ItemsMut<'_, T, true> {
        let this = self.raw();
        ItemsMut {
            raw: unsafe { Self::raw_iter(this) },
            lifetime: PhantomData,
        }
    }
    pub fn ritems_mut(self: Pin<&mut Self>) -> ItemsMut<'_, T, false> {
        let this = self.raw();
        ItemsMut {
            raw: unsafe { Self::raw_iter(this) },
            lifetime: PhantomData,
        }
    }
}

struct ResetDrop<T, const N: usize> {
    owner: *mut SkTBlockList<T, N>,
    iter: BlockIndexIterator<T, false>,
    active: bool,
}
impl<T, const N: usize> Drop for ResetDrop<T, N> {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                while let Some(item) = self.iter.next_raw() {
                    ptr::drop_in_place(item);
                }
                SkBlockAllocator::reset_raw(SkTBlockList::<T, N>::allocator_raw(self.owner));
            }
        }
    }
}
// Source: src/core/SkTBlockList.h:305-376
impl<T: Clone, const N: usize> SkTBlockList<T, N> {
    // The source's single-item fast path always copy-constructs. For the general
    // nontrivial path, Rust has no implicit user-defined C++ move constructor:
    // callers supply that type's real move operation, leaving a valid moved-from
    // value for the source-mandated destructor. clone is the native fallback for
    // SkRasterClip, which declares a copy constructor but no move constructor.
    pub fn concat<const M: usize>(self: Pin<&mut Self>, other: Pin<&mut SkTBlockList<T, M>>) {
        self.concat_with_move(other, |value| value.clone());
    }
    pub fn concat_with_move<const M: usize>(
        self: Pin<&mut Self>,
        other: Pin<&mut SkTBlockList<T, M>>,
        move_construct: impl FnMut(&mut T) -> T,
    ) {
        unsafe {
            Self::concat_impl::<M, false>(self.raw(), other.raw(), move_construct);
        }
    }
    unsafe fn concat_impl<const M: usize, const TRIVIAL: bool>(
        this: *mut Self,
        other: *mut SkTBlockList<T, M>,
        mut move_construct: impl FnMut(&mut T) -> T,
    ) {
        unsafe {
            if (*other).empty() {
                return;
            } else if (*other).count() == 1 {
                Pin::new_unchecked(&mut *this).emplace_back(|| (*other).back().clone());
                Pin::new_unchecked(&mut *other).pop_back();
                return;
            }
            let mut head_count = 0;
            let head = SkTBlockList::<T, M>::head_raw(other);
            let allocator = Self::allocator_raw(this);
            let other_allocator = SkTBlockList::<T, M>::allocator_raw(other);
            #[cfg(sk_debug)]
            let old_count = (*this).count();
            if (*head).metadata() > 0 {
                let head_start = (*head).first_aligned_offset(Self::ITEM_ALIGN, 0);
                let head_end = (*head).metadata() + Self::ITEM_SIZE as i32;
                head_count = (head_end - head_start) / Self::ITEM_SIZE as i32;
                let avail = (*(*allocator).current_block()).avail(Self::ITEM_ALIGN, 0)
                    / Self::ITEM_SIZE as i32;
                if head_count > avail {
                    SkBlockAllocator::reserve_raw(
                        allocator,
                        (head_count - avail) as usize * Self::ITEM_SIZE,
                        Self::ITEM_ALIGN,
                        0,
                        IGNORE_EXISTING_BYTES | IGNORE_GROWTH_POLICY,
                    );
                }
                if TRIVIAL {
                    crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 338; @condition b"std::is_trivially_destructible<T>::value\0".as_ptr(); !mem::needs_drop::<T>());
                    let copy = |start: i32, n: i32| {
                        let target = SkBlockAllocator::allocate_raw(
                            allocator,
                            n as usize * Self::ITEM_SIZE,
                            Self::ITEM_ALIGN,
                            0,
                        );
                        ptr::copy_nonoverlapping(
                            Block::ptr(head, start),
                            Block::ptr(target.block, target.aligned_offset),
                            n as usize * Self::ITEM_SIZE,
                        );
                        (*target.block)
                            .set_metadata(target.aligned_offset + (n - 1) * Self::ITEM_SIZE as i32);
                    };
                    if avail > 0 {
                        copy(head_start, head_count.min(avail));
                    }
                    if head_count > avail {
                        copy(
                            head_start + avail * Self::ITEM_SIZE as i32,
                            head_count - avail,
                        );
                    }
                    SkBlockAllocator::set_metadata_raw(
                        allocator,
                        (*allocator).metadata() + head_count,
                    );
                } else {
                    let mut rollback = ConcatHeadDrop::<T, M> {
                        owner: other,
                        head,
                        next: head_start,
                        end: head_end,
                        count: head_count,
                        active: true,
                    };
                    let mut i = head_start;
                    while i < head_end {
                        let source = Block::ptr(head, i).cast::<T>();
                        Pin::new_unchecked(&mut *this)
                            .emplace_back(|| move_construct(&mut *source));
                        rollback.next = i + Self::ITEM_SIZE as i32;
                        ptr::drop_in_place(source);
                        i += Self::ITEM_SIZE as i32;
                    }
                    rollback.active = false;
                }
                SkBlockAllocator::release_block_raw(other_allocator, head);
            }
            crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 370; @condition b"other.fAllocator->headBlock()->metadata() == 0 && fAllocator->metadata() == oldCount + headItemCount\0".as_ptr(); (*head).metadata() == 0 && (*allocator).metadata() == old_count + head_count);
            SkBlockAllocator::steal_heap_blocks_raw(allocator, other_allocator);
            SkBlockAllocator::set_metadata_raw(
                allocator,
                (*allocator).metadata() + ((*other_allocator).metadata() - head_count),
            );
            SkBlockAllocator::set_metadata_raw(other_allocator, 0);
        }
    }
}
impl<T: Copy, const N: usize> SkTBlockList<T, N> {
    pub fn concat_trivial<const M: usize>(
        self: Pin<&mut Self>,
        other: Pin<&mut SkTBlockList<T, M>>,
    ) {
        unsafe {
            Self::concat_impl::<M, true>(self.raw(), other.raw(), |value| *value);
        }
    }
}
struct ConcatHeadDrop<T, const N: usize> {
    owner: *mut SkTBlockList<T, N>,
    head: *mut Block,
    next: i32,
    end: i32,
    count: i32,
    active: bool,
}
impl<T, const N: usize> Drop for ConcatHeadDrop<T, N> {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                while self.next < self.end {
                    let item = Block::ptr(self.head, self.next).cast::<T>();
                    self.next += mem::size_of::<T>() as i32;
                    ptr::drop_in_place(item);
                }
                let a = SkTBlockList::<T, N>::allocator_raw(self.owner);
                SkBlockAllocator::release_block_raw(a, self.head);
                SkBlockAllocator::set_metadata_raw(a, (*a).metadata() - self.count);
            }
        }
    }
}
// Source: src/core/SkTBlockList.h:392-455
pub struct BlockIndexIterator<T, const FORWARD: bool, O = TBlockListIndex<T>> {
    blocks: BlockIter<FORWARD>,
    block: *mut Block,
    head: *mut Block,
    index: i32,
    end: i32,
    marker: PhantomData<(T, O)>,
}
impl<T, const F: bool, O: BlockIndexOps<T, F>> BlockIndexIterator<T, F, O> {
    fn new(blocks: BlockIter<F>, head: *mut Block) -> Self {
        let mut iter = Self {
            blocks,
            block: ptr::null_mut(),
            head,
            index: 0,
            end: 0,
            marker: PhantomData,
        };
        unsafe {
            iter.set_indices();
        }
        iter
    }
    unsafe fn set_indices(&mut self) {
        self.block = ptr::null_mut();
        while let Some(block) = unsafe { self.blocks.next_raw() } {
            if unsafe { (*block).metadata() } == 0 {
                continue;
            }
            self.block = if ptr::eq(block, self.head) {
                self.head
            } else {
                block
            };
            break;
        }
        if !self.block.is_null() {
            unsafe {
                self.index = O::start(self.block);
                self.end = O::end(self.block);
            }
        } else {
            self.index = 0;
            self.end = 0;
        }
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 442; @condition b"(Forward && fIndex <= fEndIndex) || (!Forward && fIndex >= fEndIndex)\0".as_ptr(); if F {
            self.index <= self.end
        } else {
            self.index >= self.end
        });
    }
    unsafe fn next_raw(&mut self) -> Option<*mut T> {
        if self.block.is_null() {
            return None;
        }
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 404; @condition b"*fBlock\0".as_ptr(); !self.block.is_null());
        let item = unsafe { O::resolve(self.block, self.index) };
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 410; @condition b"block && block->metadata() > 0\0".as_ptr(); unsafe { (*self.block).metadata() } > 0);
        crate::SkASSERT!(@source b"src/core/SkTBlockList.h\0".as_ptr(), 411; @condition b"(Forward && Next(block, fIndex) > fIndex) || (!Forward && Next(block, fIndex) < fIndex)\0".as_ptr(); unsafe {
            if F {
                O::next(self.block, self.index) > self.index
            } else {
                O::next(self.block, self.index) < self.index
            }
        });
        self.index = unsafe { O::next(self.block, self.index) };
        if if F {
            self.index > self.end
        } else {
            self.index < self.end
        } {
            unsafe {
                self.set_indices();
            }
        }
        Some(item)
    }
}
pub struct Items<'a, T, const F: bool> {
    raw: BlockIndexIterator<T, F>,
    lifetime: PhantomData<&'a T>,
}
impl<'a, T, const F: bool> Iterator for Items<'a, T, F> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.raw.next_raw().map(|p| &*p) }
    }
}
pub struct ItemsMut<'a, T, const F: bool> {
    raw: BlockIndexIterator<T, F>,
    lifetime: PhantomData<&'a mut T>,
}
impl<'a, T, const F: bool> Iterator for ItemsMut<'a, T, F> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.raw.next_raw().map(|p| &mut *p) }
    }
}
