//! Resident external values consumed by PrioritizedTile. Raster source and paint
//! worklet records remain owned by their existing packages/tiling owner.
use super::{
    geometry::PointF,
    input_types::ElementId,
    property_types::PropertyTrace,
    tile::{PictureLayerTilingId, TileId, TileTraceContext, TileTypes},
};
use std::{collections::BTreeMap, sync::Arc};
/// Source value: cc/paint/scroll_offset_map.h:14 (ordered flat_map).
pub type ScrollOffsetMap = BTreeMap<ElementId, PointF>;
pub trait PrioritizedTileTiling {
    type Types: TileTypes;
    type RasterSource: ?Sized;
    type PaintWorkletRecordMap: ?Sized;
    fn raster_source(&self) -> &Option<Arc<Self::RasterSource>>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterSourceTraceId(pub usize);
pub trait PrioritizedTileTraceContext: TileTraceContext {
    fn SetIDRef<V: PropertyTrace>(&self, id: RasterSourceTraceId, value: &mut V, name: &str);
}

/// Native non-owning pointer resolution. Each returned reference is borrowed
/// from the actual resident owner only for this call; no aggregate is copied.
pub trait PrioritizedTileContext<T: PrioritizedTileTiling + ?Sized> {
    fn TileById(&self, id: TileId) -> &crate::cc::tiles::tile::Tile<T::Types>;
    fn PrioritizedTilingById(&self, id: PictureLayerTilingId) -> &T;
    fn GetPaintWorkletRecords<'a>(
        &'a self,
        id: PictureLayerTilingId,
    ) -> &'a T::PaintWorkletRecordMap
    where
        T: 'a,
        T::Types: 'a;
    fn GetRasterInducingScrollOffsets(&self, id: PictureLayerTilingId) -> ScrollOffsetMap;
}
