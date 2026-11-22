// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_font_style_range_value.{h,cc}: typed oblique angle for element style.
use crate::{
    css_value::{CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::String;
use std::rc::Rc;
pub struct CSSFontStyleRangeValue {
    pub angle: Option<Rc<Value>>,
}
impl CSSValueSubclass for CSSFontStyleRangeValue {
    fn CustomCSSText(&self) -> String {
        String::from(self.angle.as_ref().map_or_else(
            || "oblique".to_owned(),
            |a| format!("oblique {}", a.CssText().Utf8()),
        ))
    }
    fn Equals(&self, other: &Self) -> bool {
        self.angle == other.angle
    }
}
impl CSSValueRandom for CSSFontStyleRangeValue {
    fn HasRandomFunctions(&self) -> bool {
        self.angle.as_ref().is_some_and(|a| a.HasRandomFunctions())
    }
}
