// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! serialization.cc:228–327 and styled_markup_serializer.cc:173–261. The
//! geometry context borrows the existing Position, caret and paragraph owners.
use super::super::{
    compare_positions,
    editing_boundaries::EditableBoundaryContext,
    position::{Position, PositionDOM, PositionTree, PositionWithAffinity, TextAffinity},
    position_conversion::PositionTreeScope,
    position_units_paragraph::{self as paragraph, ParagraphContext},
    text_offset_mapping, visible_navigation,
    visible_position::{DownstreamVisiblePositionContext, VisiblePosition, VisiblePositionContext},
    visible_units::EditingBoundaryCrossingRule as Rule,
};
use super::{styled_markup_accumulator::TextOffset, styled_markup_serializer::MarkupRange};
use std::cmp::Ordering;
pub trait MarkupRangeContext:
    ParagraphContext + VisiblePositionContext + EditableBoundaryContext + PositionTreeScope
{
    fn IsHTMLTag(&self, node: Self::Node, name: &str) -> bool;
    fn IsMathMLMath(&self, node: Self::Node) -> bool;
    fn IsMailBlockquote(&self, node: Self::Node) -> bool;
    fn IsTabSpan(&self, node: Self::Node) -> bool;
    fn HasLayoutOwner(&self, node: Self::Node) -> bool;
    fn SameNormalizedDOMRange(
        &self,
        node: Self::Node,
        start: &Position<Self::Node>,
        end: &Position<Self::Node>,
    ) -> bool;
}
fn TextOffsetFor<C: MarkupRangeContext>(
    context: &C,
    p: &Position<C::Node>,
) -> Option<TextOffset<C::Node>>
where
    C::Node: Copy,
{
    let p = p.ParentAnchoredEquivalent(context);
    let node = p.ComputeContainerNode(context)?;
    context.IsText(&node).then_some(TextOffset {
        node,
        offset: p.ComputeOffsetInContainerNode(context),
    })
}
fn InterchangeAfter<C: MarkupRangeContext>(context: &C, position: &Position<C::Node>) -> bool
where
    C::Node: Copy,
{
    let visible = VisiblePosition::Create(
        context,
        PositionWithAffinity {
            position: position.clone(),
            affinity: TextAffinity::Downstream,
        },
    );
    let next = visible_navigation::NextPositionOf(
        context,
        visible.ToPositionWithAffinity(),
        Rule::CannotCross,
    );
    let upstream =
        DownstreamVisiblePositionContext::MostBackwardCaretPosition(context, next.DeepEquivalent());
    let downstream = DownstreamVisiblePositionContext::MostForwardCaretPosition(
        context,
        visible.DeepEquivalent(),
    );
    paragraph::IsEndOfParagraph(context, visible.DeepEquivalent(), Rule::CannotCross)
        && paragraph::IsStartOfParagraph(context, next.DeepEquivalent(), Rule::CannotCross)
        && !(upstream.AnchorNode() == downstream.AnchorNode()
            && upstream
                .AnchorNode()
                .and_then(|n| Some(*n))
                .is_some_and(|n| context.IsHTMLTag(n, "br")))
}
pub fn PrepareRange<C: MarkupRangeContext>(
    context: &C,
    start: Position<C::Node>,
    end: Position<C::Node>,
    annotate: bool,
) -> Option<MarkupRange<C::Node>>
where
    C::Node: Copy,
{
    PrepareRangeWithConstrainingAncestor(context, start, end, annotate, None)
}
pub fn PrepareRangeWithConstrainingAncestor<C: MarkupRangeContext>(
    context: &C,
    start: Position<C::Node>,
    end: Position<C::Node>,
    annotate: bool,
    constraining_ancestor: Option<C::Node>,
) -> Option<MarkupRange<C::Node>>
where
    C::Node: Copy,
{
    let first = text_offset_mapping::NodeAsRangeFirstNode(context, &start);
    let past = end.NodeAsRangePastLastNode(context);
    let visible_end = VisiblePosition::Create(
        context,
        PositionWithAffinity {
            position: end.clone(),
            affinity: TextAffinity::Downstream,
        },
    );
    let previous = visible_navigation::PreviousPositionOf(
        context,
        visible_end.DeepEquivalent(),
        Rule::CannotCross,
    );
    let interchange_start = annotate && InterchangeAfter(context, &start);
    let interchange_end = annotate && InterchangeAfter(context, previous.DeepEquivalent());
    let next = visible_navigation::NextPositionOf(
        context,
        &PositionWithAffinity {
            position: start.clone(),
            affinity: TextAffinity::Downstream,
        },
        Rule::CannotCross,
    );
    let next_node = next.DeepEquivalent().AnchorNode().copied();
    let first_after = if let (Some(next), Some(past)) = (next_node, past) {
        if compare_positions::CompareFlatPositions(
            context,
            &Position::BeforeNode(next),
            &Position::BeforeNode(past),
        ) != Some(Ordering::Less)
        {
            Some(past)
        } else {
            Some(next)
        }
    } else {
        next_node
    };
    let common = compare_positions::CommonAncestor(
        context,
        &start.ComputeContainerNode(context)?,
        &end.ComputeContainerNode(context)?,
    )?;
    let same_range = |node: C::Node| context.SameNormalizedDOMRange(node, &start, &end);
    let highest = HighestAncestor(
        context,
        common,
        first,
        &start,
        annotate,
        &same_range,
        constraining_ancestor,
    );
    let mut body = Some(common);
    while let Some(node) = body {
        if context.IsHTMLTag(node, "body") {
            break;
        }
        body = PositionTree::Parent(context, &node);
    }
    Some(MarkupRange {
        first_node: first,
        past_end: past,
        start: TextOffsetFor(context, &start),
        end: TextOffsetFor(context, &end),
        highest_node_to_serialize: highest,
        fully_selected_root: body.filter(|n| same_range(*n)),
        interchange_newline_at_start: interchange_start,
        interchange_newline_at_end: interchange_end,
        first_node_after_start_newline: first_after,
        only_interchange_newline: interchange_start && &start == previous.DeepEquivalent(),
    })
}
fn HighestAncestor<C: MarkupRangeContext>(
    context: &C,
    common: C::Node,
    first: Option<C::Node>,
    _start: &Position<C::Node>,
    annotate: bool,
    same_range: &dyn Fn(C::Node) -> bool,
    constraining_ancestor: Option<C::Node>,
) -> Option<C::Node>
where
    C::Node: Copy,
{
    let tag = |n: C::Node, name: &str| context.IsHTMLTag(n, name);
    let mut special = None;
    if annotate {
        if let Some(block) = ParagraphContext::EnclosingBlock(
            context,
            &Position::FirstPositionInNode(common),
            Rule::CanCross,
        ) {
            if tag(block, "tbody") || tag(block, "tr") {
                let mut parent = PositionTree::Parent(context, &block);
                while let Some(n) = parent {
                    if tag(n, "table") {
                        special = Some(n);
                        break;
                    }
                    parent = PositionTree::Parent(context, &n);
                }
            } else if [
                "listing", "ol", "pre", "table", "ul", "xmp", "h1", "h2", "h3", "h4", "h5",
            ]
            .iter()
            .any(|name| tag(block, name))
            {
                special = Some(block);
            }
        }
        if let Some(first) = first {
            let mut node = Some(first);
            let mut list_item = None;
            while let Some(n) = node {
                if tag(n, "li") {
                    list_item = Some(n);
                    break;
                }
                node = PositionTree::Parent(context, &n);
            }
            if let Some(li) = list_item.filter(|n| same_range(*n)) {
                let mut node = PositionTree::Parent(context, &li);
                while let Some(n) = node {
                    if tag(n, "ul") || tag(n, "ol") {
                        special = Some(n);
                        break;
                    }
                    node = PositionTree::Parent(context, &n);
                }
            }
            let mut node = Some(first);
            while let Some(n) = node {
                if context.IsMailBlockquote(n) {
                    special = Some(n);
                }
                node = PositionTree::Parent(context, &n);
            }
            let mut node = Some(first);
            while let Some(n) = node {
                if context.IsMathMLMath(n) {
                    special = Some(n);
                }
                node = PositionTree::Parent(context, &n);
            }
        }
    }
    let check = special.unwrap_or(common);
    if context.HasLayoutOwner(check) {
        let stop = constraining_ancestor.or_else(|| {
            ParagraphContext::EnclosingBlock(
                context,
                &Position::FirstPositionInNode(check),
                Rule::CanCross,
            )
        });
        let mut node = Some(check);
        while let Some(n) = node {
            if Some(n) == stop {
                break;
            }
            if ["u", "s", "strike", "i", "em", "b", "strong"]
                .iter()
                .any(|name| tag(n, name))
            {
                special = Some(n);
            }
            node = PositionTree::Parent(context, &n);
        }
    }
    if special.is_none() {
        let tab = if context.IsText(&common) {
            PositionTree::Parent(context, &common)
        } else {
            Some(common)
        };
        if let Some(tab) = tab.filter(|n| context.IsTabSpan(*n)) {
            special = Some(tab);
        }
    }
    let mut node = Some(special.unwrap_or(common));
    while let Some(n) = node {
        if tag(n, "a") {
            special = Some(n);
            break;
        }
        node = PositionTree::Parent(context, &n);
    }
    special
}
