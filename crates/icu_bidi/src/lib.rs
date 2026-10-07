#![allow(non_camel_case_types, non_upper_case_globals)]

pub mod native;

// C ABI values from the vendored ICU headers used by the C++ tree.
// cpp: icu_bidi/unicode/ubidi.h:340-340
pub type UBiDiLevel = u8;

// cpp: icu_bidi/unicode/uscript.h:539-539
pub const USCRIPT_CODE_LIMIT: i32 = 213;

// cpp: icu_bidi/unicode/uscript.h:115-115
pub const USCRIPT_LATIN: i32 = 25;
