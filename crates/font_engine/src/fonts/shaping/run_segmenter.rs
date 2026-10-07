#![allow(non_snake_case)]

use super::run_segmenter_types::RunSegmenterRange;
use crate::fonts::font_orientation::FontOrientation;
use crate::fonts::orientation_iterator::OrientationIterator;
// ScriptRunIterator is mapped in its owning same-package module; its ICU ABI
// connection remains pending.
use crate::fonts::script_run_iterator::ScriptRunIterator;
use crate::fonts::symbols_iterator::SymbolsIterator;

// A C++ stack-only class retaining a span maps to a borrowed Rust struct.
// C++ deleted copy operations are preserved by omitting Clone and Copy.
// cpp: font_engine/fonts/shaping/run_segmenter.h:26-66
pub struct RunSegmenter<'a> {
    buffer_size_: u32,
    candidate_range_: RunSegmenterRange,
    script_run_iterator_: ScriptRunIterator<'a>,
    orientation_iterator_: Option<OrientationIterator<'a>>,
    symbols_iterator_: SymbolsIterator<'a>,
    last_split_: u32,
    script_run_iterator_position_: u32,
    orientation_iterator_position_: u32,
    symbols_iterator_position_: u32,
    at_end_: bool,
}

impl<'a> RunSegmenter<'a> {
    // cpp: font_engine/fonts/shaping/run_segmenter.cc:11-21
    pub fn new(buffer: &'a [u16], run_orientation: FontOrientation) -> Self {
        let buffer_size = u32::try_from(buffer.len()).expect("UTF-16 run exceeds 32 bits");
        let mut result = Self {
            buffer_size_: buffer_size,
            candidate_range_: RunSegmenterRange::default(),
            script_run_iterator_: ScriptRunIterator::new(buffer),
            orientation_iterator_: None,
            symbols_iterator_: SymbolsIterator::new(buffer),
            last_split_: 0,
            script_run_iterator_position_: 0,
            orientation_iterator_position_: 0,
            symbols_iterator_position_: 0,
            at_end_: buffer.is_empty(),
        };
        if run_orientation == FontOrientation::kVerticalMixed {
            result.orientation_iterator_ = Some(OrientationIterator::new(buffer, run_orientation));
        }
        result
    }

    // C++'s member template reads only last_split_ and buffer_size_. Passing
    // those values lets Rust borrow the iterator and its result field at once.
    // cpp: font_engine/fonts/shaping/run_segmenter.cc:23-35
    fn ConsumeIteratorPastLastSplit<Category, Consume>(
        mut consume: Consume,
        iterator_position: &mut u32,
        segmentation_category: &mut Category,
        last_split: u32,
        buffer_size: u32,
    ) where
        Consume: FnMut(&mut u32, &mut Category) -> bool,
    {
        if *iterator_position <= last_split && *iterator_position < buffer_size {
            while consume(iterator_position, segmentation_category) {
                if *iterator_position > last_split {
                    return;
                }
            }
        }
    }

    // cpp: font_engine/fonts/shaping/run_segmenter.cc:37-62
    pub fn Consume(&mut self, next_range: &mut RunSegmenterRange) -> bool {
        if self.at_end_ {
            return false;
        }

        Self::ConsumeIteratorPastLastSplit(
            |position, category| self.script_run_iterator_.Consume(position, category),
            &mut self.script_run_iterator_position_,
            &mut self.candidate_range_.script,
            self.last_split_,
            self.buffer_size_,
        );
        Self::ConsumeIteratorPastLastSplit(
            |position, category| self.symbols_iterator_.Consume(position, category),
            &mut self.symbols_iterator_position_,
            &mut self.candidate_range_.font_fallback_priority,
            self.last_split_,
            self.buffer_size_,
        );
        if let Some(iterator) = &mut self.orientation_iterator_ {
            Self::ConsumeIteratorPastLastSplit(
                |position, category| iterator.Consume(position, category),
                &mut self.orientation_iterator_position_,
                &mut self.candidate_range_.render_orientation,
                self.last_split_,
                self.buffer_size_,
            );
            let positions = [
                self.script_run_iterator_position_,
                self.symbols_iterator_position_,
                self.orientation_iterator_position_,
            ];
            self.last_split_ = *positions.iter().min().expect("three split positions");
        } else {
            self.last_split_ = self
                .script_run_iterator_position_
                .min(self.symbols_iterator_position_);
        }

        self.candidate_range_.start = self.candidate_range_.end;
        self.candidate_range_.end = self.last_split_;
        *next_range = self.candidate_range_;
        self.at_end_ = self.last_split_ == self.buffer_size_;
        true
    }
}
