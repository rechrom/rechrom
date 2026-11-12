//! Retained DOM identities and canonical DOM event-path projections.
//!
//! DOM freezes tree/slot/scope facts. This adapter only replaces its IDs with
//! stable dispatch handles and keeps that same snapshot alive during reentry.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use dom::{
    persistent_document::{DOMNamespace, DOMNodeType, DOMOwnerHandle},
    DOMEventPath, Document,
};
use interaction_dispatch::core::dom::events::{
    event::{
        Event, EventPathFactory, EventPathRef, EventRef, EventTarget, EventTargetKind,
        EventTargetRef,
    },
    event_dispatcher::{DispatchPathAccess, DispatchPathRef},
    event_path::{
        DOMEventPathSnapshot, DOMEventPathSnapshotRef, DOMEventPathSnapshotSource,
        DocumentIdentityRef, EventPath as DispatchEventPath, EventPathEntry, EventPathNodeFacts,
        EventPathScope, TreeScopeRef,
    },
};

use super::{event::EventTargetResolver, interaction_contract::InteractionDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TargetIdentity {
    owner: DOMOwnerHandle,
    id: u64,
}

struct DomEventTarget {
    // Retain the arena owner, rather than an arena address borrowed across JS.
    _document: Option<InteractionDocument>,
    window_document: Option<Rc<WindowDocumentOwner>>,
    kind: EventTargetKind,
    identity: TargetIdentity,
    registry: Weak<DomEventTargets>,
}

impl interaction_dispatch::core::events::ui_event::AbstractView for DomEventTarget {}

impl EventTarget for DomEventTarget {
    fn AsAbstractView(
        &self,
    ) -> Option<interaction_dispatch::core::events::ui_event::AbstractViewRef> {
        if self.kind != EventTargetKind::Window {
            return None;
        }
        let registry = self.registry.upgrade()?;
        let view = registry.targets.borrow().get(&self.identity).cloned()?;
        Some(view)
    }
    fn SecurityOriginOwner(&self) -> Option<Rc<dyn foundation::security::SecurityOriginOwner>> {
        let document = self._document.as_ref()?.borrow();
        (document.GetDocument().RootHandle() == self.identity.owner).then(|| {
            document.GetDocument().SecurityContext().Origin().clone()
                as Rc<dyn foundation::security::SecurityOriginOwner>
        })
    }
    fn NodeId(&self) -> Option<u64> {
        self._document.as_ref().map(|_| self.identity.id)
    }
    fn Kind(&self) -> EventTargetKind {
        self.kind
    }
    fn IsPseudoElement(&self) -> bool {
        self.pseudo_facts().is_some()
    }
    fn IsConnected(&self) -> bool {
        let Some(document_owner) = self._document.as_ref() else {
            return false;
        };
        let owner = document_owner.borrow();
        let document = owner.GetDocument();
        document.RootHandle() == self.identity.owner
            && document
                .FindNodeById(self.identity.id)
                .is_some_and(|node| document.IsConnectedToDocument(node))
    }
    fn HasActivationBehavior(&self) -> bool {
        self.pseudo_facts()
            .is_some_and(|facts| facts.HasActivationBehavior())
    }
    fn CorrespondingElement(&self) -> Option<EventTargetRef> {
        let id = {
            let owner = self._document.as_ref()?.borrow();
            let document = owner.GetDocument();
            if document.RootHandle() != self.identity.owner {
                return None;
            }
            let node = document.FindNodeById(self.identity.id)?;
            document
                .PseudoElementFacts(node)
                .map(|p| document.Node(p.originating_element).Id())?
        };
        Some(self.registry.upgrade()?.target(self.identity.owner, id))
    }
}
impl DomEventTarget {
    fn pseudo_facts(&self) -> Option<dom::persistent_document::DOMPseudoElementFacts> {
        let owner = self._document.as_ref()?.borrow();
        let document = owner.GetDocument();
        if document.RootHandle() != self.identity.owner {
            return None;
        }
        document.PseudoElementFacts(document.FindNodeById(self.identity.id)?)
    }
}
impl interaction_dispatch::core::dom::events::event::CSSPseudoElement for DomEventTarget {
    fn Element(&self) -> EventTargetRef {
        self.CorrespondingElement()
            .expect("CSSPseudoElement retains its originating element")
    }
}

/// The Document installed in one LocalDOMWindow. Reuse retargets this same
/// owner; a new Window detaches the old frame without changing its identity.
pub struct WindowDocumentOwner {
    capabilities: interaction_dispatch::core::input::input_device_capabilities::InputDeviceCapabilitiesConstants,
    document: RefCell<InteractionDocument>,
    attached: Cell<bool>,
}
impl WindowDocumentOwner {
    pub fn Document(&self) -> InteractionDocument {
        self.document.borrow().clone()
    }
    pub fn IsAttached(&self) -> bool {
        self.attached.get()
    }
    pub fn DetachFrame(&self) {
        self.attached.set(false);
    }
}

/// One identity registry for the live DOM owner. Zero names its Window; all
/// node and scope keys also include the arena identity, so equal IDs in other
/// documents never alias. Detached targets retain the same handle.
pub struct DomEventTargets {
    self_: Weak<Self>,
    document: InteractionDocument,
    targets: RefCell<HashMap<TargetIdentity, Rc<DomEventTarget>>>,
    identities: RefCell<HashMap<usize, TargetIdentity>>,
    scopes: RefCell<HashMap<TargetIdentity, TreeScopeRef>>,
    documents: RefCell<HashMap<DOMOwnerHandle, DocumentIdentityRef>>,
    reporting_contexts: RefCell<
        HashMap<
            DOMOwnerHandle,
            Rc<interaction_dispatch::core::frame::reporting_context::ReportingContext>,
        >,
    >,
    console_storage:
        Rc<interaction_dispatch::core::frame::reporting_context::ConsoleMessageStorage>,
}

fn target_pointer(target: &EventTargetRef) -> usize {
    Rc::as_ptr(target) as *const () as usize
}

impl DomEventTargets {
    pub fn new(document: InteractionDocument) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            document,
            targets: RefCell::new(HashMap::new()),
            identities: RefCell::new(HashMap::new()),
            scopes: RefCell::new(HashMap::new()),
            documents: RefCell::new(HashMap::new()),
            reporting_contexts: RefCell::new(HashMap::new()),
            console_storage: Rc::new(Default::default()),
        })
    }

    /// LocalDOMWindow::ClearForReuse keeps the Window and gives the new
    /// Document a fresh node/scope registry. No listener or node table is copied.
    pub fn ForWindowReuse(document: InteractionDocument, previous: &Rc<Self>) -> Rc<Self> {
        let previous_owner = previous.document.borrow().GetDocument().RootHandle();
        previous.target(previous_owner, 0);
        let window = previous.targets.borrow()[&TargetIdentity {
            owner: previous_owner,
            id: 0,
        }]
            .clone();
        let current = Self::new(document);
        let owner = current.document.borrow().GetDocument().RootHandle();
        let identity = TargetIdentity { owner, id: 0 };
        current
            .identities
            .borrow_mut()
            .insert(Rc::as_ptr(&window) as *const () as usize, identity);
        current.targets.borrow_mut().insert(identity, window);
        current
    }

    /// The registry's original Document, including an unpublished replacement.
    pub fn DocumentOwner(&self) -> InteractionDocument {
        self.document.clone()
    }

    /// document.cc:8913-8926. Keyboard and editing commands share this exact
    /// target selection, without a visible-selection query or layout barrier.
    pub(crate) fn EventTargetNodeForDocument(
        &self,
        focused: Option<u64>,
    ) -> Option<EventTargetRef> {
        if let Some(target) = focused.and_then(|id| self.ResolveTarget(id)) {
            return Some(target);
        }
        // PluginDocument is absent from the embedding's document kinds.
        let id = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            let body = (document.DocumentType(document.Root())
                == dom::persistent_document::DOMDocumentType::kHTML)
                .then(|| document.BodyElement())
                .flatten();
            document
                .Node(body.or_else(|| document.DocumentElement())?)
                .Id()
        };
        self.ResolveTarget(id)
    }

    /// Publish only after the source Window choice and graph preparation.
    pub fn PublishWindowDocument(&self) {
        let holder = self.WindowDocument();
        *holder.document.borrow_mut() = self.document.clone();
        holder.attached.set(true);
    }

    pub fn WindowDocument(&self) -> Rc<WindowDocumentOwner> {
        let owner = self.document.borrow().GetDocument().RootHandle();
        self.target(owner, 0);
        self.targets.borrow()[&TargetIdentity { owner, id: 0 }]
            .window_document
            .as_ref()
            .expect("Window document owner")
            .clone()
    }

    fn target(&self, owner: DOMOwnerHandle, id: u64) -> EventTargetRef {
        let identity = TargetIdentity { owner, id };
        if let Some(target) = self.targets.borrow().get(&identity) {
            return target.clone();
        }
        let target = Rc::new(DomEventTarget {
            _document: (id != 0).then(|| self.document.clone()),
            window_document: (id == 0).then(|| {
                Rc::new(WindowDocumentOwner {
                    capabilities: Default::default(),
                    document: RefCell::new(self.document.clone()),
                    attached: Cell::new(true),
                })
            }),
            identity,
            registry: self.self_.clone(),
            kind: if id == 0 {
                EventTargetKind::Window
            } else {
                EventTargetKind::Node
            },
        });
        self.identities
            .borrow_mut()
            .insert(Rc::as_ptr(&target) as *const () as usize, identity);
        self.targets.borrow_mut().insert(identity, target.clone());
        target
    }

    /// Resolves through the same registry while the canonical Document is
    /// already borrowed by its pre-unlink callback. This must not reborrow the
    /// InteractionDocument RefCell.
    pub(crate) fn TargetFromBorrowedDocument(
        &self,
        document: &Document,
        id: u64,
    ) -> Option<EventTargetRef> {
        if id != 0 && document.FindNodeById(id).is_none() {
            return None;
        }
        Some(self.target(document.RootHandle(), id))
    }

    pub fn InputDeviceCapabilities(
        &self,
        fires_touch: bool,
    ) -> interaction_dispatch::core::events::ui_event::InputDeviceCapabilitiesRef {
        let owner = self.document.borrow().GetDocument().RootHandle();
        self.target(owner, 0);
        let window = self
            .targets
            .borrow()
            .get(&TargetIdentity { owner, id: 0 })
            .expect("canonical Window target")
            .window_document
            .clone()
            .expect("Window document owner");
        window.capabilities.FiresTouchEvents(fires_touch)
    }

    /// UIEvent.view is the same Window allocation used as dispatch target.
    pub fn WindowView(&self) -> interaction_dispatch::core::events::ui_event::AbstractViewRef {
        let owner = self.document.borrow().GetDocument().RootHandle();
        self.target(owner, 0);
        self.targets
            .borrow()
            .get(&TargetIdentity { owner, id: 0 })
            .expect("canonical Window target")
            .clone()
    }

    /// The original Window supplement, keyed by its immutable Document owner.
    /// Event and observer projections retain this same ReportingContext Rc.
    pub fn ReportingContextForView(
        &self,
        view: &interaction_dispatch::core::events::ui_event::AbstractViewRef,
    ) -> Option<Rc<interaction_dispatch::core::frame::reporting_context::ReportingContext>> {
        let target: EventTargetRef = view.clone();
        let identity = self.identity(&target)?;
        if identity.id != 0 {
            return None;
        }
        Some(
            self.reporting_contexts
                .borrow_mut()
                .entry(identity.owner)
                .or_insert_with(|| Rc::new(Default::default()))
                .clone(),
        )
    }
    pub fn ConsoleMessageStorage(
        &self,
    ) -> Rc<interaction_dispatch::core::frame::reporting_context::ConsoleMessageStorage> {
        self.console_storage.clone()
    }

    /// The immutable owning Document identity of this canonical EventTarget.
    /// A new document may reuse numeric node IDs without becoming this target.
    pub fn OwnerDocumentHandle(&self, target: &EventTargetRef) -> Option<DOMOwnerHandle> {
        self.identity(target).map(|identity| identity.owner)
    }

    /// Node::ownerDocument() projected as the canonical Document EventTarget.
    /// Document derives from Node in Blink, so this is the existing root-node
    /// target rather than a synthetic target kind or sentinel identity.
    pub fn OwnerDocumentTarget(&self, target: &EventTargetRef) -> Option<EventTargetRef> {
        let identity = self.identity(target)?;
        Some(self.target(identity.owner, identity.owner.Id()))
    }

    fn identity(&self, target: &EventTargetRef) -> Option<TargetIdentity> {
        self.identities
            .borrow()
            .get(&target_pointer(target))
            .copied()
    }

    fn scope(&self, owner: DOMOwnerHandle, root: u64) -> TreeScopeRef {
        self.scopes
            .borrow_mut()
            .entry(TargetIdentity { owner, id: root })
            .or_insert_with(|| Rc::new(TargetIdentity { owner, id: root }))
            .clone()
    }

    fn document_identity(&self, owner: DOMOwnerHandle) -> DocumentIdentityRef {
        self.documents
            .borrow_mut()
            .entry(owner)
            .or_insert_with(|| Rc::new(owner))
            .clone()
    }
}

