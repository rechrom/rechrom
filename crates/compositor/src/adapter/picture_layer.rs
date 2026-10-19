//! Exact external Paint/debug capabilities used by PictureLayer virtuals.
//! No Layer, host, tree, or service is stored here or copied into an owner.
use super::{
    geometry::{Rect, Size},
    input_types::ElementId,
    layer_tree_host_impl::ContentColorUsage,
    layer_tree_impl::{Color4f, LayerTreeTypes},
    main_layer::{MainLayerImpl, MainLayerImplTree, MainLayerTypes},
    recording_source::{RecordingDisplayItemList, RecordingSourceServices, RecordingSourceTypes},
};
use crate::cc::{
    base::region::Region,
    layers::picture_layer::PictureLayer,
    trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
};
use std::sync::Arc;
pub trait PictureLayerTypes:
    MainLayerTypes<
    DerivedServices: PictureLayerServices<Self>,
    Layer: PictureLayerImplTarget<Self>,
    MicroBenchmark: PictureLayerMicroBenchmark<Self>,
    NodeInfo: PictureLayerNodeInfo,
>
{
    type Recording: RecordingSourceTypes<
            RegionBackend = <Self as LayerTreeTypes>::RegionBackend,
            DisplayItemList: PictureDisplayItemList<Self>,
            DirectlyCompositedImageInfo: Send + Sync + 'static,
            RasterSource: Send + Sync + 'static,
        > + 'static;
    type ObjectTracker: Send + Sync + 'static;
    type TraceScope: 'static;
    type LayerTreeTask: 'static;
    type PictureRecorder: PictureRecorder<Self>;
}
pub trait PictureLayerNodeInfo {
    fn visual_rect(&self) -> Rect;
    fn set_visual_rect(&mut self, rect: Rect);
}
pub trait PictureLayerMicroBenchmark<B: PictureLayerTypes> {
    fn RunOnPictureLayer(&mut self, layer: &PictureLayer<B>);
}
pub trait PictureDisplayItemList<B: PictureLayerTypes>:
    RecordingDisplayItemList<<B::Recording as RecordingSourceTypes>::DirectlyCompositedImageInfo>
{
    type RasterInducingScrolls<'a>: Iterator<Item = ElementId>
    where
        Self: 'a;
    fn content_color_usage(&self) -> ContentColorUsage;
    fn raster_inducing_scrolls(&self) -> Self::RasterInducingScrolls<'_>;
    fn CaptureContent(&self, rect: Rect, content: &mut Vec<B::NodeInfo>);
    fn Raster(
        &self,
        recorder: &mut B::PictureRecorder,
        scrolls: &super::prioritized_tile::ScrollOffsetMap,
    );
}
pub trait PictureRecorder<B: PictureLayerTypes> {
    fn BeginRecording(&mut self, width: i32, height: i32);
    fn Clear(&mut self, color: Color4f);
    fn FinishRecordingAsPicture(self) -> Arc<B::Picture>;
}
pub trait PictureLayerServices<B: PictureLayerTypes>:
    RecordingSourceServices<B::Recording>
{
    fn CreateObjectTracker(&self, layer_id: i32) -> B::ObjectTracker;
    fn BeginPushPropertiesTrace(&self) -> B::TraceScope;
    fn BeginUpdateTrace(&self, source_frame_number: i32) -> B::TraceScope;
    fn BeginUpdateLayerTask(&self, layer_id: i32, host_id: i32) -> B::LayerTreeTask;
    fn CreatePictureRecorder(&self) -> B::PictureRecorder;
}
/// Same-package native downcast/derived-construction interface, pending the
/// canonical PictureLayerImpl stem. Only resident setters/constructor dispatch
/// live here; no main-layer/impl-layer algorithm is an external Paint service.
pub trait PictureLayerImplTarget<B: PictureLayerTypes>: MainLayerImpl<B> {
    fn CreatePicture(
        tree: &mut LayerTreeImpl<B>,
        id: i32,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Box<Self>;
    fn set_has_non_animated_image_update_rect(&mut self);
    fn set_gpu_raster_max_texture_size(&mut self, size: Size, tree: &mut B::ImplContext<'_>);
    fn SetIsBackdropFilterMask(&mut self, value: bool, tree: &mut B::ImplContext<'_>);
    fn StageNewRasterSourceForCommit(
        &mut self,
        source: Arc<<B::Recording as RecordingSourceTypes>::RasterSource>,
        invalidation: Region<<B as LayerTreeTypes>::RegionBackend>,
    );
    fn set_should_batch_updated_tiles(&mut self);
}
