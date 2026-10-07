#![allow(non_snake_case)]

use super::glyph_data::GlyphOffset;
use super::glyph_data_range::GlyphDataRange;

// C++ specializes the template for true and false. A const generic preserves
// that call-site distinction; the false branch never reads an offsets array.
// cpp: font_engine/fonts/shaping/glyph_offset_iterator.h:15-64
pub struct GlyphOffsetIterator<'a, const HAS_NON_ZERO_GLYPH_OFFSETS: bool> {
    offsets_: &'a [GlyphOffset],
    index_: isize,
}

impl<'a, const HAS_NON_ZERO_GLYPH_OFFSETS: bool>
    GlyphOffsetIterator<'a, HAS_NON_ZERO_GLYPH_OFFSETS>
{
    pub fn new(offsets: &'a [GlyphOffset]) -> Self {
        debug_assert_eq!(!offsets.is_empty(), HAS_NON_ZERO_GLYPH_OFFSETS);
        Self {
            offsets_: offsets,
            index_: 0,
        }
    }

    pub fn from_range(range: &'a GlyphDataRange) -> Self {
        if !HAS_NON_ZERO_GLYPH_OFFSETS {
            debug_assert!(!range.HasOffsets());
        }
        Self::new(range.Offsets())
    }

    pub fn get(&self) -> GlyphOffset {
        if HAS_NON_ZERO_GLYPH_OFFSETS {
            self.offsets_[self.index_ as usize]
        } else {
            GlyphOffset::default()
        }
    }

    pub fn advance(&mut self) {
        if HAS_NON_ZERO_GLYPH_OFFSETS {
            self.index_ += 1;
        }
    }

    pub fn advance_by(&mut self, amount: isize) {
        if HAS_NON_ZERO_GLYPH_OFFSETS {
            self.index_ += amount;
        }
    }

    pub fn at(&self, index: usize) -> GlyphOffset {
        if HAS_NON_ZERO_GLYPH_OFFSETS {
            self.offsets_[(self.index_ + index as isize) as usize]
        } else {
            GlyphOffset::default()
        }
    }
}
