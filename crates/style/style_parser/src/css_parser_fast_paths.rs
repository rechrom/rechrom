// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_fast_paths.cc
//
// CSSParserFastPaths::MaybeParseValue for the simple length, color and
// simple transform branches (css_parser_fast_paths.cc:54-229, 231-835,
// 891-1136, 1999-2283). A string these branches do not accept returns None and
// the caller continues with the full property parser, exactly as
// CSSParser::ParseValue (css_parser.cc:179-232) does after a fast-path miss.
// The keyword branch (ParseKeywordValue) and hsl()/hsla() are not translated
// yet; their inputs take the full parser, which produces the same values.

#![allow(non_snake_case)]

use crate::css_parser_mode::{CSSParserMode, IsQuirksModeBehavior};
use crate::css_primitive_value::UnitType;
use crate::css_property_parser::CssValueKeywordID;
use crate::css_value_keywords::IsValueAllowedInMode;
use crate::production_css_value::{self as values, Value};
use foundation::{CSSPropertyID, CSSValueID, Color, StringView};
use std::rc::Rc;

// cpp: wtf/text/ascii_ctype.h IsHTMLSpace (html_parser_idioms.h).
#[inline]
fn IsHTMLSpace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0c | b'\r')
}

// cpp: css_parser_fast_paths.cc:54-120
#[inline]
fn IsSimpleLengthPropertyID(property_id: CSSPropertyID, accepts_negative_numbers: &mut bool) -> bool {
    use CSSPropertyID::*;
    *accepts_negative_numbers = matches!(
        property_id,
        kBottom
            | kCx
            | kCy
            | kLeft
            | kMarginBottom
            | kMarginLeft
            | kMarginRight
            | kMarginTop
            | kOffsetDistance
            | kRight
            | kTop
            | kMarginBlockEnd
            | kMarginBlockStart
            | kMarginInlineEnd
            | kMarginInlineStart
            | kX
            | kY
    );
    *accepts_negative_numbers
        || matches!(
            property_id,
            kBlockSize
                | kInlineSize
                | kMinBlockSize
                | kMinInlineSize
                | kFontSize
                | kHeight
                | kWidth
                | kMinHeight
                | kMinWidth
                | kPaddingBottom
                | kPaddingLeft
                | kPaddingRight
                | kPaddingTop
                | kScrollPaddingBlockEnd
                | kScrollPaddingBlockStart
                | kScrollPaddingBottom
                | kScrollPaddingInlineEnd
                | kScrollPaddingInlineStart
                | kScrollPaddingLeft
                | kScrollPaddingRight
                | kScrollPaddingTop
                | kPaddingBlockEnd
                | kPaddingBlockStart
                | kPaddingInlineEnd
                | kPaddingInlineStart
                | kShapeMargin
                | kR
                | kRx
                | kRy
        )
}

// cpp: css_parser_fast_paths.cc:122-144
#[inline]
fn ParseSimpleLength(characters: &[u8], unit: &mut UnitType, number: &mut f64) -> bool {
    let length = characters.len();
    let mut characters = characters;
    if length > 2
        && (characters[length - 2] | 0x20) == b'p'
        && (characters[length - 1] | 0x20) == b'x'
    {
        characters = &characters[..length - 2];
        *unit = UnitType::kPixels;
    } else if length > 1 && characters[length - 1] == b'%' {
        characters = &characters[..length - 1];
        *unit = UnitType::kPercentage;
    }
    // ParseDoubleWithPrefix() also validates that the whole range is a double.
    if !ParseDoubleWithPrefix(characters, number) {
        return false;
    }
    *number = number.clamp(-(f32::MAX as f64), f32::MAX as f64);
    true
}

// cpp: css_parser_fast_paths.cc:146-180
fn ParseSimpleLengthValue(
    property_id: CSSPropertyID,
    string: &[u8],
    css_parser_mode: CSSParserMode,
) -> Option<Rc<Value>> {
    debug_assert!(!string.is_empty());
    let mut accepts_negative_numbers = false;
    if !IsSimpleLengthPropertyID(property_id, &mut accepts_negative_numbers) {
        return None;
    }
    let mut number = 0.0;
    let mut unit = UnitType::kNumber;
    if !ParseSimpleLength(string, &mut unit, &mut number) {
        return None;
    }
    if unit == UnitType::kNumber {
        if css_parser_mode == CSSParserMode::kSVGAttributeMode {
            unit = UnitType::kUserUnits;
        } else if number == 0.0 {
            unit = UnitType::kPixels;
        } else {
            return None;
        }
    }
    if number < 0.0 && !accepts_negative_numbers {
        return None;
    }
    Some(values::numeric(number, unit))
}

