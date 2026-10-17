//! Native DOM reference changes, independent of JS engine or GC placement.
//! cpp: Node::Trace, ContainerNode::Trace, Element::Trace, ShadowRoot::Trace.
use std::{cell::RefCell, rc::Weak};

#[derive(Clone, Copy, Debug)]
pub enum NodeReferenceChange {
    Object {
        id: u64,
        document_root: bool,
    },
    OwnerDocument {
        node: u64,
        document: u64,
    },
    Child {
        parent: u64,
        child: u64,
        attached: bool,
    },
    Shadow {
        host: u64,
        root: u64,
    },
    Template {
        owner: u64,
        content: u64,
    },
}
#[derive(Default)]
pub struct NodeReferenceQueue {
    pending: RefCell<Vec<NodeReferenceChange>>,
}
impl NodeReferenceQueue {
    pub fn TakeChanges(&self) -> Vec<NodeReferenceChange> {
        std::mem::take(&mut *self.pending.borrow_mut())
    }
    pub(super) fn push(&self, change: NodeReferenceChange) {
        self.pending.borrow_mut().push(change);
    }
}
pub(super) type NodeReferenceObservers = Vec<Weak<NodeReferenceQueue>>;
