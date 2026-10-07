// C++: font_engine/fonts/glyph_metrics_map.h. The first 256 glyphs keep a
// dedicated page; other pages are allocated only when first queried.
use super::glyph::Glyph;
use foundation::gfx::{PointF, SizeF};
use foundation::RectF;
use std::collections::HashMap;

pub trait GlyphMetricsValue: Copy + PartialEq {
    fn UnknownMetrics() -> Self;
}

// cpp: font_engine/fonts/glyph_metrics_map.h:113-121
impl GlyphMetricsValue for f32 {
    fn UnknownMetrics() -> Self {
        -1.0
    }
}

impl GlyphMetricsValue for RectF {
    fn UnknownMetrics() -> Self {
        RectF::new(PointF::new(f32::MIN_POSITIVE, 0.0), SizeF::new(0.0, 0.0))
    }
}

// cpp: font_engine/fonts/glyph_metrics_map.h:45-111
pub struct GlyphMetricsMap<T: GlyphMetricsValue> {
    filled_primary_page_: bool,
    primary_page_: [T; 256],
    pages_: Option<HashMap<u32, Box<[T; 256]>>>,
}

impl<T: GlyphMetricsValue> Default for GlyphMetricsMap<T> {
    fn default() -> Self {
        Self {
            filled_primary_page_: false,
            primary_page_: [T::UnknownMetrics(); 256],
            pages_: None,
        }
    }
}

impl<T: GlyphMetricsValue> GlyphMetricsMap<T> {
    pub fn new() -> Self {
        Self::default()
    }

    // cpp: font_engine/fonts/glyph_metrics_map.h:53-60,73-90
    pub fn MetricsForGlyph(&mut self, glyph: Glyph) -> Option<T> {
        let value = self.LocatePage((glyph as u32) / 256)[glyph as usize % 256];
        (value != T::UnknownMetrics()).then_some(value)
    }

    pub fn SetMetricsForGlyph(&mut self, glyph: Glyph, metrics: T) {
        self.LocatePage((glyph as u32) / 256)[glyph as usize % 256] = metrics;
    }

    // cpp: font_engine/fonts/glyph_metrics_map.h:97-103,123-150
    fn LocatePage(&mut self, page_number: u32) -> &mut [T; 256] {
        if page_number == 0 {
            if !self.filled_primary_page_ {
                self.primary_page_.fill(T::UnknownMetrics());
                self.filled_primary_page_ = true;
            }
            return &mut self.primary_page_;
        }
        self.pages_
            .get_or_insert_with(HashMap::new)
            .entry(page_number)
            .or_insert_with(|| Box::new([T::UnknownMetrics(); 256]))
    }
}
