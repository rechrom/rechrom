// Translation status: complete
// Copyright 2016 The Chromium Authors
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/layer_collections.h:1-215;
//         cc/layers/layer_collections.cc:1-218.
// Rust modules replace include guards, includes, exports and namespaces.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};

// Source: cc/layers/layer_collections.h:23-26,35-48.
// These are the operations used by this source file on LayerImpl and
// PictureLayerImpl. The canonical LayerImpl translation implements this
// same-package facet; no external service owns or copies layer state.
pub trait LayerCollectionPicture {
    fn id(&self) -> i32;
    fn GetPaintWorkletRecordMapSize(&self) -> usize;
    fn HasAnimatedImages(&self) -> bool;
}

pub trait LayerCollectionEntry {
    type Picture: LayerCollectionPicture + ?Sized;

    fn id(&self) -> i32;
    fn GetChangedAllProperties(&self) -> bool;
    fn AsPicture(&self) -> Option<&Self::Picture>;
    fn AsPictureMut(&mut self) -> Option<&mut Self::Picture>;
}

// Source: cc/layers/layer_collections.h:35-48,189-198.
type VectorType<L> = Vec<Option<Box<L>>>;
type MapType = BTreeMap<i32, usize>;
type SetType = BTreeSet<i32>;
// Named native slice iterators allow the type family's borrowed iterator GATs
// to retain their concrete representation without allocating boxed iterators.
pub type LayerVectorIterator<'a, L> =
    std::iter::Map<std::slice::Iter<'a, Option<Box<L>>>, fn(&'a Option<Box<L>>) -> &'a L>;
