//! Native bounding-box hierarchy interfaces and the concrete RTree factory.
use crate::include::core::SkRefCnt::{sk_sp, SkRefCntObject};
use crate::src::core::SkRect::SkRect;

// Source: include/core/SkBBHFactory.h:22-24
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Metadata {
    pub isDraw: bool,
}

// Source: include/core/SkBBHFactory.h:20-45
// Native inheritance maps to the real virtual refcount protocol. Implementors
// embed their sole SkRefCnt; the trait introduces no owner or second counter.
// The deleted copy constructor/assignment have no Clone/Copy implementation.
pub trait SkBBoxHierarchy: SkRefCntObject + Send + Sync {
    // Source: include/core/SkBBHFactory.h:29-29
    /// # Safety
    /// N is nonnegative and bounds has N elements. The caller excludes all
    /// concurrent insertion/search and follows the derived insertion contract.
    unsafe fn insert(&self, bounds: &[SkRect], N: i32);

    // Source: include/core/SkBBHFactory.h:30-30
    // Source: src/core/SkBBHFactory.cpp:17-20
    /// # Safety
    /// The same insertion lease and valid bounds prefix as insert is required.
    /// Metadata is intentionally ignored by this native default implementation.
    unsafe fn insert_with_metadata(&self, bounds: &[SkRect], _metadata: &[Metadata], N: i32) {
        unsafe {
            self.insert(bounds, N);
        }
    }

    // Source: include/core/SkBBHFactory.h:35-35
    fn search(&self, query: &SkRect, results: &mut Vec<i32>);
    // Source: include/core/SkBBHFactory.h:40-40
    fn bytesUsed(&self) -> usize;
}

// Source: include/core/SkBBHFactory.h:48-59
// The C++ virtual call operator becomes make. Box<dyn SkBBHFactory> retains
// concrete virtual destruction; this abstract interface has no copy constructor.
pub trait SkBBHFactory {
    // Source: include/core/SkBBHFactory.h:53-54
    fn make(&self) -> sk_sp<dyn SkBBoxHierarchy>;
}

// Source: include/core/SkBBHFactory.h:62-65
pub struct SkRTreeFactory;

impl SkBBHFactory for SkRTreeFactory {
    // Source: src/core/SkBBHFactory.cpp:13-15
    fn make(&self) -> sk_sp<dyn SkBBoxHierarchy> {
        let tree: Box<dyn SkBBoxHierarchy> = Box::new(crate::src::core::SkRTree::SkRTree::new());
        // Adopt the complete derived allocation with its original sole count.
        unsafe { sk_sp::from_raw(Box::into_raw(tree)) }
    }
}
