#![allow(non_snake_case)]

use std::ffi::{c_char, c_double};

// cpp: css_parser/length_percentage_parser.h:13-19
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResolvedLengthPercentage {
    pub pixels: f64,
    pub percentage: f64,
    pub has_pixels: bool,
    pub has_percentage: bool,
    pub calculated: bool,
}

// cpp: css_parser/length_percentage_parser.h:23-41
fn Trim(value: &str) -> String {
    value
        .trim_matches(|character: char| character.is_ascii_whitespace())
        .to_owned()
}
fn Lower(value: &str) -> String {
    value.to_ascii_lowercase()
}

// cpp: css_parser/length_percentage_parser.h:43-54
pub fn AbsoluteLengthScale(unit: &[u8]) -> Option<f64> {
    match unit {
        b"px" => Some(1.0),
        b"in" => Some(96.0),
        b"cm" => Some(96.0 / 2.54),
        b"mm" => Some(96.0 / 25.4),
        b"q" => Some(96.0 / 101.6),
        b"pt" => Some(96.0 / 72.0),
        b"pc" => Some(16.0),
        _ => None,
    }
}

// cpp: css_parser/length_percentage_parser.h:73-79
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    kNumber,
    kLength,
}
#[derive(Clone, Copy, Debug)]
struct Value {
    kind: Kind,
    number: f64,
    length: ResolvedLengthPercentage,
}
impl Default for Value {
    fn default() -> Self {
        Self {
            kind: Kind::kNumber,
            number: 0.0,
            length: ResolvedLengthPercentage::default(),
        }
    }
}

// cpp: css_parser/length_percentage_parser.h:56-59,207-210
struct CalcLengthParser<'a> {
    input_: &'a [u8],
    cursor_: usize,
    font_size_: f64,
}

impl<'a> CalcLengthParser<'a> {
    fn new(input: &'a [u8], font_size: f64) -> Self {
        Self {
            input_: input,
            cursor_: 0,
            font_size_: font_size,
        }
    }

    // cpp: css_parser/length_percentage_parser.h:61-71
    fn Parse(&mut self) -> Option<ResolvedLengthPercentage> {
        if !self.font_size_.is_finite() {
            return None;
        }
        self.SkipWhitespace();
        let value = self.ParseSum();
        self.SkipWhitespace();
        let value = value?;
        if self.cursor_ != self.input_.len() || value.kind == Kind::kNumber {
            return None;
        }
        let mut result = value.length;
        result.calculated = true;
        Some(result)
    }

    // cpp: css_parser/length_percentage_parser.h:81-85
    fn SkipWhitespace(&mut self) {
        while self.cursor_ < self.input_.len() && self.input_[self.cursor_].is_ascii_whitespace() {
            self.cursor_ += 1;
        }
    }

    // cpp: css_parser/length_percentage_parser.h:87-108
    fn ParseSum(&mut self) -> Option<Value> {
        let mut left = self.ParseProduct()?;
        loop {
            self.SkipWhitespace();
            if self.cursor_ == self.input_.len()
                || !matches!(self.input_[self.cursor_], b'+' | b'-')
            {
                return Some(left);
            }
            let operation = self.input_[self.cursor_];
            self.cursor_ += 1;
            let right = self.ParseProduct()?;
            if right.kind != left.kind {
                return None;
            }
            let sign = if operation == b'+' { 1.0 } else { -1.0 };
            if left.kind == Kind::kNumber {
                left.number += sign * right.number;
            } else {
                left.length.pixels += sign * right.length.pixels;
                left.length.percentage += sign * right.length.percentage;
                left.length.has_pixels |= right.length.has_pixels;
                left.length.has_percentage |= right.length.has_percentage;
            }
        }
    }

