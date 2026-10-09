// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/css_primitive_value.h:74-156,451-476
// cpp: third_party/blink/renderer/core/css/css_primitive_value.cc:842-977
// generated: out/Min/gen/third_party/blink/renderer/core/css/css_primitive_value_unit_trie.cc

#![allow(non_camel_case_types, non_snake_case)]

use foundation::StringView;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitType {
    #[default]
    kUnknown,
    kNumber,
    kPercentage,
    kEms,
    kExs,
    kPixels,
    kCentimeters,
    kMillimeters,
    kInches,
    kPoints,
    kPicas,
    kQuarterMillimeters,
    kViewportWidth,
    kViewportHeight,
    kViewportInlineSize,
    kViewportBlockSize,
    kViewportMin,
    kViewportMax,
    kSmallViewportWidth,
    kSmallViewportHeight,
    kSmallViewportInlineSize,
    kSmallViewportBlockSize,
    kSmallViewportMin,
    kSmallViewportMax,
    kLargeViewportWidth,
    kLargeViewportHeight,
    kLargeViewportInlineSize,
    kLargeViewportBlockSize,
    kLargeViewportMin,
    kLargeViewportMax,
    kDynamicViewportWidth,
    kDynamicViewportHeight,
    kDynamicViewportInlineSize,
    kDynamicViewportBlockSize,
    kDynamicViewportMin,
    kDynamicViewportMax,
    kContainerWidth,
    kContainerHeight,
    kContainerInlineSize,
    kContainerBlockSize,
    kContainerMin,
    kContainerMax,
    kRems,
    kRexs,
    kRchs,
    kRics,
    kChs,
    kIcs,
    kLhs,
    kRlhs,
    kCaps,
    kRcaps,
    kUserUnits,
    kDegrees,
    kRadians,
    kGradians,
    kTurns,
    kMilliseconds,
    kSeconds,
    kHertz,
    kKilohertz,
    kDotsPerPixel,
    kX,
    kDotsPerInch,
    kDotsPerCentimeter,
    kFlex,
    kInteger,
    kIdent,
    kQuirkyEms,
}

const UNIT_NAMES: [(&str, UnitType); 63] = [
    ("em", UnitType::kEms),
    ("ex", UnitType::kExs),
    ("px", UnitType::kPixels),
    ("cm", UnitType::kCentimeters),
    ("mm", UnitType::kMillimeters),
    ("q", UnitType::kQuarterMillimeters),
    ("in", UnitType::kInches),
    ("pt", UnitType::kPoints),
    ("pc", UnitType::kPicas),
    ("deg", UnitType::kDegrees),
    ("rad", UnitType::kRadians),
    ("grad", UnitType::kGradians),
    ("ms", UnitType::kMilliseconds),
    ("s", UnitType::kSeconds),
    ("hz", UnitType::kHertz),
    ("khz", UnitType::kKilohertz),
    ("dpi", UnitType::kDotsPerInch),
    ("dpcm", UnitType::kDotsPerCentimeter),
    ("dppx", UnitType::kDotsPerPixel),
    ("x", UnitType::kX),
    ("vw", UnitType::kViewportWidth),
    ("vh", UnitType::kViewportHeight),
    ("vi", UnitType::kViewportInlineSize),
    ("vb", UnitType::kViewportBlockSize),
    ("vmin", UnitType::kViewportMin),
    ("vmax", UnitType::kViewportMax),
    ("svw", UnitType::kSmallViewportWidth),
    ("svh", UnitType::kSmallViewportHeight),
    ("svi", UnitType::kSmallViewportInlineSize),
    ("svb", UnitType::kSmallViewportBlockSize),
    ("svmin", UnitType::kSmallViewportMin),
    ("svmax", UnitType::kSmallViewportMax),
    ("lvw", UnitType::kLargeViewportWidth),
    ("lvh", UnitType::kLargeViewportHeight),
    ("lvi", UnitType::kLargeViewportInlineSize),
    ("lvb", UnitType::kLargeViewportBlockSize),
    ("lvmin", UnitType::kLargeViewportMin),
    ("lvmax", UnitType::kLargeViewportMax),
    ("dvw", UnitType::kDynamicViewportWidth),
    ("dvh", UnitType::kDynamicViewportHeight),
    ("dvi", UnitType::kDynamicViewportInlineSize),
    ("dvb", UnitType::kDynamicViewportBlockSize),
    ("dvmin", UnitType::kDynamicViewportMin),
    ("dvmax", UnitType::kDynamicViewportMax),
    ("cqw", UnitType::kContainerWidth),
    ("cqh", UnitType::kContainerHeight),
    ("cqi", UnitType::kContainerInlineSize),
    ("cqb", UnitType::kContainerBlockSize),
    ("cqmin", UnitType::kContainerMin),
    ("cqmax", UnitType::kContainerMax),
    ("rem", UnitType::kRems),
    ("rex", UnitType::kRexs),
    ("rch", UnitType::kRchs),
    ("ric", UnitType::kRics),
    ("fr", UnitType::kFlex),
    ("turn", UnitType::kTurns),
    ("ch", UnitType::kChs),
    ("ic", UnitType::kIcs),
    ("lh", UnitType::kLhs),
    ("rlh", UnitType::kRlhs),
    ("cap", UnitType::kCaps),
    ("rcap", UnitType::kRcaps),
    ("__qem", UnitType::kQuirkyEms),
];

