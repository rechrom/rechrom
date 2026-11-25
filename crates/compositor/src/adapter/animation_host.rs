//! Call-scoped AnimationHost delegate capabilities (cc::MutatorHostClient,
//! cc/trees/mutator_host_client.h), shared with the main LayerTreeHost.
pub use layerize::commit::mutator_host::{
    AnimationHostQueries, AnimationMutationTarget, ScrollAnimationReentry,
};
