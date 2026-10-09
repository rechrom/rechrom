// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Native typed CSSQuadValue and CSSBorderImageSliceValue payloads.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValuePayload, CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::String;
use std::rc::Rc;

// css_quad_value.h:34-83; .cc:13-53. Border-image consumers use quad
// serialization; the rect form is retained for the actual source subclass.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TypeForSerialization {
    kSerializeAsRect,
    kSerializeAsQuad,
}
pub struct CSSQuadValue {
    pub sides: [Rc<Value>; 4], // top, right, bottom, left
    pub serialization_type: TypeForSerialization,
}
impl CSSValueSubclass for CSSQuadValue {
    fn CustomCSSText(&self) -> String {
        let [top, right, bottom, left] = self.sides.each_ref().map(|v| v.CssText());
        if self.serialization_type == TypeForSerialization::kSerializeAsRect {
            return String::from(format!(
                "rect({}, {}, {}, {})",
                top.Utf8(),
                right.Utf8(),
                bottom.Utf8(),
                left.Utf8()
            ));
        }
        let mut result = top.clone();
        if right != top || bottom != top || left != top {
            result.push_str(" ");
            result.push_string(&right);
            if bottom != top || right != left {
                result.push_str(" ");
                result.push_string(&bottom);
                if left != right {
                    result.push_str(" ");
                    result.push_string(&left);
                }
            }
        }
        result
    }
    fn Equals(&self, other: &Self) -> bool {
        self.sides
            .iter()
            .zip(&other.sides)
            .all(|(a, b)| a.as_ref() == b.as_ref())
    }
}
impl CSSValueRandom for CSSQuadValue {
    fn HasRandomFunctions(&self) -> bool {
        self.sides.iter().any(|v| v.HasRandomFunctions())
    }
}
pub fn quad(sides: [Rc<Value>; 4]) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kQuadClass(CSSQuadValue {
        sides,
        serialization_type: TypeForSerialization::kSerializeAsQuad,
    })))
}
// css_border_image_slice_value.h:35-62; .cc:41-65.
pub struct CSSBorderImageSliceValue {
    pub slices: Rc<Value>,
    pub fill: bool,
}
impl CSSValueSubclass for CSSBorderImageSliceValue {
    fn CustomCSSText(&self) -> String {
        let mut text = self.slices.CssText();
        if self.fill {
            text.push_str(" fill");
        }
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.fill == other.fill && self.slices.as_ref() == other.slices.as_ref()
    }
}
impl CSSValueRandom for CSSBorderImageSliceValue {
    fn HasRandomFunctions(&self) -> bool {
        self.slices.HasRandomFunctions()
    }
}
pub fn slice(slices: Rc<Value>, fill: bool) -> Rc<Value> {
    assert!(matches!(slices.Payload(), CSSValuePayload::kQuadClass(_)));
    Rc::new(Value::new(CSSValuePayload::kBorderImageSliceClass(
        CSSBorderImageSliceValue { slices, fill },
    )))
}
