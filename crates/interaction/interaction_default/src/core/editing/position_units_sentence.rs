// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: position_units_sentence.cc:85-175,205-221,292-309;
// visible_units_sentence.cc:46-60. ICU ownership is supplied by the embedding.
use super::{
    editing_boundaries::{self, EditableBoundaryContext},
    iterators::text_iterator::TextIteratorContext,
    position::{Position, PositionWithAffinity, TextAffinity},
    text_offset_mapping::InlineContentsContext,
    text_segments::{FindBoundary, TextSegmentFinder, TextSegmentPosition},
    visible_position::{VisiblePosition, VisiblePositionContext},
};
pub trait SentenceBreakContext {
    fn FollowingSentenceBoundary(&self, text: &[u16], offset: usize) -> Option<usize>;
    fn PrecedingSentenceBoundary(&self, text: &[u16], offset: usize) -> Option<usize>;
}
#[derive(Clone, Copy)]
pub enum SentenceTrailingSpaceBehavior {
    IncludeSpace,
    OmitSpace,
}
struct SentenceFinder<'a, C> {
    context: &'a C,
    backward: bool,
    space: SentenceTrailingSpaceBehavior,
}
impl<C: SentenceBreakContext> TextSegmentFinder for SentenceFinder<'_, C> {
    fn Find(&mut self, text: &[u16], mut offset: usize) -> TextSegmentPosition {
        assert!(offset <= text.len());
        if self.backward {
            while offset > 0 && text[offset - 1] == b' ' as u16 {
                offset -= 1;
            }
            self.context
                .PrecedingSentenceBoundary(text, offset)
                .map_or_else(
                    || {
                        if text.is_empty() {
                            TextSegmentPosition::None
                        } else {
                            TextSegmentPosition::Before(0)
                        }
                    },
                    TextSegmentPosition::Before,
                )
        } else {
            while offset < text.len() && text[offset] == b' ' as u16 {
                offset += 1;
            }
            let Some(end) = self.context.FollowingSentenceBoundary(text, offset) else {
                return if text.is_empty() {
                    TextSegmentPosition::None
                } else {
                    TextSegmentPosition::After(text.len())
                };
            };
            if matches!(self.space, SentenceTrailingSpaceBehavior::OmitSpace)
                && end != 0
                && text[end - 1] == b' ' as u16
            {
                return TextSegmentPosition::After(end.saturating_sub(2));
            }
            if end == 0 {
                TextSegmentPosition::Before(0)
            } else {
                TextSegmentPosition::After(end - 1)
            }
        }
    }
}
pub fn StartOfSentencePosition<C>(context: &C, position: &Position<C::Node>) -> Position<C::Node>
where
    C: TextIteratorContext + InlineContentsContext + SentenceBreakContext + EditableBoundaryContext,
{
    if position.IsNull() {
        return position.clone();
    }
    let result = FindBoundary(
        context,
        position,
        &mut SentenceFinder {
            context,
            backward: true,
            space: SentenceTrailingSpaceBehavior::IncludeSpace,
        },
        true,
    );
    editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
        context,
        PositionWithAffinity {
            position: result,
            affinity: TextAffinity::Downstream,
        },
        position,
    )
    .position
}
pub fn EndOfSentence<C>(
    context: &C,
    position: &Position<C::Node>,
    space: SentenceTrailingSpaceBehavior,
) -> PositionWithAffinity<C::Node>
where
    C: TextIteratorContext + InlineContentsContext + SentenceBreakContext + EditableBoundaryContext,
{
    if position.IsNull() {
        return PositionWithAffinity::default();
    }
    let result = FindBoundary(
        context,
        position,
        &mut SentenceFinder {
            context,
            backward: false,
            space,
        },
        false,
    );
    editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
        context,
        PositionWithAffinity {
            position: result,
            affinity: TextAffinity::Downstream,
        },
        position,
    )
}
pub fn EndOfVisibleSentence<C>(
    context: &C,
    position: &VisiblePosition<C::Node>,
    space: SentenceTrailingSpaceBehavior,
) -> VisiblePosition<C::Node>
where
    C: TextIteratorContext
        + InlineContentsContext
        + SentenceBreakContext
        + EditableBoundaryContext
        + VisiblePositionContext,
{
    VisiblePosition::Create(
        context,
        EndOfSentence(context, position.DeepEquivalent(), space),
    )
}

// position_units_sentence.cc:44-77,162-205,262-285. Sentence movement differs
// from sentence boundaries: a failed ICU search continues into the next inline
// contents block rather than stopping at that block's edge.
struct SentenceMovementFinder<'a, C> {
    context: &'a C,
    backward: bool,
    stop_at_next_block: bool,
}
impl<C: SentenceBreakContext> TextSegmentFinder for SentenceMovementFinder<'_, C> {
    fn Find(&mut self, text: &[u16], mut offset: usize) -> TextSegmentPosition {
        assert!(offset <= text.len());
        if self.backward {
            while offset > 0 && text[offset - 1] == b' ' as u16 {
                offset -= 1;
            }
            return self
                .context
                .PrecedingSentenceBoundary(text, offset)
                .map_or(TextSegmentPosition::None, TextSegmentPosition::Before);
        }
        if self.stop_at_next_block {
            debug_assert_eq!(offset, 0);
            return TextSegmentPosition::Before(0);
        }
        if offset == text.len() || (offset + 1 == text.len() && text[offset] == b'\n' as u16) {
            self.stop_at_next_block = true;
            return TextSegmentPosition::None;
        }
        self.context.FollowingSentenceBoundary(text, offset).map_or(
            TextSegmentPosition::None,
            |result| {
                if result == 0 {
                    TextSegmentPosition::Before(0)
                } else {
                    TextSegmentPosition::After(result - 1)
                }
            },
        )
    }
}
fn SentencePosition<C>(
    context: &C,
    position: &Position<C::Node>,
    backward: bool,
) -> Position<C::Node>
where
    C: TextIteratorContext + InlineContentsContext + SentenceBreakContext + EditableBoundaryContext,
{
    if position.IsNull() {
        return position.clone();
    }
    let candidate = PositionWithAffinity {
        position: FindBoundary(
            context,
            position,
            &mut SentenceMovementFinder {
                context,
                backward,
                stop_at_next_block: false,
            },
            backward,
        ),
        affinity: TextAffinity::Downstream,
    };
    if backward {
        editing_boundaries::AdjustBackwardPositionToAvoidCrossingEditingBoundaries(
            context, candidate, position,
        )
    } else {
        editing_boundaries::AdjustForwardPositionToAvoidCrossingEditingBoundaries(
            context, candidate, position,
        )
    }
    .position
}
pub fn PreviousSentencePosition<C>(context: &C, position: &Position<C::Node>) -> Position<C::Node>
where
    C: TextIteratorContext + InlineContentsContext + SentenceBreakContext + EditableBoundaryContext,
{
    SentencePosition(context, position, true)
}
pub fn NextSentencePosition<C>(context: &C, position: &Position<C::Node>) -> Position<C::Node>
where
    C: TextIteratorContext + InlineContentsContext + SentenceBreakContext + EditableBoundaryContext,
{
    SentencePosition(context, position, false)
}
