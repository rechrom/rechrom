// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//! Blink tokenizer state machine, using the translated input-stream numeric conversion.
#![allow(non_snake_case)]

use super::css_parser_idioms as idioms;
use super::css_parser_idioms::{
    IsCSSNewLine, IsHTMLSpace, IsLeadingSurrogate, IsNameCodePoint, IsSurrogate,
    IsTrailingSurrogate, TwoCharsAreValidEscape,
};
use super::css_parser_token::CSSParserTokenType::*;
use super::css_parser_token::{
    BlockType, CSSParserToken, CSSParserTokenType, HashTokenType, NumericSign, NumericValueType,
};
use super::css_tokenizer_input_stream::CSSTokenizerInputStream;
use foundation::{CSSValueID, String, StringView};

// cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.h:19-143
pub struct CSSTokenizer {
    input_: CSSTokenizerInputStream,
    block_stack_: Vec<CSSParserTokenType>,
    string_pool_: Vec<String>,
    prev_offset_: u32,
    token_count_: u32,
    unicode_ranges_allowed_: bool,
}

const _: () = assert!(kLeftParenthesisToken as u8 + 1 == kRightParenthesisToken as u8);
const _: () = assert!(kLeftBracketToken as u8 + 1 == kRightBracketToken as u8);
const _: () = assert!(kLeftBraceToken as u8 + 1 == kRightBraceToken as u8);

