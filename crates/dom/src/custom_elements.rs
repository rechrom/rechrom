// Copyright 2016 The Chromium Authors
// Source-aligned translation of Blink's core/html/custom definition, registry,
// reaction queue and reaction stack. See third_party/NOTICE.chromium-custom-elements.
#![allow(non_snake_case)]

use crate::persistent_document::{
    DOMNamespace, DOMNodeType, DOMShadowRootMode, PersistentDocument,
};
use std::collections::{HashMap, VecDeque};

/// A document-local definition identity. Script functions stay in the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomElementDefinitionId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomElementDescriptor {
    pub name: String,
    pub local_name: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CustomElementCallbacks {
    pub connected: bool,
    pub disconnected: bool,
    pub connected_move: bool,
    pub adopted: bool,
    pub attribute_changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomElementDefinition {
    pub id: CustomElementDefinitionId,
    pub descriptor: CustomElementDescriptor,
    pub observed_attributes: Vec<String>,
    pub callbacks: CustomElementCallbacks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomElementRegistrationError {
    InvalidName,
    DuplicateName,
    CustomizedBuiltInUnsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomElementState {
    Uncustomized,
    Undefined,
    Precustomized,
    Custom,
    Failed,
}

/// Pure DOM facts consumed by the script host at a CEReactions boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustomElementReaction {
    Upgrade {
        element: usize,
        id: CustomElementDefinitionId,
    },
    AttributeChanged {
        element: usize,
        id: CustomElementDefinitionId,
        name: String,
        old_value: Option<String>,
        new_value: Option<String>,
        namespace: Option<String>,
    },
    Connected {
        element: usize,
        id: CustomElementDefinitionId,
    },
    Disconnected {
        element: usize,
        id: CustomElementDefinitionId,
    },
}

impl CustomElementReaction {
    pub fn Element(&self) -> usize {
        match self {
            Self::Upgrade { element, .. }
            | Self::AttributeChanged { element, .. }
            | Self::Connected { element, .. }
            | Self::Disconnected { element, .. } => *element,
        }
    }
}

#[derive(Default)]
pub(crate) struct CustomElementRegistry {
    definitions: Vec<CustomElementDefinition>,
    names: HashMap<String, CustomElementDefinitionId>,
    elements: HashMap<usize, (CustomElementState, CustomElementDefinitionId)>,
    // CustomElementReactionStack owns one shared reaction queue per element.
    // Nested scopes own separate element queues, not separate reaction lists.
    reactions: HashMap<usize, VecDeque<CustomElementReaction>>,
    scopes: Vec<VecDeque<usize>>,
    backup: VecDeque<usize>,
}

impl CustomElementRegistry {
    fn Definition(&self, id: CustomElementDefinitionId) -> Option<&CustomElementDefinition> {
        let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
        self.definitions.get(index)
    }

    fn Enqueue(&mut self, reaction: CustomElementReaction) {
        let element = reaction.Element();
        self.reactions
            .entry(element)
            .or_default()
            .push_back(reaction);
        // Blink intentionally permits repeated element entries. The shared
        // queue is drained on the first visit; subsequent entries are empty.
        self.scopes
            .last_mut()
            .unwrap_or(&mut self.backup)
            .push_back(element);
    }

    fn HasPending(&self) -> bool {
        self.scopes
            .last()
            .unwrap_or(&self.backup)
            .iter()
            .any(|element| {
                self.reactions
                    .get(element)
                    .is_some_and(|queue| !queue.is_empty())
            })
    }

    fn Pop(&mut self) -> Option<CustomElementReaction> {
        let elements = self.scopes.last_mut().unwrap_or(&mut self.backup);
        while let Some(&element) = elements.front() {
            if let Some(reaction) = self
                .reactions
                .get_mut(&element)
                .and_then(VecDeque::pop_front)
            {
                // Keep the element at the front until its next empty visit:
                // invoking this reaction can synchronously append more work.
                return Some(reaction);
            }
            self.reactions.remove(&element);
            elements.pop_front();
        }
        None
    }
}

/// CustomElement::IsValidName and Document::ParseElementLocalName. The source
/// revision uses the DOM valid-local-name rule, not the older PCENChar grammar.
pub fn IsValidCustomElementName(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.contains('-')
        && !name.bytes().any(|c| {
            c.is_ascii_uppercase()
                || matches!(c, 0 | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ' | b'/' | b'>')
        })
        && !matches!(
            name,
            "annotation-xml"
                | "color-profile"
                | "font-face"
                | "font-face-src"
                | "font-face-uri"
                | "font-face-format"
                | "font-face-name"
                | "missing-glyph"
        )
}

impl PersistentDocument {
    /// The host first extracts and validates all script-observable definition
    /// properties. Only this successful commit makes a definition visible.
    pub fn RegisterCustomElementDefinition(
        &mut self,
        descriptor: CustomElementDescriptor,
        observed_attributes: Vec<String>,
        callbacks: CustomElementCallbacks,
    ) -> Result<CustomElementDefinitionId, CustomElementRegistrationError> {
        if !IsValidCustomElementName(&descriptor.name) {
            return Err(CustomElementRegistrationError::InvalidName);
        }
        if self.custom_elements.names.contains_key(&descriptor.name) {
            return Err(CustomElementRegistrationError::DuplicateName);
        }
        if descriptor.name != descriptor.local_name {
            return Err(CustomElementRegistrationError::CustomizedBuiltInUnsupported);
        }
        let id = CustomElementDefinitionId(self.custom_elements.definitions.len() as u64 + 1);
        self.custom_elements
            .names
            .insert(descriptor.name.clone(), id);
        self.custom_elements
            .definitions
            .push(CustomElementDefinition {
                id,
                descriptor,
                // Without the callback the host must not read observedAttributes.
                observed_attributes: if callbacks.attribute_changed {
                    observed_attributes
                } else {
                    Vec::new()
                },
                callbacks,
            });
        // CollectCandidates + CustomElementUpgradeSorter: only candidates in
        // the associated live document, in shadow-including tree order.
        for element in self.CustomElementInclusiveDescendants(self.Root()) {
            if self
                .LookupCustomElementDefinition(element)
                .is_some_and(|d| d.id == id)
                && self.CustomElementState(element) == CustomElementState::Undefined
            {
                self.custom_elements
                    .Enqueue(CustomElementReaction::Upgrade { element, id });
            }
        }
        Ok(id)
    }

    pub fn HasCustomElementDefinitions(&self) -> bool {
        !self.custom_elements.definitions.is_empty()
    }

    pub fn CustomElementDefinitionForName(&self, name: &str) -> Option<&CustomElementDefinition> {
        self.custom_elements
            .Definition(*self.custom_elements.names.get(name)?)
    }

    pub fn CustomElementDefinitionForId(
        &self,
        id: CustomElementDefinitionId,
    ) -> Option<&CustomElementDefinition> {
        self.custom_elements.Definition(id)
    }

    pub fn CustomElementState(&self, element: usize) -> CustomElementState {
        if let Some(&(state, _)) = self.custom_elements.elements.get(&element) {
            return state;
        }
        let node = self.Node(element);
        if node.Type() == DOMNodeType::kElement
            && node.Namespace() == DOMNamespace::kHTML
            && IsValidCustomElementName(node.Name())
        {
            CustomElementState::Undefined
        } else {
            CustomElementState::Uncustomized
        }
    }

    /// DefinitionForElement returns an installed definition only for Custom.
    pub fn CustomElementDefinitionForElement(
        &self,
        element: usize,
    ) -> Option<&CustomElementDefinition> {
        let &(state, id) = self.custom_elements.elements.get(&element)?;
        (state == CustomElementState::Custom)
            .then(|| self.custom_elements.Definition(id))
            .flatten()
    }

    /// Registry lookup for a not-yet-upgraded autonomous HTML element.
    pub fn LookupCustomElementDefinition(
        &self,
        element: usize,
    ) -> Option<&CustomElementDefinition> {
        let node = self.Node(element);
        if node.Type() != DOMNodeType::kElement
            || node.Namespace() != DOMNamespace::kHTML
            || node.OwnerDocumentNode() != Some(self.RootHandle())
        {
            return None;
        }
        self.CustomElementDefinitionForName(node.Name())
            .filter(|definition| definition.descriptor.local_name == node.Name())
    }

    pub fn PushCustomElementReactionScope(&mut self) {
        self.custom_elements.scopes.push(VecDeque::new());
    }

    /// The host invokes reactions before popping. Refuse to silently discard
    /// pending reactions if a host forgets that part of the scope boundary.
    pub fn PopCustomElementReactionScope(&mut self) -> bool {
        if self.custom_elements.scopes.is_empty() || self.custom_elements.HasPending() {
            return false;
        }
        self.custom_elements.scopes.pop();
        true
    }

    pub fn HasPendingCustomElementReactions(&self) -> bool {
        self.custom_elements.HasPending()
    }

    /// Embeddings schedule a microtask when the backup queue acquires work.
    /// A nested CEReactions scope must not hide this scheduling signal.
    pub fn HasPendingCustomElementBackupReactions(&self) -> bool {
        self.custom_elements.backup.iter().any(|element| {
            self.custom_elements
                .reactions
                .get(element)
                .is_some_and(|queue| !queue.is_empty())
        })
    }

    pub fn HasActiveCustomElementReactionScope(&self) -> bool {
        !self.custom_elements.scopes.is_empty()
    }

    pub fn PopCustomElementReaction(&mut self) -> Option<CustomElementReaction> {
        self.custom_elements.Pop()
    }

    /// Inclusive author-shadow traversal used by CustomElementRegistry::upgrade.
    /// It visits closed roots too, and deliberately does not enter template
    /// contents or user-agent implementation roots.
    fn CustomElementInclusiveDescendants(&self, root: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            result.push(node);
            pending.extend(self.Node(node).Children().iter().rev().copied());
            if let Some(shadow) = self.ShadowRoot(node) {
                if self.ShadowRootMode(shadow) != Some(DOMShadowRootMode::kUserAgent) {
                    pending.push(shadow);
                }
            }
        }
        result
    }

    pub fn EnqueueCustomElementUpgradeSubtree(&mut self, root: usize) {
        if !self.HasCustomElementDefinitions() {
            return;
        }
        for element in self.CustomElementInclusiveDescendants(root) {
            self.EnqueueCustomElementUpgrade(element);
        }
    }

    fn EnqueueCustomElementUpgrade(&mut self, element: usize) {
        if self.CustomElementState(element) != CustomElementState::Undefined {
            return;
        }
        if let Some(id) = self.LookupCustomElementDefinition(element).map(|d| d.id) {
            self.custom_elements
                .Enqueue(CustomElementReaction::Upgrade { element, id });
        }
    }

    /// The native half of Definition::Upgrade. No JavaScript runs here. The
    /// host calls the captured constructor immediately after this returns true.
    pub fn BeginCustomElementUpgrade(
        &mut self,
        element: usize,
        id: CustomElementDefinitionId,
    ) -> bool {
        if self.CustomElementState(element) != CustomElementState::Undefined
            || !self
                .LookupCustomElementDefinition(element)
                .is_some_and(|d| d.id == id)
        {
            return false;
        }
        let definition = self.custom_elements.Definition(id).unwrap().clone();
        self.custom_elements
            .elements
            .insert(element, (CustomElementState::Failed, id));
        // Definition::EnqueueAttributeChangedCallbackForAllAttributes walks the
        // actual attribute list, not the order of observedAttributes.
        for attribute in self.Node(element).Attributes().to_vec() {
            if definition
                .observed_attributes
                .contains(&attribute.local_name)
            {
                self.custom_elements
                    .Enqueue(CustomElementReaction::AttributeChanged {
                        element,
                        id,
                        name: attribute.local_name,
                        old_value: None,
                        new_value: Some(attribute.value),
                        namespace: (!attribute.namespace_uri.is_empty())
                            .then_some(attribute.namespace_uri),
                    });
            }
        }
        if self.IsConnectedToDocument(element) && definition.callbacks.connected {
            self.custom_elements
                .Enqueue(CustomElementReaction::Connected { element, id });
        }
        // ScriptCustomElementDefinition::RunConstructor sets PreCustomized
        // immediately before invoking the constructor. Attribute/tree mutation
        // hooks only enqueue lifecycle callbacks for fully Custom elements.
        self.custom_elements
            .elements
            .insert(element, (CustomElementState::Precustomized, id));
        true
    }

    pub fn CompleteCustomElementUpgrade(
        &mut self,
        element: usize,
        id: CustomElementDefinitionId,
        succeeded: bool,
    ) {
        if self.custom_elements.elements.get(&element)
            != Some(&(CustomElementState::Precustomized, id))
        {
            return;
        }
        let state = if succeeded {
            CustomElementState::Custom
        } else {
            CustomElementState::Failed
        };
        self.custom_elements.elements.insert(element, (state, id));
        if !succeeded {
            // Definition::Upgrade clears every callback already enqueued for
            // this element, including pre-constructor attribute/connected work.
            self.custom_elements.reactions.remove(&element);
        }
        self.InvalidateNodeStyle(element);
    }

    pub(crate) fn CustomElementAttributeWillChange(
        &mut self,
        element: usize,
        name: &str,
        namespace: &str,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let Some(definition) = self.CustomElementDefinitionForElement(element) else {
            return;
        };
        if !definition
            .observed_attributes
            .iter()
            .any(|observed| observed == name)
        {
            return;
        }
        let id = definition.id;
        self.custom_elements
            .Enqueue(CustomElementReaction::AttributeChanged {
                element,
                id,
                name: name.into(),
                old_value,
                new_value,
                namespace: (!namespace.is_empty()).then(|| namespace.into()),
            });
    }

    pub(crate) fn CustomElementSubtreeInserted(&mut self, root: usize) {
        if !self.HasCustomElementDefinitions() || !self.IsConnectedToDocument(root) {
            return;
        }
        for element in self.CustomElementInclusiveDescendants(root) {
            if let Some(definition) = self.CustomElementDefinitionForElement(element) {
                if definition.callbacks.connected {
                    let id = definition.id;
                    self.custom_elements
                        .Enqueue(CustomElementReaction::Connected { element, id });
                }
            } else {
                self.EnqueueCustomElementUpgrade(element);
            }
        }
    }

    pub(crate) fn CustomElementSubtreeWillBeRemoved(&mut self, root: usize) {
        if !self.HasCustomElementDefinitions() || !self.IsConnectedToDocument(root) {
            return;
        }
        for element in self.CustomElementInclusiveDescendants(root) {
            if let Some(definition) = self.CustomElementDefinitionForElement(element) {
                if definition.callbacks.disconnected {
                    let id = definition.id;
                    self.custom_elements
                        .Enqueue(CustomElementReaction::Disconnected { element, id });
                }
            }
        }
    }
}
