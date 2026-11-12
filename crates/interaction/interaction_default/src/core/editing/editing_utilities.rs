// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source core/editing/editing_utilities.h:312; editing_utilities.cc:1469-1491.
use super::{
    commands::{
        replace_selection_command::{AtEdge, GeneralReplacementContext},
        structural_editing_context::StructuralPositionQuery as Q,
    },
    position::{Position, PositionTree},
};

pub fn ComputePlaceholderToCollapseAt<C: GeneralReplacementContext>(
    c: &C,
    p: &Position<u64>,
) -> Position<u64> {
    let downstream = c.QueryPosition(p, Q::Forward);
    if c.RawLineBreakExists(&downstream)
        && AtEdge(c, p, Q::BlockEnd)
        && AtEdge(c, p, Q::ParagraphStart)
    {
        downstream
    } else {
        Position::default()
    }
}

use super::commands::structural_editing_context::StructuralEditingContext;

// Source editing_utilities.h:167,176; editing_utilities.cc:1132-1135,1534-1541.
pub(crate) fn IsMailQuote<C: StructuralEditingContext>(ctx: &C, node: u64) -> bool {
    ctx.Tag(node, "blockquote") && ctx.AttributeEquals(node, "type", "cite")
}
pub(crate) fn IsList<C: StructuralEditingContext>(ctx: &C, node: u64) -> bool {
    ctx.Tag(node, "ul") || ctx.Tag(node, "ol")
}

/// editing_utilities.h:310-311; editing_utilities.cc:1493-1532.
/// Text siblings use the same source position
/// fixup before their original native node is removed.
pub(super) fn ComputePositionForNodeRemoval<C: PositionTree<Node = u64>>(
    c: &C,
    p: &Position<u64>,
    node: u64,
) -> Position<u64> {
    use crate::core::editing::{position::PositionAnchorType, text_offset_mapping::IsDescendantOf};
    let Some(anchor) = p.AnchorNode() else {
        return p.clone();
    };
    if p.AnchorType() == PositionAnchorType::OffsetInAnchor
        && p.ComputeContainerNode(c) == c.Parent(&node)
        && p.ComputeOffsetInContainerNode(c) > c.NodeIndex(&node)
    {
        return Position::new(*anchor, p.ComputeOffsetInContainerNode(c) - 1);
    }
    if *anchor == node || IsDescendantOf(c, anchor, &node) {
        Position::BeforeNode(node).ToOffsetInAnchor(c)
    } else {
        p.clone()
    }
}

// Source editing_utilities.h:371-383; editing_utilities.cc:120-149,1018-1036.
pub fn IsWhitespace(unit: u16) -> bool {
    matches!(unit, 0xA0 | 0x20 | 0x0A | 0x09)
}
pub fn IsCollapsibleWhitespace(unit: u16) -> bool {
    matches!(unit, 0x20 | 0x0A)
}
pub fn StringWithRebalancedWhitespace(
    text: &[u16],
    start_of_paragraph: bool,
    nbsp_before_end: bool,
) -> Vec<u16> {
    let mut output = Vec::with_capacity(text.len());
    let mut previous = 0;
    for (index, &unit) in text.iter().enumerate() {
        let next = if !IsWhitespace(unit) {
            unit
        } else if (index == 0 && start_of_paragraph) || (index + 1 == text.len() && nbsp_before_end)
        {
            0xA0
        } else if previous == 0x20 {
            0xA0
        } else if previous == 0xA0 {
            0x20
        } else if text.get(index + 1).is_some_and(|unit| IsWhitespace(*unit)) {
            0xA0
        } else {
            0x20
        };
        output.push(next);
        previous = next;
    }
    output
}
