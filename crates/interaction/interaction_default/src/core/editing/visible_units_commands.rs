// Copyright (C) 2004-2009 Apple Inc. All rights reserved.
// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: visible_units.cc:546-559; visible_units_paragraph.cc:42-55,78-91,
// 161-191; visible_units_line.cc:509-517; local_caret_rect.cc:57-86;
// commands/editing_commands_utilities.cc:274-301,843-879; edit_command.cc:54-71.
use super::{
    editing_boundaries::EditableBoundaryContext,
    position::{Position, PositionAnchorType, PositionWithAffinity, TextAffinity},
    position_units_paragraph::{self, ParagraphContext},
    visible_position::{VisiblePosition, VisiblePositionContext},
    visible_units::EditingBoundaryCrossingRule as Rule,
    visible_units_line::{self, EndOfLineContext},
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;

/// Only source leaf facts cross this port. Paragraph, block, line and caret
/// equivalence decisions remain in their translated Editing owners.
pub trait CommandPositionContext:
    ParagraphContext + VisiblePositionContext + EndOfLineContext + EditableBoundaryContext
{
    type CaretQuad: PartialEq;
    fn AbsoluteCaretBounds(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> foundation::gfx::Rect;
    fn IsHorizontalWritingMode(&self, node: &Self::Node) -> bool;

    fn IsHTMLBR(&self, node: &Self::Node) -> bool;
    fn IsLayoutText(&self, node: &Self::Node) -> bool;
    fn ShouldCollapseBreaks(&self, node: &Self::Node) -> bool;
    fn TextCodeUnit(&self, node: &Self::Node, offset: usize) -> Option<u16>;
    fn EditingUseDomPositionApiEnabled(&self) -> bool;
    fn IsBeforeNonCollapsedContent(&self, position: &Position<Self::Node>) -> bool;
    fn CaretQuad(&self, position: &Position<Self::Node>) -> Option<Self::CaretQuad>;
    fn IsLayoutBlockFlow(&self, node: &Self::Node) -> bool;
}
fn Visible<C: VisiblePositionContext>(ctx: &C, p: Position<C::Node>) -> VisiblePosition<C::Node> {
    VisiblePosition::Create(
        ctx,
        PositionWithAffinity {
            position: p,
            affinity: TextAffinity::Downstream,
        },
    )
}
pub fn StartOfParagraph<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
) -> VisiblePosition<C::Node> {
    StartOfParagraphWithRule(ctx, p, Rule::CannotCross)
}
pub fn EndOfParagraph<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
) -> VisiblePosition<C::Node> {
    EndOfParagraphWithRule(ctx, p, Rule::CannotCross)
}
pub fn StartOfParagraphWithRule<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
    rule: Rule,
) -> VisiblePosition<C::Node> {
    Visible(
        ctx,
        position_units_paragraph::StartOfParagraph(ctx, p.DeepEquivalent(), rule),
    )
}
pub fn EndOfParagraphWithRule<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
    rule: Rule,
) -> VisiblePosition<C::Node> {
    Visible(
        ctx,
        position_units_paragraph::EndOfParagraph(ctx, p.DeepEquivalent(), rule),
    )
}
pub fn IsStartOfParagraph<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
) -> bool {
    IsStartOfParagraphWithRule(ctx, p, Rule::CannotCross)
}
pub fn IsEndOfParagraph<C: CommandPositionContext>(ctx: &C, p: &VisiblePosition<C::Node>) -> bool {
    IsEndOfParagraphWithRule(ctx, p, Rule::CannotCross)
}
pub fn IsStartOfParagraphWithRule<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
    rule: Rule,
) -> bool {
    !p.IsNull() && p.DeepEquivalent() == StartOfParagraphWithRule(ctx, p, rule).DeepEquivalent()
}
pub fn IsEndOfParagraphWithRule<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
    rule: Rule,
) -> bool {
    !p.IsNull() && p.DeepEquivalent() == EndOfParagraphWithRule(ctx, p, rule).DeepEquivalent()
}
fn BlockBoundary<C: CommandPositionContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
    start: bool,
) -> VisiblePosition<C::Node> {
    let Some(container) = p.DeepEquivalent().ComputeContainerNode(ctx) else {
        return VisiblePosition::default();
    };
    let Some(block) = ParagraphContext::EnclosingBlock(
        ctx,
        &Position::FirstPositionInOrBeforeNode(ctx, &container),
        Rule::CanCross,
    ) else {
        return VisiblePosition::default();
    };
    Visible(
        ctx,
        if start {
            Position::FirstPositionInNode(block)
        } else {
            Position::LastPositionInNode(ctx, &block)
        },
    )
}
pub fn IsStartOfBlock<C: CommandPositionContext>(ctx: &C, p: &VisiblePosition<C::Node>) -> bool {
    !p.DeepEquivalent().IsNull()
        && p.DeepEquivalent() == BlockBoundary(ctx, p, true).DeepEquivalent()
}
pub fn IsEndOfBlock<C: CommandPositionContext>(ctx: &C, p: &VisiblePosition<C::Node>) -> bool {
    !p.DeepEquivalent().IsNull()
        && p.DeepEquivalent() == BlockBoundary(ctx, p, false).DeepEquivalent()
}
pub fn IsStartOfLine<C: CommandPositionContext>(ctx: &C, p: &VisiblePosition<C::Node>) -> bool {
    let start = visible_units_line::StartOfLine(ctx, p.ToPositionWithAffinity());
    !p.DeepEquivalent().IsNull()
        && p.DeepEquivalent() == VisiblePosition::Create(ctx, start).DeepEquivalent()
}
/// Source visible_units_line.cc:530-541.
pub fn IsEndOfLine<C: CommandPositionContext>(ctx: &C, p: &VisiblePosition<C::Node>) -> bool {
    let end = visible_units_line::EndOfLine(ctx, p.ToPositionWithAffinity());
    !p.DeepEquivalent().IsNull() && p.DeepEquivalent() == &end.position
}
pub fn LineBreakExistsAtPosition<C: CommandPositionContext>(
    ctx: &C,
    p: &Position<C::Node>,
) -> bool {
    let Some(node) = p.AnchorNode() else {
        return false;
    };
    if ctx.IsHTMLBR(node) && p.AtFirstEditingPositionForNode(ctx) {
        return true;
    }
    if ctx.EditingUseDomPositionApiEnabled()
        && p.ComputeNodeBeforePosition(ctx)
            .is_some_and(|n| ctx.IsHTMLBR(&n))
    {
        return true;
    }
    if !DownstreamVisiblePositionContext::HasLayoutObject(ctx, node)
        || !ctx.IsText(node)
        || ctx.ShouldCollapseBreaks(node)
    {
        return false;
    }
    ctx.TextCodeUnit(node, p.ComputeOffsetInContainerNode(ctx)) == Some(b'\n' as u16)
}
pub fn IsRenderedCharacter<C: CommandPositionContext>(ctx: &C, p: &Position<C::Node>) -> bool {
    let Some(node) = p.AnchorNode() else {
        return false;
    };
    debug_assert!(p.IsOffsetInAnchor());
    ctx.IsText(node) && ctx.IsLayoutText(node) && ctx.IsBeforeNonCollapsedContent(p)
}
pub fn RendersInDifferentPosition<C: CommandPositionContext>(
    ctx: &C,
    a: &Position<C::Node>,
    b: &Position<C::Node>,
) -> bool {
    if a.IsNull() || b.IsNull() {
        return false;
    }
    ctx.CaretQuad(a) != ctx.CaretQuad(b)
}
/// Source local_caret_rect.cc:57-86. Inline adjustment itself is performed by
/// the existing InlineAdjustment owner before this source normalization.
pub fn AdjustForInlineCaretPosition<C: CommandPositionContext>(
    ctx: &C,
    p: PositionWithAffinity<C::Node>,
) -> PositionWithAffinity<C::Node> {
    let Some(node) = p.position.AnchorNode() else {
        return p;
    };
    let position = match p.position.AnchorType() {
        PositionAnchorType::BeforeAnchor | PositionAnchorType::AfterAnchor => return p,
        PositionAnchorType::AfterChildren => Position::AfterNode(node.clone()),
        PositionAnchorType::OffsetInAnchor => {
            if ctx.IsText(node)
                || p.position.ComputeOffsetInContainerNode(ctx) != 0
                || !DownstreamVisiblePositionContext::HasLayoutObject(ctx, node)
                || ctx.IsLayoutBlockFlow(node)
            {
                return p;
            }
            Position::BeforeNode(node.clone())
        }
    };
    PositionWithAffinity {
        position,
        affinity: p.affinity,
    }
}

