//! Document supplement ownership for AnchorElementMetricsSender. The native
//! source class remains outside DOM; this handle owns that exact instance.
use super::*;
pub trait DocumentAnchorMetricsOwner {
    fn AddAnchorElement(&self, anchor: u64);
    fn RemoveAnchorElement(&self, anchor: u64);
    fn ContextDestroyed(&self);
}
impl PersistentDocument {
    pub fn AnchorElementMetricsOwner(&self) -> Option<Rc<dyn DocumentAnchorMetricsOwner>> {
        self.anchor_metrics_owner.borrow().clone()
    }
    pub fn InstallAnchorElementMetricsOwner(&self, owner: Rc<dyn DocumentAnchorMetricsOwner>) {
        assert!(
            self.anchor_metrics_owner.borrow().is_none(),
            "one source metrics sender per Document"
        );
        *self.anchor_metrics_owner.borrow_mut() = Some(owner);
    }
    pub(crate) fn DestroyAnchorMetricsOwner(&self) {
        if let Some(owner) = self.anchor_metrics_owner.borrow_mut().take() {
            owner.ContextDestroyed();
        }
    }
}
/// Source ShadowIncludingTreeOrderTraversal cursor. Only original node
/// identities are retained; callers release the Document guard before each
/// observer operation and resume from the same live tree.
pub struct ShadowIncludingAnchorMetricsCursor {
    root: DOMOwnerHandle,
    current: Option<DOMOwnerHandle>,
}
impl ShadowIncludingAnchorMetricsCursor {
    pub fn Next(&mut self, d: &PersistentDocument) -> Option<u64> {
        let root = d
            .FindNodeById(self.root.Id())
            .filter(|&n| d.NodeHandle(n) == self.root)?;
        while let Some(handle) = self.current.take() {
            let node = d
                .FindNodeById(handle.Id())
                .filter(|&n| d.NodeHandle(n) == handle)?;
            self.current = NextShadowIncluding(d, node, root).map(|n| d.NodeHandle(n));
            if d.arena[node].IsHTMLElement("a") || d.arena[node].IsHTMLElement("area") {
                return Some(handle.Id());
            }
        }
        None
    }
}
fn NextShadowIncluding(d: &PersistentDocument, node: usize, root: usize) -> Option<usize> {
    if let Some(shadow) = d.ShadowRoot(node) {
        return Some(shadow);
    }
    if let Some(&child) = d.arena[node].children.first() {
        return Some(child);
    }
    let mut current = node;
    loop {
        if current == root {
            return None;
        }
        if let Some(host) = d.ShadowHost(current) {
            if let Some(&child) = d.arena[host].children.first() {
                return Some(child);
            }
            current = host;
            continue;
        }
        if let Some(next) = d.arena[current].next_sibling {
            return Some(next);
        }
        current = d.arena[current].parent?;
    }
}
impl PersistentDocument {
    pub fn ShadowIncludingAnchorMetricsCursor(
        &self,
        document: u64,
    ) -> Option<ShadowIncludingAnchorMetricsCursor> {
        let root = self.FindNodeById(document)?;
        Some(ShadowIncludingAnchorMetricsCursor {
            root: self.NodeHandle(root),
            current: NextShadowIncluding(self, root, root).map(|n| self.NodeHandle(n)),
        })
    }
}
impl PersistentDocument {
    pub fn NextFlatAnchorMetricsNode(&self, node: u64, root: u64) -> Option<u64> {
        let mut current = self.FindNodeById(node)?;
        let root = self.FindNodeById(root)?;
        if let Some(&child) = self.FlatTreeChildren(current).first() {
            return Some(self.Node(child).Id());
        }
        while current != root {
            let parent = self.FlatTreeParent(current)?;
            let children = self.FlatTreeChildren(parent);
            let index = children
                .iter()
                .position(|&n| n == current)
                .expect("native flat child belongs to its parent");
            if let Some(&next) = children.get(index + 1) {
                return Some(self.Node(next).Id());
            }
            current = parent;
        }
        None
    }
}
