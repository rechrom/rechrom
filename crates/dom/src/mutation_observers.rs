// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Script-free registration and interest selection. Callback/record ownership
//! stays in the embedding; this document owner carries only opaque identities.
//! cpp: Node::GetRegisteredMutationObserversOfType;
//! MutationObserverRegistration::ShouldReceiveMutationFrom;
//! MutationObserverInterestGroup::CreateIfNeeded.
#![allow(non_snake_case)]

use crate::persistent_document::PersistentDocument;
use std::collections::{HashMap, HashSet};

// MutationObserver.cc:55,198. Creation order is shared across Documents;
// Window reuse must not recycle a JS-visible observer identity in the realm.
static NEXT_OBSERVER_PRIORITY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MutationObserverId(pub u64);

#[derive(Clone, Copy, Debug)]
pub enum MutationType {
    Attributes = 0,
    CharacterData = 1,
    ChildList = 2,
}

pub struct MutationObserverOptions {
    pub attributes: bool,
    pub character_data: bool,
    pub child_list: bool,
    pub subtree: bool,
    pub attribute_old_value: bool,
    pub character_data_old_value: bool,
    pub attribute_filter: Option<HashSet<String>>,
}
impl MutationObserverOptions {
    fn Includes(&self, kind: MutationType) -> bool {
        match kind {
            MutationType::Attributes => self.attributes,
            MutationType::CharacterData => self.character_data,
            MutationType::ChildList => self.child_list,
        }
    }
    fn WantsOldValue(&self, kind: MutationType) -> bool {
        match kind {
            MutationType::Attributes => self.attribute_old_value,
            MutationType::CharacterData => self.character_data_old_value,
            MutationType::ChildList => false,
        }
    }
}

#[derive(Clone)]
pub struct MutationObserverRecipient {
    pub observer: MutationObserverId,
    pub old_value: bool,
}

pub(crate) struct MutationObservers {
    nodes: HashMap<usize, HashMap<MutationObserverId, std::rc::Rc<MutationObserverOptions>>>,
    transient:
        HashMap<usize, HashMap<(usize, MutationObserverId), std::rc::Rc<MutationObserverOptions>>>,
    transient_nodes: HashMap<(usize, MutationObserverId), HashSet<usize>>,
    targets: HashMap<MutationObserverId, HashSet<usize>>,
    types: [usize; 3],
    record_sink: Option<std::rc::Weak<dyn MutationRecordSink>>,
}
impl Default for MutationObservers {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            transient: HashMap::new(),
            transient_nodes: HashMap::new(),
            targets: HashMap::new(),
            types: [0; 3],
            record_sink: None,
        }
    }
}
impl MutationObservers {
    fn Count(&mut self, options: &MutationObserverOptions, add: bool) {
        for kind in [
            MutationType::Attributes,
            MutationType::CharacterData,
            MutationType::ChildList,
        ] {
            if options.Includes(kind) {
                if add {
                    self.types[kind as usize] += 1;
                } else {
                    self.types[kind as usize] -= 1;
                }
            }
        }
    }
}

