//! LayoutObject editing positions and node/object association.
//!
//! These queries enter inline/text traversal through assembly providers, so
//! they are physically and observably owned by the inline algorithm package.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:3454-3457
    pub fn CanBeSelectionLeafInternalBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn CanBeSelectionLeafInternal(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectCanBeSelectionLeafInternal(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2187-2187
    #[inline(never)]
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        unsafe { DispatchLayoutObjectPositionForPoint(self, point) }
    }

    // cpp: layoutng/internal/layout_object.h:2188-2189
    #[inline(never)]
    pub fn CreatePositionWithAffinity(
        &self,
        offset: i32,
        affinity: TextAffinity,
    ) -> PositionWithAffinity {
        unsafe { LayoutObjectCreatePositionWithAffinityProvider(self, offset, affinity) }
    }

    // cpp: layoutng/internal/layout_object.h:2190-2190
    #[inline(never)]
    pub fn CreatePositionWithAffinityDefault(&self, offset: i32) -> PositionWithAffinity {
        unsafe { LayoutObjectCreatePositionWithAffinityDefaultProvider(self, offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2191-2191
    #[inline(never)]
    pub fn FindPosition(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectFindPositionProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2192-2192
    #[inline(never)]
    pub fn FirstPositionInOrBeforeThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectFirstPositionInOrBeforeThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2193-2193
    #[inline(never)]
    pub fn LastPositionInOrAfterThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectLastPositionInOrAfterThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2194-2194
    #[inline(never)]
    pub fn PositionAfterThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectPositionAfterThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2195-2195
    #[inline(never)]
    pub fn PositionBeforeThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectPositionBeforeThisProvider(self) }
    }
}

// cpp: layoutng/internal/layout_object.h:3949-3952
#[inline(never)]
pub fn AssociatedLayoutObjectOf(
    node: &Node,
    offset_in_node: i32,
    side: LayoutObjectSide,
) -> *const LayoutObject {
    unsafe { AssociatedLayoutObjectOfProvider(node, offset_in_node, side) }
}

pub fn AssociatedLayoutObjectOfDefault(node: &Node, offset_in_node: i32) -> *const LayoutObject {
    AssociatedLayoutObjectOf(
        node,
        offset_in_node,
        LayoutObjectSide::kRemainingTextIfOnBoundary,
    )
}
