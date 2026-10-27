//! Compatibility facade for layout-reuse admission and inlineable helpers.

#[path = "../../../layout_reuse/src/native_layout_utils.rs"]
mod layout_reuse_native_layout_utils;
pub use super::layout_alignment_utils::{
    BlockStaticPositionEdge, ComputeContentAlignmentForTableCell, InlineStaticPositionEdge,
    ResolveContentAlignment,
};
pub use layout_reuse_native_layout_utils::{
    CalculateSizeBasedLayoutCacheStatus, LayoutCacheStatus,
    MaySkipLayoutWithinBlockFormattingContext,
};
