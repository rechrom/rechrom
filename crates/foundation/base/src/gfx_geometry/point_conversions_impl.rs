// C++: src/foundation/gfx_geometry/point_conversions.cc

use super::point::Point;
use super::point_f::PointF;

fn ClampToInt(value: f32) -> i32 {
    if value.is_nan() {
        0
    } else if value >= i32::MAX as f32 {
        i32::MAX
    } else if value <= i32::MIN as f32 {
        i32::MIN
    } else {
        value as i32
    }
}

pub fn ToFlooredPoint(point: PointF) -> Point {
    Point::new(ClampToInt(point.x().floor()), ClampToInt(point.y().floor()))
}

pub fn ToCeiledPoint(point: PointF) -> Point {
    Point::new(ClampToInt(point.x().ceil()), ClampToInt(point.y().ceil()))
}

pub fn ToRoundedPoint(point: PointF) -> Point {
    Point::new(ClampToInt(point.x().round()), ClampToInt(point.y().round()))
}
