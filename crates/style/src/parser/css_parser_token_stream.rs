// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_parser_token_stream.h:53-706
// cpp: third_party/blink/renderer/core/css/parser/css_parser_token_stream.cc:9-75
// Complete stream logic over actual CSSParserToken. TokenStreamTokenizer is an
// explicit boundary for the source CSSTokenizer dependency: every operation is
// required, and this module supplies no substitute CSS tokenization semantics.
// A real CSSTokenizer adapter remains necessary to parse CSS text.

#![allow(non_snake_case)]

use super::css_parser_token::{BlockType, CSSParserToken, CSSParserTokenType};
use foundation::StringView;
#[cfg(debug_assertions)]
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use CSSParserTokenType::*;

// Dependency boundary: parser/css_tokenizer.h:25-36,47-85,129,136-143.
// These are exactly the methods and internal fields used by the source stream.
// The trait does not infer token kinds, tokenize text, or repair block stacks.
pub trait TokenStreamTokenizer {
    fn new(text: StringView, offset: u32) -> Self;
    fn TokenizeSingle(&mut self) -> CSSParserToken;
    fn TokenizeSingleWithComments(&mut self) -> CSSParserToken;
    fn Offset(&self) -> u32;
    fn PreviousOffset(&self) -> u32;
    fn StringRangeAt(&self, start: u32, length: u32) -> StringView;
    fn StringRangeFrom(&self, start: u32) -> StringView;
    fn SkipToEndOfBlock(&mut self, offset: u32);
    fn Restore(&mut self, next: &CSSParserToken, offset: u32) -> CSSParserToken;
    fn TokenCount(&self) -> u32;
    fn UnicodeRangesAllowed(&self) -> bool;
    fn SetUnicodeRangesAllowed(&mut self, allowed: bool);
    fn PopBlockStack(&mut self);
}

// cpp: css_parser_token_stream.h:53-63
// An explicit list replaces the source's token-type template parameter pack.
pub fn IsTokenTypeOneOf(t: CSSParserTokenType, types: &[CSSParserTokenType]) -> bool {
    types.iter().any(|candidate| t == *candidate)
}

// cpp: css_parser_token_stream.h:107-109
pub const fn FlagForTokenType(token_type: CSSParserTokenType) -> u64 {
    1u64 << token_type as u64
}

// cpp: css_parser_token_stream.h:500-513
#[cfg(debug_assertions)]
#[derive(Clone, Copy)]
pub struct State {
    offset_: u32,
    boundaries_: u64,
}
#[cfg(not(debug_assertions))]
pub type State = u32;

// cpp: css_parser_token_stream.h:697-706
pub struct CSSParserTokenStream<'a, T: TokenStreamTokenizer> {
    tokenizer_: T,
    next_: CSSParserToken,
    #[cfg(debug_assertions)]
    peeked_at_next_: Cell<bool>,
    offset_: u32,
    has_look_ahead_: bool,
    boundaries_: u64,
    attr_taint_ranges_: Option<&'a [(u32, u32)]>,
}

impl<'a, T: TokenStreamTokenizer> CSSParserTokenStream<'a, T> {
    // cpp: css_parser_token_stream.h:158-159,697-706
    pub fn new(text: StringView, offset: u32) -> Self {
        Self::FromTokenizer(T::new(text, offset))
    }

    /// Supplies already-tokenized input without changing any token identity.
    /// Ownership replaces the C++ stream's tokenizer member initialization.
    pub fn FromTokenizer(tokenizer: T) -> Self {
        Self {
            tokenizer_: tokenizer,
            next_: CSSParserToken::new(kEOFToken, BlockType::kNotBlock),
            #[cfg(debug_assertions)]
            peeked_at_next_: Cell::new(false),
            offset_: 0,
            has_look_ahead_: false,
            boundaries_: FlagForTokenType(kEOFToken),
            attr_taint_ranges_: None,
        }
    }

    // cpp: css_parser_token_stream.h:161-166
    pub fn WithAttrTaintRanges(text: StringView, ranges: Option<&'a [(u32, u32)]>) -> Self {
        let mut stream = Self::new(text, 0);
        stream.attr_taint_ranges_ = ranges;
        stream
    }

