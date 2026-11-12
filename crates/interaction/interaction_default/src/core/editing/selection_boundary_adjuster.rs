// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/selection_adjuster.cc:404-435,507-568,617-739;
// visible_units.cc:612-665. DOM selection adjustment retains raw positions.
use super::{
    compare_positions::{AncestorInScope, CompareDOMPositions},
    position::{Position, PositionTree},
    position_conversion::{PositionTreeScope, ToPositionInDOMTree, ToPositionInFlatTree},
    visible_units::{self, EditingBoundaryCrossingRule, SnapToClient, VisibleUnitsContext},
};
use std::cmp::Ordering;
pub trait FlatSelectionBoundaryContext: VisibleUnitsContext + PositionTreeScope {
    fn SelectionEditingBoundarySlottedContentEnabled(&self) -> bool;
    fn AssignedSlotWithoutRecalc(&self, node: &Self::Node) -> Option<Self::Node>;
    fn OwnerShadowHost(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsSlot(&self, node: &Self::Node) -> bool;
    fn HasAssignedNodesNoRecalc(&self, slot: &Self::Node) -> bool;
}
fn IsEnclosedBy<C: PositionTree>(ctx: &C, position: &Position<C::Node>, node: &C::Node) -> bool {
    let Some(anchor) = position.AnchorNode() else {
        return false;
    };
    if anchor == node {
        return !matches!(
            position.AnchorType(),
            super::position::PositionAnchorType::BeforeAnchor
                | super::position::PositionAnchorType::AfterAnchor
        );
    }
    super::text_offset_mapping::IsDescendantOf(ctx, anchor, node)
}
fn EnclosingContainHost<C: FlatSelectionBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
    start: bool,
) -> Option<C::Node> {
    let mut node = if start {
        super::text_offset_mapping::NodeAsRangeFirstNode(ctx, position)
    } else {
        super::text_offset_mapping::NodeAsRangeLastNode(ctx, position)
    };
    while let Some(current) = node {
        if ctx.IsShadowHost(&current) {
            return (IsEnclosedBy(ctx, position, &current) && ctx.IsUserSelectContain(&current))
                .then_some(current);
        }
        node = ctx.Parent(&current);
    }
    None
}
// Source: selection_adjuster.cc:404-497,517-537,580-611. Flat-tree selection
// crosses author shadow trees while respecting the source contain hosts.
pub fn AdjustFlatFocusForShadowBoundary<C: FlatSelectionBoundaryContext>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
) -> Position<C::Node> {
    if anchor == focus {
        return focus.clone();
    }
    let forward = super::compare_positions::CompareFlatPositions(ctx, anchor, focus)
        != Some(Ordering::Greater);
    let (start, end) = if forward {
        (anchor, focus)
    } else {
        (focus, anchor)
    };
    let start_host = EnclosingContainHost(ctx, start, true);
    let end_host = EnclosingContainHost(ctx, end, false);
    if start_host == end_host {
        return focus.clone();
    }
    let host = if forward {
        start_host.or(end_host)
    } else {
        end_host.or(start_host)
    }
    .expect("one contain boundary");
    let adjusted = if IsEnclosedBy(ctx, focus, &host) {
        if forward {
            if focus.AnchorType() == super::position::PositionAnchorType::AfterChildren {
                Position::AfterNode(host)
            } else {
                Position::BeforeNode(host)
            }
        } else if focus.IsBeforeChildren(ctx) {
            Position::BeforeNode(host)
        } else {
            Position::AfterNode(host)
        }
    } else if forward {
        ctx.ChildCount(&host)
            .checked_sub(1)
            .and_then(|index| ctx.ChildAt(&host, index))
            .map(Position::AfterNode)
            .unwrap_or_default()
    } else {
        ctx.ChildAt(&host, 0)
            .map(Position::BeforeNode)
            .unwrap_or_default()
    };
    if adjusted.IsNull() {
        anchor.clone()
    } else {
        adjusted
    }
}
fn IsFlatEditingBoundary<C: FlatSelectionBoundaryContext>(
    ctx: &C,
    node: &C::Node,
    previous: &C::Node,
    previous_editable: bool,
) -> bool {
    let slotted = ctx.SelectionEditingBoundarySlottedContentEnabled();
    if slotted && ctx.IsSlot(node) && ctx.AssignedSlotWithoutRecalc(previous).as_ref() == Some(node)
    {
        return false;
    }
    if ctx.IsShadowHost(node)
        && previous_editable
        && ctx.OwnerShadowHost(previous).as_ref() == Some(node)
    {
        if !slotted || !ctx.IsSlot(previous) || !ctx.HasAssignedNodesNoRecalc(previous) {
            return true;
        }
    }
    ctx.IsEditable(node) != previous_editable
}
fn FlatRootBoundary<C: FlatSelectionBoundaryContext>(ctx: &C, start: &C::Node) -> C::Node {
    if ctx.IsBody(start) {
        return start.clone();
    }
    let editable = ctx.IsEditable(start);
    let mut result = start.clone();
    let mut ancestor = ctx.Parent(start);
    while let Some(node) = ancestor {
        if IsFlatEditingBoundary(ctx, &node, &result, editable) {
            break;
        }
        result = node;
        if ctx.IsBody(&result) {
            break;
        }
        ancestor = ctx.Parent(&result);
    }
    result
}
// Source: selection_adjuster.cc:617-783, including the flat-tree slot/host
// specialization. Editability comes from the same DOM owner as DOM strategy.
pub fn AdjustFlatFocusForEditingBoundary<C: FlatSelectionBoundaryContext>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
) -> Position<C::Node> {
    let (Some(anchor_node), Some(focus_node)) = (
        anchor.ComputeContainerNode(ctx),
        focus.ComputeContainerNode(ctx),
    ) else {
        return focus.clone();
    };
    if anchor_node == focus_node {
        return focus.clone();
    }
    let anchor_root = FlatRootBoundary(ctx, &anchor_node);
    let focus_root = FlatRootBoundary(ctx, &focus_node);
    if anchor_root == focus_root {
        return focus.clone();
    }
    let forward = super::compare_positions::CompareFlatPositions(ctx, anchor, focus)
        != Some(Ordering::Greater);
    if !super::text_offset_mapping::IsDescendantOf(ctx, &focus_root, &anchor_root) {
        return if forward {
            Position::LastPositionInNode(ctx, &anchor_root)
        } else {
            Position::FirstPositionInNode(anchor_root)
        };
    }
    let mut boundary = focus_root.clone();
    let mut previous = focus_root;
    let mut previous_editable = ctx.IsEditable(&previous);
    let mut ancestor = ctx.Parent(&previous);
    while let Some(node) = ancestor {
        if IsFlatEditingBoundary(ctx, &node, &previous, previous_editable) {
            boundary = previous.clone();
        }
        if node == anchor_root || ctx.IsBody(&node) {
            break;
        }
        previous_editable = ctx.IsEditable(&node);
        previous = node;
        ancestor = ctx.Parent(&previous);
    }
    if forward {
        Position::BeforeNode(boundary)
    } else {
        Position::AfterNode(boundary)
    }
}

