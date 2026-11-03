//! LayoutBox post-layout publication at the reciprocal assembly boundary.

use foundation::PaintInvalidationReason;

use super::{LayoutBox, OverflowRecalcType};

impl LayoutBox {
    // Source: core/layout/layout_box.cc:2280-2297.
    pub fn SizeChanged(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetScrollableAreaSizeChanged(true);
        if !self.NeedsLayout() {
            self.SetShouldCheckForPaintInvalidation();
        }
        if self.HasFlippedBlocksWritingMode() {
            if self.ChildrenInline() {
                self.SetSubtreeShouldDoFullPaintInvalidation(PaintInvalidationReason::kLayout);
            } else {
                self.SetSubtreeShouldCheckForPaintInvalidation();
            }
        }
    }

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
            let layer = self.Layer();
            self.UpdateTransform();
            unsafe { &mut *layer }.UpdateScrollingAfterLayout();
        }
        self.ClearNeedsLayout();
    }
}
