//! Typed boundaries used by cc/layers/RecordingSource. Display-list storage
//! is resident Paint output; raster playback remains an injected CPU/GPU
//! executor concern.

use super::{
    geometry::{Rect, Size},
    layer_tree_impl::Color4f,
    region_backend::SkRegionBackend,
};
use crate::cc::layers::recording_source::RecordingSource;
use std::sync::Arc;

pub trait RecordingDisplayItemList<I>: Send + Sync {
    fn bounds(&self) -> Option<Rect>;
    fn NeedsAdditionalInvalidationForLCDText(&self, old: &Self) -> bool;
    fn EmitTraceSnapshot(&self);
    fn GetDirectlyCompositedImageInfo(&self) -> Option<I>;
    fn TotalOpCount(&self) -> usize;
    fn GetColorIfSolidInRect(&self, rect: Rect, color: &mut Color4f, max_ops: usize) -> bool;
}

pub trait RecordingSourceTypes: Sized {
    type RegionBackend: SkRegionBackend;
    type DirectlyCompositedImageInfo: Clone;
    type DisplayItemList: RecordingDisplayItemList<Self::DirectlyCompositedImageInfo>;
    type RasterSource: ?Sized;
}

pub trait RecordingSourceServices<B: RecordingSourceTypes> {
    fn CreateRasterSource(&self, recording: &RecordingSource<B>) -> Arc<B::RasterSource>;
    fn TraceScope(&self, category: &str, name: &str, op_count: Option<usize>);
}

pub fn LayerRect(size: Size) -> Rect {
    Rect::new(0, 0, size.width(), size.height())
}

pub fn ScaleToRoundedLayerRect(size: Size, scale: f32) -> Rect {
    Rect::new(
        0,
        0,
        (size.width() as f32 * scale).round() as i32,
        (size.height() as f32 * scale).round() as i32,
    )
}

/// Rust borrow mapping of the source's calling client/context and its external
/// services. Update obtains this live capability only after the virtual client
/// call has returned; no service or client borrow is retained by the recording.
pub trait RecordingSourceContext<B: RecordingSourceTypes> {
    type Services: RecordingSourceServices<B> + ?Sized;
    fn RecordingServices(&mut self) -> &Self::Services;
}
