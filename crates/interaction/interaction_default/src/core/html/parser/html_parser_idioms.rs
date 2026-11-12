/*
 * Copyright (C) 2010 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1.  Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 * 2.  Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE INC. AND ITS CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL APPLE INC. OR ITS CONTRIBUTORS BE LIABLE FOR
 * ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/parser/html_parser_idioms.cc:58-134
// Source: third_party/blink/renderer/core/html/parser/html_parser_idioms.h:47-62

use crate::platform::wtf::decimal::{Decimal, NumberToStringEcmaScript, StringToDouble};
use std::rc::Rc;

// Source: html_parser_idioms.cc:58-64.
pub fn SerializeForNumberType(number: Decimal) -> String {
    if number.IsZero() {
        return if number.IsNegative() { "-0" } else { "0" }.into();
    }
    number.ToString()
}
// Source: html_parser_idioms.cc:66-70 (the C++ double overload).
pub fn SerializeDoubleForNumberType(
    number: f64,
    formatter: &dyn NumberToStringEcmaScript,
) -> String {
    formatter.NumberToStringEcmaScript(number)
}
// Source: html_parser_idioms.cc:72-96.
pub fn ParseToDecimalForNumberType(
    string: &str,
    fallback_value: Decimal,
    formatter: &dyn NumberToStringEcmaScript,
) -> Decimal {
    let first_character = string.encode_utf16().next().unwrap_or(0);
    if first_character != b'-' as u16
        && first_character != b'.' as u16
        && !(b'0' as u16..=b'9' as u16).contains(&first_character)
    {
        return fallback_value;
    }
    let value = Decimal::FromString(string);
    if !value.IsFinite() {
        return fallback_value;
    }
    let double_max = Decimal::FromDouble(f64::MAX, formatter);
    if value < -double_max || value > double_max {
        return fallback_value;
    }
    if value.IsZero() {
        Decimal::FromInt(0)
    } else {
        value
    }
}
// Source: html_parser_idioms.cc:98-117.
fn CheckDoubleValue(value: f64, valid: bool, fallback_value: f64) -> f64 {
    if !valid {
        return fallback_value;
    }
    if !value.is_finite() {
        return fallback_value;
    }
    if -f64::MAX > value || value > f64::MAX {
        return fallback_value;
    }
    if value != 0.0 {
        value
    } else {
        0.0
    }
}
// Source: html_parser_idioms.cc:119-134.
pub fn ParseToDoubleForNumberType(
    string: &str,
    fallback_value: f64,
    parser: &dyn StringToDouble,
) -> f64 {
    let first_character = string.encode_utf16().next().unwrap_or(0);
    if first_character != b'-' as u16
        && first_character != b'.' as u16
        && !(b'0' as u16..=b'9' as u16).contains(&first_character)
    {
        return fallback_value;
    }
    if string.ends_with('.') {
        return fallback_value;
    }
    let value = parser.StringToDouble(string);
    CheckDoubleValue(value.unwrap_or(0.0), value.is_some(), fallback_value)
}

// Source assembly for the existing StepRange/RangeInputType parser interface.
// The two conversion algorithms are explicit constructor dependencies. This
// owner does not install a guessed formatter/parser when a leaf is missing.
pub struct NumberTypeParser {
    formatter_: Rc<dyn NumberToStringEcmaScript>,
    parser_: Rc<dyn StringToDouble>,
}
impl NumberTypeParser {
    pub fn new(
        formatter: Rc<dyn NumberToStringEcmaScript>,
        parser: Rc<dyn StringToDouble>,
    ) -> Self {
        Self {
            formatter_: formatter,
            parser_: parser,
        }
    }
    pub fn FromDoubleConversion(
        converter: Rc<dyn crate::platform::wtf::dtoa::DoubleConversion>,
    ) -> Self {
        Self::new(
            Rc::new(crate::platform::wtf::dtoa::DoubleToStringConverter::new(
                converter.clone(),
            )),
            Rc::new(
                crate::platform::wtf::text::string_to_number::StringToNumberConverter::new(
                    converter,
                ),
            ),
        )
    }
    pub fn SerializeDoubleForNumberType(&self, value: f64) -> String {
        SerializeDoubleForNumberType(value, self.formatter_.as_ref())
    }
    pub fn FromDouble(&self, value: f64) -> Decimal {
        Decimal::FromDouble(value, self.formatter_.as_ref())
    }
    pub fn ToDouble(&self, value: Decimal) -> f64 {
        value.ToDouble(self.parser_.as_ref())
    }
}
impl crate::core::html::forms::step_range::NumberTypeParser<Decimal> for NumberTypeParser {
    fn ParseToDecimalForNumberType(&self, value: &str, default: Decimal) -> Decimal {
        ParseToDecimalForNumberType(value, default, self.formatter_.as_ref())
    }
    fn ParseToDoubleForNumberType(&self, value: &str) -> f64 {
        ParseToDoubleForNumberType(value, f64::NAN, self.parser_.as_ref())
    }
    fn SerializeForNumberType(&self, value: Decimal) -> String {
        SerializeForNumberType(value)
    }
}
