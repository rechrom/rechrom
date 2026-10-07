#![allow(non_snake_case, non_camel_case_types)]

use std::collections::VecDeque;

use foundation::{BlinkString, OrdinalNumber, StringBuilder, StringView, UChar};

// cpp: html/parser/segmented_string.h:40-44,166-180
#[derive(Clone, Debug)]
pub struct SegmentedSubstring {
    string_: BlinkString,
    offset_: usize,
    end_: usize,
    do_not_exclude_line_numbers_: bool,
    is_8bit_: bool,
}

impl Default for SegmentedSubstring {
    // cpp: html/parser/segmented_string.h:44,69-73
    fn default() -> Self {
        Self {
            string_: BlinkString::default(),
            offset_: 0,
            end_: 0,
            do_not_exclude_line_numbers_: true,
            is_8bit_: true,
        }
    }
}

impl SegmentedSubstring {
    // cpp: html/parser/segmented_string.h:46-67
    pub fn new(string: &BlinkString) -> Self {
        let len = string.length() as usize;
        Self {
            string_: string.clone(),
            offset_: 0,
            end_: len,
            do_not_exclude_line_numbers_: true,
            is_8bit_: len == 0 || string.Is8Bit(),
        }
    }

    // cpp: html/parser/segmented_string.h:69-73
    pub fn Clear(&mut self) {
        self.is_8bit_ = true;
        self.offset_ = 0;
        self.end_ = 0;
    }

    // cpp: html/parser/segmented_string.h:75
    pub fn ExcludeLineNumbers(&self) -> bool {
        !self.do_not_exclude_line_numbers_
    }

    // cpp: html/parser/segmented_string.h:76
    pub fn DoNotExcludeLineNumbers(&self) -> bool {
        self.do_not_exclude_line_numbers_
    }

    // cpp: html/parser/segmented_string.h:78
    pub fn SetExcludeLineNumbers(&mut self) {
        self.do_not_exclude_line_numbers_ = false;
    }

    // cpp: html/parser/segmented_string.h:80
    pub fn NumberOfCharactersConsumed(&self) -> i32 {
        self.offset()
    }

    // cpp: html/parser/segmented_string.h:82-90
    pub fn AppendTo(&self, builder: &mut StringBuilder) {
        let len = self.length();
        if self.offset() == 0 {
            if len != 0 {
                builder.AppendString(&self.string_);
            }
        } else if len == 1 {
            builder.AppendCodeUnit(self.GetCurrentChar());
        } else {
            builder.AppendString(&self.CurrentSubString(len as u32).ToString());
        }
    }

    // cpp: html/parser/segmented_string.h:92-116
    pub fn PushIfPossible(&mut self, c: UChar) -> bool {
        if self.offset_ == 0 {
            return false;
        }
        let previous = if self.is_8bit_ {
            u16::from(self.string_.Span8().expect("8-bit substring has storage")[self.offset_ - 1])
        } else {
            self.string_.Span16().expect("16-bit substring has storage")[self.offset_ - 1]
        };
        if previous != c {
            return false;
        }
        self.offset_ -= 1;
        true
    }

    // cpp: html/parser/segmented_string.h:118-122
    pub fn GetCurrentChar(&self) -> UChar {
        if self.is_8bit_ {
            u16::from(self.string_.Span8().expect("8-bit substring has storage")[self.offset_])
        } else {
            self.string_.Span16().expect("16-bit substring has storage")[self.offset_]
        }
    }

    // cpp: html/parser/segmented_string.h:124-126
    pub fn CanAdvance(&self) -> bool {
        self.offset_ + 1 < self.end_
    }

    // cpp: html/parser/segmented_string.h:128-139
    pub fn AdvanceMany(&mut self, delta: u32) -> u32 {
        debug_assert_ne!(self.length(), 0);
        let delta = delta.min((self.length() - 1) as u32);
        self.offset_ += delta as usize;
        delta
    }

    // cpp: html/parser/segmented_string.h:141-143
    pub fn AdvanceOne(&mut self) -> UChar {
        self.offset_ += 1;
        self.GetCurrentChar()
    }

