// Copyright (C) 2009 Michelangelo De Simone <micdesim@gmail.com>
// Copyright (C) 2010 Google Inc. All rights reserved.
// This library is free software under the GNU Library General Public License,
// version 2 or (at your option) any later version; see Chromium COPYING.LIB.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/base_text_input_type.h:1-78
// Source: third_party/blink/renderer/core/html/forms/base_text_input_type.cc:1-154

use super::{
    input_type::Type,
    text_field_input_type::{TextFieldInputType, TextString, TextValue},
};
use std::{cell::RefCell, rc::Rc};

// Source: text_control_element.h:65-68.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NeedsToCheckDirtyFlag {
    kCheckDirtyFlag,
    kIgnoreDirtyFlag,
}

/// The canonical script regexp owner, including UnicodeSets mode and error.
pub trait ScriptRegexp {
    fn IsValid(&self) -> bool;
    fn ExceptionMessage(&self) -> String;
    fn Match(&self, value: &[u16], start: usize) -> (i32, usize);
}
pub trait ScriptRegexpFactory {
    fn CreateUnicodeSetsCaseSensitive(&self, pattern: &[u16]) -> Rc<dyn ScriptRegexp>;
    fn InvalidPatternConsoleMessage(&self, raw: &[u16], error: &str);
}

// Source: base_text_input_type.h:42-67. Cache stays with this owner.
pub struct BaseTextInputType {
    pub text_field: TextFieldInputType,
    pattern: PatternCache,
    regexp_factory: Rc<dyn ScriptRegexpFactory>,
}
impl BaseTextInputType {
    // Source: base_text_input_type.cc:40-56. Rc/Drop supplies source tracing.
    pub fn new(
        text_field: TextFieldInputType,
        regexp_factory: Rc<dyn ScriptRegexpFactory>,
    ) -> Self {
        Self {
            text_field,
            pattern: PatternCache::default(),
            regexp_factory,
        }
    }
    pub fn MaxLength(&self) -> i32 {
        self.text_field.value.MaxLength()
    }
    pub fn MinLength(&self) -> i32 {
        self.text_field.value.MinLength()
    }
    // Source: base_text_input_type.cc:58-71.
    pub fn TooLong(&self, value: &[u16], check: NeedsToCheckDirtyFlag) -> bool {
        Self::TooLongWithState(
            value,
            self.MaxLength(),
            check,
            self.text_field.value.HasDirtyValue(),
            self.text_field.value.LastChangeWasUserEdit(),
        )
    }
    pub fn TooLongWithState(
        value: &[u16],
        max: i32,
        check: NeedsToCheckDirtyFlag,
        dirty: bool,
        user_edit: bool,
    ) -> bool {
        max >= 0
            && (check == NeedsToCheckDirtyFlag::kIgnoreDirtyFlag || (dirty && user_edit))
            && value.len() > max as usize
    }
    // Source: base_text_input_type.cc:73-88.
    pub fn TooShort(&self, value: &[u16], check: NeedsToCheckDirtyFlag) -> bool {
        Self::TooShortWithState(
            value,
            self.MinLength(),
            check,
            self.text_field.value.HasDirtyValue(),
            self.text_field.value.LastChangeWasUserEdit(),
        )
    }
    pub fn TooShortWithState(
        value: &[u16],
        min: i32,
        check: NeedsToCheckDirtyFlag,
        dirty: bool,
        user_edit: bool,
    ) -> bool {
        min > 0
            && (check == NeedsToCheckDirtyFlag::kIgnoreDirtyFlag || (dirty && user_edit))
            && !value.is_empty()
            && value.len() < min as usize
    }
    // Source: base_text_input_type.cc:90-100.
    pub fn PatternMismatch(&self, value: &[u16]) -> bool {
        if self.text_field.input_type == Type::kEmail && self.text_field.value.Multiple() {
            for value in super::email_input_type::EmailInputType::ParseMultipleValues(value) {
                if self.PatternMismatchPerValue(value) {
                    return true;
                }
            }
            return false;
        }
        self.PatternMismatchPerValue(value)
    }
    fn PatternMismatchPerValue(&self, value: &[u16]) -> bool {
        self.text_field.value.Pattern().is_some_and(|raw| {
            self.pattern
                .PatternMismatchPerValue(raw, value, self.regexp_factory.as_ref())
        })
    }
    // Source: base_text_input_type.cc:142-152.
    pub fn SupportsPlaceholder(&self) -> bool {
        true
    }
    pub fn SupportsSelectionAPI(&self) -> bool {
        true
    }
    pub fn IsAutoDirectionalityFormAssociated(&self) -> bool {
        true
    }
}

/// InputType-owned compiled pattern. Reused by the production DOM adapter;
/// changing the attribute recompiles on this same owner.
#[derive(Default)]
pub struct PatternCache {
    regexp_: RefCell<Option<Rc<dyn ScriptRegexp>>>,
    pattern_for_regexp_: RefCell<Option<TextValue>>,
}
impl PatternCache {
    // Source: base_text_input_type.cc:102-140.
    pub fn PatternMismatchPerValue(
        &self,
        raw: TextValue,
        value: &[u16],
        factory: &dyn ScriptRegexpFactory,
    ) -> bool {
        if value.is_empty() {
            return false;
        }
        let missing = self.regexp_.borrow().is_none();
        if missing || self.pattern_for_regexp_.borrow().as_ref() != Some(&raw) {
            let regexp = factory.CreateUnicodeSetsCaseSensitive(&raw);
            if !regexp.IsValid() {
                factory.InvalidPatternConsoleMessage(&raw, &regexp.ExceptionMessage());
                *self.regexp_.borrow_mut() = Some(regexp);
                *self.pattern_for_regexp_.borrow_mut() = Some(raw);
                return false;
            }
            let mut pattern: TextString = "^(?:".encode_utf16().collect();
            pattern.extend_from_slice(&raw);
            pattern.extend(")$".encode_utf16());
            *self.regexp_.borrow_mut() = Some(factory.CreateUnicodeSetsCaseSensitive(&pattern));
            *self.pattern_for_regexp_.borrow_mut() = Some(raw);
        } else if !self.regexp_.borrow().as_ref().unwrap().IsValid() {
            return false;
        }
        let regexp = self.regexp_.borrow().as_ref().unwrap().clone();
        let (offset, length) = regexp.Match(value, 0);
        offset != 0 || length != value.len()
    }
}
