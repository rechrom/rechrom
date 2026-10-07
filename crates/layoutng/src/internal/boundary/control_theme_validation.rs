use crate::internal::layout_input_types::ControlThemeMetrics;

#[derive(Debug)]
pub struct InvalidControlThemeMetrics;

impl std::fmt::Display for InvalidControlThemeMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Control theme dimensions must be nonnegative")
    }
}

impl std::error::Error for InvalidControlThemeMetrics {}

// cpp: layoutng/internal/boundary/control_theme_validation.h:6-8
// cpp: layoutng/internal/boundary/control_theme_validation.cc:30-36
#[allow(non_snake_case)]
pub fn ValidateControlThemeMetrics(metrics: &ControlThemeMetrics) {
    for size in [metrics.checkbox, metrics.radio] {
        if let Some(size) = size {
            if size.width < 0 || size.height < 0 {
                std::panic::panic_any(InvalidControlThemeMetrics);
            }
        }
    }
}
