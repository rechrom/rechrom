// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium stable Exposure for text-box/spacing properties.
#![allow(non_snake_case)]
pub(crate) fn IsExposed(id: foundation::CSSPropertyID) -> bool {
    use foundation::CSSPropertyID::*;
    // runtime_enabled_features.json5:2020-2023 experimental,2048-2051 test.
    // These flags have no production ExecutionContext/host override adapter.
    !matches!(id, kTextDecorationInset | kTextSpacing)
        && (id != kTextFit || foundation::RuntimeEnabledFeatures::CssTextFitEnabled())
}
