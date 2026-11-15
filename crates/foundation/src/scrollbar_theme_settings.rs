//! Chromium scrollbar_theme_settings.cc:9-46 (6c1d401fcca5e1b0030563a90c2f2bba168e0c15).
//! Canonical process theme configuration, initialized by the embedding before
//! native theme acquisition. These are source globals, not per-Page defaults.
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
static MOCK: AtomicBool = AtomicBool::new(false);
static OVERLAY: AtomicBool = AtomicBool::new(false);
static FLUENT: AtomicBool = AtomicBool::new(false);
static DESKTOP_ANDROID: AtomicBool = AtomicBool::new(false);
// ScrollbarThemeMac source globals, scrollbar_theme_mac.cc:47-48. The host
// republishes only preferences actually present in NSUserDefaults.
static MAC_INITIAL_BUTTON_DELAY: AtomicU32 = AtomicU32::new(0.5_f32.to_bits());
static MAC_AUTOSCROLL_BUTTON_DELAY: AtomicU32 = AtomicU32::new(0.05_f32.to_bits());
static MAC_JUMP_ON_TRACK_CLICK: AtomicBool = AtomicBool::new(false);
pub struct ScrollbarThemeSettings;
impl ScrollbarThemeSettings {
    pub fn SetMacJumpOnTrackClick(value: bool) {
        MAC_JUMP_ON_TRACK_CLICK.store(value, Ordering::Relaxed);
    }
    pub fn MacJumpOnTrackClick() -> bool {
        MAC_JUMP_ON_TRACK_CLICK.load(Ordering::Relaxed)
    }
    /// scrollbar_theme_mac.cc:384-391, UpdateScrollbarsWithNSDefaults.
    /// An absent preference retains the previous source value.
    pub fn UpdateMacAutoscrollButtonDelays(initial: Option<f32>, repeat: Option<f32>) {
        if let Some(value) = initial {
            MAC_INITIAL_BUTTON_DELAY.store(value.to_bits(), Ordering::Relaxed);
        }
        if let Some(value) = repeat {
            MAC_AUTOSCROLL_BUTTON_DELAY.store(value.to_bits(), Ordering::Relaxed);
        }
    }
    fn ButtonDelay(value: f32) -> std::time::Duration {
        // base::Seconds stores a signed, saturating microsecond TimeDelta;
        // a nonpositive timer delay is due immediately on the task runner.
        std::time::Duration::from_micros(
            (f64::from(value) * 1_000_000.0).clamp(0.0, i64::MAX as f64) as u64,
        )
    }
    pub fn MacInitialAutoscrollTimerDelay() -> std::time::Duration {
        Self::ButtonDelay(f32::from_bits(
            MAC_INITIAL_BUTTON_DELAY.load(Ordering::Relaxed),
        ))
    }
    pub fn MacAutoscrollTimerDelay() -> std::time::Duration {
        Self::ButtonDelay(f32::from_bits(
            MAC_AUTOSCROLL_BUTTON_DELAY.load(Ordering::Relaxed),
        ))
    }
    pub fn SetMockScrollbarsEnabled(flag: bool) {
        MOCK.store(flag, Ordering::Relaxed);
    }
    pub fn MockScrollbarsEnabled() -> bool {
        MOCK.load(Ordering::Relaxed)
    }
    pub fn SetOverlayScrollbarsEnabled(flag: bool) {
        OVERLAY.store(flag, Ordering::Relaxed);
    }
    pub fn OverlayScrollbarsEnabled() -> bool {
        OVERLAY.load(Ordering::Relaxed)
    }
    pub fn SetFluentScrollbarsEnabled(flag: bool) {
        FLUENT.store(flag, Ordering::Relaxed);
    }
    pub fn FluentScrollbarsEnabled() -> bool {
        FLUENT.load(Ordering::Relaxed)
    }
    pub fn SetDesktopAndroidScrollbarsEnabled(flag: bool) {
        DESKTOP_ANDROID.store(flag, Ordering::Relaxed);
    }
    pub fn DesktopAndroidScrollbarsEnabled() -> bool {
        DESKTOP_ANDROID.load(Ordering::Relaxed)
    }
}

static SCROLLBAR_ANIMATIONS: AtomicBool = AtomicBool::new(true);
impl ScrollbarThemeSettings {
    // ui/native_theme/features/native_theme_features.cc:49, enabled default.
    pub fn ScrollbarAnimationsEnabled() -> bool {
        SCROLLBAR_ANIMATIONS.load(Ordering::Relaxed)
    }
    pub fn SetScrollbarAnimationsEnabled(enabled: bool) {
        SCROLLBAR_ANIMATIONS.store(enabled, Ordering::Relaxed);
    }
}
