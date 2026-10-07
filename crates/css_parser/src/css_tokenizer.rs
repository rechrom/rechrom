#![allow(non_snake_case)]

use crate::css_parser_idioms::{
    kCSSEndOfFileMarker, ConsumeEscape as ConsumeCSSEscape, ConsumeName as ConsumeCSSName,
    ConsumeSingleWhitespaceIfNext as ConsumeCSSSingleWhitespaceIfNext, IsCSSInputSpace,
    IsCSSNewLine, IsLeadingSurrogate, IsNameCodePoint, IsSurrogate, IsTrailingSurrogate,
    NextCharsAreIdentifier as CSSNextCharsAreIdentifier, TwoCharsAreValidEscape,
};
use crate::css_parser_token::{
    BlockType, CSSParserToken, CSSParserTokenType, CSSValueID, CssValueKeywordID, HashTokenType,
    NumericSign, NumericValueType,
};
use crate::css_tokenizer_input_stream::CSSTokenizerInputStream;
use foundation::{String, StringView, UChar, UChar32};

use CSSParserTokenType::*;

fn ascii_digit(c: UChar) -> bool {
    (0x30..=0x39).contains(&c)
}

fn ascii_hex_value(c: UChar) -> Option<i32> {
    match c {
        0x30..=0x39 => Some(i32::from(c - 0x30)),
        0x41..=0x46 => Some(i32::from(c - 0x41 + 10)),
        0x61..=0x66 => Some(i32::from(c - 0x61 + 10)),
        _ => None,
    }
}

fn token(type_: CSSParserTokenType) -> CSSParserToken {
    CSSParserToken::new(type_, BlockType::kNotBlock)
}

fn delimiter(character: UChar) -> CSSParserToken {
    CSSParserToken::with_delimiter(kDelimiterToken, character)
}

fn append_code_point(output: &mut Vec<u16>, code_point: UChar32) {
    let character = char::from_u32(code_point as u32).expect("CSS escape is a valid code point");
    let mut units = [0; 2];
    output.extend(character.encode_utf16(&mut units).iter().copied());
}

// cpp: css_parser/css_tokenizer.h:19-38,129-149
pub struct CSSTokenizer {
    input_storage_: String,
    input_lifetime_guard_: String,
    input_: CSSTokenizerInputStream,
    block_stack_: Vec<CSSParserTokenType>,
    string_pool_: Vec<String>,
    prev_offset_: u32,
    token_count_: u32,
    unicode_ranges_allowed_: bool,
}

impl CSSTokenizer {
    // cpp: css_parser/css_tokenizer.cc:20-33
    pub fn new(string: &String, offset: u32) -> Self {
        let input_storage_ = string.clone();
        let input_lifetime_guard_ = input_storage_.clone();
        let mut input_ = CSSTokenizerInputStream::new(StringView::from(&input_storage_));
        input_.Advance(offset);
        Self {
            input_storage_,
            input_lifetime_guard_,
            input_,
            block_stack_: Vec::new(),
            string_pool_: Vec::new(),
            prev_offset_: 0,
            token_count_: 0,
            unicode_ranges_allowed_: false,
        }
    }

