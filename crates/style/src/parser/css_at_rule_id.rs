// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_at_rule_id.h
// cpp: third_party/blink/renderer/core/css/parser/css_at_rule_id.cc

#![allow(non_camel_case_types, non_snake_case)]

use foundation::StringView;
use std::cmp::Ordering;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSAtRuleID {
    kCSSAtRuleInvalid,
    kCSSAtRuleViewTransition,
    kCSSAtRuleCharset,
    kCSSAtRuleFontFace,
    kCSSAtRuleFontPaletteValues,
    kCSSAtRuleImport,
    kCSSAtRuleKeyframes,
    kCSSAtRuleLayer,
    kCSSAtRuleMedia,
    kCSSAtRuleNamespace,
    kCSSAtRulePage,
    kCSSAtRulePositionTry,
    kCSSAtRuleProperty,
    kCSSAtRuleLocation,
    kCSSAtRuleNavigation,
    kCSSAtRuleContainer,
    kCSSAtRuleCounterStyle,
    kCSSAtRuleScope,
    kCSSAtRuleStartingStyle,
    kCSSAtRuleSupports,
    kCSSAtRuleWebkitKeyframes,
    kCSSAtRuleAnnotation,
    kCSSAtRuleCharacterVariant,
    kCSSAtRuleFontFeatureValues,
    kCSSAtRuleOrnaments,
    kCSSAtRuleStylistic,
    kCSSAtRuleStyleset,
    kCSSAtRuleSwash,
    kCSSAtRuleTopLeftCorner,
    kCSSAtRuleTopLeft,
    kCSSAtRuleTopCenter,
    kCSSAtRuleTopRight,
    kCSSAtRuleTopRightCorner,
    kCSSAtRuleBottomLeftCorner,
    kCSSAtRuleBottomLeft,
    kCSSAtRuleBottomCenter,
    kCSSAtRuleBottomRight,
    kCSSAtRuleBottomRightCorner,
    kCSSAtRuleLeftTop,
    kCSSAtRuleLeftMiddle,
    kCSSAtRuleLeftBottom,
    kCSSAtRuleRightTop,
    kCSSAtRuleRightMiddle,
    kCSSAtRuleRightBottom,
    kCSSAtRuleFunction,
    kCSSAtRuleMixin,
    kCSSAtRuleApplyMixin,
    kCSSAtRuleContents,
    kCSSAtRuleResult,
    kCSSAtRulePrivate,
    kCSSAtRuleCustomMedia,
    kCount,
}

// The source stores WebFeature in each table row. Keeping the feature beside
// the ID preserves CountAtRule without coupling style to a concrete telemetry
// implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtRuleFeature {
    CSSAtRuleWebkitKeyframes,
    CSSAtRuleAnnotation,
    CSSAtRulePageMargin,
    CSSAtRuleCharacterVariant,
    CSSAtRuleCharset,
    CSSAtRuleContainer,
    CSSAtRuleCounterStyle,
    CSSAtRuleFontFace,
    CSSAtRuleFontFeatureValues,
    CSSAtRuleFontPaletteValues,
    CSSAtRuleImport,
    CSSAtRuleKeyframes,
    CSSCascadeLayers,
    CSSAtRuleMedia,
    CSSAtRuleNamespace,
    CSSAtRuleOrnaments,
    CSSAtRulePage,
    CSSAnchorPositioning,
    CSSAtRuleProperty,
    CSSAtRuleScope,
    CSSAtRuleStartingStyle,
    CSSAtRuleStylistic,
    CSSAtRuleSupports,
    CSSAtRuleSwash,
    CSSAtRuleViewTransition,
    CSSMixins,
    CSSCustomMedia,
    CSSFunctions,
    CSSAtRuleRoute,
    CSSAtPrivate,
}

#[derive(Clone, Copy)]
struct AtRuleEntry {
    name: &'static str,
    id: CSSAtRuleID,
    feature: AtRuleFeature,
}

