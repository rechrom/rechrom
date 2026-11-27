//! Compatibility facade for fragment-owned PaintLayer identity.

#![allow(non_snake_case)]

mod layout_fragment_paint_layer;
// Source: core/paint/paint_layer_stacking_node.{h,cc}: z-order lists owned by
// stacking-context PaintLayers.
mod paint_layer_stacking_node;

pub use layout_fragment_paint_layer::*;
pub use paint_layer_stacking_node::{PaintLayerStackingNode, PaintLayers};
