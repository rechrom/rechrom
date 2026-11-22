// C++: layoutng_inline/line_utils.h and layoutng_inline/line_utils.cc.
#![allow(non_snake_case)]

#[allow(unused_imports)]
use crate::algorithms::*;
use font_engine::fonts::font_height::FontHeight;
use foundation::{EPosition, LayoutUnit};
use layout_base::internal::constraint_space::ConstraintSpace;
use layout_base::internal::relative_utils::ComputeRelativeOffset;
use layout_geometry::geometry::logical_offset::LogicalOffset;
use layout_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/line_utils.h:24-35
// NGContainingLineBoxOf / InSameNGLineBox: declared in the source with no
// definition in the checkout and never referenced here; the declarations are
// omitted (no link-time symbol). Editing uses
// `layout_output_editing_queries` (line_utils.cc:12-24) instead.

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
    // cpp: line_utils.cc:39-42. Blink floors both positive and negative
    // half-leading for compatibility with its legacy line-box geometry.
    let ascent = LayoutUnit::from_signed((remaining / 2i32).Floor());
    let descent = remaining - ascent;
    FontHeight::new(ascent, descent)
}
