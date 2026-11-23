//! cc/paint: the recorded paint values (PaintRecord, PaintOpBuffer,
//! PaintFlags, DisplayItemList and its op facets, image and worklet values).
//! Chromium uses cc/paint on the main thread (recording, PAC) and on the impl
//! thread (raster, tiles, image decode), so it is commit vocabulary. Bindings
//! to impl-only traits stay in the compositor.

use std::{ops::Range, sync::Arc};

pub use paint::paint_engine::{
    DisplayItem as PaintOp, PaintArtifact, RecordedDisplayItem as DisplayItem,
};

pub mod discardable_image_map;
pub mod display_item_list;
pub mod display_item_list_types;
pub mod image_metadata;
pub mod image_provider;
pub use image_provider::*;
mod image_values;
pub use image_values::CpuPaintImageInputError;
mod lowering;
pub use lowering::{CpuPaintColorFilter, CpuPaintShader, PaintSource};
pub mod metadata;
pub mod node_info;
mod node_id;
// Canvas child raster input published by the main half and replayed by the
// raster worker (moved from compositor/src/runtime/canvas_child_record.rs).
pub mod canvas_child_record;
pub use node_id::NodeId;
mod picture;
pub use picture::*;
mod record;
pub use record::CpuPaintImage as PaintImage;
pub use record::*;
mod solid_color_analyzer;
pub use solid_color_analyzer::{CpuPaintAnalysisTrace, CpuSolidColorBackend};
mod worklet;
pub use worklet::*;

/// An immutable record lease over exactly the producer's resident replay range.
///
/// The lease retains only the artifact's shared replay operations, the
/// image/font resources those operations sample and the resource-content
/// revision sidecar, never the whole `PaintArtifact`. Blink's cc::PaintRecord
/// likewise owns only its PaintOpBuffer (and the PaintImages inside it); the
/// PaintChunk property-tree states that produced it stay with the main-thread
/// PaintArtifact. Raster and impl-side owners receive these leases, so Blink
/// paint property nodes stay on the Page document sequence.
/// Source: cc/paint/paint_record.h (PaintRecord wraps sk_sp<PaintOpBuffer>);
///         third_party/blink/renderer/platform/graphics/compositing/
///         paint_chunks_to_cc_layer.cc (records receive draw ops only).
#[derive(Clone)]
pub struct PaintRecord {
    items: Arc<Vec<PaintOp>>,
    resources: Option<Arc<paint::paint_engine::PaintResources>>,
    // Resource pixels may change without replacing `resources`; the sidecar
    // is replaced on every such change and keeps SameSource exact.
    content_revisions: Arc<Vec<Vec<(u64, u64)>>>,
    range: Range<usize>,
}

impl PaintRecord {
    pub fn from_artifact(artifact: Arc<PaintArtifact>) -> Self {
        let end = artifact.items.len();
        Self {
            items: Arc::clone(&artifact.items),
            resources: artifact.resources.clone(),
            content_revisions: Arc::clone(&artifact.image_content_revisions),
            range: 0..end,
        }
    }
    pub fn from_item(artifact: &Arc<PaintArtifact>, item: &DisplayItem) -> Option<Self> {
        let range = item.record_begin..item.record_end;
        artifact.items.get(range.clone())?;
        Some(Self {
            items: Arc::clone(&artifact.items),
            resources: artifact.resources.clone(),
            content_revisions: Arc::clone(&artifact.image_content_revisions),
            range,
        })
    }
    /// The producer's immutable resource catalog sampled by these operations.
    pub fn resources(&self) -> Option<&Arc<paint::paint_engine::PaintResources>> {
        self.resources.as_ref()
    }
    pub fn ops(&self) -> &[PaintOp] {
        &self.items[self.range.clone()]
    }
    /// The same immutable replay range over the same operation, resource and
    /// resource-revision allocations. A property-only artifact replacement
    /// retains all three, so its records replay identically.
    pub fn SameSource(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.items, &other.items)
            && self.range == other.range
            && Arc::ptr_eq(&self.content_revisions, &other.content_revisions)
            && match (&self.resources, &other.resources) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }
    pub fn len(&self) -> usize {
        self.range.len()
    }
}
