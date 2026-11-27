//! Completeness of a paint traversal relative to its cull rect.
//!
//! The value lives in the `paint_input_types` value root because Layout's
//! PaintLayer retains it as `previous_paint_result_` (Chromium
//! `core/paint/paint_layer.h:485-488,749-751`); Paint names the same type.

pub use paint_input_types::paint_result::{kMaxPaintResult, PaintResult};
