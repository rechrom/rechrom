// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Generated CSSProperty::Exposure branches used by CSSComputedStyleDeclaration.
//! Pinned Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Source css_properties.cc.tmpl:39-66; css_properties.py:160-165.
//! Existing mutable runtime feature owners retain precedence over metadata defaults.
use crate::{
    css_property_names::{kNotKnownExposedPropertyBit, FindProperty, GetPropertyName},
    properties::css_property::CSSProperty,
};
use foundation::CSSPropertyID as P;
pub fn IsComputablePropertyWebExposed(id: P) -> Option<bool> {
    if CSSProperty::Get(id).IsInternal() {
        return Some(false);
    }
    use P::*;
    match id {
        kLineClamp
        | kAlternativeWebkitLineClampLonghand
        | kWebkitLineClamp
        | kContinue
        | kMaxLines
        | kBlockEllipsis
        | kAlternativeLineClampShorthand
        | kAlternativeWebkitLineClampShorthand => {
            return Some(crate::production_line_features::IsExposed(id))
        }
        kFontSizeAdjust | kFontLanguageOverride => {
            return Some(crate::production_typography_features::IsExposed(id))
        }
        kTextDecorationInset | kTextFit => {
            return Some(crate::production_text_box_features::IsExposed(id))
        }
        // css_properties.json5:1155; runtime_enabled_features.json5:628 (stable).
        kAnimationTrigger => return Some(true),
        // css_properties.json5:2872; runtime_enabled_features.json5:1520 (stable).
        kBorderShape => return Some(true),
        // css_properties.json5:3018; runtime_enabled_features.json5:1532 (stable).
        kCaretAnimation => return Some(true),
        // css_properties.json5:3046; runtime_enabled_features.json5:1543 (stable).
        kCaretShape => return Some(true),
        // css_properties.json5:4081; runtime_enabled_features.json5:1615 (stable).
        kDynamicRangeLimit => return Some(true),
        // css_properties.json5:3654; runtime_enabled_features.json5:3166 (stable).
        kFlexLineCount => return Some(true),
        // css_properties.json5:3720; runtime_enabled_features.json5:1641 (experimental).
        kFlowTolerance => return Some(false),
        // css_properties.json5:1870; runtime_enabled_features.json5:3238 (stable).
        kForcedColorAdjust => return Some(true),
        // css_properties.json5:3738; runtime_enabled_features.json5:5163 (experimental).
        kFrameSizing => return Some(false),
        // css_properties.json5:3823; runtime_enabled_features.json5:1641 (experimental).
        kGridLanesDirection => return Some(false),
        // css_properties.json5:3839; runtime_enabled_features.json5:1641 (experimental).
        kGridLanesPack => return Some(false),
        // css_properties.json5:3914; runtime_enabled_features.json5:1646 (test).
        kHangingPunctuation => return Some(false),
        // css_properties.json5:4036; runtime_enabled_features.json5:1663 (test).
        kImageAnimation => return Some(false),
        // css_properties.json5:4430; runtime_enabled_features.json5:3977 (experimental).
        kMarginTrim => return Some(false),
        // css_properties.json5:4522; runtime_enabled_features.json5:1753 (test).
        kMaxContentSizing => return Some(false),
        // css_properties.json5:6657; runtime_enabled_features.json5:4583 (stable).
        kOverlay => return Some(true),
        // css_properties.json5:5008; runtime_enabled_features.json5:4588 (experimental).
        kOverscrollContainerType => return Some(false),
        // css_properties.json5:5243; runtime_enabled_features.json5:6074 (experimental).
        kPathLength => return Some(false),
        // css_properties.json5:7899; runtime_enabled_features.json5:1892 (stable).
        kRubyOverhang => return Some(true),
        // css_properties.json5:5525; runtime_enabled_features.json5:5358 (stable).
        kScrollAxisLock => return Some(true),
        // css_properties.json5:5619; runtime_enabled_features.json5:1918 (stable).
        kScrollInitialTarget => return Some(true),
        // css_properties.json5:5546; runtime_enabled_features.json5:1858 (stable).
        kScrollMarkerGroup => return Some(true),
        // css_properties.json5:5536; runtime_enabled_features.json5:1960 (stable).
        kScrollTargetGroup => return Some(true),
        // css_properties.json5:5562; runtime_enabled_features.json5:5362 (stable).
        kScrollbarColor => return Some(true),
        // css_properties.json5:5595; runtime_enabled_features.json5:5370 (stable).
        kScrollbarWidth => return Some(true),
        // css_properties.json5:6386; runtime_enabled_features.json5:2030 (experimental).
        kTextDecorationSkipSpaces => return Some(false),
        // css_properties.json5:1196; runtime_enabled_features.json5:6347 (stable).
        kTimelineTriggerActivationRangeEnd => return Some(true),
        // css_properties.json5:1182; runtime_enabled_features.json5:6347 (stable).
        kTimelineTriggerActivationRangeStart => return Some(true),
        // css_properties.json5:1225; runtime_enabled_features.json5:6347 (stable).
        kTimelineTriggerActiveRangeEnd => return Some(true),
        // css_properties.json5:1210; runtime_enabled_features.json5:6347 (stable).
        kTimelineTriggerActiveRangeStart => return Some(true),
        // css_properties.json5:1168; runtime_enabled_features.json5:6347 (stable).
        kTimelineTriggerName => return Some(true),
        // css_properties.json5:1240; runtime_enabled_features.json5:6347 (stable).
        kTimelineTriggerSource => return Some(true),
        // css_properties.json5:1342; runtime_enabled_features.json5:628 (stable).
        kTriggerScope => return Some(true),
        // css_properties.json5:7001; runtime_enabled_features.json5:2122 (stable).
        kWindowDrag => return Some(true),
        _ => {}
    }
    let entry = FindProperty(GetPropertyName(id).as_bytes())?;
    (entry.id_and_exposed_bit & kNotKnownExposedPropertyBit == 0).then_some(true)
}
