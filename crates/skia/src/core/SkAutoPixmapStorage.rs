use crate::src::core::{
    SkData::SkData, SkImageInfo::SkImageInfo, SkMask::SkMask, SkPixmap::SkPixmap,
};
use std::sync::Arc;

// Rust allocator protocol glue: the native sk_malloc_canfail/sk_free backing and
// SkData::MakeFromMalloc use this same process malloc/free pair. No Box conversion.
unsafe extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}

// Source: src/core/SkAutoPixmapStorage.h:21-22
// Source: src/core/SkAutoPixmapStorage.h:88-88
// Composition preserves the original sole pixel owner. The descriptor can retain
// stale raw metadata after failed tryAlloc, so no safe Deref/byte view is exposed.
pub struct SkAutoPixmapStorage {
    pixmap: SkPixmap,
    fStorage: *mut u8,
}
impl SkAutoPixmapStorage {
    /// Safety: the descriptor's backing is still live (a successful tryAlloc or
    /// a live reset_with/reset_mask lease), and initialized before any pixel read.
    /// A failed tryAlloc after owning pixels invalidates that prior lease.
    pub unsafe fn pixmap(&self) -> &SkPixmap {
        &self.pixmap
    }
    pub fn info(&self) -> &SkImageInfo {
        self.pixmap.info()
    }
    pub fn rowBytes(&self) -> usize {
        self.pixmap.rowBytes()
    }
    pub fn computeByteSize(&self) -> usize {
        self.pixmap.computeByteSize()
    }

    // Source: src/core/SkAutoPixmapStorage.h:72-75
    pub fn reset(&mut self) {
        self.freeStorage();
        self.pixmap.reset();
    }
    // Source: src/core/SkAutoPixmapStorage.h:76-79
    /// Safety: addr remains live for all subsequent pixel operations; no prior
    /// lease may be used after this reset. The storage is borrowed, not adopted.
    pub unsafe fn reset_with(&mut self, info: &SkImageInfo, addr: *const u8, rb: usize) {
        self.freeStorage();
        self.pixmap.reset_with(info, addr, rb);
    }
    // Source: src/core/SkAutoPixmapStorage.h:81-84
    /// Safety: the mask's backing outlives every pixel operation through this view.
    pub unsafe fn reset_mask(&mut self, mask: &SkMask) -> bool {
        self.freeStorage();
        self.pixmap.reset_mask(mask)
    }
    // Source: src/core/SkAutoPixmapStorage.h:90-93
    fn freeStorage(&mut self) {
        unsafe {
            free(self.fStorage.cast());
        }
        self.fStorage = core::ptr::null_mut();
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:15-15
impl SkAutoPixmapStorage {
    pub fn new() -> Self {
        Self {
            pixmap: SkPixmap::default(),
            fStorage: core::ptr::null_mut(),
        }
    }
}
impl Default for SkAutoPixmapStorage {
    fn default() -> Self {
        Self::new()
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:17-19
impl Drop for SkAutoPixmapStorage {
    fn drop(&mut self) {
        self.freeStorage();
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:21-23
impl SkAutoPixmapStorage {
    pub fn move_from(other: &mut Self) -> Self {
        let mut value = Self::new();
        value.move_assign(other);
        value
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:25-33
impl SkAutoPixmapStorage {
    pub fn move_assign(&mut self, other: &mut Self) {
        // Native assignment overwrites fStorage without freeing the old pointer.
        // Preserve that behavior; no hidden extra owner or release is introduced.
        self.fStorage = other.fStorage;
        self.pixmap
            .reset_with(other.info(), self.fStorage, other.rowBytes());
        other.fStorage = core::ptr::null_mut();
        other.pixmap.reset();
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:35-41
impl SkAutoPixmapStorage {
    pub fn AllocSize(info: &SkImageInfo, row_bytes: Option<&mut usize>) -> usize {
        let rb = info.minRowBytes();
        if let Some(row_bytes) = row_bytes {
            *row_bytes = rb;
        }
        info.computeByteSize(rb)
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:43-58
impl SkAutoPixmapStorage {
    pub fn tryAlloc(&mut self, info: &SkImageInfo) -> bool {
        self.freeStorage();
        let mut rb = 0;
        let size = Self::AllocSize(info, Some(&mut rb));
        if SkImageInfo::ByteSizeOverflowed(size) {
            return false;
        }
        let pixels = unsafe { malloc(size).cast::<u8>() };
        if pixels.is_null() {
            return false;
        }
        unsafe {
            self.reset_with(info, pixels, rb);
        }
        self.fStorage = pixels;
        true
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:60-62
impl SkAutoPixmapStorage {
    pub fn alloc(&mut self, info: &SkImageInfo) {
        crate::SkASSERT_RELEASE!(@source b"src/core/SkAutoPixmapStorage.cpp\0".as_ptr(),61; @condition b"this->tryAlloc(info)\0".as_ptr(); self.tryAlloc(info));
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:64-74
impl SkAutoPixmapStorage {
    pub fn detachPixels(&mut self) -> *mut u8 {
        if self.fStorage.is_null() {
            return core::ptr::null_mut();
        }
        let data = self.fStorage;
        self.fStorage = core::ptr::null_mut();
        self.pixmap.reset();
        data
    }
}
// Source: src/core/SkAutoPixmapStorage.cpp:76-86
impl SkAutoPixmapStorage {
    /// Safety: every byte within computeByteSize has been initialized before
    /// publication as readable SkData. Ownership of the same malloc block moves.
    pub unsafe fn detachPixelsAsData(&mut self) -> Option<Arc<SkData>> {
        if self.fStorage.is_null() {
            return None;
        }
        let data = SkData::MakeFromMalloc(self.fStorage, self.computeByteSize());
        self.fStorage = core::ptr::null_mut();
        self.pixmap.reset();
        Some(data)
    }
}