impl EventTargetResolver for DomEventTargets {
    fn DocumentOwnerIdentity(&self) -> Option<u128> {
        Some(self.document.borrow().GetDocument().RootHandle().Identity())
    }
    fn ResolveTarget(&self, node_id: u64) -> Option<EventTargetRef> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        if node_id != 0 && document.FindNodeById(node_id).is_none() {
            return None;
        }
        Some(self.target(document.RootHandle(), node_id))
    }

    fn NodeId(&self, target: &EventTargetRef) -> Option<u64> {
        // No DOM borrow is needed when a listener reads target/currentTarget.
        self.identity(target).map(|identity| identity.id)
    }
}

pub type DomEventListenerQuery = Rc<dyn Fn(u64, &str) -> bool>;

struct DomEventPathsState {
    targets: Rc<DomEventTargets>,
    listeners: DomEventListenerQuery,
    // Event exposes a trait handle; the dispatcher needs its concrete owner.
    // Weak aliases add neither a second path nor another dispatch lifetime.
    paths: RefCell<HashMap<usize, Weak<RefCell<DispatchEventPath>>>>,
}

#[derive(Clone)]
pub struct DomEventPaths(Rc<DomEventPathsState>);

impl DomEventPaths {
    pub fn new(targets: Rc<DomEventTargets>, listeners: DomEventListenerQuery) -> Self {
        Self(Rc::new(DomEventPathsState {
            targets,
            listeners,
            paths: RefCell::new(HashMap::new()),
        }))
    }