    // cpp: css_parser_token_stream.h:171-183
    pub fn IsAttrTainted(&self, start_offset: u32, end_offset: u32) -> bool {
        let Some(ranges) = self.attr_taint_ranges_ else {
            return false;
        };
        for &(tainted_start, tainted_end) in ranges {
            let end = end_offset.min(tainted_end) as i64;
            let start = start_offset.max(tainted_start) as i64;
            if end - start > 0 {
                return true;
            }
        }
        false
    }

    // cpp: css_parser_token_stream.h:185-205
    pub fn EnsureLookAhead(&mut self) {
        if !self.HasLookAhead() {
            self.LookAhead();
        }
    }
    pub fn LookAhead(&mut self) {
        debug_assert!(!self.HasLookAhead());
        self.next_ = self.tokenizer_.TokenizeSingle();
        #[cfg(debug_assertions)]
        self.peeked_at_next_.set(false);
        self.has_look_ahead_ = true;
    }

    // cpp: css_parser_token_stream.h:207-212
    pub fn HasLookAhead(&self) -> bool {
        self.has_look_ahead_
    }
    pub fn Peek(&mut self) -> &CSSParserToken {
        self.EnsureLookAhead();
        self.UncheckedPeek()
    }

    // cpp: css_parser_token_stream.h:220-227
    pub fn SkipToEndOfBlock(&mut self, bytes: u32) {
        debug_assert!(self.HasLookAhead());
        debug_assert_eq!(self.next_.GetBlockType(), BlockType::kBlockStart);
        self.tokenizer_
            .SkipToEndOfBlock(self.LookAheadOffset().wrapping_add(bytes));
        self.offset_ = self.tokenizer_.Offset();
        self.has_look_ahead_ = false;
    }

    // cpp: css_parser_token_stream.h:229-235
    pub fn UncheckedPeek(&self) -> &CSSParserToken {
        debug_assert!(self.HasLookAhead());
        #[cfg(debug_assertions)]
        self.peeked_at_next_.set(true);
        &self.next_
    }

    // cpp: css_parser_token_stream.h:237-258
    pub fn Consume(&mut self) -> &CSSParserToken {
        self.EnsureLookAhead();
        self.UncheckedConsume()
    }
    pub fn UncheckedConsume(&mut self) -> &CSSParserToken {
        debug_assert!(self.HasLookAhead());
        debug_assert_ne!(self.next_.GetBlockType(), BlockType::kBlockStart);
        debug_assert_ne!(self.next_.GetBlockType(), BlockType::kBlockEnd);
        #[cfg(debug_assertions)]
        debug_assert!(
            self.peeked_at_next_.get(),
            "blind Consume without checking the token"
        );
        self.has_look_ahead_ = false;
        self.offset_ = self.tokenizer_.Offset();
        &self.next_
    }

    // cpp: css_parser_token_stream.h:263-285
    pub fn ConsumeRaw(&mut self) -> &CSSParserToken {
        self.EnsureLookAhead();
        self.UncheckedConsumeRaw()
    }
    pub fn UncheckedConsumeRaw(&mut self) -> &CSSParserToken {
        debug_assert!(self.HasLookAhead());
        self.has_look_ahead_ = false;
        self.offset_ = self.tokenizer_.Offset();
        &self.next_
    }
    pub fn AtEnd(&mut self) -> bool {
        self.EnsureLookAhead();
        self.UncheckedAtEnd()
    }
    pub fn UncheckedAtEnd(&self) -> bool {
        debug_assert!(self.HasLookAhead());
        self.boundaries_ & FlagForTokenType(self.next_.GetType()) != 0
            || self.next_.GetBlockType() == BlockType::kBlockEnd
    }

    // cpp: css_parser_token_stream.h:288-294
    pub fn Offset(&self) -> u32 {
        self.offset_
    }
    pub fn LookAheadOffset(&self) -> u32 {
        debug_assert!(self.HasLookAhead());
        self.tokenizer_.PreviousOffset()
    }

