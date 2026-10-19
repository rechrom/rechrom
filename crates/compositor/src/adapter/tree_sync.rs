//! Live Layer/LayerList ownership services for TreeSynchronizer. ReleaseLayer
//! transfers the existing owner object; CreateLayerImpl stays on the Layer
//! owner. Only stable identities, never copied layer state, cross this boundary.
use super::{
    commit_state::CommitStateTypes,
    layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList, PropertyTreesFor},
};
use crate::cc::trees::{
    commit_state::{CommitState, ThreadUnsafeCommitState},
    layer_tree_impl::LayerTreeImpl,
};
pub trait SyncTypes:
    CommitStateTypes + super::main_layer::MainLayerTypes + super::layer_impl::LayerImplTypes
{
    type MainLayer: SyncMainLayer<Self>;
}
pub trait SyncMainLayer<B: SyncTypes> {
    fn id(&self, c: &super::main_layer::LayerContext<'_, B>) -> i32;
    fn transform_tree_index_is_valid(
        &self,
        trees: &PropertyTreesFor<B>,
        c: &super::main_layer::LayerContext<'_, B>,
    ) -> bool;
    fn effect_tree_index_is_valid(
        &self,
        trees: &PropertyTreesFor<B>,
        c: &super::main_layer::LayerContext<'_, B>,
    ) -> bool;
    fn clip_tree_index_is_valid(
        &self,
        trees: &PropertyTreesFor<B>,
        c: &super::main_layer::LayerContext<'_, B>,
    ) -> bool;
    fn scroll_tree_index_is_valid(
        &self,
        trees: &PropertyTreesFor<B>,
        c: &super::main_layer::LayerContext<'_, B>,
    ) -> bool;
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        c: &mut super::main_layer::LayerContext<'_, B>,
    ) -> Box<B::Layer>;
    fn PushPropertiesTo(
        &self,
        target: &mut B::Layer,
        commit: &mut CommitState<B>,
        c: &mut super::main_layer::LayerContext<'_, B>,
        impl_context: &mut B::ImplContext<'_>,
    );
}
pub trait SyncImplLayer<B: SyncTypes>: LayerTreeLayer<B> {
    fn scroll_tree_index(&self) -> i32;
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        c: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    ) -> Box<B::Layer>;
    fn MovePropertiesToActiveLayer(
        &mut self,
        target: &mut B::Layer,
        source: &B::LayerSourceContext<'_>,
        target_context: &mut B::LayerContext<'_>,
    );
}
pub trait SyncPictureLayer {
    fn id(&self) -> i32;
}
pub trait SyncLayerList<B: SyncTypes>: LayerTreeLayerList<B> {
    fn ReleaseLayer(&mut self, id: i32) -> Option<Box<B::Layer>>;
}
pub trait SynchronizationTrace {
    fn TraceScope(&self, name: &str, count: Option<usize>);
}
pub trait SynchronizationContext<B: SyncTypes>: SynchronizationTrace {
    /// Scoped native LayerListConstIterator walk. Each callback gets the actual
    /// owner and current main context; neither reference may escape the call.
    fn ForEachMainLayer(
        &mut self,
        state: &ThreadUnsafeCommitState<B>,
        callback: impl FnMut(&B::MainLayer, &mut super::main_layer::LayerContext<'_, B>),
    );
    fn WithMainLayer<Output>(
        &mut self,
        identity: B::MainLayerRef,
        callback: impl FnOnce(&B::MainLayer, &mut super::main_layer::LayerContext<'_, B>) -> Output,
    ) -> Output;
}
