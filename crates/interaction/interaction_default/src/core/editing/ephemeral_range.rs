// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source core/editing/ephemeral_range.h:42-116; .cc:41-88,133-141.
use super::{
    compare_positions::CompareFlatPositions,
    position::{Position, PositionDOM},
    position_conversion::{PositionTreeScope, ToPositionInFlatTree},
    text_offset_mapping::{NextNode, NodeAsRangeFirstNode},
};
/// Source stack value. It never registers live boundaries or retains a second
/// DOM tree; only a DOM Range owner can adjust endpoints during mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EphemeralRange<N> {
    start: Position<N>,
    end: Position<N>,
}
impl<N> Default for EphemeralRange<N> {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}
impl<N: Clone + PartialEq> EphemeralRange<N> {
    pub fn new(start: Position<N>, end: Position<N>) -> Self {
        debug_assert_eq!(start.IsNull(), end.IsNull());
        Self { start, end }
    }
    pub fn IsNull(&self) -> bool {
        self.start.IsNull()
    }
    pub fn IsCollapsed(&self) -> bool {
        self.start == self.end
    }
    pub fn StartPosition(&self) -> &Position<N> {
        &self.start
    }
    pub fn EndPosition(&self) -> &Position<N> {
        &self.end
    }
    // Source: ephemeral_range.cc:142-145. This iterator borrows the original
    // traversal owner; it never materializes selected nodes or a copied tree.
    pub fn Nodes<'a, C: PositionDOM<Node = N>>(
        &self,
        context: &'a C,
    ) -> EphemeralRangeNodes<'a, C> {
        EphemeralRangeNodes {
            context,
            current: NodeAsRangeFirstNode(context, &self.start),
            past_last: self.end.NodeAsRangePastLastNode(context),
        }
    }
    pub fn RangeOfContents<C: PositionDOM<Node = N>>(context: &C, node: N) -> Self {
        Self::new(
            Position::FirstPositionInNode(node.clone()),
            Position::LastPositionInNode(context, &node),
        )
    }
}

pub struct EphemeralRangeNodes<'a, C: PositionDOM> {
    context: &'a C,
    current: Option<C::Node>,
    past_last: Option<C::Node>,
}
impl<C: PositionDOM> Clone for EphemeralRangeNodes<'_, C> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            current: self.current.clone(),
            past_last: self.past_last.clone(),
        }
    }
}
impl<C: PositionDOM> Iterator for EphemeralRangeNodes<'_, C> {
    type Item = C::Node;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.past_last {
            return None;
        }
        let node = self.current.take()?;
        self.current = NextNode(self.context, &node, None);
        Some(node)
    }
}

// Source: ephemeral_range.cc:218-230.
pub fn ToEphemeralRangeInFlatTree<D: PositionTreeScope, F: PositionDOM<Node = D::Node>>(
    dom: &D,
    flat: &F,
    range: &EphemeralRange<D::Node>,
) -> EphemeralRange<D::Node> {
    let start = ToPositionInFlatTree(dom, flat, range.StartPosition());
    let end = ToPositionInFlatTree(dom, flat, range.EndPosition());
    let (Some(first), Some(last)) = (start.AnchorNode(), end.AnchorNode()) else {
        return EphemeralRange::default();
    };
    let document = flat.DocumentIdentity(first);
    if document != flat.DocumentIdentity(last)
        || !start.IsValidFor(flat, document)
        || !end.IsValidFor(flat, document)
    {
        return EphemeralRange::default();
    }
    if CompareFlatPositions(flat, &start, &end) == Some(std::cmp::Ordering::Greater) {
        EphemeralRange::new(end, start)
    } else {
        EphemeralRange::new(start, end)
    }
}
