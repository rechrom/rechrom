//! LayoutBox post-layout publication at the reciprocal assembly boundary.

use super::{LayoutBox, OverflowRecalcType};

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_layout.cc:116-128
    #[inline(never)]
    pub fn UpdateAfterLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetNeedsOverflowRecalcWithType(OverflowRecalcType::kOnlyVisualOverflowRecalc);
        self.SetScrollableOverflowFromLayoutResults();

        // Transform origins and percentage translations depend on the final
        // box size. Refresh the retained transform after fragments install and
        // before ancestor overflow is calculated.
        if self.HasLayer() {
            self.UpdateTransformForLayout();
        }

        let scrollable_area = self.GetScrollableArea();
        if !scrollable_area.is_null() {
            unsafe { &mut *scrollable_area }.UpdateAfterLayout();
        }
        self.ClearNeedsLayout();
    }
}
