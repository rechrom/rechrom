#![allow(non_camel_case_types, non_snake_case)]

//! Concrete PaintOp geometry and recording helpers.
//!
//! This package turns resolved paint inputs into backend-neutral operation
//! geometry. Tree traversal, reuse, chunk grouping, and artifact publication
//! remain separate stages.

// layoutng_assembly contains the selected inline provider modules, but these
// two source virtual-dispatch entry points are otherwise referenced only by
// `extern "Rust"` declarations in a paint_record-only test link. Retain their
// provider objects for that link without adding a production dependency or
// moving either implementation out of its owning Layout subpackage.
#[cfg(test)]
#[used]
static TEST_LAYOUT_TEXT_COMBINE_PROVIDER: extern "Rust" fn(
    &layoutng_assembly::internal::layout_object::LayoutObject,
) -> bool = layoutng_assembly::layout_text_combine::DispatchIsLayoutTextCombine;
#[cfg(test)]
#[used]
static TEST_TEXT_OVERFLOW_PROVIDER: extern "Rust" fn(
    *const layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow,
) -> bool = layoutng_assembly::text_overflow_layout_service::LayoutBlockFlowShouldTruncateOverflowingTextFromInline;

use prepaint::{
    background_geometry, nine_piece_image_grid, paint_shader_resolver, pre_paint_tree_walk,
};

pub mod artifact;
mod blocking_wheel_event_regions;
pub mod box_border_painter;
pub mod box_decoration_data;
pub mod box_fragment_painter;
pub mod caret_display_item_client;
pub mod decorating_box {
    pub use layoutng_assembly::decorating_box::*;
}
pub mod decoration_line_painter;
pub mod display_item;
pub(crate) mod display_item_cache_skipper;
pub mod display_item_list;
pub mod drawing_display_item;
pub mod drawing_recorder;
pub mod fieldset_painter;
pub mod foreign_layer_display_item;
pub mod frame_set_painter;
pub mod html_canvas_painter;
pub mod inline_box_fragment_painter;
pub mod inline_paint_context {
    pub use layoutng_assembly::inline_paint_context::*;
}
pub mod ignore_paint_timing_scope;
pub mod line_relative_rect {
    pub use layoutng_assembly::line_relative_rect::*;
}
pub mod mathml_painter;
pub mod nine_piece_image_painter;
pub mod paint_auto_dark_mode;
pub mod paint_context;
pub mod paint_controller;
pub mod paint_info;
pub mod paint_op;
pub mod paint_under_invalidation_checker;
pub mod scoped_effectively_invisible;
mod scoped_paint_chunk_properties;
pub mod scoped_paint_state;
pub mod scoped_svg_paint_state;
pub mod scrollable_area_painter;
pub mod scrollbar_display_item;
mod selection_bounds_recorder;
pub mod subsequence_recorder;
pub mod svg_container_painter;
pub mod svg_mask_painter;
pub mod svg_model_object_painter;
pub mod svg_root_painter;
pub mod svg_shape_painter;
pub mod table_painters;
pub mod text_decoration_info {
    pub use layoutng_assembly::text_decoration_info::*;
}
pub mod text_decoration_painter;
pub mod theme_painter;
pub mod video_painter;
pub mod wavy_geometry;

pub use artifact::{
    DocumentPaintArtifactRecord, ImageDependencyIndex, ImagePaintDependency, PaintMutation,
    PaintMutationResult, PaintRecording, PaintResourceUpdate,
};
pub use display_item::{
    CaretGeometry, CaretPosition, DecorationPaintPass, DisplayItem, DisplayMaskLayer,
    EditingPaintState, RecordedDisplayItem, RecordedDisplayItemKind,
};
pub use display_item_list::{DisplayItemList, DisplayItemListJsonOption};
pub use foreign_layer_display_item::{
    ForeignLayer, ForeignLayerDisplayItemData, ForeignLayerSize, NewForeignLayerDisplayItem,
    RecordForeignLayer,
};
pub use layoutng_assembly::fragment_tree::RasterEffectOutset;
pub use paint_controller::{PaintController, PaintControllerPersistentData, SubsequencesData};
pub use paint_model::{PaintPhase, PaintRect, ResolveDocumentImageContainerKey};
pub use paint_op::PaintOpType;
pub use paint_op::PaintOpType as DisplayItemType;
pub use paint_under_invalidation_checker::{PaintUnderInvalidationChecker, SubsequenceMarkers};
pub use scrollbar_display_item::ScrollbarDisplayItemData;
pub use subsequence_recorder::SubsequenceRecorder;

pub use paint_chunk::paint_chunker;
pub use paint_model::{
    contoured_border_geometry, display_item_id, geometry_mapper, paint_property_tree,
};
pub mod paint_engine {
    pub use crate::{
        CaretGeometry, CaretPosition, DecorationPaintPass, DisplayItem, DisplayItemType,
        EditingPaintState, ForeignLayer, ForeignLayerDisplayItemData, ForeignLayerSize, PaintPhase,
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

pub mod svg_context_paints;
