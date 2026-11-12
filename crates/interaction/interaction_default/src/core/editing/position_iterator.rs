// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/position_iterator.cc:40-392 (SlowPositionIterator).
use super::position::{Position, PositionDOM};

pub trait PositionIteratorContext: PositionDOM {
    fn IsUserSelectContain(&self, node: &Self::Node) -> bool;
    fn HasLayoutObject(&self, node: &Self::Node) -> bool;
    fn NextGraphemeBoundary(&self, node: &Self::Node, offset: usize) -> usize;
    fn LastOffsetForEditingInDOMTree(&self, node: &Self::Node) -> usize;
    fn PreviousGraphemeBoundary(&self, node: &Self::Node, offset: usize) -> usize;
}

/// This is the source cursor's ancestor-offset stack, not a DOM snapshot.
/// Invalid offsets are resolved lazily from the owning tree when traversed.
#[derive(Clone)]
pub struct PositionIterator<N> {
    anchor: Option<N>,
    child_after: Option<N>,
    offset: usize,
    offsets: Vec<Option<usize>>,
    depth: usize,
}
impl<N: Clone + PartialEq> PositionIterator<N> {
    pub fn new<C: PositionIteratorContext<Node = N>>(ctx: &C, pos: &Position<N>) -> Self {
        let mut result = Self {
            anchor: pos.AnchorNode().cloned(),
            child_after: None,
            offset: 0,
            offsets: Vec::new(),
            depth: 0,
        };
        let Some(anchor) = &result.anchor else {
            return result;
        };
        let offset = pos.ComputeEditingOffset(ctx);
        result.child_after = ctx.ChildAt(anchor, offset);
        if result.child_after.is_none() {
            result.offset = offset;
        }
        let mut ancestor = result.SelectableParent(ctx, anchor);
        while let Some(node) = ancestor {
            result.offsets.push(None);
            result.depth += 1;
            ancestor = result.SelectableParent(ctx, &node);
        }
        if result.child_after.is_some() {
            result.offsets.push(Some(offset));
        }
        result
    }
    fn Traverse<C: PositionIteratorContext<Node = N>>(ctx: &C, node: &N) -> bool {
        ctx.ChildCount(node) != 0 && !ctx.IsUserSelectContain(node)
    }
    fn LastOffset<C: PositionIteratorContext<Node = N>>(ctx: &C, node: &N) -> usize {
        if ctx.IsUserSelectContain(node) {
            1
        } else {
            ctx.LastOffsetForEditing(node)
        }
    }
    fn SelectableParent<C: PositionIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        node: &N,
    ) -> Option<N> {
        ctx.Parent(node)
            .filter(|parent| !ctx.IsUserSelectContain(parent))
    }
    fn Store(&mut self, depth: usize, offset: Option<usize>) {
        if self.offsets.len() <= depth {
            self.offsets.resize(depth + 1, None);
        }
        self.offsets[depth] = offset;
    }
    pub fn GetNode(&self) -> Option<&N> {
        self.anchor.as_ref()
    }
    pub fn OffsetInTextNode(&self) -> usize {
        self.offset
    }
    pub fn DeprecatedComputePosition<C: PositionIteratorContext<Node = N>>(
        &self,
        ctx: &C,
    ) -> Position<N> {
        let Some(anchor) = &self.anchor else {
            return Position::default();
        };
        if self.child_after.is_some() {
            if ctx.EditingIgnoresContent(anchor) {
                return Position::BeforeNode(anchor.clone());
            }
            return Position::new(
                anchor.clone(),
                self.offsets[self.depth].expect("resolved child offset"),
            );
        }
        if ctx.ChildCount(anchor) != 0 {
            return Position::LastPositionInOrAfterNode(ctx, anchor);
        }
        Position::EditingPositionOf(ctx, anchor.clone(), self.offset)
    }
    pub fn ComputePosition<C: PositionIteratorContext<Node = N>>(&self, ctx: &C) -> Position<N> {
        let Some(anchor) = &self.anchor else {
            return Position::default();
        };
        if self.child_after.is_some() {
            return Position::new(
                anchor.clone(),
                self.offsets[self.depth].expect("resolved child offset"),
            );
        }
        if Self::Traverse(ctx, anchor) {
            return Position::LastPositionInOrAfterNode(ctx, anchor);
        }
        if ctx.IsText(anchor) {
            return Position::new(anchor.clone(), self.offset);
        }
        if self.offset != 0 {
            Position::AfterNode(anchor.clone())
        } else {
            Position::BeforeNode(anchor.clone())
        }
    }
    pub fn Increment<C: PositionIteratorContext<Node = N>>(&mut self, ctx: &C) {
        let Some(anchor) = self.anchor.clone() else {
            return;
        };
        if let Some(child) = self.child_after.clone() {
            self.anchor = Some(child.clone());
            self.child_after = Self::Traverse(ctx, &child)
                .then(|| ctx.ChildAt(&child, 0))
                .flatten();
            self.offset = 0;
            self.depth += 1;
            self.Store(self.depth, Some(0));
            return;
        }
        if ctx.HasLayoutObject(&anchor)
            && !Self::Traverse(ctx, &anchor)
            && self.offset < Self::LastOffset(ctx, &anchor)
        {
            self.offset = ctx.NextGraphemeBoundary(&anchor, self.offset);
            return;
        }
        self.child_after = Some(anchor.clone());
        self.anchor = self.SelectableParent(ctx, &anchor);
        if self.anchor.is_none() {
            return;
        }
        self.depth -= 1;
        let offset = self.offsets[self.depth]
            .map_or_else(|| ctx.NodeIndex(&anchor) + 1, |offset| offset + 1);
        self.Store(self.depth, Some(offset));
        self.child_after = ctx.NextSibling(&anchor);
        self.offset = offset;
    }
    pub fn Decrement<C: PositionIteratorContext<Node = N>>(&mut self, ctx: &C) {
        let Some(anchor) = self.anchor.clone() else {
            return;
        };
        if let Some(child) = self.child_after.clone() {
            if let Some(previous) = ctx.PreviousSibling(&child) {
                let offset = self.offsets[self.depth].unwrap_or_else(|| ctx.NodeIndex(&child)) - 1;
                self.Store(self.depth, Some(offset));
                self.anchor = Some(previous.clone());
                self.child_after = None;
                self.offset = if Self::Traverse(ctx, &previous) {
                    0
                } else {
                    Self::LastOffset(ctx, &previous)
                };
                self.depth += 1;
                self.Store(self.depth, Some(self.offset));
            } else {
                self.child_after = ctx.Parent(&child);
                self.anchor = self
                    .child_after
                    .as_ref()
                    .and_then(|node| self.SelectableParent(ctx, node));
                if self.anchor.is_none() {
                    return;
                }
                self.offset = 0;
                self.depth -= 1;
                if self.offsets[self.depth].is_none() {
                    self.Store(
                        self.depth,
                        Some(ctx.NodeIndex(self.child_after.as_ref().expect("parent"))),
                    );
                }
            }
            return;
        }
        if Self::Traverse(ctx, &anchor) {
            let child = ctx
                .ChildAt(&anchor, ctx.ChildCount(&anchor) - 1)
                .expect("last child");
            self.offset = if Self::Traverse(ctx, &child) {
                0
            } else {
                Self::LastOffset(ctx, &child)
            };
            self.anchor = Some(child);
            self.Store(self.depth, None);
            self.depth += 1;
            return;
        }
        if self.offset != 0 && ctx.HasLayoutObject(&anchor) {
            self.offset = ctx.PreviousGraphemeBoundary(&anchor, self.offset);
            return;
        }
        self.child_after = Some(anchor.clone());
        self.anchor = self.SelectableParent(ctx, &anchor);
        if self.anchor.is_none() {
            return;
        }
        self.depth -= 1;
        if self.offsets[self.depth].is_none() {
            self.offset = ctx.NodeIndex(&anchor);
            self.Store(self.depth, Some(self.offset));
        }
    }
    pub fn AtStart<C: PositionIteratorContext<Node = N>>(&self, ctx: &C) -> bool {
        let Some(anchor) = &self.anchor else {
            return true;
        };
        ctx.Parent(anchor).is_none()
            && ((ctx.ChildCount(anchor) == 0 && self.offset == 0)
                || self
                    .child_after
                    .as_ref()
                    .is_some_and(|child| ctx.PreviousSibling(child).is_none()))
    }
    pub fn AtEnd<C: PositionIteratorContext<Node = N>>(&self, ctx: &C) -> bool {
        let Some(anchor) = &self.anchor else {
            return true;
        };
        self.child_after.is_none()
            && ctx.Parent(anchor).is_none()
            && (ctx.ChildCount(anchor) != 0 || self.offset >= ctx.LastOffsetForEditing(anchor))
    }
    pub fn AtStartOfNode<C: PositionIteratorContext<Node = N>>(&self, ctx: &C) -> bool {
        let Some(anchor) = &self.anchor else {
            return true;
        };
        match &self.child_after {
            Some(child) => ctx.PreviousSibling(child).is_none(),
            None => !Self::Traverse(ctx, anchor) && self.offset == 0,
        }
    }
    pub fn AtEndOfNode<C: PositionIteratorContext<Node = N>>(&self, ctx: &C) -> bool {
        self.anchor.as_ref().is_none_or(|anchor| {
            self.child_after.is_none()
                && (ctx.ChildCount(anchor) != 0 || self.offset >= ctx.LastOffsetForEditing(anchor))
        })
    }
}

