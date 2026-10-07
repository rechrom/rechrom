#![allow(non_camel_case_types, non_snake_case)]

use crate::fonts::shaping::harfbuzz_face::HarfBuzzFace;
use crate::text::native::harfbuzz as hb;

// Values index the OpenType MATH constants subtable.
// cpp: font_engine/fonts/opentype/open_type_math_support.h:25-84
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathConstants {
    kScriptPercentScaleDown = 0,
    kScriptScriptPercentScaleDown = 1,
    kDelimitedSubFormulaMinHeight = 2,
    kDisplayOperatorMinHeight = 3,
    kMathLeading = 4,
    kAxisHeight = 5,
    kAccentBaseHeight = 6,
    kFlattenedAccentBaseHeight = 7,
    kSubscriptShiftDown = 8,
    kSubscriptTopMax = 9,
    kSubscriptBaselineDropMin = 10,
    kSuperscriptShiftUp = 11,
    kSuperscriptShiftUpCramped = 12,
    kSuperscriptBottomMin = 13,
    kSuperscriptBaselineDropMax = 14,
    kSubSuperscriptGapMin = 15,
    kSuperscriptBottomMaxWithSubscript = 16,
    kSpaceAfterScript = 17,
    kUpperLimitGapMin = 18,
    kUpperLimitBaselineRiseMin = 19,
    kLowerLimitGapMin = 20,
    kLowerLimitBaselineDropMin = 21,
    kStackTopShiftUp = 22,
    kStackTopDisplayStyleShiftUp = 23,
    kStackBottomShiftDown = 24,
    kStackBottomDisplayStyleShiftDown = 25,
    kStackGapMin = 26,
    kStackDisplayStyleGapMin = 27,
    kStretchStackTopShiftUp = 28,
    kStretchStackBottomShiftDown = 29,
    kStretchStackGapAboveMin = 30,
    kStretchStackGapBelowMin = 31,
    kFractionNumeratorShiftUp = 32,
    kFractionNumeratorDisplayStyleShiftUp = 33,
    kFractionDenominatorShiftDown = 34,
    kFractionDenominatorDisplayStyleShiftDown = 35,
    kFractionNumeratorGapMin = 36,
    kFractionNumDisplayStyleGapMin = 37,
    kFractionRuleThickness = 38,
    kFractionDenominatorGapMin = 39,
    kFractionDenomDisplayStyleGapMin = 40,
    kSkewedFractionHorizontalGap = 41,
    kSkewedFractionVerticalGap = 42,
    kOverbarVerticalGap = 43,
    kOverbarRuleThickness = 44,
    kOverbarExtraAscender = 45,
    kUnderbarVerticalGap = 46,
    kUnderbarRuleThickness = 47,
    kUnderbarExtraDescender = 48,
    kRadicalVerticalGap = 49,
    kRadicalDisplayStyleVerticalGap = 50,
    kRadicalRuleThickness = 51,
    kRadicalExtraAscender = 52,
    kRadicalKernBeforeDegree = 53,
    kRadicalKernAfterDegree = 54,
    kRadicalDegreeBottomRaisePercent = 55,
}

pub struct OpenTypeMathSupport;

impl OpenTypeMathSupport {
    // cpp: font_engine/fonts/opentype/open_type_math_support.cc:37-40
    pub fn HasMathData(face: *const HarfBuzzFace) -> bool {
        if face.is_null() {
            return false;
        }
        let font = unsafe { &*face }.GetScaledFont();
        !font.is_null() && unsafe { hb::hb_ot_math_has_data(hb::hb_font_get_face(font)) != 0 }
    }

    // cpp: font_engine/fonts/opentype/open_type_math_support.cc:42-59
    pub fn MathConstant(face: *const HarfBuzzFace, constant: MathConstants) -> Option<f32> {
        if !Self::HasMathData(face) {
            return None;
        }
        let font = unsafe { &*face }.GetScaledFont();
        let value = unsafe { hb::hb_ot_math_get_constant(font, constant as i32) };
        Some(match constant {
            MathConstants::kScriptPercentScaleDown
            | MathConstants::kScriptScriptPercentScaleDown
            | MathConstants::kRadicalDegreeBottomRaisePercent => value as f32 / 100.0,
            _ => value as f32 / 65536.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_face_has_no_math_table() {
        assert!(!OpenTypeMathSupport::HasMathData(std::ptr::null()));
        assert_eq!(
            OpenTypeMathSupport::MathConstant(
                std::ptr::null(),
                MathConstants::kScriptPercentScaleDown
            ),
            None
        );
    }
}
