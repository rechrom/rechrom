/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: url_input_type.h:38-55; url_input_type.cc:43-68.
use super::{
    base_text_input_type::{BaseTextInputType, ScriptRegexpFactory},
    input_type::Type,
    text_field_input_type::{TextFieldInputType, TextString},
};
use std::rc::Rc;
/// KURL(NullUrl(), value).IsValid() from the existing URL owner. This port
/// does not substitute a scheme heuristic or a second URL parser.
pub trait URLInputValidation {
    fn IsValidAbsoluteURL(&self, value: &[u16]) -> bool;
    fn URLTypeMismatchText(&self) -> TextString;
}
pub struct URLInputType {
    pub base_text: BaseTextInputType,
    validation: Rc<dyn URLInputValidation>,
}
impl URLInputType {
    pub fn AllowFrom(input_type: &super::input_type::InputType) -> bool {
        input_type.r#type() == Type::kURL
    }
    pub fn CountUsage(&self, counter: &dyn super::input_type::InputTypeUsageCounter) {
        counter.CountUsageIfVisible(super::input_type::InputTypeUsageFeature::URL);
    }
    pub fn new(
        mut field: TextFieldInputType,
        regexp: Rc<dyn ScriptRegexpFactory>,
        validation: Rc<dyn URLInputValidation>,
    ) -> Self {
        field.input_type = Type::kURL;
        Self {
            base_text: BaseTextInputType::new(field, regexp),
            validation,
        }
    }
    pub fn TypeMismatchFor(&self, value: &[u16]) -> bool {
        !value.is_empty() && !self.validation.IsValidAbsoluteURL(value)
    }
    pub fn TypeMismatch(&self) -> bool {
        self.TypeMismatchFor(&self.base_text.text_field.value.Value())
    }
    pub fn TypeMismatchText(&self) -> TextString {
        self.validation.URLTypeMismatchText()
    }
    pub fn SanitizeValue(&self, proposed: &[u16]) -> TextString {
        Self::SanitizeValueFor(proposed)
    }
    pub fn SanitizeValueFor(proposed: &[u16]) -> TextString {
        TextFieldInputType::SanitizeValueFor(StripLeadingAndTrailingHtmlSpaces(proposed))
    }
    pub fn SanitizeUserInputValue(&self, proposed: &[u16]) -> TextString {
        self.base_text.text_field.SanitizeValue(proposed)
    }
}
// Source: html_parser_idioms.h:42; html_parser_idioms.cc:40-45. The HTML whitespace
// predicate excludes vertical tab. This borrowed span keeps UTF-16 units intact.
pub(super) fn StripLeadingAndTrailingHtmlSpaces(mut value: &[u16]) -> &[u16] {
    while value.first().is_some_and(|&ch| IsHTMLSpace(ch)) {
        value = &value[1..];
    }
    while value.last().is_some_and(|&ch| IsHTMLSpace(ch)) {
        value = &value[..value.len() - 1];
    }
    value
}
fn IsHTMLSpace(ch: u16) -> bool {
    matches!(ch, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}