// cpp: css_at_rule_id.cc:32-75. Kept sorted for binary search.
const AT_RULE_ENTRIES: [AtRuleEntry; 41] = [
    AtRuleEntry {
        name: "-webkit-keyframes",
        id: CSSAtRuleID::kCSSAtRuleWebkitKeyframes,
        feature: AtRuleFeature::CSSAtRuleWebkitKeyframes,
    },
    AtRuleEntry {
        name: "annotation",
        id: CSSAtRuleID::kCSSAtRuleAnnotation,
        feature: AtRuleFeature::CSSAtRuleAnnotation,
    },
    AtRuleEntry {
        name: "bottom-center",
        id: CSSAtRuleID::kCSSAtRuleBottomCenter,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "bottom-left",
        id: CSSAtRuleID::kCSSAtRuleBottomLeft,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "bottom-left-corner",
        id: CSSAtRuleID::kCSSAtRuleBottomLeftCorner,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "bottom-right",
        id: CSSAtRuleID::kCSSAtRuleBottomRight,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "bottom-right-corner",
        id: CSSAtRuleID::kCSSAtRuleBottomRightCorner,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "character-variant",
        id: CSSAtRuleID::kCSSAtRuleCharacterVariant,
        feature: AtRuleFeature::CSSAtRuleCharacterVariant,
    },
    AtRuleEntry {
        name: "charset",
        id: CSSAtRuleID::kCSSAtRuleCharset,
        feature: AtRuleFeature::CSSAtRuleCharset,
    },
    AtRuleEntry {
        name: "container",
        id: CSSAtRuleID::kCSSAtRuleContainer,
        feature: AtRuleFeature::CSSAtRuleContainer,
    },
    AtRuleEntry {
        name: "counter-style",
        id: CSSAtRuleID::kCSSAtRuleCounterStyle,
        feature: AtRuleFeature::CSSAtRuleCounterStyle,
    },
    AtRuleEntry {
        name: "font-face",
        id: CSSAtRuleID::kCSSAtRuleFontFace,
        feature: AtRuleFeature::CSSAtRuleFontFace,
    },
    AtRuleEntry {
        name: "font-feature-values",
        id: CSSAtRuleID::kCSSAtRuleFontFeatureValues,
        feature: AtRuleFeature::CSSAtRuleFontFeatureValues,
    },
    AtRuleEntry {
        name: "font-palette-values",
        id: CSSAtRuleID::kCSSAtRuleFontPaletteValues,
        feature: AtRuleFeature::CSSAtRuleFontPaletteValues,
    },
    AtRuleEntry {
        name: "import",
        id: CSSAtRuleID::kCSSAtRuleImport,
        feature: AtRuleFeature::CSSAtRuleImport,
    },
    AtRuleEntry {
        name: "keyframes",
        id: CSSAtRuleID::kCSSAtRuleKeyframes,
        feature: AtRuleFeature::CSSAtRuleKeyframes,
    },
    AtRuleEntry {
        name: "layer",
        id: CSSAtRuleID::kCSSAtRuleLayer,
        feature: AtRuleFeature::CSSCascadeLayers,
    },
    AtRuleEntry {
        name: "left-bottom",
        id: CSSAtRuleID::kCSSAtRuleLeftBottom,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "left-middle",
        id: CSSAtRuleID::kCSSAtRuleLeftMiddle,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "left-top",
        id: CSSAtRuleID::kCSSAtRuleLeftTop,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "media",
        id: CSSAtRuleID::kCSSAtRuleMedia,
        feature: AtRuleFeature::CSSAtRuleMedia,
    },
    AtRuleEntry {
        name: "namespace",
        id: CSSAtRuleID::kCSSAtRuleNamespace,
        feature: AtRuleFeature::CSSAtRuleNamespace,
    },
    AtRuleEntry {
        name: "ornaments",
        id: CSSAtRuleID::kCSSAtRuleOrnaments,
        feature: AtRuleFeature::CSSAtRuleOrnaments,
    },
    AtRuleEntry {
        name: "page",
        id: CSSAtRuleID::kCSSAtRulePage,
        feature: AtRuleFeature::CSSAtRulePage,
    },
    AtRuleEntry {
        name: "position-try",
        id: CSSAtRuleID::kCSSAtRulePositionTry,
        feature: AtRuleFeature::CSSAnchorPositioning,
    },
    AtRuleEntry {
        name: "property",
        id: CSSAtRuleID::kCSSAtRuleProperty,
        feature: AtRuleFeature::CSSAtRuleProperty,
    },
    AtRuleEntry {
        name: "right-bottom",
        id: CSSAtRuleID::kCSSAtRuleRightBottom,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "right-middle",
        id: CSSAtRuleID::kCSSAtRuleRightMiddle,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "right-top",
        id: CSSAtRuleID::kCSSAtRuleRightTop,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "scope",
        id: CSSAtRuleID::kCSSAtRuleScope,
        feature: AtRuleFeature::CSSAtRuleScope,
    },
    AtRuleEntry {
        name: "starting-style",
        id: CSSAtRuleID::kCSSAtRuleStartingStyle,
        feature: AtRuleFeature::CSSAtRuleStartingStyle,
    },
    AtRuleEntry {
        name: "styleset",
        id: CSSAtRuleID::kCSSAtRuleStyleset,
        feature: AtRuleFeature::CSSAtRuleStylistic,
    },
    AtRuleEntry {
        name: "stylistic",
        id: CSSAtRuleID::kCSSAtRuleStylistic,
        feature: AtRuleFeature::CSSAtRuleStylistic,
    },
    AtRuleEntry {
        name: "supports",
        id: CSSAtRuleID::kCSSAtRuleSupports,
        feature: AtRuleFeature::CSSAtRuleSupports,
    },
    AtRuleEntry {
        name: "swash",
        id: CSSAtRuleID::kCSSAtRuleSwash,
        feature: AtRuleFeature::CSSAtRuleSwash,
    },
    AtRuleEntry {
        name: "top-center",
        id: CSSAtRuleID::kCSSAtRuleTopCenter,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "top-left",
        id: CSSAtRuleID::kCSSAtRuleTopLeft,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "top-left-corner",
        id: CSSAtRuleID::kCSSAtRuleTopLeftCorner,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "top-right",
        id: CSSAtRuleID::kCSSAtRuleTopRight,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "top-right-corner",
        id: CSSAtRuleID::kCSSAtRuleTopRightCorner,
        feature: AtRuleFeature::CSSAtRulePageMargin,
    },
    AtRuleEntry {
        name: "view-transition",
        id: CSSAtRuleID::kCSSAtRuleViewTransition,
        feature: AtRuleFeature::CSSAtRuleViewTransition,
    },
];

