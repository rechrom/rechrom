// Real PersistentDocument collaborator for the translated SelectorQuery and
// SelectorChecker. Arena indices only name live DOM nodes; they never cache
// element attributes or tree edges. Rc handles are interned by the owning
// service, so identity survives repeated queries and mutations.
#![allow(non_snake_case, unused_variables)]
use crate::css_selector::*;
use crate::css_selector_list::CSSSelectorList;
use crate::parser::css_selector_parser::{
    CSSSelectorParser, SelectorParserContext, SelectorParserOptions,
};
use crate::resolver::match_flags::MatchFlags;
use crate::selector_checker::*;
use crate::selector_filter::*;
use crate::selector_query::*;
use dom::persistent_document::{
    CompatibilityMode, DOMAttribute, DOMAutofillState, DOMNamespace, DOMNodeType, DOMOwnerHandle,
    DOMPartNamesMap, DOMShadowRootMode,
};
use dom::user_interaction_state::UserInteractionState;
use dom::Document;
use foundation::{AtomicString, Persistent};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct HasArgument {
    relation: RelationType,
    depth: i32,
    adjacent: i32,
    sibling_right: bool,
    sibling_between: bool,
    shadow: bool,
    hashes: HashSet<u32>,
}
pub struct HasTraversal {
    nodes: Vec<(Rc<usize>, i32)>,
    position: usize,
}
pub struct HasCache {
    results: RefCell<HashMap<usize, u8>>,
    hashes: HashSet<u32>,
}
pub struct HasFilter {
    allocated: Cell<bool>,
    hashes: RefCell<HashSet<u32>>,
}

// cpp: third_party/blink/renderer/core/css/part_names.h:12-31.
// PartNames is a set in Blink. A small vector keeps the same membership
// semantics here and avoids making AtomicString hashing part of this boundary.
#[derive(Debug, Default)]
pub(crate) struct PersistentPartNames {
    names: RefCell<Vec<AtomicString>>,
    pending_maps: RefCell<Vec<DOMPartNamesMap>>,
}
impl PersistentPartNames {
    pub(crate) fn FromNames(names: impl IntoIterator<Item = AtomicString>) -> Self {
        let mut result = Self::default();
        for name in names {
            if !result.names.get_mut().contains(&name) {
                result.names.get_mut().push(name);
            }
        }
        result
    }
    // cpp: part_names.cc:26-28. Blink queues retained NamesMap references and
    // applies them only when a selector asks Contains(). We retain map values
    // so a cloned collector/checker observes the same pending mapping chain.
    pub(crate) fn PushMap(&mut self, map: &DOMPartNamesMap) {
        self.pending_maps.get_mut().push(map.clone());
    }

    fn ApplyPendingMaps(&self) {
        let pending = std::mem::take(&mut *self.pending_maps.borrow_mut());
        if pending.is_empty() {
            return;
        }
        let mut names = self.names.borrow_mut();
        for map in pending {
            let mut forwarded = Vec::new();
            for name in names.iter() {
                if let Some(values) = map.Get(&name.Utf8()) {
                    for value in values {
                        let value = AtomicString::from_str(value);
                        if !forwarded.contains(&value) {
                            forwarded.push(value);
                        }
                    }
                }
            }
            *names = forwarded;
        }
    }

    pub(crate) fn Contains(&self, name: &AtomicString) -> bool {
        self.ApplyPendingMaps();
        self.names.borrow().contains(name)
    }

    pub(crate) fn Len(&self) -> usize {
        self.ApplyPendingMaps();
        self.names.borrow().len()
    }

    #[cfg(test)]
    fn NamesForTest(&self) -> Vec<String> {
        self.ApplyPendingMaps();
        self.names.borrow().iter().map(AtomicString::Utf8).collect()
    }
}
impl Clone for PersistentPartNames {
    fn clone(&self) -> Self {
        Self {
            names: RefCell::new(self.names.borrow().clone()),
            pending_maps: RefCell::new(self.pending_maps.borrow().clone()),
        }
    }
}
// cpp: style_scope_frame.h:34-124; rule_set.cc:1068-1075.
// These records own the compiled boundaries and parent chain for a rule.
// Frame caches live only for one match request, so DOM mutation cannot leave
// stale activations in the document's retained selector service.
pub struct PersistentStyleScope {
    from: Option<Rc<CSSSelectorList>>,
    to: Option<Rc<CSSSelectorList>>,
    parent: Option<Rc<Self>>,
    // cpp: style_scope_data.h:20-45. A shared StyleSheetContents can back
    // multiple CSSStyleSheets, so one parser-owned StyleScope may be
    // triggered by more than one stylesheet owner parent.
    implicit_roots: Vec<usize>,
}
pub struct PersistentStyleScopeFrame {
    element: Rc<usize>,
    parent: Option<Rc<Self>>,
    data: RefCell<HashMap<usize, Rc<PersistentStyleScopeActivations>>>,
}
impl PersistentStyleScopeFrame {
    pub(crate) fn new(element: Rc<usize>, parent: Option<Rc<Self>>) -> Rc<Self> {
        Rc::new(Self {
            element,
            parent,
            data: RefCell::new(HashMap::new()),
        })
    }

    pub(crate) fn Element(&self) -> &Rc<usize> {
        &self.element
    }
}
pub struct PersistentStyleScopeActivation {
    root: Option<Rc<usize>>,
    proximity: u32,
}
pub struct PersistentStyleScopeActivations {
    vector: Vec<Rc<PersistentStyleScopeActivation>>,
    flags: MatchFlags,
}

/// Admitted frame setting for spatial-navigation selector matching.
///
/// cpp: core/frame/settings.json5:397-402 defaults spatialNavigationEnabled
/// to false. Rechrom has no spatial-navigation controller/active-selection-end
/// producer yet, so enabling it must be rejected at this input boundary rather
/// than represented by fabricated focus state or a negative selector result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpatialNavigationPolicy {
    #[default]
    Disabled,
}

impl TryFrom<bool> for SpatialNavigationPolicy {
    type Error = PersistentSelectorError;