pub type LayerVectorIteratorMut<'a, L> = std::iter::Map<
    std::slice::IterMut<'a, Option<Box<L>>>,
    fn(&'a mut Option<Box<L>>) -> &'a mut L,
>;
fn LayerSlot<L>(slot: &Option<Box<L>>) -> &L {
    slot.as_deref().expect("source live layer slot")
}
fn LayerSlotMut<L>(slot: &mut Option<Box<L>>) -> &mut L {
    slot.as_deref_mut().expect("source live layer slot")
}

#[derive(Default)]
struct LayerMaps {
    layer_map: MapType,
    picture_layer_map: MapType,
    layer_maps_need_rebuild: bool,
}

// Source: cc/layers/layer_collections.h:28-33,189-199.
// The local lock represents C++ mutable index caches. It never protects the
// layers themselves and therefore cannot turn the resident tree into a
// snapshot or a whole-object synchronization wrapper.
pub struct OwnedLayerImplList<L: LayerCollectionEntry> {
    layers_: VectorType<L>,
    layers_that_should_push_properties_: SetType,
    picture_layers_with_animated_images_: SetType,
    picture_layers_with_worklets_: SetType,
    num_picture_layers_: u32,
    maps_: RwLock<LayerMaps>,
}

// Source: cc/layers/layer_collections.h:50-56;
//         cc/layers/layer_collections.cc:14-17.
// Rust's ordinary move semantics provide the source move constructor and move
// assignment; this type intentionally has no Clone implementation.
impl<L: LayerCollectionEntry> Default for OwnedLayerImplList<L> {
    fn default() -> Self {
        Self {
            layers_: Vec::new(),
            layers_that_should_push_properties_: BTreeSet::new(),
            picture_layers_with_animated_images_: BTreeSet::new(),
            picture_layers_with_worklets_: BTreeSet::new(),
            num_picture_layers_: 0,
            maps_: RwLock::new(LayerMaps::default()),
        }
    }
}

impl<L: LayerCollectionEntry> OwnedLayerImplList<L> {
    // Source: cc/layers/layer_collections.h:51;
    //         cc/layers/layer_collections.cc:14.
    pub fn new() -> Self {
        Self::default()
    }

    // Source: cc/layers/layer_collections.h:58-60;
    //         cc/layers/layer_collections.cc:28-30.
    pub fn empty(&self) -> bool {
        self.layers_.is_empty()
    }
    pub fn size(&self) -> usize {
        self.layers_.len()
    }
    pub fn reserve(&mut self, count: usize) {
        self.layers_.reserve(count);
    }
    /// IDs of owners left behind after TreeSynchronizer released every reused
    /// slot. Released slots are intentionally skipped.
    pub fn RemainingLayerIds(&self) -> Vec<i32> {
        self.layers_
            .iter()
            .filter_map(|slot| slot.as_deref().map(LayerCollectionEntry::id))
            .collect()
    }

    // Source: cc/layers/layer_collections.h:62-64;
    //         cc/layers/layer_collections.cc:32-43.
    // Detach first, clear every lookup path, then destroy layers. A LayerImpl
    // destructor therefore cannot re-enter this list and find another layer.
    pub fn clear(&mut self) {
        let detached_layers = std::mem::take(&mut self.layers_);
        self.layers_that_should_push_properties_.clear();
        {
            let mut maps = self.maps_.write().expect("layer map lock poisoned");
            maps.layer_map.clear();
            maps.layer_maps_need_rebuild = false;
            maps.picture_layer_map.clear();
        }
        self.picture_layers_with_animated_images_.clear();
        self.picture_layers_with_worklets_.clear();
        self.num_picture_layers_ = 0;
        drop(detached_layers);
    }

    // Source: cc/layers/layer_collections.h:64;
    //         cc/layers/layer_collections.cc:45-64.
    pub fn push_back(&mut self, value: Box<L>) {
        let id = value.id();
        let changed_all_properties = value.GetChangedAllProperties();
        let picture_state = value.AsPicture().map(|picture| {
            (
                picture.GetPaintWorkletRecordMapSize() != 0,
                picture.HasAnimatedImages(),
            )
        });
        self.layers_.push(Some(value));
        self.maps_
            .write()
            .expect("layer map lock poisoned")
            .layer_maps_need_rebuild = true;
        if changed_all_properties {
            self.layers_that_should_push_properties_.insert(id);
        }
        if let Some((has_worklets, has_animated_images)) = picture_state {
            self.num_picture_layers_ += 1;
            if has_worklets {
                self.picture_layers_with_worklets_.insert(id);
            }
            if has_animated_images {
                self.picture_layers_with_animated_images_.insert(id);
            }
        }
    }

    // Source: cc/layers/layer_collections.h:66-80.
    pub fn iter(&self) -> LayerVectorIterator<'_, L> {
        self.layers_.iter().map(LayerSlot::<L>)
    }
    pub fn iter_mut(&mut self) -> LayerVectorIteratorMut<'_, L> {
        self.layers_.iter_mut().map(LayerSlotMut::<L>)
    }
    pub fn rbegin(&self) -> impl DoubleEndedIterator<Item = &L> {
        self.iter().rev()
    }
    pub fn at(&self, pos: usize) -> &L {
        self.layers_[pos].as_deref().expect("released layer slot")
    }
    pub fn at_mut(&mut self, pos: usize) -> &mut L {
        self.layers_[pos]
            .as_deref_mut()
            .expect("released layer slot")
    }
    pub fn front(&self) -> &L {
        self.at(0)
    }
    pub fn back(&self) -> &L {
        self.at(self.layers_.len() - 1)
    }

    // Source: cc/layers/layer_collections.h:82-84;
    //         cc/layers/layer_collections.cc:118-131.
    pub fn find_index(&self, id: i32) -> Option<usize> {
        self.EnsureLayerMaps();
        self.maps_
            .read()
            .expect("layer map lock poisoned")
            .layer_map
            .get(&id)
            .copied()
    }
    pub fn find(&self, id: i32) -> Option<&L> {
        self.find_index(id)
            .and_then(|index| self.layers_[index].as_deref())
    }
    pub fn find_mut(&mut self, id: i32) -> Option<&mut L> {
        let index = self.find_index(id)?;
        self.layers_[index].as_deref_mut()
    }
    pub fn contains(&self, id: i32) -> bool {
        self.find(id).is_some()
    }

    // Source: cc/layers/layer_collections.h:86-126,161-163;
    //         cc/layers/layer_collections.cc:66-77,176-184.
    pub fn LayersThatShouldPushProperties(&self) -> LayerSetRange<'_, L> {
        LayerSetRange::new(self, &self.layers_that_should_push_properties_)
    }
    pub fn SetShouldPushProperties(&mut self, layer_id: i32) {
        self.layers_that_should_push_properties_.insert(layer_id);
    }
    pub fn ClearLayersShouldPushProperties(&mut self) {
        self.layers_that_should_push_properties_.clear();
    }

    // Source: cc/layers/layer_collections.h:128-165;
    //         cc/layers/layer_collections.cc:79-85,197-216.
    pub fn PictureLayers(&self) -> PictureMapRange<'_, L> {
        self.EnsureLayerMaps();
        PictureMapRange::new(self)
    }

    // Source: cc/layers/layer_collections.h:167-169;
    //         cc/layers/layer_collections.cc:87-101,186-195.
    pub fn PictureLayersWithAnimatedImages(&self) -> PictureSetRange<'_, L> {
        PictureSetRange::new(self, &self.picture_layers_with_animated_images_)
    }
    pub fn SetPictureLayerWithAnimatedImages(&mut self, layer_id: i32) {
        debug_assert!(self.contains(layer_id));
        debug_assert!(self
            .find(layer_id)
            .is_some_and(|layer| layer.AsPicture().is_some()));
        self.picture_layers_with_animated_images_.insert(layer_id);
    }
    pub fn RemovePictureLayerWithAnimatedImages(&mut self, layer_id: i32) {
        self.picture_layers_with_animated_images_.remove(&layer_id);
    }

    // Source: cc/layers/layer_collections.h:171-173;
    //         cc/layers/layer_collections.cc:103-116,207-216.
    pub fn PictureLayersWithWorklets(&self) -> PictureSetRange<'_, L> {
        PictureSetRange::new(self, &self.picture_layers_with_worklets_)
    }
    pub fn SetPictureLayerWithWorklet(&mut self, layer_id: i32) {
        debug_assert!(self.contains(layer_id));
        debug_assert!(self
            .find(layer_id)
            .is_some_and(|layer| layer.AsPicture().is_some()));
        self.picture_layers_with_worklets_.insert(layer_id);
    }
    pub fn RemovePictureLayerWithWorklet(&mut self, layer_id: i32) {
        self.picture_layers_with_worklets_.remove(&layer_id);
    }

    // Source: cc/layers/layer_collections.h:175,183-187;
    //         cc/layers/layer_collections.cc:158-174.
    pub fn ReleaseLayerForTesting(&mut self, layer_id: i32) -> Option<Box<L>> {
        self.ReleaseLayer(layer_id)
    }
    pub(crate) fn ReleaseLayer(&mut self, layer_id: i32) -> Option<Box<L>> {
        let index = self.find_index(layer_id)?;
        self.layers_[index].take()
    }

    // Source: cc/layers/layer_collections.h:193-198;
    //         cc/layers/layer_collections.cc:133-156.
    fn EnsureLayerMaps(&self) {
        if !self
            .maps_
            .read()
            .expect("layer map lock poisoned")
            .layer_maps_need_rebuild
        {
            return;
        }
        self.RebuildLayerMaps();
    }
    fn RebuildLayerMaps(&self) {
        Self::RebuildLayerMapsForFields(&self.layers_, self.num_picture_layers_, &self.maps_);
    }
    // Source: cc/layers/layer_collections.cc:133-156. Same body for resident
    // owner and scoped field view; cache construction remains native and lazy.
    fn RebuildLayerMapsForFields(
        layers: &VectorType<L>,
        num_picture_layers: u32,
        maps: &RwLock<LayerMaps>,
    ) {
        let mut layer_indices_vector = Vec::with_capacity(layers.len());
        let mut picture_layer_indices_vector = Vec::with_capacity(num_picture_layers as usize);
        for (index, slot) in layers.iter().enumerate() {
            let layer = slot
                .as_deref()
                .expect("maps are rebuilt before recycle slots are released");
            let layer_id = layer.id();
            layer_indices_vector.push((layer_id, index));
            if layer.AsPicture().is_some() {
                picture_layer_indices_vector.push((layer_id, index));
            }
        }
        layer_indices_vector.sort_unstable_by_key(|(id, _)| *id);
        picture_layer_indices_vector.sort_unstable_by_key(|(id, _)| *id);
        let mut maps = maps.write().expect("layer map lock poisoned");
        maps.layer_map = layer_indices_vector.into_iter().collect();
        maps.picture_layer_map = picture_layer_indices_vector.into_iter().collect();
        maps.layer_maps_need_rebuild = false;
    }
}

