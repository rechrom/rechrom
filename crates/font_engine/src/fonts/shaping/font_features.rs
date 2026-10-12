// C++: font_engine/fonts/shaping/font_features.h/.cc.
use std::ptr::NonNull;

use crate::fonts::font_description::{
    FontDescription, FontVariantPosition, Kerning, LigaturesState,
};
use crate::fonts::font_variant_east_asian::{EastAsianForm, EastAsianWidth};
use crate::fonts::font_variant_numeric::{
    NumericFigure, NumericFraction, NumericSpacing, Ordinal, SlashedZero,
};
use crate::fonts::shaping::text_spacing_trim::ShouldTrimAdjacent;
use crate::fonts::text_rendering_mode::TextRenderingMode;

// cpp: font_engine/fonts/shaping/font_features.h:21-34
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontFeatureTag {
    pub tag: u32,
}

impl FontFeatureTag {
    pub const fn new(tag: u32) -> Self {
        Self { tag }
    }
    pub const fn from_chars(c1: u8, c2: u8, c3: u8, c4: u8) -> Self {
        Self {
            tag: (((((c1 as u32) << 8) | c2 as u32) << 8 | c3 as u32) << 8) | c4 as u32,
        }
    }
}

impl From<FontFeatureTag> for u32 {
    fn from(value: FontFeatureTag) -> Self {
        value.tag
    }
}

// cpp: font_engine/fonts/shaping/font_features.h:39-42
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontFeatureValue {
    pub tag: u32,
    pub value: u32,
}

// Matches the four uint32_t fields of hb_feature_t.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HarfBuzzFeature {
    pub tag: u32,
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

// cpp: font_engine/fonts/shaping/font_features.h:47-68
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontFeatureRange {
    pub tag: u32,
    pub value: u32,
    pub start: u32,
    pub end: u32,
}

impl Default for FontFeatureRange {
    fn default() -> Self {
        Self {
            tag: 0,
            value: 0,
            start: 0,
            end: u32::MAX,
        }
    }
}

impl FontFeatureRange {
    pub const kInitialSize: u32 = 1;
    pub const fn new(tag: FontFeatureTag, value: u32) -> Self {
        Self {
            tag: tag.tag,
            value,
            start: 0,
            end: u32::MAX,
        }
    }
    pub const fn ToHarfBuzzData(features: *const Self) -> *const HarfBuzzFeature {
        features as *const HarfBuzzFeature
    }

