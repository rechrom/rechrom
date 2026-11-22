//! Scroll snap areas: `SnapArea` is fragment data (`layout_fragment`) that
//! names its Element through an opaque handle; the typed accessors are this
//! base extension trait.

#![allow(non_snake_case)]

use super::layout_node_metadata::Element;

pub use layout_fragment::internal::snap_area::*;

/// Typed Element accessors of `SnapArea`.
pub trait SnapAreaExt {
    fn GetElementIfConsumed(&self) -> *mut Element;
    fn GetElement(&self) -> *mut Element;
}

impl SnapAreaExt for SnapArea {
    #[inline]
    fn GetElementIfConsumed(&self) -> *mut Element {
        self.GetElementHandleIfConsumed().cast()
    }

    #[inline]
    fn GetElement(&self) -> *mut Element {
        self.GetElementHandle().cast()
    }
}
