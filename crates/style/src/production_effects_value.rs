// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium css_basic_shape_values.h polygon subtype; original value ownership.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValuePayload, CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::{String, WindRule};
use std::rc::Rc;
// cpp: css_basic_shape_values.h:108-156; .cc:237-327.
pub struct CSSBasicShapePolygonValue {
    pub wind_rule: WindRule,
    pub rounding_radius: Option<Rc<Value>>,
    pub coordinates: Vec<Rc<Value>>,
}
impl CSSValueSubclass for CSSBasicShapePolygonValue {
    fn CustomCSSText(&self) -> String {
        let mut prefix = Vec::new();
        if self.wind_rule == WindRule::RULE_EVENODD {
            prefix.push("evenodd".to_owned());
        }
        if let Some(radius) = self.rounding_radius.as_ref().filter(|radius| {
            !matches!(radius.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() == 0.0)
        }) {
            prefix.push(format!("round {}", radius.CssText().Utf8()));
        }
        let points = self
            .coordinates
            .chunks_exact(2)
            .map(|point| {
                format!(
                    "{} {}",
                    point[0].CssText().Utf8(),
                    point[1].CssText().Utf8()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        String::from(
            format!(
                "polygon({}{points})",
                if prefix.is_empty() {
                    std::string::String::new()
                } else {
                    format!("{}, ", prefix.join(" "))
                }
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.wind_rule == other.wind_rule
            && self.rounding_radius == other.rounding_radius
            && self.coordinates == other.coordinates
    }
}
impl CSSValueRandom for CSSBasicShapePolygonValue {
    fn HasRandomFunctions(&self) -> bool {
        self.rounding_radius
            .as_ref()
            .is_some_and(|v| v.HasRandomFunctions())
            || self.coordinates.iter().any(|v| v.HasRandomFunctions())
    }
}
