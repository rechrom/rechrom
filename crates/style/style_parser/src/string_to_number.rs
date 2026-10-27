// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.h
// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc

#![allow(non_camel_case_types)]

use super::number_parsing_options::NumberParsingOptions;
use foundation::{unicode::IsSpaceOrNewline, StringView};

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.h:16-24
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberParsingResult {
    kSuccess,
    kError,
    kOverflowMin,
    kOverflowMax,
}

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:18-29
fn IsCharacterAllowedInBase<const BASE: u32>(c: u16) -> bool {
    match BASE {
        10 => (b'0' as u16..=b'9' as u16).contains(&c),
        16 => {
            (b'0' as u16..=b'9' as u16).contains(&c)
                || (b'a' as u16..=b'f' as u16).contains(&c)
                || (b'A' as u16..=b'F' as u16).contains(&c)
        }
        _ => unreachable!("Chromium defines only base 10 and 16 specializations"),
    }
}

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:35-42
// Integral types instantiated by the source use at most 64 bits. i128 keeps
// every intermediate exact while preserving each target type's source bounds.
trait IntegralType: Copy {
    const MAX: i128;
    const MIN: i128;
    const IS_SIGNED: bool;
    fn FromCheckedI128(value: i128) -> Self;
}

macro_rules! impl_integral_type {
    ($($ty:ty),+) => { $(
        impl IntegralType for $ty {
            const MAX: i128 = <$ty>::MAX as i128;
            const MIN: i128 = <$ty>::MIN as i128;
            const IS_SIGNED: bool = <$ty>::MIN != 0;
            fn FromCheckedI128(value: i128) -> Self {
                debug_assert!((<Self as IntegralType>::MIN..=<Self as IntegralType>::MAX).contains(&value));
                value as $ty
            }
        }
    )+ };
}
impl_integral_type!(i32, u32, i64, u64);

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:31-144
fn ToIntegralType<Integral: IntegralType, const BASE: u32, CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> (Integral, NumberParsingResult) {
    let mut index = 0;
    let mut length = data.len();
    let mut value: i128 = 0;
    let mut result = NumberParsingResult::kError;
    let mut is_negative = false;
    let mut overflow = false;
    let accept_minus = Integral::IS_SIGNED || options.AcceptMinusZeroForUnsigned();
    // A Rust slice cannot carry a null data pointer. Empty slices take the
    // same error return as either a null span or an empty non-null source span.
    if options.AcceptWhitespace() {
        while length != 0 && IsSpaceOrNewline(data[index].into()) {
            length -= 1;
            index += 1;
        }
    }
    if accept_minus && length != 0 && data[index].into() == b'-' as u16 {
        length -= 1;
        index += 1;
        is_negative = true;
    } else if length != 0 && options.AcceptLeadingPlus() && data[index].into() == b'+' as u16 {
        length -= 1;
        index += 1;
    }
    if length == 0 || !IsCharacterAllowedInBase::<BASE>(data[index].into()) {
        return (Integral::FromCheckedI128(0), result);
    }
    let base = i128::from(BASE);
    while length != 0 && IsCharacterAllowedInBase::<BASE>(data[index].into()) {
        length -= 1;
        let c = data[index].into();
        let digit_value = if (b'0' as u16..=b'9' as u16).contains(&c) {
            i128::from(c - b'0' as u16)
        } else if c >= b'a' as u16 {
            i128::from(c - b'a' as u16 + 10)
        } else {
            i128::from(c - b'A' as u16 + 10)
        };
        if is_negative {
            if !Integral::IS_SIGNED && options.AcceptMinusZeroForUnsigned() {
                if digit_value != 0 {
                    result = NumberParsingResult::kError;
                    overflow = true;
                }
            } else if value < (Integral::MIN + digit_value) / base {
                result = NumberParsingResult::kOverflowMin;
                overflow = true;
            }
        } else if value > (Integral::MAX - digit_value) / base {
            result = NumberParsingResult::kOverflowMax;
            overflow = true;
        }
        if !overflow {
            value = if is_negative {
                base * value - digit_value
            } else {
                base * value + digit_value
            };
        }
        index += 1;
    }
    if options.AcceptWhitespace() {
        while length != 0 && IsSpaceOrNewline(data[index].into()) {
            length -= 1;
            index += 1;
        }
    }
    if length == 0 || options.AcceptTrailingGarbage() {
        if !overflow {
            result = NumberParsingResult::kSuccess;
        }
    } else {
        // Even if we detected overflow, return kError for trailing garbage.
        result = NumberParsingResult::kError;
    }
    (
        Integral::FromCheckedI128(if result == NumberParsingResult::kSuccess {
            value
        } else {
            0
        }),
        result,
    )
}

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:146-154
fn ToIntegralTypeOptional<
    Integral: IntegralType,
    const BASE: u32,
    CharacterType: Copy + Into<u16>,
