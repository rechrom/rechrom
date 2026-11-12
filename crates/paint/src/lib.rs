//! The initial static block slice of //src/paint. Unsupported paint features
//! fail explicitly until their source painters are translated.

pub use paint_chunk::paint_chunker;
pub use paint_model::display_item_id;
pub use paint_model::opaque_paint_cache_slot;
pub use paint_model::paint_clip_path;
pub use paint_model::path_hit_test;
pub use paint_model::{basic_shape_path, contoured_border_geometry, contoured_rect};
pub use paint_model::{geometry_mapper, geometry_mapper_source};
pub(crate) use paint_record::paint_context;
pub use prepaint::background_geometry;
pub(crate) use prepaint::client_rects;
pub mod layout_object_counter;
pub mod paint_engine;
pub mod paint_event;
pub mod paint_timing;
pub(crate) use paint_walk::paint_layer_painter;
pub mod performance;
pub use paint_commit::scroll_paint_update;
pub use paint_model::paint_order;
pub use paint_model::paint_property_tree;
pub use paint_model::property_filter_operations;
pub use paint_model::svg_filter_geometry;
pub use paint_model::{
    PaintCompositorViewport, PaintCompositorViewportMode, PaintVisualViewportState,
};
pub(crate) use prepaint::pre_paint_tree_walk;
pub use prepaint::{clip_path_clipper, resident_hit_test, resident_layer_clipper};

pub use layout_object_counter::LayoutObjectCounter;
pub use paint_engine::PaintRect;
pub use paint_engine::{ForeignLayer, LocalFrameViewPaintInput, PaintArtifact, PaintEngine};
pub use paint_event::PaintEvent;
pub use performance::PaintPerformanceSnapshot;

pub use paint_model::cc;
pub use paint_model::property_node_geometry;
pub use paint_record::paint_auto_dark_mode;
pub use paint_record::{PaintController, PaintControllerPersistentData};
pub use prepaint::{
    BlockPaintInvalidator, BoxPaintInvalidator, CompositingReasonFinder, CullRectUpdater,
    ObjectPaintInvalidator, PaintInvalidator, PaintInvalidatorContext, TableCellPaintInvalidator,
};

/// Selected DecorationLinePainter pure source value dependency.
pub use paint_record::wavy_geometry;

pub use paint_engine::drag_image;