// cpp: css_parser_fast_paths.cc:182-229. Returns the length of the angle, or
// 0 if the parse failed.
fn ParseSimpleAngle(characters: &[u8], unit: &mut UnitType, number: &mut f64) -> usize {
    let mut number_length;
    if characters.first() == Some(&b'-') {
        number_length = ParsePositiveDouble(&characters[1..], number);
        if number_length == 0 {
            return number_length;
        }
        number_length += 1;
        *number = -number.min(f32::MAX as f64);
    } else {
        number_length = ParsePositiveDouble(characters, number);
        if number_length == 0 {
            return number_length;
        }
        *number = number.min(f32::MAX as f64);
    }
    let characters = &characters[number_length..];
    let length = characters.len();
    let lower = |i: usize| characters[i] | 0x20;
    if length >= 3 && lower(0) == b'd' && lower(1) == b'e' && lower(2) == b'g' {
        *unit = UnitType::kDegrees;
        number_length + 3
    } else if length >= 4
        && lower(0) == b'g'
        && lower(1) == b'r'
        && lower(2) == b'a'
        && lower(3) == b'd'
    {
        *unit = UnitType::kGradians;
        number_length + 4
    } else if length >= 3 && lower(0) == b'r' && lower(1) == b'a' && lower(2) == b'd' {
        *unit = UnitType::kRadians;
        number_length + 3
    } else if length >= 4
        && lower(0) == b't'
        && lower(1) == b'u'
        && lower(2) == b'r'
        && lower(3) == b'n'
    {
        *unit = UnitType::kTurns;
        number_length + 4
    } else {
        // rotate: only valid for zero (checked by the caller).
        *unit = UnitType::kNumber;
        number_length
    }
}

// cpp: css_parser_fast_paths.cc:231-277
#[inline]
fn IsColorPropertyID(property_id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        property_id,
        kCaretColor
            | kColor
            | kBackgroundColor
            | kBorderBottomColor
            | kBorderLeftColor
            | kBorderRightColor
            | kBorderTopColor
            | kFill
            | kFloodColor
            | kLightingColor
            | kOutlineColor
            | kStopColor
            | kStroke
            | kBorderBlockEndColor
            | kBorderBlockStartColor
            | kBorderInlineEndColor
            | kBorderInlineStartColor
            | kColumnRuleColor
            | kRowRuleColor
            | kTextEmphasisColor
            | kWebkitTextFillColor
            | kWebkitTextStrokeColor
            | kTextDecorationColor
            | kInternalVisitedCaretColor
            | kInternalVisitedColor
            | kInternalVisitedBackgroundColor
            | kInternalVisitedBorderBottomColor
            | kInternalVisitedBorderLeftColor
            | kInternalVisitedBorderRightColor
            | kInternalVisitedBorderTopColor
            | kInternalVisitedFill
            | kInternalVisitedOutlineColor
            | kInternalVisitedStroke
            | kInternalVisitedBorderBlockEndColor
            | kInternalVisitedBorderBlockStartColor
            | kInternalVisitedBorderInlineEndColor
            | kInternalVisitedBorderInlineStartColor
            | kInternalVisitedColumnRuleColor
            | kInternalVisitedTextEmphasisColor
            | kInternalVisitedTextDecorationColor
    )
}

// cpp: css_parser_fast_paths.cc:279-291
// https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk
#[inline]
fn ColorPropertyAllowsQuirkyColor(property_id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        property_id,
        kColor
            | kBackgroundColor
            | kBorderBottomColor
            | kBorderLeftColor
            | kBorderRightColor
            | kBorderTopColor
    )
}

