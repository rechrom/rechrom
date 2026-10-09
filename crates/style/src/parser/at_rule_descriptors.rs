// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.h:15-67
// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:54-591
// Generated source: third_party/blink/renderer/build/scripts/core/css/parser/templates/at_rule_descriptors.h.tmpl
// Generated source: third_party/blink/renderer/build/scripts/core/css/parser/templates/at_rule_descriptors.cc.tmpl
// The generated header's getValueName declaration (.h:62) has no implementation
// in the generated .cc or its template; that declaration has no translated body.

#![allow(non_upper_case_globals)]

use foundation::{CSSPropertyID, StringView};

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.h:15-58
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtRuleDescriptorID {
    Invalid = 0,
    Variable = 1,
    AdditiveSymbols = 2,
    AscentOverride = 3,
    BasePalette = 4,
    BaseUrl = 5,
    DescentOverride = 6,
    Fallback = 7,
    FontDisplay = 8,
    FontFamily = 9,
    FontFeatureSettings = 10,
    FontStretch = 11,
    FontStyle = 12,
    FontVariant = 13,
    FontVariationSettings = 14,
    FontWeight = 15,
    Hash = 16,
    Hostname = 17,
    Inherits = 18,
    InitialValue = 19,
    LineGapOverride = 20,
    Navigation = 21,
    Negative = 22,
    OverrideColors = 23,
    Pad = 24,
    Pathname = 25,
    Pattern = 26,
    Port = 27,
    Prefix = 28,
    Protocol = 29,
    Range = 30,
    Result = 31,
    Search = 32,
    SizeAdjust = 33,
    SpeakAs = 34,
    Src = 35,
    Suffix = 36,
    Symbols = 37,
    Syntax = 38,
    System = 39,
    Types = 40,
    UnicodeRange = 41,
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.h:60
pub const numAtRuleDescriptors: i32 = 42;

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:63-187
const ASSO_VALUES: [u8; 256] = [
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 1, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 2, 3, 1, 6, 1, 1, 1, 1, 1, 7, 5, 4, 28, 1, 1, 1, 75, 2, 1, 2, 10, 3, 11, 29, 13, 7, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
    75, 75, 75, 75, 75, 75, 75, 75, 75, 75, 75,
];

fn descriptor_hash_function(name: &[u8]) -> usize {
    // The generated fallthrough switch accumulates each byte from last to first.
    let mut hval = 0;
    for &byte in name.iter().rev() {
        hval += ASSO_VALUES[byte as usize] as usize;
    }
    hval
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:189-276,290-333
// Rust stores the immutable pool string with its ID, replacing name_offset
// pointer arithmetic without changing the generated pool or table order.
#[derive(Clone, Copy)]
struct Property {
    name: &'static [u8],
    id: AtRuleDescriptorID,
}

const DESCRIPTOR_WORD_LIST: [Property; 41] = [
    Property {
        name: b"src",
        id: AtRuleDescriptorID::Src,
    },
    Property {
        name: b"hash",
        id: AtRuleDescriptorID::Hash,
    },
    Property {
        name: b"port",
        id: AtRuleDescriptorID::Port,
    },
    Property {
        name: b"range",
        id: AtRuleDescriptorID::Range,
    },
    Property {
        name: b"search",
        id: AtRuleDescriptorID::Search,
    },
    Property {
        name: b"pad",
        id: AtRuleDescriptorID::Pad,
    },
    Property {
        name: b"inherits",
        id: AtRuleDescriptorID::Inherits,
    },
    Property {
        name: b"pattern",
        id: AtRuleDescriptorID::Pattern,
    },
    Property {
        name: b"negative",
        id: AtRuleDescriptorID::Negative,
    },
    Property {
        name: b"protocol",
        id: AtRuleDescriptorID::Protocol,
    },
    Property {
        name: b"speak-as",
        id: AtRuleDescriptorID::SpeakAs,
    },
    Property {
        name: b"navigation",
        id: AtRuleDescriptorID::Navigation,
    },
    Property {
        name: b"font-stretch",
        id: AtRuleDescriptorID::FontStretch,
    },
    Property {
        name: b"types",
        id: AtRuleDescriptorID::Types,
    },
    Property {
        name: b"font-variant",
        id: AtRuleDescriptorID::FontVariant,
    },
    Property {
        name: b"result",
        id: AtRuleDescriptorID::Result,
    },
    Property {
        name: b"base-palette",
        id: AtRuleDescriptorID::BasePalette,
    },
    Property {
        name: b"fallback",
        id: AtRuleDescriptorID::Fallback,
    },
    Property {
        name: b"font-weight",
        id: AtRuleDescriptorID::FontWeight,
    },
    Property {
        name: b"base-url",
        id: AtRuleDescriptorID::BaseUrl,
    },
    Property {
        name: b"ascent-override",
        id: AtRuleDescriptorID::AscentOverride,
    },
    Property {
        name: b"font-style",
        id: AtRuleDescriptorID::FontStyle,
    },
    Property {
        name: b"override-colors",
        id: AtRuleDescriptorID::OverrideColors,
    },
    Property {
        name: b"unicode-range",
        id: AtRuleDescriptorID::UnicodeRange,
    },
    Property {
        name: b"line-gap-override",
        id: AtRuleDescriptorID::LineGapOverride,
    },
    Property {
        name: b"descent-override",
        id: AtRuleDescriptorID::DescentOverride,
    },
    Property {
        name: b"font-variation-settings",
        id: AtRuleDescriptorID::FontVariationSettings,
    },
    Property {
        name: b"initial-value",
        id: AtRuleDescriptorID::InitialValue,
    },
    Property {
        name: b"font-display",
        id: AtRuleDescriptorID::FontDisplay,
    },
    Property {
        name: b"prefix",
        id: AtRuleDescriptorID::Prefix,
    },
    Property {
        name: b"font-feature-settings",
        id: AtRuleDescriptorID::FontFeatureSettings,
    },
    Property {
        name: b"hostname",
        id: AtRuleDescriptorID::Hostname,
    },
    Property {
        name: b"pathname",
        id: AtRuleDescriptorID::Pathname,
    },
    Property {
        name: b"size-adjust",
        id: AtRuleDescriptorID::SizeAdjust,
    },
    Property {
        name: b"suffix",
        id: AtRuleDescriptorID::Suffix,
    },
    Property {
        name: b"system",
        id: AtRuleDescriptorID::System,
    },
    Property {
        name: b"syntax",
        id: AtRuleDescriptorID::Syntax,
    },
    Property {
        name: b"symbols",
        id: AtRuleDescriptorID::Symbols,
    },
    Property {
        name: b"font-family",
        id: AtRuleDescriptorID::FontFamily,
    },
    Property {
        name: b"-webkit-font-feature-settings",
        id: AtRuleDescriptorID::FontFeatureSettings,
    },
    Property {
        name: b"additive-symbols",
        id: AtRuleDescriptorID::AdditiveSymbols,
    },
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:335-362
const LOOKUP: [i8; 75] = [
    -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, -1, 13, 14, 15, 16, 17, 18, 19, -1,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, -1, -1, -1, 34, -1, -1, 35, -1, 36, -1,
    -1, 37, -1, -1, -1, 38, -1, -1, -1, -1, -1, 39, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    40,
];

fn findDescriptorImpl(name: &[u8]) -> Option<&'static Property> {
    if (3..=29).contains(&name.len()) {
        let key = descriptor_hash_function(name);
        if key <= 74 {
            let index = LOOKUP[key];
            if index >= 0 {
                let entry = &DESCRIPTOR_WORD_LIST[index as usize];
                // The generated first-byte, strncmp and NUL terminator checks
                // jointly require exact byte equality and equal length.
                if name == entry.name {
                    return Some(entry);
                }
            }
        }
    }
    None
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:366-368
fn FindDescriptor(name: &[u8]) -> Option<&'static Property> {
    findDescriptorImpl(name)
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:370-405
fn AsAtRuleDescriptorIDFromUnits<CharacterType>(descriptor: &[CharacterType]) -> AtRuleDescriptorID
where
    CharacterType: Copy + Into<u16>,
{
    let length = descriptor.len();
    if length == 0 {
        return AtRuleDescriptorID::Invalid;
    }
    if length >= 3 && descriptor[0].into() == 45 && descriptor[1].into() == 45 {
        return AtRuleDescriptorID::Variable;
    }
    if length > 29 {
        return AtRuleDescriptorID::Invalid;
    }
    let mut buffer = [0u8; 30];
    for (i, &unit) in descriptor.iter().enumerate() {
        let c: u16 = unit.into();
        if c == 0 || c >= 0x7f {
            return AtRuleDescriptorID::Invalid;
        }
        buffer[i] = (c as u8).to_ascii_lowercase();
    }
    buffer[length] = 0;
    match FindDescriptor(&buffer[..length]) {
        None => AtRuleDescriptorID::Invalid,
        Some(entry) => {
            // cpp: third_party/blink/renderer/core/css/hash_tools.h:30
            // All generated descriptor IDs are below kNotKnownExposedPropertyBit.
            debug_assert_eq!((entry.id as i32) & 0x8000, 0);
            entry.id
        }
    }
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:409-413
pub fn AsAtRuleDescriptorID(string: StringView) -> AtRuleDescriptorID {
    if string.Is8Bit() {
        AsAtRuleDescriptorIDFromUnits(string.Span8().expect("8-bit StringView"))
    } else {
        AsAtRuleDescriptorIDFromUnits(string.Span16())
    }
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:415-502
pub fn AtRuleDescriptorIDAsCSSPropertyID(id: AtRuleDescriptorID) -> CSSPropertyID {
    match id {
        AtRuleDescriptorID::Variable => CSSPropertyID::kVariable,
        AtRuleDescriptorID::AdditiveSymbols => CSSPropertyID::kAdditiveSymbols,
        AtRuleDescriptorID::AscentOverride => CSSPropertyID::kAscentOverride,
        AtRuleDescriptorID::BasePalette => CSSPropertyID::kBasePalette,
        AtRuleDescriptorID::BaseUrl => CSSPropertyID::kBaseUrl,
        AtRuleDescriptorID::DescentOverride => CSSPropertyID::kDescentOverride,
        AtRuleDescriptorID::Fallback => CSSPropertyID::kFallback,
        AtRuleDescriptorID::FontDisplay => CSSPropertyID::kFontDisplay,
        AtRuleDescriptorID::FontFamily => CSSPropertyID::kFontFamily,
        AtRuleDescriptorID::FontFeatureSettings => CSSPropertyID::kFontFeatureSettings,
        AtRuleDescriptorID::FontStretch => CSSPropertyID::kFontStretch,
        AtRuleDescriptorID::FontStyle => CSSPropertyID::kFontStyle,
        AtRuleDescriptorID::FontVariant => CSSPropertyID::kFontVariant,
        AtRuleDescriptorID::FontVariationSettings => CSSPropertyID::kFontVariationSettings,
        AtRuleDescriptorID::FontWeight => CSSPropertyID::kFontWeight,
        AtRuleDescriptorID::Hash => CSSPropertyID::kHash,
        AtRuleDescriptorID::Hostname => CSSPropertyID::kHostname,
        AtRuleDescriptorID::Inherits => CSSPropertyID::kInherits,
        AtRuleDescriptorID::InitialValue => CSSPropertyID::kInitialValue,
        AtRuleDescriptorID::LineGapOverride => CSSPropertyID::kLineGapOverride,
        AtRuleDescriptorID::Navigation => CSSPropertyID::kNavigation,
        AtRuleDescriptorID::Negative => CSSPropertyID::kNegative,
        AtRuleDescriptorID::OverrideColors => CSSPropertyID::kOverrideColors,
        AtRuleDescriptorID::Pad => CSSPropertyID::kPad,
        AtRuleDescriptorID::Pathname => CSSPropertyID::kPathname,
        AtRuleDescriptorID::Pattern => CSSPropertyID::kPattern,
        AtRuleDescriptorID::Port => CSSPropertyID::kPort,
        AtRuleDescriptorID::Prefix => CSSPropertyID::kPrefix,
        AtRuleDescriptorID::Protocol => CSSPropertyID::kProtocol,
        AtRuleDescriptorID::Range => CSSPropertyID::kRange,
        AtRuleDescriptorID::Result => CSSPropertyID::kResult,
        AtRuleDescriptorID::Search => CSSPropertyID::kSearch,
        AtRuleDescriptorID::SizeAdjust => CSSPropertyID::kSizeAdjust,
        AtRuleDescriptorID::SpeakAs => CSSPropertyID::kSpeakAs,
        AtRuleDescriptorID::Src => CSSPropertyID::kSrc,
        AtRuleDescriptorID::Suffix => CSSPropertyID::kSuffix,
        AtRuleDescriptorID::Symbols => CSSPropertyID::kSymbols,
        AtRuleDescriptorID::Syntax => CSSPropertyID::kSyntax,
        AtRuleDescriptorID::System => CSSPropertyID::kSystem,
        AtRuleDescriptorID::Types => CSSPropertyID::kTypes,
        AtRuleDescriptorID::UnicodeRange => CSSPropertyID::kUnicodeRange,
        AtRuleDescriptorID::Invalid => unreachable!("invalid at-rule descriptor ID"),
    }
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/parser/at_rule_descriptors.cc:504-591
pub fn CSSPropertyIDAsAtRuleDescriptor(id: CSSPropertyID) -> AtRuleDescriptorID {
    match id {
        CSSPropertyID::kVariable => AtRuleDescriptorID::Variable,
        CSSPropertyID::kAdditiveSymbols => AtRuleDescriptorID::AdditiveSymbols,
        CSSPropertyID::kAscentOverride => AtRuleDescriptorID::AscentOverride,
        CSSPropertyID::kBasePalette => AtRuleDescriptorID::BasePalette,
        CSSPropertyID::kBaseUrl => AtRuleDescriptorID::BaseUrl,
        CSSPropertyID::kDescentOverride => AtRuleDescriptorID::DescentOverride,
        CSSPropertyID::kFallback => AtRuleDescriptorID::Fallback,
        CSSPropertyID::kFontDisplay => AtRuleDescriptorID::FontDisplay,
        CSSPropertyID::kFontFamily => AtRuleDescriptorID::FontFamily,
        CSSPropertyID::kFontFeatureSettings => AtRuleDescriptorID::FontFeatureSettings,
        CSSPropertyID::kFontStretch => AtRuleDescriptorID::FontStretch,
        CSSPropertyID::kFontStyle => AtRuleDescriptorID::FontStyle,
        CSSPropertyID::kFontVariant => AtRuleDescriptorID::FontVariant,
        CSSPropertyID::kFontVariationSettings => AtRuleDescriptorID::FontVariationSettings,
        CSSPropertyID::kFontWeight => AtRuleDescriptorID::FontWeight,
        CSSPropertyID::kHash => AtRuleDescriptorID::Hash,
        CSSPropertyID::kHostname => AtRuleDescriptorID::Hostname,
        CSSPropertyID::kInherits => AtRuleDescriptorID::Inherits,
        CSSPropertyID::kInitialValue => AtRuleDescriptorID::InitialValue,
        CSSPropertyID::kLineGapOverride => AtRuleDescriptorID::LineGapOverride,
        CSSPropertyID::kNavigation => AtRuleDescriptorID::Navigation,
        CSSPropertyID::kNegative => AtRuleDescriptorID::Negative,
        CSSPropertyID::kOverrideColors => AtRuleDescriptorID::OverrideColors,
        CSSPropertyID::kPad => AtRuleDescriptorID::Pad,
        CSSPropertyID::kPathname => AtRuleDescriptorID::Pathname,
        CSSPropertyID::kPattern => AtRuleDescriptorID::Pattern,
        CSSPropertyID::kPort => AtRuleDescriptorID::Port,
        CSSPropertyID::kPrefix => AtRuleDescriptorID::Prefix,
        CSSPropertyID::kProtocol => AtRuleDescriptorID::Protocol,
        CSSPropertyID::kRange => AtRuleDescriptorID::Range,
        CSSPropertyID::kResult => AtRuleDescriptorID::Result,
        CSSPropertyID::kSearch => AtRuleDescriptorID::Search,
        CSSPropertyID::kSizeAdjust => AtRuleDescriptorID::SizeAdjust,
        CSSPropertyID::kSpeakAs => AtRuleDescriptorID::SpeakAs,
        CSSPropertyID::kSrc => AtRuleDescriptorID::Src,
        CSSPropertyID::kSuffix => AtRuleDescriptorID::Suffix,
        CSSPropertyID::kSymbols => AtRuleDescriptorID::Symbols,
        CSSPropertyID::kSyntax => AtRuleDescriptorID::Syntax,
        CSSPropertyID::kSystem => AtRuleDescriptorID::System,
        CSSPropertyID::kTypes => AtRuleDescriptorID::Types,
        CSSPropertyID::kUnicodeRange => AtRuleDescriptorID::UnicodeRange,
        _ => AtRuleDescriptorID::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::String;

    #[test]
    fn descriptor_names_aliases_and_property_conversions() {
        for &(name, alias) in super::super::at_rule_names::AT_RULE_NAMES.iter() {
            let canonical = AsAtRuleDescriptorID(StringView::from(name));
            assert_ne!(canonical, AtRuleDescriptorID::Invalid, "{name}");
            assert_ne!(canonical, AtRuleDescriptorID::Variable, "{name}");
            assert_eq!(
                CSSPropertyIDAsAtRuleDescriptor(AtRuleDescriptorIDAsCSSPropertyID(canonical)),
                canonical
            );
            let uppercase = name.to_ascii_uppercase();
            assert_eq!(
                AsAtRuleDescriptorID(StringView::from(uppercase.as_str())),
                canonical
            );
            let latin1 = String::from_latin1(name.as_bytes());
            assert_eq!(AsAtRuleDescriptorID(StringView::from(&latin1)), canonical);
            if !alias.is_empty() {
                assert_eq!(AsAtRuleDescriptorID(StringView::from(alias)), canonical);
            }
        }
        assert_eq!(
            CSSPropertyIDAsAtRuleDescriptor(CSSPropertyID::kColor),
            AtRuleDescriptorID::Invalid
        );
    }

    #[test]
    fn custom_descriptor_detection_precedes_length_and_character_validation() {
        for name in [
            "--x",
            "--💡",
            "--abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz",
        ] {
            assert_eq!(
                AsAtRuleDescriptorID(StringView::from(name)),
                AtRuleDescriptorID::Variable
            );
        }
        let units = String::from_utf16(&[45, 45, 0]);
        assert_eq!(
            AsAtRuleDescriptorID(StringView::from(&units)),
            AtRuleDescriptorID::Variable
        );
        assert_eq!(
            AsAtRuleDescriptorID(StringView::from("--")),
            AtRuleDescriptorID::Invalid
        );
    }

    #[test]
    fn hash_collision_and_invalid_character_rejection() {
        // "crs" has the same generated hash as "src"; the final name check is
        // required even after a successful hash-table lookup.
        for name in [
            "",
            "crs",
            "src ",
            "src\0",
            "src\u{7f}",
            "sṛc",
            "font-feature-settings-extra-long",
        ] {
            assert_eq!(
                AsAtRuleDescriptorID(StringView::from(name)),
                AtRuleDescriptorID::Invalid,
                "{name:?}"
            );
        }
        let surrogate = String::from_utf16(&[115, 114, 0xd800]);
        assert_eq!(
            AsAtRuleDescriptorID(StringView::from(&surrogate)),
            AtRuleDescriptorID::Invalid
        );
    }
}
