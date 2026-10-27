// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_token.h:24-170,175,179-201,217-313,318
// cpp: third_party/blink/renderer/core/css/parser/css_parser_token.cc:24-103,111-194,308-350
// cpp: third_party/blink/renderer/core/css/parser/css_property_parser.cc:257-297,384-418
// Pending: .h:172-174/.cc:104-109 property lookup (ExecutionContext/UnresolvedCSSPropertyID),
// .h:177/.cc:196-305 Serialize (css_markup, StringBuilder, double conversion),
// .h:203-214 logging (Serialize). C++ packed layout is replaced by safe storage.

#![allow(non_camel_case_types, non_snake_case)]

use super::at_rule_descriptors::{AsAtRuleDescriptorID, AtRuleDescriptorID};
use super::css_property_parser::CssValueKeywordID;
use crate::css_primitive_value::{StringToUnitType, UnitType};
use foundation::{CSSValueID, String, StringView};
use std::cell::Cell;

// cpp: css_parser_token.h:24-78
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSParserTokenType {
    kIdentToken = 0,
    kFunctionToken,
    kAtKeywordToken,
    kHashToken,
    kUrlToken,
    kBadUrlToken,
    kDelimiterToken,
    kNumberToken,
    kPercentageToken,
    kDimensionToken,
    kIncludeMatchToken,
    kDashMatchToken,
    kPrefixMatchToken,
    kSuffixMatchToken,
    kSubstringMatchToken,
    kColumnToken,
    kUnicodeRangeToken,
    kWhitespaceToken,
    kCDOToken,
    kCDCToken,
    kColonToken,
    kSemicolonToken,
    kCommaToken,
    kLeftParenthesisToken,
    kRightParenthesisToken,
    kLeftBracketToken,
    kRightBracketToken,
    kLeftBraceToken,
    kRightBraceToken,
    kStringToken,
    kBadStringToken,
    kEOFToken,
    kCommentToken,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericSign {
    kNoSign,
    kPlusSign,
    kMinusSign,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericValueType {
    kIntegerValueType,
    kNumberValueType,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashTokenType {
    kHashTokenId,
    kHashTokenUnrestricted,
}

// cpp: css_parser_token.h:82-86
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockType {
    kNotBlock,
    kBlockStart,
    kBlockEnd,
}

// cpp: css_parser_token.h:217-264,274-292
// Eight-byte Latin-1 strings really live inline; longer Latin-1 and all UTF-16
// values retain StringView backing. Safe shared ownership replaces raw pointers.
#[derive(Clone, Debug)]
enum TokenString {
    Inline8 { bytes: [u8; 8], length: u8 },
    Backing(StringView),
    Empty { is_8bit: bool },
}

impl TokenString {
    fn FromStringView(value: StringView) -> Self {
        if let Some(span) = value.Span8() {
            if span.len() <= 8 {
                let mut bytes = [0; 8];
                bytes[..span.len()].copy_from_slice(span);
                return Self::Inline8 {
                    bytes,
                    length: span.len() as u8,
                };
            }
        }
        Self::Backing(value)
    }
    fn Value(&self) -> StringView {
        match self {
            Self::Inline8 { bytes, length } => {
                StringView::from(&String::from_latin1(&bytes[..*length as usize]))
            }
            Self::Backing(value) => value.clone(),
            Self::Empty { is_8bit: true } => StringView::from(&String::from_latin1(&[])),
            Self::Empty { is_8bit: false } => StringView::from(&String::from_utf16(&[])),
        }
    }
    fn IsInline(&self) -> bool {
        matches!(self, Self::Inline8 { .. })
    }
    fn Is8Bit(&self) -> bool {
        match self {
            Self::Inline8 { .. } => true,
            Self::Backing(value) => value.Is8Bit(),
            Self::Empty { is_8bit } => *is_8bit,
        }
    }
    fn WithoutValue(&self) -> Self {
        Self::Empty {
            is_8bit: self.Is8Bit(),
        }
    }
}

// cpp: css_parser_token.h:266-310
// Source union alternatives use independent safe fields. Values marked "don't
// care" by the source are initialized but are never read for unrelated types.
#[derive(Clone, Debug)]
pub struct CSSParserToken {
    token_type: CSSParserTokenType,
    block_type: BlockType,
    numeric_value_type: NumericValueType,
    numeric_sign: NumericSign,
    unit: UnitType,
    value: TokenString,
    has_value: bool,
    delimiter: u16,
    hash_token_type: HashTokenType,
    numeric_value: f64,
    // None represents source id_ == -1. Only valid CSSValueIDs can be supplied.
    id: Cell<Option<CSSValueID>>,
    unicode_range_start: i32,
    unicode_range_end: i32,
}

impl CSSParserToken {
    // cpp: css_parser_token.h:87-98
    pub fn new(token_type: CSSParserTokenType, block_type: BlockType) -> Self {
        Self {
            token_type,
            block_type,
            numeric_value_type: NumericValueType::kIntegerValueType,
            numeric_sign: NumericSign::kNoSign,
            unit: UnitType::kUnknown,
            value: TokenString::Empty { is_8bit: false },
            has_value: false,
            delimiter: 0,
            hash_token_type: HashTokenType::kHashTokenId,
            numeric_value: 0.0,
            id: Cell::new(None),
            unicode_range_start: 0,
            unicode_range_end: 0,
        }
    }
    // cpp: css_parser_token.h:101-107
    pub fn WithValue(
        token_type: CSSParserTokenType,
        value: StringView,
        block_type: BlockType,
        id: Option<CSSValueID>,
    ) -> Self {
        let mut token = Self::new(token_type, block_type);
        token.id.set(id);
        token.InitValueFromStringView(value);
        token
    }
    // cpp: css_parser_token.cc:24-30
    pub fn WithDelimiter(token_type: CSSParserTokenType, delimiter: u16) -> Self {
        debug_assert_eq!(token_type, CSSParserTokenType::kDelimiterToken);
        let mut token = Self::new(token_type, BlockType::kNotBlock);
        token.delimiter = delimiter;
        token
    }
    // cpp: css_parser_token.cc:32-46
    // ClampTo<double> direct-comparison specialization:
    // platform/wtf/math_extras.h:150-156; NaN and signed zero are preserved.
    pub fn WithNumber(
        token_type: CSSParserTokenType,
        numeric_value: f64,
        numeric_value_type: NumericValueType,
        sign: NumericSign,
    ) -> Self {
        debug_assert_eq!(token_type, CSSParserTokenType::kNumberToken);
        let mut token = Self::new(token_type, BlockType::kNotBlock);
        token.numeric_value_type = numeric_value_type;
        token.numeric_sign = sign;
        token.unit = UnitType::kNumber;
        let limit = f32::MAX as f64;
        token.numeric_value = if numeric_value >= limit {
            limit
        } else if numeric_value <= -limit {
            -limit
        } else {
            numeric_value
        };
        token
    }
    // cpp: css_parser_token.cc:48-57
    pub fn WithUnicodeRange(token_type: CSSParserTokenType, start: i32, end: i32) -> Self {
        debug_assert_eq!(token_type, CSSParserTokenType::kUnicodeRangeToken);
        let mut token = Self::new(CSSParserTokenType::kUnicodeRangeToken, BlockType::kNotBlock);
        token.unicode_range_start = start;
        token.unicode_range_end = end;
        token
    }
    // cpp: css_parser_token.cc:59-65
    pub fn WithHash(hash_type: HashTokenType, value: StringView) -> Self {
        let mut token = Self::new(CSSParserTokenType::kHashToken, BlockType::kNotBlock);
        token.hash_token_type = hash_type;
        token.InitValueFromStringView(value);
        token
    }
    // cpp: css_parser_token.h:217-232
    fn InitValueFromStringView(&mut self, value: StringView) {
        self.value = TokenString::FromStringView(value);
        self.has_value = true;
    }
    // cpp: css_parser_token.cc:67-72
    pub fn ConvertToDimensionWithUnit(&mut self, unit: StringView) {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kNumberToken);
        self.token_type = CSSParserTokenType::kDimensionToken;
        self.unit = StringToUnitType(&unit);
        self.InitValueFromStringView(unit);
    }
    // cpp: css_parser_token.cc:74-78
    pub fn ConvertToPercentage(&mut self) {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kNumberToken);
        self.token_type = CSSParserTokenType::kPercentageToken;
        self.unit = UnitType::kPercentage;
    }
    // cpp: css_parser_token.h:130-140
    pub fn GetType(&self) -> CSSParserTokenType {
        self.token_type
    }
    pub fn Value(&self) -> StringView {
        debug_assert!(self.has_value);
        self.value.Value()
    }
    pub fn IsEOF(&self) -> bool {
        self.token_type == CSSParserTokenType::kEOFToken
    }
    // cpp: css_parser_token.cc:80-103
    pub fn Delimiter(&self) -> u16 {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kDelimiterToken);
        self.delimiter
    }
    pub fn GetNumericSign(&self) -> NumericSign {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kNumberToken);
        self.numeric_sign
    }
    pub fn GetNumericValueType(&self) -> NumericValueType {
        debug_assert!(matches!(
            self.token_type,
            CSSParserTokenType::kNumberToken
                | CSSParserTokenType::kPercentageToken
                | CSSParserTokenType::kDimensionToken
        ));
        self.numeric_value_type
    }
    pub fn NumericValue(&self) -> f64 {
        debug_assert!(matches!(
            self.token_type,
            CSSParserTokenType::kNumberToken
                | CSSParserTokenType::kPercentageToken
                | CSSParserTokenType::kDimensionToken
        ));
        self.numeric_value
    }
    // cpp: css_parser_token.h:147-159
    pub fn GetHashTokenType(&self) -> HashTokenType {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kHashToken);
        self.hash_token_type
    }
    pub fn GetBlockType(&self) -> BlockType {
        self.block_type
    }
    pub fn GetUnitType(&self) -> UnitType {
        self.unit
    }
    pub fn UnicodeRangeStart(&self) -> i32 {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kUnicodeRangeToken);
        self.unicode_range_start
    }
    pub fn UnicodeRangeEnd(&self) -> i32 {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kUnicodeRangeToken);
        self.unicode_range_end
    }
    // cpp: css_parser_token.cc:116-124
    pub fn Id(&self) -> CSSValueID {
        if self.token_type != CSSParserTokenType::kIdentToken {
            return CSSValueID::kInvalid;
        }
        if let Some(id) = self.id.get() {
            return id;
        }
        let id = CssValueKeywordID(&self.Value());
        self.id.set(Some(id));
        id
    }
    // cpp: css_parser_token.h:163-168
    // A function must carry a precomputed keyword ID. None preserves the source
    // sentinel -1 without constructing an invalid Rust enum discriminant.
    pub fn FunctionId(&self) -> Option<CSSValueID> {
        if self.token_type != CSSParserTokenType::kFunctionToken {
            Some(CSSValueID::kInvalid)
        } else {
            self.id.get()
        }
    }
    // cpp: css_parser_token.cc:111-114
    pub fn ParseAsAtRuleDescriptorID(&self) -> AtRuleDescriptorID {
        debug_assert_eq!(self.token_type, CSSParserTokenType::kIdentToken);
        AsAtRuleDescriptorID(self.Value())
    }
    // cpp: css_parser_token.cc:126-135
    pub fn HasStringBacking(&self) -> bool {
        if self.value.IsInline() {
            return false;
        }
        matches!(
            self.token_type,
            CSSParserTokenType::kIdentToken
                | CSSParserTokenType::kFunctionToken
                | CSSParserTokenType::kAtKeywordToken
                | CSSParserTokenType::kHashToken
                | CSSParserTokenType::kUrlToken
                | CSSParserTokenType::kDimensionToken
                | CSSParserTokenType::kStringToken
        )
    }
    // cpp: css_parser_token.cc:137-142
    pub fn CopyWithUpdatedString(&self, string: StringView) -> Self {
        let mut copy = self.clone();
        copy.InitValueFromStringView(string);
        copy
    }
    // cpp: css_parser_token.h:180-186
    pub fn CopyWithoutValue(&self) -> Self {
        let mut copy = self.clone();
        copy.value = copy.value.WithoutValue();
        copy
    }
    // cpp: css_parser_token.h:188-201
    pub fn ClosingTokenType(opening_type: CSSParserTokenType) -> CSSParserTokenType {
        use CSSParserTokenType::*;
        match opening_type {
            kFunctionToken | kLeftParenthesisToken => kRightParenthesisToken,
            kLeftBracketToken => kRightBracketToken,
            kLeftBraceToken => kRightBraceToken,
            _ => panic!("NOTREACHED: token does not open a block"),
        }
    }
    // cpp: css_parser_token.cc:144-159
    fn ValueDataCharRawEqual(&self, other: &Self) -> bool {
        // The source pointer shortcut and all four 8/16-bit span comparisons
        // give code-unit equality, independent of StringView nullness or width.
        self.value.Value().Span16() == other.value.Value().Span16()
    }
}