/// Source visible_units_paragraph.cc:58-65. This overload returns a DOM visible
/// position after performing only the paragraph traversal with flat strategy.
pub fn StartOfParagraphInFlatTree<D, F>(
    dom: &D,
    flat: &F,
    p: &VisiblePosition<D::Node>,
    rule: Rule,
) -> VisiblePosition<D::Node>
where
    D: CommandPositionContext,
    F: ParagraphContext<Node = D::Node> + super::position_conversion::PositionTreeScope,
{
    let position = super::position_conversion::ToPositionInFlatTree(dom, flat, p.DeepEquivalent());
    let start = position_units_paragraph::StartOfParagraph(flat, &position, rule);
    Visible(
        dom,
        super::position_conversion::ToPositionInDOMTree(dom, flat, &start),
    )
}
/// Source visible_units_paragraph.cc:94-101.
pub fn EndOfParagraphInFlatTree<D, F>(
    dom: &D,
    flat: &F,
    p: &VisiblePosition<D::Node>,
    rule: Rule,
) -> VisiblePosition<D::Node>
where
    D: CommandPositionContext,
    F: ParagraphContext<Node = D::Node> + super::position_conversion::PositionTreeScope,
{
    let position = super::position_conversion::ToPositionInFlatTree(dom, flat, p.DeepEquivalent());
    let end = position_units_paragraph::EndOfParagraph(flat, &position, rule);
    Visible(
        dom,
        super::position_conversion::ToPositionInDOMTree(dom, flat, &end),
    )
}

