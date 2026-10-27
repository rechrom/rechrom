#![allow(non_snake_case)]

use foundation::{LogicalToLogical, RuntimeEnabledFeatures, WritingDirectionMode};
use layoutng_geometry::geometry::static_position::{BlockEdge, InlineEdge};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{
    ContentDistributionType, ContentPosition, EVerticalAlign, ItemPosition, OverflowAlignment,
};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;

use super::block_node::BlockNode;

// cpp: layoutng/internal/layout_alignment_utils.h:18-25
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockContentAlignment {
    kStart,
    kBaseline,
    kSafeCenter,
    kUnsafeCenter,
    kSafeEnd,
    kUnsafeEnd,
}

// Shared alignment policy is normal layout work. Keep it close to the type it
// returns and inline it into the concrete block/table/inline/OOF caller. Cache
// admission may query this policy, but does not own it.
// cpp: layoutng/internal/layout_utils.cc:20-108
#[inline(always)]
pub fn ResolveContentAlignment(
    style: &ComputedStyle,
    behave_like_table_cell: bool,
) -> BlockContentAlignment {
    let alignment = style.AlignContent();
    let mut position = alignment.GetPosition();
    let mut overflow = alignment.Overflow();
    match alignment.Distribution() {
        ContentDistributionType::kDefault => {}
        ContentDistributionType::kSpaceBetween | ContentDistributionType::kStretch => {
            position = ContentPosition::kFlexStart;
        }
        ContentDistributionType::kSpaceAround | ContentDistributionType::kSpaceEvenly => {
            overflow = OverflowAlignment::kSafe;
            position = ContentPosition::kCenter;
        }
    }
    if position == ContentPosition::kLastBaseline {
        overflow = OverflowAlignment::kSafe;
        position = ContentPosition::kEnd;
    }
    if overflow == OverflowAlignment::kDefault {
        overflow = OverflowAlignment::kSafe;
    }
    let is_safe = overflow == OverflowAlignment::kSafe;
    match position {
        ContentPosition::kCenter => {
            if is_safe {
                BlockContentAlignment::kSafeCenter
            } else {
                BlockContentAlignment::kUnsafeCenter
            }
        }
        ContentPosition::kEnd | ContentPosition::kFlexEnd => {
            if is_safe {
                BlockContentAlignment::kSafeEnd
            } else {
                BlockContentAlignment::kUnsafeEnd
            }
        }
        ContentPosition::kNormal => {
            if !behave_like_table_cell {
                return BlockContentAlignment::kStart;
            }
            match style.VerticalAlign() {
                EVerticalAlign::kTop => BlockContentAlignment::kStart,
                EVerticalAlign::kBaselineMiddle
                | EVerticalAlign::kSub
                | EVerticalAlign::kSuper
                | EVerticalAlign::kTextTop
                | EVerticalAlign::kTextBottom
                | EVerticalAlign::kLength
                | EVerticalAlign::kBaseline => BlockContentAlignment::kBaseline,
                EVerticalAlign::kMiddle => {
                    if RuntimeEnabledFeatures::LayoutTableCellAlignmentSafeEnabled() {
                        BlockContentAlignment::kSafeCenter
                    } else {
                        BlockContentAlignment::kUnsafeCenter
                    }
                }
                EVerticalAlign::kBottom => {
                    if RuntimeEnabledFeatures::LayoutTableCellAlignmentSafeEnabled() {
                        BlockContentAlignment::kSafeEnd
                    } else {
                        BlockContentAlignment::kUnsafeEnd
                    }
                }
            }
        }
        ContentPosition::kStart | ContentPosition::kFlexStart => BlockContentAlignment::kStart,
        ContentPosition::kBaseline => BlockContentAlignment::kBaseline,
        ContentPosition::kLastBaseline | ContentPosition::kLeft | ContentPosition::kRight => {
            unreachable!("unsupported resolved content position")
        }
    }
}

// cpp: layoutng/internal/layout_utils.cc:733-736
#[inline(always)]
pub fn ComputeContentAlignmentForTableCell(style: &ComputedStyle) -> BlockContentAlignment {
    ResolveContentAlignment(style, true)
}

