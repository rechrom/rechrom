#![allow(non_snake_case, non_upper_case_globals)]

use super::harfbuzz as hb;
// The CharCategory definition and bit masks belong to //src/foundation.
use foundation::unicode::{self, CharCategory};

// cpp: font_engine/text/native/unicode_category.cc:8-45
pub fn Category(c: i32) -> CharCategory {
    let category =
        unsafe { hb::hb_unicode_general_category(hb::hb_unicode_funcs_get_default(), c as u32) };
    match category {
        0 => unicode::kOther_Control,
        1 => unicode::kOther_Format,
        2 => unicode::kOther_NotAssigned,
        3 => unicode::kOther_PrivateUse,
        4 => unicode::kOther_Surrogate,
        5 => unicode::kLetter_Lowercase,
        6 => unicode::kLetter_Modifier,
        7 => unicode::kLetter_Other,
        8 => unicode::kLetter_Titlecase,
        9 => unicode::kLetter_Uppercase,
        10 => unicode::kMark_SpacingCombining,
        11 => unicode::kMark_Enclosing,
        12 => unicode::kMark_NonSpacing,
        13 => unicode::kNumber_DecimalDigit,
        14 => unicode::kNumber_Letter,
        15 => unicode::kNumber_Other,
        16 => unicode::kPunctuation_Connector,
        17 => unicode::kPunctuation_Dash,
        18 => unicode::kPunctuation_Close,
        19 => unicode::kPunctuation_FinalQuote,
        20 => unicode::kPunctuation_InitialQuote,
        21 => unicode::kPunctuation_Other,
        22 => unicode::kPunctuation_Open,
        23 => unicode::kSymbol_Currency,
        24 => unicode::kSymbol_Modifier,
        25 => unicode::kSymbol_Math,
        26 => unicode::kSymbol_Other,
        27 => unicode::kSeparator_Line,
        28 => unicode::kSeparator_Paragraph,
        29 => unicode::kSeparator_Space,
        _ => unreachable!("HarfBuzz returned an unknown Unicode General_Category"),
    }
}
