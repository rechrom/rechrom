use crate::src::core::SkContainers::{sk_free, sk_malloc_throw};
use core::{marker::PhantomData, ptr};

// Source: include/private/SkDeque.h:38-46
impl SkDeque<'_> {
    pub fn empty(&self) -> bool {
        self.fCount == 0
    }
    pub fn count(&self) -> i32 {
        self.fCount
    }
    pub fn elemSize(&self) -> usize {
        self.fElemSize
    }
    pub fn front(&self) -> *const u8 {
        self.fFront
    }
    pub fn back(&self) -> *const u8 {
        self.fBack
    }
    pub fn front_mut(&mut self) -> *mut u8 {
        self.fFront
    }
    pub fn back_mut(&mut self) -> *mut u8 {
        self.fBack
    }
}
// Source: include/private/SkDeque.h:64-67
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum IterStart {
    kFront_IterStart,
    kBack_IterStart,
}
// Source: include/private/SkDeque.h:62-62
// Source: include/private/SkDeque.h:81-84
pub struct Iter<'d, 'storage> {
    fCurBlock: *mut Block,
    fPos: *mut u8,
    fElemSize: usize,
    borrow: PhantomData<&'d SkDeque<'storage>>,
}
// Source: include/private/SkDeque.h:87-110
pub struct F2BIter<'d, 'storage> {
    inner: Iter<'d, 'storage>,
}
impl Default for F2BIter<'_, '_> {
    fn default() -> Self {
        Self {
            inner: Iter::default(),
        }
    }
}
impl<'d, 'storage> F2BIter<'d, 'storage> {
    pub fn new(deque: &'d SkDeque<'storage>) -> Self {
        Self {
            inner: Iter::new(deque, IterStart::kFront_IterStart),
        }
    }
    pub fn next(&mut self) -> *mut u8 {
        self.inner.next()
    }
    pub fn reset(&mut self, deque: &'d SkDeque<'storage>) {
        self.inner.reset(deque, IterStart::kFront_IterStart);
    }
}
// Source: include/private/SkDeque.h:28-28
// Source: include/private/SkDeque.h:115-123
pub struct SkDeque<'storage> {
    fFront: *mut u8,
    fBack: *mut u8,
    fFrontBlock: *mut Block,
    fBackBlock: *mut Block,
    fElemSize: usize,
    fInitialStorage: *mut u8,
    fCount: i32,
    fAllocCount: i32,
    storage: PhantomData<&'storage mut [core::mem::MaybeUninit<u8>]>,
}

