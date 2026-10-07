#![allow(non_snake_case)]

use super::character_layout_property_data::{
    kCjkRanges, kCursiveRanges, kEastAsianSpacingRanges, kHanKerningRanges, LayoutPropertyValue,
};
use super::emoji_property_data::{
    kEmojiModifierBaseRanges, kEmojiPresentationRanges, kEmojiRanges, HasEmojiProperty,
};
use super::grapheme_break_property_data::IsExtendedPictographic as GraphemeIsExtendedPictographic;
use super::vertical_orientation_data::{
    kGraphemeExtendRanges, kUprightVerticalRanges, InBinaryPropertyRanges,
};
use foundation::{EastAsianSpacingType, HanKerningCharType, String};
// The declaration and category masks are foundation-owned; the Category
// implementation is supplied by this package's unicode_category.cc.
use super::unicode_category::Category;
use foundation::unicode::{
    kMark_Enclosing, kMark_NonSpacing, kMark_SpacingCombining, kOther_NotAssigned,
    kOther_PrivateUse,
};

// C++ STATIC_ONLY class: all operations are associated functions and there
// is no instance state. The remaining source declarations without definitions
// cpp: font_engine/text/native/character.h:53-56
pub struct Character;

impl Character {
    // cpp: font_engine/text/native/character.h:57-70
    pub fn IsInRange(character: i32, lower_bound: i32, upper_bound: i32) -> bool {
        character >= lower_bound && character <= upper_bound
    }
    pub fn IsBlockCjkSymbolsAndPunctuation(ch: i32) -> bool {
        Self::IsInRange(ch, 0x3000, 0x303F)
    }
    pub fn IsBlockHalfwidthAndFullwidthForms(ch: i32) -> bool {
        Self::IsInRange(ch, 0xFF00, 0xFFEF)
    }

    // cpp: font_engine/text/native/character.h:83-115
    pub fn IsUnicodeVariationSelector(character: i32) -> bool {
        Self::IsInRange(character, 0x180B, 0x180D)
            || Self::IsInRange(character, 0xFE00, 0xFE0F)
            || Self::IsInRange(character, 0xE0100, 0xE01EF)
    }
    pub fn IsUnicodeEmojiVariationSelector(character: i32) -> bool {
        character == 0xFE0E || character == 0xFE0F
    }
    pub fn IsCjkIdeographOrSymbol(c: i32) -> bool {
        if c < 0x2C7 {
            false
        } else {
            Self::IsCjkIdeographOrSymbolSlow(c)
        }
    }
    pub fn IsHangul(c: i32) -> bool {
        if c < 0x1100 {
            false
        } else {
            Self::IsHangulSlow(c)
        }
    }

    // cpp: font_engine/text/native/character.h:130-160
    pub fn MaybeHanKerningOpen(ch: i32) -> bool {
        Self::MaybeHanKerningOpenOrCloseFast(ch) && Self::MaybeHanKerningOpenSlow(ch)
    }
    pub fn MaybeHanKerningClose(ch: i32) -> bool {
        Self::MaybeHanKerningOpenOrCloseFast(ch) && Self::MaybeHanKerningCloseSlow(ch)
    }
    pub fn MaybeHanKerningOpenOrCloseFast(character: i32) -> bool {
        Self::IsInRange(character, 0x2018, 0x301F) || Self::IsInRange(character, 0xFF08, 0xFF60)
    }
    pub fn MaybeHanKerningMiddle(ch: i32) -> bool {
        Self::MaybeHanKerningMiddleSlow(ch)
    }
    pub fn IsCollapsibleSpace(c: u16) -> bool {
        c == 0x20 || c == 0x0A || c == 0x09 || c == 0x0D
    }
    pub fn IsLineFeed(c: u16) -> bool {
        c == 0x0A
    }
    pub fn IsOtherSpaceSeparator(c: i32) -> bool {
        c == 0x3000
    }

