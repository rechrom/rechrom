//! Owned external payload types and non-owning main-thread layer identities.
//! CommitState, its payload traits and its policy live in the shared commit
//! vocabulary; only the external concrete payload implementations vary.
pub use layerize::commit::commit_state::{CommitStateTypes, MainLayerListContext};
pub use layerize::commit::values::InsetsF;

/// The cc::DrawImage of an impl family's commit (LayerTreeHost::QueueImageDecode
/// -> LayerTreeHostImpl::QueueImageDecode, cc/trees/layer_tree_host_impl.cc:
/// 4442-4463): the main family's value.
pub type MainDrawImage<B> =
    <<B as super::layer_tree_impl::LayerTreeTypes>::Main as CommitStateTypes>::DrawImage;
