//! ICU unicode/ubrk.h API used by Blink's native character, word and sentence
//! iterators, implemented on ICU4X segmenters (UAX #29 with ICU4X's
//! dictionary/LSTM handling of complex scripts).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use std::ffi::c_char;

use icu_segmenter::options::{SentenceBreakInvariantOptions, WordBreakInvariantOptions};
use icu_segmenter::options::{LineBreakOptions, WordType};
use icu_segmenter::{GraphemeClusterSegmenter, LineSegmenter, SentenceSegmenter, WordSegmenter};

use crate::ustring::{self, U_ILLEGAL_ARGUMENT_ERROR};

pub const UBRK_CHARACTER: i32 = 0;
pub const UBRK_WORD: i32 = 1;
pub const UBRK_LINE: i32 = 2;
pub const UBRK_SENTENCE: i32 = 3;
pub const UBRK_DONE: i32 = -1;
pub const UBRK_WORD_NONE: i32 = 0;
pub const UBRK_WORD_NUMBER: i32 = 100;
pub const UBRK_WORD_LETTER: i32 = 200;

/// Boundaries of the current text, with the rule status of the segment that
/// ends at each one, and the iterator's current boundary.
pub struct UBreakIterator {
    kind: i32,
    boundaries: Vec<i32>,
    statuses: Vec<i32>,
    current: usize,
}

impl UBreakIterator {
    fn set_text(&mut self, text: &[u16]) {
        let mut boundaries = Vec::new();
        let mut statuses = Vec::new();
        match self.kind {
            UBRK_CHARACTER => {
                boundaries.extend(GraphemeClusterSegmenter::new().segment_utf16(text));
            }
            UBRK_WORD => {
                let mut iterator = WordSegmenter::new_auto(WordBreakInvariantOptions::default())
                    .segment_utf16(text);
                while let Some(boundary) = iterator.next() {
                    boundaries.push(boundary);
                    statuses.push(match iterator.word_type() {
                        WordType::Number => UBRK_WORD_NUMBER,
                        WordType::Letter => UBRK_WORD_LETTER,
                        _ => UBRK_WORD_NONE,
                    });
                }
            }
            UBRK_LINE => {
                boundaries.extend(LineSegmenter::new_auto(LineBreakOptions::default()).segment_utf16(text));
            }
            UBRK_SENTENCE => {
                boundaries.extend(
                    SentenceSegmenter::new(SentenceBreakInvariantOptions::default()).segment_utf16(text),
                );
            }
            _ => unreachable!("validated by ubrk_open"),
        }
        if boundaries.first() != Some(&0) {
            boundaries.insert(0, 0);
            if !statuses.is_empty() {
                statuses.insert(0, UBRK_WORD_NONE);
            }
        }
        self.boundaries = boundaries.into_iter().map(|offset| offset as i32).collect();
        statuses.resize(self.boundaries.len(), UBRK_WORD_NONE);
        self.statuses = statuses;
        self.current = 0;
    }

    fn at(&mut self, index: usize) -> i32 {
        self.current = index;
        self.boundaries[index]
    }

    fn done(&mut self) -> i32 {
        self.current = self.boundaries.len() - 1;
        UBRK_DONE
    }
}

pub unsafe fn ubrk_open(
    kind: i32,
    _locale: *const c_char,
    text: *const u16,
    length: i32,
    error: *mut i32,
) -> *mut UBreakIterator {
    if ustring::failed(error) {
        return std::ptr::null_mut();
    }
    let Some(text) = ustring::input(text, length).filter(|_| (0..=3).contains(&kind)) else {
        ustring::set_error(error, U_ILLEGAL_ARGUMENT_ERROR);
        return std::ptr::null_mut();
    };
    let mut iterator = UBreakIterator {
        kind,
        boundaries: vec![0],
        statuses: vec![UBRK_WORD_NONE],
        current: 0,
    };
    iterator.set_text(text);
    Box::into_raw(Box::new(iterator))
}

pub unsafe fn ubrk_close(iterator: *mut UBreakIterator) {
    if !iterator.is_null() {
        drop(Box::from_raw(iterator));
    }
}

pub unsafe fn ubrk_setText(iterator: *mut UBreakIterator, text: *const u16, length: i32, error: *mut i32) {
    if ustring::failed(error) {
        return;
    }
    let (Some(iterator), Some(text)) = (iterator.as_mut(), ustring::input(text, length)) else {
        ustring::set_error(error, U_ILLEGAL_ARGUMENT_ERROR);
        return;
    };
    iterator.set_text(text);
}

pub unsafe fn ubrk_first(iterator: *mut UBreakIterator) -> i32 {
    (*iterator).at(0)
}

pub unsafe fn ubrk_next(iterator: *mut UBreakIterator) -> i32 {
    let iterator = &mut *iterator;
    if iterator.current + 1 < iterator.boundaries.len() {
        iterator.at(iterator.current + 1)
    } else {
        iterator.done()
    }
}

pub unsafe fn ubrk_following(iterator: *mut UBreakIterator, offset: i32) -> i32 {
    let iterator = &mut *iterator;
    let index = iterator.boundaries.partition_point(|boundary| *boundary <= offset);
    if index < iterator.boundaries.len() {
        iterator.at(index)
    } else {
        iterator.done()
    }
}

pub unsafe fn ubrk_preceding(iterator: *mut UBreakIterator, offset: i32) -> i32 {
    let iterator = &mut *iterator;
    let index = iterator.boundaries.partition_point(|boundary| *boundary < offset);
    if index > 0 {
        iterator.at(index - 1)
    } else {
        iterator.current = 0;
        UBRK_DONE
    }
}

/// Moves to `offset` when it is a boundary, otherwise to the following one.
pub unsafe fn ubrk_isBoundary(iterator: *mut UBreakIterator, offset: i32) -> i8 {
    let iterator_ref = &mut *iterator;
    match iterator_ref.boundaries.binary_search(&offset) {
        Ok(index) => {
            iterator_ref.at(index);
            1
        }
        Err(_) => {
            ubrk_following(iterator, offset);
            0
        }
    }
}

/// The status of the rule that produced the current boundary, i.e. of the
/// segment that ends there.
pub unsafe fn ubrk_getRuleStatus(iterator: *mut UBreakIterator) -> i32 {
    let iterator = &*iterator;
    iterator.statuses[iterator.current]
}
