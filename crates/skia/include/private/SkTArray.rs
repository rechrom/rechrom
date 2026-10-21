use crate::src::core::SkContainers::{
    sk_free, sk_report_container_overflow_and_die, ByteSpan, SkContainerAllocator,
};
use core::{
    mem,
    ops::{Index, IndexMut},
    ptr,
};

// The implementation owns raw, individually constructed elements, not Vec storage.
// Rust moves transfer ownership without dropping the vacated bytes. MEM_MOVE retains
// the native bulk-copy/per-element move choice; both branches invalidate old storage.
// ZST/over-aligned Rust-only types have no supported native malloc representation.

// Source: include/private/SkTArray.h:46-46
impl<T, const MEM_MOVE: bool> Default for TArray<T, MEM_MOVE> {
    fn default() -> Self {
        assert!(mem::size_of::<T>() > 0 && mem::align_of::<T>() <= mem::align_of::<u128>());
        Self {
            fData: ptr::null_mut(),
            fSize: 0,
            fCapacityAndOwn: 1,
        }
    }
}
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rust ownership projection for native call sites whose API carries a
    /// stable owner even though TArray itself has no inline self-pointer.
    pub fn new_pinned() -> core::pin::Pin<Box<Self>> {
        Box::pin(Self::new())
    }
}
// Source: include/private/SkTArray.h:51-51
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn with_reserve(reserveCount: i32) -> Self {
        let mut a = Self::new();
        a.reserve_exact(reserveCount);
        a
    }
}
// Source: include/private/SkTArray.h:56-56
impl<T: Clone, const MEM_MOVE: bool> Clone for TArray<T, MEM_MOVE> {
    fn clone(&self) -> Self {
        Self::from_slice(self.as_slice())
    }
    fn clone_from(&mut self, that: &Self) {
        self.copy_assign(that);
    }
}
// Source: include/private/SkTArray.h:58-69
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn take_from(that: &mut Self) -> Self {
        let mut array = Self::new();
        if that.owns_memory() {
            array.set_data(that.fData, that.capacity());
            that.set_data(ptr::null_mut(), 0);
        } else {
            unsafe {
                array.init_data(that.fSize);
                that.move_raw(array.fData);
            }
        }
        array.change_size(that.fSize);
        that.change_size(0);
        array
    }
}
// Source: include/private/SkTArray.h:75-78
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    /// Safety: src addresses count initialized T values distinct from the new destination.
    pub unsafe fn from_raw_copy(src: *const T, count: i32) -> Self {
        let mut array = Self::new();
        unsafe {
            array.init_data(count);
            array.copy_raw(src);
        }
        array
    }
}
// Source: include/private/SkTArray.h:83-88
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn from_slice(data: &[T]) -> Self {
        if data.len() > Self::kMaxCapacity as usize {
            sk_report_container_overflow_and_die();
        }
        unsafe { Self::from_raw_copy(data.as_ptr(), data.len() as i32) }
    }
}
// Source: include/private/SkTArray.h:90-99
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn copy_assign(&mut self, that: &Self) {
        if ptr::eq(self, that) {
            return;
        }
        self.clear();
        self.check_realloc(that.size(), Self::kExactFit);
        self.change_size(that.fSize);
        unsafe {
            self.copy_raw(that.fData);
        }
    }
}
// Source: include/private/SkTArray.h:101-130
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn move_assign(&mut self, that: &mut Self) {
        if ptr::eq(self, that) {
            return;
        }
        self.clear();
        self.unpoison();
        that.unpoison();
        if that.owns_memory() {
            if self.owns_memory() {
                unsafe {
                    sk_free(self.fData.cast());
                }
            }
            self.fData = mem::replace(&mut that.fData, ptr::null_mut());
            self.set_capacity(that.capacity());
            that.set_capacity(0);
            self.set_own(true);
            self.change_size(that.fSize);
        } else {
            self.check_realloc(that.size(), Self::kExactFit);
            self.change_size(that.fSize);
            unsafe {
                that.move_raw(self.fData);
            }
        }
        that.change_size(0);
    }
}
// Source: include/private/SkTArray.h:132-138
impl<T, const MEM_MOVE: bool> Drop for TArray<T, MEM_MOVE> {
    fn drop(&mut self) {
        // FreeGuard additionally releases allocation if a Rust destructor unwinds.
        let _allocation = FreeGuard(if self.owns_memory() {
            self.fData.cast()
        } else {
            ptr::null_mut()
        });
        unsafe {
            self.destroy_all();
        }
        self.unpoison();
    }
}
// Source: include/private/SkTArray.h:143-151
// The explicit constructor preserves native default-initialization policy. In
// particular, Rust Default is not substituted for uninitialized native scalars.
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn reset_with(&mut self, n: i32, mut construct: impl FnMut() -> T) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 144; @condition b"n >= 0\0".as_ptr(); n>=0);
        if n < 0 {
            std::process::abort();
        }
        self.clear();
        self.check_realloc(n, Self::kExactFit);
        self.change_size(n);
        let mut guard = InitGuard {
            size: ptr::addr_of_mut!(self.fSize),
            initialized: 0,
            active: true,
        };
        for i in 0..n {
            unsafe {
                self.fData.add(i as usize).write(construct());
            }
            guard.initialized += 1;
        }
        guard.active = false;
    }
}

