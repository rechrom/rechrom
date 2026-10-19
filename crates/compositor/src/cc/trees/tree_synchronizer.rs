// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/tree_synchronizer.h:1-45; cc/trees/tree_synchronizer.cc:1-27.
#[cfg(debug_assertions)]
use super::property_ids::kInvalidPropertyNodeId;
use super::{
    commit_state::{CommitState, ThreadUnsafeCommitState},
    layer_tree_impl::LayerTreeImpl,
};
#[cfg(debug_assertions)]
use crate::adapter::layer_tree_impl::PropertyTreesFor;
use crate::adapter::{
    layer_tree_impl::{
        LayerTreeLayer, LayerTreeLayerContext, LayerTreeLayerList, LayerTreeLayerStorage,
    },
    tree_sync::*,
};
// Source: cc/trees/tree_synchronizer.cc:30-37.
#[cfg(debug_assertions)]
fn AssertValidMainPropertyTreeIndices<B: SyncTypes>(
    layer: &B::MainLayer,
    trees: &PropertyTreesFor<B>,
    c: &crate::adapter::main_layer::LayerContext<'_, B>,
) {
    debug_assert!(layer.transform_tree_index_is_valid(trees, c));
    debug_assert!(layer.effect_tree_index_is_valid(trees, c));
    debug_assert!(layer.clip_tree_index_is_valid(trees, c));
    debug_assert!(layer.scroll_tree_index_is_valid(trees, c));
}
// Source: cc/trees/tree_synchronizer.cc:39-45.
#[cfg(debug_assertions)]
fn AssertValidPropertyTreeIndices<B: SyncTypes>(layer: &B::Layer)
where
    B::Layer: SyncImplLayer<B>,
{
    debug_assert_ne!(layer.transform_tree_index(), kInvalidPropertyNodeId);
    debug_assert_ne!(layer.effect_tree_index(), kInvalidPropertyNodeId);
    debug_assert_ne!(layer.clip_tree_index(), kInvalidPropertyNodeId);
    debug_assert_ne!(layer.scroll_tree_index(), kInvalidPropertyNodeId);
}
// Source: cc/trees/tree_synchronizer.cc:47-53.
#[cfg(debug_assertions)]
fn LayerHasValidPropertyTreeIndices<B: SyncTypes>(layer: &B::Layer) -> bool
where
    B::Layer: SyncImplLayer<B>,
{
    layer.transform_tree_index() != kInvalidPropertyNodeId
        && layer.effect_tree_index() != kInvalidPropertyNodeId
        && layer.clip_tree_index() != kInvalidPropertyNodeId
        && layer.scroll_tree_index() != kInvalidPropertyNodeId
}
// Source: cc/trees/tree_synchronizer.cc:55-61.
#[cfg(debug_assertions)]
fn LayerWillPushProperties<B: SyncTypes>(tree: &LayerTreeImpl<B>, layer: &B::Layer) -> bool
where
    B::PictureLayer: SyncPictureLayer,
{
    tree.LayersThatShouldPushProperties()
        .any(|l| l.id() == layer.id())
        || tree.picture_layers().any(|l| l.id() == layer.id())
}
// Source: cc/trees/tree_synchronizer.h:18-41.
// The source deletes construction; an uninhabited Rust type has no constructor.
pub enum TreeSynchronizer {}
impl TreeSynchronizer {
    // Source: cc/trees/tree_synchronizer.cc:66-100.
    pub fn SynchronizeTreesFromMain<B: SyncTypes, C: SynchronizationContext<B>>(
        commit: &CommitState<B>,
        unsafe_state: &ThreadUnsafeCommitState<B>,
        pending: &mut LayerTreeImpl<B>,
        tree_context: &mut super::layer_tree_impl::LayerTreeContext<'_, B>,
        context: &mut C,
    ) where
        B::Layer: SyncImplLayer<B>,
        B::LayerList: SyncLayerList<B>,
    {
        #[cfg(not(debug_assertions))]
        let _ = commit;
        #[cfg(debug_assertions)]
        context.ForEachMainLayer(unsafe_state, |layer, c| {
            AssertValidMainPropertyTreeIndices::<B>(layer, &commit.property_trees, c);
        });
        context.TraceScope("TreeSynchronizer::SynchronizeTrees", None);
        let mut recycle = pending.DetachLayers();
        let mut new_layers = B::LayerList::default();
        new_layers.Reserve(unsafe_state.num_layers());
        context.ForEachMainLayer(unsafe_state, |sync, c| {
            let layer = match recycle.ReleaseLayer(sync.id(c)) {
                Some(layer) => layer,
                None => sync.CreateLayerImpl(pending, tree_context, c),
            };
            new_layers.AddLayer(layer);
        });
        pending.SwapLayers(new_layers);
        pending.DestroyDetachedLayers(recycle, tree_context);
        #[cfg(debug_assertions)]
        for layer in pending.begin() {
            debug_assert!(
                LayerHasValidPropertyTreeIndices::<B>(layer)
                    || commit
                        .layer_ids_that_should_push_properties
                        .contains(&layer.id())
            );
        }
    }
    // Source: cc/trees/tree_synchronizer.cc:102-135.
    pub fn SynchronizeTreesFromPending<B: SyncTypes>(
        pending: &LayerTreeImpl<B>,
        active: &mut LayerTreeImpl<B>,
        tree_context: &mut super::layer_tree_impl::LayerTreeContext<'_, B>,
    ) where
        B::Layer: SyncImplLayer<B>,
        B::LayerList: SyncLayerList<B>,
        B::PictureLayer: SyncPictureLayer,
    {
        #[cfg(debug_assertions)]
        for layer in pending.begin() {
            AssertValidPropertyTreeIndices::<B>(layer);
        }
        tree_context
            .host
            .DrawServices()
            .TraceScope("TreeSynchronizer::SynchronizeTrees", None);
        let mut recycle = active.DetachLayers();
        let mut new_layers = B::LayerList::default();
        new_layers.Reserve(pending.num_layers());
        for sync in pending.begin() {
            let layer = match recycle.ReleaseLayer(sync.id()) {
                Some(layer) => layer,
                None => sync.CreateLayerImpl(active, tree_context),
            };
            new_layers.AddLayer(layer);
        }
        active.SwapLayers(new_layers);
        active.DestroyDetachedLayers(recycle, tree_context);
        #[cfg(debug_assertions)]
        for layer in active.begin() {
            debug_assert!(
                LayerHasValidPropertyTreeIndices::<B>(layer)
                    || LayerWillPushProperties::<B>(
                        pending,
                        pending.LayerById(layer.id()).expect("pending layer")
                    )
            );
        }
    }
    // Source: cc/trees/tree_synchronizer.cc:137-153.
    pub fn PushLayerPropertiesFromPending<B: SyncTypes>(
        pending: &mut LayerTreeImpl<B>,
        active: &mut LayerTreeImpl<B>,
        tree_context: &mut super::layer_tree_impl::LayerTreeContext<'_, B>,
    ) where
        B::Layer: SyncImplLayer<B>,
    {
        // The original `auto layers = ...` copies a vector of pointers. This is the
        // same identity list, not a layer/tree snapshot; every mutation reborrows the
        // resident source and target owners.
        let layers: Vec<i32> = pending
            .LayersThatShouldPushProperties()
            .map(|l| l.id())
            .collect();
        let count = layers.len();
        tree_context.host.DrawServices().TraceScope(
            "TreeSynchronizer::PushLayerPropertiesTo.Active",
            Some(count),
        );
        for id in layers {
            B::WithLayerSyncContext(
                pending,
                active,
                tree_context,
                id,
                |source, source_context, target, target_context| {
                    source.MovePropertiesToActiveLayer(target, source_context, target_context);
                    source.ResetChangeTracking();
                },
            );
        }
        pending.ClearLayersThatShouldPushProperties();
    }
    // Source: cc/trees/tree_synchronizer.cc:155-169.
    pub fn PushLayerPropertiesFromMain<B: SyncTypes, C: SynchronizationContext<B>>(
        commit: &mut CommitState<B>,
        unsafe_state: &ThreadUnsafeCommitState<B>,
        tree: &mut LayerTreeImpl<B>,
        tree_context: &mut super::layer_tree_impl::LayerTreeContext<'_, B>,
        context: &mut C,
    ) where
        for<'a> B: crate::adapter::main_layer::MainLayerTypes<
            ImplContext<'a> = crate::adapter::layer_impl::LayerImplContext<'a, B>,
        >,
    {
        context.TraceScope(
            "TreeSynchronizer::PushLayerPropertiesTo.Main",
            Some(commit.layer_ids_that_should_push_properties.len()),
        );
        // Reborrow the live ordered set at each step so the callback can receive its
        // canonical CommitState &mut. Under the source iterator's validity contract,
        // this visits exactly the same ids in the same order without copying the set.
        let mut previous = None;
        loop {
            let next = match previous {
                None => commit
                    .layer_ids_that_should_push_properties
                    .iter()
                    .next()
                    .copied(),
                Some(id) => commit
                    .layer_ids_that_should_push_properties
                    .range((std::ops::Bound::Excluded(id), std::ops::Bound::Unbounded))
                    .next()
                    .copied(),
            };
            let Some(id) = next else {
                break;
            };
            let identity = unsafe_state.LayerById(id).expect("main layer");
            context.WithMainLayer(identity, |source, c| {
                // Native main Layer target setters use the same resident target
                // tree split as every Create/Push path; the layer stays in storage.
                B::WithLayerContext(
                    tree,
                    tree_context,
                    source.id(c),
                    |target, target_context| {
                        target_context.WithBase(|impl_context| {
                            source.PushPropertiesTo(target, commit, c, impl_context)
                        });
                    },
                );
            });
            previous = Some(id);
        }
    }
    // Source: cc/trees/tree_synchronizer.cc:171-186.
    pub fn SynchronizeLayerOrder<B: SyncTypes>(
        order: &[i32],
        tree: &mut LayerTreeImpl<B>,
    ) -> Result<(), String>
    where
        B::LayerList: SyncLayerList<B>,
    {
        let mut recycle = tree.DetachLayers();
        let mut new_layers = B::LayerList::default();
        new_layers.Reserve(order.len());
        for &id in order {
            let Some(layer) = recycle.ReleaseLayer(id) else {
                return Err("Invalid or duplicate layer ID".into());
            };
            new_layers.AddLayer(layer);
        }
        tree.SwapLayers(new_layers);
        Ok(())
    }
}
