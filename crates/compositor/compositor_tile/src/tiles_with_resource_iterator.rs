// Translation status: complete
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiles_with_resource_iterator.h:1-78
// Source: cc/tiles/tiles_with_resource_iterator.cc:1-115
use crate::{
    picture_layer_tiling::{PictureLayerTiling, PictureLayerTilingState, TileIterator},
    picture_layer_tiling_set::PictureLayerTilingSet,
    picture_layer_tiling_types::{PictureLayerTilingQueries, PictureLayerTilingTypes},
    tile::Tile,
    tile_types::TileId,
};
use std::{collections::HashSet, ops::ControlFlow, slice::IterMut};

/// Actual mutable resident range projection for callers that can lend each
/// layer for the entire cursor lifetime. Production map ranges use traversal.
pub trait ResourceQueueLayer<B: PictureLayerTilingTypes> {
    fn picture_layer_tiling_set(&mut self) -> &mut PictureLayerTilingSet<B>;
}

// Source: cc/tiles/tiles_with_resource_iterator.h:25-74
// The source layer-range iterator, selected Set and selected Tiling cursor are
// represented by disjoint borrows of their actual residents. No range, layer,
// tiling, map or tile is copied. Consuming one native iterator item lends the
// pointee for the traversal lifetime, so no self-reference is necessary.
pub struct TilesWithResourceIterator<'a, B, L, R>
where
    B: PictureLayerTilingTypes + 'a,
    L: ResourceQueueLayer<B> + 'a,
    R: Iterator<Item = &'a mut L>,
{
    visited_: HashSet<TileId>,
    tile_iterator_: Option<TileIterator<'a, B>>,
    current_tiling_state_: Option<&'a PictureLayerTilingState<B>>,
    current_picture_layer_tiling_index_: usize,
    current_picture_layer_tilings_: Option<IterMut<'a, Box<PictureLayerTiling<B>>>>,
    active_layers_: R,
    is_active_layers_secondary_layers_: bool,
    secondary_picture_layers_: Option<R>,
}

