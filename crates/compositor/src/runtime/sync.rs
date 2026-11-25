//! Call-scoped native TreeSynchronizer main-side access. The assembly root
//! lends PAC's actual context alongside the impl target context; cc Host never
//! discovers the main owner or retains a reverse reference to it.
use crate::{
    adapter::{
        main_layer::MainLayerHostId,
        tree_sync::{MainLayerContext, SyncMain, SyncMainLayer, SyncTypes, SynchronizationTrace},
    },
    cc::{
        layers::{
            layer::{LayerBorrow, LayerRef},
            layer_list_iterator::LayerListConstIterator,
        },
        trees::{
            commit_state::{CommitState, ThreadUnsafeCommitState},
            layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
        },
    },
};

impl<B: SyncTypes> SyncMainLayer<B> for LayerBorrow<SyncMain<B>> {
    fn id(&self, c: &MainLayerContext<'_, B>) -> i32 {
        self.WithSourceRef(c, |layer, _| layer.base().id())
    }
    fn transform_tree_index_is_valid(
        &self,
        p: &crate::adapter::layer_tree_impl::MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool {
        self.WithSourceRef(c, |layer, c| {
            layer.base().transform_tree_index_is_valid(p, c)
        })
    }
    fn effect_tree_index_is_valid(
        &self,
        p: &crate::adapter::layer_tree_impl::MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool {
        self.WithSourceRef(c, |layer, c| layer.base().effect_tree_index_is_valid(p, c))
    }
    fn clip_tree_index_is_valid(
        &self,
        p: &crate::adapter::layer_tree_impl::MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool {
        self.WithSourceRef(c, |layer, c| layer.base().clip_tree_index_is_valid(p, c))
    }
    fn scroll_tree_index_is_valid(
        &self,
        p: &crate::adapter::layer_tree_impl::MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool {
        self.WithSourceRef(c, |layer, c| layer.base().scroll_tree_index_is_valid(p, c))
    }
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        tc: &mut LayerTreeContext<'_, B>,
        c: &mut MainLayerContext<'_, B>,
    ) -> Box<B::Layer> {
        B::WithLayerImplFactory(tree, tc, |factory| {
            self.WithSource(c, |layer, c| layer.CreateLayerImpl(factory, c))
        })
    }
    fn PushPropertiesTo(
        &self,
        target: &mut B::Layer,
        commit: &mut CommitState<SyncMain<B>>,
        c: &mut MainLayerContext<'_, B>,
        ic: &mut crate::adapter::tree_sync::PushImplContext<'_, B>,
    ) {
        self.WithSource(c, |layer, c| layer.PushPropertiesTo(target, commit, c, ic));
    }
}

pub struct MainSynchronizationContext<'a, 'c, B: SyncTypes> {
    pub main: &'a mut MainLayerContext<'c, B>,
    pub host: MainLayerHostId,
    pub trace: &'a dyn SynchronizationTrace,
}
impl<B: SyncTypes> SynchronizationTrace for MainSynchronizationContext<'_, '_, B> {
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.trace.TraceScope(name, count)
    }
}
impl<B> crate::adapter::tree_sync::SynchronizationContext<B>
    for MainSynchronizationContext<'_, '_, B>
where
    B: SyncTypes<
        MainLayer = LayerBorrow<SyncMain<B>>,
        Main: crate::adapter::commit_state::CommitStateTypes<
            MainLayerRef = LayerBorrow<SyncMain<B>>,
            MainRootLayer = LayerRef<SyncMain<B>>,
        >,
    >,
{
    fn ForEachMainLayer(
        &mut self,
        state: &ThreadUnsafeCommitState<SyncMain<B>>,
        mut callback: impl FnMut(&B::MainLayer, &mut MainLayerContext<'_, B>),
    ) {
        let mut iterator = LayerListConstIterator::new(
            state.root_layer.as_ref().map(|layer| layer.borrow()),
            self.main,
        );
        while let Some(layer) = iterator.Dereference() {
            callback(layer, self.main);
            iterator.Increment(self.main);
        }
    }
    fn WithMainLayer<Output>(
        &mut self,
        identity: LayerBorrow<SyncMain<B>>,
        callback: impl FnOnce(&B::MainLayer, &mut MainLayerContext<'_, B>) -> Output,
    ) -> Output {
        callback(&identity, self.main)
    }
}

// The root passes its live main context explicitly through LayerTreeImpl's
// commit entry. No Host-to-main lookup/factory or persistent callback exists.
impl<B> crate::adapter::layer_tree_impl::LayerTreeSynchronizer<B>
    for crate::cc::trees::tree_synchronizer::TreeSynchronizer
where
    B: SyncTypes<
        MainLayer = LayerBorrow<SyncMain<B>>,
        Main: crate::adapter::commit_state::CommitStateTypes<
            MainLayerRef = LayerBorrow<SyncMain<B>>,
            MainRootLayer = LayerRef<SyncMain<B>>,
        >,
        CommitState = CommitState<SyncMain<B>>,
        UnsafeCommitState = ThreadUnsafeCommitState<SyncMain<B>>,
    >,
    B::Layer: crate::adapter::tree_sync::SyncImplLayer<B>,
    B::LayerList: crate::adapter::tree_sync::SyncLayerList<B>,
    for<'a> B::Layer: crate::adapter::main_layer::MainLayerImpl<
        SyncMain<B>,
        ImplContext<'a> = crate::adapter::layer_impl::LayerImplContext<'a, B>,
    >,
{
    type MainContext<'a, 'c>
        = MainSynchronizationContext<'a, 'c, B>
    where
        B: 'a + 'c,
        'c: 'a;
    fn SynchronizeTrees(
        commit: &mut CommitState<SyncMain<B>>,
        unsafe_state: &ThreadUnsafeCommitState<SyncMain<B>>,
        tree: &mut LayerTreeImpl<B>,
        context: &mut LayerTreeContext<'_, B>,
        main: &mut Self::MainContext<'_, '_>,
    ) {
        Self::SynchronizeTreesFromMain(commit, unsafe_state, tree, context, main);
    }
    fn PushLayerProperties(
        commit: &mut CommitState<SyncMain<B>>,
        unsafe_state: &ThreadUnsafeCommitState<SyncMain<B>>,
        tree: &mut LayerTreeImpl<B>,
        context: &mut LayerTreeContext<'_, B>,
        main: &mut Self::MainContext<'_, '_>,
        _discardable_image_updates: &mut Vec<i32>,
    ) {
        // Source cc155-169 pushes dirty fields. The caller's unique updater is used
        // by the following canonical CommitPendingRasterSource phase, not this walk.
        Self::PushLayerPropertiesFromMain(commit, unsafe_state, tree, context, main);
    }
}
