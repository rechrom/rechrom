#![allow(non_snake_case)]

use crate::{PhysicalOffset, PhysicalSize};
use foundation_base::gfx_geometry::{Point, Size};
use std::borrow::Borrow;

// C++ takes const references; accept either an owned Rust value or a borrow
// so translated call sites retain their expression order.
// cpp: foundation/blink_geometry/geometry/physical_offset.h:110-115
pub fn ToFlooredPoint(offset: impl Borrow<PhysicalOffset>) -> Point {
    foundation_base::blink_geometry::geometry::physical_offset::ToFlooredPoint(offset.borrow())
}

// cpp: foundation/blink_geometry/geometry/physical_size.h:130-133
pub fn ToRoundedSize(size: impl Borrow<PhysicalSize>) -> Size {
    foundation_base::blink_geometry::geometry::physical_size::ToRoundedSize(size.borrow())
}

// cpp: foundation/blink_geometry/geometry/physical_size.h:134-136
pub fn ToFlooredSize(size: impl Borrow<PhysicalSize>) -> Size {
    foundation_base::blink_geometry::geometry::physical_size::ToFlooredSize(size.borrow())
}
