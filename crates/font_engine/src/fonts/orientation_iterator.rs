#![allow(non_snake_case)]

use super::font_orientation::FontOrientation;
use super::orientation_iterator_types::RenderOrientation;
use super::utf16_text_iterator::UTF16TextIterator;
use crate::text::native::character::Character;

// C++ stack-only iterator owns a UTF-16 span and cannot be copied. A Rust
// borrowed slice preserves that lifetime and this type intentionally has no
// Clone implementation.
// cpp: font_engine/fonts/orientation_iterator.h:16-41
pub struct OrientationIterator<'a> {
    utf16_iterator_: UTF16TextIterator<'a>,
    at_end_: bool,
}

impl<'a> OrientationIterator<'a> {
    // cpp: font_engine/fonts/orientation_iterator.cc:10-15
    pub fn new(buffer: &'a [u16], run_orientation: FontOrientation) -> Self {
        debug_assert_eq!(run_orientation, FontOrientation::kVerticalMixed);
        Self {
            utf16_iterator_: UTF16TextIterator::new(buffer),
            at_end_: buffer.is_empty(),
        }
    }

    // cpp: font_engine/fonts/orientation_iterator.cc:17-43
    pub fn Consume(
        &mut self,
        orientation_limit: &mut u32,
        render_orientation: &mut RenderOrientation,
    ) -> bool {
        if self.at_end_ {
            return false;
        }

        let mut current = RenderOrientation::kOrientationInvalid;
        let mut character = 0;
        while self.utf16_iterator_.Consume(&mut character) {
            if current == RenderOrientation::kOrientationInvalid
                || !Character::IsGraphemeExtended(character)
            {
                let previous = current;
                current = if Character::IsUprightInMixedVertical(character) {
                    RenderOrientation::kOrientationKeep
                } else {
                    RenderOrientation::kOrientationRotateSideways
                };
                if previous != current && previous != RenderOrientation::kOrientationInvalid {
                    *orientation_limit = self.utf16_iterator_.Offset();
                    *render_orientation = previous;
                    return true;
                }
            }
            self.utf16_iterator_.Advance();
        }
        *orientation_limit = self.utf16_iterator_.Size();
        *render_orientation = current;
        self.at_end_ = true;
        true
    }
}