// Source: include/private/SkTArray.h:156-161
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn reset_copy(&mut self, src: &[T]) {
        if src.len() > Self::kMaxCapacity as usize {
            sk_report_container_overflow_and_die();
        }
        self.clear();
        self.check_realloc(src.len() as i32, Self::kExactFit);
        self.change_size(src.len() as i32);
        unsafe {
            self.copy_raw(src.as_ptr());
        }
    }
}
// Source: include/private/SkTArray.h:168-173
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn reserve(&mut self, n: i32) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 169; @condition b"n >= 0\0".as_ptr(); n>=0);
        if n < 0 {
            std::process::abort();
        }
        if n > self.size() {
            self.check_realloc(n - self.size(), Self::kGrowing);
        }
    }
}
// Source: include/private/SkTArray.h:179-184
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn reserve_exact(&mut self, n: i32) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 180; @condition b"n >= 0\0".as_ptr(); n>=0);
        if n < 0 {
            std::process::abort();
        }
        if n > self.size() {
            self.check_realloc(n - self.size(), Self::kExactFit);
        }
    }
}
// Source: include/private/SkTArray.h:186-195
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn removeShuffle(&mut self, n: i32) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 187; @condition b"n < this->size()\0".as_ptr(); n<self.size());
        if n < 0 || n >= self.size() {
            std::process::abort();
        }
        let last = self.fSize - 1;
        let removal = RemovalGuard::<T, MEM_MOVE> {
            size: ptr::addr_of_mut!(self.fSize),
            data: self.fData,
            dst: n,
            src: last,
            new_size: last,
            shuffle: n != last,
        };
        unsafe {
            ptr::drop_in_place(self.fData.add(n as usize));
        }
        drop(removal);
    }
}
// Source: include/private/SkTArray.h:197-197
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn empty(&self) -> bool {
        self.fSize == 0
    }
}
// Source: include/private/SkTArray.h:203-206
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn push_back_with(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        let old_size = self.fSize;
        let dst = unsafe { self.push_back_raw(1) };
        let mut guard = InitGuard {
            size: ptr::addr_of_mut!(self.fSize),
            initialized: old_size,
            active: true,
        };
        unsafe {
            dst.write(construct());
        }
        guard.active = false;
        unsafe { &mut *dst }
    }
}

// Source: include/private/SkTArray.h:212-225
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn push_back_clone(&mut self, t: &T) -> &mut T {
        unsafe { self.push_back_clone_raw(t) }
    }
    /// Safety: t points to an initialized T and may alias an existing element. No Rust
    /// reference overlapping the exclusive array borrow may remain live across this call.
    pub unsafe fn push_back_clone_raw(&mut self, t: *const T) -> &mut T {
        self.unpoison();
        let newT;
        if self.capacity() > self.fSize {
            newT = unsafe { self.fData.add(self.fSize as usize) };
            unsafe {
                newT.write((&*t).clone());
            }
        } else {
            newT = unsafe { self.grow_and_construct_at_end(|| (&*t).clone()) };
        }
        self.change_size(self.fSize + 1);
        unsafe { &mut *newT }
    }
}
// Source: include/private/SkTArray.h:229-242
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn push_back(&mut self, t: T) -> &mut T {
        self.unpoison();
        let newT;
        if self.capacity() > self.fSize {
            newT = unsafe { self.fData.add(self.fSize as usize) };
            unsafe {
                newT.write(t);
            }
        } else {
            newT = unsafe { self.grow_and_construct_at_end(|| t) };
        }
        self.change_size(self.fSize + 1);
        unsafe { &mut *newT }
    }
}
// Source: include/private/SkTArray.h:246-258
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn emplace_back_with(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.unpoison();
        let dst;
        if self.capacity() > self.fSize {
            dst = unsafe { self.fData.add(self.fSize as usize) };
            unsafe {
                dst.write(construct());
            }
        } else {
            dst = unsafe { self.grow_and_construct_at_end(construct) };
        }
        self.change_size(self.fSize + 1);
        unsafe { &mut *dst }
    }
}

// Source: include/private/SkTArray.h:265-272
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn push_back_n_with(&mut self, n: i32, mut construct: impl FnMut() -> T) -> &mut [T] {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 266; @condition b"n >= 0\0".as_ptr(); n>=0);
        if n < 0 {
            std::process::abort();
        }
        let old_size = self.fSize;
        let dst = unsafe { self.push_back_raw(n) };
        let mut guard = InitGuard {
            size: ptr::addr_of_mut!(self.fSize),
            initialized: old_size,
            active: true,
        };
        for i in 0..n {
            unsafe {
                dst.add(i as usize).write(construct());
            }
            guard.initialized += 1;
        }
        guard.active = false;
        if n == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(dst, n as usize) }
        }
    }
}

// Source: include/private/SkTArray.h:278-285
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn push_back_n_clone(&mut self, n: i32, value: &T) -> &mut [T] {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 279; @condition b"n >= 0\0".as_ptr(); n>=0);
        if n < 0 {
            std::process::abort();
        }
        let old_size = self.fSize;
        let dst = unsafe { self.push_back_raw(n) };
        let mut guard = InitGuard {
            size: ptr::addr_of_mut!(self.fSize),
            initialized: old_size,
            active: true,
        };
        for i in 0..n {
            unsafe {
                dst.add(i as usize).write(value.clone());
            }
            guard.initialized += 1;
        }
        guard.active = false;
        if n == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(dst, n as usize) }
        }
    }
}

