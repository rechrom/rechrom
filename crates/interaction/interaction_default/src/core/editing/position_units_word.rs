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

// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/position_units_word.cc:54-99,284-329.
use super::{
    iterators::text_iterator::TextIteratorContext,
    position::Position,
    selection_adjuster::WordSide,
    text_offset_mapping::InlineContentsContext,
    text_segments::{FindBoundary, TextSegmentFinder, TextSegmentPosition},
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
/// The platform text owner uses its native UI-locale ICU WordBreakIterator.
/// No boundary table, regex or alternate segmentation algorithm is supplied.
pub trait WordBreakContext {
    fn FollowingWordBoundary(&self, text: &[u16], offset: usize) -> Option<usize>;
    fn PrecedingWordBoundary(&self, text: &[u16], offset: usize) -> Option<usize>;
}
struct WordFinder<'a, C> {
    context: &'a C,
    side: WordSide,
    first: bool,
    backward: bool,
}
impl<C: WordBreakContext> TextSegmentFinder for WordFinder<'_, C> {
    fn Find(&mut self, text: &[u16], mut offset: usize) -> TextSegmentPosition {
        assert!(offset <= text.len());
        if self.first {
            self.first = false;
            if self.backward {
                if self.side == WordSide::NextWordIfOnBoundary {
                    if offset == text.len() {
                        return TextSegmentPosition::After(text.len());
                    }
                    offset += 1;
                } else if offset == 0 {
                    return TextSegmentPosition::Before(0);
                }
            } else if self.side == WordSide::PreviousWordIfOnBoundary {
                if offset == 0 {
                    return TextSegmentPosition::Before(0);
                }
                offset -= 1;
            } else if offset == text.len() {
                return TextSegmentPosition::After(offset);
            }
        }
        if self.backward {
            self.context
                .PrecedingWordBoundary(text, offset)
                .map_or(TextSegmentPosition::None, TextSegmentPosition::Before)
        } else {
            self.context
                .FollowingWordBoundary(text, offset)
                .filter(|&result| result != 0)
                .map_or(TextSegmentPosition::None, |result| {
                    TextSegmentPosition::After(result - 1)
                })
        }
    }
}
pub fn StartOfWordPositionInternal<
    C: TextIteratorContext + InlineContentsContext + WordBreakContext,
>(
    context: &C,
    position: &Position<C::Node>,
    side: WordSide,
) -> Position<C::Node> {
    FindBoundary(
        context,
        position,
        &mut WordFinder {
            context,
            side,
            first: true,
            backward: true,
        },
        true,
    )
}
pub fn EndOfWordPositionInternal<
    C: TextIteratorContext + InlineContentsContext + WordBreakContext,
>(
    context: &C,
    position: &Position<C::Node>,
    side: WordSide,
) -> Position<C::Node> {
    FindBoundary(
        context,
        position,
        &mut WordFinder {
            context,
            side,
            first: true,
            backward: false,
        },
        false,
    )
}
// Source: position_units_word.cc:332-345,392-431. Public flat-tree wrappers
// preserve editability adjustment; DOM callers convert at their source facade.
pub fn StartOfWordPosition<C>(
    context: &C,
    position: &Position<C::Node>,
    side: WordSide,
) -> Position<C::Node>
where
    C: TextIteratorContext
        + InlineContentsContext
        + WordBreakContext
        + super::editing_boundaries::EditableBoundaryContext,
{
    super::editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
        context,
        super::position::PositionWithAffinity {
            position: StartOfWordPositionInternal(context, position, side),
            affinity: super::position::TextAffinity::Downstream,
        },
        position,
    )
    .position
}
pub fn EndOfWordPosition<C>(
    context: &C,
    position: &Position<C::Node>,
    side: WordSide,
) -> Position<C::Node>
where
    C: TextIteratorContext
        + InlineContentsContext
        + WordBreakContext
        + super::editing_boundaries::EditableBoundaryContext,
{
    super::editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
        context,
        super::position::PositionWithAffinity {
            position: EndOfWordPositionInternal(context, position, side),
            affinity: super::position::TextAffinity::Downstream,
        },
        position,
    )
    .position
}
pub fn MiddleOfWordPosition<C: TextIteratorContext + InlineContentsContext>(
    context: &C,
    start: &Position<C::Node>,
    end: &Position<C::Node>,
) -> Position<C::Node> {
    use super::{
        iterators::{text_iterator::RangeLength, text_iterator_behavior::TextIteratorBehavior},
        text_offset_mapping::{InlineContentsIterator, TextOffsetMapping},
        visible_position::VisiblePositionContext,
    };
    if !context
        .VisibleContext()
        .CompareFlatPositions(start, end)
        .is_some_and(|order| order == std::cmp::Ordering::Less)
    {
        return Position::default();
    }
    let mut middle = RangeLength(
        context,
        start,
        end,
        TextIteratorBehavior::DefaultRangeLengthBehavior(),
    ) / 2;
    let first = InlineContentsIterator::ForwardRangeOf(context, start)
        .next()
        .expect("word starts in inline contents");
    middle += TextOffsetMapping::new(context, &first)
        .expect("word inline range")
        .ComputeTextOffset(start);
    for contents in InlineContentsIterator::ForwardRangeOf(context, start) {
        let mapping = TextOffsetMapping::new(context, &contents).expect("word inline range");
        let length = mapping.GetText().len();
        if middle < length {
            return mapping.GetPositionBefore(middle);
        }
        middle -= length;
    }
    unreachable!("middle of nonempty source word range")
}

