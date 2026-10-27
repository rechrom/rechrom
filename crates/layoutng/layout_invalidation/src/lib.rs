//! Layout invalidation vocabulary shared by LayoutNG algorithms and commit.
//!
//! Stable reason symbols and tree-agnostic policy compile standalone. Native
//! `LayoutObject` mutation lives in assembly-only modules under this package,
//! so sampling profiles still attribute structural work to invalidation.

pub mod content_change_type;
pub mod layout_invalidation_reason;
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