// Source: include/private/SkTArray.h:291-300
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn push_back_slice(&mut self, t: &[T]) -> &mut [T] {
        if t.len() > Self::kMaxCapacity as usize {
            sk_report_container_overflow_and_die();
        }
        let n = t.len() as i32;
        self.check_realloc(n, Self::kGrowing);
        let old = self.fSize;
        let end = self.fData.wrapping_add(old as usize);
        self.change_size(old + n);
        let mut guard = InitGuard {
            size: ptr::addr_of_mut!(self.fSize),
            initialized: old,
            active: true,
        };
        for value in t {
            unsafe {
                end.add((guard.initialized - old) as usize)
                    .write(value.clone());
            }
            guard.initialized += 1;
        }
        guard.active = false;
        if n == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(end, n as usize) }
        }
    }
}
// Source: include/private/SkTArray.h:305-314
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    /// Safety: source addresses n initialized values; caller must treat them as moved out.
    pub unsafe fn move_back_n(&mut self, n: i32, t: *mut T) -> *mut T {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 306; @condition b"n >= 0\0".as_ptr(); n>=0);
        if n < 0 {
            std::process::abort();
        }
        self.check_realloc(n, Self::kGrowing);
        let end = self.fData.wrapping_add(self.fSize as usize);
        self.change_size(self.fSize + n);
        for i in 0..n as usize {
            unsafe {
                end.add(i).write(t.add(i).read());
            }
        }
        end
    }
}
// Source: include/private/SkTArray.h:319-323
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn pop_back(&mut self) {
        if self.empty() {
            std::process::abort();
        }
        let n = self.fSize - 1;
        let removal = RemovalGuard::<T, MEM_MOVE> {
            size: ptr::addr_of_mut!(self.fSize),
            data: self.fData,
            dst: n,
            src: n,
            new_size: n,
            shuffle: false,
        };
        unsafe {
            ptr::drop_in_place(self.fData.add(n as usize));
        }
        drop(removal);
    }
}
// Source: include/private/SkTArray.h:328-336
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn pop_back_n(&mut self, n: i32) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 329; @condition b"n >= 0\0".as_ptr(); n>=0);
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 330; @condition b"this->size() >= n\0".as_ptr(); self.size()>=n);
        if n < 0 || n > self.size() {
            std::process::abort();
        }
        let stop = self.fSize - n;
        let mut guard = TailRemovalGuard {
            size: ptr::addr_of_mut!(self.fSize),
            data: self.fData,
            next: self.fSize - 1,
            stop,
            active: true,
        };
        while guard.next >= stop {
            let i = guard.next;
            guard.next -= 1;
            unsafe {
                ptr::drop_in_place(self.fData.add(i as usize));
            }
        }
        guard.active = false;
        self.change_size(stop);
    }
}
// Source: include/private/SkTArray.h:341-352
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn resize_back_with(&mut self, new_count: i32, construct: impl FnMut() -> T) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 342; @condition b"newCount >= 0\0".as_ptr(); new_count>=0);
        if new_count < 0 {
            std::process::abort();
        }
        if new_count > self.size() {
            if self.empty() {
                self.check_realloc(new_count, Self::kExactFit);
            }
            self.push_back_n_with(new_count - self.fSize, construct);
        } else if new_count < self.size() {
            self.pop_back_n(self.fSize - new_count);
        }
    }
}
// Source: include/private/SkTArray.h:356-375
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn swap(&mut self, that: &mut Self) {
        if ptr::eq(self, that) {
            return;
        }
        if self.owns_memory() && that.owns_memory() {
            mem::swap(&mut self.fData, &mut that.fData);
            mem::swap(&mut self.fSize, &mut that.fSize);
            mem::swap(&mut self.fCapacityAndOwn, &mut that.fCapacityAndOwn);
        } else {
            let mut copy = Self::take_from(that);
            that.move_assign(self);
            self.move_assign(&mut copy);
        }
    }
}
// Source: include/private/SkTArray.h:381-394
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn move_back(&mut self, that: &mut Self) {
        if that.empty() || ptr::eq(self, that) {
            return;
        }
        let dst = unsafe { self.push_back_raw(that.size()) };
        unsafe {
            that.move_raw(dst);
        }
        that.change_size(0);
    }
}
// Source: include/private/SkTArray.h:396-420
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn begin(&self) -> *const T {
        self.fData
    }
    pub fn begin_mut(&mut self) -> *mut T {
        self.fData
    }
    pub fn end(&self) -> *const T {
        if self.fData.is_null() {
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 407; @condition b"fSize == 0\0".as_ptr(); self.fSize == 0);
        };
        self.fData.wrapping_add(self.fSize as usize)
    }
    pub fn end_mut(&mut self) -> *mut T {
        if self.fData.is_null() {
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 413; @condition b"fSize == 0\0".as_ptr(); self.fSize == 0);
        };
        self.fData.wrapping_add(self.fSize as usize)
    }
    pub fn data(&self) -> *const T {
        self.fData
    }
    pub fn data_mut(&mut self) -> *mut T {
        self.fData
    }
    pub fn size(&self) -> i32 {
        self.fSize
    }
    pub fn size_bytes(&self) -> usize {
        Self::bytes(self.fSize)
    }
}
// Source: include/private/SkTArray.h:421-421
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn resize_with(&mut self, count: usize, construct: impl FnMut() -> T) {
        self.resize_back_with(count as i32, construct);
    }
}