impl PersistentDocument {
    pub fn InstallMutationRecordSink(&mut self, sink: std::rc::Weak<dyn MutationRecordSink>) {
        assert!(
            self.mutation_observers
                .record_sink
                .as_ref()
                .and_then(|sink| sink.upgrade())
                .is_none(),
            "one canonical record owner per document"
        );
        self.mutation_observers.record_sink = Some(sink);
    }
    fn EnqueueMutationRecord(&self, record: CapturedMutationRecord) {
        if record.recipients.is_empty() {
            return;
        }
        if let Some(sink) = self
            .mutation_observers
            .record_sink
            .as_ref()
            .and_then(|sink| sink.upgrade())
        {
            // This output port may enqueue the agent microtask, but cannot
            // dispatch script or reborrow this document during a mutation.
            sink.Enqueue(record);
        }
    }
    pub(crate) fn CaptureAttributeMutation(&self, node: usize, name: &str, namespace: &str) {
        if !self.HasMutationObserversOfType(MutationType::Attributes) {
            return;
        }
        let recipients =
            self.MutationObserverInterest(node, MutationType::Attributes, Some((name, namespace)));
        if recipients.is_empty() {
            return;
        }
        let old_value = recipients
            .iter()
            .any(|recipient| recipient.old_value)
            .then(|| {
                self.Node(node)
                    .Attributes()
                    .iter()
                    .find(|attribute| {
                        attribute.local_name == name && attribute.namespace_uri == namespace
                    })
                    .map(|attribute| {
                        std::rc::Rc::from(attribute.value.encode_utf16().collect::<Vec<_>>())
                    })
            })
            .flatten();
        self.EnqueueMutationRecord(CapturedMutationRecord {
            target: self.Node(node).Id(),
            recipients,
            data: CapturedMutationData::Attributes {
                name: name.to_owned(),
                namespace: namespace.to_owned(),
                old_value,
            },
        });
    }
    pub fn HasCanonicalMutationRecordSink(&self) -> bool {
        self.mutation_observers
            .record_sink
            .as_ref()
            .and_then(|sink| sink.upgrade())
            .is_some()
    }
    pub fn AllocateMutationObserverId(&mut self) -> MutationObserverId {
        let id = NEXT_OBSERVER_PRIORITY
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |id| id.checked_add(1),
            )
            .expect("mutation observer identity exhausted");
        MutationObserverId(id)
    }

    /// Register or replace this observer's options for exactly one target.
    pub fn RegisterMutationObserver(
        &mut self,
        id: MutationObserverId,
        node: usize,
        options: MutationObserverOptions,
    ) -> bool {
        if id.0 == 0
            || id.0 >= NEXT_OBSERVER_PRIORITY.load(std::sync::atomic::Ordering::Relaxed)
            || node >= self.NodeCount()
        {
            return false;
        }
        self.ClearTransientMutationRegistration(node, id);
        let registry = &mut self.mutation_observers;
        registry.Count(&options, true);
        if let Some(old) = registry
            .nodes
            .entry(node)
            .or_default()
            .insert(id, std::rc::Rc::new(options))
        {
            registry.Count(&old, false);
        }
        registry.targets.entry(id).or_default().insert(node);
        true
    }

    pub fn DisconnectMutationObserver(&mut self, id: MutationObserverId) {
        self.ClearTransientMutationObservers(id);
        let registry = &mut self.mutation_observers;
        if let Some(nodes) = registry.targets.remove(&id) {
            for node in nodes {
                let registrations = registry
                    .nodes
                    .get_mut(&node)
                    .expect("registered mutation target");
                let old = registrations
                    .remove(&id)
                    .expect("registered mutation observer");
                let empty = registrations.is_empty();
                if empty {
                    registry.nodes.remove(&node);
                }
                registry.Count(&old, false);
            }
        }
    }

    pub fn HasMutationObserversOfType(&self, kind: MutationType) -> bool {
        self.mutation_observers.types[kind as usize] != 0
    }
    // Node::NotifyMutationObserversNodeWillDetach cc:3336-3354 and
    // MutationObserverRegistration::ObservedSubtreeNodeWillDetach cc:66-82.
    // The transient table shares the original options owner in O(1).
    pub(crate) fn MutationObserversNodeWillDetach(&mut self, child: usize) {
        if self
            .mutation_observers
            .types
            .iter()
            .all(|&count| count == 0)
        {
            return;
        }
        let mut parent = self.Node(child).Parent();
        let mut registrations = HashMap::new();
        while let Some(node) = parent {
            if let Some(registry) = self.mutation_observers.nodes.get(&node) {
                for (&id, options) in registry {
                    if options.subtree {
                        registrations.insert((node, id), options.clone());
                    }
                }
            }
            if let Some(registry) = self.mutation_observers.transient.get(&node) {
                for (&key, options) in registry {
                    registrations.insert(key, options.clone());
                }
            }
            parent = self.Node(node).Parent();
        }
        let sink = self
            .mutation_observers
            .record_sink
            .as_ref()
            .and_then(|sink| sink.upgrade());
        for ((origin, id), options) in registrations {
            self.mutation_observers
                .transient
                .entry(child)
                .or_default()
                .insert((origin, id), options);
            self.mutation_observers
                .transient_nodes
                .entry((origin, id))
                .or_default()
                .insert(child);
            if let Some(sink) = &sink {
                sink.Activate(id);
            }
        }
    }
    fn ClearTransientMutationRegistration(&mut self, origin: usize, id: MutationObserverId) {
        let Some(nodes) = self
            .mutation_observers
            .transient_nodes
            .remove(&(origin, id))
        else {
            return;
        };
        for node in nodes {
            if let Some(registrations) = self.mutation_observers.transient.get_mut(&node) {
                registrations.remove(&(origin, id));
                if registrations.is_empty() {
                    self.mutation_observers.transient.remove(&node);
                }
            }
        }
    }
    // MutationObserver::Deliver cc:375-384 clears before swapping records.
    pub fn ClearTransientMutationObservers(&mut self, id: MutationObserverId) {
        let origins = self
            .mutation_observers
            .transient_nodes
            .keys()
            .filter(|(_, observer)| *observer == id)
            .map(|(origin, _)| *origin)
            .collect::<Vec<_>>();
        for origin in origins {
            self.ClearTransientMutationRegistration(origin, id);
        }
    }

    /// Walk raw DOM ancestry once, without crossing a ShadowRoot's host.
    /// Registrations of the same observer merge old-value delivery options.
    pub fn MutationObserverInterest(
        &self,
        target: usize,
        kind: MutationType,
        attribute: Option<(&str, &str)>,
    ) -> Vec<MutationObserverRecipient> {
        debug_assert!(matches!(kind, MutationType::Attributes) == attribute.is_some());
        if !self.HasMutationObserversOfType(kind) {
            return Vec::new();
        }
        let mut interested = HashMap::<MutationObserverId, bool>::new();
        let mut node = Some(target);
        while let Some(index) = node {
            if let Some(registrations) = self.mutation_observers.nodes.get(&index) {
                for (&observer, options) in registrations {
                    if !options.Includes(kind) || index != target && !options.subtree {
                        continue;
                    }
                    if let Some((name, namespace)) = attribute {
                        if options
                            .attribute_filter
                            .as_ref()
                            .is_some_and(|filter| !namespace.is_empty() || !filter.contains(name))
                        {
                            continue;
                        }
                    }
                    *interested.entry(observer).or_default() |= options.WantsOldValue(kind);
                }
            }
            if let Some(registrations) = self.mutation_observers.transient.get(&index) {
                for (&(_, observer), options) in registrations {
                    if !options.Includes(kind) {
                        continue;
                    }
                    if let Some((name, namespace)) = attribute {
                        if options
                            .attribute_filter
                            .as_ref()
                            .is_some_and(|filter| !namespace.is_empty() || !filter.contains(name))
                        {
                            continue;
                        }
                    }
                    *interested.entry(observer).or_default() |= options.WantsOldValue(kind);
                }
            }
            node = self.Node(index).Parent();
        }
        interested
            .into_iter()
            .map(|(observer, old_value)| MutationObserverRecipient {
                observer,
                old_value,
            })
            .collect()
    }
}