    /// Project the resident dispatch path for PointerEventManager's removal
    /// checks (pointer_event_manager.cc:1090-1096). The path itself owns the
    /// contexts; no second node list or path is published.
    pub fn PointerNodeEventContexts(
        &self,
        event: &EventRef,
    ) -> Rc<dyn crate::core::input::pointer_event_manager::PointerNodeEventContexts> {
        struct Contexts(DispatchPathRef);
        impl crate::core::input::pointer_event_manager::PointerNodeEventContexts for Contexts {
            fn Len(&self) -> usize {
                self.0.borrow().NodeEventContexts().len()
            }
            fn GetNode(&self, index: usize) -> EventTargetRef {
                self.0.borrow().at(index).GetNode()
            }
        }
        let path = event.borrow().BaseEvent().GetEventPath();
        Rc::new(Contexts(self.concrete_path(&path)))
    }

    fn concrete_path(&self, path: &EventPathRef) -> DispatchPathRef {
        let identity = Rc::as_ptr(path) as *const () as usize;
        self.0
            .paths
            .borrow()
            .get(&identity)
            .and_then(Weak::upgrade)
            .expect("event path belongs to this DOM dispatch environment")
    }
}

impl EventPathFactory for DomEventPaths {
    fn Create(&self, node: &EventTargetRef, event: &Event) -> EventPathRef {
        let snapshot = self.Snapshot(node, Some(event));
        let path = Rc::new(RefCell::new(DispatchEventPath::new(
            snapshot,
            Rc::new(self.clone()),
            Some(event),
            true,
        )));
        let interface: EventPathRef = path.clone();
        let mut paths = self.0.paths.borrow_mut();
        paths.retain(|_, owner| owner.strong_count() != 0);
        paths.insert(
            Rc::as_ptr(&interface) as *const () as usize,
            Rc::downgrade(&path),
        );
        interface
    }
}