// Source: include/private/SkTArray.h:423-426
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn clear(&mut self) {
        unsafe {
            self.destroy_all();
        }
        self.change_size(0);
    }
}
// Source: include/private/SkTArray.h:428-445
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn shrink_to_fit(&mut self) {
        if !self.owns_memory() || self.fSize == self.capacity() {
            return;
        }
        self.unpoison();
        if self.fSize == 0 {
            unsafe {
                sk_free(self.fData.cast());
            }
            self.fData = ptr::null_mut();
            self.set_capacity(0);
        } else {
            let allocation = Self::allocate(self.fSize, Self::kExactFit);
            unsafe {
                self.move_raw(allocation.0.cast());
            }
            if self.owns_memory() {
                unsafe {
                    sk_free(self.fData.cast());
                }
            }
            self.set_data_from_bytes(allocation);
        }
    }
}
// Source: include/private/SkTArray.h:450-497
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn at(&self, i: i32) -> &T {
        if i < 0 || i >= self.fSize {
            std::process::abort();
        }
        unsafe { &*self.fData.add(i as usize) }
    }
    pub fn at_mut(&mut self, i: i32) -> &mut T {
        if i < 0 || i >= self.fSize {
            std::process::abort();
        }
        unsafe { &mut *self.fData.add(i as usize) }
    }
    pub fn front(&self) -> &T {
        self.at(0)
    }
    pub fn front_mut(&mut self) -> &mut T {
        self.at_mut(0)
    }
    pub fn back(&self) -> &T {
        self.at(self.fSize - 1)
    }
    pub fn back_mut(&mut self) -> &mut T {
        self.at_mut(self.fSize - 1)
    }
    pub fn fromBack(&self, i: i32) -> &T {
        self.at(self.fSize - i - 1)
    }
    pub fn fromBack_mut(&mut self, i: i32) -> &mut T {
        self.at_mut(self.fSize - i - 1)
    }
}
// Source: include/private/SkTArray.h:499-514
impl<T: PartialEq, const MEM_MOVE: bool> PartialEq for TArray<T, MEM_MOVE> {
    fn eq(&self, right: &Self) -> bool {
        let n = self.size();
        if n != right.size() {
            return false;
        }
        for i in 0..n {
            if self.at(i) != right.at(i) {
                return false;
            }
        }
        true
    }
    fn ne(&self, right: &Self) -> bool {
        !self.eq(right)
    }
}
// Source: include/private/SkTArray.h:516-518
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    pub fn capacity(&self) -> i32 {
        (self.fCapacityAndOwn >> 1) as i32
    }
}
// Source: include/private/SkTArray.h:524-537
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn init_inline(&mut self, storage: *mut u8, storage_bytes: usize, size: i32) {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 526; @condition b"size >= 0\0".as_ptr(); size >= 0);
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 527; @condition b"storage->get() != nullptr\0".as_ptr(); !storage.is_null());
        let initial = (storage_bytes / mem::size_of::<T>()).min(Self::kMaxCapacity as usize);
        if size as usize > initial {
            unsafe {
                self.init_data(size);
            }
        } else {
            self.set_data_from_bytes((storage, storage_bytes));
            self.change_size(size);
            self.set_own(false);
        }
    }
}
// Source: include/private/SkTArray.h:542-546
impl<T: Clone, const M: bool> TArray<T, M> {
    /// Safety: storage is pinned exclusive inline storage; src/count are initialized
    /// independent elements. No previous element may be live in the destination.
    unsafe fn init_inline_copy_raw(
        &mut self,
        src: *const T,
        count: i32,
        storage: *mut u8,
        storage_bytes: usize,
    ) {
        unsafe {
            self.init_inline(storage, storage_bytes, count);
            self.copy_raw(src);
        }
    }
    // Source: include/private/SkTArray.h:547-551
    unsafe fn init_inline_copy_slice(&mut self, src: &[T], storage: *mut u8, storage_bytes: usize) {
        if src.len() > Self::kMaxCapacity as usize {
            sk_report_container_overflow_and_die();
        }
        unsafe {
            self.init_inline_copy_raw(src.as_ptr(), src.len() as i32, storage, storage_bytes);
        }
    }
}

