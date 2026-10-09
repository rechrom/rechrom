//! Translation of Chromium resolver/font_style_resolver.{h,cc}.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! font_style_resolver.h: 34 / 5 / 3 / 2 / 0.
//! font_style_resolver.cc: 134 / 75 / 75 / 0 / 0.
//! Effective excludes copyright/comments, blanks, preprocessing/includes,
//! namespaces and pure bracket/punctuation lines. Header omissions are
//! STATIC_ONLY 23 and access label 25. All other declarations and cc:22-132
//! production map: element-dependent calc and oblique-list rejection, native
//! conversion-data initialization, source-order size/family/stretch/style/caps/
//! weight handling and final actual FontBuilder.UpdateFontDescription.
//! FontDescription/Font/FontBuilder and CSSPropertyValueSet/CSSValue are reused.
//! CSSToLengthConversionData and StyleBuilderConverterBase belong to other source
//! classes; their native construction/read/conversion calls are required typed
//! operations without defaults. No second font or resolver state is stored.
#![allow(non_snake_case)]
use super::font_builder::{FontBuilder, FontBuilderBackend};
use crate::css_property_value_set::{CSSPropertyValueSet, CSSPropertyValueSetBackend};
use crate::css_value::{CSSValue, CSSValueDispatch, CSSValuePayload};
use font_engine::fonts::font::FontSelector;
use font_engine::fonts::font_description::FontVariantCaps;
use font_engine::{Font, FontDescription, FontSelectionValue};
use foundation::{CSSPropertyID, CSSValueID, WritingMode};
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

type FontValue<B> = CSSValue<<B as FontStyleResolverBackend>::ValueDispatch>;

/// Native conversion-data constructor, actual CSSValue subclass getters and
/// StyleBuilderConverterBase methods. No font description/model is supplied.
pub trait FontStyleResolverBackend: FontBuilderBackend + Sized {
    type ValueDispatch: CSSPropertyValueSetBackend;
    type LengthConversionData;
    fn MathIsElementDependent(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSMathFunctionValue,
    ) -> bool;
    fn FontStyleRangeObliqueValues(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSFontStyleRangeValue,
    ) -> Option<Rc<<Self::ValueDispatch as CSSValueDispatch>::CSSValueList>>;
    fn FontListValues(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSValueList,
    ) -> Vec<Rc<FontValue<Self>>>;
    fn FontIdentifierID(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSIdentifierValue,
    ) -> CSSValueID;
    /// Constructs the actual CSSToLengthConversionData with default line-height,
    /// container sizes and anchor data, no element, and shared ignored flags.
    fn NewFontStyleConversionData(
        &self,
        font: &Font,
        em: f32,
        rem: f32,
        font_zoom: f32,
        viewport_width: f32,
        viewport_height: f32,
        writing_mode: WritingMode,
        zoom: f32,
        ignored_flags: Rc<Cell<u32>>,
    ) -> Self::LengthConversionData;
    fn ConvertFontSize(
        &self,
        value: &FontValue<Self>,
        data: &Self::LengthConversionData,
        parent_size: &Self::Size,
        document: Option<&Self::Document>,
    ) -> Self::Size;
    fn ConvertFontFamily(
        &self,
        value: &FontValue<Self>,
        builder: Option<&FontBuilder<Self>>,
        document: Option<&Self::Document>,
    ) -> Self::FamilyDescription;
    fn ConvertFontStretch(
        &self,
        data: &Self::LengthConversionData,
        value: &FontValue<Self>,
    ) -> FontSelectionValue;
    fn ConvertFontStyle(
        &self,
        data: &Self::LengthConversionData,
        value: &FontValue<Self>,
    ) -> FontSelectionValue;
    fn ConvertFontVariantCaps(&self, value: &FontValue<Self>) -> FontVariantCaps;
    fn ConvertFontWeight(
        &self,
        data: &Self::LengthConversionData,
        value: &FontValue<Self>,
        parent_weight: FontSelectionValue,
    ) -> FontSelectionValue;
}

