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

pub use caret_shape::CaretShape;
pub use paint_resources::{PaintGlyph, PaintGlyphRun, PaintResources};