    // cpp: font_engine/text/native/character.h:161-201
    pub fn TreatAsSpace(c: i32) -> bool {
        c == 0x20 || c == 0x09 || c == 0x0A || c == 0x00A0
    }
    pub fn TreatAsZeroWidthSpace(c: i32) -> bool {
        Self::TreatAsZeroWidthSpaceInComplexScript(c) || c == 0x200C || c == 0x200D
    }
    pub fn TreatAsZeroWidthSpaceInComplexScriptLegacy(c: i32) -> bool {
        c == 0x0C
            || c == 0x0D
            || c == 0x00AD
            || c == 0x200B
            || Self::IsInRange(c, 0x200E, 0x200F)
            || Self::IsInRange(c, 0x202A, 0x202E)
            || c == 0xFEFF
            || c == 0xFFFC
    }
    pub fn TreatAsZeroWidthSpaceInComplexScript(c: i32) -> bool {
        if c == 0x0C || c == 0x0D || c == 0xFFFC {
            return true;
        }
        Self::IsDefaultIgnorable(c)
    }
    pub fn IsDefaultIgnorable(c: i32) -> bool {
        c == 0x00AD
            || c == 0x034F
            || c == 0x061C
            || Self::IsInRange(c, 0x115F, 0x1160)
            || Self::IsInRange(c, 0x17B4, 0x17B5)
            || Self::IsInRange(c, 0x180B, 0x180F)
            || Self::IsInRange(c, 0x200B, 0x200F)
            || Self::IsInRange(c, 0x202A, 0x202E)
            || Self::IsInRange(c, 0x2060, 0x206F)
            || c == 0x3164
            || Self::IsInRange(c, 0xFE00, 0xFE0F)
            || c == 0xFEFF
            || c == 0xFFA0
            || Self::IsInRange(c, 0xFFF0, 0xFFF8)
            || Self::IsInRange(c, 0x1BCA0, 0x1BCA3)
            || Self::IsInRange(c, 0x1D173, 0x1D17A)
            || Self::IsInRange(c, 0xE0000, 0xE0FFF)
    }

    // cpp: font_engine/text/native/character.h:219-246,260-269
    pub fn IsEmojiKeycapBase(ch: i32) -> bool {
        Self::IsInRange(ch, 0x30, 0x39) || ch == 0x23 || ch == 0x2A
    }
    pub fn IsModifier(c: i32) -> bool {
        Self::IsInRange(c, 0x1F3FB, 0x1F3FF)
    }
    pub fn IsNormalizedCanvasSpaceCharacter(c: i32) -> bool {
        c == 0x0009 || Self::IsInRange(c, 0x000A, 0x000D)
    }
    pub fn IsModernGeorgianUppercase(c: i32) -> bool {
        Self::IsInRange(c, 0x1C90, 0x1CBF)
    }
    pub fn LowercaseModernGeorgianUppercase(c: i32) -> i32 {
        c - (0x1C90 - 0x10D0)
    }

    // cpp: font_engine/text/native/character.h:301-334
    pub fn MaybeBidiRtlUtf16(ch: u16) -> bool {
        let ch = i32::from(ch);
        ch >= 0x0590
            && ch != 0x200B
            && !Self::IsInRange(ch, 0x2010, 0x2029)
            && !Self::IsInRange(ch, 0x206A, 0xD7FF)
            && !Self::IsInRange(ch, 0xFF00, 0xFFFF)
    }
    pub fn MaybeBidiRtl(ch: i32) -> bool {
        ch >= 0x0590
            && ch != 0x200B
            && !Self::IsInRange(ch, 0x2010, 0x2029)
            && !Self::IsInRange(ch, 0x206A, 0xD7FF)
            && !Self::IsInRange(ch, 0xFF00, 0xFFFF)
            && !Self::IsInRange(ch, 0x1AFF0, 0x1B16F)
            && !Self::IsInRange(ch, 0x20000, 0x323AF)
    }
    pub fn MaybeBidiRtlString(text: &String) -> bool {
        // The current Rust String stores UTF-16 code units. The C++ 8-bit
        // early exit is equivalent here because Latin-1 is below U+0590.
        text.Span16()
            .is_some_and(|units| units.iter().any(|&ch| Self::MaybeBidiRtlUtf16(ch)))
    }
    // cpp: font_engine/text/native/character.h:345-353
    pub fn MayNeedEastAsianSpacing(ch: i32) -> bool {
        ch >= 0x02C7 && ch != 0xFFFC && !Self::IsInRange(ch, 0x1200, 0x3004)
    }

    // cpp: font_engine/text/native/character_unicode_data.cc:47-60
    // cpp: font_engine/text/native/character_unicode_data.cc:41-45
    pub fn IsGcMark(character: i32) -> bool {
        Category(character) & (kMark_SpacingCombining | kMark_Enclosing | kMark_NonSpacing) != 0
    }

    pub fn IsUprightInMixedVertical(character: i32) -> bool {
        InBinaryPropertyRanges(character as u32, &kUprightVerticalRanges)
    }
    pub fn IsGraphemeExtended(character: i32) -> bool {
        InBinaryPropertyRanges(character as u32, &kGraphemeExtendRanges)
    }
    fn IsCjkIdeographOrSymbolSlow(character: i32) -> bool {
        LayoutPropertyValue(character as u32, &kCjkRanges) != 0
    }