    // cpp: css_parser_token_stream.cc:9-17
    pub fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
        self.tokenizer_.StringRangeAt(start, length)
    }
    pub fn RemainingText(&self) -> StringView {
        let start = if self.HasLookAhead() {
            self.LookAheadOffset()
        } else {
            self.Offset()
        };
        self.tokenizer_.StringRangeFrom(start)
    }

    // cpp: css_parser_token_stream.cc:19-35
    pub fn ConsumeWhitespace(&mut self) {
        while self.Peek().GetType() == kWhitespaceToken {
            self.UncheckedConsume();
        }
    }
    pub fn ConsumeIncludingWhitespace(&mut self) -> CSSParserToken {
        let result = self.Consume().clone();
        self.ConsumeWhitespace();
        result
    }
    pub fn ConsumeIncludingWhitespaceRaw(&mut self) -> CSSParserToken {
        let result = self.ConsumeRaw().clone();
        self.ConsumeWhitespace();
        result
    }

    // cpp: css_parser_token_stream.cc:37-50
    pub fn ConsumeCommentOrNothing(&mut self) -> bool {
        debug_assert!(!self.HasLookAhead());
        let token = self.tokenizer_.TokenizeSingleWithComments();
        if token.GetType() != kCommentToken {
            self.next_ = token;
            self.has_look_ahead_ = true;
            return false;
        }
        self.has_look_ahead_ = false;
        self.offset_ = self.tokenizer_.Offset();
        true
    }

    // cpp: css_parser_token_stream.h:323-366
    pub fn SkipUntilPeekedTypeIs(&mut self, types: &[CSSParserTokenType]) {
        self.EnsureLookAhead();
        if self.next_.IsEOF() || self.TokenMarksEnd(&self.next_, types) {
            #[cfg(debug_assertions)]
            self.peeked_at_next_.set(true);
            return;
        }
        let mut nesting_level = 0u32;
        if self.next_.GetBlockType() == BlockType::kBlockStart {
            nesting_level += 1;
        }
        loop {
            let token = self.tokenizer_.TokenizeSingle();
            if token.IsEOF() || (nesting_level == 0 && self.TokenMarksEnd(&token, types)) {
                self.next_ = token;
                #[cfg(debug_assertions)]
                self.peeked_at_next_.set(true);
                self.offset_ = self.tokenizer_.PreviousOffset();
                return;
            } else if token.GetBlockType() == BlockType::kBlockStart {
                nesting_level = nesting_level.wrapping_add(1);
            } else if token.GetBlockType() == BlockType::kBlockEnd {
                nesting_level = nesting_level.wrapping_sub(1);
            }
        }
    }

    // cpp: css_parser_token_stream.h:515-522
    pub fn Save(&self) -> State {
        debug_assert!(self.has_look_ahead_);
        #[cfg(debug_assertions)]
        {
            State {
                offset_: self.offset_,
                boundaries_: self.boundaries_,
            }
        }
        #[cfg(not(debug_assertions))]
        {
            self.offset_
        }
    }

    // cpp: css_parser_token_stream.h:524-550
    pub fn Restore(&mut self, state: State) {
        debug_assert!(self.has_look_ahead_);
        #[cfg(debug_assertions)]
        {
            if self.offset_ == state.offset_ {
                return;
            }
            self.offset_ = state.offset_;
            debug_assert_eq!(
                state.boundaries_, self.boundaries_,
                "Boundary-crossing restore"
            );
        }
        #[cfg(not(debug_assertions))]
        {
            if self.offset_ == state {
                return;
            }
            self.offset_ = state;
        }
        self.next_ = self.tokenizer_.Restore(&self.next_, self.offset_);
        #[cfg(debug_assertions)]
        self.peeked_at_next_.set(true);
    }

    // cpp: css_parser_token_stream.h:646-654
    pub fn TokenCount(&self) -> u32 {
        self.tokenizer_.TokenCount()
    }
    fn TokenMarksEnd(&self, token: &CSSParserToken, end_types: &[CSSParserTokenType]) -> bool {
        self.boundaries_ & FlagForTokenType(token.GetType()) != 0
            || token.GetBlockType() == BlockType::kBlockEnd
            || IsTokenTypeOneOf(token.GetType(), end_types)
    }

    // cpp: css_parser_token_stream.h:656-677
    fn PeekInternal(&mut self) -> &CSSParserToken {
        self.EnsureLookAhead();
        self.UncheckedPeekInternal()
    }
    fn UncheckedPeekInternal(&self) -> &CSSParserToken {
        debug_assert!(self.HasLookAhead());
        &self.next_
    }
    fn ConsumeInternal(&mut self) -> &CSSParserToken {
        self.EnsureLookAhead();
        self.UncheckedConsumeInternal()
    }
    fn UncheckedConsumeInternal(&mut self) -> &CSSParserToken {
        debug_assert!(self.HasLookAhead());
        self.has_look_ahead_ = false;
        self.offset_ = self.tokenizer_.Offset();
        &self.next_
    }

    // cpp: css_parser_token_stream.h:681-688
    fn RetokenizeLookAhead(&mut self) {
        if self.has_look_ahead_ {
            self.next_ = self
                .tokenizer_
                .Restore(&self.next_, self.tokenizer_.PreviousOffset());
            #[cfg(debug_assertions)]
            self.peeked_at_next_.set(false);
        }
    }

    // cpp: css_parser_token_stream.cc:52-75
    fn UncheckedSkipToEndOfBlock(&mut self) {
        debug_assert!(self.HasLookAhead());
        self.has_look_ahead_ = false;
        let mut nesting_level = 1u32;
        if self.next_.GetBlockType() == BlockType::kBlockStart {
            nesting_level += 1;
        } else if self.next_.GetBlockType() == BlockType::kBlockEnd {
            nesting_level -= 1;
        }
        while nesting_level != 0 {
            let token = self.tokenizer_.TokenizeSingle();
            if token.IsEOF() {
                break;
            } else if token.GetBlockType() == BlockType::kBlockStart {
                nesting_level = nesting_level.wrapping_add(1);
            } else if token.GetBlockType() == BlockType::kBlockEnd {
                nesting_level = nesting_level.wrapping_sub(1);
            }
        }
        self.offset_ = self.tokenizer_.Offset();
    }

    // cpp: css_parser_token_stream.h:695
    fn PopBlockStack(&mut self) {
        self.tokenizer_.PopBlockStack();
    }
}