    // cpp: css_parser/css_tokenizer.h:30-38
    // cpp: css_parser/css_tokenizer.cc:35-54
    pub fn TokenCount(&self) -> u32 {
        self.token_count_
    }
    pub fn Offset(&self) -> u32 {
        self.input_.Offset()
    }
    pub fn PreviousOffset(&self) -> u32 {
        self.prev_offset_
    }
    pub fn StringRangeFrom(&self, start: u32) -> StringView {
        self.input_.RangeFrom(start)
    }
    pub fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
        self.input_.RangeAt(start, length)
    }
    pub fn StringPool(&self) -> &[String] {
        &self.string_pool_
    }
    pub fn TokenizeSingle(&mut self) -> CSSParserToken {
        self.NextToken(true)
    }
    pub fn TokenizeSingleWithComments(&mut self) -> CSSParserToken {
        self.NextToken(false)
    }

    // cpp: css_parser/css_tokenizer.h:40-69
    pub fn SkipToEndOfBlock(&mut self, offset: u32) {
        debug_assert!(offset > self.input_.Offset());
        if cfg!(debug_assertions) {
            let base_nesting_level = self.block_stack_.len();
            debug_assert!(base_nesting_level >= 1);
            while self.input_.Offset() < offset - 1 {
                self.TokenizeSingle();
                debug_assert!(self.block_stack_.len() >= base_nesting_level);
            }
            debug_assert_eq!(self.input_.Offset(), offset - 1);
            debug_assert_eq!(self.block_stack_.len(), base_nesting_level);
            self.TokenizeSingle();
            debug_assert_eq!(self.input_.Offset(), offset);
            debug_assert_eq!(self.block_stack_.len(), base_nesting_level - 1);
        } else {
            self.block_stack_.pop();
        }
        self.input_.Restore(offset);
    }

    // cpp: css_parser/css_tokenizer.h:71-86
    pub fn Restore(&mut self, next: &CSSParserToken, offset: u32) -> CSSParserToken {
        if next.GetBlockType() == BlockType::kBlockStart {
            self.block_stack_.pop();
        } else if next.GetBlockType() == BlockType::kBlockEnd {
            let start_type = match next.GetType() {
                kRightParenthesisToken => kLeftParenthesisToken,
                kRightBracketToken => kLeftBracketToken,
                kRightBraceToken => kLeftBraceToken,
                _ => panic!("block-end token must close parentheses, brackets or braces"),
            };
            self.block_stack_.push(start_type);
        }
        self.input_.Restore(offset);
        self.TokenizeSingle()
    }

    // cpp: css_parser/css_tokenizer.cc:56-64
    fn Reconsume(&mut self, c: UChar) {
        self.input_.PushBack(c);
    }
    fn Consume(&mut self) -> UChar {
        let current = self.input_.NextInputChar();
        self.input_.Advance(1);
        current
    }

    // cpp: css_parser/css_tokenizer.cc:66-87
    fn BlockStart(&mut self, type_: CSSParserTokenType) -> CSSParserToken {
        self.block_stack_.push(type_);
        CSSParserToken::new(type_, BlockType::kBlockStart)
    }
    fn BlockStartFunction(
        &mut self,
        block_type: CSSParserTokenType,
        type_: CSSParserTokenType,
        name: StringView,
        id: CSSValueID,
    ) -> CSSParserToken {
        self.block_stack_.push(block_type);
        CSSParserToken::with_value(type_, name, BlockType::kBlockStart, id as i32)
    }
    fn BlockEnd(
        &mut self,
        type_: CSSParserTokenType,
        start_type: CSSParserTokenType,
    ) -> CSSParserToken {
        if self.block_stack_.last() == Some(&start_type) {
            self.block_stack_.pop();
            return CSSParserToken::new(type_, BlockType::kBlockEnd);
        }
        token(type_)
    }

    // cpp: css_parser/css_tokenizer.cc:89-127
    fn HyphenMinus(&mut self, cc: UChar) -> CSSParserToken {
        if self.NextCharsAreNumberWithFirst(cc) {
            self.Reconsume(cc);
            return self.ConsumeNumericToken();
        }
        if self.input_.PeekWithoutReplacement(0) == u16::from(b'-')
            && self.input_.PeekWithoutReplacement(1) == u16::from(b'>')
        {
            self.input_.Advance(2);
            return token(kCDCToken);
        }
        if self.NextCharsAreIdentifierWithFirst(cc) {
            self.Reconsume(cc);
            return self.ConsumeIdentLikeToken();
        }
        delimiter(cc)
    }
    fn Hash(&mut self, cc: UChar) -> CSSParserToken {
        let next_char = self.input_.NextInputChar();
        if IsNameCodePoint(next_char)
            || TwoCharsAreValidEscape(next_char, self.input_.PeekWithoutReplacement(1))
        {
            let type_ = if self.NextCharsAreIdentifier() {
                HashTokenType::kHashTokenId
            } else {
                HashTokenType::kHashTokenUnrestricted
            };
            return CSSParserToken::with_hash(type_, self.ConsumeName());
        }
        delimiter(cc)
    }
    fn LetterU(&mut self, cc: UChar) -> CSSParserToken {
        if self.unicode_ranges_allowed_
            && self.input_.PeekWithoutReplacement(0) == u16::from(b'+')
            && (ascii_hex_value(self.input_.PeekWithoutReplacement(1)).is_some()
                || self.input_.PeekWithoutReplacement(1) == u16::from(b'?'))
        {
            self.input_.Advance(1);
            return self.ConsumeUnicodeRange();
        }
        self.Reconsume(cc);
        self.ConsumeIdentLikeToken()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // cpp: css_parser/css_tokenizer_test.cc:20-65
    #[test]
    fn preserves_source_css_syntax_token_stream() {
        let source = String::from(
            r"@media (min-width: 40rem) { .card#main:hover > img[data-x^='a'] { width: calc(100% - 2.5px); color: #12aBcD; --x: \1f642; } }",
        );
        let mut tokenizer = CSSTokenizer::new(&source, 0);
        let mut saw_at_rule = false;
        let mut saw_dimension = false;
        let mut saw_percentage = false;
        let mut saw_function = false;
        let mut saw_hash_id = false;
        let mut saw_prefix_match = false;
        let mut saw_escaped_ident = false;
        let mut braces = 0;
        loop {
            let token = tokenizer.TokenizeSingleWithComments();
            match token.GetType() {
                kAtKeywordToken => saw_at_rule |= token.Value().Utf8() == "media",
                kDimensionToken => {
                    saw_dimension |= token.NumericValue() == 40.0 && token.Unit().Utf8() == "rem"
                }
                kPercentageToken => saw_percentage |= token.NumericValue() == 100.0,
                kFunctionToken => saw_function |= token.Value().Utf8() == "calc",
                kHashToken => {
                    saw_hash_id |= matches!(token.Value().Utf8().as_str(), "main" | "12aBcD")
                }
                kPrefixMatchToken => saw_prefix_match = true,
                kIdentToken => saw_escaped_ident |= token.Value().Utf8() == "🙂",
                kLeftBraceToken | kRightBraceToken => braces += 1,
                _ => {}
            }
            if token.IsEOF() {
                break;
            }
        }
        assert!(saw_at_rule);
        assert!(saw_dimension);
        assert!(saw_percentage);
        assert!(saw_function);
        assert!(saw_hash_id);
        assert!(saw_prefix_match);
        assert!(saw_escaped_ident);
        assert_eq!(braces, 4);
    }
}