// cpp: css_parser_fast_paths.cc:293-382. The SSE2 / NEON 16-byte block and
// the scalar loop are kept with the vector branch's exact result.
fn FindLengthOfValidDouble(chars: &[u8]) -> usize {
    if chars.is_empty() {
        return 0;
    }
    let mut decimal_mark_seen = false;
    let mut valid_length = 0usize;
    if chars.len() >= 16 {
        let mut is_decimal_bits = 0u16;
        let mut is_mark_bits = 0u16;
        for (i, &c) in chars[..16].iter().enumerate() {
            is_decimal_bits |= u16::from(c.is_ascii_digit()) << i;
            is_mark_bits |= u16::from(c == b'.') << i;
        }
        // Only count the first decimal mark.
        is_mark_bits &= is_mark_bits.wrapping_neg();
        if (is_decimal_bits | is_mark_bits) == 0xffff {
            decimal_mark_seen = is_mark_bits != 0;
            valid_length = 16;
        } else {
            // Drop a stray final period (one not followed by a decimal).
            is_mark_bits &= is_decimal_bits >> 1;
            let accept_bits = is_decimal_bits | is_mark_bits;
            return (!accept_bits).trailing_zeros() as usize;
        }
    }
    while valid_length < chars.len() {
        if !chars[valid_length].is_ascii_digit() {
            if !decimal_mark_seen && chars[valid_length] == b'.' {
                decimal_mark_seen = true;
            } else {
                break;
            }
        }
        valid_length += 1;
    }
    if valid_length > 0 && chars[valid_length - 1] == b'.' {
        return 0;
    }
    valid_length
}

// cpp: css_parser_fast_paths.cc:384-394
fn ContainsCharAtPos(chars: &[u8], pos: usize, ch: u8, also_accept_whitespace: bool) -> bool {
    match chars.get(pos) {
        Some(&c) => c == ch || (also_accept_whitespace && IsHTMLSpace(c)),
        None => false,
    }
}

// cpp: css_parser_fast_paths.cc:396-431. Initial whitespace and a sign are
// accepted; trailing periods and exponents take the full parser.
fn ParseDoubleWithPrefix(chars: &[u8], value: &mut f64) -> bool {
    let mut chars = chars;
    while let Some((&first, rest)) = chars.split_first() {
        if !IsHTMLSpace(first) {
            break;
        }
        chars = rest;
    }
    let Some(&first) = chars.first() else {
        return false;
    };
    if first == b'-' {
        if chars.len() == 1 {
            return false;
        }
        let mut v = 0.0;
        if ParsePositiveDouble(&chars[1..], &mut v) != chars.len() - 1 {
            return false;
        }
        *value = -v;
        return true;
    }
    ParsePositiveDouble(chars, value) == chars.len()
}

// cpp: css_parser_fast_paths.cc:433-574. Digits after the seventh decimal are
// ignored. With at least seven bytes left after the mark Chromium's SSE2 and
// AArch64 NEON branches both compute the same fixed-point fraction scaled by
// kDiv1e7; shorter inputs use the scalar loop.
fn ParsePositiveDouble(chars: &[u8], value: &mut f64) -> usize {
    let length = FindLengthOfValidDouble(chars);
    if length == 0 {
        return 0;
    }
    let mut position = 0usize;
    let mut local_value = 0.0f64;
    while position < length {
        if chars[position] == b'.' {
            break;
        }
        local_value = local_value * 10.0 + (chars[position] - b'0') as f64;
        position += 1;
    }
    position += 1;
    if position >= length {
        *value = local_value;
        return length;
    }
    const kMaxDecimals: usize = 7;
    let bytes_left = length - position;
    let num_decimals = bytes_left.min(kMaxDecimals);
    if chars.len() - position >= 7 {
        // The closest double to 1e-7, rounded up instead of to nearest.
        const kDiv1e7: f64 = 0.000000100000000000000009;
        let mut fraction = 0u32;
        let mut weight = 1_000_000u32;
        for &digit in &chars[position..position + num_decimals] {
            fraction += (digit - b'0') as u32 * weight;
            weight /= 10;
        }
        *value = local_value + fraction as f64 * kDiv1e7;
        return length;
    }
    let mut fraction = 0.0f64;
    let mut scale = 1.0f64;
    for i in 0..num_decimals {
        fraction = fraction * 10.0 + (chars[position + i] - b'0') as f64;
        scale *= 10.0;
    }
    *value = local_value + fraction / scale;
    length
}

