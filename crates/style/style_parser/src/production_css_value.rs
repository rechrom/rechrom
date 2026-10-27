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
    AddFloatToHash, AddIntToHash, CSSPropertyID, CSSValueID, Color, ColorSpace, HashInt, HashInts,
    HueInterpolationMethod, Length, LengthType, String,
};
use std::rc::Rc;
use style_model::css_property_value_set::{
    CSSPropertyValueSet, CSSPropertyValueSetBackend, MutableCSSPropertyValueSet,
};

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

pub enum Unavailable {}

// The parsed and native computed image values share one typed URL owner.
pub use layoutng_style::css::css_url_data::{CSSUrlData, ProductionURL};

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
        crate::media_query_exp::MediaQueryExpSerialization::CssText(self)
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
        let component = |value: f32, none: bool| {
            if none {
                "none".to_owned()
            } else {
                format!("{value}")
            }
        };
        let p0 = component(self.0.Param0(), self.0.Param0IsNone());
        let p1 = component(self.0.Param1(), self.0.Param1IsNone());
        let p2 = component(self.0.Param2(), self.0.Param2IsNone());
        let percent1 = if self.0.Param1IsNone() {
            "none".to_owned()
        } else {
            format!("{}%", self.0.Param1() * 100.0)
        };
        let percent2 = if self.0.Param2IsNone() {
            "none".to_owned()
        } else {
            format!("{}%", self.0.Param2() * 100.0)
        };
        let alpha = if self.0.AlphaIsNone() {
            " / none".to_owned()
        } else if self.0.IsOpaque() {
            std::string::String::new()
        } else {
            format!(" / {}", self.0.Alpha())
        };
        let text = match self.0.GetColorSpace() {
            ColorSpace::kSRGBLegacy => {
                let r = self.0.Param0().round() as u8;
                let g = self.0.Param1().round() as u8;
                let b = self.0.Param2().round() as u8;
                if self.0.IsOpaque() {
                    format!("rgb({r}, {g}, {b})")
                } else {
                    format!("rgba({r}, {g}, {b}, {})", self.0.Alpha())
                }
            }
            ColorSpace::kHSL => format!("hsl({p0} {percent1} {percent2}{alpha})"),
            ColorSpace::kHWB => format!("hwb({p0} {percent1} {percent2}{alpha})"),
            ColorSpace::kLab => format!("lab({p0} {p1} {p2}{alpha})"),
            ColorSpace::kLch => format!("lch({p0} {p1} {p2}{alpha})"),
            ColorSpace::kOklab => format!("oklab({p0} {p1} {p2}{alpha})"),
            ColorSpace::kOklch => format!("oklch({p0} {p1} {p2}{alpha})"),
            space => {
                let name = match space {
                    ColorSpace::kSRGB => "srgb",
                    ColorSpace::kSRGBLinear => "srgb-linear",
                    ColorSpace::kDisplayP3 => "display-p3",
                    ColorSpace::kDisplayP3Linear => "display-p3-linear",
                    ColorSpace::kA98RGB => "a98-rgb",
                    ColorSpace::kProPhotoRGB => "prophoto-rgb",
                    ColorSpace::kRec2020 => "rec2020",
                    ColorSpace::kRec2100Linear => "rec2100-linear",
                    ColorSpace::kXYZD50 => "xyz-d50",
                    ColorSpace::kXYZD65 => "xyz-d65",
                    ColorSpace::kNone => "srgb",
                    _ => unreachable!(),
                };
                format!("color({name} {p0} {p1} {p2}{alpha})")
            }
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

// cpp: css_light_dark_value_pair.h:16-31;
// css_light_dark_value_pair.cc:12-19. The pair deliberately remains a
// distinct CSSValue class even though its storage is CSSValuePair: resolver
// state must choose a branch from the element's used color scheme before
// ordinary color conversion.
pub struct CSSLightDarkValuePair {
    pair: CSSValuePair,
}

// cpp: css_color_mix_value.h:20-78; css_color_mix_value.cc:14-67.
// Percentages remain typed CSS values so normalization happens at computed
// value time. The production parser currently admits the default Oklab family
// used by html.css; explicit interpolation spaces remain an honest parser
// boundary until their native Color conversions are retained.
pub struct CSSColorMixValue {
    pub color1: Rc<Value>,
    pub color2: Rc<Value>,
    pub percentage1: Option<Rc<Value>>,
    pub percentage2: Option<Rc<Value>>,
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
}

impl CSSColorMixValue {
    fn Percentage(value: Option<&Rc<Value>>) -> Option<f64> {
        let CSSValuePayload::kNumericLiteralClass(value) = value?.Payload() else {
            return None;
        };
        (value.GetType() == UnitType::kPercentage).then(|| value.DoubleValue())
    }

    // cpp: CSSColorMixValue::NormalizePercentages. Parser-owned percentages in
    // this family are literals, so the length resolver is not needed here.
    pub fn NormalizePercentages(&self) -> Option<(f64, f64)> {
        let first = Self::Percentage(self.percentage1.as_ref());
        let second = Self::Percentage(self.percentage2.as_ref());
        let mut p1 = first.unwrap_or(50.0).clamp(0.0, 100.0) / 100.0;
        let mut p2 = second.unwrap_or(50.0).clamp(0.0, 100.0) / 100.0;
        if first.is_some() && second.is_none() {
            p2 = 1.0 - p1;
        } else if first.is_none() && second.is_some() {
            p1 = 1.0 - p2;
        }
        if p1 == 0.0 && p2 == 0.0 {
            return None;
        }
        let scale = p1 + p2;
        p1 /= scale;
        p2 /= scale;
        Some((if p1 == 0.0 { 1.0 } else { p2 }, scale.min(1.0)))
    }

    // cpp: CSSColorMixValue::PercentageValuesForSerialization.
    fn PercentageValuesForSerialization(&self) -> (Option<Rc<Value>>, Option<Rc<Value>>) {
        let first = Self::Percentage(self.percentage1.as_ref());
        let second = Self::Percentage(self.percentage2.as_ref());
        if first == Some(50.0) && (second.is_none() || second == Some(50.0)) {
            return (None, None);
        }
        if let (Some(first), None) = (first, second) {
            return (
                self.percentage1.clone(),
                Some(numeric(100.0 - first, UnitType::kPercentage)),
            );
        }
        if let (None, Some(second)) = (first, second) {
            if second == 50.0 {
                return (None, None);
            }
            return (
                Some(numeric(100.0 - second, UnitType::kPercentage)),
                self.percentage2.clone(),
            );
        }
        (self.percentage1.clone(), self.percentage2.clone())
    }
}

impl CSSValueSubclass for CSSColorMixValue {
    fn CustomCSSText(&self) -> String {
        let (percentage1, percentage2) = self.PercentageValuesForSerialization();
        let mut text = String::from("color-mix(");
        text.push_str(&self.color1.CssText().Utf8());
        if let Some(value) = percentage1 {
            text.push_str(" ");
            text.push_str(&value.CssText().Utf8());
        }
        text.push_str(", ");
        text.push_str(&self.color2.CssText().Utf8());
        if let Some(value) = percentage2 {
            text.push_str(" ");
            text.push_str(&value.CssText().Utf8());
        }
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.color1.as_ref() == other.color1.as_ref()
            && self.color2.as_ref() == other.color2.as_ref()
            && self.percentage1.as_deref() == other.percentage1.as_deref()
            && self.percentage2.as_deref() == other.percentage2.as_deref()
            && self.color_interpolation_space == other.color_interpolation_space
            && self.hue_interpolation_method == other.hue_interpolation_method
    }
}

impl CSSValueRandom for CSSColorMixValue {
    fn HasRandomFunctions(&self) -> bool {
        self.color1.HasRandomFunctions()
            || self.color2.HasRandomFunctions()
            || self
                .percentage1
                .as_ref()
                .is_some_and(|value| value.HasRandomFunctions())
            || self
                .percentage2
                .as_ref()
                .is_some_and(|value| value.HasRandomFunctions())
    }
}

pub fn color_mix(
    color1: Rc<Value>,
    color2: Rc<Value>,
    percentage1: Option<Rc<Value>>,
    percentage2: Option<Rc<Value>>,
) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kColorMixClass(
        CSSColorMixValue {
            color1,
            color2,
            percentage1,
            percentage2,
            color_interpolation_space: ColorSpace::kOklab,
            hue_interpolation_method: HueInterpolationMethod::kShorter,
        },
    )))
}

