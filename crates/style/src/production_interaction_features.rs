// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium stable runtime defaults for interaction/render-hint properties.
#![allow(non_snake_case)]
use foundation::CSSPropertyID;

pub(crate) fn IsExposed(id: CSSPropertyID) -> bool {
    // runtime_enabled_features.json5:1646-1648 (test),3977-3979
    // (experimental); generated longhands Exposure. No document feature owner
    // currently overrides these defaults in the production resolver.
    !matches!(
        id,
        CSSPropertyID::kHangingPunctuation | CSSPropertyID::kMarginTrim
    )
}