// cpp: css_parser_fast_paths.cc:576-621
fn ParseFloatWithMaxValue(
    chars: &mut &[u8],
    max_value: i32,
    value: &mut f64,
    negative: &mut bool,
) -> bool {
    *value = 0.0;
    let mut current = *chars;
    while let Some((&first, rest)) = current.split_first() {
        if !IsHTMLSpace(first) {
            break;
        }
        current = rest;
    }
    if current.first() == Some(&b'-') {
        *negative = true;
        current = &current[1..];
    } else {
        *negative = false;
    }
    if !current.first().is_some_and(u8::is_ascii_digit) {
        return false;
    }
    while let Some(&digit) = current.first().filter(|c| c.is_ascii_digit()) {
        let new_value = *value * 10.0 + (digit - b'0') as f64;
        current = &current[1..];
        if new_value >= max_value as f64 {
            // Clamp values at 255 or 100 (depending on the caller).
            *value = max_value as f64;
            while current.first().is_some_and(u8::is_ascii_digit) {
                current = &current[1..];
            }
            break;
        }
        *value = new_value;
    }
    if current.first() == Some(&b'.') {
        // The integral part is parsed; try to parse the fraction part.
        let mut fractional = 0.0;
        let num_characters_parsed = ParsePositiveDouble(current, &mut fractional);
        if num_characters_parsed == 0 {
            return false;
        }
        current = &current[num_characters_parsed..];
        *value += fractional;
    }
    *chars = current;
    true
}

// cpp: css_parser_fast_paths.cc:623-641
#[derive(Clone, Copy, PartialEq, Eq)]
enum TerminatorStatus {
    // rgb(10 20 30)
    kMustWhitespaceTerminate,
    // rgb(10 , 20,30)
    kMustCharacterTerminate,
    // First element: either variant, decided here.
    kCouldWhitespaceTerminate,
}

// cpp: css_parser_fast_paths.cc:643-678
fn SkipToTerminator(
    chars: &mut &[u8],
    terminator: u8,
    terminator_status: &mut TerminatorStatus,
) -> bool {
    let mut current = *chars;
    while let Some((&first, rest)) = current.split_first() {
        if !IsHTMLSpace(first) {
            break;
        }
        current = rest;
    }
    let skipped_whitespace = current.len() != chars.len();
    let mut status = *terminator_status;
    if status == TerminatorStatus::kCouldWhitespaceTerminate {
        if current.first() == Some(&terminator) {
            *terminator_status = TerminatorStatus::kMustCharacterTerminate;
            *chars = &current[1..];
            return true;
        }
        *terminator_status = TerminatorStatus::kMustWhitespaceTerminate;
        status = TerminatorStatus::kMustWhitespaceTerminate;
    }
    match status {
        TerminatorStatus::kMustWhitespaceTerminate => {
            // At least one space must precede something else (or the end).
            if !skipped_whitespace {
                return false;
            }
        }
        TerminatorStatus::kMustCharacterTerminate => {
            if current.first() != Some(&terminator) {
                return false;
            }
            current = &current[1..];
        }
        TerminatorStatus::kCouldWhitespaceTerminate => unreachable!(),
    }
    *chars = current;
    true
}

// cpp: css_parser_fast_paths.cc:680-725
fn ParseColorNumberOrPercentage(
    chars: &mut &[u8],
    terminator: u8,
    terminator_status: &mut TerminatorStatus,
    expect: &mut UnitType,
    value: &mut i32,
) -> bool {
    let mut current = *chars;
    let mut local_value = 0.0;
    let mut negative = false;
    if !ParseFloatWithMaxValue(&mut current, 255, &mut local_value, &mut negative) {
        return false;
    }
    let Some(&next) = current.first() else {
        return false;
    };
    if *expect == UnitType::kPercentage && next != b'%' {
        return false;
    }
    if *expect == UnitType::kNumber && next == b'%' {
        return false;
    }
    if next == b'%' {
        *expect = UnitType::kPercentage;
        local_value = local_value / 100.0 * 255.0;
        // Clamp values at 255 for percentages over 100%.
        if local_value > 255.0 {
            local_value = 255.0;
        }
        current = &current[1..];
    } else {
        *expect = UnitType::kNumber;
    }
    if !SkipToTerminator(&mut current, terminator, terminator_status) {
        return false;
    }
    // Clamp negative values at zero.
    *value = if negative { 0 } else { local_value.round() as i32 };
    *chars = current;
    true
}

