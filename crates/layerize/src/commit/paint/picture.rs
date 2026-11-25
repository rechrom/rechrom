//! Immutable recording/raster metadata products. Playback and pixel allocation
//! remain injected raster capabilities; this module owns no renderer or cache.
//! RecordingSource (main) creates the RasterSource that the impl tiling reads,
//! so these values are commit vocabulary; impl-only facets of them are
//! implemented in the compositor (runtime/paint/picture.rs).
use super::{
    display_item_list_types::DirectlyCompositedImageInfo, ContentDisplayItemList,
    CpuPaintOpBuffer, CpuPaintRecord, NodeInfo,
};
use crate::{
    adapter::{
        geometry::Rect,
        layer_tree_impl::Color4f,
        recording_source::{
            RecordingDisplayItemList, RecordingSourceTypes,
        },
    },
    cc::layers::recording_source::RecordingSource,
    commit::skia_value_backend::PaintSkiaBackend,
};
use std::{
    marker::PhantomData,

    sync::{Arc, RwLock},
};
pub struct Picture<R: PaintSkiaBackend> {
    pub record: CpuPaintRecord<R>,
    pub bounds: Rect,
}
pub struct CpuRecordingTypes<R: PaintSkiaBackend>(PhantomData<fn() -> R>);
impl<R: PaintSkiaBackend> RecordingSourceTypes for CpuRecordingTypes<R> {
    type RegionBackend = R;
    type DirectlyCompositedImageInfo = DirectlyCompositedImageInfo;
    type DisplayItemList = ContentDisplayItemList<R>;
    type RasterSource = RasterSourceStorage<R>;
}
impl<R: PaintSkiaBackend> RecordingDisplayItemList<DirectlyCompositedImageInfo>
    for ContentDisplayItemList<R>
{
    fn bounds(&self) -> Option<Rect> {
        Self::bounds(self)
    }
    fn NeedsAdditionalInvalidationForLCDText(&self, old: &Self) -> bool {
        Self::NeedsAdditionalInvalidationForLCDText(self, old)
    }
    fn EmitTraceSnapshot(&self) {
        Self::EmitTraceSnapshot(self)
    }
    fn GetDirectlyCompositedImageInfo(&self) -> Option<DirectlyCompositedImageInfo> {
        Self::GetDirectlyCompositedImageInfo(self)
    }
    fn TotalOpCount(&self) -> usize {
        Self::TotalOpCount(self)
    }
    fn GetColorIfSolidInRect(&self, rect: Rect, color: &mut Color4f, max_ops: usize) -> bool {
        Self::GetColorIfSolidInRect(self, rect, color, max_ops as i32)
    }
}
// cc::RasterSource is a cc/raster value (cc_raster), read by the impl tilings
// and the raster service.
pub use cc_raster::raster_source::{RasterDebugName, RasterSourceStorage};

