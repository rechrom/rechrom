/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 * Copyright (C) 2013 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// cpp: third_party/blink/renderer/core/css/resolver/font_builder.h
// cpp: third_party/blink/renderer/core/css/resolver/font_builder.cc
// Source ledger (Chromium commit 6c1d401fcca5e1b0030563a90c2f2bba168e0c15):
// /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
// font_builder.h: physical=243 effective=152 mapped=143 omitted=9 pending=0.
// font_builder.cc: physical=705 effective=419 mapped=412 omitted=7 pending=0.
// Effective counts exclude comments/copyright, blank lines, preprocessing,
// includes, namespaces and lines containing only braces/parentheses/semicolons.
// Header omitted: forward declarations 40-43, STACK_ALLOCATED 46, public/private
// labels 48/171, deleted-copy boilerplate 50-51 (Rust has no Clone implementation).
// Implementation omitted: DCHECK 40/308/331/640-641/658/687 only. NOTREACHED 97
// maps to unreachable!, and the compile-time flag-width assertion is retained.
// All other effective declarations in h:45-238 and behavior in cc:39-703 map
// below. Missing accessors are required methods of the real FontDescription;
// dirty bits, equality branches, sizes and font selection remain in this file.
#![allow(non_snake_case, non_camel_case_types)]

use font_engine::fonts::font::FontSelector;
use font_engine::fonts::font_description::{
    FontVariantCaps, GenericFamilyType, Kerning, StyleSyntax,
};
use font_engine::fonts::font_selection_types::{
    kNormalSlopeValue, kNormalWeightValue, kNormalWidthValue,
};
use font_engine::fonts::font_size_adjust::{Metric, ValueType};
use font_engine::fonts::text_rendering_mode::TextRenderingMode;
use font_engine::{
    Font, FontDescription, FontFamily, FontFamilyType, FontFeatureSettings, FontOrientation,
    FontPalette, FontSelectionValue, FontSizeAdjust, FontSmoothingMode, FontVariantAlternates,
    FontVariantEastAsian, FontVariantEmoji, FontVariantNumeric, FontVariationSettings,
    LayoutLocale, OpticalSizing, SimpleFontData, TextSpacingTrim,
};
use foundation::{AtomicString, CSSValueID, Member, Persistent};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use std::rc::Rc;
use std::sync::Arc;