// cpp: css_parser_fast_paths.cc:755-768
fn IsTenthAlpha(chars: &[u8]) -> bool {
    // "0.X"
    if chars.len() == 3 && chars[0] == b'0' && chars[1] == b'.' && chars[2].is_ascii_digit() {
        return true;
    }
    // ".X"
    chars.len() == 2 && chars[0] == b'.' && chars[1].is_ascii_digit()
}

// cpp: css_parser_fast_paths.cc:770-833
fn ParseAlphaValue(chars: &mut &[u8], terminator: u8, value: &mut i32) -> bool {
    while let Some((&first, rest)) = chars.split_first() {
        if !IsHTMLSpace(first) {
            break;
        }
        *chars = rest;
    }
    let mut negative = false;
    if chars.first() == Some(&b'-') {
        negative = true;
        *chars = &chars[1..];
    }
    *value = 0;
    let length = chars.len();
    if length < 2 {
        return false;
    }
    if chars[length - 1] != terminator || !chars[length - 2].is_ascii_digit() {
        return false;
    }
    if chars[0] != b'0' && chars[0] != b'1' && chars[0] != b'.' {
        let double_length = FindLengthOfValidDouble(chars);
        if double_length > 0 && ContainsCharAtPos(chars, double_length, terminator, false) {
            *value = if negative { 0 } else { 255 };
            *chars = &[];
            return true;
        }
        return false;
    }
    if length == 2 && chars[0] != b'.' {
        *value = if !negative && chars[0] == b'1' { 255 } else { 0 };
        *chars = &[];
        return true;
    }
    if IsTenthAlpha(&chars[..length - 1]) {
        // 0.1 alpha steps between 0.0 and 0.9; 0.1 is 26 (25.5 rounded).
        const kTenthAlphaValues: [i32; 10] = [0, 26, 51, 77, 102, 128, 153, 179, 204, 230];
        *value = if negative {
            0
        } else {
            kTenthAlphaValues[(chars[length - 2] - b'0') as usize]
        };
        *chars = &[];
        return true;
    }
    let mut alpha = 0.0;
    let dbl_length = ParsePositiveDouble(chars, &mut alpha);
    if dbl_length == 0 || !ContainsCharAtPos(chars, dbl_length, terminator, false) {
        return false;
    }
    *value = if negative {
        0
    } else {
        (alpha.min(1.0) * 255.0).round() as i32
    };
    *chars = &[];
    true
}

// cpp: css_parser_fast_paths.cc:891-900
#[inline]
fn MightBeRGBOrRGBA(chars: &[u8]) -> bool {
    chars.len() >= 5
        && &chars[..3] == b"rgb"
        && (chars[3] == b'(' || (chars[3] == b'a' && chars[4] == b'('))
}

// cpp: platform/graphics/color.cc:81-124 ParseHexColorInternal
// (CSSHexAlphaColor is enabled).
fn ParseHexColor(name: &[u8], color: &mut Color) -> bool {
    if !matches!(name.len(), 3 | 4 | 6 | 8) {
        return false;
    }
    let mut value = 0u32;
    for &digit in name {
        let Some(nibble) = (digit as char).to_digit(16) else {
            return false;
        };
        value = (value << 4) | nibble;
    }
    *color = Color::FromRGBA32(match name.len() {
        6 => 0xFF000000 | value,
        // RGBA order parsed; RGBA32 is ARGB, so rotate right eight bits.
        8 => value << 24 | value >> 8,
        // #abcd converts to ddaabbcc.
        4 => {
            (value & 0xF) << 28
                | (value & 0xF) << 24
                | (value & 0xF000) << 8
                | (value & 0xF000) << 4
                | (value & 0xF00) << 4
                | (value & 0xF00)
                | (value & 0xF0)
                | (value & 0xF0) >> 4
        }
        // #abc converts to #aabbcc.
        _ => {
            0xFF000000
                | (value & 0xF00) << 12
                | (value & 0xF00) << 8
                | (value & 0xF0) << 8
                | (value & 0xF0) << 4
                | (value & 0xF) << 4
                | (value & 0xF)
        }
    });
    true
}

