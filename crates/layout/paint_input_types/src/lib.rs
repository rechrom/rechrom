//! Plain paint value types crossing the Layout -> Paint boundary.
//!
//! These are the value declarations of Chromium's
//! `layoutng/internal/layout_input.h`, `layout_input_types.h`, the retained
//! paint input schema and the `layoutng_fragment_tree/fragment_tree.h`
//! resource values. They carry no LayoutObject, ComputedStyle reader or Layout
//! algorithm, so Paint, Raster, Compositor and Style name them without
//! depending on Layout. Layout re-exports every item from its original module
//! paths.

#![allow(non_snake_case)]

pub mod caret_shape;
pub mod layout_input;
pub mod layout_input_types;
pub mod paint_input;
pub mod paint_resources;
// Resolved SVG filter region/primitive geometry over the authored filter
// graph values (Blink SVGFilterBuilder / FilterEffect geometry): shared by
// SVG Layout, Paint property filters and Raster image filters.
pub mod svg_filter_geometry;

pub use caret_shape::CaretShape;
pub use paint_resources::{PaintGlyph, PaintGlyphRun, PaintResources};

/// A source physical ink rect, relative to the owning fragment's origin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FragmentPaintRect {
    pub offset: foundation::LayoutOffset,
    pub size: foundation::LayoutSize,
}
