// C++: src/foundation/blink_geometry/geometry/length_box.h:29-76
// cpp: foundation/blink_geometry/geometry/length_box.h:29-76

use crate::{Length, LengthType};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LengthBox {
    pub left_: Length,
    pub right_: Length,
    pub top_: Length,
    pub bottom_: Length,
}

impl LengthBox {
    pub fn from_type(type_: LengthType) -> Self {
        Self {
            left_: Length::from_type(type_),
            right_: Length::from_type(type_),
            top_: Length::from_type(type_),
            bottom_: Length::from_type(type_),
        }
    }

    pub fn from_int(value: i32) -> Self {
        Self {
            left_: Length::Fixed(value),
            right_: Length::Fixed(value),
            top_: Length::Fixed(value),
            bottom_: Length::Fixed(value),
        }
    }

    pub fn new(top: Length, right: Length, bottom: Length, left: Length) -> Self {
        Self {
            left_: left,
            right_: right,
            top_: top,
            bottom_: bottom,
        }
    }

    pub fn from_int_edges(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self::new(
            Length::Fixed(top),
            Length::Fixed(right),
            Length::Fixed(bottom),
            Length::Fixed(left),
        )
    }

    pub fn Left(&self) -> &Length {
        &self.left_
    }
    pub fn Right(&self) -> &Length {
        &self.right_
    }
    pub fn Top(&self) -> &Length {
        &self.top_
    }
    pub fn Bottom(&self) -> &Length {
        &self.bottom_
    }
    pub fn NonZero(&self) -> bool {
        !(self.left_.IsZero()
            && self.right_.IsZero()
            && self.top_.IsZero()
            && self.bottom_.IsZero())
    }
}
