// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Stable generated property Exposure for typography.
#![allow(non_snake_case)]
pub fn IsExposed(id: foundation::CSSPropertyID) -> bool {
    // runtime_enabled_features.json5:2029-2032: experimental; no host override.
    id != foundation::CSSPropertyID::kTextDecorationSkipSpaces
        && (id != foundation::CSSPropertyID::kFontSizeAdjust || CSSFontSizeAdjustEnabled())
        && (id != foundation::CSSPropertyID::kFontLanguageOverride || FontLanguageOverrideEnabled())
}

// runtime_enabled_features.json5:1631-1634,3216-3219, stable defaults.
use std::sync::atomic::{AtomicBool, Ordering};
static FONT_SIZE_ADJUST: AtomicBool = AtomicBool::new(true);
static FONT_LANGUAGE_OVERRIDE: AtomicBool = AtomicBool::new(true);
pub fn CSSFontSizeAdjustEnabled() -> bool {
    FONT_SIZE_ADJUST.load(Ordering::Relaxed)
}
pub fn SetCSSFontSizeAdjustEnabled(value: bool) {
    FONT_SIZE_ADJUST.store(value, Ordering::Relaxed);
}
pub fn FontLanguageOverrideEnabled() -> bool {
    FONT_LANGUAGE_OVERRIDE.load(Ordering::Relaxed)
}
pub fn SetFontLanguageOverrideEnabled(value: bool) {
    FONT_LANGUAGE_OVERRIDE.store(value, Ordering::Relaxed);
}
