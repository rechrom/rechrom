// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Script-free registration and interest selection. Callback/record ownership
//! stays in the embedding; this document owner carries only opaque identities.
//! cpp: Node::GetRegisteredMutationObserversOfType;
//! MutationObserverRegistration::ShouldReceiveMutationFrom;
//! MutationObserverInterestGroup::CreateIfNeeded.
#![allow(non_snake_case)]

use crate::persistent_document::PersistentDocument;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

pub struct MutationObserverRecipient {
    pub observer: MutationObserverId,
    pub old_value: bool,
}

pub(crate) struct MutationObservers {
    next_id: u64,
    nodes: HashMap<usize, HashMap<MutationObserverId, MutationObserverOptions>>,
    targets: HashMap<MutationObserverId, HashSet<usize>>,
    types: [usize; 3],
}
impl Default for MutationObservers {
    fn default() -> Self {
        Self {
            next_id: 1,
            nodes: HashMap::new(),
            targets: HashMap::new(),
            types: [0; 3],
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
    pub fn AllocateMutationObserverId(&mut self) -> MutationObserverId {
        let id = self.mutation_observers.next_id;
        self.mutation_observers.next_id = id
            .checked_add(1)
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
        if id.0 == 0 || id.0 >= self.mutation_observers.next_id || node >= self.NodeCount() {
            return false;
        }
        let registry = &mut self.mutation_observers;
        registry.Count(&options, true);
        if let Some(old) = registry.nodes.entry(node).or_default().insert(id, options) {
            registry.Count(&old, false);
        }
        registry.targets.entry(id).or_default().insert(node);
        true
    }

    pub fn DisconnectMutationObserver(&mut self, id: MutationObserverId) {
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