/// Script-free records emitted by compound DOM operations. The outer WebAPI
/// mutation boundary creates JavaScript records and schedules delivery once.
/// Recipients are selected at the primitive mutation, before detaching nodes.
#[derive(Clone)]
pub enum CapturedMutationData {
    Attributes {
        name: String,
        namespace: String,
        old_value: Option<std::rc::Rc<[u16]>>,
    },
    CharacterData {
        old_value: Option<std::rc::Rc<[u16]>>,
    },
    ChildList {
        added_nodes: Vec<u64>,
        removed_nodes: Vec<u64>,
        previous_sibling: Option<u64>,
        next_sibling: Option<u64>,
    },
}
#[derive(Clone)]
pub struct CapturedMutationRecord {
    pub target: u64,
    pub recipients: Vec<MutationObserverRecipient>,
    pub data: CapturedMutationData,
}
pub trait MutationRecordSink {
    fn Activate(&self, observer: MutationObserverId);
    /// Must only retain immutable records and enqueue delivery. No synchronous
    /// callback or DOM query is allowed while the source mutation holds DOM.
    fn Enqueue(&self, record: CapturedMutationRecord);
}
type RecordBuffer = std::rc::Rc<std::cell::RefCell<Vec<CapturedMutationRecord>>>;
#[derive(Default)]
pub(crate) struct MutationRecordCaptures {
    scopes: std::rc::Rc<std::cell::RefCell<Vec<RecordBuffer>>>,
    suppressed_children: std::rc::Rc<std::cell::RefCell<HashMap<usize, usize>>>,
}
pub struct MutationRecordCapture {
    scopes: std::rc::Rc<std::cell::RefCell<Vec<RecordBuffer>>>,
    buffer: RecordBuffer,
}
impl MutationRecordCapture {
    /// End capture before the first JavaScript entry. Nested source operations
    /// append to the current buffer and never create a microtask checkpoint.
    pub fn Finish(self) -> Vec<CapturedMutationRecord> {
        let records = std::mem::take(&mut *self.buffer.borrow_mut());
        drop(self);
        records
    }
}
impl Drop for MutationRecordCapture {
    fn drop(&mut self) {
        let last = self
            .scopes
            .borrow_mut()
            .pop()
            .expect("balanced DOM mutation capture");
        assert!(
            std::rc::Rc::ptr_eq(&last, &self.buffer),
            "nested DOM captures end in source order"
        );
    }
}
impl PersistentDocument {
    pub fn BeginMutationRecordCapture(&self) -> MutationRecordCapture {
        let buffer = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let scopes = self.mutation_record_captures.scopes.clone();
        scopes.borrow_mut().push(buffer.clone());
        MutationRecordCapture { scopes, buffer }
    }
    pub(crate) fn CaptureCharacterDataMutation(&self, node: usize) {
        let buffer = self
            .mutation_record_captures
            .scopes
            .borrow()
            .last()
            .cloned();
        if buffer.is_none() && self.mutation_observers.record_sink.is_none() {
            return;
        }
        let recipients = self.MutationObserverInterest(node, MutationType::CharacterData, None);
        if recipients.is_empty() {
            return;
        }
        let old_value = recipients
            .iter()
            .any(|recipient| recipient.old_value)
            .then(|| self.CharacterDataUTF16(node));
        let record = CapturedMutationRecord {
            target: self.Node(node).Id(),
            recipients,
            data: CapturedMutationData::CharacterData { old_value },
        };
        if self.mutation_observers.record_sink.is_some() {
            self.EnqueueMutationRecord(record.clone());
        }
        if let Some(buffer) = buffer {
            buffer.borrow_mut().push(record);
        }
    }
    pub(crate) fn CaptureChildListMutation(
        &self,
        parent: usize,
        added: &[usize],
        removed: &[usize],
        previous: Option<usize>,
        next: Option<usize>,
    ) {
        if self
            .mutation_record_captures
            .suppressed_children
            .borrow()
            .contains_key(&parent)
        {
            return;
        }
        let buffer = self
            .mutation_record_captures
            .scopes
            .borrow()
            .last()
            .cloned();
        if buffer.is_none() && self.mutation_observers.record_sink.is_none() {
            return;
        }
        let recipients = self.MutationObserverInterest(parent, MutationType::ChildList, None);
        if recipients.is_empty() {
            return;
        }
        let record = CapturedMutationRecord {
            target: self.Node(parent).Id(),
            recipients,
            data: CapturedMutationData::ChildList {
                added_nodes: added.iter().map(|&node| self.Node(node).Id()).collect(),
                removed_nodes: removed.iter().map(|&node| self.Node(node).Id()).collect(),
                previous_sibling: previous.map(|node| self.Node(node).Id()),
                next_sibling: next.map(|node| self.Node(node).Id()),
            },
        };
        if self.mutation_observers.record_sink.is_some() {
            self.EnqueueMutationRecord(record.clone());
        }
        if let Some(buffer) = buffer {
            buffer.borrow_mut().push(record);
        }
    }
}