// cpp: css_parser_fast_paths.cc:913-1092 FastParseColorInternal. The hsl()
// branch is not translated; hsl()/hsla() strings take the full parser.
fn FastParseColorInternal(color: &mut Color, chars: &[u8], quirks_mode: bool) -> bool {
    let length = chars.len();
    if length >= 4 && chars[0] == b'#' {
        return ParseHexColor(&chars[1..], color);
    }
    if quirks_mode && (length == 3 || length == 6) && ParseHexColor(chars, color) {
        return true;
    }
    // rgb() and rgba() have the same syntax.
    if MightBeRGBOrRGBA(chars) {
        let mut current = &chars[if chars[3] == b'a' { 5 } else { 4 }..];
        let (mut red, mut green, mut blue, mut alpha) = (0, 0, 0, 0);
        let mut should_have_alpha = false;
        let mut terminator_status = TerminatorStatus::kCouldWhitespaceTerminate;
        let mut expect = UnitType::kUnknown;
        if !ParseColorNumberOrPercentage(
            &mut current,
            b',',
            &mut terminator_status,
            &mut expect,
            &mut red,
        ) {
            return false;
        }
        if !ParseColorNumberOrPercentage(
            &mut current,
            b',',
            &mut terminator_status,
            &mut expect,
            &mut green,
        ) {
            return false;
        }
        let mut no_whitespace_check = TerminatorStatus::kMustCharacterTerminate;
        if !ParseColorNumberOrPercentage(
            &mut current,
            b',',
            &mut no_whitespace_check,
            &mut expect,
            &mut blue,
        ) {
            // Might have slash as separator.
            if ParseColorNumberOrPercentage(
                &mut current,
                b'/',
                &mut no_whitespace_check,
                &mut expect,
                &mut blue,
            ) {
                if terminator_status != TerminatorStatus::kMustWhitespaceTerminate {
                    return false;
                }
                should_have_alpha = true;
            }
            // Might not have alpha.
            else if !ParseColorNumberOrPercentage(
                &mut current,
                b')',
                &mut no_whitespace_check,
                &mut expect,
                &mut blue,
            ) {
                return false;
            }
        } else {
            if terminator_status != TerminatorStatus::kMustCharacterTerminate {
                return false;
            }
            should_have_alpha = true;
        }
        if should_have_alpha {
            if !ParseAlphaValue(&mut current, b')', &mut alpha) {
                return false;
            }
            *color = Color::FromRGBA(red, green, blue, alpha);
        } else {
            if !current.is_empty() {
                return false;
            }
            *color = Color::FromRGB(red, green, blue);
        }
        return true;
    }
    false
}

// cpp: style_color.cc:528-551 StyleColor::IsColorKeyword.
#[inline]
fn IsColorKeyword(id: CSSValueID) -> bool {
    (CSSValueID::kAqua as i32..=CSSValueID::kInternalCurrentSearchTextColor as i32)
        .contains(&(id as i32))
        || (CSSValueID::kAliceblue as i32..=CSSValueID::kYellowgreen as i32).contains(&(id as i32))
        || id == CSSValueID::kMenu
}

enum ParseColorResult {
    kColor(Color),
    kKeyword(CSSValueID),
    kFailure,
}

// cpp: css_parser_fast_paths.cc:1094-1124. CSSAccentColorKeyword is disabled
// in this build (ConsumeColor rejects it too).
fn ParseColor(
    property_id: CSSPropertyID,
    string: &StringView,
    chars: &[u8],
    parser_mode: CSSParserMode,
) -> ParseColorResult {
    debug_assert!(!chars.is_empty());
    debug_assert!(IsColorPropertyID(property_id));
    let value_id = CssValueKeywordID(string);
    if matches!(
        value_id,
        CSSValueID::kAccentcolor | CSSValueID::kAccentcolortext
    ) {
        return ParseColorResult::kFailure;
    }
    if IsColorKeyword(value_id) {
        if !IsValueAllowedInMode(value_id, parser_mode) {
            return ParseColorResult::kFailure;
        }
        return ParseColorResult::kKeyword(value_id);
    }
    let quirks_mode =
        IsQuirksModeBehavior(parser_mode) && ColorPropertyAllowsQuirkyColor(property_id);
    let mut color = Color::default();
    if FastParseColorInternal(&mut color, chars, quirks_mode) {
        ParseColorResult::kColor(color)
    } else {
        ParseColorResult::kFailure
    }
}

