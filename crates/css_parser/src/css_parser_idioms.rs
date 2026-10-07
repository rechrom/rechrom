#![allow(non_snake_case, non_upper_case_globals)]

use crate::css_tokenizer_input_stream::CSSTokenizerInputStream;
use foundation::{String, UChar, UChar32};

// cpp: css_parser/css_parser_idioms.h:41-42
pub const kCSSEndOfFileMarker: UChar = 0;
pub const kCSSReplacementCharacter: UChar = 0xFFFD;

// cpp: css_parser/css_parser_idioms.h:44-48
pub fn IsCSSSpace(c: UChar) -> bool {
    c == u16::from(b' ') || c == u16::from(b'\t') || c == u16::from(b'\n')
}

// cpp: css_parser/css_parser_idioms.h:50-55
pub fn IsCSSInputSpace(c: UChar) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0A | 0x0D | 0x0C)
}

// cpp: css_parser/css_parser_idioms.h:57-60
pub fn IsCSSNewLine(cc: UChar) -> bool {
    matches!(cc, 0x0D | 0x0A | 0x0C)
}

// cpp: css_parser/css_parser_idioms.h:62-72
pub fn IsNameStartCodePoint(c: UChar) -> bool {
    (0x41..=0x5A).contains(&c) || (0x61..=0x7A).contains(&c) || c == 0x5F || c > 0x7F
}
pub fn IsNameCodePoint(c: UChar) -> bool {
    IsNameStartCodePoint(c) || (0x30..=0x39).contains(&c) || c == 0x2D
}

fn ascii_hex_value(c: UChar) -> Option<u32> {
    match c {
        0x30..=0x39 => Some(u32::from(c - 0x30)),
        0x41..=0x46 => Some(u32::from(c - 0x41 + 10)),
        0x61..=0x66 => Some(u32::from(c - 0x61 + 10)),
        _ => None,
    }
}

// cpp: css_parser/css_parser_idioms.h:74-77
pub fn TwoCharsAreValidEscape(first: UChar, second: UChar) -> bool {
    first == u16::from(b'\\') && !IsCSSNewLine(second)
}

// cpp: css_parser/css_parser_idioms.h:79-92
pub fn IsSurrogate(cc: UChar) -> bool {
    (0xD800..=0xDFFF).contains(&cc)
}
pub fn IsLeadingSurrogate(cc: UChar) -> bool {
    (0xD800..=0xDBFF).contains(&cc)
}
pub fn IsTrailingSurrogate(cc: UChar) -> bool {
    (0xDC00..=0xDFFF).contains(&cc)
}

// cpp: css_parser/css_parser_idioms.cc:14-22
pub fn ConsumeSingleWhitespaceIfNext(input: &mut CSSTokenizerInputStream) {
    let next = input.PeekWithoutReplacement(0);
    if next == u16::from(b'\r') && input.PeekWithoutReplacement(1) == u16::from(b'\n') {
        input.Advance(2);
    } else if IsCSSInputSpace(next) {
        input.Advance(1);
    }
}

// cpp: css_parser/css_parser_idioms.cc:25-54
pub fn ConsumeEscape(input: &mut CSSTokenizerInputStream) -> UChar32 {
    let mut cc = input.NextInputChar();
    input.Advance(1);
    debug_assert!(!IsCSSNewLine(cc));
    if let Some(first_hex) = ascii_hex_value(cc) {
        let mut consumed_hex_digits = 1;
        let mut code_point = first_hex;
        while consumed_hex_digits < 6 && ascii_hex_value(input.PeekWithoutReplacement(0)).is_some()
        {
            cc = input.NextInputChar();
            input.Advance(1);
            code_point = code_point * 16 + ascii_hex_value(cc).expect("hex digit");
            consumed_hex_digits += 1;
        }
        ConsumeSingleWhitespaceIfNext(input);
        if code_point == 0 || (0xD800..=0xDFFF).contains(&code_point) || code_point > 0x10FFFF {
            return i32::from(kCSSReplacementCharacter);
        }
        return code_point as UChar32;
    }
    if cc == kCSSEndOfFileMarker {
        return i32::from(kCSSReplacementCharacter);
    }
    i32::from(cc)
}

// cpp: css_parser/css_parser_idioms.cc:57-71
pub fn ConsumeName(input: &mut CSSTokenizerInputStream) -> String {
    let mut result = Vec::<u16>::new();
    loop {
        let cc = input.NextInputChar();
        if IsNameCodePoint(cc) {
            input.Advance(1);
            result.push(cc);
        } else if TwoCharsAreValidEscape(cc, input.PeekWithoutReplacement(1)) {
            input.Advance(1);
            let escaped = ConsumeEscape(input) as u32;
            let ch = char::from_u32(escaped).expect("escaped CSS code point is valid");
            let mut units = [0; 2];
            result.extend(ch.encode_utf16(&mut units).iter().copied());
        } else {
            return String::from_utf16(&result);
        }
    }
}

// cpp: css_parser/css_parser_idioms.cc:74-86
pub fn NextCharsAreIdentifier(first: UChar, input: &CSSTokenizerInputStream) -> bool {
    let second = input.NextInputChar();
    if IsNameStartCodePoint(first) || TwoCharsAreValidEscape(first, second) {
        return true;
    }
    if first == u16::from(b'-') {
        return IsNameStartCodePoint(second)
            || second == u16::from(b'-')
            || TwoCharsAreValidEscape(second, input.PeekWithoutReplacement(1));
    }
    false
}
