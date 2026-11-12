//! DisplayLockUtilities.cc330-491 and DisplayLockDocumentState.cc300-371.
//! The original live Range/Node is registered while style/layout can create
//! new DisplayLockContexts; those contexts join this same forced scope.
use super::*;
use crate::{persistent_document::DOMOwnerHandle, range::DOMRangeHandle};
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub enum DisplayLockForcedEffect {
    RemoveAllPendingUpdates,
    ForcedRenderWarning {
        element: u64,
        is_warning: bool,
        last_message: bool,
    },
    NotifyExistingSpeculationRules {
        roots: Vec<u64>,
    },
}
#[derive(Clone)]
pub struct DisplayLockForcedScopeHandle(Rc<RefCell<ForcedScope>>);
impl std::fmt::Debug for DisplayLockForcedScopeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DisplayLockForcedScope")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}
enum Target {
    Node {
        node: Option<usize>,
        include_self: bool,
    },
    Range(DOMRangeHandle),
}
struct ForcedScope {
    document: DOMOwnerHandle,
    target: Target,
    phase: DisplayLockForcedPhase,
    only_cv_auto: bool,
    emit_warnings: bool,
    in_script_context: bool,
    contexts: HashSet<usize>,
    active: bool,
}
impl DisplayLockForcedScopeHandle {
    fn AddContext(&self, document: &mut PersistentDocument, element: usize, range_initial: bool) {
        let (phase, only_cv_auto, emit_warnings, in_script_context, already_forced) = {
            let scope = self.0.borrow();
            (
                scope.phase,
                scope.only_cv_auto,
                scope.emit_warnings,
                scope.in_script_context,
                scope.contexts.contains(&element),
            )
        };
        if already_forced && !range_initial {
            return;
        }
        if only_cv_auto
            && !document.DisplayLockIsActivatable(element, DisplayLockActivationReason::Viewport)
        {
            return;
        }
        self.0.borrow_mut().contexts.insert(element);
        document.UpgradeDisplayLockForcedScope(element, DisplayLockForcedPhase::None, phase);
        document.IssueDisplayLockForcedRenderWarningIfNeeded(
            element,
            emit_warnings,
            in_script_context,
        );
    }
    /// SetRequestedState can run inside Style's live immutable DOM borrow.
    /// Its source AutoReset suppresses style dirtying in this phase; membership
    /// and original counters must still be updated before traversing children.
    fn AddContextDuringStyle(&self, document: &PersistentDocument, element: usize) {
        let (phase, only_cv_auto, emit_warnings, in_script_context) = {
            let scope = self.0.borrow();
            if scope.contexts.contains(&element) {
                return;
            }
            (
                scope.phase,
                scope.only_cv_auto,
                scope.emit_warnings,
                scope.in_script_context,
            )
        };
        if only_cv_auto
            && !document.DisplayLockIsActivatable(element, DisplayLockActivationReason::Viewport)
        {
            return;
        }
        self.0.borrow_mut().contexts.insert(element);
        document.ChangeDisplayLock(element, |context, _| {
            debug_assert!(
                context.set_requested_state_scope
                    || document.Lifecycle().GetState()
                        == crate::document_lifecycle::LifecycleState::InStyleRecalc
            );
            context.forced.start(phase);
        });
        document.IssueDisplayLockForcedRenderWarningIfNeeded(
            element,
            emit_warnings,
            in_script_context,
        );
    }
    fn Includes(&self, document: &PersistentDocument, element: usize) -> bool {
        let scope = self.0.borrow();
        match &scope.target {
            Target::Node { node, include_self } => {
                let mut next = if *include_self {
                    *node
                } else {
                    node.and_then(|node| document.FlatTreeParent(node))
                };
                while let Some(node) = next {
                    if node == element {
                        return true;
                    }
                    next = document.FlatTreeParent(node);
                }
                false
            }
            Target::Range(range) => document.InclusiveAncestorsOfRange(range).contains(&element),
        }
    }
}
impl PersistentDocument {
    /// Source range's DOM traversal plus flat ancestors for slotted children.
    pub fn InclusiveAncestorsOfRange(&self, range: &DOMRangeHandle) -> Vec<usize> {
        let Some(range) = self.Range(range) else {
            return Vec::new();
        };
        let first = self.RangeFirstNodeForPoint(range.start);
        let past_last = self.RangePastLastNodeForPoint(range.end);
        let mut elements = Vec::new();
        let mut next = first;
        while next != past_last {
            let Some(node) = next else {
                break;
            };
            next = self.RangeNextNode(node);
            if self
                .Node(node)
                .Parent()
                .is_some_and(|parent| self.ShadowRoot(parent).is_some())
            {
                let mut ancestor = Some(node);
                while let Some(node) = ancestor {
                    if self.Node(node).Type() == DOMNodeType::kElement {
                        elements.push(node);
                    }
                    ancestor = self.FlatTreeParent(node);
                }
            } else if self.Node(node).Type() == DOMNodeType::kElement {
                elements.push(node);
            }
        }
        let mut ancestor = first;
        while let Some(node) = ancestor {
            if self.Node(node).Type() == DOMNodeType::kElement {
                elements.push(node);
            }
            ancestor = self.FlatTreeParent(node);
        }
        elements
    }
    pub fn BeginRangeDisplayLockForcedScope(
        &mut self,
        range: DOMRangeHandle,
        phase: DisplayLockForcedPhase,
        in_script_context: bool,
    ) -> DisplayLockForcedScopeHandle {
        let first = self
            .Range(&range)
            .and_then(|range| self.RangeFirstNodeForPoint(range.start));
        let scope = DisplayLockForcedScopeHandle(Rc::new(RefCell::new(ForcedScope {
            document: self.RootHandle(),
            target: Target::Range(range.clone()),
            phase,
            only_cv_auto: false,
            emit_warnings: true,
            in_script_context,
            contexts: HashSet::new(),
            active: first.is_some(),
        })));
        if first.is_none() {
            return scope;
        }
        self.display_lock_document_state
            .borrow_mut()
            .forced_scopes
            .push(scope.clone());
        if self.DisplayLockLockedCount() == 0 {
            return scope;
        }
        // Range's initial loop notifies for each source vector entry; later
        // AddForcedUpdateScopeForContext guards membership exactly as source.
        for element in self.InclusiveAncestorsOfRange(&range) {
            if self.arena[element].display_lock_context.borrow().is_some() {
                scope.AddContext(self, element, true);
            }
        }
        scope
    }
    pub fn BeginNodeDisplayLockForcedScope(
        &mut self,
        node: Option<usize>,
        phase: DisplayLockForcedPhase,
        include_self: bool,
        only_cv_auto: bool,
        emit_warnings: bool,
        in_script_context: bool,
    ) -> DisplayLockForcedScopeHandle {
        let active = node.is_some();
        let original = node;
        let scope = DisplayLockForcedScopeHandle(Rc::new(RefCell::new(ForcedScope {
            document: self.RootHandle(),
            target: Target::Node {
                node: original,
                include_self,
            },
            phase,
            only_cv_auto,
            emit_warnings,
            in_script_context,
            contexts: HashSet::new(),
            active,
        })));
        if !active {
            return scope;
        }
        let mut node = original.expect("active source node scope");
        self.display_lock_document_state
            .borrow_mut()
            .forced_scopes
            .push(scope.clone());
        if self.DisplayLockLockedCount() == 0 {
            return scope;
        }
        if let Some(host) = self.ShadowHost(node) {
            node = host;
        }
        let inclusive = self.Node(node).Type() == DOMNodeType::kElement
            && include_self
            && self.arena[node].display_lock_context.borrow().is_some();
        let mut ancestor = if inclusive {
            Some(node)
        } else {
            self.FlatTreeParent(node)
        };
        while let Some(element) = ancestor {
            ancestor = self.FlatTreeParent(element);
            if self.Node(element).Type() == DOMNodeType::kElement
                && self.arena[element].display_lock_context.borrow().is_some()
            {
                scope.AddContext(self, element, false);
            }
        }
        scope
    }
    pub fn EnsureMinimumDisplayLockForcedPhase(&mut self, phase: DisplayLockForcedPhase) {
        let scopes = self
            .display_lock_document_state
            .borrow()
            .forced_scopes
            .clone();
        for scope in scopes {
            let (old, contexts, emit_warnings, in_script_context) = {
                let scope = scope.0.borrow();
                (
                    scope.phase,
                    scope.contexts.iter().copied().collect::<Vec<_>>(),
                    scope.emit_warnings,
                    scope.in_script_context,
                )
            };
            if old >= phase {
                continue;
            }
            for element in contexts {
                self.UpgradeDisplayLockForcedScope(element, old, phase);
                self.IssueDisplayLockForcedRenderWarningIfNeeded(
                    element,
                    emit_warnings,
                    in_script_context,
                );
            }
            scope.0.borrow_mut().phase = phase;
        }
    }
    pub fn EndDisplayLockForcedUpdate(&mut self, scope: DisplayLockForcedScopeHandle) {
        let (phase, contexts) = {
            let mut data = scope.0.borrow_mut();
            assert_eq!(
                data.document,
                self.RootHandle(),
                "forced scope belongs to the original Document"
            );
            if !data.active {
                return;
            }
            data.active = false;
            (
                data.phase,
                data.contexts.iter().copied().collect::<Vec<_>>(),
            )
        };
        let mut state = self.display_lock_document_state.borrow_mut();
        let index = state
            .forced_scopes
            .iter()
            .position(|registered| Rc::ptr_eq(&registered.0, &scope.0))
            .expect("source forced scope registered once");
        state.forced_scopes.remove(index);
        drop(state);
        let mut locked_roots = Vec::new();
        for element in contexts {
            self.EndDisplayLockForcedScope(element, phase);
            if self.DisplayLockIsLocked(element) {
                locked_roots.push(self.Node(element).Id());
            }
        }
        self.display_lock_document_state
            .borrow_mut()
            .forced_effects
            .push_back(DisplayLockForcedEffect::NotifyExistingSpeculationRules {
                roots: locked_roots,
            });
    }
    pub(super) fn ForceActiveDisplayLockScopesForElementDuringStyle(&self, element: usize) {
        let scopes = self
            .display_lock_document_state
            .borrow()
            .forced_scopes
            .clone();
        for scope in scopes {
            if scope.Includes(self, element) {
                scope.AddContextDuringStyle(self, element);
            }
        }
    }
    pub(super) fn ForceActiveDisplayLockScopesForElement(&mut self, element: usize) {
        let scopes = self
            .display_lock_document_state
            .borrow()
            .forced_scopes
            .clone();
        for scope in scopes {
            if scope.Includes(self, element) {
                scope.AddContext(self, element, false);
            }
        }
    }
    fn IssueDisplayLockForcedRenderWarningIfNeeded(
        &self,
        element: usize,
        emit_warnings: bool,
        in_script_context: bool,
    ) {
        let should_warn = emit_warnings && in_script_context && self.arena[element].display_lock_context.borrow().as_ref().is_some_and(|context| context.is_locked && (!context.IsActivatable(DisplayLockActivationReason::Any) || foundation::RuntimeEnabledFeatures::WarnOnContentVisibilityRenderAccessEnabled()));
        if !should_warn {
            return;
        }
        let mut state = self.display_lock_document_state.borrow_mut();
        if state.forced_render_warnings >= 500 {
            return;
        }
        state.forced_render_warnings += 1;
        let last_message = state.forced_render_warnings == 500;
        state.forced_effects.push_back(DisplayLockForcedEffect::ForcedRenderWarning { element: self.Node(element).Id(), is_warning: foundation::RuntimeEnabledFeatures::WarnOnContentVisibilityRenderAccessEnabled(), last_message });
    }
    pub fn TakeDisplayLockForcedEffects(&self) -> VecDeque<DisplayLockForcedEffect> {
        std::mem::take(&mut self.display_lock_document_state.borrow_mut().forced_effects)
    }
}
