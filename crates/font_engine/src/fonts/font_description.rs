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

// cpp: platform/fonts/font_description.h:152-174.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VariantLigatures {
    pub common: LigaturesState,
    pub discretionary: LigaturesState,
    pub historical: LigaturesState,
    pub contextual: LigaturesState,
}
impl VariantLigatures {
    pub fn new(state: LigaturesState) -> Self {
        Self {
            common: state,
            discretionary: state,
            historical: state,
            contextual: state,
        }
    }
}
impl Default for VariantLigatures {
    fn default() -> Self {
        Self::new(LigaturesState::kNormalLigaturesState)
    }
}
// cpp: platform/fonts/font_description.h:120-124.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontVariantPosition {
    kNormalVariantPosition,
    kSubVariantPosition,
    kSuperVariantPosition,
}

// cpp: platform/fonts/font_description.h:107-119.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSynthesisWeight {
    kAutoFontSynthesisWeight,
    kNoneFontSynthesisWeight,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSynthesisStyle {
    kAutoFontSynthesisStyle,
    kNoneFontSynthesisStyle,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSynthesisSmallCaps {
    kAutoFontSynthesisSmallCaps,
    kNoneFontSynthesisSmallCaps,
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
            && self.font_palette_ == other.font_palette_
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

    // cpp: platform/fonts/font_description.h:260,288-290,349-354.
    pub fn GetKerning(&self) -> Kerning {
        match self.fields_.get(11, 2) {
            0 => Kerning::kAutoKerning,
            1 => Kerning::kNormalKerning,
            2 => Kerning::kNoneKerning,
            _ => unreachable!(),
        }
    }
    pub fn FontOpticalSizing(&self) -> super::font_optical_sizing::OpticalSizing {
        use super::font_optical_sizing::OpticalSizing;
        if self.fields_.get(54, 1) == 0 {
            OpticalSizing::kAutoOpticalSizing
        } else {
            OpticalSizing::kNoneOpticalSizing
        }
    }
    pub fn FeatureSettings(&self) -> Option<&FontFeatureSettings> {
        self.feature_settings_.as_ref()
    }
    pub fn VariationSettings(&self) -> Option<&FontVariationSettings> {
        self.variation_settings_.as_ref()
    }
    // cpp: platform/fonts/font_description.h:397-400,408-410,445-452.
    pub fn SetKerning(&mut self, kerning: Kerning) {
        self.fields_.set(11, 2, kerning as u64);
        self.UpdateTypesettingFeatures();
    }
    pub fn SetFontOpticalSizing(&mut self, sizing: super::font_optical_sizing::OpticalSizing) {
        self.fields_.set(54, 1, sizing as u64);
    }
    pub fn SetFeatureSettings(&mut self, settings: Option<FontFeatureSettings>) {
        self.feature_settings_ = settings;
    }
    pub fn SetVariationSettings(&mut self, settings: Option<FontVariationSettings>) {
        self.variation_settings_ = settings;
    }

    // cpp: platform/fonts/font_description.h:238-240,264-270,292-294,355-357;
    // platform/fonts/font_description.cc:233-269. Original bitfield positions.
    // cpp: platform/fonts/font_description.h:244,384.
    pub fn GetStyleSyntax(&self) -> StyleSyntax {
        self.style_syntax_
    }
    pub fn SetStyleSyntax(&mut self, value: StyleSyntax) {
        self.style_syntax_ = value;
    }
    pub fn VariantCaps(&self) -> FontVariantCaps {
        match self.fields_.get(CAPS, 3) {
            0 => FontVariantCaps::kCapsNormal,
            1 => FontVariantCaps::kSmallCaps,
            2 => FontVariantCaps::kAllSmallCaps,
            3 => FontVariantCaps::kPetiteCaps,
            4 => FontVariantCaps::kAllPetiteCaps,
            5 => FontVariantCaps::kUnicase,
            6 => FontVariantCaps::kTitlingCaps,
            _ => unreachable!(),
        }
    }
    pub fn SetVariantCaps(&mut self, value: FontVariantCaps) {
        self.fields_.set(CAPS, 3, value as u64);
        self.UpdateTypesettingFeatures();
    }
    pub fn GetVariantLigatures(&self) -> VariantLigatures {
        let state = |shift| match self.fields_.get(shift, 2) {
            0 => LigaturesState::kNormalLigaturesState,
            1 => LigaturesState::kDisabledLigaturesState,
            2 => LigaturesState::kEnabledLigaturesState,
            _ => unreachable!(),
        };
        VariantLigatures {
            common: state(13),
            discretionary: state(15),
            historical: state(17),
            contextual: state(19),
        }
    }
    pub fn SetVariantLigatures(&mut self, value: &VariantLigatures) {
        for (shift, state) in [
            (13, value.common),
            (15, value.discretionary),
            (17, value.historical),
            (19, value.contextual),
        ] {
            self.fields_.set(shift, 2, state as u64);
        }
        self.UpdateTypesettingFeatures();
    }
    pub fn VariantNumeric(&self) -> super::font_variant_numeric::FontVariantNumeric {
        super::font_variant_numeric::FontVariantNumeric::InitializeFromUnsigned(
            self.fields_.get(39, 8) as u32,
        )
    }
    pub fn SetVariantNumeric(&mut self, value: &super::font_variant_numeric::FontVariantNumeric) {
        self.fields_.set(39, 8, value.FieldsAsUnsigned() as u64);
        self.UpdateTypesettingFeatures();
    }
    pub fn VariantEastAsian(&self) -> super::font_variant_east_asian::FontVariantEastAsian {
        super::font_variant_east_asian::FontVariantEastAsian::InitializeFromUnsigned(
            self.fields_.get(47, 6) as u32,
        )
    }
    pub fn SetVariantEastAsian(
        &mut self,
        value: super::font_variant_east_asian::FontVariantEastAsian,
    ) {
        self.fields_.set(47, 6, value.FieldsAsUnsigned() as u64);
    }
    pub fn VariantPosition(&self) -> FontVariantPosition {
        match self.fields_.get(56, 2) {
            0 => FontVariantPosition::kNormalVariantPosition,
            1 => FontVariantPosition::kSubVariantPosition,
            2 => FontVariantPosition::kSuperVariantPosition,
            _ => unreachable!(),
        }
    }
    pub fn SetVariantPosition(&mut self, value: FontVariantPosition) {
        self.fields_.set(56, 2, value as u64);
    }
    // cpp: font_description.h:309-324,435-444,582-584. Original bit order.
    pub fn GetFontSynthesisWeight(&self) -> FontSynthesisWeight {
        if self.fields_.get(32, 1) == 0 {
            FontSynthesisWeight::kAutoFontSynthesisWeight
        } else {
            FontSynthesisWeight::kNoneFontSynthesisWeight
        }
    }
    pub fn GetFontSynthesisStyle(&self) -> FontSynthesisStyle {
        if self.fields_.get(33, 1) == 0 {
            FontSynthesisStyle::kAutoFontSynthesisStyle
        } else {
            FontSynthesisStyle::kNoneFontSynthesisStyle
        }
    }
    pub fn GetFontSynthesisSmallCaps(&self) -> FontSynthesisSmallCaps {
        if self.fields_.get(34, 1) == 0 {
            FontSynthesisSmallCaps::kAutoFontSynthesisSmallCaps
        } else {
            FontSynthesisSmallCaps::kNoneFontSynthesisSmallCaps
        }
    }
    pub fn SetFontSynthesisWeight(&mut self, value: FontSynthesisWeight) {
        self.fields_.set(32, 1, value as u64);
    }
    pub fn SetFontSynthesisStyle(&mut self, value: FontSynthesisStyle) {
        self.fields_.set(33, 1, value as u64);
    }
    pub fn SetFontSynthesisSmallCaps(&mut self, value: FontSynthesisSmallCaps) {
        self.fields_.set(34, 1, value as u64);
    }
    pub fn FontLanguageOverride(&self) -> &AtomicString {
        &self.language_override_
    }
    pub fn SetFontLanguageOverride(&mut self, value: AtomicString) {
        self.language_override_ = value;
    }
    pub fn FontVariantAlternatesValue(&self) -> Option<&Arc<FontVariantAlternates>> {
        self.font_variant_alternates_.as_ref()
    }
    pub fn SetFontVariantAlternates(&mut self, value: Option<Arc<FontVariantAlternates>>) {
        self.font_variant_alternates_ = value;
    }

    // cpp: font_engine/fonts/font_description.h:274-279
    pub fn GetFontPalette(&self) -> *const FontPalette {
        self.font_palette_
            .as_ref()
            .map_or(std::ptr::null(), Arc::as_ptr)
    }
    // cpp: font_description.h:291,411-413; scoped_refptr inheritance retains
    // actual palette ownership without manufacturing a font selector provider.
    pub fn FontPaletteValue(&self) -> Option<Arc<FontPalette>> {
        self.font_palette_.clone()
    }
    pub fn SetFontPalette(&mut self, value: Option<Arc<FontPalette>>) {
        self.font_palette_ = value;
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

    // cpp: platform/fonts/font_description.h:401-403
    pub fn SetTextSpacingTrim(
        &mut self,
        value: super::shaping::text_spacing_trim::TextSpacingTrim,
    ) {
        self.fields_.set(TEXT_SPACING_TRIM, 2, value as u64);
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

    // cpp: font_engine/fonts/font_description.h:361-363
    pub fn VariantEmoji(&self) -> FontVariantEmoji {
        match self.fields_.get(VARIANT_EMOJI, 2) {
            0 => FontVariantEmoji::kNormalVariantEmoji,
            1 => FontVariantEmoji::kTextVariantEmoji,
            2 => FontVariantEmoji::kEmojiVariantEmoji,
            3 => FontVariantEmoji::kUnicodeVariantEmoji,
            _ => unreachable!("font variant emoji is stored in two bits"),
        }
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
    // cpp: font_description.h Size/FamilyDescription accessors and bitfields.
    pub fn IsAbsoluteSize(&self) -> bool {
        self.fields_.get(7, 1) != 0
    }
    pub fn SetIsAbsoluteSize(&mut self, value: bool) {
        self.fields_.set(7, 1, value as u64);
    }
    pub fn KeywordSize(&self) -> u32 {
        self.fields_.get(21, 4) as u32
    }
    pub fn SetKeywordSize(&mut self, value: u32) {
        self.fields_.set(21, 4, value as u64);
    }
    pub fn GenericFamily(&self) -> GenericFamilyType {
        use GenericFamilyType::*;
        match self.fields_.get(8, 3) {
            0 => kNoFamily,
            1 => kStandardFamily,
            2 => kWebkitBodyFamily,
            3 => kSerifFamily,
            4 => kSansSerifFamily,
            5 => kMonospaceFamily,
            6 => kCursiveFamily,
            _ => kFantasyFamily,
        }
    }
    pub fn SetGenericFamily(&mut self, value: GenericFamilyType) {
        self.fields_.set(8, 3, value as u64);
    }
    pub fn IsMonospace(&self) -> bool {
        self.GenericFamily() == GenericFamilyType::kMonospaceFamily
    }
    // cpp: font_description.h SetTextRendering also updates typesetting features.
    pub fn SetTextRendering(&mut self, value: TextRenderingMode) {
        self.fields_.set(27, 2, value as u64);
        self.UpdateTypesettingFeatures();
    }

    // cpp: font_description.h:467-469.
    pub fn GetTypesettingFeatures(&self) -> TypesettingFeatures {
        self.fields_.get(TYPESETTING_FEATURES, 3) as TypesettingFeatures
    }
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

#[cfg(test)]
mod core_font_input_tests {
    use super::*;
    #[test]
    fn font_metadata_uses_source_bitfields_and_affects_equality() {
        let original = FontDescription::default();
        let mut d = original.clone();
        d.SetKeywordSize(8);
        d.SetGenericFamily(GenericFamilyType::kMonospaceFamily);
        d.SetIsAbsoluteSize(true);
        d.SetFontSmoothing(FontSmoothingMode::kAntialiased);
        assert_eq!(d.KeywordSize(), 8);
        assert!(d.IsMonospace() && d.IsAbsoluteSize());
        assert_eq!(d.FontSmoothing(), FontSmoothingMode::kAntialiased);
        assert_eq!(d.GetVariantLigatures(), original.GetVariantLigatures());
        assert!(d != original);
        let inherited = d.clone();
        assert!(inherited == d);
    }
    #[test]
    fn text_rendering_updates_native_typesetting_features() {
        let mut d = FontDescription::default();
        d.SetTextRendering(TextRenderingMode::kOptimizeLegibility);
        assert_eq!(d.TextRendering(), TextRenderingMode::kOptimizeLegibility);
        assert_eq!(
            d.GetTypesettingFeatures() & (kKerning | kLigatures),
            kKerning | kLigatures
        );
        d.SetKerning(Kerning::kNoneKerning);
        assert_eq!(d.GetTypesettingFeatures() & kKerning, 0);
        d.SetTextRendering(TextRenderingMode::kOptimizeSpeed);
        assert_eq!(d.GetTypesettingFeatures() & kLigatures, 0);
    }
}
