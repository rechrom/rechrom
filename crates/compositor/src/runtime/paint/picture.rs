//! Immutable recording/raster metadata products. Playback and pixel allocation
//! remain injected raster capabilities; this module owns no renderer or cache.
use super::{CpuPaintOpBuffer, CpuPaintRecord, CpuPaintTypes};
use crate::{
    adapter::{
        geometry::{Rect, Size},
        layer_tree_impl::{Color4f, LayerTreeTypes},
        picture_layer_impl::{DirectlyCompositedImageInfo, PictureRasterSource},
        picture_layer_tiling::TilingRasterSource,
        property_types::PropertyTrace,
        recording_source::{RecordingDisplayItemList, RecordingSourceTypes},
    },
    cc::{layers::recording_source::RecordingSource, paint::display_item_list::DisplayItemList},
    runtime::types::CpuSkiaBackend,
};
use std::{
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard},
};
pub type ContentDisplayItemList<R> = DisplayItemList<CpuPaintTypes<R>>;
pub use super::CpuDrawImage as DrawImage;
pub use super::CpuFilterQuality as FilterQuality;
pub use crate::runtime::display_item_list::NodeInfo;
// Source: cc/paint/paint_flags.h:95-133. The native high limit is (0,0).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DynamicRangeLimit {
    pub standard_mix: f32,
    pub constrained_high_mix: f32,
}
pub struct Picture<R: CpuSkiaBackend> {
    pub record: CpuPaintRecord<R>,
    pub bounds: Rect,
}
pub struct CpuRecordingTypes<R: CpuSkiaBackend>(PhantomData<fn() -> R>);
impl<R: CpuSkiaBackend> RecordingSourceTypes for CpuRecordingTypes<R> {
    type RegionBackend = R;
    type DirectlyCompositedImageInfo = DirectlyCompositedImageInfo;
    type DisplayItemList = ContentDisplayItemList<R>;
    type RasterSource = NativeRasterSource<R>;
}
impl<R: CpuSkiaBackend> RecordingDisplayItemList<DirectlyCompositedImageInfo>
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
/// Source: cc/raster/raster_source.cc:26-40. These are precisely the immutable
/// metadata values owned by RasterSource; its recording lease remains shared.
pub struct NativeRasterSource<R: CpuSkiaBackend> {
    display_list: Option<Arc<ContentDisplayItemList<R>>>,
    pub background_color: Color4f,
    pub requires_clear: bool,
    is_solid_color: bool,
    solid_color: Color4f,
    recorded_bounds: Rect,
    size: Size,
    pub slow_down_raster_scale_factor_for_debug: i32,
    pub recording_scale_factor: f32,
    directly_composited_image_info: Option<DirectlyCompositedImageInfo>,
    debug_name: RwLock<String>,
}
impl<R: CpuSkiaBackend> NativeRasterSource<R> {
    pub fn from_recording(recording: &RecordingSource<CpuRecordingTypes<R>>) -> Self {
        let value = recording.RasterSourceFields();
        let source = Self {
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
            debug_name: RwLock::new(String::new()),
        };
        debug_assert!(
            source.recorded_bounds.IsEmpty()
                || Rect::new(0, 0, source.size.width(), source.size.height())
                    .Contains(source.recorded_bounds)
        );
        source
    }
    pub fn display_list(&self) -> &Option<Arc<ContentDisplayItemList<R>>> {
        &self.display_list
    }
    pub fn DebugName(&self) -> RasterDebugName<'_> {
        RasterDebugName(
            self.debug_name
                .read()
                .expect("native source debug name poisoned"),
        )
    }
}
pub struct RasterDebugName<'a>(RwLockReadGuard<'a, String>);
impl Deref for RasterDebugName<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}
impl<R: CpuSkiaBackend> TilingRasterSource for NativeRasterSource<R> {
    fn IsSolidColor(&self) -> bool {
        self.is_solid_color
    }
    fn size(&self) -> Size {
        self.size
    }
    // Source: cc/raster/raster_source.h:118-122. Direct image rasterization uses
    // full size; the stored recorded rectangle itself is never rewritten.
    fn recorded_bounds(&self) -> Rect {
        if self.directly_composited_image_info.is_some() {
            Rect::new(0, 0, self.size.width(), self.size.height())
        } else {
            self.recorded_bounds
        }
    }
    fn IntersectsRect(&self, rect: Rect) -> bool {
        self.recorded_bounds().Intersects(rect)
    }
}
impl<R: CpuSkiaBackend> PictureRasterSource for NativeRasterSource<R> {
    type DisplayItemList = ContentDisplayItemList<R>;
    fn GetDisplayItemList(&self) -> &Option<Arc<Self::DisplayItemList>> {
        &self.display_list
    }
    // Source: cc/raster/raster_source.cc:166-168.
    fn HasRecordings(&self) -> bool {
        self.display_list.is_some() && !self.recorded_bounds.IsEmpty()
    }
    fn GetSolidColor(&self) -> Color4f {
        debug_assert!(self.is_solid_color);
        self.solid_color
    }
    fn directly_composited_image_info(&self) -> Option<&DirectlyCompositedImageInfo> {
        self.directly_composited_image_info.as_ref()
    }
    fn DidBeginTracing(&self) {
        if let Some(list) = &self.display_list {
            list.EmitTraceSnapshot();
        }
    }
    fn set_debug_name(&self, name: &str) {
        *self
            .debug_name
            .write()
            .expect("native source debug name poisoned") = name.to_owned();
    }
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        if let Some(list) = &self.display_list {
            value.BeginDictionaryUnnamed();
            value.SetString("id_ref", &format!("0x{:x}", Arc::as_ptr(list) as usize));
            value.EndDictionary();
        }
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::tile_manager::TileManagerRasterSource<
        crate::runtime::types::CpuCompositorTypes<R>,
    > for NativeRasterSource<R>
{
    type DisplayItemList = ContentDisplayItemList<R>;
    type DebugName<'a>
        = RasterDebugName<'a>
    where
        Self: 'a;
    fn debug_name(&self) -> Self::DebugName<'_> {
        self.DebugName()
    }
    fn content_color_usage(&self) -> crate::adapter::layer_tree_host_impl::ContentColorUsage {
        self.display_list
            .as_ref()
            .expect("source CHECK(display_list_)")
            .content_color_usage()
    }
    fn GetDisplayItemList(&self) -> &Self::DisplayItemList {
        self.display_list
            .as_deref()
            .expect("source CHECK(display_list_)")
    }
    // Source: cc/raster/raster_source.cc:137-145.
    fn PerformSolidColorAnalysis(&self, rect: &Rect, color: &mut Color4f, max_ops: i32) -> bool {
        let mut rect = *rect;
        rect.Intersect(Rect::new(0, 0, self.size.width(), self.size.height()));
        rect = crate::adapter::geometry::ScaleToRoundedRect(rect, self.recording_scale_factor);
        self.display_list
            .as_ref()
            .expect("source CHECK(display_list_)")
            .GetColorIfSolidInRect(rect, color, max_ops)
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::picture_layer_impl::PictureDisplayItemListFor<
        crate::runtime::types::CpuCompositorTypes<R>,
    > for ContentDisplayItemList<R>
{
    fn GenerateDiscardableImageMap(
        &self,
        offsets: crate::adapter::prioritized_tile::ScrollOffsetMap,
        decoding: Option<&mut crate::adapter::picture_layer_impl::PictureImageDecodingModeMap>,
        worklets: Option<
            &mut crate::adapter::picture_layer_impl::PicturePaintWorkletInputs<
                crate::runtime::types::CpuCompositorTypes<R>,
            >,
        >,
    ) -> Arc<super::CpuDiscardableImageMap> {
        Arc::new(Self::GenerateDiscardableImageMap(
            self, &offsets, decoding, worklets,
        ))
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::picture_layer_impl::PictureDiscardableImageMap<
        crate::runtime::types::CpuCompositorTypes<R>,
    > for super::CpuDiscardableImageMap
{
    fn empty(&self) -> bool {
        self.0.empty()
    }
    fn animated_images_metadata(
        &self,
    ) -> &std::collections::BTreeMap<
        crate::adapter::image_animation::PaintImageId,
        crate::adapter::image_animation::AnimatedImageMetadata,
    > {
        self.0.animated_images_metadata()
    }
    fn GetRectsForImage(&self, id: crate::adapter::image_animation::PaintImageId) -> &[Rect] {
        self.0.GetRectsForImage(id)
    }
    fn GetDiscardableImagesInRect(&self, rect: &Rect) -> Vec<&super::CpuDrawImage> {
        self.0.GetDiscardableImagesInRect(rect)
    }
}
/// Actual external micro-benchmark implementation; current resident receivers
/// are lent on each source virtual call, never retained as owner references.
pub trait CpuPictureMicroBenchmark<B: crate::adapter::picture_layer::PictureLayerTypes> {
    fn RunOnPictureLayer(&mut self, layer: &crate::cc::layers::picture_layer::PictureLayer<B>);
}
pub struct MicroBenchmark<B: crate::adapter::picture_layer::PictureLayerTypes>(
    pub Box<dyn CpuPictureMicroBenchmark<B>>,
);
impl<B: crate::adapter::picture_layer::PictureLayerTypes>
    crate::adapter::picture_layer::PictureLayerMicroBenchmark<B> for MicroBenchmark<B>
{
    fn RunOnPictureLayer(&mut self, layer: &crate::cc::layers::picture_layer::PictureLayer<B>) {
        self.0.RunOnPictureLayer(layer)
    }
}
/// Derived Layer services remain external/call-scoped; RecordingSource and all
/// Picture/Tile policy is canonical. Implementors provide real recorder values.
pub trait CpuPaintServices<B: crate::adapter::picture_layer::PictureLayerTypes>:
    crate::adapter::picture_layer::PictureLayerServices<B>
    + crate::adapter::solid_color_layer::SolidColorLayerServices<B>
    + crate::adapter::paint_artifact_compositor::SynthesizedClipPaintServices<B>
    + crate::adapter::content_layer_client_impl::ContentLayerClientServices<B>
    + crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJsonColor
where
    B: crate::adapter::solid_color_layer::SolidColorLayerTypes
        + crate::adapter::paint_artifact_compositor::SynthesizedClipTypes,
{
}
impl<
        B: crate::adapter::picture_layer::PictureLayerTypes
            + crate::adapter::solid_color_layer::SolidColorLayerTypes
            + crate::adapter::paint_artifact_compositor::SynthesizedClipTypes,
        T: crate::adapter::picture_layer::PictureLayerServices<B>
            + crate::adapter::solid_color_layer::SolidColorLayerServices<B>
            + crate::adapter::paint_artifact_compositor::SynthesizedClipPaintServices<B>
        + crate::adapter::content_layer_client_impl::ContentLayerClientServices<B>
        + crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJsonColor
            + ?Sized,
    > CpuPaintServices<B> for T
{
}

/// Actual external SkPicture recorder. The wrapper owns its recording target,
/// and only borrows a buffer for the duration of playback.
pub trait CpuPictureRecorderValue<R: CpuSkiaBackend>: super::CpuPaintCanvas<R> {
    fn BeginRecording(&mut self, width: i32, height: i32);
    fn Clear(&mut self, color: Color4f);
    fn DrawColor(&mut self, color: Color4f);
    fn FinishRecordingAsPicture(self: Box<Self>) -> Arc<Picture<R>>;
}
pub struct CpuPictureRecorder<R: CpuSkiaBackend>(pub Box<dyn CpuPictureRecorderValue<R>>);
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_layer::SolidColorPictureRecorder<
        crate::runtime::types::CpuCompositorTypes<R>,
    > for CpuPictureRecorder<R>
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
impl<R: CpuSkiaBackend> crate::adapter::display_item_list::DisplayCanvas for CpuPictureRecorder<R> {
    fn LocalClipBounds(&self) -> Option<Rect> {
        self.0.LocalClipBounds()
    }
}
impl<R: CpuSkiaBackend> super::CpuPaintCanvas<R> for CpuPictureRecorder<R> {
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
        R: CpuSkiaBackend,
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
        R: CpuSkiaBackend,
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
            crate::cc::paint::display_item_list::RasterInducingScrollInfo,
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

/// Source: ui/gfx/geometry/axis_transform2d.h:38-55,97-113,132-156.
/// Pure native gfx value subset consumed by Tile/coverage. Float clamping is
/// performed by the existing Foundation geometry owner, including NaN rules.
#[derive(Clone, Debug, PartialEq)]
pub struct RasterTransform {
    scale: crate::adapter::geometry::Vector2dF,
    translation: crate::adapter::geometry::Vector2dF,
}
impl Default for RasterTransform {
    fn default() -> Self {
        Self {
            scale: crate::adapter::geometry::Vector2dF::new(1., 1.),
            translation: Default::default(),
        }
    }
}
impl crate::adapter::geometry::AxisTransform2d for RasterTransform {
    fn FromScaleAndTranslation(
        scale: crate::adapter::geometry::Vector2dF,
        translation: crate::adapter::geometry::Vector2dF,
    ) -> Self {
        Self { scale, translation }
    }
    fn scale(&self) -> &crate::adapter::geometry::Vector2dF {
        &self.scale
    }
    fn translation(&self) -> &crate::adapter::geometry::Vector2dF {
        &self.translation
    }
    fn PreScale(&mut self, value: crate::adapter::geometry::Vector2dF) {
        self.scale = crate::adapter::geometry::Vector2dF::new(
            self.scale.x() * value.x(),
            self.scale.y() * value.y(),
        );
    }
    fn MapRect(&self, rect: crate::adapter::geometry::RectF) -> crate::adapter::geometry::RectF {
        debug_assert!(self.scale.x() >= 0. && self.scale.y() >= 0.);
        let clamp = |v: f32| foundation::gfx::Transform::clamp_float_geometry(f64::from(v));
        crate::adapter::geometry::RectF::new(
            clamp(rect.x() * self.scale.x() + self.translation.x()),
            clamp(rect.y() * self.scale.y() + self.translation.y()),
            clamp(rect.width() * self.scale.x()),
            clamp(rect.height() * self.scale.y()),
        )
    }
    fn InverseMapRect(
        &self,
        rect: crate::adapter::geometry::RectF,
    ) -> crate::adapter::geometry::RectF {
        debug_assert!(self.scale.x() > 0. && self.scale.y() > 0.);
        let clamp = |v: f32| foundation::gfx::Transform::clamp_float_geometry(f64::from(v));
        crate::adapter::geometry::RectF::new(
            clamp((rect.x() - self.translation.x()) * (1. / self.scale.x())),
            clamp((rect.y() - self.translation.y()) * (1. / self.scale.y())),
            clamp(rect.width() * (1. / self.scale.x())),
            clamp(rect.height() * (1. / self.scale.y())),
        )
    }
}