// Source position_units_word.cc:227-281,373-382,431-438.
pub fn IsWordBreak(ch: u16) -> bool {
    (foundation::unicode::IsPrintableChar(ch as i32) && !super::editing_utilities::IsWhitespace(ch))
        || (0xd800..=0xdfff).contains(&ch)
        || matches!(ch, 0x0a | 0x0d | 0x5f)
}
pub fn IsWordBoundary(ch: u16) -> bool {
    foundation::unicode::IsPunct(ch as i32)
        || foundation::unicode::IsSymbol(ch as i32)
        || (0xd800..=0xdfff).contains(&ch)
}
struct PreviousWordFinder<'a, C> {
    context: &'a C,
    first: bool,
}
impl<C: WordBreakContext> TextSegmentFinder for PreviousWordFinder<'_, C> {
    fn Find(&mut self, text: &[u16], offset: usize) -> TextSegmentPosition {
        assert!(offset <= text.len());
        if !self.first && !text.is_empty() && offset > 0 && IsWordBreak(text[offset - 1]) {
            return TextSegmentPosition::Before(offset);
        }
        self.first = false;
        if offset == 0 || text.is_empty() {
            return TextSegmentPosition::None;
        }
        let mut punctuation = None;
        let mut boundary = self.context.PrecedingWordBoundary(text, offset);
        while let Some(runner) = boundary {
            if runner < text.len() && IsWordBoundary(text[runner]) {
                if runner > 0 && foundation::unicode::IsAlphanumeric(text[runner - 1] as i32) {
                    return TextSegmentPosition::Before(runner);
                }
                punctuation = Some(runner);
            } else {
                if let Some(punctuation) = punctuation {
                    return TextSegmentPosition::Before(punctuation);
                }
                if runner < text.len() && IsWordBreak(text[runner]) {
                    return TextSegmentPosition::Before(runner);
                }
            }
            boundary = self.context.PrecedingWordBoundary(text, runner);
        }
        TextSegmentPosition::Before(0)
    }
}
pub fn PreviousWordPosition<C>(
    context: &C,
    start: &Position<C::Node>,
) -> super::position::PositionWithAffinity<C::Node>
where
    C: TextIteratorContext
        + InlineContentsContext
        + WordBreakContext
        + super::editing_boundaries::EditableBoundaryContext,
{
    if start.IsNull() {
        return Default::default();
    }
    let previous = FindBoundary(
        context,
        start,
        &mut PreviousWordFinder {
            context,
            first: true,
        },
        true,
    );
    super::editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
        context,
        super::position::PositionWithAffinity {
            position: previous,
            affinity: super::position::TextAffinity::Downstream,
        },
        start,
    )
}