impl CSSValueSubclass for CSSLightDarkValuePair {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from("light-dark(");
        text.push_str(&self.pair.first.CssText().Utf8());
        text.push_str(", ");
        text.push_str(&self.pair.second.CssText().Utf8());
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.pair.Equals(&other.pair)
    }
}

impl CSSValuePairSubclass<ProductionCSSValueDispatch> for CSSLightDarkValuePair {
    fn AsValuePair(&self) -> &CSSValuePair {
        &self.pair
    }
}

pub fn light_dark(first: Rc<Value>, second: Rc<Value>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kLightDarkValuePairClass(
        CSSLightDarkValuePair {
            pair: CSSValuePair {
                first,
                second,
                drop_identical: false,
            },
        },
    )))
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
    pub url_data: CSSUrlData,
}
impl CSSValueSubclass for CSSImageValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from("url(");
        text.push_string(&crate::css_markup::SerializeString(
            self.url_data.ValueForSerialization(),
        ));
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.url_data == other.url_data
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
    image_from_url_data(CSSUrlData::unresolved(url))
}
pub fn image_from_url_data(url_data: CSSUrlData) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kImageClass(CSSImageValue {
        url: url_data.resolved_url.clone(),
        url_data,
    })))
}

// cpp: css_image_set_option_value.h / css_image_set_value.h. Keep the
// authored candidates immutable; computed-style application selects one for
// the current device scale and carries its resolution on StyleFetchedImage.
pub struct CSSImageSetOptionValue {
    pub image: Rc<Value>,
    pub resolution: f32,
}
impl CSSValueSubclass for CSSImageSetOptionValue {
    fn CustomCSSText(&self) -> String {
        String::from(format!("{} {}x", self.image.CssText().Utf8(), self.resolution).as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self.image == other.image && self.resolution.to_bits() == other.resolution.to_bits()
    }
}
impl CSSValueRandom for CSSImageSetOptionValue {
    fn HasRandomFunctions(&self) -> bool {
        self.image.HasRandomFunctions()
    }
}

pub struct CSSImageSetValue {
    pub list: CSSValueList,
}
impl CSSValueSubclass for CSSImageSetValue {
    fn CustomCSSText(&self) -> String {
        String::from(format!("image-set({})", self.list.CustomCSSText().Utf8()).as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self.list.Equals(&other.list)
    }
}
impl CSSValueRandom for CSSImageSetValue {
    fn HasRandomFunctions(&self) -> bool {
        self.list.HasRandomFunctions()
    }
}
impl CSSValueSubresources for CSSImageSetValue {
    fn HasFailedOrCanceledSubresources(&self) -> bool {
        self.list.HasFailedOrCanceledSubresources()
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for CSSImageSetValue {
    fn AsValueList(&self) -> &CSSValueList {
        &self.list
    }
}
pub fn image_set_option(image: Rc<Value>, resolution: f32) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kImageSetOptionClass(
        CSSImageSetOptionValue { image, resolution },
    )))
}
pub fn image_set(options: Vec<Rc<Value>>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kImageSetClass(
        CSSImageSetValue {
            list: CSSValueList::new(options, ListSeparator::Comma),
        },
    )))
}
// cpp: css_uri_value.h:23-60. This URL retains parsed identity until the
// document supplies its real SVG resource binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSURIValue {
    pub url: String,
    pub url_data: CSSUrlData,
}
impl CSSValueSubclass for CSSURIValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::from("url(");
        text.push_string(&crate::css_markup::SerializeString(
            self.url_data.ValueForSerialization(),
        ));
        text.push_str(")");
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.url_data == other.url_data
    }
}
impl CSSValueUrl<()> for CSSURIValue {
    fn ReResolveUrl(&self, _: &()) {
        panic!("CSSURIValue::ReResolveURL requires application Document/URL binding")
    }
}
pub fn uri(url: String) -> Rc<Value> {
    uri_from_url_data(CSSUrlData::unresolved(url))
}
pub fn uri_from_url_data(url_data: CSSUrlData) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kURIClass(CSSURIValue {
        url: url_data.resolved_url.clone(),
        url_data,
    })))
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
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
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
        let mut interpolation = Vec::new();
        AppendGradientInterpolationMethod(
            &mut interpolation,
            self.color_interpolation_space,
            self.hue_interpolation_method,
        );
        if let Some(interpolation) = interpolation.pop() {
            if let Some(geometry) = items.last_mut() {
                geometry.push(' ');
                geometry.push_str(&interpolation);
            } else {
                items.push(interpolation);
            }
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
            && self.color_interpolation_space == other.color_interpolation_space
            && self.hue_interpolation_method == other.hue_interpolation_method
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSRadialGradientShape {
    Circle,
    Ellipse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSRadialGradientExtent {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
}

pub struct CSSRadialGradientValue {
    pub shape: Option<CSSRadialGradientShape>,
    pub extent: Option<CSSRadialGradientExtent>,
    pub size_x: Option<Rc<Value>>,
    pub size_y: Option<Rc<Value>>,
    pub position_x: Rc<Value>,
    pub position_y: Rc<Value>,
    pub repeating: bool,
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
    pub stops: Vec<CSSGradientColorStop>,
}
impl CSSValueSubclass for CSSRadialGradientValue {
    fn CustomCSSText(&self) -> String {
        let mut prefix = Vec::new();
        if let Some(shape) = self.shape {
            prefix.push(match shape {
                CSSRadialGradientShape::Circle => "circle".to_owned(),
                CSSRadialGradientShape::Ellipse => "ellipse".to_owned(),
            });
        }
        if let Some(extent) = self.extent {
            prefix.push(
                match extent {
                    CSSRadialGradientExtent::ClosestSide => "closest-side",
                    CSSRadialGradientExtent::ClosestCorner => "closest-corner",
                    CSSRadialGradientExtent::FarthestSide => "farthest-side",
                    CSSRadialGradientExtent::FarthestCorner => "farthest-corner",
                }
                .to_owned(),
            );
        } else {
            for size in [&self.size_x, &self.size_y].into_iter().flatten() {
                prefix.push(size.CssText().Utf8());
            }
        }
        prefix.push(format!(
            "at {} {}",
            self.position_x.CssText().Utf8(),
            self.position_y.CssText().Utf8()
        ));
        AppendGradientInterpolationMethod(
            &mut prefix,
            self.color_interpolation_space,
            self.hue_interpolation_method,
        );
        let mut items = vec![prefix.join(" ")];
        AppendGradientStops(&mut items, &self.stops);
        String::from(
            format!(
                "{}radial-gradient({})",
                if self.repeating { "repeating-" } else { "" },
                items.join(", ")
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.shape == other.shape
            && self.extent == other.extent
            && self.size_x == other.size_x
            && self.size_y == other.size_y
            && self.position_x == other.position_x
            && self.position_y == other.position_y
            && self.repeating == other.repeating
            && self.color_interpolation_space == other.color_interpolation_space
            && self.hue_interpolation_method == other.hue_interpolation_method
            && self.stops == other.stops
    }
}
impl CSSValueRandom for CSSRadialGradientValue {
    fn HasRandomFunctions(&self) -> bool {
        [&self.size_x, &self.size_y]
            .into_iter()
            .flatten()
            .any(|value| value.HasRandomFunctions())
            || self.position_x.HasRandomFunctions()
            || self.position_y.HasRandomFunctions()
            || GradientStopsHaveRandom(&self.stops)
    }
}

pub struct CSSConicGradientValue {
    pub angle: Option<Rc<Value>>,
    pub position_x: Rc<Value>,
    pub position_y: Rc<Value>,
    pub repeating: bool,
    pub color_interpolation_space: ColorSpace,
    pub hue_interpolation_method: HueInterpolationMethod,
    pub stops: Vec<CSSGradientColorStop>,
}
impl CSSValueSubclass for CSSConicGradientValue {
    fn CustomCSSText(&self) -> String {
        let mut prefix = Vec::new();
        if let Some(angle) = &self.angle {
            prefix.push(format!("from {}", angle.CssText().Utf8()));
        }
        prefix.push(format!(
            "at {} {}",
            self.position_x.CssText().Utf8(),
            self.position_y.CssText().Utf8()
        ));
        AppendGradientInterpolationMethod(
            &mut prefix,
            self.color_interpolation_space,
            self.hue_interpolation_method,
        );
        let mut items = vec![prefix.join(" ")];
        AppendGradientStops(&mut items, &self.stops);
        String::from(
            format!(
                "{}conic-gradient({})",
                if self.repeating { "repeating-" } else { "" },
                items.join(", ")
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.angle == other.angle
            && self.position_x == other.position_x
            && self.position_y == other.position_y
            && self.repeating == other.repeating
            && self.color_interpolation_space == other.color_interpolation_space
            && self.hue_interpolation_method == other.hue_interpolation_method
            && self.stops == other.stops
    }
}
impl CSSValueRandom for CSSConicGradientValue {
    fn HasRandomFunctions(&self) -> bool {
        self.angle
            .as_ref()
            .is_some_and(|value| value.HasRandomFunctions())
            || self.position_x.HasRandomFunctions()
            || self.position_y.HasRandomFunctions()
            || GradientStopsHaveRandom(&self.stops)
    }
}

fn AppendGradientStops(items: &mut Vec<std::string::String>, stops: &[CSSGradientColorStop]) {
    for stop in stops {
        let mut pieces = Vec::new();
        if let Some(color) = &stop.color {
            pieces.push(color.CssText().Utf8());
        }
        if let Some(offset) = &stop.offset {
            pieces.push(offset.CssText().Utf8());
        }
        items.push(pieces.join(" "));
    }
}

fn AppendGradientInterpolationMethod(
    items: &mut Vec<std::string::String>,
    space: ColorSpace,
    hue: HueInterpolationMethod,
) {
    if space == ColorSpace::kNone {
        return;
    }
    let mut text = format!(
        "in {}",
        match space {
            ColorSpace::kSRGB => "srgb",
            ColorSpace::kSRGBLinear => "srgb-linear",
            ColorSpace::kDisplayP3 => "display-p3",
            ColorSpace::kHSL => "hsl",
            ColorSpace::kHWB => "hwb",
            ColorSpace::kA98RGB => "a98-rgb",
            ColorSpace::kProPhotoRGB => "prophoto-rgb",
            ColorSpace::kRec2020 => "rec2020",
            ColorSpace::kLab => "lab",
            ColorSpace::kLch => "lch",
            ColorSpace::kOklab => "oklab",
            ColorSpace::kOklch => "oklch",
            ColorSpace::kXYZD50 => "xyz-d50",
            ColorSpace::kXYZD65 => "xyz-d65",
            _ => return,
        }
    );
    if Color::ColorSpaceHasHue(space) && hue != HueInterpolationMethod::kShorter {
        text.push(' ');
        text.push_str(match hue {
            HueInterpolationMethod::kShorter => "shorter hue",
            HueInterpolationMethod::kLonger => "longer hue",
            HueInterpolationMethod::kIncreasing => "increasing hue",
            HueInterpolationMethod::kDecreasing => "decreasing hue",
        });
    }
    items.push(text);
}

fn GradientStopsHaveRandom(stops: &[CSSGradientColorStop]) -> bool {
    stops.iter().any(|stop| {
        [&stop.color, &stop.offset]
            .into_iter()
            .flatten()
            .any(|value| value.HasRandomFunctions())
    })
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
    Url(CSSUrlData),
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
            FontFaceResource::Url(url) => ("url", url.ValueForSerialization()),
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

// cpp: css_initial_color_value.h:17-36;
// css_initial_color_value.cc:12-18. This internal value is inserted into the
// document element's MatchResult at the lowest UA priority. It is distinct
// from the CSS-wide `initial` keyword because Color::ApplyValue resolves it
// only after color-scheme has established the builder's used scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSInitialColorValue;
impl CSSValueSubclass for CSSInitialColorValue {
    fn CustomCSSText(&self) -> String {
        String::default()
    }
    fn Equals(&self, _: &Self) -> bool {
        true
    }
}

// cpp: css_pending_system_font_value.h:26-50; .cc:14-36. The font shorthand
// expands one retained marker to every font longhand, deferring the platform
// family/size lookup until the document resolver applies each property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSPendingSystemFontValue {
    pub system_font_id: CSSValueID,
}
impl CSSValueSubclass for CSSPendingSystemFontValue {
    fn CustomCSSText(&self) -> String {
        String::default()
    }
    fn Equals(&self, other: &Self) -> bool {
        self.system_font_id == other.system_font_id
    }
}

// cpp: css_variable_data.h:31-169; css_variable_data.cc:24-117,158-164.
// Retained tokenizer output is the Rust ownership equivalent of OriginalText:
// substitution copies tokens, preserving token boundaries without re-tokenizing.
#[derive(Clone, Debug)]
pub struct VariableToken {
    pub token: crate::css_parser_token::CSSParserToken,
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
    pub fn ExtractFeatures(token: &crate::css_parser_token::CSSParserToken) -> u8 {
        use crate::css_parser_token::CSSParserTokenType::*;
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
        use crate::css_parser_token::CSSParserTokenType::*;
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
    pub mode: crate::css_parser_mode::CSSParserMode,
    pub base_url: ProductionURL,
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
pub fn unparsed(data: CSSVariableData, mode: crate::css_parser_mode::CSSParserMode) -> Rc<Value> {
    unparsed_with_base(data, mode, ProductionURL::Null)
}
pub fn unparsed_with_base(
    data: CSSVariableData,
    mode: crate::css_parser_mode::CSSParserMode,
    base_url: ProductionURL,
) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kUnparsedDeclarationClass(
        CSSUnparsedDeclarationValue {
            data: Rc::new(data),
            mode,
            base_url,
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
    type CSSColorMixValue = CSSColorMixValue;
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
    type CSSLightDarkValuePair = CSSLightDarkValuePair;
    type CSSParamValuePair = UnavailableCSSValue;
    type CSSScrollValue = UnavailableCSSValue;
    type CSSViewValue = UnavailableCSSValue;
    type CSSRatioValue = CSSRatioValue;
    type CSSRelativeColorValue = UnavailableCSSValue;
    type CSSBasicShapeCircleValue = crate::production_effects_value::CSSBasicShapeCircleValue;
    type CSSBasicShapeEllipseValue = crate::production_effects_value::CSSBasicShapeEllipseValue;
    type CSSBasicShapePolygonValue = crate::production_effects_value::CSSBasicShapePolygonValue;
    type CSSBasicShapeInsetValue = crate::production_effects_value::CSSBasicShapeInsetValue;
    type CSSBasicShapeRectValue = crate::production_effects_value::CSSBasicShapeRectValue;
    type CSSBasicShapeXYWHValue = UnavailableCSSValue;
    type CSSPathValue = crate::production_effects_value::CSSPathValue;
    type CSSShapeValue = UnavailableCSSValue;
    type CSSImageValue = CSSImageValue;
    type CSSCursorImageValue = UnavailableCSSValue;
    type CSSCrossfadeValue = UnavailableCSSValue;
    type CSSPaintValue = UnavailableCSSValue;
    type CSSLinearGradientValue = CSSLinearGradientValue;
    type CSSRadialGradientValue = CSSRadialGradientValue;
    type CSSConicGradientValue = CSSConicGradientValue;
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
    type CSSPendingSystemFontValue = CSSPendingSystemFontValue;
    type CSSInvalidVariableValue = UnavailableCSSValue;
    type CSSCyclicVariableValue = UnavailableCSSValue;
    type CSSFlipRevertValue = UnavailableCSSValue;
    type CSSLayoutFunctionValue = UnavailableCSSValue;
    type CSSContentDistributionValue = CSSContentDistributionValue;
    type CSSKeyframeShorthandValue = UnavailableCSSValue;
    type CSSInitialColorValue = CSSInitialColorValue;
    type CSSImageSetOptionValue = CSSImageSetOptionValue;
    type CSSImageSetTypeValue = UnavailableCSSValue;
    type CSSRepeatStyleValue = CSSRepeatStyleValue;
    type CSSSuperellipseValue = crate::production_corner_value::CSSSuperellipseValue;
    type CSSSymbolsValue = UnavailableCSSValue;
    type CSSTriggerAttachmentValue = CSSTriggerAttachmentValue;
    type CSSRepeatValue = UnavailableCSSValue;
    type CSSValueList = CSSValueList;
    type CSSFunctionValue = CSSFunctionValue;
    type CSSImageSetValue = CSSImageSetValue;
    type CSSBracketedValueList = CSSBracketedValueList;
    type CSSGridAutoRepeatValue = CSSGridAutoRepeatValue;
    type CSSGridIntegerRepeatValue = CSSGridIntegerRepeatValue;
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
    // cpp: CSSPrimitiveValue::CreateFromLength / CSSMathFunctionValue::Create.
    fn CreatePrimitiveFromLength(length: &Length, zoom: f32) -> Rc<Value> {
        if length.IsFixed() {
            numeric((length.Pixels() / zoom) as f64, UnitType::kPixels)
        } else if length.IsPercent() {
            numeric(length.PercentValue() as f64, UnitType::kPercentage)
        } else if length.IsCalculated() {
            let value = length.GetCalculationValue();
            // CSSPrimitiveValue::CreateFromLength collapses a non-expression
            // calculation with zero pixels to its (range-clamped) percentage.
            if !value.IsExpression() && value.Pixels() == 0.0 {
                let percent = value.Percent();
                return numeric(
                    if value.IsNonNegative() {
                        percent.max(0.0)
                    } else {
                        percent
                    } as f64,
                    UnitType::kPercentage,
                );
            }
            let expression =
                crate::css_math_expression_node::CSSMathExpressionNode::FromCalculationValue(
                    &value, zoom,
                )
                .expect("computed length must have a serializable calculation");
            math(
                expression,
                if value.IsNonNegative() {
                    crate::css_math_function_value::ValueRange::NonNegative
                } else {
                    crate::css_math_function_value::ValueRange::All
                },
            )
        } else if length.IsFlex() {
            numeric(length.FlexValue() as f64, UnitType::kFlex)
        } else {
            unreachable!("CSSPrimitiveValue::CreateFromLength precondition")
        }
    }
}
impl CSSPropertyValueBackend for ProductionCSSValueDispatch {
    fn MatchingShorthandsForLonghand(id: CSSPropertyID) -> Vec<CSSPropertyID> {
        crate::production_property_metadata::MatchingShorthands(id)
    }
    // cpp: css_property.h:120-122; direction/unicode-bidi source overrides.
    fn IsAffectedByAll(id: CSSPropertyID) -> bool {
        let property = crate::properties::css_property::CSSProperty::Get(id);
        !property.IsInternal()
            && property.IsProperty()
            && !crate::production_property_metadata::ExcludedFromAll(id)
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
pub fn initial_color() -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kInitialColorValueClass(
        CSSInitialColorValue,
    )))
}
pub fn pending_system_font(system_font_id: CSSValueID) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kPendingSystemFontValueClass(
        CSSPendingSystemFontValue { system_font_id },
    )))
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

// cpp: css_bracketed_value_list.h:13-28. Grid line names are a distinct
// CSSValue subclass even though their children use the ordinary space-list
// storage and iteration contract.
pub struct CSSBracketedValueList {
    pub list: CSSValueList,
}
impl CSSValueSubclass for CSSBracketedValueList {
    fn CustomCSSText(&self) -> String {
        String::from(format!("[{}]", self.list.CustomCSSText().Utf8()).as_str())
    }
    fn Equals(&self, other: &Self) -> bool {
        self.list.Equals(&other.list)
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for CSSBracketedValueList {
    fn AsValueList(&self) -> &CSSValueList {
        &self.list
    }
}
pub fn grid_line_names(values: Vec<Rc<Value>>) -> Rc<Value> {
    Rc::new(Value::new(CSSValuePayload::kGridLineNamesClass(
        CSSBracketedValueList {
            list: CSSValueList::new(values, ListSeparator::Space),
        },
    )))
}

// cpp: css_grid_auto_repeat_value.{h,cc}. Auto-repeat retains its keyword
// independently from the repeated typed entries so computed conversion can
// select the native auto-fill/auto-fit repeater without inspecting CSS text.
pub struct CSSGridAutoRepeatValue {
    pub kind: CSSValueID,
    pub list: CSSValueList,
}
impl CSSValueSubclass for CSSGridAutoRepeatValue {
    fn CustomCSSText(&self) -> String {
        String::from(
            format!(
                "repeat({}, {})",
                GetCSSValueName(self.kind),
                self.list.CustomCSSText().Utf8()
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.kind == other.kind && self.list.Equals(&other.list)
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for CSSGridAutoRepeatValue {
    fn AsValueList(&self) -> &CSSValueList {
        &self.list
    }
}

// cpp: css_grid_integer_repeat_value.{h,cc}. Keep the count expression typed
// until computed-value conversion, and retain the parser's track-count clamp.
pub struct CSSGridIntegerRepeatValue {
    pub repetitions: Rc<Value>,
    pub extra_clamp: u32,
    pub list: CSSValueList,
}
impl CSSGridIntegerRepeatValue {
    pub fn ClampRepetitions(&self, repetitions: u32) -> u32 {
        repetitions
            .min(layoutng_style::style::grid_area::K_GRID_MAX_TRACKS as u32)
            .min(self.extra_clamp)
    }
}
impl CSSValueSubclass for CSSGridIntegerRepeatValue {
    fn CustomCSSText(&self) -> String {
        String::from(
            format!(
                "repeat({}, {})",
                self.repetitions.CssText().Utf8(),
                self.list.CustomCSSText().Utf8()
            )
            .as_str(),
        )
    }
    fn Equals(&self, other: &Self) -> bool {
        self.repetitions == other.repetitions && self.list.Equals(&other.list)
    }
}
impl CSSValueListSubclass<ProductionCSSValueDispatch> for CSSGridIntegerRepeatValue {
    fn AsValueList(&self) -> &CSSValueList {
        &self.list
    }
}
impl CSSValueRandom for CSSGridIntegerRepeatValue {
    fn HasRandomFunctions(&self) -> bool {
        self.repetitions.HasRandomFunctions()
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
    crate::production_property_metadata::NamedColor(GetCSSValueName(id))
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

impl CSSPropertyValueSetBackend for ProductionCSSValueDispatch {
    type CSSStyleDeclaration = Unavailable;
    type ExecutionContext = Unavailable;

    fn ShorthandForProperty(id: CSSPropertyID) -> Vec<CSSPropertyID> {
        style_model::property_metadata::ShorthandFor(id).to_vec()
    }

    fn IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
        a: CSSPropertyID,
        b: CSSPropertyID,
    ) -> bool {
        style_model::properties::css_property::CSSProperty::Get(a)
            .IsInSameLogicalPropertyGroupWithDifferentMappingLogic(b)
    }

    fn SerializeShorthand(_set: &CSSPropertyValueSet<Self>, _id: CSSPropertyID) -> String {
        panic!("production CSS declaration serialization is unavailable")
    }

    fn AsText(_set: &CSSPropertyValueSet<Self>) -> String {
        panic!("production CSS declaration text serialization is unavailable")
    }

    fn CreateIdentifier(id: CSSValueID) -> Rc<CSSValue<Self>> {
        identifier(id)
    }

    fn DeclarationPropertyValueSet(
        _style: &Self::CSSStyleDeclaration,
    ) -> Option<&CSSPropertyValueSet<Self>> {
        panic!("production CSS declaration property access is unavailable")
    }

    fn DeclarationPropertyMatches(
        _style: &Self::CSSStyleDeclaration,
        _id: CSSPropertyID,
        _value: &CSSValue<Self>,
    ) -> bool {
        panic!("production CSS declaration matching is unavailable")
    }

    fn NewCSSStyleDeclaration(
        _context: Option<&Self::ExecutionContext>,
        _set: &MutableCSSPropertyValueSet<Self>,
    ) -> Rc<Self::CSSStyleDeclaration> {
        panic!("production CSS declaration construction is unavailable")
    }
}

impl crate::media_query_exp::MediaQueryExpSerialization for Value {
    fn CssText(&self) -> String {
        self.CssText()
    }
}

impl crate::media_query_exp::MediaQueryExpSerialization for CSSUnparsedDeclarationValue {
    fn CssText(&self) -> String {
        self.CustomCSSText()
    }
}

impl crate::media_query_exp::MediaQueryExpUnitFlags for Value {
    fn GetUnitFlags(&self) -> u32 {
        use crate::media_query_exp::MediaQueryExpUnitFlags;
        match self.Payload() {
            CSSValuePayload::kNumericLiteralClass(value) => value.GetUnitFlags(),
            CSSValuePayload::kMathFunctionClass(value) => value.GetUnitFlags(),
            _ => 0,
        }
    }
}
