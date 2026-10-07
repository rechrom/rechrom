#![allow(non_snake_case)]

use crate::error::{invalid_argument, logic_error};
use crate::ImageResourceMetadata;
use cssom::{CSSDeclaration, CSSOMEvent, CSSOMMutation, CSSStyleSheet, CSSOM};
use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

static NEXT_ARENA_ID: AtomicU64 = AtomicU64::new(1);

// A C++ DOMNode* can name a document node in a different Document arena.
// Arena indices alone cannot carry that identity across Rust documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DOMOwnerHandle {
    arena_id: u64,
    node_id: u64,
}

impl DOMOwnerHandle {
    // Identity adapter for source uses of OwnerDocumentNode()->Id().
    // The arena component remains part of handle equality and adoption.
    pub fn Id(self) -> u64 {
        self.node_id
    }
}

// cpp: dom/document.h:19-27
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DOMNodeType {
    kDocument,
    kDocumentType,
    kElement,
    kText,
    kComment,
    kProcessingInstruction,
    kDocumentFragment,
}

// cpp: dom/document.h:29-29
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DOMNamespace {
    kNone,
    kHTML,
    kSVG,
    kMathML,
}

// cpp: dom/document.h:31-36
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMAttribute {
    pub prefix: String,
    pub local_name: String,
    pub namespace_uri: String,
    pub value: String,
}

// cpp: dom/document.h:43-77
// Stable arena indices stand in for stable C++ node addresses. No Rust reference
// to an arena element is retained across allocation or reparenting.
#[derive(Clone, Debug)]
pub struct DOMNode {
    id: u64,
    node_type: DOMNodeType,
    namespace: DOMNamespace,
    name: String,
    data: String,
    attributes: Vec<DOMAttribute>,
    owner_document_handle: Option<DOMOwnerHandle>,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl DOMNode {
    // cpp: dom/document.h:45-53
    pub fn Type(&self) -> DOMNodeType {
        self.node_type
    }
    pub fn Namespace(&self) -> DOMNamespace {
        self.namespace
    }
    pub fn Id(&self) -> u64 {
        self.id
    }
    pub fn Name(&self) -> &str {
        &self.name
    }
    pub fn Data(&self) -> &str {
        &self.data
    }
    pub fn Attributes(&self) -> &[DOMAttribute] {
        &self.attributes
    }
    pub fn OwnerDocumentNode(&self) -> Option<DOMOwnerHandle> {
        self.owner_document_handle
    }
    pub fn Parent(&self) -> Option<usize> {
        self.parent
    }
    pub fn Children(&self) -> &[usize] {
        &self.children
    }

    // cpp: dom/document.cc:24-28
    pub fn FindAttribute(&self, local_name: &str) -> Option<&DOMAttribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.local_name == local_name)
    }

    // cpp: dom/document.cc:30-34
    pub fn IsElement(&self, local_name: &str, node_namespace: DOMNamespace) -> bool {
        self.node_type == DOMNodeType::kElement
            && self.namespace == node_namespace
            && self.name == local_name
    }
    // cpp: dom/document.h:56-57
    pub fn IsHTMLElement(&self, local_name: &str) -> bool {
        self.IsElement(local_name, DOMNamespace::kHTML)
    }
}

// cpp: dom/document.h:169-196
#[derive(Clone, PartialEq)]
pub struct PseudoElement {
    pub style: ComputedStyle,
    pub text: String,
    pub display_contents: bool,
}

impl Default for PseudoElement {
    fn default() -> Self {
        Self {
            style: ComputedStyle::default(),
            text: String::new(),
            display_contents: false,
        }
    }
}

#[derive(Clone)]
pub struct ResolvedNodeStyle {
    pub style: ComputedStyle,
    pub generates_box: bool,
    pub own_generates_box: bool,
    pub own_display_contents: bool,
    pub display_contents: bool,
    pub custom_properties: std::sync::Arc<HashMap<String, Option<String>>>,
    /// StyleBuilder's explicit-inheritance dependency after variable resolution.
    /// None denotes an external/old style without a dependency proof.
    pub has_explicit_inheritance: Option<bool>,
    pub before: Option<PseudoElement>,
    pub after: Option<PseudoElement>,
    pub first_letter: Option<PseudoElement>,
    pub placeholder: Option<PseudoElement>,
}

impl PartialEq for ResolvedNodeStyle {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style
            && self.generates_box == other.generates_box
            && self.own_generates_box == other.own_generates_box
            && self.own_display_contents == other.own_display_contents
            && self.display_contents == other.display_contents
            && (std::sync::Arc::ptr_eq(&self.custom_properties, &other.custom_properties)
                || self.custom_properties == other.custom_properties)
            && self.has_explicit_inheritance == other.has_explicit_inheritance
            && self.before == other.before
            && self.after == other.after
            && self.first_letter == other.first_letter
            && self.placeholder == other.placeholder
    }
}

impl Default for ResolvedNodeStyle {
    fn default() -> Self {
        Self {
            style: ComputedStyle::default(),
            generates_box: true,
            own_generates_box: true,
            own_display_contents: false,
            display_contents: false,
            custom_properties: Default::default(),
            has_explicit_inheritance: None,
            before: None,
            after: None,
            first_letter: None,
            placeholder: None,
        }
    }
}

fn StyleImageSources(style: &ResolvedNodeStyle) -> impl Iterator<Item = &str> {
    std::iter::once(&style.style)
        .chain(
            style
                .before
                .iter()
                .chain(style.after.iter())
                .chain(style.first_letter.iter())
                .chain(style.placeholder.iter())
                .map(|pseudo| &pseudo.style),
        )
        .flat_map(|style| {
            style
                .paint
                .background_images
                .iter()
                .chain(style.paint.mask_images.iter().map(|mask| &mask.image))
                .map(|image| image.source_url.as_str())
        })
}

// cpp: dom/document.h:79-85
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CompatibilityMode {
    kNoQuirks,
    kLimitedQuirks,
    kQuirks,
}

// cpp: dom/document.h:207-232
// Re-exported as dom::Document. The older parser's narrow tree is exposed
// separately as dom::ParsedDocument until its callers migrate.
pub struct PersistentDocument {
    arena: Vec<DOMNode>,
    arena_id: u64,
    collection_membership_revision: u64,
    root: usize,
    next_id: u64,
    compatibility: CompatibilityMode,
    template_contents: HashMap<u64, usize>,
    animated_styles: HashMap<u64, BTreeMap<u64, Vec<CSSDeclaration>>>,
    cssom: CSSOM<DOMOwnerHandle>,
    image_resources: HashMap<String, ImageResourceMetadata>,
    scroll_offsets: HashMap<u64, Offset>,
    resolved_styles: HashMap<u64, std::sync::Arc<ResolvedNodeStyle>>,
    // StyleFetchedImage::AddClient/RemoveClient: only resolved CSS consumers
    // need their background/mask resource IDs refreshed on image completion.
    style_image_consumers: HashMap<String, BTreeSet<usize>>,
    style_state: crate::style_state::StyleState,
    control_checked_state: HashMap<u64, bool>,
    control_value_state: HashMap<u64, String>,
}