    fn try_from(enabled: bool) -> Result<Self, Self::Error> {
        if enabled {
            Err(PersistentSelectorError::Unsupported(
                "enabled spatial navigation requires its interaction controller and listbox active-selection-end owner".into(),
            ))
        } else {
            Ok(Self::Disabled)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopedSelectorMatch {
    pub specificity: u32,
    pub proximity: u32,
}
// Blink's AttributeCollection is a borrowed view over ElementData. Keep the
// document's attributes resident instead of copying their strings and building
// per-attribute Rc owners for every attribute revision.
fn AttributeNameMatches(data: &DOMAttribute, name: &QualifiedName, ignore_case: bool) -> bool {
    (if ignore_case {
        name.LocalName()
            .EqualsUtf8IgnoringAsciiCase(&data.local_name)
    } else {
        name.LocalName().EqualsUtf8(&data.local_name)
    }) && (name.NamespaceURI().IsNull() && data.namespace_uri.is_empty()
        || name.NamespaceURI().EqualsUtf8("*")
        || name.NamespaceURI().EqualsUtf8(&data.namespace_uri))
}

#[derive(Default)]
struct PersistentElementFilters {
    ancestor: Rc<RefCell<SelectorFilter>>,
    subjects: HashMap<usize, (u64, u32)>,
    subtrees: HashMap<usize, (u64, u32)>,
    identifiers: HashMap<usize, (u64, Rc<Vec<u32>>)>,
    classes: HashMap<usize, (u64, Rc<[AtomicString]>)>,
    // Blink's Attribute stores atomized names and values in ElementData. The
    // compact DOM boundary keeps UTF-8 strings, so retain their exact atoms in
    // the document-owned selector cache instead of re-entering the global atom
    // table for every RuleData candidate.
    attribute_atoms: HashMap<String, AtomicString>,
    empty_classes: Rc<[AtomicString]>,
    local_name_atoms: HashMap<usize, AtomicString>,
    id_atoms: HashMap<usize, (u64, AtomicString)>,
    ancestor_revision: Option<(DOMOwnerHandle, u64)>,
}
pub(crate) struct PersistentSelectorBackend<'d> {
    document: &'d Document,
    ancestor: Rc<RefCell<SelectorFilter>>,
    filters: Rc<RefCell<PersistentElementFilters>>,
    nodes: Rc<RefCell<HashMap<usize, Rc<usize>>>>,
    interaction: UserInteractionState,
    spatial_navigation_policy: SpatialNavigationPolicy,
    forced: HashMap<u64, Vec<PseudoType>>,
    frame_active: bool,
    slot_assignment_dirty: HashSet<u64>,
    vtt_elements: HashSet<u64>,
    has_depth: Cell<u32>,
    query_depth: Cell<u32>,
    nth_depth: Cell<u32>,
    element_flags: Rc<RefCell<HashMap<usize, Vec<ElementInvalidationFlag>>>>,
    subject_flags: Rc<RefCell<HashMap<usize, crate::resolver::match_flags::MatchFlags>>>,
    sibling_has_flags: Rc<RefCell<HashMap<usize, u8>>>,
    container_flags: Rc<RefCell<HashMap<usize, Vec<ContainerInvalidationFlag>>>>,
    logical_pseudo_enabled: Cell<bool>,
}
impl<'d> PersistentSelectorBackend<'d> {
    fn new(
        document: &'d Document,
        filters: Rc<RefCell<PersistentElementFilters>>,
        element_flags: Rc<RefCell<HashMap<usize, Vec<ElementInvalidationFlag>>>>,
        subject_flags: Rc<RefCell<HashMap<usize, crate::resolver::match_flags::MatchFlags>>>,
        sibling_has_flags: Rc<RefCell<HashMap<usize, u8>>>,
        container_flags: Rc<RefCell<HashMap<usize, Vec<ContainerInvalidationFlag>>>>,
        nodes: Rc<RefCell<HashMap<usize, Rc<usize>>>>,
        interaction: UserInteractionState,
        spatial_navigation_policy: SpatialNavigationPolicy,
        forced: HashMap<u64, Vec<PseudoType>>,
        frame_active: bool,
        slot_assignment_dirty: HashSet<u64>,
        vtt_elements: HashSet<u64>,
    ) -> Rc<Self> {
        let ancestor = filters.borrow().ancestor.clone();
        Rc::new(Self {
            document,
            ancestor,
            filters,
            nodes,
            interaction,
            spatial_navigation_policy,
            forced,
            frame_active,
            slot_assignment_dirty,
            vtt_elements,
            has_depth: Cell::new(0),
            query_depth: Cell::new(0),
            nth_depth: Cell::new(0),
            element_flags,
            subject_flags,
            sibling_has_flags,
            container_flags,
            logical_pseudo_enabled: Cell::new(true),
        })
    }
    pub(crate) fn handle(&self, index: usize) -> Rc<usize> {
        assert!(
            index < self.document.NodeCount(),
            "selector node belongs to the document"
        );
        self.nodes
            .borrow_mut()
            .entry(index)
            .or_insert_with(|| Rc::new(index))
            .clone()
    }

    pub(crate) fn document(&self) -> &'d Document {
        self.document
    }

    /// Runs one translated SelectorChecker operation with Chromium's runtime
    /// feature state for logical pseudo matching, then restores the shared
    /// selector service before it can be reused by querySelector or another
    /// rule-collection pass.
    pub(crate) fn WithLogicalPseudoMode<T>(
        &self,
        enabled: bool,
        operation: impl FnOnce() -> T,
    ) -> T {
        struct LogicalPseudoScope<'a> {
            value: &'a Cell<bool>,
            previous: bool,
        }
        impl Drop for LogicalPseudoScope<'_> {
            fn drop(&mut self) {
                self.value.set(self.previous);
            }
        }
        let _scope = LogicalPseudoScope {
            value: &self.logical_pseudo_enabled,
            previous: self.logical_pseudo_enabled.replace(enabled),
        };
        operation()
    }
    fn element(&self, index: usize) -> Option<Rc<usize>> {
        (self.document.Node(index).Type() == DOMNodeType::kElement).then(|| self.handle(index))
    }
    fn attr(&self, index: usize, name: &str) -> Option<&str> {
        self.document
            .Node(index)
            .FindAttribute(name)
            .map(|v| v.value.as_str())
    }
    fn scope(&self, index: usize) -> usize {
        let mut i = index;
        while let Some(p) = self.document.Node(i).Parent() {
            i = p;
        }
        i
    }
    // cpp: user_action_element_traversal.h:19-35;
    //      document.cc:8977-9099. Blink materializes :hover/:active on the
    // target's inclusive flat-tree ancestor chain. HTMLLabelElement also
    // mirrors both states to its associated control. The stable top-layer
    // behavior stops before leaving a modal/popover subtree.
    fn in_user_action_chain(&self, element: usize, target: Option<u64>) -> bool {
        let Some(mut current) = target.and_then(|id| self.document.FindNodeById(id)) else {
            return false;
        };
        loop {
            if current == element
                || (self.document.Node(current).IsHTMLElement("label")
                    && self.document.LabelControl(current) == Some(element))
            {
                return true;
            }
            if self.document.IsInTopLayer(current) {
                return false;
            }
            let Some(parent) = self.document.FlatTreeParentElement(current) else {
                return false;
            };
            current = parent;
        }
    }
    fn sibling(&self, index: usize, forward: bool, elements: bool) -> Option<Rc<usize>> {
        let siblings = self
            .document
            .Node(self.document.Node(index).Parent()?)
            .Children();
        let position = siblings.iter().position(|&v| v == index)?;
        let candidates: Box<dyn Iterator<Item = &usize>> = if forward {
            Box::new(siblings[position + 1..].iter())
        } else {
            Box::new(siblings[..position].iter().rev())
        };
        candidates
            .filter(|&&v| !elements || self.document.Node(v).Type() == DOMNodeType::kElement)
            .next()
            .map(|&v| self.handle(v))
    }
    fn descendants(&self, index: usize, out: &mut Vec<Rc<usize>>) {
        for &child in self.document.Node(index).Children() {
            if let Some(e) = self.element(child) {
                out.push(e);
            }
            self.descendants(child, out);
        }
    }
    fn class_contains(&self, index: usize, class: &AtomicString) -> bool {
        let classes = self.ClassNamesForElement(index);
        classes.iter().any(|candidate| {
            if self.InQuirksMode(&index) {
                candidate.EqualsUtf8IgnoringAsciiCase(&class.Utf8())
            } else {
                candidate == class
            }
        })
    }
    fn namespace(namespace: DOMNamespace) -> AtomicString {
        static HTML: std::sync::LazyLock<AtomicString> =
            std::sync::LazyLock::new(|| AtomicString::from_str("http://www.w3.org/1999/xhtml"));
        static SVG: std::sync::LazyLock<AtomicString> =
            std::sync::LazyLock::new(|| AtomicString::from_str("http://www.w3.org/2000/svg"));
        static MATHML: std::sync::LazyLock<AtomicString> = std::sync::LazyLock::new(|| {
            AtomicString::from_str("http://www.w3.org/1998/Math/MathML")
        });
        match namespace {
            DOMNamespace::kHTML => HTML.clone(),
            DOMNamespace::kSVG => SVG.clone(),
            DOMNamespace::kMathML => MATHML.clone(),
            DOMNamespace::kNone => AtomicString::from_str(""),
        }
    }

    pub(crate) fn AttributeAtom(&self, value: &str) -> AtomicString {
        if let Some(atom) = self.filters.borrow().attribute_atoms.get(value) {
            return atom.clone();
        }
        let atom = AtomicString::from_str(value);
        self.filters
            .borrow_mut()
            .attribute_atoms
            .insert(value.to_owned(), atom.clone());
        atom
    }
    pub(crate) fn RememberSubjectFlags(
        &self,
        element: usize,
        flags: crate::resolver::match_flags::MatchFlags,
    ) {
        *self.subject_flags.borrow_mut().entry(element).or_default() |= flags;
    }
    pub(crate) fn ElementSubjectFilter(&self, index: usize) -> u32 {
        let revision = self.document.SelectorFilterRevisions(index).0;
        if let Some((_, bits)) = self
            .filters
            .borrow()
            .subjects
            .get(&index)
            .filter(|(key, _)| *key == revision)
        {
            return *bits;
        }
        let mut bits = 0;
        for atom in self.ClassNamesForElement(index).iter() {
            bits |= FilterForString(&if self.InQuirksMode(&index) {
                atom.ToAsciiLower()
            } else {
                atom.clone()
            });
        }
        for a in self.document.Node(index).Attributes() {
            let name = QualifiedName::new(
                AtomicString::from_str(&a.prefix),
                AtomicString::from_str(&a.local_name),
                AtomicString::from_str(&a.namespace_uri),
            );
            if !self.IsExcludedAttribute(
                &name,
                AttributesToExcludeHashesFor::kExcludeAllLazilySynchronizedAttributes,
            ) {
                bits |= FilterForAttribute(&name);
            }
        }
        self.filters
            .borrow_mut()
            .subjects
            .insert(index, (revision, bits));
        bits
    }
    fn bloom(&self, index: usize) -> u32 {
        // cpp: Element::UpdateSubtreeBloomFilterAfterInsert/ChildRemoval.
        // Blink retains these subtree facts and propagates attribute/child
        // changes through parents. The DOM supplies that affected path through
        // revisions, so clean branches keep their existing style-owned bloom.
        let revision = self.document.SelectorFilterRevisions(index).1;
        if let Some((_, bits)) = self
            .filters
            .borrow()
            .subtrees
            .get(&index)
            .filter(|(key, _)| *key == revision)
        {
            return *bits;
        }
        let _trace = browser_tracing::span("style_match", "SelectorFilter.BuildSubtree");
        let mut bits = self.ElementSubjectFilter(index);
        for &child in self.document.Node(index).Children() {
            bits |= self.bloom(child);
        }
        self.filters
            .borrow_mut()
            .subtrees
            .insert(index, (revision, bits));
        bits
    }
    // cpp: SelectorFilter::PushAllParentsOf/PushAncestors/PushParent.
    pub(crate) fn SelectorFilterForElement(
        &self,
        element: usize,
    ) -> std::cell::Ref<'_, SelectorFilter> {
        let parent = self.document.ParentOrShadowHostElement(element);
        let filter = self.ancestor.borrow();
        // A valid parent stack also serves every sibling and each later
        // origin/pseudo pass of this element. DOM membership revisions reset
        // the owner before a new session, including ancestor attribute edits.
        if filter.ParentStackIsConsistent(parent) {
            return filter;
        }
        drop(filter);
        let mut parents = Vec::new();
        let mut parent = parent;
        while let Some(index) = parent {
            parents.push(index);
            parent = self.document.ParentOrShadowHostElement(index);
        }
        parents.reverse();
        let mut filter = self.ancestor.borrow_mut();
        let common = filter
            .Parents()
            .iter()
            .zip(&parents)
            .take_while(|(a, b)| a == b)
            .count();
        filter.PopToParentCount(common);
        for &index in &parents[common..] {
            let hashes = self.ElementIdentifierHashes(index);
            filter.PushParent(index, hashes.iter().copied());
        }
        drop(filter);
        // ElementRuleCollector borrows the resolver's resident SelectorFilter
        // in Blink. Keep that lifetime separate from mutable identifier caches;
        // no bitmap, parent stack or identifier-vector copy is made per pass.
        self.ancestor.borrow()
    }

    // ElementData retains the atomized class list in Blink. Reuse the
    // unchanged element's immutable owner across collectors and RuleSetGroups.
    // Typed attribute revisions revoke only that element's retained facts.
    pub(crate) fn ClassNamesForElement(&self, index: usize) -> Rc<[AtomicString]> {
        let revision = self.document.SelectorFilterRevisions(index).0;
        if let Some((_, classes)) = self
            .filters
            .borrow()
            .classes
            .get(&index)
            .filter(|(key, _)| *key == revision)
        {
            return classes.clone();
        }
        let mut names = self
            .attr(index, "class")
            .unwrap_or("")
            .split_ascii_whitespace()
            .peekable();
        let quirks = self.InQuirksMode(&index);
        let classes: Rc<[AtomicString]> = if names.peek().is_none() {
            self.filters.borrow().empty_classes.clone()
        } else {
            names
                .map(AtomicString::from_str)
                .map(|name| if quirks { name.ToAsciiLower() } else { name })
                .collect::<Vec<_>>()
                .into()
        };
        self.filters
            .borrow_mut()
            .classes
            .insert(index, (revision, classes.clone()));
        classes
    }

    fn ElementIdentifierHashes(&self, index: usize) -> Rc<Vec<u32>> {
        let revision = self.document.SelectorFilterRevisions(index).0;
        if let Some((_, hashes)) = self
            .filters
            .borrow()
            .identifiers
            .get(&index)
            .filter(|(key, _)| *key == revision)
        {
            return hashes.clone();
        }
        let mut hashes = vec![self
            .ElementLocalName(&index)
            .Hash()
            .wrapping_mul(kTagNameSalt)];
        if self.HasID(&index) {
            hashes.push(
                self.IdForStyleResolution(&index)
                    .Hash()
                    .wrapping_mul(kIdSalt),
            );
        }
        for atom in self.ClassNamesForElement(index).iter() {
            let atom = if self.InQuirksMode(&index) {
                atom.ToAsciiLower()
            } else {
                atom.clone()
            };
            hashes.push(atom.Hash().wrapping_mul(kClassSalt));
        }
        for attribute in self.document.Node(index).Attributes() {
            let name = AtomicString::from_str(&attribute.local_name).ToAsciiLower();
            if !matches!(name.Utf8().as_str(), "id" | "class" | "style") {
                hashes.push(name.Hash().wrapping_mul(kAttributeSalt));
            }
        }
        let hashes = Rc::new(hashes);
        self.filters
            .borrow_mut()
            .identifiers
            .insert(index, (revision, hashes.clone()));
        hashes
    }
    fn tag_matches(&self, index: usize, name: &QualifiedName) -> bool {
        self.ElementTagQName(&index) == *name
    }
    fn identifier_hashes(&self, index: usize) -> HashSet<u32> {
        let mut hashes = HashSet::new();
        hashes.insert(self.ElementLocalName(&index).Hash().wrapping_mul(13));
        if let Some(id) = self.attr(index, "id") {
            hashes.insert(self.IdForStyleResolution(&index).Hash().wrapping_mul(17));
        }
        if let Some(classes) = self.attr(index, "class") {
            for c in classes.split_ascii_whitespace() {
                let atom = AtomicString::from_str(c);
                hashes.insert(
                    if self.InQuirksMode(&index) {
                        atom.ToAsciiLower()
                    } else {
                        atom
                    }
                    .Hash()
                    .wrapping_mul(19),
                );
            }
        }
        for a in self.document.Node(index).Attributes() {
            hashes.insert(
                AtomicString::from_str(&a.local_name)
                    .ToAsciiLower()
                    .Hash()
                    .wrapping_mul(23),
            );
        }
        hashes
    }
    fn has_walk(&self, index: usize, depth: i32, limit: i32, out: &mut Vec<(Rc<usize>, i32)>) {
        out.push((self.handle(index), depth));
        if depth < limit {
            for &child in self.document.Node(index).Children() {
                if self.element(child).is_some() {
                    self.has_walk(child, depth + 1, limit, out);
                }
            }
        }
    }
}
impl SelectorQueryFilterBackend for PersistentSelectorBackend<'_> {
    fn IsExcludedAttribute(
        &self,
        name: &QualifiedName,
        policy: AttributesToExcludeHashesFor,
    ) -> bool {
        // PersistentDocument synchronizes all stored attributes eagerly; only
        // the standard identifiers excluded by Blink remain excluded.
        matches!(name.LocalName().Utf8().as_str(), "id" | "class" | "style")
    }
}
impl SelectorQueryBackend for PersistentSelectorBackend<'_> {
    type QueryNthIndexCache = ();
    fn QueryParentNode(&self, node: &usize) -> Option<Rc<usize>> {
        self.document.Node(*node).Parent().map(|v| self.handle(v))
    }
    fn QueryPreviousSiblingNode(&self, node: &usize) -> Option<Rc<usize>> {
        self.sibling(*node, false, false)
    }
    fn QueryFirstChildElement(&self, node: &usize) -> Option<Rc<usize>> {
        self.document
            .Node(*node)
            .Children()
            .iter()
            .find_map(|&v| self.element(v))
    }
    fn QueryNextSiblingElement(&self, node: &usize) -> Option<Rc<usize>> {
        self.sibling(*node, true, true)
    }
    fn QueryIsShadowRoot(&self, node: &usize) -> bool {
        self.TreeScopeIsShadowRoot(node)
    }
    fn QueryIsHTMLDocument(&self, node: &usize) -> bool {
        self.IsInHTMLDocument(node)
    }
    fn QueryInQuirksMode(&self, node: &usize) -> bool {
        self.InQuirksMode(node)
    }
    fn QueryIsInTreeScope(&self, node: &usize) -> bool {
        let root = self.scope(*node);
        self.document.Node(root).Type() == DOMNodeType::kDocument
            || self.document.ShadowHost(root).is_some()
    }
    fn QueryTreeScope(&self, node: &usize) -> Rc<usize> {
        self.handle(self.scope(*node))
    }
    fn QuerySubtreeBloomFilter(&self, e: &usize) -> u32 {
        self.bloom(*e)
    }
    fn QueryContainsMultipleElementsWithId(&self, scope: &usize, id: &AtomicString) -> bool {
        self.QueryAllElementsById(scope, id).len() > 1
    }
    fn QueryAllElementsById(&self, scope: &usize, id: &AtomicString) -> Vec<Rc<usize>> {
        let mut out = vec![];
        self.descendants(*scope, &mut out);
        out.retain(|v| self.IdForStyleResolution(v) == *id);
        out
    }
    fn QueryElementById(&self, scope: &usize, id: &AtomicString) -> Option<Rc<usize>> {
        self.QueryAllElementsById(scope, id).into_iter().next()
    }
    fn EnterQueryHasCacheScope(&self, root: &usize) -> bool {
        self.query_depth.set(self.query_depth.get() + 1);
        false
    }
    fn EnterQueryNthIndexCache(&self, root: &usize) {
        self.nth_depth.set(self.nth_depth.get() + 1);
    }
    fn ExitQueryNthIndexCache(&self, cache: ()) {
        self.nth_depth.set(self.nth_depth.get() - 1);
    }
}
impl SelectorCheckerBackend for PersistentSelectorBackend<'_> {
    type Element = usize;
    type ContainerNode = usize;
    type TreeScope = usize;
    type StyleScope = PersistentStyleScope;
    type StyleScopeFrame = PersistentStyleScopeFrame;
    type StyleScopeActivations = PersistentStyleScopeActivations;
    type StyleScopeActivation = PersistentStyleScopeActivation;
    type ElementResolveContext = ();
    type PartNames = PersistentPartNames;
    type CustomScrollbar = ();
    type ScrollbarPart = ();
    type StyleRequest = ();
    type Attribute = DOMAttribute;
    type ViewTransition = ();
    type Node = usize;
    type HasArgumentContext = HasArgument;
    type HasTraversal = HasTraversal;
    type HasCacheScope = bool;
    type HasCacheContext = HasCache;
    type HasFastRejectFilter = HasFilter;
    fn ResolveUltimateOriginatingElementOrSelf(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Rc<Self::Element> {
        panic!("Chromium DOM/runtime collaborator pending: ResolveUltimateOriginatingElementOrSelf")
    }
    fn ResolvePseudoElement(&self, c: &Self::ElementResolveContext) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ResolvePseudoElement")
    }
    fn ResolvePseudoElementAncestors(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Vec<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ResolvePseudoElementAncestors")
    }
    fn RequestScrollbar(&self, r: &Self::StyleRequest) -> Option<Rc<Self::CustomScrollbar>> {
        panic!("Chromium DOM/runtime collaborator pending: RequestScrollbar")
    }
    fn RequestPseudoArgument(&self, r: &Self::StyleRequest) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: RequestPseudoArgument")
    }
    fn RequestPseudoIdentList(&self, r: &Self::StyleRequest) -> Vec<AtomicString> {
        panic!("Chromium DOM/runtime collaborator pending: RequestPseudoIdentList")
    }
    fn RequestScrollbarPart(&self, r: &Self::StyleRequest) -> Self::ScrollbarPart {
        panic!("Chromium DOM/runtime collaborator pending: RequestScrollbarPart")
    }
    fn NoScrollbarPart(&self) -> Self::ScrollbarPart {
        ()
    }
    fn TreeScopeShadowHost(&self, s: &Self::TreeScope) -> Option<Rc<Self::Element>> {
        self.document.ShadowHost(*s).map(|host| self.handle(host))
    }
    fn TreeScopeIsShadowRoot(&self, s: &Self::TreeScope) -> bool {
        self.document.ShadowHost(*s).is_some()
    }
    fn ParentTreeScope(&self, s: &Self::TreeScope) -> Option<Rc<Self::TreeScope>> {
        self.document
            .ShadowHost(*s)
            .map(|host| self.handle(self.document.TreeScopeRoot(host)))
    }
    fn ElementTreeScope(&self, e: &Self::Element) -> Rc<Self::TreeScope> {
        self.handle(self.scope(*e))
    }
    fn ParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .Node(*e)
            .Parent()
            .and_then(|v| self.element(v))
    }
    fn ParentOrShadowHostElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .ParentOrShadowHostElement(*e)
            .map(|element| self.handle(element))
    }
    fn PreviousSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.sibling(*e, false, true)
    }
    fn NextSibling(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.sibling(*e, true, true)
    }
    fn PreviousSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>> {
        let mut current = self.PreviousSibling(e);
        while let Some(v) = current {
            if self.tag_matches(*v, name) {
                return Some(v);
            }
            current = self.PreviousSibling(&v);
        }
        None
    }
    fn NextSiblingWithTagName(
        &self,
        e: &Self::Element,
        name: &QualifiedName,
    ) -> Option<Rc<Self::Element>> {
        let mut current = self.NextSibling(e);
        while let Some(v) = current {
            if self.tag_matches(*v, name) {
                return Some(v);
            }
            current = self.NextSibling(&v);
        }
        None
    }
    fn AssignedSlot(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document.AssignedSlot(*e).map(|slot| self.handle(slot))
    }
    fn IsSlotSupportingAssignment(&self, e: &Self::Element) -> bool {
        self.document.IsSlotSupportingAssignment(*e)
    }
    fn InQuirksMode(&self, e: &Self::Element) -> bool {
        self.document.GetCompatibilityMode() == CompatibilityMode::kQuirks
    }
    fn IsLink(&self, e: &Self::Element) -> bool {
        matches!(self.document.Node(*e).Name(), "a" | "area" | "link")
            && self.attr(*e, "href").is_some()
    }
    fn IsPseudoElement(&self, e: &Self::Element) -> bool {
        self.document.Node(*e).Type() != DOMNodeType::kElement
    }
    fn IsScrollButtonPseudoElement(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsScrollButtonPseudoElement")
    }
    fn ParentComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>> {
        panic!("Chromium DOM/runtime collaborator pending: ParentComputedStyle")
    }
    fn ScrollButtonPseudoIdFromArgument(
        &self,
        argument: &AtomicString,
        style: &ComputedStyle,
    ) -> PseudoId {
        panic!("Chromium DOM/runtime collaborator pending: ScrollButtonPseudoIdFromArgument")
    }
    fn ElementPseudoId(&self, e: &Self::Element) -> PseudoId {
        PseudoId::kPseudoIdNone
    }
    fn TransitionForElement(&self, _: &Self::Element) -> Option<Rc<Self::ViewTransition>> {
        // PersistentDocument does not currently retain a ViewTransition
        // object. Ordinary element matching therefore has no transition.
        None
    }
    fn CSSLogicalCombinationPseudoEnabled(&self) -> bool {
        self.logical_pseudo_enabled.get()
    }
    fn ParentElementOrShadowRoot(&self, e: &Self::Element) -> Option<Rc<Self::ContainerNode>> {
        self.document
            .Node(*e)
            .Parent()
            .filter(|&p| {
                self.document.Node(p).Type() == DOMNodeType::kElement
                    || self.document.ShadowHost(p).is_some()
            })
            .map(|p| self.handle(p))
    }
    fn SetChildrenAffectedByDirectAdjacentRules(&self, node: &Self::ContainerNode) {
        self.SetContainerFlag(
            node,
            ContainerInvalidationFlag::ChildrenAffectedByDirectAdjacentRules,
        );
    }
    fn SetChildrenAffectedByIndirectAdjacentRules(&self, node: &Self::ContainerNode) {
        self.SetContainerFlag(
            node,
            ContainerInvalidationFlag::ChildrenAffectedByIndirectAdjacentRules,
        );
    }
    fn OwnerShadowHost(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .OwnerShadowHost(*e)
            .map(|host| self.handle(host))
    }
    fn ActivationMatchFlags(&self, activations: &Self::StyleScopeActivations) -> MatchFlags {
        activations.flags
    }
    fn ActivationVector(
        &self,
        activations: &Self::StyleScopeActivations,
    ) -> Vec<Rc<Self::StyleScopeActivation>> {
        activations.vector.clone()
    }
    fn ActivationRoot(
        &self,
        activation: &Self::StyleScopeActivation,
    ) -> Option<Rc<Self::ContainerNode>> {
        activation.root.clone()
    }
    fn ActivationProximity(&self, activation: &Self::StyleScopeActivation) -> u32 {
        activation.proximity
    }
    fn SynchronizeAttribute(&self, e: &Self::Element, name: &AtomicString) { // PersistentDocument::SetAttribute commits attribute values synchronously.
    }
    fn CouldHaveAttribute(&self, e: &Self::Element, name: &QualifiedName) -> bool {
        let html_element = self.IsHTMLElement(e);
        let html_document = self.IsInHTMLDocument(e);
        self.document.Node(*e).Attributes().iter().any(|attribute| {
            AttributeNameMatches(attribute, name, false)
                || (!html_element && html_document && AttributeNameMatches(attribute, name, true))
        })
    }
    fn CSSAttributeValueCaseSensitiveNonHTMLEnabled(&self) -> bool {
        true
    }
    fn IsHTMLElement(&self, e: &Self::Element) -> bool {
        self.document.Node(*e).Namespace() == DOMNamespace::kHTML
    }
    fn IsInHTMLDocument(&self, e: &Self::Element) -> bool {
        self.document.IsInHTMLDocument(*e)
    }
    fn AttributesWithoutUpdate(&self, e: &Self::Element) -> &[Self::Attribute] {
        self.document.Node(*e).Attributes()
    }
    fn AttributeMatchesName(&self, a: &Self::Attribute, name: &QualifiedName) -> bool {
        AttributeNameMatches(a, name, false)
    }
    fn AttributeMatchesNameCaseInsensitive(
        &self,
        a: &Self::Attribute,
        name: &QualifiedName,
    ) -> bool {
        AttributeNameMatches(a, name, true)
    }
    fn AttributeValue(&self, a: &Self::Attribute) -> AtomicString {
        self.AttributeAtom(&a.value)
    }
    fn CouldHaveClass(&self, e: &Self::Element, class: &AtomicString) -> bool {
        self.class_contains(*e, class)
    }
    fn HasClass(&self, e: &Self::Element) -> bool {
        !self.ClassNamesForElement(*e).is_empty()
    }
    fn ClassNamesContain(&self, e: &Self::Element, class: &AtomicString) -> bool {
        self.class_contains(*e, class)
    }
    fn HasID(&self, e: &Self::Element) -> bool {
        self.attr(*e, "id").is_some_and(|v| !v.is_empty())
    }
    fn IdForStyleResolution(&self, e: &Self::Element) -> AtomicString {
        let revision = self.document.SelectorFilterRevisions(*e).0;
        if let Some((_, value)) = self
            .filters
            .borrow()
            .id_atoms
            .get(e)
            .filter(|(key, _)| *key == revision)
        {
            return value.clone();
        }
        let value = self
            .attr(*e, "id")
            .filter(|v| !v.is_empty())
            .map(|v| {
                let a = AtomicString::from_str(v);
                if self.InQuirksMode(e) {
                    a.ToAsciiLower()
                } else {
                    a
                }
            })
            .unwrap_or_default();
        self.filters
            .borrow_mut()
            .id_atoms
            .insert(*e, (revision, value.clone()));
        value
    }
    fn ElementIs(&self, e: &Self::Element, class: ElementClass) -> bool {
        let node = self.document.Node(*e);
        let name = node.Name();
        use ElementClass::*;
        match class {
            Option => node.IsHTMLElement("option"),
            Input => node.IsHTMLElement("input"),
            Select => node.IsHTMLElement("select"),
            SVG => node.Namespace() == DOMNamespace::kSVG,
            Image => node.IsHTMLElement("img"),
            TextControl => node.IsHTMLElement("textarea") || self.IsTextField(e),
            Form => node.IsHTMLElement("form"),
            HTMLElement => node.Namespace() == DOMNamespace::kHTML,
            FormControl | FormControlWithState => {
                node.Namespace() == DOMNamespace::kHTML
                    && matches!(
                        name,
                        "button"
                            | "fieldset"
                            | "input"
                            | "object"
                            | "output"
                            | "select"
                            | "textarea"
                            | "option"
                            | "optgroup"
                    )
            }
            Anchor => matches!(name, "a" | "area"),
            Dialog => node.IsHTMLElement("dialog"),
            Details => node.IsHTMLElement("details"),
            Media => matches!(name, "audio" | "video"),
            Video => node.IsHTMLElement("video"),
            Slot => node.IsHTMLElement("slot"),
            VTT => self.vtt_elements.contains(&node.Id()),
            MenuItem => node.IsHTMLElement("menuitem"),
            _ => panic!("DOM class collaborator pending: {:?}", class),
        }
    }
    fn SetElementFlag(&self, e: &Self::Element, flag: ElementInvalidationFlag) {
        let mut all = self.element_flags.borrow_mut();
        let flags = all.entry(*e).or_default();
        if !flags.contains(&flag) {
            flags.push(flag);
        }
    }
    fn SetContainerFlag(&self, n: &Self::ContainerNode, flag: ContainerInvalidationFlag) {
        let mut all = self.container_flags.borrow_mut();
        let flags = all.entry(*n).or_default();
        if !flags.contains(&flag) {
            flags.push(flag);
        }
    }
    fn FeatureEnabled(&self, e: &Self::Element, feature: SelectorRuntimeFeature) -> bool {
        match feature {
            SelectorRuntimeFeature::CSSLangExtendedRanges => true,
            _ => panic!(
                "runtime feature not admitted by capability preflight: {:?}",
                feature
            ),
        }
    }
    fn ElementLocalName(&self, e: &Self::Element) -> AtomicString {
        if let Some(value) = self.filters.borrow().local_name_atoms.get(e) {
            return value.clone();
        }
        let value = AtomicString::from_str(self.document.Node(*e).Name());
        self.filters
            .borrow_mut()
            .local_name_atoms
            .insert(*e, value.clone());
        value
    }
    fn ElementNamespaceURI(&self, e: &Self::Element) -> AtomicString {
        Self::namespace(self.document.Node(*e).Namespace())
    }
    fn ElementTagQName(&self, e: &Self::Element) -> QualifiedName {
        QualifiedName::new(
            AtomicString::default(),
            self.ElementLocalName(e),
            self.ElementNamespaceURI(e),
        )
    }
    fn FrameFocusedAndActive(&self, e: &Self::Element) -> Option<bool> {
        Some(self.frame_active)
    }
    fn MenuOwnerItems(&self, e: &Self::Element) -> Vec<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: MenuOwnerItems")
    }
    fn CorrespondingSVGElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .CorrespondingSVGElement(*e)
            .map(|corresponding| self.handle(corresponding))
    }
    fn HasShadowRoot(&self, e: &Self::Element) -> bool {
        self.document.ShadowRoot(*e).is_some()
    }
    fn ShadowRootIsUserAgent(&self, e: &Self::Element) -> bool {
        self.document
            .ShadowRoot(*e)
            .and_then(|root| self.document.ShadowRootMode(root))
            == Some(DOMShadowRootMode::kUserAgent)
    }
    fn ContainingShadowRootIsUserAgent(&self, e: &Self::Element) -> Option<bool> {
        self.document
            .ContainingShadowRoot(*e)
            .and_then(|root| self.document.ShadowRootMode(root))
            .map(|mode| mode == DOMShadowRootMode::kUserAgent)
    }
    fn UltimateOriginatingElement(&self, e: &Self::Element) -> Rc<Self::Element> {
        self.handle(self.document.UltimateOriginatingElement(*e))
    }
    fn ShadowPseudoId(&self, e: &Self::Element) -> AtomicString {
        self.document
            .ShadowPseudoId(*e)
            .map(AtomicString::from_str)
            .unwrap_or_default()
    }
    fn ElementPseudoIdForStyling(&self, e: &Self::Element) -> PseudoId {
        PseudoId::kPseudoIdNone
    }
    fn ElementPseudoArgument(&self, e: &Self::Element) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: ElementPseudoArgument")
    }
    fn ElementViewTransitionClasses(&self, e: &Self::Element) -> Vec<AtomicString> {
        panic!("Chromium DOM/runtime collaborator pending: ElementViewTransitionClasses")
    }
    fn PartNamesContain(&self, p: &Self::PartNames, name: &AtomicString) -> bool {
        p.Contains(name)
    }
    fn TreeScopeRoot(&self, s: &Self::TreeScope) -> Rc<Self::ContainerNode> {
        self.handle(*s)
    }
    fn ContainerAsElement(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        self.element(*n)
    }
    fn ContainerShadowHost(&self, n: &Self::ContainerNode) -> Option<Rc<Self::Element>> {
        self.document.ShadowHost(*n).map(|host| self.handle(host))
    }
    fn ElementAsContainer(&self, e: &Self::Element) -> Rc<Self::ContainerNode> {
        self.handle(*e)
    }
    fn DocumentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        let root = self.document.Root();
        self.QueryFirstChildElement(&root)
    }
    fn FlatTreeParentElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .FlatTreeParentElement(*e)
            .map(|parent| self.handle(parent))
    }
    fn ParentElementOrDocumentFragment(
        &self,
        e: &Self::Element,
    ) -> Option<Rc<Self::ContainerNode>> {
        self.document
            .Node(*e)
            .Parent()
            .filter(|&p| {
                matches!(
                    self.document.Node(p).Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                )
            })
            .map(|p| self.handle(p))
    }
    fn IsFinishedParsingChildren(&self, n: &Self::ContainerNode) -> bool {
        true
    }
    fn FirstChildElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.QueryFirstChildElement(e)
    }
    fn FirstChildNode(&self, e: &Self::Element) -> Option<Rc<Self::Node>> {
        self.document
            .Node(*e)
            .Children()
            .first()
            .map(|&v| self.handle(v))
    }
    fn NextSiblingNode(&self, n: &Self::Node) -> Option<Rc<Self::Node>> {
        self.sibling(*n, true, false)
    }
    fn NodeIsElement(&self, n: &Self::Node) -> bool {
        self.document.Node(*n).Type() == DOMNodeType::kElement
    }
    fn TextNodeData(&self, n: &Self::Node) -> Option<AtomicString> {
        (self.document.Node(*n).Type() == DOMNodeType::kText)
            .then(|| AtomicString::from_str(self.document.Node(*n).Data()))
    }
    fn NumDescendantInputs(&self, e: &Self::Element) -> usize {
        self.document.NumDescendantInputs(*e)
    }
    fn SlottedButton(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .SelectSlottedButton(*e)
            .map(|button| self.handle(button))
    }
    fn CssTarget(&self, _e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document.CssTarget().map(|target| self.handle(target))
    }
    fn ProbeForcePseudoState(&self, e: &Self::Element, pseudo: PseudoType) -> bool {
        self.forced
            .get(&self.document.Node(*e).Id())
            .is_some_and(|v| v.contains(&pseudo))
    }
    fn SameElement(&self, a: &Self::Element, b: &Self::Element) -> bool {
        a == b
    }
    fn NavigationLocationMatches(
        &self,
        location: &dyn crate::css_selector::CSSSelectorNavigationLocation,
        e: &Self::Element,
    ) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: NavigationLocationMatches")
    }
    fn OwnerDataListElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: OwnerDataListElement")
    }
    fn OwnerSelectElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document
            .OptionOwnerSelect(*e)
            .map(|select| self.handle(select))
    }
    fn ActiveOption(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ActiveOption")
    }
    fn InterestInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: InterestInvoker")
    }
    fn ScrollMarkerSelected(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: ScrollMarkerSelected")
    }
    fn HasScrollMarkerGroupData(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasScrollMarkerGroupData")
    }
    fn CompareLayoutPreorder(&self, a: &Self::Element, b: &Self::Element) -> i32 {
        panic!("Chromium DOM/runtime collaborator pending: CompareLayoutPreorder")
    }
    fn NavigationSourceElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: NavigationSourceElement")
    }
    fn ComputeInheritedLanguage(&self, e: &Self::Element) -> AtomicString {
        let mut current = Some(*e);
        while let Some(v) = current {
            if let Some(lang) = self.attr(v, "lang") {
                return AtomicString::from_str(lang);
            }
            current = self.document.Node(v).Parent();
        }
        AtomicString::default()
    }
    fn VttLanguage(&self, e: &Self::Element) -> AtomicString {
        panic!("Chromium DOM/runtime collaborator pending: VttLanguage")
    }
    fn SlotAssignmentDirty(&self, e: &Self::Element) -> bool {
        self.slot_assignment_dirty
            .contains(&self.document.Node(*e).Id())
    }
    fn RecalcSlotAssignments(&self, e: &Self::Element) {
        panic!("Chromium DOM/runtime collaborator pending: RecalcSlotAssignments")
    }
    fn CachedDirectionIsRtl(&self, e: &Self::Element) -> bool {
        self.document.CachedDirectionality(*e) == foundation::TextDirection::kRtl
    }
    fn PopoverInvoker(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: PopoverInvoker")
    }
    fn OwningMenuElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        panic!("Chromium DOM/runtime collaborator pending: OwningMenuElement")
    }
    fn InternalsHasState(&self, e: &Self::Element, state: &AtomicString) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: InternalsHasState")
    }
    fn GetPseudoElement(&self, e: &Self::Element, pseudo: PseudoId) -> Option<Rc<Self::Element>> {
        let shadow_id = match pseudo {
            PseudoId::kPseudoIdPlaceholder => "-webkit-input-placeholder",
            _ => return None,
        };
        self.document
            .UAShadowPseudoElement(*e, shadow_id)
            .map(|element| self.handle(element))
    }
    fn FocusedElement(&self, e: &Self::Element) -> Option<Rc<Self::Element>> {
        self.interaction
            .focused_node_id
            .and_then(|id| self.document.FindNodeById(id))
            .and_then(|v| self.element(v))
    }
    fn FocusVisibleOption(&self, e: &Self::Element) -> Option<bool> {
        Some(self.interaction.focus_visible_node_id == Some(self.document.Node(*e).Id()))
    }
    fn TransitionScope(&self, t: &Self::ViewTransition) -> Rc<Self::Element> {
        panic!("Chromium DOM/runtime collaborator pending: TransitionScope")
    }
    fn TransitionMatchOnlyChild(
        &self,
        t: &Self::ViewTransition,
        id: PseudoId,
        argument: &AtomicString,
    ) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: TransitionMatchOnlyChild")
    }
    fn TransitionMatchActive(&self, t: &Self::ViewTransition) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: TransitionMatchActive")
    }
    fn TransitionMatchActiveType(&self, t: &Self::ViewTransition, idents: &[AtomicString]) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: TransitionMatchActiveType")
    }
    fn ScrollbarEnabled(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarEnabled")
    }
    fn ScrollbarHoveredPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarHoveredPart")
    }
    fn ScrollbarPressedPart(&self, s: &Self::CustomScrollbar) -> Self::ScrollbarPart {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarPressedPart")
    }
    fn ScrollbarPartBits(&self, p: &Self::ScrollbarPart) -> u32 {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarPartBits")
    }
    fn ScrollbarHorizontal(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarHorizontal")
    }
    fn ScrollbarNativeThemeHasButtons(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarNativeThemeHasButtons")
    }
    fn ScrollbarCornerVisible(&self, s: &Self::CustomScrollbar) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollbarCornerVisible")
    }
    fn InPseudoHasChecking(&self, e: &Self::Element) -> bool {
        self.has_depth.get() > 0
    }
    fn ExitHasCacheScope(&self, scope: Self::HasCacheScope) {
        if scope {
            self.has_depth.set(self.has_depth.get() - 1);
        } else {
            self.query_depth.set(self.query_depth.get() - 1);
        }
    }
    fn EnterHasCacheScope(
        &self,
        e: &Self::Element,
        within_selector_checking: bool,
    ) -> Self::HasCacheScope {
        if within_selector_checking {
            self.has_depth.set(self.has_depth.get() + 1);
        } else {
            self.query_depth.set(self.query_depth.get() + 1);
        }
        within_selector_checking
    }
    fn CreateHasArgumentContext(
        &self,
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::ContainerNode>,
        shadow: bool,
    ) -> Self::HasArgumentContext {
        let mut result = HasArgument {
            relation: RelationType::kSubSelector,
            depth: 0,
            adjacent: 0,
            sibling_right: false,
            sibling_between: false,
            shadow,
            hashes: HashSet::new(),
        };
        let mut child_seen = false;
        let mut sibling_left = false;
        for simple in selector.SimpleSelectors() {
            match simple.Match() {
                MatchType::kId => {
                    result.hashes.insert(simple.Value().Hash().wrapping_mul(17));
                }
                MatchType::kClass => {
                    result.hashes.insert(simple.Value().Hash().wrapping_mul(19));
                }
                MatchType::kTag => {
                    result
                        .hashes
                        .insert(simple.TagQName().LocalName().Hash().wrapping_mul(13));
                }
                _ => {}
            }
            use RelationType::*;
            let relation = simple.Relation();
            match relation {
                kRelativeChild | kChild => {
                    if relation == kRelativeChild {
                        result.relation = relation;
                    }
                    if sibling_left {
                        result.sibling_between = true;
                        sibling_left = false;
                    }
                    child_seen = true;
                    if result.depth != i32::MAX {
                        result.depth += 1;
                    }
                    result.adjacent = 0;
                }
                kRelativeDescendant | kDescendant => {
                    if relation == kRelativeDescendant {
                        result.relation = relation;
                    }
                    if sibling_left {
                        result.sibling_between = true;
                        sibling_left = false;
                    }
                    child_seen = true;
                    result.depth = i32::MAX;
                    result.adjacent = 0;
                }
                kRelativeDirectAdjacent | kDirectAdjacent => {
                    if relation == kRelativeDirectAdjacent {
                        result.relation = relation;
                    }
                    if child_seen {
                        sibling_left = true;
                    } else {
                        result.sibling_right = true;
                    }
                    if result.adjacent != i32::MAX {
                        result.adjacent += 1;
                    }
                }
                kRelativeIndirectAdjacent | kIndirectAdjacent => {
                    if relation == kRelativeIndirectAdjacent {
                        result.relation = relation;
                    }
                    if child_seen {
                        sibling_left = true;
                    } else {
                        result.sibling_right = true;
                    }
                    result.adjacent = i32::MAX;
                }
                _ => {}
            }
        }
        result
    }
    fn HasInvalidShadowTraversal(&self, c: &Self::HasArgumentContext) -> bool {
        c.shadow
            && !matches!(
                c.relation,
                RelationType::kRelativeChild | RelationType::kRelativeDescendant
            )
    }
    fn HasLeftmostRelation(&self, c: &Self::HasArgumentContext) -> RelationType {
        c.relation
    }
    fn HasDepthLimit(&self, c: &Self::HasArgumentContext) -> i32 {
        c.depth
    }
    fn HasDepthFixed(&self, c: &Self::HasArgumentContext) -> bool {
        c.depth != i32::MAX
    }
    fn HasAdjacentDistanceLimit(&self, c: &Self::HasArgumentContext) -> i32 {
        c.adjacent
    }
    fn HasAdjacentDistanceFixed(&self, c: &Self::HasArgumentContext) -> bool {
        c.adjacent != i32::MAX
    }
    fn HasSiblingCombinatorAtRightmost(&self, c: &Self::HasArgumentContext) -> bool {
        c.sibling_right
    }
    fn HasSiblingCombinatorBetweenChildOrDescendant(&self, c: &Self::HasArgumentContext) -> bool {
        c.sibling_between
    }
    fn HasSiblingsAffectedFlags(&self, c: &Self::HasArgumentContext) -> u8 {
        if c.adjacent == 0 {
            0
        } else if c.depth == 0 {
            1
        } else {
            2
        }
    }
    fn SetSiblingsAffectedByHasFlags(&self, e: &Self::Element, flags: u8) {
        *self.sibling_has_flags.borrow_mut().entry(*e).or_default() |= flags;
    }
    fn CreateHasTraversal(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasTraversal {
        let mut nodes = vec![];
        if c.adjacent == 0 {
            for &child in self.document.Node(*e).Children() {
                if self.element(child).is_some() {
                    self.has_walk(child, 1, c.depth, &mut nodes);
                }
            }
        } else {
            let mut distance = 1;
            let mut current = self.NextSibling(e);
            while let Some(v) = current {
                if c.adjacent == i32::MAX || distance == c.adjacent {
                    self.has_walk(*v, 0, c.depth, &mut nodes);
                } else {
                    nodes.push((v.clone(), 0));
                }
                if distance == c.adjacent {
                    break;
                }
                current = self.NextSibling(&v);
                distance += 1;
            }
        }
        nodes.reverse();
        HasTraversal { nodes, position: 0 }
    }
    fn HasTraversalCurrent(&self, it: &Self::HasTraversal) -> Option<Rc<Self::Element>> {
        it.nodes.get(it.position).map(|v| v.0.clone())
    }
    fn HasTraversalDepth(&self, it: &Self::HasTraversal) -> i32 {
        it.nodes[it.position].1
    }
    fn HasTraversalNext(&self, it: &mut Self::HasTraversal) {
        it.position += 1;
    }
    fn CreateHasCacheContext(
        &self,
        e: &Self::Element,
        c: &Self::HasArgumentContext,
    ) -> Self::HasCacheContext {
        HasCache {
            results: RefCell::new(HashMap::new()),
            hashes: c.hashes.clone(),
        }
    }
    fn HasCacheAllowed(&self, c: &Self::HasCacheContext) -> bool {
        true
    }
    fn HasCacheGetResult(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8 {
        c.results.borrow().get(e).copied().unwrap_or(0)
    }
    fn HasCacheAlreadyChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) -> bool {
        self.HasCacheGetResult(c, e) & 1 != 0
    }
    fn HasCacheSetChecked(&self, c: &Self::HasCacheContext, e: &Self::Element) {
        let mut results = c.results.borrow_mut();
        *results.entry(*e).or_default() |= 1;
    }
    fn HasCacheSetMatched(&self, c: &Self::HasCacheContext, e: &Self::Element) -> u8 {
        let mut results = c.results.borrow_mut();
        let old = results.get(e).copied().unwrap_or(0);
        *results.entry(*e).or_default() |= 3;
        old
    }
    fn HasCacheSetAllTraversedChecked(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
        depth: i32,
    ) {
        self.HasCacheSetChecked(c, e);
    }
    fn HasCacheEnsureFastRejectFilter(
        &self,
        c: &Self::HasCacheContext,
        e: &Self::Element,
    ) -> (Rc<Self::HasFastRejectFilter>, bool) {
        (
            Rc::new(HasFilter {
                allocated: Cell::new(false),
                hashes: RefCell::new(HashSet::new()),
            }),
            true,
        )
    }
    fn HasBloomAllocated(&self, f: &Self::HasFastRejectFilter) -> bool {
        f.allocated.get()
    }
    fn HasAllocateBloom(&self, f: &Self::HasFastRejectFilter) {
        f.allocated.set(true);
    }
    fn HasAddElementIdentifierHashes(&self, f: &Self::HasFastRejectFilter, e: &Self::Element) {
        f.hashes.borrow_mut().extend(self.identifier_hashes(*e));
    }
    fn HasFastReject(&self, f: &Self::HasFastRejectFilter, c: &Self::HasArgumentContext) -> bool {
        !c.hashes.is_subset(&f.hashes.borrow())
    }
    fn StyleScopeParent(&self, s: &Self::StyleScope) -> Option<Rc<Self::StyleScope>> {
        s.parent.clone()
    }
    fn StyleScopeFrom(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>> {
        s.from.clone()
    }
    fn StyleScopeTo(&self, s: &Self::StyleScope) -> Option<Rc<CSSSelectorList>> {
        s.to.clone()
    }
    fn ElementTriggersScope(&self, e: &Self::Element, s: &Self::StyleScope) -> bool {
        s.implicit_roots.contains(e)
    }
    fn StyleScopeFrameElement(&self, f: &Self::StyleScopeFrame) -> Rc<Self::Element> {
        f.element.clone()
    }
    fn ParentStyleScopeFrame(
        &self,
        f: &Self::StyleScopeFrame,
        e: &Self::Element,
    ) -> Option<Rc<Self::StyleScopeFrame>> {
        f.parent.as_ref().filter(|p| *p.element == *e).cloned()
    }
    fn InsertScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
    ) -> Option<Rc<Self::StyleScopeActivations>> {
        f.data.borrow().get(&(s as *const _ as usize)).cloned()
    }
    fn SetScopeActivationCacheEntry(
        &self,
        f: &Self::StyleScopeFrame,
        s: &Self::StyleScope,
        a: Rc<Self::StyleScopeActivations>,
    ) {
        f.data.borrow_mut().insert(s as *const _ as usize, a);
    }
    fn CreateActivation(
        &self,
        root: Option<Rc<Self::ContainerNode>>,
        proximity: u32,
    ) -> Rc<Self::StyleScopeActivation> {
        Rc::new(PersistentStyleScopeActivation { root, proximity })
    }
    fn CreateActivations(
        &self,
        vector: Vec<Rc<Self::StyleScopeActivation>>,
        flags: MatchFlags,
    ) -> Rc<Self::StyleScopeActivations> {
        Rc::new(PersistentStyleScopeActivations { vector, flags })
    }
    fn SpatialNavigationFocused(&self, e: &Self::Element) -> bool {
        // cpp: html_option_element.cc:670-674. The association belongs to DOM;
        // focus is the immutable input supplied by the interaction owner.
        let Some(select) = self.document.OptionOwnerSelect(*e) else {
            return false;
        };
        if !self.IsFocused(&select) {
            return false;
        }
        // cpp: SelectType::SpatialNavigationFocusedOption (select_type.cc:
        // 2145-2147) returns null for menu lists. ListBoxSelectType's disabled
        // branch (1673-1675) also returns null. Only admitted policies reach
        // this matcher; an enabled request fails before constructing one.
        if self.document.SelectUsesMenuList(select) {
            return false;
        }
        match self.spatial_navigation_policy {
            SpatialNavigationPolicy::Disabled => false,
        }
    }
    fn HasDatalist(&self, e: &Self::Element) -> bool {
        self.document.InputHasDataList(*e)
    }
    fn IsSubmenuOpen(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsSubmenuOpen")
    }
    fn UsesMenuList(&self, e: &Self::Element) -> bool {
        self.document.SelectUsesMenuList(*e)
    }
    fn IsMultiSelectFocused(&self, e: &Self::Element) -> bool {
        let Some(select) = self.document.OptionOwnerSelect(*e) else {
            return false;
        };
        self.document
            .Node(select)
            .FindAttribute("multiple")
            .is_some()
            && !self.document.SelectUsesMenuList(select)
            && self.interaction.multi_select_focused_option_id == Some(self.document.Node(*e).Id())
    }
    fn IsResourceTarget(&self, e: &Self::Element) -> bool {
        self.ElementIs(e, ElementClass::SVG)
            && self.attr(*e, "id").is_some_and(|v| !v.is_empty())
            && self
                .forced
                .get(&self.document.Node(*e).Id())
                .is_some_and(|v| v.contains(&PseudoType::kPseudoTarget))
    }
    fn CachedImageIsAnimated(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: CachedImageIsAnimated")
    }
    fn IsPlaceholderVisible(&self, e: &Self::Element) -> bool {
        self.document.TextControlPlaceholderIsVisible(*e)
    }
    fn MatchesToolFormActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesToolFormActive")
    }
    fn IsUnboundedElementActive(&self, e: &Self::Element) -> bool {
        // cpp: HTMLElement::IsUnboundedElementActive reads a dedicated element
        // flag, never the mouse :active state. The unbounded-surface API is not
        // exposed by this host, so no representable element can set that flag.
        false
    }
    fn MatchesToolSubmitActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesToolSubmitActive")
    }
    fn IsDragged(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsDragged")
    }
    fn HasFocusWithin(&self, e: &Self::Element) -> bool {
        let Some(id) = self.interaction.focused_node_id else {
            return false;
        };
        let mut current = self.document.FindNodeById(id);
        while let Some(v) = current {
            if v == *e {
                return true;
            }
            current = self.document.Node(v).Parent();
        }
        false
    }
    fn IsFiltered(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsFiltered")
    }
    fn HasInterest(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasInterest")
    }
    fn HasFlattenedAssignedNodes(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasFlattenedAssignedNodes")
    }
    fn IsHovered(&self, e: &Self::Element) -> bool {
        self.in_user_action_chain(*e, self.interaction.hovered_node_id)
    }
    fn IsActive(&self, e: &Self::Element) -> bool {
        self.in_user_action_chain(*e, self.interaction.pressed_node_id)
    }
    fn MatchesEnabled(&self, e: &Self::Element) -> bool {
        self.document.MatchesEnabledPseudoClass(*e)
    }
    fn IsMediaDocument(&self, e: &Self::Element) -> bool {
        // cpp: selector_checker.cc kPseudoFullPageMedia asks the owning
        // Document class, not the element. PersistentDocument currently
        // represents HTML/XML/SVG document trees; the media-document parser
        // and its generated full-page media wrapper are not exposed by the
        // DOM input boundary. Consequently every document representable by
        // this backend answers false. Keep the query here so introducing a
        // media-document kind later remains a DOM boundary change rather than
        // a selector special case.
        false
    }
    fn MatchesDefault(&self, e: &Self::Element) -> bool {
        self.document.MatchesDefaultPseudoClass(*e)
    }
    fn MatchesDisabled(&self, e: &Self::Element) -> bool {
        self.document.MatchesDisabledPseudoClass(*e)
    }
    fn MatchesReadOnly(&self, e: &Self::Element) -> bool {
        !self.MatchesReadWrite(e)
    }
    fn MatchesReadWrite(&self, e: &Self::Element) -> bool {
        if self
            .document
            .InputType(*e)
            .is_some_and(|input_type| input_type.SupportsReadOnly())
            || self.document.Node(*e).IsHTMLElement("textarea")
        {
            !self.MatchesDisabled(e) && self.attr(*e, "readonly").is_none()
        } else {
            let mut current = Some(*e);
            while let Some(v) = current {
                if let Some(value) = self.attr(v, "contenteditable") {
                    return value.is_empty()
                        || value.eq_ignore_ascii_case("true")
                        || value.eq_ignore_ascii_case("plaintext-only");
                }
                current = self.document.Node(v).Parent();
            }
            false
        }
    }
    fn IsOptionalFormControl(&self, e: &Self::Element) -> bool {
        self.document.IsOptionalFormControl(*e)
    }
    fn IsRequiredFormControl(&self, e: &Self::Element) -> bool {
        self.document.IsRequiredFormControl(*e)
    }
    fn MatchesUserInvalid(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesUserInvalid")
    }
    fn MatchesUserValid(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesUserValid")
    }
    fn MatchesValidity(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesValidity")
    }
    fn IsValid(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsValid")
    }
    fn ShouldAppearChecked(&self, e: &Self::Element) -> bool {
        self.document.ControlChecked(*e)
    }
    fn ShouldAppearIndeterminate(&self, e: &Self::Element) -> bool {
        self.document.ShouldAppearIndeterminate(*e)
    }
    fn OptionSelected(&self, e: &Self::Element) -> bool {
        self.document.OptionSelected(*e)
    }
    fn ScrollMarkerIsSelected(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: ScrollMarkerIsSelected")
    }
    fn IsTextField(&self, e: &Self::Element) -> bool {
        self.document
            .InputType(*e)
            .is_some_and(|input_type| input_type.IsTextFieldInputType())
    }
    fn IsDialogInTopLayer(&self, e: &Self::Element) -> bool {
        self.document.IsDialogInTopLayer(*e)
    }
    fn IsPopoverInTopLayer(&self, e: &Self::Element) -> bool {
        self.document.IsPopoverInTopLayer(*e)
    }
    fn IsPopover(&self, e: &Self::Element) -> bool {
        self.document.IsPopover(*e)
    }
    fn PopoverOpen(&self, e: &Self::Element) -> bool {
        self.document.PopoverOpen(*e)
    }
    fn MatchesOverscrollOpen(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MatchesOverscrollOpen")
    }
    fn HasOpenAttribute(&self, e: &Self::Element) -> bool {
        self.attr(*e, "open").is_some()
    }
    fn PopupIsVisible(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: PopupIsVisible")
    }
    fn IsPickerVisible(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPickerVisible")
    }
    fn FullscreenFlag(&self, e: &Self::Element) -> bool {
        self.document.FullscreenFlag(*e)
    }
    fn ContainsFullScreenElement(&self, e: &Self::Element) -> bool {
        self.document.ContainsFullscreenElement(*e)
    }
    fn Granted(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: Granted")
    }
    fn IsPictureInPicture(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsPictureInPicture")
    }
    fn MediaPaused(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaPaused")
    }
    fn MediaSeeking(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaSeeking")
    }
    fn MediaBuffering(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaBuffering")
    }
    fn MediaStalled(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaStalled")
    }
    fn MediaMuted(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: MediaMuted")
    }
    fn VideoPersistent(&self, _e: &Self::Element) -> bool {
        false
    }
    fn ContainsPersistentVideo(&self, _e: &Self::Element) -> bool {
        false
    }
    fn IsXrOverlay(&self, _e: &Self::Element) -> bool {
        false
    }
    fn IsFullscreenElement(&self, e: &Self::Element) -> bool {
        self.document.FullscreenElement() == Some(*e)
    }
    fn IsInRange(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsInRange")
    }
    fn IsOutOfRange(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: IsOutOfRange")
    }
    fn VttIsPastNode(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: VttIsPastNode")
    }
    fn IsDefined(&self, e: &Self::Element) -> bool {
        self.IsHTMLElement(e) && !self.document.Node(*e).Name().contains("-")
    }
    fn GetComputedStyle(&self, e: &Self::Element) -> Option<Persistent<ComputedStyle>> {
        panic!("Chromium DOM/runtime collaborator pending: GetComputedStyle")
    }
    fn HasEffectiveAppearance(&self, style: &ComputedStyle) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: HasEffectiveAppearance")
    }
    fn PageIsActive(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: PageIsActive")
    }
    fn DidAttachInternals(&self, e: &Self::Element) -> bool {
        panic!("Chromium DOM/runtime collaborator pending: DidAttachInternals")
    }
    fn DialogIsModal(&self, e: &Self::Element) -> bool {
        self.document.DialogIsModal(*e)
    }
    fn IsAutofilled(&self, e: &Self::Element) -> bool {
        self.document.AutofillState(*e) == DOMAutofillState::kAutofilled
    }
    fn IsPreviewed(&self, e: &Self::Element) -> bool {
        self.document.AutofillState(*e) == DOMAutofillState::kPreviewed
    }
    fn AutofillStateIsPreviewed(&self, e: &Self::Element) -> bool {
        self.document.AutofillState(*e) == DOMAutofillState::kPreviewed
    }
    fn IsFocused(&self, e: &Self::Element) -> bool {
        self.interaction.focused_node_id == Some(self.document.Node(*e).Id())
    }
    fn AccessibilityAlwaysShowFocus(&self, e: &Self::Element) -> bool {
        self.interaction.focus_visible_node_id == Some(self.document.Node(*e).Id())
    }
    fn MayTriggerVirtualKeyboard(&self, e: &Self::Element) -> bool {
        self.IsTextField(e) || self.document.Node(*e).IsHTMLElement("textarea")
    }
    fn LastFocusTypeIsMouse(&self, e: &Self::Element) -> bool {
        self.interaction.pressed_node_id.is_some()
    }
    fn HadKeyboardEvent(&self, e: &Self::Element) -> bool {
        self.interaction.focus_visible_node_id.is_some()
    }
    fn AffectedByMultipleHas(&self, e: &Self::Element) -> bool {
        self.element_flags
            .borrow()
            .get(e)
            .is_some_and(|v| v.contains(&ElementInvalidationFlag::AffectedByMultipleHas))
    }
}

