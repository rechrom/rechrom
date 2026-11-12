//! Source: spin_button_element.cc:231-251. One synchronous native box query.
use crate::{PhysicalDirection, PhysicalSize, Point};
pub struct SpinButtonGeometry {
    pub local: Point,
    pub size: PhysicalSize,
    pub line_over: PhysicalDirection,
}
