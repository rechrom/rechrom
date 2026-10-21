// C++: src/foundation/gfx_geometry/size_f.cc

use super::size_f::SizeF;

impl SizeF {
    pub fn GetArea(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn Enlarge(&mut self, grow_width: f32, grow_height: f32) {
        self.SetSize(self.width() + grow_width, self.height() + grow_height);
    }

    pub fn SetToMin(&mut self, other: &SizeF) {
        self.set_width(if other.width() < self.width() {
            other.width()
        } else {
            self.width()
        });
        self.set_height(if other.height() < self.height() {
            other.height()
        } else {
            self.height()
        });
    }

    pub fn SetToMax(&mut self, other: &SizeF) {
        self.set_width(if self.width() < other.width() {
            other.width()
        } else {
            self.width()
        });
        self.set_height(if self.height() < other.height() {
            other.height()
        } else {
            self.height()
        });
    }

    pub fn ToString(&self) -> String {
        format!("{}x{}", self.width(), self.height())
    }
}

pub fn ScaleSize(size: SizeF, x_scale: f32, y_scale: f32) -> SizeF {
    let mut result = size;
    result.Scale(x_scale, y_scale);
    result
}

pub fn ScaleSizeUniform(size: SizeF, scale: f32) -> SizeF {
    ScaleSize(size, scale, scale)
}