>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<Integral> {
    let (value, result) = ToIntegralType::<Integral, BASE, CharacterType>(data, options);
    (result == NumberParsingResult::kSuccess).then_some(value)
}

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.h:33-36
// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:168-176
pub fn HexCharactersToUInt<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<u32> {
    ToIntegralTypeOptional::<u32, 16, CharacterType>(data, options)
}

// cpp: string_to_number.cc:156-166
pub fn CharactersToUIntWithResult<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> (u32, NumberParsingResult) {
    ToIntegralType::<u32, 10, CharacterType>(data, options)
}

// cpp: string_to_number.h:27-67
// cpp: string_to_number.cc:178-211
pub fn HexCharactersToUInt64<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<u64> {
    ToIntegralTypeOptional::<u64, 16, CharacterType>(data, options)
}

pub fn CharactersToInt<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<i32> {
    ToIntegralTypeOptional::<i32, 10, CharacterType>(data, options)
}

pub fn CharactersToUInt<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<u32> {
    ToIntegralTypeOptional::<u32, 10, CharacterType>(data, options)
}

pub fn CharactersToInt64<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<i64> {
    ToIntegralTypeOptional::<i64, 10, CharacterType>(data, options)
}

pub fn CharactersToUInt64<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    options: NumberParsingOptions,
) -> Option<u64> {
    ToIntegralTypeOptional::<u64, 10, CharacterType>(data, options)
}

fn VisitStringView<Output>(
    input: &StringView,
    visit8: impl FnOnce(&[u8]) -> Output,
    visit16: impl FnOnce(&[u16]) -> Output,
) -> Output {
    if input.Is8Bit() {
        visit8(input.Span8().expect("8-bit StringView"))
    } else {
        visit16(input.Span16())
    }
}

// cpp: string_to_number.h:119-175
// cpp: string_to_number.cc:293-355
pub fn StringToInt(input: &StringView, options: NumberParsingOptions) -> Option<i32> {
    VisitStringView(
        input,
        |chars| CharactersToInt(chars, options),
        |chars| CharactersToInt(chars, options),
    )
}

pub fn StringToUint(input: &StringView, options: NumberParsingOptions) -> Option<u32> {
    VisitStringView(
        input,
        |chars| CharactersToUInt(chars, options),
        |chars| CharactersToUInt(chars, options),
    )
}

pub fn StringToInt64(input: &StringView, options: NumberParsingOptions) -> Option<i64> {
    VisitStringView(
        input,
        |chars| CharactersToInt64(chars, options),
        |chars| CharactersToInt64(chars, options),
    )
}

pub fn StringToUint64(input: &StringView, options: NumberParsingOptions) -> Option<u64> {
    VisitStringView(
        input,
        |chars| CharactersToUInt64(chars, options),
        |chars| CharactersToUInt64(chars, options),
    )
}

// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.h:150-151
// cpp: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:329-333
pub fn HexStringToUint(input: &StringView, options: NumberParsingOptions) -> Option<u32> {
    if input.Is8Bit() {
        HexCharactersToUInt(input.Span8().expect("8-bit StringView"), options)
    } else {
        HexCharactersToUInt(input.Span16(), options)
    }
}

pub fn HexStringToUint64(input: &StringView, options: NumberParsingOptions) -> Option<u64> {
    VisitStringView(
        input,
        |chars| HexCharactersToUInt64(chars, options),
        |chars| HexCharactersToUInt64(chars, options),
    )
}

pub fn StringToIntStrict(input: &StringView) -> Option<i32> {
    StringToInt(input, NumberParsingOptions::Strict())
}

pub fn StringToUintStrict(input: &StringView) -> Option<u32> {
    StringToUint(input, NumberParsingOptions::Strict())
}

pub fn StringToIntLoose(input: &StringView) -> Option<i32> {
    StringToInt(input, NumberParsingOptions::Loose())
}

pub fn StringToUintLoose(input: &StringView) -> Option<u32> {
    StringToUint(input, NumberParsingOptions::Loose())
}

// cpp: string_to_number.cc:213-291
const fn IsAsciiSpace(c: u16) -> bool {
    c == b' ' as u16 || (c >= b'\t' as u16 && c <= b'\r' as u16)
}