// Source: include/private/SkTArray.h:555-563
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    const kExactFit: f64 = 1.0;
    const kGrowing: f64 = 1.5;
    const kMinHeapAllocCount: i32 = 8;
    const kMaxCapacity: i32 = {
        let max = usize::MAX / mem::size_of::<T>();
        if max < i32::MAX as usize {
            max as i32
        } else {
            i32::MAX
        }
    };
}
// Source: include/private/SkTArray.h:565-582
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn set_data_from_bytes(&mut self, allocation: ByteSpan) {
        let size = (allocation.1 / mem::size_of::<T>()).min(Self::kMaxCapacity as usize);
        self.set_data(allocation.0.cast(), size as i32);
    }
    fn set_data(&mut self, data: *mut T, capacity: i32) {
        self.unpoison();
        self.fData = data;
        self.fCapacityAndOwn = (capacity as u32) << 1 | 1;
        self.poison();
    }
}
// Source: include/private/SkTArray.h:584-603
// Selected native !SK_SANITIZE_ADDRESS branch is empty. ASAN hooks are excluded
// from this initial CPU slice, rather than provided with fake poison callbacks.
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn unpoison(&mut self) {}
    fn poison(&mut self) {}
}
// Source: include/private/SkTArray.h:605-609
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn change_size(&mut self, n: i32) {
        self.unpoison();
        self.fSize = n;
        self.poison();
    }
}
// Source: include/private/SkTArray.h:619-630
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn TCast(buffer: *mut u8) -> *mut T {
        buffer.cast()
    }
    fn bytes(n: i32) -> usize {
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 624; @condition b"n <= kMaxCapacity\0".as_ptr(); n <= Self::kMaxCapacity);
        (n as usize) * mem::size_of::<T>()
    }
    fn allocate(capacity: i32, growth: f64) -> ByteSpan {
        SkContainerAllocator::new(mem::size_of::<T>(), Self::kMaxCapacity)
            .allocate(capacity, growth)
    }
}
// Source: include/private/SkTArray.h:632-635
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn init_data(&mut self, count: i32) {
        self.set_data_from_bytes(Self::allocate(count, Self::kExactFit));
        self.change_size(count);
    }
}
// Source: include/private/SkTArray.h:637-646
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn destroy_all(&mut self) {
        if !self.empty() {
            let mut guard = ForwardDropGuard {
                size: ptr::addr_of_mut!(self.fSize),
                data: self.fData,
                next: 0,
                end: self.fSize,
                active: true,
            };
            while guard.next < guard.end {
                let i = guard.next;
                guard.next += 1;
                unsafe {
                    ptr::drop_in_place(self.fData.add(i as usize));
                }
            }
            guard.active = false;
        }
    }
}
// Source: include/private/SkTArray.h:651-661
// Rust Clone implements the native nontrivial copy path. Unwind guard adjusts only
// exceptional initialization count, retaining native fSize ordering on normal paths.
impl<T: Clone, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn copy_raw(&mut self, src: *const T) {
        let mut guard = InitGuard {
            size: ptr::addr_of_mut!(self.fSize),
            initialized: 0,
            active: true,
        };
        for i in 0..self.fSize {
            unsafe {
                self.fData
                    .add(i as usize)
                    .write((&*src.add(i as usize)).clone());
            }
            guard.initialized += 1;
        }
        guard.active = false;
    }
}
// Source: include/private/SkTArray.h:652-656
// Rust cannot specialize a Clone bound on Copy. This explicit Copy entry
// preserves the native trivial-copy branch without invoking user Clone code.
impl<T: Copy, const M: bool> TArray<T, M> {
    unsafe fn copy_trivial_raw(&mut self, source: *const T) {
        if !self.empty() && !source.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(source, self.fData, self.fSize as usize);
            }
        }
    }
    pub fn from_copy_slice(source: &[T]) -> Self {
        if source.len() > Self::kMaxCapacity as usize {
            sk_report_container_overflow_and_die();
        }
        let mut array = Self::new();
        unsafe {
            array.init_data(source.len() as i32);
            array.copy_trivial_raw(source.as_ptr());
        }
        array
    }
    pub fn reset_copy_trivial(&mut self, source: &[T]) {
        if source.len() > Self::kMaxCapacity as usize {
            sk_report_container_overflow_and_die();
        }
        self.clear();
        self.check_realloc(source.len() as i32, Self::kExactFit);
        self.change_size(source.len() as i32);
        unsafe {
            self.copy_trivial_raw(source.as_ptr());
        }
    }
}

