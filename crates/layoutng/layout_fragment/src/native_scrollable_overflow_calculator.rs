#![allow(non_snake_case)]

use foundation::{
    gfx, DynamicTo, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, RuntimeEnabledFeatures,
    To, WritingDirectionMode,
};
use layout_fragment::fragment_item::FragmentItem;
use layout_fragment::fragment_items::FragmentItems;
use layout_fragment::fragment_items_builder::ItemWithOffset;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

use super::super::block_node::BlockNode;
use super::super::layout_box::LayoutBox;
use super::super::layout_object::{AncestorSkipInfo, LayoutObject};
use super::super::transform_utils::GetTransformForChildFragment;

#[path = "native_scrollable_overflow_calculation.rs"]
mod layout_fragment_native_scrollable_overflow_calculation;

// The source stores a value-copy of BlockNode. Recreating it from the same
// LayoutBox pointer preserves that handle while Rust avoids a raw memcpy.
// cpp: layoutng/internal/scrollable_overflow_calculator.h:20-104
pub struct ScrollableOverflowCalculator {
    node_: BlockNode,
    writing_direction_: WritingDirectionMode,
    is_scroll_container_: bool,
    is_view_: bool,
    scrolls_all_directions_: bool,
    has_left_overflow_: bool,
    has_top_overflow_: bool,
    has_non_visible_overflow_: bool,
    has_block_fragmentation_: bool,
    padding_: PhysicalBoxStrut,
    size_: PhysicalSize,
    padding_rect_: PhysicalRect,
    scrollable_overflow_: PhysicalRect,
}

