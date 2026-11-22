//! Layout invalidation vocabulary shared by LayoutNG algorithms and commit.
//!
//! A vocabulary crate below base: stable reason symbols, marking state,
//! tree-agnostic propagation over traits and the `layout_invalidation.*`
//! trace phases. It names no base type. Chromium `LayoutObject` mutation and
//! the engine's tree-update / node-input / mutation phases are base modules
//! (`layout_core/src/invalidation/`) that call this crate's trace phases, so
//! profiles still attribute structural work to invalidation.

pub mod content_change_type;
pub mod layout_invalidation_reason;
// Definitions of the `layout_invalidation_reason.h` extern arrays
// (layout_invalidation_reason.cc). They live beside the declarations so the
// reason symbols resolve inside this crate, not through a base definition.
mod layout_invalidation_reason_symbols;
pub mod layout_invalidation_state;
pub mod propagation;
pub mod trace;

pub use content_change_type::ContentChangeType;
pub use layout_invalidation_state::{LayoutInvalidationState, MarkingBehavior};
pub use propagation::{
    propagate_container_chain_for_layout, propagate_inside_multicol,
    propagate_intrinsic_widths_dirty, propagate_layout_paint_invalidation,
    propagate_may_contain_anchor, propagate_needs_collect_inlines, propagate_paint_invalidation,
    propagate_spanner_or_oof_parent_change, propagate_subtree_change_listener,
    AnchorInvalidationTree, ContainerChainInvalidationTree, InlineCollectionInvalidationTree,
    IntrinsicWidthInvalidationTree, MulticolDescendantTree, PaintInvalidationTree,
    SpannerParentInvalidationTree, SubtreeChangeListenerTree,
};
pub use trace::{span as trace_span, TracePhase};