// cpp: css_parser_token.cc:161-194
impl PartialEq for CSSParserToken {
    fn eq(&self, other: &Self) -> bool {
        use CSSParserTokenType::*;
        if self.token_type != other.token_type {
            return false;
        }
        match self.token_type {
            kDelimiterToken => self.Delimiter() == other.Delimiter(),
            kHashToken => {
                self.hash_token_type == other.hash_token_type && self.ValueDataCharRawEqual(other)
            }
            kIdentToken | kFunctionToken | kStringToken | kUrlToken => {
                self.ValueDataCharRawEqual(other)
            }
            kDimensionToken => {
                self.ValueDataCharRawEqual(other)
                    && self.numeric_sign == other.numeric_sign
                    && self.numeric_value == other.numeric_value
                    && self.numeric_value_type == other.numeric_value_type
            }
            kNumberToken | kPercentageToken => {
                self.numeric_sign == other.numeric_sign
                    && self.numeric_value == other.numeric_value
                    && self.numeric_value_type == other.numeric_value_type
            }
            kUnicodeRangeToken => {
                self.unicode_range_start == other.unicode_range_start
                    && self.unicode_range_end == other.unicode_range_end
            }
            _ => true,
        }
    }
}

// cpp: css_parser_token.cc:308-350
pub fn NeedsInsertedComment(a: &CSSParserToken, b: &CSSParserToken) -> bool {
    use CSSParserTokenType::*;
    let at = a.GetType();
    let bt = b.GetType();
    if matches!(
        at,
        kIdentToken | kAtKeywordToken | kHashToken | kDimensionToken | kNumberToken
    ) || (at == kDelimiterToken && matches!(a.Delimiter(), 35 | 45))
    {
        if at == kIdentToken && bt == kLeftParenthesisToken {
            return true;
        }
        if at == kNumberToken && bt == kDelimiterToken {
            if b.Delimiter() == b'-' as u16 {
                return false;
            }
            if b.Delimiter() == b'%' as u16 {
                return true;
            }
        }
        return matches!(
            bt,
            kIdentToken
                | kFunctionToken
                | kUrlToken
                | kBadUrlToken
                | kNumberToken
                | kPercentageToken
                | kDimensionToken
                | kCDCToken
        ) || (bt == kDelimiterToken && b.Delimiter() == b'-' as u16);
    }
    if at == kDelimiterToken && a.Delimiter() == b'@' as u16 {
        return matches!(
            bt,
            kIdentToken | kFunctionToken | kUrlToken | kBadUrlToken | kCDCToken
        ) || (bt == kDelimiterToken && b.Delimiter() == b'-' as u16);
    }
    if at == kDelimiterToken && matches!(a.Delimiter(), 46 | 43) {
        return matches!(bt, kNumberToken | kPercentageToken | kDimensionToken);
    }
    at == kDelimiterToken
        && bt == kDelimiterToken
        && a.Delimiter() == b'/' as u16
        && b.Delimiter() == b'*' as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use CSSParserTokenType::*;
    fn value(kind: CSSParserTokenType, text: String) -> CSSParserToken {
        CSSParserToken::WithValue(kind, StringView::from(&text), BlockType::kNotBlock, None)
    }
    fn number(n: f64, sign: NumericSign) -> CSSParserToken {
        CSSParserToken::WithNumber(kNumberToken, n, NumericValueType::kIntegerValueType, sign)
    }
    fn delimiter(c: u8) -> CSSParserToken {
        CSSParserToken::WithDelimiter(kDelimiterToken, c as u16)
    }

    #[test]
    fn inline_width_copy_and_source_equality_cases() {
        let short = value(kIdentToken, String::from_latin1(b"abcdefgh"));
        let wide = value(
            kIdentToken,
            String::from_utf16(&[97, 98, 99, 100, 101, 102, 103, 104]),
        );
        assert!(!short.HasStringBacking());
        assert!(wide.HasStringBacking());
        assert_eq!(short, wide);
        assert!(short.Value().Is8Bit());
        assert!(!wide.Value().Is8Bit());
        let long =
            short.CopyWithUpdatedString(StringView::from(&String::from_latin1(b"abcdefghi")));
        assert!(long.HasStringBacking());
        assert_eq!(long.Value().length(), 9);
        let cleared = short.CopyWithoutValue();
        assert!(cleared.HasStringBacking());
        assert!(cleared.Value().Is8Bit());
        assert!(cleared.Value().IsEmpty());
        assert!(!wide.CopyWithoutValue().Value().Is8Bit());
        assert_eq!(
            value(kAtKeywordToken, String::from("a")),
            value(kAtKeywordToken, String::from("b"))
        );
        assert_ne!(
            CSSParserToken::WithHash(HashTokenType::kHashTokenId, short.Value()),
            CSSParserToken::WithHash(HashTokenType::kHashTokenUnrestricted, short.Value())
        );
        let start = CSSParserToken::WithValue(
            kFunctionToken,
            short.Value(),
            BlockType::kBlockStart,
            Some(CSSValueID::kCalc),
        );
        let plain = CSSParserToken::WithValue(
            kFunctionToken,
            short.Value(),
            BlockType::kNotBlock,
            Some(CSSValueID::kInvalid),
        );
        assert_eq!(start, plain);
        assert_eq!(start.FunctionId(), Some(CSSValueID::kCalc));
        assert_eq!(short.FunctionId(), Some(CSSValueID::kInvalid));
    }
    #[test]
    fn number_clamping_conversion_and_sign_equality() {
        assert_eq!(
            number(f64::INFINITY, NumericSign::kPlusSign).NumericValue(),
            f32::MAX as f64
        );
        assert_eq!(
            number(f64::NEG_INFINITY, NumericSign::kMinusSign).NumericValue(),
            -(f32::MAX as f64)
        );
        let nan = number(f64::NAN, NumericSign::kNoSign);
        assert!(nan.NumericValue().is_nan());
        assert_ne!(nan, nan.clone());
        assert!(number(-0.0, NumericSign::kMinusSign)
            .NumericValue()
            .is_sign_negative());
        assert_ne!(
            number(0.0, NumericSign::kNoSign),
            number(0.0, NumericSign::kPlusSign)
        );
        let mut dimension = number(2.0, NumericSign::kNoSign);
        dimension.ConvertToDimensionWithUnit(StringView::from(&String::from_latin1(b"PX")));
        assert_eq!(dimension.GetType(), kDimensionToken);
        assert_eq!(dimension.GetUnitType(), UnitType::kPixels);
        assert_eq!(dimension.Value().Span8(), Some(&b"PX"[..]));
        assert_eq!(dimension.NumericValue(), 2.0);
        let mut percentage = number(2.0, NumericSign::kPlusSign);
        percentage.ConvertToPercentage();
        assert_eq!(percentage.GetType(), kPercentageToken);
        assert_eq!(percentage.GetUnitType(), UnitType::kPercentage);
        assert_eq!(
            percentage.GetNumericValueType(),
            NumericValueType::kIntegerValueType
        );
        assert_eq!(
            CSSParserToken::ClosingTokenType(kFunctionToken),
            kRightParenthesisToken
        );
        assert_eq!(
            CSSParserToken::ClosingTokenType(kLeftBracketToken),
            kRightBracketToken
        );
        assert_eq!(
            CSSParserToken::ClosingTokenType(kLeftBraceToken),
            kRightBraceToken
        );
        assert_eq!(
            value(kIdentToken, String::from("FONT-FAMILY")).ParseAsAtRuleDescriptorID(),
            AtRuleDescriptorID::FontFamily
        );
    }
    #[test]
    fn keyword_lookup_preserves_lazy_cache_and_storage_width() {
        let ident = value(kIdentToken, String::from_latin1(b"INHERIT"));
        assert_eq!(ident.id.get(), None);
        assert_eq!(ident.Id(), CSSValueID::kInherit);
        assert_eq!(ident.id.get(), Some(CSSValueID::kInherit));
        // Source CopyWithUpdatedString preserves the cached ID.
        let copy = ident.CopyWithUpdatedString(StringView::from("none"));
        assert_eq!(copy.Id(), CSSValueID::kInherit);
        let uncached = value(kIdentToken, String::from("inherit"));
        assert_eq!(
            uncached
                .CopyWithUpdatedString(StringView::from("none"))
                .Id(),
            CSSValueID::kNone
        );
        assert_eq!(
            value(kIdentToken, String::from("INHERIT")).Id(),
            CSSValueID::kInherit
        );
        for text in [
            "",
            "inherit\0",
            "\u{161}olid",
            "inherit_",
            "@@@@@@@@",
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        ] {
            assert_eq!(
                value(kIdentToken, String::from(text)).Id(),
                CSSValueID::kInvalid
            );
        }
        assert_eq!(
            value(kIdentToken, String::from_latin1(&[0xff])).Id(),
            CSSValueID::kInvalid
        );
        let supplied = CSSParserToken::WithValue(
            kIdentToken,
            StringView::from("inherit"),
            BlockType::kNotBlock,
            Some(CSSValueID::kInvalid),
        );
        assert_eq!(supplied.Id(), CSSValueID::kInvalid);
        let function = value(kFunctionToken, String::from("inherit"));
        assert_eq!(function.Id(), CSSValueID::kInvalid);
        assert_eq!(function.id.get(), None);
    }

    #[test]
    fn comment_insertion_covers_serialization_table_boundaries() {
        let ident = value(kIdentToken, String::from("a"));
        let n = number(1.0, NumericSign::kNoSign);
        assert!(NeedsInsertedComment(
            &ident,
            &CSSParserToken::new(kLeftParenthesisToken, BlockType::kNotBlock)
        ));
        assert!(NeedsInsertedComment(&ident, &delimiter(b'-')));
        assert!(!NeedsInsertedComment(&n, &delimiter(b'-')));
        assert!(NeedsInsertedComment(&n, &delimiter(b'%')));
        assert!(NeedsInsertedComment(&delimiter(b'#'), &ident));
        assert!(NeedsInsertedComment(&delimiter(b'@'), &ident));
        assert!(!NeedsInsertedComment(&delimiter(b'@'), &n));
        assert!(NeedsInsertedComment(&delimiter(b'+'), &n));
        assert!(NeedsInsertedComment(&delimiter(b'.'), &n));
        assert!(!NeedsInsertedComment(&delimiter(b'.'), &ident));
        assert!(NeedsInsertedComment(&delimiter(b'/'), &delimiter(b'*')));
        assert!(!NeedsInsertedComment(&delimiter(b'*'), &delimiter(b'/')));
        assert!(!NeedsInsertedComment(
            &CSSParserToken::new(kWhitespaceToken, BlockType::kNotBlock),
            &ident
        ));
    }
}