pub struct PersistentSelectorService {
    container_flags: Rc<RefCell<HashMap<usize, Vec<ContainerInvalidationFlag>>>>,
    element_flags: Rc<RefCell<HashMap<usize, Vec<ElementInvalidationFlag>>>>,
    subject_flags: Rc<RefCell<HashMap<usize, crate::resolver::match_flags::MatchFlags>>>,
    sibling_has_flags: Rc<RefCell<HashMap<usize, u8>>>,
    filter_owner: Option<(DOMOwnerHandle, u64, CompatibilityMode)>,
    filters: Rc<RefCell<PersistentElementFilters>>,
    cache: SelectorQueryCache,
    owner: Option<(DOMOwnerHandle, CompatibilityMode)>,
    nodes: Rc<RefCell<HashMap<usize, Rc<usize>>>>,
    pub interaction: UserInteractionState,
    pub spatial_navigation_policy: SpatialNavigationPolicy,
    pub forced_pseudo_states: HashMap<u64, Vec<PseudoType>>,
    pub frame_focused_and_active: bool,
    pub slot_assignment_dirty: HashSet<u64>,
    pub vtt_elements: HashSet<u64>,
}
pub struct PersistentSelectorMatchSession<'d> {
    backend: Rc<PersistentSelectorBackend<'d>>,
}
#[derive(Debug)]
pub enum PersistentSelectorError {
    Syntax(SelectorQuerySyntaxError),
    Unsupported(std::string::String),
}
impl From<SelectorQuerySyntaxError> for PersistentSelectorError {
    fn from(value: SelectorQuerySyntaxError) -> Self {
        Self::Syntax(value)
    }
}
impl Default for PersistentSelectorService {
    fn default() -> Self {
        Self {
            container_flags: Default::default(),
            element_flags: Default::default(),
            subject_flags: Default::default(),
            sibling_has_flags: Default::default(),
            filter_owner: None,
            filters: Default::default(),
            cache: SelectorQueryCache::default(),
            owner: None,
            nodes: Default::default(),
            interaction: Default::default(),
            spatial_navigation_policy: Default::default(),
            forced_pseudo_states: HashMap::new(),
            frame_focused_and_active: true,
            slot_assignment_dirty: HashSet::new(),
            vtt_elements: HashSet::new(),
        }
    }
}
impl PersistentSelectorService {
    fn backend<'d>(&mut self, document: &'d Document) -> Rc<PersistentSelectorBackend<'d>> {
        let owner = (document.RootHandle(), document.GetCompatibilityMode());
        if self.owner.map(|previous| previous.0) != Some(owner.0) {
            self.nodes = Default::default();
            self.container_flags = Default::default();
            self.element_flags = Default::default();
            self.subject_flags = Default::default();
            self.sibling_has_flags = Default::default();
        }
        if self.owner != Some(owner) {
            self.owner = Some(owner);
            self.cache.Invalidate();
        }
        // Native Node identity is needed only for visited elements. Sessions
        // share these stable handles without preallocating or copying an
        // arena-sized vector when unrelated detached nodes are created.
        // Local attributes and affected ancestor paths carry their own
        // revisions. Replace the complete owner only for document/mode changes,
        // unknown NodeMut edits or exhausted revisions, never ordinary input.
        let filter_owner = document
            .SelectorFilterKey()
            .map(|(owner, generation)| (owner, generation, document.GetCompatibilityMode()));
        if filter_owner.is_none() || self.filter_owner != filter_owner {
            self.filters = Default::default();
            self.filter_owner = filter_owner;
        }
        // Parent-stack membership is distinct from local element facts.
        // Reset its aggregate on mutations while retaining unaffected cached
        // attributes and subtree blooms. Sessions borrow immutable DOM state.
        let membership = document.CollectionMembershipKey();
        {
            let mut filters = self.filters.borrow_mut();
            if membership.is_none() || filters.ancestor_revision != membership {
                *filters.ancestor.borrow_mut() = Default::default();
                filters.ancestor_revision = membership;
            }
        }
        PersistentSelectorBackend::new(
            document,
            self.filters.clone(),
            self.element_flags.clone(),
            self.subject_flags.clone(),
            self.sibling_has_flags.clone(),
            self.container_flags.clone(),
            self.nodes.clone(),
            self.interaction,
            self.spatial_navigation_policy,
            self.forced_pseudo_states.clone(),
            self.frame_focused_and_active,
            self.slot_assignment_dirty.clone(),
            self.vtt_elements.clone(),
        )
    }

    pub(crate) fn ElementHasInvalidationFlag(
        &self,
        element: usize,
        flag: ElementInvalidationFlag,
    ) -> bool {
        self.element_flags
            .borrow()
            .get(&element)
            .is_some_and(|flags| flags.contains(&flag))
    }
    pub(crate) fn ContainerHasInvalidationFlag(
        &self,
        container: usize,
        flag: ContainerInvalidationFlag,
    ) -> bool {
        self.container_flags
            .borrow()
            .get(&container)
            .is_some_and(|flags| flags.contains(&flag))
    }
    pub(crate) fn SiblingsAffectedByHasFlags(&self, element: usize) -> u8 {
        self.sibling_has_flags
            .borrow()
            .get(&element)
            .copied()
            .unwrap_or_default()
    }
    pub(crate) fn SubjectMatchFlags(
        &self,
        element: usize,
    ) -> crate::resolver::match_flags::MatchFlags {
        self.subject_flags
            .borrow()
            .get(&element)
            .copied()
            .unwrap_or_default()
    }
    pub(crate) fn ResetSubjectMatchFlags(&self, element: usize) {
        self.subject_flags.borrow_mut().remove(&element);
    }
    pub fn StyleMatchSession<'d>(
        &mut self,
        document: &'d Document,
    ) -> PersistentSelectorMatchSession<'d> {
        PersistentSelectorMatchSession {
            backend: self.backend(document),
        }
    }

    fn prepare<'d>(
        &mut self,
        document: &'d Document,
        selector: &str,
    ) -> Result<(Rc<SelectorQuery>, Rc<PersistentSelectorBackend<'d>>), PersistentSelectorError>
    {
        let backend = self.backend(document);
        let context = SelectorParserContext {
            html: backend.IsInHTMLDocument(&document.Root()),
            quirks: backend.InQuirksMode(&document.Root()),
        };
        let query = self.cache.AddForSelectorParser(
            &AtomicString::from_str(selector),
            &context,
            &*backend,
            |selectors| match CSSSelectorParser::UnsupportedParsedFeature(selectors) {
                Some(feature) => Err(PersistentSelectorError::Unsupported(format!(
                    "selector backend pending: :{}",
                    feature
                ))),
                None => Ok(()),
            },
        )?;
        Ok((query, backend))
    }
    // cpp: element_rule_collector.cc:551-572; selector_checker.cc:1035-1072,
    // 3897-4081. All activation traversal and limit matching stay in the
    // translated checker; this method supplies the live DOM owners.
    pub fn MatchingScopedSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        list: &CSSSelectorList,
        scopes: &[cssom::CSSStyleScope],
        owner_node_id: u64,
    ) -> Result<Option<ScopedSelectorMatch>, PersistentSelectorError> {
        use crate::parser::css_nesting_type::CSSNestingType;
        let (_, b) = self.prepare(document, "*")?;
        if let Some(feature) = CSSSelectorParser::UnsupportedParsedFeature(&list.CopySelectors()) {
            return Err(PersistentSelectorError::Unsupported(format!(
                "scoped selector backend pending: :{}",
                feature
            )));
        }
        let context = SelectorParserContext {
            html: true,
            quirks: b.InQuirksMode(&index),
        };
        // cpp: scoped_style_resolver.cc:446-478. Ownerless sheets have no
        // implicit activation; documentElement is not an invented fallback.
        let implicit_root = document
            .FindNodeById(owner_node_id)
            .and_then(|i| document.Node(i).Parent())
            .and_then(|i| b.element(i))
            .map(|i| *i);
        let mut parent = None;
        for record in scopes {
            let parse = |text: &str,
                         nesting|
             -> Result<Rc<CSSSelectorList>, PersistentSelectorError> {
                let text = foundation::String::from(text);
                let list =
                    CSSSelectorList::AdoptSelectorVector(CSSSelectorParser::ParseScopeBoundary(
                        &text,
                        &context,
                        nesting,
                        &SelectorParserOptions::default(),
                    ));
                if !list.IsValid() {
                    return Err(PersistentSelectorError::Unsupported(
                        "invalid compiled scope boundary".into(),
                    ));
                }
                if let Some(feature) =
                    CSSSelectorParser::UnsupportedParsedFeature(&list.CopySelectors())
                {
                    return Err(PersistentSelectorError::Unsupported(format!(
                        "scope boundary backend pending: :{}",
                        feature
                    )));
                }
                Ok(list)
            };
            let from = match &record.root {
                cssom::CSSScopeRoot::ImplicitStylesheetOwner => None,
                cssom::CSSScopeRoot::ExplicitSelectorList(text) => Some(parse(
                    text,
                    if parent.is_some() {
                        CSSNestingType::kScope
                    } else {
                        CSSNestingType::kNone
                    },
                )?),
            };
            let to = record
                .limit
                .as_deref()
                .map(|text| parse(text, CSSNestingType::kScope))
                .transpose()?;
            parent = Some(Rc::new(PersistentStyleScope {
                from,
                to,
                parent,
                implicit_roots: implicit_root.into_iter().collect(),
            }));
        }
        let mut ancestors = Vec::new();
        let mut ancestor = Some(b.handle(index));
        while let Some(e) = ancestor {
            ancestor = b.ParentOrShadowHostElement(&e);
            ancestors.push(e);
        }
        let mut frame = None;
        for element in ancestors.into_iter().rev() {
            frame = Some(Rc::new(PersistentStyleScopeFrame {
                element,
                parent: frame,
                data: RefCell::new(HashMap::new()),
            }));
        }
        let checker = SelectorChecker::new(b.clone(), Mode::kResolvingStyle);
        let mut c = SelectorCheckingContext::new(b.handle(index));
        c.tree_scope = Some(b.ElementTreeScope(&index));
        c.style_scope = parent;
        c.style_scope_frame = frame;
        let mut best: Option<ScopedSelectorMatch> = None;
        for selector in list.ComplexSelectors() {
            c.selector = Some(selector);
            let mut result = MatchResult::default();
            if checker.Match(&c, &mut result) {
                let matched = ScopedSelectorMatch {
                    specificity: selector.Specificity(),
                    proximity: result.proximity,
                };
                if best.is_none_or(|old| {
                    (matched.specificity, std::cmp::Reverse(matched.proximity))
                        > (old.specificity, std::cmp::Reverse(old.proximity))
                }) {
                    best = Some(matched);
                }
            }
        }
        Ok(best)
    }
    pub fn Specificity(
        &mut self,
        document: &Document,
        selector: &str,
    ) -> Result<u32, PersistentSelectorError> {
        let (q, _) = self.prepare(document, selector)?;
        Ok(q.Specificity())
    }
    pub fn MatchingSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Option<u32>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.MatchingSpecificity(b.clone(), b.handle(index)))
    }

    // cpp: element_rule_collector.cc:551-572; selector_checker.cc:1035-1072.
    // Style rules are compiled when the stylesheet is installed. Match that
    // retained selector list directly instead of feeding its serialized text
    // through the querySelector cache for every element/rule pair.
    pub fn MatchingCompiledSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        list: &CSSSelectorList,
    ) -> Option<u32> {
        self.StyleMatchSession(document)
            .MatchingCompiledSpecificity(index, list)
    }
}

