//! Transient Node::DetachLayoutTree requests from the original removal owner.
//! Source: container_node.cc:1039-1065; element.cc:4894-4959;
//! container_node.cc:1448-1452. Only identities cross the DOM/Layout boundary.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentLayoutTreeDetachment {
    pub document: DOMOwnerHandle,
    pub nodes: Vec<u64>,
}
impl PersistentDocument {
    /// Capture the old flat tree before unlink/slot invalidation. Postorder
    /// retains the source's child-before-parent detach order, including native
    /// shadow/assigned children. Text roots also own LayoutObjects.
    pub(super) fn RecordRemovedLayoutTree(&mut self, root: usize) {
        if !self.IsActive() || !self.IsConnectedToDocument(root) {
            return;
        }
        let document = self.RootHandle();
        let mut pending = vec![(root, false)];
        let mut nodes = Vec::new();
        while let Some((node, finished)) = pending.pop() {
            if finished {
                if matches!(
                    self.arena[node].Type(),
                    DOMNodeType::kElement | DOMNodeType::kText
                ) {
                    nodes.push(self.arena[node].id);
                }
            } else {
                pending.push((node, true));
                let children = self.FlatTreeChildren(node);
                pending.extend(children.iter().rev().map(|&child| (child, false)));
            }
        }
        self.layout_tree_detachments
            .push(DocumentLayoutTreeDetachment { document, nodes });
    }
    pub fn TakeLayoutTreeDetachments(&mut self) -> Vec<DocumentLayoutTreeDetachment> {
        std::mem::take(&mut self.layout_tree_detachments)
    }
}

impl PersistentDocument {
    /// StyleEngine.cc:4861. This is an owning DOM class/tree query, not
    /// Page-side children or pseudo-name inference.
    pub fn DetachParentMayHaveFlatTreeChildren(&self, node: u64) -> Option<bool> {
        let node = self.FindNodeById(node)?;
        (self.Node(node).Type() == DOMNodeType::kElement).then(|| {
            !self.Node(node).Children().is_empty()
                || self.ShadowRoot(node).is_some()
                || self.IsSlotSupportingAssignment(node)
        })
    }
    fn DirtyForStyleRecalc(&self, node: usize) -> bool {
        self.style_state.all_dirty
            || self.style_state.style_recalc_dirty_nodes.contains(&node)
            || self.style_state.subtree_style_recalc_roots.contains(&node)
    }
    /// Node.cc1568-1615. Preserve the real style-parent chain, dirty-bit early
    /// exits, flat-tree membership and original display-lock stopping points.
    pub fn MarkAncestorsWithChildNeedsStyleRecalc(&mut self, id: u64) {
        let Some(node) = self.FindNodeById(id) else {
            return;
        };
        let parent = self.FlatTreeParentElement(node);
        let parent_dirty = parent.is_some_and(|parent| self.DirtyForStyleRecalc(parent));
        let mut ancestor = parent;
        while let Some(current) = ancestor {
            if self.style_state.child_needs_style_recalc.contains(&current) {
                break;
            }
            if !self.IsConnectedToDocument(current) {
                return;
            }
            self.style_state.child_needs_style_recalc.insert(current);
            if self.DirtyForStyleRecalc(current) || !self.DisplayLockShouldStyleChildren(current) {
                break;
            }
            ancestor = self.FlatTreeParentElement(current);
        }
        if !self.IsConnectedToDocument(node) || parent_dirty || self.FlatTreeParent(node).is_none()
        {
            return;
        }
        if self.DisplayLockLockedCount() > 0 {
            let mut current = ancestor;
            while let Some(ancestor) = current {
                if !self.DisplayLockShouldStyleChildren(ancestor) {
                    return;
                }
                current = self.FlatTreeParentElement(ancestor);
            }
        }
        self.style_state.detach_parent_recalc_roots.insert(node);
        // This is the existing targeted layout-tree attachment input, not a
        // layout flush. Rebuild revisits native whitespace/list-item flags.
        self.style_state.layout_tree_roots.insert(id);
        self.lifecycle.ScheduleVisualUpdate();
    }
}

/// Source LocalSVGResource notification interface. There is no default
/// implementation; the actual resource owner must register its original Rc.
pub trait PendingSVGResourceInvalidation {
    fn NotifyContentChanged(&self);
}
impl PersistentDocument {
    pub fn QueueSVGResourceInvalidation(
        &mut self,
        resource: Rc<dyn PendingSVGResourceInvalidation>,
    ) {
        let identity = Rc::as_ptr(&resource).cast::<()>() as usize;
        self.svg_resources_needing_invalidation
            .entry(identity)
            .or_insert(resource);
    }
    /// Document.cc3279-3285 swaps the live set before notifying clients. Page
    /// releases the DOM borrow before invoking the original resource owners.
    pub fn HasPendingSVGResourceInvalidations(&self) -> bool {
        !self.svg_resources_needing_invalidation.is_empty()
    }
    pub fn TakePendingSVGResourceInvalidations(
        &mut self,
    ) -> Vec<Rc<dyn PendingSVGResourceInvalidation>> {
        std::mem::take(&mut self.svg_resources_needing_invalidation)
            .into_values()
            .collect()
    }
}