// This is the accepted numeric grammar of WTF ParseDouble's configured
// StringToDoubleConverter. Scanning first is essential: Rust's parser also
// accepts spellings such as NaN and inf, which Chromium rejects here.
fn ParseDoublePrefix<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
) -> Option<(f64, usize)> {
    let mut index = 0;
    if data
        .get(index)
        .is_some_and(|c| matches!((*c).into(), x if x == b'+' as u16 || x == b'-' as u16))
    {
        index += 1;
    }

    let integer_start = index;
    while data
        .get(index)
        .is_some_and(|c| (b'0' as u16..=b'9' as u16).contains(&(*c).into()))
    {
        index += 1;
    }
    let mut digit_count = index - integer_start;

    if data.get(index).is_some_and(|c| (*c).into() == b'.' as u16) {
        index += 1;
        let fraction_start = index;
        while data
            .get(index)
            .is_some_and(|c| (b'0' as u16..=b'9' as u16).contains(&(*c).into()))
        {
            index += 1;
        }
        digit_count += index - fraction_start;
    }

    if digit_count == 0 {
        return None;
    }

    let exponent_marker = index;
    if data.get(index).is_some_and(|c| {
        let c = (*c).into();
        c == b'e' as u16 || c == b'E' as u16
    }) {
        index += 1;
        if data.get(index).is_some_and(|c| {
            let c = (*c).into();
            c == b'+' as u16 || c == b'-' as u16
        }) {
            index += 1;
        }
        let exponent_start = index;
        while data
            .get(index)
            .is_some_and(|c| (b'0' as u16..=b'9' as u16).contains(&(*c).into()))
        {
            index += 1;
        }
        if exponent_start == index {
            index = exponent_marker;
        }
    }

    let mut ascii = Vec::with_capacity(index);
    for character in &data[..index] {
        let value = (*character).into();
        debug_assert!(value <= 0x7f);
        ascii.push(value as u8);
    }
    let spelling = std::str::from_utf8(&ascii).expect("CSS number grammar is ASCII");
    let number = spelling
        .parse::<f64>()
        .expect("scanned CSS decimal must be accepted as an IEEE double");
    Some((number, index))
}

fn ToDoubleType<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    allow_trailing_junk: bool,
) -> Option<(f64, usize)> {
    let leading_spaces = data
        .iter()
        .take_while(|character| IsAsciiSpace((**character).into()))
        .count();
    let (number, parsed) = ParseDoublePrefix(&data[leading_spaces..])?;
    let parsed_length = leading_spaces + parsed;
    if !allow_trailing_junk && parsed_length != data.len() {
        return None;
    }
    Some((number, parsed_length))
}

pub fn CharactersToDouble<CharacterType: Copy + Into<u16>>(data: &[CharacterType]) -> Option<f64> {
    ToDoubleType(data, false).map(|(value, _)| value)
}

pub fn CharactersToDoubleWithParsedLength<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    parsed_length: &mut usize,
) -> f64 {
    match ToDoubleType(data, true) {
        Some((value, length)) => {
            *parsed_length = length;
            value
        }
        None => {
            *parsed_length = 0;
            0.0
        }
    }
}

pub fn CharactersToFloat<CharacterType: Copy + Into<u16>>(data: &[CharacterType]) -> Option<f32> {
    CharactersToDouble(data).map(|value| value as f32)
}

pub fn CharactersToFloatWithParsedLength<CharacterType: Copy + Into<u16>>(
    data: &[CharacterType],
    parsed_length: &mut usize,
) -> f32 {
    CharactersToDoubleWithParsedLength(data, parsed_length) as f32
}

pub fn StringToDouble(input: &StringView) -> Option<f64> {
    VisitStringView(input, CharactersToDouble, CharactersToDouble)
}

