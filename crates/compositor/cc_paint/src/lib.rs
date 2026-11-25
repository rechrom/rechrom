#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

//! cc/paint: the recorded paint values (PaintRecord, PaintOpBuffer,
//! PaintFlags, DisplayItemList and its op facets, image and worklet values)
//! and the Skia value backend they are generic over. Chromium's cc/paint is
//! its own component below cc and viz (cc/paint/BUILD.gn): Blink records into
//! it on the main thread, cc rasters and decodes it on the impl side, and
//! viz's PictureDrawQuad carries its DisplayItemList. layerize::commit::paint
//! re-exports it as commit vocabulary; producer conversions from the
//! PaintArtifact stay with Paint and layerize.

use std::{ops::Range, sync::Arc};

pub use paint_model::display_item::DisplayItem as PaintOp;

pub mod geometry {
    //! gfx geometry plus the Skia path facet cc/paint reads.
    pub use crate::skia_backend::Path;
    pub use compositor_tree::geometry::*;
}
pub mod document_image_record;
pub use document_image_record::DocumentImagePaintRecord;
pub mod rtree;
pub mod skia_backend;
pub mod skia_region;

pub mod discardable_image_map;
pub mod display_item_list;
pub mod display_item_list_types;
pub mod image_metadata;
pub mod image_provider;
pub use image_provider::*;
mod image_values;
pub use image_values::CpuPaintImageInputError;
mod lowering;
pub use lowering::{CpuPaintColorFilter, CpuPaintShader, LowerPaintRecord, PaintSource};
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
    resources: Option<Arc<paint_input_types::PaintResources>>,
    // Resource pixels may change without replacing `resources`; the sidecar
    // is replaced on every such change and keeps SameSource exact.
    content_revisions: Arc<Vec<Vec<(u64, u64)>>>,
    range: Range<usize>,
}

impl PaintRecord {
    /// A lease over `range` of a producer's shared op allocation, with the
    /// resource catalog and resource-revision sidecar those ops sample. The
    /// producer (Paint's artifact, layerize's per-item records) passes its
    /// existing Arcs; nothing is copied. None when `range` is out of bounds.
    pub fn new(
        items: Arc<Vec<PaintOp>>,
        resources: Option<Arc<paint_input_types::PaintResources>>,
        content_revisions: Arc<Vec<Vec<(u64, u64)>>>,
        range: Range<usize>,
    ) -> Option<Self> {
        items.get(range.clone())?;
        Some(Self { items, resources, content_revisions, range })
    }
    /// The producer's immutable resource catalog sampled by these operations.
    pub fn resources(&self) -> Option<&Arc<paint_input_types::PaintResources>> {
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
