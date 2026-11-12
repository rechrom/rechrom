// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/visible_units.cc:98-229;
// core/editing/visible_position.cc:70-158.
use super::position::{Position, PositionDOM, PositionWithAffinity, TextAffinity};
use std::cmp::Ordering;

/// These queries borrow resident LayoutNG caret/line/offset-mapping owners.
/// They never infer visual equivalence or line membership from DOM topology.
pub trait DownstreamVisiblePositionContext: PositionDOM {
    fn MostBackwardCaretPosition(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    fn MostForwardCaretPosition(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    fn IsVisuallyEquivalentCandidate(&self, position: &Position<Self::Node>) -> bool;
    fn NextCandidate(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    fn PreviousCandidate(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn RootEditableElement(&self, position: &Position<Self::Node>) -> Option<Self::Node>;
    fn IsDescendantOf(&self, node: &Self::Node, ancestor: &Self::Node) -> bool;
    fn EnclosingBlockFlowElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn DocumentElement(&self, node: &Self::Node) -> Option<Self::Node>;
    fn Body(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsDocument(&self, node: &Self::Node) -> bool;
    fn UsePositionIfIsVisuallyEquivalentCandidateEnabled(&self) -> bool;
    fn HasLayoutObject(&self, node: &Self::Node) -> bool;
    fn InSameLine(
        &self,
        first: &PositionWithAffinity<Self::Node>,
        second: &PositionWithAffinity<Self::Node>,
    ) -> bool;
    fn InlineFormattingContext(&self, position: &Position<Self::Node>) -> Option<u64>;
    fn CompareFlatPositions(
        &self,
        first: &Position<Self::Node>,
        second: &Position<Self::Node>,
    ) -> Option<Ordering>;
    fn StartOfLine(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> PositionWithAffinity<Self::Node>;
}
pub trait VisiblePositionContext: DownstreamVisiblePositionContext {
    fn BidiCaretAffinityEnabled(&self) -> bool;
    fn CaretBoundsEqual(
        &self,
        first: &PositionWithAffinity<Self::Node>,
        second: &PositionWithAffinity<Self::Node>,
    ) -> bool;
}
fn CanonicalizeCandidate<C: DownstreamVisiblePositionContext>(
    context: &C,
    candidate: Position<C::Node>,
) -> Position<C::Node> {
    if candidate.IsNull() {
        return candidate;
    }
    let upstream = context.MostBackwardCaretPosition(&candidate);
    if context.IsVisuallyEquivalentCandidate(&upstream) {
        upstream
    } else {
        candidate
    }
}
fn InSameBlock<C: DownstreamVisiblePositionContext>(
    context: &C,
    original: Option<&C::Node>,
    new: &C::Node,
) -> bool {
    let Some(original) = original else {
        return false;
    };
    if context.IsDescendantOf(new, original) {
        return true;
    }
    let block = context.EnclosingBlockFlowElement(original);
    if context.IsEditable(original) && context.IsEditable(new) {
        return block
            .as_ref()
            .is_some_and(|block| new == block || context.IsDescendantOf(new, block));
    }
    block == context.EnclosingBlockFlowElement(new)
}
pub fn CanonicalPositionOf<C: DownstreamVisiblePositionContext>(
    context: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    if position.IsNull() {
        return Position::default();
    }
    let backward = context.MostBackwardCaretPosition(position);
    if context.IsVisuallyEquivalentCandidate(&backward) {
        return backward;
    }
    let forward = context.MostForwardCaretPosition(position);
    if context.IsVisuallyEquivalentCandidate(&forward) {
        return forward;
    }
    let next = CanonicalizeCandidate(context, context.NextCandidate(position));
    let prev = CanonicalizeCandidate(context, context.PreviousCandidate(position));
    let container = position.ComputeContainerNode(context);
    if let Some(node) = &container {
        if context.DocumentElement(node).as_ref() == Some(node)
            && !context.IsEditable(node)
            && context
                .Body(node)
                .is_some_and(|body| context.IsEditable(&body))
        {
            return if !next.IsNull() { next } else { prev };
        }
    }
    let root = context.RootEditableElement(position);
    let anchor = position.AnchorNode().expect("non-null position");
    if root
        .as_ref()
        .is_some_and(|root| context.DocumentElement(root).as_ref() == Some(root))
        || context.IsDocument(anchor)
    {
        return if !next.IsNull() { next } else { prev };
    }
    let prev_same = !prev.IsNull() && context.RootEditableElement(&prev) == root;
    let next_same = !next.IsNull() && context.RootEditableElement(&next) == root;
    if prev_same && !next_same {
        return prev;
    }
    if next_same && !prev_same {
        return next;
    }
    if !next_same && !prev_same {
        if root.is_some()
            && context.UsePositionIfIsVisuallyEquivalentCandidateEnabled()
            && context.IsVisuallyEquivalentCandidate(position)
        {
            return position.clone();
        }
        return Position::default();
    }
    let next_same_block = next
        .AnchorNode()
        .is_some_and(|node| InSameBlock(context, container.as_ref(), node));
    let prev_same_block = prev
        .AnchorNode()
        .is_some_and(|node| InSameBlock(context, container.as_ref(), node));
    if prev_same_block && !next_same_block {
        prev
    } else {
        next
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisiblePosition<N> {
    position: PositionWithAffinity<N>,
}
impl<N> Default for VisiblePosition<N> {
    fn default() -> Self {
        Self {
            position: PositionWithAffinity::default(),
        }
    }
}
impl<N: Clone + PartialEq> VisiblePosition<N> {
    pub fn IsNull(&self) -> bool {
        self.position.position.IsNull()
    }
    pub fn DeepEquivalent(&self) -> &Position<N> {
        &self.position.position
    }
    pub fn ToPositionWithAffinity(&self) -> &PositionWithAffinity<N> {
        &self.position
    }
    pub fn CreateDownstream<C: DownstreamVisiblePositionContext<Node = N>>(
        context: &C,
        position: PositionWithAffinity<N>,
    ) -> Self {
        debug_assert_eq!(position.affinity, TextAffinity::Downstream);
        if position.position.IsNull() {
            return Self::default();
        }
        let deep = CanonicalPositionOf(context, &position.position);
        if deep.IsNull() {
            return Self::default();
        }
        let downstream = PositionWithAffinity {
            position: deep.clone(),
            affinity: TextAffinity::Downstream,
        };
        if position == downstream {
            return Self {
                position: downstream,
            };
        }
        let different_lines_same_context =
            |first: &PositionWithAffinity<N>, second: &PositionWithAffinity<N>| {
                if first == second
                    || !first
                        .position
                        .AnchorNode()
                        .is_some_and(|node| context.HasLayoutObject(node))
                    || !second
                        .position
                        .AnchorNode()
                        .is_some_and(|node| context.HasLayoutObject(node))
                    || context.InSameLine(first, second)
                {
                    return false;
                }
                let block = context.InlineFormattingContext(&first.position);
                block.is_some() && block == context.InlineFormattingContext(&second.position)
            };
        if context.CompareFlatPositions(&deep, &position.position) == Some(Ordering::Less)
            && different_lines_same_context(&position, &downstream)
        {
            let start = context.StartOfLine(&position);
            if !start.position.IsNull() {
                return Self { position: start };
            }
        }
        return Self {
            position: downstream,
        };
    }
    pub fn Create<C: VisiblePositionContext<Node = N>>(
        context: &C,
        position: PositionWithAffinity<N>,
    ) -> Self {
        if position.affinity == TextAffinity::Downstream {
            return Self::CreateDownstream(context, position);
        }
        if position.position.IsNull() {
            return Self::default();
        }
        let deep = CanonicalPositionOf(context, &position.position);
        if deep.IsNull() {
            return Self::default();
        }
        let downstream = PositionWithAffinity {
            position: deep.clone(),
            affinity: TextAffinity::Downstream,
        };
        let upstream = PositionWithAffinity {
            position: deep,
            affinity: TextAffinity::Upstream,
        };
        if context.BidiCaretAffinityEnabled()
            && context
                .InlineFormattingContext(&upstream.position)
                .is_some()
        {
            return Self {
                position: if context.CaretBoundsEqual(&downstream, &upstream) {
                    downstream
                } else {
                    upstream
                },
            };
        }
        Self {
            position: if context.InSameLine(&downstream, &upstream) {
                downstream
            } else {
                upstream
            },
        }
    }
}
