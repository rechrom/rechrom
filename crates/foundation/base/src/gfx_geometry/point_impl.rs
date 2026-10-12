// C++: src/foundation/gfx_geometry/point.cc

use super::point::Point;
use super::point_conversions::{ToCeiledPoint, ToFlooredPoint, ToRoundedPoint};
use super::point_f::PointF;
use super::point_f_impl::ScalePoint;

impl Point {
    pub fn SetToMin(&mut self, other: &Self) {
        self.set_x(self.x().min(other.x()));
        self.set_y(self.y().min(other.y()));
    }

    pub fn SetToMax(&mut self, other: &Self) {
        self.set_x(self.x().max(other.x()));
        self.set_y(self.y().max(other.y()));
    }

    pub fn ToString(&self) -> String {
        format!("{},{}", self.x(), self.y())
    }
}

pub fn ScaleToCeiledPoint(point: Point, x_scale: f32, y_scale: f32) -> Point {
    if x_scale == 1.0 && y_scale == 1.0 {
        return point;
    }
    ToCeiledPoint(ScalePoint(PointF::from_point(point), x_scale, y_scale))
}

pub fn ScaleToCeiledPointUniform(point: Point, scale: f32) -> Point {
    if scale == 1.0 {
        return point;
    }
    ToCeiledPoint(ScalePoint(PointF::from_point(point), scale, scale))
}

pub fn ScaleToFlooredPoint(point: Point, x_scale: f32, y_scale: f32) -> Point {
    if x_scale == 1.0 && y_scale == 1.0 {
        return point;
    }
    ToFlooredPoint(ScalePoint(PointF::from_point(point), x_scale, y_scale))
}

pub fn ScaleToFlooredPointUniform(point: Point, scale: f32) -> Point {
    if scale == 1.0 {
        return point;
    }
    ToFlooredPoint(ScalePoint(PointF::from_point(point), scale, scale))
}

pub fn ScaleToRoundedPoint(point: Point, x_scale: f32, y_scale: f32) -> Point {
    if x_scale == 1.0 && y_scale == 1.0 {
        return point;
    }
    ToRoundedPoint(ScalePoint(PointF::from_point(point), x_scale, y_scale))
}

pub fn ScaleToRoundedPointUniform(point: Point, scale: f32) -> Point {
    if scale == 1.0 {
        return point;
    }
    ToRoundedPoint(ScalePoint(PointF::from_point(point), scale, scale))
}
