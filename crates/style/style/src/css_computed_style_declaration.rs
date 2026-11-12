// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/css/css_computed_style_declaration.h:69-81;
// css_computed_style_declaration.cc:62-67,140-175,229-237,332-390,465-482.
//! Synchronous computed declaration over Element's canonical style owner.
//! The document host completes the source lifecycle before lending this borrow;
//! no DOM callbacks, CSS serialization, parser, or copied style is involved.
use foundation::{CSSPropertyID, CSSValueID};
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
use style_model::{
    css_property_name::CSSPropertyName, css_property_value::CSSPropertyValue,
    css_property_value_set::MutableCSSPropertyValueSet,
};
use style_parser::production_css_value::{self as values, ProductionCSSValueDispatch, Value};
pub use style_resolve::css_value_from_computed_style::{
    CSSValueFromComputedStyle, ComputedStyleValueError, ComputedStyleValueOptions,
};
pub type ComputedPropertySet = MutableCSSPropertyValueSet<ProductionCSSValueDispatch>;

#[derive(Clone, Copy)]
pub struct CSSComputedStyleDeclaration<'a> {
    style: Option<&'a ComputedStyle>,
    options: ComputedStyleValueOptions,
}
impl<'a> CSSComputedStyleDeclaration<'a> {
    pub fn new(style: Option<&'a ComputedStyle>, options: ComputedStyleValueOptions) -> Self {
        Self { style, options }
    }
    /// css_computed_style_declaration.cc:103-112: the generated order and
    /// first-use exposure-filtered cache belong to this declaration owner.
    pub fn ComputableProperties() -> Result<&'static [CSSPropertyID], ComputedStyleValueError> {
        static PROPERTIES: std::sync::OnceLock<
            Result<Vec<CSSPropertyID>, ComputedStyleValueError>,
        > = std::sync::OnceLock::new();
        match PROPERTIES.get_or_init(|| {
            style_model::css_property_names::kCSSComputableProperties
                .iter()
                .copied()
                .filter_map(|property| {
                    match style_model::computed_property_exposure::IsComputablePropertyWebExposed(
                        property,
                    ) {
                        Some(true) => Some(Ok(property)),
                        Some(false) => None,
                        None => Some(Err(ComputedStyleValueError::ContextDependentExposure(
                            property,
                        ))),
                    }
                })
                .collect()
        }) {
            Ok(properties) => Ok(properties.as_slice()),
            Err(error) => Err(*error),
        }
    }
    /// Source CopyProperties always queries each real property in source
    /// order. An untranslated value remains a precise typed error rather
    /// than silently becoming a missing property in the returned set.
    pub fn CopyProperties(&self) -> Result<ComputedPropertySet, ComputedStyleValueError> {
        if self.style.is_none() {
            return Ok(ComputedPropertySet::FromProperties(&[]));
        }
        self.CopyPropertiesInSet(Self::ComputableProperties()?)
    }
    pub fn CopyPropertiesInSet(
        &self,
        properties: &[CSSPropertyID],
    ) -> Result<ComputedPropertySet, ComputedStyleValueError> {
        let mut list = Vec::with_capacity(properties.len());
        for &property in properties {
            if let Some(value) = self.GetPropertyCSSValue(property)? {
                list.push(CSSPropertyValue::new(
                    &CSSPropertyName::new(property),
                    value,
                    false,
                    false,
                    0,
                    false,
                ));
            }
        }
        Ok(ComputedPropertySet::FromProperties(&list))
    }
    pub fn GetPropertyCSSValue(
        &self,
        property: CSSPropertyID,
    ) -> Result<Option<Rc<Value>>, ComputedStyleValueError> {
        if property == CSSPropertyID::kVariable {
            return Ok(None);
        }
        let Some(style) = self.style else {
            return Ok(None);
        };
        CSSValueFromComputedStyle(property, style, self.options).map(Some)
    }
    pub fn GetFontSizeCSSValuePreferringKeyword(&self) -> Option<Rc<Value>> {
        let style = self.style?;
        let font = style.GetFontDescription();
        let keyword = match font.KeywordSize() {
            0 => {
                return Some(style_resolve::computed_style_utils::ZoomAdjustedPixelValue(
                    font.ComputedPixelSize() as f64,
                    style,
                ))
            }
            1 => CSSValueID::kXxSmall,
            2 => CSSValueID::kXSmall,
            3 => CSSValueID::kSmall,
            4 => CSSValueID::kMedium,
            5 => CSSValueID::kLarge,
            6 => CSSValueID::kXLarge,
            7 => CSSValueID::kXxLarge,
            8 => CSSValueID::kXxxLarge,
            _ => unreachable!("canonical FontDescription keyword size"),
        };
        Some(values::identifier(keyword))
    }
    pub fn IsMonospaceFont(&self) -> bool {
        self.style
            .is_some_and(|style| style.GetFontDescription().IsMonospace())
    }
}

/// Generated CSS property Exposure projection for the editing property cache.
/// `known_exposed` is Chromium's explicit unconditional-web bit, not IsInternal
/// or a local editing allowlist. Context-dependent properties require their
/// real ExecutionContext exposure owner instead of guessing false.
/// Source: build/scripts/core/css/css_properties.py:160-165;
/// make_css_property_names.py:87-93; css_properties.cc.tmpl:51-67.
pub fn IsEditingPropertyWebExposed(
    property: CSSPropertyID,
) -> Result<bool, ComputedStyleValueError> {
    use style_model::css_property_names::{
        kNotKnownExposedPropertyBit, FindProperty, GetPropertyName,
    };
    let entry = FindProperty(GetPropertyName(property).as_bytes())
        .ok_or(ComputedStyleValueError::ContextDependentExposure(property))?;
    if entry.id_and_exposed_bit & kNotKnownExposedPropertyBit != 0 {
        return Err(ComputedStyleValueError::ContextDependentExposure(property));
    }
    Ok(true)
}
