// Translation status: complete
// Copyright 2016 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidation_tracking.h:1-120.
// Source: third_party/blink/renderer/platform/graphics/paint/raster_invalidation_tracking.cc:1-250.
// Includes, namespaces, Oilpan allocation and Visitor tracing map to Rust
// modules, ordinary ownership and generic external Paint/bitmap capabilities.
// The invalidation ordering, region growth and pixel comparison remain here.

use crate::{
    adapter::{geometry::Rect, region_backend::SkRegionBackend},
    cc::{base::region::Region, layers::layer::LayerDebugInfo},
};
use foundation::{
    graphics_types::graphics::paint::display_item_client_types::DisplayItemClientId,
    PaintInvalidationReason,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
};

// Source: raster_invalidation_tracking.h:27-47.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterInvalidationInfo {
    // Comparison-only identity. It is never dereferenced after the client dies.
    pub client_id: DisplayItemClientId,
    pub client_debug_name: String,
    pub rect: Rect,
    pub reason: PaintInvalidationReason,
}

impl Default for RasterInvalidationInfo {
    fn default() -> Self {
        Self {
            client_id: 0,
            client_debug_name: String::new(),
            rect: Rect::default(),
            reason: PaintInvalidationReason::kLayout,
        }
    }
}

impl std::fmt::Display for RasterInvalidationInfo {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "{}:{} rect={} reason={}",
            self.client_id,
            self.client_debug_name,
            self.rect.ToString(),
            PaintInvalidationReasonToString(self.reason)
        )
    }
}

// Source: raster_invalidation_tracking.h:49-57.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterUnderInvalidation {
    pub x: i32,
    pub y: i32,
    // Source TODO crbug.com/1351544: these remain packed SkColor values.
    pub old_pixel: u32,
    pub new_pixel: u32,
}

/// External bitmap allocation and PaintRecord playback only. Row comparison
/// and clearing preserve the source fast path without exposing pixel storage.
pub trait RasterInvalidationBitmap {
    fn RowsEqual(&self, other: &Self, y: i32) -> bool;
    fn ClearRow(&mut self, y: i32);
    fn Pixel(&self, x: i32, y: i32) -> u32;
    fn SetPixel(&mut self, x: i32, y: i32, color: u32);
}

/// The JSON owner receives already source-sorted and source-filtered values.
/// It owns JSON allocation and color formatting, not invalidation policy.
pub trait RasterInvalidationJson {
    fn SetInvalidations(&mut self, detailed: bool, values: &[RasterInvalidationInfo]);
    fn SetUnderInvalidations(&mut self, values: &[RasterUnderInvalidation]);
}

/// Static native Paint/trace feature entry points. This is a type-level value
/// backend and retains no compositor, Page, layer, tree or service pointer.
pub trait RasterInvalidationTrackingRuntime<T: RasterInvalidationTrackingTypes> {
    fn PaintUnderInvalidationCheckingEnabled(&self) -> bool;
    fn IsTracingRasterInvalidations(&self) -> bool;
    fn Rasterize(&self, record: &T::PaintRecord, rect: Rect) -> Option<T::Bitmap>;
    fn BuildUnderInvalidationRecord(
        &self,
        previous: T::PaintRecord,
        overlay: T::Bitmap,
        rect: Rect,
    ) -> T::PaintRecord;
    fn LogUnderInvalidation(&self, layer_debug_name: &str, x: i32, y: i32, old_pixel: u32, new_pixel: u32);
    fn LogMoreUnderInvalidations(&self);
}

pub trait RasterInvalidationTrackingTypes: Sized {
    type RegionBackend: SkRegionBackend;
    type PaintRecord: Clone + Default;
    type Bitmap: RasterInvalidationBitmap;
    type Runtime: RasterInvalidationTrackingRuntime<Self>;
}

static SIMULATE_RASTER_UNDER_INVALIDATIONS: AtomicBool = AtomicBool::new(false);

// Source: raster_invalidation_tracking.h:59-119.
pub struct RasterInvalidationTracking<B: RasterInvalidationTrackingTypes> {
    invalidations_: Vec<RasterInvalidationInfo>,
    last_painted_record_: Option<B::PaintRecord>,
    last_interest_rect_: Rect,
    invalidation_region_since_last_paint_: Region<B::RegionBackend>,
    under_invalidations_: Vec<RasterUnderInvalidation>,
    under_invalidation_record_: B::PaintRecord,
    runtime_: B::Runtime,
}

