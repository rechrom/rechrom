#![allow(non_snake_case)]

use super::glyph_data::GlyphOffset;
use crate::fonts::font_baseline::FontBaseline;
use crate::fonts::font_metrics::FontMetrics;
use foundation::gfx::{PointF, RectF, SizeF};

// cpp: font_engine/fonts/shaping/glyph_bounds_accumulator.h:44-94
#[derive(Clone, Copy, Debug, Default)]
pub struct GlyphBoundsAccumulator<const IS_HORIZONTAL_RUN: bool> {
    has_bounds_: bool,
    min_x_: f32,
    max_x_: f32,
    min_y_: f32,
    max_y_: f32,
}

impl<const IS_HORIZONTAL_RUN: bool> GlyphBoundsAccumulator<IS_HORIZONTAL_RUN> {
    pub fn Unite(&mut self, mut bounds: RectF, origin: f32, glyph_offset: GlyphOffset) {
        if bounds.size().IsEmpty() {
            return;
        }
        if IS_HORIZONTAL_RUN {
            bounds = RectF::new(PointF::new(bounds.x() + origin, bounds.y()), bounds.size());
        } else {
            bounds = RectF::new(PointF::new(bounds.x(), bounds.y() + origin), bounds.size());
        }
        bounds.Offset(glyph_offset.x(), glyph_offset.y());

        if !self.has_bounds_ {
            self.min_x_ = bounds.x();
            self.max_x_ = bounds.right();
            self.min_y_ = bounds.y();
            self.max_y_ = bounds.bottom();
            self.has_bounds_ = true;
            return;
        }
        self.min_x_ = self.min_x_.min(bounds.x());
        self.max_x_ = self.max_x_.max(bounds.right());
        self.min_y_ = self.min_y_.min(bounds.y());
        self.max_y_ = self.max_y_.max(bounds.bottom());
    }

    pub fn BuildBounds(mut self, metrics: &FontMetrics) -> RectF {
        if !self.has_bounds_ {
            return RectF::default();
        }
        if !IS_HORIZONTAL_RUN {
            std::mem::swap(&mut self.min_x_, &mut self.min_y_);
            std::mem::swap(&mut self.max_x_, &mut self.max_y_);
            let baseline_adjust = metrics.AscentFor(FontBaseline::kCentralBaseline)
                - metrics.AscentFor(FontBaseline::kAlphabeticBaseline);
            self.min_y_ += baseline_adjust as f32;
            self.max_y_ += baseline_adjust as f32;
        }
        RectF::new(
            PointF::new(self.min_x_, self.min_y_),
            SizeF::new(self.max_x_ - self.min_x_, self.max_y_ - self.min_y_),
        )
    }
}