// Source: src/core/SkDeque.cpp:14-29
#[repr(C)]
struct Block {
    fNext: *mut Block,
    fPrev: *mut Block,
    fBegin: *mut u8,
    fEnd: *mut u8,
    fStop: *mut u8,
}
impl Block {
    unsafe fn start(block: *mut Self) -> *mut u8 {
        unsafe { block.add(1).cast() }
    }
    unsafe fn start_const(block: *const Self) -> *const u8 {
        unsafe { block.add(1).cast() }
    }
    unsafe fn init(block: *mut Self, size: usize) {
        unsafe {
            ptr::write(
                block,
                Self {
                    fNext: ptr::null_mut(),
                    fPrev: ptr::null_mut(),
                    fBegin: ptr::null_mut(),
                    fEnd: ptr::null_mut(),
                    fStop: block.cast::<u8>().add(size),
                },
            );
        }
    }
}
// Source: src/core/SkDeque.cpp:31-39
impl SkDeque<'static> {
    pub fn new(elemSize: usize, allocCount: i32) -> Self {
        assert!(
            elemSize > 0 && allocCount >= 1,
            "native deque element/block preconditions"
        );
        Self {
            fElemSize: elemSize,
            fInitialStorage: ptr::null_mut(),
            fCount: 0,
            fAllocCount: allocCount,
            fFrontBlock: ptr::null_mut(),
            fBackBlock: ptr::null_mut(),
            fFront: ptr::null_mut(),
            fBack: ptr::null_mut(),
            storage: PhantomData,
        }
    }
}
// Source: src/core/SkDeque.cpp:41-57
impl<'storage> SkDeque<'storage> {
    /// Safety: storage is stable, exclusively borrowed, Block-aligned, and live until Drop.
    /// In addition to native element destruction, callers must respect native external-block
    /// restrictions: a pop must never cause freeBlock to free that externally owned block.
    /// Canvas's original push_back/pop_back stack usage satisfies this restriction.
    pub unsafe fn with_storage(
        elemSize: usize,
        storage: *mut u8,
        storageSize: usize,
        allocCount: i32,
    ) -> Self {
        assert!(elemSize > 0 && allocCount >= 1);
        crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 46; @condition b"storageSize == 0 || storage != nullptr\0".as_ptr(); storageSize == 0 || !storage.is_null());
        let mut first = ptr::null_mut();
        if storageSize
            >= core::mem::size_of::<Block>()
                .checked_add(elemSize)
                .expect("deque size overflow")
        {
            first = storage.cast::<Block>();
            unsafe {
                Block::init(first, storageSize);
            }
        }
        Self {
            fElemSize: elemSize,
            fInitialStorage: storage,
            fCount: 0,
            fAllocCount: allocCount,
            fFrontBlock: first,
            fBackBlock: first,
            fFront: ptr::null_mut(),
            fBack: ptr::null_mut(),
            storage: PhantomData,
        }
    }
}
// Source: src/core/SkDeque.cpp:59-70
impl Drop for SkDeque<'_> {
    fn drop(&mut self) {
        let mut head = self.fFrontBlock;
        let initialHead = self.fInitialStorage.cast::<Block>();
        while !head.is_null() {
            let next = unsafe { (*head).fNext };
            if head != initialHead {
                unsafe {
                    Self::freeBlock(head);
                }
            }
            head = next;
        }
    }
}
// Source: src/core/SkDeque.cpp:72-110
impl SkDeque<'_> {
    /// Safety: externally supplied storage retains its native block/lifetime contract.
    /// Returned bytes are uninitialized; placement construction/destruction is caller-owned.
    pub unsafe fn push_front(&mut self) -> *mut u8 {
        unsafe {
            self.fCount += 1;
            if self.fFrontBlock.is_null() {
                self.fFrontBlock = self.allocateBlock(self.fAllocCount);
                self.fBackBlock = self.fFrontBlock;
            }
            let mut first = self.fFrontBlock;
            let begin;
            if (*first).fBegin.is_null() {
                (*first).fEnd = (*first).fStop;
                begin = (*first).fStop.wrapping_sub(self.fElemSize);
            } else {
                let candidate = (*first).fBegin.wrapping_sub(self.fElemSize);
                if candidate < Block::start(first) {
                    first = self.allocateBlock(self.fAllocCount);
                    (*first).fNext = self.fFrontBlock;
                    (*self.fFrontBlock).fPrev = first;
                    self.fFrontBlock = first;
                    (*first).fEnd = (*first).fStop;
                    begin = (*first).fStop.wrapping_sub(self.fElemSize);
                } else {
                    begin = candidate;
                }
            }
            (*first).fBegin = begin;
            if self.fFront.is_null() {
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 102; @condition b"nullptr == fBack\0".as_ptr(); self.fBack.is_null());
                self.fFront = begin;
                self.fBack = begin;
            } else {
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 105; @condition b"fBack\0".as_ptr(); !self.fBack.is_null());
                self.fFront = begin;
            }
            begin
        }
    }
}
// Source: src/core/SkDeque.cpp:112-151
impl SkDeque<'_> {
    /// Safety: same raw-placement/external-storage contract as push_front.
    pub unsafe fn push_back(&mut self) -> *mut u8 {
        unsafe {
            self.fCount += 1;
            if self.fBackBlock.is_null() {
                self.fBackBlock = self.allocateBlock(self.fAllocCount);
                self.fFrontBlock = self.fBackBlock;
            }
            let mut last = self.fBackBlock;
            let mut end;
            if (*last).fBegin.is_null() {
                (*last).fBegin = Block::start(last);
                end = (*last).fBegin.wrapping_add(self.fElemSize);
            } else {
                end = (*last).fEnd.wrapping_add(self.fElemSize);
                if end > (*last).fStop {
                    last = self.allocateBlock(self.fAllocCount);
                    (*last).fPrev = self.fBackBlock;
                    (*self.fBackBlock).fNext = last;
                    self.fBackBlock = last;
                    (*last).fBegin = Block::start(last);
                    end = (*last).fBegin.wrapping_add(self.fElemSize);
                }
            }
            (*last).fEnd = end;
            end = end.wrapping_sub(self.fElemSize);
            if self.fBack.is_null() {
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 143; @condition b"nullptr == fFront\0".as_ptr(); self.fFront.is_null());
                self.fFront = end;
                self.fBack = end;
            } else {
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 146; @condition b"fFront\0".as_ptr(); !self.fFront.is_null());
                self.fBack = end;
            }
            end
        }
    }
}
// Source: src/core/SkDeque.cpp:153-185
impl SkDeque<'_> {
    /// Safety: deque is nonempty and caller has destroyed the front element. A transition
    /// away from an empty external block must never free non-C-allocated initial storage.
    pub unsafe fn pop_front(&mut self) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 154; @condition b"fCount > 0\0".as_ptr(); self.fCount > 0);
            self.fCount -= 1;
            let mut first = self.fFrontBlock;
            crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 159; @condition b"first != nullptr\0".as_ptr(); !first.is_null());
            if (*first).fBegin.is_null() {
                first = (*first).fNext;
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 163; @condition b"first != nullptr\0".as_ptr(); !first.is_null());
                (*first).fPrev = ptr::null_mut();
                Self::freeBlock(self.fFrontBlock);
                self.fFrontBlock = first;
            }
            let begin = (*first).fBegin.wrapping_add(self.fElemSize);
            crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 170; @condition b"begin <= first->fEnd\0".as_ptr(); begin <= (*first).fEnd);
            if begin < (*self.fFrontBlock).fEnd {
                (*first).fBegin = begin;
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 174; @condition b"first->fBegin\0".as_ptr(); !(*first).fBegin.is_null());
                self.fFront = (*first).fBegin;
            } else {
                (*first).fBegin = ptr::null_mut();
                (*first).fEnd = ptr::null_mut();
                if (*first).fNext.is_null() {
                    self.fFront = ptr::null_mut();
                    self.fBack = ptr::null_mut();
                } else {
                    crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 181; @condition b"first->fNext->fBegin\0".as_ptr(); !(*(*first).fNext).fBegin.is_null());
                    self.fFront = (*(*first).fNext).fBegin;
                }
            }
        }
    }
}
// Source: src/core/SkDeque.cpp:187-219
impl SkDeque<'_> {
    /// Safety: deque is nonempty and back element was destroyed. Same native external-block
    /// restriction as pop_front; Canvas's one-ended stack never frees its initial block.
    pub unsafe fn pop_back(&mut self) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 188; @condition b"fCount > 0\0".as_ptr(); self.fCount > 0);
            self.fCount -= 1;
            let mut last = self.fBackBlock;
            crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 193; @condition b"last != nullptr\0".as_ptr(); !last.is_null());
            if (*last).fEnd.is_null() {
                last = (*last).fPrev;
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 197; @condition b"last != nullptr\0".as_ptr(); !last.is_null());
                (*last).fNext = ptr::null_mut();
                Self::freeBlock(self.fBackBlock);
                self.fBackBlock = last;
            }
            let end = (*last).fEnd.wrapping_sub(self.fElemSize);
            crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 204; @condition b"end >= last->fBegin\0".as_ptr(); end >= (*last).fBegin);
            if end > (*last).fBegin {
                (*last).fEnd = end;
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 208; @condition b"last->fEnd\0".as_ptr(); !(*last).fEnd.is_null());
                self.fBack = (*last).fEnd.wrapping_sub(self.fElemSize);
            } else {
                (*last).fBegin = ptr::null_mut();
                (*last).fEnd = ptr::null_mut();
                if (*last).fPrev.is_null() {
                    self.fFront = ptr::null_mut();
                    self.fBack = ptr::null_mut();
                } else {
                    crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 215; @condition b"last->fPrev->fEnd\0".as_ptr(); !(*(*last).fPrev).fEnd.is_null());
                    self.fBack = (*(*last).fPrev).fEnd.wrapping_sub(self.fElemSize);
                }
            }
        }
    }
}
// Source: src/core/SkDeque.cpp:221-229
impl SkDeque<'_> {
    pub fn numBlocksAllocated(&self) -> i32 {
        let mut numBlocks = 0;
        let mut temp = self.fFrontBlock;
        while !temp.is_null() {
            numBlocks += 1;
            temp = unsafe { (*temp).fNext };
        }
        numBlocks
    }
}
// Source: src/core/SkDeque.cpp:231-235
impl SkDeque<'_> {
    unsafe fn allocateBlock(&self, allocCount: i32) -> *mut Block {
        let size = (allocCount as usize)
            .checked_mul(self.fElemSize)
            .and_then(|v| v.checked_add(core::mem::size_of::<Block>()))
            .expect("deque allocation overflow");
        let newBlock = sk_malloc_throw(size).cast::<Block>();
        unsafe {
            Block::init(newBlock, size);
        }
        newBlock
    }
}
// Source: src/core/SkDeque.cpp:237-239
impl SkDeque<'_> {
    unsafe fn freeBlock(block: *mut Block) {
        unsafe {
            sk_free(block.cast());
        }
    }
}

