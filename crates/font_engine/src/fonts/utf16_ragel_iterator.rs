#![allow(non_snake_case)]

use crate::text::native::emoji_segmentation_category::{
    EmojiSegmentationCategory, GetEmojiSegmentationCategory,
};
use std::ops::{Add, AddAssign, Sub, SubAssign};

// A C++ span and cursor become a borrowed slice and a code-unit offset.
// Copy retains the source's cached category, including when end() changes
// only the cursor for comparison with a scanner iterator.
// cpp: font_engine/fonts/utf16_ragel_iterator.h:20-45,157-160
#[derive(Clone, Copy, Debug)]
pub struct UTF16RagelIterator<'a> {
    buffer_: &'a [u16],
    cursor_: u32,
    cached_category_: EmojiSegmentationCategory,
}

impl Default for UTF16RagelIterator<'_> {
    fn default() -> Self {
        Self {
            buffer_: &[],
            cursor_: 0,
            cached_category_: EmojiSegmentationCategory::kInvalidCacheEntry,
        }
    }
}

impl<'a> UTF16RagelIterator<'a> {
    // cpp: font_engine/fonts/utf16_ragel_iterator.h:38-60
    pub fn new(buffer: &'a [u16], cursor: u32) -> Self {
        assert!(buffer.len() <= u32::MAX as usize);
        Self {
            buffer_: buffer,
            cursor_: cursor,
            cached_category_: EmojiSegmentationCategory::kInvalidCacheEntry,
        }
    }

    pub fn end(&self) -> Self {
        let mut ret = *self;
        ret.cursor_ = self.buffer_.len() as u32;
        ret
    }

    pub fn size(&self) -> usize {
        self.buffer_.len()
    }

    pub fn SetCursor(&mut self, new_cursor: u32) -> &mut Self {
        assert!((new_cursor as usize) < self.buffer_.len());
        self.cursor_ = new_cursor;
        self.InvalidateCache();
        self
    }

    pub fn Cursor(&self) -> u32 {
        self.cursor_
    }

    // U16_GET preserves an unpaired surrogate as its code-unit value and
    // joins a valid pair even when `index` points to its trail unit.
    // cpp: icu_bidi/unicode/utf16.h:201-214
    fn CodePointAt(&self, index: usize) -> i32 {
        let current = u32::from(self.buffer_[index]);
        if (0xD800..=0xDBFF).contains(&current) && index + 1 < self.buffer_.len() {
            let trail = u32::from(self.buffer_[index + 1]);
            if (0xDC00..=0xDFFF).contains(&trail) {
                return (0x10000 + ((current - 0xD800) << 10) + (trail - 0xDC00)) as i32;
            }
        } else if (0xDC00..=0xDFFF).contains(&current) && index > 0 {
            let lead = u32::from(self.buffer_[index - 1]);
            if (0xD800..=0xDBFF).contains(&lead) {
                return (0x10000 + ((lead - 0xD800) << 10) + (current - 0xDC00)) as i32;
            }
        }
        current as i32
    }

    // cpp: icu_bidi/unicode/utf16.h:433-438,643-647
    fn ForwardOne(&mut self) {
        assert!((self.cursor_ as usize) < self.buffer_.len());
        let first = self.buffer_[self.cursor_ as usize];
        self.cursor_ += 1;
        if (0xD800..=0xDBFF).contains(&first)
            && (self.cursor_ as usize) < self.buffer_.len()
            && (0xDC00..=0xDFFF).contains(&self.buffer_[self.cursor_ as usize])
        {
            self.cursor_ += 1;
        }
    }

    fn BackOne(&mut self) {
        assert!(self.cursor_ > 0);
        self.cursor_ -= 1;
        let last = self.buffer_[self.cursor_ as usize];
        if (0xDC00..=0xDFFF).contains(&last)
            && self.cursor_ > 0
            && (0xD800..=0xDBFF).contains(&self.buffer_[(self.cursor_ - 1) as usize])
        {
            self.cursor_ -= 1;
        }
    }

