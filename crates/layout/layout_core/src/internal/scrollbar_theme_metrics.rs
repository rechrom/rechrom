use foundation::{EScrollbarWidth, UnsupportedLayout};

// cpp: layoutng/internal/scrollbar_theme_metrics.h:8-12
pub use super::layout_input_types::ScrollbarThemeMetrics;

#[derive(Debug)]
pub struct InvalidScrollbarThemeMetrics;

impl std::fmt::Display for InvalidScrollbarThemeMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Scrollbar theme sizes must be nonnegative")
    }
}

impl std::error::Error for InvalidScrollbarThemeMetrics {}

// cpp: layoutng/internal/scrollbar_theme_metrics.cc:8-13
#[allow(non_snake_case)]
pub fn RequireScrollbarTheme(theme: &Option<ScrollbarThemeMetrics>) -> &ScrollbarThemeMetrics {
    match theme {
        Some(metrics) => metrics,
        None => std::panic::panic_any(UnsupportedLayout::new(
            "Scrollbar dimensions require ConstraintSpace::scrollbar_theme",
        )),
    }
}

// cpp: layoutng/internal/scrollbar_theme_metrics.cc:14-18
#[allow(non_snake_case)]
pub fn ValidateScrollbarThemeMetrics(metrics: &ScrollbarThemeMetrics) {
    if metrics.auto_thickness < 0 || metrics.thin_thickness < 0 || metrics.minimum_thumb_length < 0
    {
        std::panic::panic_any(InvalidScrollbarThemeMetrics);
    }
}

// cpp: layoutng/internal/scrollbar_theme_metrics.cc:19-26
#[allow(non_snake_case)]
pub fn ScrollbarThemeThickness(metrics: &ScrollbarThemeMetrics, width: EScrollbarWidth) -> i32 {
    match width {
        EScrollbarWidth::kAuto => metrics.auto_thickness,
        EScrollbarWidth::kThin => metrics.thin_thickness,
        EScrollbarWidth::kNone => 0,
    }
}
