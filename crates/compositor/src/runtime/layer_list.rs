//! Native OwnedLayerImplList bindings. Iterator enums select the actual source
//! vector/set/map cursor; no identity list or layer state is materialized.
use super::layer::CompositorLayer;
use crate::{
    adapter::{
        input_types::ElementId,
        layer_impl::LayerImplOwner,
        layer_tree_impl::{LayerTreeLayerList, LayerTreeLayerStorage, LayerTreeTypes},
        picture_layer_impl::PictureLayerImplTypes,
        solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes,
        tree_sync::{SyncLayerList, SyncTypes},
    },
    cc::layers::{
        layer_collections::{
            self, LayerCollectionEntry, LayerCollectionPicture, OwnedLayerImplList,
        },
        layer_impl::LayerImpl,
        picture_layer_impl::PictureLayerImpl,
    },
};
use std::ops::ControlFlow;
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> LayerCollectionEntry
    for CompositorLayer<B>
{
    type Picture = PictureLayerImpl<B>;
    fn id(&self) -> i32 {
        self.impl_base().id()
    }
    fn GetChangedAllProperties(&self) -> bool {
        self.impl_base()
            .GetChangeFlag(LayerImpl::<B>::kChangedAllProperties)
    }
    fn AsPicture(&self) -> Option<&PictureLayerImpl<B>> {
        if let Self::Picture(layer) = self {
            Some(layer)
        } else {
            None
        }
    }
    fn AsPictureMut(&mut self) -> Option<&mut PictureLayerImpl<B>> {
        if let Self::Picture(layer) = self {
            Some(layer)
        } else {
            None
        }
    }
}
impl<B: PictureLayerImplTypes> LayerCollectionPicture for PictureLayerImpl<B> {
    fn id(&self) -> i32 {
        self.base().id()
    }
    fn GetPaintWorkletRecordMapSize(&self) -> usize {
        self.GetPaintWorkletRecordMap().len()
    }
    fn HasAnimatedImages(&self) -> bool {
        PictureLayerImpl::HasAnimatedImages(self)
    }
}
pub enum LayerIterator<'a, L: LayerCollectionEntry> {
    Vector(layer_collections::LayerVectorIterator<'a, L>),
    Tracked(layer_collections::LayerSetIterator<'a, L>),
}
impl<'a, L: LayerCollectionEntry> Iterator for LayerIterator<'a, L> {
    type Item = &'a L;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Vector(it) => it.next(),
            Self::Tracked(it) => it.next(),
        }
    }
}
impl<L: LayerCollectionEntry> DoubleEndedIterator for LayerIterator<'_, L> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Vector(it) => it.next_back(),
            Self::Tracked(it) => it.next_back(),
        }
    }
}
pub enum PictureIterator<'a, L: LayerCollectionEntry> {
    Map(layer_collections::PictureMapIterator<'a, L>),
    Set(layer_collections::PictureSetIterator<'a, L>),
}
impl<'a, L: LayerCollectionEntry> Iterator for PictureIterator<'a, L> {
    type Item = &'a L::Picture;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Map(it) => it.next(),
            Self::Set(it) => it.next(),
        }
    }
}
impl<B> LayerTreeLayerStorage<B> for layer_collections::LayerStorage<'_, CompositorLayer<B>>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<Layer = CompositorLayer<B>, PictureLayer = PictureLayerImpl<B>>,
{
    type Iter<'a>
        = layer_collections::LayerVectorIterator<'a, CompositorLayer<B>>
    where
        Self: 'a,
        B: 'a;
    type IterMut<'a>
        = layer_collections::LayerVectorIteratorMut<'a, CompositorLayer<B>>
    where
        Self: 'a,
        B: 'a;
    fn iter(&self) -> Self::Iter<'_> {
        layer_collections::LayerStorage::iter(self)
    }
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        layer_collections::LayerStorage::iter_mut(self)
    }
    fn LayerByIdMut(&mut self, id: i32) -> Option<&mut B::Layer> {
        self.find_mut(id)
    }
    fn ForEachPictureLayer(
        &mut self,
        callback: &mut dyn FnMut(&mut B::PictureLayer) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        layer_collections::LayerStorage::ForEachPictureLayer(self, callback)
    }
}
impl<B> LayerTreeLayerList<B> for OwnedLayerImplList<CompositorLayer<B>>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<Layer = CompositorLayer<B>, PictureLayer = PictureLayerImpl<B>>,
    CompositorLayer<B>: crate::adapter::layer_tree_impl::LayerTreeLayer<B>,
{
    type Iter<'a>
        = LayerIterator<'a, CompositorLayer<B>>
    where
        Self: 'a,
        B: 'a;
    type IterMut<'a>
        = layer_collections::LayerVectorIteratorMut<'a, CompositorLayer<B>>
    where
        Self: 'a,
        B: 'a;
    type PictureIter<'a>
        = PictureIterator<'a, CompositorLayer<B>>
    where
        Self: 'a,
        B: 'a;
    fn len(&self) -> usize {
        self.size()
    }
    fn is_empty(&self) -> bool {
        self.empty()
    }
    fn iter(&self) -> Self::Iter<'_> {
        LayerIterator::Vector(OwnedLayerImplList::iter(self))
    }
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        OwnedLayerImplList::iter_mut(self)
    }
    fn PictureLayers(&self) -> Self::PictureIter<'_> {
        PictureIterator::Map(OwnedLayerImplList::PictureLayers(self).into_iter())
    }
    fn ForEachPictureLayer(
        &mut self,
        callback: &mut dyn FnMut(&mut B::PictureLayer) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        OwnedLayerImplList::ForEachPictureLayer(self, callback)
    }
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(
            &mut dyn crate::adapter::layer_tree_impl::LayerTreeScrollbarGeometry<B>,
            &mut std::collections::BTreeSet<i32>,
        ),
    ) {
        use crate::adapter::layer_tree_impl::LayerTreeLayer;
        self.WithStorageAndTracking(|storage, tracking| {
            let scrollbar = storage
                .find_mut(id)
                .expect("registered scrollbar exists")
                .AsScrollbarLayerMut()
                .expect("registered layer is a scrollbar");
            callback(scrollbar, tracking.should_push_properties);
        });
    }
    fn LayerAt(&self, index: usize) -> &B::Layer {
        self.at(index)
    }
    fn LayerById(&self, id: i32) -> Option<&B::Layer> {
        self.find(id)
    }
    fn LayerByIdMut(&mut self, id: i32) -> Option<&mut B::Layer> {
        self.find_mut(id)
    }
    // Source LayerTreeImpl::LayerByElementId is an intentionally linear scan
    // (cc/trees/layer_tree_impl.cc:2023-2034), rather than a second index.
    fn LayerByElementId(&self, id: ElementId) -> Option<&B::Layer> {
        OwnedLayerImplList::iter(self).find(|layer| layer.impl_base().element_id() == id)
    }
    fn AddLayer(&mut self, layer: Box<B::Layer>) {
        self.push_back(layer)
    }
    fn Reserve(&mut self, count: usize) {
        self.reserve(count)
    }
    fn ReleaseLayerForTesting(&mut self, id: i32) -> Box<B::Layer> {
        OwnedLayerImplList::ReleaseLayerForTesting(self, id).expect("source existing testing layer")
    }
    fn SetPictureLayerWithAnimatedImages(&mut self, id: i32) {
        OwnedLayerImplList::SetPictureLayerWithAnimatedImages(self, id)
    }
    fn RemovePictureLayerWithAnimatedImages(&mut self, id: i32) {
        OwnedLayerImplList::RemovePictureLayerWithAnimatedImages(self, id)
    }
    fn HasPictureLayerWithAnimatedImages(&self, id: i32) -> bool {
        OwnedLayerImplList::PictureLayersWithAnimatedImages(self)
            .into_iter()
            .any(|layer| layer.base().id() == id)
    }
    fn SetPictureLayerWithWorklet(&mut self, id: i32) {
        OwnedLayerImplList::SetPictureLayerWithWorklet(self, id)
    }
    fn RemovePictureLayerWithWorklet(&mut self, id: i32) {
        OwnedLayerImplList::RemovePictureLayerWithWorklet(self, id)
    }
    fn HasPictureLayerWithWorklet(&self, id: i32) -> bool {
        OwnedLayerImplList::PictureLayersWithWorklets(self)
            .into_iter()
            .any(|layer| layer.base().id() == id)
    }
    fn PictureLayersWithAnimatedImages(&self) -> Self::PictureIter<'_> {
        PictureIterator::Set(OwnedLayerImplList::PictureLayersWithAnimatedImages(self).into_iter())
    }
    fn PictureLayersWithWorklets(&self) -> Self::PictureIter<'_> {
        PictureIterator::Set(OwnedLayerImplList::PictureLayersWithWorklets(self).into_iter())
    }
    fn LayersThatShouldPushProperties(&self) -> Self::Iter<'_> {
        LayerIterator::Tracked(OwnedLayerImplList::LayersThatShouldPushProperties(self).into_iter())
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.SetShouldPushProperties(id)
    }
    fn ClearLayersThatShouldPushProperties(&mut self) {
        self.ClearLayersShouldPushProperties()
    }
    fn RemainingLayerIds(&self) -> Vec<i32> {
        OwnedLayerImplList::RemainingLayerIds(self)
    }
}
impl<B> SyncLayerList<B> for OwnedLayerImplList<CompositorLayer<B>>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + SyncTypes
        + LayerTreeTypes<Layer = CompositorLayer<B>, PictureLayer = PictureLayerImpl<B>>,
    CompositorLayer<B>: crate::adapter::layer_tree_impl::LayerTreeLayer<B>,
{
    fn ReleaseLayer(&mut self, id: i32) -> Option<Box<B::Layer>> {
        OwnedLayerImplList::ReleaseLayer(self, id)
    }
}

impl<B> crate::adapter::layer_tree_impl::LayerTreeRemainingStorage<B>
    for layer_collections::LayerStorageRemaining<'_, CompositorLayer<B>>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<Layer = CompositorLayer<B>>,
    CompositorLayer<B>: crate::adapter::layer_tree_impl::LayerTreeLayer<B>,
{
    fn LayerObjectIdentity(&self, id: i32) -> Option<usize> {
        self.find(id)
            .map(|layer| layer as *const super::layer::CompositorLayer<B> as usize)
    }

    fn selected_id(&self) -> i32 {
        self.selected_id()
    }
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(
            &mut dyn crate::adapter::layer_tree_impl::LayerTreeScrollbarGeometry<B>,
        ),
    ) {
        use crate::adapter::layer_tree_impl::LayerTreeLayer;
        callback(
            self.find_mut(id)
                .expect("native registered scrollbar exists")
                .AsScrollbarLayerMut()
                .expect("native registered scrollbar subtype"),
        );
    }
}
