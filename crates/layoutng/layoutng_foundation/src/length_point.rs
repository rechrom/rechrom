// C++: src/foundation/blink_geometry/geometry/length_point.h:35-60
// cpp: foundation/blink_geometry/geometry/length_point.h:35-60

use crate::Length;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LengthPoint {
    x_: Length,
    y_: Length,
}

impl LengthPoint {
    pub fn new(x: &Length, y: &Length) -> Self {
        Self {
            x_: x.clone(),
            y_: y.clone(),
        }
    }
    pub fn SetX(&mut self, x: &Length) {
        self.x_ = x.clone();
    }
    pub fn X(&self) -> &Length {
        &self.x_
    }
    pub fn SetY(&mut self, y: &Length) {
        self.y_ = y.clone();
    }
    pub fn Y(&self) -> &Length {
        &self.y_
    }
}
