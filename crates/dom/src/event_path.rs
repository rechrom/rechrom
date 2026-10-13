//! Immutable DOM event-dispatch path and observer-relative composed paths.
//!
//! cpp: Blink core/dom/events/event_path.{cc,h} and
//! tree_scope_event_context.{cc,h}, revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. See
//! third_party/NOTICE.chromium-event-path for source attribution.

use crate::persistent_document::DOMShadowRootMode;
use crate::Document;
use std::collections::HashMap;

/// The node invoked at one dispatch step and the target exposed to its scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMEventPathEntry {
    pub invocation_target: u64,
    pub shadow_adjusted_target: u64,
}

/// A dispatch-time snapshot. No arena references or live assignment/tree state
/// are retained, so callbacks may reparent nodes or change slot assignments.
/// Window belongs to the dispatching environment and is not a DOM path entry.
#[derive(Clone, Debug)]
pub struct DOMEventPath {
    entries: Vec<DOMEventPathEntry>,
    entry_tree_scopes: Vec<usize>,
    tree_scopes: Vec<TreeScopeEventContext>,
    document_target: Option<u64>,
    post_dispatch_target: Option<u64>,
}

#[derive(Clone, Debug)]
struct TreeScopeEventContext {
    target: Option<u64>,
    parent: Option<usize>,
    children: Vec<usize>,
    closed: bool,
    containing_closed_tree: Option<usize>,
    pre_order: usize,
    post_order: usize,
}

impl DOMEventPath {
    // cpp: EventPath::CalculatePath, CalculateAdjustedTargets and
    // CalculateTreeOrderAndSetNearestAncestorClosedTree.
    pub fn new(document: &Document, target: usize, composed: bool) -> Self {
        let original_shadow_root = document.ContainingShadowRoot(target);
        let mut nodes = Vec::new();
        let mut current = Some(target);
        while let Some(node) = current {
            nodes.push(node);
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
            entry_tree_scopes: Vec::with_capacity(nodes.len()),
            tree_scopes: Vec::new(),
            document_target: None,
            post_dispatch_target: None,
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
        self.tree_scopes.push(TreeScopeEventContext {
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

    fn IsInclusiveAncestorOf(&self, ancestor: usize, descendant: usize) -> bool {
        let ancestor = &self.tree_scopes[ancestor];
        let descendant = &self.tree_scopes[descendant];
        ancestor.pre_order <= descendant.pre_order && descendant.post_order <= ancestor.post_order
    }

    // cpp: TreeScopeEventContext::IsUnclosedTreeOf, including its exclusive
    // branch rule for slot paths traversing sibling shadow trees.
    fn IsUnclosedTreeOf(&self, candidate: usize, observer: usize) -> bool {
        if self.IsInclusiveAncestorOf(candidate, observer) {
            return true;
        }
        let Some(closed) = self.tree_scopes[candidate].containing_closed_tree else {
            return true;
        };
        if self.IsInclusiveAncestorOf(observer, candidate) {
            return closed == observer || !self.IsInclusiveAncestorOf(observer, closed);
        }
        false
    }
}
