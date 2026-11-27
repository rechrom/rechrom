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
    pub computed_nodes: usize,
    pub reused_nodes: usize,
    pub invalidation_only_nodes: usize,
    pub same_geometry_rebuilds: usize,
    pub property_geometry_changes: usize,
    pub property_geometry_changes_with_own_keys: usize,
    pub property_geometry_changes_needing_property_update: usize,
    /// Of the property geometry changes, those local to their object, which
    /// kept the retained post-update results instead of forcing the
    /// whole-tree post-update pass.
    pub local_geometry_changes: usize,
    pub skipped_subtrees: usize,
    pub same_context_nodes: usize,
    pub retained_context_nodes: usize,
    pub retained_geometry_covered_nodes: usize,
    /// Retained nodes whose own inputs are unchanged: no own dirty bit, same
    /// link offset and size, same source identity and live clients, same
    /// document resources for their usage.
    pub self_unchanged_nodes: usize,
    /// Of those, nodes whose descendants are also unchanged: no descendant
    /// dirty bit, no pending child walk, same resources for the subtree.
    pub subtree_unchanged_nodes: usize,
    /// Retained nodes rejected by the first failing admission rule: own
    /// bits, offset, size, identity, owners, resources; then for the
    /// subtree: descendant bits, pending child walk, subtree resources.
    pub admission_rejects: [usize; 9],
    pub source_tree_compatible_nodes: usize,
    pub invalidation_nodes: usize,
    pub post_walk: bool,
    pub rebound_nodes: usize,
    pub rebind_walk: bool,
    /// The walk's source is a replacement LayoutResult of the retained Paint
    /// publication state (no rebind walk; clean reused subtrees are skipped).
    pub source_result_replaced: bool,
    /// Nodes whose cull-rect change forced a layer repaint this pass.
    pub cull_change_repaints: usize,
    /// UpdateNode self time by section, excluding child recursion. Recorded
    /// only while tracing is enabled.
    pub profile_admission_ns: u64,
    pub profile_build_ns: u64,
    pub profile_invalidation_ns: u64,
    pub profile_children_setup_ns: u64,
    pub profile_child_loop_ns: u64,
    pub profile_tail_ns: u64,
    /// Nodes updated by shifting their retained geometry (offset-only).
    pub shifted_nodes: usize,
    /// Verification mode: shifted nodes that differed from a rebuild.
    pub shift_check_mismatches: usize,
    /// Rebuilt nodes by the first reason they could not be shifted (trace
    /// only): no basis, own inputs changed, own property nodes, identity,
    /// paint layer, SVG, own clip/cull/transform, other context change.
    pub shift_rejects: [usize; 8],
    /// Build-section time and count of rebuilt nodes by what they own
    /// (trace only): transform roles, clip roles only, effect roles only,
    /// SVG, paint layer without own nodes, plain.
    pub build_kind_ns: [u64; 6],
    pub build_kind_nodes: [usize; 6],
    /// Trace only: time in the per-node shift branch of UpdateNode.
    pub shift_branch_ns: u64,
}
