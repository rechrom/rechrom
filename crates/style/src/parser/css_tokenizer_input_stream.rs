// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_tokenizer_input_stream.h
// cpp: third_party/blink/renderer/core/css/parser/css_tokenizer_input_stream.cc:15-18,37-51

#![allow(non_snake_case)]

use foundation::StringView;

use super::css_parser_idioms::{IsHTMLSpace, IsLeadingSurrogate, IsTrailingSurrogate};
use super::string_to_number::StringToDouble;

pub struct CSSTokenizerInputStream {
    string_: StringView,
    // Blink's rest_ is a non-owning view. Our StringView retains shared
    // backing storage, so rebuilding it for every character would perform
    // atomic reference-count operations on the tokenizer's hottest path.
    // Represent the same suffix by its offset into one retained source view.
    offset_: u32,
}

impl CSSTokenizerInputStream {
    pub fn new(input: StringView) -> Self {
        Self {
            string_: input,
            offset_: 0,
        }
    }

    // css_tokenizer_input_stream.h:31-54
    pub fn NextInputChar(&self) -> u16 {
        if self.AtEnd() {
            return 0;
        }
        let result = self.string_[self.offset_ as usize];
        if result == 0
            || (IsLeadingSurrogate(result) && !IsTrailingSurrogate(self.PeekWithoutReplacement(1)))
            || (IsTrailingSurrogate(result)
                && !IsLeadingSurrogate(self.PeekPreviousCharWithoutReplacement()))
        {
            0xfffd
        } else {
            result
        }
    }

    pub fn PeekPreviousCharWithoutReplacement(&self) -> u16 {
        if self.Offset() == 0 {
            0
        } else {
            self.string_[(self.Offset() - 1) as usize]
        }
    }

    pub fn PeekWithoutReplacement(&self, lookahead_offset: u32) -> u16 {
        if lookahead_offset >= self.RemainingLength() {
            0
        } else {
            self.string_[(self.offset_ + lookahead_offset) as usize]
        }
    }

    pub fn Peek(&self) -> StringView {
        self.string_.Substring(self.offset_, self.RemainingLength())
    }

    pub fn AdvanceBy(&mut self, offset: u32) {
        self.offset_ += offset.min(self.RemainingLength());
    }

    pub fn Advance(&mut self) {
        self.AdvanceBy(1);
    }

    pub fn PushBack(&mut self, cc: u16) {
        let offset = self
            .Offset()
            .checked_sub(1)
            .expect("cannot push back at stream start");
        self.Restore(offset);
        debug_assert_eq!(self.NextInputChar(), cc);
    }

    pub fn SkipWhilePredicate(&self, mut offset: u32, predicate: impl Fn(u16) -> bool) -> u32 {
        let remaining = &self.string_.Span16()[self.offset_ as usize..];
        while (offset as usize) < remaining.len() && predicate(remaining[offset as usize]) {
            offset += 1;
        }
        offset
    }

    pub fn AdvanceUntilNonWhitespace(&mut self) {
        let offset = self.SkipWhilePredicate(0, IsHTMLSpace);
        self.AdvanceBy(offset);
    }

    // Exact fast path from .cc:37-48.
    pub fn GetNaturalNumberAsDoubleFast(&self, start: u32, end: u32) -> Option<f64> {
        assert!(start <= end && end <= self.RemainingLength());
        if start < end && self.string_.Is8Bit() && end - start <= 14 {
            let bytes = &self.string_.Span8().expect("8-bit source")[self.offset_ as usize..];
            let mut result = 0u64;
            for &ch in &bytes[start as usize..end as usize] {
                result = result * 10 + u64::from(ch - b'0');
            }
            Some(result as f64)
        } else {
            None
        }
    }

    pub fn GetNaturalNumberAsDouble(&self, start: u32, end: u32) -> f64 {
        self.GetNaturalNumberAsDoubleFast(start, end)
            .unwrap_or_else(|| self.GetDouble(start, end))
    }

    pub fn GetDouble(&self, start: u32, end: u32) -> f64 {
        assert!(start <= end && end <= self.RemainingLength());
        let value = self.string_.Substring(self.offset_ + start, end - start);
        StringToDouble(&value).expect("tokenizer supplies a syntactically valid CSS number")
    }

    pub fn length(&self) -> u32 {
        self.string_.length()
    }
    pub fn Offset(&self) -> u32 {
        self.offset_
    }
    pub fn AtEnd(&self) -> bool {
        self.offset_ == self.string_.length()
    }
    pub fn RangeFrom(&self, start: u32) -> StringView {
        self.string_.Substring(start, self.string_.length() - start)
    }
    pub fn RangeAt(&self, start: u32, length: u32) -> StringView {
        assert!(start + length <= self.string_.length());
        self.string_.Substring(start, length)
    }
    pub fn Restore(&mut self, offset: u32) {
        assert!(offset <= self.string_.length());
        self.offset_ = offset;
    }

    fn RemainingLength(&self) -> u32 {
        self.string_.length() - self.offset_
    }
}
