#![allow(non_snake_case)]

// `base::span<const UChar>` is a borrowed UTF-16 slice. The two source
// pointers are represented by this slice and a code-unit offset, which also
// keeps pointer arithmetic within the borrowed allocation.
// cpp: font_engine/fonts/utf16_text_iterator.h:34-48,80-85
pub struct UTF16TextIterator<'a> {
    characters_: &'a [u16],
    offset_: usize,
    current_glyph_length_: usize,
}

impl<'a> UTF16TextIterator<'a> {
    pub fn new(characters: &'a [u16]) -> Self {
        assert!(characters.len() <= u32::MAX as usize);
        Self {
            characters_: characters,
            offset_: 0,
            current_glyph_length_: 0,
        }
    }

    // cpp: font_engine/fonts/utf16_text_iterator.h:50-61
    pub fn Consume(&mut self, character: &mut i32) -> bool {
        if self.offset_ >= self.characters_.len() {
            return false;
        }
        *character = i32::from(self.characters_[self.offset_]);
        self.current_glyph_length_ = 1;
        if !(0xd800..=0xdfff).contains(character) {
            return true;
        }
        self.ConsumeSurrogatePair(character)
    }

    // cpp: font_engine/fonts/utf16_text_iterator.h:63-73
    pub fn Advance(&mut self) {
        self.offset_ += self.current_glyph_length_;
    }

    pub fn Offset(&self) -> u32 {
        self.offset_ as u32
    }

    pub fn Size(&self) -> u32 {
        self.characters_.len() as u32
    }

    pub fn Characters(&self) -> *const u16 {
        self.characters_.as_ptr().wrapping_add(self.offset_)
    }

    pub fn GlyphEnd(&self) -> *const u16 {
        self.characters_
            .as_ptr()
            .wrapping_add(self.offset_ + self.current_glyph_length_)
    }

    // cpp: font_engine/fonts/utf16_text_iterator.cc:17-23
    fn IsValidSurrogatePair(&self, character: i32) -> bool {
        if !(0xd800..=0xdbff).contains(&character) {
            return false;
        }
        if self.offset_ + 1 >= self.characters_.len() {
            return false;
        }
        (0xdc00..=0xdfff).contains(&self.characters_[self.offset_ + 1])
    }

    // cpp: font_engine/fonts/utf16_text_iterator.cc:25-34
    fn ConsumeSurrogatePair(&mut self, character: &mut i32) -> bool {
        debug_assert!((0xd800..=0xdfff).contains(character));
        if !self.IsValidSurrogatePair(*character) {
            *character = 0xfffd;
            return true;
        }
        let lead = *character - 0xd800;
        let trail = i32::from(self.characters_[self.offset_ + 1]) - 0xdc00;
        *character = 0x10000 + (lead << 10) + trail;
        self.current_glyph_length_ = 2;
        true
    }
}

// utf16_text_iterator.h:78 declares ConsumeMultipleUChar with no definition
// in the supplied source. The header remains blocked; no behavior is invented.