// Source: src/core/SkDeque.cpp:243-243
impl Default for Iter<'_, '_> {
    fn default() -> Self {
        Self {
            fCurBlock: ptr::null_mut(),
            fPos: ptr::null_mut(),
            fElemSize: 0,
            borrow: PhantomData,
        }
    }
}
// Source: src/core/SkDeque.cpp:245-247
impl<'d, 'storage> Iter<'d, 'storage> {
    pub fn new(deque: &'d SkDeque<'storage>, startLoc: IterStart) -> Self {
        let mut iter = Self::default();
        iter.reset(deque, startLoc);
        iter
    }
}
// Source: src/core/SkDeque.cpp:251-266
impl Iter<'_, '_> {
    pub fn next(&mut self) -> *mut u8 {
        let pos = self.fPos;
        if !pos.is_null() {
            let mut next = pos.wrapping_add(self.fElemSize);
            unsafe {
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 256; @condition b"next <= fCurBlock->fEnd\0".as_ptr(); next <= (*self.fCurBlock).fEnd);
                if next == (*self.fCurBlock).fEnd {
                    loop {
                        self.fCurBlock = (*self.fCurBlock).fNext;
                        if self.fCurBlock.is_null() || !(*self.fCurBlock).fBegin.is_null() {
                            break;
                        }
                    }
                    next = if self.fCurBlock.is_null() {
                        ptr::null_mut()
                    } else {
                        (*self.fCurBlock).fBegin
                    };
                }
            }
            self.fPos = next;
        }
        pos
    }
}
// Source: src/core/SkDeque.cpp:270-285
impl Iter<'_, '_> {
    pub fn prev(&mut self) -> *mut u8 {
        let pos = self.fPos;
        if !pos.is_null() {
            let mut prev = pos.wrapping_sub(self.fElemSize);
            unsafe {
                crate::SkASSERT!(@source b"src/core/SkDeque.cpp\0".as_ptr(), 275; @condition b"prev >= fCurBlock->fBegin - fElemSize\0".as_ptr(); prev >= (*self.fCurBlock).fBegin.wrapping_sub(self.fElemSize));
                if prev < (*self.fCurBlock).fBegin {
                    loop {
                        self.fCurBlock = (*self.fCurBlock).fPrev;
                        if self.fCurBlock.is_null() || !(*self.fCurBlock).fEnd.is_null() {
                            break;
                        }
                    }
                    prev = if self.fCurBlock.is_null() {
                        ptr::null_mut()
                    } else {
                        (*self.fCurBlock).fEnd.wrapping_sub(self.fElemSize)
                    };
                }
            }
            self.fPos = prev;
        }
        pos
    }
}
// Source: src/core/SkDeque.cpp:292-310
impl<'d, 'storage> Iter<'d, 'storage> {
    pub fn reset(&mut self, deque: &'d SkDeque<'storage>, startLoc: IterStart) {
        self.fElemSize = deque.fElemSize;
        unsafe {
            if startLoc == IterStart::kFront_IterStart {
                self.fCurBlock = deque.fFrontBlock;
                while !self.fCurBlock.is_null() && (*self.fCurBlock).fBegin.is_null() {
                    self.fCurBlock = (*self.fCurBlock).fNext;
                }
                self.fPos = if self.fCurBlock.is_null() {
                    ptr::null_mut()
                } else {
                    (*self.fCurBlock).fBegin
                };
            } else {
                self.fCurBlock = deque.fBackBlock;
                while !self.fCurBlock.is_null() && (*self.fCurBlock).fEnd.is_null() {
                    self.fCurBlock = (*self.fCurBlock).fPrev;
                }
                self.fPos = if self.fCurBlock.is_null() {
                    ptr::null_mut()
                } else {
                    (*self.fCurBlock).fEnd.wrapping_sub(self.fElemSize)
                };
            }
        }
    }
}

