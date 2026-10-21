//! Native POD array storage and growth. Slots added without a source remain uninitialized;
//! typed callers must initialize them before any typed read. No element constructors/destructors.
use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::ptr;
const MALLOC_ALIGNMENT: usize = std::mem::align_of::<u128>();
impl SkTDStorage {
    // Source: include/private/SkTDArray.h:41-53
    pub fn empty(&self) -> bool {
        self.fSize == 0
    }
    pub fn clear(&mut self) {
        self.fSize = 0;
    }
    pub fn size(&self) -> i32 {
        self.fSize
    }
    pub fn size_bytes(&self) -> usize {
        self.bytes(self.fSize)
    }
    pub fn capacity(&self) -> i32 {
        self.fCapacity
    }
    pub fn data(&self) -> *mut u8 {
        self.fStorage
    }
    // Source: include/private/SkTDArray.h:70-73
    pub fn pop_back(&mut self) {
        crate::SkASSERT!(@source b"include/private/SkTDArray.h\0".as_ptr(),71; @condition b"fSize > 0\0".as_ptr(); self.fSize>0);
        self.fSize -= 1;
    }
    // Source: include/private/SkTDArray.h:75-78
    /// # Safety
    /// Same initialized-byte and element-width contract as storage_byte_equals.
    pub unsafe fn byte_not_equals(&self, other: &Self) -> bool {
        !unsafe { storage_byte_equals(self, other) }
    }
    // Source: include/private/SkTDArray.h:82-92
    fn bytes(&self, n: i32) -> usize {
        n as usize * self.fSizeOfT as usize
    }
    fn safe_bytes(&self, n: i32) -> usize {
        let size = n as usize;
        let sizeOfT = self.fSizeOfT as usize;
        if size > usize::MAX / sizeOfT {
            std::process::abort();
        }
        size * sizeOfT
    }
    fn address(&self, n: i32) -> *mut u8 {
        self.fStorage.wrapping_add(self.bytes(n))
    }
}
// Source: include/private/SkTDArray.h:104-107
pub struct SkTDStorage {
    fSizeOfT: i32,
    fStorage: *mut u8,
    fCapacity: i32,
    fSize: i32,
}
// Rust raw allocation substitution, retaining malloc's alignment and no initialized byte promise.
unsafe impl Send for SkTDStorage {}
unsafe impl Sync for SkTDStorage {}
// Source: include/private/SkTDArray.h:110-112
pub fn swap_storage(a: &mut SkTDStorage, b: &mut SkTDStorage) {
    a.swap(b);
}
impl<T: Copy> SkTDArray<T> {
    // Source: include/private/SkTDArray.h:119-121
    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() > 0 && std::mem::size_of::<T>() <= i32::MAX as usize);
        assert!(std::mem::align_of::<T>() <= MALLOC_ALIGNMENT);
        Self {
            fStorage: SkTDStorage::new(std::mem::size_of::<T>() as i32),
            marker: PhantomData,
        }
    }
    pub fn from_slice(src: &[T]) -> Self {
        assert!(src.len() <= i32::MAX as usize);
        let mut out = Self::new();
        out.fStorage = unsafe {
            SkTDStorage::from_raw(
                src.as_ptr().cast(),
                src.len() as i32,
                std::mem::size_of::<T>() as i32,
            )
        };
        out
    }
}
// Source: include/private/SkTDArray.h:124-129
impl<T: Copy> Clone for SkTDArray<T> {
    fn clone(&self) -> Self {
        Self::from_slice(self.as_slice())
    }
    fn clone_from(&mut self, src: &Self) {
        self.fStorage.clone_from(&src.fStorage);
    }
}
impl<T: Copy> SkTDArray<T> {
    // Source: include/private/SkTDArray.h:137-140
    /// # Safety
    /// Every compared byte, including POD padding, must be initialized in both arrays.
    /// This is byte equality, including floating-point representations, as in native memcmp.
    pub unsafe fn byte_equals(&self, other: &Self) -> bool {
        unsafe { storage_byte_equals(&self.fStorage, &other.fStorage) }
    }
    /// # Safety
    /// Same initialized-byte contract as byte_equals.
    pub unsafe fn byte_not_equals(&self, other: &Self) -> bool {
        !unsafe { self.byte_equals(other) }
    }
    // Source: include/private/SkTDArray.h:142-147
    pub fn swap(&mut self, that: &mut Self) {
        self.fStorage.swap(&mut that.fStorage);
    }
    pub fn empty(&self) -> bool {
        self.fStorage.empty()
    }
    // Source: include/private/SkTDArray.h:150-158
    pub fn size(&self) -> i32 {
        self.fStorage.size()
    }
    pub fn capacity(&self) -> i32 {
        self.fStorage.capacity()
    }
    pub fn size_bytes(&self) -> usize {
        self.fStorage.size_bytes()
    }
    // Source: include/private/SkTDArray.h:160-165
    pub fn data(&self) -> *const T {
        self.fStorage.data().cast()
    }
    pub fn data_mut(&mut self) -> *mut T {
        self.fStorage.data().cast()
    }
    pub fn begin(&self) -> *const T {
        self.data()
    }
    pub fn begin_mut(&mut self) -> *mut T {
        self.data_mut()
    }
    // wrapping_add also represents native's empty/null + zero without Rust inbounds requirements.
    pub fn end(&self) -> *const T {
        self.data().wrapping_add(self.size() as usize)
    }
    pub fn end_mut(&mut self) -> *mut T {
        self.data_mut().wrapping_add(self.size() as usize)
    }
    pub fn as_slice(&self) -> &[T] {
        let p = if self.empty() {
            ptr::NonNull::<T>::dangling().as_ptr()
        } else {
            self.data()
        };
        unsafe { std::slice::from_raw_parts(p, self.size() as usize) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let p = if self.empty() {
            ptr::NonNull::<T>::dangling().as_ptr()
        } else {
            self.data_mut()
        };
        unsafe { std::slice::from_raw_parts_mut(p, self.size() as usize) }
    }
}
// Source: include/private/SkTDArray.h:167-172
impl<T: Copy> Index<usize> for SkTDArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.as_slice()[index]
    }
}
impl<T: Copy> IndexMut<usize> for SkTDArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.as_mut_slice()[index]
    }
}
impl<T: Copy> SkTDArray<T> {
    // Source: include/private/SkTDArray.h:174-181
    pub fn back(&self) -> &T {
        self.as_slice().last().expect("empty SkTDArray")
    }
    pub fn back_mut(&mut self) -> &mut T {
        self.as_mut_slice().last_mut().expect("empty SkTDArray")
    }
    // Source: include/private/SkTDArray.h:183-189
    pub fn reset(&mut self) {
        self.fStorage.reset();
    }
    pub fn clear(&mut self) {
        self.fStorage.clear();
    }
    // Source: include/private/SkTDArray.h:195-201
    /// # Safety
    /// Every newly added slot must be fully initialized before any typed access or clone.
    pub unsafe fn resize(&mut self, count: i32) {
        self.fStorage.resize(count);
    }
    pub fn reserve(&mut self, n: i32) {
        self.fStorage.reserve(n);
    }
    // Source: include/private/SkTDArray.h:203-213
    /// # Safety
    /// Initialize all returned slots before any typed access or clone; capacity growth can
    /// invalidate previously returned pointers. No alias into this allocation may survive growth.
    pub unsafe fn append_one_uninit(&mut self) -> *mut T {
        self.fStorage.append_one();
        self.fStorage.address(self.size() - 1).cast()
    }
    /// # Safety
    /// The complete returned range must be initialized before typed access.
    pub unsafe fn append_uninit(&mut self, count: i32) -> *mut T {
        self.fStorage.append_count(count);
        self.fStorage.address(self.size() - count).cast()
    }
    pub fn append(&mut self, src: &[T]) -> *mut T {
        assert!(src.len() <= i32::MAX as usize);
        unsafe {
            self.fStorage
                .append_src(src.as_ptr().cast(), src.len() as i32)
                .cast()
        }
    }
    // Source: include/private/SkTDArray.h:215-220
    /// # Safety
    /// Initialize all inserted slots before typed access/clone. All prior element pointers
    /// may be invalidated. Existing initialized tail elements are moved with memmove semantics.
    pub unsafe fn insert_one_uninit(&mut self, index: i32) -> *mut T {
        self.fStorage.insert_one(index).cast()
    }
    /// # Safety
    /// The complete returned range must be initialized before typed access.
    pub unsafe fn insert_uninit(&mut self, index: i32, count: i32) -> *mut T {
        unsafe { self.fStorage.insert(index, count, ptr::null()).cast() }
    }
    pub fn insert(&mut self, index: i32, src: &[T]) -> *mut T {
        assert!(src.len() <= i32::MAX as usize);
        unsafe {
            self.fStorage
                .insert(index, src.len() as i32, src.as_ptr().cast())
                .cast()
        }
    }
    // Source: include/private/SkTDArray.h:222-228
    pub fn remove(&mut self, index: i32, count: i32) {
        self.fStorage.erase(index, count);
    }
    pub fn removeShuffle(&mut self, index: i32) {
        self.fStorage.removeShuffle(index);
    }
    // Source: include/private/SkTDArray.h:231-239
    pub fn push_back(&mut self, v: T) {
        unsafe {
            let p = self.append_uninit(1);
            p.write(v);
        }
    }
    pub fn pop_back(&mut self) {
        self.fStorage.pop_back();
    }
    pub fn shrink_to_fit(&mut self) {
        self.fStorage.shrink_to_fit();
    }
}
// Source: include/private/SkTDArray.h:242-242
pub struct SkTDArray<T: Copy> {
    fStorage: SkTDStorage,
    marker: PhantomData<T>,
}
impl<T: Copy> Default for SkTDArray<T> {
    fn default() -> Self {
        Self::new()
    }
}
// Source: include/private/SkTDArray.h:245-245
pub fn swap_array<T: Copy>(a: &mut SkTDArray<T>, b: &mut SkTDArray<T>) {
    a.swap(b);
}
// Language moves transfer allocation ownership without cloning or dropping POD elements.
impl SkTDStorage {
    // Source: src/core/SkTDArray.cpp:20-22
    pub fn new(sizeOfT: i32) -> Self {
        crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),21; @condition b"sizeOfT > 0\0".as_ptr(); sizeOfT>0);
        Self {
            fSizeOfT: sizeOfT,
            fStorage: ptr::null_mut(),
            fCapacity: 0,
            fSize: 0,
        }
    }
    // Source: src/core/SkTDArray.cpp:24-36
    /// # Safety
    /// src must address size*sizeOfT readable bytes (possibly containing POD padding).
    pub unsafe fn from_raw(src: *const u8, size: i32, sizeOfT: i32) -> Self {
        assert!(size >= 0 && sizeOfT > 0);
        let mut out = Self {
            fSizeOfT: sizeOfT,
            fStorage: ptr::null_mut(),
            fCapacity: size,
            fSize: size,
        };
        if size > 0 {
            crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),30; @condition b"src != nullptr\0".as_ptr(); !src.is_null());
            let bytes = out.safe_bytes(size);
            out.fStorage = unsafe { allocate_bytes(bytes) };
            unsafe {
                ptr::copy_nonoverlapping(src, out.fStorage, bytes);
            }
        }
        out
    }
}
// Source: src/core/SkTDArray.cpp:38-53
impl Clone for SkTDStorage {
    fn clone(&self) -> Self {
        unsafe { Self::from_raw(self.fStorage, self.fSize, self.fSizeOfT) }
    }
    fn clone_from(&mut self, that: &Self) {
        if !ptr::eq(self, that) {
            assert_eq!(self.fSizeOfT, that.fSizeOfT);
            if that.fSize <= self.fCapacity {
                self.fSize = that.fSize;
                if self.fSize > 0 {
                    unsafe {
                        ptr::copy_nonoverlapping(that.data(), self.fStorage, that.size_bytes());
                    }
                }
            } else {
                *self = that.clone();
            }
        }
    }
}
// Source: src/core/SkTDArray.cpp:69-71
impl Drop for SkTDStorage {
    fn drop(&mut self) {
        if !self.fStorage.is_null() {
            unsafe {
                dealloc(self.fStorage, byte_layout(self.safe_bytes(self.fCapacity)));
            }
        }
    }
}
impl SkTDStorage {
    // Source: src/core/SkTDArray.cpp:73-77
    pub fn reset(&mut self) {
        let sizeOfT = self.fSizeOfT;
        *self = Self::new(sizeOfT);
    }
    // Source: src/core/SkTDArray.cpp:79-84
    pub fn swap(&mut self, that: &mut Self) {
        crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),79; @condition b"fSizeOfT == that.fSizeOfT\0".as_ptr(); (self.fSizeOfT)==(that.fSizeOfT));
        std::mem::swap(&mut self.fStorage, &mut that.fStorage);
        std::mem::swap(&mut self.fCapacity, &mut that.fCapacity);
        std::mem::swap(&mut self.fSize, &mut that.fSize);
    }
    // Source: src/core/SkTDArray.cpp:86-92
    pub fn resize(&mut self, newSize: i32) {
        crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),87; @condition b"newSize >= 0\0".as_ptr(); newSize>=0);
        if newSize > self.fCapacity {
            self.reserve(newSize);
        }
        self.fSize = newSize;
    }
    // Source: src/core/SkTDArray.cpp:94-127
    pub fn reserve(&mut self, newCapacity: i32) {
        crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),95; @condition b"newCapacity >= 0\0".as_ptr(); newCapacity>=0);
        if newCapacity > self.fCapacity {
            let kMaxCount = i32::MAX;
            let mut expandedReserve = kMaxCount;
            if kMaxCount - newCapacity > 4 {
                let growth = 4 + ((newCapacity + 4) >> 2);
                if kMaxCount - newCapacity > growth {
                    expandedReserve = newCapacity + growth;
                }
            }
            if self.fSizeOfT == 1 {
                expandedReserve = expandedReserve
                    .checked_add(15)
                    .unwrap_or_else(|| std::process::abort())
                    & !15;
            }
            let oldCapacity = self.fCapacity;
            self.fCapacity = expandedReserve;
            let bytes = self.safe_bytes(self.fCapacity);
            self.fStorage =
                unsafe { reallocate_bytes(self.fStorage, self.safe_bytes(oldCapacity), bytes) };
        }
    }
    // Source: src/core/SkTDArray.cpp:129-142
    pub fn shrink_to_fit(&mut self) {
        if self.fCapacity != self.fSize {
            let oldCapacity = self.fCapacity;
            self.fCapacity = self.fSize;
            if self.fCapacity > 0 {
                self.fStorage = unsafe {
                    reallocate_bytes(
                        self.fStorage,
                        self.safe_bytes(oldCapacity),
                        self.safe_bytes(self.fCapacity),
                    )
                };
            } else {
                if !self.fStorage.is_null() {
                    unsafe {
                        dealloc(self.fStorage, byte_layout(self.safe_bytes(oldCapacity)));
                    }
                }
                self.fStorage = ptr::null_mut();
            }
        }
    }
    // Source: src/core/SkTDArray.cpp:144-155
    pub fn erase(&mut self, index: i32, count: i32) {
        assert!(count >= 0 && self.fSize >= count && index >= 0 && index <= self.fSize);
        if count > 0 {
            assert!(index <= self.fSize - count);
            let newCount = self.calculateSizeOrDie(-count);
            unsafe {
                self.moveTail(index, index + count, self.fSize);
            }
            self.resize(newCount);
        }
    }
    // Source: src/core/SkTDArray.cpp:157-164
    pub fn removeShuffle(&mut self, index: i32) {
        assert!(self.fSize > 0 && index >= 0 && index < self.fSize);
        let newCount = self.calculateSizeOrDie(-1);
        unsafe {
            self.moveTail(index, self.fSize - 1, self.fSize);
        }
        self.resize(newCount);
    }
    // Source: src/core/SkTDArray.cpp:166-176
    pub fn prepend(&mut self) -> *mut u8 {
        self.insert_one(0)
    }
    pub fn append_one(&mut self) {
        if self.fSize < self.fCapacity {
            self.fSize += 1;
        } else {
            self.insert_one(self.fSize);
        }
    }
    // Source: src/core/SkTDArray.cpp:178-186
    pub fn append_count(&mut self, count: i32) {
        crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),179; @condition b"count >= 0\0".as_ptr(); count>=0);
        if self.fCapacity - self.fSize >= count {
            self.fSize += count;
        } else {
            unsafe {
                self.insert(self.fSize, count, ptr::null());
            }
        }
    }
    // Source: src/core/SkTDArray.cpp:188-194
    /// # Safety
    /// Non-null src contains count readable elements and may not alias storage invalidated by grow.
    pub unsafe fn append_src(&mut self, src: *const u8, count: i32) -> *mut u8 {
        unsafe { self.insert(self.fSize, count, src) }
    }
    pub fn insert_one(&mut self, index: i32) -> *mut u8 {
        unsafe { self.insert(index, 1, ptr::null()) }
    }
    // Source: src/core/SkTDArray.cpp:196-212
    /// # Safety
    /// Non-null src addresses count readable elements and may not be invalidated by reallocation.
    pub unsafe fn insert(&mut self, index: i32, count: i32, src: *const u8) -> *mut u8 {
        assert!(index >= 0 && index <= self.fSize && count >= 0);
        if count > 0 {
            let oldCount = self.fSize;
            let newCount = self.calculateSizeOrDie(count);
            self.resize(newCount);
            unsafe {
                self.moveTail(index + count, index, oldCount);
                if !src.is_null() {
                    self.copySrc(index, src, count);
                }
            }
        }
        self.address(index)
    }
}
// Source: src/core/SkTDArray.cpp:214-216
/// # Safety
/// Both same-element-width stores contain initialized bytes throughout their live elements.
/// Padding must be initialized too: Copy alone does not provide this Rust guarantee.
pub unsafe fn storage_byte_equals(a: &SkTDStorage, b: &SkTDStorage) -> bool {
    a.size() == b.size()
        && (a.empty()
            || unsafe {
                std::slice::from_raw_parts(a.data(), a.bytes(a.size()))
                    == std::slice::from_raw_parts(b.data(), a.bytes(a.size()))
            })
}
impl SkTDStorage {
    // Source: src/core/SkTDArray.cpp:218-229
    fn calculateSizeOrDie(&self, delta: i32) -> i32 {
        if -self.fSize > delta {
            std::process::abort();
        }
        const _: () = assert!(u32::MAX >= i32::MAX as u32 + i32::MAX as u32);
        let testCount = (self.fSize as u32).wrapping_add(delta as u32);
        if testCount > i32::MAX as u32 {
            std::process::abort();
        }
        testCount as i32
    }
    // Source: src/core/SkTDArray.cpp:231-237
    unsafe fn moveTail(&mut self, to: i32, tailStart: i32, tailEnd: i32) {
        assert!(
            to >= 0
                && to <= self.fSize
                && tailStart >= 0
                && tailStart <= tailEnd
                && tailEnd <= self.fSize
        );
        if to != tailStart && tailStart != tailEnd {
            unsafe {
                self.copySrc(to, self.address(tailStart), tailEnd - tailStart);
            }
        }
    }
    // Source: src/core/SkTDArray.cpp:239-242
    unsafe fn copySrc(&mut self, dstIndex: i32, src: *const u8, count: i32) {
        crate::SkASSERT!(@source b"src/core/SkTDArray.cpp\0".as_ptr(),240; @condition b"count > 0\0".as_ptr(); count>0);
        unsafe {
            ptr::copy(src, self.address(dstIndex), self.bytes(count));
        }
    }
}
// Rust allocator adaptation: original throw-on-failure raw malloc/realloc/free lifecycle.
fn byte_layout(bytes: usize) -> Layout {
    Layout::from_size_align(bytes, MALLOC_ALIGNMENT).unwrap_or_else(|_| std::process::abort())
}
unsafe fn allocate_bytes(bytes: usize) -> *mut u8 {
    let layout = byte_layout(bytes);
    let p = unsafe { alloc(layout) };
    if p.is_null() {
        handle_alloc_error(layout);
    }
    p
}
unsafe fn reallocate_bytes(p: *mut u8, oldBytes: usize, newBytes: usize) -> *mut u8 {
    if p.is_null() {
        unsafe { allocate_bytes(newBytes) }
    } else {
        let new = unsafe { realloc(p, byte_layout(oldBytes), newBytes) };
        if new.is_null() {
            handle_alloc_error(byte_layout(newBytes));
        }
        new
    }
}
