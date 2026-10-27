//! Native scrollable-overflow calculation internals.
//!
//! Included by `scrollable_overflow_calculator` at the reciprocal assembly
//! boundary. Fragment recursion, fragment-item traversal, and ancestor clip
//! propagation retain separate fragment-qualified sampling frames.

use super::*;

// cpp: layoutng/internal/scrollable_overflow_calculator.cc:25-67
#[inline(never)]
pub(super) fn RecalculateScrollableOverflowForFragment(
    fragment: &PhysicalBoxFragment,
    has_block_fragmentation: bool,
) -> PhysicalRect {
    let box_ = To::<LayoutBox>(fragment.GetSelfOrContainerLayoutObject());
    let node = BlockNode::new(box_ as *mut LayoutBox);
    debug_assert!(!node.IsReplaced() || node.IsMedia());
    let writing_direction = node.Style().GetWritingDirection();
    let mut calculator = ScrollableOverflowCalculator::new(
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
            let mut child_overflow =
                RecalculateScrollableOverflowForFragment(box_fragment, has_block_fragmentation);
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

// cpp: layoutng/internal/scrollable_overflow_calculator.cc:124-180
#[inline(never)]
pub(super) fn AddItemsInternal<'a>(
    calculator: &mut ScrollableOverflowCalculator,
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
            calculator.scrollable_overflow_.UniteEvenIfEmpty(&line_rect);
            continue;
        }
        if item.IsText() {
            // cpp: layoutng/internal/scrollable_overflow_calculator.cc:153-163
            let mut child_overflow = *item.RectInContainerFragment();
            if has_hanging {
                child_overflow = calculator.AdjustOverflowForHanging(&line_rect, child_overflow);
            }
            calculator.AddOverflow(child_overflow, false);
            continue;
        }
        let child_box_fragment = item.BoxFragment();
        // cpp: layoutng/internal/scrollable_overflow_calculator.cc:165-178
        if !child_box_fragment.is_null() {
            let child_box_fragment = unsafe { &*child_box_fragment };
            let mut child_overflow =
                calculator.ScrollableOverflowForPropagation(child_box_fragment);
            child_overflow.offset += *item.OffsetInContainerFragment();
            if child_box_fragment.IsInlineBox() && has_hanging {
                child_overflow = calculator.AdjustOverflowForHanging(&line_rect, child_overflow);
            }
            calculator.AddOverflow(child_overflow, false);
        }
    }
}

// cpp: layoutng/internal/scrollable_overflow_calculator.cc:238-333
#[inline(never)]
pub(super) fn ScrollableOverflowForPropagation(
    calculator: &ScrollableOverflowCalculator,
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
    let container_object = unsafe { &*(calculator.node_.GetLayoutBox() as *const LayoutObject) };
    if let Some(transform) =
        GetTransformForChildFragment(child_fragment, container_object, calculator.size_)
    {
        overflow = PhysicalRect::EnclosingRect(&transform.MapRect(gfx::RectF::from(overflow)));
    }

    // cpp: layoutng/internal/scrollable_overflow_calculator.cc:299-330
    if calculator.has_block_fragmentation_
        && child_fragment.IsOutOfFlowPositioned()
        && !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
    {
        let container = calculator.node_.GetLayoutBox() as *const LayoutObject;
        let mut skip_info = AncestorSkipInfo::new(container);
        let child_layout_object = unsafe { &*child_fragment.GetLayoutObject() };
        let mut clipped_axes: OverflowClipAxes = kNoOverflowClip;
        let mut walker =
            child_layout_object.ContainingBlockWithSkipInfo(&mut skip_info) as *const LayoutObject;
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