// position_units_word.cc:102-225,349-363.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformWordBehavior {
    SkipSpaces,
    DontSkipSpaces,
}
struct NextWordFinder<'a, C> {
    context: &'a C,
    first: bool,
    behavior: PlatformWordBehavior,
}
fn line_break(unit: u16) -> bool {
    matches!(unit, 10 | 13)
}
impl<C: WordBreakContext> NextWordFinder<'_, C> {
    fn SkipWhitespaceIfNeeded(&self, text: &[u16], offset: usize) -> TextSegmentPosition {
        if self.behavior == PlatformWordBehavior::SkipSpaces {
            for runner in offset..text.len() {
                let unit = text[runner];
                if !(super::editing_utilities::IsWhitespace(unit)
                    || foundation::unicode::Direction(unit) == 9)
                    || line_break(unit)
                {
                    return TextSegmentPosition::Before(runner);
                }
            }
        }
        TextSegmentPosition::Before(offset)
    }
}
impl<C: WordBreakContext> TextSegmentFinder for NextWordFinder<'_, C> {
    fn Find(&mut self, text: &[u16], offset: usize) -> TextSegmentPosition {
        assert!(offset <= text.len());
        if !self.first && offset < text.len() && IsWordBreak(text[offset]) {
            return self.SkipWhitespaceIfNeeded(text, offset);
        }
        self.first = false;
        if offset == text.len() || text.is_empty() {
            return TextSegmentPosition::None;
        }
        let mut punctuation = None;
        let mut space_before_punctuation = false;
        let mut boundary = self.context.FollowingWordBoundary(text, offset);
        while let Some(runner) = boundary {
            assert!(runner <= text.len());
            let pending = self.behavior == PlatformWordBehavior::SkipSpaces
                && punctuation.is_some()
                && space_before_punctuation;
            // WTF String's code-unit accessor at length is its zero terminator.
            if line_break(text.get(runner).copied().unwrap_or(0)) {
                if foundation::RuntimeEnabledFeatures::WordSkipSpacesPunctuationFixEnabled()
                    && pending
                {
                    return self.SkipWhitespaceIfNeeded(text, punctuation.unwrap());
                }
                return self.SkipWhitespaceIfNeeded(text, runner);
            }
            if runner < text.len() && IsWordBoundary(text[runner]) {
                if foundation::unicode::IsAlphanumeric(text[runner - 1] as i32) {
                    return self.SkipWhitespaceIfNeeded(text, runner);
                }
                if self.behavior == PlatformWordBehavior::SkipSpaces
                    && punctuation.is_none()
                    && runner > 0
                    && super::editing_utilities::IsWhitespace(text[runner - 1])
                {
                    punctuation = Some(runner);
                    space_before_punctuation = true;
                }
                boundary = self.context.FollowingWordBoundary(text, runner);
                continue;
            }
            if pending {
                return self.SkipWhitespaceIfNeeded(text, punctuation.unwrap());
            }
            if runner < text.len()
                && (IsWordBreak(text[runner - 1])
                    || (self.behavior == PlatformWordBehavior::SkipSpaces
                        && super::editing_utilities::IsWhitespace(text[runner - 1])
                        && IsWordBreak(text[runner])))
            {
                return self.SkipWhitespaceIfNeeded(text, runner);
            }
            boundary = self.context.FollowingWordBoundary(text, runner);
        }
        if text.last() != Some(&10) {
            TextSegmentPosition::After(text.len() - 1)
        } else {
            TextSegmentPosition::None
        }
    }
}
pub fn NextWordPosition<C>(
    context: &C,
    start: &Position<C::Node>,
    behavior: PlatformWordBehavior,
) -> super::position::PositionWithAffinity<C::Node>
where
    C: TextIteratorContext
        + InlineContentsContext
        + WordBreakContext
        + super::editing_boundaries::EditableBoundaryContext,
{
    if start.IsNull() {
        return Default::default();
    }
    let next = FindBoundary(
        context,
        start,
        &mut NextWordFinder {
            context,
            first: true,
            behavior,
        },
        false,
    );
    let adjusted = super::editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
        context,
        super::position::PositionWithAffinity {
            position: next,
            affinity: super::position::TextAffinity::Downstream,
        },
        start,
    );
    debug_assert!(adjusted.affinity == super::position::TextAffinity::Downstream);
    adjusted
}
