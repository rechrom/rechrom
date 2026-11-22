//! Serialization's native child-replacement helpers. Source serialization.cc
//! 718-774, Chromium 6c1d401fcca5. These mutate original nodes and share the
//! existing mutation-observer, Range and attachment hooks.
use super::{DOMNodeType, PersistentDocument};
impl PersistentDocument {
    pub fn ReplaceChildrenWithTextUTF16(&mut self, container: usize, text: &[u16]) {
        // Source always creates a fresh Text, including for empty contents.
        let text = self.CreateTextUTF16(text);
        let removed = self.Node(container).Children().to_vec();
        self.CaptureChildListMutation(container, &[text], &removed, None, None);
        let _scope = self.SuppressCapturedChildListMutations(container);
        for child in removed {
            self.Remove(child);
        }
        self.AppendChild(container, text);
    }
    pub fn ReplaceChildrenWithFragment(&mut self, container: usize, fragment: usize) {
        assert_eq!(self.Node(fragment).Type(), DOMNodeType::kDocumentFragment);
        let added = self.Node(fragment).Children().to_vec();
        let removed = self.Node(container).Children().to_vec();
        if added.is_empty() && removed.is_empty() {
            return;
        }
        let _container_scope = self.SuppressCapturedChildListMutations(container);
        for &child in &removed {
            self.Remove(child);
        }
        if !added.is_empty() {
            self.CaptureChildListMutation(fragment, &[], &added, None, None);
            let _fragment_scope = self.SuppressCapturedChildListMutations(fragment);
            for &child in &added {
                self.Remove(child);
            }
        }
        // The fragment's removal record precedes the container scope's
        // combined replacement record (ContainerNode::ReplaceChild 809-839).
        drop(_container_scope);
        self.CaptureChildListMutation(container, &added, &removed, None, None);
        let _insertion_scope = self.SuppressCapturedChildListMutations(container);
        for child in added {
            self.AppendChild(container, child);
        }
    }
    pub(super) fn RemoveAllChildrenForInnerText(&mut self, element: usize) {
        let children = self.Node(element).Children().to_vec();
        if children.is_empty() {
            return;
        }
        self.CaptureChildListMutation(element, &[], &children, None, None);
        let _scope = self.SuppressCapturedChildListMutations(element);
        for child in children {
            self.Remove(child);
        }
    }
}