// cpp: css_parser_fast_paths.cc:1999-2022
fn ParseTransformTranslateArguments(
    chars: &mut &[u8],
    mut expected_count: u32,
    arguments: &mut Vec<Rc<Value>>,
) -> bool {
    while expected_count != 0 {
        let terminator = if expected_count == 1 { b')' } else { b',' };
        let Some(delimiter) = chars.iter().position(|&c| c == terminator) else {
            return false;
        };
        let mut unit = UnitType::kNumber;
        let mut number = 0.0;
        if !ParseSimpleLength(&chars[..delimiter], &mut unit, &mut number) {
            return false;
        }
        if unit != UnitType::kPixels && (number != 0.0 || unit != UnitType::kNumber) {
            return false;
        }
        arguments.push(values::numeric(number, UnitType::kPixels));
        *chars = &chars[delimiter + 1..];
        expected_count -= 1;
    }
    true
}

// cpp: css_parser_fast_paths.cc:2024-2047
fn ParseTransformRotateArgument(chars: &mut &[u8], arguments: &mut Vec<Rc<Value>>) -> bool {
    let Some(delimiter) = chars.iter().position(|&c| c == b')') else {
        return false;
    };
    let mut unit = UnitType::kNumber;
    let mut number = 0.0;
    if ParseSimpleAngle(&chars[..delimiter], &mut unit, &mut number) != delimiter {
        return false;
    }
    if unit == UnitType::kNumber {
        if number != 0.0 {
            return false;
        }
        // Matches ConsumeNumericLiteralAngle().
        unit = UnitType::kDegrees;
    }
    arguments.push(values::numeric(number, unit));
    *chars = &chars[delimiter + 1..];
    true
}

// cpp: css_parser_fast_paths.cc:2049-2067
fn ParseTransformNumberArguments(
    chars: &mut &[u8],
    mut expected_count: u32,
    arguments: &mut Vec<Rc<Value>>,
) -> bool {
    while expected_count != 0 {
        let terminator = if expected_count == 1 { b')' } else { b',' };
        let Some(delimiter) = chars.iter().position(|&c| c == terminator) else {
            return false;
        };
        let mut number = 0.0;
        if !ParseDoubleWithPrefix(&chars[..delimiter], &mut number) {
            return false;
        }
        arguments.push(values::numeric(number, UnitType::kNumber));
        *chars = &chars[delimiter + 1..];
        expected_count -= 1;
    }
    true
}

// cpp: css_parser_fast_paths.cc:2069
const kShortestValidTransformStringLength: usize = 12;

// cpp: css_parser_fast_paths.cc:2071-2156
fn ParseSimpleTransformValue(chars: &mut &[u8]) -> Option<Rc<Value>> {
    if chars.len() < kShortestValidTransformStringLength {
        return None;
    }
    let mut arguments = Vec::new();
    if chars.starts_with(b"translate") {
        let (transform_type, expected_argument_count, argument_start) =
            if (chars[9] | 0x20) == b'x' && chars[10] == b'(' {
                (CSSValueID::kTranslateX, 1, 11)
            } else if (chars[9] | 0x20) == b'y' && chars[10] == b'(' {
                (CSSValueID::kTranslateY, 1, 11)
            } else if (chars[9] | 0x20) == b'z' && chars[10] == b'(' {
                (CSSValueID::kTranslateZ, 1, 11)
            } else if chars[9] == b'(' {
                (CSSValueID::kTranslate, 2, 10)
            } else if chars[9] == b'3' && chars[10] == b'd' && chars[11] == b'(' {
                (CSSValueID::kTranslate3d, 3, 12)
            } else {
                return None;
            };
        *chars = &chars[argument_start..];
        if !ParseTransformTranslateArguments(chars, expected_argument_count, &mut arguments) {
            return None;
        }
        return Some(values::function(transform_type, arguments));
    }
    if chars.starts_with(b"matrix3d(") {
        *chars = &chars[9..];
        if !ParseTransformNumberArguments(chars, 16, &mut arguments) {
            return None;
        }
        return Some(values::function(CSSValueID::kMatrix3d, arguments));
    }
    if chars.starts_with(b"scale3d(") {
        *chars = &chars[8..];
        if !ParseTransformNumberArguments(chars, 3, &mut arguments) {
            return None;
        }
        return Some(values::function(CSSValueID::kScale3d, arguments));
    }
    if chars.starts_with(b"rotate") {
        let rotate_value_id = if chars[6] == b'(' {
            *chars = &chars[7..];
            CSSValueID::kRotate
        } else if (chars[6] | 0x20) == b'z' && chars[7] == b'(' {
            *chars = &chars[8..];
            CSSValueID::kRotateZ
        } else {
            return None;
        };
        if !ParseTransformRotateArgument(chars, &mut arguments) {
            return None;
        }
        return Some(values::function(rotate_value_id, arguments));
    }
    None
}

