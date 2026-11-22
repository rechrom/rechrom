//! Chromium core/dom/node_child_removal_tracker.h:41-78.
//! The document owns only the synchronous removal stack. A guard holds no DOM
//! borrow, so focus/blur listeners can reenter while the node is being removed.
use crate::persistent_document::{DOMOwnerHandle, PersistentDocument};
use std::{cell::RefCell, rc::Rc};

thread_local! {
    // C++ last_ is the synchronous DOM-thread stack, shared by documents.
    // Qualified node identities preserve adoption/reentry without raw pointers.
    static REMOVALS: Rc<RefCell<Vec<DOMOwnerHandle>>> = Rc::new(RefCell::new(Vec::new()));
}
pub struct NodeChildRemovalTracker {
    stack: Rc<RefCell<Vec<DOMOwnerHandle>>>,
    node: DOMOwnerHandle,
}
impl Drop for NodeChildRemovalTracker {
    fn drop(&mut self) {
        assert_eq!(
            self.stack.borrow_mut().pop(),
            Some(self.node),
            "child-removal scopes unwind in source stack order"
        );
    }
}
impl PersistentDocument {
    pub fn TrackChildRemoval(&self, node: usize) -> NodeChildRemovalTracker {
        let node = self.NodeHandle(node);
        let stack = REMOVALS.with(Clone::clone);
        stack.borrow_mut().push(node);
        NodeChildRemovalTracker { stack, node }
    }
    pub fn IsBeingRemoved(&self, node: usize) -> bool {
        let stack = REMOVALS.with(Clone::clone);
        let stack = stack.borrow();
        let mut ancestor = Some(node);
        while let Some(node) = ancestor {
            if stack.contains(&self.NodeHandle(node)) {
                return true;
            }
            ancestor = self.Node(node).Parent().or_else(|| self.ShadowHost(node));
        }
        false
    }
}
