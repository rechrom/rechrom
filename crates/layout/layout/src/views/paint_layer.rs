//! PaintLayer family: the Layout-owned PaintLayer and its stacking node, as
//! PrePaint, Paint and hit testing read and write them.
//!
//! Chromium keeps PaintLayer in core/paint but its owner is the
//! LayoutBoxModelObject (layout_box_model_object.cc:347-367), and every paint
//! stage touches the same object:
//! - PrePaint's CullRectUpdater reads/sets/clears the cull-rect bits
//!   (cull_rect_updater.cc; paint_layer.h:462-480) and paint invalidation sets
//!   the paint-phase bits (paint_layer.h:490-507).
//! - PaintLayerPainter writes `PreviousPaintResult` (paint_layer.h:481-488,
//!   paint_layer.cc:2827-2832) and clears repaint bits
//!   (`ClearNeedsRepaintRecursively`, paint_layer.cc:2749-2785).
//! - PaintLayerPaintOrderIterator reads the stacking node's z-order lists and
//!   overlay overflow-control map (paint_layer_paint_order_iterator.cc:39-158,
//!   paint_layer_stacking_node.h:108-124) for painting and hit testing.
//!
//! The layer for a painted object is reached through the borrowed fragment
//! cursor (`PaintFragmentCursor::LayerForPaintObject`). Writable state is
//! sequence-confined interior state, so these shared borrows suffice; the
//! stacking node's lists are rebuilt only by Layout's
//! `PaintLayer::UpdateDescendantDependentFlags` (paint_layer.cc:480,864-882).

pub use layout_base::internal::paint_layer::{PaintLayer, PaintLayerStackingNode, PaintLayers};
/// Source core/paint/paint_result.h:12-25, retained as
/// `PaintLayer::PreviousPaintResult`.
pub use paint_input_types::paint_result::PaintResult;
