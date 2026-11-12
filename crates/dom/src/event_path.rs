//! Immutable DOM event-dispatch path and observer-relative composed paths.
//!
//! cpp: Blink core/dom/events/event_path.{cc,h} and
//! tree_scope_event_context.{cc,h}, revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. See
//! third_party/NOTICE.chromium-event-path for source attribution.

use crate::persistent_document::{DOMNamespace, DOMNodeType, DOMOwnerHandle, DOMShadowRootMode};
use crate::Document;
use std::collections::HashMap;

/// Only the native class facts needed by Node::KeepEventInNode. A custom type
/// name on a KeyboardEvent or TouchEvent does not turn it into a base Event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMEventPathClass {
    Other,
    Keyboard,
    Touch,
}
#[derive(Clone, Copy)]
pub struct DOMEventPathEvent<'a> {
    pub name: &'a str,
    pub class: DOMEventPathClass,
}
impl DOMEventPathEvent<'_> {
    // MediaControlElementsHelper::IsUserInteractionEvent, cc:23-32.
    pub(crate) fn IsUserInteraction(self) -> bool {
        matches!(
            self.name,
            "pointerdown"
                | "pointerup"
                | "mousedown"
                | "mouseup"
                | "click"
                | "dblclick"
                | "gesturetap"
        ) || matches!(
            self.class,
            DOMEventPathClass::Keyboard | DOMEventPathClass::Touch
        )
    }
}

/// The node invoked at one dispatch step and the target exposed to its scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMEventPathEntry {
    pub invocation_target: u64,
    pub shadow_adjusted_target: u64,
}

/// Node-local dispatch facts frozen alongside the canonical path. Generated
/// pseudos retain DOM identities but source CalculatePath exposes their
/// ultimate originating element in the observer-visible invocation path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMEventPathNodeFacts {
    pub in_shadow_tree: bool,
    pub is_document: bool,
    pub is_disabled_form_control: bool,
    pub is_svg_element: bool,
    pub is_pseudo_element: bool,
    pub has_parent: bool,
    /// Full document identity; node IDs alone can collide across adopted arenas.
    pub document_owner: DOMOwnerHandle,
}