// Source: cc/layers/layer_collections.h:86-126,128-159;
//         cc/layers/layer_collections.cc:176-216.
// Source iterator caches raw pointers. Rust instead resolves each stable id or
// index against the resident list on demand, so no layer-state snapshot or
// backpointer is retained.
pub struct LayerSetRange<'a, L: LayerCollectionEntry> {
    list: &'a OwnedLayerImplList<L>,
    ids: &'a SetType,
}
impl<'a, L: LayerCollectionEntry> LayerSetRange<'a, L> {
    fn new(list: &'a OwnedLayerImplList<L>, ids: &'a SetType) -> Self {
        Self { list, ids }
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}
impl<'a, L: LayerCollectionEntry> IntoIterator for LayerSetRange<'a, L> {
    type Item = &'a L;
    type IntoIter = LayerSetIterator<'a, L>;
    fn into_iter(self) -> Self::IntoIter {
        LayerSetIterator {
            list: self.list,
            ids: self.ids.iter(),
        }
    }
}
pub struct LayerSetIterator<'a, L: LayerCollectionEntry> {
    list: &'a OwnedLayerImplList<L>,
    ids: std::collections::btree_set::Iter<'a, i32>,
}
impl<'a, L: LayerCollectionEntry> Iterator for LayerSetIterator<'a, L> {
    type Item = &'a L;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.find(*self.ids.next()?)
    }
}

