// C++: font_engine/fonts/font_description.h and font_description_data.cc.
// This maps the source's stored state and the supplied size, spacing, style,
// and typesetting behavior. The source declares additional methods without
// definitions; this file remains blocked until those interfaces connect.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use foundation::{AtomicString, EvaluationInput, Length, LengthType};

use super::font_family::FontFamily;
use super::font_orientation::{FontOrientation, IsVerticalAnyUpright};
use super::font_palette::FontPalette;
use super::font_selection_types::{
    kNormalSlopeValue, kNormalWeightValue, kNormalWidthValue, FontSelectionRequest,
    FontSelectionValue,
};
use super::font_size_adjust::FontSizeAdjust;
use super::font_smoothing_mode::FontSmoothingMode;
use super::font_variant_alternates::FontVariantAlternates;
use super::font_variant_emoji::FontVariantEmoji;
use super::font_width_variant::FontWidthVariant;
use super::opentype::font_settings::{FontFeatureSettings, FontVariationSettings};
use super::resolved_font_features::ResolvedFontFeatures;
use super::text_rendering_mode::TextRenderingMode;
use super::typesetting_features::{kCaps, kKerning, kLigatures, TypesettingFeatures};
use crate::LayoutLocale;

// cpp: font_engine/fonts/font_description.h:67-69
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashCategory {
    kHashEmptyValue,
    kHashDeletedValue,
    kHashRegularValue,
}

// cpp: font_engine/fonts/font_description.h:71-80
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericFamilyType {
    kNoFamily,
    kStandardFamily,
    kWebkitBodyFamily,
    kSerifFamily,
    kSansSerifFamily,
    kMonospaceFamily,
    kCursiveFamily,
    kFantasyFamily,
}

// cpp: font_engine/fonts/font_description.h:83-87
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LigaturesState {
    kNormalLigaturesState,
    kDisabledLigaturesState,
    kEnabledLigaturesState,
}

// cpp: font_engine/fonts/font_description.h:90
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kerning {
    kAutoKerning,
    kNormalKerning,
    kNoneKerning,
}

// cpp: font_engine/fonts/font_description.h:94-103
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontVariantCaps {
    kCapsNormal,
    kSmallCaps,
    kAllSmallCaps,
    kPetiteCaps,
    kAllPetiteCaps,
    kUnicase,
    kTitlingCaps,
}

// cpp: font_engine/fonts/font_description.h:130-139
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleSyntax {
    kItalicKeyword,
    kImplicitAngle,
    kExplicitAngle,
}

// cpp: font_engine/fonts/font_description.h:548-604
// The C++ union has two 32-bit parts. Each translated field occupies its
// original width and order within this 64-bit value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BitFields(u64);

impl BitFields {
    fn get(self, shift: u32, width: u32) -> u64 {
        (self.0 >> shift) & ((1u64 << width) - 1)
    }
    fn set(&mut self, shift: u32, width: u32, value: u64) {
        let mask = ((1u64 << width) - 1) << shift;
        self.0 = (self.0 & !mask) | ((value << shift) & mask);
    }
}

const ORIENTATION: u32 = 0;
const WIDTH_VARIANT: u32 = 2;
const CAPS: u32 = 4;
const FONT_SMOOTHING: u32 = 25;
const SYNTHETIC_OBLIQUE: u32 = 31;
const SUBPIXEL_POSITIONING: u32 = 35;
const TYPESETTING_FEATURES: u32 = 36;
const SIZE_ADJUST_DESCRIPTOR: u32 = 55;
const VARIANT_EMOJI: u32 = 58;
const TEXT_SPACING_TRIM: u32 = 60;
const HASH_CATEGORY: u32 = 62;

static USE_SUBPIXEL_TEXT_POSITIONING: AtomicBool = AtomicBool::new(false);

// cpp: font_engine/fonts/font_description.h:60-61,514-607
#[derive(Clone)]
pub struct FontDescription {
    family_list_: FontFamily,
    feature_settings_: Option<FontFeatureSettings>,
    variation_settings_: Option<FontVariationSettings>,
    locale_: Option<Arc<LayoutLocale>>,
    font_palette_: Option<Arc<FontPalette>>,
    font_variant_alternates_: Option<Arc<FontVariantAlternates>>,
    specified_size_: f32,
    computed_size_: f32,
    adjusted_size_: f32,
    letter_spacing_: Length,
    word_spacing_: Length,
    size_adjust_: FontSizeAdjust,
    resolved_font_features_: ResolvedFontFeatures,
    font_selection_request_: FontSelectionRequest,
    original_slope: FontSelectionValue,
    style_syntax_: StyleSyntax,
    fields_: BitFields,
    language_override_: AtomicString,
}

