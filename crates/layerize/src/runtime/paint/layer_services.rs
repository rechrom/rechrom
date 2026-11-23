//! Main Layer service bundle of the CPU family and the PictureLayer
//! micro-benchmark hook (cc/layers/picture_layer.h RunMicroBenchmark).
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


