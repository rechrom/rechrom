// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Rechrom binding helpers for live structural command host queries.
//! These position/tree projections have no one-to-one Chromium source mapping.
use super::structural_editing_context::{StructuralEditingContext, StructuralPositionQuery};
use crate::core::editing::{
    position::{Position, PositionTree},
    text_offset_mapping,
};
use std::cmp::Ordering;

pub(crate) fn ParentBefore<C: PositionTree<Node = u64>>(ctx: &C, node: u64) -> Position<u64> {
    Position::BeforeNode(node).ToOffsetInAnchor(ctx)
}
pub(crate) fn ParentAfter<C: PositionTree<Node = u64>>(ctx: &C, node: u64) -> Position<u64> {
    Position::AfterNode(node).ToOffsetInAnchor(ctx)
}
pub(crate) fn ChildIndex<C: PositionTree<Node = u64>>(ctx: &C, node: u64) -> usize {
    ctx.NodeIndex(&node)
}
pub(crate) fn Contains<C: PositionTree<Node = u64>>(ctx: &C, parent: u64, node: u64) -> bool {
    parent == node || text_offset_mapping::IsDescendantOf(ctx, &node, &parent)
}
pub(crate) fn BeforeOrEqual<C: StructuralEditingContext>(
    ctx: &C,
    a: &Position<u64>,
    b: &Position<u64>,
) -> bool {
    ctx.Compare(a, b)
        .is_some_and(|order| order != Ordering::Greater)
}
pub(crate) fn Ancestor<C: StructuralEditingContext>(
    ctx: &C,
    position: &Position<u64>,
    predicate: impl Fn(u64) -> bool,
) -> Option<u64> {
    let mut node = position.ComputeContainerNode(ctx);
    while let Some(current) = node {
        if predicate(current) {
            return Some(current);
        }
        node = ctx.Parent(&current);
    }
    None
}
pub(crate) fn LineBreakAt<C: StructuralEditingContext>(ctx: &C, p: &Position<u64>) -> bool {
    let p = ctx.QueryPosition(p, StructuralPositionQuery::Forward);
    p.AnchorNode().is_some_and(|node| ctx.NodeFacts(*node).br)
}
