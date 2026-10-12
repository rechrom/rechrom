// C++: src/foundation/gfx_geometry/point_f.h

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::point::Point;
use super::vector2d_f::Vector2dF;

// C++ class gfx::PointF -> Rust struct + impl.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointF {
    x_: f32,
    y_: f32,
}

impl PointF {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x_: x, y_: y }
    }

    pub fn from_point(point: Point) -> Self {
        Self::new(point.x() as f32, point.y() as f32)
    }

    pub fn x(&self) -> f32 {
        self.x_
    }

    pub fn y(&self) -> f32 {
        self.y_
    }

    pub fn set_x(&mut self, x: f32) {
        self.x_ = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.y_ = y;
    }

    pub fn SetPoint(&mut self, x: f32, y: f32) {
        self.x_ = x;
        self.y_ = y;
    }

    pub fn Offset(&mut self, delta_x: f32, delta_y: f32) {
        self.x_ += delta_x;
        self.y_ += delta_y;
    }

    pub fn IsOrigin(&self) -> bool {
        self.x_ == 0.0 && self.y_ == 0.0
    }

    pub fn OffsetFromOrigin(&self) -> Vector2dF {
        Vector2dF::new(self.x_, self.y_)
    }

    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        self.SetPoint(self.x_ * x_scale, self.y_ * y_scale);
    }

    pub fn ScaleUniform(&mut self, scale: f32) {
        self.Scale(scale, scale);
    }

    pub fn InvScale(&mut self, inv_x_scale: f32, inv_y_scale: f32) {
        self.x_ /= inv_x_scale;
        self.y_ /= inv_y_scale;
    }

    pub fn InvScaleUniform(&mut self, scale: f32) {
        self.InvScale(scale, scale);
    }

    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.x_, &mut self.y_);
    }
}

impl PartialOrd for PointF {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match self.y_.partial_cmp(&rhs.y_)? {
            Ordering::Equal => self.x_.partial_cmp(&rhs.x_),
            result => Some(result),
        }
    }
}

impl AddAssign<Vector2dF> for PointF {
    fn add_assign(&mut self, rhs: Vector2dF) {
        self.x_ += rhs.x();
        self.y_ += rhs.y();
    }
}

impl SubAssign<Vector2dF> for PointF {
    fn sub_assign(&mut self, rhs: Vector2dF) {
        self.x_ -= rhs.x();
        self.y_ -= rhs.y();
    }
}

impl Add<Vector2dF> for PointF {
    type Output = Self;
    fn add(mut self, rhs: Vector2dF) -> Self {
        self += rhs;
        self
    }
}

impl Sub<Vector2dF> for PointF {
    type Output = Self;
    fn sub(mut self, rhs: Vector2dF) -> Self {
        self -= rhs;
        self
    }
}

impl Sub<PointF> for PointF {
    type Output = Vector2dF;
    fn sub(self, rhs: PointF) -> Vector2dF {
        Vector2dF::new(self.x_ - rhs.x_, self.y_ - rhs.y_)
    }
}

pub fn PointAtOffsetFromOriginF(offset: Vector2dF) -> PointF {
    PointF::new(offset.x(), offset.y())
}

pub fn TransposePointF(point: PointF) -> PointF {
    PointF::new(point.y(), point.x())
}
