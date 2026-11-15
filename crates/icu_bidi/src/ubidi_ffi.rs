#![allow(non_snake_case, clippy::missing_safety_doc)]
// ICU4C numeric values are this module's contract; ICU4X marks the
// conversions deprecated only to discourage new ICU4C-shaped APIs.
#![allow(deprecated)]

//! The ICU4C `ubidi.h`, `uchar.h`, `uscript.h` and `ustring.h` entry points
//! Blink's translated font, text and bidi modules call, implemented on ICU4X
//! character properties, ICU4X case mapping and the `unicode-bidi` UBA.
//!
//! Signatures, integer property ids and enum values are ICU4C's, so callers
//! keep the C API's contract (raw UTF-16 pointers, UErrorCode, preflighting).

use std::ffi::{c_char, CStr, CString};
use std::sync::OnceLock;

use icu_casemap::CaseMapper;
use icu_locale_core::LanguageIdentifier;
use icu_normalizer::DecomposingNormalizerBorrowed;
use icu_properties::props::{
    self, BidiClass, BidiMirroringGlyph, BidiPairedBracketType, CanonicalCombiningClass,
    EastAsianWidth, GeneralCategory, LineBreak, Script,
};
use icu_properties::script::ScriptWithExtensions;
use icu_properties::{CodePointMapData, CodePointSetData, PropertyNamesShort};
use unicode_bidi::{utf16, Level};

use crate::ustring::{self, U_BUFFER_OVERFLOW_ERROR, U_ILLEGAL_ARGUMENT_ERROR};

// ---------------------------------------------------------------------------
// uchar.h

fn general_category(ch: i32) -> GeneralCategory {
    CodePointMapData::<GeneralCategory>::new().get32(ch as u32)
}

/// ICU4C `UCharCategory` numbering.
fn char_category_value(category: GeneralCategory) -> i8 {
    use GeneralCategory::*;
    match category {
        Unassigned => 0,
        UppercaseLetter => 1,
        LowercaseLetter => 2,
        TitlecaseLetter => 3,
        ModifierLetter => 4,
        OtherLetter => 5,
        NonspacingMark => 6,
        EnclosingMark => 7,
        SpacingMark => 8,
        DecimalNumber => 9,
        LetterNumber => 10,
        OtherNumber => 11,
        SpaceSeparator => 12,
        LineSeparator => 13,
        ParagraphSeparator => 14,
        Control => 15,
        Format => 16,
        PrivateUse => 17,
        Surrogate => 18,
        DashPunctuation => 19,
        OpenPunctuation => 20,
        ClosePunctuation => 21,
        ConnectorPunctuation => 22,
        OtherPunctuation => 23,
        MathSymbol => 24,
        CurrencySymbol => 25,
        ModifierSymbol => 26,
        OtherSymbol => 27,
        InitialPunctuation => 28,
        FinalPunctuation => 29,
    }
}

pub unsafe fn u_charType(ch: i32) -> i8 {
    char_category_value(general_category(ch))
}

/// `u_isprint`: every code point except general category C (Cc, Cf, Cs, Co, Cn).
pub unsafe fn u_isprint(ch: i32) -> i8 {
    !matches!(u_charType(ch), 0 | 15 | 16 | 17 | 18) as i8
}

pub unsafe fn u_charDirection(ch: i32) -> i32 {
    i32::from(CodePointMapData::<BidiClass>::new().get32(ch as u32).to_icu4c_value())
}

pub unsafe fn u_getCombiningClass(ch: i32) -> u8 {
    CodePointMapData::<CanonicalCombiningClass>::new()
        .get32(ch as u32)
        .to_icu4c_value()
}

pub unsafe fn u_getBidiPairedBracket(ch: i32) -> i32 {
    let data = CodePointMapData::<BidiMirroringGlyph>::new().get32(ch as u32);
    match (data.paired_bracket_type, data.mirroring_glyph) {
        (BidiPairedBracketType::None, _) | (_, None) => ch,
        (_, Some(pair)) => pair as i32,
    }
}