impl CSSTokenizer {
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:22-33
    pub fn new(string: StringView, offset: u32) -> Self {
        let mut input = CSSTokenizerInputStream::new(string);
        input.AdvanceBy(offset);
        Self {
            input_: input,
            block_stack_: Vec::new(),
            string_pool_: Vec::new(),
            prev_offset_: 0,
            token_count_: 0,
            unicode_ranges_allowed_: false,
        }
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.h:31-35
    pub fn Offset(&self) -> u32 {
        self.input_.Offset()
    }
    pub fn PreviousOffset(&self) -> u32 {
        self.prev_offset_
    }
    pub fn StringPool(&self) -> &[String] {
        &self.string_pool_
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:35-37
    pub fn StringRangeFrom(&self, start: u32) -> StringView {
        self.input_.RangeFrom(start)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:39-42
    pub fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
        self.input_.RangeAt(start, length)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:44-46
    pub fn TokenizeSingle(&mut self) -> CSSParserToken {
        self.NextToken::<true>()
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:48-50
    pub fn TokenizeSingleWithComments(&mut self) -> CSSParserToken {
        self.NextToken::<false>()
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:52-54
    pub fn TokenCount(&self) -> u32 {
        self.token_count_
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.h:44-68
    pub fn SkipToEndOfBlock(&mut self, offset: u32) {
        debug_assert!(offset > self.input_.Offset());
        #[cfg(debug_assertions)]
        {
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
        }
        #[cfg(not(debug_assertions))]
        {
            self.block_stack_
                .pop()
                .expect("SkipToEndOfBlock requires an open block");
        }
        self.input_.Restore(offset);
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.h:72-88
    pub fn Restore(&mut self, next: &CSSParserToken, offset: u32) -> CSSParserToken {
        if next.GetBlockType() == BlockType::kBlockStart {
            self.block_stack_
                .pop()
                .expect("restore must undo a block start");
        } else if next.GetBlockType() == BlockType::kBlockEnd {
            self.block_stack_.push(match next.GetType() {
                kRightParenthesisToken => kLeftParenthesisToken,
                kRightBracketToken => kLeftBracketToken,
                kRightBraceToken => kLeftBraceToken,
                _ => unreachable!("only closing delimiters have kBlockEnd"),
            });
        }
        self.input_.Restore(offset);
        self.TokenizeSingle()
    }
    // CSSParserTokenStream is the source friend that writes this field. This
    // accessor is only the Rust boundary for that same operation.
    pub(crate) fn SetUnicodeRangesAllowed(&mut self, allowed: bool) {
        self.unicode_ranges_allowed_ = allowed;
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:56-58
    fn Reconsume(&mut self, c: u16) {
        self.input_.PushBack(c);
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:60-64
    fn Consume(&mut self) -> u16 {
        let current = self.input_.NextInputChar();
        self.input_.Advance();
        current
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:66-69
    fn BlockStart(&mut self, token_type: CSSParserTokenType) -> CSSParserToken {
        self.block_stack_.push(token_type);
        CSSParserToken::new(token_type, BlockType::kBlockStart)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:71-78
    fn BlockStartNamed(
        &mut self,
        block_type: CSSParserTokenType,
        token_type: CSSParserTokenType,
        name: StringView,
        id: CSSValueID,
    ) -> CSSParserToken {
        self.block_stack_.push(block_type);
        CSSParserToken::WithValue(token_type, name, BlockType::kBlockStart, Some(id))
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:80-87
    fn BlockEnd(
        &mut self,
        token_type: CSSParserTokenType,
        start_type: CSSParserTokenType,
    ) -> CSSParserToken {
        if self.block_stack_.last() == Some(&start_type) {
            self.block_stack_.pop();
            return CSSParserToken::new(token_type, BlockType::kBlockEnd);
        }
        PlainToken(token_type)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:89-104
    fn HyphenMinus(&mut self, cc: u16) -> CSSParserToken {
        if self.NextCharsAreNumberStartingWith(cc) {
            self.Reconsume(cc);
            return self.ConsumeNumericToken();
        }
        if self.input_.PeekWithoutReplacement(0) == b'-' as u16
            && self.input_.PeekWithoutReplacement(1) == b'>' as u16
        {
            self.input_.AdvanceBy(2);
            return PlainToken(kCDCToken);
        }
        if self.NextCharsAreIdentifierStartingWith(cc) {
            self.Reconsume(cc);
            return self.ConsumeIdentLikeToken();
        }
        DelimiterToken(cc)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:106-116
    fn Hash(&mut self, cc: u16) -> CSSParserToken {
        let next_char = self.input_.NextInputChar();
        if IsNameCodePoint(next_char)
            || TwoCharsAreValidEscape(next_char, self.input_.PeekWithoutReplacement(1))
        {
            let hash_type = if self.NextCharsAreIdentifier() {
                HashTokenType::kHashTokenId
            } else {
                HashTokenType::kHashTokenUnrestricted
            };
            return CSSParserToken::WithHash(hash_type, self.ConsumeName());
        }
        DelimiterToken(cc)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:118-127
    fn LetterU(&mut self, cc: u16) -> CSSParserToken {
        if self.unicode_ranges_allowed_
            && self.input_.PeekWithoutReplacement(0) == b'+' as u16
            && (IsAsciiHexDigit(self.input_.PeekWithoutReplacement(1))
                || self.input_.PeekWithoutReplacement(1) == b'?' as u16)
        {
            self.input_.Advance();
            return self.ConsumeUnicodeRange();
        }
        self.Reconsume(cc);
        self.ConsumeIdentLikeToken()
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:129-298
    fn NextToken<const SKIP_COMMENTS: bool>(&mut self) -> CSSParserToken {
        loop {
            self.prev_offset_ = self.input_.Offset();
            let cc = self.Consume();
            self.token_count_ = self.token_count_.wrapping_add(1);
            match cc {
                0 => return PlainToken(kEOFToken),
                0x09 | 0x0a | 0x0c | 0x0d | 0x20 => {
                    self.input_.AdvanceUntilNonWhitespace();
                    return PlainToken(kWhitespaceToken);
                }
                0x27 | 0x22 => return self.ConsumeStringTokenUntil(cc),
                0x30..=0x39 => {
                    self.Reconsume(cc);
                    return self.ConsumeNumericToken();
                }
                0x28 => return self.BlockStart(kLeftParenthesisToken),
                0x29 => return self.BlockEnd(kRightParenthesisToken, kLeftParenthesisToken),
                0x5b => return self.BlockStart(kLeftBracketToken),
                0x5d => return self.BlockEnd(kRightBracketToken, kLeftBracketToken),
                0x7b => return self.BlockStart(kLeftBraceToken),
                0x7d => return self.BlockEnd(kRightBraceToken, kLeftBraceToken),
                0x2b | 0x2e => {
                    if self.NextCharsAreNumberStartingWith(cc) {
                        self.Reconsume(cc);
                        return self.ConsumeNumericToken();
                    }
                    return DelimiterToken(cc);
                }
                0x2d => return self.HyphenMinus(cc),
                0x2a => {
                    return if self.ConsumeIfNext(b'=' as u16) {
                        PlainToken(kSubstringMatchToken)
                    } else {
                        DelimiterToken(cc)
                    }
                }
                0x3c => {
                    if self.input_.PeekWithoutReplacement(0) == b'!' as u16
                        && self.input_.PeekWithoutReplacement(1) == b'-' as u16
                        && self.input_.PeekWithoutReplacement(2) == b'-' as u16
                    {
                        self.input_.AdvanceBy(3);
                        return PlainToken(kCDOToken);
                    }
                    return DelimiterToken(cc);
                }
                0x2c => return PlainToken(kCommaToken),
                0x2f => {
                    if self.ConsumeIfNext(b'*' as u16) {
                        self.ConsumeUntilCommentEndFound();
                        if SKIP_COMMENTS {
                            continue;
                        }
                        return PlainToken(kCommentToken);
                    }
                    return DelimiterToken(cc);
                }
                0x5c => {
                    if TwoCharsAreValidEscape(cc, self.input_.PeekWithoutReplacement(0)) {
                        self.Reconsume(cc);
                        return self.ConsumeIdentLikeToken();
                    }
                    return DelimiterToken(cc);
                }
                0x3a => return PlainToken(kColonToken),
                0x3b => return PlainToken(kSemicolonToken),
                0x23 => return self.Hash(cc),
                0x5e => {
                    return if self.ConsumeIfNext(b'=' as u16) {
                        PlainToken(kPrefixMatchToken)
                    } else {
                        DelimiterToken(cc)
                    }
                }
                0x24 => {
                    return if self.ConsumeIfNext(b'=' as u16) {
                        PlainToken(kSuffixMatchToken)
                    } else {
                        DelimiterToken(cc)
                    }
                }
                0x7c => {
                    if self.ConsumeIfNext(b'=' as u16) {
                        return PlainToken(kDashMatchToken);
                    }
                    if self.ConsumeIfNext(b'|' as u16) {
                        return PlainToken(kColumnToken);
                    }
                    return DelimiterToken(cc);
                }
                0x7e => {
                    return if self.ConsumeIfNext(b'=' as u16) {
                        PlainToken(kIncludeMatchToken)
                    } else {
                        DelimiterToken(cc)
                    }
                }
                0x40 => {
                    if self.NextCharsAreIdentifier() {
                        let name = self.ConsumeName();
                        return CSSParserToken::WithValue(
                            kAtKeywordToken,
                            name,
                            BlockType::kNotBlock,
                            None,
                        );
                    }
                    return DelimiterToken(cc);
                }
                0x75 | 0x55 => return self.LetterU(cc),
                1..=8 | 11 | 14..=31 | 0x21 | 0x25 | 0x26 | 0x3d | 0x3e | 0x3f | 0x60 | 127 => {
                    return DelimiterToken(cc)
                }
                _ => {
                    self.Reconsume(cc);
                    return self.ConsumeIdentLikeToken();
                }
            }
        }
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:303-360
    fn ConsumeNumber(&mut self) -> CSSParserToken {
        debug_assert!(self.NextCharsAreNumber());
        let mut value_type = NumericValueType::kIntegerValueType;
        let mut sign = NumericSign::kNoSign;
        let mut number_length = 0;
        let mut sign_length = 0;
        let mut next = self.input_.PeekWithoutReplacement(0);
        if next == b'+' as u16 {
            number_length += 1;
            sign_length += 1;
            sign = NumericSign::kPlusSign;
        } else if next == b'-' as u16 {
            number_length += 1;
            sign_length += 1;
            sign = NumericSign::kMinusSign;
        }
        number_length = self.input_.SkipWhilePredicate(number_length, IsAsciiDigit);
        next = self.input_.PeekWithoutReplacement(number_length);
        if next == b'.' as u16
            && IsAsciiDigit(self.input_.PeekWithoutReplacement(number_length + 1))
        {
            value_type = NumericValueType::kNumberValueType;
            number_length = self
                .input_
                .SkipWhilePredicate(number_length + 2, IsAsciiDigit);
            next = self.input_.PeekWithoutReplacement(number_length);
        }
        if next == b'E' as u16 || next == b'e' as u16 {
            next = self.input_.PeekWithoutReplacement(number_length + 1);
            if IsAsciiDigit(next) {
                value_type = NumericValueType::kNumberValueType;
                number_length = self
                    .input_
                    .SkipWhilePredicate(number_length + 1, IsAsciiDigit);
            } else if (next == b'+' as u16 || next == b'-' as u16)
                && IsAsciiDigit(self.input_.PeekWithoutReplacement(number_length + 2))
            {
                value_type = NumericValueType::kNumberValueType;
                number_length = self
                    .input_
                    .SkipWhilePredicate(number_length + 3, IsAsciiDigit);
            }
        }
        let value = if value_type == NumericValueType::kIntegerValueType {
            let mut value = self
                .input_
                .GetNaturalNumberAsDouble(sign_length, number_length);
            if sign == NumericSign::kMinusSign {
                value = -value;
            }
            debug_assert_eq!(value, self.input_.GetDouble(0, number_length));
            self.input_.AdvanceBy(number_length);
            value
        } else {
            let value = self.input_.GetDouble(0, number_length);
            self.input_.AdvanceBy(number_length);
            value
        };
        CSSParserToken::WithNumber(kNumberToken, value, value_type, sign)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:363-371
    fn ConsumeNumericToken(&mut self) -> CSSParserToken {
        let mut token = self.ConsumeNumber();
        if self.NextCharsAreIdentifier() {
            token.ConvertToDimensionWithUnit(self.ConsumeName());
        } else if self.ConsumeIfNext(b'%' as u16) {
            token.ConvertToPercentage();
        }
        token
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:374-390
    fn ConsumeIdentLikeToken(&mut self) -> CSSParserToken {
        let name = self.ConsumeName();
        if self.ConsumeIfNext(b'(' as u16) {
            if EqualsUrlIgnoringAsciiCase(&name) {
                self.input_.AdvanceUntilNonWhitespace();
                let next = self.input_.PeekWithoutReplacement(0);
                if next != b'"' as u16 && next != b'\'' as u16 {
                    return self.ConsumeUrlToken();
                }
            }
            // Id() uses the already translated CssValueKeywordID implementation.
            let id =
                CSSParserToken::WithValue(kIdentToken, name.clone(), BlockType::kNotBlock, None)
                    .Id();
            return self.BlockStartNamed(kLeftParenthesisToken, kFunctionToken, name, id);
        }
        CSSParserToken::WithValue(kIdentToken, name, BlockType::kNotBlock, None)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:393-435
    fn ConsumeStringTokenUntil(&mut self, ending_code_point: u16) -> CSSParserToken {
        let mut size = 0;
        loop {
            let cc = self.input_.PeekWithoutReplacement(size);
            if cc == ending_code_point {
                let start_offset = self.input_.Offset();
                self.input_.AdvanceBy(size + 1);
                return CSSParserToken::WithValue(
                    kStringToken,
                    self.input_.RangeAt(start_offset, size),
                    BlockType::kNotBlock,
                    None,
                );
            }
            if IsCSSNewLine(cc) {
                self.input_.AdvanceBy(size);
                return PlainToken(kBadStringToken);
            }
            if cc == 0 || cc == b'\\' as u16 {
                break;
            }
            size += 1;
        }
        let mut output = Vec::new();
        loop {
            let cc = self.Consume();
            if cc == ending_code_point || cc == 0 {
                let value = self.RegisterString(String::from_utf16(&output));
                return CSSParserToken::WithValue(kStringToken, value, BlockType::kNotBlock, None);
            }
            if IsCSSNewLine(cc) {
                self.Reconsume(cc);
                return PlainToken(kBadStringToken);
            }
            if cc == b'\\' as u16 {
                if self.input_.NextInputChar() == 0 {
                    continue;
                }
                if IsCSSNewLine(self.input_.PeekWithoutReplacement(0)) {
                    self.ConsumeSingleWhitespaceIfNext();
                } else {
                    let escaped = self.ConsumeEscape();
                    AppendCodePoint(&mut output, escaped);
                }
            } else {
                output.push(cc);
            }
        }
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:437-469
    fn ConsumeUnicodeRange(&mut self) -> CSSParserToken {
        debug_assert!(
            IsAsciiHexDigit(self.input_.PeekWithoutReplacement(0))
                || self.input_.PeekWithoutReplacement(0) == b'?' as u16
        );
        let mut length_remaining = 6;
        let mut start: i32 = 0;
        while length_remaining != 0 && IsAsciiHexDigit(self.input_.PeekWithoutReplacement(0)) {
            start = start * 16 + ToAsciiHexValue(self.Consume());
            length_remaining -= 1;
        }
        let mut end = start;
        if length_remaining != 0 && self.ConsumeIfNext(b'?' as u16) {
            loop {
                start *= 16;
                end = end * 16 + 0xf;
                length_remaining -= 1;
                if length_remaining == 0 || !self.ConsumeIfNext(b'?' as u16) {
                    break;
                }
            }
        } else if self.input_.PeekWithoutReplacement(0) == b'-' as u16
            && IsAsciiHexDigit(self.input_.PeekWithoutReplacement(1))
        {
            self.input_.Advance();
            length_remaining = 6;
            end = 0;
            loop {
                end = end * 16 + ToAsciiHexValue(self.Consume());
                length_remaining -= 1;
                if length_remaining == 0 || !IsAsciiHexDigit(self.input_.PeekWithoutReplacement(0))
                {
                    break;
                }
            }
        }
        CSSParserToken::WithUnicodeRange(kUnicodeRangeToken, start, end)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:478-528
    fn ConsumeUrlToken(&mut self) -> CSSParserToken {
        self.input_.AdvanceUntilNonWhitespace();
        let mut size = 0;
        loop {
            let cc = self.input_.PeekWithoutReplacement(size);
            if cc == b')' as u16 {
                let start_offset = self.input_.Offset();
                self.input_.AdvanceBy(size + 1);
                return CSSParserToken::WithValue(
                    kUrlToken,
                    self.input_.RangeAt(start_offset, size),
                    BlockType::kNotBlock,
                    None,
                );
            }
            if cc <= b' ' as u16
                || cc == b'\\' as u16
                || cc == b'"' as u16
                || cc == b'\'' as u16
                || cc == b'(' as u16
                || cc == 0x7f
            {
                break;
            }
            size += 1;
        }
        let mut result = Vec::new();
        loop {
            let cc = self.Consume();
            if cc == b')' as u16 || cc == 0 {
                let value = self.RegisterString(String::from_utf16(&result));
                return CSSParserToken::WithValue(kUrlToken, value, BlockType::kNotBlock, None);
            }
            if IsHTMLSpace(cc) {
                self.input_.AdvanceUntilNonWhitespace();
                if self.ConsumeIfNext(b')' as u16) || self.input_.NextInputChar() == 0 {
                    let value = self.RegisterString(String::from_utf16(&result));
                    return CSSParserToken::WithValue(kUrlToken, value, BlockType::kNotBlock, None);
                }
                break;
            }
            if cc == b'"' as u16
                || cc == b'\'' as u16
                || cc == b'(' as u16
                || IsNonPrintableCodePoint(cc)
            {
                break;
            }
            if cc == b'\\' as u16 {
                if TwoCharsAreValidEscape(cc, self.input_.PeekWithoutReplacement(0)) {
                    let escaped = self.ConsumeEscape();
                    AppendCodePoint(&mut result, escaped);
                    continue;
                }
                break;
            }
            result.push(cc);
        }
        self.ConsumeBadUrlRemnants();
        PlainToken(kBadUrlToken)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:531-541
    fn ConsumeBadUrlRemnants(&mut self) {
        loop {
            let cc = self.Consume();
            if cc == b')' as u16 || cc == 0 {
                return;
            }
            if TwoCharsAreValidEscape(cc, self.input_.PeekWithoutReplacement(0)) {
                self.ConsumeEscape();
            }
        }
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:543-545
    fn ConsumeSingleWhitespaceIfNext(&mut self) {
        idioms::ConsumeSingleWhitespaceIfNext(&mut self.input_);
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:547-562
    fn ConsumeUntilCommentEndFound(&mut self) {
        let mut c = self.Consume();
        loop {
            if c == 0 {
                return;
            }
            if c != b'*' as u16 {
                c = self.Consume();
                continue;
            }
            c = self.Consume();
            if c == b'/' as u16 {
                return;
            }
        }
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:564-574
    fn ConsumeIfNext(&mut self, character: u16) -> bool {
        debug_assert_ne!(character, 0);
        if self.input_.PeekWithoutReplacement(0) == character {
            self.input_.Advance();
            return true;
        }
        false
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:594-686
    fn ConsumeName(&mut self) -> StringView {
        let buffer = self.input_.Peek();
        let mut size = 0;
        #[cfg(any(target_feature = "sse2", target_feature = "neon"))]
        if let Some(bytes) = buffer.Span8() {
            while size + 16 <= buffer.length() {
                // Same 16-lane mask as source SSE2/NEON: non-name ASCII lanes
                // become one bits; non-ASCII lanes are always name characters.
                // Scalar mask formation preserves the intrinsic branch results.
                let mut bits: u16 = 0;
                for lane in 0..16 {
                    let byte = bytes[(size + lane) as usize];
                    if byte < 0x80 && !IsNameCodePoint(u16::from(byte)) {
                        bits |= 1 << lane;
                    }
                }
                if bits == 0 {
                    size += 16;
                    continue;
                }
                size += bits.trailing_zeros();
                if bytes[size as usize] == 0 || bytes[size as usize] == b'\\' {
                    let name = idioms::ConsumeName(&mut self.input_);
                    return self.RegisterString(name);
                }
                self.input_.AdvanceBy(size);
                return buffer.Substring(0, size);
            }
        }
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
                let name = idioms::ConsumeName(&mut self.input_);
                return self.RegisterString(name);
            }
            if !IsNameCodePoint(cc) {
                if cc == 0 || cc == b'\\' as u16 {
                    let name = idioms::ConsumeName(&mut self.input_);
                    return self.RegisterString(name);
                }
                self.input_.AdvanceBy(size);
                return buffer.Substring(0, size);
            }
            size += 1;
        }
        self.input_.AdvanceBy(size);
        buffer
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:689-691
    fn ConsumeEscape(&mut self) -> u32 {
        idioms::ConsumeEscape(&mut self.input_)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:693-696
    fn NextTwoCharsAreValidEscape(&self) -> bool {
        TwoCharsAreValidEscape(
            self.input_.PeekWithoutReplacement(0),
            self.input_.PeekWithoutReplacement(1),
        )
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:699-712
    fn NextCharsAreNumberStartingWith(&self, first: u16) -> bool {
        let second = self.input_.PeekWithoutReplacement(0);
        if IsAsciiDigit(first) {
            return true;
        }
        if first == b'+' as u16 || first == b'-' as u16 {
            return IsAsciiDigit(second)
                || (second == b'.' as u16 && IsAsciiDigit(self.input_.PeekWithoutReplacement(1)));
        }
        if first == b'.' as u16 {
            return IsAsciiDigit(second);
        }
        false
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:714-719
    fn NextCharsAreNumber(&mut self) -> bool {
        let first = self.Consume();
        let are_number = self.NextCharsAreNumberStartingWith(first);
        self.Reconsume(first);
        are_number
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:722-724
    fn NextCharsAreIdentifierStartingWith(&self, first: u16) -> bool {
        idioms::NextCharsAreIdentifier(first, &self.input_)
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:726-734
    fn NextCharsAreIdentifier(&mut self) -> bool {
        if self.input_.AtEnd() {
            return false;
        }
        let first = self.Consume();
        let are_identifier = self.NextCharsAreIdentifierStartingWith(first);
        self.Reconsume(first);
        are_identifier
    }
    // cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:736-739
    fn RegisterString(&mut self, string: String) -> StringView {
        let value = StringView::from(&string);
        self.string_pool_.push(string);
        value
    }
}

// Rust ownership adapter for the actual source friend CSSParserTokenStream.
// cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.h:25-36,44-88,131-143
// cpp dependency: third_party/blink/renderer/core/css/parser/css_parser_token_stream.h:143-149,695
impl super::css_parser_token_stream::TokenStreamTokenizer for CSSTokenizer {
    fn new(text: StringView, offset: u32) -> Self {
        CSSTokenizer::new(text, offset)
    }
    fn TokenizeSingle(&mut self) -> CSSParserToken {
        CSSTokenizer::TokenizeSingle(self)
    }
    fn TokenizeSingleWithComments(&mut self) -> CSSParserToken {
        CSSTokenizer::TokenizeSingleWithComments(self)
    }
    fn Offset(&self) -> u32 {
        CSSTokenizer::Offset(self)
    }
    fn PreviousOffset(&self) -> u32 {
        CSSTokenizer::PreviousOffset(self)
    }
    fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
        CSSTokenizer::StringRangeAt(self, start, length)
    }
    fn StringRangeFrom(&self, start: u32) -> StringView {
        CSSTokenizer::StringRangeFrom(self, start)
    }
    fn SkipToEndOfBlock(&mut self, offset: u32) {
        CSSTokenizer::SkipToEndOfBlock(self, offset);
    }
    fn Restore(&mut self, next: &CSSParserToken, offset: u32) -> CSSParserToken {
        CSSTokenizer::Restore(self, next, offset)
    }
    fn TokenCount(&self) -> u32 {
        CSSTokenizer::TokenCount(self)
    }
    fn UnicodeRangesAllowed(&self) -> bool {
        self.unicode_ranges_allowed_
    }
    fn SetUnicodeRangesAllowed(&mut self, allowed: bool) {
        CSSTokenizer::SetUnicodeRangesAllowed(self, allowed);
    }
    fn PopBlockStack(&mut self) {
        self.block_stack_
            .pop()
            .expect("source stream requires an open block");
    }
}

fn PlainToken(token_type: CSSParserTokenType) -> CSSParserToken {
    CSSParserToken::new(token_type, BlockType::kNotBlock)
}
fn DelimiterToken(cc: u16) -> CSSParserToken {
    CSSParserToken::WithDelimiter(kDelimiterToken, cc)
}
fn EqualsUrlIgnoringAsciiCase(name: &StringView) -> bool {
    name.length() == 3
        && name[0] | 0x20 == b'u' as u16
        && name[1] | 0x20 == b'r' as u16
        && name[2] | 0x20 == b'l' as u16
}
// cpp: third_party/blink/renderer/core/css/parser/css_tokenizer.cc:472-475
fn IsNonPrintableCodePoint(cc: u16) -> bool {
    cc <= 8 || cc == 11 || (14..=31).contains(&cc) || cc == 127
}
// C++ dependency: platform/wtf/ASCIICType.h IsAsciiDigit/IsAsciiHexDigit/ToAsciiHexValue.
fn IsAsciiDigit(cc: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&cc)
}
fn IsAsciiHexDigit(cc: u16) -> bool {
    IsAsciiDigit(cc) || (b'a' as u16..=b'f' as u16).contains(&(cc | 0x20))
}
fn ToAsciiHexValue(cc: u16) -> i32 {
    debug_assert!(IsAsciiHexDigit(cc));
    if IsAsciiDigit(cc) {
        i32::from(cc - b'0' as u16)
    } else {
        i32::from((cc | 0x20) - b'a' as u16) + 10
    }
}
fn AppendCodePoint(output: &mut Vec<u16>, c: u32) {
    // StringBuilder::Append(UChar32), WTF UTF-16 encoding; lone surrogates stay intact.
    if c <= 0xffff {
        output.push(c as u16);
    } else {
        let c = c - 0x10000;
        output.push(0xd800 + (c >> 10) as u16);
        output.push(0xdc00 + (c & 0x3ff) as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use NumericSign::*;
    use NumericValueType::*;

    fn tokenize(text: &str) -> CSSTokenizer {
        CSSTokenizer::new(StringView::from(&String::FromUtf8(text.as_bytes())), 0)
    }
    fn value(token: &CSSParserToken) -> std::string::String {
        token.Value().ToString().Utf8()
    }
    fn text_token(text: &str, token_type: CSSParserTokenType, expected: &str) {
        let mut tokenizer = tokenize(text);
        let token = tokenizer.TokenizeSingle();
        assert_eq!(token.GetType(), token_type, "{text:?}");
        assert_eq!(value(&token), expected, "{text:?}");
        assert!(tokenizer.TokenizeSingle().IsEOF(), "{text:?}");
    }
    // cpp test vectors: css_tokenizer_test.cc:171-219,506-520
    #[test]
    fn punctuation_comments_offsets_and_token_count_match_source() {
        for (text, expected) in [
            ("(", kLeftParenthesisToken),
            (")", kRightParenthesisToken),
            ("[", kLeftBracketToken),
            ("]", kRightBracketToken),
            (",", kCommaToken),
            (":", kColonToken),
            (";", kSemicolonToken),
            ("~=", kIncludeMatchToken),
            ("|=", kDashMatchToken),
            ("^=", kPrefixMatchToken),
            ("$=", kSuffixMatchToken),
            ("*=", kSubstringMatchToken),
            ("||", kColumnToken),
            ("<!--", kCDOToken),
            ("-->", kCDCToken),
        ] {
            let mut tokenizer = tokenize(text);
            assert_eq!(tokenizer.TokenizeSingle().GetType(), expected, "{text}");
            assert!(tokenizer.TokenizeSingle().IsEOF());
        }
        let mut tokenizer = tokenize("/*comment*/a");
        assert_eq!(value(&tokenizer.TokenizeSingle()), "a");
        assert_eq!(tokenizer.TokenCount(), 2);
        assert_eq!(tokenizer.PreviousOffset(), 11);
        assert_eq!(tokenizer.Offset(), 12);
        assert_eq!(tokenizer.StringRangeFrom(11).ToString().Utf8(), "a");
        assert_eq!(tokenizer.StringRangeAt(0, 2).ToString().Utf8(), "/*");
        assert!(tokenizer.StringPool().is_empty());
        let mut comments = tokenize("/**y*a*y**/ ");
        assert_eq!(
            comments.TokenizeSingleWithComments().GetType(),
            kCommentToken
        );
        assert_eq!(
            comments.TokenizeSingleWithComments().GetType(),
            kWhitespaceToken
        );
        assert_eq!(comments.TokenizeSingleWithComments().GetType(), kEOFToken);
        assert_eq!(comments.TokenCount(), 3);
        let mut comments = tokenize("/**\\2f**//");
        assert_eq!(comments.TokenizeSingle().Delimiter(), b'/' as u16);
        assert!(comments.TokenizeSingle().IsEOF());
        assert_eq!(
            tokenize("/*unterminated").TokenizeSingle().GetType(),
            kEOFToken
        );
        let mut offset = CSSTokenizer::new(StringView::from("ignore ident"), 7);
        assert_eq!(value(&offset.TokenizeSingle()), "ident");
    }
    // cpp test vectors: css_tokenizer_test.cc:220-312
    #[test]
    fn escaped_names_simd_boundaries_nulls_and_surrogates_match_source() {
        for (text, expected) in [
            (r"hel\6Co", "hello"),
            (r"\26 B", "&B"),
            ("spac\\65\r\ns", "spaces"),
            ("sp\\61\tc\\65\u{c}s", "spaces"),
            (r"te\s\t", "test"),
            (r"\.\,\:\!", ".,:!"),
            (r"null\0000", "null�"),
            (r"large\110000", "large�"),
            (r"surrogate\D800", "surrogate�"),
            (r"eof\", "eof�"),
            (r"\10fFfF0", "\u{10ffff}0"),
            (r"abcdefghijklmnopqr\61suffix", "abcdefghijklmnopqrasuffix"),
        ] {
            text_token(text, kIdentToken, expected);
        }
        let mut tokenizer = tokenize("abcdefghijklmnopqr!suffix");
        assert_eq!(value(&tokenizer.TokenizeSingle()), "abcdefghijklmnopqr");
        assert_eq!(tokenizer.TokenizeSingle().Delimiter(), b'!' as u16);
        assert!(tokenizer.StringPool().is_empty());
        let mut tokenizer = tokenize(r"abcdefghijklmnopqr\61suffix");
        assert_eq!(
            value(&tokenizer.TokenizeSingle()),
            "abcdefghijklmnopqrasuffix"
        );
        assert_eq!(tokenizer.StringPool().len(), 1);
        for (units, expected) in [
            (
                vec![b'a' as u16, 0, b'b' as u16],
                vec![b'a' as u16, 0xfffd, b'b' as u16],
            ),
            (
                vec![b'a' as u16, 0xd800, b'b' as u16],
                vec![b'a' as u16, 0xfffd, b'b' as u16],
            ),
            (
                vec![b'a' as u16, 0xdc00, b'b' as u16],
                vec![b'a' as u16, 0xfffd, b'b' as u16],
            ),
            (
                vec![b'a' as u16, 0xd83d, 0xde00, b'b' as u16],
                vec![b'a' as u16, 0xd83d, 0xde00, b'b' as u16],
            ),
        ] {
            let source = String::from_utf16(&units);
            let mut tokenizer = CSSTokenizer::new(StringView::from(&source), 0);
            assert_eq!(tokenizer.TokenizeSingle().Value().Span16(), expected);
            assert!(tokenizer.TokenizeSingle().IsEOF());
        }
        let mut tokenizer = tokenize("test\\\n");
        assert_eq!(value(&tokenizer.TokenizeSingle()), "test");
        assert_eq!(tokenizer.TokenizeSingle().Delimiter(), b'\\' as u16);
        assert_eq!(tokenizer.TokenizeSingle().GetType(), kWhitespaceToken);
        let tokenizer = tokenize(r"\x");
        assert!(tokenizer.NextTwoCharsAreValidEscape());
    }
    // cpp test vectors: css_tokenizer_test.cc:313-439
    #[test]
    fn strings_urls_functions_and_hashes_match_source() {
        for (text, expected) in [
            ("'text'", "text"),
            ("\"mismatch'", "mismatch'"),
            ("'esca\\\nped'", "escaped"),
            ("\"new\\\r\nline\"", "newline"),
            ("'hel\0lo'", "hel�lo"),
            ("'h\\65l\0lo'", "hel�lo"),
        ] {
            text_token(text, kStringToken, expected);
        }
        for (text, expected) in [
            ("url(foo.gif)", "foo.gif"),
            (
                "urL(https://example.com/cats.png)",
                "https://example.com/cats.png",
            ),
            (
                r"uRl(what-a.crazy^URL~this\ is!)",
                "what-a.crazy^URL~this is!",
            ),
            ("UrL(   whitespace   )", "whitespace"),
            ("URl( whitespace-eof ", "whitespace-eof"),
            ("URL(eof", "eof"),
            ("url(not/*a*/comment)", "not/*a*/comment"),
            ("urL()", ""),
        ] {
            text_token(text, kUrlToken, expected);
        }
        for (text, trailing) in [
            ("uRl(white space),", kCommaToken),
            ("Url(b(ad),", kCommaToken),
            ("uRl(ba'd):", kColonToken),
            ("Url(b\\\rad):", kColonToken),
            ("url(b\\\nad):", kColonToken),
        ] {
            let mut tokenizer = tokenize(text);
            assert_eq!(
                tokenizer.TokenizeSingle().GetType(),
                kBadUrlToken,
                "{text:?}"
            );
            assert_eq!(tokenizer.TokenizeSingle().GetType(), trailing);
        }
        let mut tokenizer = tokenize("'bad\r\nstring");
        assert_eq!(tokenizer.TokenizeSingle().GetType(), kBadStringToken);
        assert_eq!(tokenizer.TokenizeSingle().GetType(), kWhitespaceToken);
        assert_eq!(value(&tokenizer.TokenizeSingle()), "string");
        let mut tokenizer = tokenize("url(  'bar.gif')");
        let function = tokenizer.TokenizeSingle();
        assert_eq!(function.GetType(), kFunctionToken);
        assert_eq!(function.GetBlockType(), BlockType::kBlockStart);
        assert_eq!(function.FunctionId(), Some(CSSValueID::kUrl));
        assert_eq!(value(&tokenizer.TokenizeSingle()), "bar.gif");
        assert_eq!(
            tokenizer.TokenizeSingle().GetBlockType(),
            BlockType::kBlockEnd
        );
        for (text, expected, hash_type) in [
            ("#id-selector", "id-selector", HashTokenType::kHashTokenId),
            ("#3377FF", "3377FF", HashTokenType::kHashTokenUnrestricted),
            (r"#\ ", " ", HashTokenType::kHashTokenId),
            ("#-\0", "-�", HashTokenType::kHashTokenId),
        ] {
            let token = tokenize(text).TokenizeSingle();
            assert_eq!(token.GetType(), kHashToken);
            assert_eq!(value(&token), expected);
            assert_eq!(token.GetHashTokenType(), hash_type);
        }
    }
    // cpp test vectors: css_tokenizer_test.cc:410-469
    #[test]
    fn integer_float_dimension_percentage_and_exponent_boundaries_match_source() {
        for (text, expected, value_type, sign) in [
            ("10", 10.0, kIntegerValueType, kNoSign),
            ("12.0", 12.0, kNumberValueType, kNoSign),
            ("+45.6", 45.6, kNumberValueType, kPlusSign),
            ("-7", -7.0, kIntegerValueType, kMinusSign),
            ("010", 10.0, kIntegerValueType, kNoSign),
            ("10e0", 10.0, kNumberValueType, kNoSign),
            ("12e3", 12000.0, kNumberValueType, kNoSign),
            ("3e+1", 30.0, kNumberValueType, kNoSign),
            ("12E-1", 1.2, kNumberValueType, kNoSign),
            (".7", 0.7, kNumberValueType, kNoSign),
            ("-.3", -0.3, kNumberValueType, kMinusSign),
            ("+637.54e-2", 6.3754, kNumberValueType, kPlusSign),
            ("-12.34E+2", -1234.0, kNumberValueType, kMinusSign),
            (
                "1000000000000000000000000",
                1e24,
                kIntegerValueType,
                kNoSign,
            ),
        ] {
            for source in [
                String::FromUtf8(text.as_bytes()),
                String::from_utf16(&text.encode_utf16().collect::<Vec<_>>()),
            ] {
                let mut tokenizer = CSSTokenizer::new(StringView::from(&source), 0);
                let token = tokenizer.TokenizeSingle();
                assert_eq!(token.GetType(), kNumberToken, "{text}");
                assert_eq!(token.NumericValue(), expected, "{text}");
                assert_eq!(token.GetNumericValueType(), value_type);
                assert_eq!(token.GetNumericSign(), sign);
                assert!(tokenizer.TokenizeSingle().IsEOF());
            }
        }
        for (text, expected, unit, value_type) in [
            ("10px", 10.0, "px", kIntegerValueType),
            ("12.0em", 12.0, "em", kNumberValueType),
            ("5e", 5.0, "e", kIntegerValueType),
            ("5px-2px", 5.0, "px-2px", kIntegerValueType),
            (r"40\70\78", 40.0, "px", kIntegerValueType),
            ("4e3e2", 4000.0, "e2", kNumberValueType),
            ("0x10px", 0.0, "x10px", kIntegerValueType),
        ] {
            let token = tokenize(text).TokenizeSingle();
            assert_eq!(token.GetType(), kDimensionToken, "{text}");
            assert_eq!(token.NumericValue(), expected);
            assert_eq!(value(&token), unit);
            assert_eq!(token.GetNumericValueType(), value_type);
        }
        for (text, expected, value_type) in [
            ("10%", 10.0, kIntegerValueType),
            ("+12.0%", 12.0, kNumberValueType),
            ("-48.99%", -48.99, kNumberValueType),
            ("6e-1%", 0.6, kNumberValueType),
        ] {
            let token = tokenize(text).TokenizeSingle();
            assert_eq!(token.GetType(), kPercentageToken);
            assert_eq!(token.NumericValue(), expected);
            assert_eq!(token.GetNumericValueType(), value_type);
        }
        let mut tokenizer = tokenize("1.e2");
        assert_eq!(tokenizer.TokenizeSingle().NumericValue(), 1.0);
        assert_eq!(tokenizer.TokenizeSingle().Delimiter(), b'.' as u16);
        assert_eq!(value(&tokenizer.TokenizeSingle()), "e2");
        let token = tokenize("-0").TokenizeSingle();
        assert_eq!(token.NumericValue().to_bits(), (-0.0f64).to_bits());
        let token = tokenize("1e9999").TokenizeSingle();
        assert_eq!(token.NumericValue(), f32::MAX as f64);
    }
    // cpp test vectors: css_tokenizer_test.cc:470-504
    #[test]
    fn unicode_ranges_limit_consumption_and_require_explicit_enablement() {
        for (text, start, end) in [
            ("u+012345-123456", 0x12345, 0x123456),
            ("u+222-111", 0x222, 0x111),
            ("U+CafE-d00D", 0xcafe, 0xd00d),
            ("U+2??", 0x200, 0x2ff),
            ("U+ab12??", 0xab1200, 0xab12ff),
            ("u+??????", 0, 0xffffff),
            ("u+??", 0, 0xff),
        ] {
            let mut tokenizer = tokenize(text);
            tokenizer.SetUnicodeRangesAllowed(true);
            let token = tokenizer.TokenizeSingle();
            assert_eq!(token.GetType(), kUnicodeRangeToken);
            assert_eq!(token.UnicodeRangeStart(), start);
            assert_eq!(token.UnicodeRangeEnd(), end);
            assert!(tokenizer.TokenizeSingle().IsEOF());
        }
        let mut tokenizer = tokenize("u+12345678");
        tokenizer.SetUnicodeRangesAllowed(true);
        assert_eq!(tokenizer.TokenizeSingle().UnicodeRangeEnd(), 0x123456);
        assert_eq!(tokenizer.TokenizeSingle().NumericValue(), 78.0);
        let mut tokenizer = tokenize("u+cake");
        tokenizer.SetUnicodeRangesAllowed(true);
        assert_eq!(tokenizer.TokenizeSingle().UnicodeRangeEnd(), 0xca);
        assert_eq!(value(&tokenizer.TokenizeSingle()), "ke");
        let mut tokenizer = tokenize("u+a1?-123");
        tokenizer.SetUnicodeRangesAllowed(true);
        assert_eq!(tokenizer.TokenizeSingle().UnicodeRangeStart(), 0xa10);
        assert_eq!(tokenizer.TokenizeSingle().NumericValue(), -123.0);
        let mut tokenizer = tokenize("u+012345");
        assert_eq!(value(&tokenizer.TokenizeSingle()), "u");
        assert_eq!(tokenizer.TokenizeSingle().NumericValue(), 12345.0);
    }
    #[test]
    fn block_mismatches_restore_and_skip_preserve_stack_transitions() {
        let mut tokenizer = tokenize("([)]}");
        assert_eq!(
            tokenizer.TokenizeSingle().GetBlockType(),
            BlockType::kBlockStart
        );
        let bracket = tokenizer.TokenizeSingle();
        assert_eq!(bracket.GetBlockType(), BlockType::kBlockStart);
        assert_eq!(
            tokenizer.TokenizeSingle().GetBlockType(),
            BlockType::kNotBlock
        );
        let end = tokenizer.TokenizeSingle();
        assert_eq!(end.GetBlockType(), BlockType::kBlockEnd);
        assert_eq!(tokenizer.block_stack_, [kLeftParenthesisToken]);
        let replayed = tokenizer.Restore(&end, 3);
        assert_eq!(replayed.GetType(), kRightBracketToken);
        assert_eq!(replayed.GetBlockType(), BlockType::kBlockEnd);
        assert_eq!(tokenizer.block_stack_, [kLeftParenthesisToken]);
        let mut tokenizer = tokenize("(a[b])tail");
        let start = tokenizer.TokenizeSingle();
        let replayed = tokenizer.Restore(&start, 0);
        assert_eq!(replayed.GetBlockType(), BlockType::kBlockStart);
        assert_eq!(tokenizer.block_stack_, [kLeftParenthesisToken]);
        tokenizer.SkipToEndOfBlock(6);
        assert_eq!(tokenizer.Offset(), 6);
        assert!(tokenizer.block_stack_.is_empty());
        assert_eq!(value(&tokenizer.TokenizeSingle()), "tail");
    }
}

#[cfg(test)]
mod stream_integration_tests {
    use super::super::css_parser_token_stream::{
        BlockGuard, CSSParserTokenStream, EnableUnicodeRanges,
    };
    use super::*;
    #[test]
    fn actual_stream_guards_retokenize_and_skip_with_real_tokenizer() {
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from("u+23"), 0);
        assert_eq!(stream.Peek().GetType(), kIdentToken);
        {
            let mut enabled = EnableUnicodeRanges::new(&mut stream, true);
            assert_eq!(enabled.Peek().UnicodeRangeStart(), 0x23);
        }
        assert_eq!(stream.Peek().GetType(), kIdentToken);
        assert_eq!(stream.ConsumeRaw().Value().ToString().Utf8(), "u");
        assert_eq!(stream.ConsumeRaw().NumericValue(), 23.0);
        assert!(stream.Peek().IsEOF());
        let mut stream =
            CSSParserTokenStream::<CSSTokenizer>::new(StringView::from("calc(a[b])tail"), 0);
        assert_eq!(stream.Peek().GetType(), kFunctionToken);
        {
            let mut block = BlockGuard::new(&mut stream);
            assert_eq!(block.Peek().Value().ToString().Utf8(), "a");
        }
        assert_eq!(stream.Peek().Value().ToString().Utf8(), "tail");
    }
}
