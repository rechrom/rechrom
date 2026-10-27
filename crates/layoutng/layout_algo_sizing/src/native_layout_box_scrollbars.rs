//! Native LayoutBox scrollbar and stable-gutter sizing.
//!
//! Scrollbar decisions are queried for many boxes during layout. The complete
//! geometry calculation stays out of line so sampling profiles attribute the
//! cumulative work to `layout_algo_sizing`, while scalar admissions remain
//! inlineable.

#![allow(non_snake_case)]

use foundation::{DynamicTo, EOverflow, LayoutUnit, OverlayScrollbarClipBehavior};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;

use super::super::layout_box::LayoutBox;
use super::super::layout_box_layout::{ShouldClampToContentBox, ShouldIncludeScrollbarGutter};
use super::super::layout_node_metadata::{Element, Node};
use super::super::scrollbar_orientation::ScrollbarOrientation;
use super::super::scrollbar_theme_metrics::ScrollbarThemeThickness;

// cpp: layoutng/internal/layout_box_layout.cc:41-50
fn HypotheticalScrollbarThickness(box_: &LayoutBox, _orientation: ScrollbarOrientation) -> i32 {
    let element = DynamicTo::<Element>(box_.GetNode());
    assert!(!box_
        .StyleRef()
        .HasCustomScrollbarStyle(element as *mut layoutng_style::style::forward::Element));
    ScrollbarThemeThickness(
        box_.ScrollbarThemeForLayout(),
        box_.StyleRef().UsedScrollbarWidth(),
    )
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_layout.cc:29-37
    pub fn CanSkipComputeScrollbars(&self) -> bool {
        self.CheckIsNotDestroyed();
        ((!self.IsLayoutView()
            && (self.StyleRef().IsOverflowVisibleAlongBothAxes() || !self.HasNonVisibleOverflow()))
            || {
                let scrollable_area = self.GetScrollableArea();
                !scrollable_area.is_null()
                    && !unsafe { &*scrollable_area }.HasHorizontalScrollbar()
                    && !unsafe { &*scrollable_area }.HasVerticalScrollbar()
            })
            && self.StyleRef().IsScrollbarGutterAuto()
    }

    // cpp: layoutng/internal/layout_box_layout.cc:54-114
    #[inline(never)]
    pub fn ComputeScrollbarsInternal(
        &self,
        clamp_to_content_box: ShouldClampToContentBox,
        overlay_scrollbar_clip_behavior: OverlayScrollbarClipBehavior,
        include_scrollbar_gutter: ShouldIncludeScrollbarGutter,
    ) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let mut scrollbars = PhysicalBoxStrut::default();
        let scrollable_area = self.GetScrollableArea();

        if include_scrollbar_gutter == ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter
            && self.HasScrollbarGutters(ScrollbarOrientation::kVerticalScrollbar)
        {
            let gutter_size = LayoutUnit::from_signed(HypotheticalScrollbarThickness(
                self,
                ScrollbarOrientation::kVerticalScrollbar,
            ));
            if self.ShouldPlaceVerticalScrollbarOnLeft() {
                scrollbars.left = gutter_size;
                if self.StyleRef().IsScrollbarGutterBothEdges() {
                    scrollbars.right = gutter_size;
                }
            } else {
                scrollbars.right = gutter_size;
                if self.StyleRef().IsScrollbarGutterBothEdges() {
                    scrollbars.left = gutter_size;
                }
            }
        } else if !scrollable_area.is_null() {
            let width = unsafe { &*scrollable_area }
                .VerticalScrollbarWidth(overlay_scrollbar_clip_behavior);
            if self.ShouldPlaceVerticalScrollbarOnLeft() {
                scrollbars.left = LayoutUnit::from_signed(width);
            } else {
                scrollbars.right = LayoutUnit::from_signed(width);
            }
        }

        if include_scrollbar_gutter == ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter
            && self.HasScrollbarGutters(ScrollbarOrientation::kHorizontalScrollbar)
        {
            let gutter_size = LayoutUnit::from_signed(HypotheticalScrollbarThickness(
                self,
                ScrollbarOrientation::kHorizontalScrollbar,
            ));
            scrollbars.bottom = gutter_size;
            if self.StyleRef().IsScrollbarGutterBothEdges() {
                scrollbars.top = gutter_size;
            }
        } else if !scrollable_area.is_null() {
            scrollbars.bottom = LayoutUnit::from_signed(
                unsafe { &*scrollable_area }
                    .HorizontalScrollbarHeight(overlay_scrollbar_clip_behavior),
            );
        }

        if scrollbars.left > LayoutUnit::default()
            && clamp_to_content_box == ShouldClampToContentBox::kClampToContentBox
        {
            let max_width = self.StitchedSize().width
                - (self.BorderOutsets() + self.PaddingOutsets()).HorizontalSum();
            scrollbars.left = std::cmp::min(scrollbars.left, max_width.ClampNegativeToZero());
        }
        scrollbars
    }

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
        self.ScrollbarThemeForLayout().uses_overlay_scrollbars
    }

    // cpp: layoutng/internal/layout_box_scrollbar_gutter.cc:43-74
    pub fn HasScrollbarGutters(&self, orientation: ScrollbarOrientation) -> bool {
        self.CheckIsNotDestroyed();
        if self.StyleRef().IsScrollbarGutterAuto() || !self.RespectsCSSOverflow() {
            return false;
        }
        debug_assert!(self.StyleRef().IsScrollbarGutterStable());

        // Viewport overflow:visible behaves as auto, and clip as hidden.
        // The root's gutter propagates independently of the body's overflow.
        let viewport_overflow = |overflow| {
            if self.IsLayoutView() {
                match overflow {
                    EOverflow::kVisible => EOverflow::kAuto,
                    EOverflow::kClip => EOverflow::kHidden,
                    value => value,
                }
            } else {
                overflow
            }
        };
        if orientation == ScrollbarOrientation::kVerticalScrollbar {
            let overflow = viewport_overflow(self.StyleRef().OverflowY());
            self.StyleRef().IsHorizontalWritingMode()
                && matches!(
                    overflow,
                    EOverflow::kAuto | EOverflow::kScroll | EOverflow::kHidden
                )
                && !self.UsesOverlayScrollbars()
                && (self.IsLayoutView()
                    || self.GetNode() != self.ViewportDefiningElementForLayout() as *mut Node)
        } else {
            let overflow = viewport_overflow(self.StyleRef().OverflowX());
            !self.StyleRef().IsHorizontalWritingMode()
                && matches!(
                    overflow,
                    EOverflow::kAuto | EOverflow::kScroll | EOverflow::kHidden
                )
                && !self.UsesOverlayScrollbars()
                && (self.IsLayoutView()
                    || self.GetNode() != self.ViewportDefiningElementForLayout() as *mut Node)
        }
    }
}
