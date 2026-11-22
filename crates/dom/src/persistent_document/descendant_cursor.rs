//! A call-scoped NodeTraversal::InclusiveDescendantsOf cursor over the live
//! DOM arena. Its stack contains only the current ancestry/child indices.
use super::*;
pub struct DOMInclusiveDescendantCursor {
    document: DOMOwnerHandle,
    root: Option<usize>,
    stack: Vec<(usize, usize)>,
}
impl DOMInclusiveDescendantCursor {
    pub fn Next(&mut self, document: &PersistentDocument) -> Option<u64> {
        assert_eq!(
            self.document,
            document.RootHandle(),
            "same DOM traversal arena"
        );
        if let Some(root) = self.root.take() {
            self.stack.push((root, 0));
            return Some(document.Node(root).Id());
        }
        while let Some((node, index)) = self.stack.last_mut() {
            let child = document.Node(*node).Children().get(*index).copied();
            *index += 1;
            if let Some(child) = child {
                self.stack.push((child, 0));
                return Some(document.Node(child).Id());
            }
            self.stack.pop();
        }
        None
    }
}
impl PersistentDocument {
    pub fn InclusiveDescendantCursor(&self, node: usize) -> DOMInclusiveDescendantCursor {
        DOMInclusiveDescendantCursor {
            document: self.RootHandle(),
            root: Some(node),
            stack: Vec::new(),
        }
    }
}