// cpp: css_parser_token_stream.h:75-105
// Guards exclusively borrow the stream; Deref replaces the source's reference
// alias so parser calls can continue through the guard. Drop preserves RAII.
pub struct BlockGuard<'s, 'a, T: TokenStreamTokenizer> {
    stream_: &'s mut CSSParserTokenStream<'a, T>,
    skipped_to_end_of_block_: bool,
    boundaries_: u64,
}
impl<'s, 'a, T: TokenStreamTokenizer> BlockGuard<'s, 'a, T> {
    pub fn new(stream: &'s mut CSSParserTokenStream<'a, T>) -> Self {
        let boundaries_ = stream.boundaries_;
        let next = stream.ConsumeInternal();
        debug_assert_eq!(next.GetBlockType(), BlockType::kBlockStart);
        stream.boundaries_ = FlagForTokenType(kEOFToken);
        Self {
            stream_: stream,
            skipped_to_end_of_block_: false,
            boundaries_,
        }
    }
    pub fn SkipToEndOfBlock(&mut self) {
        debug_assert!(!self.skipped_to_end_of_block_);
        self.stream_.EnsureLookAhead();
        self.stream_.UncheckedSkipToEndOfBlock();
        self.skipped_to_end_of_block_ = true;
    }
}
impl<T: TokenStreamTokenizer> Drop for BlockGuard<'_, '_, T> {
    fn drop(&mut self) {
        if !self.skipped_to_end_of_block_ {
            self.SkipToEndOfBlock();
        }
        self.stream_.boundaries_ = self.boundaries_;
    }
}
impl<'a, T: TokenStreamTokenizer> Deref for BlockGuard<'_, 'a, T> {
    type Target = CSSParserTokenStream<'a, T>;
    fn deref(&self) -> &Self::Target {
        self.stream_
    }
}
impl<T: TokenStreamTokenizer> DerefMut for BlockGuard<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream_
    }
}