/// A dispatch-time snapshot. No arena references or live assignment/tree state
/// are retained, so callbacks may reparent nodes or change slot assignments.
/// Window belongs to the dispatching environment and is not a DOM path entry.
#[derive(Clone, Debug)]
pub struct DOMEventPath {
    entries: Vec<DOMEventPathEntry>,
    entry_facts: Vec<DOMEventPathNodeFacts>,
    entry_tree_scopes: Vec<usize>,
    tree_scopes: Vec<DOMEventPathTreeScope>,
    document_target: Option<u64>,
    post_dispatch_target: Option<u64>,
    pseudo_element_target: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DOMEventPathTreeScope {
    pub root_node_id: u64,
    pub target: Option<u64>,
    pub parent: Option<usize>,
    children: Vec<usize>,
    pub closed: bool,
    pub containing_closed_tree: Option<usize>,
    pub pre_order: usize,
    pub post_order: usize,
    pub is_shadow_root: bool,
}

impl DOMEventPath {
    // cpp: EventPath::CalculatePath, CalculateAdjustedTargets and
    // CalculateTreeOrderAndSetNearestAncestorClosedTree.
    pub fn new(document: &Document, target: usize, composed: bool) -> Self {
        Self::ForEvent(document, target, composed, None)
    }
    pub fn ForEvent(
        document: &Document,
        target: usize,
        composed: bool,
        event: Option<DOMEventPathEvent<'_>>,
    ) -> Self {
        let pseudo_element_target = document
            .PseudoElementFacts(target)
            .map(|_| document.Node(target).Id());
        let target = document
            .PseudoElementFacts(target)
            .map_or(target, |p| p.originating_element);
        let original_shadow_root = document.ContainingShadowRoot(target);
        let mut nodes = Vec::new();
        let mut current = Some(target);
        while let Some(node) = current {
            nodes.push(node);
            // EventPath::CalculatePath cc:126-127 runs this source virtual
            // before assigned-slot, shadow-host or parent traversal.
            if event.is_some_and(|event| document.KeepMediaControlEventInNode(node, event)) {
                break;
            }
            // Assignment has one owner. In particular this also consumes the
            // retained manual assignment, without rebuilding a parallel map.
            current = if let Some(slot) = document.AssignedSlot(node) {
                Some(slot)
            } else if let Some(host) = document.ShadowHost(node) {
                // A non-composed event stops only at its ORIGINAL target's
                // shadow root. Slotted light-DOM targets cross other roots.
                (composed || Some(node) != original_shadow_root).then_some(host)
            } else {
                document.Node(node).Parent()
            };
        }

        let mut path = Self {
            entries: Vec::with_capacity(nodes.len()),
            entry_facts: Vec::with_capacity(nodes.len()),
            entry_tree_scopes: Vec::with_capacity(nodes.len()),
            tree_scopes: Vec::new(),
            document_target: None,
            post_dispatch_target: None,
            pseudo_element_target,
        };
        let mut scope_indices = HashMap::new();
        for &node in &nodes {
            let scope = path.EnsureTreeScopeEventContext(
                document,
                document.TreeScopeRoot(node),
                Some(document.Node(node).Id()),
                &mut scope_indices,
            );
            path.entry_tree_scopes.push(scope);
            path.entries.push(DOMEventPathEntry {
                invocation_target: document.Node(node).Id(),
                shadow_adjusted_target: path.tree_scopes[scope]
                    .target
                    .expect("an invocation supplies its tree scope's target"),
            });
            let dom_node = document.Node(node);
            let is_document = dom_node.Type() == DOMNodeType::kDocument;
            path.entry_facts.push(DOMEventPathNodeFacts {
                in_shadow_tree: path.tree_scopes[scope].is_shadow_root,
                is_document,
                is_disabled_form_control: dom_node.Namespace() == DOMNamespace::kHTML
                    && matches!(
                        dom_node.Name(),
                        "button" | "fieldset" | "input" | "select" | "textarea"
                    )
                    && document.MatchesDisabledPseudoClass(node),
                is_svg_element: dom_node.Type() == DOMNodeType::kElement
                    && dom_node.Namespace() == DOMNamespace::kSVG,
                is_pseudo_element: false,
                has_parent: dom_node.Parent().is_some(),
                document_owner: document.NodeDocumentHandle(node),
            });
        }
        let mut order = 0;
        for scope in 0..path.tree_scopes.len() {
            if path.tree_scopes[scope].parent.is_none() {
                order = path.CalculateTreeOrderAndSetNearestAncestorClosedTree(scope, order, None);
            }
        }

        let top = *nodes.last().expect("an event path includes its target");
        // Only the active document can supply the caller's Window. Documents
        // made by createHTMLDocument and detached fragments have no Window.
        path.document_target = (top == document.Root()).then(|| document.Node(top).Id());
        // cpp: WindowEventContext::WindowEventContext, with the stable
        // ClearTargetOnlyIfInShadowTree behavior. Detached light-DOM targets
        // are retained; an event confined to a shadow tree clears its target.
        if document.ContainingShadowRoot(top).is_none() {
            path.post_dispatch_target = path
                .entries
                .last()
                .map(|entry| entry.shadow_adjusted_target);
        }
        path
    }

    pub fn Entries(&self) -> &[DOMEventPathEntry] {
        &self.entries
    }

    pub fn EntryFacts(&self, index: usize) -> &DOMEventPathNodeFacts {
        &self.entry_facts[index]
    }

    pub fn EntryTreeScope(&self, index: usize) -> usize {
        self.entry_tree_scopes[index]
    }

    /// Scope identity, retargeting and tree order share the same snapshot used
    /// by ComposedPath; adapters must not rebuild them from the live DOM.
    pub fn TreeScopes(&self) -> &[DOMEventPathTreeScope] {
        &self.tree_scopes
    }

    pub fn PseudoElementTarget(&self) -> Option<u64> {
        self.pseudo_element_target
    }
    pub fn Target(&self) -> u64 {
        self.entries[0].invocation_target
    }

    pub fn ReachesDocument(&self) -> bool {
        self.document_target.is_some()
    }

    /// Identity of the active document reached by this path, if any. A Window
    /// listener uses this observer to request the document-visible path.
    pub fn DocumentTarget(&self) -> Option<u64> {
        self.document_target
    }

    pub fn PostDispatchTarget(&self) -> Option<u64> {
        self.post_dispatch_target
    }

