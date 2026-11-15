// C++: src/foundation/gfx_geometry/vector2d_f.h

use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

// C++ class gfx::Vector2dF -> Rust struct + impl.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2dF {
    x_: f32,
    y_: f32,
}

impl Vector2dF {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x_: x, y_: y }
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

    pub fn SetToMin(&mut self, other: &Self) {
        self.x_ = if other.x_ < self.x_ {
            other.x_
        } else {
            self.x_
        };
        self.y_ = if other.y_ < self.y_ {
            other.y_
        } else {
            self.y_
        };
    }

    pub fn SetToMax(&mut self, other: &Self) {
        self.x_ = if self.x_ < other.x_ {
            other.x_
        } else {
            self.x_
        };
        self.y_ = if self.y_ < other.y_ {
            other.y_
        } else {
            self.y_
        };
    }

    pub fn AspectRatio(&self) -> f32 {
        self.x_ / self.y_
    }

    pub fn ScaleUniform(&mut self, scale: f32) {
        self.Scale(scale, scale);
    }

    pub fn InvScaleUniform(&mut self, scale: f32) {
        self.InvScale(scale, scale);
    }

    pub fn Normalize(&mut self) {
        self.InvScaleUniform(self.Length());
    }

    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.x_, &mut self.y_);
    }
}

impl Neg for Vector2dF {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x_, -self.y_)
    }
}

impl Add for Vector2dF {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.Add(&rhs);
        self
    }
}

impl Sub for Vector2dF {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        self.Add(&-rhs);
        self
    }
}

impl AddAssign for Vector2dF {
    fn add_assign(&mut self, rhs: Self) {
        self.Add(&rhs);
    }
}

impl SubAssign for Vector2dF {
    fn sub_assign(&mut self, rhs: Self) {
        self.Subtract(&rhs);
    }
}

pub fn ScaleVector2dUniform(vector: Vector2dF, scale: f32) -> Vector2dF {
    ScaleVector2d(vector, scale, scale)
}

pub fn TransposeVector2dF(vector: Vector2dF) -> Vector2dF {
    Vector2dF::new(vector.y(), vector.x())
}

pub fn NormalizeVector2d(vector: Vector2dF) -> Vector2dF {
    let mut normal = vector;
    normal.Normalize();
    normal
}

pub use super::vector2d_f_impl::{CrossProduct, DotProduct, ScaleVector2d};
