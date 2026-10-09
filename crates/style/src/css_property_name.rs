// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/css_property_name.h:19-68
// cpp: third_party/blink/renderer/core/css/css_property_name.cc:25-48

use crate::css_property_names::GetPropertyNameAtomicString;
use foundation::{kLastUnresolvedCSSProperty, AtomicString, CSSPropertyID};
use std::hash::{Hash, Hasher};

// Represents the name of any valid CSS property, including custom properties.
#[derive(Clone, Debug)]
pub struct CSSPropertyName {
    value_: i32,
    custom_property_name_: AtomicString,
}

impl CSSPropertyName {
    pub fn new(property_id: CSSPropertyID) -> Self {
        debug_assert_ne!(property_id, CSSPropertyID::kInvalid);
        debug_assert_ne!(property_id, CSSPropertyID::kVariable);
        Self {
            value_: property_id as i32,
            custom_property_name_: AtomicString::default(),
        }
    }

    pub fn custom(custom_property_name: AtomicString) -> Self {
        debug_assert!(!custom_property_name.IsNull());
        Self {
            value_: CSSPropertyID::kVariable as i32,
            custom_property_name_: custom_property_name,
        }
    }

    pub fn Id(&self) -> CSSPropertyID {
        // cpp: css_property_name.h:50-53 performs a direct cast because this
        // type deliberately also stores unresolved alias IDs, which sit after
        // kLastCSSProperty in the generated contiguous enum.
        debug_assert!(
            (CSSPropertyID::kInvalid as i32..=kLastUnresolvedCSSProperty as i32)
                .contains(&self.value_)
        );
        // SAFETY: constructors only store a generated CSSPropertyID value.
        unsafe { std::mem::transmute::<i32, CSSPropertyID>(self.value_) }
    }

    pub fn IsCustomProperty(&self) -> bool {
        self.Id() == CSSPropertyID::kVariable
    }

    // cpp: css_property_name.cc:36-41
    pub fn ToAtomicString(&self) -> &AtomicString {
        if self.IsCustomProperty() {
            &self.custom_property_name_
        } else {
            GetPropertyNameAtomicString(self.Id())
        }
    }

    // This is the custom-property arm of ToAtomicString(). Native property
    // names remain tied to the generated CSSProperty table and are translated
    // together with that table.
    pub fn CustomPropertyName(&self) -> Option<&AtomicString> {
        self.IsCustomProperty()
            .then_some(&self.custom_property_name_)
    }

    // cpp: css_property_name.cc:43-48
    pub fn GetHash(&self) -> u32 {
        if self.IsCustomProperty() {
            self.custom_property_name_.Hash()
        } else {
            self.value_ as u32
        }
    }
}

// cpp: css_property_name.cc:25-34
impl PartialEq for CSSPropertyName {
    fn eq(&self, other: &Self) -> bool {
        if self.value_ != other.value_ {
            return false;
        }
        if self.value_ != CSSPropertyID::kVariable as i32 {
            return true;
        }
        self.custom_property_name_ == other.custom_property_name_
    }
}

impl Eq for CSSPropertyName {}

impl Hash for CSSPropertyName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.GetHash());
    }
}

// cpp: third_party/blink/renderer/core/css/css_property_name_test.cc:27-38,
//      48-57,125-160
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn IdStandardAndCustomProperty() {
        let native = CSSPropertyName::new(CSSPropertyID::kFontSize);
        assert_eq!(native.Id(), CSSPropertyID::kFontSize);

        let custom = CSSPropertyName::custom(AtomicString::from_str("--x"));
        assert_eq!(custom.Id(), CSSPropertyID::kVariable);
        assert!(custom.IsCustomProperty());

        let alias = CSSPropertyName::new(CSSPropertyID::kAliasWordWrap);
        assert_eq!(alias.Id(), CSSPropertyID::kAliasWordWrap);
        assert_eq!(alias.ToAtomicString().Utf8(), "word-wrap");
    }

    #[test]
    fn OperatorEquals() {
        assert_eq!(
            CSSPropertyName::custom(AtomicString::from_str("--x")),
            CSSPropertyName::custom(AtomicString::from_str("--x"))
        );
        assert_eq!(
            CSSPropertyName::new(CSSPropertyID::kColor),
            CSSPropertyName::new(CSSPropertyID::kColor)
        );
        assert_ne!(
            CSSPropertyName::custom(AtomicString::from_str("--x")),
            CSSPropertyName::custom(AtomicString::from_str("--y"))
        );
        assert_ne!(
            CSSPropertyName::new(CSSPropertyID::kColor),
            CSSPropertyName::new(CSSPropertyID::kBackgroundColor)
        );
    }

    #[test]
    fn HashMapBasic() {
        let mut map = HashMap::new();
        map.insert(
            CSSPropertyName::custom(AtomicString::from_str("--x")),
            AtomicString::from_str("foo"),
        );
        map.insert(
            CSSPropertyName::custom(AtomicString::from_str("--y")),
            AtomicString::from_str("foo"),
        );
        map.insert(
            CSSPropertyName::custom(AtomicString::from_str("--x")),
            AtomicString::from_str("bar"),
        );

        assert_eq!(
            map.remove(&CSSPropertyName::custom(AtomicString::from_str("--x"))),
            Some(AtomicString::from_str("bar"))
        );
        assert_eq!(
            map.remove(&CSSPropertyName::custom(AtomicString::from_str("--y"))),
            Some(AtomicString::from_str("foo"))
        );

        map.insert(
            CSSPropertyName::new(CSSPropertyID::kFontSize),
            AtomicString::from_str("foo"),
        );
        map.insert(
            CSSPropertyName::new(CSSPropertyID::kFontSize),
            AtomicString::from_str("bar"),
        );
        assert_eq!(
            map.remove(&CSSPropertyName::new(CSSPropertyID::kFontSize)),
            Some(AtomicString::from_str("bar"))
        );
    }
}