// Rust provenance adaptation for an owner-pinned inline initial block. A fresh
// exclusive borrow of the enclosing owner supplies a fresh pointer to the same
// bytes; no block address, element address, count or capacity changes.
impl SkDeque<'_> {
    /// # Safety
    /// storage denotes exactly the same live initial storage allocation/address
    /// and byte size passed to with_storage. Caller exclusively owns its bytes,
    /// holds no live element/iterator references, and has not popped/free-crossed
    /// the externally owned initial block. This does not permit moving storage.
    pub(crate) unsafe fn rebind_initial_storage(&mut self, storage: *mut u8, size: usize) {
        debug_assert_eq!(storage.addr(), self.fInitialStorage.addr());
        if storage.is_null() {
            return;
        }
        let old = self.fInitialStorage;
        let old_block = old.cast::<Block>();
        let fresh = storage.cast::<Block>();
        let rebase = |p: *mut u8| {
            let address = p.addr();
            let base = old.addr();
            if !p.is_null() && address >= base && address - base <= size {
                storage.wrapping_add(address - base)
            } else {
                p
            }
        };
        self.fFront = rebase(self.fFront);
        self.fBack = rebase(self.fBack);
        if self.fFrontBlock == old_block {
            self.fFrontBlock = fresh;
        }
        if self.fBackBlock == old_block {
            self.fBackBlock = fresh;
        }
        // A too-small native initial storage has no initialized Block header.
        if size >= core::mem::size_of::<Block>() + self.fElemSize {
            unsafe {
                (*fresh).fBegin = rebase((*fresh).fBegin);
                (*fresh).fEnd = rebase((*fresh).fEnd);
                (*fresh).fStop = rebase((*fresh).fStop);
                let prev = (*fresh).fPrev;
                let next = (*fresh).fNext;
                if !prev.is_null() {
                    (*prev).fNext = fresh;
                }
                if !next.is_null() {
                    (*next).fPrev = fresh;
                }
            }
        }
        self.fInitialStorage = storage;
    }
}
