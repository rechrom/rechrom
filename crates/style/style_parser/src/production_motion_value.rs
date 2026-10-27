// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! core/css/css_ray_value.h:26-66 / .cc:17-77.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValuePayload, CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::{CSSValueID, String};
use std::rc::Rc;
pub struct CSSRayValue {
    pub angle: Rc<Value>,
    pub size: Rc<Value>,
    pub contain: Option<Rc<Value>>,
    pub center: Option<(Rc<Value>, Rc<Value>)>,
}
impl CSSValueSubclass for CSSRayValue {
    fn CustomCSSText(&self) -> String {
        let mut text = format!("ray({}", self.angle.CssText().Utf8());
        if !matches!(self.size.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kClosestSide)
        {
            text.push(' ');
            text.push_str(&self.size.CssText().Utf8());
        }
        if let Some(c) = &self.contain {
            text.push(' ');
            text.push_str(&c.CssText().Utf8());
        }
        if let Some((x, y)) = &self.center {
            text.push_str(&format!(
                " at {} {}",
                x.CssText().Utf8(),
                y.CssText().Utf8()
            ));
        }
        text.push(')');
        String::from(text)
    }
    fn Equals(&self, other: &Self) -> bool {
        self.angle == other.angle
            && self.size == other.size
            && self.contain == other.contain
            && self.center == other.center
    }
}
impl CSSValueRandom for CSSRayValue {
    fn HasRandomFunctions(&self) -> bool {
        self.angle.HasRandomFunctions()
            || self
                .center
                .as_ref()
                .is_some_and(|(x, y)| x.HasRandomFunctions() || y.HasRandomFunctions())
    }
}
