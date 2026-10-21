use crate::include::core::SkRefCnt::{
    SkRawDisposeProc, SkRefCnt, SkRefCntBase, SkRefCntDispose, SkRefCntObject,
};
// Source: src/codec/SkColorPalette.h:22-48
// Private representation reverses members then parent for native destruction.
pub struct SkColorPalette {
    // Source: src/codec/SkColorPalette.h:46-46
    fCount: i32,
    // Source: src/codec/SkColorPalette.h:45-45
    fColors: *mut u32,
    // Source: src/codec/SkColorPalette.h:22-22
    base: SkRefCnt,
}
impl SkColorPalette {
    // Source: src/codec/SkColorPalette.cpp:13-21
    /// Safety: colors supplies count initialized colors; the native copy and
    /// allocation preconditions hold, including libc's zero-length copy contract.
    pub unsafe fn new(colors: *const u32, count: i32) -> Self {
        let base = SkRefCnt::new();
        crate::SkASSERT!(@source b"src/codec/SkColorPalette.cpp\0".as_ptr(),14;@condition b"0 == count || colors\0".as_ptr();count==0||!colors.is_null());
        crate::SkASSERT!(@source b"src/codec/SkColorPalette.cpp\0".as_ptr(),15;@condition b"count >= 0 && count <= 256\0".as_ptr();count>=0&&count<=256);
        let fCount = count;
        let size = (count as usize).wrapping_mul(core::mem::size_of::<u32>());
        // Source: include/private/SkMalloc.h:40-40 (SK_MALLOC_THROW = 1 << 1).
        let fColors = unsafe { crate::src::ports::SkMemory_malloc::sk_malloc_flags(size, 1 << 1) }
            .cast::<u32>();
        unsafe {
            libc::memcpy(fColors.cast(), colors.cast(), size);
        }
        Self {
            fCount,
            fColors,
            base,
        }
    }
    // Source: src/codec/SkColorPalette.h:31-31
    pub fn count(&self) -> i32 {
        self.fCount
    }
    // Source: src/codec/SkColorPalette.h:36-39
    /// Safety: index is within the initialized native color array.
    pub unsafe fn at(&self, index: i32) -> u32 {
        crate::SkASSERT!(@source b"src/codec/SkColorPalette.h\0".as_ptr(),37;@condition b"fColors != nullptr && (unsigned)index < (unsigned)fCount\0".as_ptr();!self.fColors.is_null()&&(index as u32)<self.fCount as u32);
        unsafe { self.fColors.offset(index as isize).read() }
    }
    // Source: src/codec/SkColorPalette.h:42-42
    pub fn readColors(&self) -> *const u32 {
        self.fColors
    }
}
// Source: src/codec/SkColorPalette.cpp:23-25
impl Drop for SkColorPalette {
    fn drop(&mut self) {
        unsafe {
            crate::src::ports::SkMemory_malloc::sk_free(self.fColors.cast());
        }
    }
}
// Source: src/codec/SkColorPalette.h:22-22
impl core::ops::Deref for SkColorPalette {
    type Target = SkRefCnt;
    fn deref(&self) -> &SkRefCnt {
        &self.base
    }
}
unsafe impl SkRefCntObject for SkColorPalette {
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.base.ref_cnt_base()
    }
    fn ref_(&self) {
        self.base.ref_();
    }
    fn internal_dispose_raw_proc(&self) -> SkRawDisposeProc {
        SkRefCntDispose::<Self>
    }
}
// Source: src/codec/SkColorPalette.h:16-20
// Native explicitly thread-safe: colors/count are immutable after construction;
// only the sole atomic parent changes, and final destruction owns the allocation.
unsafe impl Send for SkColorPalette {}
unsafe impl Sync for SkColorPalette {}
