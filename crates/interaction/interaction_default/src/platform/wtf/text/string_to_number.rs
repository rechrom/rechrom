// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/wtf/text/string_to_number.cc:229-266,361-364
// Only the no-trailing-junk double path required by Decimal and HTML number input.

use crate::platform::wtf::dtoa::{DoubleConversion, ParseDoubleUtf16};
use std::rc::Rc;

// Source: string_to_number.cc:229-266. The caller uses kDisallowTrailingJunk.
fn ToDoubleType(data: &[u16], converter: &dyn DoubleConversion) -> Option<f64> {
    let length = data.len();
    let mut leading_spaces_length = 0;
    while leading_spaces_length < length && IsAsciiSpace(data[leading_spaces_length]) {
        leading_spaces_length += 1;
    }
    let parsed = ParseDoubleUtf16(&data[leading_spaces_length..], converter);
    if parsed.parsed_length == 0 {
        return None;
    }
    let parsed_length = parsed.parsed_length + leading_spaces_length;
    if parsed_length != length {
        return None;
    }
    Some(parsed.value)
}
// Source: wtf/text/ascii_ctype.h:101-104 (ASCII space or HT through CR).
fn IsAsciiSpace(ch: u16) -> bool {
    ch == 0x20 || (0x09..=0x0d).contains(&ch)
}

pub struct StringToNumberConverter {
    converter_: Rc<dyn DoubleConversion>,
}
impl StringToNumberConverter {
    pub fn new(converter: Rc<dyn DoubleConversion>) -> Self {
        Self {
            converter_: converter,
        }
    }
}
// Source: string_to_number.cc:361-364. Rust strings are projected to the source
// UChar overload; the numeric parser receives the original UTF-16 code-unit count.
impl crate::platform::wtf::decimal::StringToDouble for StringToNumberConverter {
    fn StringToDouble(&self, input: &str) -> Option<f64> {
        let data: Vec<u16> = input.encode_utf16().collect();
        ToDoubleType(&data, self.converter_.as_ref())
    }
}

// Source: string_to_number.cc:32-151,357-359, uint32/decimal Loose branch;
// number_parsing_options.h:20-29; unicode.h:176-180. The non-ASCII whitespace
// table is Bidi_Class=WS from this revision's ICU UnicodeData.txt.
pub fn StringToUintLoose(input: &[u16]) -> Option<u32> {
    let mut index = 0;
    while index < input.len() && IsIntegralWhitespace(input[index]) {
        index += 1;
    }
    if input.get(index) == Some(&(b'+' as u16)) {
        index += 1;
    }
    let first = index;
    let mut value = 0u32;
    while index < input.len() && (b'0' as u16..=b'9' as u16).contains(&input[index]) {
        value = value
            .checked_mul(10)?
            .checked_add(u32::from(input[index] - b'0' as u16))?;
        index += 1;
    }
    (index != first).then_some(value)
}
fn IsIntegralWhitespace(ch: u16) -> bool {
    if ch <= 0x7f {
        return ch == 0x20 || (0x09..=0x0d).contains(&ch);
    }
    matches!(
        ch,
        0x1680
            | 0x2000
            | 0x2001
            | 0x2002
            | 0x2003
            | 0x2004
            | 0x2005
            | 0x2006
            | 0x2007
            | 0x2008
            | 0x2009
            | 0x200a
            | 0x2028
            | 0x205f
            | 0x3000
    )
}

// Source: string_to_number.cc:32-151,353-355, signed int/decimal Loose branch.
pub fn StringToIntLoose(input: &[u16]) -> Option<i32> {
    let mut index = 0;
    while index < input.len() && IsIntegralWhitespace(input[index]) {
        index += 1;
    }
    let negative = input.get(index) == Some(&(b'-' as u16));
    if negative || input.get(index) == Some(&(b'+' as u16)) {
        index += 1;
    }
    let first = index;
    let mut value = 0u32;
    let limit = if negative {
        i32::MAX as u32 + 1
    } else {
        i32::MAX as u32
    };
    while index < input.len() && (b'0' as u16..=b'9' as u16).contains(&input[index]) {
        value = value
            .checked_mul(10)?
            .checked_add(u32::from(input[index] - b'0' as u16))?;
        if value > limit {
            return None;
        }
        index += 1;
    }
    if index == first {
        return None;
    }
    Some(if negative {
        -(i64::from(value)) as i32
    } else {
        value as i32
    })
}
