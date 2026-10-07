#![allow(non_snake_case, non_camel_case_types)]

use foundation::{String, StringView, UChar, UChar32};

// cpp: css_parser/css_parser_token.h:15-49
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
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

// cpp: css_parser/css_parser_token.h:51-53
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumericSign {
    #[default]
    kNoSign,
    kPlusSign,
    kMinusSign,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumericValueType {
    #[default]
    kIntegerValueType,
    kNumberValueType,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HashTokenType {
    kHashTokenId,
    #[default]
    kHashTokenUnrestricted,
}

// cpp: css_parser/css_parser_token.h:55-59
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CSSValueID {
    kInvalid = -1,
}
pub fn CssValueKeywordID(_value: StringView) -> CSSValueID {
    CSSValueID::kInvalid
}

// cpp: css_parser/css_parser_token.h:61-63
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockType {
    #[default]
    kNotBlock,
    kBlockStart,
    kBlockEnd,
}

// cpp: css_parser/css_parser_token.h:120-133
#[derive(Clone, Debug)]
pub struct CSSParserToken {
    type_: CSSParserTokenType,
    block_type_: BlockType,
    value_: String,
    unit_: String,
    id_: i32,
    delimiter_: UChar,
    numeric_sign_: NumericSign,
    numeric_value_type_: NumericValueType,
    hash_token_type_: HashTokenType,
    numeric_value_: f64,
    unicode_start_: UChar32,
    unicode_end_: UChar32,
}

impl CSSParserToken {
    fn initial(type_: CSSParserTokenType, block_type: BlockType) -> Self {
        Self {
            type_,
            block_type_: block_type,
            value_: String::default(),
            unit_: String::default(),
            id_: -1,
            delimiter_: 0,
            numeric_sign_: NumericSign::kNoSign,
            numeric_value_type_: NumericValueType::kIntegerValueType,
            hash_token_type_: HashTokenType::kHashTokenUnrestricted,
            numeric_value_: 0.0,
            unicode_start_: 0,
            unicode_end_: 0,
        }
    }

    // cpp: css_parser/css_parser_token.h:65-92
    pub fn new(type_: CSSParserTokenType, block_type: BlockType) -> Self {
        Self::initial(type_, block_type)
    }
    pub fn with_value(
        type_: CSSParserTokenType,
        value: StringView,
        block_type: BlockType,
        id: i32,
    ) -> Self {
        let mut token = Self::initial(type_, block_type);
        token.value_ = value.ToString();
        token.id_ = id;
        token
    }
    pub fn with_delimiter(type_: CSSParserTokenType, delimiter: UChar) -> Self {
        debug_assert_eq!(type_, CSSParserTokenType::kDelimiterToken);
        let mut token = Self::initial(type_, BlockType::kNotBlock);
        token.delimiter_ = delimiter;
        token
    }
    pub fn with_number(
        type_: CSSParserTokenType,
        number: f64,
        number_type: NumericValueType,
        sign: NumericSign,
    ) -> Self {
        debug_assert_eq!(type_, CSSParserTokenType::kNumberToken);
        let mut token = Self::initial(type_, BlockType::kNotBlock);
        token.numeric_sign_ = sign;
        token.numeric_value_type_ = number_type;
        token.numeric_value_ = number;
        token
    }
    pub fn with_unicode_range(type_: CSSParserTokenType, start: UChar32, end: UChar32) -> Self {
        debug_assert_eq!(type_, CSSParserTokenType::kUnicodeRangeToken);
        let mut token = Self::initial(type_, BlockType::kNotBlock);
        token.unicode_start_ = start;
        token.unicode_end_ = end;
        token
    }
    pub fn with_hash(hash_type: HashTokenType, value: StringView) -> Self {
        let mut token = Self::initial(CSSParserTokenType::kHashToken, BlockType::kNotBlock);
        token.value_ = value.ToString();
        token.hash_token_type_ = hash_type;
        token
    }

    // cpp: css_parser/css_parser_token.h:94-108
    pub fn GetType(&self) -> CSSParserTokenType {
        self.type_
    }
    pub fn GetBlockType(&self) -> BlockType {
        self.block_type_
    }
    pub fn IsEOF(&self) -> bool {
        self.type_ == CSSParserTokenType::kEOFToken
    }
    pub fn Value(&self) -> &String {
        &self.value_
    }
    pub fn Delimiter(&self) -> UChar {
        debug_assert_eq!(self.type_, CSSParserTokenType::kDelimiterToken);
        self.delimiter_
    }
    pub fn GetNumericSign(&self) -> NumericSign {
        self.numeric_sign_
    }
    pub fn GetNumericValueType(&self) -> NumericValueType {
        self.numeric_value_type_
    }
    pub fn NumericValue(&self) -> f64 {
        self.numeric_value_
    }
    pub fn GetHashTokenType(&self) -> HashTokenType {
        self.hash_token_type_
    }
    pub fn UnicodeRangeStart(&self) -> UChar32 {
        self.unicode_start_
    }
    pub fn UnicodeRangeEnd(&self) -> UChar32 {
        self.unicode_end_
    }
    pub fn Unit(&self) -> &String {
        &self.unit_
    }

    // cpp: css_parser/css_parser_token.h:110-118
    pub fn ConvertToDimensionWithUnit(&mut self, unit: StringView) {
        debug_assert_eq!(self.type_, CSSParserTokenType::kNumberToken);
        self.type_ = CSSParserTokenType::kDimensionToken;
        self.unit_ = unit.ToString();
    }
    pub fn ConvertToPercentage(&mut self) {
        debug_assert_eq!(self.type_, CSSParserTokenType::kNumberToken);
        self.type_ = CSSParserTokenType::kPercentageToken;
    }
}
