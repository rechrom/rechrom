//! Exact native Paint/Picture/Tile owners for the production CPU family.
//! Registered jointly with owner_types; this file contains no service owner.
//! The main-half traits (PAC, PaintChunksToCcLayer, PendingLayer, the main
//! layers) are blanket impls over LayerizeFamily in layerize
//! (runtime/main_family.rs).
use super::{
    paint,
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::adapter::{
    paint_worklet::PaintWorkletTypes,
    picture_layer_impl::PictureLayerImplTypes,
    tile::TileTypes,
};
impl<R: CpuSkiaBackend> TileTypes for CpuCompositorTypes<R> {
    type ResourceId = crate::adapter::layer_tree_impl::ResourceId;
    type Resource = crate::cc::resources::resource_pool::InUsePoolResource<Self>;
    type RasterTransform = paint::RasterTransform;
    type RasterTask = dyn crate::cc::raster::tile_task::TileTaskImpl<
        crate::cc::tiles::tile_manager::RasterCompletion<Self>,
    >;
    fn TaskContainsLCPCandidateImages(task: &Self::RasterTask) -> bool {
        crate::cc::raster::tile_task::TileTaskImpl::TaskContainsLCPCandidateImages(task)
    }
}
impl<R: CpuSkiaBackend> compositor_tile::picture_layer_tiling_types::PictureLayerTilingTypes
    for CpuCompositorTypes<R>
{
    type RasterSource = paint::RasterSourceStorage<R>;
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
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_layer::SolidColorLayerImplTarget<layerize::commit::CommitFamily<R>>
    for super::layer::CompositorLayer<CpuCompositorTypes<R>>
{
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
