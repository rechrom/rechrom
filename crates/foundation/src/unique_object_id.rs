use std::sync::atomic::{AtomicU64, Ordering};

// cpp: foundation/graphics_types/graphics/unique_object_id.h:9-10
pub type UniqueObjectId = u64;

// cpp: foundation/graphics_types/graphics/compositor_element_id.cc:11-14
// The C++ static counter is process-global. Atomic storage preserves that
// sequence when Rust callers reach it from more than one thread.
#[allow(non_snake_case)]
pub fn NewUniqueObjectId() -> UniqueObjectId {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}