pub fn StringToFloat(input: &StringView) -> Option<f32> {
    VisitStringView(input, CharactersToFloat, CharactersToFloat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::String;

    #[test]
    fn strict_hex_boundaries_and_string_widths() {
        for (text, expected) in [
            ("ffffff", Some(0x00ff_ffff)),
            ("FFFFFFFF", Some(u32::MAX)),
            ("100000000", None),
            ("0000000000ffffffff", Some(u32::MAX)),
            ("+aF", Some(175)),
            ("0xff", None),
            ("-0", None),
            ("", None),
            ("fZ", None),
        ] {
            assert_eq!(
                HexStringToUint(&StringView::from(text), NumberParsingOptions::Strict()),
                expected,
                "{text}"
            );
            let latin1 = String::from_latin1(text.as_bytes());
            assert_eq!(
                HexStringToUint(&StringView::from(&latin1), NumberParsingOptions::Strict()),
                expected,
                "{text}"
            );
        }
        assert_eq!(
            HexStringToUint(
                &StringView::from("\u{2003}+f\u{2003}"),
                NumberParsingOptions::Strict()
            ),
            Some(15)
        );
        assert_eq!(
            HexStringToUint(&StringView::from("\u{a0}f"), NumberParsingOptions::Strict()),
            None
        );
    }

    #[test]
    fn overflow_result_and_trailing_garbage_precedence() {
        let strict = NumberParsingOptions::Strict();
        assert_eq!(
            ToIntegralType::<u32, 16, u8>(b"100000000", strict),
            (0, NumberParsingResult::kOverflowMax)
        );
        assert_eq!(
            ToIntegralType::<u32, 16, u8>(b"100000000Z", strict),
            (0, NumberParsingResult::kError)
        );
        assert_eq!(
            ToIntegralType::<u32, 16, u8>(b"100000000Z", NumberParsingOptions::Loose()),
            (0, NumberParsingResult::kOverflowMax)
        );
        assert_eq!(
            ToIntegralType::<i32, 10, u8>(b"-2147483648", strict),
            (i32::MIN, NumberParsingResult::kSuccess)
        );
        assert_eq!(
            ToIntegralType::<i32, 10, u8>(b"-2147483649", strict),
            (0, NumberParsingResult::kOverflowMin)
        );
        assert_eq!(
            ToIntegralType::<u64, 16, u8>(b"ffffffffffffffff", strict),
            (u64::MAX, NumberParsingResult::kSuccess)
        );
        assert_eq!(
            ToIntegralType::<i64, 16, u8>(b"-8000000000000000", strict),
            (i64::MIN, NumberParsingResult::kSuccess)
        );
    }

    #[test]
    fn unsigned_minus_zero_and_options_are_immutable() {
        let original = NumberParsingOptions::new();
        let minus_zero = original.SetAcceptMinusZeroForUnsigned();
        assert!(!original.AcceptMinusZeroForUnsigned());
        assert_eq!(HexCharactersToUInt(b"-000", minus_zero), Some(0));
        assert_eq!(HexCharactersToUInt(b"-001", minus_zero), None);
        assert_eq!(HexCharactersToUInt(b"+0", minus_zero), None);
        assert_eq!(
            HexCharactersToUInt(b"fZ", NumberParsingOptions::Loose()),
            Some(15)
        );
    }

    #[test]
    fn all_integral_entry_points_preserve_width_and_options() {
        let strict = NumberParsingOptions::Strict();
        assert_eq!(CharactersToInt(b" -2147483648 ", strict), Some(i32::MIN));
        assert_eq!(CharactersToUInt(b" +4294967295 ", strict), Some(u32::MAX));
        assert_eq!(
            CharactersToInt64(b"-9223372036854775808", strict),
            Some(i64::MIN)
        );
        assert_eq!(
            CharactersToUInt64(b"18446744073709551615", strict),
            Some(u64::MAX)
        );
        assert_eq!(
            HexCharactersToUInt64(b"ffffffffffffffff", strict),
            Some(u64::MAX)
        );
        assert_eq!(HexCharactersToUInt64(b"10000000000000000", strict), None);
        assert_eq!(
            CharactersToUIntWithResult(b"4294967296", strict),
            (0, NumberParsingResult::kOverflowMax)
        );

        let wide = String::from_utf16(&" +42 ".encode_utf16().collect::<Vec<_>>());
        let wide_view = StringView::from(&wide);
        assert_eq!(StringToIntStrict(&wide_view), Some(42));
        assert_eq!(StringToUintStrict(&wide_view), Some(42));
        assert_eq!(StringToIntLoose(&StringView::from("-19junk")), Some(-19));
        assert_eq!(StringToUintLoose(&StringView::from("19junk")), Some(19));
        assert_eq!(StringToInt64(&StringView::from("-9"), strict), Some(-9));
        assert_eq!(StringToUint64(&StringView::from("9"), strict), Some(9));
        assert_eq!(
            HexStringToUint64(&StringView::from("abcdef"), strict),
            Some(0xabcdef)
        );
    }

    #[test]
    fn floating_point_entry_points_match_css_number_and_parsed_length_rules() {
        for (text, expected) in [
            ("+.5", Some(0.5)),
            ("3.", Some(3.0)),
            (" -1.25e+2", Some(-125.0)),
            ("1e999", Some(f64::INFINITY)),
            ("NaN", None),
            ("Infinity", None),
            ("1 ", None),
            (".", None),
        ] {
            let actual = CharactersToDouble(text.as_bytes());
            match (actual, expected) {
                (Some(actual), Some(expected)) if expected.is_infinite() => {
                    assert!(actual.is_infinite() && actual.is_sign_positive(), "{text}");
                }
                _ => assert_eq!(actual, expected, "{text}"),
            }
        }

        let mut parsed = usize::MAX;
        assert_eq!(
            CharactersToDoubleWithParsedLength(b" \t1.5e2tail", &mut parsed),
            150.0
        );
        assert_eq!(parsed, 7);
        assert_eq!(
            CharactersToDoubleWithParsedLength(b"1e+tail", &mut parsed),
            1.0
        );
        assert_eq!(parsed, 1);

        let wide = String::from_utf16(&".25".encode_utf16().collect::<Vec<_>>());
        assert_eq!(StringToDouble(&StringView::from(&wide)), Some(0.25));
        assert_eq!(StringToFloat(&StringView::from("2.5")), Some(2.5));
    }
}
