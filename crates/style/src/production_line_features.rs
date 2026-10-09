// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium generated longhand/shorthand Exposure, with real runtime defaults.
#![allow(non_snake_case)]
use foundation::{CSSPropertyID, RuntimeEnabledFeatures};

pub(crate) fn IsExposed(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    let line = RuntimeEnabledFeatures::CSSLineClampEnabled();
    let shorthand = RuntimeEnabledFeatures::CSSLineClampAsShorthandEnabled();
    // generated longhands.cc:2195-2204,10053-10063,11347-11352,
    // 18566-18576; generated shorthands.cc Alternative*::Exposure.
    match id {
        kLineClamp | kAlternativeWebkitLineClampLonghand => line && !shorthand,
        kAlternativeLineClampShorthand
        | kAlternativeWebkitLineClampShorthand
        | kMaxLines
        | kContinue
        | kBlockEllipsis => shorthand,
        kWebkitLineClamp => !line && !shorthand,
        _ => true,
    }
}