#[derive(Clone, Copy)]
enum RuntimeGate {
    CSSMixins,
    CSSCustomMedia,
    CSSFunctions,
    RouteMatching,
    CSSPrivate,
}

#[derive(Clone, Copy)]
struct FlaggedAtRuleEntry {
    entry: AtRuleEntry,
    gate: RuntimeGate,
}

// Explicit equivalent of the five RuntimeEnabledFeatures predicates used by
// css_at_rule_id.cc:92-109. The application can snapshot those settings and
// pass them without making the style crate depend on its runtime host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtRuleRuntimeFeatures {
    pub css_mixins: bool,
    pub css_custom_media: bool,
    pub css_functions: bool,
    pub route_matching: bool,
    pub css_private: bool,
}

impl AtRuleRuntimeFeatures {
    const fn enabled(self, gate: RuntimeGate) -> bool {
        match gate {
            RuntimeGate::CSSMixins => self.css_mixins,
            RuntimeGate::CSSCustomMedia => self.css_custom_media,
            RuntimeGate::CSSFunctions => self.css_functions,
            RuntimeGate::RouteMatching => self.route_matching,
            RuntimeGate::CSSPrivate => self.css_private,
        }
    }
}

const FLAGGED_AT_RULE_ENTRIES: [FlaggedAtRuleEntry; 9] = [
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "apply",
            id: CSSAtRuleID::kCSSAtRuleApplyMixin,
            feature: AtRuleFeature::CSSMixins,
        },
        gate: RuntimeGate::CSSMixins,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "contents",
            id: CSSAtRuleID::kCSSAtRuleContents,
            feature: AtRuleFeature::CSSMixins,
        },
        gate: RuntimeGate::CSSMixins,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "custom-media",
            id: CSSAtRuleID::kCSSAtRuleCustomMedia,
            feature: AtRuleFeature::CSSCustomMedia,
        },
        gate: RuntimeGate::CSSCustomMedia,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "function",
            id: CSSAtRuleID::kCSSAtRuleFunction,
            feature: AtRuleFeature::CSSFunctions,
        },
        gate: RuntimeGate::CSSFunctions,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "location",
            id: CSSAtRuleID::kCSSAtRuleLocation,
            feature: AtRuleFeature::CSSAtRuleRoute,
        },
        gate: RuntimeGate::RouteMatching,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "mixin",
            id: CSSAtRuleID::kCSSAtRuleMixin,
            feature: AtRuleFeature::CSSMixins,
        },
        gate: RuntimeGate::CSSMixins,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "navigation",
            id: CSSAtRuleID::kCSSAtRuleNavigation,
            feature: AtRuleFeature::CSSAtRuleRoute,
        },
        gate: RuntimeGate::RouteMatching,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "private",
            id: CSSAtRuleID::kCSSAtRulePrivate,
            feature: AtRuleFeature::CSSAtPrivate,
        },
        gate: RuntimeGate::CSSPrivate,
    },
    FlaggedAtRuleEntry {
        entry: AtRuleEntry {
            name: "result",
            id: CSSAtRuleID::kCSSAtRuleResult,
            feature: AtRuleFeature::CSSMixins,
        },
        gate: RuntimeGate::CSSMixins,
    },
];