#[cfg(test)]
mod compound_record_tests {
    use super::*;
    use crate::persistent_document::DOMNamespace;
    use crate::{DOMBoundaryPoint, DOM};

    #[test]
    fn compound_range_insert_keeps_character_data_and_child_list_order() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        document.AppendChild(document.Root(), parent);
        let text = document.CreateText("abcd".into());
        document.AppendChild(parent, text);
        let observer = document.AllocateMutationObserverId();
        assert!(document.RegisterMutationObserver(
            observer,
            parent,
            MutationObserverOptions {
                attributes: false,
                character_data: true,
                child_list: true,
                subtree: true,
                attribute_old_value: false,
                character_data_old_value: true,
                attribute_filter: None,
            }
        ));
        let point = DOMBoundaryPoint {
            node_id: document.Node(text).Id(),
            offset: 2,
        };
        let range = document.CreateRange(point, point).unwrap();
        let inserted = document.CreateText("X".into());
        let capture = document.BeginMutationRecordCapture();
        document
            .InsertRangeNode(&range, document.Node(inserted).Id())
            .unwrap();
        let records = capture.Finish();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].target, document.Node(text).Id());
        assert!(
            matches!(&records[0].data, CapturedMutationData::CharacterData { old_value: Some(value) } if value.as_ref() == &[97,98,99,100])
        );
        let right = document.Node(parent).Children()[2];
        assert!(
            matches!(&records[1].data, CapturedMutationData::ChildList { added_nodes, removed_nodes, previous_sibling: Some(previous), next_sibling: None } if added_nodes == &[document.Node(right).Id()] && removed_nodes.is_empty() && *previous == document.Node(text).Id())
        );
        assert!(
            matches!(&records[2].data, CapturedMutationData::ChildList { added_nodes, removed_nodes, previous_sibling: Some(previous), next_sibling: Some(next) } if added_nodes == &[document.Node(inserted).Id()] && removed_nodes.is_empty() && *previous == document.Node(text).Id() && *next == document.Node(right).Id())
        );
        assert!(records.iter().all(
            |record| record.recipients.len() == 1 && record.recipients[0].observer == observer
        ));
        let fragment = document.CreateDocumentFragment();
        let first = document.CreateText("Y".into());
        let second = document.CreateText("Z".into());
        document.AppendChild(fragment, first);
        document.AppendChild(fragment, second);
        assert!(document.RegisterMutationObserver(
            observer,
            fragment,
            MutationObserverOptions {
                attributes: false,
                character_data: false,
                child_list: true,
                subtree: false,
                attribute_old_value: false,
                character_data_old_value: false,
                attribute_filter: None,
            }
        ));
        let point = DOMBoundaryPoint {
            node_id: document.Node(parent).Id(),
            offset: 1,
        };
        let range = document.CreateRange(point, point).unwrap();
        let capture = document.BeginMutationRecordCapture();
        document
            .InsertRangeNode(&range, document.Node(fragment).Id())
            .unwrap();
        let records = capture.Finish();
        assert_eq!(records.len(), 2);
        let nodes = [document.Node(first).Id(), document.Node(second).Id()];
        assert_eq!(records[0].target, document.Node(fragment).Id());
        assert!(
            matches!(&records[0].data, CapturedMutationData::ChildList {added_nodes, removed_nodes, previous_sibling:None, next_sibling:None} if added_nodes.is_empty() && removed_nodes == &nodes)
        );
        assert_eq!(records[1].target, document.Node(parent).Id());
        assert!(
            matches!(&records[1].data, CapturedMutationData::ChildList {added_nodes, removed_nodes, previous_sibling:Some(previous), next_sibling:Some(next)} if added_nodes == &nodes && removed_nodes.is_empty() && *previous == document.Node(text).Id() && *next == document.Node(inserted).Id())
        );
    }
}

pub(crate) struct ChildListMutationSuppression {
    parents: std::rc::Rc<std::cell::RefCell<HashMap<usize, usize>>>,
    parent: usize,
}
impl Drop for ChildListMutationSuppression {
    fn drop(&mut self) {
        let mut parents = self.parents.borrow_mut();
        let count = parents
            .get_mut(&self.parent)
            .expect("balanced child-list mutation scope");
        *count -= 1;
        if *count == 0 {
            parents.remove(&self.parent);
        }
    }
}
impl PersistentDocument {
    /// ContainerNode's fragment insertion uses one ChildListMutationScope for
    /// all removed/added children. Its exact record is captured at that scope.
    pub(crate) fn SuppressCapturedChildListMutations(
        &self,
        parent: usize,
    ) -> ChildListMutationSuppression {
        let parents = self.mutation_record_captures.suppressed_children.clone();
        *parents.borrow_mut().entry(parent).or_default() += 1;
        ChildListMutationSuppression { parents, parent }
    }
}
