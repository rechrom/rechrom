/* Copyright (C) 2003-2012 The Chromium Authors. BSD-style license. */
// cpp: third_party/blink/renderer/core/css/css_markup.h:32-45
// cpp: third_party/blink/renderer/core/css/css_markup.cc:48-216

#![allow(non_snake_case)]

use crate::parser::css_parser_idioms::{IsNameCodePoint, IsNameStartCodePoint};
use foundation::{String, StringView};

pub fn IsCSSTokenizerIdentifier(string: &StringView) -> bool {
    let chars = string.Span16();
    if chars.is_empty() {
        return false;
    }
    let mut index = usize::from(chars[0] == b'-' as u16);
    if index == chars.len() || !IsNameStartCodePoint(chars[index]) {
        return false;
    }
    index += 1;
    chars[index..].iter().all(|&c| IsNameCodePoint(c))
}

pub fn IsCSSTokenizerIdentSequence(string: &StringView) -> bool {
    let chars = string.Span16();
    if chars.is_empty() || chars[0] == b' ' as u16 {
        return false;
    }
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == b'-' as u16 {
            index += 1;
        }
        if index >= chars.len() || !IsNameStartCodePoint(chars[index]) {
            return false;
        }
        index += 1;
        while index < chars.len() && IsNameCodePoint(chars[index]) {
            index += 1;
        }
        if index < chars.len() {
            if chars[index] != b' ' as u16 {
                return false;
            }
            index += 1;
            if index >= chars.len() || chars[index] == b' ' as u16 {
                return false;
            }
        }
    }
    true
}

fn AppendCodePoint(c: u32, output: &mut Vec<u16>) {
    if c <= 0xffff {
        output.push(c as u16);
    } else {
        let scalar = c - 0x10000;
        output.push(0xd800 + (scalar >> 10) as u16);
        output.push(0xdc00 + (scalar & 0x3ff) as u16);
    }
}
fn SerializeCharacter(c: u32, output: &mut Vec<u16>) {
    output.push(b'\\' as u16);
    AppendCodePoint(c, output);
}
fn SerializeCharacterAsCodePoint(c: u32, output: &mut Vec<u16>) {
    output.extend(format!("\\{c:x} ").encode_utf16());
}

pub fn SerializeIdentifierTo(identifier: &String, output: &mut Vec<u16>, skip_start_checks: bool) {
    let units = identifier.Span16().unwrap_or_default();
    let mut is_first = !skip_start_checks;
    let mut is_second = false;
    let mut is_first_char_hyphen = false;
    let mut index = 0;
    while index < units.len() {
        let first = units[index];
        let mut c = u32::from(first);
        if (0xd800..=0xdbff).contains(&first)
            && index + 1 < units.len()
            && (0xdc00..=0xdfff).contains(&units[index + 1])
        {
            c = 0x10000
                + (u32::from(first) - 0xd800) * 0x400
                + (u32::from(units[index + 1]) - 0xdc00);
        }
        index += if c > 0xffff { 2 } else { 1 };
        if c == 0 {
            output.push(0xfffd);
        } else if c <= 0x1f
            || c == 0x7f
            || ((0x30..=0x39).contains(&c) && (is_first || (is_second && is_first_char_hyphen)))
        {
            SerializeCharacterAsCodePoint(c, output);
        } else if c == 0x2d && is_first && index == units.len() {
            SerializeCharacter(c, output);
        } else if c >= 0x80
            || c == 0x2d
            || c == 0x5f
            || (0x30..=0x39).contains(&c)
            || (0x41..=0x5a).contains(&c)
            || (0x61..=0x7a).contains(&c)
        {
            AppendCodePoint(c, output);
        } else {
            SerializeCharacter(c, output);
        }
        if is_first {
            is_first = false;
            is_second = true;
            is_first_char_hyphen = c == 0x2d;
        } else if is_second {
            is_second = false;
        }
    }
}

pub fn SerializeIdentifier(identifier: &String, skip_start_checks: bool) -> String {
    let mut output = Vec::new();
    SerializeIdentifierTo(identifier, &mut output, skip_start_checks);
    String::from_utf16(&output)
}

pub fn SerializeStringTo(string: &String, output: &mut Vec<u16>) {
    output.push(b'"' as u16);
    let units = string.Span16().unwrap_or_default();
    let mut index = 0;
    while index < units.len() {
        let first = units[index];
        let mut c = u32::from(first);
        if (0xd800..=0xdbff).contains(&first)
            && index + 1 < units.len()
            && (0xdc00..=0xdfff).contains(&units[index + 1])
        {
            c = 0x10000
                + (u32::from(first) - 0xd800) * 0x400
                + (u32::from(units[index + 1]) - 0xdc00);
        }
        index += if c > 0xffff { 2 } else { 1 };
        if c <= 0x1f || c == 0x7f {
            SerializeCharacterAsCodePoint(c, output);
        } else if c == 0x22 || c == 0x5c {
            SerializeCharacter(c, output);
        } else {
            AppendCodePoint(c, output);
        }
    }
    output.push(b'"' as u16);
}

pub fn SerializeString(string: &String) -> String {
    let mut output = Vec::new();
    SerializeStringTo(string, &mut output);
    String::from_utf16(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_identifier_and_sequence_match_source_grammar() {
        for value in ["foo", "-foo", "é", "a9-b"] {
            assert!(
                IsCSSTokenizerIdentifier(&StringView::from(value)),
                "{value}"
            );
        }
        for value in ["", "-", "9foo", "foo bar", "foo\\bar"] {
            assert!(
                !IsCSSTokenizerIdentifier(&StringView::from(value)),
                "{value}"
            );
        }
        assert!(IsCSSTokenizerIdentSequence(&StringView::from("Open Sans")));
        assert!(IsCSSTokenizerIdentSequence(&StringView::from("-foo bar")));
        for value in [" Open", "Open ", "Open  Sans", "Open, Sans"] {
            assert!(
                !IsCSSTokenizerIdentSequence(&StringView::from(value)),
                "{value}"
            );
        }
    }

    #[test]
    fn identifier_and_string_serialization_preserve_cssom_rules() {
        for (input, expected) in [
            ("1foo", "\\31 foo"),
            ("-1foo", "-\\31 foo"),
            ("-", "\\-"),
            ("a b", "a\\ b"),
            ("a\0", "a�"),
            ("😀", "😀"),
        ] {
            assert_eq!(
                SerializeIdentifier(&String::FromUtf8(input.as_bytes()), false).Utf8(),
                expected
            );
        }
        assert_eq!(
            SerializeString(&String::from("a\"b\\c\n")).Utf8(),
            "\"a\\\"b\\\\c\\a \""
        );
    }
}
