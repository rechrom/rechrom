//! Source virtual dispatch on the actual resident most-derived layer. Tree
//! queries explicitly borrow this call's property trees; no layer view is owned.
use super::layer::CompositorLayer;
use crate::{
    adapter::{
        layer_impl::LayerImplOwner,
        layer_tree_host_impl::{ContentColorUsage, HostImplLayer},
        layer_tree_impl::PropertyTreesFor,
        picture_layer_impl::PictureLayerImplTypes,
        solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes,
    },
    cc::base::region::Region,
};

impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> HostImplLayer
    for CompositorLayer<B>
{
    type Types = B;
    type Transform = B::Transform;
    type RegionBackend = <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend;
    // Source virtuals: cc/layers/layer_impl.h:494; picture_layer_impl.h:67.
    fn GetInvalidationRegionForDebugging(
        &self,
    ) -> Region<<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend> {
        match self {
            Self::Picture(layer) => layer.GetInvalidationRegionForDebugging(),
            _ => self.impl_base().GetInvalidationRegionForDebugging(),
        }
    }
    fn draws_content(&self) -> bool {
        self.impl_base().draws_content()
    }
    // Source: cc/layers/layer_impl.cc:929-945. The sole canonical helper resolves
    // the actual transform node rather than retaining any tree in this layer.
    fn ScreenSpaceTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base().ScreenSpaceTransformForPropertyTrees(trees)
    }
    // Source: cc/layers/layer_impl.cc:947-959.
    fn VisibleLayerRect(&self, trees: &PropertyTreesFor<B>) -> crate::adapter::geometry::Rect {
        self.impl_base().VisibleLayerRectForPropertyTrees(trees)
    }
    fn transform_tree_index(&self) -> i32 {
        self.impl_base().transform_tree_index()
    }
    // Source virtuals: cc/layers/layer_impl.h:554; picture_layer_impl.h:68.
    fn GetContentColorUsage(&self) -> ContentColorUsage {
        match self {
            Self::Picture(layer) => layer.GetContentColorUsage(),
            _ => self.impl_base().GetContentColorUsage(),
        }
    }
}

impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes>
    crate::adapter::layer_tree_host_impl::HostImplHitTestLayer<B> for CompositorLayer<B>
{
    fn draws_content(&self) -> bool {
        self.impl_base().draws_content()
    }
    fn AsHitTestSurfaceLayer(
        &self,
    ) -> Option<&dyn crate::adapter::layer_tree_host_impl::HostHitTestSurfaceLayer<B>> {
        // This enum's actual subtype family contains no SurfaceLayerImpl.
        // A future real Surface variant must project its resident owner here.
        None
    }
    fn is_clipped(&self) -> bool {
        self.impl_base().is_clipped()
    }
}
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes, F>
    crate::adapter::throttle::ThrottleLayer<F> for CompositorLayer<B>
{
    fn surface_range(&self) -> Option<&dyn crate::adapter::throttle::SurfaceRange<F>> {
        // Source cc/trees/throttle_decider.cc:90-96 only enters this subtype.
        None
    }
}

impl<B: crate::adapter::tile::TileTypes> crate::adapter::layer_tree_host_impl::HostTile
    for crate::cc::tiles::tile::Tile<B>
{
    fn tiling_id(&self) -> Option<crate::adapter::tile::PictureLayerTilingId> {
        self.tiling()
    }
    fn layer_id(&self) -> i32 {
        crate::cc::tiles::tile::Tile::layer_id(self)
    }
    fn required_for_draw(&self) -> bool {
        crate::cc::tiles::tile::Tile::required_for_draw(self)
    }
}

impl<B: PictureLayerImplTypes> crate::adapter::layer_tree_host_impl::PictureLayerDisplayRead
    for crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>
{
    fn id(&self) -> i32 {
        self.base().id()
    }
}
impl crate::adapter::layer_tree_host_impl::PictureLayerDisplayRead
    for crate::cc::layers::layer_impl::LayerImplTileClient<'_>
{
    fn id(&self) -> i32 {
        crate::cc::layers::layer_impl::LayerImplTileClient::id(self)
    }
}
impl<B: crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>
    crate::adapter::layer_tree_host_impl::HostDisplayTileTiling
    for crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>
{
    fn raster_translation(&self) -> &crate::adapter::geometry::Vector2dF {
        crate::adapter::geometry::AxisTransform2d::translation(self.raster_transform())
    }
    fn raster_scale(&self) -> &crate::adapter::geometry::Vector2dF {
        crate::adapter::geometry::AxisTransform2d::scale(self.raster_transform())
    }
    fn tile_size(&self) -> crate::adapter::geometry::Size {
        crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState::tile_size(self)
    }
    fn tiling_rect(&self) -> &crate::adapter::geometry::Rect {
        crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState::tiling_rect(self)
    }
}
