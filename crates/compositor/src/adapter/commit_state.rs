//! Owned external payload types and non-owning main-thread layer identities.
//! CommitState's fields, default/reset and persistent-copy policy live in the
//! translated file. Only the external concrete payload implementations vary.
use super::layer_tree_impl::LayerTreeTypes;
use crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState;
pub use layerize::commit::values::InsetsF;
pub trait CommitStateTypes: LayerTreeTypes<DebugState = LayerTreeDebugState> {
    type BeginMainFrameMetrics;
    type DrawImage;
    type MicroBenchmarkImpl;
    // `MainRootLayer` is the owner's scoped_refptr-equivalent strong handle.
    // The map and mutator fields are non-owning typed identities, like raw_ptr.
    type MainRootLayer: Clone;
    type MainLayerRef: Clone + Eq;
    type MutatorHostHandle: Copy;
}
pub trait MainLayerListContext<B: CommitStateTypes> {
    type Iterator<'a>
    where
        Self: 'a,
        B: 'a;
    fn LayerListConstIterator<'a>(
        &'a self,
        root: Option<&'a B::MainRootLayer>,
    ) -> Self::Iterator<'a>;
}
