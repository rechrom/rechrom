// cpp: layoutng_style/style/style_hyphenate_limit_chars.h:13-25
/// CSS hyphenate-limit-chars values, stored as saturated u8 counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleHyphenateLimitChars {
    min_word_chars_: u8,
    min_before_chars_: u8,
    min_after_chars_: u8,
}

#[allow(non_snake_case)]
impl StyleHyphenateLimitChars {
    pub fn new(min_word_chars: u32, min_before_chars: u32, min_after_chars: u32) -> Self {
        Self {
            min_word_chars_: min_word_chars.min(u8::MAX as u32) as u8,
            min_before_chars_: min_before_chars.min(u8::MAX as u32) as u8,
            min_after_chars_: min_after_chars.min(u8::MAX as u32) as u8,
        }
    }

    // cpp: layoutng_style/style/style_hyphenate_limit_chars.h:27-36
    pub fn MinWordChars(&self) -> u32 {
        self.min_word_chars_ as u32
    }
    pub fn MinBeforeChars(&self) -> u32 {
        self.min_before_chars_ as u32
    }
    pub fn MinAfterChars(&self) -> u32 {
        self.min_after_chars_ as u32
    }
    pub fn IsAuto(&self) -> bool {
        self.min_word_chars_ == 0 && self.min_before_chars_ == 0 && self.min_after_chars_ == 0
    }
}

// cpp: layoutng_style/style/style_hyphenate_limit_chars.h:38-51
// The derived PartialEq compares all three u8 fields in source order.
