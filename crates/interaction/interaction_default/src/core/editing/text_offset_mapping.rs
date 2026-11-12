// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: text_offset_mapping.cc:97-106,354-365,417-448;
// position.cc:311-326; html/forms/text_control_element.cc:1262-1280.
use super::position::{Position, PositionTree};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;

pub trait TextOffsetMappingContext: PositionTree {
    type InlineContents: Clone + PartialEq;
    /// The native owner performs ComputeInlineContentsFromNode and retains the
    /// source block identity; an anonymous block is never a fabricated DOM id.
    fn ComputeInlineContentsFromNode(&self, node: &Self::Node) -> Option<Self::InlineContents>;
    fn IsTextControl(&self, node: &Self::Node) -> bool;
    fn HasUserAgentShadowRoot(&self, node: &Self::Node) -> bool;
    fn UserAgentShadowHost(&self, node: &Self::Node) -> Option<Self::Node>;
}
// Source: text_offset_mapping.cc:69-82,365-414.
fn PreviousNode<C: PositionTree>(
    ctx: &C,
    node: &C::Node,
    stay_within: Option<&C::Node>,
) -> Option<C::Node> {
    if Some(node) == stay_within {
        return None;
    }
    if let Some(mut previous) = ctx.PreviousSibling(node) {
        while let Some(last) = ctx
            .ChildCount(&previous)
            .checked_sub(1)
            .and_then(|index| ctx.ChildAt(&previous, index))
        {
            previous = last;
        }
        Some(previous)
    } else {
        ctx.Parent(node)
    }
}
fn PreviousNodeSkippingAncestors<C: PositionTree>(ctx: &C, node: &C::Node) -> Option<C::Node> {
    let mut parent = ctx.Parent(node);
    let mut runner = PreviousNode(ctx, node, None);
    while let Some(node) = runner {
        if parent.as_ref() != Some(&node) {
            return Some(node);
        }
        parent = ctx.Parent(&node);
        runner = PreviousNode(ctx, &node, None);
    }
    None
}
pub fn NodeAsRangeLastNode<C: PositionTree>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::Node> {
    let position = position.ToOffsetInAnchor(ctx);
    let node = position.AnchorNode()?;
    let past_last = if ctx.CharacterDataLength(node).is_some() {
        NextSkippingChildren(ctx, node, None)
    } else {
        ctx.ChildAt(node, position.OffsetInAnchor())
            .or_else(|| NextSkippingChildren(ctx, node, None))
    };
    if let Some(past_last) = past_last {
        return PreviousNode(ctx, &past_last, None);
    }
    let mut last = position.ComputeContainerNode(ctx)?;
    while let Some(child) = ctx
        .ChildCount(&last)
        .checked_sub(1)
        .and_then(|index| ctx.ChildAt(&last, index))
    {
        last = child;
    }
    Some(last)
}
pub fn FindBackwardInlineContents<C: TextOffsetMappingContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::InlineContents> {
    let previous = NodeAsRangeLastNode(ctx, position)?;
    if let Some(control) = EnclosingTextControl(ctx, position) {
        if !IsDescendantOf(ctx, &previous, &control) {
            return None;
        }
        let mut node = Some(previous);
        while let Some(current) = node {
            if let Some(contents) = ctx.ComputeInlineContentsFromNode(&current) {
                return Some(contents);
            }
            node = PreviousNode(ctx, &current, Some(&control));
        }
        return None;
    }
    let control = EnclosingTextControl(ctx, &Position::new(previous.clone(), 0));
    let mut node = Some(control.unwrap_or(previous));
    while let Some(current) = node {
        if let Some(contents) = ctx.ComputeInlineContentsFromNode(&current) {
            return Some(contents);
        }
        node = PreviousNodeSkippingAncestors(ctx, &current).map(|previous| {
            EnclosingTextControl(ctx, &Position::new(previous.clone(), 0))
                .or_else(|| ctx.UserAgentShadowHost(&previous))
                .unwrap_or(previous)
        });
    }
    None
}

