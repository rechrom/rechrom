//! LayoutObject selection state and retained-fragment geometry queries.
//!
//! Selection rectangles may aggregate retained geometry, so that entry point
//! keeps a fragment-owned sampling frame. Fixed-cost state and virtual-policy
//! predicates remain inlineable because they are called per object.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2593-2593
    pub fn CanUpdateSelectionOnRootLineBoxes(&self) -> bool {
        unsafe { LayoutObjectCanUpdateSelectionOnRootLineBoxesProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2612-2612
    #[inline(never)]
    pub fn AbsoluteSelectionRect(&self) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteSelectionRectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2614-2614
    pub fn CanBeSelectionLeaf(&self) -> bool {
        unsafe { LayoutObjectCanBeSelectionLeafProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2615-2615
    pub fn IsSelected(&self) -> bool {
        unsafe { LayoutObjectIsSelectedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2616-2616
    pub fn IsSelectable(&self) -> bool {
        unsafe { LayoutObjectIsSelectableProvider(self) }
    }
}
