#![allow(non_snake_case)]

//! Paint-order, phase, and cull decisions over caller-owned trees.
//!
//! This package classifies traversal and retained cull coverage. It does not
//! build property nodes, record paint operations, group chunks, or commit an
//! artifact.

pub mod cull;
pub mod paint_layer_painter;
pub mod paint_order;
pub mod phase;

pub use paint_model::{display_item_id, geometry_mapper, PaintRect};
pub use paint_model::{PaintPhase, PaintWalkMetrics};
pub use paint_record::{box_fragment_painter, paint_context, paint_info};
pub use prepaint::pre_paint_tree_walk;
