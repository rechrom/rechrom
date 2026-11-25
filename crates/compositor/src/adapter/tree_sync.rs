//! Live Layer/LayerList ownership services for TreeSynchronizer. ReleaseLayer
//! transfers the existing owner object; CreateLayerImpl stays on the Layer
//! owner. Only stable identities, never copied layer state, cross this boundary.
use super::{
    commit_state::CommitStateTypes,
    layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList, MainPropertyTreesFor},
};
use crate::cc::trees::{
    commit_state::{CommitState, ThreadUnsafeCommitState},
    layer_tree_impl::LayerTreeImpl,
};
/// The main family of an impl family: the cc::Layer side of its commit.
pub type SyncMain<B> = <B as super::layer_tree_impl::LayerTreeTypes>::Main;
/// The main-side context of a main-layer call over the commit's main family.
pub type MainLayerContext<'a, B> = super::main_layer::LayerContext<'a, SyncMain<B>>;
/// The main layers are pushed from through the main family's own layer
/// traits (DerivedLayerTypes), never through LayerizeFamily (see
/// LayerTreeTypes::Main).
pub trait SyncTypes:
    super::layer_impl::LayerImplTypes
    + super::layer_tree_impl::LayerTreeTypes<
        Main: layerize::commit::layers::layer::DerivedLayerTypes,
        Layer: layerize::commit::layers::layer::MainLayerPushTarget<
            <Self as super::layer_tree_impl::LayerTreeTypes>::Main,
        >,
    >
{
    type MainLayer: SyncMainLayer<Self>;
    /// Lends the pending tree to main layers as the LayerImplFactory of
    /// Layer::CreateLayerImpl (cc/trees/tree_synchronizer.cc PushLayerList).
    fn WithLayerImplFactory<Output>(
        tree: &mut LayerTreeImpl<Self>,
        context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, Self>,
        callback: impl FnOnce(
            &mut dyn super::main_layer::LayerImplFactory<
                <Self as super::layer_tree_impl::LayerTreeTypes>::Layer,
            >,
        ) -> Output,
    ) -> Output;
}
/// The impl-tree borrow the push target writes through.
pub type PushImplContext<'a, B> = <<B as super::layer_tree_impl::LayerTreeTypes>::Layer as super::main_layer::MainLayerImpl<SyncMain<B>>>::ImplContext<'a>;
pub trait SyncMainLayer<B: SyncTypes> {
    fn id(&self, c: &MainLayerContext<'_, B>) -> i32;
    fn transform_tree_index_is_valid(
        &self,
        trees: &MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool;
    fn effect_tree_index_is_valid(
        &self,
        trees: &MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool;
    fn clip_tree_index_is_valid(
        &self,
        trees: &MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool;
    fn scroll_tree_index_is_valid(
        &self,
        trees: &MainPropertyTreesFor<B>,
        c: &MainLayerContext<'_, B>,
    ) -> bool;
    fn CreateLayerImpl(
        &self,
        tree: &mut LayerTreeImpl<B>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        c: &mut MainLayerContext<'_, B>,
    ) -> Box<B::Layer>;
    fn PushPropertiesTo(
        &self,
        target: &mut B::Layer,
        commit: &mut CommitState<SyncMain<B>>,
        c: &mut MainLayerContext<'_, B>,
        impl_context: &mut PushImplContext<'_, B>,
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
pub use layerize::commit::main_callbacks::SynchronizationTrace;
pub trait SynchronizationContext<B: SyncTypes>: SynchronizationTrace {
    /// Scoped native LayerListConstIterator walk. Each callback gets the actual
    /// owner and current main context; neither reference may escape the call.
    fn ForEachMainLayer(
        &mut self,
        state: &ThreadUnsafeCommitState<SyncMain<B>>,
        callback: impl FnMut(&B::MainLayer, &mut MainLayerContext<'_, B>),
    );
    fn WithMainLayer<Output>(
        &mut self,
        identity: <SyncMain<B> as CommitStateTypes>::MainLayerRef,
        callback: impl FnOnce(&B::MainLayer, &mut MainLayerContext<'_, B>) -> Output,
    ) -> Output;
}