fn EqualIgnoringAsciiCase(view: &StringView, ascii: &[u8]) -> bool {
    let units = view.Span16();
    units.len() == ascii.len()
        && units
            .iter()
            .zip(ascii)
            .all(|(&unit, &byte)| unit <= 0x7f && (unit as u8).to_ascii_lowercase() == byte)
}

pub fn StringToUnitType(string: &StringView) -> UnitType {
    UNIT_NAMES
        .iter()
        .find(|(name, _)| EqualIgnoringAsciiCase(string, name.as_bytes()))
        .map_or(UnitType::kUnknown, |(_, unit)| *unit)
}

pub const fn UnitTypeToString(unit: UnitType) -> Option<&'static str> {
    match unit {
        UnitType::kNumber | UnitType::kInteger | UnitType::kUserUnits => Some(""),
        UnitType::kPercentage => Some("%"),
        UnitType::kEms | UnitType::kQuirkyEms => Some("em"),
        UnitType::kExs => Some("ex"),
        UnitType::kPixels => Some("px"),
        UnitType::kCentimeters => Some("cm"),
        UnitType::kMillimeters => Some("mm"),
        UnitType::kInches => Some("in"),
        UnitType::kPoints => Some("pt"),
        UnitType::kPicas => Some("pc"),
        UnitType::kQuarterMillimeters => Some("q"),
        UnitType::kViewportWidth => Some("vw"),
        UnitType::kViewportHeight => Some("vh"),
        UnitType::kViewportInlineSize => Some("vi"),
        UnitType::kViewportBlockSize => Some("vb"),
        UnitType::kViewportMin => Some("vmin"),
        UnitType::kViewportMax => Some("vmax"),
        UnitType::kSmallViewportWidth => Some("svw"),
        UnitType::kSmallViewportHeight => Some("svh"),
        UnitType::kSmallViewportInlineSize => Some("svi"),
        UnitType::kSmallViewportBlockSize => Some("svb"),
        UnitType::kSmallViewportMin => Some("svmin"),
        UnitType::kSmallViewportMax => Some("svmax"),
        UnitType::kLargeViewportWidth => Some("lvw"),
        UnitType::kLargeViewportHeight => Some("lvh"),
        UnitType::kLargeViewportInlineSize => Some("lvi"),
        UnitType::kLargeViewportBlockSize => Some("lvb"),
        UnitType::kLargeViewportMin => Some("lvmin"),
        UnitType::kLargeViewportMax => Some("lvmax"),
        UnitType::kDynamicViewportWidth => Some("dvw"),
        UnitType::kDynamicViewportHeight => Some("dvh"),
        UnitType::kDynamicViewportInlineSize => Some("dvi"),
        UnitType::kDynamicViewportBlockSize => Some("dvb"),
        UnitType::kDynamicViewportMin => Some("dvmin"),
        UnitType::kDynamicViewportMax => Some("dvmax"),
        UnitType::kContainerWidth => Some("cqw"),
        UnitType::kContainerHeight => Some("cqh"),
        UnitType::kContainerInlineSize => Some("cqi"),
        UnitType::kContainerBlockSize => Some("cqb"),
        UnitType::kContainerMin => Some("cqmin"),
        UnitType::kContainerMax => Some("cqmax"),
        UnitType::kRems => Some("rem"),
        UnitType::kRexs => Some("rex"),
        UnitType::kRchs => Some("rch"),
        UnitType::kRics => Some("ric"),
        UnitType::kChs => Some("ch"),
        UnitType::kIcs => Some("ic"),
        UnitType::kLhs => Some("lh"),
        UnitType::kRlhs => Some("rlh"),
        UnitType::kCaps => Some("cap"),
        UnitType::kRcaps => Some("rcap"),
        UnitType::kDegrees => Some("deg"),
        UnitType::kRadians => Some("rad"),
        UnitType::kGradians => Some("grad"),
        UnitType::kTurns => Some("turn"),
        UnitType::kMilliseconds => Some("ms"),
        UnitType::kSeconds => Some("s"),
        UnitType::kHertz => Some("hz"),
        UnitType::kKilohertz => Some("khz"),
        UnitType::kDotsPerPixel => Some("dppx"),
        UnitType::kX => Some("x"),
        UnitType::kDotsPerInch => Some("dpi"),
        UnitType::kDotsPerCentimeter => Some("dpcm"),
        UnitType::kFlex => Some("fr"),
        UnitType::kUnknown | UnitType::kIdent => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::String;

    #[test]
    fn all_generated_units_match_for_both_storage_widths_and_case() {
        for (name, expected) in UNIT_NAMES {
            assert_eq!(StringToUnitType(&StringView::from(name)), expected);
            let upper = name.to_ascii_uppercase();
            let latin1 = String::from_latin1(upper.as_bytes());
            assert_eq!(StringToUnitType(&StringView::from(&latin1)), expected);
        }
        assert_eq!(
            StringToUnitType(&StringView::from("pixels")),
            UnitType::kUnknown
        );
        assert_eq!(StringToUnitType(&StringView::from("é")), UnitType::kUnknown);
    }

    #[test]
    fn source_string_mapping_includes_non_parsed_units() {
        assert_eq!(UnitTypeToString(UnitType::kNumber), Some(""));
        assert_eq!(UnitTypeToString(UnitType::kPercentage), Some("%"));
        assert_eq!(UnitTypeToString(UnitType::kQuirkyEms), Some("em"));
        assert_eq!(UnitTypeToString(UnitType::kUnknown), None);
    }
}

// cpp: css_primitive_value.cc:492-551. Relative units are converted by the
// required CSSLengthResolver; this scale converts canonical absolute units.
pub fn ConversionToCanonicalUnitsScaleFactor(unit: UnitType) -> f64 {
    use UnitType::*;
    match unit {
        kMilliseconds => 0.001,
        kCentimeters => 96.0 / 2.54,
        kDotsPerCentimeter => 2.54 / 96.0,
        kMillimeters => 96.0 / 25.4,
        kQuarterMillimeters => 96.0 / 101.6,
        kInches => 96.0,
        kDotsPerInch => 1.0 / 96.0,
        kPoints => 96.0 / 72.0,
        kPicas => 16.0,
        kRadians => 180.0 / std::f64::consts::PI,
        kGradians => 0.9,
        kTurns => 360.0,
        kKilohertz => 1000.0,
        _ => 1.0,
    }
}
