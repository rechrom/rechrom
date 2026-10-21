#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, EPosition, LayoutUnit, Length, MinimumValueForLength, WritingDirectionMode,
    WritingMode,
};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToPhysicalSize};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng/internal/relative_utils.cc:27-36
fn ResolveInset(length: &Length, size: LayoutUnit) -> Option<LayoutUnit> {
    if length.IsAuto() || (length.HasPercent() && size == kIndefiniteSize) {
        None
    } else {
        Some(MinimumValueForLength(length, size))
    }
}

// cpp: layoutng/internal/relative_utils.h:16-22
// cpp: layoutng/internal/relative_utils.cc:17-89
pub fn ComputeRelativeOffset(
    child_style: &ComputedStyle,
    container_writing_direction: WritingDirectionMode,
    available_size: &LogicalSize,
) -> LogicalOffset {
    if child_style.GetPosition() != EPosition::kRelative {
        return LogicalOffset::default();
    }

    let physical_size = ToPhysicalSize(
        *available_size,
        container_writing_direction.GetWritingMode(),
    );
    let mut left = ResolveInset(child_style.Left(), physical_size.width);
    let mut right = ResolveInset(child_style.Right(), physical_size.width);
    let mut top = ResolveInset(child_style.Top(), physical_size.height);
    let mut bottom = ResolveInset(child_style.Bottom(), physical_size.height);

    if left.is_none() && right.is_none() && top.is_none() && bottom.is_none() {
        return LogicalOffset::default();
    }

    if left.is_none() && right.is_none() {
        left = Some(LayoutUnit::default());
        right = Some(LayoutUnit::default());
    } else if left.is_none() {
        left = Some(-right.unwrap());
    } else if right.is_none() {
        right = Some(-left.unwrap());
    }

    if top.is_none() && bottom.is_none() {
        top = Some(LayoutUnit::default());
        bottom = Some(LayoutUnit::default());
    } else if top.is_none() {
        top = Some(-bottom.unwrap());
    } else if bottom.is_none() {
        bottom = Some(-top.unwrap());
    }

    let (left, right, top, bottom) = (left.unwrap(), right.unwrap(), top.unwrap(), bottom.unwrap());
    let is_ltr = container_writing_direction.IsLtr();
    match container_writing_direction.GetWritingMode() {
        WritingMode::kHorizontalTb => LogicalOffset::new(if is_ltr { left } else { right }, top),
        WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
            LogicalOffset::new(if is_ltr { top } else { bottom }, right)
        }
        WritingMode::kVerticalLr => LogicalOffset::new(if is_ltr { top } else { bottom }, left),
        WritingMode::kSidewaysLr => LogicalOffset::new(if is_ltr { bottom } else { top }, left),
        _ => panic!("unreachable writing mode"),
    }
}

// cpp: layoutng/internal/relative_utils.h:24-27
// cpp: layoutng/internal/relative_utils.cc:91-100
pub fn ComputeRelativeOffsetForBoxFragment(
    fragment: &PhysicalBoxFragment,
    container_writing_direction: WritingDirectionMode,
    available_size: &LogicalSize,
) -> LogicalOffset {
    let child_style = fragment.Style();
    debug_assert_eq!(child_style.GetPosition(), EPosition::kRelative);
    ComputeRelativeOffset(child_style, container_writing_direction, available_size)
}