impl DispatchPathAccess for DomEventPaths {
    fn InitEventPath(&self, node: &EventTargetRef, event: &EventRef) -> DispatchPathRef {
        let interface = {
            let mut event = event.borrow_mut();
            let base = event.BaseEventMut();
            base.InitEventPath(node, self);
            base.GetEventPath()
        };
        let path = self.concrete_path(&interface);
        let effect = path.borrow_mut().TakeInitializationEffect();
        if let Some(pseudo) = effect.pseudo_element_target {
            // CSSPseudoElementInterface is stable in the source revision.
            event.borrow_mut().BaseEventMut().SetPseudoElementTarget(
                &pseudo,
                true,
                self.0.targets.as_ref(),
            );
        }
        path
    }

    fn EventTargetRespectingTargetRules(&self, node: &EventTargetRef) -> EventTargetRef {
        if node.IsPseudoElement() && !node.HasActivationBehavior() && node.IsConnected() {
            node.CorrespondingElement()
                .expect("pseudo target has an originating element")
        } else {
            node.clone()
        }
    }
}

struct DomEventPathProjection {
    canonical: DOMEventPath,
    owner: DOMOwnerHandle,
    paths: DomEventPaths,
    entries: Vec<EventPathEntry>,
    scopes: Vec<EventPathScope>,
}

