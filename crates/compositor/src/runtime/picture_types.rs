//! Exact native Paint/Picture/Tile owners for the production CPU family.
//! Registered jointly with owner_types; this file contains no service owner.
use super::{
    paint,
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::adapter::{
    paint_worklet::PaintWorkletTypes, picture_layer::PictureLayerTypes,
    picture_layer_impl::PictureLayerImplTypes, picture_layer_tiling::PictureLayerTilingTypes,
    tile::TileTypes,
};
impl<R: CpuSkiaBackend> crate::adapter::paint_artifact_compositor::PaintArtifactCompositorTypes
    for CpuCompositorTypes<R>
{
    type CanvasSnapshotCallback = std::sync::Arc<dyn paint::CpuCanvasSnapshot<R>>;
    type CompositingReasons = paint::CpuCompositingReasons;
    type LayersAsJson = paint::CpuLayersAsJson<R>;
    type CrashKeyScope = paint::PaintTraceScope;
    type UpdateTraceScope = paint::PaintTraceScope;
}
impl<R: CpuSkiaBackend> crate::adapter::paint_chunks_to_cc_layer::PaintChunksTypes
    for CpuCompositorTypes<R>
{
    type ChunkMapper = paint::CpuGeometryMapper<R>;
    type RoundedRect = crate::adapter::geometry::RRectF;
    type PaintPath = std::sync::Arc<::paint::paint_clip_path::PaintClipPath>;
    type ScrollNode = paint::ScrollNode<R>;
    type PaintFlags = paint::CpuPaintFlags<R>;
    type PaintFilter = paint::CpuPaintFilter;
    type FilterOperations = ::paint::cc::paint::filter_operations::FilterOperations;
    type HitTestData = ::paint::paint_engine::PaintHitTestData;
    type RegionCaptureData = Vec<([u64; 2], ::paint::paint_engine::PaintRect)>;
    type TrackedData = Vec<::paint::paint_engine::PaintTrackedElementRect>;
    type SelectionData = ::paint::paint_engine::PaintLayerSelectionData;
    type CaptureId = super::metadata::RegionCaptureCropId;
    type TrackingFeature = super::metadata::TrackedElementFeature;
    type TrackedId = [u64; 2];
    type FrameToken = Option<super::metadata::FrameToken>;
    type PaintRecorder = paint::CpuConversionRecorder<R>;
    type UnderInvalidationPicture = paint::CpuPaintRecord<R>;
}
impl<R: CpuSkiaBackend> crate::adapter::paint_artifact_compositor::SynthesizedClipTypes
    for CpuCompositorTypes<R>
{
    type SkRoundedRect = super::skia::SkRoundedRect;
}
impl<R: CpuSkiaBackend> crate::adapter::property_tree_manager::PropertyTreeManagerTypes
    for CpuCompositorTypes<R>
{
    fn RootTransformNode() -> paint::TransformNode<R> {
        paint::PropertyTreeState::<R>::default().transform()
    }
    fn RootClipNode() -> paint::ClipNode<R> {
        paint::PropertyTreeState::<R>::default().clip()
    }
    fn RootEffectNode() -> paint::EffectNode<R> {
        paint::PropertyTreeState::<R>::default().effect()
    }
    fn RootScrollNode() -> paint::ScrollNode<R> {
        paint::ScrollNode::from_arc(
            ::paint::paint_property_tree::PropertyTreeState::default()
                .transform
                .scroll
                .as_ref()
                .expect("native ScrollRoot")
                .clone(),
        )
    }
    fn ScrollNodeForTransform(t: &paint::TransformNode<R>) -> Option<paint::ScrollNode<R>> {
        t.node()
            .scroll
            .as_ref()
            .map(|s| paint::ScrollNode::from_arc(s.clone()))
    }
    fn UniformScaleTransform(scale: f32) -> super::geometry::Transform<R> {
        let mut t = super::geometry::Transform::default();
        t.native_mut().Scale(scale, scale);
        t
    }
    fn SourceToDestinationProjection(
        s: &paint::TransformNode<R>,
        d: &paint::TransformNode<R>,
    ) -> super::geometry::Transform<R> {
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper::GeometryMapper::<paint::CpuGeometryMapper<R>>::SourceToDestinationProjection(s,d)
    }
    // Source: platform/graphics/compositor_element_id.cc:41-49. Replace only
    // namespace bits, preserving the original unique object identity.
    fn EffectMaskElementId(
        effect: crate::adapter::input_types::ElementId,
    ) -> crate::adapter::input_types::ElementId {
        let n = foundation::CompositorElementIdNamespace::kEffectMask as u64;
        let bits = foundation::compositor_element_id::kCompositorNamespaceBitCount
            + foundation::compositor_element_id::kCompositorReservedBitCount;
        crate::adapter::input_types::ElementId::new(
            (effect.GetInternalValue() & !((1u64 << bits) - 1))
                | (n << foundation::compositor_element_id::kCompositorReservedBitCount),
        )
    }
    fn NewUniqueElementId() -> crate::adapter::input_types::ElementId {
        crate::adapter::input_types::ElementId::new(
            foundation::CompositorElementIdFromUniqueObjectId(
                foundation::NewUniqueObjectId(),
                foundation::CompositorElementIdNamespace::kUniqueObjectId,
            )
            .GetInternalValue(),
        )
    }
}
impl<R: CpuSkiaBackend> crate::adapter::pending_layer::PendingLayerTypes for CpuCompositorTypes<R> {
    type ContentDisplayItemList = paint::ContentDisplayItemList<R>;
    type RasterInvalidator = paint::CpuRasterInvalidator<R>;
    type RasterInvalidationTracking=crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::RasterInvalidationTracking<paint::CpuRasterTrackingTypes<R>>;
    type PaintRecord = paint::CpuPaintRecord<R>;
    type PaintOpBuffer = paint::CpuPaintOpBuffer<R>;
    type Json = serde_json::Value;
    type JsonArray = Vec<serde_json::Value>;
    type LoggingCanvas = paint::CpuLoggingCanvas<R>;
    type PaintArtifact = paint::PaintArtifact;
    type PaintChunk = paint::PaintChunk;
    type DisplayItem = paint::DisplayItem;
    type PaintChunkSubset = paint::PaintChunkSubset;
    type PropertyTreeState = paint::PropertyTreeState<R>;
    type TransformNode = paint::TransformNode<R>;
    type ClipNode = paint::ClipNode<R>;
    type EffectNode = paint::EffectNode<R>;
    type PropertyNodeIdentity = usize;
    type PaintPropertyChange = paint::PaintPropertyChange;
    type DomNodeId = super::types::NodeId;
    type ClientId = usize;
    type CanvasChildPaintRecord = paint::CpuCanvasChildPaintRecord<R>;
    type CanvasChildPaintState =
        std::sync::Arc<::paint::paint_property_tree::PaintCanvasChildState>;
    fn UnchangedPaintPropertyChange() -> Self::PaintPropertyChange {
        Self::PaintPropertyChange::Unchanged
    }
    fn ChangedOnlySimpleValues() -> Self::PaintPropertyChange {
        Self::PaintPropertyChange::ChangedOnlySimpleValues
    }
    fn InvalidDomNodeId() -> Self::DomNodeId {
        super::types::NodeId(0)
    }
}
impl<R: CpuSkiaBackend> TileTypes for CpuCompositorTypes<R> {
    type Resource = crate::cc::resources::resource_pool::InUsePoolResource<Self>;
    type RasterTransform = paint::RasterTransform;
    type RasterTask = dyn crate::cc::raster::tile_task::TileTaskImpl<
        crate::cc::tiles::tile_manager::RasterCompletion<Self>,
    >;
}
impl<R: CpuSkiaBackend> PictureLayerTilingTypes for CpuCompositorTypes<R> {
    type RasterSource = paint::NativeRasterSource<R>;
    type RegionBackend = R;
    type OcclusionTransform = super::geometry::Transform<R>;
    type PaintWorkletRecordMap = crate::adapter::picture_layer_impl::PaintWorkletRecordMap<
        paint::CpuWorkletInput,
        paint::CpuPaintRecord<R>,
    >;
    type DrawImage = paint::CpuDrawImage;
}
impl<R: CpuSkiaBackend> PaintWorkletTypes for CpuCompositorTypes<R> {
    type PropertyKey = paint::WorkletPropertyKey;
    type PropertyValue = paint::WorkletPropertyValue;
    type Input = paint::CpuWorkletInput;
    type PaintImageId = crate::adapter::image_animation::PaintImageId;
}
impl<R: CpuSkiaBackend> crate::adapter::tile_based_layer_impl::TileBasedLayerImplTypes
    for CpuCompositorTypes<R>
{
}
impl<R: CpuSkiaBackend> PictureLayerImplTypes for CpuCompositorTypes<R> {
    type PaintRecord = paint::CpuPaintRecord<R>;
    type PictureTargetColorParams = paint::TargetColorParams;
    type DiscardableImageMap = paint::CpuDiscardableImageMap;
    fn WrapPictureLayerImpl(
        layer: crate::cc::layers::picture_layer_impl::PictureLayerImpl<Self>,
    ) -> Self::Layer {
        super::layer::CompositorLayer::Picture(layer)
    }
}
impl<R: CpuSkiaBackend> PictureLayerTypes for CpuCompositorTypes<R> {
    type Recording = paint::CpuRecordingTypes<R>;
    type ObjectTracker = paint::ObjectTracker;
    type TraceScope = paint::PaintTraceScope;
    type LayerTreeTask = paint::PaintTraceScope;
    type PictureRecorder = paint::CpuPictureRecorder<R>;
}
impl<R: CpuSkiaBackend> crate::adapter::solid_color_layer::SolidColorLayerTypes
    for CpuCompositorTypes<R>
{
    type SolidPictureRecorder = paint::CpuPictureRecorder<R>;
}
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_layer::SolidColorLayerImplTarget<CpuCompositorTypes<R>>
    for super::layer::CompositorLayer<CpuCompositorTypes<R>>
{
    fn CreateSolidColor(
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<CpuCompositorTypes<R>>,
        id: i32,
        context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<
            '_,
            CpuCompositorTypes<R>,
        >,
    ) -> Box<Self> {
        use crate::adapter::layer_impl::LayerImplTypes;
        CpuCompositorTypes::<R>::WithLayerImplContext(tree, context, |tree_id, c| {
            crate::cc::layers::solid_color_layer_impl::SolidColorLayerImpl::Create(tree_id, id, c)
        })
    }
}
impl<R: CpuSkiaBackend> crate::adapter::tile_manager::TileManagerTypes for CpuCompositorTypes<R> {
    type PaintImage = paint::CpuPaintImage;
    type PaintImageId = crate::adapter::image_animation::PaintImageId;
    type TargetColorParams = paint::TargetColorParams;
    type ContentColorUsage = crate::adapter::layer_tree_host_impl::ContentColorUsage;
    type RasterBuffer = paint::TileRasterBuffer<R>;
    type PlaybackImageProvider = paint::CpuPlaybackImageProvider<R>;
    type PaintWorkletImageProvider = paint::CpuWorkletProvider<R>;
    type ScopedRasterResult = paint::CpuScopedRasterResult<R>;
    fn TileRasterTaskResourceSource() -> Self::ResourceSource {
        super::resources::RasterResourceSource::TileRasterTask
    }
    fn HdrContentColorUsage() -> Self::ContentColorUsage {
        crate::adapter::layer_tree_host_impl::ContentColorUsage::kHDR
    }
    fn SrgbContentColorUsage() -> Self::ContentColorUsage {
        crate::adapter::layer_tree_host_impl::ContentColorUsage::kSRGB
    }
    fn TestResourceSource() -> Self::ResourceSource {
        super::resources::RasterResourceSource::Test
    }
    fn TestRasterSyncToken() -> Self::SyncToken {
        super::resources::RasterSyncToken::new(
            super::resources::CommandBufferNamespace::GpuIo,
            1,
            1,
        )
    }
    fn IsHDR(space: &Self::ColorSpace) -> bool {
        space.IsHDR()
    }
}
