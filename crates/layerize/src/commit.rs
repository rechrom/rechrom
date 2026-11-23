//! Commit vocabulary shared by the main (Layerize) and impl (compositor)
//! halves: the values that cross ProxyMain::BeginMainFrame ->
//! ProxyImpl::NotifyReadyToCommitOnImpl and the protected-sequence contract
//! under which LayerTreeHostImpl::FinishCommit reads main-thread state.
//! Nothing here names Blink paint types.

pub mod protected_sequence_synchronizer;

// cc property-tree identities and synchronized values used by both halves.
// Source: cc/base/synced_property.h; cc/trees/property_ids.h;
//         cc/trees/viewport_property_ids.h.
pub use compositor_tree::{property_ids, synced_property, viewport_property_ids};
