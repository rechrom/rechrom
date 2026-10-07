// C++: font_engine/fonts/font_metrics_override.h.
// cpp: font_engine/fonts/font_metrics_override.h:12-16
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontMetricsOverride {
    pub ascent_override: Option<f32>,
    pub descent_override: Option<f32>,
    pub line_gap_override: Option<f32>,
}
