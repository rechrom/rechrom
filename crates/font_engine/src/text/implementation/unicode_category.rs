#![allow(non_snake_case, non_upper_case_globals)]

// The Unicode General_Category that HarfBuzz's default unicode functions
// report (hb_unicode_general_category), read from ICU4X's compiled UCD data.
// The CharCategory definition and bit masks belong to //src/foundation.
use foundation::unicode::{self, CharCategory};
use icu_properties::props::GeneralCategory;
use icu_properties::CodePointMapData;

pub fn Category(c: i32) -> CharCategory {
    let Ok(code_point) = u32::try_from(c) else {
        return unicode::kOther_NotAssigned;
    };
    match CodePointMapData::<GeneralCategory>::new().get32(code_point) {
        GeneralCategory::Control => unicode::kOther_Control,
        GeneralCategory::Format => unicode::kOther_Format,
        GeneralCategory::Unassigned => unicode::kOther_NotAssigned,
        GeneralCategory::PrivateUse => unicode::kOther_PrivateUse,
        GeneralCategory::Surrogate => unicode::kOther_Surrogate,
        GeneralCategory::LowercaseLetter => unicode::kLetter_Lowercase,
        GeneralCategory::ModifierLetter => unicode::kLetter_Modifier,
        GeneralCategory::OtherLetter => unicode::kLetter_Other,
        GeneralCategory::TitlecaseLetter => unicode::kLetter_Titlecase,
        GeneralCategory::UppercaseLetter => unicode::kLetter_Uppercase,
        GeneralCategory::SpacingMark => unicode::kMark_SpacingCombining,
        GeneralCategory::EnclosingMark => unicode::kMark_Enclosing,
        GeneralCategory::NonspacingMark => unicode::kMark_NonSpacing,
        GeneralCategory::DecimalNumber => unicode::kNumber_DecimalDigit,
        GeneralCategory::LetterNumber => unicode::kNumber_Letter,
        GeneralCategory::OtherNumber => unicode::kNumber_Other,
        GeneralCategory::ConnectorPunctuation => unicode::kPunctuation_Connector,
        GeneralCategory::DashPunctuation => unicode::kPunctuation_Dash,
        GeneralCategory::ClosePunctuation => unicode::kPunctuation_Close,
        GeneralCategory::FinalPunctuation => unicode::kPunctuation_FinalQuote,
        GeneralCategory::InitialPunctuation => unicode::kPunctuation_InitialQuote,
        GeneralCategory::OtherPunctuation => unicode::kPunctuation_Other,
        GeneralCategory::OpenPunctuation => unicode::kPunctuation_Open,
        GeneralCategory::CurrencySymbol => unicode::kSymbol_Currency,
        GeneralCategory::ModifierSymbol => unicode::kSymbol_Modifier,
        GeneralCategory::MathSymbol => unicode::kSymbol_Math,
        GeneralCategory::OtherSymbol => unicode::kSymbol_Other,
        GeneralCategory::LineSeparator => unicode::kSeparator_Line,
        GeneralCategory::ParagraphSeparator => unicode::kSeparator_Paragraph,
        GeneralCategory::SpaceSeparator => unicode::kSeparator_Space,
    }
}