impl DOMEventPathSnapshotSource for DomEventPaths {
    fn Snapshot(&self, node: &EventTargetRef, event: Option<&Event>) -> DOMEventPathSnapshotRef {
        let identity = self
            .0
            .targets
            .identity(node)
            .expect("dispatch target belongs to the DOM owner");
        let owner = self.0.targets.document.borrow();
        let document = owner.GetDocument();
        assert_eq!(
            identity.owner,
            document.RootHandle(),
            "target arena is current"
        );
        let index = document
            .FindNodeById(identity.id)
            .expect("dispatch node remains retained by DOM");
        // Related-target paths use EventPath(node, nullptr), which crosses the
        // original shadow root. DOM alone owns assigned-slot/host traversal.
        let canonical = DOMEventPath::ForEvent(
            document,
            index,
            event.is_none_or(Event::composed),
            event.map(|event| dom::event_path::DOMEventPathEvent {
                name: event.r#type(),
                class: match event.EventPathClass() {
                    interaction_dispatch::core::dom::events::event::EventPathClass::Other => {
                        dom::event_path::DOMEventPathClass::Other
                    }
                    interaction_dispatch::core::dom::events::event::EventPathClass::Keyboard => {
                        dom::event_path::DOMEventPathClass::Keyboard
                    }
                    interaction_dispatch::core::dom::events::event::EventPathClass::Touch => {
                        dom::event_path::DOMEventPathClass::Touch
                    }
                },
            }),
        );
        let targets = &self.0.targets;
        let scopes = canonical
            .TreeScopes()
            .iter()
            .map(|scope| EventPathScope {
                identity: targets.scope(identity.owner, scope.root_node_id),
                root_node: targets.target(identity.owner, scope.root_node_id),
                parent: scope.parent.map(|parent| {
                    targets.scope(identity.owner, canonical.TreeScopes()[parent].root_node_id)
                }),
                target: scope.target.map(|id| targets.target(identity.owner, id)),
                is_closed_shadow_root: scope.closed,
                containing_closed_shadow_tree: scope.containing_closed_tree.map(|closed| {
                    targets.scope(identity.owner, canonical.TreeScopes()[closed].root_node_id)
                }),
                pre_order: i32::try_from(scope.pre_order).expect("scope order fits i32"),
                post_order: i32::try_from(scope.post_order).expect("scope order fits i32"),
            })
            .collect();
        let entries = canonical
            .Entries()
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let facts = canonical.EntryFacts(index);
                let scope = &canonical.TreeScopes()[canonical.EntryTreeScope(index)];
                let node = targets.target(identity.owner, entry.invocation_target);
                EventPathEntry {
                    node: node.clone(),
                    current_target: node,
                    tree_scope: targets.scope(identity.owner, scope.root_node_id),
                    invocation_target_in_shadow_tree: facts.in_shadow_tree,
                    is_document: facts.is_document,
                    // Only DOM's active document supplies this environment's
                    // Window. Detached documents have no Window.
                    dom_window: (canonical.DocumentTarget() == Some(entry.invocation_target))
                        .then(|| targets.target(identity.owner, 0)),
                    is_disabled_form_control: facts.is_disabled_form_control,
                }
            })
            .collect();
        Rc::new(DomEventPathProjection {
            canonical,
            owner: identity.owner,
            paths: self.clone(),
            entries,
            scopes,
        })
        // `owner` and every arena reference are released here, before any
        // translated context or listener can invoke script.
    }
}