    // cpp: html/parser/segmented_string.h:145-147
    pub fn CurrentSubString(&self, len: u32) -> StringView {
        debug_assert!(len <= self.length() as u32);
        StringView::from_blink_string_range(&self.string_, self.offset_ as u32, len)
    }

    // cpp: html/parser/segmented_string.h:149-152
    pub fn offset(&self) -> i32 {
        self.offset_ as i32
    }

    // cpp: html/parser/segmented_string.h:154-157,160-164
    pub fn length(&self) -> i32 {
        (self.end_ - self.offset_) as i32
    }
}

// cpp: html/parser/segmented_string.h:207-210
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrependType {
    kNewInput = 0,
    kUnconsume = 1,
}

// cpp: html/parser/segmented_string.h:232-236
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookAheadResult {
    kDidNotMatch,
    kDidMatch,
    kNotEnoughCharacters,
}

fn equal_ignoring_ascii_case(lhs: &[UChar], rhs: &[UChar]) -> bool {
    lhs.len() == rhs.len()
        && lhs.iter().zip(rhs).all(|(&a, &b)| {
            let lower = |cc: UChar| {
                if cc <= 0x7F {
                    (cc as u8).to_ascii_lowercase() as UChar
                } else {
                    cc
                }
            };
            lower(a) == lower(b)
        })
}

// cpp: html/parser/segmented_string.h:182-202,390-401
#[derive(Clone, Debug)]
pub struct SegmentedString {
    current_string_: SegmentedSubstring,
    number_of_characters_consumed_prior_to_current_string_: i32,
    number_of_characters_consumed_prior_to_current_line_: i32,
    current_line_: i32,
    substrings_: VecDeque<SegmentedSubstring>,
    closed_: bool,
    empty_: bool,
    current_char_: UChar,
    next_segmented_string_: *const SegmentedString,
}

impl Default for SegmentedString {
    // cpp: html/parser/segmented_string.h:186-192
    fn default() -> Self {
        Self {
            current_string_: SegmentedSubstring::default(),
            number_of_characters_consumed_prior_to_current_string_: 0,
            number_of_characters_consumed_prior_to_current_line_: 0,
            current_line_: 0,
            substrings_: VecDeque::new(),
            closed_: false,
            empty_: true,
            current_char_: 0,
            next_segmented_string_: std::ptr::null(),
        }
    }
}

impl SegmentedString {
    // cpp: html/parser/segmented_string.h:194-201
    pub fn from_string(string: &BlinkString) -> Self {
        let current_string = SegmentedSubstring::new(string);
        let empty = string.length() == 0;
        Self {
            current_char_: if empty {
                0
            } else {
                current_string.GetCurrentChar()
            },
            current_string_: current_string,
            empty_: empty,
            ..Self::default()
        }
    }

    // cpp: html/parser/segmented_string.h:203
    // cpp: html/parser/segmented_string.cc:41-49
    pub fn Clear(&mut self) {
        self.current_string_.Clear();
        self.number_of_characters_consumed_prior_to_current_string_ = 0;
        self.number_of_characters_consumed_prior_to_current_line_ = 0;
        self.current_line_ = 0;
        self.substrings_.clear();
        self.closed_ = false;
        self.empty_ = true;
    }

    // cpp: html/parser/segmented_string.h:204
    // cpp: html/parser/segmented_string.cc:106-110
    pub fn Close(&mut self) {
        debug_assert!(!self.closed_);
        self.closed_ = true;
    }

    // cpp: html/parser/segmented_string.h:206
    // cpp: html/parser/segmented_string.cc:112-120
    pub fn Append(&mut self, source: &SegmentedString) {
        debug_assert!(!self.closed_);
        self.append_substring(&source.current_string_);
        if source.IsComposite() {
            for substring in &source.substrings_ {
                self.append_substring(substring);
            }
        }
    }

    // cpp: html/parser/segmented_string.h:211
    // cpp: html/parser/segmented_string.cc:122-130
    pub fn Prepend(&mut self, source: &SegmentedString, prepend_type: PrependType) {
        if source.IsComposite() {
            for substring in source.substrings_.iter().rev() {
                self.prepend_substring(substring, prepend_type);
            }
        }
        self.prepend_substring(&source.current_string_, prepend_type);
    }