/// `u_hasBinaryProperty` for the `UProperty` binary ids ICU4X provides.
pub unsafe fn u_hasBinaryProperty(ch: i32, property: i32) -> i8 {
    let ch = ch as u32;
    macro_rules! set {
        ($property:ty) => {
            CodePointSetData::new::<$property>().contains32(ch)
        };
    }
    let value = match property {
        0 => set!(props::Alphabetic),
        1 => set!(props::AsciiHexDigit),
        2 => set!(props::BidiControl),
        3 => set!(props::BidiMirrored),
        4 => set!(props::Dash),
        5 => set!(props::DefaultIgnorableCodePoint),
        6 => set!(props::Deprecated),
        7 => set!(props::Diacritic),
        8 => set!(props::Extender),
        9 => set!(props::FullCompositionExclusion),
        10 => set!(props::GraphemeBase),
        11 => set!(props::GraphemeExtend),
        12 => set!(props::GraphemeLink),
        13 => set!(props::HexDigit),
        14 => set!(props::Hyphen),
        15 => set!(props::IdContinue),
        16 => set!(props::IdStart),
        17 => set!(props::Ideographic),
        18 => set!(props::IdsBinaryOperator),
        19 => set!(props::IdsTrinaryOperator),
        20 => set!(props::JoinControl),
        21 => set!(props::LogicalOrderException),
        22 => set!(props::Lowercase),
        23 => set!(props::Math),
        24 => set!(props::NoncharacterCodePoint),
        25 => set!(props::QuotationMark),
        26 => set!(props::Radical),
        27 => set!(props::SoftDotted),
        28 => set!(props::TerminalPunctuation),
        29 => set!(props::UnifiedIdeograph),
        30 => set!(props::Uppercase),
        31 => set!(props::WhiteSpace),
        32 => set!(props::XidContinue),
        33 => set!(props::XidStart),
        34 => set!(props::CaseSensitive),
        35 => set!(props::SentenceTerminal),
        36 => set!(props::VariationSelector),
        44 => set!(props::PatternSyntax),
        45 => set!(props::PatternWhiteSpace),
        49 => set!(props::Cased),
        50 => set!(props::CaseIgnorable),
        57 => set!(props::Emoji),
        58 => set!(props::EmojiPresentation),
        59 => set!(props::EmojiModifier),
        60 => set!(props::EmojiModifierBase),
        61 => set!(props::EmojiComponent),
        62 => set!(props::RegionalIndicator),
        63 => set!(props::PrependedConcatenationMark),
        64 => set!(props::ExtendedPictographic),
        _ => panic!("UProperty {property} is not a supported binary property"),
    };
    value as i8
}

/// ICU4C `UDecompositionType`, distinguishing none (0), canonical (1) and
/// compatibility (2) decompositions; compatibility subtypes report 2.
fn decomposition_type(ch: u32) -> i32 {
    let Some(character) = char::from_u32(ch) else {
        return 0;
    };
    let mut buffer = [0u8; 4];
    let text = character.encode_utf8(&mut buffer);
    if DecomposingNormalizerBorrowed::new_nfd().normalize(text) != *text {
        1
    } else if DecomposingNormalizerBorrowed::new_nfkd().normalize(text) != *text {
        2
    } else {
        0
    }
}

