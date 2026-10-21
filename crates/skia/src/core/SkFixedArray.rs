// UNVERIFIED new translation: user prohibited all validation commands.
use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};
/// Rust primitive constraint for native std::is_trivial_v<T>. Implementors are
/// Copy, have no destructor/padding observability, and permit native T() zero initialization.
pub unsafe trait NativeTrivial: Copy {}
macro_rules! trivial{($($t:ty),*)=>{$(unsafe impl NativeTrivial for $t{})*};}
trivial!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, f32, f64, bool);
unsafe impl<T: NativeTrivial, const M: usize> NativeTrivial for [T; M] {}
// Source: src/core/SkFixedArray.h:29-30
// Source: src/core/SkFixedArray.h:226-227
// Native unused slots remain uninitialized; no array of fabricated default T values.
#[repr(C)]
pub struct FixedArray<const N: usize, T: NativeTrivial> {
    fData: [MaybeUninit<T>; N],
    fSize: u8,
}
impl<const N: usize, T: NativeTrivial> FixedArray<N, T> {
    // Source: src/core/SkFixedArray.h:34-34
    pub fn new() -> Self {
        let () = Self::NATIVE_CONSTRAINTS;
        Self {
            fData: [const { MaybeUninit::uninit() }; N],
            fSize: 0,
        }
    }
    // Source: src/core/SkFixedArray.h:36-41
    pub unsafe fn from_values(values: &[T]) -> Self {
        let mut result = Self::new();
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),37; @condition b"values.size() <= N\0".as_ptr(); values.len()<=N);
        for &value in values {
            let index = result.fSize as usize;
            result.fSize += 1;
            result.fData[index].write(value);
        }
        result
    }
    // Source: src/core/SkFixedArray.h:43-48
    pub unsafe fn with_reserve(reserveCount: i32) -> Self {
        let value = Self::new();
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),46; @condition b"reserveCount >= 0\0".as_ptr(); reserveCount>=0);
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),47; @condition b"reserveCount <= N\0".as_ptr(); reserveCount as usize<=N);
        value
    }
    // Source: src/core/SkFixedArray.h:50-52
    pub unsafe fn from_slice(array: &[T]) -> Self {
        let mut result = Self::new();
        result.reset(array);
        result
    }
    // Source: src/core/SkFixedArray.h:54-56
    pub fn copy_construct(that: &Self) -> Self {
        unsafe { Self::from_slice(that.as_slice()) }
    }
    // Source: src/core/SkFixedArray.h:58-63
    pub fn assign(&mut self, that: &Self) -> &mut Self {
        if !core::ptr::eq(self, that) {
            unsafe {
                self.reset(that.as_slice());
            }
        }
        self
    }
    // Source: src/core/SkFixedArray.h:65-68
    pub fn at_mut(&mut self, index: usize) -> &mut T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),66; @condition b"index < fSize\0".as_ptr(); index<self.fSize as usize);
        &mut self.as_mut_slice()[index]
    }
    // Source: src/core/SkFixedArray.h:70-73
    pub fn at(&self, index: usize) -> &T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),71; @condition b"index < fSize\0".as_ptr(); index<self.fSize as usize);
        &self.as_slice()[index]
    }
    // Source: src/core/SkFixedArray.h:75-77
    pub fn equals(&self, that: &Self) -> bool {
        unsafe extern "C" {
            fn memcmp(
                a: *const core::ffi::c_void,
                b: *const core::ffi::c_void,
                n: usize,
            ) -> core::ffi::c_int;
        }
        self.fSize == that.fSize
            && unsafe {
                memcmp(
                    self.data().cast(),
                    that.data().cast(),
                    self.fSize as usize * core::mem::size_of::<T>(),
                ) == 0
            }
    }
    // Source: src/core/SkFixedArray.h:79-81
    pub fn not_equals(&self, that: &Self) -> bool {
        !self.equals(that)
    }
    // Source: src/core/SkFixedArray.h:83-85
    pub fn size(&self) -> i32 {
        self.fSize as i32
    }
    // Source: src/core/SkFixedArray.h:87-89
    pub fn empty(&self) -> bool {
        self.fSize == 0
    }
    // Source: src/core/SkFixedArray.h:91-93
    pub fn clear(&mut self) {
        self.fSize = 0;
    }
    // Source: src/core/SkFixedArray.h:95-99
    /// Safety: native capacity/trivial initialized-element and nonoverlapping memcpy leases hold.
    pub unsafe fn reset(&mut self, array: &[T]) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),96; @condition b"array.size() <= N\0".as_ptr(); array.len()<=N);
        self.fSize = array.len() as u8;
        core::ptr::copy_nonoverlapping(
            array.as_ptr(),
            self.fData.as_mut_ptr().cast::<T>(),
            array.len(),
        );
    }
    // Source: src/core/SkFixedArray.h:101-112
    /// Safety: native size/capacity preconditions hold, as for the mutators below.
    pub unsafe fn resize(&mut self, newSize: i32) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),102; @condition b"newSize >= 0\0".as_ptr(); newSize>=0);
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),103; @condition b"newSize <= N\0".as_ptr(); newSize as usize<=N);
        if self.fSize as i32 > newSize {
            self.fSize = newSize as u8;
        } else {
            while (self.fSize as i32) < newSize {
                let index = self.fSize as usize;
                self.fSize += 1;
                self.fData[index].write(core::mem::zeroed());
            }
        }
    }
    // Source: src/core/SkFixedArray.h:114-119
    pub unsafe fn push_back_default(&mut self) -> &mut T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),115; @condition b"fSize < N\0".as_ptr(); (self.fSize as usize)<N);
        let index = self.fSize as usize;
        self.fSize += 1;
        self.fData[index].write(core::mem::zeroed())
    }
    // Source: src/core/SkFixedArray.h:121-124
    pub unsafe fn push_back(&mut self, x: T) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),122; @condition b"fSize < N\0".as_ptr(); (self.fSize as usize)<N);
        let index = self.fSize as usize;
        self.fSize += 1;
        self.fData[index].write(x);
    }
    // Source: src/core/SkFixedArray.h:126-135
    pub unsafe fn push_back_n(&mut self, n: i32) -> *mut T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),127; @condition b"n >= 0\0".as_ptr(); n>=0);
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),128; @condition b"fSize + n <= N\0".as_ptr(); self.fSize as i32+n<=N as i32);
        let ptr = self.fData.as_mut_ptr().cast::<T>().add(self.fSize as usize);
        for index in 0..n {
            ptr.add(index as usize).write(core::mem::zeroed());
        }
        self.fSize = (self.fSize as i32 + n) as u8;
        ptr
    }
    // Source: src/core/SkFixedArray.h:137-146
    pub unsafe fn push_back_n_value(&mut self, n: i32, t: &T) -> *mut T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),138; @condition b"n >= 0\0".as_ptr(); n>=0);
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),139; @condition b"fSize + n <= N\0".as_ptr(); self.fSize as i32+n<=N as i32);
        let ptr = self.fData.as_mut_ptr().cast::<T>().add(self.fSize as usize);
        for index in 0..n {
            ptr.add(index as usize).write(*t);
        }
        self.fSize = (self.fSize as i32 + n) as u8;
        ptr
    }
    // Source: src/core/SkFixedArray.h:148-151
    pub unsafe fn pop_back(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),149; @condition b"fSize >= 1\0".as_ptr(); self.fSize>=1);
        self.fSize = self.fSize.wrapping_sub(1);
    }
    // Source: src/core/SkFixedArray.h:153-156
    pub unsafe fn pop_back_n(&mut self, n: i32) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),154; @condition b"fSize >= n\0".as_ptr(); self.fSize as i32>=n);
        self.fSize = (self.fSize as i32 - n) as u8;
    }
    // Source: src/core/SkFixedArray.h:158-165
    pub unsafe fn removeShuffle(&mut self, n: i32) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),159; @condition b"n < fSize\0".as_ptr(); n<self.fSize as i32);
        let last = self.fSize as i32 - 1;
        if n != last {
            let value = *self.fData[last as usize].assume_init_ref();
            self.fData[n as usize].write(value);
        }
        self.fSize = last as u8;
    }
    // Source: src/core/SkFixedArray.h:167-169
    pub fn data_mut(&mut self) -> *mut T {
        self.fData.as_mut_ptr().cast()
    }
    // Source: src/core/SkFixedArray.h:171-173
    pub fn data(&self) -> *const T {
        self.fData.as_ptr().cast()
    }
    // Source: src/core/SkFixedArray.h:175-177
    pub fn begin_mut(&mut self) -> *mut T {
        self.data_mut()
    }
    // Source: src/core/SkFixedArray.h:179-181
    pub fn begin(&self) -> *const T {
        self.data()
    }
    // Source: src/core/SkFixedArray.h:183-185
    pub fn end_mut(&mut self) -> *mut T {
        unsafe { self.data_mut().add(self.fSize as usize) }
    }
    // Source: src/core/SkFixedArray.h:187-189
    pub fn end(&self) -> *const T {
        unsafe { self.data().add(self.fSize as usize) }
    }
    // Source: src/core/SkFixedArray.h:191-194
    pub fn front_mut(&mut self) -> &mut T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),192; @condition b"fSize > 0\0".as_ptr(); self.fSize>0);
        let index = 0;
        self.at_mut(index)
    }
    // Source: src/core/SkFixedArray.h:196-199
    pub fn front(&self) -> &T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),197; @condition b"fSize > 0\0".as_ptr(); self.fSize>0);
        let index = 0;
        self.at(index)
    }
    // Source: src/core/SkFixedArray.h:201-204
    pub fn back_mut(&mut self) -> &mut T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),202; @condition b"fSize > 0\0".as_ptr(); self.fSize>0);
        let index = self.fSize as usize - 1;
        self.at_mut(index)
    }
    // Source: src/core/SkFixedArray.h:206-209
    pub fn back(&self) -> &T {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),207; @condition b"fSize > 0\0".as_ptr(); self.fSize>0);
        let index = self.fSize as usize - 1;
        self.at(index)
    }
    // Source: src/core/SkFixedArray.h:211-215
    pub fn reserve(&mut self, size: i32) {
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),213; @condition b"size >= 0\0".as_ptr(); size>=0);
        crate::SkASSERT!(@source b"src/core/SkFixedArray.h\0".as_ptr(),214; @condition b"size <= N\0".as_ptr(); size as usize<=N);
    }
    // Source: src/core/SkFixedArray.h:217-219
    pub const fn capacity(&self) -> i32 {
        N as i32
    }
    // Source: src/core/SkFixedArray.h:222-224
    // NativeTrivial is the explicit Rust std::is_trivial adapter; N's assertions
    // remain compile-time associated state, evaluated before constructing storage.
    const NATIVE_CONSTRAINTS: () = {
        assert!(!core::mem::needs_drop::<T>());
        assert!(core::mem::size_of::<T>() > 0);
        assert!(N > 0);
        assert!(N < 256);
    };
    // Language-only initialized-prefix views; the native unsafe preconditions of
    // all mutators preserve the same initialized fData[0..fSize] invariant.
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data(), self.fSize as usize) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data_mut(), self.fSize as usize) }
    }
}
impl<const N: usize, T: NativeTrivial> Default for FixedArray<N, T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize, T: NativeTrivial> Clone for FixedArray<N, T> {
    fn clone(&self) -> Self {
        Self::copy_construct(self)
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}
impl<const N: usize, T: NativeTrivial> PartialEq for FixedArray<N, T> {
    fn eq(&self, that: &Self) -> bool {
        self.equals(that)
    }
}
impl<const N: usize, T: NativeTrivial> Eq for FixedArray<N, T> {}
impl<const N: usize, T: NativeTrivial> Index<usize> for FixedArray<N, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.at(index)
    }
}
impl<const N: usize, T: NativeTrivial> IndexMut<usize> for FixedArray<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.at_mut(index)
    }
}
