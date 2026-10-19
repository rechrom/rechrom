//! Temporary live service facets until canonical ScrollTree/LayerTreeHostImpl
//! expose these source methods. Canonical owners implement these traits by
//! direct calls; no scrolling policy or copied node/tree state belongs here.
use crate::cc::trees::scroll_node::ScrollNode;
pub trait ScrollTree {
    fn CanRealizeScrollsOnActiveTree(&self, node: &ScrollNode) -> bool;
    fn CanRealizeScrollsOnPendingTree(&self, node: &ScrollNode) -> bool;
}
pub trait LayerTreeHostImpl {
    fn CommitsToActiveTree(&self) -> bool;
}