impl DOMEventPathSnapshot for DomEventPathProjection {
    fn Target(&self) -> EventTargetRef {
        self.paths
            .0
            .targets
            .target(self.owner, self.canonical.Target())
    }

    fn Entries(&self) -> &[EventPathEntry] {
        &self.entries
    }

    fn TreeScopes(&self) -> &[EventPathScope] {
        &self.scopes
    }

    fn NodeFacts(&self, target: &EventTargetRef) -> Option<EventPathNodeFacts> {
        let targets = &self.paths.0.targets;
        let identity = targets.identity(target)?;
        if identity.owner != self.owner || identity.id == 0 {
            return None;
        }
        if let Some(index) = self
            .canonical
            .Entries()
            .iter()
            .position(|entry| entry.invocation_target == identity.id)
        {
            let facts = self.canonical.EntryFacts(index);
            return Some(EventPathNodeFacts {
                tree_scope: self.scopes[self.canonical.EntryTreeScope(index)]
                    .identity
                    .clone(),
                document: targets.document_identity(facts.document_owner),
                is_pseudo_element: facts.is_pseudo_element,
                has_parent_node: facts.has_parent,
                is_svg_element: facts.is_svg_element,
            });
        }
        // Related/touch adjustment asks for targets outside the invocation
        // path before dispatch. Read just those node facts from their owner;
        // the canonical related Snapshot supplies all scope traversal/order.
        let owner = targets.document.borrow();
        let document = owner.GetDocument();
        if document.RootHandle() != self.owner {
            return None;
        }
        let index = document.FindNodeById(identity.id)?;
        let node = document.Node(index);
        Some(EventPathNodeFacts {
            tree_scope: targets.scope(
                self.owner,
                document.Node(document.TreeScopeRoot(index)).Id(),
            ),
            document: targets.document_identity(document.NodeDocumentHandle(index)),
            is_pseudo_element: false, // DOM pseudos are not arena EventTargets.
            has_parent_node: node.Parent().is_some(),
            is_svg_element: node.Type() == DOMNodeType::kElement
                && node.Namespace() == DOMNamespace::kSVG,
        })
    }

    fn IsInclusiveAncestorTreeScopeOf(
        &self,
        ancestor: &TreeScopeRef,
        scope: &TreeScopeRef,
    ) -> bool {
        let index = |identity| {
            self.scopes
                .iter()
                .position(|scope| Rc::ptr_eq(&scope.identity, identity))
        };
        let (Some(ancestor), Some(scope)) = (index(ancestor), index(scope)) else {
            return false;
        };
        self.canonical
            .IsInclusiveAncestorTreeScopeOf(ancestor, scope)
    }

