//! Owned external payload types and non-owning main-thread layer identities.
//! CommitState, its payload traits and its policy live in the shared commit
//! vocabulary; only the external concrete payload implementations vary.
pub use layerize::commit::commit_state::{CommitStateTypes, MainLayerListContext};
pub use layerize::commit::values::InsetsF;
