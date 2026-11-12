#![allow(non_snake_case)]

//! Paint-order and phase traversal over caller-owned PrePaint trees.
//!
//! This package classifies recording traversal and paint phases. It does not
//! build property nodes, record paint operations, group chunks, or commit an
//! artifact.

mod paint_layer_paint_order_iterator;
pub use paint_layer_paint_order_iterator::EnsureResidentHitTestLayerOrder;
pub mod paint_layer_painter;
pub mod phase;

use paint_record::{
    box_fragment_painter, ignore_paint_timing_scope, paint_context, paint_info,
    scoped_effectively_invisible, scoped_svg_paint_state, svg_container_painter,
    svg_model_object_painter, svg_root_painter,
};
use prepaint::pre_paint_tree_walk;

pub use paint_model::{display_item_id, geometry_mapper, PaintRect};
pub use paint_model::{PaintPhase, PaintWalkMetrics};
