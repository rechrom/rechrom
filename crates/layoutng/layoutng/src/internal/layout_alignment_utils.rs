#![allow(non_snake_case)]

use foundation::WritingDirectionMode;
use layoutng_geometry::geometry::static_position::InlineEdge;
use layoutng_style::style::computed_style::ComputedStyle;

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

// cpp: layoutng/internal/layout_alignment_utils.h:26-45
// The four definitions are mapped from layout_utils.cc.
pub use super::layout_utils::{
    BlockStaticPositionEdge, ComputeContentAlignmentForTableCell, InlineStaticPositionEdge,
    ResolveContentAlignment,
};

// cpp: layoutng/internal/layout_alignment_utils.h:35-39
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
