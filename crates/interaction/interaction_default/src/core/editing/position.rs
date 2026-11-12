// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/position.cc:177-308,362-387,465-551;
// core/editing/position_with_affinity.h; core/editing/text_affinity.h.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PositionAnchorType {
    #[default]
    OffsetInAnchor,
    BeforeAnchor,
    AfterAnchor,
    AfterChildren,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAffinity {
    #[default]
    Downstream,
    Upstream,
}
/// A source position is a small value anchored to the canonical node owner.
/// Neighbor-anchor kinds survive until the DOM Range boundary conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position<N> {
    anchor: Option<N>,
    offset: usize,
    kind: PositionAnchorType,
}
impl<N> Default for Position<N> {
    fn default() -> Self {
        Self {
            anchor: None,
            offset: 0,
            kind: PositionAnchorType::OffsetInAnchor,
        }
    }
}
/// Typed leaves on the live DOM. The Position implementation owns conversions;
/// implementations neither cache nodes nor reconstruct editing positions.
pub trait PositionTree {
    type Node: Clone + PartialEq;
    fn Parent(&self, node: &Self::Node) -> Option<Self::Node>;
    fn ChildAt(&self, node: &Self::Node, index: usize) -> Option<Self::Node>;
    fn ChildCount(&self, node: &Self::Node) -> usize;
    fn NodeIndex(&self, node: &Self::Node) -> usize;
    fn PreviousSibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn NextSibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn IsText(&self, node: &Self::Node) -> bool;
    fn CharacterDataLength(&self, node: &Self::Node) -> Option<usize>;
}
pub trait PositionDOM: PositionTree {
    fn EditingIgnoresContent(&self, node: &Self::Node) -> bool;
    fn IsDisplayInsideTable(&self, node: &Self::Node) -> bool;
    fn IsConnected(&self, node: &Self::Node) -> bool;
    fn DocumentIdentity(&self, node: &Self::Node) -> u128;
    fn LastDOMOffsetForEditing(&self, node: &Self::Node) -> usize {
        self.LastOffsetForEditing(node)
    }
    // Source: editing_strategy.cc:61-81.
    fn LastOffsetForEditing(&self, node: &Self::Node) -> usize {
        if let Some(length) = self.CharacterDataLength(node) {
            return length;
        }
        let children = self.ChildCount(node);
        if children != 0 {
            return children;
        }
        usize::from(self.EditingIgnoresContent(node))
    }
}
impl<N: Clone + PartialEq> Position<N> {
    // Source: position.cc:367-371.
    pub fn IsBeforeChildren<D: PositionTree<Node = N>>(&self, dom: &D) -> bool {
        if self.AnchorType() == PositionAnchorType::BeforeAnchor {
            return self
                .AnchorNode()
                .is_some_and(|node| dom.PreviousSibling(node).is_none());
        }
        self.IsOffsetInAnchor() && self.OffsetInAnchor() == 0
    }
    // Source: position.cc:421-428. Equivalent anchor forms are normalized by
    // their original strategy; caret pixel rounding is not a substitute.
    pub fn IsEquivalent<D: PositionTree<Node = N>>(&self, dom: &D, other: &Self) -> bool {
        if self.IsNull() {
            return other.IsNull();
        }
        if self.AnchorType() == other.AnchorType() {
            return self == other;
        }
        self.ToOffsetInAnchor(dom) == other.ToOffsetInAnchor(dom)
    }
    // Source editing_utilities.cc:789-835, kCodeUnit branch.
    pub fn PreviousCodeUnitPosition<D: PositionDOM<Node = N>>(&self, dom: &D) -> Self {
        self.PreviousWithOffset(dom, |_, offset| offset - 1)
    }
    pub(crate) fn PreviousWithOffset<D: PositionDOM<Node = N>>(
        &self,
        dom: &D,
        previous: impl Fn(&N, usize) -> usize,
    ) -> Self {
        let Some(node) = self.AnchorNode() else {
            return self.clone();
        };
        let offset = self.ComputeEditingOffset(dom);
        if offset > 0 {
            if dom.EditingIgnoresContent(node) {
                return Self::BeforeNode(node.clone());
            }
            if let Some(child) = dom.ChildAt(node, offset - 1) {
                return Self::LastPositionInOrAfterNode(dom, &child);
            }
            return Self::new(node.clone(), previous(node, offset));
        }
        if let Some(parent) = dom.Parent(node) {
            return if dom.EditingIgnoresContent(&parent) {
                Self::BeforeNode(parent)
            } else {
                Self::new(parent, dom.NodeIndex(node))
            };
        }
        self.clone()
    }
    // Source: position.cc:71-91. Editing positions use neighbor anchors for
    // nodes whose contents cannot contain a Range endpoint.
    pub fn EditingPositionOf<D: PositionDOM<Node = N>>(dom: &D, node: N, offset: usize) -> Self {
        if dom.IsText(&node) || !dom.EditingIgnoresContent(&node) {
            return Self::new(node, offset);
        }
        if offset == 0 {
            Self::BeforeNode(node)
        } else {
            Self::AfterNode(node)
        }
    }
    pub fn new(anchor: N, offset: usize) -> Self {
        Self {
            anchor: Some(anchor),
            offset,
            kind: PositionAnchorType::OffsetInAnchor,
        }
    }
    pub fn IsNull(&self) -> bool {
        self.anchor.is_none()
    }
    pub fn AnchorNode(&self) -> Option<&N> {
        self.anchor.as_ref()
    }
    pub fn AnchorType(&self) -> PositionAnchorType {
        self.kind
    }
    pub fn OffsetInAnchor(&self) -> usize {
        self.offset
    }
    pub fn IsBeforeAnchor(&self) -> bool {
        self.kind == PositionAnchorType::BeforeAnchor
    }
    pub fn IsOffsetInAnchor(&self) -> bool {
        self.kind == PositionAnchorType::OffsetInAnchor
    }
    pub fn IsAfterAnchorOrAfterChildren(&self) -> bool {
        matches!(
            self.kind,
            PositionAnchorType::AfterAnchor | PositionAnchorType::AfterChildren
        )
    }
    pub fn ComputeContainerNode<D: PositionTree<Node = N>>(&self, dom: &D) -> Option<N> {
        let anchor = self.anchor.as_ref()?;
        match self.kind {
            PositionAnchorType::OffsetInAnchor | PositionAnchorType::AfterChildren => {
                Some(anchor.clone())
            }
            PositionAnchorType::BeforeAnchor | PositionAnchorType::AfterAnchor => {
                dom.Parent(anchor)
            }
        }
    }
    pub fn LastOffsetInNode<D: PositionTree<Node = N>>(dom: &D, node: &N) -> usize {
        dom.CharacterDataLength(node)
            .unwrap_or_else(|| dom.ChildCount(node))
    }
    pub fn ComputeOffsetInContainerNode<D: PositionTree<Node = N>>(&self, dom: &D) -> usize {
        let Some(anchor) = self.anchor.as_ref() else {
            return 0;
        };
        match self.kind {
            PositionAnchorType::AfterChildren => Self::LastOffsetInNode(dom, anchor),
            PositionAnchorType::OffsetInAnchor => {
                self.offset.min(Self::LastOffsetInNode(dom, anchor))
            }
            PositionAnchorType::BeforeAnchor => dom.NodeIndex(anchor),
            PositionAnchorType::AfterAnchor => dom.NodeIndex(anchor) + 1,
        }
    }
    pub fn ComputeEditingOffset<D: PositionDOM<Node = N>>(&self, dom: &D) -> usize {
        if self.IsAfterAnchorOrAfterChildren() {
            self.anchor
                .as_ref()
                .map(|node| dom.LastOffsetForEditing(node))
                .unwrap_or(0)
        } else {
            self.offset
        }
    }
    pub fn ParentAnchoredEquivalent<D: PositionDOM<Node = N>>(&self, dom: &D) -> Self {
        let Some(anchor) = self.anchor.as_ref() else {
            return Self::default();
        };
        let special = dom.EditingIgnoresContent(anchor) || dom.IsDisplayInsideTable(anchor);
        if self.offset == 0 && !self.IsAfterAnchorOrAfterChildren() {
            if dom.Parent(anchor).is_some() && special {
                return Self::InParentBeforeNode(dom, anchor);
            }
            return Self::new(anchor.clone(), 0);
        }
        if dom.CharacterDataLength(anchor).is_none()
            && (self.IsAfterAnchorOrAfterChildren() || self.offset == dom.ChildCount(anchor))
            && special
            && self.ComputeContainerNode(dom).is_some()
        {
            return Self::InParentAfterNode(dom, anchor);
        }
        self.ToOffsetInAnchor(dom)
    }
    pub fn ToOffsetInAnchor<D: PositionTree<Node = N>>(&self, dom: &D) -> Self {
        self.ComputeContainerNode(dom)
            .map(|node| Self::new(node, self.ComputeOffsetInContainerNode(dom)))
            .unwrap_or_default()
    }
    pub fn ComputeNodeBeforePosition<D: PositionTree<Node = N>>(&self, dom: &D) -> Option<N> {
        let anchor = self.anchor.as_ref()?;
        match self.kind {
            PositionAnchorType::AfterChildren => dom
                .ChildCount(anchor)
                .checked_sub(1)
                .and_then(|index| dom.ChildAt(anchor, index)),
            PositionAnchorType::OffsetInAnchor => self
                .offset
                .checked_sub(1)
                .and_then(|index| dom.ChildAt(anchor, index)),
            PositionAnchorType::BeforeAnchor => dom.PreviousSibling(anchor),
            PositionAnchorType::AfterAnchor => Some(anchor.clone()),
        }
    }
    // Source position.cc:334-345, Range::PastLastNode.
    pub fn NodeAsRangePastLastNode<D: PositionTree<Node = N>>(&self, dom: &D) -> Option<N> {
        let position = self.ToOffsetInAnchor(dom);
        let node = position.AnchorNode()?;
        if dom.CharacterDataLength(node).is_none() {
            if let Some(child) = dom.ChildAt(node, position.OffsetInAnchor()) {
                return Some(child);
            }
        }
        super::text_offset_mapping::NextSkippingChildren(dom, node, None)
    }
    pub fn ComputeNodeAfterPosition<D: PositionTree<Node = N>>(&self, dom: &D) -> Option<N> {
        let anchor = self.anchor.as_ref()?;
        match self.kind {
            PositionAnchorType::AfterChildren => None,
            PositionAnchorType::OffsetInAnchor => dom.ChildAt(anchor, self.offset),
            PositionAnchorType::BeforeAnchor => Some(anchor.clone()),
            PositionAnchorType::AfterAnchor => dom.NextSibling(anchor),
        }
    }
    pub fn IsConnected<D: PositionDOM<Node = N>>(&self, dom: &D) -> bool {
        self.anchor
            .as_ref()
            .is_some_and(|node| dom.IsConnected(node))
    }
    pub fn IsValidFor<D: PositionDOM<Node = N>>(&self, dom: &D, document: u128) -> bool {
        let Some(node) = self.anchor.as_ref() else {
            return true;
        };
        dom.DocumentIdentity(node) == document
            && dom.IsConnected(node)
            && (!self.IsOffsetInAnchor() || self.offset <= Self::LastOffsetInNode(dom, node))
    }
    pub fn AtFirstEditingPositionForNode<D: PositionDOM<Node = N>>(&self, dom: &D) -> bool {
        let Some(node) = self.anchor.as_ref() else {
            return true;
        };
        match self.kind {
            PositionAnchorType::OffsetInAnchor => self.offset == 0,
            PositionAnchorType::BeforeAnchor => true,
            PositionAnchorType::AfterAnchor | PositionAnchorType::AfterChildren => {
                dom.LastDOMOffsetForEditing(node) == 0
            }
        }
    }
    pub fn AtLastEditingPositionForNode<D: PositionDOM<Node = N>>(&self, dom: &D) -> bool {
        self.anchor.as_ref().is_none_or(|node| {
            self.IsAfterAnchorOrAfterChildren() || self.offset >= dom.LastDOMOffsetForEditing(node)
        })
    }
    pub fn AtStartOfTree<D: PositionTree<Node = N>>(&self, dom: &D) -> bool {
        self.anchor.as_ref().is_none_or(|node| {
            dom.Parent(node).is_none() && self.ComputeNodeBeforePosition(dom).is_none()
        })
    }
    pub fn InParentBeforeNode<D: PositionTree<Node = N>>(dom: &D, node: &N) -> Self {
        dom.Parent(node)
            .map(|parent| Self::new(parent, dom.NodeIndex(node)))
            .unwrap_or_default()
    }
    pub fn InParentAfterNode<D: PositionTree<Node = N>>(dom: &D, node: &N) -> Self {
        dom.Parent(node)
            .map(|parent| Self::new(parent, dom.NodeIndex(node) + 1))
            .unwrap_or_default()
    }
    pub fn BeforeNode(node: N) -> Self {
        Self {
            anchor: Some(node),
            offset: 0,
            kind: PositionAnchorType::BeforeAnchor,
        }
    }
    pub fn AfterNode(node: N) -> Self {
        Self {
            anchor: Some(node),
            offset: 0,
            kind: PositionAnchorType::AfterAnchor,
        }
    }
    pub fn AfterChildren(node: N) -> Self {
        Self {
            anchor: Some(node),
            offset: 0,
            kind: PositionAnchorType::AfterChildren,
        }
    }
    pub fn FirstPositionInNode(node: N) -> Self {
        Self::new(node, 0)
    }
    pub fn LastPositionInNode<D: PositionTree<Node = N>>(dom: &D, node: &N) -> Self {
        if dom.IsText(node) {
            Self::new(node.clone(), Self::LastOffsetInNode(dom, node))
        } else {
            Self {
                anchor: Some(node.clone()),
                offset: 0,
                kind: PositionAnchorType::AfterChildren,
            }
        }
    }
    pub fn FirstPositionInOrBeforeNode<D: PositionDOM<Node = N>>(dom: &D, node: &N) -> Self {
        if dom.EditingIgnoresContent(node) {
            Self::BeforeNode(node.clone())
        } else {
            Self::FirstPositionInNode(node.clone())
        }
    }
    pub fn LastPositionInOrAfterNode<D: PositionDOM<Node = N>>(dom: &D, node: &N) -> Self {
        if dom.EditingIgnoresContent(node) && dom.Parent(node).is_some() {
            Self::AfterNode(node.clone())
        } else {
            Self::LastPositionInNode(dom, node)
        }
    }
}
#[derive(Clone, Debug)]
pub struct PositionWithAffinity<N> {
    pub position: Position<N>,
    pub affinity: TextAffinity,
}
impl<N: PartialEq> PartialEq for PositionWithAffinity<N> {
    fn eq(&self, other: &Self) -> bool {
        if self.position.anchor.is_none() {
            return other.position.anchor.is_none();
        }
        self.position == other.position && self.affinity == other.affinity
    }
}
impl<N: Eq> Eq for PositionWithAffinity<N> {}
impl<N> Default for PositionWithAffinity<N> {
    fn default() -> Self {
        Self {
            position: Position::default(),
            affinity: TextAffinity::Downstream,
        }
    }
}