// Source: include/private/SkTArray.h:663-672
// Rust ownership adaptation: moved source bytes are invalidated, so no second
// destructor runs on their transferred value. Guard calls this same native branch.
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn move_element_ptr(data: *mut T, dst: i32, src: i32) {
        if MEM_MOVE {
            unsafe {
                ptr::copy_nonoverlapping(data.add(src as usize), data.add(dst as usize), 1);
            }
        } else {
            unsafe {
                data.add(dst as usize).write(data.add(src as usize).read());
            }
        }
    }
}
// Source: include/private/SkTArray.h:674-683
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn move_raw(&self, dst: *mut T) {
        if MEM_MOVE {
            if self.fSize != 0 {
                unsafe {
                    ptr::copy_nonoverlapping(self.fData, dst, self.fSize as usize);
                }
            }
        } else {
            for i in 0..self.fSize as usize {
                unsafe {
                    dst.add(i).write(self.fData.add(i).read());
                }
            }
        }
    }
}
// Source: include/private/SkTArray.h:687-692
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn push_back_raw(&mut self, n: i32) -> *mut T {
        self.check_realloc(n, Self::kGrowing);
        let p = self.fData.wrapping_add(self.fSize as usize);
        self.change_size(self.fSize + n);
        p
    }
}
// Source: include/private/SkTArray.h:695-701
// Rust constructor closure preserves native construction before old-buffer relocation.
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn grow_and_construct_at_end(&mut self, construct: impl FnOnce() -> T) -> *mut T {
        let buffer = self.preallocate_new_data(1, Self::kGrowing);
        let mut free = FreeGuard(buffer.0);
        let newT = unsafe { buffer.0.cast::<T>().add(self.fSize as usize) };
        unsafe {
            newT.write(construct());
            self.install_data_and_update_capacity(buffer);
        }
        free.0 = ptr::null_mut();
        newT
    }
}
// Source: include/private/SkTArray.h:703-713
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn check_realloc(&mut self, delta: i32, growth: f64) {
        {
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 704; @condition b"delta >= 0\0".as_ptr(); delta >= 0);
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 705; @condition b"fSize >= 0\0".as_ptr(); self.fSize >= 0);
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 706; @condition b"fCapacity >= 0\0".as_ptr(); self.capacity() >= 0);
        };
        if self.capacity() - self.fSize < delta {
            let new = self.preallocate_new_data(delta, growth);
            unsafe {
                self.install_data_and_update_capacity(new);
            }
        }
    }
}
// Source: include/private/SkTArray.h:715-730
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn preallocate_new_data(&self, delta: i32, growth: f64) -> ByteSpan {
        {
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 716; @condition b"delta >= 0\0".as_ptr(); delta >= 0);
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 717; @condition b"fSize >= 0\0".as_ptr(); self.fSize >= 0);
            crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 718; @condition b"fCapacity >= 0\0".as_ptr(); self.capacity() >= 0);
        };
        if delta > Self::kMaxCapacity - self.fSize {
            sk_report_container_overflow_and_die();
        }
        Self::allocate(self.fSize + delta, growth)
    }
}
// Source: include/private/SkTArray.h:732-739
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    unsafe fn install_data_and_update_capacity(&mut self, allocation: ByteSpan) {
        unsafe {
            self.move_raw(allocation.0.cast());
            if self.owns_memory() {
                sk_free(self.fData.cast());
            }
        }
        self.set_data_from_bytes(allocation);
        crate::SkASSERT!(@source b"include/private/SkTArray.h\0".as_ptr(), 738; @condition b"fData != nullptr\0".as_ptr(); !self.fData.is_null());
    }
}
// Source: include/private/SkTArray.h:39-39
// Source: include/private/SkTArray.h:741-744
#[repr(C)]
pub struct TArray<T, const MEM_MOVE: bool = false> {
    fData: *mut T,
    fSize: i32,
    fCapacityAndOwn: u32,
}
impl<T, const MEM_MOVE: bool> TArray<T, MEM_MOVE> {
    fn owns_memory(&self) -> bool {
        self.fCapacityAndOwn & 1 != 0
    }
    fn set_own(&mut self, own: bool) {
        self.fCapacityAndOwn = (self.fCapacityAndOwn & !1) | u32::from(own);
    }
    fn set_capacity(&mut self, n: i32) {
        self.fCapacityAndOwn = (n as u32) << 1 | (self.fCapacityAndOwn & 1);
    }
    pub fn as_slice(&self) -> &[T] {
        if self.fSize == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.fData, self.fSize as usize) }
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.fSize == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(self.fData, self.fSize as usize) }
        }
    }
}
// Source: include/private/SkTArray.h:750-752
pub fn swap<T, const M: bool>(a: &mut TArray<T, M>, b: &mut TArray<T, M>) {
    a.swap(b);
}

// Source: include/private/SkTArray.h:754-770
// Storage size is rounded by its native 8-byte alignment. The raw pointer is
// bound only after Pin supplies the stable address. This constructor/address
// adaptation is not claimed as literal source coverage. T values are not
// structurally pinned: references to elements may be replaced/moved normally.
#[repr(C, align(8))]
struct InlineStorage<T, const N: usize> {
    slots: [mem::MaybeUninit<T>; N],
}
#[repr(C)]
pub struct STArray<T, const N: usize, const MEM_MOVE: bool = false> {
    storage: InlineStorage<T, N>,
    array: TArray<T, MEM_MOVE>,
    pinned: core::marker::PhantomPinned,
}
impl<T, const N: usize, const M: bool> Default for STArray<T, N, M> {
    fn default() -> Self {
        assert!(N > 0);
        let mut array = TArray::new();
        array.set_own(false);
        Self {
            storage: InlineStorage {
                slots: [const { mem::MaybeUninit::uninit() }; N],
            },
            array,
            pinned: core::marker::PhantomPinned,
        }
    }
}
impl<T, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Allocate the native inline-storage owner at its final address and bind
    /// the inherited TArray pointer before exposing mutable operations.
    pub fn new_pinned() -> core::pin::Pin<Box<Self>> {
        let mut value = Box::pin(Self::new());
        value.as_mut().init();
        value
    }
    const INLINE_CAPACITY: usize = SkContainerAllocator::RoundUp::<T>(N);
    fn pending_inline(&self) -> bool {
        !self.array.owns_memory() && self.array.capacity() == 0
    }
    /// Safety: caller holds the structural Pin projection of this owner and
    /// never moves/replaces the private array/storage fields after binding.
    unsafe fn bind_inline(&mut self) {
        if self.pending_inline() {
            let p = ptr::addr_of_mut!(self.storage).cast::<u8>();
            let bytes = Self::INLINE_CAPACITY * mem::size_of::<T>();
            debug_assert!(bytes <= mem::size_of::<InlineStorage<T, N>>());
            unsafe {
                self.array.init_inline(p, bytes, 0);
            }
        } else if !self.array.owns_memory() {
            // Pure Rust provenance adaptation: each new exclusive owner borrow
            // refreshes the same inline address without changing capacity/size/own bits.
            self.array.fData = ptr::addr_of_mut!(self.storage).cast::<T>();
        }
    }
    pub fn init(self: core::pin::Pin<&mut Self>) {
        unsafe {
            self.get_unchecked_mut().bind_inline();
        }
    }
}