impl<'a, L: LayerCollectionEntry> DoubleEndedIterator for LayerSetIterator<'a, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.find(*self.ids.next_back()?)
    }
}
pub struct PictureSetRange<'a, L: LayerCollectionEntry> {
    list: &'a OwnedLayerImplList<L>,
    ids: &'a SetType,
}
impl<'a, L: LayerCollectionEntry> PictureSetRange<'a, L> {
    fn new(list: &'a OwnedLayerImplList<L>, ids: &'a SetType) -> Self {
        Self { list, ids }
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}
impl<'a, L: LayerCollectionEntry> IntoIterator for PictureSetRange<'a, L> {
    type Item = &'a L::Picture;
    type IntoIter = PictureSetIterator<'a, L>;
    fn into_iter(self) -> Self::IntoIter {
        PictureSetIterator {
            list: self.list,
            ids: self.ids.iter(),
        }
    }
}
pub struct PictureSetIterator<'a, L: LayerCollectionEntry> {
    list: &'a OwnedLayerImplList<L>,
    ids: std::collections::btree_set::Iter<'a, i32>,
}
impl<'a, L: LayerCollectionEntry> Iterator for PictureSetIterator<'a, L> {
    type Item = &'a L::Picture;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.find(*self.ids.next()?)?.AsPicture()
    }
}