// cpp: css_parser_fast_paths.cc:2158-2215. A quick scan that rejects most
// transforms the fast path cannot take before allocating any value.
fn TransformCanLikelyUseFastPath(chars: &[u8]) -> bool {
    let length = chars.len();
    let mut i = 0usize;
    while i < length {
        if chars[i] == b' ' {
            i += 1;
            continue;
        }
        if length - i < kShortestValidTransformStringLength {
            return false;
        }
        match chars[i] {
            // translate, translateX, translateY, translateZ, translate3d.
            b't' => {
                if chars[i + 8] != b'e' {
                    return false;
                }
                i += 9;
            }
            // matrix3d.
            b'm' => {
                if chars[i + 7] != b'd' {
                    return false;
                }
                i += 8;
            }
            // scale3d.
            b's' => {
                if chars[i + 6] != b'd' {
                    return false;
                }
                i += 7;
            }
            // rotate.
            b'r' => {
                if chars[i + 5] != b'e' {
                    return false;
                }
                i += 6;
            }
            // All other things, ex. skew.
            _ => return false,
        }
        let Some(arguments_end) = chars[i..].iter().position(|&c| c == b')') else {
            return false;
        };
        // Advance to the end of the arguments.
        i += arguments_end + 1;
    }
    i == length
}

// cpp: css_parser_fast_paths.cc:2217-2247
fn ParseSimpleTransform(property_id: CSSPropertyID, chars: &[u8]) -> Option<Rc<Value>> {
    debug_assert!(!chars.is_empty());
    if property_id != CSSPropertyID::kTransform {
        return None;
    }
    if !TransformCanLikelyUseFastPath(chars) {
        return None;
    }
    let mut chars = chars;
    let mut transform_list = Vec::new();
    while !chars.is_empty() {
        while chars.first() == Some(&b' ') {
            chars = &chars[1..];
        }
        if chars.is_empty() {
            break;
        }
        transform_list.push(ParseSimpleTransformValue(&mut chars)?);
    }
    // An all-space string reaches here with no function; Chromium returns
    // its null list, so the full parser decides.
    if transform_list.is_empty() {
        return None;
    }
    Some(values::list(transform_list, values::ListSeparator::Space))
}

/// CSSParserFastPaths::MaybeParseValue (css_parser_fast_paths.cc:2249-2283)
/// for a resolved longhand. `mode` is the parser context's mode.
pub fn MaybeParseValue(
    property_id: CSSPropertyID,
    string: &StringView,
    mode: CSSParserMode,
) -> Option<Rc<Value>> {
    // Non-ASCII strings never match these fast paths.
    let chars = string.Span8()?;
    if chars.is_empty() {
        return None;
    }
    // The full parser rejects a property hidden by its exposure gate before
    // any consumer runs; the fast path must not accept it either.
    if crate::production_property_parser::ExposureError(property_id, mode).is_some() {
        return None;
    }
    if let Some(length) = ParseSimpleLengthValue(property_id, chars, mode) {
        return Some(length);
    }
    if IsColorPropertyID(property_id) {
        match ParseColor(property_id, string, chars, mode) {
            ParseColorResult::kFailure => {}
            ParseColorResult::kKeyword(id) => return Some(values::identifier(id)),
            ParseColorResult::kColor(color) => return Some(values::color(color)),
        }
    }
    ParseSimpleTransform(property_id, chars)
}
