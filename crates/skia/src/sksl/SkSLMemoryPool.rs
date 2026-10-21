use crate::src::core::SkArenaAlloc::SkSTArenaAlloc;
use core::{cell::UnsafeCell, pin::Pin};
impl MemoryPool {
    // Source: src/sksl/SkSLMemoryPool.h:20-22
    pub fn Make() -> Pin<Box<Self>> {
        Box::pin(Self {
            fArena: UnsafeCell::new(SkSTArenaAlloc::new(32768)),
        })
    }
    // Source: src/sksl/SkSLMemoryPool.h:23-25
    /// Safety: pool remains pinned/live; allocation is serialized on its attached
    /// thread, and no live borrow aliases this arena's allocator state.
    pub unsafe fn allocate(self: Pin<&Self>, size: usize) -> *mut u8 {
        let arena = Pin::new_unchecked(&mut *self.fArena.get());
        arena.arena().makeBytesAlignedTo(size, kAlignment)
    }
    // Source: src/sksl/SkSLMemoryPool.h:26-28
    pub fn release(&self, _ptr: *mut u8) { /* Native arena deliberately never reclaims individual allocations. */
    }
}
// Source: src/sksl/SkSLMemoryPool.h:35-39
// Selected platform C max_align_t: Apple ARM64=8, non-Apple supported64bit
// ABIs=16. Explicit SK_FORCE_8_BYTE_ALIGNMENT branch remains available.
#[cfg(any(SK_FORCE_8_BYTE_ALIGNMENT, target_vendor = "apple"))]
pub const kAlignment: usize = 8;
#[cfg(not(any(SK_FORCE_8_BYTE_ALIGNMENT, target_vendor = "apple")))]
pub const kAlignment: usize = 16;
// Source: src/sksl/SkSLMemoryPool.h:41-41
// Adapted: native unique_ptr's stable address becomes Pin<Box<MemoryPool>>;
// UnsafeCell permits actual thread-local raw allocation without invented owners.
pub struct MemoryPool {
    fArena: UnsafeCell<SkSTArenaAlloc<65536>>,
}