    // cpp: html/parser/segmented_string.h:213-215
    pub fn NextSegmentedString(&self) -> *const SegmentedString {
        self.next_segmented_string_
    }

    // cpp: html/parser/segmented_string.h:216-218
    pub fn SetNextSegmentedString(&mut self, next: *const SegmentedString) {
        self.next_segmented_string_ = next;
    }

    // cpp: html/parser/segmented_string.h:225
    // cpp: html/parser/segmented_string.cc:67-81
    pub fn Push(&mut self, c: UChar) {
        assert_ne!(c, 0);
        if self.current_string_.PushIfPossible(c) {
            self.current_char_ = self.current_string_.GetCurrentChar();
            return;
        }
        let source = SegmentedString::from_string(&BlinkString::from_utf16(&[c]));
        self.Prepend(&source, PrependType::kUnconsume);
    }

    // cpp: html/parser/segmented_string.h:228
    // cpp: html/parser/segmented_string.cc:24-31
    pub fn length(&self) -> u32 {
        let mut length = self.current_string_.length() as u32;
        if self.IsComposite() {
            for substring in &self.substrings_ {
                length += substring.length() as u32;
            }
        }
        length
    }

    // cpp: html/parser/segmented_string.h:220-222
    pub fn ExcludeLineNumbers(&self) -> bool {
        self.current_string_.ExcludeLineNumbers()
    }

    // cpp: html/parser/segmented_string.h:223
    // cpp: html/parser/segmented_string.cc:33-39
    pub fn SetExcludeLineNumbers(&mut self) {
        self.current_string_.SetExcludeLineNumbers();
        if self.IsComposite() {
            for substring in &mut self.substrings_ {
                substring.SetExcludeLineNumbers();
            }
        }
    }

    // cpp: html/parser/segmented_string.h:227
    pub fn IsEmpty(&self) -> bool {
        self.empty_
    }

    // cpp: html/parser/segmented_string.h:230
    pub fn IsClosed(&self) -> bool {
        self.closed_
    }

    // cpp: html/parser/segmented_string.h:238-240
    pub fn LookAhead(&mut self, string: &BlinkString) -> LookAheadResult {
        self.LookAheadInline(string, true)
    }

    // cpp: html/parser/segmented_string.h:241-243
    pub fn LookAheadIgnoringCase(&mut self, string: &BlinkString) -> LookAheadResult {
        self.LookAheadInline(string, false)
    }

    // cpp: html/parser/segmented_string.h:249
    // cpp: html/parser/segmented_string.cc:132-149
    pub fn AdvanceWithCounts(&mut self, mut num_chars: u32, num_lines: u32, current_column: i32) {
        debug_assert!(num_chars <= self.length());
        self.current_line_ += num_lines as i32;
        while num_chars != 0 {
            num_chars -= self.current_string_.AdvanceMany(num_chars);
            if num_chars != 0 {
                debug_assert_eq!(self.current_string_.length(), 1);
                self.AdvanceSubstring();
                num_chars -= 1;
            }
        }
        self.number_of_characters_consumed_prior_to_current_line_ =
            self.NumberOfCharactersConsumed() - current_column;
        self.current_char_ = if self.empty_ {
            0
        } else {
            self.current_string_.GetCurrentChar()
        };
    }

    // cpp: html/parser/segmented_string.h:251-257
    pub fn Advance(&mut self) -> UChar {
        if self.current_string_.CanAdvance() {
            self.current_char_ = self.current_string_.AdvanceOne();
            return self.current_char_;
        }
        self.AdvanceSubstring()
    }

    // cpp: html/parser/segmented_string.h:259-267
    pub fn UpdateLineNumber(&mut self) {
        if self.current_string_.DoNotExcludeLineNumbers() {
            self.current_line_ += 1;
            self.number_of_characters_consumed_prior_to_current_line_ =
                self.NumberOfCharactersConsumed() + 1;
        }
    }

    // cpp: html/parser/segmented_string.h:269-274
    pub fn AdvanceAndUpdateLineNumber(&mut self) -> UChar {
        debug_assert!(self.current_string_.length() >= 1);
        if self.current_char_ == b'\n' as UChar {
            self.UpdateLineNumber();
        }
        self.Advance()
    }

