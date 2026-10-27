// Translation status: complete
// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use compositor_tile::{
    picture_layer_tiling::PictureLayerTilingState,
    picture_layer_tiling_types::{PictureLayerTilingQueries, PictureLayerTilingTypes},
    tile::Tile,
    tile_types::TileId,
    tiles_with_resource_iterator::ForEachResourceTileInTilingSet,
};
use std::collections::HashSet;

pub use compositor_tile::tiles_with_resource_iterator::{
    ResourceQueueLayer, TilesWithResourceIterator,
};

// Interface projection of this same resident iterator; no second iteration
// implementation is provided at the embedding boundary.
impl<'a, B, L, R> crate::adapter::tile_manager::ResourceTileIterator<B>
    for TilesWithResourceIterator<'a, B, L, R>
where
    B: crate::adapter::tile_manager::TileManagerTypes + 'a,
    L: ResourceQueueLayer<B> + 'a,
    R: Iterator<Item = &'a mut L>,
{
    fn AtEnd(&self) -> bool {
        Self::AtEnd(self)
    }
    fn GetCurrent(&mut self) -> Option<&mut Tile<B>> {
        Self::GetCurrent(self)
    }
    fn Next(&mut self) {
        Self::Next(self);
    }
    fn IsCurrentTileOccluded(&mut self, queries: &dyn PictureLayerTilingQueries<B>) -> bool {
        Self::IsCurrentTileOccluded(self, queries)
    }
}

// Source: cc/tiles/tiles_with_resource_iterator.cc:13-22,37-103.
// A lending native layer range cannot keep an iterator into a previously lent
// layer across calls in safe Rust. The synchronous traversal below keeps the
// same FindNext* control flow on the call stack. Every leaf is the actual Tile;
// ControlFlow expresses caller early exit without a self-reference or relookup.
pub struct LayerResourceTileTraversal<'a, B>
where
    B: crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::layer_tree_impl::LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    active_layers: &'a mut B::LayerList,
    secondary_layers: Option<&'a mut B::LayerList>,
}
impl<'a, B> LayerResourceTileTraversal<'a, B>
where
    B: crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::layer_tree_impl::LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    pub(crate) fn FromResidentLayers(
        active_layers: &'a mut B::LayerList,
        secondary_layers: Option<&'a mut B::LayerList>,
    ) -> Self {
        Self {
            active_layers,
            secondary_layers,
        }
    }
}
impl<B> crate::adapter::tile_manager::ResourceTileTraversal<B> for LayerResourceTileTraversal<'_, B>
where
    B: crate::adapter::tile_manager::TileManagerTypes
        + crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::layer_tree_impl::LayerTreeTypes<
            PictureLayer = crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
        >,
{
    fn ForEachResourceTile(
        &mut self,
        callback: &mut dyn FnMut(
            &mut Tile<B>,
            &PictureLayerTilingState<B>,
        ) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()> {
        use crate::adapter::layer_tree_impl::LayerTreeLayerList;
        // Native visited_ spans both layer ranges for this traversal.
        let mut visited = HashSet::new();
        let mut visit_layer =
            |layer: &mut crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>| {
                ForEachResourceTileInPicture(
                    layer,
                    &mut visited,
                    &mut |tile, state, _notification| callback(tile, state),
                )
            };
        self.active_layers.ForEachPictureLayer(&mut visit_layer)?;
        if let Some(secondary) = self.secondary_layers.as_deref_mut() {
            secondary.ForEachPictureLayer(&mut visit_layer)?;
        }
        std::ops::ControlFlow::Continue(())
    }
}

// Source: cc/tiles/tiles_with_resource_iterator.cc:84-103. Shared traversal
// body for query-only and notifying callers. Every callback lends the current
// actual tile together with its own Picture's disjoint notification fields.
pub(crate) fn ForEachResourceTileInPicture<B>(
    layer: &mut crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>,
    visited: &mut HashSet<TileId>,
    callback: &mut dyn FnMut(
        &mut Tile<B>,
        &PictureLayerTilingState<B>,
        &mut crate::cc::layers::picture_layer_impl::PictureResourceTileNotificationParts<'_>,
    ) -> std::ops::ControlFlow<()>,
) -> std::ops::ControlFlow<()>
where
    B: crate::adapter::picture_layer_impl::PictureLayerImplTypes,
{
    let (set, mut notification) = layer.SplitResourceTraversal();
    ForEachResourceTileInTilingSet(set, visited, &mut notification, callback)
}
