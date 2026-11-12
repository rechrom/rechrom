// Copyright (C) 2013 Google Inc. All rights reserved.
// Use of this source code is governed by the BSD-style license retained in
// third_party/NOTICE.chromium-event-path.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_path.h:1-167
// Source: third_party/blink/renderer/core/dom/events/event_path.cc:1-466
// C++ includes/namespace/GC annotations map to modules and retained Rc owners.

use std::any::Any;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use super::event::{Event, EventPath as EventPathInterface, EventRef, EventTargetRef};
use super::node_event_context::{NodeEventContext, TreeScopeEventContextRef};
use super::tree_scope_event_context::TreeScopeEventContext;
use super::window_event_context::WindowEventContext;

// Opaque, shared identity supplied by DOM's existing document/tree-scope
// owner. No DOM tree or mutable arena is exposed at this boundary.
pub type TreeScopeRef = Rc<dyn Any>;
pub type DocumentIdentityRef = Rc<dyn Any>;
pub type DOMEventPathSnapshotRef = Rc<dyn DOMEventPathSnapshot>;

// Source: event_path.cc:107-151,216-231
// Immutable projection of one source path entry. DOM's snapshot supplies the
// path order, slot assignment, pseudo projection and scope-adjusted target.
pub struct EventPathEntry {
    pub node: EventTargetRef,
    pub current_target: EventTargetRef,
    pub tree_scope: TreeScopeRef,
    pub invocation_target_in_shadow_tree: bool,
    pub is_document: bool,
    pub dom_window: Option<EventTargetRef>,
    pub is_disabled_form_control: bool,
}

// Source: event_path.cc:153-231; tree_scope_event_context.cc:106-122
// These are the canonical scope results retained by the DOM path snapshot,
// including closed-root visibility and the source preorder/postorder values.
pub struct EventPathScope {
    pub identity: TreeScopeRef,
    pub root_node: EventTargetRef,
    pub parent: Option<TreeScopeRef>,
    pub target: Option<EventTargetRef>,
    pub is_closed_shadow_root: bool,
    pub containing_closed_shadow_tree: Option<TreeScopeRef>,
    pub pre_order: i32,
    pub post_order: i32,
}

// Only immutable node facts read by the related/touch adjustment algorithms.
pub struct EventPathNodeFacts {
    pub tree_scope: TreeScopeRef,
    pub document: DocumentIdentityRef,
    pub is_pseudo_element: bool,
    pub has_parent_node: bool,
    pub is_svg_element: bool,
}

// Source: event_path.cc:42-231,267-287,424-429
// DOM owns the tree/slot snapshot and these arrays. Interaction borrows them
// and retains the same owner while dispatch listeners mutate the live DOM.
// Implementations must not reconstruct a second general tree at this boundary.
pub trait DOMEventPathSnapshot {
    fn Target(&self) -> EventTargetRef;
    fn Entries(&self) -> &[EventPathEntry];
    fn TreeScopes(&self) -> &[EventPathScope];
    fn NodeFacts(&self, target: &EventTargetRef) -> Option<EventPathNodeFacts>;
    fn IsInclusiveAncestorTreeScopeOf(&self, ancestor: &TreeScopeRef, scope: &TreeScopeRef)
        -> bool;
    fn HasEventListeners(&self, node: &EventTargetRef, event_type: &str) -> bool;
    fn PseudoElementTarget(&self) -> Option<EventTargetRef>;
}

// The existing DOM path constructor supplies its immutable result, including
// pseudo/slot/shadow-root parent branches from event_path.cc:42-151. Related
// paths use the same document owner and the source's event=null construction.
pub trait DOMEventPathSnapshotSource {
    fn Snapshot(&self, node: &EventTargetRef, event: Option<&Event>) -> DOMEventPathSnapshotRef;
}

// Source: node_event_context.cc:62; window_event_context.cc:71
// Listener invocation receives the same Event owner with no borrow held by
// these contexts, preserving synchronous nested dispatch and script reentry.
/// Source Node::HandleLocalEvents virtual-class dispatch. The base callback
/// borrows the current listener/runtime scope and must run synchronously.
/// Implementations retain only canonical class owners, never a listener copy.
pub trait NodeLocalEventHandler {
    fn HandleNodeLocalEvents(
        &self,
        node: &EventTargetRef,
        event: &EventRef,
        base: &dyn Fn(&EventRef),
    );
}