fn CompareIgnoringAsciiCase(name: &str, target: &StringView) -> Ordering {
    let name = name.as_bytes();
    let target = target.Span16();
    for index in 0..name.len().min(target.len()) {
        let left = name[index].to_ascii_lowercase() as u16;
        let unit = target[index];
        let right = if unit <= 0x7f {
            (unit as u8).to_ascii_lowercase() as u16
        } else {
            unit
        };
        match left.cmp(&right) {
            Ordering::Equal => {}
            order => return order,
        }
    }
    name.len().cmp(&target.len())
}

// cpp: css_at_rule_id.cc:128-149
pub fn CssAtRuleID(name: &StringView, features: AtRuleRuntimeFeatures) -> CSSAtRuleID {
    if let Ok(index) =
        AT_RULE_ENTRIES.binary_search_by(|entry| CompareIgnoringAsciiCase(entry.name, name))
    {
        return AT_RULE_ENTRIES[index].id;
    }
    for entry in FLAGGED_AT_RULE_ENTRIES {
        if features.enabled(entry.gate)
            && CompareIgnoringAsciiCase(entry.entry.name, name) == Ordering::Equal
        {
            return entry.entry.id;
        }
    }
    CSSAtRuleID::kCSSAtRuleInvalid
}

// cpp: css_at_rule_id.cc:151-161
pub fn CssAtRuleIDToString(id: CSSAtRuleID) -> Option<&'static str> {
    AT_RULE_ENTRIES
        .iter()
        .map(|entry| entry.entry_view())
        .chain(FLAGGED_AT_RULE_ENTRIES.iter().map(|entry| &entry.entry))
        .find(|entry| entry.id == id)
        .map(|entry| entry.name)
}

impl AtRuleEntry {
    const fn entry_view(&self) -> &Self {
        self
    }
}

// cpp: css_at_rule_id.cc:165-176
pub fn AtRuleFeatureForID(id: CSSAtRuleID) -> Option<AtRuleFeature> {
    AT_RULE_ENTRIES
        .iter()
        .chain(FLAGGED_AT_RULE_ENTRIES.iter().map(|entry| &entry.entry))
        .find(|entry| entry.id == id)
        .map(|entry| entry.feature)
}

pub trait AtRuleCounter {
    fn Count(&mut self, feature: AtRuleFeature);
}

// The C++ CSSParserContext call is represented by its one-method boundary.
pub fn CountAtRule(counter: &mut impl AtRuleCounter, id: CSSAtRuleID) {
    if let Some(feature) = AtRuleFeatureForID(id) {
        counter.Count(feature);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_table_is_case_insensitive_and_round_trips() {
        let features = AtRuleRuntimeFeatures::default();
        for entry in AT_RULE_ENTRIES {
            assert_eq!(
                CssAtRuleID(&StringView::from(entry.name), features),
                entry.id
            );
            assert_eq!(CssAtRuleIDToString(entry.id), Some(entry.name));
        }
        assert_eq!(
            CssAtRuleID(&StringView::from("FoNt-FaCe"), features),
            CSSAtRuleID::kCSSAtRuleFontFace
        );
        assert_eq!(
            CssAtRuleID(&StringView::from("unknown"), features),
            CSSAtRuleID::kCSSAtRuleInvalid
        );
        assert_eq!(
            CssAtRuleID(&StringView::from("média"), features),
            CSSAtRuleID::kCSSAtRuleInvalid
        );
    }

    #[test]
    fn flagged_rules_follow_their_exact_runtime_gate() {
        let disabled = AtRuleRuntimeFeatures::default();
        for entry in FLAGGED_AT_RULE_ENTRIES {
            assert_eq!(
                CssAtRuleID(&StringView::from(entry.entry.name), disabled),
                CSSAtRuleID::kCSSAtRuleInvalid
            );
        }
        let enabled = AtRuleRuntimeFeatures {
            css_mixins: true,
            css_custom_media: true,
            css_functions: true,
            route_matching: true,
            css_private: true,
        };
        for entry in FLAGGED_AT_RULE_ENTRIES {
            assert_eq!(
                CssAtRuleID(&StringView::from(entry.entry.name), enabled),
                entry.entry.id
            );
            assert_eq!(CssAtRuleIDToString(entry.entry.id), Some(entry.entry.name));
        }
    }

    #[test]
    fn count_routes_the_table_feature() {
        #[derive(Default)]
        struct Counter(Vec<AtRuleFeature>);
        impl AtRuleCounter for Counter {
            fn Count(&mut self, feature: AtRuleFeature) {
                self.0.push(feature);
            }
        }
        let mut counter = Counter::default();
        CountAtRule(&mut counter, CSSAtRuleID::kCSSAtRuleTopLeft);
        CountAtRule(&mut counter, CSSAtRuleID::kCSSAtRuleInvalid);
        assert_eq!(counter.0, vec![AtRuleFeature::CSSAtRulePageMargin]);
    }
}
