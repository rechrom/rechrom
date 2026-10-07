#![allow(non_snake_case, non_upper_case_globals)]

use foundation::blink_geometry::geometry::TextRunLayoutUnit;
use foundation::gfx::Vector2dF;
use foundation::{Traceable, Visitor};

// cpp: font_engine/fonts/shaping/glyph_data.h:16-20
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum SafeToBreak {
    #[default]
    kSafe = 0,
    kUnsafe = 1,
}

pub fn IsSafeToBreak(value: SafeToBreak) -> bool {
    value == SafeToBreak::kSafe
}

// C++ packs three unsigned bitfields into one 32-bit allocation unit. Keep
// their bit positions and truncation while exposing named accessors to Rust.
// cpp: font_engine/fonts/shaping/glyph_data.h:22-75
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct HarfBuzzRunGlyphData {
    packed_glyph_and_index: u32,
    pub advance: TextRunLayoutUnit,
}

impl HarfBuzzRunGlyphData {
    pub const kCharacterIndexBits: u32 = 15;
    pub const kMaxCharacters: u32 = 1 << Self::kCharacterIndexBits;
    pub const kMaxCharacterIndex: u32 = Self::kMaxCharacters - 1;
    pub const kMaxGlyphs: u32 = Self::kMaxCharacters;

    pub fn new(
        glyph: u32,
        character_index: u32,
        safe_to_break_before: SafeToBreak,
        advance: TextRunLayoutUnit,
    ) -> Self {
        Self {
            packed_glyph_and_index: (glyph & 0xffff)
                | ((character_index & Self::kMaxCharacterIndex) << 16)
                | ((safe_to_break_before as u32) << 31),
            advance,
        }
    }

    pub fn glyph(&self) -> u32 {
        self.packed_glyph_and_index & 0xffff
    }

    pub fn character_index(&self) -> u32 {
        (self.packed_glyph_and_index >> 16) & Self::kMaxCharacterIndex
    }

    // The C++ bitfield truncates assignments to 15 bits. Future callers use
    // this setter for the source's direct character_index mutations.
    pub fn SetCharacterIndex(&mut self, index: u32) {
        self.packed_glyph_and_index = (self.packed_glyph_and_index
            & !(Self::kMaxCharacterIndex << 16))
            | ((index & Self::kMaxCharacterIndex) << 16);
    }

    pub fn SafeToBreakBefore(&self) -> SafeToBreak {
        if self.packed_glyph_and_index & (1 << 31) == 0 {
            SafeToBreak::kSafe
        } else {
            SafeToBreak::kUnsafe
        }
    }

    pub fn IsSafeToBreakBefore(&self) -> bool {
        IsSafeToBreak(self.SafeToBreakBefore())
    }

    pub fn SetSafeToBreakBefore(&mut self, value: SafeToBreak) {
        self.packed_glyph_and_index =
            (self.packed_glyph_and_index & !(1 << 31)) | ((value as u32) << 31);
    }

    pub fn SetAdvance(&mut self, value: TextRunLayoutUnit) {
        self.advance = value;
    }

    pub fn AddAdvance(&mut self, value: TextRunLayoutUnit) {
        self.advance += value;
    }

    pub fn SetAdvanceFloat(&mut self, value: f32) {
        self.advance = TextRunLayoutUnit::FromFloatRound(value);
    }

    pub fn AddAdvanceFloat(&mut self, value: f32) {
        self.advance += TextRunLayoutUnit::FromFloatRound(value);
    }
}

const _: [(); 8] = [(); std::mem::size_of::<HarfBuzzRunGlyphData>()];

// cpp: font_engine/fonts/shaping/glyph_data.h:77-82
pub type GlyphOffset = Vector2dF;

impl Traceable for HarfBuzzRunGlyphData {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}
