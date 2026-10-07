#![allow(non_snake_case)]

use foundation::{
    gfx, DynamicTo, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, RuntimeEnabledFeatures,
    To, WritingDirectionMode,
};
use layoutng_fragment_tree::fragment_item::FragmentItem;
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_fragment_tree::fragment_items_builder::ItemWithOffset;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;

use super::block_node::BlockNode;
use super::layout_box::LayoutBox;
use super::layout_object::{AncestorSkipInfo, LayoutObject};
use super::transform_utils::GetTransformForChildFragment;

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
        let box_ = To::<LayoutBox>(fragment.GetSelfOrContainerLayoutObject());
        let node = BlockNode::new(box_ as *mut LayoutBox);
        debug_assert!(!node.IsReplaced() || node.IsMedia());
        let writing_direction = node.Style().GetWritingDirection();
        let mut calculator = Self::new(
            &node,
            fragment.IsCSSBox(),
            has_block_fragmentation,
            &fragment.Borders(),
            &fragment.Scrollbar(),
            &fragment.Padding(),
            fragment.Size(),
            writing_direction,
        );
        let items = fragment.Items();
        if !items.is_null() {
            calculator.AddFragmentItems(fragment, unsafe { &*items });
        }
        for child in fragment.PostLayoutChildren().iter() {
            let child_fragment = child.fragment.Get();
            let box_fragment = DynamicTo::<PhysicalBoxFragment>(child_fragment);
            if box_fragment.is_null() {
                continue;
            }
            let box_fragment = unsafe { &*box_fragment };
            if box_fragment.IsFragmentainerBox() {
                let mut child_overflow = Self::RecalculateScrollableOverflowForFragment(
                    box_fragment,
                    has_block_fragmentation,
                );
                child_overflow.offset += child.offset;
                calculator.AddOverflow(child_overflow, true);
            } else {
                calculator.AddChild(box_fragment, child.offset);
            }
        }
        if !fragment.TableCollapsedBorders().is_null() {
            calculator.AddTableSelfRect();
        }
        calculator.Result(fragment.InflowBounds())
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
        let mut has_hanging = false;
        let mut line_rect = PhysicalRect::default();
        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:131-135
        if !layout_object.is_null() && unsafe { &*layout_object }.IsLayoutTextCombine() {
            return;
        }
        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:137-151
        for item in items {
            if item.IsHiddenForPaint() {
                continue;
            }
            let line_box = item.LineBoxFragment();
            if !line_box.is_null() {
                has_hanging = unsafe { &*line_box }.HasHanging();
                line_rect = *item.RectInContainerFragment();
                if line_rect.IsEmpty() {
                    continue;
                }
                self.scrollable_overflow_.UniteEvenIfEmpty(&line_rect);
                continue;
            }
            if item.IsText() {
                // cpp: layoutng/internal/scrollable_overflow_calculator.cc:153-163
                let mut child_overflow = *item.RectInContainerFragment();
                if has_hanging {
                    child_overflow = self.AdjustOverflowForHanging(&line_rect, child_overflow);
                }
                self.AddOverflow(child_overflow, false);
                continue;
            }
            let child_box_fragment = item.BoxFragment();
            // cpp: layoutng/internal/scrollable_overflow_calculator.cc:165-178
            if !child_box_fragment.is_null() {
                let child_box_fragment = unsafe { &*child_box_fragment };
                let mut child_overflow = self.ScrollableOverflowForPropagation(child_box_fragment);
                child_overflow.offset += *item.OffsetInContainerFragment();
                if child_box_fragment.IsInlineBox() && has_hanging {
                    child_overflow = self.AdjustOverflowForHanging(&line_rect, child_overflow);
                }
                self.AddOverflow(child_overflow, false);
                continue;
            }
        }
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
        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:249-256
        if child_fragment.IsHiddenForPaint() {
            return PhysicalRect::default();
        }
        if !child_fragment.IsCSSBox() {
            return child_fragment.ScrollableOverflow();
        }

        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:258-264
        let mut overflow = PhysicalRect::new(PhysicalOffset::default(), child_fragment.Size());
        let ignore_scrollable_overflow = child_fragment.ShouldApplyLayoutContainment()
            || child_fragment.IsInlineBox()
            || (child_fragment.ShouldClipOverflowAlongBothAxis()
                && !child_fragment.ShouldApplyOverflowClipMargin());

        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:266-290
        if !ignore_scrollable_overflow {
            let mut child_overflow = child_fragment.ScrollableOverflow();
            if child_fragment.HasNonVisibleOverflow() {
                let overflow_clip_axes = child_fragment.GetOverflowClipAxes();
                if child_fragment.ShouldApplyOverflowClipMargin() {
                    debug_assert_eq!(overflow_clip_axes, kOverflowClipBothAxis);
                    let mut child_overflow_rect =
                        PhysicalRect::new(PhysicalOffset::default(), child_fragment.Size());
                    child_overflow_rect.Expand(&child_fragment.OverflowClipMarginOutsets());
                    child_overflow.Intersect(&child_overflow_rect);
                } else {
                    if overflow_clip_axes & kOverflowClipX != 0 {
                        child_overflow.offset.left = LayoutUnit::default();
                        child_overflow.size.width = child_fragment.Size().width;
                    }
                    if overflow_clip_axes & kOverflowClipY != 0 {
                        child_overflow.offset.top = LayoutUnit::default();
                        child_overflow.size.height = child_fragment.Size().height;
                    }
                }
            }
            overflow.UniteEvenIfEmpty(&child_overflow);
        }

        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:292-297
        let container_object = unsafe { &*(self.node_.GetLayoutBox() as *const LayoutObject) };
        if let Some(transform) =
            GetTransformForChildFragment(child_fragment, container_object, self.size_)
        {
            overflow = PhysicalRect::EnclosingRect(&transform.MapRect(gfx::RectF::from(overflow)));
        }

        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:299-330
        if self.has_block_fragmentation_
            && child_fragment.IsOutOfFlowPositioned()
            && !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
        {
            let container = self.node_.GetLayoutBox() as *const LayoutObject;
            let mut skip_info = AncestorSkipInfo::new(container);
            let child_layout_object = unsafe { &*child_fragment.GetLayoutObject() };
            let mut clipped_axes: OverflowClipAxes = kNoOverflowClip;
            let mut walker = child_layout_object.ContainingBlockWithSkipInfo(&mut skip_info)
                as *const LayoutObject;
            while walker != container && !skip_info.AncestorSkipped() {
                let object = unsafe { &*walker };
                let axes_to_clip = object.GetOverflowClipAxes();
                if axes_to_clip != 0 {
                    if axes_to_clip & kOverflowClipX != 0 {
                        overflow.offset.left = LayoutUnit::default();
                        overflow.size.width =
                            std::cmp::min(overflow.size.width, LayoutUnit::from_signed(1));
                    }
                    if axes_to_clip & kOverflowClipY != 0 {
                        overflow.offset.top = LayoutUnit::default();
                        overflow.size.height =
                            std::cmp::min(overflow.size.height, LayoutUnit::from_signed(1));
                    }
                    clipped_axes |= axes_to_clip;
                    if clipped_axes == kOverflowClipBothAxis {
                        break;
                    }
                }
                walker = object.ContainingBlockWithSkipInfo(&mut skip_info) as *const LayoutObject;
            }
        }

        overflow
    }
}