// cpp: layoutng/internal/layout_utils.cc:738-786
#[inline(always)]
pub fn InlineStaticPositionEdge(
    oof_node: &BlockNode,
    justify_items_style: *const ComputedStyle,
    parent_writing_direction: WritingDirectionMode,
    should_swap_inline_axis: bool,
) -> InlineEdge {
    assert!(oof_node.IsOutOfFlowPositioned());
    let normal_value_behavior =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kStart, OverflowAlignment::kDefault);
    let align_self = oof_node
        .Style()
        .ResolvedJustifySelf(&normal_value_behavior, justify_items_style)
        .GetPosition();
    match align_self {
        ItemPosition::kEnd
        | ItemPosition::kFlexEnd
        | ItemPosition::kLastBaseline
        | ItemPosition::kRight => {
            if should_swap_inline_axis {
                InlineEdge::kInlineStart
            } else {
                InlineEdge::kInlineEnd
            }
        }
        ItemPosition::kAnchorCenter | ItemPosition::kCenter => InlineEdge::kInlineCenter,
        ItemPosition::kBaseline
        | ItemPosition::kFlexStart
        | ItemPosition::kLeft
        | ItemPosition::kStart
        | ItemPosition::kStretch => {
            if should_swap_inline_axis {
                InlineEdge::kInlineEnd
            } else {
                InlineEdge::kInlineStart
            }
        }
        ItemPosition::kSelfStart | ItemPosition::kSelfEnd => {
            let logical = LogicalToLogical::new(
                oof_node.Style().GetWritingDirection(),
                parent_writing_direction,
                InlineEdge::kInlineStart,
                InlineEdge::kInlineEnd,
                InlineEdge::kInlineStart,
                InlineEdge::kInlineEnd,
            );
            if align_self == ItemPosition::kSelfStart {
                logical.InlineStart()
            } else {
                logical.InlineEnd()
            }
        }
        ItemPosition::kAuto | ItemPosition::kLegacy | ItemPosition::kNormal => {
            unreachable!("resolved justify-self cannot be auto, legacy, or normal")
        }
    }
}

// cpp: layoutng/internal/layout_utils.cc:788-829
#[inline(always)]
pub fn BlockStaticPositionEdge(
    oof_node: &BlockNode,
    align_items_style: *const ComputedStyle,
    parent_writing_direction: WritingDirectionMode,
) -> BlockEdge {
    assert!(oof_node.IsOutOfFlowPositioned());
    let normal_value_behavior =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kStart, OverflowAlignment::kDefault);
    let align_self = oof_node
        .Style()
        .ResolvedAlignSelf(&normal_value_behavior, align_items_style)
        .GetPosition();
    match align_self {
        ItemPosition::kEnd | ItemPosition::kFlexEnd | ItemPosition::kLastBaseline => {
            BlockEdge::kBlockEnd
        }
        ItemPosition::kAnchorCenter | ItemPosition::kCenter => BlockEdge::kBlockCenter,
        ItemPosition::kBaseline
        | ItemPosition::kFlexStart
        | ItemPosition::kStart
        | ItemPosition::kStretch => BlockEdge::kBlockStart,
        ItemPosition::kSelfEnd | ItemPosition::kSelfStart => {
            let logical = LogicalToLogical::new(
                oof_node.Style().GetWritingDirection(),
                parent_writing_direction,
                BlockEdge::kBlockStart,
                BlockEdge::kBlockEnd,
                BlockEdge::kBlockStart,
                BlockEdge::kBlockEnd,
            );
            if align_self == ItemPosition::kSelfStart {
                logical.BlockStart()
            } else {
                logical.BlockEnd()
            }
        }
        ItemPosition::kAuto
        | ItemPosition::kLeft
        | ItemPosition::kRight
        | ItemPosition::kLegacy
        | ItemPosition::kNormal => {
            unreachable!("resolved align-self cannot be auto, left, right, legacy, or normal")
        }
    }
}

// cpp: layoutng/internal/layout_alignment_utils.h:35-39
#[inline(always)]
pub fn InlineStaticPositionEdgeDefault(
    oof_node: &BlockNode,
    justify_items_style: *const ComputedStyle,
    parent_writing_direction: WritingDirectionMode,
) -> InlineEdge {
    InlineStaticPositionEdge(
        oof_node,
        justify_items_style,
        parent_writing_direction,
        false,
    )
}
