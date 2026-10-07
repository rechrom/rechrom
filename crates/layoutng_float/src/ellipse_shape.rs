#![allow(non_snake_case)]

use foundation::{gfx, LayoutUnit};
use layoutng_assembly::internal::shapes::shape::{LineSegment, Shape, ShapeBase};
use layoutng_geometry::geometry::logical_rect::LogicalRect;

// cpp: layoutng_float/ellipse_shape.h:41-66
pub struct EllipseShape {
    base: ShapeBase,
    center_: gfx::PointF,
    radius_inline_: f32,
    radius_block_: f32,
}

impl EllipseShape {
    // cpp: layoutng_float/ellipse_shape.h:45-53
    pub fn new(center: &gfx::PointF, radius_inline: f32, radius_block: f32) -> Self {
        debug_assert!(radius_inline >= 0.0);
        debug_assert!(radius_block >= 0.0);
        Self {
            base: ShapeBase::default(),
            center_: *center,
            radius_inline_: radius_inline,
            radius_block_: radius_block,
        }
    }

    // cpp: layoutng_float/ellipse_shape.h:60-65
    // cpp: layoutng_float/ellipse_shape.cc:46-49
    fn InlineAndBlockRadiiIncludingMargin(&self) -> (f32, f32) {
        (
            self.radius_inline_ + self.ShapeMargin(),
            self.radius_block_ + self.ShapeMargin(),
        )
    }
}

// cpp: layoutng_float/ellipse_shape.cc:38-42
fn EllipseXIntercept(y: f32, radius_inline: f32, radius_block: f32) -> f32 {
    debug_assert!(radius_block > 0.0);
    radius_inline * (1.0 - (y * y) / (radius_block * radius_block)).sqrt()
}

// cpp: layoutng_float/ellipse_shape.h:43-58
impl Shape for EllipseShape {
    fn base(&self) -> &ShapeBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ShapeBase {
        &mut self.base
    }

    // cpp: layoutng_float/ellipse_shape.h:55-55
    // cpp: layoutng_float/ellipse_shape.cc:51-59
    fn ShapeMarginLogicalBoundingBox(&self) -> LogicalRect {
        debug_assert!(self.ShapeMargin() >= 0.0);
        let (margin_radius_inline, margin_radius_block) = self.InlineAndBlockRadiiIncludingMargin();
        LogicalRect::from_units(
            LayoutUnit::from_f32(self.center_.x() - margin_radius_inline),
            LayoutUnit::from_f32(self.center_.y() - margin_radius_block),
            LayoutUnit::from_f32(margin_radius_inline * 2.0),
            LayoutUnit::from_f32(margin_radius_block * 2.0),
        )
    }

    // cpp: layoutng_float/ellipse_shape.h:56-56
    fn IsEmpty(&self) -> bool {
        self.radius_inline_ == 0.0 || self.radius_block_ == 0.0
    }

    // cpp: layoutng_float/ellipse_shape.h:57-58
    // cpp: layoutng_float/ellipse_shape.cc:61-82
    fn GetExcludedInterval(
        &self,
        logical_top: LayoutUnit,
        logical_height: LayoutUnit,
    ) -> LineSegment {
        let (margin_radius_inline, margin_radius_block) = self.InlineAndBlockRadiiIncludingMargin();
        if margin_radius_inline == 0.0 || margin_radius_block == 0.0 {
            return LineSegment::default();
        }

        let y1 = logical_top.ToFloat();
        let y2 = (logical_top + logical_height).ToFloat();
        let top = self.center_.y() - margin_radius_block;
        let bottom = self.center_.y() + margin_radius_block;
        if y2 < top || y1 >= bottom {
            return LineSegment::default();
        }

        let mut x_intercept = margin_radius_inline;
        if y1 > self.center_.y() || y2 < self.center_.y() {
            let y_intercept = if y1 > self.center_.y() {
                y1 - self.center_.y()
            } else {
                y2 - self.center_.y()
            };
            x_intercept = EllipseXIntercept(y_intercept, margin_radius_inline, margin_radius_block);
        }
        LineSegment::new(
            self.center_.x() - x_intercept,
            self.center_.x() + x_intercept,
        )
    }
}
