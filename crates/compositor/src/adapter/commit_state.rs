//! Owned external payload types and non-owning main-thread layer identities.
//! CommitState's fields, default/reset and persistent-copy policy live in the
//! translated file. Only the external concrete payload implementations vary.
use super::layer_tree_impl::{LayerTreeTypes, UIResourceId};
use crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState;
use std::cmp::Ordering;
// Source dependency: gfx::InsetsF native value (top,left,bottom,right).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InsetsF {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}
// Source dependency: cc/resources/ui_resource_client.h:18 UIResourceId=int.
impl PartialOrd for UIResourceId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for UIResourceId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
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
