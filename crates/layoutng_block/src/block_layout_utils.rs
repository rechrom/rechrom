#![allow(non_snake_case)]

use foundation::{IsLtr, LayoutUnit, MinimumValueForLength};
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::fragmentation_utils::{IsBreakInside, ShouldIncludeBlockEndBorderPadding};
use layoutng::internal::layout_alignment_utils::BlockContentAlignment;
use layoutng::internal::layout_utils::ResolveContentAlignment;
use layoutng::internal::length_utils::LineOffsetForTextAlign;
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_block/block_layout_utils.h:16-19
// cpp: layoutng_block/block_layout_utils.cc:17-26
pub fn ComputeContentAlignmentForBlock(style: &ComputedStyle) -> BlockContentAlignment {
    if !style.IsDisplayBlockContainer() {
        return BlockContentAlignment::kStart;
    }
    let behave_like_table_cell = style.IsPageMarginBox();
    ResolveContentAlignment(style, behave_like_table_cell)
}

// cpp: layoutng_block/block_layout_utils.h:20-26
// cpp: layoutng_block/block_layout_utils.cc:28-69
pub fn CalculateOutOfFlowStaticInlineLevelOffset(
    container_style: &ComputedStyle,
    origin_bfc_offset: &BfcOffset,
    exclusion_space: &ExclusionSpace,
    child_available_inline_size: LayoutUnit,
) -> LayoutUnit {
    let direction = container_style.Direction();
    let opportunity = exclusion_space.FindLayoutOpportunityDefault(
        origin_bfc_offset,
        child_available_inline_size,
        direction,
    );
    let child_line_offset = if IsLtr(direction) {
        opportunity.rect.LineStartOffset()
    } else {
        opportunity.rect.LineEndOffset()
    };
    let relative_line_offset = child_line_offset - origin_bfc_offset.line_offset;
    // The out-of-flow child has zero inline size for this conversion.
    let mut inline_offset = if IsLtr(direction) {
        relative_line_offset
    } else {
        child_available_inline_size - relative_line_offset
    };
    let line_offset = LineOffsetForTextAlign(
        container_style.GetTextAlign(),
        direction,
        opportunity.rect.InlineSize(),
    );
    if IsLtr(direction) {
        inline_offset += line_offset;
    } else {
        inline_offset += opportunity.rect.InlineSize() - line_offset;
    }
    let text_indent = container_style.TextIndent();
    if !text_indent.IsZero() && !container_style.IsTextIndentHanging() {
        inline_offset += MinimumValueForLength(text_indent, child_available_inline_size);
    }
    inline_offset
}

// cpp: layoutng_block/block_layout_utils.h:27-32
// cpp: layoutng_block/block_layout_utils.cc:71-117
pub fn AlignBlockContent(
    style: &ComputedStyle,
    break_token: *const BlockBreakToken,
    content_block_size: LayoutUnit,
    builder: &mut BoxFragmentBuilder,
) {
    if IsBreakInside(break_token) {
        return;
    }
    let mut free_space = builder.FragmentBlockSize() - content_block_size;
    if style.AlignContentBlockCenter() {
        if builder.Node().IsButtonOrInputButton() {
            free_space = free_space.ClampNegativeToZero();
        }
        builder.MoveChildrenInDirection(free_space / 2, true, None);
        return;
    }
    if !ShouldIncludeBlockEndBorderPadding(builder) {
        return;
    }
    let alignment = ComputeContentAlignmentForBlock(style);
    if matches!(
        alignment,
        BlockContentAlignment::kSafeCenter | BlockContentAlignment::kSafeEnd
    ) {
        free_space = free_space.ClampNegativeToZero();
    }
    match alignment {
        BlockContentAlignment::kStart | BlockContentAlignment::kBaseline => {}
        BlockContentAlignment::kSafeCenter | BlockContentAlignment::kUnsafeCenter => {
            builder.MoveChildrenInDirection(free_space / 2, true, None);
        }
        BlockContentAlignment::kSafeEnd | BlockContentAlignment::kUnsafeEnd => {
            builder.MoveChildrenInDirection(free_space, true, None);
        }
    }
}