    // cpp: css_parser/length_percentage_parser.h:110-140
    fn ParseProduct(&mut self) -> Option<Value> {
        let mut left = self.ParseValue()?;
        loop {
            self.SkipWhitespace();
            if self.cursor_ == self.input_.len()
                || !matches!(self.input_[self.cursor_], b'*' | b'/')
            {
                return Some(left);
            }
            let operation = self.input_[self.cursor_];
            self.cursor_ += 1;
            let right = self.ParseValue()?;
            if operation == b'/' {
                if right.kind != Kind::kNumber || right.number == 0.0 {
                    return None;
                }
                Self::Scale(&mut left, 1.0 / right.number);
            } else if left.kind == Kind::kNumber && right.kind == Kind::kLength {
                let factor = left.number;
                left = right;
                Self::Scale(&mut left, factor);
            } else if left.kind == Kind::kLength && right.kind == Kind::kNumber {
                Self::Scale(&mut left, right.number);
            } else if left.kind == Kind::kNumber && right.kind == Kind::kNumber {
                left.number *= right.number;
            } else {
                return None;
            }
        }
    }

    // cpp: css_parser/length_percentage_parser.h:142-149
    fn Scale(value: &mut Value, factor: f64) {
        if value.kind == Kind::kNumber {
            value.number *= factor;
        } else {
            value.length.pixels *= factor;
            value.length.percentage *= factor;
        }
    }

    // cpp: css_parser/length_percentage_parser.h:151-205
    fn ParseValue(&mut self) -> Option<Value> {
        self.SkipWhitespace();
        let mut sign = 1.0;
        if self.cursor_ < self.input_.len() && matches!(self.input_[self.cursor_], b'+' | b'-') {
            if self.input_[self.cursor_] == b'-' {
                sign = -1.0;
            }
            self.cursor_ += 1;
            self.SkipWhitespace();
        }
        if self.cursor_ < self.input_.len() && self.input_[self.cursor_] == b'(' {
            self.cursor_ += 1;
            let mut value = self.ParseSum()?;
            self.SkipWhitespace();
            if self.cursor_ == self.input_.len() || self.input_[self.cursor_] != b')' {
                return None;
            }
            self.cursor_ += 1;
            Self::Scale(&mut value, sign);
            return Some(value);
        }
        let start = self.cursor_;
        let mut remaining = self.input_[self.cursor_..].to_vec();
        remaining.push(0);
        let begin = remaining.as_ptr().cast::<c_char>();
        let mut end = std::ptr::null_mut();
        unsafe extern "C" {
            fn strtod(begin: *const c_char, end: *mut *mut c_char) -> c_double;
        }
        let number = unsafe { strtod(begin, &mut end) };
        let consumed = unsafe { end.offset_from(begin) };
        if consumed == 0 || !number.is_finite() {
            return None;
        }
        self.cursor_ += consumed as usize;
        let unit_start = self.cursor_;
        while self.cursor_ < self.input_.len()
            && (self.input_[self.cursor_].is_ascii_alphabetic()
                || self.input_[self.cursor_] == b'%')
        {
            self.cursor_ += 1;
        }
        let unit = self.input_[unit_start..self.cursor_].to_ascii_lowercase();
        let mut value = Value::default();
        if unit.is_empty() {
            value.number = sign * number;
            return Some(value);
        }
        value.kind = Kind::kLength;
        if unit == b"%" {
            value.length.percentage = sign * number;
            value.length.has_percentage = true;
        } else if let Some(scale) = AbsoluteLengthScale(&unit) {
            value.length.pixels = sign * number * scale;
            value.length.has_pixels = true;
        } else if unit == b"em" {
            value.length.pixels = sign * number * self.font_size_;
            value.length.has_pixels = true;
        } else if unit == b"rem" {
            value.length.pixels = sign * number * 16.0;
            value.length.has_pixels = true;
        } else {
            self.cursor_ = start;
            return None;
        }
        Some(value)
    }
}

// cpp: css_parser/length_percentage_parser.h:214-224
pub fn ParseLengthPercentage(input: &str, font_size: f64) -> Option<ResolvedLengthPercentage> {
    let value = Lower(&Trim(input));
    if !value.starts_with("calc(") || !value.ends_with(')') {
        return None;
    }
    CalcLengthParser::new(&value.as_bytes()[5..value.len() - 1], font_size).Parse()
}
