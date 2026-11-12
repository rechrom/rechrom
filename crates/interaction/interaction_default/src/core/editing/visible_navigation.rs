/*
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009 Apple Inc. All rights
 * reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: visible_units.cc:1191-1282; position_units.cc:140-194.
use super::{
    editing_boundaries::{self, EditableBoundaryContext},
    position::{Position, PositionWithAffinity, TextAffinity},
    visible_position::{DownstreamVisiblePositionContext, VisiblePosition, VisiblePositionContext},
    visible_units::EditingBoundaryCrossingRule as Rule,
};
fn WithPosition<N>(position: Position<N>) -> PositionWithAffinity<N> {
    PositionWithAffinity {
        position,
        affinity: TextAffinity::Downstream,
    }
}
pub fn SkipToEndOfEditingBoundary<C: EditableBoundaryContext>(
    ctx: &C,
    position: Position<C::Node>,
    anchor: &Position<C::Node>,
) -> Position<C::Node> {
    if position.IsNull() {
        return position;
    }
    let root = ctx.HighestEditableRoot(anchor);
    let position_root = ctx.HighestEditableRoot(&position);
    if root == position_root {
        return position;
    }
    if root.is_none() {
        if let Some(root) = position_root {
            return Position::AfterNode(root).ParentAnchoredEquivalent(ctx);
        }
    }
    editing_boundaries::FirstEditablePositionAfterPositionInRoot(
        ctx,
        &position,
        &root.expect("source editable root"),
    )
}
pub fn SkipToStartOfEditingBoundary<C: EditableBoundaryContext>(
    ctx: &C,
    position: Position<C::Node>,
    anchor: &Position<C::Node>,
) -> Position<C::Node> {
    if position.IsNull() {
        return position;
    }
    let root = ctx.HighestEditableRoot(anchor);
    let position_root = ctx.HighestEditableRoot(&position);
    if root == position_root {
        return position;
    }
    if root.is_none() {
        if let Some(root) = position_root {
            return editing_boundaries::PreviousVisuallyDistinctCandidate(
                ctx,
                &Position::BeforeNode(root).ParentAnchoredEquivalent(ctx),
            );
        }
    }
    editing_boundaries::LastEditablePositionBeforeInRoot(
        ctx,
        &position,
        &root.expect("source editable root"),
    )
}
pub fn NextPositionOf<C: EditableBoundaryContext + VisiblePositionContext>(
    ctx: &C,
    position: &PositionWithAffinity<C::Node>,
    rule: Rule,
) -> VisiblePosition<C::Node> {
    let next = VisiblePosition::Create(
        ctx,
        PositionWithAffinity {
            position: editing_boundaries::VisuallyDistinctCandidate(
                ctx,
                &position.position,
                rule,
                false,
            ),
            affinity: position.affinity,
        },
    );
    match rule {
        Rule::CanCross => next,
        Rule::CannotCross => VisiblePosition::Create(
            ctx,
            editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
                ctx,
                next.ToPositionWithAffinity().clone(),
                &position.position,
            ),
        ),
        Rule::CanSkipOver => VisiblePosition::Create(
            ctx,
            WithPosition(SkipToEndOfEditingBoundary(
                ctx,
                next.DeepEquivalent().clone(),
                &position.position,
            )),
        ),
    }
}
pub fn PreviousPositionOf<C: EditableBoundaryContext + DownstreamVisiblePositionContext>(
    ctx: &C,
    position: &Position<C::Node>,
    rule: Rule,
) -> VisiblePosition<C::Node> {
    let previous = editing_boundaries::VisuallyDistinctCandidate(ctx, position, rule, true);
    if previous.AtStartOfTree(ctx) {
        return VisiblePosition::default();
    }
    let previous = VisiblePosition::CreateDownstream(ctx, WithPosition(previous));
    if previous.DeepEquivalent() == position {
        return VisiblePosition::default();
    }
    match rule {
        Rule::CanCross => previous,
        Rule::CannotCross => VisiblePosition::CreateDownstream(
            ctx,
            editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
                ctx,
                previous.ToPositionWithAffinity().clone(),
                position,
            ),
        ),
        Rule::CanSkipOver => VisiblePosition::CreateDownstream(
            ctx,
            WithPosition(SkipToStartOfEditingBoundary(
                ctx,
                previous.DeepEquivalent().clone(),
                position,
            )),
        ),
    }
}
// Source: position_units.cc:42-64,248-269.
pub fn StartOfDocument<C: VisiblePositionContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    position
        .AnchorNode()
        .and_then(|node| ctx.DocumentElement(node))
        .map_or_else(Position::default, Position::FirstPositionInNode)
}
pub fn EndOfDocument<C: VisiblePositionContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    position
        .AnchorNode()
        .and_then(|node| ctx.DocumentElement(node))
        .map_or_else(Position::default, |node| {
            Position::LastPositionInNode(ctx, &node)
        })
}
pub fn IsEndOfEditableOrNonEditableContent<C: EditableBoundaryContext + VisiblePositionContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> bool {
    if position.IsNull() {
        return false;
    }
    let end = ctx.HighestEditableRoot(position).map_or_else(
        || EndOfDocument(ctx, position),
        |root| Position::LastPositionInNode(ctx, &root),
    );
    position == &end
}
// Source: visible_units.cc:1284-1335. Searching stops at the original DOM
// enclosing block-flow while emitted characters/positions use this strategy.
pub fn SkipWhitespace<C>(ctx: &C, position: &Position<C::Node>) -> Position<C::Node>
where
    C: VisiblePositionContext + super::iterators::text_iterator::TextIteratorContext,
{
    let Some(node) = position.ComputeContainerNode(ctx) else {
        return position.clone();
    };
    if ctx.DocumentElement(&node).is_none() {
        return position.clone();
    }
    let Some(block) = ctx.EnclosingBlockFlowElement(&node) else {
        return position.clone();
    };
    let end = Position::LastPositionInNode(ctx, &block);
    let mut behavior = super::iterators::text_iterator_behavior::TextIteratorBehavior::default();
    behavior.SetEmitsCharactersBetweenAllVisiblePositions(true);
    let mut iterator =
        super::iterators::character_iterator::CharacterIterator::new(ctx, position, &end, behavior);
    let mut runner = position.clone();
    while iterator.length() != 0 {
        let ch = iterator.CharacterAt(0);
        if (!foundation::unicode::IsSpaceOrNewline(ch) && ch != 0x00a0) || ch == 10 {
            return runner;
        }
        runner = iterator.EndPosition();
        iterator.Advance(1);
    }
    runner
}

// position_units.cc:218-234. These are source positions in the current flat
// strategy's highest editable root, not a DOM-text-offset approximation.
pub fn StartOfEditableContent<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    ctx.HighestEditableRoot(position)
        .map_or_else(Position::default, Position::FirstPositionInNode)
}
pub fn EndOfEditableContent<C: EditableBoundaryContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    ctx.HighestEditableRoot(position)
        .map_or_else(Position::default, |root| {
            Position::LastPositionInNode(ctx, &root)
        })
}
