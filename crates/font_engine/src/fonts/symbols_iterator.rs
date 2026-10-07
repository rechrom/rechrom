#![allow(non_snake_case)]

use super::emoji_presentation_scanner::scan_emoji_presentation;
use super::font_fallback_priority::FontFallbackPriority;
use super::utf16_ragel_iterator::UTF16RagelIterator;
use crate::text::native::character::Character;

// The C++ span remains borrowed; the source-deleted copy constructor is not
// exposed as Clone. Offsets continue to count UTF-16 code units.
// cpp: font_engine/fonts/symbols_iterator.h:40-57
pub struct SymbolsIterator<'a> {
    buffer_iterator_: UTF16RagelIterator<'a>,
    cursor_: u32,
    next_token_end_: u32,
    next_token_emoji_: bool,
    next_token_has_vs_: bool,
}

impl<'a> SymbolsIterator<'a> {
    // cpp: font_engine/fonts/symbols_iterator.cc:20-29
    pub fn new(buffer: &'a [u16]) -> Self {
        let mut result = Self {
            buffer_iterator_: UTF16RagelIterator::default(),
            cursor_: 0,
            next_token_end_: 0,
            next_token_emoji_: false,
            next_token_has_vs_: false,
        };
        if !buffer.is_empty() {
            result.buffer_iterator_ = UTF16RagelIterator::new(buffer, 0);
            let token_end = scan_emoji_presentation(
                result.buffer_iterator_,
                result.buffer_iterator_.end(),
                &mut result.next_token_emoji_,
                &mut result.next_token_has_vs_,
            );
            result.next_token_end_ = result
                .cursor_
                .wrapping_add((token_end - result.buffer_iterator_) as u32);
        }
        result
    }

    // cpp: font_engine/fonts/symbols_iterator.cc:31-76
    pub fn Consume(
        &mut self,
        symbols_limit: &mut u32,
        font_fallback_priority: &mut FontFallbackPriority,
    ) -> bool {
        if self.cursor_ as usize >= self.buffer_iterator_.size() {
            return false;
        }

        let mut current_token_emoji = false;
        let mut curr_has_vs = false;
        loop {
            self.cursor_ = self.next_token_end_;
            current_token_emoji = self.next_token_emoji_;
            curr_has_vs = self.next_token_has_vs_;

            if self.cursor_ >= self.buffer_iterator_.end().Cursor() {
                break;
            }

            if !current_token_emoji
                && !Character::MaybeEmojiPresentation(self.buffer_iterator_.PeekCodepoint())
            {
                self.buffer_iterator_.PreIncrement();
                self.next_token_end_ = self.buffer_iterator_.Cursor();
                self.next_token_has_vs_ = false;
            } else {
                self.buffer_iterator_.SetCursor(self.cursor_);
                let token_end = scan_emoji_presentation(
                    self.buffer_iterator_,
                    self.buffer_iterator_.end(),
                    &mut self.next_token_emoji_,
                    &mut self.next_token_has_vs_,
                );
                self.next_token_end_ = self
                    .cursor_
                    .wrapping_add((token_end - self.buffer_iterator_) as u32);
            }

            if current_token_emoji != self.next_token_emoji_
                || curr_has_vs != self.next_token_has_vs_
            {
                break;
            }
        }

        if curr_has_vs {
            *font_fallback_priority = if current_token_emoji {
                FontFallbackPriority::kEmojiEmojiWithVS
            } else {
                FontFallbackPriority::kEmojiTextWithVS
            };
        } else {
            *font_fallback_priority = if current_token_emoji {
                FontFallbackPriority::kEmojiEmoji
            } else {
                FontFallbackPriority::kText
            };
        }
        *symbols_limit = self.cursor_;
        true
    }
}