    // cpp: html/parser/segmented_string.h:278-281
    pub fn AdvanceExpecting(&mut self, expected_character: UChar) -> UChar {
        debug_assert_eq!(expected_character, self.CurrentChar());
        self.Advance()
    }

    // cpp: html/parser/segmented_string.h:283-295
    pub fn AdvanceExpectingLiteral<const N: usize>(&mut self, expected: &[u8; N]) {
        assert!(
            N > 0 && expected[N - 1] == 0,
            "expected a NUL-terminated literal"
        );
        for &character in &expected[..N - 1] {
            debug_assert_eq!(self.CurrentChar(), character as UChar);
            self.Advance();
        }
    }

    // cpp: html/parser/segmented_string.h:297-311
    pub fn AdvanceExpectingIgnoringAsciiCase<const N: usize>(&mut self, expected: &[u8; N]) {
        assert!(
            N > 0 && expected[N - 1] == 0,
            "expected a NUL-terminated literal"
        );
        for &character in &expected[..N - 1] {
            let current = self.CurrentChar();
            debug_assert!(equal_ignoring_ascii_case(&[current], &[character as UChar]));
            self.Advance();
        }
    }

    // cpp: html/parser/segmented_string.h:313-316
    pub fn AdvancePastNonNewline(&mut self) -> UChar {
        debug_assert_ne!(self.CurrentChar(), b'\n' as UChar);
        self.Advance()
    }

    // cpp: html/parser/segmented_string.h:318-323
    pub fn AdvancePastNewlineAndUpdateLineNumber(&mut self) -> UChar {
        debug_assert_eq!(self.CurrentChar(), b'\n' as UChar);
        debug_assert!(self.current_string_.length() >= 1);
        self.UpdateLineNumber();
        self.Advance()
    }

    // cpp: html/parser/segmented_string.h:334
    pub fn CurrentChar(&self) -> UChar {
        self.current_char_
    }

    // cpp: html/parser/segmented_string.h:325-330
    pub fn NumberOfCharactersConsumed(&self) -> i32 {
        let number_of_pushed_characters = 0;
        self.number_of_characters_consumed_prior_to_current_string_
            + self.current_string_.NumberOfCharactersConsumed()
            - number_of_pushed_characters
    }

    // cpp: html/parser/segmented_string.h:336
    // cpp: html/parser/segmented_string.cc:189-191
    pub fn CurrentLine(&self) -> OrdinalNumber {
        OrdinalNumber::FromZeroBasedInt(self.current_line_)
    }

    // cpp: html/parser/segmented_string.h:335
    // cpp: html/parser/segmented_string.cc:193-197
    pub fn CurrentColumn(&self) -> OrdinalNumber {
        let zero_based_column = self.NumberOfCharactersConsumed()
            - self.number_of_characters_consumed_prior_to_current_line_;
        OrdinalNumber::FromZeroBasedInt(zero_based_column)
    }

    // cpp: html/parser/segmented_string.h:339-341
    // cpp: html/parser/segmented_string.cc:199-206
    pub fn SetCurrentPosition(
        &mut self,
        line: OrdinalNumber,
        column_aftre_prolog: OrdinalNumber,
        prolog_length: i32,
    ) {
        self.current_line_ = line.ZeroBasedInt();
        self.number_of_characters_consumed_prior_to_current_line_ =
            self.NumberOfCharactersConsumed() + prolog_length - column_aftre_prolog.ZeroBasedInt();
    }

    // cpp: html/parser/segmented_string.h:332
    // cpp: html/parser/segmented_string.cc:171-179
    pub fn ToString(&self) -> BlinkString {
        let mut result = StringBuilder::new();
        self.current_string_.AppendTo(&mut result);
        if self.IsComposite() {
            for substring in &self.substrings_ {
                substring.AppendTo(&mut result);
            }
        }
        result.ToString()
    }

