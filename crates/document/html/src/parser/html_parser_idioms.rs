#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use super::literal_buffer::UCharLiteralBuffer;
use foundation::BlinkString;

// cpp: html/parser/html_parser_idioms.h:12-12
pub const kAttributePrealloc: usize = 10;

// cpp: html/parser/html_parser_idioms.h:14-19
pub fn IsHTMLSpace<CharType: Copy + Into<u32>>(character: CharType) -> bool {
    let character = character.into();
    character <= u32::from(b' ') && matches!(character, 0x20 | 0x0A | 0x09 | 0x0D | 0x0C)
}

// cpp: html/parser/html_parser_idioms.h:21-21
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CharacterWidth {
    kLikely8Bit,
    kForce8Bit,
    kForce16Bit,
}

// cpp: html/parser/html_parser_idioms.h:23-25
pub fn AttemptStaticStringCreationLChar(characters: &[u8]) -> BlinkString {
    BlinkString::from_latin1(characters)
}

// cpp: html/parser/html_parser_idioms.h:27-34
pub fn AttemptStaticStringCreationUChar(characters: &[u16], width: CharacterWidth) -> BlinkString {
    if width == CharacterWidth::kLikely8Bit {
        return BlinkString::Create8BitIfPossible(characters);
    }
    if width == CharacterWidth::kForce8Bit {
        debug_assert!(characters.iter().all(|character| *character <= 0xff));
        return BlinkString::Make8BitFrom16BitSource(characters);
    }
    BlinkString::from_utf16(characters)
}

// cpp: html/parser/html_parser_idioms.h:36-41
pub fn AttemptStaticStringCreationBuffer<const INLINE: usize>(
    buffer: &UCharLiteralBuffer<INLINE>,
) -> BlinkString {
    AttemptStaticStringCreationUChar(
        buffer.as_slice(),
        if buffer.Is8Bit() {
            CharacterWidth::kForce8Bit
        } else {
            CharacterWidth::kForce16Bit
        },
    )
}