pub trait LocalEventDispatch {
    /// Default actions borrow the current ScriptState after listener borrows
    /// have ended. Native-only dispatch has no VM scope to lend.
    fn WithScriptInvocation(
        &self,
        callback: &mut dyn FnMut(Option<&dyn crate::bindings::script_invocation::ScriptInvocation>),
    ) {
        callback(None);
    }

    /// Window::DispatchEvent uses EventTarget's direct at-target source path;
    /// it has no DOM ancestor event path to construct.
    fn DispatchWindowEvent(
        &self,
        window: &EventTargetRef,
        event: &EventRef,
    ) -> super::event_dispatch_result::DispatchEventResult;
    /// LocalDOMWindow::DispatchEvent(event, target) keeps Window currentTarget
    /// distinct from its legacy target override (notably Document for load).
    fn DispatchWindowEventWithTarget(
        &self,
        window: &EventTargetRef,
        event: &EventRef,
        legacy_target: &EventTargetRef,
    ) -> super::event_dispatch_result::DispatchEventResult;
    fn HandleNodeLocalEvents(&self, node: &EventTargetRef, event: &EventRef);
    fn FireWindowEventListeners(&self, window: &EventTargetRef, event: &EventRef);
    /// Release the script binding projection created for this canonical event.
    /// This runs once at the outer dispatch boundary, after source postprocess.
    fn DidFinishEvent(&self, event: &EventRef);
}

// Source: event_path.cc:342-405 and core/input/touch.h: CloneWithNewTarget.
// These narrow ports retain the concrete Touch/TouchList/TouchEventContext
// owners. All retarget iteration and append ordering remains below.
pub type TouchRef = Rc<dyn Touch>;
pub type TouchListRef = Rc<RefCell<dyn TouchList>>;
pub type TouchEventContextRef = Rc<dyn TouchEventContext>;
pub trait Touch {
    /// Touch::clientX reads the original retained client_pos_, distinct from
    /// page coordinates and AbsoluteLocation (touch.h:87, touch.cc:61-76).
    fn ClientX(&self) -> f64;
    // Source: Touch::AbsoluteLocation, retained with this retargeted Touch.
    fn AbsoluteLocation(
        &self,
    ) -> interaction_input::public::common::input::web_pointer_properties::PointF;
    fn target(&self) -> Option<EventTargetRef>;
    fn CloneWithNewTarget(&self, target: EventTargetRef) -> TouchRef;
}
pub trait TouchList {
    fn length(&self) -> usize;
    fn item(&self, index: usize) -> TouchRef;
    fn Append(&mut self, touch: TouchRef);
}
pub trait TouchEventContext {
    fn Touches(&self) -> TouchListRef;
    fn TargetTouches(&self) -> TouchListRef;
    fn ChangedTouches(&self) -> TouchListRef;
    fn HandleLocalEvents(&self, event: &EventRef);
}
pub trait TouchEventContextFactory {
    fn Create(&self) -> TouchEventContextRef;
}
pub trait TouchEvent {
    fn touches(&self) -> Option<TouchListRef>;
    fn targetTouches(&self) -> Option<TouchListRef>;
    fn changedTouches(&self) -> Option<TouchListRef>;
}

// Source: event_path.cc:118-121
// Apply this effect through Event::SetPseudoElementTarget at initialization,
// before listeners run. It carries the retained pseudo identity, not a lookup.
#[derive(Default)]
pub struct EventPathInitializationEffect {
    pub pseudo_element_target: Option<EventTargetRef>,
}

// Source: event_path.h:47-51,158-162
// Context vectors are per-dispatch state. The DOM snapshot is the only owner
// of path/tree facts; copying EventPath itself remains disabled.
pub struct EventPath {
    node_event_contexts_: Vec<NodeEventContext>,
    node_: EventTargetRef,
    event_type_: Option<String>,
    tree_scope_event_contexts_: Vec<TreeScopeEventContextRef>,
    window_event_context_: OnceCell<WindowEventContext>,
    snapshot_: DOMEventPathSnapshotRef,
    source_: Rc<dyn DOMEventPathSnapshotSource>,
    clear_target_only_if_in_shadow_tree_: bool,
    initialization_effect_: EventPathInitializationEffect,
}

// Source: event_path.h:149
// Keep the scope owner beside the opaque identity key so its allocation cannot
// be recycled while the map is live. Lookup has the source HashMap cost.
type RelatedTargetMap = HashMap<usize, (TreeScopeRef, Option<EventTargetRef>)>;