impl PersistentDocument {
    // cpp: dom/document.cc:36-39
    fn new() -> Self {
        let mut document = Self {
            arena: Vec::new(),
            arena_id: NEXT_ARENA_ID.fetch_add(1, Ordering::Relaxed),
            collection_membership_revision: 0,
            root: 0,
            next_id: 1,
            compatibility: CompatibilityMode::kNoQuirks,
            template_contents: HashMap::new(),
            animated_styles: HashMap::new(),
            cssom: CSSOM::default(),
            image_resources: HashMap::new(),
            scroll_offsets: HashMap::new(),
            resolved_styles: HashMap::new(),
            style_image_consumers: HashMap::new(),
            style_state: Default::default(),
            control_checked_state: HashMap::new(),
            control_value_state: HashMap::new(),
        };
        document.root = document.Allocate(
            DOMNodeType::kDocument,
            DOMNamespace::kNone,
            "#document".into(),
            String::new(),
            Vec::new(),
        );
        document
    }

    // cpp: dom/document.h:87-89
    pub fn Root(&self) -> usize {
        self.root
    }
    // Identity adapter for C++ &Document::Root(), which is distinct from
    // Root().OwnerDocumentNode(): a document node has no owner document.
    pub fn RootHandle(&self) -> DOMOwnerHandle {
        DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        }
    }
    // The arena key moves with parser document swaps. Exhaustion permanently
    // disables reuse; an old cached revision can never become valid again.
    pub fn CollectionMembershipKey(&self) -> Option<(DOMOwnerHandle, u64)> {
        (self.collection_membership_revision != u64::MAX)
            .then(|| (self.RootHandle(), self.collection_membership_revision))
    }
    fn BumpCollectionMembershipRevision(&mut self) {
        self.collection_membership_revision = self.collection_membership_revision.saturating_add(1);
    }
    pub fn GetCompatibilityMode(&self) -> CompatibilityMode {
        self.compatibility
    }
    pub fn SetCompatibilityMode(&mut self, mode: CompatibilityMode) {
        self.compatibility = mode;
    }

    pub fn Node(&self, index: usize) -> &DOMNode {
        &self.arena[index]
    }
    // Index-to-reference adapter for source callbacks receiving DOMNode&.
    pub fn NodeMut(&mut self, index: usize) -> &mut DOMNode {
        self.BumpCollectionMembershipRevision();
        self.InvalidateNodeStyle(index);
        self.MarkStyleElementTextDirty(index);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        &mut self.arena[index]
    }
    // cpp: dom/document.cc:11-22
    // cpp: dom/document.cc:41-51
    fn Allocate(
        &mut self,
        node_type: DOMNodeType,
        namespace: DOMNamespace,
        name: String,
        data: String,
        attributes: Vec<DOMAttribute>,
    ) -> usize {
        self.BumpCollectionMembershipRevision();
        let index = self.arena.len();
        let id = self.next_id;
        self.next_id += 1;
        let owner_document_handle = (node_type != DOMNodeType::kDocument).then(|| DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        });
        self.arena.push(DOMNode {
            id,
            node_type,
            namespace,
            name,
            data,
            attributes,
            owner_document_handle,
            parent: None,
            children: Vec::new(),
        });
        index
    }

    // cpp: dom/document.cc:53-58
    pub fn AdoptNode(&mut self, node: usize, document_node: usize) {
        self.AdoptNodeByHandle(
            node,
            DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: self.arena[document_node].id,
            },
        );
    }

    pub fn AdoptNodeByHandle(&mut self, node: usize, owner: DOMOwnerHandle) {
        self.BumpCollectionMembershipRevision();
        if self.arena[node].node_type == DOMNodeType::kDocument {
            return;
        }
        if owner.arena_id == self.arena_id {
            let index = self
                .FindNodeById(owner.node_id)
                .expect("owner document must exist in the local arena");
            assert_eq!(self.arena[index].node_type, DOMNodeType::kDocument);
        }
        self.arena[node].owner_document_handle = Some(owner);
        let children = self.arena[node].children.clone();
        for child in children {
            self.AdoptNodeByHandle(child, owner);
        }
        if let Some(contents) = self.TemplateContents(node) {
            self.AdoptNodeByHandle(contents, owner);
        }
    }

    // cpp: dom/document.cc:60-68
    pub fn CreateHTMLDocument(&mut self, title: Option<String>) -> usize {
        let root = self.Allocate(
            DOMNodeType::kDocument,
            DOMNamespace::kNone,
            "#document".into(),
            String::new(),
            Vec::new(),
        );
        let doctype = self.CreateDocumentType("html".into(), String::new(), String::new());
        self.AppendChild(root, doctype);
        let html = self.CreateElement(DOMNamespace::kHTML, "html".into(), Vec::new());
        self.AppendChild(root, html);
        let head = self.CreateElement(DOMNamespace::kHTML, "head".into(), Vec::new());
        self.AppendChild(html, head);
        if let Some(title) = title {
            let element = self.CreateElement(DOMNamespace::kHTML, "title".into(), Vec::new());
            self.AppendChild(head, element);
            self.AppendText(element, &title);
        }
        let body = self.CreateElement(DOMNamespace::kHTML, "body".into(), Vec::new());
        self.AppendChild(html, body);
        root
    }

    // cpp: dom/document.cc:70-80
    pub fn CreateDocumentType(
        &mut self,
        name: String,
        public_identifier: String,
        system_identifier: String,
    ) -> usize {
        let identifiers = vec![
            DOMAttribute {
                local_name: "public".into(),
                value: public_identifier,
                ..Default::default()
            },
            DOMAttribute {
                local_name: "system".into(),
                value: system_identifier,
                ..Default::default()
            },
        ];
        self.Allocate(
            DOMNodeType::kDocumentType,
            DOMNamespace::kNone,
            name,
            String::new(),
            identifiers,
        )
    }

    // cpp: dom/document.h:95-97
    pub fn CreateDocumentTypeDefault(&mut self, name: String) -> usize {
        self.CreateDocumentType(name, String::new(), String::new())
    }

    // cpp: dom/document.cc:82-89
    pub fn CreateElement(
        &mut self,
        namespace: DOMNamespace,
        name: String,
        attributes: Vec<DOMAttribute>,
    ) -> usize {
        let element = self.Allocate(
            DOMNodeType::kElement,
            namespace,
            name,
            String::new(),
            attributes,
        );
        if self.arena[element].IsElement("template", DOMNamespace::kHTML) {
            self.EnsureTemplateContents(element);
        }
        element
    }

    // cpp: dom/document.h:98-100
    pub fn CreateElementDefault(&mut self, namespace: DOMNamespace, name: String) -> usize {
        self.CreateElement(namespace, name, Vec::new())
    }

    // cpp: dom/document.cc:91-109
    pub fn CreateDocumentFragment(&mut self) -> usize {
        self.Allocate(
            DOMNodeType::kDocumentFragment,
            DOMNamespace::kNone,
            "#document-fragment".into(),
            String::new(),
            Vec::new(),
        )
    }
    pub fn CreateText(&mut self, data: String) -> usize {
        self.Allocate(
            DOMNodeType::kText,
            DOMNamespace::kNone,
            "#text".into(),
            data,
            Vec::new(),
        )
    }
    pub fn CreateComment(&mut self, data: String) -> usize {
        self.Allocate(
            DOMNodeType::kComment,
            DOMNamespace::kNone,
            "#comment".into(),
            data,
            Vec::new(),
        )
    }
    pub fn CreateProcessingInstruction(&mut self, target: String, data: String) -> usize {
        self.Allocate(
            DOMNodeType::kProcessingInstruction,
            DOMNamespace::kNone,
            target,
            data,
            Vec::new(),
        )
    }

    // cpp: dom/document.cc:111-135
    pub fn CloneNode(&mut self, source: usize, deep: bool) -> usize {
        let original = self.arena[source].clone();
        let clone = self.Allocate(
            original.node_type,
            original.namespace,
            original.name,
            original.data,
            original.attributes,
        );
        if let Some(owner_document) = original.owner_document_handle {
            self.AdoptNodeByHandle(clone, owner_document);
        }
        if self.arena[source].IsElement("template", DOMNamespace::kHTML) {
            if let Some(contents) = self.TemplateContents(source) {
                let cloned_contents = self.EnsureTemplateContents(clone);
                if deep {
                    let children = self.arena[contents].children.clone();
                    for child in children {
                        let cloned_child = self.CloneNode(child, true);
                        self.AppendChild(cloned_contents, cloned_child);
                    }
                }
            }
        }
        if deep {
            let children = self.arena[source].children.clone();
            for child in children {
                let cloned_child = self.CloneNode(child, true);
                self.AppendChild(clone, cloned_child);
            }
        }
        let source_id = self.arena[source].id;
        let clone_id = self.arena[clone].id;
        if let Some(&checked) = self.control_checked_state.get(&source_id) {
            self.control_checked_state.insert(clone_id, checked);
        }
        if let Some(value) = self.control_value_state.get(&source_id).cloned() {
            self.control_value_state.insert(clone_id, value);
        }
        clone
    }

    // cpp: dom/document.cc:137-151
    pub fn EnsureTemplateContents(&mut self, template_element: usize) -> usize {
        if !self.arena[template_element].IsElement("template", DOMNamespace::kHTML) {
            invalid_argument("template contents require an HTML template element");
        }
        if let Some(existing) = self.TemplateContents(template_element) {
            return existing;
        }
        let fragment = self.CreateDocumentFragment();
        self.template_contents
            .insert(self.arena[template_element].id, fragment);
        fragment
    }
    pub fn TemplateContents(&self, template_element: usize) -> Option<usize> {
        self.template_contents
            .get(&self.arena[template_element].id)
            .copied()
    }

    // cpp: dom/document.cc:153-162
    pub fn Remove(&mut self, child: usize) {
        let Some(parent) = self.arena[child].parent else {
            return;
        };
        let position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == child)
            .unwrap_or_else(|| logic_error("DOM parent/child links are inconsistent"));
        self.arena[parent].children.remove(position);
        self.arena[child].parent = None;
        self.StyleSheetConnectivityChanged(child);
        let mut detached = vec![child];
        while let Some(node) = detached.pop() {
            self.RemoveResolvedStyle(node);
            self.style_state
                .rules
                .inline
                .borrow_mut()
                .remove(&self.arena[node].id);
            detached.extend_from_slice(&self.arena[node].children);
        }
        self.RecordNonAppendChildrenChange(parent);
    }

    // cpp: dom/document.cc:164-175
    pub fn AppendChild(&mut self, parent: usize, child: usize) {
        if parent == child {
            invalid_argument("a DOM node cannot contain itself");
        }
        let mut ancestor = Some(parent);
        while let Some(index) = ancestor {
            if index == child {
                invalid_argument("DOM insertion would create a cycle");
            }
            ancestor = self.arena[index].parent;
        }
        self.Remove(child);
        let previous_last = self.arena[parent]
            .children
            .iter()
            .rev()
            .copied()
            .find(|&index| self.arena[index].node_type == DOMNodeType::kElement);
        let previous_same_type = (self.arena[child].node_type == DOMNodeType::kElement)
            .then(|| {
                self.arena[parent]
                    .children
                    .iter()
                    .rev()
                    .copied()
                    .find(|&index| {
                        self.arena[index].node_type == DOMNodeType::kElement
                            && self.arena[index].name == self.arena[child].name
                            && self.arena[index].namespace == self.arena[child].namespace
                    })
            })
            .flatten();
        self.arena[parent].children.push(child);
        self.arena[child].parent = Some(parent);
        self.StyleSheetConnectivityChanged(child);
        self.RecordAppendChildrenChange(parent, child, previous_last, previous_same_type);
        let owner = if self.arena[parent].node_type == DOMNodeType::kDocument {
            Some(DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: self.arena[parent].id,
            })
        } else {
            self.arena[parent].owner_document_handle
        };
        if let Some(owner) = owner {
            self.AdoptNodeByHandle(child, owner);
        }
    }

    // cpp: dom/document.cc:177-193
    pub fn InsertBefore(&mut self, parent: usize, child: usize, next_child: usize) {
        if self.arena[next_child].parent != Some(parent) {
            invalid_argument("DOM insertion point is not a child");
        }
        if child == next_child {
            return;
        }
        let mut ancestor = Some(parent);
        while let Some(index) = ancestor {
            if index == child {
                invalid_argument("DOM insertion would create a cycle");
            }
            ancestor = self.arena[index].parent;
        }
        self.Remove(child);
        let position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == next_child)
            .expect("DOM insertion point is not a child");
        self.arena[parent].children.insert(position, child);
        self.arena[child].parent = Some(parent);
        self.StyleSheetConnectivityChanged(child);
        self.RecordNonAppendChildrenChange(parent);
        let owner = if self.arena[parent].node_type == DOMNodeType::kDocument {
            Some(DOMOwnerHandle {
                arena_id: self.arena_id,
                node_id: self.arena[parent].id,
            })
        } else {
            self.arena[parent].owner_document_handle
        };
        if let Some(owner) = owner {
            self.AdoptNodeByHandle(child, owner);
        }
    }

    // cpp: dom/document.cc:195-204
    pub fn AppendText(&mut self, parent: usize, data: &str) -> usize {
        if let Some(&last) = self.arena[parent].children.last() {
            if self.arena[last].node_type == DOMNodeType::kText {
                let was_empty = self.arena[last].data.is_empty();
                self.arena[last].data.push_str(data);
                self.RecordCharacterDataChange(parent, was_empty, self.arena[last].data.is_empty());
                return last;
            }
        }
        let text = self.CreateText(data.into());
        self.AppendChild(parent, text);
        text
    }

    // cpp: dom/document.cc:206-222
    pub fn InsertTextBefore(&mut self, parent: usize, next_child: usize, data: &str) -> usize {
        if self.arena[next_child].parent != Some(parent) {
            invalid_argument("DOM insertion point is not a child");
        }
        let next_position = self.arena[parent]
            .children
            .iter()
            .position(|&index| index == next_child)
            .expect("DOM insertion point is not a child");
        if next_position > 0 {
            let previous = self.arena[parent].children[next_position - 1];
            if self.arena[previous].node_type == DOMNodeType::kText {
                let was_empty = self.arena[previous].data.is_empty();
                self.arena[previous].data.push_str(data);
                self.RecordCharacterDataChange(
                    parent,
                    was_empty,
                    self.arena[previous].data.is_empty(),
                );
                return previous;
            }
        }
        let text = self.CreateText(data.into());
        self.InsertBefore(parent, text, next_child);
        text
    }

    // cpp: dom/document.cc:224-236
    pub fn SetAttribute(&mut self, element: usize, attribute: DOMAttribute) {
        if self.arena[element].node_type != DOMNodeType::kElement {
            invalid_argument("attributes require an element");
        }
        let existing = self.arena[element].attributes.iter().position(|candidate| {
            candidate.local_name == attribute.local_name
                && candidate.namespace_uri == attribute.namespace_uri
        });
        let changed = !existing.is_some_and(|i| self.arena[element].attributes[i] == attribute);
        if std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION").is_some() {
            eprintln!("style-attribute-mutation-profile node_id={} name={:?} namespace={:?} changed={} inline_style={}",
                self.arena[element].id, attribute.local_name, attribute.namespace_uri, changed,
                attribute.local_name == "style" && attribute.namespace_uri.is_empty());
        }
        if !changed {
            return;
        }
        let sibling_sensitive_before = self.NodeMayAffectSiblingSelectors(element);
        let descendant_sensitive_before = self.NodeMayAffectDescendantSelectors(element);
        self.BumpCollectionMembershipRevision();
        let affects_metadata = !matches!(attribute.local_name.as_str(), "style" | "class" | "id");
        let selector_only = if attribute.local_name == "style" && attribute.namespace_uri.is_empty()
        {
            self.RecordStyleChange(crate::style_state::StyleChange::InlineStyle(element));
            if sibling_sensitive_before && !self.style_state.all_dirty {
                self.style_state.sibling_sensitive_nodes.insert(element);
            }
            false
        } else {
            self.RecordAttributeStyleChange(
                element,
                &attribute.local_name,
                &attribute.namespace_uri,
                sibling_sensitive_before,
            )
        };
        // These admitted attributes have no layout metadata consumer. Any
        // selector/attr() presentation change is merged by the style resolver.
        if affects_metadata && !selector_only {
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        if let Some(index) = existing {
            self.arena[element].attributes[index] = attribute;
        } else {
            self.arena[element].attributes.push(attribute);
        }
        // Keep both sides of the mutation: removing a matching class/id is as
        // relevant as adding one, and the old state is no longer observable.
        self.RecordSiblingSensitivity(element, false);
        self.RecordDescendantSensitivity(element, descendant_sensitive_before);
    }

    // cpp: dom/document.cc:238-247
    pub fn RemoveAttribute(&mut self, element: usize, local_name: &str, namespace_uri: &str) {
        if self.arena[element].node_type != DOMNodeType::kElement {
            invalid_argument("attributes require an element");
        }
        if !self.arena[element]
            .attributes
            .iter()
            .any(|a| a.local_name == local_name && a.namespace_uri == namespace_uri)
        {
            return;
        }
        let sibling_sensitive_before = self.NodeMayAffectSiblingSelectors(element);
        let descendant_sensitive_before = self.NodeMayAffectDescendantSelectors(element);
        self.BumpCollectionMembershipRevision();
        if std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION").is_some() {
            eprintln!("style-attribute-mutation-profile node_id={} name={:?} namespace={:?} changed=true inline_style={} removed=true",
                self.arena[element].id, local_name, namespace_uri,
                local_name == "style" && namespace_uri.is_empty());
        }
        let selector_only = if local_name == "style" && namespace_uri.is_empty() {
            self.RecordStyleChange(crate::style_state::StyleChange::InlineStyle(element));
            if sibling_sensitive_before && !self.style_state.all_dirty {
                self.style_state.sibling_sensitive_nodes.insert(element);
            }
            false
        } else {
            self.RecordAttributeStyleChange(
                element,
                local_name,
                namespace_uri,
                sibling_sensitive_before,
            )
        };
        if !selector_only && !matches!(local_name, "style" | "class" | "id") {
            self.style_state
                .impact
                .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        }
        self.arena[element].attributes.retain(|attribute| {
            attribute.local_name != local_name || attribute.namespace_uri != namespace_uri
        });
        self.RecordSiblingSensitivity(element, false);
        self.RecordDescendantSensitivity(element, descendant_sensitive_before);
    }

    // cpp: dom/document.h:115-117
    pub fn RemoveAttributeDefault(&mut self, element: usize, local_name: &str) {
        self.RemoveAttribute(element, local_name, "")
    }

    // cpp: dom/document.cc:249-265
    pub fn SetTextContent(&mut self, node: usize, data: String) {
        if matches!(
            self.arena[node].node_type,
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            if self.arena[node].data == data {
                return;
            }
            let was_empty = self.arena[node].data.is_empty();
            self.arena[node].data = data;
            if let Some(parent) = self.arena[node].parent {
                self.RecordCharacterDataChange(parent, was_empty, self.arena[node].data.is_empty());
            }
            return;
        }
        if !matches!(
            self.arena[node].node_type,
            DOMNodeType::kElement | DOMNodeType::kDocumentFragment
        ) {
            invalid_argument("text content requires a character or container node");
        }
        let children = self.arena[node].children.clone();
        for child in children {
            self.Remove(child);
        }
        if !data.is_empty() {
            self.AppendText(node, &data);
        }
    }

    // cpp: dom/document.cc:267-278
    pub fn ControlChecked(&self, node: usize) -> bool {
        self.control_checked_state
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or_else(|| self.arena[node].FindAttribute("checked").is_some())
    }
    pub fn SetControlChecked(&mut self, node: usize, checked: bool) {
        if !self.arena[node].IsElement("input", DOMNamespace::kHTML) {
            invalid_argument("checked state requires an input element");
        }
        let changed = self.ControlChecked(node) != checked;
        self.control_checked_state
            .insert(self.arena[node].id, checked);
        if !changed {
            return;
        }
        self.InvalidateNodeStyle(node);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }

    // cpp: dom/document.cc:280-326
    fn AppendDescendantText(&self, node: usize, output: &mut String) {
        if self.arena[node].node_type == DOMNodeType::kText {
            output.push_str(&self.arena[node].data);
        }
        for &child in &self.arena[node].children {
            self.AppendDescendantText(child, output);
        }
    }
    fn VisitOptions(&self, node: usize, first: &mut Option<usize>, selected: &mut Option<usize>) {
        for &child in &self.arena[node].children {
            if self.arena[child].IsElement("option", DOMNamespace::kHTML) {
                if first.is_none() {
                    *first = Some(child);
                }
                if selected.is_none() && self.arena[child].FindAttribute("selected").is_some() {
                    *selected = Some(child);
                }
            }
            self.VisitOptions(child, first, selected);
        }
    }
    pub fn ControlValue(&self, node: usize) -> String {
        if let Some(value) = self.control_value_state.get(&self.arena[node].id) {
            return value.clone();
        }
        if self.arena[node].IsElement("input", DOMNamespace::kHTML) {
            return self.arena[node]
                .FindAttribute("value")
                .map_or_else(String::new, |value| value.value.clone());
        }
        if self.arena[node].IsElement("textarea", DOMNamespace::kHTML) {
            let mut value = String::new();
            self.AppendDescendantText(node, &mut value);
            return value;
        }
        if self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            let mut first = None;
            let mut selected = None;
            self.VisitOptions(node, &mut first, &mut selected);
            let Some(option) = selected.or(first) else {
                return String::new();
            };
            if let Some(value) = self.arena[option].FindAttribute("value") {
                return value.value.clone();
            }
            let mut value = String::new();
            self.AppendDescendantText(option, &mut value);
            return value;
        }
        invalid_argument("control value requires an input, textarea, or select");
    }

    // cpp: dom/document.cc:328-333
    pub fn SetControlValue(&mut self, node: usize, value: String) {
        if !self.arena[node].IsElement("input", DOMNamespace::kHTML)
            && !self.arena[node].IsElement("textarea", DOMNamespace::kHTML)
            && !self.arena[node].IsElement("select", DOMNamespace::kHTML)
        {
            invalid_argument("control value requires an input, textarea, or select");
        }
        let previous = self.ControlValue(node);
        let changed = previous != value;
        let empty_changed = previous.is_empty() != value.is_empty();
        self.control_value_state.insert(self.arena[node].id, value);
        if !changed {
            return;
        }
        // Live text values affect selector matching through :placeholder-shown.
        // Changing one nonempty value to another leaves that state (and every
        // attribute selector) unchanged. Keep the broad dependency fallback for
        // an empty/nonempty transition, including ancestors matched by :has().
        // Layout still receives every edit, even when the cascade is unchanged.
        if empty_changed || self.arena[node].IsElement("select", DOMNamespace::kHTML) {
            self.InvalidateNodeStyle(node);
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }

    // cpp: dom/document.cc:335-349
    pub fn FindNodeById(&self, id: u64) -> Option<usize> {
        if id == 0 || id > self.arena.len() as u64 {
            return None;
        }
        let index = id as usize - 1;
        (self.arena[index].id == id).then_some(index)
    }
    pub fn TakeAllChildren(&mut self, source: usize, destination: usize) {
        let children = self.arena[source].children.clone();
        for child in children {
            self.AppendChild(destination, child);
        }
    }

    // cpp: dom/document.h:134-141
    pub fn AppendStyleSheet(&mut self, sheet: CSSStyleSheet) {
        if self
            .FindNodeById(sheet.owner_node_id)
            .is_some_and(|node| self.arena[node].IsHTMLElement("style"))
        {
            self.style_state
                .parsed_style_elements
                .insert(sheet.owner_node_id);
            self.style_state
                .dirty_style_elements
                .remove(&sheet.owner_node_id);
        }
        let root_handle = DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        };
        let detached_owner = self.FindNodeById(sheet.owner_node_id).and_then(|owner| {
            let document = self.arena[owner]
                .owner_document_handle
                .expect("a stylesheet owner must have an owner document");
            (document != root_handle).then_some(document)
        });
        let state = &mut self.style_state;
        self.cssom.ApplyMutation(
            CSSOMMutation::AddStyleSheet {
                sheet,
                detached_owner,
            },
            |event| match event {
                CSSOMEvent::StyleSheetChanged => {
                    state.sheet_revision += 1;
                    state.all_dirty = true;
                }
            },
        );
    }

    // cpp: dom/document.h:142-147
    // Stable arena adapter for source StyleSheets(DOMNode*). This names the
    // receiver itself, including a detached Document; ownerDocument is distinct.
    pub fn StyleSheetsForNode(&self, node: usize) -> &[CSSStyleSheet] {
        self.StyleSheets(Some(DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[node].id,
        }))
    }

    fn StyleSheetConnectivityChanged(&mut self, subtree: usize) {
        let contains_sheet = self.cssom.GetStyleSheets(None).iter().any(|sheet| {
            let mut owner = self.FindNodeById(sheet.owner_node_id);
            while let Some(node) = owner {
                if node == subtree {
                    return true;
                }
                owner = self.arena[node].parent;
            }
            false
        });
        if contains_sheet {
            self.style_state.sheet_revision += 1;
            self.style_state.all_dirty = true;
        }
    }
    pub fn ActiveStyleSheets(&self) -> impl Iterator<Item = &CSSStyleSheet> {
        self.cssom.GetStyleSheets(None).iter().filter(|sheet| {
            if sheet.owner_node_id == 0 {
                return true;
            }
            let Some(mut owner) = self.FindNodeById(sheet.owner_node_id) else {
                return false;
            };
            while let Some(parent) = self.arena[owner].parent {
                owner = parent;
            }
            owner == self.root
        })
    }

    pub fn StyleSheets(&self, owner_document: Option<DOMOwnerHandle>) -> &[CSSStyleSheet] {
        let Some(owner_document) = owner_document else {
            return self.cssom.GetStyleSheets(None);
        };
        let root_handle = DOMOwnerHandle {
            arena_id: self.arena_id,
            node_id: self.arena[self.root].id,
        };
        if owner_document == root_handle {
            return self.cssom.GetStyleSheets(None);
        }
        self.cssom.GetStyleSheets(Some(&owner_document))
    }

    // cpp: dom/document.h:149-156
    pub fn SetAnimationStyle(&mut self, node: u64, effect: u64, declarations: Vec<CSSDeclaration>) {
        if let Some(index) = self.FindNodeById(node) {
            self.RecordStyleChange(crate::style_state::StyleChange::Animation(index));
        }
        let effects = self.animated_styles.entry(node).or_default();
        if declarations.is_empty() {
            effects.remove(&effect);
        } else {
            effects.insert(effect, declarations);
        }
    }
    pub fn AnimationStyles(&self, node: u64) -> &BTreeMap<u64, Vec<CSSDeclaration>> {
        static EMPTY: OnceLock<BTreeMap<u64, Vec<CSSDeclaration>>> = OnceLock::new();
        self.animated_styles
            .get(&node)
            .unwrap_or_else(|| EMPTY.get_or_init(BTreeMap::new))
    }

    // cpp: dom/document.cc:366-378
    pub fn SetImageResource(&mut self, source: String, metadata: ImageResourceMetadata) {
        // Image completion changes the resource and intrinsic geometry, not
        // selector matching across the entire document. Our style adapter
        // embeds background/mask ids, so refresh only their actual consumers;
        // replaced-image metadata is synchronized by the layout tree builder.
        if let Some(consumers) = self.style_image_consumers.get(&source) {
            let consumers: Vec<_> = consumers.iter().copied().collect();
            for index in consumers {
                self.RecordStyleChange(crate::style_state::StyleChange::Resource(index));
            }
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        match self.image_resources.entry(source) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(metadata);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                invalid_argument("image source must be unique");
            }
        }
    }
    pub fn ImageResourceFor(&self, source: &str) -> Option<&ImageResourceMetadata> {
        self.image_resources.get(source)
    }
    pub fn UpdateImageResourceIntrinsicSize(
        &mut self,
        source: &str,
        metadata: ImageResourceMetadata,
    ) -> bool {
        let Some(resource) = self.image_resources.get_mut(source) else {
            return false;
        };
        *resource = metadata;
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
        true
    }

    // cpp: dom/document.h:197-199
    // cpp: dom/document.cc:351-364
    pub fn ResolvedStyleFor(&self, node: usize) -> Option<&ResolvedNodeStyle> {
        self.resolved_styles
            .get(&self.arena[node].id)
            .map(std::sync::Arc::as_ref)
    }
    /// Compatibility entry for a full style invalidation. Keep old values for
    /// comparison and sharing; style reads update before exposing current data.
    pub fn ClearResolvedStyles(&mut self) {
        self.InvalidateAllStyles();
    }
    pub fn StyleState(&self) -> &crate::style_state::StyleState {
        &self.style_state
    }
    pub fn StyleStateMut(&mut self) -> &mut crate::style_state::StyleState {
        &mut self.style_state
    }
    pub fn InvalidateAllStyles(&mut self) {
        self.style_state.all_dirty = true;
        self.style_state.changes.clear();
        self.style_state.sibling_sensitive_nodes.clear();
        self.style_state.append_only_children.clear();
        self.style_state.non_append_children.clear();
        self.style_state.inserted_style_subtrees.clear();
        self.style_state.append_affected_subtrees.clear();
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }
    fn RecordStyleChange(&mut self, change: crate::style_state::StyleChange) {
        if !self.style_state.all_dirty {
            self.style_state.changes.insert(change);
        }
    }
    fn NodeMayAffectSiblingSelectors(&self, node: usize) -> bool {
        let mut has_compiled_rules = false;
        for rules in [
            &self.style_state.rules.user_agent,
            &self.style_state.rules.author,
        ]
        .into_iter()
        .filter_map(|rules| rules.as_ref())
        {
            has_compiled_rules = true;
            if rules.sibling_invalidation_hosts.iter().any(|host| {
                crate::style_resolver::persistent_selector::MatchesSelector(self, node, host)
            }) {
                return true;
            }
        }
        // Before the first rule compilation, retain the conservative behavior.
        !has_compiled_rules
    }
    fn NodeMayAffectDescendantSelectors(&self, node: usize) -> bool {
        let mut has_compiled_rules = false;
        for rules in [
            &self.style_state.rules.user_agent,
            &self.style_state.rules.author,
        ]
        .into_iter()
        .filter_map(|rules| rules.as_ref())
        {
            has_compiled_rules = true;
            if rules.descendant_invalidation_hosts.iter().any(|host| {
                crate::style_resolver::persistent_selector::MatchesSelector(self, node, host)
            }) {
                return true;
            }
        }
        !has_compiled_rules
    }
    fn RecordSiblingSensitivity(&mut self, node: usize, conservative: bool) {
        if self.style_state.all_dirty {
            return;
        }
        let sensitive = conservative || self.NodeMayAffectSiblingSelectors(node);
        if sensitive {
            self.style_state.sibling_sensitive_nodes.insert(node);
        }
    }
    fn RecordDescendantSensitivity(&mut self, node: usize, conservative: bool) {
        if self.style_state.all_dirty {
            return;
        }
        if conservative || self.NodeMayAffectDescendantSelectors(node) {
            self.style_state.descendant_sensitive_nodes.insert(node);
        }
    }
    fn MarkStyleElementTextDirty(&mut self, node: usize) {
        let mut ancestor = Some(node);
        while let Some(index) = ancestor {
            if self.arena[index].IsHTMLElement("style") {
                self.style_state
                    .dirty_style_elements
                    .insert(self.arena[index].id);
                break;
            }
            ancestor = self.arena[index].parent;
        }
    }
    // Changing a coalesced nonempty text node does not add/remove an element
    // sibling or change :empty. Retain text layout and stylesheet reparsing,
    // without promoting every parser body chunk through sibling/:has restyles.
    // Textarea default content also feeds its control value/placeholder state.
    fn RecordCharacterDataChange(&mut self, parent: usize, was_empty: bool, is_empty: bool) {
        self.BumpCollectionMembershipRevision();
        self.MarkStyleElementTextDirty(parent);
        if was_empty != is_empty || self.arena[parent].IsHTMLElement("textarea") {
            self.RecordStyleChange(crate::style_state::StyleChange::Children(parent));
            if !self.style_state.all_dirty {
                self.style_state.non_append_children.insert(parent);
                self.style_state.append_only_children.remove(&parent);
            }
        }
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }
    fn RecordChildrenChange(&mut self, parent: usize) {
        self.BumpCollectionMembershipRevision();
        self.MarkStyleElementTextDirty(parent);
        self.RecordStyleChange(crate::style_state::StyleChange::Children(parent));
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
    }
    fn RecordNonAppendChildrenChange(&mut self, parent: usize) {
        self.RecordChildrenChange(parent);
        if self.style_state.all_dirty {
            return;
        }
        self.style_state.non_append_children.insert(parent);
        self.style_state.append_only_children.remove(&parent);
    }
    fn RecordAppendChildrenChange(
        &mut self,
        parent: usize,
        child: usize,
        previous_last: Option<usize>,
        previous_same_type: Option<usize>,
    ) {
        self.RecordChildrenChange(parent);
        if self.style_state.all_dirty {
            return;
        }
        if !self.style_state.non_append_children.contains(&parent) {
            self.style_state.append_only_children.insert(parent);
        }
        if self.arena[child].node_type == DOMNodeType::kElement {
            self.style_state.inserted_style_subtrees.insert(child);
        }
        for node in [previous_last, previous_same_type].into_iter().flatten() {
            self.style_state.append_affected_subtrees.insert(node);
        }
    }
    fn RecordAttributeStyleChange(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
        sibling_sensitive: bool,
    ) -> bool {
        use cssom::compiled_rules::SelectorOnlyAttribute;
        let kind = if namespace.is_empty() && self.arena[node].namespace == DOMNamespace::kHTML {
            match name {
                "data-pcr" => Some(SelectorOnlyAttribute::DataPcr),
                "src" if self.arena[node].IsHTMLElement("script") => {
                    Some(SelectorOnlyAttribute::ScriptSrc)
                }
                "nonce" if self.arena[node].IsHTMLElement("script") => {
                    Some(SelectorOnlyAttribute::ScriptNonce)
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(kind) = kind {
            self.RecordStyleChange(crate::style_state::StyleChange::Attribute(node, kind));
            if sibling_sensitive && !self.style_state.all_dirty {
                self.style_state.sibling_sensitive_nodes.insert(node);
            }
            true
        } else {
            self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
            if sibling_sensitive && !self.style_state.all_dirty {
                self.style_state.sibling_sensitive_nodes.insert(node);
            }
            false
        }
    }
    pub fn InvalidateNodeStyle(&mut self, node: usize) {
        self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
        self.RecordSiblingSensitivity(node, true);
        self.RecordDescendantSensitivity(node, true);
    }
    pub fn ResolvedStyleHandle(&self, node: usize) -> Option<std::sync::Arc<ResolvedNodeStyle>> {
        self.resolved_styles.get(&self.arena[node].id).cloned()
    }
    pub fn SetResolvedStyle(&mut self, node: usize, style: ResolvedNodeStyle) {
        if self.ResolvedStyleFor(node) == Some(&style) {
            return;
        }
        let same_sources = self
            .ResolvedStyleFor(node)
            .is_some_and(|old| StyleImageSources(old).eq(StyleImageSources(&style)));
        if !same_sources {
            self.RemoveResolvedStyle(node);
            for source in StyleImageSources(&style) {
                self.style_image_consumers
                    .entry(source.to_owned())
                    .or_default()
                    .insert(node);
            }
        }
        self.resolved_styles
            .insert(self.arena[node].id, std::sync::Arc::new(style));
    }
    fn RemoveResolvedStyle(&mut self, node: usize) {
        let Some(style) = self.resolved_styles.remove(&self.arena[node].id) else {
            return;
        };
        for source in StyleImageSources(&style) {
            if let Some(consumers) = self.style_image_consumers.get_mut(source) {
                consumers.remove(&node);
                if consumers.is_empty() {
                    self.style_image_consumers.remove(source);
                }
            }
        }
    }

    // cpp: dom/document.h:166-167
    // cpp: dom/document.cc:380-388
    pub fn SetScrollOffset(&mut self, node: usize, offset: Offset) {
        self.scroll_offsets.insert(self.arena[node].id, offset);
    }
    pub fn ScrollOffsetFor(&self, node: usize) -> Offset {
        self.scroll_offsets
            .get(&self.arena[node].id)
            .copied()
            .unwrap_or_default()
    }

    // cpp: dom/document.h:132-132
    pub fn NodeCount(&self) -> usize {
        self.arena.len()
    }
    // cpp: dom/document.h:129-129
    pub fn NextNodeId(&self) -> u64 {
        self.next_id
    }
}

// cpp: dom/document.h:237-252
// cpp: dom/document.cc:390-399
pub struct DOM {
    document: PersistentDocument,
}
impl Default for DOM {
    fn default() -> Self {
        Self::new()
    }
}
impl DOM {
    pub fn new() -> Self {
        Self {
            document: PersistentDocument::new(),
        }
    }
    pub fn GetDocument(&self) -> &PersistentDocument {
        &self.document
    }
    pub fn GetDocumentMut(&mut self) -> &mut PersistentDocument {
        &mut self.document
    }

    pub fn GetCSSOM(&self) -> &CSSOM<DOMOwnerHandle> {
        &self.document.cssom
    }

    /// Apply a mutation using the existing DOM validation/application algorithm.
    /// The Document's ordinary mutation methods continue accumulating style
    /// invalidations; this facade neither resolves styles nor runs layout.
    pub fn ApplyMutation(
        &mut self,
        mutation: &crate::dom_mutation::DOMMutation,
    ) -> Result<(), crate::dom_mutation::DOMMutationError> {
        crate::dom_mutation::TryApplyDOMMutations(
            &mut self.document,
            std::slice::from_ref(mutation),
        )
    }

    /// HTML depends on DOM, so its fragment parser is injected by the owner.
    /// Only innerHTML invokes it; all other mutations use ApplyMutation.
    pub fn ApplyMutationWithFragmentParser(
        &mut self,
        mutation: &crate::dom_mutation::DOMMutation,
        parser: impl FnOnce(
            &mut PersistentDocument,
            &crate::dom_mutation::DOMMutation,
        ) -> Result<(), crate::dom_mutation::DOMMutationError>,
    ) -> Result<(), crate::dom_mutation::DOMMutationError> {
        if mutation.mutation_type != crate::dom_mutation::DOMMutationType::kSetInnerHTML {
            return self.ApplyMutation(mutation);
        }
        crate::dom_mutation::ValidateDOMMutation(&self.document, mutation, true)?;
        parser(&mut self.document, mutation)
    }

    // A paused HTML parser owns an exclusive Rust borrow of its Document.
    // Swapping transfers that same arena to the script host for one task,
    // then returns it before parsing resumes. DOM node IDs and arena identity
    // stay with the Document value; this performs no cloning or reparsing.
    pub fn SwapDocument(&mut self, document: &mut PersistentDocument) {
        std::mem::swap(&mut self.document, document);
    }

    // During a parser pause the script host temporarily owns the same arena.
    // Restore it on both normal return and unwinding, before parsing resumes.
    pub fn WithDocument<R>(
        owner: &Rc<RefCell<Self>>,
        document: &mut PersistentDocument,
        task: impl FnOnce() -> R,
    ) -> R {
        struct RestoreDocument<'a> {
            owner: Rc<RefCell<DOM>>,
            document: &'a mut PersistentDocument,
        }
        impl Drop for RestoreDocument<'_> {
            fn drop(&mut self) {
                self.owner.borrow_mut().SwapDocument(self.document);
            }
        }
        owner.borrow_mut().SwapDocument(document);
        let _restore = RestoreDocument {
            owner: owner.clone(),
            document,
        };
        task()
    }
}

