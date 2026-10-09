// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_reflect_value.h:40-65 and .cc:34-51 typed reflection payload.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValuePayload, CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::String;
use std::rc::Rc;
pub struct CSSReflectValue {
    pub direction: Rc<Value>,
    pub offset: Rc<Value>,
    pub mask: Option<Rc<Value>>,
}
impl CSSValueSubclass for CSSReflectValue {
    fn CustomCSSText(&self) -> String {
        let text = format!(
            "{} {}",
            self.direction.CssText().Utf8(),
            self.offset.CssText().Utf8()
        );
        String::from(
            if let Some(mask) = &self.mask {
                format!("{text} {}", mask.CssText().Utf8())
            } else {
                text
            }
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.direction, &other.direction)
            && self.offset == other.offset
            && self.mask == other.mask
    }
}
impl CSSValueRandom for CSSReflectValue {
    fn HasRandomFunctions(&self) -> bool {
        self.offset.HasRandomFunctions()
            || self.mask.as_ref().is_some_and(|m| m.HasRandomFunctions())
    }
}
pub fn reflect(direction: Rc<Value>, offset: Rc<Value>, mask: Option<Rc<Value>>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kReflectClass(
        CSSReflectValue {
            direction,
            offset,
            mask,
        },
    )))
}