impl<'d> PersistentSelectorMatchSession<'d> {
    pub(crate) fn RuleStyleScope(
        scope: &crate::style_scope::StyleScope,
        implicit_root: Option<usize>,
    ) -> Rc<PersistentStyleScope> {
        let implicit_roots = implicit_root.into_iter().collect::<Vec<_>>();
        Self::RuleStyleScopeWithImplicitRoots(scope, &implicit_roots)
    }

    /// Builds the SelectorChecker view of one retained StyleScope chain with
    /// the exact StyleScopeData trigger set installed by active sheet owners.
    /// Each chain belongs to one StyleSheetContents, so every implicit scope
    /// in the chain observes the same owner-parent set, as in
    /// ScopedStyleResolver::ForEachImplicitScopeTrigger.
    pub(crate) fn RuleStyleScopeWithImplicitRoots(
        scope: &crate::style_scope::StyleScope,
        implicit_roots: &[usize],
    ) -> Rc<PersistentStyleScope> {
        Rc::new(PersistentStyleScope {
            from: scope.From().map(|selectors| Rc::new(selectors.clone())),
            to: scope.To().map(|selectors| Rc::new(selectors.clone())),
            parent: scope
                .Parent()
                .map(|parent| Self::RuleStyleScopeWithImplicitRoots(parent, implicit_roots)),
            implicit_roots: implicit_roots.to_vec(),
        })
    }