    fn HasEventListeners(&self, node: &EventTargetRef, event_type: &str) -> bool {
        self.paths.0.targets.identity(node).is_some_and(|identity| {
            identity.owner == self.owner && (self.paths.0.listeners)(identity.id, event_type)
        })
    }

    fn PseudoElementTarget(&self) -> Option<EventTargetRef> {
        self.canonical
            .PseudoElementTarget()
            .map(|id| self.paths.0.targets.target(self.owner, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::DOM;
    use interaction_dispatch::core::dom::events::event::EventInit;
    use interaction_input::base::time::time::TimeTicks;

    #[test]
    fn target_handles_preserve_owner_node_and_window_identity() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let targets = DomEventTargets::new(document.clone());
        let id = {
            let mut owner = document.borrow_mut();
            let document = owner.GetDocumentMut();
            let node = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            document.Node(node).Id()
        };
        let first = targets.ResolveTarget(id).unwrap();
        let second = targets.ResolveTarget(id).unwrap();
        assert!(Rc::ptr_eq(&first, &second));
        assert_eq!(targets.NodeId(&first), Some(id));
        let window = targets.ResolveTarget(0).unwrap();
        assert!(Rc::ptr_eq(&window, &targets.ResolveTarget(0).unwrap()));
        assert!(!Rc::ptr_eq(&window, &first));
        let other = DomEventTargets::new(Rc::new(RefCell::new(DOM::new())));
        assert_eq!(other.NodeId(&first), None);
        assert!(!Rc::ptr_eq(&window, &other.ResolveTarget(0).unwrap()));
    }

    #[test]
    fn event_and_dispatcher_share_the_frozen_path_after_dom_reentry() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let (parent, child, child_index) = {
            let mut owner = document.borrow_mut();
            let document = owner.GetDocumentMut();
            let parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            let child = document.CreateElementDefault(DOMNamespace::kHTML, "span".into());
            document.AppendChild(document.Root(), parent);
            document.AppendChild(parent, child);
            (document.Node(parent).Id(), document.Node(child).Id(), child)
        };
        let targets = DomEventTargets::new(document.clone());
        let reentrant_document = document.clone();
        let paths = DomEventPaths::new(
            targets.clone(),
            Rc::new(move |_, _| {
                let mut owner = reentrant_document.borrow_mut();
                let document = owner.GetDocumentMut();
                document.AppendChild(document.Root(), child_index);
                true
            }),
        );
        let event = EventRef::new(Event::WithInitializer(
            "probe",
            &EventInit {
                composed: true,
                ..Default::default()
            },
            TimeTicks::default(),
        ));
        let child = targets.ResolveTarget(child).unwrap();
        let path = paths.InitEventPath(&child, &event);
        let interface = event.borrow().BaseEvent().GetEventPath();
        let concrete_interface: EventPathRef = path.clone();
        assert!(Rc::ptr_eq(&interface, &concrete_interface));
        event
            .borrow_mut()
            .BaseEventMut()
            .SetCurrentTarget(targets.ResolveTarget(parent));
        let composed_ids = || {
            event
                .borrow()
                .BaseEvent()
                .composedPath(true)
                .iter()
                .filter_map(|target| targets.NodeId(target))
                .collect::<Vec<_>>()
        };
        let before = composed_ids();
        // The injected query mutates DOM synchronously, proving snapshot
        // construction kept no arena borrow across host callback entry.
        assert!(path.borrow().HasEventListenersInPath("probe"));
        assert_eq!(composed_ids(), before);
        assert!(before.contains(&parent));
    }
}

impl interaction_dispatch::core::dom::events::event::CSSPseudoElementFactory for DomEventTargets {
    fn From(
        &self,
        pseudo: &EventTargetRef,
    ) -> Rc<dyn interaction_dispatch::core::dom::events::event::CSSPseudoElement> {
        let identity = self.identity(pseudo).expect("canonical pseudo EventTarget");
        self.targets
            .borrow()
            .get(&identity)
            .expect("retained canonical pseudo owner")
            .clone()
    }
}
