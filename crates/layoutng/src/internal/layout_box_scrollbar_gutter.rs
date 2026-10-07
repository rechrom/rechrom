#![allow(non_snake_case)]

use foundation::{DynamicTo, EOverflow};

use super::layout_box::LayoutBox;
use super::layout_node_metadata::{Element, Node};
use super::scrollbar_orientation::ScrollbarOrientation;

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_scrollbar_gutter.cc:32-41
    pub fn UsesOverlayScrollbars(&self) -> bool {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        // The style crate's forward Element type and this package's native
        // Element are the same layout-node boundary, pending crate connection.
        if self
            .StyleRef()
            .HasCustomScrollbarStyle(element as *mut layoutng_style::style::forward::Element)
        {
            return false;
        }
        if self.ScrollbarThemeForLayout().uses_overlay_scrollbars {
            return true;
        }
        false
    }

    // cpp: layoutng/internal/layout_box_scrollbar_gutter.cc:43-74
    pub fn HasScrollbarGutters(&self, orientation: ScrollbarOrientation) -> bool {
        self.CheckIsNotDestroyed();
        if self.StyleRef().IsScrollbarGutterAuto() {
            return false;
        }
        if !self.RespectsCSSOverflow() {
            return false;
        }
        debug_assert!(self.StyleRef().IsScrollbarGutterStable());

        if orientation == ScrollbarOrientation::kVerticalScrollbar {
            let overflow = self.StyleRef().OverflowY();
            self.StyleRef().IsHorizontalWritingMode()
                && (overflow == EOverflow::kAuto
                    || overflow == EOverflow::kScroll
                    || overflow == EOverflow::kHidden)
                && !self.UsesOverlayScrollbars()
                && self.GetNode() != self.ViewportDefiningElementForLayout() as *mut Node
        } else {
            let overflow = self.StyleRef().OverflowX();
            !self.StyleRef().IsHorizontalWritingMode()
                && (overflow == EOverflow::kAuto
                    || overflow == EOverflow::kScroll
                    || overflow == EOverflow::kHidden)
                && !self.UsesOverlayScrollbars()
                && self.GetNode() != self.ViewportDefiningElementForLayout() as *mut Node
        }
    }
}
