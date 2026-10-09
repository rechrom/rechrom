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
    rest_: StringView,
}

impl CSSTokenizerInputStream {
    pub fn new(input: StringView) -> Self {
        Self {
            string_: input.clone(),
            rest_: input,
        }
    }

    // css_tokenizer_input_stream.h:31-54
    pub fn NextInputChar(&self) -> u16 {
        if self.rest_.IsEmpty() {
            return 0;
        }
        let result = self.rest_[0];
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
        if lookahead_offset >= self.rest_.length() {
            0
        } else {
            self.rest_[lookahead_offset as usize]
        }
    }

    pub fn Peek(&self) -> StringView {
        self.rest_.clone()
    }

    pub fn AdvanceBy(&mut self, offset: u32) {
        let advance = offset.min(self.rest_.length());
        self.rest_ = self.rest_.Substring(advance, self.rest_.length() - advance);
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
        while offset < self.rest_.length() && predicate(self.rest_[offset as usize]) {
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
        assert!(start <= end && end <= self.rest_.length());
        if start < end && self.string_.Is8Bit() && end - start <= 14 {
            let bytes = self.rest_.Span8().expect("8-bit source");
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
        assert!(start <= end && end <= self.rest_.length());
        let value = self.rest_.Substring(start, end - start);
        StringToDouble(&value).expect("tokenizer supplies a syntactically valid CSS number")
    }

    pub fn length(&self) -> u32 {
        self.string_.length()
    }
    pub fn Offset(&self) -> u32 {
        self.string_.length() - self.rest_.length()
    }
    pub fn AtEnd(&self) -> bool {
        self.rest_.IsEmpty()
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
        self.rest_ = self
            .string_
            .Substring(offset, self.string_.length() - offset);
    }
}
