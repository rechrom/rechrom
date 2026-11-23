//! Impl-side facets of the cc recording values (PictureRasterSource,
//! TileManagerRasterSource, discardable-image facets), the CPU family's
//! external micro-benchmark and derived Layer services, and gfx
//! AxisTransform2d. The values themselves live in layerize::commit::paint.
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
