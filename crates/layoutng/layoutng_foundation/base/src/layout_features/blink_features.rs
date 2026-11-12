//! Process-wide Blink base-feature state used below the browser embedding.

#![allow(non_snake_case)]

use std::sync::atomic::{AtomicBool, Ordering};

// Source: third_party/blink/common/features.cc:58. Chromium declares this
// Android-only feature disabled by default and lets FeatureList override it at
// process startup. Keep the same default while exposing the corresponding
// embedding hook rather than compiling either paint branch away.
static ANDROID_SPELLCHECK_NATIVE_UI_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn AndroidSpellcheckNativeUiEnabled() -> bool {
    ANDROID_SPELLCHECK_NATIVE_UI_ENABLED.load(Ordering::Relaxed)
}

pub fn SetAndroidSpellcheckNativeUiEnabled(enabled: bool) {
    ANDROID_SPELLCHECK_NATIVE_UI_ENABLED.store(enabled, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_spellcheck_native_ui_retains_disabled_default_and_override() {
        SetAndroidSpellcheckNativeUiEnabled(false);
        assert!(!AndroidSpellcheckNativeUiEnabled());
        SetAndroidSpellcheckNativeUiEnabled(true);
        assert!(AndroidSpellcheckNativeUiEnabled());
        SetAndroidSpellcheckNativeUiEnabled(false);
    }
}

// Chromium common/features.cc:151-152,2304. FeatureList ownership stays here;
// HTML/default-action adapters only read the canonical process feature values.
static AUTOFILL_KEYDOWN_EDITABLE_ELEMENT: AtomicBool = AtomicBool::new(false);
static DEVTOOLS_ALLOW_INTEREST_FORCING: AtomicBool = AtomicBool::new(true);
pub fn AutofillKeydownEditableElementEnabled() -> bool {
    AUTOFILL_KEYDOWN_EDITABLE_ELEMENT.load(Ordering::Relaxed)
}
pub fn SetAutofillKeydownEditableElementEnabled(enabled: bool) {
    AUTOFILL_KEYDOWN_EDITABLE_ELEMENT.store(enabled, Ordering::Relaxed);
}
pub fn DevToolsAllowInterestForcingEnabled() -> bool {
    DEVTOOLS_ALLOW_INTEREST_FORCING.load(Ordering::Relaxed)
}
pub fn SetDevToolsAllowInterestForcingEnabled(enabled: bool) {
    DEVTOOLS_ALLOW_INTEREST_FORCING.store(enabled, Ordering::Relaxed);
}

// common/features.cc:345-351, Android-only enabled initial value.
static CONTENT_SCHEME_IS_LOCAL: AtomicBool = AtomicBool::new(cfg!(target_os = "android"));
pub fn ContentSchemeIsLocalEnabled() -> bool {
    CONTENT_SCHEME_IS_LOCAL.load(Ordering::Relaxed)
}
pub fn SetContentSchemeIsLocalEnabled(enabled: bool) {
    CONTENT_SCHEME_IS_LOCAL.store(enabled, Ordering::Relaxed);
}

// html_media_element.cc:170, enabled source base::Feature.
static VIDEO_SOURCE_MEDIA_QUERY_SUPPORT: AtomicBool = AtomicBool::new(true);
pub fn VideoSourceMediaQuerySupportEnabled() -> bool {
    VIDEO_SOURCE_MEDIA_QUERY_SUPPORT.load(Ordering::Relaxed)
}
pub fn SetVideoSourceMediaQuerySupportEnabled(enabled: bool) {
    VIDEO_SOURCE_MEDIA_QUERY_SUPPORT.store(enabled, Ordering::Relaxed);
}

// features.cc1934-1946: renderer startup FeatureList state, enabled by default.
static RENDER_BLOCKING_FONTS_ENABLED: AtomicBool = AtomicBool::new(true);
pub fn RenderBlockingFontsEnabled() -> bool {
    RENDER_BLOCKING_FONTS_ENABLED.load(Ordering::Relaxed)
}
pub fn SetRenderBlockingFontsEnabledForEmbedding(enabled: bool) {
    RENDER_BLOCKING_FONTS_ENABLED.store(enabled, Ordering::Relaxed);
}

// Chromium common/features.cc884; ResourceLoader.cc1367-1376.
static KEEP_ALIVE_IN_BROWSER_MIGRATION: AtomicBool = AtomicBool::new(true);
pub fn KeepAliveInBrowserMigrationEnabled() -> bool {
    KEEP_ALIVE_IN_BROWSER_MIGRATION.load(Ordering::Relaxed)
}
pub fn SetKeepAliveInBrowserMigrationForEmbedding(enabled: bool) {
    KEEP_ALIVE_IN_BROWSER_MIGRATION.store(enabled, Ordering::Relaxed);
}
