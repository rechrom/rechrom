// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_font_palette_values.h
// cpp: third_party/blink/renderer/core/css/style_rule_font_palette_values.cc
// Source physical h56 + cc155; production logic pending: 0.
// Oilpan tracing/allocation, export annotations and downcast boilerplate are
// represented by Rust ownership and StyleRuleBase dispatch.

#![allow(non_snake_case, non_camel_case_types)]

use crate::css_property_names::CSSPropertyID;
use crate::css_property_value_set::{
    CSSPropertyValueSetBackend, CSSPropertyValueSetRuleAdapter, CSSPropertyValueSetRuleHandle,
    MutableCSSPropertyValueSet,
};
use crate::css_value::{CSSValue, CSSValueDispatch};
use crate::style_rule::RuleType;
use foundation::AtomicString;
use std::cell::RefCell;
use std::rc::Rc;

// cpp: platform/fonts/font_palette.h:43-58. Kept here as the small neutral
// value passed from this CSS rule to the eventual font implementation.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BasePaletteValueType {
    kNoBasePalette,
    kLightBasePalette,
    kDarkBasePalette,
    kIndexBasePalette,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BasePaletteValue {
    pub type_: BasePaletteValueType,
    pub index: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontPaletteOverride<Color> {
    pub index: u16,
    pub color: Color,
}

// CSSIdentifierValue/CSSPrimitiveValue, MediaValues and ResolveColorValue are
// real dependencies of this source pair. Required hooks expose their typed
// results; this rule retains Chromium's branching, iteration and clamping.
pub trait StyleRuleFontPaletteValuesBackend: CSSPropertyValueSetBackend {
    type Color: Clone;

    /// Returns `Some(kLightBasePalette | kDarkBasePalette)` for an identifier,
    /// or `None` for the numeric CSSPrimitiveValue path.
    fn BasePaletteIdentifier(value: &CSSValue<Self>) -> Option<BasePaletteValueType>;
    fn ComputeInteger(
        value: &CSSValue<Self>,
        document: &<Self as CSSValueDispatch>::Document,
    ) -> i32;

    /// CSSValueList/CSSValuePair expose their existing value identities. Color
    /// conversion remains split below so this rule owns Chromium's branch
    /// order instead of hiding it behind one aggregate callback.
    fn OverrideColorPairs(
        value: &CSSValue<Self>,
    ) -> Option<Vec<(Rc<CSSValue<Self>>, Rc<CSSValue<Self>>)>>;
    /// Outer `Some` means the value is a CSSIdentifierValue; the inner option
    /// is the keyword conversion result.
    fn ColorFromIdentifier(
        value: &CSSValue<Self>,
        document: &<Self as CSSValueDispatch>::Document,
    ) -> Option<Option<Self::Color>>;
    fn CSSColorValue(value: &CSSValue<Self>) -> Option<Self::Color>;
    fn ResolveAbsoluteColor(
        value: &CSSValue<Self>,
        document: &<Self as CSSValueDispatch>::Document,
    ) -> Option<Self::Color>;
}

// cpp: style_rule_font_palette_values.h:21-51
pub struct StyleRuleFontPaletteValues<D: StyleRuleFontPaletteValuesBackend> {
    name_: AtomicString,
    properties_: RefCell<Rc<CSSPropertyValueSetRuleHandle<D>>>,
}

impl<D: StyleRuleFontPaletteValuesBackend> Clone for StyleRuleFontPaletteValues<D> {
    // cpp: style_rule_font_palette_values.cc:30-31. The source default copy
    // shares the property set until MutableProperties performs copy-on-write.
    fn clone(&self) -> Self {
        Self {
            name_: self.name_.clone(),
            properties_: RefCell::new(self.properties_.borrow().clone()),
        }
    }
}

impl<D: StyleRuleFontPaletteValuesBackend> StyleRuleFontPaletteValues<D> {
    // cpp: style_rule_font_palette_values.cc:22-27
    pub fn new(name: AtomicString, properties: Rc<CSSPropertyValueSetRuleHandle<D>>) -> Self {
        Self {
            name_: name,
            properties_: RefCell::new(properties),
        }
    }

    pub fn GetType(&self) -> RuleType {
        RuleType::kFontPaletteValues
    }
    pub fn GetName(&self) -> AtomicString {
        self.name_.clone()
    }

    // cpp: style_rule_font_palette_values.cc:35-43
    pub fn GetFontFamily(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kFontFamily)
    }
    pub fn GetBasePalette(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kBasePalette)
    }
    pub fn GetOverrideColors(&self) -> Option<Rc<CSSValue<D>>> {
        self.GetDescriptor(CSSPropertyID::kOverrideColors)
    }
    fn GetDescriptor(&self, id: CSSPropertyID) -> Option<Rc<CSSValue<D>>> {
        self.properties_.borrow().GetPropertyCSSValue(id)
    }

    // cpp: style_rule_font_palette_values.cc:44-71
    pub fn GetBasePaletteIndex(
        &self,
        document: &<D as CSSValueDispatch>::Document,
    ) -> BasePaletteValue {
        let Some(base_palette) = self.GetBasePalette() else {
            return BasePaletteValue {
                type_: BasePaletteValueType::kNoBasePalette,
                index: 0,
            };
        };
        if let Some(type_) = D::BasePaletteIdentifier(base_palette.as_ref()) {
            debug_assert!(matches!(
                type_,
                BasePaletteValueType::kLightBasePalette | BasePaletteValueType::kDarkBasePalette
            ));
            return BasePaletteValue { type_, index: 0 };
        }
        BasePaletteValue {
            type_: BasePaletteValueType::kIndexBasePalette,
            index: D::ComputeInteger(base_palette.as_ref(), document),
        }
    }

    // cpp: style_rule_font_palette_values.cc:73-137
    pub fn GetOverrideColorsAsVector(
        &self,
        document: &<D as CSSValueDispatch>::Document,
    ) -> Vec<FontPaletteOverride<D::Color>> {
        let Some(value) = self.GetOverrideColors() else {
            return Vec::new();
        };
        let Some(pairs) = D::OverrideColorPairs(value.as_ref()) else {
            return Vec::new();
        };
        let mut overrides = Vec::with_capacity(pairs.len());
        for (palette_index, color_value) in pairs {
            let palette_index = D::ComputeInteger(palette_index.as_ref(), document);
            let color = if let Some(identifier_result) =
                D::ColorFromIdentifier(color_value.as_ref(), document)
            {
                identifier_result
            } else if let Some(color) = D::CSSColorValue(color_value.as_ref()) {
                Some(color)
            } else {
                D::ResolveAbsoluteColor(color_value.as_ref(), document)
            };
            let Some(color) = color else {
                continue;
            };
            overrides.push(FontPaletteOverride {
                index: palette_index.clamp(0, u16::MAX as i32) as u16,
                color,
            });
        }
        overrides
    }

    // cpp: style_rule_font_palette_values.cc:139-145
    pub fn MutableProperties(&self) -> Rc<RefCell<MutableCSSPropertyValueSet<D>>> {
        let current = self.properties_.borrow().clone();
        if !current.IsMutable() {
            *self.properties_.borrow_mut() =
                CSSPropertyValueSetRuleHandle::FromMutable(current.MutableCopy());
        }
        self.properties_
            .borrow()
            .MutablePropertySet()
            .expect("MutableProperties converted the property store")
    }

    // cpp: style_rule_font_palette_values.h:39-41
    pub fn Copy(&self) -> Self {
        self.clone()
    }
}