#[cfg(test)]
mod task_ownership_tests {
    use super::*;

    #[test]
    fn image_completion_tracks_current_style_and_pseudo_consumers() {
        use crate::style_state::StyleChange;
        use layoutng_assembly::internal::paint_input::{BackgroundImageLayer, PaintMaskLayer};
        fn image(source: &str) -> BackgroundImageLayer {
            BackgroundImageLayer {
                source_url: source.into(),
                ..Default::default()
            }
        }
        fn pseudo(source: &str) -> PseudoElement {
            let mut pseudo = PseudoElement::default();
            pseudo.style.paint.mask_images.push(PaintMaskLayer {
                image: image(source),
                ..Default::default()
            });
            pseudo
        }
        fn reset(document: &mut PersistentDocument) {
            let state = document.StyleStateMut();
            state.all_dirty = false;
            state.changes.clear();
            state.impact = Default::default();
        }
        let mut document = PersistentDocument::new();
        let first = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let second = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let unrelated = document.CreateElementDefault(DOMNamespace::kHTML, "img".into());
        for node in [first, parent, unrelated] {
            document.AppendChild(document.Root(), node);
        }
        document.AppendChild(parent, second);
        let mut style = ResolvedNodeStyle::default();
        style.style.paint.background_images.push(image("shared"));
        style.style.paint.mask_images.push(PaintMaskLayer {
            image: image("shared"),
            ..Default::default()
        });
        style.before = Some(pseudo("before"));
        style.after = Some(pseudo("after"));
        style.first_letter = Some(pseudo("first-letter"));
        style.placeholder = Some(pseudo("placeholder"));
        document.SetResolvedStyle(first, style);
        let mut style = ResolvedNodeStyle::default();
        style.style.paint.background_images.push(image("shared"));
        document.SetResolvedStyle(second, style);
        document.SetResolvedStyle(unrelated, Default::default());
        reset(&mut document);
        document.SetImageResource("shared".into(), Default::default());
        assert_eq!(
            document.StyleState().changes,
            [StyleChange::Resource(first), StyleChange::Resource(second)]
                .into_iter()
                .collect()
        );
        assert!(document.StyleState().impact.layout); // Preserve replaced-image geometry invalidation.
        for source in ["before", "after", "first-letter", "placeholder"] {
            reset(&mut document);
            document.SetImageResource(source.into(), Default::default());
            assert_eq!(
                document.StyleState().changes,
                [StyleChange::Resource(first)].into_iter().collect()
            );
        }
        let mut replacement = ResolvedNodeStyle::default();
        replacement
            .style
            .paint
            .background_images
            .push(image("replacement"));
        document.SetResolvedStyle(first, replacement);
        assert_eq!(
            document.style_image_consumers.get("shared"),
            Some(&BTreeSet::from([second]))
        );
        assert!(!document.style_image_consumers.contains_key("before"));
        document.Remove(parent); // Removing a subtree also removes descendant clients.
        assert!(!document.style_image_consumers.contains_key("shared"));
        assert!(document.ResolvedStyleFor(second).is_none());
        reset(&mut document);
        document.SetImageResource("unreferenced".into(), Default::default());
        assert!(document.StyleState().changes.is_empty());
        assert!(document.StyleState().impact.layout);
        reset(&mut document);
        document.SetImageResource("replacement".into(), Default::default());
        assert_eq!(
            document.StyleState().changes,
            [StyleChange::Resource(first)].into_iter().collect()
        );
    }