fn ScopeIdentity(scope: &TreeScopeRef) -> usize {
    Rc::as_ptr(scope) as *const () as usize
}

impl EventPath {
    // Source: event_path.h:49; event_path.cc:77-79
    pub fn new(
        snapshot: DOMEventPathSnapshotRef,
        source: Rc<dyn DOMEventPathSnapshotSource>,
        event: Option<&Event>,
        clear_target_only_if_in_shadow_tree: bool,
    ) -> Self {
        let mut path = Self {
            node_: snapshot.Target(),
            event_type_: event.map(|event| event.r#type().to_owned()),
            node_event_contexts_: Vec::new(),
            tree_scope_event_contexts_: Vec::new(),
            window_event_context_: OnceCell::new(),
            snapshot_: snapshot,
            source_: source,
            clear_target_only_if_in_shadow_tree_: clear_target_only_if_in_shadow_tree,
            initialization_effect_: EventPathInitializationEffect::default(),
        };
        path.Initialize();
        path
    }

    // Source: event_path.h:53; event_path.cc:81-88
    pub fn InitializeWithSnapshot(
        &mut self,
        snapshot: DOMEventPathSnapshotRef,
        event: Option<&Event>,
    ) {
        self.node_ = snapshot.Target();
        self.event_type_ = event.map(|event| event.r#type().to_owned());
        self.window_event_context_ = OnceCell::new();
        self.node_event_contexts_.clear();
        self.tree_scope_event_contexts_.clear();
        self.initialization_effect_ = EventPathInitializationEffect::default();
        self.snapshot_ = snapshot;
        self.Initialize();
    }

    // Source: event_path.h:55-92
    pub fn NodeEventContexts(&self) -> &[NodeEventContext] {
        &self.node_event_contexts_
    }
    pub fn NodeEventContextsMut(&mut self) -> &mut [NodeEventContext] {
        &mut self.node_event_contexts_
    }
    pub fn at(&self, index: usize) -> &NodeEventContext {
        &self.node_event_contexts_[index]
    }
    pub fn Last(&self) -> &NodeEventContext {
        self.node_event_contexts_
            .last()
            .expect("nonempty event path")
    }

    // Source: event_path.h:94-98; event_path.cc:437-443
    pub fn GetWindowEventContext(&self) -> &WindowEventContext {
        self.window_event_context_
            .get()
            .expect("window event context must be initialized")
    }
    pub fn EnsureWindowEventContext(&self) {
        let event_type = self
            .event_type_
            .as_ref()
            .expect("window routing requires an event");
        self.window_event_context_.get_or_init(|| {
            WindowEventContext::new(
                event_type,
                self.TopNodeEventContext(),
                &self.snapshot_.Entries()[self.size() - 1],
                self.clear_target_only_if_in_shadow_tree_,
            )
        });
    }

    // Source: event_path.h:100-101
    pub fn IsEmpty(&self) -> bool {
        self.node_event_contexts_.is_empty()
    }
    pub fn size(&self) -> usize {
        self.node_event_contexts_.len()
    }

    // Source: event_path.h:113; event_path.cc:432-435
    pub fn TopNodeEventContext(&self) -> &NodeEventContext {
        debug_assert!(!self.IsEmpty());
        self.Last()
    }

    // Source: event_path.h:115; event_path.cc:42-58
    // DOM calculated the pseudo activation/originating-element rule once when
    // freezing this entry. Return its original retained current-target owner.
    pub fn EventTargetRespectingTargetRules(entry: &EventPathEntry) -> EventTargetRef {
        entry.current_target.clone()
    }

    // Source: event_path.h:117-121; event_path.cc:458-464
    // Rc owners replace Trace. Clear explicitly releases the two source vectors.
    pub fn Clear(&mut self) {
        self.node_event_contexts_.clear();
        self.tree_scope_event_contexts_.clear();
    }

    pub fn TakeInitializationEffect(&mut self) -> EventPathInitializationEffect {
        std::mem::take(&mut self.initialization_effect_)
    }

    // Source: event_path.h:126; event_path.cc:90-105
    fn Initialize(&mut self) {
        // The source orphaned-pseudo/stopped-document rejection (cc:90-96) is
        // part of the DOM snapshot constructor and yields an empty entry slice.
        if self.snapshot_.Entries().is_empty() {
            return;
        }
        self.CalculatePath();
        self.CalculateAdjustedTargets();
        self.CalculateTreeOrderAndSetNearestAncestorClosedTree();
    }

    // Source: event_path.h:127; event_path.cc:60-75,107-151
    // The immutable DOM path already executed KeepEventInNode, assigned-slot,
    // non-composed original shadow-root and pseudo-parent branches. Project its
    // exact order; a second live-DOM traversal here would violate its snapshot.
    fn CalculatePath(&mut self) {
        debug_assert!(self.node_event_contexts_.is_empty());
        self.initialization_effect_.pseudo_element_target = self.snapshot_.PseudoElementTarget();
        self.node_event_contexts_ = self
            .snapshot_
            .Entries()
            .iter()
            .map(NodeEventContext::new)
            .collect();
    }

    // Source: event_path.h:128; event_path.cc:176-231
    // Scope grouping and target inheritance also come from the same DOM path
    // owner. Interaction creates only mutable dispatch contexts for those scopes.
    fn CalculateAdjustedTargets(&mut self) {
        for index in 0..self.snapshot_.TreeScopes().len() {
            let mut context = TreeScopeEventContext::new(self.snapshot_.clone(), index);
            if let Some(target) = &self.snapshot_.TreeScopes()[index].target {
                context.SetTarget(target.clone());
            }
            self.tree_scope_event_contexts_
                .push(Rc::new(RefCell::new(context)));
        }
        for (context, entry) in self
            .node_event_contexts_
            .iter_mut()
            .zip(self.snapshot_.Entries())
        {
            let scope = self
                .snapshot_
                .TreeScopes()
                .iter()
                .position(|scope| Rc::ptr_eq(&scope.identity, &entry.tree_scope))
                .expect("DOM path entry has a scope");
            context.SetTreeScopeEventContext(self.tree_scope_event_contexts_[scope].clone());
        }
    }

    // Source: event_path.h:129; event_path.cc:153-174
    fn CalculateTreeOrderAndSetNearestAncestorClosedTree(&mut self) {
        for (index, scope) in self.snapshot_.TreeScopes().iter().enumerate() {
            let closed = scope.containing_closed_shadow_tree.as_ref().map(|closed| {
                self.GetTreeScopeEventContext(closed)
                    .expect("closed scope occurs in DOM path")
            });
            self.tree_scope_event_contexts_[index]
                .borrow_mut()
                .ImportSnapshotOrder(scope, closed.as_ref());
            if let Some(parent) = &scope.parent {
                self.GetTreeScopeEventContext(parent)
                    .expect("parent scope occurs in DOM path")
                    .borrow_mut()
                    .AddChild(self.tree_scope_event_contexts_[index].clone());
            }
        }
    }

    // Source: event_path.h:131-134
    fn Shrink(&mut self, new_size: usize) {
        debug_assert!(self.window_event_context_.get().is_none());
        self.node_event_contexts_.truncate(new_size);
    }

    // Source: event_path.h:145; event_path.cc:176-185
    pub fn GetTreeScopeEventContext(
        &self,
        scope: &TreeScopeRef,
    ) -> Option<TreeScopeEventContextRef> {
        self.tree_scope_event_contexts_
            .iter()
            .find(|context| Rc::ptr_eq(&context.borrow().GetTreeScope(), scope))
            .cloned()
    }

    // Source: event_path.h:146-147; event_path.cc:187-214
    // A snapshot contains the recursively ensured parent scopes. This accessor
    // resolves that canonical result without performing a second tree walk.
    pub fn EnsureTreeScopeEventContext(
        &self,
        scope: Option<&TreeScopeRef>,
    ) -> Option<TreeScopeEventContextRef> {
        scope.and_then(|scope| self.GetTreeScopeEventContext(scope))
    }

    // Source: event_path.h:151; event_path.cc:233-245
    fn BuildRelatedNodeMap(&self, related_node: &EventTargetRef, map: &mut RelatedTargetMap) {
        let snapshot = self.source_.Snapshot(related_node, None);
        for scope in snapshot.TreeScopes() {
            Self::InsertRelatedNode(map, scope.identity.clone(), scope.target.clone());
        }
        // The temporary DOM snapshot owner is released after extracting only
        // the source per-scope related-target handles; no tree is retained.
    }

    // Source: event_path.h:152; event_path.cc:247-265
    fn FindRelatedNode(&self, scope: &TreeScopeRef, map: &mut RelatedTargetMap) -> EventTargetRef {
        let mut parents = Vec::new();
        let mut current = Some(scope.clone());
        let mut related = None;
        while let Some(scope) = current {
            parents.push(scope.clone());
            if let Some((_, Some(target))) = map.get(&ScopeIdentity(&scope)) {
                related = Some(target.clone());
                break;
            }
            current = self
                .snapshot_
                .TreeScopes()
                .iter()
                .find(|entry| Rc::ptr_eq(&entry.identity, &scope))
                .and_then(|entry| entry.parent.clone());
        }
        let related = related.expect("related path reaches a common document scope");
        for scope in parents {
            Self::InsertRelatedNode(map, scope, Some(related.clone()));
        }
        related
    }

    fn InsertRelatedNode(
        map: &mut RelatedTargetMap,
        scope: TreeScopeRef,
        target: Option<EventTargetRef>,
    ) {
        // HeapHashMap::insert does not overwrite an existing entry.
        map.entry(ScopeIdentity(&scope)).or_insert((scope, target));
    }

    // Source: event_path.h:103; event_path.cc:267-287
    pub fn AdjustForRelatedTarget(
        &mut self,
        target: &EventTargetRef,
        related_target: Option<EventTargetRef>,
    ) {
        let Some(related) = related_target else {
            return;
        };
        let Some(related_node) = self.snapshot_.NodeFacts(&related) else {
            return;
        };
        if related_node.is_pseudo_element && !related_node.has_parent_node {
            return;
        }
        let target_node = self
            .snapshot_
            .NodeFacts(target)
            .expect("event target is a node");
        if !Rc::ptr_eq(&target_node.document, &related_node.document) {
            return;
        }
        self.RetargetRelatedTarget(&related);
        self.ShrinkForRelatedTarget(target, &related);
    }

    // Source: event_path.h:136; event_path.cc:289-303
    fn RetargetRelatedTarget(&mut self, related_target: &EventTargetRef) {
        let mut map = RelatedTargetMap::new();
        self.BuildRelatedNodeMap(related_target, &mut map);
        for context in &self.tree_scope_event_contexts_ {
            let scope = context.borrow().GetTreeScope();
            let adjusted = self.FindRelatedNode(&scope, &mut map);
            context.borrow_mut().SetRelatedTarget(adjusted);
        }
        map.clear();
    }

    // Source: event_path.cc:307-327
    fn ShouldStopEventPath(
        &self,
        target: &EventTargetRef,
        related: &EventTargetRef,
        event_target: &EventTargetRef,
        event_related: &EventTargetRef,
    ) -> bool {
        if !Rc::ptr_eq(target, related) {
            return false;
        }
        let Some(adjusted_target) = self.snapshot_.NodeFacts(target) else {
            return false;
        };
        let Some(adjusted_related) = self.snapshot_.NodeFacts(related) else {
            return false;
        };
        let original_target = self
            .snapshot_
            .NodeFacts(event_target)
            .expect("event target node");
        let original_related = self
            .snapshot_
            .NodeFacts(event_related)
            .expect("event related node");
        !(Rc::ptr_eq(&adjusted_target.tree_scope, &original_target.tree_scope)
            && Rc::ptr_eq(&adjusted_related.tree_scope, &original_related.tree_scope))
    }

    // Source: event_path.h:138-139; event_path.cc:331-340
    fn ShrinkForRelatedTarget(&mut self, target: &EventTargetRef, related: &EventTargetRef) {
        for index in 0..self.size() {
            let context = &self.node_event_contexts_[index];
            if self.ShouldStopEventPath(
                &context.Target().expect("adjusted target"),
                &context.RelatedTarget().expect("adjusted related target"),
                target,
                related,
            ) {
                self.Shrink(index);
                break;
            }
        }
    }

    // Source: event_path.h:104; event_path.cc:342-377
    pub fn AdjustForTouchEvent(
        &self,
        event: &dyn TouchEvent,
        factory: &dyn TouchEventContextFactory,
    ) {
        let mut touches = Vec::new();
        let mut target_touches = Vec::new();
        let mut changed_touches = Vec::new();
        let mut scopes = Vec::new();
        for context in &self.tree_scope_event_contexts_ {
            let mut context = context.borrow_mut();
            let touch = context.EnsureTouchEventContext(factory);
            touches.push(touch.Touches());
            target_touches.push(touch.TargetTouches());
            changed_touches.push(touch.ChangedTouches());
            scopes.push(context.GetTreeScope());
        }
        self.AdjustTouchList(event.touches(), touches, &scopes);
        self.AdjustTouchList(event.targetTouches(), target_touches, &scopes);
        self.AdjustTouchList(event.changedTouches(), changed_touches, &scopes);
        #[cfg(debug_assertions)]
        for context in &self.tree_scope_event_contexts_ {
            let context = context.borrow();
            let scope = context.GetTreeScope();
            let touch = context
                .GetTouchEventContext()
                .expect("touch context was ensured");
            self.CheckReachability(&scope, &touch.Touches());
            self.CheckReachability(&scope, &touch.TargetTouches());
            self.CheckReachability(&scope, &touch.ChangedTouches());
        }
    }

    // Source: event_path.h:141-143; event_path.cc:379-405
    fn AdjustTouchList(
        &self,
        touches: Option<TouchListRef>,
        adjusted: Vec<TouchListRef>,
        scopes: &[TreeScopeRef],
    ) {
        let Some(touches) = touches else {
            return;
        };
        let length = touches.borrow().length();
        for index in 0..length {
            let touch = touches.borrow().item(index);
            let Some(target) = touch.target() else {
                continue;
            };
            if self.snapshot_.NodeFacts(&target).is_none() {
                continue;
            }
            let mut map = RelatedTargetMap::new();
            self.BuildRelatedNodeMap(&target, &mut map);
            for (index, scope) in scopes.iter().enumerate() {
                let target = self.FindRelatedNode(scope, &mut map);
                adjusted[index]
                    .borrow_mut()
                    .Append(touch.CloneWithNewTarget(target));
            }
            map.clear();
        }
    }

    // Source: event_path.h:105-108; event_path.cc:407-414
    pub fn AdjustForDisabledFormControl(&mut self) {
        for index in 0..self.node_event_contexts_.len() {
            if self.snapshot_.Entries()[index].is_disabled_form_control {
                self.Shrink(index);
                return;
            }
        }
    }

    // Source: event_path.h:110; event_path.cc:416-422
    pub fn DisabledFormControlExistsInPath(&self) -> bool {
        self.snapshot_.Entries()[..self.size()]
            .iter()
            .any(|entry| entry.is_disabled_form_control)
    }

    // Source: event_path.h:111; event_path.cc:424-430
    pub fn HasEventListenersInPath(&self, event_type: &str) -> bool {
        self.node_event_contexts_.iter().any(|context| {
            self.snapshot_
                .HasEventListeners(&context.GetNode(), event_type)
        })
    }

    // Source: event_path.h:154-156; event_path.cc:445-456
    #[cfg(debug_assertions)]
    fn CheckReachability(&self, scope: &TreeScopeRef, list: &TouchListRef) {
        let list = list.borrow();
        for index in 0..list.length() {
            let target = list.item(index).target().expect("adjusted touch target");
            let node = self
                .snapshot_
                .NodeFacts(&target)
                .expect("adjusted touch is a node");
            debug_assert!(self
                .snapshot_
                .IsInclusiveAncestorTreeScopeOf(&node.tree_scope, scope));
        }
    }
}

// Event's resident path projection. Scope and target identity remain the
// snapshot's identities even if listeners reparent nodes in the live DOM.
impl EventPathInterface for EventPath {
    fn NodeTargets(&self) -> Vec<EventTargetRef> {
        self.node_event_contexts_
            .iter()
            .map(NodeEventContext::GetNode)
            .collect()
    }
    fn IsEmpty(&self) -> bool {
        self.IsEmpty()
    }
    fn InitializeWith(&mut self, node: &EventTargetRef, event: &Event) {
        let snapshot = self.source_.Snapshot(node, Some(event));
        self.InitializeWithSnapshot(snapshot, Some(event));
    }
    fn ComposedPathForNode(&self, node: &EventTargetRef) -> Option<Vec<EventTargetRef>> {
        self.EnsureWindowEventContext();
        let context = self
            .node_event_contexts_
            .iter()
            .find(|context| Rc::ptr_eq(&context.GetNode(), node))?;
        let scope = context.GetTreeScopeEventContext();
        let path = scope.borrow().EnsureEventPath(self);
        Some(path.as_ref().clone())
    }
    fn TopScopeComposedPath(&self) -> Vec<EventTargetRef> {
        self.EnsureWindowEventContext();
        let scope = self.TopNodeEventContext().GetTreeScopeEventContext();
        let path = scope.borrow().EnsureEventPath(self);
        path.as_ref().clone()
    }
}
