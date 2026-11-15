//! FontCache's Windows system-font statics, translated from
//! third_party/blink/renderer/platform/fonts/font_cache.h:183-200 and
//! win/font_cache_skia_win.cc:76-155 (Chromium 6c1d401f).
//!
//! As in Chromium, these are process-wide values written by the embedding from
//! its renderer preferences (WebViewImpl::SetRendererPreferences calls
//! WebFontRendering::Set*FontMetrics) and read by the Windows
//! LayoutThemeFontProvider and FontCache::SystemFontFamily.

use std::sync::RwLock;

#[derive(Clone, Debug, Default, PartialEq)]
struct FontMetrics {
    family_name: String,
    height: i32,
}

#[derive(Default)]
struct SystemFontMetrics {
    menu: FontMetrics,
    small_caption: FontMetrics,
    status: FontMetrics,
}

static METRICS: RwLock<SystemFontMetrics> = RwLock::new(SystemFontMetrics {
    menu: FontMetrics {
        family_name: String::new(),
        height: 0,
    },
    small_caption: FontMetrics {
        family_name: String::new(),
        height: 0,
    },
    status: FontMetrics {
        family_name: String::new(),
        height: 0,
    },
});

/// The process ANSI code page (Win32 GetACP), supplied by the embedding before
/// it sets the system font metrics: this package performs no platform query.
static ANSI_CODE_PAGE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn SetAnsiCodePage(code_page: u32) {
    ANSI_CODE_PAGE.store(code_page, std::sync::atomic::Ordering::Relaxed);
}

// Source: win/font_cache_skia_win.cc:76-81
fn EnsureMinimumFontHeightIfNeeded(font_height: i32) -> i32 {
    // Adjustment for codepage 936 to make the fonts more legible in Simplified
    // Chinese.  Please refer to LayoutThemeFontProviderWin.cpp for more
    // information.
    let code_page = ANSI_CODE_PAGE.load(std::sync::atomic::Ordering::Relaxed);
    if (font_height as f32) < 12.0 && code_page == 936 {
        12
    } else {
        font_height
    }
}

fn Set(slot: fn(&mut SystemFontMetrics) -> &mut FontMetrics, family_name: &str, font_height: i32) {
    let mut metrics = METRICS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *slot(&mut metrics) = FontMetrics {
        family_name: family_name.to_owned(),
        height: EnsureMinimumFontHeightIfNeeded(font_height),
    };
}

fn Get<T>(read: impl FnOnce(&SystemFontMetrics) -> T) -> T {
    read(
        &METRICS
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    )
}

// Source: win/font_cache_skia_win.cc:134-139
pub fn SetMenuFontMetrics(family_name: &str, font_height: i32) {
    Set(|metrics| &mut metrics.menu, family_name, font_height);
}

// Source: win/font_cache_skia_win.cc:141-146
pub fn SetSmallCaptionFontMetrics(family_name: &str, font_height: i32) {
    Set(
        |metrics| &mut metrics.small_caption,
        family_name,
        font_height,
    );
}

// Source: win/font_cache_skia_win.cc:148-153
pub fn SetStatusFontMetrics(family_name: &str, font_height: i32) {
    Set(|metrics| &mut metrics.status, family_name, font_height);
}

// Source: font_cache.h:189-200
pub fn MenuFontHeight() -> i32 {
    Get(|metrics| metrics.menu.height)
}
pub fn MenuFontFamily() -> String {
    Get(|metrics| metrics.menu.family_name.clone())
}
pub fn SmallCaptionFontHeight() -> i32 {
    Get(|metrics| metrics.small_caption.height)
}
pub fn SmallCaptionFontFamily() -> String {
    Get(|metrics| metrics.small_caption.family_name.clone())
}
pub fn StatusFontHeight() -> i32 {
    Get(|metrics| metrics.status.height)
}
pub fn StatusFontFamily() -> String {
    Get(|metrics| metrics.status.family_name.clone())
}

// Source: win/font_cache_skia_win.cc:129-132
pub fn SystemFontFamily() -> String {
    MenuFontFamily()
}

// Source: font_cache.h:176-181 and :315-316. Written by the embedding from
// its renderer preferences (WebViewImpl::SetRendererPreferences calls
// WebFontRendering::SetAntialiasedTextEnabled/SetLCDTextEnabled) and read by
// FontPlatformData::QuerySystemForRenderStyle.
static ANTIALIASED_TEXT_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static LCD_TEXT_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn AntialiasedTextEnabled() -> bool {
    ANTIALIASED_TEXT_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn LcdTextEnabled() -> bool {
    LCD_TEXT_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn SetAntialiasedTextEnabled(enabled: bool) {
    ANTIALIASED_TEXT_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}
pub fn SetLCDTextEnabled(enabled: bool) {
    LCD_TEXT_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}