    #[test]
    fn script_task_restores_the_same_dom_on_unwind() {
        let owner = Rc::new(RefCell::new(DOM::new()));
        let mut tree = PersistentDocument::new();
        let element = tree.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        tree.AppendChild(tree.Root(), element);
        let element_id = tree.Node(element).Id();
        let arena_handle = tree.Node(element).OwnerDocumentNode();
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            DOM::WithDocument(&owner, &mut tree, || {
                let mut live_owner = owner.borrow_mut();
                let live_tree = live_owner.GetDocumentMut();
                assert_eq!(live_tree.Node(element).OwnerDocumentNode(), arena_handle);
                live_tree.SetAttribute(
                    element,
                    DOMAttribute {
                        local_name: "class".into(),
                        value: "changed".into(),
                        ..Default::default()
                    },
                );
                panic!("script task unwound");
            });
        }));
        assert!(failure.is_err());
        assert_eq!(tree.FindNodeById(element_id), Some(element));
        assert_eq!(tree.Node(element).OwnerDocumentNode(), arena_handle);
        assert_eq!(
            tree.Node(element).FindAttribute("class").unwrap().value,
            "changed"
        );
        assert_eq!(owner.borrow().GetDocument().NodeCount(), 1);
    }
}

#[cfg(test)]
mod collection_membership_epoch_tests {
    use super::*;
    #[test]
    fn membership_epoch_covers_direct_tree_attributes_clone_and_whole_node_replacement() {
        let mut d = PersistentDocument::new();
        let root = d.Root();
        let before = d.CollectionMembershipKey();
        let e = d.CreateElementDefault(DOMNamespace::kHTML, "p".into());
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.AppendChild(root, e);
        assert_ne!(before, d.CollectionMembershipKey());
        let attr = DOMAttribute {
            local_name: "class".into(),
            value: "one".into(),
            ..Default::default()
        };
        let before = d.CollectionMembershipKey();
        d.SetAttribute(e, attr.clone());
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.SetAttribute(e, attr);
        assert_eq!(before, d.CollectionMembershipKey());
        d.RemoveAttributeDefault(e, "class");
        assert_ne!(before, d.CollectionMembershipKey());
        let replacement = d.Node(e).clone();
        let before = d.CollectionMembershipKey();
        *d.NodeMut(e) = replacement;
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        let clone = d.CloneNode(e, true);
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.InsertBefore(root, clone, e);
        assert_ne!(before, d.CollectionMembershipKey());
        let before = d.CollectionMembershipKey();
        d.Remove(clone);
        assert_ne!(before, d.CollectionMembershipKey());
    }
    #[test]
    fn membership_epoch_identity_moves_with_document_swap_and_never_wraps() {
        let mut a = DOM::new();
        let mut b = PersistentDocument::new();
        a.document.collection_membership_revision = 21;
        b.collection_membership_revision = 21;
        assert_ne!(
            a.GetDocument().CollectionMembershipKey(),
            b.CollectionMembershipKey()
        );
        let ka = a.GetDocument().CollectionMembershipKey();
        let kb = b.CollectionMembershipKey();
        a.SwapDocument(&mut b);
        assert_eq!(a.GetDocument().CollectionMembershipKey(), kb);
        assert_eq!(b.CollectionMembershipKey(), ka);
        b.collection_membership_revision = u64::MAX - 1;
        assert!(b.CollectionMembershipKey().is_some());
        let root = b.Root();
        let _ = b.NodeMut(root);
        assert!(b.CollectionMembershipKey().is_none());
        let _ = b.NodeMut(root);
        assert_eq!(b.collection_membership_revision, u64::MAX);
        assert!(b.CollectionMembershipKey().is_none());
    }
}