pub struct PictureMapRange<'a, L: LayerCollectionEntry> {
    list: &'a OwnedLayerImplList<L>,
}
impl<'a, L: LayerCollectionEntry> PictureMapRange<'a, L> {
    fn new(list: &'a OwnedLayerImplList<L>) -> Self {
        Self { list }
    }
    pub fn len(&self) -> usize {
        self.list.num_picture_layers_ as usize
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<'a, L: LayerCollectionEntry> IntoIterator for PictureMapRange<'a, L> {
    type Item = &'a L::Picture;
    type IntoIter = PictureMapIterator<'a, L>;
    fn into_iter(self) -> Self::IntoIter {
        PictureMapIterator {
            list: self.list,
            previous_id: None,
        }
    }
}
pub struct PictureMapIterator<'a, L: LayerCollectionEntry> {
    list: &'a OwnedLayerImplList<L>,
    previous_id: Option<i32>,
}
impl<'a, L: LayerCollectionEntry> Iterator for PictureMapIterator<'a, L> {
    type Item = &'a L::Picture;
    fn next(&mut self) -> Option<Self::Item> {
        use std::ops::Bound::{Excluded, Unbounded};
        let (id, index) = {
            let maps = self.list.maps_.read().expect("layer map lock poisoned");
            match self.previous_id {
                None => maps.picture_layer_map.first_key_value(),
                Some(id) => maps
                    .picture_layer_map
                    .range((Excluded(id), Unbounded))
                    .next(),
            }
            .map(|(id, index)| (*id, *index))?
        };
        self.previous_id = Some(id);
        self.list.layers_[index].as_deref()?.AsPicture()
    }
}

// Source: cc/layers/layer_collections.h:201-211.
pub type LayerList<L> = Vec<Arc<L>>;
pub type LayerImplList<'a, L> = Vec<&'a L>;
pub type RenderSurfaceList = Vec<i32>;
pub type LayerImplRange<'a, L> = LayerSetRange<'a, L>;
pub type PictureLayerImplRange<'a, L> = PictureMapRange<'a, L>;

// Source: cc/layers/layer_collections.cc:19-26.
impl<L: LayerCollectionEntry> Drop for OwnedLayerImplList<L> {
    fn drop(&mut self) {
        self.clear();
    }
}

// Source ownership expression: cc/layers/layer_collections.h:189-198.
// These two views split the existing owner fields only. They allocate nothing,
// hold no copied indexes/flags, and cannot escape the synchronous callback.
pub struct LayerStorage<'a, L: LayerCollectionEntry> {
    layers: &'a mut VectorType<L>,
    num_picture_layers: &'a u32,
    maps: &'a RwLock<LayerMaps>,
}
pub struct LayerTracking<'a> {
    pub should_push_properties: &'a mut BTreeSet<i32>,
    pub with_animated_images: &'a mut BTreeSet<i32>,
    pub with_worklets: &'a mut BTreeSet<i32>,
}
impl<L: LayerCollectionEntry> OwnedLayerImplList<L> {
    pub fn WithStorageAndTracking<Output>(
        &mut self,
        callback: impl FnOnce(&mut LayerStorage<'_, L>, &mut LayerTracking<'_>) -> Output,
    ) -> Output {
        let mut storage = LayerStorage {
            layers: &mut self.layers_,
            num_picture_layers: &self.num_picture_layers_,
            maps: &self.maps_,
        };
        let mut tracking = LayerTracking {
            should_push_properties: &mut self.layers_that_should_push_properties_,
            with_animated_images: &mut self.picture_layers_with_animated_images_,
            with_worklets: &mut self.picture_layers_with_worklets_,
        };
        callback(&mut storage, &mut tracking)
    }
    pub fn ForEachPictureLayer(
        &mut self,
        callback: &mut dyn FnMut(&mut L::Picture) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()> {
        self.WithStorageAndTracking(|storage, _tracking| storage.ForEachPictureLayer(callback))
    }
}
impl<L: LayerCollectionEntry> LayerStorage<'_, L> {
    fn EnsureLayerMaps(&self) {
        if self
            .maps
            .read()
            .expect("layer map lock poisoned")
            .layer_maps_need_rebuild
        {
            OwnedLayerImplList::<L>::RebuildLayerMapsForFields(
                self.layers,
                *self.num_picture_layers,
                self.maps,
            );
        }
    }
    pub fn iter(&self) -> LayerVectorIterator<'_, L> {
        self.layers.iter().map(LayerSlot::<L>)
    }
    pub fn iter_mut(&mut self) -> LayerVectorIteratorMut<'_, L> {
        self.layers.iter_mut().map(LayerSlotMut::<L>)
    }
    pub fn find_mut(&mut self, id: i32) -> Option<&mut L> {
        self.EnsureLayerMaps();
        let index = *self
            .maps
            .read()
            .expect("layer map lock poisoned")
            .layer_map
            .get(&id)?;
        self.layers[index].as_deref_mut()
    }
    pub fn ForEachPictureLayer(
        &mut self,
        callback: &mut dyn FnMut(&mut L::Picture) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()> {
        use std::ops::Bound::{Excluded, Unbounded};
        self.EnsureLayerMaps();
        let mut previous = None;
        loop {
            let next = {
                let maps = self.maps.read().expect("layer map lock poisoned");
                match previous {
                    None => maps.picture_layer_map.first_key_value(),
                    Some(id) => maps
                        .picture_layer_map
                        .range((Excluded(id), Unbounded))
                        .next(),
                }
                .map(|(id, index)| (*id, *index))
            };
            let Some((id, index)) = next else {
                return std::ops::ControlFlow::Continue(());
            };
            previous = Some(id);
            let layer = self.layers[index]
                .as_deref_mut()
                .expect("source live picture layer slot");
            callback(layer.AsPictureMut().expect("source ToPictureLayerImpl"))?;
        }
    }
}