    pub(crate) fn Backend(&self) -> Rc<PersistentSelectorBackend<'d>> {
        self.backend.clone()
    }

    fn StyleScopeFrame(&self, index: usize) -> Rc<PersistentStyleScopeFrame> {
        let backend = &self.backend;
        let mut ancestors = Vec::new();
        let mut ancestor = Some(backend.handle(index));
        while let Some(element) = ancestor {
            ancestor = backend.ParentOrShadowHostElement(&element);
            ancestors.push(element);
        }
        let mut frame = None;
        for element in ancestors.into_iter().rev() {
            frame = Some(Rc::new(PersistentStyleScopeFrame {
                element,
                parent: frame,
                data: RefCell::new(HashMap::new()),
            }));
        }
        frame.expect("an element always contributes a style scope frame")
    }

    fn SubjectMayMatch(
        &self,
        index: usize,
        selector: crate::css_selector::CSSSelectorComplex<'_>,
    ) -> bool {
        use crate::css_selector::MatchType;
        for simple in selector.SimpleSelectors() {
            let matches = match simple.Match() {
                MatchType::kId => {
                    self.backend.HasID(&index)
                        && self.backend.IdForStyleResolution(&index) == *simple.Value()
                }
                MatchType::kClass => self.backend.ClassNamesContain(&index, &simple.Value()),
                MatchType::kTag => {
                    // cpp: selector_checker.cc:1561-1573.  Parsed HTML type
                    // selectors normally carry the wildcard namespace; a
                    // strict QualifiedName comparison would reject every one
                    // of them before SelectorChecker sees it (including the
                    // UA `head`, `style`, `script`, and `title` rules).
                    let name = simple.TagQName();
                    let local = self.backend.ElementLocalName(&index);
                    let local_matches = name.LocalName() == &local
                        || (!self.backend.IsHTMLElement(&index)
                            && self.backend.IsInHTMLDocument(&index)
                            && local.EqualsUtf8IgnoringAsciiCase(&name.LocalName().Utf8()));
                    local_matches
                        && (name.NamespaceURI() == &AtomicString::from_str("*")
                            || name.NamespaceURI() == &self.backend.ElementNamespaceURI(&index))
                }
                _ => true,
            };
            if !matches {
                return false;
            }
            if simple.Relation() != RelationType::kSubSelector {
                break;
            }
        }
        true
    }

    fn AncestorsMayMatch(
        &self,
        index: usize,
        selector: crate::css_selector::CSSSelectorComplex<'_>,
    ) -> bool {
        // cpp: ElementRuleCollector::CollectMatchingRules. Compatibility
        // declarations still use the translated SelectorChecker, so they need
        // the same ancestor rejection before it records dependency flags.
        // Without it, testing an impossible .class:hover ancestor can mark
        // html/body as hover dependencies before the class check fails.
        if self.backend.document.GetCompatibilityMode() == CompatibilityMode::kQuirks {
            return true;
        }
        let filter = self.backend.SelectorFilterForElement(index);
        if !filter.ParentStackIsConsistent(self.backend.document.FlatTreeParentElement(index)) {
            return true;
        }
        let mut hashes = Vec::new();
        let mut subject = 0;
        crate::selector_filter::CollectIdentifierHashes(
            selector,
            None::<&crate::style_scope::StyleScope>,
            &mut hashes,
            &mut subject,
        );
        !filter.FastRejectSelector(&hashes)
    }

    /// Matches one retained complex selector from a translated `RuleData`.
    ///
    /// ElementRuleCollector operates on the selector selected by
    /// `RuleData::SelectorIndex`, rather than the complete selector list owned
    /// by the style rule.  Keeping this entry point on the match session lets
    /// the production RuleSet bridge use the same translated SelectorChecker
    /// without serializing or reparsing the selector.
    pub fn MatchesCompiledComplex(
        &self,
        index: usize,
        selector: crate::css_selector::CSSSelectorComplex<'_>,
    ) -> bool {
        if !self.SubjectMayMatch(index, selector) {
            return false;
        }
        let backend = &self.backend;
        let checker = SelectorChecker::new(backend.clone(), Mode::kResolvingStyle);
        let mut context = SelectorCheckingContext::new(backend.handle(index));
        context.selector = Some(selector);
        context.tree_scope = Some(backend.ElementTreeScope(&index));
        checker.Match(&context, &mut MatchResult::default())
    }

    /// Matches retained `RuleData` selector storage with the contexts that
    /// ElementRuleCollector normally installs before invoking SelectorChecker.
    /// The scope boundaries are the parser-owned typed lists; no selector text
    /// is serialized or parsed again.
    pub(crate) fn MatchesCompiledComplexInRuleContext(
        &self,
        index: usize,
        selector: crate::css_selector::CSSSelectorComplex<'_>,
        scope: Option<&crate::style_scope::StyleScope>,
        implicit_scope_root: Option<usize>,
        pseudo_id: layoutng_style::style::computed_style_constants::PseudoId,
        originating_pseudo_id: layoutng_style::style::computed_style_constants::PseudoId,
        pseudo_argument: Option<&AtomicString>,
        pseudo_ident_list: &[AtomicString],
    ) -> Option<ScopedSelectorMatch> {
        if !self.SubjectMayMatch(index, selector) {
            return None;
        }
        let backend = &self.backend;
        let mut checker = SelectorChecker::ForPseudoRequest(
            backend.clone(),
            Mode::kResolvingStyle,
            pseudo_argument.cloned().unwrap_or_default(),
            pseudo_ident_list.to_vec(),
        );
        checker.SetOriginatingPseudoId(originating_pseudo_id);
        let mut context = SelectorCheckingContext::new(backend.handle(index));
        context.selector = Some(selector);
        context.tree_scope = Some(backend.ElementTreeScope(&index));
        context.pseudo_id = pseudo_id;
        context.pseudo_argument = pseudo_argument.cloned();
        if let Some(scope) = scope {
            context.style_scope = Some(Self::RuleStyleScope(scope, implicit_scope_root));
            context.style_scope_frame = Some(self.StyleScopeFrame(index));
        }
        let mut result = MatchResult::default();
        // The source feature is disabled at the audited Chromium commit. A
        // pseudo StyleRequest therefore uses SelectorChecker's dynamic-pseudo
        // result, while selector queries retain their existing logical mode.
        let did_match = backend.WithLogicalPseudoMode(pseudo_id == PseudoId::kPseudoIdNone, || {
            checker.Match(&context, &mut result)
        });
        let matched = did_match.then_some(ScopedSelectorMatch {
            specificity: selector.Specificity(),
            proximity: result.proximity,
        });
        matched
    }

    pub fn MatchingCompiledSpecificity(&self, index: usize, list: &CSSSelectorList) -> Option<u32> {
        let backend = &self.backend;
        let checker = SelectorChecker::new(backend.clone(), Mode::kResolvingStyle);
        let mut best = None;
        for selector in list.ComplexSelectors() {
            if !self.SubjectMayMatch(index, selector) || !self.AncestorsMayMatch(index, selector) {
                continue;
            }
            let mut context = SelectorCheckingContext::new(backend.handle(index));
            context.selector = Some(selector);
            context.tree_scope = Some(backend.ElementTreeScope(&index));
            let mut result = MatchResult::default();
            if checker.Match(&context, &mut result) {
                best = Some(best.map_or(selector.Specificity(), |specificity: u32| {
                    specificity.max(selector.Specificity())
                }));
            }
        }
        best
    }
}

