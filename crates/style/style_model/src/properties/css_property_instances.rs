// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.h:845-863
// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.cc:4985-7530
// The C++ union's concrete constructors are represented by their actual
// CSSProperty base metadata. Alias identities remain separate from resolved
// objects. Concrete longhand/shorthand parser, DOM and computed-style virtual
// methods and specialized typed getters are not implemented by this table.

#![allow(non_upper_case_globals)]

use super::css_property::CSSProperty;
use foundation::CSSPropertyID;
use std::sync::LazyLock;

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.h:850
// Source ABI width; Rust uses typed indexing rather than raw pointer strides.
pub const kCSSPropertyUnionBytes: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSPropertyUnion {
    Invalid,
    Resolved(CSSProperty),
    Alias(CSSPropertyID),
}

impl CSSPropertyUnion {
    pub fn AsResolvedProperty(&self) -> Option<&CSSProperty> {
        match self {
            Self::Resolved(property) => Some(property),
            Self::Invalid | Self::Alias(_) => None,
        }
    }
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.cc:4985-5812
pub static kCssProperties: LazyLock<[CSSPropertyUnion; 826]> = LazyLock::new(|| {
    [
        CSSPropertyUnion::Invalid,
        // cpp: third_party/blink/renderer/core/css/properties/longhands/variable.h:22,35-41
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kVariable,
            CSSProperty::kLonghand
                | CSSProperty::kProperty
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForHighlightLegacy
                | CSSProperty::kValidForHighlight
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kNotLegacyOverlapping
                | 1,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:37
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColorScheme,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:55
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kForcedColorAdjust,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:72
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMathDepth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:89
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPosition,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:105
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPositionAnchor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:122
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextSizeAdjust,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:139
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kVisitedHighlightColors
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:157
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAppearance,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:174
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kHighlightColors
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:192
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kDirection,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:209
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontFamily,
            CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:226
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontFeatureSettings,
            CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:243
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontKerning,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:259
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontLanguageOverride,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:277
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontOpticalSizing,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:293
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontPalette,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:310
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontSize,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:327
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontSizeAdjust,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:345
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontStretch,
            CSSProperty::kInterpolable
                | CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:362
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontStyle,
            CSSProperty::kInterpolable
                | CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:379
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontSynthesisSmallCaps,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:395
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontSynthesisStyle,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:411
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontSynthesisWeight,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:427
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantAlternates,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:444
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantCaps,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:461
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantEastAsian,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:478
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantEmoji,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:494
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantLigatures,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:511
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantNumeric,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:528
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariantPosition,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:544
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariationSettings,
            CSSProperty::kInterpolable
                | CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:561
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontWeight,
            CSSProperty::kInterpolable
                | CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:578
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPositionArea,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:595
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextOrientation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:611
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextRendering,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:627
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextSpacingTrim,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:643
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitFontSmoothing,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:659
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitLocale,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kAffectsFont
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:676
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextOrientation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:690
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitWritingMode,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:704
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWritingMode,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:720
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kZoom,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:737
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalForcedVisitedColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:755
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBackgroundColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kVisitedHighlightColors
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:773
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderBlockEndColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:800
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderBlockStartColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:827
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderBottomColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:848
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderInlineEndColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:875
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderInlineStartColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:902
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderLeftColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:923
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderRightColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:944
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedBorderTopColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:965
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedCaretColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:983
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedColumnRuleColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1001
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedFill,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1019
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedOutlineColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1037
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedStroke,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1055
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedTextDecorationColor,
            CSSProperty::kProperty
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1073
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedTextEmphasisColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1091
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedTextFillColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1109
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalVisitedTextStrokeColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1127
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAccentColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1145
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAdditiveSymbols,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1157
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlignContent,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1174
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlignItems,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1191
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlignSelf,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1208
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlignmentBaseline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1224
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAll,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1240
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlternativeWebkitLineClampLonghand,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1261
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnchorName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1278
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnchorScope,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1295
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationComposition,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1313
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationDelay,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1331
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationDirection,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1349
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationDuration,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1367
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationFillMode,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1385
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationIterationCount,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1403
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1421
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationPlayState,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1439
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationRangeEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1457
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationRangeStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1475
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationTimeline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1493
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationTimingFunction,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1511
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationTrigger,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1530
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAppRegion,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1544
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAscentOverride,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1556
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAspectRatio,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1573
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackdropFilter,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1591
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackfaceVisibility,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1607
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundAttachment,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1624
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundBlendMode,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1641
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundClip,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1658
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundColor,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kBackground
                | CSSProperty::kHighlightColors
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1677
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundImage,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1695
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundOrigin,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1712
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundPositionX,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1729
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundPositionY,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1746
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundRepeat,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1763
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundSize,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kBackground
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1780
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBasePalette,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1792
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBaseUrl,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1805
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBaselineShift,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1822
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBaselineSource,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1838
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBlockEllipsis,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1855
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBlockSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1883
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockEndColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1909
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockEndStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1934
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockEndWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1960
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockStartColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:1986
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockStartStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2011
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockStartWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2037
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBottomColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2060
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBottomLeftRadius,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kBorderRadius
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2080
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBottomRightRadius,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kBorderRadius
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2100
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBottomStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2120
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBottomWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2141
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderCollapse,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2157
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderEndEndRadius,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2183
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderEndStartRadius,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2209
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderImageOutset,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kBorder
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2227
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderImageRepeat,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kBorder
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2245
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderImageSlice,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kBorder
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2263
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderImageSource,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kBorder
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2282
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderImageWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kBorder
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2300
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineEndColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2326
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineEndStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2351
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineEndWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2377
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineStartColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2403
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineStartStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2428
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineStartWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2454
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderLeftColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2477
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderLeftStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2497
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderLeftWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2518
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderRightColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2541
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderRightStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2561
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderRightWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2582
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderShape,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2600
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderStartEndRadius,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2626
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderStartStartRadius,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2652
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderTopColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2675
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderTopLeftRadius,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kBorderRadius
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2695
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderTopRightRadius,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kBorderRadius
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2715
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderTopStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2735
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderTopWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kBorder
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2756
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBottom,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2778
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBoxDecorationBreak,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2794
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBoxShadow,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2812
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBoxSizing,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2828
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBreakAfter,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2844
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBreakBefore,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2860
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBreakInside,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2876
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBufferedRendering,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2892
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCaptionSide,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2908
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCaretAnimation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2925
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCaretColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForVisited
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2944
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCaretShape,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2961
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kClear,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2977
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kClip,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:2994
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kClipPath,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3011
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kClipRule,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3027
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColorInterpolation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3043
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColorInterpolationFilters,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3059
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColorRendering,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3075
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnCount,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3092
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnFill,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3108
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnGap,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3125
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnHeight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3142
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleBreak,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3158
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForVisited
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3177
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetCapEnd,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3194
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetCapStart,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3211
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetJunctionEnd,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3228
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetJunctionStart,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3245
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3262
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleVisibilityItems,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3278
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3295
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnSpan,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3311
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3328
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnWrap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3344
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContain,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3361
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainIntrinsicBlockSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3387
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainIntrinsicHeight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3404
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainIntrinsicInlineSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3430
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainIntrinsicWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3447
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainerName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3464
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainerType,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3481
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContent,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3498
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContentVisibility,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3514
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContinue,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3531
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBottomLeftShape,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3551
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBottomRightShape,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3571
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerEndEndShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3597
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerEndStartShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3623
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerStartEndShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3649
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerStartStartShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3675
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerTopLeftShape,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3695
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerTopRightShape,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3715
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCounterIncrement,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3732
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCounterReset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3749
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCounterSet,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3766
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCursor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3783
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCx,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3800
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCy,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3817
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kD,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3834
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kDescentOverride,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3846
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kDisplay,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3863
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kDominantBaseline,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3879
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kDynamicRangeLimit,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3897
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kEmptyCells,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3913
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFallback,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3925
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFieldSizing,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3941
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFill,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kValidForVisited
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3960
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFillOpacity,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3977
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFillRule,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:3993
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFilter,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4011
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexBasis,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4028
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexDirection,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4045
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexGrow,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4062
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexLineCount,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4080
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexShrink,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4097
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexWrap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4115
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFloat,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4131
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFloodColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4150
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFloodOpacity,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4167
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlowTolerance,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4185
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontDisplay,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4197
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFrameSizing,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4214
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridAutoColumns,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4232
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridAutoFlow,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4250
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridAutoRows,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4268
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridColumnEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4285
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridColumnStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4302
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridLanesDirection,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4320
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridLanesPack,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4337
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridRowEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4354
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridRowStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4371
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridTemplateAreas,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4389
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridTemplateColumns,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4408
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridTemplateRows,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4427
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHangingPunctuation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4445
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHash,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4458
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHeight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4480
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHostname,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4493
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHyphenateCharacter,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4510
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHyphenateLimitChars,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4527
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kHyphens,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4543
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kImageAnimation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4560
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kImageOrientation,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4577
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kImageRendering,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4593
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInherits,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4605
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInitialLetter,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4622
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInitialValue,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4634
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInlineSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4662
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInsetBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4690
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInsetBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4718
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInsetInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4746
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInsetInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4774
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInteractivity,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4790
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInterestDelayEnd,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4807
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInterestDelayStart,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4824
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalAlignContentBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4841
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalEmptyLineHeight,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4858
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalFontSizeDelta,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4875
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalForcedBackgroundColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4894
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalForcedBorderColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4913
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalForcedColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForVisited
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4932
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalForcedOutlineColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4951
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalOverscrollContainer,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4968
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalOverscrollPosition,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:4985
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInternalUnbounded,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kInternal
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5002
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInterpolateSize,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5018
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kIsolation,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5034
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kJustifyContent,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5051
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kJustifyItems,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5068
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kJustifySelf,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5085
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLeft,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5107
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLetterSpacing,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5124
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLightingColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5143
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLineBreak,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5159
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLineClamp,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5180
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLineGapOverride,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5192
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kLineHeight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5209
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kListStyleImage,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5227
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kListStylePosition,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5243
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kListStyleType,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5260
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5288
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5316
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginBottom,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5338
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5366
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5394
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginLeft,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5416
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginRight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5438
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginTop,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5460
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginTrim,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5478
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarkerEnd,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5495
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarkerMid,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5512
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarkerStart,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5529
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskClip,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5547
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskImage,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5565
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskMode,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5583
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskOrigin,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5601
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskRepeat,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5619
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskSize,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5637
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskType,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5653
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMathShift,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5669
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMathStyle,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5685
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaxBlockSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5711
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaxContentSizing,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5728
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaxHeight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5748
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaxInlineSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5774
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaxLines,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5792
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaxWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5812
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMinBlockSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5838
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMinHeight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5858
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMinInlineSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5884
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMinWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5904
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMixBlendMode,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5920
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kNavigation,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5932
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kNegative,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5944
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kObjectFit,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5960
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kObjectPosition,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5977
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kObjectViewBox,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:5994
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOffsetAnchor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6011
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOffsetDistance,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6028
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOffsetPath,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6045
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOffsetPosition,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6062
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOffsetRotate,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6079
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOpacity,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6096
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOrder,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6113
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOriginTrialTestProperty,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6130
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOrphans,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6147
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOutlineColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForVisited
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6166
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOutlineOffset,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6183
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOutlineStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6199
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOutlineWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6216
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowAnchor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6232
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6257
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowClipMargin,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6274
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6299
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowWrap,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6315
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowX,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6334
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflowY,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6353
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverlay,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6370
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverrideColors,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6382
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverscrollBehaviorBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6407
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverscrollBehaviorInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6432
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverscrollBehaviorX,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6451
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverscrollBehaviorY,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6470
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverscrollContainerType,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6487
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPad,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6499
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6527
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6555
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingBottom,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6577
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6605
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6633
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingLeft,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6655
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingRight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6677
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingTop,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6699
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPage,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6716
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPageMarginSafety,
            CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6732
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPageOrientation,
            CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6747
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaintOrder,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6764
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPathLength,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6782
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPathname,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6795
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPattern,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6808
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPerspective,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6825
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPerspectiveOrigin,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6844
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPointerEvents,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6860
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPort,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6873
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPositionTryFallbacks,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6890
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPositionTryOrder,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6907
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPositionVisibility,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6924
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPrefix,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6936
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPrintColorAdjust,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6952
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kProtocol,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6965
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kQuotes,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6982
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kR,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:6999
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRange,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7011
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kReadingFlow,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7027
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kReadingOrder,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7044
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kResize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7060
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kResult,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7073
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRight,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7095
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRotate,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7112
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowGap,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7129
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleBreak,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7145
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7164
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetCapEnd,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7181
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetCapStart,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7198
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetJunctionEnd,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7215
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetJunctionStart,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7232
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7249
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleVisibilityItems,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7265
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7282
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRubyAlign,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7298
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRubyOverhang,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7316
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRubyPosition,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7333
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleOverlap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7349
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRx,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7366
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRy,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7383
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScale,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7400
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollAxisLock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7417
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollBehavior,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7433
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollInitialTarget,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7450
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7476
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7502
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginBottom,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7522
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7548
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7574
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginLeft,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7594
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginRight,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7614
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginTop,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7634
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarkerGroup,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7652
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7678
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7704
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingBottom,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7724
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7750
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7776
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingLeft,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7796
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingRight,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7816
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingTop,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7836
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollSnapAlign,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7853
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollSnapStop,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7869
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollSnapType,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7886
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollTargetGroup,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7903
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollTimelineAxis,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7921
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollTimelineName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7939
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollbarColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7957
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollbarGutter,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7974
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollbarWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:7991
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSearch,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8004
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kShapeImageThreshold,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8021
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kShapeMargin,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8038
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kShapeOutside,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8055
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kShapeRendering,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8071
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8087
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSizeAdjust,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8099
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSpeak,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8115
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSpeakAs,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8127
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSrc,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8139
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStopColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8158
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStopOpacity,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8175
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStroke,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kValidForVisited
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8194
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeDasharray,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8211
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeDashoffset,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8228
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeLinecap,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8244
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeLinejoin,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8260
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeMiterlimit,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8277
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeOpacity,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8294
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kStrokeWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kValidForHighlight
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8311
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSuffix,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8323
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSymbols,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8335
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSyntax,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8347
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kSystem,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8359
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTabSize,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8376
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTableLayout,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8392
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextAlign,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8408
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextAlignLast,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8424
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextAnchor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8440
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextAutospace,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8456
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextBoxEdge,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8473
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextBoxTrim,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8489
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextCombineUpright,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8505
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForVisited
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8524
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationInset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8542
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationLine,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8559
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationSkipInk,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8575
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationSkipSpaces,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            ' ',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8593
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8609
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecorationThickness,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8626
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextEmphasisColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForVisited
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8645
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextEmphasisPosition,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8662
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextEmphasisStyle,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8679
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextFit,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8697
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextIndent,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8714
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextJustify,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8731
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextOverflow,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8748
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextShadow,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8766
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextTransform,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8783
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextUnderlineOffset,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForHighlight
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8800
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextUnderlinePosition,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8817
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextWrapMode,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8833
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextWrapStyle,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8849
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineScope,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8866
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerActivationRangeEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8885
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerActivationRangeStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8904
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerActiveRangeEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8923
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerActiveRangeStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8942
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8961
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerSource,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:8980
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTop,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9002
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTouchAction,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9019
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransform,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9038
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransformBox,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9054
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransformOrigin,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9073
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransformStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9089
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransitionBehavior,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9107
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransitionDelay,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9125
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransitionDuration,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9143
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransitionProperty,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9161
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransitionTimingFunction,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9179
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTranslate,
            CSSProperty::kInterpolable
                | CSSProperty::kCompositableProperty
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9198
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTriggerScope,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9216
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTypes,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9228
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kUnicodeBidi,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9245
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kUnicodeRange,
            CSSProperty::kDescriptor
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9257
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kUserSelect,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9273
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kVectorEffect,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9289
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kVerticalAlign,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9306
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTimelineAxis,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9324
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTimelineInset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9342
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTimelineName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            ',',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9360
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTransitionClass,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9377
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTransitionGroup,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9394
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTransitionName,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9411
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTransitionScope,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9427
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kVisibility,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9443
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBorderHorizontalSpacing,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9460
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBorderImage,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9478
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBorderVerticalSpacing,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9495
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxAlign,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9511
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxDecorationBreak,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9525
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxDirection,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9541
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxFlex,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kAcceptsNumericLiteral
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9558
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxOrdinalGroup,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9575
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxOrient,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9591
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxPack,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9607
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitBoxReflect,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9625
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitLineBreak,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9639
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitLineClamp,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9660
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskBoxImageOutset,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9677
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskBoxImageRepeat,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9694
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskBoxImageSlice,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9711
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskBoxImageSource,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9729
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskBoxImageWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9746
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskPositionX,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9764
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskPositionY,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9782
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitPerspectiveOriginX,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9799
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitPerspectiveOriginY,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9816
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitRtlOrdering,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9832
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitRubyPosition,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9846
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTapHighlightColor,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9865
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextCombine,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kSurrogate
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9879
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextDecorationsInEffect,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9896
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextFillColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForVisited
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9914
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextSecurity,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9930
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextStrokeColor,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForVisited
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9949
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextStrokeWidth,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9966
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTransformOriginX,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:9983
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTransformOriginY,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10000
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTransformOriginZ,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kOverlapping
                | CSSProperty::kLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10017
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitUserDrag,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10033
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitUserModify,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10050
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWhiteSpaceCollapse,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForCue
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10066
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWidows,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10083
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWidth,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10105
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWillChange,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10122
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWindowDrag,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10139
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWordBreak,
            CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10155
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWordSpacing,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kInherited
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForFirstLetter
                | CSSProperty::kValidForFirstLine
                | CSSProperty::kValidForMarker
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDoNotDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10172
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kX,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10189
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kY,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kPercentagesDependOnUsedValue
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10206
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kZIndex,
            CSSProperty::kInterpolable
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.h:10223
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskComposite,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kLonghand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:37
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlternativeLineClampShorthand,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:52
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAlternativeWebkitLineClampShorthand,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:67
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimation,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:81
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kAnimationRange,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:95
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackground,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:109
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBackgroundPosition,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:123
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorder,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:137
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:151
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:165
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:191
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:217
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:231
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBlockWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:245
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderBottom,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:262
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:276
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderImage,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:290
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:304
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:318
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:344
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kSurrogate
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:370
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:384
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderInlineWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:398
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderLeft,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:415
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderRadius,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:429
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderRight,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:446
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderSpacing,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:460
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:474
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderTop,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kInLogicalPropertyGroup
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:491
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kBorderWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:505
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRule,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:519
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:533
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetCap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:547
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:561
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetJunction,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:575
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumnRuleInsetStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:589
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kColumns,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:603
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainIntrinsicSize,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:617
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kContainer,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:631
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCorner,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:646
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBlockEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:661
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBlockEndShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:675
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBlockStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:690
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBlockStartShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:704
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBottom,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:719
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBottomLeft,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:734
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBottomRight,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:749
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerBottomShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:763
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerEndEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:778
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerEndStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:793
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerInlineEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:808
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerInlineEndShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:822
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerInlineStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:837
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerInlineStartShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:851
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerLeft,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:866
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerLeftShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:880
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerRight,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:895
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerRightShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:909
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:923
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerStartEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:938
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerStartStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:953
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerTop,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:968
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerTopLeft,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:983
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerTopRight,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:998
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kCornerTopShape,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1012
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlex,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1026
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFlexFlow,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1040
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFont,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1054
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontSynthesis,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1068
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kFontVariant,
            CSSProperty::kDescriptor
                | CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1082
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1096
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGrid,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1112
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridArea,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1126
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridColumn,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1140
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridLanes,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1157
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridRow,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1171
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kGridTemplate,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1187
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1203
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInsetBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1219
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInsetInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1235
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kInterestDelay,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1249
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kListStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1263
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMargin,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kValidForPermissionElement
                | CSSProperty::kValidForPermissionIcon
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1279
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1295
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarginInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1311
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMarker,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1325
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMask,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1339
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kMaskPosition,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1353
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOffset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1367
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOutline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1381
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverflow,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1395
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kOverscrollBehavior,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1409
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPadding,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kSupportsIncrementalStyle
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1425
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1439
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPaddingInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1453
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPageBreakAfter,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1467
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPageBreakBefore,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1481
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPageBreakInside,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1495
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPlaceContent,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1509
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPlaceItems,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1523
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPlaceSelf,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPositionTry
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1537
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kPositionTry,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1551
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRule,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1565
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1579
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetCap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1593
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1607
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetJunction,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1621
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRowRuleInsetStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1635
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRule,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1649
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleBreak,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1663
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleColor,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1677
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleInset,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1691
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleInsetCap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1705
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleInsetEnd,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1719
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleInsetJunction,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1733
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleInsetStart,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1747
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleStyle,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1761
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleVisibilityItems,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1775
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kRuleWidth,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1789
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMargin,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1803
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1817
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollMarginInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1831
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPadding,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1845
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingBlock,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1859
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollPaddingInline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1873
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kScrollTimeline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1887
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextBox,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1901
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextDecoration,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1915
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextEmphasis,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1929
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextSpacing,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1944
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTextWrap,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1958
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTrigger,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1973
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerActivationRange,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:1988
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTimelineTriggerActiveRange,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2003
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kTransition,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2017
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kViewTimeline,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2031
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitColumnBreakAfter,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2045
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitColumnBreakBefore,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2059
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitColumnBreakInside,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2073
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitMaskBoxImage,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2087
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWebkitTextStroke,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kShorthand,
            '\0',
        )),
        // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.h:2101
        CSSPropertyUnion::Resolved(CSSProperty::new(
            CSSPropertyID::kWhiteSpace,
            CSSProperty::kProperty
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kIdempotent
                | CSSProperty::kNotLegacyOverlapping
                | CSSProperty::kValidForKeyframe
                | CSSProperty::kValidForPageContext
                | CSSProperty::kShorthand,
            '\0',
        )),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAppearance),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAppRegion),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskClip),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskComposite),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskImage),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskOrigin),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskRepeat),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskSize),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderEndColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderEndStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderEndWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderStartColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderStartStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderStartWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderBeforeColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderBeforeStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderBeforeWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderAfterColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderAfterStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderAfterWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMarginEnd),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMarginStart),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMarginBefore),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMarginAfter),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPaddingEnd),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPaddingStart),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPaddingBefore),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPaddingAfter),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitLogicalWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitLogicalHeight),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMinLogicalWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMinLogicalHeight),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaxLogicalWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaxLogicalHeight),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPrintColorAdjust),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderAfter),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderBefore),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderEnd),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderStart),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMask),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitMaskPosition),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubCaptionSide),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubTextCombine),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubTextEmphasis),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubTextEmphasisColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubTextEmphasisStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubTextOrientation),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubTextTransform),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubWordBreak),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasEpubWritingMode),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAlignContent),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAlignItems),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAlignSelf),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimation),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationDelay),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationDirection),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationDuration),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationFillMode),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationIterationCount),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationName),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationPlayState),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitAnimationTimingFunction),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBackfaceVisibility),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBackgroundClip),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBackgroundOrigin),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBackgroundSize),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderBottomLeftRadius),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderBottomRightRadius),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderRadius),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderTopLeftRadius),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBorderTopRightRadius),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBoxShadow),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitBoxSizing),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitClipPath),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnCount),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnGap),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnRule),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnRuleColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnRuleStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnRuleWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnSpan),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumnWidth),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitColumns),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFilter),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlex),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlexBasis),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlexDirection),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlexFlow),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlexGrow),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlexShrink),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFlexWrap),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitFontFeatureSettings),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitHyphenateCharacter),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitJustifyContent),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitOpacity),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitOrder),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPerspective),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitPerspectiveOrigin),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitShapeImageThreshold),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitShapeMargin),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitShapeOutside),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTextEmphasis),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTextEmphasisColor),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTextEmphasisPosition),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTextEmphasisStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTextSizeAdjust),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransform),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransformOrigin),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransformStyle),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransition),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransitionDelay),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransitionDuration),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransitionProperty),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitTransitionTimingFunction),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWebkitUserSelect),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasWordWrap),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasGridColumnGap),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasGridRowGap),
        CSSPropertyUnion::Alias(CSSPropertyID::kAliasGridGap),
    ]
});

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.h:859-863
pub fn GetPropertyInternal(id: CSSPropertyID) -> &'static CSSPropertyUnion {
    &kCssProperties[id as usize]
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.cc:5816-6643
pub const kPropertyVisitedIDs: [u8; 826] = [
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBackgroundColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderBlockEndColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderBlockStartColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderBottomColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderInlineEndColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderInlineStartColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderLeftColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderRightColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderTopColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedCaretColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedColumnRuleColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedFill as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalForcedVisitedColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedOutlineColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedStroke as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedTextDecorationColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedTextEmphasisColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedTextFillColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedTextStrokeColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderInlineEndColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderInlineStartColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderBlockStartColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedBorderBlockEndColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedTextEmphasisColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedColumnRuleColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInternalVisitedTextEmphasisColor as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
    CSSPropertyID::kInvalid as u8,
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.cc:6649-6676
const _: () = {
    assert!((CSSPropertyID::kInvalid as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBackgroundColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderBlockEndColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderBlockStartColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderBottomColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderInlineEndColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderInlineStartColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderLeftColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderRightColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedBorderTopColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedCaretColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedColumnRuleColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedFill as usize) < 256);
    assert!((CSSPropertyID::kInternalForcedVisitedColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedOutlineColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedStroke as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedTextDecorationColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedTextEmphasisColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedTextFillColor as usize) < 256);
    assert!((CSSPropertyID::kInternalVisitedTextStrokeColor as usize) < 256);
};

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.cc:6680-7507
pub const kPropertyUnvisitedIDs: [u16; 826] = [
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kColor as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInternalForcedColor as u16,
    CSSPropertyID::kBackgroundColor as u16,
    CSSPropertyID::kBorderBlockEndColor as u16,
    CSSPropertyID::kBorderBlockStartColor as u16,
    CSSPropertyID::kBorderBottomColor as u16,
    CSSPropertyID::kBorderInlineEndColor as u16,
    CSSPropertyID::kBorderInlineStartColor as u16,
    CSSPropertyID::kBorderLeftColor as u16,
    CSSPropertyID::kBorderRightColor as u16,
    CSSPropertyID::kBorderTopColor as u16,
    CSSPropertyID::kCaretColor as u16,
    CSSPropertyID::kColumnRuleColor as u16,
    CSSPropertyID::kFill as u16,
    CSSPropertyID::kOutlineColor as u16,
    CSSPropertyID::kStroke as u16,
    CSSPropertyID::kTextDecorationColor as u16,
    CSSPropertyID::kTextEmphasisColor as u16,
    CSSPropertyID::kWebkitTextFillColor as u16,
    CSSPropertyID::kWebkitTextStrokeColor as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
    CSSPropertyID::kInvalid as u16,
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/css_property_instances.cc:7510-7530
const _: () = {
    assert!((CSSPropertyID::kInvalid as usize) < 65536);
    assert!((CSSPropertyID::kColor as usize) < 65536);
    assert!((CSSPropertyID::kInternalForcedColor as usize) < 65536);
    assert!((CSSPropertyID::kBackgroundColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderBlockEndColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderBlockStartColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderBottomColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderInlineEndColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderInlineStartColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderLeftColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderRightColor as usize) < 65536);
    assert!((CSSPropertyID::kBorderTopColor as usize) < 65536);
    assert!((CSSPropertyID::kCaretColor as usize) < 65536);
    assert!((CSSPropertyID::kColumnRuleColor as usize) < 65536);
    assert!((CSSPropertyID::kFill as usize) < 65536);
    assert!((CSSPropertyID::kOutlineColor as usize) < 65536);
    assert!((CSSPropertyID::kStroke as usize) < 65536);
    assert!((CSSPropertyID::kTextDecorationColor as usize) < 65536);
    assert!((CSSPropertyID::kTextEmphasisColor as usize) < 65536);
    assert!((CSSPropertyID::kWebkitTextFillColor as usize) < 65536);
    assert!((CSSPropertyID::kWebkitTextStrokeColor as usize) < 65536);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_instances_keep_aliases_out_of_resolved_objects() {
        assert_eq!(kCssProperties.len(), 826);
        assert_eq!(
            GetPropertyInternal(CSSPropertyID::kInvalid),
            &CSSPropertyUnion::Invalid
        );
        let mut resolved_count = 0;
        let mut alias_count = 0;
        for (index, instance) in kCssProperties.iter().enumerate() {
            match instance {
                CSSPropertyUnion::Invalid => assert_eq!(index, 0),
                CSSPropertyUnion::Resolved(property) => {
                    resolved_count += 1;
                    assert_eq!(property.PropertyID() as usize, index);
                    assert!(index <= 706);
                }
                CSSPropertyUnion::Alias(id) => {
                    alias_count += 1;
                    assert_eq!(*id as usize, index);
                    assert!(index >= 707);
                    assert!(instance.AsResolvedProperty().is_none());
                }
            }
        }
        assert_eq!((resolved_count, alias_count), (706, 119));
        let color = GetPropertyInternal(CSSPropertyID::kColor)
            .AsResolvedProperty()
            .unwrap();
        assert!(color.IsLonghand() && color.IsInherited() && color.IsInterpolable());
        let image = GetPropertyInternal(CSSPropertyID::kBackgroundImage)
            .AsResolvedProperty()
            .unwrap();
        assert_eq!(image.RepetitionSeparator(), ' ');
        let margin = GetPropertyInternal(CSSPropertyID::kMargin)
            .AsResolvedProperty()
            .unwrap();
        assert!(margin.IsShorthand() && !margin.IsLonghand());
    }

    #[test]
    fn visited_and_unvisited_tables_form_source_pairs() {
        for (index, &visited) in kPropertyVisitedIDs.iter().enumerate() {
            if visited == 0 {
                continue;
            }
            let canonical_index = match kCssProperties[index] {
                CSSPropertyUnion::Alias(id) => {
                    crate::css_property_names::ResolveCSSPropertyID(id) as usize
                }
                _ => index,
            };
            assert_eq!(
                kPropertyUnvisitedIDs[visited as usize] as usize,
                canonical_index
            );
            let visited_property = kCssProperties[visited as usize]
                .AsResolvedProperty()
                .unwrap();
            assert!(visited_property.IsVisited());
        }
        assert_eq!(
            kPropertyVisitedIDs[CSSPropertyID::kColor as usize],
            CSSPropertyID::kInternalVisitedColor as u8
        );
        assert_eq!(
            kPropertyUnvisitedIDs[CSSPropertyID::kInternalVisitedColor as usize],
            CSSPropertyID::kColor as u16
        );
        assert_eq!(
            kPropertyVisitedIDs[CSSPropertyID::kAliasWebkitAppearance as usize],
            0
        );
    }
}