/// A borrow of the same canonical vector with exactly one resident slot lent
/// separately. Native id-to-index map remains the owner's single live map.
/// Looking up the selected id must explicitly use the current receiver; it is
/// never represented as absent and never duplicated into another layer table.
pub struct LayerStorageRemaining<'a, L: LayerCollectionEntry> {
    before: &'a mut [Option<Box<L>>],
    after: &'a mut [Option<Box<L>>],
    // A detached layer is not in the resident vector, so every resident slot
    // belongs to `before` and there is no index adjustment after it.
    selected_index: Option<usize>,
    selected_id: i32,
    maps: &'a RwLock<LayerMaps>,
}
impl<L: LayerCollectionEntry> LayerStorage<'_, L> {
    pub fn WithLayer<Output>(
        &mut self,
        id: i32,
        callback: impl FnOnce(&mut L, LayerStorageRemaining<'_, L>) -> Output,
    ) -> Output {
        self.EnsureLayerMaps();
        let index = *self
            .maps
            .read()
            .expect("layer map lock poisoned")
            .layer_map
            .get(&id)
            .expect("source live selected layer");
        let (before, selected_and_after) = self.layers.split_at_mut(index);
        let (selected, after) = selected_and_after
            .split_first_mut()
            .expect("source selected index");
        callback(
            selected.as_deref_mut().expect("source live selected slot"),
            LayerStorageRemaining {
                before,
                after,
                selected_index: Some(index),
                selected_id: id,
                maps: self.maps,
            },
        )
    }

    /// Lend a layer whose native owner has already been removed from this
    /// collection while preserving access to every still-resident sibling.
    /// This is the Rust expression of a C++ destructor running after the
    /// owning `unique_ptr` was erased from `LayerList`.
    pub fn WithDetachedLayer<Output>(
        &mut self,
        layer: &mut L,
        callback: impl FnOnce(&mut L, LayerStorageRemaining<'_, L>) -> Output,
    ) -> Output {
        self.EnsureLayerMaps();
        let id = layer.id();
        assert!(
            !self
                .maps
                .read()
                .expect("layer map lock poisoned")
                .layer_map
                .contains_key(&id),
            "detached layer must not remain in the resident collection"
        );
        callback(
            layer,
            LayerStorageRemaining {
                before: self.layers,
                after: &mut [],
                selected_index: None,
                selected_id: id,
                maps: self.maps,
            },
        )
    }
}
impl<L: LayerCollectionEntry> LayerStorageRemaining<'_, L> {
    pub fn selected_id(&self) -> i32 {
        self.selected_id
    }
    pub fn find(&self, id: i32) -> Option<&L> {
        assert_ne!(
            id, self.selected_id,
            "selected layer must use the explicit live receiver"
        );
        let index = *self
            .maps
            .read()
            .expect("layer map lock poisoned")
            .layer_map
            .get(&id)?;
        if let Some(selected_index) = self.selected_index {
            if index < selected_index {
                self.before[index].as_deref()
            } else {
                self.after[index - selected_index - 1].as_deref()
            }
        } else {
            self.before[index].as_deref()
        }
    }
    pub fn find_mut(&mut self, id: i32) -> Option<&mut L> {
        assert_ne!(
            id, self.selected_id,
            "selected layer must use the explicit live receiver"
        );
        let index = *self
            .maps
            .read()
            .expect("layer map lock poisoned")
            .layer_map
            .get(&id)?;
        if let Some(selected_index) = self.selected_index {
            if index < selected_index {
                self.before[index].as_deref_mut()
            } else {
                self.after[index - selected_index - 1].as_deref_mut()
            }
        } else {
            self.before[index].as_deref_mut()
        }
    }
    /// Scoped source pointer resolution. The selected instance and every other
    /// instance are borrowed from their original, disjoint resident slot.
    pub fn WithCurrentLayer<Output>(
        &mut self,
        current: &mut L,
        id: i32,
        callback: impl FnOnce(&mut L) -> Output,
    ) -> Option<Output> {
        assert_eq!(
            current.id(),
            self.selected_id,
            "source current receiver identity"
        );
        if id == self.selected_id {
            Some(callback(current))
        } else {
            self.find_mut(id).map(callback)
        }
    }
}

impl LayerTracking<'_> {
 pub fn reborrow(&mut self)->LayerTracking<'_> {LayerTracking{should_push_properties:&mut *self.should_push_properties,with_animated_images:&mut *self.with_animated_images,with_worklets:&mut *self.with_worklets}}
}