// cpp: font_engine/fonts/font_description_data.cc:64-105
impl Default for FontDescription {
    fn default() -> Self {
        let mut fields = BitFields::default();
        fields.set(ORIENTATION, 2, FontOrientation::kHorizontal as u64);
        fields.set(
            SUBPIXEL_POSITIONING,
            1,
            USE_SUBPIXEL_TEXT_POSITIONING.load(Ordering::Relaxed) as u64,
        );
        fields.set(HASH_CATEGORY, 2, HashCategory::kHashRegularValue as u64);
        Self {
            family_list_: FontFamily::default(),
            feature_settings_: None,
            variation_settings_: None,
            locale_: None,
            font_palette_: None,
            font_variant_alternates_: None,
            specified_size_: 0.0,
            computed_size_: 0.0,
            adjusted_size_: 0.0,
            letter_spacing_: Length::Fixed(0.0),
            word_spacing_: Length::Fixed(0.0),
            size_adjust_: FontSizeAdjust::default(),
            resolved_font_features_: Vec::new(),
            font_selection_request_: FontSelectionRequest::new(
                kNormalWeightValue,
                kNormalWidthValue,
                kNormalSlopeValue,
            ),
            original_slope: kNormalSlopeValue,
            style_syntax_: StyleSyntax::kImplicitAngle,
            fields_: fields,
            language_override_: AtomicString::default(),
        }
    }
}