impl CSSTokenizer {
    // cpp: css_parser/css_tokenizer.cc:129-298
    fn NextToken(&mut self, skip_comments: bool) -> CSSParserToken {
        loop {
            self.prev_offset_ = self.input_.Offset();
            let cc = self.Consume();
            self.token_count_ += 1;
            match cc {
                0 => return token(kEOFToken),
                0x09 | 0x0A | 0x0C | 0x0D | 0x20 => {
                    self.input_.AdvanceUntilNonWhitespace();
                    return token(kWhitespaceToken);
                }
                0x27 | 0x22 => return self.ConsumeStringTokenUntil(cc),
                0x30..=0x39 => {
                    self.Reconsume(cc);
                    return self.ConsumeNumericToken();
                }
                0x28 => return self.BlockStart(kLeftParenthesisToken),
                0x29 => return self.BlockEnd(kRightParenthesisToken, kLeftParenthesisToken),
                0x5B => return self.BlockStart(kLeftBracketToken),
                0x5D => return self.BlockEnd(kRightBracketToken, kLeftBracketToken),
                0x7B => return self.BlockStart(kLeftBraceToken),
                0x7D => return self.BlockEnd(kRightBraceToken, kLeftBraceToken),
                0x2B | 0x2E => {
                    if self.NextCharsAreNumberWithFirst(cc) {
                        self.Reconsume(cc);
                        return self.ConsumeNumericToken();
                    }
                    return delimiter(cc);
                }
                0x2D => return self.HyphenMinus(cc),
                0x2A => {
                    if self.ConsumeIfNext(0x3D) {
                        return token(kSubstringMatchToken);
                    }
                    return delimiter(cc);
                }
                0x3C => {
                    if self.input_.PeekWithoutReplacement(0) == 0x21
                        && self.input_.PeekWithoutReplacement(1) == 0x2D
                        && self.input_.PeekWithoutReplacement(2) == 0x2D
                    {
                        self.input_.Advance(3);
                        return token(kCDOToken);
                    }
                    return delimiter(cc);
                }
                0x2C => return token(kCommaToken),
                0x2F => {
                    if self.ConsumeIfNext(0x2A) {
                        self.ConsumeUntilCommentEndFound();
                        if skip_comments {
                            continue;
                        }
                        return token(kCommentToken);
                    }
                    return delimiter(cc);
                }
                0x5C => {
                    if TwoCharsAreValidEscape(cc, self.input_.PeekWithoutReplacement(0)) {
                        self.Reconsume(cc);
                        return self.ConsumeIdentLikeToken();
                    }
                    return delimiter(cc);
                }
                0x3A => return token(kColonToken),
                0x3B => return token(kSemicolonToken),
                0x23 => return self.Hash(cc),
                0x5E => {
                    if self.ConsumeIfNext(0x3D) {
                        return token(kPrefixMatchToken);
                    }
                    return delimiter(cc);
                }
                0x24 => {
                    if self.ConsumeIfNext(0x3D) {
                        return token(kSuffixMatchToken);
                    }
                    return delimiter(cc);
                }
                0x7C => {
                    if self.ConsumeIfNext(0x3D) {
                        return token(kDashMatchToken);
                    }
                    if self.ConsumeIfNext(0x7C) {
                        return token(kColumnToken);
                    }
                    return delimiter(cc);
                }
                0x7E => {
                    if self.ConsumeIfNext(0x3D) {
                        return token(kIncludeMatchToken);
                    }
                    return delimiter(cc);
                }
                0x40 => {
                    if self.NextCharsAreIdentifier() {
                        return CSSParserToken::with_value(
                            kAtKeywordToken,
                            self.ConsumeName(),
                            BlockType::kNotBlock,
                            -1,
                        );
                    }
                    return delimiter(cc);
                }
                0x55 | 0x75 => return self.LetterU(cc),
                0x01..=0x08
                | 0x0B
                | 0x0E..=0x1F
                | 0x21
                | 0x25
                | 0x26
                | 0x3D
                | 0x3E
                | 0x3F
                | 0x60
                | 0x7F => return delimiter(cc),
                _ => {
                    self.Reconsume(cc);
                    return self.ConsumeIdentLikeToken();
                }
            }
        }
    }