impl<B: RasterInvalidationTrackingTypes> Default for RasterInvalidationTracking<B> where B::Runtime: Default {
    fn default() -> Self { Self::new(B::Runtime::default()) }
}
impl<B: RasterInvalidationTrackingTypes> RasterInvalidationTracking<B> {
    /// Independent injected CPU/value capability; no owner or registry.
    pub fn new(runtime: B::Runtime) -> Self {
        Self {
            invalidations_: Vec::new(),
            last_painted_record_: None,
            last_interest_rect_: Rect::default(),
            invalidation_region_since_last_paint_: Region::default(),
            under_invalidations_: Vec::new(),
            under_invalidation_record_: B::PaintRecord::default(),
            runtime_: runtime,
        }
    }
}

impl<B: RasterInvalidationTrackingTypes> RasterInvalidationTracking<B> {
    // Source: raster_invalidation_tracking.cc:24-26.
    pub fn SimulateRasterUnderInvalidations(enable: bool) {
        SIMULATE_RASTER_UNDER_INVALIDATIONS.store(enable, Ordering::Relaxed);
    }

    // Source: raster_invalidation_tracking.cc:28-32.
    pub fn ShouldAlwaysTrack(runtime: &B::Runtime) -> bool {
        runtime.PaintUnderInvalidationCheckingEnabled() || Self::IsTracingRasterInvalidations(runtime)
    }

    // Source: raster_invalidation_tracking.cc:34-38.
    pub fn IsTracingRasterInvalidations(runtime: &B::Runtime) -> bool {
        runtime.IsTracingRasterInvalidations()
    }

    // Source: raster_invalidation_tracking.cc:40-62.
    pub fn AddInvalidation(
        &mut self,
        client_id: DisplayItemClientId,
        debug_name: &str,
        rect: Rect,
        reason: PaintInvalidationReason,
    ) {
        if rect.IsEmpty() {
            return;
        }
        self.invalidations_.push(RasterInvalidationInfo {
            client_id,
            client_debug_name: debug_name.to_owned(),
            rect,
            reason,
        });

        // Some antialiasing effects overflow the invalidation rectangle.
        let expanded = Rect::FromBounds(
            rect.x.saturating_sub(1),
            rect.y.saturating_sub(1),
            rect.right().saturating_add(1),
            rect.bottom().saturating_add(1),
        );
        self.invalidation_region_since_last_paint_.Union(&expanded);
    }

    pub fn HasInvalidations(&self) -> bool {
        !self.invalidations_.is_empty()
    }
    pub fn Invalidations(&self) -> &[RasterInvalidationInfo] {
        &self.invalidations_
    }
    pub fn ClearInvalidations(&mut self) {
        self.invalidations_.clear();
    }

    // Source: raster_invalidation_tracking.cc:64-84.
    fn CompareRasterInvalidationInfo(
        first: &RasterInvalidationInfo,
        second: &RasterInvalidationInfo,
    ) -> std::cmp::Ordering {
        second
            .rect
            .width
            .cmp(&first.rect.width)
            .then_with(|| second.rect.height.cmp(&first.rect.height))
            .then_with(|| second.rect.x.cmp(&first.rect.x))
            .then_with(|| second.rect.y.cmp(&first.rect.y))
            .then_with(|| first.client_debug_name.cmp(&second.client_debug_name))
            .then_with(|| first.reason.cmp(&second.reason))
    }

    // Source: raster_invalidation_tracking.cc:86-132.
    pub fn AsJSON(&self, json: &mut impl RasterInvalidationJson, detailed: bool) {
        if !self.invalidations_.is_empty() {
            let mut sorted = self.invalidations_.clone();
            sorted.sort_by(Self::CompareRasterInvalidationInfo);
            if detailed {
                json.SetInvalidations(true, &sorted);
            } else {
                let mut visible = Vec::new();
                for info in sorted {
                    if visible
                        .iter()
                        .all(|previous: &RasterInvalidationInfo| !previous.rect.Contains(info.rect))
                    {
                        visible.push(info);
                    }
                }
                json.SetInvalidations(false, &visible);
            }
        }
        if !self.under_invalidations_.is_empty() {
            json.SetUnderInvalidations(&self.under_invalidations_);
        }
    }

    // Source: raster_invalidation_tracking.cc:134-146.
    pub fn AddToLayerDebugInfo<N: Copy + Default>(&self, debug_info: &mut LayerDebugInfo<N>) {
        for info in &self.invalidations_ {
            if !info.rect.IsEmpty() {
                debug_info
                    .invalidations
                    .push(crate::cc::layers::layer::Invalidation {
                        rect: info.rect,
                        reason: PaintInvalidationReasonToString(info.reason),
                        client: info.client_debug_name.clone(),
                    });
            }
        }
    }