    // cpp: html/parser/segmented_string.h:347
    // cpp: html/parser/segmented_string.cc:51-65
    fn append_substring(&mut self, substring: &SegmentedSubstring) {
        debug_assert!(!self.closed_);
        if substring.length() == 0 {
            return;
        }
        if self.current_string_.length() == 0 {
            self.number_of_characters_consumed_prior_to_current_string_ +=
                self.current_string_.NumberOfCharactersConsumed();
            self.current_string_ = substring.clone();
            self.current_char_ = self.current_string_.GetCurrentChar();
        } else {
            self.substrings_.push_back(substring.clone());
        }
        self.empty_ = false;
    }

    // cpp: html/parser/segmented_string.h:348
    // cpp: html/parser/segmented_string.cc:83-104
    fn prepend_substring(&mut self, substring: &SegmentedSubstring, prepend_type: PrependType) {
        debug_assert_eq!(substring.NumberOfCharactersConsumed(), 0);
        if substring.length() == 0 {
            return;
        }
        self.number_of_characters_consumed_prior_to_current_string_ +=
            self.current_string_.NumberOfCharactersConsumed();
        if prepend_type == PrependType::kUnconsume {
            self.number_of_characters_consumed_prior_to_current_string_ -= substring.length();
        }
        if self.current_string_.length() == 0 {
            self.current_string_ = substring.clone();
        } else {
            self.substrings_.push_front(self.current_string_.clone());
            self.current_string_ = substring.clone();
        }
        self.current_char_ = self.current_string_.GetCurrentChar();
        self.empty_ = false;
    }

    // cpp: html/parser/segmented_string.h:350
    // cpp: html/parser/segmented_string.cc:151-169
    fn AdvanceSubstring(&mut self) -> UChar {
        self.number_of_characters_consumed_prior_to_current_string_ +=
            self.current_string_.NumberOfCharactersConsumed() + 1;
        if self.IsComposite() {
            self.current_string_ = self
                .substrings_
                .pop_front()
                .expect("composite has a substring");
            self.number_of_characters_consumed_prior_to_current_string_ -=
                self.current_string_.NumberOfCharactersConsumed();
            self.current_char_ = self.current_string_.GetCurrentChar();
            return self.CurrentChar();
        }
        self.current_string_.Clear();
        self.empty_ = true;
        self.current_char_ = 0;
        0
    }

    // cpp: html/parser/segmented_string.h:354
    // cpp: html/parser/segmented_string.cc:181-187
    fn AdvanceAndCollect(&mut self, characters: &mut [UChar]) {
        assert!(characters.len() <= self.length() as usize);
        for character in characters {
            *character = self.CurrentChar();
            self.Advance();
        }
    }

    // cpp: html/parser/segmented_string.h:356-369
    fn LookAheadInline(&mut self, string: &BlinkString, case_sensitive: bool) -> LookAheadResult {
        let expected = StringView::from(string);
        if string.length() <= self.current_string_.length() as u32 {
            let current = self.current_string_.CurrentSubString(string.length());
            let matches = if case_sensitive {
                current == expected
            } else {
                equal_ignoring_ascii_case(current.Span16(), expected.Span16())
            };
            return if matches {
                LookAheadResult::kDidMatch
            } else {
                LookAheadResult::kDidNotMatch
            };
        }
        self.LookAheadSlowCase(string, case_sensitive)
    }

    // cpp: html/parser/segmented_string.h:371-388
    fn LookAheadSlowCase(&mut self, string: &BlinkString, case_sensitive: bool) -> LookAheadResult {
        let count = string.length();
        if count > self.length() {
            return LookAheadResult::kNotEnoughCharacters;
        }
        let mut consumed_characters = vec![0; count as usize];
        self.AdvanceAndCollect(&mut consumed_characters);
        let expected = string.Span16().unwrap_or(&[]);
        let matches = if case_sensitive {
            consumed_characters == expected
        } else {
            equal_ignoring_ascii_case(&consumed_characters, expected)
        };
        let result = if matches {
            LookAheadResult::kDidMatch
        } else {
            LookAheadResult::kDidNotMatch
        };
        let consumed_prefix =
            SegmentedString::from_string(&BlinkString::from_utf16(&consumed_characters));
        self.Prepend(&consumed_prefix, PrependType::kUnconsume);
        result
    }

    // cpp: html/parser/segmented_string.h:390
    fn IsComposite(&self) -> bool {
        !self.substrings_.is_empty()
    }
}
