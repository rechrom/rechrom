// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Concrete CSSValue payloads for the production property parser.
//! Unparsed declarations retain token identity until computed-value substitution.
//! Unavailable derived classes remain uninhabited and report typed Unsupported.
#![allow(non_snake_case)]
use crate::css_numeric_literal_value::CSSNumericLiteralValue;
use crate::css_primitive_value::UnitType;
use crate::css_property_value::CSSPropertyValueBackend;
use crate::css_value::*;
use crate::css_value_keywords::GetCSSValueName;
use foundation::{
    AddFloatToHash, AddIntToHash, CSSPropertyID, CSSValueID, Color, HashInt, HashInts, Length,
    LengthType, String,
};
use std::rc::Rc;

#[path = "production_counter_value.rs"]
mod counter_value;
pub use counter_value::{counter, CSSCounterValue};
#[path = "production_scoped_keyword_value.rs"]
mod scoped_keyword_value;
pub use scoped_keyword_value::{scoped_keyword, CSSScopedKeywordValue};

#[path = "production_font_style_value.rs"]
mod font_style_value;
pub use font_style_value::CSSFontStyleRangeValue;

pub struct ProductionCSSValueDispatch;
pub type Value = CSSValue<ProductionCSSValueDispatch>;
pub type PropertyValue = crate::css_property_value::CSSPropertyValue<ProductionCSSValueDispatch>;

