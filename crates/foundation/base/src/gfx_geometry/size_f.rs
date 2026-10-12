// C++: src/foundation/gfx_geometry/size_f.h

use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::size::Size;

// C++ class gfx::SizeF -> Rust struct + impl.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SizeF {
    width_: f32,
    height_: f32,
}

impl SizeF {
    pub const kTrivial: f32 = 8.0 * f32::EPSILON;

    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width_: Self::clamp(width),
            height_: Self::clamp(height),
        }
    }

    pub fn from_size(size: Size) -> Self {
        Self::new(size.width() as f32, size.height() as f32)
    }

    pub fn width(&self) -> f32 {
        self.width_
    }

    pub fn height(&self) -> f32 {
        self.height_
    }

    pub fn set_width(&mut self, width: f32) {
        self.width_ = Self::clamp(width);
    }

    pub fn set_height(&mut self, height: f32) {
        self.height_ = Self::clamp(height);
    }

    pub fn AspectRatio(&self) -> f32 {
        self.width_ / self.height_
    }

    pub fn SetSize(&mut self, width: f32, height: f32) {
        self.set_width(width);
        self.set_height(height);
    }

    pub fn SetToNextWidth(&mut self) {
        self.width_ = Self::next(self.width_);
    }

    pub fn SetToNextHeight(&mut self) {
        self.height_ = Self::next(self.height_);
    }

    pub fn IsEmpty(&self) -> bool {
        self.width_ == 0.0 || self.height_ == 0.0
    }

    pub fn IsZero(&self) -> bool {
        self.width_ == 0.0 && self.height_ == 0.0
    }

    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        self.SetSize(self.width_ * x_scale, self.height_ * y_scale);
    }

    pub fn ScaleUniform(&mut self, scale: f32) {
        self.Scale(scale, scale);
    }

    pub fn InvScale(&mut self, inv_x_scale: f32, inv_y_scale: f32) {
        self.width_ /= inv_x_scale;
        self.height_ /= inv_y_scale;
    }

    pub fn InvScaleUniform(&mut self, inv_scale: f32) {
        self.InvScale(inv_scale, inv_scale);
    }

    pub fn Transpose(&mut self) {
        std::mem::swap(&mut self.width_, &mut self.height_);
    }

    fn clamp(value: f32) -> f32 {
        if value > Self::kTrivial {
            value
        } else {
            0.0
        }
    }

    fn next(value: f32) -> f32 {
        let value = value.max(Self::kTrivial);
        if value.is_infinite() {
            return f32::MAX;
        }
        if value == f32::MAX {
            return value;
        }
        f32::from_bits(value.to_bits() + 1)
    }
}

impl AddAssign for SizeF {
    fn add_assign(&mut self, rhs: Self) {
        self.SetSize(self.width_ + rhs.width_, self.height_ + rhs.height_);
    }
}

impl SubAssign for SizeF {
    fn sub_assign(&mut self, rhs: Self) {
        self.SetSize(self.width_ - rhs.width_, self.height_ - rhs.height_);
    }
}

impl Add for SizeF {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.width_ + rhs.width_, self.height_ + rhs.height_)
    }
}

impl Sub for SizeF {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.width_ - rhs.width_, self.height_ - rhs.height_)
    }
}

pub fn TransposeSizeF(size: SizeF) -> SizeF {
    SizeF::new(size.height(), size.width())
}
