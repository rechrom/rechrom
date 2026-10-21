// C++: src/foundation/gfx_geometry/size.cc

use super::size::Size;
use super::size_conversions::{ToCeiledSize, ToFlooredSize, ToRoundedSize};
use super::size_f::SizeF;
use super::size_f_impl::ScaleSize;

impl Size {
    pub fn GetArea(&self) -> i32 {
        self.GetCheckedArea().expect("Size area overflow")
    }

    // C++ returns base::CheckedNumeric<int>; Rust represents its valid/invalid
    // state with Option until the base numerics source is translated.
    pub fn GetCheckedArea(&self) -> Option<i32> {
        self.width().checked_mul(self.height())
    }

    pub fn Enlarge(&mut self, grow_width: i32, grow_height: i32) {
        self.SetSize(
            self.width().saturating_add(grow_width),
            self.height().saturating_add(grow_height),
        );
    }

    pub fn SetToMin(&mut self, other: &Size) {
        self.set_width(self.width().min(other.width()));
        self.set_height(self.height().min(other.height()));
    }

    pub fn SetToMax(&mut self, other: &Size) {
        self.set_width(self.width().max(other.width()));
        self.set_height(self.height().max(other.height()));
    }

    pub fn ToString(&self) -> String {
        format!("{}x{}", self.width(), self.height())
    }
}

pub fn ScaleToCeiledSize(size: Size, x_scale: f32, y_scale: f32) -> Size {
    if x_scale == 1.0 && y_scale == 1.0 {
        return size;
    }
    ToCeiledSize(ScaleSize(SizeF::from_size(size), x_scale, y_scale))
}

pub fn ScaleToCeiledSizeUniform(size: Size, scale: f32) -> Size {
    if scale == 1.0 {
        return size;
    }
    ToCeiledSize(ScaleSize(SizeF::from_size(size), scale, scale))
}

pub fn ScaleToFlooredSize(size: Size, x_scale: f32, y_scale: f32) -> Size {
    if x_scale == 1.0 && y_scale == 1.0 {
        return size;
    }
    ToFlooredSize(ScaleSize(SizeF::from_size(size), x_scale, y_scale))
}

pub fn ScaleToFlooredSizeUniform(size: Size, scale: f32) -> Size {
    if scale == 1.0 {
        return size;
    }
    ToFlooredSize(ScaleSize(SizeF::from_size(size), scale, scale))
}

pub fn ScaleToRoundedSize(size: Size, x_scale: f32, y_scale: f32) -> Size {
    if x_scale == 1.0 && y_scale == 1.0 {
        return size;
    }
    ToRoundedSize(ScaleSize(SizeF::from_size(size), x_scale, y_scale))
}

pub fn ScaleToRoundedSizeUniform(size: Size, scale: f32) -> Size {
    if scale == 1.0 {
        return size;
    }
    ToRoundedSize(ScaleSize(SizeF::from_size(size), scale, scale))
}
