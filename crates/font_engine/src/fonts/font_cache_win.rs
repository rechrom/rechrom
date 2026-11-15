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

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetACP() -> u32;
}

// Source: win/font_cache_skia_win.cc:76-81
fn EnsureMinimumFontHeightIfNeeded(font_height: i32) -> i32 {
    // Adjustment for codepage 936 to make the fonts more legible in Simplified
    // Chinese.  Please refer to LayoutThemeFontProviderWin.cpp for more
    // information.
    #[cfg(target_os = "windows")]
    let code_page = unsafe { GetACP() };
    #[cfg(not(target_os = "windows"))]
    let code_page = 0;
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