/// `u_getIntPropertyValue` for the enumerated `UProperty` ids Blink queries
/// and, as in ICU, for binary property ids.
pub unsafe fn u_getIntPropertyValue(ch: i32, property: i32) -> i32 {
    let code_point = ch as u32;
    match property {
        0..=0x0fff => i32::from(u_hasBinaryProperty(ch, property)),
        0x1000 => u_charDirection(ch),
        0x1003 => decomposition_type(code_point),
        0x1004 => i32::from(CodePointMapData::<EastAsianWidth>::new().get32(code_point).to_icu4c_value()),
        0x1005 => i32::from(u_charType(ch)),
        0x1008 => i32::from(CodePointMapData::<LineBreak>::new().get32(code_point).to_icu4c_value()),
        0x100A => i32::from(CodePointMapData::<Script>::new().get32(code_point).to_icu4c_value()),
        0x1015 => match CodePointMapData::<BidiMirroringGlyph>::new()
            .get32(code_point)
            .paired_bracket_type
        {
            BidiPairedBracketType::Open => 1,
            BidiPairedBracketType::Close => 2,
            _ => 0,
        },
        _ => panic!("UProperty {property} is not a supported enumerated property"),
    }
}

// ---------------------------------------------------------------------------
// uscript.h

pub unsafe fn uscript_getScript(ch: i32, status: *mut i32) -> i32 {
    if ustring::failed(status) {
        return 0;
    }
    if !(0..=0x10ffff).contains(&ch) {
        ustring::set_error(status, U_ILLEGAL_ARGUMENT_ERROR);
        return 0;
    }
    i32::from(
        ScriptWithExtensions::new()
            .get_script_val32(ch as u32)
            .to_icu4c_value(),
    )
}

pub unsafe fn uscript_getScriptExtensions(
    ch: i32,
    scripts: *mut i32,
    capacity: i32,
    status: *mut i32,
) -> i32 {
    if ustring::failed(status) {
        return 0;
    }
    if capacity < 0 || (capacity > 0 && scripts.is_null()) {
        ustring::set_error(status, U_ILLEGAL_ARGUMENT_ERROR);
        return 0;
    }
    let extensions = ScriptWithExtensions::new().get_script_extensions_val32(ch as u32);
    let values: Vec<i32> = extensions
        .iter()
        .map(|script| i32::from(script.to_icu4c_value()))
        .collect();
    if values.len() > capacity as usize {
        ustring::set_error(status, U_BUFFER_OVERFLOW_ERROR);
    } else {
        std::ptr::copy_nonoverlapping(values.as_ptr(), scripts, values.len());
    }
    values.len() as i32
}

/// `uscript_getShortName`: the ISO 15924 code, as a static C string.
pub unsafe fn uscript_getShortName(script: i32) -> *const c_char {
    static NAMES: OnceLock<Vec<Option<CString>>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let lookup = PropertyNamesShort::<Script>::new();
        (0..crate::USCRIPT_CODE_LIMIT)
            .map(|code| {
                lookup
                    .get(Script::from_icu4c_value(code as u16))
                    .map(|name| CString::new(name).expect("script name"))
            })
            .collect()
    });
    usize::try_from(script)
        .ok()
        .and_then(|index| names.get(index))
        .and_then(|name| name.as_ref())
        .map_or(std::ptr::null(), |name| name.as_ptr())
}

// ---------------------------------------------------------------------------
// ustring.h case mapping

/// `u_strFoldCase` with U_FOLD_CASE_DEFAULT (full, locale-independent folding).
pub unsafe fn u_strFoldCase(
    destination: *mut u16,
    capacity: i32,
    source: *const u16,
    source_length: i32,
    options: u32,
    status: *mut i32,
) -> i32 {
    if ustring::failed(status) {
        return 0;
    }
    let Some(source) = ustring::input(source, source_length) else {
        ustring::set_error(status, U_ILLEGAL_ARGUMENT_ERROR);
        return 0;
    };
    assert_eq!(options, 0, "only U_FOLD_CASE_DEFAULT is supported");
    let mapper = CaseMapper::new();
    let folded = ustring::map_utf16(source, |run, out| out.push_str(&mapper.fold_string(run)));
    ustring::output(&folded, destination, capacity, status)
}

