#![allow(non_snake_case)]

use foundation::{gfx, LayoutUnit};
use layoutng_assembly::internal::shapes::shape::{LineSegment, Shape, ShapeBase};
use layoutng_geometry::geometry::logical_rect::LogicalRect;

// cpp: layoutng_float/inset_shape.h:41-66
pub struct InsetShape {
    base: ShapeBase,
    bounds_: gfx::RectF,
    radii_: [gfx::SizeF; 4],
}

// cpp: layoutng_float/inset_shape.h:53-56
struct Geometry {
    bounds: gfx::RectF,
    radii: [gfx::SizeF; 4],
}

// cpp: layoutng_float/inset_shape.cc:39-43
fn CornerXIntercept(y: f32, radius_inline: f32, radius_block: f32) -> f32 {
    if radius_inline <= 0.0 || radius_block <= 0.0 {
        return 0.0;
    }
    let ratio = y / radius_block;
    radius_inline * (1.0_f32 - ratio * ratio).max(0.0).sqrt()
}

impl InsetShape {
    // cpp: layoutng_float/inset_shape.h:45-45
    // cpp: layoutng_float/inset_shape.cc:47-64
    pub fn new(bounds: gfx::RectF, radii: [gfx::SizeF; 4]) -> Self {
        let mut shape = Self {
            base: ShapeBase::default(),
            bounds_: bounds,
            radii_: radii,
        };
        let mut factor = 1.0_f32;
        let horizontal_sum = (shape.radii_[0].width() + shape.radii_[1].width())
            .max(shape.radii_[3].width() + shape.radii_[2].width());
        if horizontal_sum > shape.bounds_.width() {
            factor = factor.min(shape.bounds_.width() / horizontal_sum);
        }
        let vertical_sum = (shape.radii_[0].height() + shape.radii_[3].height())
            .max(shape.radii_[1].height() + shape.radii_[2].height());
        if vertical_sum > shape.bounds_.height() {
            factor = factor.min(shape.bounds_.height() / vertical_sum);
        }
        if factor < 1.0 {
            for radius in &mut shape.radii_ {
                radius.ScaleUniform(factor);
            }
        }
        shape
    }

    // cpp: layoutng_float/inset_shape.h:58-58
    // cpp: layoutng_float/inset_shape.cc:66-76
    fn ShapeMarginGeometry(&self) -> Geometry {
        let mut result = Geometry {
            bounds: self.bounds_,
            radii: self.radii_,
        };
        let margin = self.ShapeMargin();
        if margin == 0.0 {
            return result;
        }
        result.bounds.Inset(-margin);
        for radius in &mut result.radii {
            radius.set_width(radius.width() + margin);
            radius.set_height(radius.height() + margin);
        }
        result
    }

    // cpp: layoutng_float/inset_shape.h:59-62
    // cpp: layoutng_float/inset_shape.cc:84-123
    fn XInterceptsAtY(geometry: &Geometry, y: f32, minimum: &mut f32, maximum: &mut f32) -> bool {
        let bounds = &geometry.bounds;
        if y < bounds.y() || y > bounds.bottom() {
            return false;
        }
        let top_left = &geometry.radii[0];
        let top_right = &geometry.radii[1];
        let bottom_right = &geometry.radii[2];
        let bottom_left = &geometry.radii[3];

        if top_left.width() != 0.0 && top_left.height() != 0.0 && y < bounds.y() + top_left.height()
        {
            *minimum = bounds.x() + top_left.width()
                - CornerXIntercept(
                    bounds.y() + top_left.height() - y,
                    top_left.width(),
                    top_left.height(),
                );
        } else if bottom_left.width() != 0.0
            && bottom_left.height() != 0.0
            && y >= bounds.bottom() - bottom_left.height()
        {
            *minimum = bounds.x() + bottom_left.width()
                - CornerXIntercept(
                    y - (bounds.bottom() - bottom_left.height()),
                    bottom_left.width(),
                    bottom_left.height(),
                );
        } else {
            *minimum = bounds.x();
        }

        if top_right.width() != 0.0
            && top_right.height() != 0.0
            && y <= bounds.y() + top_right.height()
        {
            *maximum = bounds.right() - top_right.width()
                + CornerXIntercept(
                    bounds.y() + top_right.height() - y,
                    top_right.width(),
                    top_right.height(),
                );
        } else if bottom_right.width() != 0.0
            && bottom_right.height() != 0.0
            && y >= bounds.bottom() - bottom_right.height()
        {
            *maximum = bounds.right() - bottom_right.width()
                + CornerXIntercept(
                    y - (bounds.bottom() - bottom_right.height()),
                    bottom_right.width(),
                    bottom_right.height(),
                );
        } else {
            *maximum = bounds.right();
        }
        true
    }
}

