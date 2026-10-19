// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/prioritized_tile.h:1-20,67-69;
//         cc/tiles/prioritized_tile.cc:1-10,44.
// Modules replace includes, forward declarations, export and namespaces.
use super::{
    tile::Tile,
    tile_priority::{TilePriority, TileResolutionToString},
};
use crate::adapter::{
    prioritized_tile::{
        PrioritizedTileContext, PrioritizedTileTiling, PrioritizedTileTraceContext,
        RasterSourceTraceId, ScrollOffsetMap,
    },
    property_types::PropertyTrace,
    tile::{PictureLayerTilingId, TileId},
};
use std::sync::Arc;

// Source: cc/tiles/prioritized_tile.h:56-65.
// Native non-owning pointers map to typed identities, resolved through a live
// context on each access. Only source-owned priority/flags are stored by value.
pub struct PrioritizedTile {
    tile_: Option<TileId>,
    source_tiling_: Option<PictureLayerTilingId>,
    priority_: TilePriority,
    is_occluded_: bool,
    is_process_for_images_only_: bool,
    should_decode_checkered_images_for_tile_: bool,
}
impl Default for PrioritizedTile {
    // Source: cc/tiles/prioritized_tile.h:24,59-64;
    //         cc/tiles/prioritized_tile.cc:12.
    fn default() -> Self {
        Self {
            tile_: None,
            source_tiling_: None,
            priority_: TilePriority::default(),
            is_occluded_: false,
            is_process_for_images_only_: false,
            should_decode_checkered_images_for_tile_: false,
        }
    }
}
// Source: cc/tiles/prioritized_tile.h:20-65 (implicit copy operations);
//         cc/tiles/prioritized_tile.cc:28 (default destructor).
// Copies retain only pointer identities/value fields, never clone Tile/tiling
// state. Default Rust drop does not release either resident owner.
impl Clone for PrioritizedTile {
    fn clone(&self) -> Self {
        Self {
            tile_: self.tile_,
            source_tiling_: self.source_tiling_,
            priority_: self.priority_,
            is_occluded_: self.is_occluded_,
            is_process_for_images_only_: self.is_process_for_images_only_,
            should_decode_checkered_images_for_tile_: self.should_decode_checkered_images_for_tile_,
        }
    }
}
impl PrioritizedTile {
    // Source: cc/tiles/prioritized_tile.h:25-31;
    //         cc/tiles/prioritized_tile.cc:14-26.
    pub fn new(
        tile: Option<TileId>,
        source_tiling: Option<PictureLayerTilingId>,
        priority: &TilePriority,
        is_occluded: bool,
        is_process_for_images_only: bool,
        should_decode_checkered_images_for_tile: bool,
    ) -> Self {
        Self {
            tile_: tile,
            source_tiling_: source_tiling,
            priority_: *priority,
            is_occluded_: is_occluded,
            is_process_for_images_only_: is_process_for_images_only,
            should_decode_checkered_images_for_tile_: should_decode_checkered_images_for_tile,
        }
    }
    // Source: cc/tiles/prioritized_tile.h:33.
    pub fn tile<'c, T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> Option<&'c Tile<T::Types>> {
        self.tile_.map(|id| context.TileById(id))
    }
    // Rust equivalent of testing the source pointer, without dereferencing it.
    pub(crate) fn tile_id(&self) -> Option<TileId> {
        self.tile_
    }
    pub(crate) fn source_tiling_id(&self) -> Option<PictureLayerTilingId> {
        self.source_tiling_
    }
    // Source: cc/tiles/prioritized_tile.h:34-36.
    // Borrow the existing scoped_refptr/Arc slot; do not increase its refcount.
    pub fn raster_source<'c, T: PrioritizedTileTiling + ?Sized + 'c>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> &'c Option<Arc<T::RasterSource>> {
        context
            .PrioritizedTilingById(self.source_tiling_.expect("source nonnull tiling"))
            .raster_source()
    }
    // Source: cc/tiles/prioritized_tile.h:37-39.
    pub fn GetPaintWorkletRecords<'c, T: PrioritizedTileTiling + ?Sized + 'c>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> &'c T::PaintWorkletRecordMap
    where
        T::Types: 'c,
    {
        context.GetPaintWorkletRecords(self.source_tiling_.expect("source nonnull tiling"))
    }
    // Source: cc/tiles/prioritized_tile.h:40-42.
    // The upstream method explicitly returns the offsets as a value.
    pub fn GetRasterInducingScrollOffsets<T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &(impl PrioritizedTileContext<T> + ?Sized),
    ) -> ScrollOffsetMap {
        context.GetRasterInducingScrollOffsets(self.source_tiling_.expect("source nonnull tiling"))
    }
    // Source: cc/tiles/prioritized_tile.h:43-50.
    pub fn priority(&self) -> &TilePriority {
        &self.priority_
    }
    pub fn is_occluded(&self) -> bool {
        self.is_occluded_
    }
    pub fn is_process_for_images_only(&self) -> bool {
        self.is_process_for_images_only_
    }
    pub fn should_decode_checkered_images_for_tile(&self) -> bool {
        self.should_decode_checkered_images_for_tile_
    }
    // Source: cc/tiles/prioritized_tile.h:52;
    //         cc/tiles/prioritized_tile.cc:30-42.
    pub fn AsValueInto<T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &(impl PrioritizedTileContext<T> + ?Sized),
        value: &mut impl PropertyTrace,
        trace: &impl PrioritizedTileTraceContext,
    ) {
        self.AsValueIntoWithResident(
            self.tile(context).expect("source nonnull tile"),
            self.raster_source(context),
            value,
            trace,
        );
    }
    // Source: cc/tiles/prioritized_tile.cc:30-42.
    // The source raw pointers are already resolved to resident owners for this
    // call. Share the one serialization body with the generic context entry.
    pub(crate) fn AsValueIntoWithResident<B: crate::adapter::tile::TileTypes, R: ?Sized>(
        &self,
        tile: &Tile<B>,
        raster_source: &Option<Arc<R>>,
        value: &mut impl PropertyTrace,
        trace: &impl PrioritizedTileTraceContext,
    ) {
        tile.AsValueInto(value, trace);
        let id = raster_source
            .as_ref()
            .map_or(0, |source| Arc::as_ptr(source) as *const () as usize);
        trace.SetIDRef(RasterSourceTraceId(id), value, "picture_pile");
        value.BeginDictionary("combined_priority");
        self.priority().AsValueInto(value);
        value.SetBoolean("is_occluded", self.is_occluded_);
        value.EndDictionary();
        value.SetString(
            "resolution",
            TileResolutionToString(self.priority().resolution),
        );
    }
    // Source: cc/tiles/prioritized_tile.h:54.
    pub fn source_tiling<'c, T: PrioritizedTileTiling + ?Sized>(
        &self,
        context: &'c (impl PrioritizedTileContext<T> + ?Sized),
    ) -> Option<&'c T> {
        self.source_tiling_
            .map(|id| context.PrioritizedTilingById(id))
    }
}
