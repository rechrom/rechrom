// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_counter_value.h:41-83; css_counter_value.cc:14-33.
use super::*;

pub struct CSSCounterValue {
    pub identifier: Rc<Value>,
    pub value: Option<Rc<Value>>,
    pub is_reversed: bool,
}
impl CSSValueSubclass for CSSCounterValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from(if self.is_reversed { "reversed(" } else { "" });
        text.push_string(&self.identifier.CssText());
        if self.is_reversed {
            text.push_str(")");
        }
        if let Some(value) = &self.value {
            text.push_str(" ");
            text.push_string(&value.CssText());
        }
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identifier, &other.identifier)
            && match (&self.value, &other.value) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && self.is_reversed == other.is_reversed
    }
}
impl CSSValueRandom for CSSCounterValue {
    fn HasRandomFunctions(&self) -> bool {
        self.identifier.HasRandomFunctions()
            || self
                .value
                .as_ref()
                .is_some_and(|value| value.HasRandomFunctions())
    }
}
pub fn counter(identifier: Rc<Value>, value: Option<Rc<Value>>, is_reversed: bool) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kCounterClass(
        CSSCounterValue {
            identifier,
            value,
            is_reversed,
        },
    )))
}