// cpp: css_parser_token_stream.h:119-131
pub struct Boundary<'s, 'a, T: TokenStreamTokenizer> {
    stream_: &'s mut CSSParserTokenStream<'a, T>,
    original_: u64,
}
impl<'s, 'a, T: TokenStreamTokenizer> Boundary<'s, 'a, T> {
    pub fn new(
        stream: &'s mut CSSParserTokenStream<'a, T>,
        boundary_type: CSSParserTokenType,
    ) -> Self {
        let original_ = stream.boundaries_;
        stream.boundaries_ |= FlagForTokenType(boundary_type);
        Self {
            stream_: stream,
            original_,
        }
    }
}
impl<T: TokenStreamTokenizer> Drop for Boundary<'_, '_, T> {
    fn drop(&mut self) {
        self.stream_.boundaries_ = self.original_;
    }
}
impl<'a, T: TokenStreamTokenizer> Deref for Boundary<'_, 'a, T> {
    type Target = CSSParserTokenStream<'a, T>;
    fn deref(&self) -> &Self::Target {
        self.stream_
    }
}
impl<T: TokenStreamTokenizer> DerefMut for Boundary<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream_
    }
}

// cpp: css_parser_token_stream.h:136-156
pub struct EnableUnicodeRanges<'s, 'a, T: TokenStreamTokenizer> {
    stream_: &'s mut CSSParserTokenStream<'a, T>,
    old_unicode_ranges_allowed_: bool,
}
impl<'s, 'a, T: TokenStreamTokenizer> EnableUnicodeRanges<'s, 'a, T> {
    pub fn new(stream: &'s mut CSSParserTokenStream<'a, T>, unicode_ranges_allowed: bool) -> Self {
        let old_unicode_ranges_allowed_ = stream.tokenizer_.UnicodeRangesAllowed();
        stream
            .tokenizer_
            .SetUnicodeRangesAllowed(unicode_ranges_allowed);
        stream.RetokenizeLookAhead();
        Self {
            stream_: stream,
            old_unicode_ranges_allowed_,
        }
    }
}
impl<T: TokenStreamTokenizer> Drop for EnableUnicodeRanges<'_, '_, T> {
    fn drop(&mut self) {
        self.stream_
            .tokenizer_
            .SetUnicodeRangesAllowed(self.old_unicode_ranges_allowed_);
        self.stream_.RetokenizeLookAhead();
    }
}
impl<'a, T: TokenStreamTokenizer> Deref for EnableUnicodeRanges<'_, 'a, T> {
    type Target = CSSParserTokenStream<'a, T>;
    fn deref(&self) -> &Self::Target {
        self.stream_
    }
}
impl<T: TokenStreamTokenizer> DerefMut for EnableUnicodeRanges<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream_
    }
}

