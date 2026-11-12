//! MutationObserverAgentData: native delegates and script observers share one
//! record owner, creation priority and notify microtask on the JS Agent queue.
//! Chromium 6c1d401fcca5: mutation_observer.cc:60-147,248-310,355-395.
use super::*;
use dom::mutation_observers::{
    CapturedMutationData, CapturedMutationRecord, MutationObserverId, MutationObserverOptions,
    MutationRecordSink,
};
use javascript::javascript_runtime::{JavaScriptAgentMicrotasks, WeakJavaScriptFunction};
use std::{collections::BTreeSet, rc::Weak};

// Existing root reasons 3/4/5 are focused/active-event/currentScript. This
// document's pending MutationRecords use their own counted activity reason.
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

const MUTATION_RECORD_ROOT: u64 = 6;
struct MutationRoots {
    graph: Option<Rc<DOMHostGraph>>,
    counts: RefCell<HashMap<u64, usize>>,
}
struct RecordRootLease {
    roots: Rc<MutationRoots>,
    nodes: Vec<u64>,
}
impl Drop for RecordRootLease {
    fn drop(&mut self) {
        let mut counts = self.roots.counts.borrow_mut();
        for &node in &self.nodes {
            let count = counts.get_mut(&node).expect("balanced mutation node root");
            *count -= 1;
            if *count == 0 {
                counts.remove(&node);
                if let Some(graph) = &self.roots.graph {
                    graph.SetRoot(node, MUTATION_RECORD_ROOT, false);
                }
            }
        }
    }
}
impl MutationRoots {
    fn Retain(self: &Rc<Self>, target: u64, data: &CapturedMutationData) -> Rc<RecordRootLease> {
        let mut nodes = vec![target];
        if let CapturedMutationData::ChildList {
            added_nodes,
            removed_nodes,
            previous_sibling,
            next_sibling,
        } = data
        {
            nodes.extend(added_nodes);
            nodes.extend(removed_nodes);
            nodes.extend(previous_sibling);
            nodes.extend(next_sibling);
        }
        nodes.sort_unstable();
        nodes.dedup();
        let mut counts = self.counts.borrow_mut();
        for &node in &nodes {
            let count = counts.entry(node).or_default();
            if *count == 0 {
                if let Some(graph) = &self.graph {
                    graph.SetRoot(node, MUTATION_RECORD_ROOT, true);
                }
            }
            *count += 1;
        }
        Rc::new(RecordRootLease {
            roots: self.clone(),
            nodes,
        })
    }
}
#[derive(Clone)]
struct ObserverRecord {
    target: u64,
    data: Rc<CapturedMutationData>,
    old_value: bool,
    _roots: Rc<RecordRootLease>,
}
pub struct DOMNativeMutationObservers {
    self_: Weak<Self>,
    document: Weak<RefCell<DOM>>,
    document_identity: dom::persistent_document::DOMOwnerHandle,
    queue: RefCell<Option<Rc<dyn JavaScriptAgentMicrotasks>>>,
    script_delivery: RefCell<WeakJavaScriptFunction>,
    object_namespace: RefCell<WeakJavaScriptHostObjectNamespace>,
    queued: Cell<bool>,
    pending: RefCell<BTreeSet<MutationObserverId>>,
    records: RefCell<HashMap<MutationObserverId, Vec<ObserverRecord>>>,
    delegates: RefCell<HashMap<MutationObserverId, Weak<dyn NativeMutationObserverDelegate>>>,
    roots: Rc<MutationRoots>,
}
impl DOMNativeMutationObservers {
    /// Alias the existing Agent queue. Native clients must enqueue on this
    /// exact source owner, including scripted realms' native microtasks.
    pub fn AgentMicrotasks(&self) -> Option<Rc<dyn JavaScriptAgentMicrotasks>> {
        self.queue.borrow().clone()
    }

