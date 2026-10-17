//! Host lifetime facts projected from DOM ownership. Portable callback handles
//! are retained only until their traced edges are committed. Engine-specific
//! values and GC operations stay in the realm on its owner thread.
use super::*;
use javascript::javascript_runtime::{HostObjectTraceChange as C, HostObjectTraceSource};
const PARENT: u64 = u64::MAX - 1;
const SHADOW_ROOT: u64 = u64::MAX - 2;
const SHADOW_HOST: u64 = u64::MAX - 3;
const TEMPLATE_CONTENT: u64 = u64::MAX - 4;
const COLLECTION_OWNER: u64 = u64::MAX - 5;
const OWNER_DOCUMENT: u64 = u64::MAX - 6;
pub(super) type LiveCollectionIds =
    HashMap<HostObjectId, HashMap<String, HashMap<String, HostObjectId>>>;
pub(super) struct DOMHostGraph {
    document: Rc<RefCell<DOM>>,
    streams: RefCell<
        HashMap<
            dom::persistent_document::DOMOwnerHandle,
            Rc<dom::node_references::NodeReferenceQueue>,
        >,
    >,
    pending: RefCell<Vec<C>>,
    collections: Rc<RefCell<HashMap<HostObjectId, NodeCollection>>>,
    live_ids: Rc<RefCell<LiveCollectionIds>>,
    iterators: Rc<RefCell<HashMap<HostObjectId, CollectionIterator>>>,
    records: Rc<RefCell<HashMap<HostObjectId, HashMap<String, HostValue>>>>,
    listeners: Rc<RefCell<HashMap<HostObjectId, HashMap<String, Vec<Listener>>>>>,
    events: Rc<RefCell<services::BoundEvents>>,
    retire: RefCell<Vec<HostObjectId>>,
    reaction_roots: RefCell<std::collections::HashSet<HostObjectId>>,
}
impl DOMHostGraph {
    pub(super) fn new(
        document: Rc<RefCell<DOM>>,
        collections: Rc<RefCell<HashMap<HostObjectId, NodeCollection>>>,
        live_ids: Rc<RefCell<LiveCollectionIds>>,
        iterators: Rc<RefCell<HashMap<HostObjectId, CollectionIterator>>>,
        records: Rc<RefCell<HashMap<HostObjectId, HashMap<String, HostValue>>>>,
        listeners: Rc<RefCell<HashMap<HostObjectId, HashMap<String, Vec<Listener>>>>>,
        events: Rc<RefCell<services::BoundEvents>>,
    ) -> Self {
        Self {
            document,
            collections,
            live_ids,
            iterators,
            records,
            listeners,
            events,
            streams: Default::default(),
            pending: Default::default(),
            retire: Default::default(),
            reaction_roots: Default::default(),
        }
    }
    pub(super) fn SetRoot(&self, id: HostObjectId, reason: u64, rooted: bool) {
        if id != 0 {
            self.pending
                .borrow_mut()
                .push(C::Root { id, reason, rooted });
        }
    }
    pub(super) fn ListenerReference(
        &self,
        owner: HostObjectId,
        slot: u64,
        function: Option<JavaScriptFunction>,
    ) {
        self.pending.borrow_mut().push(C::FunctionReference {
            owner,
            slot,
            function,
        });
    }
    pub(super) fn Collection(&self, id: HostObjectId, collection: &NodeCollection) {
        let mut pending = self.pending.borrow_mut();
        pending.push(C::Object { id });
        if collection.kind == "snapshot" {
            // StaticNodeList::Trace visits its HeapVector directly. Preserve
            // that native vector instead of expanding it into keyed writes.
            pending.push(C::ReferenceList {
                owner: id,
                targets: collection.snapshot.clone().into(),
            });
        } else {
            pending.push(C::Reference {
                owner: id,
                slot: COLLECTION_OWNER,
                target: Some(collection.root),
            });
            pending.push(C::Reference {
                owner: collection.root,
                slot: id,
                target: Some(id),
            });
        }
    }
    pub(super) fn Iterator(&self, id: HostObjectId, collection: HostObjectId) {
        self.pending.borrow_mut().extend([
            C::Object { id },
            C::Reference {
                owner: id,
                slot: COLLECTION_OWNER,
                target: Some(collection),
            },
        ]);
    }
    pub(super) fn Record(&self, id: HostObjectId, fields: &HashMap<String, HostValue>) {
        let mut pending = self.pending.borrow_mut();
        pending.push(C::Object { id });
        for (index, value) in fields.values().enumerate() {
            if let HostValue::Object(object) = value {
                if object.id != 0 {
                    pending.push(C::Reference {
                        owner: id,
                        slot: index as u64,
                        target: Some(object.id),
                    });
                }
            }
        }
    }
    fn RetireRecords(&self) {
        if self.retire.borrow().is_empty() {
            return;
        }
        let (
            Ok(mut collections),
            Ok(mut live_ids),
            Ok(mut iterators),
            Ok(mut records),
            Ok(mut listeners),
            Ok(mut events),
        ) = (
            self.collections.try_borrow_mut(),
            self.live_ids.try_borrow_mut(),
            self.iterators.try_borrow_mut(),
            self.records.try_borrow_mut(),
            self.listeners.try_borrow_mut(),
            self.events.try_borrow_mut(),
        )
        else {
            return;
        };
        for id in self.retire.borrow_mut().drain(..) {
            if let Some(kinds) = listeners.remove(&id) {
                for listener in kinds.into_values().flatten() {
                    listener.active.set(false);
                }
            }
            if let Some(collection) = collections.remove(&id) {
                if collection.kind != "snapshot" {
                    self.pending.borrow_mut().push(C::Reference {
                        owner: collection.root,
                        slot: id,
                        target: None,
                    });
                    if let Some(kinds) = live_ids.get_mut(&collection.root) {
                        if let Some(queries) = kinds.get_mut(&collection.kind) {
                            queries.remove(&collection.query);
                            if queries.is_empty() {
                                kinds.remove(&collection.kind);
                            }
                        }
                        if kinds.is_empty() {
                            live_ids.remove(&collection.root);
                        }
                    }
                }
            }
            events.Retire(id);
            iterators.remove(&id);
            records.remove(&id);
            self.pending.borrow_mut().push(C::Retire { id });
        }
    }
}
impl HostObjectTraceSource for DOMHostGraph {
    fn TakeChanges(&self) -> Vec<C> {
        self.RetireRecords();
        // Parser swaps transfer the actual arena during a JS turn. Subscribe
        // once per arena; its stream moves with the document while parsing.
        if let Ok(mut owner) = self.document.try_borrow_mut() {
            let document = owner.GetDocumentMut();
            let pending = document
                .PendingCustomElementReferenceRoots()
                .into_iter()
                .collect::<std::collections::HashSet<_>>();
            let mut roots = self.reaction_roots.borrow_mut();
            for &id in roots.difference(&pending) {
                self.SetRoot(id, 4, false);
            }
            for &id in pending.difference(&roots) {
                self.SetRoot(id, 4, true);
            }
            *roots = pending;
            let key = document.RootHandle();
            if !self.streams.borrow().contains_key(&key) {
                let stream = document.SubscribeNodeReferences();
                self.streams.borrow_mut().insert(key, stream);
            }
        }
        let mut changes = Vec::new();
        for stream in self.streams.borrow().values() {
            for change in stream.TakeChanges() {
                use dom::node_references::NodeReferenceChange as N;
                match change {
                    N::Object { id, document_root } => {
                        changes.push(C::Object { id });
                        if document_root {
                            changes.push(C::Root {
                                id,
                                reason: 0,
                                rooted: true,
                            });
                        }
                    }
                    N::OwnerDocument { node, document } => changes.push(C::Reference {
                        owner: node,
                        slot: OWNER_DOCUMENT,
                        target: Some(document),
                    }),
                    N::Child {
                        parent,
                        child,
                        attached,
                    } => {
                        changes.push(C::Reference {
                            owner: parent,
                            slot: child,
                            target: attached.then_some(child),
                        });
                        changes.push(C::Reference {
                            owner: child,
                            slot: PARENT,
                            target: attached.then_some(parent),
                        });
                    }
                    N::Shadow { host, root } => {
                        changes.push(C::Reference {
                            owner: host,
                            slot: SHADOW_ROOT,
                            target: Some(root),
                        });
                        changes.push(C::Reference {
                            owner: root,
                            slot: SHADOW_HOST,
                            target: Some(host),
                        });
                    }
                    N::Template { owner, content } => changes.push(C::Reference {
                        owner,
                        slot: TEMPLATE_CONTENT,
                        target: Some(content),
                    }),
                }
            }
        }
        changes.extend(self.events.borrow_mut().TakeChanges());
        changes.extend(self.pending.borrow_mut().drain(..));
        changes
    }
    fn Released(&self, ids: &[HostObjectId]) {
        // DOM storage stays with DOM. Binding records and listener metadata
        // can retire after their owner cell dies, outside collector callbacks.
        self.retire.borrow_mut().extend(
            ids.iter()
                .copied()
                .filter(|id| (*id >= 1 << 62) || self.listeners.borrow().contains_key(id)),
        );
    }
}
