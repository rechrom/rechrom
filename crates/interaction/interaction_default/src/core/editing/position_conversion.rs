// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/position.cc:553-642.
use super::position::{Position, PositionAnchorType, PositionTree};

pub trait PositionTreeScope: PositionTree {
    fn IsShadowRoot(&self, node: &Self::Node) -> bool;
    fn ShadowHost(&self, root: &Self::Node) -> Option<Self::Node>;
    fn TreeScopeRoot(&self, node: &Self::Node) -> Self::Node;
    fn IsShadowHost(&self, node: &Self::Node) -> bool;
}
/// Position::CommonAncestorTreeScope, position.cc:60-67;
/// TreeScope::CommonAncestorTreeScope, tree_scope.cc:843-863.
pub fn CommonAncestorTreeScope<C: PositionTreeScope>(
    ctx: &C,
    a: &Position<C::Node>,
    b: &Position<C::Node>,
) -> Option<C::Node> {
    let chain = |node: C::Node| {
        let mut result = Vec::new();
        let mut scope = Some(ctx.TreeScopeRoot(&node));
        while let Some(root) = scope {
            scope = ctx.ShadowHost(&root).map(|host| ctx.TreeScopeRoot(&host));
            result.push(root);
        }
        result
    };
    let mut a = chain(a.ComputeContainerNode(ctx)?);
    let mut b = chain(b.ComputeContainerNode(ctx)?);
    let mut common = None;
    while !a.is_empty() && !b.is_empty() && a.last() == b.last() {
        common = a.pop();
        b.pop();
    }
    common
}
fn WithAnchor<C: PositionTree>(
    ctx: &C,
    node: C::Node,
    kind: PositionAnchorType,
) -> Position<C::Node> {
    match kind {
        PositionAnchorType::BeforeAnchor => Position::BeforeNode(node),
        PositionAnchorType::AfterAnchor => Position::AfterNode(node),
        PositionAnchorType::AfterChildren => Position::LastPositionInNode(ctx, &node),
        PositionAnchorType::OffsetInAnchor => Position::new(node, 0),
    }
}
pub fn ToPositionInFlatTree<D: PositionTreeScope, F: PositionTree<Node = D::Node>>(
    dom: &D,
    flat: &F,
    position: &Position<D::Node>,
) -> Position<D::Node> {
    let Some(anchor) = position.AnchorNode() else {
        return Position::default();
    };
    if position.IsOffsetInAnchor() {
        let offset = position.ComputeOffsetInContainerNode(dom);
        if dom.CharacterDataLength(anchor).is_some() {
            return Position::new(anchor.clone(), offset);
        }
        let host_or_anchor = || {
            if dom.IsShadowRoot(anchor) {
                dom.ShadowHost(anchor).expect("attached shadow root")
            } else {
                anchor.clone()
            }
        };
        if offset == 0 {
            return Position::FirstPositionInNode(host_or_anchor());
        }
        let Some(child) = dom.ChildAt(anchor, offset) else {
            return Position::LastPositionInNode(flat, &host_or_anchor());
        };
        if dom.IsShadowRoot(&child) {
            return Position::new(host_or_anchor(), offset);
        }
        if let Some(parent) = flat.Parent(&child) {
            return Position::new(parent, flat.NodeIndex(&child));
        }
        return Position::LastPositionInNode(flat, &host_or_anchor());
    }
    if dom.IsShadowRoot(anchor) {
        return WithAnchor(
            flat,
            dom.ShadowHost(anchor).expect("attached shadow root"),
            position.AnchorType(),
        );
    }
    if matches!(
        position.AnchorType(),
        PositionAnchorType::BeforeAnchor | PositionAnchorType::AfterAnchor
    ) && flat.Parent(anchor).is_none()
    {
        return Position::default();
    }
    WithAnchor(flat, anchor.clone(), position.AnchorType())
}
pub fn ToPositionInDOMTree<D: PositionTree, F: PositionTree<Node = D::Node>>(
    dom: &D,
    flat: &F,
    position: &Position<D::Node>,
) -> Position<D::Node> {
    let Some(anchor) = position.AnchorNode() else {
        return Position::default();
    };
    if !position.IsOffsetInAnchor() {
        return WithAnchor(dom, anchor.clone(), position.AnchorType());
    }
    let offset = position.ComputeOffsetInContainerNode(flat);
    if dom.CharacterDataLength(anchor).is_some() {
        return Position::new(anchor.clone(), offset);
    }
    if offset == 0 {
        return Position::FirstPositionInNode(anchor.clone());
    }
    if let Some(child) = flat.ChildAt(anchor, offset) {
        return Position::InParentBeforeNode(dom, &child);
    }
    Position::LastPositionInNode(dom, anchor)
}

/// Source ShadowIncludingTreeOrderTraversal inclusive ancestry, used by
/// deletion's live-node checks without flattening a shadow subtree.
pub fn IsShadowIncludingInclusiveAncestor<C: PositionTreeScope>(
    context: &C,
    ancestor: &C::Node,
    node: &C::Node,
) -> bool {
    let mut current = Some(node.clone());
    while let Some(node) = current {
        if &node == ancestor {
            return true;
        }
        current = context.Parent(&node).or_else(|| context.ShadowHost(&node));
    }
    false
}
