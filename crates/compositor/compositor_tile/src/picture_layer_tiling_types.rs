//! External immutable raster values and call-scoped live owner capabilities.
//! All tiling/priority/creation/occlusion algorithms remain in the canonical
//! translated PictureLayerTiling; none of these services mirrors owner state.
use crate::{
    picture_layer_tiling::{PictureLayerTiling, PictureLayerTilingClient, PictureLayerTilingState},
    tile::{CreateInfo, Tile},
    tile_types::{PictureLayerTilingId, TileId, TileManagerId, TileTypes},
};
use compositor_draw::occlusion::OcclusionTransform;
use compositor_tree::{
    geometry::{Rect, Size},
    region::Region,
    region_backend::SkRegionBackend,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PictureLayerTilingClientId(pub u64);
/// Non-owning identity of an existing owner's resident invalidation field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PendingInvalidationId(pub u64);
/// Immutable resident raster-source metadata; no playback or platform work.
pub trait TilingRasterSource {
    fn IsSolidColor(&self) -> bool;
    fn size(&self) -> Size;
    fn recorded_bounds(&self) -> Rect;
    fn IntersectsRect(&self, rect: Rect) -> bool;
}
pub trait PictureLayerTilingTypes: TileTypes {
    type RasterSource: TilingRasterSource;
    type RegionBackend: SkRegionBackend;
    type OcclusionTransform: OcclusionTransform;
    type PaintWorkletRecordMap: ?Sized;
    type DrawImage: ?Sized;
}
pub trait PictureLayerTilingQueries<B: PictureLayerTilingTypes> {
    /// Source client pointers are borrowed only for this synchronous call.
    fn WithClient(
        &self,
        id: PictureLayerTilingClientId,
        callback: &mut dyn FnMut(&dyn PictureLayerTilingClient<B>),
    );
    /// This native field reference belongs to the actual Picture allocation,
    /// independently of a temporary client-vtable wrapper.
    fn PaintWorkletRecordsForClient(
        &self,
        id: PictureLayerTilingClientId,
    ) -> &B::PaintWorkletRecordMap;
    fn TilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<B>;
    fn TileById(&self, id: TileId) -> &Tile<B> {
        self.TilingById(id.tiling).TileById(id)
    }
    fn IsClientOnActiveOrPendingTree(&self, id: PictureLayerTilingClientId) -> bool;
    /// Current resident source Set selection; scoped selected-twin contexts
    /// lend the same state directly while its owning Set is mutably executing.
    fn TilingStateForClientAndScale(
        &self,
        client: PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&PictureLayerTilingState<B>>;
    fn PendingInvalidationById(&self, id: PendingInvalidationId) -> &Region<B::RegionBackend>;
}
pub trait PictureLayerTilingContext<B: PictureLayerTilingTypes>:
    PictureLayerTilingQueries<B> + PictureLayerTileDestroyer<B>
{
    fn CalculateTileSizeForClient(&mut self, id: PictureLayerTilingClientId, bounds: Size) -> Size;
    // Source callers: cc/tiles/picture_layer_tiling.cc:91 and native Tile
    // destruction (cc/tiles/tile.cc:40-48). These are synchronous field-split
    // forwarding points into canonical PictureLayerImpl/TileManager bodies.
    // No manager/pool/host reference is retained across the call.
    fn CreateTileForClient(
        &mut self,
        client: PictureLayerTilingClientId,
        info: &CreateInfo<B::RasterTransform>,
    ) -> Option<Box<Tile<B>>>;
    fn IsCCSlimmingEnabled(&self) -> bool;
}

/// The destructor chain never queries a client. The current owner lends only
/// the synchronous canonical TileManager destroy/release action.
pub trait PictureLayerTileDestroyer<B: PictureLayerTilingTypes> {
    /// Rust identity bridge for C++'s stable `Tile*`. Moving the same Tile
    /// allocation between twin tilings changes only its resolver location.
    fn RebindTileWithManager(&mut self, old: TileId, new: TileId);
    fn DestroyTileWithManager(
        &mut self,
        manager: TileManagerId,
        tile: &mut Tile<B>,
        tiling: &PictureLayerTilingState<B>,
    );
}

/// A single scoped virtual call can return a native value, but cannot let its
/// borrowed client wrapper escape. No client is cached or allocated here.
pub fn WithTilingClient<B: PictureLayerTilingTypes, Q: PictureLayerTilingQueries<B> + ?Sized, R>(
    queries: &Q,
    id: PictureLayerTilingClientId,
    action: impl FnOnce(&dyn PictureLayerTilingClient<B>) -> R,
) -> R {
    let mut action = Some(action);
    let mut result = None;
    queries.WithClient(id, &mut |client| {
        result = Some(action.take().expect("one synchronous native client borrow")(client))
    });
    result.expect("native client identity must resolve exactly once")
}