// cpp: css_parser_token_stream.h:568-644
pub struct RestoringBlockGuard<'s, 'a, T: TokenStreamTokenizer> {
    stream_: &'s mut CSSParserTokenStream<'a, T>,
    boundaries_: u64,
    state_: State,
    released_: bool,
}
impl<'s, 'a, T: TokenStreamTokenizer> RestoringBlockGuard<'s, 'a, T> {
    fn ResetStreamBoundaries(stream: &mut CSSParserTokenStream<'a, T>) -> u64 {
        let original = stream.boundaries_;
        stream.boundaries_ = FlagForTokenType(kEOFToken);
        original
    }
    pub fn new(stream: &'s mut CSSParserTokenStream<'a, T>) -> Self {
        let boundaries_ = Self::ResetStreamBoundaries(stream);
        let state_ = stream.Save();
        let next = stream.ConsumeInternal();
        debug_assert_eq!(next.GetBlockType(), BlockType::kBlockStart);
        Self {
            stream_: stream,
            boundaries_,
            state_,
            released_: false,
        }
    }
    pub fn Release(&mut self) -> bool {
        debug_assert!(!self.released_);
        self.stream_.EnsureLookAhead();
        if self.stream_.next_.IsEOF() || self.stream_.next_.GetBlockType() == BlockType::kBlockEnd {
            self.stream_.UncheckedConsumeInternal();
            self.released_ = true;
            return true;
        }
        false
    }
}
impl<T: TokenStreamTokenizer> Drop for RestoringBlockGuard<'_, '_, T> {
    fn drop(&mut self) {
        self.stream_.EnsureLookAhead();
        if !self.released_ {
            self.stream_.Restore(self.state_);
            self.stream_.PopBlockStack();
        }
        self.stream_.boundaries_ = self.boundaries_;
    }
}
impl<'a, T: TokenStreamTokenizer> Deref for RestoringBlockGuard<'_, 'a, T> {
    type Target = CSSParserTokenStream<'a, T>;
    fn deref(&self) -> &Self::Target {
        self.stream_
    }
}
impl<T: TokenStreamTokenizer> DerefMut for RestoringBlockGuard<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream_
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::String;

    // A test transport for explicit tokenizer output. It does not classify CSS
    // characters; every emitted token and source end offset is supplied by the
    // fixture. Stack mutation/restore follows css_tokenizer.h:72-86.
    struct FixtureTokenizer {
        records: Vec<(CSSParserToken, u32)>,
        text: StringView,
        index: usize,
        offset: u32,
        previous: u32,
        count: u32,
        unicode_allowed: bool,
        stack: Vec<CSSParserTokenType>,
        restore_count: u32,
        unicode_alternative: Option<CSSParserToken>,
    }
    impl FixtureTokenizer {
        fn Emit(&mut self) -> CSSParserToken {
            self.previous = self.offset;
            self.count += 1;
            let (mut token, end) = self.records[self.index].clone();
            self.offset = end;
            if !token.IsEOF() {
                self.index += 1;
            }
            if self.unicode_allowed && self.previous == 0 {
                if let Some(alternative) = &self.unicode_alternative {
                    token = alternative.clone();
                }
            }
            match token.GetBlockType() {
                BlockType::kBlockStart => {
                    self.stack.push(if token.GetType() == kFunctionToken {
                        kLeftParenthesisToken
                    } else {
                        token.GetType()
                    });
                }
                BlockType::kBlockEnd => {
                    self.stack.pop().expect("fixture block stack");
                }
                BlockType::kNotBlock => {}
            }
            token
        }
    }
    impl TokenStreamTokenizer for FixtureTokenizer {
        fn new(text: StringView, offset: u32) -> Self {
            // This adapter only constructs an empty transport. Nonempty CSS
            // input needs the real CSSTokenizer and is deliberately rejected.
            assert!(text.IsEmpty() && offset == 0);
            Self {
                records: vec![(token(kEOFToken, BlockType::kNotBlock), 0)],
                text,
                index: 0,
                offset: 0,
                previous: 0,
                count: 0,
                unicode_allowed: false,
                stack: Vec::new(),
                restore_count: 0,
                unicode_alternative: None,
            }
        }
        fn TokenizeSingle(&mut self) -> CSSParserToken {
            loop {
                let token = self.Emit();
                if token.GetType() != kCommentToken {
                    return token;
                }
            }
        }
        fn TokenizeSingleWithComments(&mut self) -> CSSParserToken {
            self.Emit()
        }
        fn Offset(&self) -> u32 {
            self.offset
        }
        fn PreviousOffset(&self) -> u32 {
            self.previous
        }
        fn StringRangeAt(&self, start: u32, length: u32) -> StringView {
            self.text.Substring(start, length)
        }
        fn StringRangeFrom(&self, start: u32) -> StringView {
            self.text.Substring(start, self.text.length() - start)
        }
        fn SkipToEndOfBlock(&mut self, offset: u32) {
            while self.offset < offset {
                self.TokenizeSingle();
            }
            assert_eq!(self.offset, offset);
        }
        fn Restore(&mut self, next: &CSSParserToken, offset: u32) -> CSSParserToken {
            match next.GetBlockType() {
                BlockType::kBlockStart => {
                    self.stack.pop().expect("undo block start");
                }
                BlockType::kBlockEnd => self.stack.push(match next.GetType() {
                    kRightParenthesisToken => kLeftParenthesisToken,
                    kRightBracketToken => kLeftBracketToken,
                    kRightBraceToken => kLeftBraceToken,
                    _ => panic!("fixture invalid closing token"),
                }),
                BlockType::kNotBlock => {}
            }
            self.index = (0..self.records.len())
                .find(|&index| {
                    let start = if index == 0 {
                        0
                    } else {
                        self.records[index - 1].1
                    };
                    start == offset
                })
                .expect("restore offset is a fixture token start");
            self.offset = offset;
            self.restore_count += 1;
            self.TokenizeSingle()
        }
        fn TokenCount(&self) -> u32 {
            self.count
        }
        fn UnicodeRangesAllowed(&self) -> bool {
            self.unicode_allowed
        }
        fn SetUnicodeRangesAllowed(&mut self, allowed: bool) {
            self.unicode_allowed = allowed;
        }
        fn PopBlockStack(&mut self) {
            self.stack.pop().expect("pop fixture block stack");
        }
    }

    fn token(kind: CSSParserTokenType, block: BlockType) -> CSSParserToken {
        CSSParserToken::new(kind, block)
    }
    fn stream(
        kinds: &[(CSSParserTokenType, BlockType)],
    ) -> CSSParserTokenStream<'static, FixtureTokenizer> {
        let mut stream: CSSParserTokenStream<FixtureTokenizer> =
            CSSParserTokenStream::new(StringView::from(&String::from("")), 0);
        stream.tokenizer_.records = kinds
            .iter()
            .enumerate()
            .map(|(index, &(kind, block))| (token(kind, block), index as u32 + 1))
            .collect();
        stream
            .tokenizer_
            .records
            .push((token(kEOFToken, BlockType::kNotBlock), kinds.len() as u32));
        stream.tokenizer_.text = StringView::from(&String::from_latin1(&vec![b'x'; kinds.len()]));
        stream
    }

    #[test]
    fn lookahead_offset_whitespace_comments_and_restore() {
        let mut stream = stream(&[
            (kIdentToken, BlockType::kNotBlock),
            (kWhitespaceToken, BlockType::kNotBlock),
            (kCommentToken, BlockType::kNotBlock),
            (kIdentToken, BlockType::kNotBlock),
        ]);
        assert!(!stream.HasLookAhead());
        assert_eq!(stream.Peek().GetType(), kIdentToken);
        let state = stream.Save();
        assert_eq!(stream.Offset(), 0);
        assert_eq!(stream.TokenCount(), 1);
        assert_eq!(stream.LookAheadOffset(), 0);
        assert_eq!(stream.RemainingText().length(), 4);
        assert_eq!(stream.ConsumeIncludingWhitespace().GetType(), kIdentToken);
        assert_eq!(stream.Offset(), 2);
        assert_eq!(stream.LookAheadOffset(), 3); // Comment skipped by normal tokenize.
        assert_eq!(stream.RemainingText().length(), 1);
        stream.Restore(state);
        assert_eq!(stream.Peek().GetType(), kIdentToken);
        assert_eq!(stream.Offset(), 0);
        let restores = stream.tokenizer_.restore_count;
        stream.Restore(stream.Save()); // Same-offset shortcut must not retokenize.
        assert_eq!(stream.tokenizer_.restore_count, restores);
        stream.Consume();
        stream.Peek();
        stream.Consume();
        assert!(stream.ConsumeCommentOrNothing());
        assert_eq!(stream.Offset(), 3);
        assert!(!stream.ConsumeCommentOrNothing());
        assert_eq!(stream.UncheckedPeek().GetType(), kIdentToken);
        assert_eq!(stream.StringRangeAt(1, 2).length(), 2);
    }

    #[test]
    fn boundaries_are_suspended_in_blocks_and_nested_skip_stops_at_outer_boundary() {
        let records = [
            (kLeftBracketToken, BlockType::kBlockStart),
            (kSemicolonToken, BlockType::kNotBlock),
            (kLeftParenthesisToken, BlockType::kBlockStart),
            (kSemicolonToken, BlockType::kNotBlock),
            (kRightParenthesisToken, BlockType::kBlockEnd),
            (kRightBracketToken, BlockType::kBlockEnd),
            (kSemicolonToken, BlockType::kNotBlock),
            (kIdentToken, BlockType::kNotBlock),
        ];
        let mut stream = stream(&records);
        {
            let mut boundary = Boundary::new(&mut stream, kSemicolonToken);
            assert_eq!(boundary.Peek().GetType(), kLeftBracketToken);
            {
                let mut block = BlockGuard::new(&mut boundary);
                assert!(!block.AtEnd()); // Semicolon boundary does not apply in block.
                block.Peek();
                block.Consume();
                block.SkipToEndOfBlock();
                assert!(!block.HasLookAhead());
            }
            assert!(boundary.AtEnd());
            assert_eq!(boundary.Peek().GetType(), kSemicolonToken);
        }
        assert!(!stream.AtEnd());
        assert!(stream.tokenizer_.stack.is_empty());
        let mut skipped = super::tests::stream(&records);
        skipped.SkipUntilPeekedTypeIs(&[kSemicolonToken]);
        assert_eq!(skipped.Offset(), 6);
        assert_eq!(skipped.Peek().GetType(), kSemicolonToken);
        assert!(skipped.tokenizer_.stack.is_empty());
        skipped.Consume();
        skipped.SkipUntilPeekedTypeIs(&[]);
        assert!(skipped.AtEnd());
        let mut fast = super::tests::stream(&records);
        fast.Peek();
        fast.SkipToEndOfBlock(6);
        assert_eq!(fast.Offset(), 6);
        assert!(!fast.HasLookAhead());
        assert_eq!(fast.Peek().GetType(), kSemicolonToken);
        let mut unclosed = super::tests::stream(&[
            (kLeftBraceToken, BlockType::kBlockStart),
            (kIdentToken, BlockType::kNotBlock),
        ]);
        unclosed.Peek();
        {
            let _guard = BlockGuard::new(&mut unclosed);
        }
        assert_eq!(unclosed.Offset(), 2);
        assert!(unclosed.AtEnd());
    }

    #[test]
    fn restoring_guard_aborts_or_releases_and_unicode_guard_retokenizes() {
        let records = [
            (kLeftParenthesisToken, BlockType::kBlockStart),
            (kIdentToken, BlockType::kNotBlock),
            (kRightParenthesisToken, BlockType::kBlockEnd),
            (kIdentToken, BlockType::kNotBlock),
        ];
        let mut stream = stream(&records);
        stream.Peek();
        {
            let mut guard = RestoringBlockGuard::new(&mut stream);
            assert!(!guard.Release());
        }
        assert_eq!(stream.Offset(), 0);
        assert_eq!(stream.Peek().GetType(), kLeftParenthesisToken);
        // Restore pushes an extra marker for its recreated opening lookahead;
        // the guard removes only that extra marker, preserving the original.
        assert_eq!(stream.tokenizer_.stack.len(), 1);
        // The same restored opening lookahead can now be entered successfully.
        let mut released = stream;
        {
            let mut guard = RestoringBlockGuard::new(&mut released);
            guard.Peek();
            guard.Consume();
            assert!(guard.Release());
        }
        assert_eq!(released.Offset(), 3);
        assert_eq!(released.Peek().GetType(), kIdentToken);
        assert!(released.tokenizer_.stack.is_empty());
        let mut unicode = super::tests::stream(&[(kIdentToken, BlockType::kNotBlock)]);
        unicode.tokenizer_.unicode_alternative =
            Some(token(kUnicodeRangeToken, BlockType::kNotBlock));
        unicode.Peek();
        {
            let mut guard = EnableUnicodeRanges::new(&mut unicode, true);
            assert_eq!(guard.Peek().GetType(), kUnicodeRangeToken);
        }
        assert_eq!(unicode.Peek().GetType(), kIdentToken);
        assert_eq!(unicode.tokenizer_.restore_count, 2);
    }

    #[test]
    fn attr_taint_uses_nonempty_half_open_interval_overlap() {
        let ranges = [(5, 10), (15, 20), (u32::MAX - 2, u32::MAX)];
        let mut stream: CSSParserTokenStream<FixtureTokenizer> =
            CSSParserTokenStream::WithAttrTaintRanges(
                StringView::from(&String::from("")),
                Some(&ranges),
            );
        for (start, end) in [(0, 5), (10, 15), (20, 30), (5, 5), (10, 5)] {
            assert!(!stream.IsAttrTainted(start, end));
        }
        for (start, end) in [(4, 6), (5, 10), (9, 16), (u32::MAX - 1, u32::MAX)] {
            assert!(stream.IsAttrTainted(start, end));
        }
        stream.attr_taint_ranges_ = None;
        assert!(!stream.IsAttrTainted(0, u32::MAX));
    }
}