pub struct FontStyleResolver<B: FontStyleResolverBackend>(PhantomData<fn() -> B>);
impl<B: FontStyleResolverBackend> FontStyleResolver<B> {
    fn ContainsElementDependentCalc(backend: &B, value: Option<&FontValue<B>>) -> bool {
        let Some(value) = value else {
            return false;
        };
        if value.IsPrimitiveValue() {
            // CSSPrimitiveValue::IsCalculated is exactly IsMathFunctionValue.
            return match value.Payload() {
                CSSValuePayload::kMathFunctionClass(math) => backend.MathIsElementDependent(math),
                _ => false,
            };
        }
        if let CSSValuePayload::kFontStyleRangeClass(range) = value.Payload() {
            if let Some(oblique) = backend.FontStyleRangeObliqueValues(range) {
                for value in backend.FontListValues(&oblique) {
                    if let CSSValuePayload::kMathFunctionClass(math) = value.Payload() {
                        if backend.MathIsElementDependent(math) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    pub fn ComputeFont(
        backend: Rc<B>,
        property_set: &CSSPropertyValueSet<B::ValueDispatch>,
        font_selector: *mut FontSelector,
    ) -> Option<FontDescription> {
        let mut builder = FontBuilder::new(backend.clone(), None);
        let mut description = FontDescription::default();
        let font = backend.NewFont(description.clone(), font_selector);
        // Persistent keeps this actual font rooted for every conversion.
        let font_ref = unsafe { &*font.Get() };
        let ignored_flags = Rc::new(Cell::new(0));
        let data = backend.NewFontStyleConversionData(
            font_ref,
            10.0,
            10.0,
            1.0,
            0.0,
            0.0,
            WritingMode::kHorizontalTb,
            1.0,
            ignored_flags,
        );
        if property_set.HasProperty(CSSPropertyID::kFontSize) {
            let value = property_set
                .GetPropertyCSSValue(CSSPropertyID::kFontSize)
                .expect("present font-size");
            let is_math = matches!(value.Payload(), CSSValuePayload::kIdentifierClass(identifier) if backend.FontIdentifierID(identifier) == CSSValueID::kMath);
            if is_math {
                builder.SetSize(&backend.NewSize(0, 0.0, false));
            } else if Self::ContainsElementDependentCalc(&backend, Some(value)) {
                return None;
            } else {
                builder.SetSize(&backend.ConvertFontSize(
                    value,
                    &data,
                    &backend.NewSize(0, 0.0, false),
                    None,
                ));
            }
        }
        if property_set.HasProperty(CSSPropertyID::kFontFamily) {
            let value = property_set
                .GetPropertyCSSValue(CSSPropertyID::kFontFamily)
                .expect("present font-family");
            builder.SetFamilyDescription(&backend.ConvertFontFamily(value, Some(&builder), None));
        }
        if property_set.HasProperty(CSSPropertyID::kFontStretch) {
            let value = property_set
                .GetPropertyCSSValue(CSSPropertyID::kFontStretch)
                .expect("present font-stretch");
            if Self::ContainsElementDependentCalc(&backend, Some(value)) {
                return None;
            }
            builder.SetStretch(backend.ConvertFontStretch(&data, value));
        }
        if property_set.HasProperty(CSSPropertyID::kFontStyle) {
            let value = property_set
                .GetPropertyCSSValue(CSSPropertyID::kFontStyle)
                .expect("present font-style");
            if Self::ContainsElementDependentCalc(&backend, Some(value)) {
                return None;
            }
            builder.SetStyle(backend.ConvertFontStyle(&data, value));
        }
        if property_set.HasProperty(CSSPropertyID::kFontVariantCaps) {
            let value = property_set
                .GetPropertyCSSValue(CSSPropertyID::kFontVariantCaps)
                .expect("present font-variant-caps");
            builder.SetVariantCaps(backend.ConvertFontVariantCaps(value));
        }
        if property_set.HasProperty(CSSPropertyID::kFontWeight) {
            let value = property_set
                .GetPropertyCSSValue(CSSPropertyID::kFontWeight)
                .expect("present font-weight");
            if Self::ContainsElementDependentCalc(&backend, Some(value)) {
                return None;
            }
            builder.SetWeight(backend.ConvertFontWeight(
                &data,
                value,
                FontBuilder::<B>::InitialWeight(),
            ));
        }
        builder.UpdateFontDescription(&mut description, None);
        Some(description)
    }
}
