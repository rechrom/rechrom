// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_view_transition.h/.cc
// GC tracing/allocation and downcast boilerplate are represented by Rc/traits.

#![allow(non_snake_case, non_camel_case_types)]

use crate::css_property_names::CSSPropertyID;
use foundation::{CSSValueID, String};
use std::rc::Rc;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewTransitionNavigationType {
    kAuto,
    kNone,
    kPreview,
    kUnspecified,
}

// Required operations supplied by the actual CSSValue derived classes and
// CSSPropertyValueSet. There is no fallback list or identifier coercion.
pub trait StyleRuleViewTransitionBackend: Sized {
    type PropertySet;
    type CSSValue;
    fn GetPropertyCSSValue(
        properties: &Self::PropertySet,
        id: CSSPropertyID,
    ) -> Option<Rc<Self::CSSValue>>;
    fn IsIdentifierValue(value: &Self::CSSValue) -> bool;
    fn IdentifierValueID(value: &Self::CSSValue) -> CSSValueID;
    fn ValueList(value: &Self::CSSValue) -> Option<Vec<Rc<Self::CSSValue>>>;
    fn CustomIdentValue(value: &Self::CSSValue) -> String;
}

// cpp: style_rule_view_transition.h:13-42
#[derive(Clone)]
pub struct StyleRuleViewTransition<D: StyleRuleViewTransitionBackend> {
    navigation_: Option<Rc<D::CSSValue>>,
    types_value_: Option<Rc<D::CSSValue>>,
    types_: Vec<String>,
}
impl<D: StyleRuleViewTransitionBackend> StyleRuleViewTransition<D> {
    // cpp: style_rule_view_transition.cc:17-33,35-41
    pub fn new(properties: &D::PropertySet) -> Self {
        let navigation = D::GetPropertyCSSValue(properties, CSSPropertyID::kNavigation);
        let types_value = D::GetPropertyCSSValue(properties, CSSPropertyID::kTypes);
        let types = types_value
            .as_deref()
            .and_then(D::ValueList)
            .unwrap_or_default()
            .iter()
            .map(|value| D::CustomIdentValue(value))
            .collect();
        Self {
            navigation_: navigation,
            types_value_: types_value,
            types_: types,
        }
    }

    // cpp: style_rule_view_transition.cc:49-69
    pub fn GetNavigation(&self) -> ViewTransitionNavigationType {
        let Some(value) = self.navigation_.as_deref() else {
            return ViewTransitionNavigationType::kUnspecified;
        };
        if D::IsIdentifierValue(value) {
            return match D::IdentifierValueID(value) {
                CSSValueID::kNone => ViewTransitionNavigationType::kNone,
                CSSValueID::kAuto => ViewTransitionNavigationType::kAuto,
                CSSValueID::kPreview => ViewTransitionNavigationType::kPreview,
                _ => panic!("unexpected @view-transition navigation identifier"),
            };
        }
        ViewTransitionNavigationType::kAuto
    }
    pub fn GetNavigationValue(&self) -> Option<&Rc<D::CSSValue>> {
        self.navigation_.as_ref()
    }
    pub fn GetTypes(&self) -> &[String] {
        &self.types_
    }
    pub fn GetTypesValue(&self) -> Option<&Rc<D::CSSValue>> {
        self.types_value_.as_ref()
    }
    pub fn Copy(&self) -> Self {
        Self {
            navigation_: self.navigation_.clone(),
            types_value_: self.types_value_.clone(),
            types_: self.types_.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    enum Value {
        Ident(CSSValueID),
        Custom(&'static str),
        List(Vec<Rc<Value>>),
    }
    struct Backend;
    impl StyleRuleViewTransitionBackend for Backend {
        type PropertySet = HashMap<CSSPropertyID, Rc<Value>>;
        type CSSValue = Value;
        fn GetPropertyCSSValue(
            properties: &Self::PropertySet,
            id: CSSPropertyID,
        ) -> Option<Rc<Self::CSSValue>> {
            properties.get(&id).cloned()
        }
        fn IsIdentifierValue(value: &Value) -> bool {
            matches!(value, Value::Ident(_))
        }
        fn IdentifierValueID(value: &Value) -> CSSValueID {
            match value {
                Value::Ident(id) => *id,
                _ => unreachable!(),
            }
        }
        fn ValueList(value: &Value) -> Option<Vec<Rc<Value>>> {
            match value {
                Value::List(values) => Some(values.clone()),
                _ => None,
            }
        }
        fn CustomIdentValue(value: &Value) -> String {
            match value {
                Value::Custom(value) => String::from(*value),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn extracts_navigation_and_custom_ident_type_list() {
        let properties = HashMap::from([
            (
                CSSPropertyID::kNavigation,
                Rc::new(Value::Ident(CSSValueID::kPreview)),
            ),
            (
                CSSPropertyID::kTypes,
                Rc::new(Value::List(vec![
                    Rc::new(Value::Custom("forward")),
                    Rc::new(Value::Custom("reload")),
                ])),
            ),
        ]);
        let rule = StyleRuleViewTransition::<Backend>::new(&properties);
        assert_eq!(rule.GetNavigation(), ViewTransitionNavigationType::kPreview);
        assert_eq!(
            rule.GetTypes().iter().map(String::Utf8).collect::<Vec<_>>(),
            vec!["forward", "reload"]
        );
        assert!(rule.GetTypesValue().is_some());
    }
}
