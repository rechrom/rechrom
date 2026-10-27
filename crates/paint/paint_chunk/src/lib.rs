#![allow(non_snake_case)]

//! DisplayItem grouping policy.
//!
//! The package owns chunk-boundary and client metadata decisions. It does not
//! record operations or publish a PaintArtifact.

use layoutng_assembly::fragment_tree::FragmentPaintData;
use paint_model::display_item_id::DisplayItemIdType;

pub mod model;
pub mod paint_chunker;

pub use model::{
    PaintChunk, PaintChunkBackgroundColorInfo, PaintChunkCompositorData, PaintChunks,
    PaintFrameToken, PaintHitTestData, PaintHitTestOpaqueness, PaintLayerSelectionData,
    PaintSelectionBound, PaintSelectionBoundType, PaintTrackedElementRect,
};
pub use paint_chunker::PaintChunker;

#[derive(Clone, Copy, Debug)]
pub struct DisplayItemClientInfo {
    pub invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub scrollbar_needs_update_display: Option<bool>,
    pub is_cacheable: bool,
    pub is_just_created: bool,
}

impl DisplayItemClientInfo {
    pub fn from_fragment(fragment: &FragmentPaintData) -> Self {
        Self {
            invalidation_reason: fragment.paint.display_item_client_invalidation_reason,
            scrollbar_needs_update_display: None,
            is_cacheable: fragment.paint.display_item_client_is_cacheable,
            is_just_created: fragment.paint.display_item_client_is_just_created.get(),
        }
    }
}

pub fn ForcesNewChunk(item_type: DisplayItemIdType) -> bool {
    (DisplayItemIdType::kForeignLayerFirst.0..=DisplayItemIdType::kForeignLayerLast.0)
        .contains(&item_type.0)
        || item_type == DisplayItemIdType::kScrollbarHorizontal
        || item_type == DisplayItemIdType::kScrollbarVertical
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintChunkMetrics {
    pub chunks: usize,
    pub display_items: usize,
    pub max_items_per_chunk: usize,
}

pub fn MeasureChunks(ranges: impl IntoIterator<Item = (u32, u32)>) -> PaintChunkMetrics {
    let mut metrics = PaintChunkMetrics::default();
    for (begin, end) in ranges {
        metrics.chunks += 1;
        let size = end.saturating_sub(begin) as usize;
        metrics.display_items += size;
        metrics.max_items_per_chunk = metrics.max_items_per_chunk.max(size);
    }
    metrics
}