// Refresh the inline tag at destruction before TArray field drop reborrows its
// metadata. No initialized element is moved and the native allocation flag is unchanged.
impl<T, const N: usize, const M: bool> Drop for STArray<T, N, M> {
    fn drop(&mut self) {
        if !self.array.owns_memory() && self.array.capacity() != 0 {
            self.array.fData = ptr::addr_of_mut!(self.storage).cast::<T>();
        }
    }
}
// Source: include/private/SkTArray.h:772-790
// Constructor staging adaptation: new() makes an empty movable shell; once
// pinned these methods initialize the original inline/heap native owner exactly
// once. No initialized inline T is ever moved as padding of an unpinned shell.
impl<T: Clone, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn init_from_slice(self: core::pin::Pin<&mut Self>, src: &[T]) {
        let this = unsafe { self.get_unchecked_mut() };
        if !this.pending_inline() {
            std::process::abort();
        }
        let storage = ptr::addr_of_mut!(this.storage).cast::<u8>();
        let bytes = Self::INLINE_CAPACITY * mem::size_of::<T>();
        unsafe {
            this.array.init_inline_copy_slice(src, storage, bytes);
        }
    }
    pub fn init_clone(self: core::pin::Pin<&mut Self>, source: &Self) {
        self.init_from_slice(source.as_slice());
    }
    pub fn init_copy_array(self: core::pin::Pin<&mut Self>, source: &TArray<T, M>) {
        self.init_from_slice(source.as_slice());
    }
}
impl<T, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn init_reserve(self: core::pin::Pin<&mut Self>, count: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        if !this.pending_inline() {
            std::process::abort();
        }
        unsafe {
            this.bind_inline();
        }
        this.array.reserve_exact(count);
    }
}
// Source: include/private/SkTArray.h:792-796
impl<T, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn init_move(self: core::pin::Pin<&mut Self>, source: core::pin::Pin<&mut Self>) {
        if !self.as_ref().get_ref().pending_inline() {
            std::process::abort();
        }
        self.move_assign(source);
    }
    pub fn init_move_array(self: core::pin::Pin<&mut Self>, source: &mut TArray<T, M>) {
        if !self.as_ref().get_ref().pending_inline() {
            std::process::abort();
        }
        self.move_assign_array(source);
    }
}
// Source: include/private/SkTArray.h:798-806
impl<T: Clone, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn copy_assign(self: core::pin::Pin<&mut Self>, that: &Self) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.copy_assign(&that.array);
    }
    pub fn copy_assign_array(self: core::pin::Pin<&mut Self>, that: &TArray<T, M>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.copy_assign(that);
    }
}
// Source: include/private/SkTArray.h:808-816
impl<T, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn move_assign(self: core::pin::Pin<&mut Self>, that: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        let that = unsafe { that.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
            that.bind_inline();
        }
        this.array.move_assign(&mut that.array);
    }
    pub fn move_assign_array(self: core::pin::Pin<&mut Self>, that: &mut TArray<T, M>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.move_assign(that);
    }
}
// Source: include/private/SkTArray.h:818-820
// Native inherited TArray APIs are forwarded without exposing &mut TArray or
// DerefMut, which would permit moving an array that points into pinned storage.
impl<T, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn size(&self) -> i32 {
        self.array.size()
    }
    pub fn empty(&self) -> bool {
        self.array.empty()
    }
    pub fn capacity(&self) -> i32 {
        if self.pending_inline() {
            Self::INLINE_CAPACITY as i32
        } else {
            self.array.capacity()
        }
    }
    pub fn data(&self) -> *const T {
        if self.pending_inline() {
            ptr::addr_of!(self.storage).cast()
        } else {
            self.array.data()
        }
    }
    pub fn begin(&self) -> *const T {
        self.data()
    }
    pub fn end(&self) -> *const T {
        self.data().wrapping_add(self.size() as usize)
    }
    pub fn as_slice(&self) -> &[T] {
        self.array.as_slice()
    }
    pub fn at(&self, i: i32) -> &T {
        self.array.at(i)
    }
    pub fn front(&self) -> &T {
        self.array.front()
    }
    pub fn back(&self) -> &T {
        self.array.back()
    }
    pub fn fromBack(&self, i: i32) -> &T {
        self.array.fromBack(i)
    }
    pub fn push_back(self: core::pin::Pin<&mut Self>, value: T) -> &mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.push_back(value)
    }
    pub fn reset_with(self: core::pin::Pin<&mut Self>, n: i32, construct: impl FnMut() -> T) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.reset_with(n, construct);
    }
    pub fn emplace_back_with(
        self: core::pin::Pin<&mut Self>,
        construct: impl FnOnce() -> T,
    ) -> &mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.emplace_back_with(construct)
    }
    pub fn push_back_n_with(
        self: core::pin::Pin<&mut Self>,
        n: i32,
        construct: impl FnMut() -> T,
    ) -> &mut [T] {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.push_back_n_with(n, construct)
    }
    pub fn resize_back_with(self: core::pin::Pin<&mut Self>, n: i32, construct: impl FnMut() -> T) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.resize_back_with(n, construct);
    }
    pub fn pop_back(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.pop_back();
    }
    pub fn pop_back_n(self: core::pin::Pin<&mut Self>, n: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.pop_back_n(n);
    }
    pub fn clear(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.clear();
    }
    pub fn reserve(self: core::pin::Pin<&mut Self>, n: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.reserve(n);
    }
    pub fn reserve_exact(self: core::pin::Pin<&mut Self>, n: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.reserve_exact(n);
    }
    pub fn shrink_to_fit(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.shrink_to_fit();
    }
    pub fn removeShuffle(self: core::pin::Pin<&mut Self>, n: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.removeShuffle(n);
    }
    pub fn at_mut(self: core::pin::Pin<&mut Self>, i: i32) -> &mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.at_mut(i)
    }
    pub fn back_mut(self: core::pin::Pin<&mut Self>) -> &mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.back_mut()
    }
    pub fn as_mut_slice(self: core::pin::Pin<&mut Self>) -> &mut [T] {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.as_mut_slice()
    }
    pub fn data_mut(self: core::pin::Pin<&mut Self>) -> *mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.data_mut()
    }
    pub fn swap(self: core::pin::Pin<&mut Self>, that: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        let that = unsafe { that.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
            that.bind_inline();
        }
        this.array.swap(&mut that.array);
    }
    pub fn move_back(self: core::pin::Pin<&mut Self>, that: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        let that = unsafe { that.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
            that.bind_inline();
        }
        this.array.move_back(&mut that.array);
    }
}
impl<T: Clone, const N: usize, const M: bool> STArray<T, N, M> {
    pub fn push_back_clone(self: core::pin::Pin<&mut Self>, value: &T) -> &mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.push_back_clone(value)
    }
    // Native push_back(array[i]) alias case: derive the source after acquiring the
    // current exclusive Pin borrow, then preserve clone-before-reallocation order.
    pub fn push_back_clone_at(self: core::pin::Pin<&mut Self>, index: i32) -> &mut T {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        if index < 0 || index >= this.array.size() {
            std::process::abort();
        }
        let p = this.array.data().wrapping_add(index as usize);
        unsafe { this.array.push_back_clone_raw(p) }
    }
    pub fn reset_copy(self: core::pin::Pin<&mut Self>, src: &[T]) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.reset_copy(src);
    }
    pub fn push_back_slice(self: core::pin::Pin<&mut Self>, src: &[T]) -> &mut [T] {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.bind_inline();
        }
        this.array.push_back_slice(src)
    }
}

