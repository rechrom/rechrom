// C++: src/foundation/gfx_geometry/vector2d.h

use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use super::vector2d_f::Vector2dF;

// C++ class gfx::Vector2d -> Rust struct + impl.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Vector2d {
    x_: i32,
    y_: i32,
}

impl Vector2d {
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

    pub fn SetToMin(&mut self, other: &Self) {
        self.x_ = self.x_.min(other.x_);
        self.y_ = self.y_.min(other.y_);
    }

    pub fn SetToMax(&mut self, other: &Self) {
        self.x_ = self.x_.max(other.x_);
        self.y_ = self.y_.max(other.y_);
    }

    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.x_, &mut self.y_);
    }
}

impl From<Vector2d> for Vector2dF {
    fn from(value: Vector2d) -> Self {
        Self::new(value.x() as f32, value.y() as f32)
    }
}

impl Neg for Vector2d {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(self.x_.saturating_neg(), self.y_.saturating_neg())
    }
}

impl Add for Vector2d {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.Add(&rhs);
        self
    }
}

impl Sub for Vector2d {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        self.Subtract(&rhs);
        self
    }
}

impl AddAssign for Vector2d {
    fn add_assign(&mut self, rhs: Self) {
        self.Add(&rhs);
    }
}

impl SubAssign for Vector2d {
    fn sub_assign(&mut self, rhs: Self) {
        self.Subtract(&rhs);
    }
}

pub fn TransposeVector2d(vector: Vector2d) -> Vector2d {
    Vector2d::new(vector.y(), vector.x())
}