    // cpp: TreeScopeEventContext::EnsureEventPath. Filtering uses the saved
    // listener scope, never its possibly mutated position in the live tree.
    pub fn ComposedPath(&self, current_target: u64) -> Vec<u64> {
        let Some(observer) = self
            .entries
            .iter()
            .position(|entry| entry.invocation_target == current_target)
        else {
            return Vec::new();
        };
        let observer_scope = self.entry_tree_scopes[observer];
        self.entries
            .iter()
            .zip(&self.entry_tree_scopes)
            .filter(|(_, scope)| self.IsUnclosedTreeOf(**scope, observer_scope))
            .map(|(entry, _)| entry.invocation_target)
            .collect()
    }

    // cpp: EventPath::EnsureTreeScopeEventContext. The first invocation in a
    // scope supplies its adjusted target unless an ancestor scope already has
    // one. That inheritance keeps light-DOM targets unchanged inside slots.
    fn EnsureTreeScopeEventContext(
        &mut self,
        document: &Document,
        root: usize,
        current_target: Option<u64>,
        scope_indices: &mut HashMap<usize, usize>,
    ) -> usize {
        if let Some(&scope) = scope_indices.get(&root) {
            if self.tree_scopes[scope].target.is_none() {
                self.tree_scopes[scope].target = current_target;
            }
            return scope;
        }
        let scope = self.tree_scopes.len();
        scope_indices.insert(root, scope);
        self.tree_scopes.push(DOMEventPathTreeScope {
            root_node_id: document.Node(root).Id(),
            target: None,
            parent: None,
            children: Vec::new(),
            closed: matches!(
                document.ShadowRootMode(root),
                Some(DOMShadowRootMode::kClosed | DOMShadowRootMode::kUserAgent)
            ),
            containing_closed_tree: None,
            pre_order: 0,
            post_order: 0,
            is_shadow_root: document.ShadowHost(root).is_some(),
        });
        let parent = document.ShadowHost(root).map(|host| {
            self.EnsureTreeScopeEventContext(
                document,
                document.TreeScopeRoot(host),
                None,
                scope_indices,
            )
        });
        self.tree_scopes[scope].parent = parent;
        self.tree_scopes[scope].target = parent
            .and_then(|parent| self.tree_scopes[parent].target)
            .or(current_target);
        if let Some(parent) = parent {
            self.tree_scopes[parent].children.push(scope);
        }
        scope
    }

    // cpp: TreeScopeEventContext::CalculateTreeOrderAndSetNearestAncestorClosedTree.
    fn CalculateTreeOrderAndSetNearestAncestorClosedTree(
        &mut self,
        scope: usize,
        mut order: usize,
        nearest_closed: Option<usize>,
    ) -> usize {
        self.tree_scopes[scope].pre_order = order;
        let nearest_closed = if self.tree_scopes[scope].closed {
            Some(scope)
        } else {
            nearest_closed
        };
        self.tree_scopes[scope].containing_closed_tree = nearest_closed;
        for child in 0..self.tree_scopes[scope].children.len() {
            let child = self.tree_scopes[scope].children[child];
            order = self.CalculateTreeOrderAndSetNearestAncestorClosedTree(
                child,
                order + 1,
                nearest_closed,
            );
        }
        self.tree_scopes[scope].post_order = order + 1;
        order + 1
    }

    pub fn IsInclusiveAncestorTreeScopeOf(&self, ancestor: usize, descendant: usize) -> bool {
        let ancestor = &self.tree_scopes[ancestor];
        let descendant = &self.tree_scopes[descendant];
        ancestor.pre_order <= descendant.pre_order && descendant.post_order <= ancestor.post_order
    }

    // cpp: TreeScopeEventContext::IsUnclosedTreeOf, including its exclusive
    // branch rule for slot paths traversing sibling shadow trees.
    fn IsUnclosedTreeOf(&self, candidate: usize, observer: usize) -> bool {
        if self.IsInclusiveAncestorTreeScopeOf(candidate, observer) {
            return true;
        }
        let Some(closed) = self.tree_scopes[candidate].containing_closed_tree else {
            return true;
        };
        if self.IsInclusiveAncestorTreeScopeOf(observer, candidate) {
            return closed == observer || !self.IsInclusiveAncestorTreeScopeOf(observer, closed);
        }
        false
    }
}
