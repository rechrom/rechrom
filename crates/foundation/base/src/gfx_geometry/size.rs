// C++: src/foundation/gfx_geometry/size.h

use std::ops::{Add, AddAssign, Sub, SubAssign};

// C++ class gfx::Size -> Rust struct + impl. Both components remain nonnegative.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size {
    width_: i32,
    height_: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width_: width.max(0),
            height_: height.max(0),
        }
    }

    pub fn width(&self) -> i32 {
        self.width_
    }

    pub fn height(&self) -> i32 {
        self.height_
    }

    pub fn set_width(&mut self, width: i32) {
        self.width_ = width.max(0);
    }

    pub fn set_height(&mut self, height: i32) {
        self.height_ = height.max(0);
    }

    pub fn Area64(&self) -> u64 {
        self.width_ as u64 * self.height_ as u64
    }

    pub fn SetSize(&mut self, width: i32, height: i32) {
        self.set_width(width);
        self.set_height(height);
    }

    pub fn IsEmpty(&self) -> bool {
        self.width_ == 0 || self.height_ == 0
    }

    pub fn IsZero(&self) -> bool {
        self.width_ == 0 && self.height_ == 0
    }

    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.width_, &mut self.height_);
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.Enlarge(rhs.width(), rhs.height());
    }
}

impl SubAssign for Size {
    fn sub_assign(&mut self, rhs: Self) {
        self.Enlarge(-rhs.width(), -rhs.height());
    }
}

impl Add for Size {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl Sub for Size {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

pub fn TransposeSize(size: Size) -> Size {
    Size::new(size.height(), size.width())
}
