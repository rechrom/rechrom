// C++: platform/fonts/font_palette.h and font_palette.cc:83-104. Native request
// ownership, inline accessors and equality; ToString/GetHash remain untranslated.
use foundation::{AtomicString, Color, ColorSpace, HueInterpolationMethod};
use std::sync::Arc;

// cpp: font_engine/fonts/font_palette.h:28-63
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum KeywordPaletteName {
    #[default]
    kNormalPalette = 0,
    kLightPalette = 1,
    kDarkPalette = 2,
    kCustomPalette = 3,
    kInterpolablePalette = 4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontPaletteOverride {
    pub index: u16,
    pub color: Color,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BasePaletteValueType {
    #[default]
    kNoBasePalette,
    kLightBasePalette,
    kDarkBasePalette,
    kIndexBasePalette,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BasePaletteValue {
    pub r#type: BasePaletteValueType,
    pub index: i32,
}

impl BasePaletteValue {
    pub fn hasValue(&self) -> bool {
        self.r#type != BasePaletteValueType::kNoBasePalette
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NonNormalizedPercentages {
    pub start: f64,
    pub end: f64,
}

// cpp: font_engine/fonts/font_palette.h:67-228
pub struct FontPalette {
    palette_keyword_: KeywordPaletteName,
    palette_values_name_: AtomicString,
    base_palette_: BasePaletteValue,
    match_font_family_: AtomicString,
    palette_overrides_: Vec<FontPaletteOverride>,
    start_: Option<Arc<FontPalette>>,
    end_: Option<Arc<FontPalette>>,
    percentages_: NonNormalizedPercentages,
    normalized_percentage_: f64,
    alpha_multiplier_: f64,
    color_interpolation_space_: ColorSpace,
    hue_interpolation_method_: Option<HueInterpolationMethod>,
}

impl Default for FontPalette {
    fn default() -> Self {
        Self {
            palette_keyword_: KeywordPaletteName::kNormalPalette,
            palette_values_name_: AtomicString::default(),
            base_palette_: BasePaletteValue::default(),
            match_font_family_: AtomicString::default(),
            palette_overrides_: Vec::new(),
            start_: None,
            end_: None,
            percentages_: NonNormalizedPercentages::default(),
            normalized_percentage_: 0.0,
            alpha_multiplier_: 0.0,
            color_interpolation_space_: ColorSpace::kSRGB,
            hue_interpolation_method_: None,
        }
    }
}

impl FontPalette {
    // cpp: font_engine/fonts/font_palette.h:72-103,201-215
    pub fn Create() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub fn CreateKeyword(palette_name: KeywordPaletteName) -> Arc<Self> {
        debug_assert!(palette_name != KeywordPaletteName::kCustomPalette);
        Arc::new(Self {
            palette_keyword_: palette_name,
            ..Self::default()
        })
    }
    pub fn CreateCustom(palette_values_name: AtomicString) -> Arc<Self> {
        Arc::new(Self {
            palette_keyword_: KeywordPaletteName::kCustomPalette,
            palette_values_name_: palette_values_name,
            ..Self::default()
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn Mix(
        start: Arc<FontPalette>,
        end: Arc<FontPalette>,
        start_percentage: f64,
        end_percentage: f64,
        normalized_percentage: f64,
        alpha_multiplier: f64,
        color_interpolation_space: ColorSpace,
        hue_interpolation_method: Option<HueInterpolationMethod>,
    ) -> Arc<Self> {
        Arc::new(Self {
            palette_keyword_: KeywordPaletteName::kInterpolablePalette,
            start_: Some(start),
            end_: Some(end),
            percentages_: NonNormalizedPercentages {
                start: start_percentage,
                end: end_percentage,
            },
            normalized_percentage_: normalized_percentage,
            alpha_multiplier_: alpha_multiplier,
            color_interpolation_space_: color_interpolation_space,
            hue_interpolation_method_: hue_interpolation_method,
            ..Self::default()
        })
    }

    // cpp: font_engine/fonts/font_palette.h:106-188
    pub fn SetBasePalette(&mut self, base_palette: BasePaletteValue) {
        self.base_palette_ = base_palette;
    }
    pub fn SetColorOverrides(&mut self, overrides: Vec<FontPaletteOverride>) {
        self.palette_overrides_ = overrides;
    }
    pub fn IsNormalPalette(&self) -> bool {
        self.palette_keyword_ == KeywordPaletteName::kNormalPalette
    }
    pub fn IsCustomPalette(&self) -> bool {
        self.palette_keyword_ == KeywordPaletteName::kCustomPalette
    }
    pub fn IsInterpolablePalette(&self) -> bool {
        self.palette_keyword_ == KeywordPaletteName::kInterpolablePalette
    }
    pub fn GetPaletteNameKind(&self) -> KeywordPaletteName {
        self.palette_keyword_
    }
    pub fn GetPaletteValuesName(&self) -> &AtomicString {
        debug_assert!(self.IsCustomPalette());
        &self.palette_values_name_
    }
    pub fn GetColorOverrides(&self) -> &[FontPaletteOverride] {
        &self.palette_overrides_
    }
    pub fn GetBasePalette(&self) -> BasePaletteValue {
        self.base_palette_
    }
    pub fn SetMatchFamilyName(&mut self, family_name: AtomicString) {
        self.match_font_family_ = family_name;
    }
    pub fn GetMatchFamilyName(&self) -> AtomicString {
        self.match_font_family_.clone()
    }
    pub fn GetStart(&self) -> Arc<FontPalette> {
        debug_assert!(self.IsInterpolablePalette());
        self.start_
            .as_ref()
            .expect("interpolable palette has no start")
            .clone()
    }
    pub fn GetEnd(&self) -> Arc<FontPalette> {
        debug_assert!(self.IsInterpolablePalette());
        self.end_
            .as_ref()
            .expect("interpolable palette has no end")
            .clone()
    }
    pub fn GetStartPercentage(&self) -> f64 {
        debug_assert!(self.IsInterpolablePalette());
        self.percentages_.start
    }
    pub fn GetEndPercentage(&self) -> f64 {
        debug_assert!(self.IsInterpolablePalette());
        self.percentages_.end
    }
    pub fn GetNormalizedPercentage(&self) -> f64 {
        debug_assert!(self.IsInterpolablePalette());
        self.normalized_percentage_
    }
    pub fn ComputeEndpointPercentagesFromNormalized(
        normalized_percentage: f64,
    ) -> NonNormalizedPercentages {
        let end_percentage = normalized_percentage * 100.0;
        let start_percentage = 100.0 - end_percentage;
        NonNormalizedPercentages {
            start: start_percentage,
            end: end_percentage,
        }
    }
    pub fn GetAlphaMultiplier(&self) -> f64 {
        debug_assert!(self.IsInterpolablePalette());
        self.alpha_multiplier_
    }
    pub fn GetColorInterpolationSpace(&self) -> ColorSpace {
        debug_assert!(self.IsInterpolablePalette());
        self.color_interpolation_space_
    }
    pub fn GetHueInterpolationMethod(&self) -> Option<HueInterpolationMethod> {
        debug_assert!(self.IsInterpolablePalette());
        self.hue_interpolation_method_
    }
}

// cpp: platform/fonts/font_palette.cc:83-104, ValuesEquivalent in
// font_description.cc:154. Equality includes actual custom data and mixes.
impl PartialEq for FontPalette {
    fn eq(&self, other: &Self) -> bool {
        if self.IsInterpolablePalette() != other.IsInterpolablePalette() {
            return false;
        }
        if self.IsInterpolablePalette() {
            return self.start_ == other.start_
                && self.end_ == other.end_
                && self.percentages_ == other.percentages_
                && self.normalized_percentage_ == other.normalized_percentage_
                && self.alpha_multiplier_ == other.alpha_multiplier_
                && self.color_interpolation_space_ == other.color_interpolation_space_
                && self.hue_interpolation_method_ == other.hue_interpolation_method_;
        }
        self.palette_keyword_ == other.palette_keyword_
            && self.palette_values_name_ == other.palette_values_name_
            && self.match_font_family_ == other.match_font_family_
            && self.base_palette_ == other.base_palette_
            && self.palette_overrides_ == other.palette_overrides_
    }
}
