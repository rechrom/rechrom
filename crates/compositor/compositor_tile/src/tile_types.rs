//! Typed resident dependencies of the canonical Tile owner. This module does
//! not implement raster tasks, resources, tilings, manager state, or tracing.
use crate::{
    tile::{Id, Tile},
    tile_draw_info::TileDrawResource,
};
use compositor_tree::{geometry::AxisTransform2d, property_types::PropertyTrace};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileManagerId(pub u64);
/// Identity of the resident native Tile*, scoped by its TileManager.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileId {
    pub manager: TileManagerId,
    pub id: Id,
    pub tiling: PictureLayerTilingId,
    pub index: crate::tile_index::TileIndex,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PictureLayerTilingId(pub u64);
/// Source tracing address only, never a dereferenceable owner route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileTraceId(pub usize);
pub trait TileTypes: Sized {
    type ResourceId: Copy;
    type Resource: TileDrawResource<ResourceId = Self::ResourceId>;
    type RasterTransform: AxisTransform2d;
    type RasterTask: ?Sized;
    fn TaskContainsLCPCandidateImages(task: &Self::RasterTask) -> bool;
}
/// Diagnostic capability consumed synchronously by the canonical source body.
/// The implicit-snapshot name refers only to Viz tracing metadata.
pub trait TileTraceContext {
    fn MakeDictIntoImplicitSnapshotWithCategory<V: PropertyTrace>(
        &self,
        category: &str,
        value: &mut V,
        name: &str,
        id: TileTraceId,
    );
    fn TraceTileDeleted(&self, category: &str, event: &str, id: TileTraceId, flow_scope: &str);
}
/// The caller lends the actual resident TileManager for this operation. No
/// closure or reference to it survives the constructor/explicit destructor.
pub trait TileIdAllocator: TileTraceContext {
    fn identity(&self) -> TileManagerId;
    fn GetUniqueTileId(&mut self) -> Id;
}
pub trait TileManagerContext<B: TileTypes>: TileIdAllocator {
    fn Release(&mut self, tile: &mut Tile<B>);
}
