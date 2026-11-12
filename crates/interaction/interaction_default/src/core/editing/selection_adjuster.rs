/*
 * Copyright (C) 2004, 2005, 2006, 2007, 2008, 2009, 2010 Apple Inc. All rights
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
// Source: core/editing/selection_adjuster.cc:109-402 (GranularityAdjuster).
use super::{
    position::{Position, PositionWithAffinity, TextAffinity},
    text_granularity::TextGranularity,
    visible_position::{VisiblePosition, VisiblePositionContext},
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WordSide {
    PreviousWordIfOnBoundary,
    NextWordIfOnBoundary,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WordInclusion {
    Default,
    Middle,
}

/// The source word specialization can be instantiated without admitting
/// unrelated sentence/document operations. Both selection entry points below
/// call the same granularity helpers.
pub trait WordGranularityContext: VisiblePositionContext {
    fn CompareWordPositions(
        &self,
        a: &Position<Self::Node>,
        b: &Position<Self::Node>,
    ) -> Option<Ordering>;
    fn StartOfWord(&self, p: &Position<Self::Node>, side: WordSide) -> Position<Self::Node>;
    fn EndOfWord(&self, p: &Position<Self::Node>, side: WordSide) -> Position<Self::Node>;
    fn MiddleOfWord(
        &self,
        start: &Position<Self::Node>,
        end: &Position<Self::Node>,
    ) -> Position<Self::Node>;
    fn IsStartOfLine(&self, p: &VisiblePosition<Self::Node>) -> bool;
    fn IsEndOfLine(&self, p: &VisiblePosition<Self::Node>) -> bool;
    fn IsEndOfParagraph(&self, p: &VisiblePosition<Self::Node>) -> bool;
    fn IsEndOfEditableOrNonEditableContent(&self, p: &VisiblePosition<Self::Node>) -> bool;
    fn NextPosition(
        &self,
        p: &VisiblePosition<Self::Node>,
        cannot_cross: bool,
    ) -> VisiblePosition<Self::Node>;
    fn IsEmptyTableCell(&self, node: Option<&Self::Node>) -> bool;
    fn EnclosingTableCell(&self, p: &Position<Self::Node>) -> Option<Self::Node>;
    fn TableElementJustBefore(&self, p: &VisiblePosition<Self::Node>) -> Option<Self::Node>;
    fn IsEnclosingBlock(&self, node: &Self::Node) -> bool;
}

pub fn ComputeWordStart<C: WordGranularityContext>(
    context: &C,
    passed: &PositionWithAffinity<C::Node>,
    inclusion: WordInclusion,
) -> Position<C::Node> {
    let visible = VisiblePosition::Create(context, passed.clone());
    let side = ChooseWordSide(context, &visible);
    let start = context.StartOfWord(&passed.position, side);
    if inclusion == WordInclusion::Middle {
        let end = context.EndOfWord(&passed.position, side);
        let middle = context.MiddleOfWord(&start, &end);
        if context.CompareWordPositions(&passed.position, &middle) == Some(Ordering::Greater) {
            return end;
        }
    }
    Visible(context, start).DeepEquivalent().clone()
}
pub fn ComputeWordEnd<C: WordGranularityContext>(
    context: &C,
    start: &Position<C::Node>,
    passed: &PositionWithAffinity<C::Node>,
    inclusion: WordInclusion,
) -> Position<C::Node> {
    let original = VisiblePosition::Create(context, passed.clone());
    let paragraph_end = context.IsEndOfParagraph(&original);
    let side = ChooseWordSide(context, &original);
    let word_end = if paragraph_end {
        original.clone()
    } else {
        Visible(context, context.EndOfWord(&passed.position, side))
    };
    if inclusion == WordInclusion::Middle {
        let word_start = context.StartOfWord(&passed.position, side);
        let middle = context.MiddleOfWord(&word_start, word_end.DeepEquivalent());
        if middle.IsNull()
            || context.CompareWordPositions(&middle, &passed.position) == Some(Ordering::Greater)
        {
            return word_start;
        }
    }
    if !paragraph_end
        || context.IsEmptyTableCell(start.AnchorNode())
        || context
            .EnclosingTableCell(original.DeepEquivalent())
            .is_some()
    {
        return word_end.DeepEquivalent().clone();
    }
    IncludeParagraphBreak(context, word_end)
}

/// Editing's visible-units operations over the same strategy as its Positions.
/// The implementations read resident caret/line/text-iterator owners; no
/// projected DOM text or additional selection state is admitted here.
pub trait GranularityContext: WordGranularityContext {
    fn ComparePositions(
        &self,
        a: &Position<Self::Node>,
        b: &Position<Self::Node>,
    ) -> Option<Ordering>;
    fn StartOfSentence(&self, p: &Position<Self::Node>) -> Position<Self::Node>;
    fn EndOfSentence(&self, p: &VisiblePosition<Self::Node>) -> VisiblePosition<Self::Node>;
    fn EndOfLine(&self, p: &PositionWithAffinity<Self::Node>) -> PositionWithAffinity<Self::Node>;
    fn StartOfParagraph(&self, p: &VisiblePosition<Self::Node>) -> VisiblePosition<Self::Node>;
    fn EndOfParagraph(&self, p: &VisiblePosition<Self::Node>) -> VisiblePosition<Self::Node>;
    fn StartOfDocument(&self, p: &Position<Self::Node>) -> Position<Self::Node>;
    fn EndOfDocument(&self, p: &VisiblePosition<Self::Node>) -> VisiblePosition<Self::Node>;
    fn PreviousPosition(&self, p: &VisiblePosition<Self::Node>) -> VisiblePosition<Self::Node>;
}

fn ChooseWordSide<C: WordGranularityContext>(
    context: &C,
    p: &VisiblePosition<C::Node>,
) -> WordSide {
    if context.IsEndOfEditableOrNonEditableContent(p)
        || (context.IsEndOfLine(p) && !context.IsStartOfLine(p) && !context.IsEndOfParagraph(p))
    {
        WordSide::PreviousWordIfOnBoundary
    } else {
        WordSide::NextWordIfOnBoundary
    }
}
fn Visible<C: VisiblePositionContext>(
    context: &C,
    p: Position<C::Node>,
) -> VisiblePosition<C::Node> {
    VisiblePosition::Create(
        context,
        PositionWithAffinity {
            position: p,
            affinity: TextAffinity::Downstream,
        },
    )
}

pub fn ComputeStartRespectingGranularity<C: GranularityContext>(
    context: &C,
    passed_start: &PositionWithAffinity<C::Node>,
    granularity: TextGranularity,
    inclusion: WordInclusion,
) -> Position<C::Node> {
    assert!(!passed_start.position.IsNull());
    match granularity {
        TextGranularity::kCharacter => passed_start.position.clone(),
        TextGranularity::kWord => ComputeWordStart(context, passed_start, inclusion),
        TextGranularity::kSentence | TextGranularity::kSentenceBoundary => {
            context.StartOfSentence(&passed_start.position)
        }
        TextGranularity::kLine | TextGranularity::kLineBoundary => {
            let p = VisiblePosition::Create(context, passed_start.clone());
            VisiblePosition::Create(context, context.StartOfLine(p.ToPositionWithAffinity()))
                .DeepEquivalent()
                .clone()
        }
        TextGranularity::kParagraph => {
            let p = VisiblePosition::Create(context, passed_start.clone());
            let p = if context.IsStartOfLine(&p) && context.IsEndOfEditableOrNonEditableContent(&p)
            {
                context.PreviousPosition(&p)
            } else {
                p
            };
            context.StartOfParagraph(&p).DeepEquivalent().clone()
        }
        TextGranularity::kDocumentBoundary => {
            Visible(context, context.StartOfDocument(&passed_start.position))
                .DeepEquivalent()
                .clone()
        }
        TextGranularity::kParagraphBoundary => context
            .StartOfParagraph(&VisiblePosition::Create(context, passed_start.clone()))
            .DeepEquivalent()
            .clone(),
    }
}

// The paragraph break after a block table's last cell extends to the next
// paragraph. An inline table has no such break. Preserve null fallback and the
// explicit cannot-cross-editing-boundary second step from the source.
fn IncludeParagraphBreak<C: WordGranularityContext>(
    context: &C,
    end: VisiblePosition<C::Node>,
) -> Position<C::Node> {
    let next = context.NextPosition(&end, false);
    let Some(table) = context.TableElementJustBefore(&next) else {
        return if next.IsNull() { end } else { next }
            .DeepEquivalent()
            .clone();
    };
    if !context.IsEnclosingBlock(&table) {
        return end.DeepEquivalent().clone();
    }
    let after = context.NextPosition(&next, true);
    if after.IsNull() { end } else { after }
        .DeepEquivalent()
        .clone()
}

pub fn ComputeEndRespectingGranularity<C: GranularityContext>(
    context: &C,
    start: &Position<C::Node>,
    passed_end: &PositionWithAffinity<C::Node>,
    granularity: TextGranularity,
    inclusion: WordInclusion,
) -> Position<C::Node> {
    assert!(!passed_end.position.IsNull());
    match granularity {
        TextGranularity::kCharacter => passed_end.position.clone(),
        TextGranularity::kWord => ComputeWordEnd(context, start, passed_end, inclusion),
        TextGranularity::kSentence | TextGranularity::kSentenceBoundary => context
            .EndOfSentence(&VisiblePosition::Create(context, passed_end.clone()))
            .DeepEquivalent()
            .clone(),
        TextGranularity::kLine => {
            let end = VisiblePosition::Create(context, context.EndOfLine(passed_end));
            if !context.IsEndOfParagraph(&end) {
                return end.DeepEquivalent().clone();
            }
            let next = context.NextPosition(&end, false);
            if next.IsNull() { end } else { next }
                .DeepEquivalent()
                .clone()
        }
        TextGranularity::kLineBoundary => context.EndOfLine(passed_end).position,
        TextGranularity::kParagraph => {
            let end = context.EndOfParagraph(&VisiblePosition::Create(context, passed_end.clone()));
            if context.EnclosingTableCell(end.DeepEquivalent()).is_some() {
                return end.DeepEquivalent().clone();
            }
            IncludeParagraphBreak(context, end)
        }
        TextGranularity::kDocumentBoundary => context
            .EndOfDocument(&VisiblePosition::Create(context, passed_end.clone()))
            .DeepEquivalent()
            .clone(),
        TextGranularity::kParagraphBoundary => context
            .EndOfParagraph(&VisiblePosition::Create(context, passed_end.clone()))
            .DeepEquivalent()
            .clone(),
    }
}

/// The canonical source selection supplies its ordered endpoints, affinity and
/// direction. Only new endpoint values are returned; no selection is retained.
pub fn AdjustSelectionRespectingGranularity<C: GranularityContext>(
    context: &C,
    start: &Position<C::Node>,
    end: &Position<C::Node>,
    affinity: TextAffinity,
    granularity: TextGranularity,
    inclusion: WordInclusion,
    dom_tree: bool,
) -> (Position<C::Node>, Position<C::Node>) {
    let expanded_start = ComputeStartRespectingGranularity(
        context,
        &PositionWithAffinity {
            position: start.clone(),
            affinity,
        },
        granularity,
        inclusion,
    );
    let expanded_start = if expanded_start.IsNull() {
        start.clone()
    } else {
        expanded_start
    };
    let expanded_end = ComputeEndRespectingGranularity(
        context,
        &expanded_start,
        &PositionWithAffinity {
            position: end.clone(),
            affinity,
        },
        granularity,
        inclusion,
    );
    let expanded_end = if expanded_end.IsNull() {
        end.clone()
    } else {
        expanded_end
    };
    if dom_tree
        && context.ComparePositions(&expanded_start, &expanded_end) == Some(Ordering::Greater)
    {
        (expanded_end, expanded_start)
    } else {
        (expanded_start, expanded_end)
    }
}

/// Source: selection_controller.cc:974-983 delegates range endpoint adjustment
/// to BidiAdjustment. Kept with the source selection adjusters, not Page policy.
pub fn AdjustSelectionRespectingBidiBoundary<C: super::bidi_adjustment::BidiAdjustmentContext>(
    context: &C,
    anchor: &PositionWithAffinity<C::Node>,
    focus: &PositionWithAffinity<C::Node>,
) -> (Position<C::Node>, Position<C::Node>) {
    super::bidi_adjustment::AdjustForRangeSelection(context, anchor, focus)
}
