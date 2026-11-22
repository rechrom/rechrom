//! TypingCommand::DoApply cc:623-633. The scope crosses only the typed
//! boundary; it retains the resident ScrollAnchor suppression count itself.
use super::*;
pub use crate::internal::scroll_anchor::SuppressScrollAnchorScope;
impl LayoutEngine {
    pub fn SuppressScrollAnchoringForNode(
        &self,
        anchor_node: Option<u64>,
    ) -> SuppressScrollAnchorScope {
        let _scope = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let area = anchor_node
            .and_then(|id| self.GetLayoutTree()?.GetLayoutObjectForNode(id))
            .and_then(|object| unsafe { object.ContainingScrollContainerAnyAxisDefault().as_ref() })
            .and_then(|container| unsafe { container.GetScrollableArea().as_mut() });
        SuppressScrollAnchorScope::new(area)
    }
}
