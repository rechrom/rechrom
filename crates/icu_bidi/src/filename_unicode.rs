//! Charset conversion and normalization for net filename generation, on
//! `encoding_rs` and ICU4X. Source net/base/net_string_util_icu.cc:23-67;
//! base/i18n/icu_string_conversions.cc:153-234; base/i18n/file_util_icu.cc:122-165.
#![allow(non_snake_case)]

use encoding_rs::Encoding;
use icu_normalizer::ComposingNormalizerBorrowed;

/// Resolves a charset name as a C string (an embedded NUL ends it).
fn encoding(charset: &[u8]) -> Option<&'static Encoding> {
    let charset = charset.split(|&byte| byte == 0).next()?;
    Encoding::for_label_no_replacement(charset)
}

/// ConvertToUtf8: malformed input becomes U+FFFD (ICU's substitution callback).
pub fn ConvertToUtf8(input: &[u8], charset: &[u8]) -> Option<String> {
    let (text, _) = encoding(charset)?.decode_without_bom_handling(input);
    Some(text.into_owned())
}

/// CodepageToUTF16(...FAIL): malformed input fails the conversion.
pub fn ConvertToUTF16(input: &[u8], charset: &[u8]) -> Option<Vec<u16>> {
    let text = encoding(charset)?.decode_without_bom_handling_and_without_replacement(input)?;
    Some(text.encode_utf16().collect())
}

pub fn ConvertToUtf8AndNormalize(input: &[u8], charset: &[u8]) -> Option<String> {
    let text = encoding(charset)?.decode_without_bom_handling_and_without_replacement(input)?;
    Some(ComposingNormalizerBorrowed::new_nfc().normalize(&text).into_owned())
}

pub fn IsIllegalAnywhere(ch: char) -> bool {
    let cp = ch as u32;
    matches!(ch, '"' | '*' | '/' | ':' | '<' | '>' | '?' | '\\' | '|')
        || matches!(unsafe { crate::ubidi_ffi::u_charType(cp as i32) }, 15 | 16)
        || (0xfdd0..=0xfdef).contains(&cp)
        || (cp & 0xfffe) == 0xfffe
}

pub fn IsIllegalAtEnds(ch: char) -> bool {
    matches!(ch, '.' | '~') || unsafe { crate::ubidi_ffi::u_hasBinaryProperty(ch as i32, 31) } != 0
}
