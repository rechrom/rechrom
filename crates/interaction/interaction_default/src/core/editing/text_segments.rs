// Copyright 2018 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/text_segments.cc:12-94.
use super::{
    iterators::text_iterator::TextIteratorContext,
    position::Position,
    text_offset_mapping::{InlineContentsContext, InlineContentsIterator, TextOffsetMapping},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSegmentPosition {
    None,
    Before(usize),
    After(usize),
}
pub trait TextSegmentFinder {
    fn Find(&mut self, text: &[u16], offset: usize) -> TextSegmentPosition;
}
pub fn FindBoundary<C: TextIteratorContext + InlineContentsContext>(
    context: &C,
    position: &Position<C::Node>,
    finder: &mut impl TextSegmentFinder,
    backward: bool,
) -> Position<C::Node> {
    assert!(!position.IsNull());
    let mut last = position.clone();
    let mut first = true;
    let range = if backward {
        InlineContentsIterator::BackwardRangeOf(context, position)
    } else {
        InlineContentsIterator::ForwardRangeOf(context, position)
    };
    for contents in range {
        let Some(mapping) = TextOffsetMapping::new(context, &contents) else {
            continue;
        };
        let offset = if first {
            mapping.ComputeTextOffset(position)
        } else if backward {
            mapping.GetText().len()
        } else {
            0
        };
        first = false;
        match finder.Find(mapping.GetText(), offset) {
            TextSegmentPosition::Before(offset) => return mapping.GetPositionBefore(offset),
            TextSegmentPosition::After(offset) => return mapping.GetPositionAfter(offset),
            TextSegmentPosition::None => {}
        }
        last = if backward {
            mapping.GetRange().0.clone()
        } else {
            mapping.GetRange().1.clone()
        };
    }
    last
}