    // cpp: font_engine/fonts/shaping/font_features.cc:32-198
    pub fn FromFontDescription(description: &FontDescription, features: &mut FontFeatureRanges) {
        debug_assert!(features.is_empty());
        let horizontal = !description.IsVerticalAnyUpright();
        let tag = |bytes: &[u8; 4]| FontFeatureTag::new(u32::from_be_bytes(*bytes));
        let push = |features: &mut FontFeatureRanges, bytes: &[u8; 4], value: u32| {
            features.push(Self::new(tag(bytes), value));
        };

        if description.GetKerning() == Kerning::kNoneKerning {
            push(features, if horizontal { b"kern" } else { b"vkrn" }, 0);
        }

        let ligatures = description.GetVariantLigatures();
        let default_is_off = description.TextRendering() == TextRenderingMode::kOptimizeSpeed;
        let letter_spacing = description.LetterSpacing() != 0.0;
        if letter_spacing
            || ligatures.common == LigaturesState::kDisabledLigaturesState
            || (ligatures.common == LigaturesState::kNormalLigaturesState && default_is_off)
        {
            push(features, b"liga", 0);
            push(features, b"clig", 0);
        }
        if !letter_spacing && ligatures.discretionary == LigaturesState::kEnabledLigaturesState {
            push(features, b"dlig", 1);
        }
        if !letter_spacing && ligatures.historical == LigaturesState::kEnabledLigaturesState {
            push(features, b"hlig", 1);
        }
        if letter_spacing
            || ligatures.contextual == LigaturesState::kDisabledLigaturesState
            || (ligatures.contextual == LigaturesState::kNormalLigaturesState && default_is_off)
        {
            push(features, b"calt", 0);
        }

        match description.WidthVariant() {
            crate::fonts::font_width_variant::FontWidthVariant::kHalfWidth => {
                push(features, b"hwid", 1)
            }
            crate::fonts::font_width_variant::FontWidthVariant::kThirdWidth => {
                push(features, b"twid", 1)
            }
            crate::fonts::font_width_variant::FontWidthVariant::kQuarterWidth => {
                push(features, b"qwid", 1)
            }
            crate::fonts::font_width_variant::FontWidthVariant::kRegularWidth => {}
        }

        let east_asian = description.VariantEastAsian();
        if !east_asian.IsAllNormal() {
            let form = if east_asian.Form() == EastAsianForm::kJis78 {
                Some(b"jp78")
            } else if east_asian.Form() == EastAsianForm::kJis83 {
                Some(b"jp83")
            } else if east_asian.Form() == EastAsianForm::kJis90 {
                Some(b"jp90")
            } else if east_asian.Form() == EastAsianForm::kJis04 {
                Some(b"jp04")
            } else if east_asian.Form() == EastAsianForm::kSimplified {
                Some(b"smpl")
            } else if east_asian.Form() == EastAsianForm::kTraditional {
                Some(b"trad")
            } else {
                None
            };
            if let Some(form) = form {
                push(features, form, 1);
            }
            if east_asian.Width() == EastAsianWidth::kFullWidth {
                push(features, b"fwid", 1);
            } else if east_asian.Width() == EastAsianWidth::kProportionalWidth {
                push(features, b"pwid", 1);
            }
            if east_asian.Ruby() {
                push(features, b"ruby", 1);
            }
        }

        let numeric = description.VariantNumeric();
        if numeric.NumericFigureValue() == NumericFigure::kLiningNums {
            push(features, b"lnum", 1);
        } else if numeric.NumericFigureValue() == NumericFigure::kOldstyleNums {
            push(features, b"onum", 1);
        }
        if numeric.NumericSpacingValue() == NumericSpacing::kProportionalNums {
            push(features, b"pnum", 1);
        } else if numeric.NumericSpacingValue() == NumericSpacing::kTabularNums {
            push(features, b"tnum", 1);
        }
        if numeric.NumericFractionValue() == NumericFraction::kStackedFractions {
            push(features, b"afrc", 1);
        } else if numeric.NumericFractionValue() == NumericFraction::kDiagonalFractions {
            push(features, b"frac", 1);
        }
        if numeric.OrdinalValue() == Ordinal::kOrdinalOn {
            push(features, b"ordn", 1);
        }
        if numeric.SlashedZeroValue() == SlashedZero::kSlashedZeroOn {
            push(features, b"zero", 1);
        }

        let chws = if horizontal {
            tag(b"chws")
        } else {
            tag(b"vchw")
        };
        let halt = if horizontal {
            tag(b"halt")
        } else {
            tag(b"vhal")
        };
        let palt = if horizontal {
            tag(b"palt")
        } else {
            tag(b"vpal")
        };
        let mut default_enable_chws = ShouldTrimAdjacent(description.GetTextSpacingTrim());
        if let Some(settings) = description.FeatureSettings() {
            for setting in settings.iter() {
                let feature = Self::new(FontFeatureTag::new(setting.Tag()), setting.Value() as u32);
                if default_enable_chws
                    && (feature.tag == chws.tag
                        || (feature.value != 0
                            && (feature.tag == halt.tag || feature.tag == palt.tag)))
                {
                    default_enable_chws = false;
                }
                features.push(feature);
            }
        }
        if default_enable_chws {
            features.push(Self::new(chws, 1));
        }

        match description.VariantPosition() {
            FontVariantPosition::kSubVariantPosition => push(features, b"subs", 1),
            FontVariantPosition::kSuperVariantPosition => push(features, b"sups", 1),
            FontVariantPosition::kNormalVariantPosition => {}
        }
    }
}

const _: () =
    assert!(std::mem::size_of::<FontFeatureRange>() == std::mem::size_of::<HarfBuzzFeature>());
const _: () =
    assert!(std::mem::align_of::<FontFeatureRange>() == std::mem::align_of::<HarfBuzzFeature>());

// cpp: font_engine/fonts/shaping/font_features.h:72-73
// Rust Vec owns its backing; C++ inline capacity 6 is not a semantic field.
pub type FontFeatureRanges = Vec<FontFeatureRange>;

// cpp: font_engine/fonts/shaping/font_features.h:79-106
pub struct FontFeatureRangesSaver {
    features: NonNull<FontFeatureRanges>,
    num_features_before: usize,
}

impl FontFeatureRangesSaver {
    /// # Safety
    /// The vector must outlive the saver and remain at a stable address.
    pub unsafe fn new(features: &mut FontFeatureRanges) -> Self {
        Self {
            features: NonNull::from(&mut *features),
            num_features_before: features.len(),
        }
    }
}

impl Drop for FontFeatureRangesSaver {
    fn drop(&mut self) {
        let features = unsafe { self.features.as_mut() };
        if features.len() > self.num_features_before {
            features.truncate(self.num_features_before);
        }
    }
}