/// Original InlineContents range and the two traversal boundaries. Layout
/// retains pointer identity; Editing receives canonical native Position values.
pub trait InlineContentsContext: TextOffsetMappingContext {
    fn InlineContentsRange(
        &self,
        contents: &Self::InlineContents,
    ) -> Option<(Position<Self::Node>, Position<Self::Node>)>;
    fn FirstPositionAfterBlockFlow(&self, contents: &Self::InlineContents) -> Position<Self::Node>;
    fn LastPositionBeforeBlockFlow(&self, contents: &Self::InlineContents) -> Position<Self::Node>;
}
pub struct InlineContentsIterator<'a, C: InlineContentsContext> {
    context: &'a C,
    current: Option<C::InlineContents>,
    backward: bool,
}
impl<'a, C: InlineContentsContext> InlineContentsIterator<'a, C> {
    pub fn ForwardRangeOf(context: &'a C, position: &Position<C::Node>) -> Self {
        Self {
            context,
            current: FindForwardInlineContents(context, position),
            backward: false,
        }
    }
    pub fn BackwardRangeOf(context: &'a C, position: &Position<C::Node>) -> Self {
        Self {
            context,
            current: FindBackwardInlineContents(context, position),
            backward: true,
        }
    }
}
impl<C: InlineContentsContext> Iterator for InlineContentsIterator<'_, C> {
    type Item = C::InlineContents;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        let position = if self.backward {
            self.context.LastPositionBeforeBlockFlow(&current)
        } else {
            self.context.FirstPositionAfterBlockFlow(&current)
        };
        if !position.IsNull() {
            self.current = if self.backward {
                FindBackwardInlineContents(self.context, &position)
            } else {
                FindForwardInlineContents(self.context, &position)
            };
        }
        Some(current)
    }
}