fn same_arc<T>(left: &Option<Arc<T>>, right: &Option<Arc<T>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

// cpp: font_engine/fonts/font_description_data.cc:109-125
impl PartialEq for FontDescription {
    fn eq(&self, other: &Self) -> bool {
        self.family_list_ == other.family_list_
            && same_arc(&self.locale_, &other.locale_)
            && self.specified_size_ == other.specified_size_
            && self.computed_size_ == other.computed_size_
            && self.adjusted_size_ == other.adjusted_size_
            && self.size_adjust_ == other.size_adjust_
            && self.letter_spacing_ == other.letter_spacing_
            && self.word_spacing_ == other.word_spacing_
            && self.font_selection_request_ == other.font_selection_request_
            && self.style_syntax_ == other.style_syntax_
            && self.fields_ == other.fields_
            && self.feature_settings_ == other.feature_settings_
            && self.variation_settings_ == other.variation_settings_
            && same_arc(&self.font_palette_, &other.font_palette_)
            && same_arc(
                &self.font_variant_alternates_,
                &other.font_variant_alternates_,
            )
    }
}

// cpp: font_engine/fonts/font_description_data.cc:36
#[allow(non_snake_case)]
impl FontDescription {
    pub fn SetSubpixelPositioning(value: bool) {
        USE_SUBPIXEL_TEXT_POSITIONING.store(value, Ordering::Relaxed);
    }
    pub fn SubpixelPositioning() -> bool {
        USE_SUBPIXEL_TEXT_POSITIONING.load(Ordering::Relaxed)
    }

    // cpp: font_engine/fonts/font_description.h:193-216
    pub fn Family(&self) -> &FontFamily {
        &self.family_list_
    }
    pub fn SpecifiedSize(&self) -> f32 {
        self.specified_size_
    }
    pub fn ComputedSize(&self) -> f32 {
        self.computed_size_
    }
    pub fn AdjustedSize(&self) -> f32 {
        self.adjusted_size_
    }
    pub fn SizeAdjust(&self) -> FontSizeAdjust {
        self.size_adjust_
    }
    pub fn HasSizeAdjust(&self) -> bool {
        self.size_adjust_.IsSet()
    }
    pub fn ComputedPixelSize(&self) -> i32 {
        (self.computed_size_ + 0.5) as i32
    }

    // cpp: font_engine/fonts/font_description.h:221-232
    pub fn Weight(&self) -> FontSelectionValue {
        self.font_selection_request_.weight
    }
    pub fn Style(&self) -> FontSelectionValue {
        self.font_selection_request_.slope
    }
    pub fn Stretch(&self) -> FontSelectionValue {
        self.font_selection_request_.width
    }

    // cpp: font_engine/fonts/font_description.h:274-279
    pub fn GetFontPalette(&self) -> *const FontPalette {
        self.font_palette_
            .as_ref()
            .map_or(std::ptr::null(), Arc::as_ptr)
    }

    // cpp: font_engine/fonts/font_description.h:261-263
    pub fn GetTextSpacingTrim(&self) -> super::shaping::text_spacing_trim::TextSpacingTrim {
        use super::shaping::text_spacing_trim::TextSpacingTrim;
        match self.fields_.get(TEXT_SPACING_TRIM, 2) {
            0 => TextSpacingTrim::kNormal,
            1 => TextSpacingTrim::kSpaceAll,
            2 => TextSpacingTrim::kSpaceFirst,
            _ => TextSpacingTrim::kTrimStart,
        }
    }

    // cpp: font_engine/fonts/font_description.h:283-287
    pub fn Locale(&self) -> *const LayoutLocale {
        self.locale_.as_ref().map_or(std::ptr::null(), Arc::as_ptr)
    }

    // cpp: font_engine/fonts/font_description.h:299-301
    pub fn LocaleOrDefault(&self) -> &LayoutLocale {
        unsafe { &*LayoutLocale::ValueOrDefault(self.Locale()) }
    }

    // cpp: font_engine/fonts/font_description.h:317-322
    pub fn ComputedWordSpacing(&self) -> &Length {
        &self.word_spacing_
    }
    pub fn ComputedLetterSpacing(&self) -> &Length {
        &self.letter_spacing_
    }

    // cpp: font_engine/fonts/font_description.h:328-330
    pub fn Orientation(&self) -> FontOrientation {
        match self.fields_.get(ORIENTATION, 2) {
            0 => FontOrientation::kHorizontal,
            1 => FontOrientation::kVerticalRotated,
            2 => FontOrientation::kVerticalMixed,
            _ => FontOrientation::kVerticalUpright,
        }
    }

    // cpp: font_engine/fonts/font_description.h:346-348
    pub fn WidthVariant(&self) -> FontWidthVariant {
        match self.fields_.get(WIDTH_VARIANT, 2) {
            0 => FontWidthVariant::kRegularWidth,
            1 => FontWidthVariant::kHalfWidth,
            2 => FontWidthVariant::kThirdWidth,
            _ => FontWidthVariant::kQuarterWidth,
        }
    }

    // cpp: font_engine/fonts/font_description.h:423-425
    pub fn SetWidthVariant(&mut self, variant: FontWidthVariant) {
        self.fields_.set(WIDTH_VARIANT, 2, variant as u64);
    }
    pub fn IsVerticalAnyUpright(&self) -> bool {
        IsVerticalAnyUpright(self.Orientation())
    }

    // cpp: font_engine/fonts/font_description.h:305-305
    pub fn IsSyntheticOblique(&self) -> bool {
        self.fields_.get(SYNTHETIC_OBLIQUE, 1) != 0
    }

    // cpp: font_engine/fonts/font_description.h:284-287
    pub fn FontSmoothing(&self) -> FontSmoothingMode {
        match self.fields_.get(FONT_SMOOTHING, 2) {
            0 => FontSmoothingMode::kAutoSmoothing,
            1 => FontSmoothingMode::kNoSmoothing,
            2 => FontSmoothingMode::kAntialiased,
            _ => FontSmoothingMode::kSubpixelAntialiased,
        }
    }

    // cpp: font_engine/fonts/font_description.h:379-391
    pub fn SetFamily(&mut self, family: &FontFamily) {
        self.family_list_ = family.clone();
    }
    pub fn SetComputedSize(&mut self, size: f32) {
        self.computed_size_ = clamp_float(size);
    }
    pub fn SetSpecifiedSize(&mut self, size: f32) {
        self.specified_size_ = clamp_float(size);
    }
    pub fn SetAdjustedSize(&mut self, size: f32) {
        self.adjusted_size_ = clamp_float(size);
    }
    pub fn SetSizeAdjust(&mut self, size_adjust: &FontSizeAdjust) {
        self.size_adjust_ = *size_adjust;
    }

    // cpp: font_engine/fonts/font_description.h:399-404
    pub fn SetWeight(&mut self, value: FontSelectionValue) {
        self.font_selection_request_.weight = value;
    }
    pub fn SetStretch(&mut self, value: FontSelectionValue) {
        self.font_selection_request_.width = value;
    }

    // cpp: font_engine/fonts/font_description_data.cc:147-161
    pub fn SetOrientation(&mut self, orientation: FontOrientation) {
        self.fields_.set(ORIENTATION, 2, orientation as u64);
        self.UpdateSyntheticOblique();
    }
    pub fn SetStyle(&mut self, style: FontSelectionValue) {
        self.original_slope = style;
        self.UpdateSyntheticOblique();
    }
    // cpp: font_engine/fonts/font_description.h:405-407
    pub fn SetFontSmoothing(&mut self, smoothing: FontSmoothingMode) {
        self.fields_.set(FONT_SMOOTHING, 2, smoothing as u64);
    }
    fn UpdateSyntheticOblique(&mut self) {
        let synthetic_oblique =
            self.IsVerticalAnyUpright() && self.original_slope < FontSelectionValue::from_int(0);
        self.fields_
            .set(SYNTHETIC_OBLIQUE, 1, synthetic_oblique as u64);
        self.font_selection_request_.slope = if synthetic_oblique {
            kNormalSlopeValue
        } else {
            self.original_slope
        };
    }

    // cpp: font_engine/fonts/font_description.h:430-432
    pub fn SetLocale(&mut self, locale: Option<Arc<LayoutLocale>>) {
        self.locale_ = locale;
    }

    // cpp: font_engine/fonts/font_description.h:459-463
    pub fn SetVariantEmoji(&mut self, emoji: FontVariantEmoji) {
        self.fields_.set(VARIANT_EMOJI, 2, emoji as u64);
    }
    pub fn SetWordSpacing(&mut self, spacing: &Length) {
        self.word_spacing_ = spacing.clone();
    }
    pub fn SetLetterSpacing(&mut self, spacing: &Length) {
        self.letter_spacing_ = spacing.clone();
        self.UpdateTypesettingFeatures();
    }

    // cpp: font_engine/fonts/font_description_data.cc:38-49
    pub fn EffectiveFontSize(&self) -> f32 {
        let size = if self.HasSizeAdjust() || self.fields_.get(SIZE_ADJUST_DESCRIPTOR, 1) != 0 {
            self.AdjustedSize()
        } else {
            self.ComputedSize()
        };
        (size * 100.0).floor() / 100.0
    }

    // cpp: font_engine/fonts/font_description_data.cc:127-145
    pub fn LetterSpacing(&self) -> f32 {
        spacing_value(&self.letter_spacing_, self.computed_size_)
    }
    pub fn WordSpacing(&self) -> f32 {
        spacing_value(&self.word_spacing_, self.computed_size_)
    }

    // cpp: font_engine/fonts/font_description.h:295-297
    pub fn TextRendering(&self) -> TextRenderingMode {
        match self.fields_.get(27, 2) {
            0 => TextRenderingMode::kAutoTextRendering,
            1 => TextRenderingMode::kOptimizeSpeed,
            2 => TextRenderingMode::kOptimizeLegibility,
            3 => TextRenderingMode::kGeometricPrecision,
            _ => unreachable!(),
        }
    }

    // cpp: font_engine/fonts/font_description_data.cc:164-184
    fn UpdateTypesettingFeatures(&mut self) {
        let mut features: TypesettingFeatures = 0;
        let rendering = self.fields_.get(27, 2);
        if rendering == TextRenderingMode::kGeometricPrecision as u64
            || rendering == TextRenderingMode::kOptimizeLegibility as u64
        {
            features |= kKerning | kLigatures;
        }
        let kerning = self.fields_.get(11, 2);
        if kerning == Kerning::kNoneKerning as u64 {
            features &= !kKerning;
        } else if kerning == Kerning::kNormalKerning as u64 {
            features |= kKerning;
        }
        if self.letter_spacing_.IsZero() {
            if self.fields_.get(13, 2) == LigaturesState::kDisabledLigaturesState as u64 {
                features &= !kLigatures;
            } else if self.fields_.get(13, 2) == LigaturesState::kEnabledLigaturesState as u64 {
                features |= kLigatures;
            }
            if [15, 17, 19].iter().any(|shift| {
                self.fields_.get(*shift, 2) == LigaturesState::kEnabledLigaturesState as u64
            }) {
                features |= kLigatures;
            }
        }
        if self.fields_.get(CAPS, 3) != FontVariantCaps::kCapsNormal as u64 {
            features |= kCaps;
        }
        self.fields_.set(TYPESETTING_FEATURES, 3, features as u64);
    }
}

fn clamp_float(value: f32) -> f32 {
    if value >= f32::MAX {
        f32::MAX
    } else if value <= f32::MIN {
        f32::MIN
    } else {
        value
    }
}

fn spacing_value(spacing: &Length, computed_size: f32) -> f32 {
    match spacing.GetType() {
        LengthType::kFixed => spacing.Pixels(),
        LengthType::kPercent => spacing.PercentValue() / 100.0 * computed_size,
        LengthType::kCalculated => {
            spacing.NonNanCalculatedValue(computed_size, &EvaluationInput::default())
        }
        _ => panic!("invalid font spacing Length type"),
    }
}

// The inline algorithm calls the source-owned bitfield setter across the
// font/layout package boundary.
// cpp: font_engine/fonts/font_description.h:423-425
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn FontDescriptionSetWidthVariantForInline(
    description: &mut FontDescription,
    variant: FontWidthVariant,
) {
    description.SetWidthVariant(variant);
}
