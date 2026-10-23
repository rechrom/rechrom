// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium css_basic_shape_values.h polygon subtype; original value ownership.
#![allow(non_snake_case)]
use crate::{
    css_value::{CSSValueCustomHash, CSSValuePayload, CSSValueRandom, CSSValueSubclass},
    production_css_value::Value,
};
use foundation::{String, WindRule};
use layoutng_style::style::svg_path_byte_stream::SVGPathByteStream;
use std::rc::Rc;

pub struct CSSBasicShapeCircleValue {
    pub center_x: Option<Rc<Value>>,
    pub center_y: Option<Rc<Value>>,
    pub radius: Option<Rc<Value>>,
}

pub struct CSSBasicShapeEllipseValue {
    pub center_x: Option<Rc<Value>>,
    pub center_y: Option<Rc<Value>>,
    pub radius_x: Option<Rc<Value>>,
    pub radius_y: Option<Rc<Value>>,
}

pub struct CSSPathValue {
    pub byte_stream: SVGPathByteStream,
    pub wind_rule: WindRule,
    pub path_text: String,
}

fn CenterAndRadiusText(
    name: &str,
    radii: &[Option<&Rc<Value>>],
    center_x: Option<&Rc<Value>>,
    center_y: Option<&Rc<Value>>,
) -> String {
    let mut items = radii
        .iter()
        .flatten()
        .map(|value| value.CssText().Utf8())
        .collect::<Vec<_>>();
    if let (Some(x), Some(y)) = (center_x, center_y) {
        items.push("at".to_owned());
        items.push(x.CssText().Utf8());
        items.push(y.CssText().Utf8());
    }
    String::from(format!("{name}({})", items.join(" ")).as_str())
}

