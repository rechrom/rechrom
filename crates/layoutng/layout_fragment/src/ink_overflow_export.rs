//! Physical rect conversion for fragment paint geometry.
//!
//! Box and inline ink overflow are read live from the resident
//! PhysicalBoxFragment / FragmentItem (ResidentBoxGeometry::ViewInkOverflows,
//! ViewTextGlyphInk); nothing here stores ink overflow.

#![allow(non_snake_case)]

use super::*;

pub(super) fn ExportInkRect(rect: PhysicalRect) -> FragmentPaintRect {
    FragmentPaintRect {
        offset: Offset {
            x: Number(rect.offset.left),
            y: Number(rect.offset.top),
        },
        size: Size {
            width: Number(rect.size.width),
            height: Number(rect.size.height),
        },
    }
}
