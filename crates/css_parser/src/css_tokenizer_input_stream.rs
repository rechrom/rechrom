#![allow(non_snake_case)]

use crate::css_parser_idioms::{IsCSSInputSpace, IsLeadingSurrogate, IsTrailingSurrogate};
use foundation::{StringView, UChar};

// cpp: css_parser/css_tokenizer_input_stream.h:15-136
pub struct CSSTokenizerInputStream {
    string_: StringView,
    rest_: StringView,
}

impl CSSTokenizerInputStream {
    // cpp: css_parser/css_tokenizer_input_stream.h:17-18
    pub fn new(input: StringView) -> Self {
        Self {
            string_: input.clone(),
            rest_: input,
        }
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:26-47
    pub fn NextInputChar(&self) -> UChar {
        if self.rest_.IsEmpty() {
            return 0;
        }
        let result = self.rest_[0];
        if result == 0
            || (IsLeadingSurrogate(result) && !IsTrailingSurrogate(self.PeekWithoutReplacement(1)))
            || (IsTrailingSurrogate(result)
                && !IsLeadingSurrogate(self.PeekPreviousCharWithoutReplacement()))
        {
            return 0xFFFD;
        }
        result
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:51-56
    pub fn PeekPreviousCharWithoutReplacement(&self) -> UChar {
        if self.Offset() == 0 {
            return 0;
        }
        self.string_[(self.Offset() - 1) as usize]
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:61-68
    pub fn PeekWithoutReplacement(&self, lookahead_offset: u32) -> UChar {
        if lookahead_offset >= self.rest_.length() {
            return 0;
        }
        self.rest_[lookahead_offset as usize]
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:69-69
    pub fn Peek(&self) -> StringView {
        self.rest_.clone()
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:74-76
    pub fn Advance(&mut self, offset: u32) {
        self.rest_ = self.rest_.Substring(
            offset.min(self.rest_.length()),
            self.rest_.length() - offset.min(self.rest_.length()),
        );
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:77-80
    pub fn PushBack(&mut self, cc: UChar) {
        assert!(self.Offset() > 0);
        self.rest_ = self
            .string_
            .Substring(self.Offset() - 1, self.length() - self.Offset() + 1);
        debug_assert_eq!(self.NextInputChar(), cc);
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:88-110
    pub fn SkipWhilePredicate(&self, mut offset: u32, predicate: impl Fn(UChar) -> bool) -> u32 {
        let length = self.rest_.length();
        // StringView keeps UTF-16 units but not the source StringImpl's
        // original 8-bit flag. Both source branches perform the same scan.
        let chars = self.rest_.Span16();
        while offset < length && predicate(chars[offset as usize]) {
            offset += 1;
        }
        offset
    }

    // cpp: css_parser/css_tokenizer_input_stream.cc:14-17
    pub fn AdvanceUntilNonWhitespace(&mut self) {
        let count = self.SkipWhilePredicate(0, IsCSSInputSpace);
        self.Advance(count);
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:114-116
    pub fn length(&self) -> u32 {
        self.string_.length()
    }
    pub fn Offset(&self) -> u32 {
        self.string_.length() - self.rest_.length()
    }
    pub fn AtEnd(&self) -> bool {
        self.rest_.IsEmpty()
    }

    // cpp: css_parser/css_tokenizer_input_stream.h:118-128
    pub fn RangeFrom(&self, start: u32) -> StringView {
        self.string_.Substring(start, self.length() - start)
    }
    pub fn RangeAt(&self, start: u32, length: u32) -> StringView {
        debug_assert!(start + length <= self.string_.length());
        self.string_.Substring(start, length)
    }
    pub fn Restore(&mut self, offset: u32) {
        self.rest_ = self.string_.Substring(offset, self.length() - offset);
    }

    // cpp: css_parser/css_tokenizer_input_stream.cc:19-29
    pub fn GetDouble(&self, start: u32, end: u32) -> f64 {
        debug_assert!(start <= end && end <= self.rest_.length());
        let result = if start < end {
            self.rest_
                .Substring(start, end - start)
                .ToString()
                .Utf8()
                .parse::<f64>()
                .ok()
        } else {
            None
        };
        result.unwrap_or(0.0)
    }

    // cpp: css_parser/css_tokenizer_input_stream.cc:31-50
    pub fn GetNaturalNumberAsDouble(&self, start: u32, end: u32) -> f64 {
        debug_assert!(start <= end && end <= self.rest_.length());
        if start < end && self.string_.Is8Bit() && end - start <= 14 {
            let mut result = 0_u64;
            for &ch in &self.rest_.Span16()[start as usize..end as usize] {
                result = result * 10 + u64::from(ch - u16::from(b'0'));
            }
            result as f64
        } else {
            self.GetDouble(start, end)
        }
    }
}

#[cfg(test)]
mod loading_tests {
    use super::*;

    #[test]
    #[ignore = "manual Debug large numeric stylesheet scan timing"]
    fn profile_large_numeric_input_stream() {
        let source = foundation::String::from("123px;".repeat(20000).as_str());
        let mut stream = CSSTokenizerInputStream::new(StringView::from(&source));
        let started = std::time::Instant::now();
        for _ in 0..20000 {
            assert_eq!(stream.GetNaturalNumberAsDouble(0, 3), 123.0);
            stream.Advance(6);
        }
        assert!(stream.AtEnd());
        eprintln!(
            "css-numeric-stream-bench ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
    }

    #[test]
    fn view_ranges_preserve_stream_restore_and_number_conversion() {
        for source in [
            foundation::String::from("12 345 123456789012345 1.25"),
            foundation::String::from_utf16(
                &"12 345 123456789012345 1.25"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            ),
        ] {
            let mut stream = CSSTokenizerInputStream::new(StringView::from(&source));
            assert_eq!(stream.GetNaturalNumberAsDouble(0, 2), 12.0);
            stream.Advance(3);
            assert_eq!(stream.Offset(), 3);
            assert_eq!(stream.GetNaturalNumberAsDouble(0, 3), 345.0);
            stream.PushBack(b' ' as u16);
            assert_eq!(stream.NextInputChar(), b' ' as u16);
            stream.Restore(7);
            assert_eq!(stream.GetNaturalNumberAsDouble(0, 15), 123456789012345.0);
            stream.Restore(23);
            assert_eq!(stream.GetDouble(0, 4), 1.25);
            assert_eq!(stream.RangeAt(3, 3).ToString().Utf8(), "345");
            assert_eq!(stream.RangeFrom(23).ToString().Utf8(), "1.25");
            stream.Advance(1000);
            assert!(stream.AtEnd());
        }
        let source = foundation::String::from_utf16(&[b'x' as u16, 0xd83d, 0xde00, 0xd800, 0]);
        let mut stream = CSSTokenizerInputStream::new(StringView::from(&source));
        stream.Advance(1);
        assert_eq!(stream.NextInputChar(), 0xd83d);
        stream.Advance(1);
        assert_eq!(stream.NextInputChar(), 0xde00);
        stream.Advance(1);
        assert_eq!(stream.NextInputChar(), 0xfffd);
        stream.Advance(1);
        assert_eq!(stream.NextInputChar(), 0xfffd);
    }
}
