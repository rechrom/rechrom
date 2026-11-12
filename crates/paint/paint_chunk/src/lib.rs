#![allow(non_snake_case)]

//! DisplayItem grouping policy.
//!
//! The package owns chunk-boundary and client metadata decisions. It does not
//! record operations or publish a PaintArtifact.

use layoutng_assembly::fragment_tree::FragmentPaintData;
use paint_model::display_item_id::DisplayItemIdType;

pub mod hit_test_data;
pub mod layer_selection_data;
pub mod model;
pub mod paint_chunker;
pub mod painted_selection_bound;
pub mod region_capture_data;
pub mod tracked_element_data;

pub use hit_test_data::{NullableHitTestDataToString, PaintHitTestData};
pub use layer_selection_data::PaintLayerSelectionData;
pub use model::{
    PaintChunk, PaintChunkBackgroundColorInfo, PaintChunkCompositorData, PaintChunks,
    PaintHitTestOpaqueness, K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO,
};
pub use paint_chunker::PaintChunker;
pub use painted_selection_bound::{PaintSelectionBound, PaintSelectionBoundType};
pub use region_capture_data::{PaintRegionCaptureData, RegionCaptureDataToString};
pub use tracked_element_data::{
    PaintFrameToken, PaintTrackedElementRect, PaintTrackedElementRects,
    PaintTrackedElementRelativeRect, PaintTrackedElementSubRect, PaintTrackedElementSubRectType,
    TrackedElementRectsToString,
};

#[derive(Clone, Copy, Debug)]
pub struct DisplayItemClientInfo {
    pub invalidation_reason: Option<foundation::PaintInvalidationReason>,
    pub is_cacheable: bool,
    pub is_just_created: bool,
}

impl DisplayItemClientInfo {
    pub fn from_fragment(fragment: &FragmentPaintData) -> Self {
        Self {
            invalidation_reason: fragment.paint.display_item_client_invalidation_reason,
            is_cacheable: fragment.paint.display_item_client_is_cacheable,
            is_just_created: fragment.paint.display_item_client_is_just_created.get(),
        }
    }

    pub fn from_paint_layer(fragment: &FragmentPaintData) -> Option<Self> {
        (fragment.paint.paint_layer_client_id != 0).then(|| Self {
            invalidation_reason: fragment.paint.paint_layer_client_invalidation_reason,
            is_cacheable: fragment.paint.paint_layer_client_is_cacheable,
            is_just_created: fragment.paint.paint_layer_client_is_just_created.get(),
        })
    }

    pub fn from_scrolling_background(fragment: &FragmentPaintData) -> Option<Self> {
        let scroll = fragment.paint.scroll_container.as_ref()?;
        (scroll.scrolling_background_client_id != 0).then(|| Self {
            invalidation_reason: scroll.scrolling_background_client_invalidation_reason,
            is_cacheable: scroll.scrolling_background_client_is_cacheable,
            is_just_created: scroll.scrolling_background_client_is_just_created.get(),
        })
    }

    pub fn from_selection(fragment: &FragmentPaintData) -> Option<Self> {
        (fragment.paint.selection_client_id != 0).then(|| Self {
            invalidation_reason: fragment.paint.selection_client_invalidation_reason,
            is_cacheable: fragment.paint.selection_client_is_cacheable,
            is_just_created: fragment.paint.selection_client_is_just_created.get(),
        })
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
