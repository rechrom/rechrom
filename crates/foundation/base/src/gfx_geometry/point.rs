// C++: src/foundation/gfx_geometry/point.h

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::vector2d::Vector2d;

// C++ class gfx::Point -> Rust struct + impl.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Point {
    x_: i32,
    y_: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x_: x, y_: y }
    }

    pub fn x(&self) -> i32 {
        self.x_
    }

    pub fn y(&self) -> i32 {
        self.y_
    }

    pub fn set_x(&mut self, x: i32) {
        self.x_ = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y_ = y;
    }

    pub fn SetPoint(&mut self, x: i32, y: i32) {
        self.x_ = x;
        self.y_ = y;
    }

    pub fn Offset(&mut self, delta_x: i32, delta_y: i32) {
        self.x_ = self.x_.saturating_add(delta_x);
        self.y_ = self.y_.saturating_add(delta_y);
    }

    pub fn IsOrigin(&self) -> bool {
        self.x_ == 0 && self.y_ == 0
    }

    pub fn OffsetFromOrigin(&self) -> Vector2d {
        Vector2d::new(self.x_, self.y_)
    }

    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.x_, &mut self.y_);
    }
}

impl Ord for Point {
    fn cmp(&self, rhs: &Self) -> Ordering {
        (self.y_, self.x_).cmp(&(rhs.y_, rhs.x_))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl AddAssign<Vector2d> for Point {
    fn add_assign(&mut self, vector: Vector2d) {
        self.x_ = self.x_.saturating_add(vector.x());
        self.y_ = self.y_.saturating_add(vector.y());
    }
}

impl SubAssign<Vector2d> for Point {
    fn sub_assign(&mut self, vector: Vector2d) {
        self.x_ = self.x_.saturating_sub(vector.x());
        self.y_ = self.y_.saturating_sub(vector.y());
    }
}

impl Add<Vector2d> for Point {
    type Output = Point;
    fn add(mut self, rhs: Vector2d) -> Self::Output {
        self += rhs;
        self
    }
}

impl Sub<Vector2d> for Point {
    type Output = Point;
    fn sub(mut self, rhs: Vector2d) -> Self::Output {
        self -= rhs;
        self
    }
}

impl Sub<Point> for Point {
    type Output = Vector2d;
    fn sub(self, rhs: Point) -> Self::Output {
        Vector2d::new(
            self.x_.saturating_sub(rhs.x_),
            self.y_.saturating_sub(rhs.y_),
        )
    }
}

pub fn PointAtOffsetFromOrigin(offset: Vector2d) -> Point {
    Point::new(offset.x(), offset.y())
}

pub fn TransposePoint(point: Point) -> Point {
    Point::new(point.y(), point.x())
}