use super::iterators::{
    character_iterator::CharacterIterator,
    text_iterator::{PlainText, RangeLength, TextIteratorContext},
    text_iterator_behavior::TextIteratorBehavior,
};
/// Source text_offset_mapping.cc:280-346. The range's text is emitted by the
/// same rendered TextIterator used by selection serialization. Mapping back is
/// performed by CharacterIterator, including generated separators/replacements.
pub struct TextOffsetMapping<'a, C: TextIteratorContext + InlineContentsContext> {
    context: &'a C,
    range: (Position<C::Node>, Position<C::Node>),
    behavior: TextIteratorBehavior,
    text: Vec<u16>,
}
impl<'a, C: TextIteratorContext + InlineContentsContext> TextOffsetMapping<'a, C> {
    pub fn new(context: &'a C, contents: &C::InlineContents) -> Option<Self> {
        let mut behavior = TextIteratorBehavior::default();
        behavior
            .SetEmitsCharactersBetweenAllVisiblePositions(true)
            .SetEmitsSmallXForTextSecurity(true);
        Self::WithBehavior(context, contents, behavior)
    }
    pub fn WithBehavior(
        context: &'a C,
        contents: &C::InlineContents,
        behavior: TextIteratorBehavior,
    ) -> Option<Self> {
        let range = context.InlineContentsRange(contents)?;
        let text = PlainText(context, &range.0, &range.1, behavior);
        Some(Self {
            context,
            range,
            behavior,
            text,
        })
    }
    pub fn GetText(&self) -> &[u16] {
        &self.text
    }
    pub fn GetRange(&self) -> &(Position<C::Node>, Position<C::Node>) {
        &self.range
    }
    pub fn ComputeTextOffset(&self, position: &Position<C::Node>) -> usize {
        use std::cmp::Ordering;
        if self
            .context
            .VisibleContext()
            .CompareFlatPositions(position, &self.range.0)
            .is_some_and(|order| order != Ordering::Greater)
        {
            return 0;
        }
        if self
            .context
            .VisibleContext()
            .CompareFlatPositions(position, &self.range.1)
            .is_some_and(|order| order != Ordering::Less)
        {
            return self.text.len();
        }
        RangeLength(self.context, &self.range.0, position, self.behavior)
    }
    pub fn GetPositionBefore(&self, offset: usize) -> Position<C::Node> {
        assert!(offset <= self.text.len());
        let mut iterator =
            CharacterIterator::new(self.context, &self.range.0, &self.range.1, self.behavior);
        if offset >= 1 && offset == self.text.len() {
            iterator.Advance(offset - 1);
            return iterator.GetPositionAfter();
        }
        iterator.Advance(offset);
        iterator.GetPositionBefore()
    }
    pub fn GetPositionAfter(&self, offset: usize) -> Position<C::Node> {
        assert!(offset <= self.text.len());
        let mut iterator =
            CharacterIterator::new(self.context, &self.range.0, &self.range.1, self.behavior);
        iterator.Advance(offset);
        iterator.GetPositionAfter()
    }
    pub fn ComputeRange(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(Position<C::Node>, Position<C::Node>)> {
        assert!(start <= end && end <= self.text.len());
        (start != end).then(|| (self.GetPositionBefore(start), self.GetPositionAfter(end)))
    }
    pub fn FindNonWhitespaceCharacterFrom(&self, offset: usize) -> usize {
        (offset..self.text.len())
            .find(|&index| !matches!(self.text[index], 0x00a0 | 0x20 | 0x0a | 0x09))
            .unwrap_or(self.text.len())
    }
}
pub fn NextSkippingChildren<C: PositionTree>(
    ctx: &C,
    node: &C::Node,
    stay_within: Option<&C::Node>,
) -> Option<C::Node> {
    let mut node = node.clone();
    loop {
        if Some(&node) == stay_within {
            return None;
        }
        if let Some(next) = ctx.NextSibling(&node) {
            return Some(next);
        }
        node = ctx.Parent(&node)?;
    }
}
pub fn NextNode<C: PositionTree>(
    ctx: &C,
    node: &C::Node,
    stay_within: Option<&C::Node>,
) -> Option<C::Node> {
    ctx.ChildAt(node, 0)
        .or_else(|| NextSkippingChildren(ctx, node, stay_within))
}
pub fn IsDescendantOf<C: PositionTree>(ctx: &C, node: &C::Node, ancestor: &C::Node) -> bool {
    let mut parent = ctx.Parent(node);
    while let Some(node) = parent {
        if &node == ancestor {
            return true;
        }
        parent = ctx.Parent(&node);
    }
    false
}
pub fn NodeAsRangeFirstNode<C: PositionTree>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::Node> {
    let position = position.ToOffsetInAnchor(ctx);
    let node = position.AnchorNode()?;
    if ctx.CharacterDataLength(node).is_some() {
        return Some(node.clone());
    }
    if let Some(child) = ctx.ChildAt(node, position.OffsetInAnchor()) {
        return Some(child);
    }
    if position.OffsetInAnchor() == 0 {
        return Some(node.clone());
    }
    NextSkippingChildren(ctx, node, None)
}
pub fn EnclosingTextControl<C: TextOffsetMappingContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::Node> {
    let container = position.ComputeContainerNode(ctx)?;
    if ctx.IsTextControl(&container) {
        return Some(container);
    }
    ctx.UserAgentShadowHost(&container)
        .filter(|host| ctx.IsTextControl(host))
}
pub fn FindForwardInlineContents<C: TextOffsetMappingContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Option<C::InlineContents> {
    let next = NodeAsRangeFirstNode(ctx, position)?;
    let control = EnclosingTextControl(ctx, position);
    if control
        .as_ref()
        .is_some_and(|control| !IsDescendantOf(ctx, &next, control))
    {
        return None;
    }
    let mut node = Some(next);
    while let Some(current) = node {
        if let Some(contents) = ctx.ComputeInlineContentsFromNode(&current) {
            return Some(contents);
        }
        node = if let Some(control) = &control {
            NextNode(ctx, &current, Some(control))
        } else if ctx.IsTextControl(&current) || ctx.HasUserAgentShadowRoot(&current) {
            NextSkippingChildren(ctx, &current, None)
        } else {
            NextNode(ctx, &current, None)
        };
    }
    None
}

use super::visible_position::VisiblePositionContext;