impl<'a, B, L, R> TilesWithResourceIterator<'a, B, L, R>
where
    B: PictureLayerTilingTypes + 'a,
    L: ResourceQueueLayer<B> + 'a,
    R: Iterator<Item = &'a mut L>,
{
    // Source: cc/tiles/tiles_with_resource_iterator.h:26-31
    // Source: cc/tiles/tiles_with_resource_iterator.cc:13-22
    pub fn new(picture_layers: R, secondary_picture_layers: R) -> Self {
        let mut result = Self {
            secondary_picture_layers_: Some(secondary_picture_layers),
            is_active_layers_secondary_layers_: false,
            active_layers_: picture_layers,
            current_picture_layer_tiling_index_: 0,
            current_picture_layer_tilings_: None,
            current_tiling_state_: None,
            tile_iterator_: None,
            visited_: HashSet::new(),
        };
        result.FindNextInPictureLayers();
        result
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:33
    // Source: cc/tiles/tiles_with_resource_iterator.cc:24-26
    pub fn AtEnd(&self) -> bool {
        self.tile_iterator_.is_none()
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:35
    // Source: cc/tiles/tiles_with_resource_iterator.cc:28-30
    pub fn GetCurrent(&mut self) -> Option<&mut Tile<B>> {
        if self.AtEnd() {
            None
        } else {
            self.tile_iterator_.as_mut().unwrap().GetCurrent()
        }
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:37-38
    // Source: cc/tiles/tiles_with_resource_iterator.cc:32-35
    pub fn IsCurrentTileOccluded(&mut self, queries: &dyn PictureLayerTilingQueries<B>) -> bool {
        let Some(iterator) = self.tile_iterator_.as_mut() else {
            return false;
        };
        let Some(tile) = iterator.GetCurrent() else {
            return false;
        };
        // Source tile->tiling() is this cursor's resident state, lent alongside
        // the same tiling's map. The native occlusion algorithm remains there.
        let state = self.current_tiling_state_.unwrap();
        debug_assert_eq!(state.identity(), tile.identity().tiling);
        state.IsTileOccluded(tile, queries)
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:34
    // Source: cc/tiles/tiles_with_resource_iterator.cc:37-52
    pub fn Next(&mut self) {
        if self.AtEnd() {
            return;
        }
        debug_assert!(self.tile_iterator_.is_some());
        self.tile_iterator_.as_mut().unwrap().Next();
        if self.FindNextInTileIterator() {
            return;
        }
        self.current_picture_layer_tiling_index_ += 1;
        if self.FindNextInPictureLayerTilingSet() {
            return;
        }
        // The native ++current_picture_layer_ is the next call to the actual
        // remaining layer iterator (its previous item was lent, not copied).
        if self.FindNextInPictureLayers() {
            return;
        }
        debug_assert!(self.AtEnd());
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:44
    // Source: cc/tiles/tiles_with_resource_iterator.cc:54-69
    fn FindNextInPictureLayers(&mut self) -> bool {
        if self.FindNextInActiveLayers() {
            return true;
        }
        debug_assert!(self.AtEnd());
        if self.is_active_layers_secondary_layers_ {
            return false;
        }
        self.is_active_layers_secondary_layers_ = true;
        self.active_layers_ = self.secondary_picture_layers_.take().unwrap();
        // The source empty test and begin reset are expressed by the actual
        // iterator's first next in FindNextInActiveLayers, with no layer read.
        self.FindNextInActiveLayers()
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:45
    // Source: cc/tiles/tiles_with_resource_iterator.cc:71-82,105-108
    fn FindNextInActiveLayers(&mut self) -> bool {
        while let Some(layer) = self.active_layers_.next() {
            self.current_picture_layer_tiling_index_ = 0;
            // CurrentPictureLayerTilingSet: lends this layer's actual owner.
            self.current_picture_layer_tilings_ =
                Some(layer.picture_layer_tiling_set().TilingsMutable());
            if self.FindNextInPictureLayerTilingSet() {
                return true;
            }
        }
        self.tile_iterator_ = None;
        self.current_tiling_state_ = None;
        false
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:46,49-50
    // Source: cc/tiles/tiles_with_resource_iterator.cc:84-94,110-113
    fn FindNextInPictureLayerTilingSet(&mut self) -> bool {
        while let Some(tiling) = self.current_picture_layer_tilings_.as_mut().unwrap().next() {
            // CurrentPictureLayerTiling: current index selects the actual Box.
            let (state, iterator) = tiling.StateAndTileIterator();
            self.current_tiling_state_ = Some(state);
            self.tile_iterator_ = Some(iterator);
            if self.FindNextInTileIterator() {
                return true;
            }
            // The pinned source increments once in the body (line 91) and
            // once in the for clause (line 87). Preserve both, including the
            // skipped tiling; do not replace this with a conventional scan.
            self.current_picture_layer_tiling_index_ += 1;
            self.current_picture_layer_tilings_.as_mut().unwrap().next();
            self.current_picture_layer_tiling_index_ += 1;
        }
        false
    }

    // Source: cc/tiles/tiles_with_resource_iterator.h:47,71-73
    // Source: cc/tiles/tiles_with_resource_iterator.cc:96-103
    fn FindNextInTileIterator(&mut self) -> bool {
        let iterator = self.tile_iterator_.as_mut().unwrap();
        while !iterator.AtEnd() {
            let tile = iterator
                .GetCurrent()
                .expect("native TileMap entry is non-null");
            if self.visited_.insert(tile.identity()) && tile.draw_info().has_resource() {
                return true;
            }
            iterator.Next();
        }
        false
    }
}

// Source: cc/tiles/tiles_with_resource_iterator.cc:84-103. This is the
// package-owned resource eligibility and ordering algorithm. The facade lends
// the concrete Picture notification state as an opaque value.
pub fn ForEachResourceTileInTilingSet<B, N>(
    set: &mut PictureLayerTilingSet<B>,
    visited: &mut HashSet<TileId>,
    notification: &mut N,
    callback: &mut dyn FnMut(&mut Tile<B>, &PictureLayerTilingState<B>, &mut N) -> ControlFlow<()>,
) -> ControlFlow<()>
where
    B: PictureLayerTilingTypes,
{
    let mut tilings = set.TilingsMutable();
    while let Some(tiling) = tilings.next() {
        let (state, mut tiles) = tiling.StateAndTileIterator();
        let mut found_in_tiling = false;
        while !tiles.AtEnd() {
            let tile = tiles
                .GetCurrent()
                .expect("native TileMap entry is non-null");
            if visited.insert(tile.identity()) && tile.draw_info().has_resource() {
                found_in_tiling = true;
                callback(tile, state, notification)?;
            }
            tiles.Next();
        }
        // FindNextInPictureLayerTilingSet increments twice only when its
        // initial FindNextInTileIterator found no eligible tile. After a
        // successful tile, Next advances once at cc:45.
        if !found_in_tiling {
            tilings.next();
        }
    }
    ControlFlow::Continue(())
}