fn Contains<C: PositionTree>(ctx: &C, ancestor: &C::Node, node: &C::Node) -> bool {
    let mut node = Some(node.clone());
    while let Some(current) = node {
        if &current == ancestor {
            return true;
        }
        node = ctx.Parent(&current);
    }
    false
}
/// Source DOM ShadowBoundaryAdjuster. Flat-tree user-select:contain has its own
/// source specialization and is deliberately not substituted by this function.
pub fn AdjustDOMFocusForShadowBoundary<C: PositionTreeScope>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
) -> Position<C::Node> {
    if anchor == focus {
        return focus.clone();
    }
    let (Some(anchor_node), Some(focus_node)) = (anchor.AnchorNode(), focus.AnchorNode()) else {
        return focus.clone();
    };
    if ctx.TreeScopeRoot(anchor_node) == ctx.TreeScopeRoot(focus_node) {
        return focus.clone();
    }
    let (Some(anchor_container), Some(focus_container)) = (
        anchor.ComputeContainerNode(ctx),
        focus.ComputeContainerNode(ctx),
    ) else {
        return Position::default();
    };
    let scope = ctx.TreeScopeRoot(&anchor_container);
    let forward = CompareDOMPositions(ctx, anchor, focus) != Some(Ordering::Greater);
    let adjusted = if let Some(ancestor) = AncestorInScope(ctx, &focus_container, &scope) {
        if Contains(ctx, &ancestor, &anchor_container) == forward {
            Position::AfterNode(ancestor)
        } else {
            Position::BeforeNode(ancestor)
        }
    } else if forward {
        ctx.ChildCount(&scope)
            .checked_sub(1)
            .and_then(|offset| ctx.ChildAt(&scope, offset))
            .map(Position::AfterNode)
            .unwrap_or_default()
    } else {
        ctx.ChildAt(&scope, 0)
            .map(Position::BeforeNode)
            .unwrap_or_default()
    };
    if adjusted.IsNull() {
        anchor.clone()
    } else {
        adjusted
    }
}
fn RootBoundary<C: VisibleUnitsContext>(ctx: &C, start: &C::Node) -> C::Node {
    if ctx.IsBody(start) {
        return start.clone();
    }
    let editable = ctx.IsEditable(start);
    let mut result = start.clone();
    while let Some(parent) = ctx.Parent(&result) {
        if ctx.IsEditable(&parent) != editable {
            break;
        }
        result = parent;
        if ctx.IsBody(&result) {
            break;
        }
    }
    result
}
pub fn AdjustDOMFocusForEditingBoundary<C: VisibleUnitsContext + PositionTreeScope>(
    ctx: &C,
    anchor: &Position<C::Node>,
    focus: &Position<C::Node>,
) -> Position<C::Node> {
    let (Some(anchor_node), Some(focus_node)) = (
        anchor.ComputeContainerNode(ctx),
        focus.ComputeContainerNode(ctx),
    ) else {
        return focus.clone();
    };
    if anchor_node == focus_node {
        return focus.clone();
    }
    let anchor_root = RootBoundary(ctx, &anchor_node);
    let focus_root = RootBoundary(ctx, &focus_node);
    if anchor_root == focus_root {
        return focus.clone();
    }
    let forward = CompareDOMPositions(ctx, anchor, focus) != Some(Ordering::Greater);
    if !Contains(ctx, &anchor_root, &focus_root) {
        return if forward {
            Position::LastPositionInNode(ctx, &anchor_root)
        } else {
            Position::FirstPositionInNode(anchor_root)
        };
    }
    let mut boundary = focus_root.clone();
    let mut previous = focus_root;
    let mut previous_editable = ctx.IsEditable(&previous);
    while let Some(ancestor) = ctx.Parent(&previous) {
        let editable = ctx.IsEditable(&ancestor);
        if editable != previous_editable {
            boundary = previous.clone();
        }
        if ancestor == anchor_root || ctx.IsBody(&ancestor) {
            break;
        }
        previous = ancestor;
        previous_editable = editable;
    }
    if forward {
        Position::BeforeNode(boundary)
    } else {
        Position::AfterNode(boundary)
    }
}
/// Source DOM facade: traversal occurs in the canonical flat tree and its
/// resulting position is adjusted by the original DOM selection algorithms.
pub fn DOMCaretPosition<D, F>(
    dom: &D,
    flat: &F,
    position: &Position<D::Node>,
    rule: EditingBoundaryCrossingRule,
    client: SnapToClient,
    backward: bool,
) -> Position<D::Node>
where
    D: PositionTreeScope + VisibleUnitsContext,
    F: VisibleUnitsContext<Node = D::Node>,
{
    let Some(start) = position.AnchorNode() else {
        return Position::default();
    };
    let in_flat = ToPositionInFlatTree(dom, flat, position);
    let candidate_flat = if backward {
        visible_units::MostBackwardCaretPosition(flat, &in_flat, rule, client)
    } else {
        visible_units::MostForwardCaretPosition(flat, &in_flat, rule, client)
    };
    let candidate = ToPositionInDOMTree(dom, flat, &candidate_flat);
    let Some(end) = candidate.AnchorNode() else {
        return position.clone();
    };
    let in_shadow = |node: &D::Node| dom.IsShadowRoot(&dom.TreeScopeRoot(node));
    if !in_shadow(start) && !dom.IsShadowHost(start) && !in_shadow(end) && !dom.IsShadowHost(end) {
        return candidate;
    }
    if position == &candidate {
        return candidate;
    }
    let adjusted = AdjustDOMFocusForShadowBoundary(dom, position, &candidate);
    if CompareDOMPositions(dom, position, &candidate)
        == CompareDOMPositions(dom, &candidate, &adjusted)
    {
        return position.clone();
    }
    if rule == EditingBoundaryCrossingRule::CannotCross && candidate != adjusted {
        return AdjustDOMFocusForEditingBoundary(dom, position, &adjusted);
    }
    adjusted
}
