#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Immutable values shared by the Paint pipeline.
//!
//! This is the dependency root of the split Paint subsystem. It owns paint
//! phase identifiers, coordinate values, display-item identity, paint property trees,
//! paths, filters, and low-level paint value types. It does not walk layout,
//! decide reuse, record display items, group chunks, or publish artifacts.

pub mod boundary;
pub mod contoured_border_geometry;
pub mod cull;
pub mod cull_rect;
pub mod display_item_id;
pub mod geometry;
pub mod geometry_mapper;
pub mod opaque_paint_cache_slot;
pub mod paint_clip_path;
pub mod paint_flags;
pub mod paint_order;
pub mod paint_phase;
pub mod paint_property_tree;
pub mod paint_result;
pub mod path_hit_test;
pub mod property_filter_operations;
pub mod property_node_geometry;
pub mod stroke_data;
pub mod styled_stroke_data;
pub mod svg_filter_geometry;
pub mod viewport;

pub use boundary::{DisplayMaskLayer, ResolveDocumentImageContainerKey};
pub use cull_rect::CullRect;
pub use geometry::PaintRect;
pub use paint_flags::PaintFlags;
pub use paint_phase::PaintPhase;
pub use paint_result::{kMaxPaintResult, PaintResult};
pub use viewport::{
    PaintCompositorViewport, PaintCompositorViewportMode, PaintVisualViewportState,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintWalkMetrics {
    pub layer_visits: usize,
    pub phase_visits: usize,
    pub skipped_hidden: usize,
    pub skipped_phase: usize,
}

pub mod cc {
    pub mod paint {
        pub mod blend_mode;
        pub mod clear_for_opaque_raster;
        pub mod color;
        pub mod color_filter;
        pub mod discardable_image_map;
        pub mod draw_looper;
        pub mod filter_operation;
        pub mod filter_operations;
        pub mod paint_flags;
        pub mod paint_op;
        pub mod paint_op_buffer;
        pub mod paint_op_buffer_iterator;
        pub mod path_effect;
        pub mod render_surface_filters;
        pub mod sampling_options;
        pub mod solid_color_analyzer;
        pub mod tile_mode;
    }
}

pub mod geometry_mapper_source;
pub mod source_enum_set;
pub mod source_geometry;
pub mod source_input_geometry;
pub mod source_transform;

pub mod basic_shape_commands;
pub mod basic_shape_path;
pub mod style_shape_path;

pub mod border_shape_path;

pub mod contoured_rect;

pub mod paint_timing;
