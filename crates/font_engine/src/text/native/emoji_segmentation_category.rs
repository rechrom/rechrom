#![allow(non_snake_case, non_camel_case_types)]

use super::character::Character;
use std::cmp::Ordering;
use std::ops::Sub;

// cpp: font_engine/text/native/emoji_segmentation_category.h:14-37
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmojiSegmentationCategory {
    EMOJI = 0,
    EMOJI_TEXT_PRESENTATION = 1,
    EMOJI_EMOJI_PRESENTATION = 2,
    EMOJI_MODIFIER_BASE = 3,
    EMOJI_MODIFIER = 4,
    EMOJI_VS_BASE = 5,
    REGIONAL_INDICATOR = 6,
    KEYCAP_BASE = 7,
    COMBINING_ENCLOSING_KEYCAP = 8,
    COMBINING_ENCLOSING_CIRCLE_BACKSLASH = 9,
    ZWJ = 10,
    VS15 = 11,
    VS16 = 12,
    TAG_BASE = 13,
    TAG_SEQUENCE = 14,
    TAG_TERM = 15,
    kMaxCategory = 16,
    kInvalidCacheEntry = 17,
}

// C++ overloads five mixed enum/u8 comparisons for the generated scanner.
// Rust traits preserve their argument order and u8 subtraction wraparound.
// cpp: font_engine/text/native/emoji_segmentation_category.h:39-56
impl PartialEq<u8> for EmojiSegmentationCategory {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}

impl PartialOrd<u8> for EmojiSegmentationCategory {
    fn partial_cmp(&self, other: &u8) -> Option<Ordering> {
        (*self as u8).partial_cmp(other)
    }
}

impl PartialEq<EmojiSegmentationCategory> for u8 {
    fn eq(&self, other: &EmojiSegmentationCategory) -> bool {
        *self == *other as u8
    }
}

impl PartialOrd<EmojiSegmentationCategory> for u8 {
    fn partial_cmp(&self, other: &EmojiSegmentationCategory) -> Option<Ordering> {
        self.partial_cmp(&(*other as u8))
    }
}

impl Sub<u8> for EmojiSegmentationCategory {
    type Output = u8;

    fn sub(self, other: u8) -> Self::Output {
        (self as u8).wrapping_sub(other)
    }
}

// cpp: font_engine/text/native/emoji_segmentation_category.h:58-65
pub fn IsEmojiPresentationCategory(emoji: EmojiSegmentationCategory) -> bool {
    emoji != EmojiSegmentationCategory::kMaxCategory
        && emoji != EmojiSegmentationCategory::KEYCAP_BASE
        && emoji != EmojiSegmentationCategory::EMOJI_TEXT_PRESENTATION
        && emoji != EmojiSegmentationCategory::VS15
}

// This is a header-only function with internal linkage in C++. Keeping it in
// the same Rust module as its enum exposes one callable source-faithful unit.
// cpp: font_engine/text/native/emoji_segmentation_category_inline_header.h:13-78
pub fn GetEmojiSegmentationCategory(codepoint: i32) -> EmojiSegmentationCategory {
    use EmojiSegmentationCategory::*;
    if codepoint <= 0x7F {
        if Character::IsEmojiKeycapBase(codepoint) {
            return KEYCAP_BASE;
        }
        return kMaxCategory;
    }

    if codepoint == 0x20E3 {
        return COMBINING_ENCLOSING_KEYCAP;
    }
    if codepoint == 0x20E0 {
        return COMBINING_ENCLOSING_CIRCLE_BACKSLASH;
    }
    if codepoint == 0x200D {
        return ZWJ;
    }
    if codepoint == 0xFE0E {
        return VS15;
    }
    if codepoint == 0xFE0F {
        return VS16;
    }
    if codepoint == 0x1F3F4 {
        return TAG_BASE;
    }
    if Character::IsEmojiTagSequence(codepoint) {
        return TAG_SEQUENCE;
    }
    if codepoint == 0xE007F {
        return TAG_TERM;
    }
    if Character::IsEmojiModifierBase(codepoint) {
        return EMOJI_MODIFIER_BASE;
    }
    if Character::IsModifier(codepoint) {
        return EMOJI_MODIFIER;
    }
    if Character::IsRegionalIndicator(codepoint) {
        return REGIONAL_INDICATOR;
    }
    if Character::IsEmojiEmojiDefault(codepoint) {
        return EMOJI_EMOJI_PRESENTATION;
    }
    if Character::IsEmojiTextDefault(codepoint) {
        return EMOJI_TEXT_PRESENTATION;
    }
    if Character::IsEmojiIncludingReserved(codepoint) {
        return EMOJI;
    }
    kMaxCategory
}
