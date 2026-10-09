// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_dynamic_range_limit_mix_value.h:17-44 / .cc:15-73.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::String;
use std::rc::Rc;
pub struct CSSDynamicRangeLimitMixValue {
    pub limits: Vec<Rc<Value>>,
    pub percentages: Vec<Rc<Value>>,
}
impl CSSDynamicRangeLimitMixValue {
    pub fn new(limits: Vec<Rc<Value>>, percentages: Vec<Rc<Value>>) -> Self {
        assert_eq!(limits.len(), percentages.len());
        Self {
            limits,
            percentages,
        }
    }
}
impl CSSValueSubclass for CSSDynamicRangeLimitMixValue {
    fn CustomCSSText(&self) -> String {
        String::from(format!(
            "dynamic-range-limit-mix({})",
            self.limits
                .iter()
                .zip(&self.percentages)
                .map(|(l, p)| format!("{} {}", l.CssText().Utf8(), p.CssText().Utf8()))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
    fn Equals(&self, other: &Self) -> bool {
        self.limits == other.limits && self.percentages == other.percentages
    }
}
impl CSSValueRandom for CSSDynamicRangeLimitMixValue {
    fn HasRandomFunctions(&self) -> bool {
        self.limits
            .iter()
            .chain(&self.percentages)
            .any(|v| v.HasRandomFunctions())
    }
}
