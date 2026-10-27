/* Copyright (C) 2013 Google Inc. All rights reserved. */
// cpp: third_party/blink/renderer/core/css/parser/css_parser_idioms.h

#![allow(non_snake_case, non_upper_case_globals)]

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
