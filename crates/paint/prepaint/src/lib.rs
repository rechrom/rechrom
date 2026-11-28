#![allow(non_snake_case)]

//! PrePaint property, coordinate, cull, and invalidation lifecycle.
//!
//! Shared property values remain owned by `paint_model`; this package owns
//! `PrePaintTreeWalk`, property-tree construction, `CullRectUpdater`, and paint
//! invalidation. Recording traversal stays in `paint_walk`, and equality
//! proofs stay in `paint_reuse`.

pub use paint_model::paint_property_tree;
pub use paint_model::property_node_geometry;
pub use paint_reuse::pre_paint_state_compare;

pub mod background_geometry;
pub mod block_paint_invalidator;
pub mod box_paint_invalidator;
pub mod client_rects;
pub mod committed_paint_clients;
pub mod clip_path_clipper;
pub mod compositing_reason_finder;
pub mod contoured_border_geometry;
pub mod cull_rect_updater;
pub mod fragment_data;
pub mod nine_piece_geometry;
pub mod nine_piece_image_grid;
pub mod object_paint_invalidator;
pub mod object_paint_properties;
pub mod paint_invalidator;
// PaintLayer classification of a Layout fragment (core/paint/paint_layer.cc).
pub mod paint_order;
pub mod paint_shader_resolver;
pub mod pre_paint_tree_walk;
pub mod resident_hit_test;
pub mod resident_layer_clipper;
pub mod special_paint;
pub mod table_cell_paint_invalidator;

pub use block_paint_invalidator::BlockPaintInvalidator;
pub use box_paint_invalidator::BoxPaintInvalidator;
pub use compositing_reason_finder::CompositingReasonFinder;
pub use cull_rect_updater::CullRectUpdater;
pub use object_paint_invalidator::ObjectPaintInvalidator;
pub use paint_invalidator::{PaintInvalidator, PaintInvalidatorContext};
pub use pre_paint_tree_walk::{
    CompositedAnimationProperty, CompositedAnimationPropertyBinding,
    DirectCompositedAnimationPropertyUpdate, PaintFragmentSource, PaintTreeNode, PrePaintTreeWalk,
};
pub use table_cell_paint_invalidator::TableCellPaintInvalidator;

#[derive(Clone, Copy, Debug, Default)]
pub struct PrePaintUpdateStats {
    /// Objects PaintPropertyTreeBuilder built (visited) this pass.
    pub computed_nodes: usize,
    /// Subtrees the walk returned from before building a context for their
    /// root (pre_paint_tree_walk.cc:1477-1482).
    pub skipped_subtrees: usize,
    pub invalidation_nodes: usize,
    /// Visited objects that needed no property update: the walk advanced
    /// their children's context from their FragmentData without a build
    /// (paint_property_tree_builder.cc, !NeedsPaintPropertyUpdate).
    pub clean_visits: usize,
    /// Deferred transform updates applied in place before the walk
    /// (PaintPropertyTreeBuilder::DirectlyUpdateTransformMatrix).
    pub direct_transform_updates: usize,
    /// A post-walk pass (logical out-of-flow effect fix-ups, inline outline
    /// groups) or a direct transform update rewrote FragmentData outside the
    /// walk.
    pub post_walk: bool,
    /// The walk's source is a replacement LayoutResult of the committed
    /// Paint publication state.
    pub source_result_replaced: bool,
    /// Objects whose cull-rect change forced a layer repaint this pass.
    pub cull_change_repaints: usize,
    /// Walk self time by section, excluding child recursion. Recorded only
    /// while tracing is enabled.
    pub profile_admission_ns: u64,
    pub profile_build_ns: u64,
    pub profile_invalidation_ns: u64,
    pub profile_children_setup_ns: u64,
    pub profile_child_loop_ns: u64,
    pub profile_tail_ns: u64,
}
