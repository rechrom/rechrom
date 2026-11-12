// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// MutationObserver::Delegate is the native source client of the document's
// existing observer/agent microtask owner, never a second mutation scheduler.
use std::rc::{Rc, Weak};

pub struct NativeMutationObserverOptions {
    pub attributes: bool,
    pub attribute_old_value: bool,
    pub attribute_filter: Option<Vec<&'static str>>,
    pub character_data: bool,
    pub character_data_old_value: bool,
    pub child_list: bool,
    pub subtree: bool,
}
pub enum NativeMutationData {
    Attributes {
        name: String,
        namespace: String,
        old_value: Option<Rc<[u16]>>,
    },
    CharacterData {
        old_value: Option<Rc<[u16]>>,
    },
    ChildList {
        added_nodes: Vec<u64>,
        removed_nodes: Vec<u64>,
    },
}
pub struct NativeMutationRecord {
    pub target: u64,
    pub data: NativeMutationData,
}
pub trait NativeMutationObserverDelegate {
    fn Deliver(&self, records: &[NativeMutationRecord]);
}
pub trait NativeMutationObserver {
    fn Observe(&self, target: u64, options: NativeMutationObserverOptions);
    fn Disconnect(&self);
}
/// This capability must be the canonical document/agent observer owner. The
/// native widget host cannot implement it with a task or another record FIFO.
pub trait NativeMutationObserverFactory {
    fn Create(
        &self,
        delegate: Weak<dyn NativeMutationObserverDelegate>,
    ) -> Rc<dyn NativeMutationObserver>;
}
