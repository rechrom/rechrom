//! The initial static block slice of //src/paint. Unsupported paint features
//! fail explicitly until their source painters are translated.

pub use paint_chunk::paint_chunker;
pub use paint_model::border_shape_utils;
pub use paint_model::display_item_id;
pub use paint_model::geometry_mapper;
pub use paint_model::opaque_paint_cache_slot;
pub use paint_model::paint_clip_path;
pub(crate) use paint_record::paint_context;
pub use prepaint::background_geometry;
#[cfg(feature = "translation_in_progress")]
pub(crate) use prepaint::client_rects;
pub mod paint_engine;
#[cfg(feature = "translation_in_progress")]
pub(crate) use paint_walk::paint_layer_painter;
pub mod performance;
#[cfg(feature = "translation_in_progress")]
pub use paint_commit::scroll_paint_update;
pub use paint_model::paint_property_tree;
pub use paint_model::property_filter_operations;
pub use paint_walk::paint_order;
pub(crate) use prepaint::pre_paint_tree_walk;

pub use paint_engine::PaintRect;
#[cfg(feature = "translation_in_progress")]
pub use paint_engine::{PaintArtifact, PaintEngine};
pub use performance::PaintPerformanceSnapshot;

pub use paint_model::cc;
pub use paint_model::property_node_geometry;

/// Selected DecorationLinePainter pure native value dependency.
pub use paint_record::wavy_geometry;