// Rust ownership/unwind adaptations: initialized-prefix rollback and exactly-once
// destruction. Normal native forward/reverse destruction and size ordering remain.
struct FreeGuard(*mut u8);
impl Drop for FreeGuard {
    fn drop(&mut self) {
        unsafe {
            sk_free(self.0);
        }
    }
}
struct InitGuard {
    size: *mut i32,
    initialized: i32,
    active: bool,
}
impl Drop for InitGuard {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                *self.size = self.initialized;
            }
        }
    }
}
struct RemovalGuard<T, const M: bool> {
    size: *mut i32,
    data: *mut T,
    dst: i32,
    src: i32,
    new_size: i32,
    shuffle: bool,
}
impl<T, const M: bool> Drop for RemovalGuard<T, M> {
    fn drop(&mut self) {
        unsafe {
            if self.shuffle {
                TArray::<T, M>::move_element_ptr(self.data, self.dst, self.src);
            }
            *self.size = self.new_size;
        }
    }
}
struct TailRemovalGuard<T> {
    size: *mut i32,
    data: *mut T,
    next: i32,
    stop: i32,
    active: bool,
}
impl<T> Drop for TailRemovalGuard<T> {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                *self.size = self.stop;
                while self.next >= self.stop {
                    let i = self.next;
                    self.next -= 1;
                    ptr::drop_in_place(self.data.add(i as usize));
                }
            }
        }
    }
}
struct ForwardDropGuard<T> {
    size: *mut i32,
    data: *mut T,
    next: i32,
    end: i32,
    active: bool,
}
impl<T> Drop for ForwardDropGuard<T> {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                *self.size = 0;
                while self.next < self.end {
                    let i = self.next;
                    self.next += 1;
                    ptr::drop_in_place(self.data.add(i as usize));
                }
            }
        }
    }
}
impl<T, const M: bool> Index<usize> for TArray<T, M> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        if i > i32::MAX as usize {
            std::process::abort();
        }
        self.at(i as i32)
    }
}
impl<T, const M: bool> IndexMut<usize> for TArray<T, M> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        if i > i32::MAX as usize {
            std::process::abort();
        }
        self.at_mut(i as i32)
    }
}
