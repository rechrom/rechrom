//! Impl-side facets of the cc recording values (PictureRasterSource,
//! TileManagerRasterSource, discardable-image facets) and gfx AxisTransform2d.
//! The values live in layerize::commit::paint; the micro-benchmark and the
//! main Layer service bundle in layerize::runtime::paint.
use super::{ContentDisplayItemList, RasterDebugName, RasterSourceStorage};
use crate::{
    adapter::{
        geometry::Rect, layer_tree_impl::Color4f,
        picture_layer_impl::{DirectlyCompositedImageInfo, PictureRasterSource},
        property_types::PropertyTrace,
    },
    runtime::types::CpuSkiaBackend,
};
use std::sync::Arc;
impl<R: CpuSkiaBackend> PictureRasterSource for RasterSourceStorage<R> {
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
    > for RasterSourceStorage<R>
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
// gfx::AxisTransform2d is a cc/gfx shared value.
pub use compositor_values::axis_transform2d::RasterTransform;