    pub(super) fn new(document: &Rc<RefCell<DOM>>, graph: Rc<DOMHostGraph>) -> Rc<Self> {
        Self::Create(document, Some(graph), None)
    }
    pub fn ForNativeDocument(
        document: &Rc<RefCell<DOM>>,
        queue: Rc<dyn JavaScriptAgentMicrotasks>,
    ) -> Rc<Self> {
        Self::Create(document, None, Some(queue))
    }
    fn Create(
        document: &Rc<RefCell<DOM>>,
        graph: Option<Rc<DOMHostGraph>>,
        queue: Option<Rc<dyn JavaScriptAgentMicrotasks>>,
    ) -> Rc<Self> {
        let identity = document.borrow().GetDocument().RootHandle();
        let result = Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            document: Rc::downgrade(document),
            document_identity: identity,
            queue: RefCell::new(queue),
            script_delivery: RefCell::new(Default::default()),
            object_namespace: RefCell::new(Default::default()),
            queued: Cell::new(false),
            pending: RefCell::new(BTreeSet::new()),
            records: RefCell::new(HashMap::new()),
            delegates: RefCell::new(HashMap::new()),
            roots: Rc::new(MutationRoots {
                graph,
                counts: RefCell::new(HashMap::new()),
            }),
        });
        let sink: Rc<dyn MutationRecordSink> = result.clone();
        document
            .borrow_mut()
            .GetDocumentMut()
            .InstallMutationRecordSink(Rc::downgrade(&sink));
        result
    }
    pub(super) fn BindRuntime(&self, realm: &JavaScriptRealm) {
        *self.queue.borrow_mut() = realm.NativeMicrotasks();
    }
    pub(super) fn BindHostObjectNamespace(&self, namespace: &JavaScriptHostObjectNamespace) {
        *self.object_namespace.borrow_mut() = namespace.Downgrade();
    }
    pub(super) fn BindScriptDelivery(&self, callback: &JavaScriptFunction) {
        *self.script_delivery.borrow_mut() = callback.Downgrade();
    }
    fn CurrentDocument(&self) -> Option<Rc<RefCell<DOM>>> {
        let document = self.document.upgrade()?;
        let current = document.borrow().GetDocument().RootHandle() == self.document_identity;
        current.then_some(document)
    }
    fn Document(&self) -> Rc<RefCell<DOM>> {
        self.CurrentDocument()
            .expect("observer belongs to its original live document")
    }
    fn Queue(&self, id: MutationObserverId) {
        self.pending.borrow_mut().insert(id);
        if self.queued.replace(true) {
            return;
        }
        let owner = self.self_.clone();
        let queue = self
            .queue
            .borrow()
            .clone()
            .expect("bind the canonical Agent queue before observing mutations");
        queue
            .Enqueue(Box::new(move |runtime| {
                if let Some(owner) = owner.upgrade() {
                    owner.Notify(runtime)
                }
            }))
            .expect("enqueue MutationObserver notify microtask");
    }
    fn Notify(&self, mut runtime: Option<&mut dyn JavaScriptHostRuntime>) {
        self.queued.set(false);
        if self
            .CurrentDocument()
            .is_none_or(|document| document.borrow().GetDocument().IsContextDestroyed())
        {
            self.pending.borrow_mut().clear();
            self.records.borrow_mut().clear();
            return;
        }
        let pending = std::mem::take(&mut *self.pending.borrow_mut());
        for id in pending {
            self.Document()
                .borrow_mut()
                .GetDocumentMut()
                .ClearTransientMutationObservers(id);
            if self.records.borrow().get(&id).is_none_or(Vec::is_empty) {
                continue;
            }
            let delegate = self.delegates.borrow().get(&id).cloned();
            if let Some(delegate) = delegate {
                let records = self.Take(id);
                if records.is_empty() {
                    continue;
                }
                if let Some(delegate) = delegate.upgrade() {
                    let projected = records
                        .iter()
                        .map(|record| NativeMutationRecord {
                            target: record.target,
                            data: match record.data.as_ref() {
                                CapturedMutationData::Attributes {
                                    name,
                                    namespace,
                                    old_value,
                                } => NativeMutationData::Attributes {
                                    name: name.clone(),
                                    namespace: namespace.clone(),
                                    old_value: record
                                        .old_value
                                        .then(|| old_value.clone())
                                        .flatten(),
                                },
                                CapturedMutationData::CharacterData { old_value } => {
                                    NativeMutationData::CharacterData {
                                        old_value: record
                                            .old_value
                                            .then(|| old_value.clone())
                                            .flatten(),
                                    }
                                }
                                CapturedMutationData::ChildList {
                                    added_nodes,
                                    removed_nodes,
                                    ..
                                } => NativeMutationData::ChildList {
                                    added_nodes: added_nodes.clone(),
                                    removed_nodes: removed_nodes.clone(),
                                },
                            },
                        })
                        .collect::<Vec<_>>();
                    delegate.Deliver(&projected);
                }
            } else {
                assert!(
                    runtime.is_some(),
                    "a native-only Agent cannot contain script observers"
                );
                let callback = self
                    .script_delivery
                    .borrow()
                    .Upgrade()
                    .expect("installed script observer delivery wrapper");
                let document = self.Document();
                let owner = document.borrow();
                let document_id = owner.GetDocument().Node(owner.GetDocument().Root()).Id();
                drop(owner);
                let document_value = HostValue::Object(HostObjectRef { id: document_id });
                let document_value = self.object_namespace.borrow().Upgrade().map_or_else(
                    || document_value.clone(),
                    |namespace| namespace.Wrap(document_value.clone()),
                );
                // The wrapper takes records at this observer's turn, after
                // preceding observers have run, exactly as source Deliver.
                runtime
                    .as_mut()
                    .expect("a native-only Agent cannot contain script observers")
                    .Call(
                        &callback,
                        &HostValue::Undefined(Default::default()),
                        &[HostValue::String(id.0.to_string()), document_value],
                    );
            }
        }
    }
    fn Take(&self, id: MutationObserverId) -> Vec<ObserverRecord> {
        self.records.borrow_mut().remove(&id).unwrap_or_default()
    }
    pub(super) fn Disconnect(&self, id: MutationObserverId) {
        if let Some(document) = self.CurrentDocument() {
            document
                .borrow_mut()
                .GetDocumentMut()
                .DisconnectMutationObserver(id);
        }
        self.records.borrow_mut().remove(&id);
    }
    pub(super) fn TakeRecordsValue(
        &self,
        bindings: &DOMJavaScriptBindings,
        id: MutationObserverId,
    ) -> HostValue {
        let records = Rc::new(self.Take(id));
        let mut items = Vec::with_capacity(records.len() + 1);
        items.push(("length".into(), HostValue::Number(records.len() as f64)));
        let null = || HostValue::Null(JavaScriptNull);
        for (index, record) in records.iter().enumerate() {
            let (kind, name, namespace, old, added, removed, previous, next) =
                match record.data.as_ref() {
                    CapturedMutationData::Attributes {
                        name,
                        namespace,
                        old_value,
                    } => (
                        "attributes",
                        HostValue::String(name.clone()),
                        if namespace.is_empty() {
                            null()
                        } else {
                            HostValue::String(namespace.clone())
                        },
                        if record.old_value {
                            old_value
                                .clone()
                                .map(HostValue::String16)
                                .unwrap_or_else(null)
                        } else {
                            null()
                        },
                        null(),
                        null(),
                        null(),
                        null(),
                    ),
                    CapturedMutationData::CharacterData { old_value } => (
                        "characterData",
                        null(),
                        null(),
                        if record.old_value {
                            old_value
                                .clone()
                                .map(HostValue::String16)
                                .unwrap_or_else(null)
                        } else {
                            null()
                        },
                        null(),
                        null(),
                        null(),
                        null(),
                    ),
                    CapturedMutationData::ChildList {
                        added_nodes,
                        removed_nodes,
                        previous_sibling,
                        next_sibling,
                    } => (
                        "childList",
                        null(),
                        null(),
                        null(),
                        if added_nodes.is_empty() {
                            null()
                        } else {
                            HostValue::Object(bindings.StaticNodeList(added_nodes.clone()))
                        },
                        if removed_nodes.is_empty() {
                            null()
                        } else {
                            HostValue::Object(bindings.StaticNodeList(removed_nodes.clone()))
                        },
                        previous_sibling
                            .map(|id| HostValue::Object(HostObjectRef { id }))
                            .unwrap_or_else(null),
                        next_sibling
                            .map(|id| HostValue::Object(HostObjectRef { id }))
                            .unwrap_or_else(null),
                    ),
                };
            items.push((
                index.to_string(),
                HostValue::Record(vec![
                    (
                        "target".into(),
                        HostValue::Object(HostObjectRef { id: record.target }),
                    ),
                    ("type".into(), HostValue::String(kind.into())),
                    ("attributeName".into(), name),
                    ("attributeNamespace".into(), namespace),
                    ("oldValue".into(), old),
                    ("addedNodes".into(), added),
                    ("removedNodes".into(), removed),
                    ("previousSibling".into(), previous),
                    ("nextSibling".into(), next),
                ]),
            ));
        }
        // These original source roots live until the current engine has
        // converted every record/node list into its traced JS objects.
        HostValue::Retained(records, Box::new(HostValue::Record(items)))
    }
}
impl MutationRecordSink for DOMNativeMutationObservers {
    fn Activate(&self, id: MutationObserverId) {
        self.Queue(id);
    }
    fn Enqueue(&self, record: CapturedMutationRecord) {
        let roots = self.roots.Retain(record.target, &record.data);
        let data = Rc::new(record.data);
        let mut ids = Vec::with_capacity(record.recipients.len());
        {
            let mut records = self.records.borrow_mut();
            for recipient in record.recipients {
                records
                    .entry(recipient.observer)
                    .or_default()
                    .push(ObserverRecord {
                        target: record.target,
                        data: data.clone(),
                        old_value: recipient.old_value,
                        _roots: roots.clone(),
                    });
                ids.push(recipient.observer);
            }
        }
        for id in ids {
            self.Queue(id);
        }
    }
}
struct DOMNativeMutationObserver {
    owner: Weak<DOMNativeMutationObservers>,
    id: MutationObserverId,
}
impl NativeMutationObserver for DOMNativeMutationObserver {
    fn Observe(&self, target: u64, options: NativeMutationObserverOptions) {
        let owner = self.owner.upgrade().expect("canonical observer owner");
        let document = owner.Document();
        let mut document = document.borrow_mut();
        let d = document.GetDocumentMut();
        let target = d
            .FindNodeById(target)
            .expect("native observer target belongs to this document");
        assert!(d.RegisterMutationObserver(
            self.id,
            target,
            MutationObserverOptions {
                attributes: options.attributes,
                attribute_old_value: options.attribute_old_value,
                attribute_filter: options
                    .attribute_filter
                    .map(|names| names.into_iter().map(str::to_owned).collect()),
                character_data: options.character_data,
                character_data_old_value: options.character_data_old_value,
                child_list: options.child_list,
                subtree: options.subtree,
            }
        ));
    }
    fn Disconnect(&self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.Disconnect(self.id);
        }
    }
}
impl Drop for DOMNativeMutationObserver {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.Disconnect(self.id);
            owner.delegates.borrow_mut().remove(&self.id);
        }
    }
}
impl NativeMutationObserverFactory for DOMNativeMutationObservers {
    fn Create(
        &self,
        delegate: Weak<dyn NativeMutationObserverDelegate>,
    ) -> Rc<dyn NativeMutationObserver> {
        assert!(
            self.queue.borrow().is_some(),
            "bind the document Agent before native observer construction"
        );
        let id = self
            .Document()
            .borrow_mut()
            .GetDocumentMut()
            .AllocateMutationObserverId();
        self.delegates.borrow_mut().insert(id, delegate);
        Rc::new(DOMNativeMutationObserver {
            owner: self.self_.clone(),
            id,
        })
    }
}