impl PersistentSelectorService {
    pub fn Matches(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<bool, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.Matches(b.clone(), b.handle(index)))
    }
    pub fn Closest(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Option<usize>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.Closest(b.clone(), b.handle(index)).map(|v| *v))
    }
    pub fn QueryFirst(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Option<usize>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.QueryFirst(b.clone(), b.handle(index)).map(|v| *v))
    }
    pub fn QueryAll(
        &mut self,
        document: &Document,
        index: usize,
        selector: &str,
    ) -> Result<Vec<usize>, PersistentSelectorError> {
        let (q, b) = self.prepare(document, selector)?;
        Ok(q.QueryAll(b.clone(), b.handle(index))
            .into_iter()
            .map(|v| *v)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn element(d: &mut Document, parent: usize, tag: &str, id: &str, class: &str) -> usize {
        let i = d.CreateElementDefault(DOMNamespace::kHTML, tag.into());
        for (name, value) in [("id", id), ("class", class)] {
            if !value.is_empty() {
                d.SetAttribute(
                    i,
                    DOMAttribute {
                        local_name: name.into(),
                        value: value.into(),
                        ..Default::default()
                    },
                );
            }
        }
        d.AppendChild(parent, i);
        i
    }
    #[test]
    fn mapped_queries_read_real_dom_order_scope_and_mutations() {
        let mut owner = dom::DOM::new();
        let d = owner.GetDocumentMut();
        let html = element(d, d.Root(), "html", "", "");
        let root = element(d, html, "main", "root", "");
        let a = element(d, root, "p", "duplicate", "one two");
        d.AppendText(root, "between");
        let b = element(d, root, "p", "duplicate", "two");
        let section = element(d, root, "section", "section", "");
        let leaf = element(d, section, "span", "leaf", "leaf");
        let mut service = PersistentSelectorService::default();
        let class_owner = service.backend(d).ClassNamesForElement(a);
        assert!(Rc::ptr_eq(
            &class_owner,
            &service.backend(d).ClassNamesForElement(a)
        ));
        {
            let backend = service.backend(d);
            let attributes = backend.AttributesWithoutUpdate(&a);
            assert!(
                std::ptr::eq(attributes, d.Node(a).Attributes()),
                "selector checks borrow the canonical DOM attribute collection"
            );
        }
        assert_eq!(
            service.QueryFirst(d, d.Root(), "#duplicate").unwrap(),
            Some(a)
        );
        assert_eq!(service.QueryAll(d, root, "#duplicate").unwrap(), vec![a, b]);
        assert_eq!(service.QueryFirst(d, root, "#root").unwrap(), None);
        assert_eq!(service.QueryAll(d, root, ":scope > p").unwrap(), vec![a, b]);
        assert!(service
            .Matches(d, a, "main > p.one.two:first-child")
            .unwrap());
        assert!(service.Matches(d, b, "p + p:nth-child(2)").unwrap());
        assert_eq!(service.Closest(d, leaf, "main").unwrap(), Some(root));
        assert!(service
            .Matches(d, a, ":is(#duplicate,.absent):not(.absent):where(p)")
            .unwrap());
        assert!(service.Matches(d, root, ":has(> section)").unwrap());
        assert!(!service.Matches(d, root, ":has(> .leaf)").unwrap());
        assert!(service.Matches(d, root, ":has(> section .leaf)").unwrap());
        assert!(service.Matches(d, a, "*").unwrap());
        assert!(!service.Matches(d, a, "p::before").unwrap());
        // Inline writes and ordinary value changes preserve filter inputs,
        // while actual selector matching must read the new attribute values.
        let filter_revision = d.SelectorFilterRevisions(root);
        for value in ["width: 40px", "width: 41px"] {
            d.SetAttribute(
                a,
                DOMAttribute {
                    local_name: "style".into(),
                    value: value.into(),
                    ..Default::default()
                },
            );
            assert_eq!(d.SelectorFilterRevisions(root), filter_revision);
            assert!(service
                .Matches(d, a, &format!("[style='{value}']"))
                .unwrap());
            assert!(Rc::ptr_eq(
                &class_owner,
                &service.backend(d).ClassNamesForElement(a)
            ));
        }
        d.RemoveAttribute(a, "style", "");
        assert_eq!(d.SelectorFilterRevisions(root), filter_revision);
        assert!(!service.Matches(d, a, "[style]").unwrap());
        d.SetAttribute(
            a,
            DOMAttribute {
                local_name: "data-state".into(),
                value: "old".into(),
                ..Default::default()
            },
        );
        assert_ne!(d.SelectorFilterRevisions(root), filter_revision);
        assert!(service.Matches(d, root, ":has([data-state=old])").unwrap());
        let filter_revision = d.SelectorFilterRevisions(root);
        d.SetAttribute(
            a,
            DOMAttribute {
                local_name: "data-state".into(),
                value: "new".into(),
                ..Default::default()
            },
        );
        assert_eq!(d.SelectorFilterRevisions(root), filter_revision);
        assert!(!service.Matches(d, root, ":has([data-state=old])").unwrap());
        assert!(service.Matches(d, root, ":has([data-state=new])").unwrap());
        d.RemoveAttribute(a, "data-state", "");
        assert_ne!(d.SelectorFilterRevisions(root), filter_revision);
        assert!(!service.Matches(d, root, ":has([data-state])").unwrap());
        let identity = service.backend(d).handle(a);
        let visited_handles = service.nodes.borrow().len();
        for _ in 0..64 {
            d.CreateElementDefault(DOMNamespace::kHTML, "unvisited".into());
        }
        drop(service.backend(d));
        assert_eq!(
            service.nodes.borrow().len(),
            visited_handles,
            "detached allocation does not materialize selector handles"
        );
        d.SetAttribute(
            a,
            DOMAttribute {
                local_name: "class".into(),
                value: "changed".into(),
                ..Default::default()
            },
        );
        d.Remove(a);
        d.AppendChild(root, a);
        assert_eq!(service.QueryAll(d, root, "p").unwrap(), vec![b, a]);
        assert!(!service.Matches(d, a, ".one").unwrap());
        assert!(service.Matches(d, a, ".changed:last-child").unwrap());
        let changed_classes = service.backend(d).ClassNamesForElement(a);
        assert!(!Rc::ptr_eq(&class_owner, &changed_classes));
        assert_eq!(changed_classes[0].Utf8(), "changed");
        {
            let backend = service.backend(d);
            let attributes = backend.AttributesWithoutUpdate(&a);
            assert!(std::ptr::eq(attributes, d.Node(a).Attributes()));
            assert!(attributes
                .iter()
                .any(|attribute| attribute.local_name == "class" && attribute.value == "changed"));
        }
        assert!(service.Matches(d, a, "[class=changed]").unwrap());
        assert!(Rc::ptr_eq(&identity, &service.backend(d).handle(a)));
        assert!(service.QueryFirst(d, root, "p >").is_err());
        assert!(service.QueryAll(d, root, "").is_err());
    }

    #[test]
    fn compiled_html_type_selector_preserves_wildcard_namespace_matching() {
        let mut owner = dom::DOM::new();
        let d = owner.GetDocumentMut();
        let html = element(d, d.Root(), "html", "", "");
        let head = element(d, html, "head", "", "");
        let body = element(d, html, "body", "", "");
        let selector = CSSSelectorList::AdoptSelectorVector(CSSSelectorParser::ParseSelector(
            &foundation::String::from("head"),
            &SelectorParserContext {
                html: true,
                quirks: false,
            },
        ));
        let mut service = PersistentSelectorService::default();
        assert!(service
            .MatchingCompiledSpecificity(d, head, &selector)
            .is_some());
        assert!(service
            .MatchingCompiledSpecificity(d, body, &selector)
            .is_none());
    }

    #[test]
    fn part_names_apply_retained_export_maps_in_order() {
        let mut owner = dom::DOM::new();
        let d = owner.GetDocumentMut();
        let inner_host = d.CreateElement(
            DOMNamespace::kHTML,
            "x-inner".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "label: middle, label: badge".into(),
                ..Default::default()
            }],
        );
        let outer_host = d.CreateElement(
            DOMNamespace::kHTML,
            "x-outer".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "middle: public, badge: public-badge".into(),
                ..Default::default()
            }],
        );
        let mut names = PersistentPartNames::FromNames([AtomicString::from_str("label")]);
        names.PushMap(d.PartNamesMap(inner_host).unwrap());
        names.PushMap(d.PartNamesMap(outer_host).unwrap());

        assert!(names.Contains(&AtomicString::from_str("public")));
        assert!(names.Contains(&AtomicString::from_str("public-badge")));
        assert!(!names.Contains(&AtomicString::from_str("label")));
        assert_eq!(names.NamesForTest(), vec!["public", "public-badge"]);
    }
}