/// RecordingSource::CreateRasterSource (cc/layers/recording_source.cc) with
/// the RasterSource(const RecordingSource*) constructor
/// (cc/raster/raster_source.cc:26-40): the immutable metadata values the
/// recording owns; the display-list lease stays shared.
pub fn CreateRasterSource<R: PaintSkiaBackend>(
    recording: &RecordingSource<CpuRecordingTypes<R>>,
) -> RasterSourceStorage<R> {
    let value = recording.RasterSourceFields();
    if std::env::var_os("RECHROM_TRACE_PARTIAL_RASTER").is_some() {
        eprintln!(
            "partial-full-replay-source count={} requires_clear={}",
            value.full_replays_for_partial_raster.len(),
            value.requires_clear
        );
    }
    let source = RasterSourceStorage {
        display_list: value.display_list.clone(),
        background_color: *value.background_color,
        requires_clear: *value.requires_clear,
        is_solid_color: *value.is_solid_color,
        solid_color: *value.solid_color,
        recorded_bounds: *value.recorded_bounds,
        size: *value.size,
        slow_down_raster_scale_factor_for_debug: *value.slow_down_raster_scale_factor_for_debug,
        recording_scale_factor: *value.recording_scale_factor,
        directly_composited_image_info: value.directly_composited_image_info.clone(),
        full_replays_for_partial_raster: value.full_replays_for_partial_raster.to_vec(),
        debug_name: RwLock::new(String::new()),
    };
    debug_assert!(
        source.recorded_bounds.IsEmpty()
            || Rect::new(0, 0, source.size.width(), source.size.height())
                .Contains(source.recorded_bounds)
    );
    source
}
/// Actual external SkPicture recorder. The wrapper owns its recording target,
/// and only borrows a buffer for the duration of playback.
pub trait CpuPictureRecorderValue<R: PaintSkiaBackend>: super::CpuPaintCanvas<R> {
    fn BeginRecording(&mut self, width: i32, height: i32);
    fn Clear(&mut self, color: Color4f);
    fn DrawColor(&mut self, color: Color4f);
    fn FinishRecordingAsPicture(self: Box<Self>) -> Arc<Picture<R>>;
}
pub struct CpuPictureRecorder<R: PaintSkiaBackend>(pub Box<dyn CpuPictureRecorderValue<R>>);
impl<
        R: PaintSkiaBackend,
        B: crate::adapter::solid_color_layer::SolidColorLayerTypes<Picture = Picture<R>>,
    > crate::adapter::solid_color_layer::SolidColorPictureRecorder<B> for CpuPictureRecorder<R>
{
    fn BeginRecording(&mut self, width: i32, height: i32) {
        self.0.BeginRecording(width, height)
    }
    fn DrawColor(&mut self, color: Color4f) {
        self.0.DrawColor(color)
    }
    fn FinishRecordingAsPicture(self) -> Arc<Picture<R>> {
        self.0.FinishRecordingAsPicture()
    }
}
impl<R: PaintSkiaBackend> super::display_item_list_types::DisplayCanvas for CpuPictureRecorder<R> {
    fn LocalClipBounds(&self) -> Option<Rect> {
        self.0.LocalClipBounds()
    }
}
impl<R: PaintSkiaBackend> super::CpuPaintCanvas<R> for CpuPictureRecorder<R> {
    fn Playback(
        &mut self,
        buffer: &CpuPaintOpBuffer<R>,
        params: &mut super::RasterPlaybackParams<'_, R>,
        local: bool,
        offsets: Option<&[usize]>,
    ) {
        self.0.Playback(buffer, params, local, offsets)
    }
}
impl<
        R: PaintSkiaBackend,
        B: crate::adapter::picture_layer::PictureLayerTypes<
            Picture = Picture<R>,
            PictureRecorder = Self,
        >,
    > crate::adapter::picture_layer::PictureRecorder<B> for CpuPictureRecorder<R>
{
    fn BeginRecording(&mut self, width: i32, height: i32) {
        self.0.BeginRecording(width, height)
    }
    fn Clear(&mut self, color: Color4f) {
        self.0.Clear(color)
    }
    fn FinishRecordingAsPicture(self) -> Arc<Picture<R>> {
        self.0.FinishRecordingAsPicture()
    }
}
impl<
        R: PaintSkiaBackend,
        B: crate::adapter::picture_layer::PictureLayerTypes<
            Recording = CpuRecordingTypes<R>,
            NodeInfo = NodeInfo,
            PictureRecorder = CpuPictureRecorder<R>,
        >,
    > crate::adapter::picture_layer::PictureDisplayItemList<B> for ContentDisplayItemList<R>
{
    type RasterInducingScrolls<'a> = std::iter::Copied<
        std::collections::btree_map::Keys<
            'a,
            crate::adapter::input_types::ElementId,
            super::display_item_list::RasterInducingScrollInfo,
        >,
    >;
    fn content_color_usage(&self) -> crate::adapter::layer_tree_host_impl::ContentColorUsage {
        Self::content_color_usage(self)
    }
    fn raster_inducing_scrolls(&self) -> Self::RasterInducingScrolls<'_> {
        Self::raster_inducing_scrolls(self).keys().copied()
    }
    fn CaptureContent(&self, rect: Rect, content: &mut Vec<NodeInfo>) {
        Self::CaptureContent(self, rect, content)
    }
    fn Raster(
        &self,
        recorder: &mut CpuPictureRecorder<R>,
        scrolls: &crate::adapter::prioritized_tile::ScrollOffsetMap,
    ) {
        Self::Raster(self, recorder, Some(scrolls))
    }
}