impl CSSValueSubclass for CSSBasicShapeCircleValue {
    fn CustomCSSText(&self) -> String {
        CenterAndRadiusText(
            "circle",
            &[self.radius.as_ref()],
            self.center_x.as_ref(),
            self.center_y.as_ref(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.center_x == other.center_x
            && self.center_y == other.center_y
            && self.radius == other.radius
    }
}
impl CSSValueRandom for CSSBasicShapeCircleValue {
    fn HasRandomFunctions(&self) -> bool {
        [&self.center_x, &self.center_y, &self.radius]
            .into_iter()
            .flatten()
            .any(|value| value.HasRandomFunctions())
    }
}

impl CSSValueSubclass for CSSBasicShapeEllipseValue {
    fn CustomCSSText(&self) -> String {
        CenterAndRadiusText(
            "ellipse",
            &[self.radius_x.as_ref(), self.radius_y.as_ref()],
            self.center_x.as_ref(),
            self.center_y.as_ref(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.center_x == other.center_x
            && self.center_y == other.center_y
            && self.radius_x == other.radius_x
            && self.radius_y == other.radius_y
    }
}
impl CSSValueRandom for CSSBasicShapeEllipseValue {
    fn HasRandomFunctions(&self) -> bool {
        [
            &self.center_x,
            &self.center_y,
            &self.radius_x,
            &self.radius_y,
        ]
        .into_iter()
        .flatten()
        .any(|value| value.HasRandomFunctions())
    }
}

impl CSSValueSubclass for CSSPathValue {
    fn CustomCSSText(&self) -> String {
        let prefix = if self.wind_rule == WindRule::RULE_EVENODD {
            "evenodd, "
        } else {
            ""
        };
        let serialized = crate::css_markup::SerializeString(&self.path_text);
        String::from(format!("path({prefix}{})", serialized.Utf8()).as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self.wind_rule == other.wind_rule && self.byte_stream == other.byte_stream
    }
}
impl CSSValueCustomHash for CSSPathValue {
    fn CustomHash(&self) -> u32 {
        // Same semantic inputs as Equals. A stable byte hash avoids making the
        // CSS value's cache key depend on the author string spelling.
        let mut hash = if self.wind_rule == WindRule::RULE_EVENODD {
            2166136261u32 ^ 1
        } else {
            2166136261u32
        };
        for byte in self.byte_stream.Span() {
            hash = (hash ^ u32::from(*byte)).wrapping_mul(16777619);
        }
        hash
    }
}
// cpp: css_basic_shape_values.h:108-156; .cc:237-327.
pub struct CSSBasicShapePolygonValue {
    pub wind_rule: WindRule,
    pub rounding_radius: Option<Rc<Value>>,
    pub coordinates: Vec<Rc<Value>>,
}

/// css_basic_shape_values.h:158-220.  The four inset sides and optional
/// corner radii stay typed until the resolver converts them to
/// `BasicShapeInset`; no paint geometry is manufactured in the parser.
pub struct CSSBasicShapeInsetValue {
    pub sides: [Rc<Value>; 4],              // top, right, bottom, left
    pub radii: Option<[[Rc<Value>; 2]; 4]>, // tl, tr, br, bl; horizontal, vertical
}

/// css_basic_shape_values.h:219-281. `rect()` keeps edge coordinates until
/// computed-value conversion, where Chromium lowers it to the equivalent
/// `inset(top calc(100% - right) calc(100% - bottom) left)` shape.
pub struct CSSBasicShapeRectValue {
    pub sides: [Rc<Value>; 4],              // top, right, bottom, left
    pub radii: Option<[[Rc<Value>; 2]; 4]>, // tl, tr, br, bl; horizontal, vertical
}

impl CSSValueSubclass for CSSBasicShapeInsetValue {
    fn CustomCSSText(&self) -> String {
        fn four(values: &[Rc<Value>; 4]) -> Vec<&Rc<Value>> {
            let mut count = 4;
            if values[3].as_ref() == values[1].as_ref() {
                count = 3;
                if values[2].as_ref() == values[0].as_ref() {
                    count = 2;
                    if values[1].as_ref() == values[0].as_ref() {
                        count = 1;
                    }
                }
            }
            values[..count].iter().collect()
        }
        let mut text = format!(
            "inset({}",
            four(&self.sides)
                .into_iter()
                .map(|value| value.CssText().Utf8())
                .collect::<Vec<_>>()
                .join(" ")
        );
        if let Some(radii) = &self.radii {
            let horizontal = radii.each_ref().map(|pair| pair[0].clone());
            let vertical = radii.each_ref().map(|pair| pair[1].clone());
            text.push_str(" round ");
            text.push_str(
                &four(&horizontal)
                    .into_iter()
                    .map(|value| value.CssText().Utf8())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            if horizontal
                .iter()
                .zip(&vertical)
                .any(|(x, y)| x.as_ref() != y.as_ref())
            {
                text.push_str(" / ");
                text.push_str(
                    &four(&vertical)
                        .into_iter()
                        .map(|value| value.CssText().Utf8())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        text.push(')');
        String::from(text.as_str())
    }

    fn Equals(&self, other: &Self) -> bool {
        self.sides == other.sides && self.radii == other.radii
    }
}

impl CSSValueRandom for CSSBasicShapeInsetValue {
    fn HasRandomFunctions(&self) -> bool {
        self.sides.iter().any(|value| value.HasRandomFunctions())
            || self.radii.as_ref().is_some_and(|radii| {
                radii
                    .iter()
                    .flatten()
                    .any(|value| value.HasRandomFunctions())
            })
    }
}

impl CSSValueSubclass for CSSBasicShapeRectValue {
    fn CustomCSSText(&self) -> String {
        let mut text = format!(
            "rect({})",
            self.sides
                .iter()
                .map(|value| value.CssText().Utf8())
                .collect::<Vec<_>>()
                .join(" ")
        );
        if let Some(radii) = &self.radii {
            let horizontal = radii
                .iter()
                .map(|pair| pair[0].CssText().Utf8())
                .collect::<Vec<_>>()
                .join(" ");
            let vertical = radii
                .iter()
                .map(|pair| pair[1].CssText().Utf8())
                .collect::<Vec<_>>()
                .join(" ");
            text.pop();
            text.push_str(" round ");
            text.push_str(&horizontal);
            if radii
                .iter()
                .any(|pair| pair[0].as_ref() != pair[1].as_ref())
            {
                text.push_str(" / ");
                text.push_str(&vertical);
            }
            text.push(')');
        }
        String::from(text.as_str())
    }

    fn Equals(&self, other: &Self) -> bool {
        self.sides == other.sides && self.radii == other.radii
    }
}

impl CSSValueRandom for CSSBasicShapeRectValue {
    fn HasRandomFunctions(&self) -> bool {
        self.sides.iter().any(|value| value.HasRandomFunctions())
            || self.radii.as_ref().is_some_and(|radii| {
                radii
                    .iter()
                    .flatten()
                    .any(|value| value.HasRandomFunctions())
            })
    }
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
