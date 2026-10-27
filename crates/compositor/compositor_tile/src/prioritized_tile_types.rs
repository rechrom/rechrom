//! Resident external values consumed by PrioritizedTile. Raster source and paint
//! worklet records remain owned by their existing packages/tiling owner.
use crate::{
    prioritized_tile::PrioritizedTile,
    tile::Tile,
    tile_types::{PictureLayerTilingId, TileId, TileTraceContext, TileTypes},
};
use compositor_tree::{geometry::PointF, property_types::PropertyTrace, ElementId};
use std::collections::BTreeMap;
use std::sync::Arc;
/// Source value: cc/paint/scroll_offset_map.h:14 (ordered flat_map).
pub type ScrollOffsetMap = BTreeMap<ElementId, PointF>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterSourceTraceId(pub usize);
pub trait PrioritizedTileTraceContext: TileTraceContext {
    fn SetIDRef<V: PropertyTrace>(&self, id: RasterSourceTraceId, value: &mut V, name: &str);
}

pub trait PrioritizedTileTiling {
    type Types: TileTypes;
    type RasterSource: ?Sized;
    type PaintWorkletRecordMap: ?Sized;
    fn raster_source(&self) -> &Option<Arc<Self::RasterSource>>;
}

pub trait PrioritizedTileContext<T: PrioritizedTileTiling + ?Sized> {
    fn TileById(&self, id: TileId) -> &Tile<T::Types>;
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

pub trait PrioritizedTileResidentExt {
    fn tile<'c, T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> Option<&'c Tile<T::Types>>;
    fn raster_source<'c, T: PrioritizedTileTiling + ?Sized + 'c>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> &'c Option<Arc<T::RasterSource>>;
    fn GetPaintWorkletRecords<'c, T: PrioritizedTileTiling + ?Sized + 'c>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> &'c T::PaintWorkletRecordMap
    where
        T::Types: 'c;
    fn GetRasterInducingScrollOffsets<T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &(impl PrioritizedTileContext<T> + ?Sized),
    ) -> ScrollOffsetMap;
    fn source_tiling<'c, T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> Option<&'c T>;
}

impl PrioritizedTileResidentExt for PrioritizedTile {
    fn tile<'c, T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> Option<&'c Tile<T::Types>> {
        self.tile_id().map(|id| context.TileById(id))
    }
    fn raster_source<'c, T: PrioritizedTileTiling + ?Sized + 'c>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> &'c Option<Arc<T::RasterSource>> {
        context
            .PrioritizedTilingById(self.source_tiling_id().expect("source nonnull tiling"))
            .raster_source()
    }
    fn GetPaintWorkletRecords<'c, T: PrioritizedTileTiling + ?Sized + 'c>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> &'c T::PaintWorkletRecordMap
    where
        T::Types: 'c,
    {
        context.GetPaintWorkletRecords(self.source_tiling_id().expect("source nonnull tiling"))
    }
    fn GetRasterInducingScrollOffsets<T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &(impl PrioritizedTileContext<T> + ?Sized),
    ) -> ScrollOffsetMap {
        context
            .GetRasterInducingScrollOffsets(self.source_tiling_id().expect("source nonnull tiling"))
    }
    fn source_tiling<'c, T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> Option<&'c T> {
        self.source_tiling_id()
            .map(|id| context.PrioritizedTilingById(id))
    }
}