    // cpp: css_parser/css_tokenizer.cc:300-360
    fn ConsumeNumber(&mut self) -> CSSParserToken {
        debug_assert!(self.NextCharsAreNumber());
        let mut type_ = NumericValueType::kIntegerValueType;
        let mut sign = NumericSign::kNoSign;
        let mut number_length = 0;
        let mut sign_length = 0;

        let mut next = self.input_.PeekWithoutReplacement(0);
        if next == 0x2B {
            number_length += 1;
            sign_length += 1;
            sign = NumericSign::kPlusSign;
        } else if next == 0x2D {
            number_length += 1;
            sign_length += 1;
            sign = NumericSign::kMinusSign;
        }

        number_length = self.input_.SkipWhilePredicate(number_length, ascii_digit);
        next = self.input_.PeekWithoutReplacement(number_length);
        if next == 0x2E && ascii_digit(self.input_.PeekWithoutReplacement(number_length + 1)) {
            type_ = NumericValueType::kNumberValueType;
            number_length = self
                .input_
                .SkipWhilePredicate(number_length + 2, ascii_digit);
            next = self.input_.PeekWithoutReplacement(number_length);
        }

        if next == 0x45 || next == 0x65 {
            next = self.input_.PeekWithoutReplacement(number_length + 1);
            if ascii_digit(next) {
                type_ = NumericValueType::kNumberValueType;
                number_length = self
                    .input_
                    .SkipWhilePredicate(number_length + 1, ascii_digit);
            } else if (next == 0x2B || next == 0x2D)
                && ascii_digit(self.input_.PeekWithoutReplacement(number_length + 2))
            {
                type_ = NumericValueType::kNumberValueType;
                number_length = self
                    .input_
                    .SkipWhilePredicate(number_length + 3, ascii_digit);
            }
        }

        let value = if type_ == NumericValueType::kIntegerValueType {
            let mut value = self
                .input_
                .GetNaturalNumberAsDouble(sign_length, number_length);
            if sign == NumericSign::kMinusSign {
                value = -value;
            }
            debug_assert_eq!(value, self.input_.GetDouble(0, number_length));
            self.input_.Advance(number_length);
            value
        } else {
            let value = self.input_.GetDouble(0, number_length);
            self.input_.Advance(number_length);
            value
        };
        CSSParserToken::with_number(kNumberToken, value, type_, sign)
    }

