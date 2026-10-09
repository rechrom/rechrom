/* Copyright (C) 2013 Google Inc. All rights reserved. */
// cpp: third_party/blink/renderer/core/css/parser/css_parser_idioms.h
// cpp: third_party/blink/renderer/core/css/parser/css_parser_idioms.cc

#![allow(non_snake_case)]

use super::css_tokenizer_input_stream::CSSTokenizerInputStream;
use foundation::String;

pub const kReplacementCharacter: u32 = 0xfffd;

pub const fn IsCSSSpace(c: u16) -> bool {
    c == b' ' as u16 || c == b'\t' as u16 || c == b'\n' as u16
}
pub const fn IsCSSNewLine(c: u16) -> bool {
    c == b'\r' as u16 || c == b'\n' as u16 || c == 0x0c
}
pub const fn IsHTMLSpace(c: u16) -> bool {
    IsCSSSpace(c) || c == b'\r' as u16 || c == 0x0c
}
pub const fn IsNameStartCodePoint(c: u16) -> bool {
    (c >= b'a' as u16 && c <= b'z' as u16)
        || (c >= b'A' as u16 && c <= b'Z' as u16)
        || c == b'_' as u16
        || c > 0x7f
}
pub const fn IsNameCodePoint(c: u16) -> bool {
    IsNameStartCodePoint(c) || (c >= b'0' as u16 && c <= b'9' as u16) || c == b'-' as u16
}
pub const fn TwoCharsAreValidEscape(first: u16, second: u16) -> bool {
    first == b'\\' as u16 && !IsCSSNewLine(second)
}
pub const fn IsSurrogate(c: u16) -> bool {
    c >= 0xd800 && c <= 0xdfff
}
pub const fn IsLeadingSurrogate(c: u16) -> bool {
    c >= 0xd800 && c <= 0xdbff
}
pub const fn IsTrailingSurrogate(c: u16) -> bool {
    c >= 0xdc00 && c <= 0xdfff
}

pub fn ConsumeSingleWhitespaceIfNext(input: &mut CSSTokenizerInputStream) {
    let next = input.PeekWithoutReplacement(0);
    if next == b'\r' as u16 && input.PeekWithoutReplacement(1) == b'\n' as u16 {
        input.AdvanceBy(2);
    } else if IsHTMLSpace(next) {
        input.Advance();
    }
}

const fn IsAsciiHexDigit(c: u16) -> bool {
    (c >= b'0' as u16 && c <= b'9' as u16)
        || (c >= b'a' as u16 && c <= b'f' as u16)
        || (c >= b'A' as u16 && c <= b'F' as u16)
}
const fn HexValue(c: u16) -> u32 {
    if c <= b'9' as u16 {
        (c - b'0' as u16) as u32
    } else if c >= b'a' as u16 {
        (c - b'a' as u16 + 10) as u32
    } else {
        (c - b'A' as u16 + 10) as u32
    }
}

pub fn ConsumeEscape(input: &mut CSSTokenizerInputStream) -> u32 {
    let mut cc = input.NextInputChar();
    input.Advance();
    debug_assert!(!IsCSSNewLine(cc));
    if IsAsciiHexDigit(cc) {
        let mut consumed = 1;
        let mut code_point = HexValue(cc);
        while consumed < 6 && IsAsciiHexDigit(input.PeekWithoutReplacement(0)) {
            cc = input.NextInputChar();
            input.Advance();
            code_point = code_point * 16 + HexValue(cc);
            consumed += 1;
        }
        ConsumeSingleWhitespaceIfNext(input);
        if code_point == 0 || (0xd800..=0xdfff).contains(&code_point) || code_point > 0x10ffff {
            return kReplacementCharacter;
        }
        return code_point;
    }
    if cc == 0 {
        kReplacementCharacter
    } else {
        u32::from(cc)
    }
}

fn AppendCodePoint(output: &mut Vec<u16>, code_point: u32) {
    if code_point <= 0xffff {
        output.push(code_point as u16);
    } else {
        let scalar = code_point - 0x10000;
        output.push(0xd800 + (scalar >> 10) as u16);
        output.push(0xdc00 + (scalar & 0x3ff) as u16);
    }
}

