#![allow(non_camel_case_types, non_snake_case)]

//! Concrete PaintOp geometry and recording helpers.
//!
//! This package turns resolved paint inputs into backend-neutral operation
//! geometry. Tree traversal, reuse, chunk grouping, and artifact publication
//! remain separate stages.

pub use prepaint::background_geometry;
pub mod artifact;
pub mod box_fragment_painter;
pub mod caret_display_item_client;
pub mod display_item;
pub mod drawing_recorder;
pub mod fieldset_painter;
pub mod frame_set_painter;
pub mod mathml_painter;
pub use prepaint::nine_piece_image_grid;
pub mod nine_piece_image_painter;
pub mod paint_context;
pub mod paint_info;
pub mod paint_op;
pub mod scrollable_area_painter;
pub mod svg_shape_painter;
pub mod table_painters;
pub mod theme_painter;
pub mod wavy_geometry;

pub use artifact::{
    DocumentPaintArtifactRecord, ImageDependencyIndex, ImagePaintDependency, PaintMutation,
    PaintMutationResult, PaintRecording, PaintResourceUpdate,
};
pub use display_item::{
    CaretGeometry, CaretPosition, DisplayItem, DisplayItemList, DisplayMaskLayer,
    EditingPaintState, RecordedDisplayItem, RecordedDisplayItemKind, ScrollbarDisplayItemData,
};
pub use layoutng_assembly::fragment_tree::RasterEffectOutset;
pub use paint_model::{PaintPhase, PaintRect, ResolveDocumentImageContainerKey};
pub use paint_op::PaintOpType;
pub use paint_op::PaintOpType as DisplayItemType;

pub use paint_chunk::paint_chunker;
pub use paint_model::{border_shape_utils, display_item_id, geometry_mapper, paint_property_tree};
pub use prepaint::{paint_shader_resolver, pre_paint_tree_walk};

pub mod paint_engine {
    pub use crate::{
        CaretGeometry, CaretPosition, DisplayItem, DisplayItemType, EditingPaintState, PaintPhase,
        PaintRect, RasterEffectOutset, RecordedDisplayItem, RecordedDisplayItemKind,
        ResolveDocumentImageContainerKey, ScrollbarDisplayItemData,
    };
    pub use paint_chunk::{
        PaintChunkBackgroundColorInfo, PaintHitTestData, PaintHitTestOpaqueness,
    };
    pub use paint_model::DisplayMaskLayer;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintRecordMetrics {
    pub paint_ops: usize,
    pub display_items: usize,
}
