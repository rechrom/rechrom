// C++: font_engine/fonts/font_backend.h. FontVariation is the shared
// Layout/Paint boundary record from layoutng/internal/layout_input.h.
use std::sync::Arc;

// cpp: layoutng/internal/layout_input.h:1018-1023
pub use foundation::FontVariation;

// cpp: font_engine/fonts/font_backend.h:13-23
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontBackendMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub leading: f32,
    pub x_height: f32,
    pub cap_height: f32,
    pub underline_position: f32,
    pub underline_thickness: f32,
    pub has_underline_position: bool,
    pub has_underline_thickness: bool,
}

// cpp: font_engine/fonts/font_backend.h:25-32
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontBackendGlyphMetrics {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
}

// cpp: font_engine/fonts/font_backend.h:34-45
pub trait FontBackend {
    fn Metrics(
        &self,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> FontBackendMetrics;
    fn GlyphMetrics(
        &self,
        glyph: u16,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> FontBackendGlyphMetrics;
    /// Source: skia/skia_text_metrics.cc SkFontGetGlyphWidthForHarfBuzz.
    ///
    /// Shaping asks the selected platform strike for advances before GPOS is
    /// applied. Backends that can query width without computing ink bounds
    /// should override this hot path.
    fn GlyphAdvanceForShaping(
        &self,
        glyph: u16,
        size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
    ) -> f32 {
        self.GlyphMetrics(glyph, size, synthetic_bold, synthetic_italic)
            .advance
    }
    fn WithVariations(&self, variations: &[FontVariation]) -> Arc<dyn FontBackend>;
}

// cpp: font_engine/fonts/font_backend.h:47-56
pub trait FontBackendFactory {
    fn Create(
        &self,
        bytes: &[u8],
        face_index: u32,
        variations: &[FontVariation],
        resolved_family: &str,
        metrics_family: &str,
        weight: f64,
        italic: bool,
    ) -> Arc<dyn FontBackend>;
}