/// Source visible_units.cc:1385-1436. The zero-width/height endpoint rectangle
/// is intentional; a selected-range union is not this source query.
pub fn FirstRectForRange<C: CommandPositionContext>(
    ctx: &C,
    start: &Position<C::Node>,
    end: &Position<C::Node>,
) -> foundation::gfx::Rect {
    use foundation::gfx::{Point, Rect, Size};
    let start = PositionWithAffinity {
        position: Visible(ctx, start.clone()).DeepEquivalent().clone(),
        affinity: TextAffinity::Downstream,
    };
    let mut start_rect = ctx.AbsoluteCaretBounds(&start);
    if start_rect.IsEmpty() {
        return Rect::default();
    }
    let end = PositionWithAffinity {
        position: Visible(ctx, end.clone()).DeepEquivalent().clone(),
        affinity: TextAffinity::Upstream,
    };
    let end = if visible_units_line::InSameLine(ctx, &start, &end) {
        end
    } else {
        visible_units_line::EndOfLine(ctx, &start)
    };
    let mut end_rect = ctx.AbsoluteCaretBounds(&end);
    if end_rect.IsEmpty() {
        return Rect::default();
    }
    if start
        .position
        .AnchorNode()
        .is_some_and(|node| ctx.IsHorizontalWritingMode(node))
    {
        start_rect.set_width(0);
        end_rect.set_width(0);
    } else {
        start_rect.set_height(0);
        end_rect.set_height(0);
    }
    let x = start_rect.x().min(end_rect.x());
    let y = start_rect.y().min(end_rect.y());
    Rect::new(
        Point::new(x, y),
        Size::new(
            start_rect.right().max(end_rect.right()) - x,
            start_rect.bottom().max(end_rect.bottom()) - y,
        ),
    )
}
