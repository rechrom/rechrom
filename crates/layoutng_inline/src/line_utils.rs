// C++: layoutng_inline/line_utils.h and layoutng_inline/line_utils.cc.
#![allow(non_snake_case)]

use font_engine::fonts::font_height::FontHeight;
use foundation::{EPosition, LayoutUnit};
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::editing::forward::PositionWithAffinity;
use layoutng::internal::relative_utils::ComputeRelativeOffset;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/line_utils.h:24-35
// These two declarations have no definition anywhere in the source checkout.
// Retain their external ABI without manufacturing caret-position behavior.
unsafe extern "Rust" {
    pub fn NGContainingLineBoxOf(position: &PositionWithAffinity) -> InlineCursor;
    pub fn InSameNGLineBox(first: &PositionWithAffinity, second: &PositionWithAffinity) -> bool;
}

// cpp: layoutng_inline/line_utils.h:17-19
// cpp: layoutng_inline/line_utils.cc:12-36
pub fn ComputeRelativeOffsetForInline(
    space: &ConstraintSpace,
    child_style: &ComputedStyle,
) -> LogicalOffset {
    if child_style.GetPosition() != EPosition::kRelative {
        return LogicalOffset::default();
    }

    let writing_direction = space.GetWritingDirection();
    let mut relative_offset =
        ComputeRelativeOffset(child_style, writing_direction, &space.AvailableSize());
    if writing_direction.IsRtl() {
        relative_offset.inline_offset = -relative_offset.inline_offset;
    }
    if writing_direction.IsFlippedLines() {
        relative_offset.block_offset = -relative_offset.block_offset;
    }
    relative_offset
}

// cpp: layoutng_inline/line_utils.h:20-22
// cpp: layoutng_inline/line_utils.cc:38-63
pub fn ComputeRelativeOffsetForOOFInInline(
    space: &ConstraintSpace,
    child_style: &ComputedStyle,
) -> LogicalOffset {
    if child_style.GetPosition() != EPosition::kRelative {
        return LogicalOffset::default();
    }

    let writing_direction = space.GetWritingDirection();
    let mut relative_offset =
        ComputeRelativeOffset(child_style, writing_direction, &space.AvailableSize());
    if writing_direction.IsRtl() {
        relative_offset.inline_offset = -relative_offset.inline_offset;
    }
    relative_offset
}

// cpp: layoutng_inline/line_utils.h:37-42
// cpp: layoutng_inline/line_utils.cc:65-74
pub fn CalculateLeadingSpace(line_height: &LayoutUnit, current_height: &FontHeight) -> FontHeight {
    let remaining = *line_height - current_height.LineHeight();
    // The C++ function floors the ascent half before assigning the remainder
    // to descent, preserving odd fixed-point units and legacy text dumps.
    let ascent = LayoutUnit::from_signed((remaining / 2i32).Floor());
    let descent = remaining - ascent;
    FontHeight::new(ascent, descent)
}