// The genuine Size/FamilyDescription/nested font-description enums are not yet
// present in font_engine. Their typed interfaces and missing accessors act on
// the existing FontDescription only; no parallel description/state is stored.
// Every method is required, including real settings/font-size/font allocation
// collaborators. No default result, font, locale or selector is supplied.
pub trait FontBuilderBackend {
    type Document;
    type TreeScope;
    type Size;
    type FamilyDescription;
    type VariantLigatures: Clone + PartialEq;
    type FontSynthesisWeight: Clone + PartialEq;
    type FontSynthesisStyle: Clone + PartialEq;
    type FontSynthesisSmallCaps: Clone + PartialEq;
    type FontVariantPosition: Clone + PartialEq;
    fn FamilyGeneric(&self, family: &Self::FamilyDescription) -> GenericFamilyType;
    fn FamilyValue(&self, family: &Self::FamilyDescription) -> FontFamily;
    fn NewFamilyDescription(&self, generic: GenericFamilyType) -> Self::FamilyDescription;
    fn SizeValue(&self, size: &Self::Size) -> f32;
    fn SizeKeyword(&self, size: &Self::Size) -> u32;
    fn SizeIsAbsolute(&self, size: &Self::Size) -> bool;
    fn NewSize(&self, keyword: u32, value: f32, is_absolute: bool) -> Self::Size;
    fn GenericFamily(&self, d: &FontDescription) -> GenericFamilyType;
    fn SetGenericFamily(&self, d: &mut FontDescription, v: GenericFamilyType);
    fn KeywordSize(&self, d: &FontDescription) -> u32;
    fn SetKeywordSize(&self, d: &mut FontDescription, v: u32);
    fn IsAbsoluteSize(&self, d: &FontDescription) -> bool;
    fn SetIsAbsoluteSize(&self, d: &mut FontDescription, v: bool);
    fn IsMonospace(&self, d: &FontDescription) -> bool;
    fn GetStyleSyntax(&self, d: &FontDescription) -> StyleSyntax;
    fn SetStyleSyntax(&self, d: &mut FontDescription, v: StyleSyntax);
    fn StandardFontFamilySetting(&self, document: &Self::Document) -> Option<AtomicString>;
    fn InferredFamilyType(&self, name: &AtomicString) -> FontFamilyType;
    fn FontSizeForKeyword(
        &self,
        document: Option<&Self::Document>,
        keyword: u32,
        is_monospace: bool,
    ) -> f32;
    fn MaximumAllowedFontSize(&self) -> f32;
    fn FrameTextZoomFactor(&self, document: &Self::Document) -> Option<f32>;
    fn TextSizeAdjustEnabled(&self, document: &Self::Document) -> bool;
    fn DefaultFontSizes(&self, document: &Self::Document) -> Option<(u32, u32)>;
    fn ComputedSizeFromSpecifiedSize(
        &self,
        document: &Self::Document,
        zoom: f32,
        is_absolute: bool,
        specified: f32,
    ) -> f32;
    fn DocumentFontSelector(&self, document: &Self::Document) -> *mut FontSelector;
    fn ExistingFontSelector(&self, font: &Font) -> *mut FontSelector;
    fn NewFont(
        &self,
        description: FontDescription,
        selector: *mut FontSelector,
    ) -> Persistent<Font>;
    fn FontAspectValue(&self, data: &SimpleFontData, metric: Metric) -> Option<f32>;
    fn MetricsMultiplierAdjustedFontSize(
        &self,
        data: &SimpleFontData,
        description: &FontDescription,
    ) -> Option<f32>;
    fn FeatureSettingsValue(&self, d: &FontDescription) -> Option<FontFeatureSettings>;
    fn SetFeatureSettings(&self, d: &mut FontDescription, v: Option<FontFeatureSettings>);
    /// The source compares scoped_refptr identity, not settings-list contents.
    fn SameFeatureSettings(&self, a: &FontDescription, b: &FontDescription) -> bool;
    fn LocaleValue(&self, d: &FontDescription) -> Option<Arc<LayoutLocale>>;
    fn VariantCaps(&self, d: &FontDescription) -> FontVariantCaps;
    fn SetVariantCaps(&self, d: &mut FontDescription, v: FontVariantCaps);
    fn VariantEastAsian(&self, d: &FontDescription) -> FontVariantEastAsian;
    fn SetVariantEastAsian(&self, d: &mut FontDescription, v: FontVariantEastAsian);
    fn VariantLigatures(&self, d: &FontDescription) -> Self::VariantLigatures;
    fn SetVariantLigatures(&self, d: &mut FontDescription, v: Self::VariantLigatures);
    fn InitialVariantLigatures(&self) -> Self::VariantLigatures;
    fn VariantNumeric(&self, d: &FontDescription) -> FontVariantNumeric;
    fn SetVariantNumeric(&self, d: &mut FontDescription, v: FontVariantNumeric);
    fn VariationSettingsValue(&self, d: &FontDescription) -> Option<FontVariationSettings>;
    fn SetVariationSettings(&self, d: &mut FontDescription, v: Option<FontVariationSettings>);
    /// The source compares scoped_refptr identity, not settings-list contents.
    fn SameVariationSettings(&self, a: &FontDescription, b: &FontDescription) -> bool;
    fn FontLanguageOverride(&self, d: &FontDescription) -> AtomicString;
    fn SetFontLanguageOverride(&self, d: &mut FontDescription, v: AtomicString);
    fn FontSynthesisWeight(&self, d: &FontDescription) -> Self::FontSynthesisWeight;
    fn SetFontSynthesisWeight(&self, d: &mut FontDescription, v: Self::FontSynthesisWeight);
    fn InitialFontSynthesisWeight(&self) -> Self::FontSynthesisWeight;
    fn FontSynthesisStyle(&self, d: &FontDescription) -> Self::FontSynthesisStyle;
    fn SetFontSynthesisStyle(&self, d: &mut FontDescription, v: Self::FontSynthesisStyle);
    fn InitialFontSynthesisStyle(&self) -> Self::FontSynthesisStyle;
    fn FontSynthesisSmallCaps(&self, d: &FontDescription) -> Self::FontSynthesisSmallCaps;
    fn SetFontSynthesisSmallCaps(&self, d: &mut FontDescription, v: Self::FontSynthesisSmallCaps);
    fn InitialFontSynthesisSmallCaps(&self) -> Self::FontSynthesisSmallCaps;
    fn SetTextRendering(&self, d: &mut FontDescription, v: TextRenderingMode);
    fn Kerning(&self, d: &FontDescription) -> Kerning;
    fn SetKerning(&self, d: &mut FontDescription, v: Kerning);
    fn SetTextSpacingTrim(&self, d: &mut FontDescription, v: TextSpacingTrim);
    fn FontOpticalSizing(&self, d: &FontDescription) -> OpticalSizing;
    fn SetFontOpticalSizing(&self, d: &mut FontDescription, v: OpticalSizing);
    fn FontPaletteValue(&self, d: &FontDescription) -> Option<Arc<FontPalette>>;
    fn SetFontPalette(&self, d: &mut FontDescription, v: Option<Arc<FontPalette>>);
    fn FontVariantAlternatesValue(&self, d: &FontDescription)
        -> Option<Arc<FontVariantAlternates>>;
    fn SetFontVariantAlternates(
        &self,
        d: &mut FontDescription,
        v: Option<Arc<FontVariantAlternates>>,
    );
    /// The source compares scoped_refptr identity.
    fn SameFontVariantAlternates(&self, a: &FontDescription, b: &FontDescription) -> bool;
    fn VariantPosition(&self, d: &FontDescription) -> Self::FontVariantPosition;
    fn SetVariantPosition(&self, d: &mut FontDescription, v: Self::FontVariantPosition);
    fn InitialVariantPosition(&self) -> Self::FontVariantPosition;
    fn VariantEmoji(&self, d: &FontDescription) -> FontVariantEmoji;
}

