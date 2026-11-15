// C++: src/foundation/blink_geometry/geometry/length_size.h:29-55
// cpp: foundation/blink_geometry/geometry/length_size.h:29-55

use crate::Length;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LengthSize {
    width_: Length,
    height_: Length,
}

impl LengthSize {
    pub fn new(width: &Length, height: &Length) -> Self {
        Self {
            width_: width.clone(),
            height_: height.clone(),
        }
    }
    pub fn SetWidth(&mut self, width: &Length) {
        self.width_ = width.clone();
    }
    pub fn Width(&self) -> &Length {
        &self.width_
    }
    pub fn SetHeight(&mut self, height: &Length) {
        self.height_ = height.clone();
    }
    pub fn Height(&self) -> &Length {
        &self.height_
    }
}