    // cpp: css_parser/css_tokenizer.cc:362-371
    fn ConsumeNumericToken(&mut self) -> CSSParserToken {
        let mut token = self.ConsumeNumber();
        if self.NextCharsAreIdentifier() {
            token.ConvertToDimensionWithUnit(self.ConsumeName());
        } else if self.ConsumeIfNext(0x25) {
            token.ConvertToPercentage();
        }
        token
    }

    // cpp: css_parser/css_tokenizer.cc:373-390
    fn ConsumeIdentLikeToken(&mut self) -> CSSParserToken {
        let name = self.ConsumeName();
        if self.ConsumeIfNext(0x28) {
            if name.ToString().Utf8().eq_ignore_ascii_case("url") {
                self.input_.AdvanceUntilNonWhitespace();
                let next = self.input_.PeekWithoutReplacement(0);
                if next != 0x22 && next != 0x27 {
                    return self.ConsumeUrlToken();
                }
            }
            return self.BlockStartFunction(
                kLeftParenthesisToken,
                kFunctionToken,
                name.clone(),
                CssValueKeywordID(name),
            );
        }
        CSSParserToken::with_value(kIdentToken, name, BlockType::kNotBlock, -1)
    }

    // cpp: css_parser/css_tokenizer.cc:392-435
    fn ConsumeStringTokenUntil(&mut self, ending_code_point: UChar) -> CSSParserToken {
        for size in 0.. {
            let cc = self.input_.PeekWithoutReplacement(size);
            if cc == ending_code_point {
                let start_offset = self.input_.Offset();
                self.input_.Advance(size + 1);
                return CSSParserToken::with_value(
                    kStringToken,
                    self.input_.RangeAt(start_offset, size),
                    BlockType::kNotBlock,
                    -1,
                );
            }
            if IsCSSNewLine(cc) {
                self.input_.Advance(size);
                return token(kBadStringToken);
            }
            if cc == 0 || cc == 0x5C {
                break;
            }
        }

        let mut output = Vec::<u16>::new();
        loop {
            let cc = self.Consume();
            if cc == ending_code_point || cc == kCSSEndOfFileMarker {
                let value = self.RegisterString(&String::from_utf16(&output));
                return CSSParserToken::with_value(kStringToken, value, BlockType::kNotBlock, -1);
            }
            if IsCSSNewLine(cc) {
                self.Reconsume(cc);
                return token(kBadStringToken);
            }
            if cc == 0x5C {
                if self.input_.NextInputChar() == kCSSEndOfFileMarker {
                    continue;
                }
                if IsCSSNewLine(self.input_.PeekWithoutReplacement(0)) {
                    self.ConsumeSingleWhitespaceIfNext();
                } else {
                    append_code_point(&mut output, self.ConsumeEscape());
                }
            } else {
                output.push(cc);
            }
        }
    }

