// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: editing_utilities.cc:318-342,520-566,664-699;
// visible_units.cc:235-276.
use super::{
    compare_positions::AncestorInScope,
    position::{Position, PositionWithAffinity, TextAffinity},
    position_conversion::PositionTreeScope,
    position_iterator::PositionIterator,
    text_offset_mapping::IsDescendantOf,
    visible_units::{self, EditingBoundaryCrossingRule, VisibleUnitsContext},
};
use std::cmp::Ordering;
// Source: editing_utilities.cc:345-367,382-395. DOM strategy supplies
// editability/root identity; flat callers convert before invoking these leaves.
pub trait EditingPositionRootContext: super::position::PositionDOM {
    fn RootEditableElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsEditableNode(&self, node: &Self::Node) -> bool;
    fn IsDocumentNode(&self, node: &Self::Node) -> bool;
}
pub fn RootEditableElementOf<C: EditingPositionRootContext>(
    ctx: &C,
    p: &Position<C::Node>,
) -> Option<C::Node> {
    let mut node = p.ComputeContainerNode(ctx)?;
    if ctx.IsDisplayInsideTable(&node) {
        node = ctx.Parent(&node)?;
    }
    ctx.RootEditableElement(&node)
}
pub fn IsEditablePosition<C: EditingPositionRootContext>(ctx: &C, p: &Position<C::Node>) -> bool {
    let Some(mut node) = p.ComputeContainerNode(ctx) else {
        return false;
    };
    if ctx.IsDisplayInsideTable(&node) {
        let Some(parent) = ctx.Parent(&node) else {
            return false;
        };
        node = parent;
    }
    !ctx.IsDocumentNode(&node) && ctx.IsEditableNode(&node)
}
// Source: editing_utilities.cc:179-184; DOM strategy is chosen by caller.
pub fn IsAtomicNode<C: super::position::PositionDOM>(ctx: &C, node: &C::Node) -> bool {
    ctx.ChildCount(node) == 0 || ctx.EditingIgnoresContent(node)
}
pub trait EditableBoundaryContext: VisibleUnitsContext + PositionTreeScope {
    fn RootEditableElementOf(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn HighestEditableRoot(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn IsEditablePosition(&self, position: &Position<Self::Node>) -> bool;
    fn IsAtomicNodeInDOMTree(&self, node: &Self::Node) -> bool;
    fn ComparePositions(
        &self,
        first: &Position<Self::Node>,
        second: &Position<Self::Node>,
    ) -> Option<Ordering>;
    fn DOMContains(&self, ancestor: &Self::Node, node: &Self::Node) -> bool;
}
// Source: editing_utilities.cc:427-474,520-566. Both strategy traversals share
// the source's candidate test; direction changes only cursor stepping/end.
pub fn VisuallyDistinctCandidate<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
    rule: EditingBoundaryCrossingRule,
    backward: bool,
) -> Position<C::Node> {
    if position.IsNull() {
        return Position::default();
    }
    let skip = rule == EditingBoundaryCrossingRule::CanSkipOver && ctx.IsEditablePosition(position);
    let boundary_rule = if skip {
        EditingBoundaryCrossingRule::CanCross
    } else {
        rule
    };
    let downstream = ctx.BoundaryCaretPosition(position, boundary_rule, false);
    let upstream = ctx.BoundaryCaretPosition(position, boundary_rule, true);
    let mut cursor = PositionIterator::new(ctx, position);
    if backward {
        cursor.Decrement(ctx);
    } else {
        cursor.Increment(ctx);
    }
    while if backward {
        !cursor.AtStart(ctx)
    } else {
        !cursor.AtEnd(ctx)
    } {
        let candidate = cursor.ComputePosition(ctx);
        let skip_candidate = skip
            && !ctx.IsEditablePosition(&candidate)
            && ctx
                .RootEditableElementOf(position)
                .as_ref()
                .zip(candidate.AnchorNode())
                .is_some_and(|(root, node)| node != root && ctx.DOMContains(root, node));
        if !skip_candidate
            && visible_units::IsVisuallyEquivalentCandidate(ctx, &candidate)
            && ctx.BoundaryCaretPosition(&candidate, boundary_rule, false) != downstream
            && ctx.BoundaryCaretPosition(&candidate, boundary_rule, true) != upstream
        {
            return candidate;
        }
        if backward {
            cursor.Decrement(ctx);
        } else {
            cursor.Increment(ctx);
        }
    }
    Position::default()
}
// Source: editing_utilities.cc:1186-1225. The predicate belongs to its caller;
// strategy ancestry and editable-root exclusion remain this source algorithm.
pub fn EnclosingNodeOfType<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
    predicate: impl Fn(&C::Node) -> bool,
    rule: EditingBoundaryCrossingRule,
) -> Option<C::Node> {
    assert!(matches!(
        rule,
        EditingBoundaryCrossingRule::CanCross | EditingBoundaryCrossingRule::CannotCross
    ));
    if position.IsNull() {
        return None;
    }
    let root = if rule == EditingBoundaryCrossingRule::CannotCross {
        ctx.RootEditableElementOf(position)
    } else {
        None
    };
    let mut current = position.AnchorNode().cloned();
    while let Some(node) = current {
        current = ctx.Parent(&node);
        if root.is_some() && !ctx.IsEditable(&node) {
            continue;
        }
        if predicate(&node) {
            return Some(node);
        }
        if root.as_ref() == Some(&node) {
            return None;
        }
    }
    None
}
/// Caller supplies DOM strategy; the public flat overload converts to DOM
/// before entering this source routine because editability is DOM-defined.
pub fn HighestEditableRoot<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::Node> {
    let mut root = ctx.RootEditableElementOf(position)?;
    if ctx.IsBody(&root) {
        return Some(root);
    }
    let mut parent = ctx.Parent(&root);
    while let Some(node) = parent {
        if ctx.IsEditable(&node) {
            root = node.clone();
        }
        if ctx.IsBody(&node) {
            break;
        }
        parent = ctx.Parent(&node);
    }
    Some(root)
}
pub fn PreviousVisuallyDistinctCandidate<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    VisuallyDistinctCandidate(
        ctx,
        position,
        EditingBoundaryCrossingRule::CannotCross,
        true,
    )
}
pub fn LastEditablePositionBeforeInRoot<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
    root: &C::Node,
) -> Position<C::Node> {
    let last = Position::LastPositionInNode(ctx, root);
    if ctx.ComparePositions(position, &last) == Some(Ordering::Greater) {
        return last;
    }
    let mut editable = position.clone();
    let Some(anchor) = editable.AnchorNode() else {
        return editable;
    };
    if ctx.TreeScopeRoot(anchor) != ctx.TreeScopeRoot(root) {
        let Some(ancestor) = AncestorInScope(ctx, anchor, &ctx.TreeScopeRoot(root)) else {
            return Position::default();
        };
        editable = Position::FirstPositionInOrBeforeNode(ctx, &ancestor);
    }
    while editable.AnchorNode().is_some_and(|node| {
        !ctx.IsEditablePosition(&editable) && node != root && ctx.DOMContains(root, node)
    }) {
        let node = editable.AnchorNode().expect("non-null");
        editable = if ctx.IsAtomicNodeInDOMTree(node) {
            Position::InParentBeforeNode(ctx, node)
        } else {
            PreviousVisuallyDistinctCandidate(ctx, &editable)
        };
    }
    if editable
        .AnchorNode()
        .is_some_and(|node| node != root && !ctx.DOMContains(root, node))
    {
        Position::default()
    } else {
        editable
    }
}
pub fn AdjustBackwardPositionToAvoidCrossingEditingBoundaries<C: EditableBoundaryContext>(
    ctx: &C,
    position: PositionWithAffinity<C::Node>,
    anchor: &Position<C::Node>,
) -> PositionWithAffinity<C::Node> {
    if position.position.IsNull() {
        return position;
    }
    let root = ctx.HighestEditableRoot(anchor);
    if root.as_ref().is_some_and(|root| {
        let node = position.position.AnchorNode().expect("non-null");
        node == root || !ctx.DOMContains(root, node)
    }) {
        if let Some(node) = anchor
            .ComputeContainerNode(ctx)
            .filter(|node| ctx.IsText(node))
        {
            let first = Position::FirstPositionInNode(node);
            if anchor != &first {
                return PositionWithAffinity {
                    position: first,
                    affinity: TextAffinity::Downstream,
                };
            }
        }
        return PositionWithAffinity::default();
    }
    if ctx.HighestEditableRoot(&position.position) == root {
        return position;
    }
    let Some(root) = root else {
        return PositionWithAffinity::default();
    };
    PositionWithAffinity {
        position: LastEditablePositionBeforeInRoot(ctx, &position.position, &root),
        affinity: TextAffinity::Downstream,
    }
}
// Source: editing_utilities.cc:427-474,584-648; visible_units.cc:295-342.
pub fn NextVisuallyDistinctCandidate<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    VisuallyDistinctCandidate(
        ctx,
        position,
        EditingBoundaryCrossingRule::CannotCross,
        false,
    )
}
pub fn FirstEditablePositionAfterPositionInRoot<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
    root: &C::Node,
) -> Position<C::Node> {
    let first = Position::FirstPositionInNode(root.clone());
    if ctx.ComparePositions(position, &first) == Some(Ordering::Less) && ctx.IsEditable(root) {
        return first;
    }
    let mut editable = position.clone();
    if let Some(anchor) = editable.AnchorNode() {
        if ctx.TreeScopeRoot(anchor) != ctx.TreeScopeRoot(root) {
            let Some(ancestor) = AncestorInScope(ctx, anchor, &ctx.TreeScopeRoot(root)) else {
                return Position::default();
            };
            editable = Position::AfterNode(ancestor);
        }
    }
    let mut non_editable = None;
    while editable.AnchorNode().is_some_and(|node| {
        !ctx.IsEditablePosition(&editable) && node != root && ctx.DOMContains(root, node)
    }) {
        let node = editable.AnchorNode().unwrap();
        non_editable = Some(node.clone());
        editable = if ctx.IsAtomicNodeInDOMTree(node) {
            Position::InParentAfterNode(ctx, node)
        } else {
            NextVisuallyDistinctCandidate(ctx, &editable)
        };
    }
    if editable
        .AnchorNode()
        .is_some_and(|node| node != root && !ctx.DOMContains(root, node))
    {
        return Position::default();
    }
    let need_next = editable
        .AnchorNode()
        .and_then(|node| {
            ctx.ChildCount(node)
                .checked_sub(1)
                .and_then(|index| ctx.ChildAt(node, index))
        })
        .as_ref()
        .is_some_and(|last| non_editable.as_ref() == Some(last));
    if need_next {
        let boundary = Position::LastPositionInNode(ctx, root);
        let next = visible_units::NextCandidate(ctx, &editable);
        editable =
            if !next.IsNull() && ctx.ComparePositions(&next, &boundary) == Some(Ordering::Less) {
                next
            } else {
                boundary
            };
    }
    editable
}
pub fn AdjustForwardPositionToAvoidCrossingEditingBoundaries<C: EditableBoundaryContext>(
    ctx: &C,
    position: PositionWithAffinity<C::Node>,
    anchor: &Position<C::Node>,
) -> PositionWithAffinity<C::Node> {
    if position.position.IsNull() {
        return position;
    }
    let root = ctx.HighestEditableRoot(anchor);
    if root.as_ref().is_some_and(|root| {
        position
            .position
            .AnchorNode()
            .is_some_and(|node| node == root || !ctx.DOMContains(root, node))
    }) {
        if let Some(node) = anchor
            .ComputeContainerNode(ctx)
            .filter(|node| ctx.IsText(node))
        {
            let last = Position::LastPositionInNode(ctx, &node);
            if anchor != &last {
                return PositionWithAffinity {
                    position: last,
                    affinity: TextAffinity::Downstream,
                };
            }
        }
        return PositionWithAffinity::default();
    }
    if ctx.HighestEditableRoot(&position.position) == root {
        return position;
    }
    let Some(root) = root else {
        let Some(mut last) = anchor.ComputeContainerNode(ctx) else {
            return PositionWithAffinity::default();
        };
        let mut parent = ctx.Parent(&last);
        while let Some(ancestor) = parent {
            if ctx.IsEditable(&ancestor) {
                return PositionWithAffinity {
                    position: Position::LastPositionInNode(ctx, &last),
                    affinity: TextAffinity::Downstream,
                };
            }
            parent = ctx.Parent(&ancestor);
            last = ancestor;
        }
        return PositionWithAffinity::default();
    };
    PositionWithAffinity {
        position: FirstEditablePositionAfterPositionInRoot(ctx, &position.position, &root),
        affinity: TextAffinity::Downstream,
    }
}