fn language(locale: *const c_char) -> LanguageIdentifier {
    if locale.is_null() {
        return LanguageIdentifier::UNKNOWN;
    }
    let name = unsafe { CStr::from_ptr(locale) }.to_string_lossy();
    let name = name.split(['@', '.']).next().unwrap_or("").replace('_', "-");
    name.parse().unwrap_or(LanguageIdentifier::UNKNOWN)
}

pub unsafe fn u_strToLower(
    destination: *mut u16,
    capacity: i32,
    source: *const u16,
    source_length: i32,
    locale: *const c_char,
    status: *mut i32,
) -> i32 {
    if ustring::failed(status) {
        return 0;
    }
    let Some(source) = ustring::input(source, source_length) else {
        ustring::set_error(status, U_ILLEGAL_ARGUMENT_ERROR);
        return 0;
    };
    let language = language(locale);
    let mapper = CaseMapper::new();
    let lower = ustring::map_utf16(source, |run, out| {
        out.push_str(&mapper.lowercase_to_string(run, &language))
    });
    ustring::output(&lower, destination, capacity, status)
}

/// `u_memcasecmp`: compares the full case foldings of two equal-length
/// UTF-16 strings in code point order.
pub unsafe fn u_memcasecmp(first: *const u16, second: *const u16, length: i32, options: u32) -> i32 {
    assert_eq!(options, 0, "only U_FOLD_CASE_DEFAULT is supported");
    let length = length.max(0) as usize;
    let (first, second) = if length == 0 {
        (&[][..], &[][..])
    } else {
        (
            std::slice::from_raw_parts(first, length),
            std::slice::from_raw_parts(second, length),
        )
    };
    let mapper = CaseMapper::new();
    let fold = |text: &[u16]| ustring::map_utf16(text, |run, out| out.push_str(&mapper.fold_string(run)));
    let (first, second) = (fold(first), fold(second));
    let first = ustring::code_points(&first).map(|(_, cp)| cp);
    let second = ustring::code_points(&second).map(|(_, cp)| cp);
    match first.cmp(second) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

// ---------------------------------------------------------------------------
// ubidi.h

const UBIDI_DEFAULT_LTR: u8 = 0xFE;
const UBIDI_DEFAULT_RTL: u8 = 0xFF;
const UBIDI_LTR: i32 = 0;
const UBIDI_RTL: i32 = 1;
const UBIDI_MIXED: i32 = 2;

/// A paragraph object after `ubidi_setPara`: per-code-unit embedding levels
/// with rule L1 applied, as ICU leaves them for `ubidi_getLogicalRun`.
#[derive(Default)]
pub struct UBidi {
    levels: Vec<u8>,
    para_level: u8,
    direction: i32,
}

pub unsafe fn ubidi_open() -> *mut UBidi {
    Box::into_raw(Box::default())
}

pub unsafe fn ubidi_close(bidi: *mut UBidi) {
    if !bidi.is_null() {
        drop(Box::from_raw(bidi));
    }
}

/// Rule P2/P3 over the first paragraph, skipping isolate content.
fn first_strong_is_rtl(text: &[u16]) -> Option<bool> {
    let classes = CodePointMapData::<BidiClass>::new();
    let mut isolate_depth = 0usize;
    for (_, code_point) in ustring::code_points(text) {
        match classes.get32(code_point) {
            BidiClass::ParagraphSeparator => return None,
            BidiClass::LeftToRightIsolate
            | BidiClass::RightToLeftIsolate
            | BidiClass::FirstStrongIsolate => isolate_depth += 1,
            BidiClass::PopDirectionalIsolate => isolate_depth = isolate_depth.saturating_sub(1),
            BidiClass::LeftToRight if isolate_depth == 0 => return Some(false),
            BidiClass::RightToLeft | BidiClass::ArabicLetter if isolate_depth == 0 => {
                return Some(true)
            }
            _ => {}
        }
    }
    None
}

pub unsafe fn ubidi_setPara(
    bidi: *mut UBidi,
    text: *const u16,
    length: i32,
    para_level: u8,
    embedding_levels: *mut u8,
    error: *mut i32,
) {
    if ustring::failed(error) {
        return;
    }
    let (Some(bidi), Some(text)) = (bidi.as_mut(), ustring::input(text, length)) else {
        ustring::set_error(error, U_ILLEGAL_ARGUMENT_ERROR);
        return;
    };
    assert!(embedding_levels.is_null(), "explicit embedding levels are not supported");
    let default_level = match para_level {
        UBIDI_DEFAULT_LTR => None,
        UBIDI_DEFAULT_RTL => Some(Level::new(u8::from(first_strong_is_rtl(text).unwrap_or(true))).unwrap()),
        level => match Level::new(level) {
            Ok(level) => Some(level),
            Err(_) => {
                ustring::set_error(error, U_ILLEGAL_ARGUMENT_ERROR);
                return;
            }
        },
    };
    let info = utf16::BidiInfo::new(text, default_level);
    let mut levels = Vec::with_capacity(text.len());
    for paragraph in &info.paragraphs {
        let reordered = info.reordered_levels(paragraph, paragraph.range.clone());
        levels.extend(
            reordered[paragraph.range.clone()]
                .iter()
                .map(|level| level.number()),
        );
    }
    bidi.para_level = info.paragraphs.first().map_or_else(
        || default_level.map_or(0, |level| level.number()),
        |paragraph| paragraph.level.number(),
    );
    bidi.direction = if levels.iter().all(|level| level % 2 == 0) {
        if levels.is_empty() && bidi.para_level % 2 == 1 {
            UBIDI_RTL
        } else {
            UBIDI_LTR
        }
    } else if levels.iter().all(|level| level % 2 == 1) {
        UBIDI_RTL
    } else {
        UBIDI_MIXED
    };
    bidi.levels = levels;
}

pub unsafe fn ubidi_getDirection(bidi: *const UBidi) -> i32 {
    bidi.as_ref().map_or(UBIDI_LTR, |bidi| bidi.direction)
}

pub unsafe fn ubidi_getParaLevel(bidi: *const UBidi) -> u8 {
    bidi.as_ref().map_or(0, |bidi| bidi.para_level)
}

pub unsafe fn ubidi_getLogicalRun(bidi: *const UBidi, start: i32, end: *mut i32, level: *mut u8) {
    let Some(bidi) = bidi.as_ref() else {
        return;
    };
    let Ok(start) = usize::try_from(start) else {
        return;
    };
    if start >= bidi.levels.len() {
        return;
    }
    let run_level = bidi.levels[start];
    let run_end = bidi.levels[start..]
        .iter()
        .position(|value| *value != run_level)
        .map_or(bidi.levels.len(), |offset| start + offset);
    if !end.is_null() {
        *end = run_end as i32;
    }
    if !level.is_null() {
        *level = run_level;
    }
}

/// `ubidi_reorderVisual`: rule L2 over explicit levels; `indices[visual]` is
/// the logical index.
pub unsafe fn ubidi_reorderVisual(levels: *const u8, length: i32, indices: *mut i32) {
    if levels.is_null() || indices.is_null() || length <= 0 {
        return;
    }
    let levels = std::slice::from_raw_parts(levels, length as usize);
    let mut map: Vec<i32> = (0..length).collect();
    let highest = levels.iter().copied().max().unwrap_or(0);
    let lowest_odd = levels.iter().copied().filter(|level| level % 2 == 1).min();
    if let Some(lowest_odd) = lowest_odd {
        let mut level = highest;
        while level >= lowest_odd {
            let mut index = 0;
            while index < levels.len() {
                if levels[map[index] as usize] >= level {
                    let start = index;
                    while index < levels.len() && levels[map[index] as usize] >= level {
                        index += 1;
                    }
                    map[start..index].reverse();
                } else {
                    index += 1;
                }
            }
            level -= 1;
        }
    }
    std::ptr::copy_nonoverlapping(map.as_ptr(), indices, map.len());
}
