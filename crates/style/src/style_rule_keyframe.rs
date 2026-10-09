// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_keyframe.h
// cpp: third_party/blink/renderer/core/css/style_rule_keyframe.cc
// Oilpan tracing/allocation is represented by Rc ownership.

#![allow(non_snake_case, non_camel_case_types)]

use crate::style_rule::StyleRulePropertySet;
use foundation::String;
use std::cell::RefCell;
use std::rc::Rc;

// Generated V8TimelineRange::Enum dependency used by TimelineOffset.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimelineNamedRange {
    #[default]
    kNone,
    kCover,
    kContain,
    kEntry,
    kEntryCrossing,
    kExit,
    kExitCrossing,
    kScroll,
}

// cpp: style_rule_keyframe.h:17-30
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyframeOffset {
    pub name: TimelineNamedRange,
    pub percent: f64,
}
impl KeyframeOffset {
    pub fn new(name: TimelineNamedRange, percent: f64) -> Self {
        Self { name, percent }
    }
}
impl Default for KeyframeOffset {
    fn default() -> Self {
        Self::new(TimelineNamedRange::kNone, 0.0)
    }
}

// Parser, generated-enum serialization and StylePropertySerializer are real
// external dependencies. No method has a permissive default.
pub trait StyleRuleKeyframeBackend: Sized {
    type ExecutionContext;
    type PropertySet: StyleRulePropertySet;
    fn ParseKeyframeKeyList(
        context: &Self::ExecutionContext,
        text: &String,
    ) -> Option<Vec<KeyframeOffset>>;
    fn TimelineRangeNameToString(name: TimelineNamedRange) -> String;
    fn FormatNumber(value: f64) -> String;
    fn PropertiesAsText(properties: &Self::PropertySet) -> String;
}

// cpp: style_rule_keyframe.h:32-57
pub struct StyleRuleKeyframe<D: StyleRuleKeyframeBackend> {
    properties_: RefCell<Rc<D::PropertySet>>,
    keys_: Vec<KeyframeOffset>,
}
impl<D: StyleRuleKeyframeBackend> StyleRuleKeyframe<D> {
    // cpp: style_rule_keyframe.cc:15-19
    pub fn new(keys: Vec<KeyframeOffset>, properties: Rc<D::PropertySet>) -> Self {
        Self {
            properties_: RefCell::new(properties),
            keys_: keys,
        }
    }

    // cpp: style_rule_keyframe.cc:21-40
    pub fn KeyText(&self) -> String {
        debug_assert!(!self.keys_.is_empty());
        let mut units = Vec::new();
        for (index, key) in self.keys_.iter().enumerate() {
            if index != 0 {
                units.extend_from_slice(&[b',' as u16, b' ' as u16]);
            }
            if key.name != TimelineNamedRange::kNone {
                let range = D::TimelineRangeNameToString(key.name);
                units.extend_from_slice(range.Span16().unwrap_or_default());
                units.push(b' ' as u16);
            }
            let number = D::FormatNumber(key.percent * 100.0);
            units.extend_from_slice(number.Span16().unwrap_or_default());
            units.push(b'%' as u16);
        }
        String::from_utf16(&units)
    }

    // cpp: style_rule_keyframe.cc:42-56
    pub fn SetKeyText(&mut self, context: &D::ExecutionContext, key_text: &String) -> bool {
        let Some(keys) = D::ParseKeyframeKeyList(context, key_text) else {
            return false;
        };
        if keys.is_empty() {
            return false;
        }
        self.keys_ = keys;
        true
    }

    // cpp: style_rule_keyframe.cc:58-60
    pub fn Keys(&self) -> &[KeyframeOffset] {
        &self.keys_
    }
    pub fn Properties(&self) -> Rc<D::PropertySet> {
        self.properties_.borrow().clone()
    }

    // cpp: style_rule_keyframe.cc:62-68
    pub fn MutableProperties(&self) -> Rc<D::PropertySet> {
        let current = self.properties_.borrow().clone();
        if !current.IsMutable() {
            *self.properties_.borrow_mut() = current.MutableCopy();
        }
        let result = self.properties_.borrow().clone();
        debug_assert!(result.IsMutable());
        result
    }

    // cpp: style_rule_keyframe.cc:70-81
    pub fn CssText(&self) -> String {
        let key_text = self.KeyText();
        let properties = self.properties_.borrow();
        let declarations = D::PropertiesAsText(properties.as_ref());
        let mut units = Vec::new();
        units.extend_from_slice(key_text.Span16().unwrap_or_default());
        units.extend_from_slice(&[b' ' as u16, b'{' as u16, b' ' as u16]);
        units.extend_from_slice(declarations.Span16().unwrap_or_default());
        if !declarations.empty() {
            units.push(b' ' as u16);
        }
        units.push(b'}' as u16);
        String::from_utf16(&units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css_property_names::CSSPropertyID;

    struct Property(bool);
    impl StyleRulePropertySet for Property {
        type CSSValue = ();
        fn IsMutable(&self) -> bool {
            self.0
        }
        fn MutableCopy(&self) -> Rc<Self> {
            Rc::new(Self(true))
        }
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            false
        }
        fn GetPropertyCSSValue(&self, _: CSSPropertyID) -> Option<Rc<Self::CSSValue>> {
            None
        }
    }
    struct Backend;
    impl StyleRuleKeyframeBackend for Backend {
        type ExecutionContext = ();
        type PropertySet = Property;
        fn ParseKeyframeKeyList(_: &(), text: &String) -> Option<Vec<KeyframeOffset>> {
            (text.Utf8() == "to").then(|| vec![KeyframeOffset::new(TimelineNamedRange::kNone, 1.0)])
        }
        fn TimelineRangeNameToString(name: TimelineNamedRange) -> String {
            String::from(match name {
                TimelineNamedRange::kEntry => "entry",
                _ => "none",
            })
        }
        fn FormatNumber(value: f64) -> String {
            let formatted = format!("{value}");
            String::from(formatted.as_str())
        }
        fn PropertiesAsText(_: &Property) -> String {
            String::from("opacity: 1;")
        }
    }

    #[test]
    fn serializes_named_ranges_and_rejects_failed_key_updates() {
        let mut rule = StyleRuleKeyframe::<Backend>::new(
            vec![
                KeyframeOffset::new(TimelineNamedRange::kEntry, 0.25),
                KeyframeOffset::new(TimelineNamedRange::kNone, 1.0),
            ],
            Rc::new(Property(false)),
        );
        assert_eq!(rule.KeyText().Utf8(), "entry 25%, 100%");
        assert_eq!(rule.CssText().Utf8(), "entry 25%, 100% { opacity: 1; }");
        assert!(!rule.SetKeyText(&(), &String::from("bad")));
        assert_eq!(rule.Keys().len(), 2);
        assert!(rule.SetKeyText(&(), &String::from("to")));
        assert_eq!(rule.KeyText().Utf8(), "100%");
        assert!(rule.MutableProperties().IsMutable());
    }
}