pub fn ConsumeName(input: &mut CSSTokenizerInputStream) -> String {
    let mut result = Vec::new();
    loop {
        let cc = input.NextInputChar();
        if IsNameCodePoint(cc) {
            input.Advance();
            result.push(cc);
        } else if TwoCharsAreValidEscape(cc, input.PeekWithoutReplacement(1)) {
            input.Advance();
            AppendCodePoint(&mut result, ConsumeEscape(input));
        } else {
            return String::from_utf16(&result);
        }
    }
}

pub fn NextCharsAreIdentifier(first: u16, input: &CSSTokenizerInputStream) -> bool {
    let second = input.NextInputChar();
    if IsNameStartCodePoint(first) || TwoCharsAreValidEscape(first, second) {
        return true;
    }
    if first == b'-' as u16 {
        return IsNameStartCodePoint(second)
            || second == b'-' as u16
            || TwoCharsAreValidEscape(second, input.PeekWithoutReplacement(1));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::StringView;

    #[test]
    fn predicates_match_css_syntax_code_points() {
        assert!(IsCSSSpace(b' ' as u16));
        assert!(!IsCSSSpace(b'\r' as u16));
        assert!(IsCSSNewLine(b'\r' as u16));
        assert!(IsNameStartCodePoint('é' as u16));
        assert!(IsNameCodePoint(b'9' as u16));
        assert!(TwoCharsAreValidEscape(b'\\' as u16, b'x' as u16));
        assert!(!TwoCharsAreValidEscape(b'\\' as u16, b'\n' as u16));
    }

    #[test]
    fn input_preprocessing_replaces_only_null_and_lone_surrogates() {
        let null = String::from_utf16(&[0]);
        assert_eq!(
            CSSTokenizerInputStream::new(StringView::from(&null)).NextInputChar(),
            0xfffd
        );
        let lone = String::from_utf16(&[0xd800]);
        assert_eq!(
            CSSTokenizerInputStream::new(StringView::from(&lone)).NextInputChar(),
            0xfffd
        );
        let pair = String::from_utf16(&[0xd83d, 0xde00]);
        let mut input = CSSTokenizerInputStream::new(StringView::from(&pair));
        assert_eq!(input.NextInputChar(), 0xd83d);
        input.Advance();
        assert_eq!(input.NextInputChar(), 0xde00);
    }

    #[test]
    fn consumes_escapes_names_and_crlf_as_source_specifies() {
        let mut escape = CSSTokenizerInputStream::new(StringView::from("1f600 "));
        assert_eq!(ConsumeEscape(&mut escape), 0x1f600);
        assert!(escape.AtEnd());

        let mut invalid = CSSTokenizerInputStream::new(StringView::from("0 "));
        assert_eq!(ConsumeEscape(&mut invalid), kReplacementCharacter);

        let mut name = CSSTokenizerInputStream::new(StringView::from("hello\\1f600 world!"));
        assert_eq!(ConsumeName(&mut name).Utf8(), "hello😀world");
        assert_eq!(name.NextInputChar(), b'!' as u16);

        let mut whitespace = CSSTokenizerInputStream::new(StringView::from("\r\nx"));
        ConsumeSingleWhitespaceIfNext(&mut whitespace);
        assert_eq!(whitespace.Offset(), 2);
    }

    #[test]
    fn identifier_lookahead_and_stream_ranges_are_stable() {
        let text = String::from_latin1(b"-x123");
        let mut input = CSSTokenizerInputStream::new(StringView::from(&text));
        assert!(NextCharsAreIdentifier(b'-' as u16, &input));
        input.AdvanceBy(2);
        assert_eq!(input.Offset(), 2);
        assert_eq!(input.Peek().ToString().Utf8(), "123");
        assert_eq!(input.RangeAt(1, 2).ToString().Utf8(), "x1");
        input.Restore(0);
        assert_eq!(input.GetNaturalNumberAsDoubleFast(2, 5), Some(123.0));

        let long = String::from_latin1(b"123456789012345");
        let input = CSSTokenizerInputStream::new(StringView::from(&long));
        assert_eq!(input.GetNaturalNumberAsDoubleFast(0, 15), None);
        assert_eq!(input.GetNaturalNumberAsDouble(0, 15), 123456789012345.0);
    }
}