// cpp: font_builder.h:194-238
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum PropertySetFlag {
    kWeight,
    kSize,
    kStretch,
    kFamily,
    kFeatureSettings,
    kLocale,
    kStyle,
    kSizeAdjust,
    kVariantCaps,
    kVariantEastAsian,
    kVariantLigatures,
    kVariantNumeric,
    kVariantEmoji,
    kVariantPosition,
    kVariationSettings,
    kFontLanguageOverride,
    kTextRendering,
    kKerning,
    kTextSpacingTrim,
    kFontOpticalSizing,
    kFontPalette,
    kFontVariantAlternates,
    kFontSmoothing,
    kFontSynthesisWeight,
    kFontSynthesisStyle,
    kFontSynthesisSmallCaps,
    kEffectiveZoom,
    kTextOrientation,
    kWritingMode,
    kTextSizeAdjust,
    kNumFlags,
}
const _: () = assert!(PropertySetFlag::kNumFlags as usize <= u32::BITS as usize);

// cpp: font_builder.h:190-238
pub struct FontBuilder<B: FontBuilderBackend> {
    backend: Rc<B>,
    document_: Option<Rc<B::Document>>,
    family_tree_scope_: Option<Rc<B::TreeScope>>,
    font_description_: FontDescription,
    flags_: u32,
}
impl<B: FontBuilderBackend> FontBuilder<B> {
    // cpp: font_builder.cc:39-41
    pub fn new(backend: Rc<B>, document: Option<Rc<B::Document>>) -> Self {
        Self {
            backend,
            document_: document,
            family_tree_scope_: None,
            font_description_: FontDescription::default(),
            flags_: 0,
        }
    }
    // cpp: font_builder.h:231-236
    fn Set(&mut self, flag: PropertySetFlag) {
        self.flags_ |= 1 << flag as u32;
    }
    fn IsSet(&self, flag: PropertySetFlag) -> bool {
        self.flags_ & (1 << flag as u32) != 0
    }
    pub fn FontDirty(&self) -> bool {
        self.flags_ != 0
    }
    // cpp: font_builder.cc:43-59
    pub fn DidChangeEffectiveZoom(&mut self) {
        self.Set(PropertySetFlag::kEffectiveZoom);
    }
    pub fn DidChangeTextOrientation(&mut self) {
        self.Set(PropertySetFlag::kTextOrientation);
    }
    pub fn DidChangeWritingMode(&mut self) {
        self.Set(PropertySetFlag::kWritingMode);
    }
    pub fn DidChangeTextSizeAdjust(&mut self) {
        self.Set(PropertySetFlag::kTextSizeAdjust);
    }
    // cpp: font_builder.cc:61-103
    pub fn StandardFontFamily(&self) -> FontFamily {
        let name = self.StandardFontFamilyName();
        let kind = self.backend.InferredFamilyType(&name);
        FontFamily::new(name, kind, None)
    }
    pub fn StandardFontFamilyName(&self) -> AtomicString {
        self.document_
            .as_deref()
            .and_then(|d| self.backend.StandardFontFamilySetting(d))
            .unwrap_or_default()
    }
    pub fn GenericFontFamilyName(&self, generic: GenericFamilyType) -> AtomicString {
        match generic {
            GenericFamilyType::kNoFamily => AtomicString::default(),
            GenericFamilyType::kWebkitBodyFamily => self.StandardFontFamilyName(),
            GenericFamilyType::kSerifFamily => AtomicString::from_str("serif"),
            GenericFamilyType::kSansSerifFamily => AtomicString::from_str("sans-serif"),
            GenericFamilyType::kMonospaceFamily => AtomicString::from_str("monospace"),
            GenericFamilyType::kCursiveFamily => AtomicString::from_str("cursive"),
            GenericFamilyType::kFantasyFamily => AtomicString::from_str("fantasy"),
            GenericFamilyType::kStandardFamily => {
                unreachable!("standard is not a CSS generic family")
            }
        }
    }
    pub fn FontSizeForKeyword(&self, keyword: u32, is_monospace: bool) -> f32 {
        self.backend
            .FontSizeForKeyword(self.document_.as_deref(), keyword, is_monospace)
    }
    // cpp: font_builder.cc:113-116
    pub fn SetFamilyTreeScope(&mut self, scope: Option<Rc<B::TreeScope>>) {
        self.family_tree_scope_ = scope;
    }
    // cpp: font_builder.cc:107-110,258-289
    pub fn SetFamilyDescription(&mut self, family: &B::FamilyDescription) {
        self.Set(PropertySetFlag::kFamily);
        let generic = self.backend.FamilyGeneric(family);
        let mut value = self.backend.FamilyValue(family);
        if generic == GenericFamilyType::kStandardFamily && value.FamilyName().empty() {
            value = self.StandardFontFamily();
        }
        self.backend
            .SetGenericFamily(&mut self.font_description_, generic);
        self.font_description_.SetFamily(&value);
    }
    pub fn SetSize(&mut self, size: &B::Size) {
        let value = self.backend.SizeValue(size);
        if value < 0.0 {
            return;
        }
        self.Set(PropertySetFlag::kSize);
        self.backend
            .SetKeywordSize(&mut self.font_description_, self.backend.SizeKeyword(size));
        self.font_description_
            .SetSpecifiedSize(self.backend.MaximumAllowedFontSize().min(value));
        self.backend.SetIsAbsoluteSize(
            &mut self.font_description_,
            self.backend.SizeIsAbsolute(size),
        );
    }
    fn ApplyFamilyToDescription(&mut self, family: &B::FamilyDescription, d: &mut FontDescription) {
        self.Set(PropertySetFlag::kFamily);
        let generic = self.backend.FamilyGeneric(family);
        let value = self.backend.FamilyValue(family);
        let value = if generic == GenericFamilyType::kStandardFamily && value.FamilyName().empty() {
            self.StandardFontFamily()
        } else {
            value
        };
        self.backend.SetGenericFamily(d, generic);
        d.SetFamily(&value);
    }
    fn ApplySizeToDescription(&mut self, size: &B::Size, d: &mut FontDescription) {
        let value = self.backend.SizeValue(size);
        if value < 0.0 {
            return;
        }
        self.Set(PropertySetFlag::kSize);
        self.backend
            .SetKeywordSize(d, self.backend.SizeKeyword(size));
        d.SetSpecifiedSize(self.backend.MaximumAllowedFontSize().min(value));
        self.backend
            .SetIsAbsoluteSize(d, self.backend.SizeIsAbsolute(size));
    }
    // cpp: font_builder.cc:122-131
    pub fn SetStyle(&mut self, value: FontSelectionValue) {
        self.Set(PropertySetFlag::kStyle);
        self.font_description_.SetStyle(value);
    }
    pub fn SetStyleSyntax(&mut self, value: StyleSyntax) {
        self.Set(PropertySetFlag::kStyle);
        self.backend
            .SetStyleSyntax(&mut self.font_description_, value);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetWeight(&mut self, value: FontSelectionValue) {
        self.Set(PropertySetFlag::kWeight);
        let d = &mut self.font_description_;
        let v = value;
        d.SetWeight(v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetStretch(&mut self, value: FontSelectionValue) {
        self.Set(PropertySetFlag::kStretch);
        let d = &mut self.font_description_;
        let v = value;
        d.SetStretch(v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetSizeAdjust(&mut self, value: FontSizeAdjust) {
        self.Set(PropertySetFlag::kSizeAdjust);
        let d = &mut self.font_description_;
        let v = value;
        d.SetSizeAdjust(&v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFeatureSettings(&mut self, value: Option<FontFeatureSettings>) {
        self.Set(PropertySetFlag::kFeatureSettings);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFeatureSettings(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetLocale(&mut self, value: Option<Arc<LayoutLocale>>) {
        self.Set(PropertySetFlag::kLocale);
        let d = &mut self.font_description_;
        let v = value;
        d.SetLocale(v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariantCaps(&mut self, value: FontVariantCaps) {
        self.Set(PropertySetFlag::kVariantCaps);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetVariantCaps(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariantEastAsian(&mut self, value: FontVariantEastAsian) {
        self.Set(PropertySetFlag::kVariantEastAsian);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetVariantEastAsian(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariantLigatures(&mut self, value: B::VariantLigatures) {
        self.Set(PropertySetFlag::kVariantLigatures);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetVariantLigatures(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariantNumeric(&mut self, value: FontVariantNumeric) {
        self.Set(PropertySetFlag::kVariantNumeric);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetVariantNumeric(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariationSettings(&mut self, value: Option<FontVariationSettings>) {
        self.Set(PropertySetFlag::kVariationSettings);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetVariationSettings(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontLanguageOverride(&mut self, value: AtomicString) {
        self.Set(PropertySetFlag::kFontLanguageOverride);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontLanguageOverride(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontSynthesisWeight(&mut self, value: B::FontSynthesisWeight) {
        self.Set(PropertySetFlag::kFontSynthesisWeight);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontSynthesisWeight(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontSynthesisStyle(&mut self, value: B::FontSynthesisStyle) {
        self.Set(PropertySetFlag::kFontSynthesisStyle);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontSynthesisStyle(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontSynthesisSmallCaps(&mut self, value: B::FontSynthesisSmallCaps) {
        self.Set(PropertySetFlag::kFontSynthesisSmallCaps);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontSynthesisSmallCaps(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetTextRendering(&mut self, value: TextRenderingMode) {
        self.Set(PropertySetFlag::kTextRendering);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetTextRendering(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetKerning(&mut self, value: Kerning) {
        self.Set(PropertySetFlag::kKerning);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetKerning(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetTextSpacingTrim(&mut self, value: TextSpacingTrim) {
        self.Set(PropertySetFlag::kTextSpacingTrim);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetTextSpacingTrim(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontOpticalSizing(&mut self, value: OpticalSizing) {
        self.Set(PropertySetFlag::kFontOpticalSizing);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontOpticalSizing(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontPalette(&mut self, value: Option<Arc<FontPalette>>) {
        self.Set(PropertySetFlag::kFontPalette);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontPalette(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontVariantAlternates(&mut self, value: Option<Arc<FontVariantAlternates>>) {
        self.Set(PropertySetFlag::kFontVariantAlternates);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetFontVariantAlternates(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetFontSmoothing(&mut self, value: FontSmoothingMode) {
        self.Set(PropertySetFlag::kFontSmoothing);
        let d = &mut self.font_description_;
        let v = value;
        d.SetFontSmoothing(v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariantPosition(&mut self, value: B::FontVariantPosition) {
        self.Set(PropertySetFlag::kVariantPosition);
        let d = &mut self.font_description_;
        let v = value;
        self.backend.SetVariantPosition(d, v);
    }
    // cpp: font_builder.cc:116-255,291-302 (property setter)
    pub fn SetVariantEmoji(&mut self, value: FontVariantEmoji) {
        self.Set(PropertySetFlag::kVariantEmoji);
        let d = &mut self.font_description_;
        let v = value;
        d.SetVariantEmoji(v);
    }

    // cpp: font_builder.cc:304-376
    fn GetComputedSizeFromSpecifiedSize(
        &self,
        d: &FontDescription,
        builder: &ComputedStyleBuilder,
        specified: f32,
    ) -> f32 {
        let document = self
            .document_
            .as_deref()
            .expect("computed font size requires document");
        let mut zoom = builder.EffectiveZoom();
        if let Some(text_zoom) = self.backend.FrameTextZoomFactor(document) {
            zoom *= text_zoom;
        }
        if !builder.GetTextSizeAdjust().IsAuto() && self.backend.TextSizeAdjustEnabled(document) {
            zoom *= builder.GetTextSizeAdjust().Multiplier();
        }
        self.backend.ComputedSizeFromSpecifiedSize(
            document,
            zoom,
            self.backend.IsAbsoluteSize(d),
            specified,
        )
    }
    fn CheckForGenericFamilyChange(&self, parent: &FontDescription, d: &mut FontDescription) {
        if self.backend.IsAbsoluteSize(d)
            || self.backend.IsMonospace(d) == self.backend.IsMonospace(parent)
        {
            return;
        }
        let keyword = self.backend.KeywordSize(d);
        let size = if keyword != 0 {
            self.FontSizeForKeyword(keyword, self.backend.IsMonospace(d))
        } else {
            let document = self
                .document_
                .as_deref()
                .expect("family size requires document");
            let scale = match self.backend.DefaultFontSizes(document) {
                Some((fixed, normal)) if fixed != 0 && normal != 0 => fixed as f32 / normal as f32,
                _ => 1.0,
            };
            if self.backend.IsMonospace(parent) {
                d.SpecifiedSize() / scale
            } else {
                d.SpecifiedSize() * scale
            }
        };
        d.SetSpecifiedSize(size);
    }
    fn UpdateSpecifiedSize(&self, d: &mut FontDescription, parent: &FontDescription) {
        let mut size = d.SpecifiedSize();
        let keyword = self.backend.KeywordSize(d);
        if size == 0.0 && keyword != 0 {
            size = self.FontSizeForKeyword(keyword, self.backend.IsMonospace(d));
        }
        d.SetSpecifiedSize(size);
        self.CheckForGenericFamilyChange(parent, d);
    }
    // cpp: font_builder.cc:378-424
    fn UpdateAdjustedSize(&self, d: &mut FontDescription, selector: *mut FontSelector) {
        let computed = d.ComputedSize();
        if !d.HasSizeAdjust() || computed == 0.0 {
            return;
        }
        d.SetAdjustedSize(computed);
        let font = self.backend.NewFont(d.clone(), selector);
        let primary = unsafe { (&*font.Get()).PrimaryFont() };
        if primary.is_null() {
            return;
        }
        // Font's Persistent root keeps the primary font reachable for these calls.
        let primary = unsafe { &*primary };
        let adjust = d.SizeAdjust();
        if adjust.IsFromFont() && adjust.Value() == FontSizeAdjust::kFontSizeAdjustNone {
            let aspect = self.backend.FontAspectValue(primary, adjust.GetMetric());
            d.SetSizeAdjust(&FontSizeAdjust::with_metric_and_type(
                aspect.unwrap_or(FontSizeAdjust::kFontSizeAdjustNone),
                adjust.GetMetric(),
                ValueType::kFromFont,
            ));
        }
        if let Some(size) = self.backend.MetricsMultiplierAdjustedFontSize(primary, d) {
            d.SetAdjustedSize(size);
        }
    }
    fn UpdateComputedSize(&self, d: &mut FontDescription, builder: &ComputedStyleBuilder) {
        d.SetComputedSize(self.GetComputedSizeFromSpecifiedSize(d, builder, d.SpecifiedSize()));
    }
    // cpp: font_builder.cc:426-633. Each dirty-property comparison is kept here;
    // required accessors above are methods of the actual external description.
    pub fn UpdateFontDescription(
        &self,
        description: &mut FontDescription,
        orientation: Option<FontOrientation>,
    ) -> bool {
        let orientation = orientation.unwrap_or(FontOrientation::kHorizontal);
        let mut modified = false;
        if self.IsSet(PropertySetFlag::kFamily)
            && (self.backend.GenericFamily(description)
                != self.backend.GenericFamily(&self.font_description_)
                || description.Family() != self.font_description_.Family())
        {
            modified = true;
            self.backend.SetGenericFamily(
                description,
                self.backend.GenericFamily(&self.font_description_),
            );
            description.SetFamily(self.font_description_.Family());
        }
        if self.IsSet(PropertySetFlag::kSize)
            && (self.backend.KeywordSize(description)
                != self.backend.KeywordSize(&self.font_description_)
                || description.SpecifiedSize() != self.font_description_.SpecifiedSize()
                || self.backend.IsAbsoluteSize(description)
                    != self.backend.IsAbsoluteSize(&self.font_description_))
        {
            modified = true;
            self.backend.SetKeywordSize(
                description,
                self.backend.KeywordSize(&self.font_description_),
            );
            description.SetSpecifiedSize(self.font_description_.SpecifiedSize());
            self.backend.SetIsAbsoluteSize(
                description,
                self.backend.IsAbsoluteSize(&self.font_description_),
            );
        }
        if self.IsSet(PropertySetFlag::kSizeAdjust)
            && !(description.SizeAdjust() == self.font_description_.SizeAdjust())
        {
            modified = true;
            let v = self.font_description_.SizeAdjust();
            description.SetSizeAdjust(&v);
        }
        if self.IsSet(PropertySetFlag::kWeight)
            && !(description.Weight() == self.font_description_.Weight())
        {
            modified = true;
            let v = self.font_description_.Weight();
            description.SetWeight(v);
        }
        if self.IsSet(PropertySetFlag::kStretch)
            && !(description.Stretch() == self.font_description_.Stretch())
        {
            modified = true;
            let v = self.font_description_.Stretch();
            description.SetStretch(v);
        }
        if self.IsSet(PropertySetFlag::kFeatureSettings)
            && !(self
                .backend
                .SameFeatureSettings(description, &self.font_description_))
        {
            modified = true;
            let v = self.backend.FeatureSettingsValue(&self.font_description_);
            self.backend.SetFeatureSettings(description, v);
        }
        if self.IsSet(PropertySetFlag::kLocale)
            && !(description.Locale() == self.font_description_.Locale())
        {
            modified = true;
            let v = self.backend.LocaleValue(&self.font_description_);
            description.SetLocale(v);
        }
        if self.IsSet(PropertySetFlag::kStyle)
            && (description.Style() != self.font_description_.Style()
                || self.backend.GetStyleSyntax(description)
                    != self.backend.GetStyleSyntax(&self.font_description_))
        {
            modified = true;
            description.SetStyle(self.font_description_.Style());
            self.backend.SetStyleSyntax(
                description,
                self.backend.GetStyleSyntax(&self.font_description_),
            );
        }
        if self.IsSet(PropertySetFlag::kVariantCaps)
            && !(self.backend.VariantCaps(description)
                == self.backend.VariantCaps(&self.font_description_))
        {
            modified = true;
            let v = self.backend.VariantCaps(&self.font_description_);
            self.backend.SetVariantCaps(description, v);
        }
        if self.IsSet(PropertySetFlag::kVariantEastAsian)
            && !(self.backend.VariantEastAsian(description)
                == self.backend.VariantEastAsian(&self.font_description_))
        {
            modified = true;
            let v = self.backend.VariantEastAsian(&self.font_description_);
            self.backend.SetVariantEastAsian(description, v);
        }
        if self.IsSet(PropertySetFlag::kVariantLigatures)
            && !(self.backend.VariantLigatures(description)
                == self.backend.VariantLigatures(&self.font_description_))
        {
            modified = true;
            let v = self.backend.VariantLigatures(&self.font_description_);
            self.backend.SetVariantLigatures(description, v);
        }
        if self.IsSet(PropertySetFlag::kVariantNumeric)
            && !(self.backend.VariantNumeric(description)
                == self.backend.VariantNumeric(&self.font_description_))
        {
            modified = true;
            let v = self.backend.VariantNumeric(&self.font_description_);
            self.backend.SetVariantNumeric(description, v);
        }
        if self.IsSet(PropertySetFlag::kVariationSettings)
            && !(self
                .backend
                .SameVariationSettings(description, &self.font_description_))
        {
            modified = true;
            let v = self.backend.VariationSettingsValue(&self.font_description_);
            self.backend.SetVariationSettings(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontLanguageOverride)
            && !(self.backend.FontLanguageOverride(description)
                == self.backend.FontLanguageOverride(&self.font_description_))
        {
            modified = true;
            let v = self.backend.FontLanguageOverride(&self.font_description_);
            self.backend.SetFontLanguageOverride(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontSynthesisWeight)
            && !(self.backend.FontSynthesisWeight(description)
                == self.backend.FontSynthesisWeight(&self.font_description_))
        {
            modified = true;
            let v = self.backend.FontSynthesisWeight(&self.font_description_);
            self.backend.SetFontSynthesisWeight(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontSynthesisStyle)
            && !(self.backend.FontSynthesisStyle(description)
                == self.backend.FontSynthesisStyle(&self.font_description_))
        {
            modified = true;
            let v = self.backend.FontSynthesisStyle(&self.font_description_);
            self.backend.SetFontSynthesisStyle(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontSynthesisSmallCaps)
            && !(self.backend.FontSynthesisSmallCaps(description)
                == self.backend.FontSynthesisSmallCaps(&self.font_description_))
        {
            modified = true;
            let v = self.backend.FontSynthesisSmallCaps(&self.font_description_);
            self.backend.SetFontSynthesisSmallCaps(description, v);
        }
        if self.IsSet(PropertySetFlag::kTextRendering)
            && !(description.TextRendering() == self.font_description_.TextRendering())
        {
            modified = true;
            let v = self.font_description_.TextRendering();
            self.backend.SetTextRendering(description, v);
        }
        if self.IsSet(PropertySetFlag::kKerning)
            && !(self.backend.Kerning(description) == self.backend.Kerning(&self.font_description_))
        {
            modified = true;
            let v = self.backend.Kerning(&self.font_description_);
            self.backend.SetKerning(description, v);
        }
        if self.IsSet(PropertySetFlag::kTextSpacingTrim)
            && !(description.GetTextSpacingTrim() == self.font_description_.GetTextSpacingTrim())
        {
            modified = true;
            let v = self.font_description_.GetTextSpacingTrim();
            self.backend.SetTextSpacingTrim(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontOpticalSizing)
            && !(self.backend.FontOpticalSizing(description)
                == self.backend.FontOpticalSizing(&self.font_description_))
        {
            modified = true;
            let v = self.backend.FontOpticalSizing(&self.font_description_);
            self.backend.SetFontOpticalSizing(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontPalette)
            && !(description.GetFontPalette() == self.font_description_.GetFontPalette())
        {
            modified = true;
            let v = self.backend.FontPaletteValue(&self.font_description_);
            self.backend.SetFontPalette(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontVariantAlternates)
            && !(self
                .backend
                .SameFontVariantAlternates(description, &self.font_description_))
        {
            modified = true;
            let v = self
                .backend
                .FontVariantAlternatesValue(&self.font_description_);
            self.backend.SetFontVariantAlternates(description, v);
        }
        if self.IsSet(PropertySetFlag::kFontSmoothing)
            && !(description.FontSmoothing() == self.font_description_.FontSmoothing())
        {
            modified = true;
            let v = self.font_description_.FontSmoothing();
            description.SetFontSmoothing(v);
        }
        if (self.IsSet(PropertySetFlag::kTextOrientation)
            || self.IsSet(PropertySetFlag::kWritingMode))
            && description.Orientation() != orientation
        {
            modified = true;
            description.SetOrientation(orientation);
        }
        if self.IsSet(PropertySetFlag::kVariantPosition)
            && !(self.backend.VariantPosition(description)
                == self.backend.VariantPosition(&self.font_description_))
        {
            modified = true;
            let v = self.backend.VariantPosition(&self.font_description_);
            self.backend.SetVariantPosition(description, v);
        }
        if self.IsSet(PropertySetFlag::kVariantEmoji)
            && !(self.backend.VariantEmoji(description)
                == self.backend.VariantEmoji(&self.font_description_))
        {
            modified = true;
            let v = self.backend.VariantEmoji(&self.font_description_);
            description.SetVariantEmoji(v);
        }
        if !modified
            && !self.IsSet(PropertySetFlag::kEffectiveZoom)
            && !self.IsSet(PropertySetFlag::kTextSizeAdjust)
        {
            return false;
        }
        let mut size = description.SpecifiedSize();
        let keyword = self.backend.KeywordSize(description);
        if size == 0.0 && keyword != 0 {
            size = self.FontSizeForKeyword(keyword, self.backend.IsMonospace(description));
        }
        description.SetSpecifiedSize(size);
        description.SetComputedSize(size);
        if size != 0.0 && description.HasSizeAdjust() {
            description.SetAdjustedSize(size);
        }
        true
    }
    // cpp: font_builder.cc:635-654
    fn FontSelectorFromTreeScope(&self, _scope: Option<&B::TreeScope>) -> *mut FontSelector {
        self.backend.DocumentFontSelector(
            self.document_
                .as_deref()
                .expect("font selector requires document"),
        )
    }
    fn ComputeFontSelector(&self, builder: &ComputedStyleBuilder) -> *mut FontSelector {
        if self.IsSet(PropertySetFlag::kFamily) {
            self.FontSelectorFromTreeScope(self.family_tree_scope_.as_deref())
        } else {
            self.backend
                .ExistingFontSelector(unsafe { &*builder.GetFont() })
        }
    }
    // cpp: font_builder.cc:656-684
    pub fn CreateFont(
        &mut self,
        builder: &mut ComputedStyleBuilder,
        parent: Option<&ComputedStyle>,
    ) {
        if self.flags_ == 0 {
            return;
        }
        let parent_description = parent.map_or_else(
            || builder.GetFontDescription().clone(),
            |p| p.GetFontDescription().clone(),
        );
        let mut description = builder.GetFontDescription().clone();
        if !self.UpdateFontDescription(&mut description, Some(builder.ComputeFontOrientation())) {
            self.flags_ = 0;
            return;
        }
        self.UpdateSpecifiedSize(&mut description, &parent_description);
        self.UpdateComputedSize(&mut description, builder);
        let selector = self.ComputeFontSelector(builder);
        self.UpdateAdjustedSize(&mut description, selector);
        let font = self.backend.NewFont(description, selector);
        builder.SetFont(Member::from_ptr(font.Get()));
        self.flags_ = 0;
    }
    // cpp: font_builder.cc:686-703
    pub fn CreateInitialFont(&mut self, builder: &mut ComputedStyleBuilder) {
        let mut description = FontDescription::default();
        description.SetLocale(self.backend.LocaleValue(builder.GetFontDescription()));
        let family = self.InitialFamilyDescription();
        self.ApplyFamilyToDescription(&family, &mut description);
        let size = self.InitialSize();
        self.ApplySizeToDescription(&size, &mut description);
        self.UpdateSpecifiedSize(&mut description, builder.GetFontDescription());
        self.UpdateComputedSize(&mut description, builder);
        description.SetOrientation(builder.ComputeFontOrientation());
        let selector = self.backend.DocumentFontSelector(
            self.document_
                .as_deref()
                .expect("initial font requires document"),
        );
        let font = self.backend.NewFont(description, selector);
        builder.SetFont(Member::from_ptr(font.Get()));
    }
    // cpp: font_builder.h:113-170. Typed factories represent the not-yet-
    // translated nested constructors, while initial values remain explicit.
    pub fn InitialGenericFamily() -> GenericFamilyType {
        GenericFamilyType::kStandardFamily
    }
    pub fn InitialFamilyDescription(&self) -> B::FamilyDescription {
        self.backend
            .NewFamilyDescription(Self::InitialGenericFamily())
    }
    pub fn InitialSize(&self) -> B::Size {
        let keyword = (CSSValueID::kMedium as i32 - CSSValueID::kXxSmall as i32 + 1) as u32;
        self.backend.NewSize(keyword, 0.0, false)
    }
    pub fn InitialStyle() -> FontSelectionValue {
        kNormalSlopeValue
    }
    pub fn InitialWeight() -> FontSelectionValue {
        kNormalWeightValue
    }
    pub fn InitialStretch() -> FontSelectionValue {
        kNormalWidthValue
    }
    pub fn InitialSizeAdjust() -> FontSizeAdjust {
        FontSizeAdjust::default()
    }
    pub fn InitialFeatureSettings() -> Option<FontFeatureSettings> {
        None
    }
    pub fn InitialLocale() -> Option<Arc<LayoutLocale>> {
        None
    }
    pub fn InitialVariantCaps() -> FontVariantCaps {
        FontVariantCaps::kCapsNormal
    }
    pub fn InitialVariantEastAsian() -> FontVariantEastAsian {
        FontVariantEastAsian::default()
    }
    pub fn InitialVariantLigatures(&self) -> B::VariantLigatures {
        self.backend.InitialVariantLigatures()
    }
    pub fn InitialVariantNumeric() -> FontVariantNumeric {
        FontVariantNumeric::default()
    }
    pub fn InitialVariationSettings() -> Option<FontVariationSettings> {
        None
    }
    pub fn InitialFontLanguageOverride() -> AtomicString {
        AtomicString::default()
    }
    pub fn InitialFontSynthesisWeight(&self) -> B::FontSynthesisWeight {
        self.backend.InitialFontSynthesisWeight()
    }
    pub fn InitialFontSynthesisStyle(&self) -> B::FontSynthesisStyle {
        self.backend.InitialFontSynthesisStyle()
    }
    pub fn InitialFontSynthesisSmallCaps(&self) -> B::FontSynthesisSmallCaps {
        self.backend.InitialFontSynthesisSmallCaps()
    }
    pub fn InitialTextRendering() -> TextRenderingMode {
        TextRenderingMode::kAutoTextRendering
    }
    pub fn InitialKerning() -> Kerning {
        Kerning::kAutoKerning
    }
    pub fn InitialTextSpacingTrim() -> TextSpacingTrim {
        TextSpacingTrim::kInitial
    }
    pub fn InitialFontOpticalSizing() -> OpticalSizing {
        OpticalSizing::kAutoOpticalSizing
    }
    pub fn InitialFontPalette() -> Option<Arc<FontPalette>> {
        None
    }
    pub fn InitialFontVariantAlternates() -> Option<Arc<FontVariantAlternates>> {
        None
    }
    pub fn InitialFontSmoothing() -> FontSmoothingMode {
        FontSmoothingMode::kAutoSmoothing
    }
    pub fn InitialVariantPosition(&self) -> B::FontVariantPosition {
        self.backend.InitialVariantPosition()
    }
    pub fn InitialVariantEmoji() -> FontVariantEmoji {
        FontVariantEmoji::kNormalVariantEmoji
    }
}

// The same FontBuilder instance is used by StyleResolverState; these forward
// only its consumed methods, with no second flags or font description.
impl<B: FontBuilderBackend> super::style_resolver_state::StyleResolverStateFontBuilder
    for FontBuilder<B>
{
    fn CreateFont(&mut self, builder: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>) {
        FontBuilder::CreateFont(self, builder, parent);
    }
    fn DidChangeEffectiveZoom(&mut self) {
        FontBuilder::DidChangeEffectiveZoom(self);
    }
    fn DidChangeWritingMode(&mut self) {
        FontBuilder::DidChangeWritingMode(self);
    }
    fn DidChangeTextSizeAdjust(&mut self) {
        FontBuilder::DidChangeTextSizeAdjust(self);
    }
    fn DidChangeTextOrientation(&mut self) {
        FontBuilder::DidChangeTextOrientation(self);
    }
}