    // cpp: css_parser/css_tokenizer.cc:437-469
    fn ConsumeUnicodeRange(&mut self) -> CSSParserToken {
        debug_assert!(
            ascii_hex_value(self.input_.PeekWithoutReplacement(0)).is_some()
                || self.input_.PeekWithoutReplacement(0) == 0x3F
        );
        let mut length_remaining = 6;
        let mut start: UChar32 = 0;
        while length_remaining > 0
            && ascii_hex_value(self.input_.PeekWithoutReplacement(0)).is_some()
        {
            start = start * 16 + ascii_hex_value(self.Consume()).expect("hex digit");
            length_remaining -= 1;
        }
        let mut end = start;
        if length_remaining > 0 && self.ConsumeIfNext(0x3F) {
            loop {
                start *= 16;
                end = end * 16 + 0xF;
                length_remaining -= 1;
                if length_remaining == 0 || !self.ConsumeIfNext(0x3F) {
                    break;
                }
            }
        } else if self.input_.PeekWithoutReplacement(0) == 0x2D
            && ascii_hex_value(self.input_.PeekWithoutReplacement(1)).is_some()
        {
            self.input_.Advance(1);
            length_remaining = 6;
            end = 0;
            loop {
                end = end * 16 + ascii_hex_value(self.Consume()).expect("hex digit");
                length_remaining -= 1;
                if length_remaining == 0
                    || ascii_hex_value(self.input_.PeekWithoutReplacement(0)).is_none()
                {
                    break;
                }
            }
        }
        CSSParserToken::with_unicode_range(kUnicodeRangeToken, start, end)
    }

    // cpp: css_parser/css_tokenizer.cc:471-475
    fn IsNonPrintableCodePoint(cc: UChar) -> bool {
        cc <= 0x08 || cc == 0x0B || (0x0E..=0x1F).contains(&cc) || cc == 0x7F
    }

    // cpp: css_parser/css_tokenizer.cc:477-528
    fn ConsumeUrlToken(&mut self) -> CSSParserToken {
        self.input_.AdvanceUntilNonWhitespace();
        for size in 0.. {
            let cc = self.input_.PeekWithoutReplacement(size);
            if cc == 0x29 {
                let start_offset = self.input_.Offset();
                self.input_.Advance(size + 1);
                return CSSParserToken::with_value(
                    kUrlToken,
                    self.input_.RangeAt(start_offset, size),
                    BlockType::kNotBlock,
                    -1,
                );
            }
            if cc <= 0x20 || matches!(cc, 0x5C | 0x22 | 0x27 | 0x28 | 0x7F) {
                break;
            }
        }
        let mut result = Vec::<u16>::new();
        loop {
            let cc = self.Consume();
            if cc == 0x29 || cc == kCSSEndOfFileMarker {
                let value = self.RegisterString(&String::from_utf16(&result));
                return CSSParserToken::with_value(kUrlToken, value, BlockType::kNotBlock, -1);
            }
            if IsCSSInputSpace(cc) {
                self.input_.AdvanceUntilNonWhitespace();
                if self.ConsumeIfNext(0x29) || self.input_.NextInputChar() == kCSSEndOfFileMarker {
                    let value = self.RegisterString(&String::from_utf16(&result));
                    return CSSParserToken::with_value(kUrlToken, value, BlockType::kNotBlock, -1);
                }
                break;
            }
            if matches!(cc, 0x22 | 0x27 | 0x28) || Self::IsNonPrintableCodePoint(cc) {
                break;
            }
            if cc == 0x5C {
                if TwoCharsAreValidEscape(cc, self.input_.PeekWithoutReplacement(0)) {
                    append_code_point(&mut result, self.ConsumeEscape());
                    continue;
                }
                break;
            }
            result.push(cc);
        }
        self.ConsumeBadUrlRemnants();
        token(kBadUrlToken)
    }

    // cpp: css_parser/css_tokenizer.cc:530-541
    fn ConsumeBadUrlRemnants(&mut self) {
        loop {
            let cc = self.Consume();
            if cc == 0x29 || cc == kCSSEndOfFileMarker {
                return;
            }
            if TwoCharsAreValidEscape(cc, self.input_.PeekWithoutReplacement(0)) {
                self.ConsumeEscape();
            }
        }
    }