    // Source: raster_invalidation_tracking.cc:166-248.
    pub fn CheckUnderInvalidations(
        &mut self,
        layer_debug_name: &str,
        new_record: B::PaintRecord,
        new_interest_rect: Rect,
    ) {
        let old_interest_rect = self.last_interest_rect_;
        let mut invalidation_region = Region::<B::RegionBackend>::default();
        if !SIMULATE_RASTER_UNDER_INVALIDATIONS.load(Ordering::Relaxed) {
            invalidation_region.Assign(&self.invalidation_region_since_last_paint_);
        }
        let old_record = self.last_painted_record_.take();
        self.last_painted_record_ = Some(new_record.clone());
        self.last_interest_rect_ = new_interest_rect;
        self.invalidation_region_since_last_paint_.Clear();
        let Some(old_record) = old_record else {
            return;
        };

        let mut rect = old_interest_rect;
        rect.Intersect(new_interest_rect);
        if rect.IsEmpty() {
            return;
        }
        let Some(old_bitmap) = self.runtime_.Rasterize(&old_record, rect) else {
            return;
        };
        let Some(mut new_bitmap) = self.runtime_.Rasterize(&new_record, rect) else {
            return;
        };

        let mut mismatching_pixels = 0;
        const K_MAX_MISMATCHES_TO_REPORT: i32 = 50;
        for bitmap_y in 0..rect.height {
            if old_bitmap.RowsEqual(&new_bitmap, bitmap_y) {
                new_bitmap.ClearRow(bitmap_y);
                continue;
            }
            let layer_y = bitmap_y + rect.y;
            for bitmap_x in 0..rect.width {
                let layer_x = bitmap_x + rect.x;
                let old_pixel = old_bitmap.Pixel(bitmap_x, bitmap_y);
                let new_pixel = new_bitmap.Pixel(bitmap_x, bitmap_y);
                if PixelsDiffer(old_pixel, new_pixel)
                    && !invalidation_region
                        .Contains(&crate::adapter::geometry::Point::new(layer_x, layer_y))
                {
                    if mismatching_pixels < K_MAX_MISMATCHES_TO_REPORT {
                        self.under_invalidations_.push(RasterUnderInvalidation {
                            x: layer_x,
                            y: layer_y,
                            old_pixel,
                            new_pixel,
                        });
                        self.runtime_.LogUnderInvalidation(
                            layer_debug_name,
                            layer_x,
                            layer_y,
                            old_pixel,
                            new_pixel,
                        );
                    } else if mismatching_pixels == K_MAX_MISMATCHES_TO_REPORT {
                        self.runtime_.LogMoreUnderInvalidations();
                    }
                    mismatching_pixels += 1;
                    new_bitmap.SetPixel(bitmap_x, bitmap_y, 0xffa00000);
                } else {
                    new_bitmap.SetPixel(bitmap_x, bitmap_y, 0x00000000);
                }
            }
        }
        if mismatching_pixels == 0 {
            return;
        }
        self.under_invalidation_record_ = self.runtime_.BuildUnderInvalidationRecord(
            std::mem::take(&mut self.under_invalidation_record_),
            new_bitmap,
            rect,
        );
    }

    pub fn UnderInvalidationRecord(&self) -> B::PaintRecord {
        self.under_invalidation_record_.clone()
    }
}

// Source: raster_invalidation_tracking.cc:148-164.
fn PixelComponentsDiffer(first: u8, second: u8) -> bool {
    if first == 0 || first == 255 || second == 0 || second == 255 {
        return first != second;
    }
    first.abs_diff(second) > 2
}

fn PixelsDiffer(first: u32, second: u32) -> bool {
    [24, 16, 8, 0]
        .into_iter()
        .any(|shift| PixelComponentsDiffer((first >> shift) as u8, (second >> shift) as u8))
}

pub fn PaintInvalidationReasonToString(reason: PaintInvalidationReason) -> &'static str {
    use PaintInvalidationReason::*;
    match reason {
        kNone => "none",
        kIncremental => "incremental",
        kHitTest => "hit testing change",
        kStyle => "style change",
        kOutline => "outline",
        kImage => "image",
        kBackplate => "backplate",
        kBackground => "background",
        kSelection => "selection",
        kCaret => "caret",
        kLayout => "geometry",
        kAppeared => "appeared",
        kDisappeared => "disappeared",
        kScrollControl => "scroll control",
        kSubtree => "subtree",
        kSVGResource => "SVG resource change",
        kDocumentMarker => "DocumentMarker change",
        kJustCreated => "just created",
        kReordered => "reordered",
        kChunkAppeared => "chunk appeared",
        kChunkDisappeared => "chunk disappeared",
        kChunkUncacheable => "chunk uncacheable",
        kChunkReordered => "chunk reordered",
        kPaintProperty => "paint property change",
        kFullLayer => "full layer",
        kUncacheable => "uncacheable",
    }
}