    // cpp: font_engine/fonts/utf16_ragel_iterator.h:62-105
    pub fn AddSteps(&mut self, steps: i32) -> &mut Self {
        if steps > 0 {
            for _ in 0..steps {
                if self.cursor_ as usize >= self.buffer_.len() {
                    break;
                }
                self.ForwardOne();
            }
        } else if steps < 0 {
            for _ in 0..-(steps as i64) {
                if self.cursor_ == 0 {
                    break;
                }
                self.BackOne();
            }
        }
        self.InvalidateCache();
        self
    }

    pub fn PreIncrement(&mut self) -> &mut Self {
        self.ForwardOne();
        self.InvalidateCache();
        self
    }

    pub fn PreDecrement(&mut self) -> &mut Self {
        self.BackOne();
        self.InvalidateCache();
        self
    }

    pub fn PostIncrement(&mut self) -> Self {
        let ret = *self;
        self.PreIncrement();
        ret
    }

    pub fn PostDecrement(&mut self) -> Self {
        let ret = *self;
        self.PreDecrement();
        ret
    }

    // Ragel assigns literal zero to `ts` and `te`. The source returns a
    // modified copy and does not invalidate its copied cache.
    // cpp: font_engine/fonts/utf16_ragel_iterator.h:107-115
    pub fn AssignZero(&self, value: i32) -> Self {
        assert_eq!(value, 0);
        let mut ret = *self;
        ret.cursor_ = 0;
        ret
    }

    // cpp: font_engine/fonts/utf16_ragel_iterator.h:117-130
    pub fn Category(&mut self) -> EmojiSegmentationCategory {
        assert!(!self.buffer_.is_empty());
        if self.cached_category_ == EmojiSegmentationCategory::kInvalidCacheEntry {
            let codepoint = self.CodePointAt(self.cursor_ as usize);
            self.cached_category_ = GetEmojiSegmentationCategory(codepoint);
        }
        self.cached_category_
    }

    pub fn InvalidateCache(&mut self) {
        self.cached_category_ = EmojiSegmentationCategory::kInvalidCacheEntry;
    }

    // cpp: font_engine/fonts/utf16_ragel_iterator.h:143-154
    pub fn PeekCodepoint(&self) -> i32 {
        let mut output = 0xFFFD;
        let mut next = *self;
        if (next.cursor_ as usize) < next.buffer_.len() {
            next.ForwardOne();
            if (next.cursor_ as usize) < next.buffer_.len() {
                output = next.CodePointAt(next.cursor_ as usize);
            }
        }
        output
    }
}

// C++ iterator arithmetic and equality map to Rust operator traits. Their
// source call sites can retain the same sequencing when the scanner is ported.
// cpp: font_engine/fonts/utf16_ragel_iterator.h:64-89,132-135
impl AddAssign<i32> for UTF16RagelIterator<'_> {
    fn add_assign(&mut self, rhs: i32) {
        self.AddSteps(rhs);
    }
}

impl SubAssign<i32> for UTF16RagelIterator<'_> {
    fn sub_assign(&mut self, rhs: i32) {
        self.AddSteps(rhs.wrapping_neg());
    }
}

impl Add<i32> for UTF16RagelIterator<'_> {
    type Output = Self;
    fn add(mut self, rhs: i32) -> Self::Output {
        self += rhs;
        self
    }
}

impl Sub<i32> for UTF16RagelIterator<'_> {
    type Output = Self;
    fn sub(mut self, rhs: i32) -> Self::Output {
        self -= rhs;
        self
    }
}

impl Sub<Self> for UTF16RagelIterator<'_> {
    type Output = i32;
    fn sub(self, other: Self) -> Self::Output {
        debug_assert!(std::ptr::eq(self.buffer_.as_ptr(), other.buffer_.as_ptr()));
        debug_assert_eq!(self.buffer_.len(), other.buffer_.len());
        self.cursor_.wrapping_sub(other.cursor_) as i32
    }
}

impl PartialEq for UTF16RagelIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.buffer_.as_ptr(), other.buffer_.as_ptr())
            && self.buffer_.len() == other.buffer_.len()
            && self.cursor_ == other.cursor_
    }
}