    // cpp: css_parser/css_tokenizer.cc:543-545
    fn ConsumeSingleWhitespaceIfNext(&mut self) {
        ConsumeCSSSingleWhitespaceIfNext(&mut self.input_);
    }

    // cpp: css_parser/css_tokenizer.cc:547-562
    fn ConsumeUntilCommentEndFound(&mut self) {
        let mut c = self.Consume();
        loop {
            if c == kCSSEndOfFileMarker {
                return;
            }
            if c != 0x2A {
                c = self.Consume();
                continue;
            }
            c = self.Consume();
            if c == 0x2F {
                return;
            }
        }
    }

    // cpp: css_parser/css_tokenizer.cc:564-574
    fn ConsumeIfNext(&mut self, character: UChar) -> bool {
        debug_assert_ne!(character, 0);
        if self.input_.PeekWithoutReplacement(0) == character {
            self.input_.Advance(1);
            return true;
        }
        false
    }

    // cpp: css_parser/css_tokenizer.cc:576-686
    fn ConsumeName(&mut self) -> StringView {
        let buffer = self.input_.Peek();
        let mut size = 0;
        // The source's SSE2/NEON branch is an optimization for 8-bit input.
        // StringView currently stores UTF-16 units, so the source slow path
        // preserves token behavior without pretending to have 8-bit storage.
        while size < buffer.length() {
            let cc = buffer[size as usize];
            if IsSurrogate(cc) {
                if IsLeadingSurrogate(cc)
                    && size + 1 < buffer.length()
                    && IsTrailingSurrogate(buffer[(size + 1) as usize])
                {
                    size += 2;
                    continue;
                }
                let name = ConsumeCSSName(&mut self.input_);
                return self.RegisterString(&name);
            }
            if !IsNameCodePoint(cc) {
                if cc == 0 || cc == 0x5C {
                    let name = ConsumeCSSName(&mut self.input_);
                    return self.RegisterString(&name);
                }
                self.input_.Advance(size);
                return buffer.Substring(0, size);
            }
            size += 1;
        }
        self.input_.Advance(size);
        buffer
    }

    // cpp: css_parser/css_tokenizer.cc:688-696
    fn ConsumeEscape(&mut self) -> UChar32 {
        ConsumeCSSEscape(&mut self.input_)
    }
    fn NextTwoCharsAreValidEscape(&self) -> bool {
        TwoCharsAreValidEscape(
            self.input_.PeekWithoutReplacement(0),
            self.input_.PeekWithoutReplacement(1),
        )
    }

    // cpp: css_parser/css_tokenizer.cc:698-719
    fn NextCharsAreNumberWithFirst(&self, first: UChar) -> bool {
        let second = self.input_.PeekWithoutReplacement(0);
        if ascii_digit(first) {
            return true;
        }
        if first == 0x2B || first == 0x2D {
            return ascii_digit(second)
                || (second == 0x2E && ascii_digit(self.input_.PeekWithoutReplacement(1)));
        }
        if first == 0x2E {
            return ascii_digit(second);
        }
        false
    }
    fn NextCharsAreNumber(&mut self) -> bool {
        let first = self.Consume();
        let are_number = self.NextCharsAreNumberWithFirst(first);
        self.Reconsume(first);
        are_number
    }

    // cpp: css_parser/css_tokenizer.cc:721-734
    fn NextCharsAreIdentifierWithFirst(&self, first: UChar) -> bool {
        CSSNextCharsAreIdentifier(first, &self.input_)
    }
    fn NextCharsAreIdentifier(&mut self) -> bool {
        if self.input_.AtEnd() {
            return false;
        }
        let first = self.Consume();
        let are_identifier = self.NextCharsAreIdentifierWithFirst(first);
        self.Reconsume(first);
        are_identifier
    }

    // cpp: css_parser/css_tokenizer.cc:736-739
    fn RegisterString(&mut self, string: &String) -> StringView {
        self.string_pool_.push(string.clone());
        StringView::from(self.string_pool_.last().expect("string just registered"))
    }
}