/// Source editing_utilities.cc:789-835, kGraphemeCluster branch.
pub fn PreviousGraphemePosition<C: PositionIteratorContext>(
    context: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    position.PreviousWithOffset(context, |node, offset| {
        context.PreviousGraphemeBoundary(node, offset)
    })
}

/// NodeTraversal::LastWithinOrSelf, following only canonical child identities.
pub fn LastWithinOrSelf<C: super::position::PositionTree>(context: &C, node: &C::Node) -> C::Node {
    let mut last = node.clone();
    while let Some(child) = context
        .ChildCount(&last)
        .checked_sub(1)
        .and_then(|index| context.ChildAt(&last, index))
    {
        last = child;
    }
    last
}

/// Source editing_utilities.cc:869-922, kGraphemeCluster branch; source
/// intentionally reads DOM LastOffset even for its flat-tree overload.
pub fn NextGraphemePosition<C: PositionIteratorContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    let Some(node) = position.AnchorNode() else {
        return position.clone();
    };
    let offset = position.ComputeEditingOffset(ctx);
    if let Some(child) = ctx.ChildAt(node, offset) {
        return Position::FirstPositionInOrBeforeNode(ctx, &child);
    }
    if ctx.ChildAt(node, 0).is_none() && offset < ctx.LastOffsetForEditingInDOMTree(node) {
        return Position::EditingPositionOf(
            ctx,
            node.clone(),
            ctx.NextGraphemeBoundary(node, offset),
        );
    }
    if let Some(parent) = ctx.Parent(node) {
        return Position::EditingPositionOf(ctx, parent, ctx.NodeIndex(node) + 1);
    }
    position.clone()
}