// cpp: layoutng_float/inset_shape.h:43-50
impl Shape for InsetShape {
    fn base(&self) -> &ShapeBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ShapeBase {
        &mut self.base
    }

    // cpp: layoutng_float/inset_shape.h:47-47
    // cpp: layoutng_float/inset_shape.cc:78-82
    fn ShapeMarginLogicalBoundingBox(&self) -> LogicalRect {
        let bounds = self.ShapeMarginGeometry().bounds;
        LogicalRect::from_units(
            LayoutUnit::from_f32(bounds.x()),
            LayoutUnit::from_f32(bounds.y()),
            LayoutUnit::from_f32(bounds.width()),
            LayoutUnit::from_f32(bounds.height()),
        )
    }

    // cpp: layoutng_float/inset_shape.h:48-48
    fn IsEmpty(&self) -> bool {
        self.bounds_.IsEmpty()
    }

    // cpp: layoutng_float/inset_shape.h:49-50
    // cpp: layoutng_float/inset_shape.cc:125-171
    fn GetExcludedInterval(
        &self,
        logical_top: LayoutUnit,
        logical_height: LayoutUnit,
    ) -> LineSegment {
        let geometry = self.ShapeMarginGeometry();
        if geometry.bounds.IsEmpty()
            || !self.LineOverlapsShapeMarginBounds(logical_top, logical_height)
        {
            return LineSegment::default();
        }

        let y1 = logical_top.ToFloat();
        let y2 = (logical_top + logical_height).ToFloat();
        let radii = &geometry.radii;
        let rounded = radii
            .iter()
            .any(|radius| radius.width() != 0.0 || radius.height() != 0.0);
        if !rounded {
            return LineSegment::new(geometry.bounds.x(), geometry.bounds.right());
        }

        let top_corner_max_y = geometry.bounds.y() + radii[0].height().max(radii[1].height());
        let bottom_corner_min_y =
            geometry.bounds.bottom() - radii[3].height().max(radii[2].height());
        if top_corner_max_y <= bottom_corner_min_y
            && y1 <= top_corner_max_y
            && y2 >= bottom_corner_min_y
        {
            return LineSegment::new(geometry.bounds.x(), geometry.bounds.right());
        }

        let mut left = geometry.bounds.right();
        let mut right = geometry.bounds.x();
        if y1 <= geometry.bounds.y() + radii[0].height()
            && y2 >= geometry.bounds.bottom() - radii[3].height()
        {
            left = geometry.bounds.x();
        }
        if y1 <= geometry.bounds.y() + radii[1].height()
            && y2 >= geometry.bounds.bottom() - radii[2].height()
        {
            right = geometry.bounds.right();
        }

        let mut minimum = 0.0;
        let mut maximum = 0.0;
        if Self::XInterceptsAtY(&geometry, y1, &mut minimum, &mut maximum) {
            left = left.min(minimum);
            right = right.max(maximum);
        }
        if Self::XInterceptsAtY(&geometry, y2, &mut minimum, &mut maximum) {
            left = left.min(minimum);
            right = right.max(maximum);
        }
        if right >= left {
            LineSegment::new(left, right)
        } else {
            LineSegment::default()
        }
    }
}