// cpp: css_identifier_value.h:43-57; css_identifier_value.cc:30-42
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSIdentifierValue(pub CSSValueID);
impl CSSValueSubclass for CSSIdentifierValue {
    fn CustomCSSText(&self) -> String {
        String::from(GetCSSValueName(self.0))
    }
    fn Equals(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl CSSValueCustomHash for CSSIdentifierValue {
    fn CustomHash(&self) -> u32 {
        self.0 as u32
    }
}
// cpp: css_numeric_literal_value.cc:222-323,397-412
impl CSSValueSubclass for CSSNumericLiteralValue {
    fn CustomCSSText(&self) -> String {
        crate::media_queries::media_query_exp::MediaQueryExpSerialization::CssText(self)
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
impl CSSValueCustomHash for CSSNumericLiteralValue {
    fn CustomHash(&self) -> u32 {
        let bits = self.DoubleValue().to_bits();
        HashInts(
            self.GetType() as u32,
            HashInts((bits >> 32) as u32, bits as u32),
        )
    }
}
// cpp: css_color.h:25-38; platform/graphics/color.cc:891-902,1022-1109.
// Production color consumer currently constructs legacy sRGB only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CSSColor(pub Color);
impl CSSValueSubclass for CSSColor {
    fn CustomCSSText(&self) -> String {
        let r = (self.0.Param0()).round() as u8;
        let g = (self.0.Param1()).round() as u8;
        let b = (self.0.Param2()).round() as u8;
        let text = if self.0.IsOpaque() {
            format!("rgb({r}, {g}, {b})")
        } else {
            format!("rgba({r}, {g}, {b}, {})", self.0.Alpha())
        };
        String::from(text.as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl CSSValueCustomHash for CSSColor {
    fn CustomHash(&self) -> u32 {
        let mut hash = HashInt(self.0.GetColorSpace() as u32);
        for value in [
            self.0.Param0(),
            self.0.Param1(),
            self.0.Param2(),
            self.0.Alpha(),
        ] {
            AddFloatToHash(&mut hash, value);
        }
        for none in [
            self.0.Param0IsNone(),
            self.0.Param1IsNone(),
            self.0.Param2IsNone(),
            self.0.AlphaIsNone(),
        ] {
            AddIntToHash(&mut hash, none as u32);
        }
        hash
    }
}
// cpp: css_value_pair.h:33-65; css_value_pair.cc:26-71
pub struct CSSValuePair {
    pub first: Rc<Value>,
    pub second: Rc<Value>,
    pub drop_identical: bool,
}
// cpp: css_ratio_value.h:23-51; css_ratio_value.cc:14-36.
pub struct CSSRatioValue {
    pub first: Rc<Value>,
    pub second: Rc<Value>,
}
impl CSSValueSubclass for CSSRatioValue {
    fn CustomCSSText(&self) -> String {
        let mut text = self.first.CssText();
        text.push_str(" / ");
        text.push_str(&self.second.CssText().Utf8());
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.first.as_ref() == other.first.as_ref() && self.second.as_ref() == other.second.as_ref()
    }
}
impl CSSValueRandom for CSSRatioValue {
    fn HasRandomFunctions(&self) -> bool {
        self.first.HasRandomFunctions() || self.second.HasRandomFunctions()
    }
}
pub fn ratio(first: Rc<Value>, second: Rc<Value>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kRatioClass(CSSRatioValue {
        first,
        second,
    })))
}
impl CSSValueSubclass for CSSValuePair {
    fn CustomCSSText(&self) -> String {
        let mut text = self.first.CssText();
        if !self.drop_identical || self.first.as_ref() != self.second.as_ref() {
            text.push_str(" ");
            text.push_str(&self.second.CssText().Utf8());
        }
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.first.as_ref() == other.first.as_ref() && self.second.as_ref() == other.second.as_ref()
    }
}
impl CSSValueCustomHash for CSSValuePair {
    fn CustomHash(&self) -> u32 {
        HashInts(self.first.Hash(), self.second.Hash())
    }
}
impl CSSValueRandom for CSSValuePair {
    fn HasRandomFunctions(&self) -> bool {
        self.first.HasRandomFunctions() || self.second.HasRandomFunctions()
    }
}
// cpp: css_content_distribution_value.h:18-47; .cc:14-69.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSContentDistributionValue {
    pub distribution: CSSValueID,
    pub position: CSSValueID,
    pub overflow: CSSValueID,
}
impl CSSValueSubclass for CSSContentDistributionValue {
    fn CustomCSSText(&self) -> String {
        let mut items = Vec::new();
        if self.distribution != CSSValueID::kInvalid {
            items.push(identifier(self.distribution));
        }
        if self.position != CSSValueID::kInvalid {
            if matches!(
                self.position,
                CSSValueID::kFirstBaseline | CSSValueID::kLastBaseline
            ) {
                items.push(identifier(if self.position == CSSValueID::kFirstBaseline {
                    CSSValueID::kFirst
                } else {
                    CSSValueID::kLast
                }));
                items.push(identifier(CSSValueID::kBaseline));
            } else {
                if self.overflow != CSSValueID::kInvalid {
                    items.push(identifier(self.overflow));
                }
                items.push(identifier(self.position));
            }
        }
        list(items, ListSeparator::Space).CssText()
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
impl CSSValueCustomHash for CSSContentDistributionValue {
    fn CustomHash(&self) -> u32 {
        HashInts(
            self.distribution as u32,
            HashInts(self.position as u32, self.overflow as u32),
        )
    }
}
// cpp: css_custom_ident_value.h:21-78; .cc:20-30,85-106,118-126.
// Parsing constructs literal names/property IDs. TreeScope/ident() bindings are
// unavailable in this dispatch and are explicit collaborators, never fabricated.
#[path = "production_grid_values.rs"]
mod grid_values;
pub use crate::production_border_image_values::{CSSBorderImageSliceValue, CSSQuadValue};
pub use grid_values::CSSGridTemplateAreasValue;

pub struct CSSCustomIdentValue {
    pub name: foundation::AtomicString,
    pub property: CSSPropertyID,
}
impl CSSValueSubclass for CSSCustomIdentValue {
    fn CustomCSSText(&self) -> String {
        if self.property != CSSPropertyID::kInvalid {
            String::from(crate::css_property_names::GetPropertyName(self.property))
        } else {
            crate::css_markup::SerializeIdentifier(
                &String::from_utf16(self.name.utf16_units().unwrap_or_default()),
                false,
            )
        }
    }
    fn Equals(&self, other: &Self) -> bool {
        self.property == other.property && self.name == other.name
    }
}
impl CSSValueCustomHash for CSSCustomIdentValue {
    fn CustomHash(&self) -> u32 {
        if self.property != CSSPropertyID::kInvalid {
            HashInt(self.property as u32)
        } else {
            self.name.Hash()
        }
    }
}
impl CSSValueRandom for CSSCustomIdentValue {
    fn HasRandomFunctions(&self) -> bool {
        false
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSCustomIdentValue {
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        panic!("CSSCustomIdentValue::PopulateWithTreeScope requires application TreeScope binding")
    }
}
pub fn custom_ident(name: &String, property: CSSPropertyID) -> Rc<Value> {
    let mut value = Value::new(CSSValuePayload::kCustomIdentClass(CSSCustomIdentValue {
        name: if property == CSSPropertyID::kInvalid {
            foundation::AtomicString::from_utf16(name.Span16().unwrap_or_default())
        } else {
            foundation::AtomicString::default()
        },
        property,
    }));
    value
        .StateMut()
        .SetNeedsTreeScopePopulation(property == CSSPropertyID::kInvalid);
    Rc::new(value)
}
// cpp: css_counter_content_value.h:42-119; .cc:14-70.
// The document-root production path has no bound TreeScope. Keep the names
// typed and the scope-population dependency visible until that service exists.
pub struct CSSCounterContentValue {
    pub identifier: Rc<Value>,
    pub list_style: Rc<Value>,
    pub separator: String,
}
impl CSSValueSubclass for CSSCounterContentValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from(if self.separator.length() == 0 {
            "counter("
        } else {
            "counters("
        });
        text.push_string(&self.identifier.CssText());
        if self.separator.length() != 0 {
            text.push_str(", ");
            text.push_string(&crate::css_markup::SerializeString(&self.separator));
        }
        let CSSValuePayload::kCustomIdentClass(style) = self.list_style.Payload() else {
            unreachable!("symbols() requires anonymous CounterStyle binding")
        };
        if style.name != "decimal" {
            text.push_str(", ");
            text.push_string(&self.list_style.CssText());
        }
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.identifier == other.identifier
            && self.list_style == other.list_style
            && self.separator == other.separator
    }
}
impl CSSValueRandom for CSSCounterContentValue {
    fn HasRandomFunctions(&self) -> bool {
        self.identifier.HasRandomFunctions() || self.list_style.HasRandomFunctions()
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSCounterContentValue {
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        panic!(
            "CSSCounterContentValue::PopulateWithTreeScope requires application TreeScope binding"
        )
    }
}
pub fn counter_content(
    identifier: Rc<Value>,
    list_style: Rc<Value>,
    separator: String,
) -> Rc<Value> {
    let mut value = Value::new(CSSValuePayload::kCounterContentClass(
        CSSCounterContentValue {
            identifier,
            list_style,
            separator,
        },
    ));
    value.StateMut().SetNeedsTreeScopePopulation(true);
    Rc::new(value)
}
// cpp: css_image_value.h:39-107; .cc:49-50,159-175,196-201.
// This is the un-fetched URL image value. A Document/resource service must
// bind URL resolution, referrer/request modifiers and a cached StyleImage;
// production application reports Unsupported before manufacturing an image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSImageValue {
    pub url: String,
}
impl CSSValueSubclass for CSSImageValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from("url(");
        text.push_string(&crate::css_markup::SerializeString(&self.url));
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
impl CSSValueSubresources for CSSImageValue {
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        false
    }
}
impl CSSValueUrl<()> for CSSImageValue {
    fn ReResolveUrl(&self, _: &()) {
        panic!("CSSImageValue::ReResolveURL requires application Document/URL binding")
    }
}
pub fn image(url: String) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kImageClass(CSSImageValue {
        url,
    })))
}
// cpp: css_uri_value.h:23-60. This URL retains parsed identity until the
// document supplies its real SVG resource binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSURIValue {
    pub url: String,
}
impl CSSValueSubclass for CSSURIValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from("url(");
        text.push_string(&crate::css_markup::SerializeString(&self.url));
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
impl CSSValueUrl<()> for CSSURIValue {
    fn ReResolveUrl(&self, _: &()) {
        panic!("CSSURIValue::ReResolveURL requires application Document/URL binding")
    }
}
pub fn uri(url: String) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kURIClass(CSSURIValue { url })))
}
// cpp: css_gradient_value.h CSSGradientColorStop, CSSLinearGradientValue;
// css_gradient_value.cc:1192-1276,1649-1669 (unprefixed branches).
#[derive(PartialEq)]
pub struct CSSGradientColorStop {
    pub color: Option<Rc<Value>>,
    pub offset: Option<Rc<Value>>,
}
pub struct CSSLinearGradientValue {
    pub angle: Option<Rc<Value>>,
    pub end_x: Option<CSSValueID>,
    pub end_y: Option<CSSValueID>,
    pub repeating: bool,
    pub stops: Vec<CSSGradientColorStop>,
}
impl CSSLinearGradientValue {
    pub fn Degrees(&self) -> Option<f64> {
        let CSSValuePayload::kNumericLiteralClass(angle) = self.angle.as_ref()?.Payload() else {
            return None;
        };
        Some(
            angle.DoubleValue()
                * match angle.GetType() {
                    UnitType::kDegrees => 1.0,
                    UnitType::kRadians => 180.0 / std::f64::consts::PI,
                    UnitType::kGradians => 0.9,
                    UnitType::kTurns => 360.0,
                    _ => return None,
                },
        )
    }
}
impl CSSValueSubclass for CSSLinearGradientValue {
    fn CustomCSSText(&self) -> String {
        let mut items = Vec::new();
        if let Some(angle) = &self.angle {
            if self.Degrees() != Some(180.0) {
                items.push(angle.CssText().Utf8());
            }
        } else if self.end_x.is_some() || self.end_y.is_some_and(|y| y != CSSValueID::kBottom) {
            let mut direction = vec!["to".to_owned()];
            for side in [self.end_x, self.end_y].into_iter().flatten() {
                direction.push(GetCSSValueName(side).to_owned());
            }
            items.push(direction.join(" "));
        }
        for stop in &self.stops {
            let mut pieces = Vec::new();
            if let Some(color) = &stop.color {
                pieces.push(color.CssText().Utf8());
            }
            if let Some(offset) = &stop.offset {
                pieces.push(offset.CssText().Utf8());
            }
            items.push(pieces.join(" "));
        }
        String::from(
            format!(
                "{}linear-gradient({})",
                if self.repeating { "repeating-" } else { "" },
                items.join(", ")
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.angle == other.angle
            && self.end_x == other.end_x
            && self.end_y == other.end_y
            && self.repeating == other.repeating
            && self.stops == other.stops
    }
}
impl CSSValueRandom for CSSLinearGradientValue {
    fn HasRandomFunctions(&self) -> bool {
        self.angle.as_ref().is_some_and(|v| v.HasRandomFunctions())
            || self.stops.iter().any(|stop| {
                [&stop.color, &stop.offset]
                    .into_iter()
                    .flatten()
                    .any(|v| v.HasRandomFunctions())
            })
    }
}
// cpp: css_font_feature_value.h:42-65; .cc:40-69.
pub struct CSSFontFeatureValue {
    pub tag: foundation::AtomicString,
    pub value: Rc<Value>,
}
impl CSSValueSubclass for CSSFontFeatureValue {
    fn CustomCSSText(&self) -> String {
        let mut text = crate::css_markup::SerializeString(&String::from_utf16(
            self.tag.utf16_units().unwrap_or_default(),
        ));
        let omit = matches!(self.value.Payload(), CSSValuePayload::kNumericLiteralClass(n) if n.DoubleValue() as i32 == 1);
        if !omit {
            text.push_str(" ");
            text.push_string(&self.value.CssText());
        }
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.tag == other.tag && self.value == other.value
    }
}
impl CSSValueRandom for CSSFontFeatureValue {
    fn HasRandomFunctions(&self) -> bool {
        self.value.HasRandomFunctions()
    }
}
// cpp: css_font_variation_value.h:18-38; .cc:14-40.
pub struct CSSFontVariationValue {
    pub tag: foundation::AtomicString,
    pub value: Rc<Value>,
}
impl CSSValueSubclass for CSSFontVariationValue {
    fn CustomCSSText(&self) -> String {
        let mut text = crate::css_markup::SerializeString(&String::from_utf16(
            self.tag.utf16_units().unwrap_or_default(),
        ));
        text.push_str(" ");
        text.push_string(&self.value.CssText());
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.tag == other.tag && self.value == other.value
    }
}
impl CSSValueRandom for CSSFontVariationValue {
    fn HasRandomFunctions(&self) -> bool {
        self.value.HasRandomFunctions()
    }
}
// cpp: css_shadow_value.h:43-64; css_shadow_value.cc:29-98.
// Shared by box-shadow and text-shadow. Omitted optional components remain
// absent in specified values; zero/currentcolor defaults bind in the converter.
pub struct CSSShadowValue {
    pub x: Rc<Value>,
    pub y: Rc<Value>,
    pub blur: Option<Rc<Value>>,
    pub spread: Option<Rc<Value>>,
    pub style: Option<Rc<Value>>,
    pub color: Option<Rc<Value>>,
}
impl CSSValueSubclass for CSSShadowValue {
    fn CustomCSSText(&self) -> String {
        let mut parts = Vec::new();
        if let Some(color) = &self.color {
            parts.push(color.CssText().Utf8());
        }
        parts.push(self.x.CssText().Utf8());
        parts.push(self.y.CssText().Utf8());
        for value in [&self.blur, &self.spread, &self.style]
            .into_iter()
            .flatten()
        {
            parts.push(value.CssText().Utf8());
        }
        String::from(parts.join(" ").as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self.color == other.color
            && self.x == other.x
            && self.y == other.y
            && self.blur == other.blur
            && self.spread == other.spread
            && self.style == other.style
    }
}
impl CSSValueRandom for CSSShadowValue {
    fn HasRandomFunctions(&self) -> bool {
        self.x.HasRandomFunctions()
            || self.y.HasRandomFunctions()
            || [&self.blur, &self.spread, &self.style, &self.color]
                .into_iter()
                .flatten()
                .any(|value| value.HasRandomFunctions())
    }
}
// cpp: css_string_value.h:16-31; .cc:12-17.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSStringValue(pub String);
impl CSSValueSubclass for CSSStringValue {
    fn CustomCSSText(&self) -> String {
        crate::css_markup::SerializeString(&self.0)
    }
    fn Equals(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl CSSValueCustomHash for CSSStringValue {
    fn CustomHash(&self) -> u32 {
        foundation::AtomicString::from_utf16(self.0.Span16().unwrap_or_default()).Hash()
    }
}
// cpp: css_repeat_style_value.h:19-45; .cc:12-76.
pub struct CSSRepeatStyleValue {
    pub x: Rc<Value>,
    pub y: Rc<Value>,
}
impl CSSValueSubclass for CSSRepeatStyleValue {
    fn CustomCSSText(&self) -> String {
        if self.x == self.y {
            return self.x.CssText();
        }
        if self.AxisID(&self.x) == CSSValueID::kRepeat
            && self.AxisID(&self.y) == CSSValueID::kNoRepeat
        {
            return String::from("repeat-x");
        }
        if self.AxisID(&self.x) == CSSValueID::kNoRepeat
            && self.AxisID(&self.y) == CSSValueID::kRepeat
        {
            return String::from("repeat-y");
        }
        let mut text = self.x.CssText();
        text.push_str(" ");
        text.push_string(&self.y.CssText());
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl CSSRepeatStyleValue {
    fn AxisID(&self, value: &Value) -> CSSValueID {
        match value.Payload() {
            CSSValuePayload::kIdentifierClass(value) => value.0,
            _ => panic!("CSSRepeatStyleValue requires CSSIdentifierValue axes"),
        }
    }
    pub fn IsRepeat(&self) -> bool {
        self.AxisID(&self.x) == CSSValueID::kRepeat && self.AxisID(&self.y) == CSSValueID::kRepeat
    }
}
impl CSSValueRandom for CSSRepeatStyleValue {
    fn HasRandomFunctions(&self) -> bool {
        self.x.HasRandomFunctions() || self.y.HasRandomFunctions()
    }
}
// cpp: css_trigger_attachment_value.h:30-66; .cc:12-54.
pub struct CSSTriggerAttachmentValue {
    pub name: Rc<Value>,
    pub enter: Rc<Value>,
    pub exit: Option<Rc<Value>>,
}
impl CSSValueSubclass for CSSTriggerAttachmentValue {
    fn CustomCSSText(&self) -> String {
        let mut text = self.name.CssText();
        text.push_str(" ");
        text.push_string(&self.enter.CssText());
        if let Some(exit) = &self.exit {
            text.push_str(" ");
            text.push_string(&exit.CssText());
        }
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.name == other.name
            && Rc::ptr_eq(&self.enter, &other.enter)
            && match (&self.exit, &other.exit) {
                (Some(first), Some(second)) => Rc::ptr_eq(first, second),
                (None, None) => true,
                _ => false,
            }
    }
}
impl CSSValueRandom for CSSTriggerAttachmentValue {
    fn HasRandomFunctions(&self) -> bool {
        self.name.HasRandomFunctions()
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSTriggerAttachmentValue {
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        panic!(
            "CSSTriggerAttachmentValue::PopulateWithTreeScope requires application TreeScope binding"
        )
    }
}
// cpp: css_timing_function_value.h:67-126; .cc:57-109.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CSSCubicBezierTimingFunctionValue(pub [f64; 4]);
impl CSSValueSubclass for CSSCubicBezierTimingFunctionValue {
    fn CustomCSSText(&self) -> String {
        let args = self
            .0
            .iter()
            .map(|&n| numeric(n, UnitType::kNumber).CssText().Utf8())
            .collect::<Vec<_>>()
            .join(", ");
        String::from(format!("cubic-bezier({args})").as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
pub struct CSSStepsTimingFunctionValue {
    pub steps: Rc<Value>,
    pub position: CSSValueID,
}
impl CSSValueSubclass for CSSStepsTimingFunctionValue {
    fn CustomCSSText(&self) -> String {
        let count = self.steps.CssText().Utf8();
        if matches!(self.position, CSSValueID::kEnd | CSSValueID::kJumpEnd) {
            String::from(format!("steps({count})").as_str())
        } else {
            String::from(format!("steps({count}, {})", GetCSSValueName(self.position)).as_str())
        }
    }
    fn Equals(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.steps, &other.steps) && self.position == other.position
    }
}
impl CSSValueRandom for CSSStepsTimingFunctionValue {
    fn HasRandomFunctions(&self) -> bool {
        self.steps.HasRandomFunctions()
    }
}
// cpp: css_timing_function_value.h:46-65; .cc:36-55.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearEasingPoint {
    pub input: f64,
    pub output: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CSSLinearTimingFunctionValue(pub Vec<LinearEasingPoint>);
impl CSSValueSubclass for CSSLinearTimingFunctionValue {
    fn CustomCSSText(&self) -> String {
        let points = self
            .0
            .iter()
            .map(|point| {
                format!(
                    "{} {}%",
                    numeric(point.output, UnitType::kNumber).CssText().Utf8(),
                    numeric(point.input, UnitType::kNumber).CssText().Utf8()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        String::from(format!("linear({points})").as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
// cpp: css_font_family_value.h:23-46; css_font_family_value.cc:29-42
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSFontFamilyValue(pub String);
impl CSSValueSubclass for CSSFontFamilyValue {
    fn CustomCSSText(&self) -> String {
        // cpp: css_markup.cc:225-232; css_parsing_utils.cc:4415-4432.
        // CSSFontFamilySerialization is experimental and disabled by default.
        let name = self.0.Utf8();
        let reserved = [
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "default",
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "ui-serif",
            "ui-sans-serif",
            "ui-monospace",
            "ui-rounded",
            "math",
            "fangsong",
        ];
        if reserved.iter().any(|word| name.eq_ignore_ascii_case(word))
            || !crate::css_markup::IsCSSTokenizerIdentifier(&foundation::StringView::from(&self.0))
        {
            crate::css_markup::SerializeString(&self.0)
        } else {
            self.0.clone()
        }
    }
    fn Equals(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
// cpp: css_font_face_src_value.cc:110-140. URL resolution/loading belongs to
// the document consumer; parser values retain the decoded CSS URL reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontFaceResource {
    Local(String),
    Url(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSFontFaceSrcValue {
    pub resource: FontFaceResource,
    pub format: Option<String>,
    pub technologies: Vec<CSSValueID>,
}
impl CSSValueSubclass for CSSFontFaceSrcValue {
    fn CustomCSSText(&self) -> String {
        let (function, resource) = match &self.resource {
            FontFaceResource::Local(name) => ("local", name),
            FontFaceResource::Url(url) => ("url", url),
        };
        let mut text = format!(
            "{function}({})",
            crate::css_markup::SerializeString(resource).Utf8()
        );
        if let Some(format) = &self.format {
            text.push_str(&format!(
                " format({})",
                crate::css_markup::SerializeString(format).Utf8()
            ));
        }
        if !self.technologies.is_empty() {
            text.push_str(" tech(");
            text.push_str(
                &self
                    .technologies
                    .iter()
                    .map(|id| GetCSSValueName(*id))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            text.push(')');
        }
        String::from_utf16(&text.encode_utf16().collect::<Vec<_>>())
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
impl CSSValueSubresources for CSSFontFaceSrcValue {
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        false
    }
}
// cpp: css_unicode_range_value.cc:41-51.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSUnicodeRangeValue {
    pub from: u32,
    pub to: u32,
}
impl CSSValueSubclass for CSSUnicodeRangeValue {
    fn CustomCSSText(&self) -> String {
        String::from(
            if self.from == self.to {
                format!("U+{:X}", self.from)
            } else {
                format!("U+{:X}-{:X}", self.from, self.to)
            }
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
// cpp: css_inherited_value.h; css_initial_value.h; css_unset_value.h;
// css_revert_value.h; css_revert_layer_value.h; css_revert_rule_value.h.
macro_rules! wide_keyword {
    ($name:ident,$text:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name;
        impl CSSValueSubclass for $name {
            fn CustomCSSText(&self) -> String {
                String::from($text)
            }
            fn Equals(&self, _: &Self) -> bool {
                true
            }
        }
    };
}
wide_keyword!(CSSInheritedValue, "inherit");
wide_keyword!(CSSInitialValue, "initial");
wide_keyword!(CSSUnsetValue, "unset");
wide_keyword!(CSSRevertValue, "revert");
wide_keyword!(CSSRevertLayerValue, "revert-layer");
wide_keyword!(CSSRevertRuleValue, "revert-rule");

// cpp: css_variable_data.h:31-169; css_variable_data.cc:24-117,158-164.
// Retained tokenizer output is the Rust ownership equivalent of OriginalText:
// substitution copies tokens, preserving token boundaries without re-tokenizing.
#[derive(Clone, Debug)]
pub struct VariableToken {
    pub token: crate::parser::css_parser_token::CSSParserToken,
    pub text: String,
}
#[derive(Clone, Debug)]
pub struct CSSVariableData {
    pub tokens: Vec<VariableToken>,
    pub original_text: String,
    pub features: u8,
    pub is_animation_tainted: bool,
    pub is_attr_tainted: bool,
}
impl CSSVariableData {
    pub const MAX_VARIABLE_BYTES: usize = 2_097_152;
    pub const HAS_FONT_UNITS: u8 = 1;
    pub const HAS_ROOT_FONT_UNITS: u8 = 2;
    pub const HAS_LINE_HEIGHT_UNITS: u8 = 4;
    pub const HAS_DASHED_FUNCTIONS: u8 = 8;
    pub const HAS_REFERENCES: u8 = 16;
    pub const HAS_RANDOM_FUNCTIONS: u8 = 32;
    pub fn NeedsVariableResolution(&self) -> bool {
        self.features & Self::HAS_REFERENCES != 0
    }
    pub fn ExtractFeatures(token: &crate::parser::css_parser_token::CSSParserToken) -> u8 {
        use crate::parser::css_parser_token::CSSParserTokenType::*;
        let mut features = 0;
        if token.GetType() == kDimensionToken {
            features |= match token.GetUnitType() {
                UnitType::kEms
                | UnitType::kChs
                | UnitType::kExs
                | UnitType::kIcs
                | UnitType::kCaps => Self::HAS_FONT_UNITS,
                UnitType::kRems
                | UnitType::kRexs
                | UnitType::kRchs
                | UnitType::kRics
                | UnitType::kRlhs
                | UnitType::kRcaps => Self::HAS_ROOT_FONT_UNITS,
                UnitType::kLhs => Self::HAS_LINE_HEIGHT_UNITS,
                _ => 0,
            };
        }
        if token.GetType() == kFunctionToken {
            if token.Value().ToString().Utf8().starts_with("--") {
                features |= Self::HAS_DASHED_FUNCTIONS;
            }
            if token.FunctionId() == Some(CSSValueID::kRandom) {
                features |= Self::HAS_RANDOM_FUNCTIONS;
            }
            if matches!(
                token.FunctionId(),
                Some(
                    CSSValueID::kVar
                        | CSSValueID::kEnv
                        | CSSValueID::kAttr
                        | CSSValueID::kInherit
                        | CSSValueID::kIf
                )
            ) {
                features |= Self::HAS_REFERENCES;
            }
        }
        features
    }
    // cpp: css_variable_data.cc:119-156 Serialize EOF escape handling.
    pub fn Serialize(&self) -> String {
        use crate::parser::css_parser_token::CSSParserTokenType::*;
        if self
            .original_text
            .CodeUnitAt(self.original_text.length().wrapping_sub(1))
            != b'\\' as u16
        {
            return self.original_text.clone();
        }
        let mut units = self.original_text.Span16().unwrap_or_default().to_vec();
        units.pop();
        let last = self
            .tokens
            .iter()
            .rfind(|item| item.token.GetType() != kCommentToken)
            .map_or(kEOFToken, |item| item.token.GetType());
        if last != kStringToken {
            units.push(0xfffd);
        }
        if last == kStringToken {
            units.push(b'"' as u16);
        }
        if last == kUrlToken {
            units.push(b')' as u16);
        }
        String::Create8BitIfPossible(&units)
    }
    pub fn FromTokens(tokens: Vec<VariableToken>, animation: bool, attr: bool) -> Self {
        let units = tokens
            .iter()
            .flat_map(|item| item.text.Span16().unwrap_or_default().iter().copied())
            .collect::<Vec<_>>();
        let original_text = String::Create8BitIfPossible(&units);
        let mut features = 0;
        for item in &tokens {
            features |= Self::ExtractFeatures(&item.token);
        }
        Self {
            tokens,
            original_text,
            features,
            is_animation_tainted: animation,
            is_attr_tainted: attr,
        }
    }
}
// cpp: css_unparsed_declaration_value.h:21-62; .cc:12-32.
#[derive(Clone)]
pub struct CSSUnparsedDeclarationValue {
    pub data: Rc<CSSVariableData>,
    pub mode: crate::parser::css_parser_mode::CSSParserMode,
}
impl CSSValueSubclass for CSSUnparsedDeclarationValue {
    fn CustomCSSText(&self) -> String {
        self.data.Serialize()
    }
    fn Equals(&self, other: &Self) -> bool {
        self.data.original_text == other.data.original_text
            && self.data.is_attr_tainted == other.data.is_attr_tainted
    }
}
impl CSSValueCustomHash for CSSUnparsedDeclarationValue {
    fn CustomHash(&self) -> u32 {
        if let Some(bytes) = self.data.original_text.Span8() {
            foundation::rapidhash::rapidhash(bytes) as u32
        } else {
            let bytes = self
                .data
                .original_text
                .Span16()
                .unwrap_or_default()
                .iter()
                .flat_map(|unit| unit.to_le_bytes())
                .collect::<Vec<_>>();
            foundation::rapidhash::rapidhash(&bytes) as u32
        }
    }
}
impl CSSValueRandom for CSSUnparsedDeclarationValue {
    fn HasRandomFunctions(&self) -> bool {
        self.data.features & CSSVariableData::HAS_RANDOM_FUNCTIONS != 0
    }
}
// cpp: css_pending_substitution_value.h:17-46; .cc:12-26.
pub struct CSSPendingSubstitutionValue {
    pub shorthand: CSSPropertyID,
    pub value: Rc<Value>,
}
impl CSSValueSubclass for CSSPendingSubstitutionValue {
    fn CustomCSSText(&self) -> String {
        String::from("")
    }
    fn Equals(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
pub fn unparsed(
    data: CSSVariableData,
    mode: crate::parser::css_parser_mode::CSSParserMode,
) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kUnparsedDeclarationClass(
        CSSUnparsedDeclarationValue {
            data: Rc::new(data),
            mode,
        },
    )))
}
pub fn pending_substitution(shorthand: CSSPropertyID, value: Rc<Value>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kPendingSubstitutionValueClass(
        CSSPendingSubstitutionValue { shorthand, value },
    )))
}

// This enum has no inhabitants and cannot become a CSSValue payload. It keeps
// untranslated derived-class dependencies explicit in CSSValue's dispatch.
pub enum UnavailableCSSValue {}
impl CSSValueSubclass for UnavailableCSSValue {
    fn CustomCSSText(&self) -> String {
        match *self {}
    }
    fn Equals(&self, _: &Self) -> bool {
        match *self {}
    }
}
impl CSSValueCustomHash for UnavailableCSSValue {
    fn CustomHash(&self) -> u32 {
        match *self {}
    }
}
impl CSSValueRandom for UnavailableCSSValue {
    fn HasRandomFunctions(&self) -> bool {
        match *self {}
    }
}
impl CSSValueSubresources for UnavailableCSSValue {
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        match *self {}
    }
}
impl CSSValueUrl<()> for UnavailableCSSValue {
    fn ReResolveUrl(&self, _: &()) {
        match *self {}
    }
}
impl CSSValueListUrls for UnavailableCSSValue {
    fn MayContainUrl(&self) -> bool {
        match *self {}
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for UnavailableCSSValue {
    fn AsValueList(&self) -> &CSSValueList {
        match *self {}
    }
}
impl CSSValuePairSubclass<ProductionCSSValueDispatch> for UnavailableCSSValue {
    fn AsValuePair(&self) -> &CSSValuePair {
        match *self {}
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for UnavailableCSSValue {
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        match *self {}
    }
}
impl CSSValueDispatch for ProductionCSSValueDispatch {
    type Document = ();
    type TreeScope = ();
    type CSSNumericLiteralValue = CSSNumericLiteralValue;
    type CSSMathFunctionValue = crate::css_math_function_value::CSSMathFunctionValue;
    type CSSIdentifierValue = CSSIdentifierValue;
    type CSSScopedKeywordValue = CSSScopedKeywordValue;
    type CSSColor = CSSColor;
    type CSSUnresolvedColorValue = UnavailableCSSValue;
    type CSSColorMixValue = UnavailableCSSValue;
    type CSSAlphaColorValue = UnavailableCSSValue;
    type CSSContrastColorValue = UnavailableCSSValue;
    type CSSCounterValue = CSSCounterValue;
    type CSSCounterContentValue = CSSCounterContentValue;
    type CSSQuadValue = CSSQuadValue;
    type CSSCustomIdentValue = CSSCustomIdentValue;
    type CSSStringValue = CSSStringValue;
    type CSSURIValue = CSSURIValue;
    type CSSURLPatternValue = UnavailableCSSValue;
    type CSSValuePair = CSSValuePair;
    type CSSLightDarkValuePair = UnavailableCSSValue;
    type CSSParamValuePair = UnavailableCSSValue;
    type CSSScrollValue = UnavailableCSSValue;
    type CSSViewValue = UnavailableCSSValue;
    type CSSRatioValue = CSSRatioValue;
    type CSSRelativeColorValue = UnavailableCSSValue;
    type CSSBasicShapeCircleValue = UnavailableCSSValue;
    type CSSBasicShapeEllipseValue = UnavailableCSSValue;
    type CSSBasicShapePolygonValue = crate::production_effects_value::CSSBasicShapePolygonValue;
    type CSSBasicShapeInsetValue = UnavailableCSSValue;
    type CSSBasicShapeRectValue = UnavailableCSSValue;
    type CSSBasicShapeXYWHValue = UnavailableCSSValue;
    type CSSPathValue = UnavailableCSSValue;
    type CSSShapeValue = UnavailableCSSValue;
    type CSSImageValue = CSSImageValue;
    type CSSCursorImageValue = UnavailableCSSValue;
    type CSSCrossfadeValue = UnavailableCSSValue;
    type CSSPaintValue = UnavailableCSSValue;
    type CSSLinearGradientValue = CSSLinearGradientValue;
    type CSSRadialGradientValue = UnavailableCSSValue;
    type CSSConicGradientValue = UnavailableCSSValue;
    type CSSConstantGradientValue = UnavailableCSSValue;
    type CSSColorImageValue = UnavailableCSSValue;
    type CSSLinearTimingFunctionValue = CSSLinearTimingFunctionValue;
    type CSSCubicBezierTimingFunctionValue = CSSCubicBezierTimingFunctionValue;
    type CSSStepsTimingFunctionValue = CSSStepsTimingFunctionValue;
    type CSSProgressValue = UnavailableCSSValue;
    type CSSBorderImageSliceValue = CSSBorderImageSliceValue;
    type CSSDynamicRangeLimitMixValue =
        crate::production_dynamic_range_value::CSSDynamicRangeLimitMixValue;
    type CSSFontFeatureValue = CSSFontFeatureValue;
    type CSSFontFaceSrcValue = CSSFontFaceSrcValue;
    type CSSFontFamilyValue = CSSFontFamilyValue;
    type CSSFontStyleRangeValue = CSSFontStyleRangeValue;
    type CSSFontVariationValue = CSSFontVariationValue;
    type CSSAlternateValue = CSSAlternateValue;
    type CSSInheritedValue = CSSInheritedValue;
    type CSSInitialValue = CSSInitialValue;
    type CSSUnsetValue = CSSUnsetValue;
    type CSSRevertValue = CSSRevertValue;
    type CSSRevertLayerValue = CSSRevertLayerValue;
    type CSSRevertRuleValue = CSSRevertRuleValue;
    type CSSReflectValue = crate::production_reflect_value::CSSReflectValue;
    type CSSShadowValue = CSSShadowValue;
    type CSSUnicodeRangeValue = CSSUnicodeRangeValue;
    type CSSGridTemplateAreasValue = CSSGridTemplateAreasValue;
    type CSSPaletteMixValue = UnavailableCSSValue;
    type CSSRayValue = crate::production_motion_value::CSSRayValue;
    type CSSUnparsedDeclarationValue = CSSUnparsedDeclarationValue;
    type CSSPendingSubstitutionValue = CSSPendingSubstitutionValue;
    type CSSPendingSystemFontValue = UnavailableCSSValue;
    type CSSInvalidVariableValue = UnavailableCSSValue;
    type CSSCyclicVariableValue = UnavailableCSSValue;
    type CSSFlipRevertValue = UnavailableCSSValue;
    type CSSLayoutFunctionValue = UnavailableCSSValue;
    type CSSContentDistributionValue = CSSContentDistributionValue;
    type CSSKeyframeShorthandValue = UnavailableCSSValue;
    type CSSInitialColorValue = UnavailableCSSValue;
    type CSSImageSetOptionValue = UnavailableCSSValue;
    type CSSImageSetTypeValue = UnavailableCSSValue;
    type CSSRepeatStyleValue = CSSRepeatStyleValue;
    type CSSSuperellipseValue = crate::production_corner_value::CSSSuperellipseValue;
    type CSSSymbolsValue = UnavailableCSSValue;
    type CSSTriggerAttachmentValue = CSSTriggerAttachmentValue;
    type CSSRepeatValue = UnavailableCSSValue;
    type CSSValueList = CSSValueList;
    type CSSFunctionValue = CSSFunctionValue;
    type CSSImageSetValue = UnavailableCSSValue;
    type CSSBracketedValueList = UnavailableCSSValue;
    type CSSGridAutoRepeatValue = UnavailableCSSValue;
    type CSSGridIntegerRepeatValue = UnavailableCSSValue;
    type CSSAxisValue = CSSAxisValue;
    // cpp: css_identifier_value.cc:55-83
    fn CreateIdentifierFromLength(length: &Length) -> Rc<Value> {
        let id = match length.GetType() {
            LengthType::kAuto => CSSValueID::kAuto,
            LengthType::kMinContent => CSSValueID::kMinContent,
            LengthType::kMaxContent => CSSValueID::kMaxContent,
            LengthType::kStretch => CSSValueID::kStretch,
            LengthType::kFitContent => CSSValueID::kFitContent,
            LengthType::kContent => CSSValueID::kContent,
            _ => unreachable!("CSSValue::CreateFromLength identifier precondition"),
        };
        identifier(id)
    }
    // cpp: css_primitive_value.cc:CreateFromLength fixed/percent branches.
    fn CreatePrimitiveFromLength(length: &Length, zoom: f32) -> Rc<Value> {
        if length.IsFixed() {
            numeric((length.Pixels() / zoom) as f64, UnitType::kPixels)
        } else if length.IsPercent() {
            numeric(length.PercentValue() as f64, UnitType::kPercentage)
        } else {
            unreachable!("calculated Length is unavailable to production literal parser")
        }
    }
}
impl CSSPropertyValueBackend for ProductionCSSValueDispatch {
    fn MatchingShorthandsForLonghand(id: CSSPropertyID) -> Vec<CSSPropertyID> {
        crate::parser::production_property_metadata::MatchingShorthands(id)
    }
    // cpp: css_property.h:120-122; direction/unicode-bidi source overrides.
    fn IsAffectedByAll(id: CSSPropertyID) -> bool {
        let property = crate::properties::css_property::CSSProperty::Get(id);
        !property.IsInternal()
            && property.IsProperty()
            && !crate::parser::production_property_metadata::ExcludedFromAll(id)
    }
}
// cpp: css_identifier_value.cc:15-23; css_numeric_literal_value.cc:33-80;
// css_value_pool.h:61,105-140. Production Rc values share source cached payloads.
thread_local! {
    static IDENTIFIER_CACHE: std::cell::RefCell<std::collections::HashMap<CSSValueID, Rc<Value>>> = std::cell::RefCell::new(std::collections::HashMap::new());
    static NUMERIC_CACHE: std::cell::RefCell<std::collections::HashMap<(u16, u16), Rc<Value>>> = std::cell::RefCell::new(std::collections::HashMap::new());
}
// cpp: css_string_value.h:16-31.
pub fn string(value: String) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kStringClass(CSSStringValue(
        value,
    ))))
}

// cpp: longhands_custom.cc:3066-3067,3108 ParseContentValue. Content shares
// Chromium's CSSValueList payloads, including after attr() replay.
pub fn content_list(items: Vec<Rc<Value>>) -> Rc<Value> {
    list(
        vec![list(items, ListSeparator::Space)],
        ListSeparator::Slash,
    )
}

pub fn identifier(id: CSSValueID) -> Rc<Value> {
    IDENTIFIER_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                Rc::new(Value::new(CSSValuePayload::kIdentifierClass(
                    CSSIdentifierValue(id),
                )))
            })
            .clone()
    })
}
pub fn numeric(number: f64, unit: UnitType) -> Rc<Value> {
    let cached_unit = match unit {
        UnitType::kNumber | UnitType::kInteger => Some(UnitType::kInteger),
        UnitType::kPixels | UnitType::kPercentage => Some(unit),
        _ => None,
    };
    if let Some(unit) = cached_unit {
        if (0.0..=255.0).contains(&number)
            && number == number.trunc()
            && !(number == 0.0 && number.is_sign_negative())
        {
            return NUMERIC_CACHE.with(|cache| {
                cache
                    .borrow_mut()
                    .entry((unit as u16, number as u16))
                    .or_insert_with(|| {
                        Rc::new(Value::new(CSSValuePayload::kNumericLiteralClass(
                            CSSNumericLiteralValue::Create(number, unit),
                        )))
                    })
                    .clone()
            });
        }
    }
    Rc::new(Value::new(CSSValuePayload::kNumericLiteralClass(
        CSSNumericLiteralValue::Create(number, unit),
    )))
}
pub fn color(color: Color) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kColorClass(CSSColor(color))))
}
pub fn wide(id: CSSValueID) -> Option<Rc<Value>> {
    let payload = match id {
        CSSValueID::kInherit => CSSValuePayload::kInheritedClass(CSSInheritedValue),
        CSSValueID::kInitial => CSSValuePayload::kInitialClass(CSSInitialValue),
        CSSValueID::kUnset => CSSValuePayload::kUnsetClass(CSSUnsetValue),
        CSSValueID::kRevert => CSSValuePayload::kRevertClass(CSSRevertValue),
        CSSValueID::kRevertLayer => CSSValuePayload::kRevertLayerClass(CSSRevertLayerValue),
        _ => return None,
    };
    Some(Rc::new(Value::new(payload)))
}

// cpp: css_value_list.h:40-119; css_value_list.cc CustomCSSText/Equals/
// CustomHash/HasFailedOrCanceledSubresources/MayContainUrl/ReResolveUrl.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ListSeparator {
    Space = 0,
    Comma = 1,
    Slash = 2,
}
pub struct CSSValueList {
    pub values: Vec<Rc<Value>>,
    pub separator: ListSeparator,
    scoped: std::cell::OnceCell<Box<Value>>,
}
impl CSSValueList {
    pub fn new(values: Vec<Rc<Value>>, separator: ListSeparator) -> Self {
        Self {
            values,
            separator,
            scoped: std::cell::OnceCell::new(),
        }
    }
}
impl CSSValueSubclass for CSSValueList {
    fn CustomCSSText(&self) -> String {
        let separator = match self.separator {
            ListSeparator::Space => " ",
            ListSeparator::Comma => ", ",
            ListSeparator::Slash => " / ",
        };
        String::from(
            self.values
                .iter()
                .map(|value| value.CssText().Utf8())
                .collect::<Vec<_>>()
                .join(separator)
                .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.separator == other.separator && self.values == other.values
    }
}
impl CSSValueCustomHash for CSSValueList {
    fn CustomHash(&self) -> u32 {
        let mut hash = self.separator as u32;
        for value in &self.values {
            AddIntToHash(&mut hash, value.Hash());
        }
        hash
    }
}
impl CSSValueRandom for CSSValueList {
    fn HasRandomFunctions(&self) -> bool {
        self.values.iter().any(|value| value.HasRandomFunctions())
    }
}
impl CSSValueSubresources for CSSValueList {
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        self.values
            .iter()
            .any(|value| value.HasFailedOrCanceledSubresources())
    }
}
impl CSSValueListUrls for CSSValueList {
    fn MayContainUrl(&self) -> bool {
        self.values.iter().any(|value| value.MayContainUrl())
    }
}
impl CSSValueUrl<()> for CSSValueList {
    fn ReResolveUrl(&self, document: &()) {
        for value in &self.values {
            value.ReResolveUrl(document);
        }
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSValueList {
    // cpp: css_value_list.cc PopulateWithTreeScope. Production lists contain
    // only literals, identifiers, font families, and nested lists with no
    // scope-dependent payload. A Box retains the source GC result's lifetime;
    // no raw self-pointer is required to return the allocated base value.
    fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a Value {
        assert!(
            self.values
                .iter()
                .all(|value| !value.State().NeedsTreeScopePopulation()),
            "CSSValueList::PopulateWithTreeScope requires application TreeScope binding for custom identifiers"
        );
        self.scoped.get_or_init(|| {
            Box::new(Value::new(CSSValuePayload::kValueListClass(CSSValueList {
                values: self.values.clone(),
                separator: self.separator,
                scoped: std::cell::OnceCell::new(),
            })))
        })
    }
}
// cpp: css_function_value.h:14-40; css_function_value.cc:14-21.
pub struct CSSFunctionValue {
    pub function_id: CSSValueID,
    pub arguments: CSSValueList,
}
impl CSSValueSubclass for CSSFunctionValue {
    fn CustomCSSText(&self) -> String {
        String::from(
            format!(
                "{}({})",
                GetCSSValueName(self.function_id),
                self.arguments.CustomCSSText().Utf8()
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.function_id == other.function_id && self.arguments.Equals(&other.arguments)
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for CSSFunctionValue {
    fn AsValueList(&self) -> &CSSValueList {
        &self.arguments
    }
}
pub fn function(function_id: CSSValueID, arguments: Vec<Rc<Value>>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kFunctionClass(
        CSSFunctionValue {
            function_id,
            arguments: CSSValueList::new(arguments, ListSeparator::Comma),
        },
    )))
}
// cpp: css_axis_value.cc:22-114. The native list retains typed primitive values.
pub struct CSSAxisValue {
    pub axis_name: CSSValueID,
    pub dimensions: CSSValueList,
}
impl CSSAxisValue {
    pub fn new(mut dimensions: Vec<Rc<Value>>) -> Self {
        assert_eq!(dimensions.len(), 3);
        let mut axis_name = CSSValueID::kInvalid;
        if let Some(numbers) = dimensions
            .iter()
            .map(|v| match v.Payload() {
                CSSValuePayload::kNumericLiteralClass(n) => Some(n.DoubleValue()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
        {
            for index in 0..3 {
                if numbers[index] > 0.0
                    && (0..3).all(|other| other == index || numbers[other] == 0.0)
                {
                    dimensions[index] = numeric(1.0, UnitType::kNumber);
                    axis_name = [CSSValueID::kX, CSSValueID::kY, CSSValueID::kZ][index];
                    break;
                }
            }
        }
        Self {
            axis_name,
            dimensions: CSSValueList::new(dimensions, ListSeparator::Space),
        }
    }
}
impl CSSValueSubclass for CSSAxisValue {
    fn CustomCSSText(&self) -> String {
        if self.axis_name == CSSValueID::kInvalid {
            self.dimensions.CustomCSSText()
        } else {
            String::from(GetCSSValueName(self.axis_name))
        }
    }
    fn Equals(&self, other: &Self) -> bool {
        self.dimensions.Equals(&other.dimensions)
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for CSSAxisValue {
    fn AsValueList(&self) -> &CSSValueList {
        &self.dimensions
    }
}
pub fn list(values: Vec<Rc<Value>>, separator: ListSeparator) -> Rc<Value> {
    let needs_scope = values
        .iter()
        .any(|value| value.State().NeedsTreeScopePopulation());
    let mut value = Value::new(CSSValuePayload::kValueListClass(CSSValueList {
        values,
        separator,
        scoped: std::cell::OnceCell::new(),
    }));
    value.StateMut().SetNeedsTreeScopePopulation(needs_scope);
    Rc::new(value)
}
/// Source named-color table; excludes context-dependent system/current colors.
pub fn NamedColor(id: CSSValueID) -> Option<Color> {
    crate::parser::production_property_metadata::NamedColor(GetCSSValueName(id))
}

pub fn math(
    expression: crate::css_math_expression_node::CSSMathExpressionNode,
    range: crate::css_math_function_value::ValueRange,
) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kMathFunctionClass(
        crate::css_math_function_value::CSSMathFunctionValue::Create(expression, range),
    )))
}

// cpp: css_alternate_value.h:17-43; css_alternate_value.cc:14-38.
pub struct CSSAlternateValue {
    pub function: CSSFunctionValue,
    pub aliases: CSSValueList,
}
impl CSSValueSubclass for CSSAlternateValue {
    fn CustomCSSText(&self) -> String {
        String::from(
            format!(
                "{}({})",
                GetCSSValueName(self.function.function_id),
                self.aliases.CustomCSSText().Utf8()
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.function.Equals(&other.function) && self.aliases.Equals(&other.aliases)
    }
}
impl CSSValueRandom for CSSAlternateValue {
    fn HasRandomFunctions(&self) -> bool {
        self.function.arguments.HasRandomFunctions() || self.aliases.HasRandomFunctions()
    }
}
pub fn alternate(function_id: CSSValueID, aliases: Vec<Rc<Value>>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kAlternateClass(
        CSSAlternateValue {
            function: CSSFunctionValue {
                function_id,
                arguments: CSSValueList::new(Vec::new(), ListSeparator::Comma),
            },
            aliases: CSSValueList::new(aliases, ListSeparator::Comma),
        },
    )))
}
