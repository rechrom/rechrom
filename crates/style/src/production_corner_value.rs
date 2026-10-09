// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! CSSSuperellipseValue: css_superellipse_value.h:18-38 / .cc:16-31.
use crate::{css_value::{CSSValueCustomHash, CSSValueRandom, CSSValueSubclass}, production_css_value::Value};
use foundation::String;
use std::rc::Rc;

pub struct CSSSuperellipseValue { param: Rc<Value> }
#[allow(non_snake_case)]
impl CSSSuperellipseValue {
    pub fn new(param: Rc<Value>) -> Self { Self { param } }
    pub fn Param(&self) -> &Rc<Value> { &self.param }
}
impl CSSValueSubclass for CSSSuperellipseValue {
    fn CustomCSSText(&self) -> String { String::from(format!("superellipse({})", self.param.CssText().Utf8())) }
    fn Equals(&self, other: &Self) -> bool { self.param == other.param }
}
impl CSSValueCustomHash for CSSSuperellipseValue {
    fn CustomHash(&self) -> u32 { self.param.Hash() }
}
impl CSSValueRandom for CSSSuperellipseValue {
    fn HasRandomFunctions(&self) -> bool { self.param.HasRandomFunctions() }
}