impl ScrollableOverflowCalculator {
    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:25-67
    pub fn RecalculateScrollableOverflowForFragment(
        fragment: &PhysicalBoxFragment,
        has_block_fragmentation: bool,
    ) -> PhysicalRect {
        layout_fragment_native_scrollable_overflow_calculation::RecalculateScrollableOverflowForFragment(
            fragment,
            has_block_fragmentation,
        )
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:69-99
    pub fn new(
        node: &BlockNode,
        is_css_box: bool,
        has_block_fragmentation: bool,
        borders: &PhysicalBoxStrut,
        scrollbar: &PhysicalBoxStrut,
        padding: &PhysicalBoxStrut,
        size: PhysicalSize,
        writing_direction: WritingDirectionMode,
    ) -> Self {
        let node_ = node.clone();
        let is_scroll_container_ = is_css_box && node_.IsScrollContainer();
        let is_view_ = node_.IsView();
        let scrolls_all_directions_ = is_css_box && node_.IsOverscrollAreaParent();
        let has_left_overflow_ = is_css_box && node_.HasLeftOverflow();
        let has_top_overflow_ = is_css_box && node_.HasTopOverflow();
        let has_non_visible_overflow_ = is_css_box && node_.HasNonVisibleOverflow();
        let border_scrollbar = *borders + *scrollbar;
        let padding_rect_ = PhysicalRect::new(
            PhysicalOffset::new(border_scrollbar.left, border_scrollbar.top),
            PhysicalSize::new(
                (size.width - border_scrollbar.HorizontalSum()).ClampNegativeToZero(),
                (size.height - border_scrollbar.VerticalSum()).ClampNegativeToZero(),
            ),
        );
        Self {
            node_,
            writing_direction_: writing_direction,
            is_scroll_container_,
            is_view_,
            scrolls_all_directions_,
            has_left_overflow_,
            has_top_overflow_,
            has_non_visible_overflow_,
            has_block_fragmentation_: has_block_fragmentation,
            padding_: *padding,
            size_: size,
            padding_rect_,
            scrollable_overflow_: padding_rect_,
        }
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:101-118
    pub fn Result(&mut self, inflow_bounds: Option<PhysicalRect>) -> PhysicalRect {
        let Some(inflow_bounds) = inflow_bounds else {
            return self.scrollable_overflow_;
        };
        if !self.is_scroll_container_ {
            return self.scrollable_overflow_;
        }
        let start_offset = inflow_bounds.MinXMinYCorner()
            - PhysicalOffset::new(self.padding_.left, self.padding_.top);
        let end_offset = inflow_bounds.MaxXMaxYCorner()
            + PhysicalOffset::new(self.padding_.right, self.padding_.bottom);
        let inflow_overflow = PhysicalRect::new(
            start_offset,
            PhysicalSize::new(
                end_offset.left - start_offset.left,
                end_offset.top - start_offset.top,
            ),
        );
        let inflow_overflow = self.AdjustOverflowForScrollOrigin(&inflow_overflow);
        self.scrollable_overflow_.UniteEvenIfEmpty(&inflow_overflow);
        self.scrollable_overflow_
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:120-122
    pub fn AddTableSelfRect(&mut self) {
        self.AddOverflow(
            PhysicalRect::new(PhysicalOffset::default(), self.size_),
            false,
        );
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.h:44-53
    pub fn AddChild(&mut self, child_fragment: &PhysicalBoxFragment, offset: PhysicalOffset) {
        if self.is_view_ && child_fragment.IsFixedPositioned() {
            return;
        }
        let mut child_overflow = self.ScrollableOverflowForPropagation(child_fragment);
        child_overflow.offset += offset;
        self.AddOverflow(child_overflow, child_fragment.IsFragmentainerBox());
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:124-180
    fn AddItemsInternal<'a>(
        &mut self,
        layout_object: *const LayoutObject,
        items: impl Iterator<Item = &'a FragmentItem>,
    ) {
        layout_fragment_native_scrollable_overflow_calculation::AddItemsInternal(
            self,
            layout_object,
            items,
        )
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:182-186
    pub fn AddBuilderItems(
        &mut self,
        layout_object: *const LayoutObject,
        items: &[ItemWithOffset],
    ) {
        self.AddItemsInternal(layout_object, items.iter().map(|item| &item.item));
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:188-192
    pub fn AddFragmentItems(&mut self, box_fragment: &PhysicalBoxFragment, items: &FragmentItems) {
        self.AddItemsInternal(box_fragment.GetLayoutObject(), items.Items().iter());
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.h:75-84
    fn AddOverflow(&mut self, mut child_overflow: PhysicalRect, child_is_fragmentainer: bool) {
        if self.is_scroll_container_ {
            child_overflow = self.AdjustOverflowForScrollOrigin(&child_overflow);
        }
        if !child_overflow.IsEmpty() || child_is_fragmentainer {
            self.scrollable_overflow_.UniteEvenIfEmpty(&child_overflow);
        }
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:194-210
    fn AdjustOverflowForHanging(
        &self,
        line_rect: &PhysicalRect,
        mut overflow: PhysicalRect,
    ) -> PhysicalRect {
        if self.writing_direction_.IsHorizontal() {
            if overflow.offset.left < line_rect.offset.left {
                overflow.offset.left = line_rect.offset.left;
            }
            if overflow.Right() > line_rect.Right() {
                overflow.ShiftRightEdgeTo(line_rect.Right());
            }
        } else {
            if overflow.offset.top < line_rect.offset.top {
                overflow.offset.top = line_rect.offset.top;
            }
            if overflow.Bottom() > line_rect.Bottom() {
                overflow.ShiftBottomEdgeTo(line_rect.Bottom());
            }
        }
        overflow
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:212-236
    fn AdjustOverflowForScrollOrigin(&self, overflow: &PhysicalRect) -> PhysicalRect {
        let left_offset = if self.scrolls_all_directions_ || self.has_left_overflow_ {
            std::cmp::min(self.padding_rect_.Right(), overflow.offset.left)
        } else {
            std::cmp::max(self.padding_rect_.offset.left, overflow.offset.left)
        };
        let right_offset = if self.scrolls_all_directions_ || !self.has_left_overflow_ {
            std::cmp::max(self.padding_rect_.offset.left, overflow.Right())
        } else {
            std::cmp::min(self.padding_rect_.Right(), overflow.Right())
        };
        let top_offset = if self.scrolls_all_directions_ || self.has_top_overflow_ {
            std::cmp::min(self.padding_rect_.Bottom(), overflow.offset.top)
        } else {
            std::cmp::max(self.padding_rect_.offset.top, overflow.offset.top)
        };
        let bottom_offset = if self.scrolls_all_directions_ || !self.has_top_overflow_ {
            std::cmp::max(self.padding_rect_.offset.top, overflow.Bottom())
        } else {
            std::cmp::min(self.padding_rect_.Bottom(), overflow.Bottom())
        };
        PhysicalRect::new(
            PhysicalOffset::new(left_offset, top_offset),
            PhysicalSize::new(right_offset - left_offset, bottom_offset - top_offset),
        )
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:238-333
    fn ScrollableOverflowForPropagation(
        &self,
        child_fragment: &PhysicalBoxFragment,
    ) -> PhysicalRect {
        layout_fragment_native_scrollable_overflow_calculation::ScrollableOverflowForPropagation(
            self,
            child_fragment,
        )
    }
}