    // cpp: font_engine/text/native/character_unicode_data.cc:62-72
    pub fn IsBidiControl(character: i32) -> bool {
        character == 0x061C
            || Self::IsInRange(character, 0x200E, 0x200F)
            || Self::IsInRange(character, 0x202A, 0x202E)
            || Self::IsInRange(character, 0x2066, 0x2069)
    }
    fn IsHangulSlow(character: i32) -> bool {
        Self::IsInRange(character, 0x1100, 0x11FF)
            || Self::IsInRange(character, 0xA960, 0xA97F)
            || Self::IsInRange(character, 0xAC00, 0xD7FF)
    }

    // C++ static_cast from the generated tables preserves enum discriminants.
    // cpp: font_engine/text/native/character_unicode_data.cc:74-103
    pub fn GetHanKerningCharType(character: i32) -> HanKerningCharType {
        let value = LayoutPropertyValue(character as u32, &kHanKerningRanges);
        assert!(value <= HanKerningCharType::kInvalid as u8);
        unsafe { std::mem::transmute::<u8, HanKerningCharType>(value) }
    }
    pub fn GetEastAsianSpacingType(character: i32) -> EastAsianSpacingType {
        let value = LayoutPropertyValue(character as u32, &kEastAsianSpacingRanges);
        assert!(value <= EastAsianSpacingType::kWide as u8);
        unsafe { std::mem::transmute::<u8, EastAsianSpacingType>(value) }
    }
    fn MaybeHanKerningOpenSlow(character: i32) -> bool {
        matches!(
            Self::GetHanKerningCharType(character),
            HanKerningCharType::kOpen | HanKerningCharType::kOpenQuote
        )
    }
    fn MaybeHanKerningCloseSlow(character: i32) -> bool {
        matches!(
            Self::GetHanKerningCharType(character),
            HanKerningCharType::kClose | HanKerningCharType::kCloseQuote
        )
    }
    fn MaybeHanKerningMiddleSlow(character: i32) -> bool {
        Self::GetHanKerningCharType(character) == HanKerningCharType::kMiddle
    }
    pub fn IsCursiveScript(character: i32) -> bool {
        LayoutPropertyValue(character as u32, &kCursiveRanges) != 0
    }

    // cpp: font_engine/text/native/character_unicode_data.cc:105-138
    pub fn IsExtendedPictographic(character: i32) -> bool {
        GraphemeIsExtendedPictographic(character as u32)
    }
    pub fn IsEmoji(character: i32) -> bool {
        HasEmojiProperty(character as u32, &kEmojiRanges)
    }
    pub fn IsEmojiTextDefault(character: i32) -> bool {
        Self::IsEmoji(character) && !Self::IsEmojiEmojiDefault(character)
    }
    pub fn IsEmojiEmojiDefault(character: i32) -> bool {
        HasEmojiProperty(character as u32, &kEmojiPresentationRanges)
    }
    pub fn IsEmojiModifierBase(character: i32) -> bool {
        HasEmojiProperty(character as u32, &kEmojiModifierBaseRanges)
    }
    // cpp: font_engine/text/native/character_unicode_data.cc:128-132
    pub fn IsEmojiReserved(character: i32) -> bool {
        Category(character) & kOther_NotAssigned != 0 && Self::IsExtendedPictographic(character)
    }
    pub fn IsEmojiIncludingReserved(character: i32) -> bool {
        Self::IsEmoji(character) || Self::IsEmojiReserved(character)
    }
    pub fn IsRegionalIndicator(character: i32) -> bool {
        Self::IsInRange(character, 0x1F1E6, 0x1F1FF)
    }

    // cpp: font_engine/text/native/character_unicode_data.cc:141-167
    pub fn IsEmojiTagSequence(character: i32) -> bool {
        Self::IsInRange(character, 0xE0030, 0xE0039) || Self::IsInRange(character, 0xE0061, 0xE007A)
    }
    pub fn MaybeEmojiPresentation(character: i32) -> bool {
        if (0..=0x7F).contains(&character) {
            return character == 0xA9 || character == 0xAE || Self::IsEmojiKeycapBase(character);
        }
        character == 0x200D
            || Self::IsInRange(character, 0x203C, 0x2B55)
            || character == 0xFE0E
            || character == 0x3030
            || character == 0x303D
            || character == 0x3297
            || character == 0x3299
            || character == 0xFE0F
            || character >= 0x10000
    }
    pub fn IsNonCharacter(character: i32) -> bool {
        // U_IS_UNICODE_NONCHAR from icu_bidi/unicode/utf.h:161-163.
        character >= 0xFDD0
            && (character <= 0xFDEF || (character & 0xFFFE) == 0xFFFE)
            && character <= 0x10FFFF
    }
    // cpp: font_engine/text/native/character_unicode_data.cc:159-161
    pub fn IsPrivateUse(character: i32) -> bool {
        Category(character) & kOther_PrivateUse != 0
    }
}
