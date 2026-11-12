// Copyright (C) 2010 Google Inc. All rights reserved.
// Use of this source code is governed by the BSD license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/text_input_type.h:1-57
// Source: third_party/blink/renderer/core/html/forms/text_input_type.cc:1-56

use super::{
    base_text_input_type::{BaseTextInputType, ScriptRegexpFactory},
    input_type::Type,
    text_field_input_type::TextFieldInputType,
};
use std::rc::Rc;

pub struct TextInputType {
    pub base_text: BaseTextInputType,
}
impl TextInputType {
    // Source: text_input_type.h:38-46.
    pub fn new(
        mut text_field: TextFieldInputType,
        regexp_factory: Rc<dyn ScriptRegexpFactory>,
    ) -> Self {
        text_field.input_type = Type::kText;
        Self {
            base_text: BaseTextInputType::new(text_field, regexp_factory),
        }
    }
    // Source: text_input_type.cc:39-50.
    // TODO: CountUsageIfVisible InputTypeText/MaxLength/fallback counters.
    pub fn CountUsage(&self) {}
    // Source: text_input_type.cc:52-54.
    pub fn SupportsInputModeAttribute(&self) -> bool {
        true
    }
}
