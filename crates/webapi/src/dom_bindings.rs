#![allow(non_snake_case)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use dom::dom_mutation::{DOMMutation, DOMMutationEmitter, DOMMutationType};
use dom::persistent_document::{DOMNamespace, DOMNodeType};
use dom::{Document, DOM};
use javascript::javascript_runtime::{
    HostCall, HostContinuation, HostFunctionBinding, HostMethodRef, HostObjectId, HostObjectRef,
    HostOperation, HostResult, HostSymbol, HostValue, JavaScriptExceptionKind, JavaScriptFunction,
    JavaScriptHostBindings, JavaScriptHostObjectNamespace, JavaScriptHostRuntime, JavaScriptNull,
    JavaScriptRealm, JavaScriptValue, WeakJavaScriptHostObjectNamespace, WeakJavaScriptRealm,
};
use layoutng_assembly::internal::layout_input::PaintPathVerb;

#[path = "dom_attributes.rs"]
mod attributes;
#[path = "dom_custom_elements.rs"]
mod custom_elements;
#[path = "dom_editor_commands.rs"]
mod editor_commands;
#[path = "dom_element_internals.rs"]
mod element_internals;
#[path = "dom_form_validation.rs"]
mod form_validation;
#[path = "dom_host_graph.rs"]
mod host_graph;
#[path = "dom_input_event.rs"]
mod input_event;
#[path = "dom_range_selection.rs"]
mod range_selection;
#[path = "dom_services.rs"]
mod services;
#[path = "dom_text_control_selection.rs"]
mod text_control_selection;
use host_graph::{DOMHostGraph, LiveCollectionIds};
#[path = "dom_abort_signals.rs"]
mod abort_signals;
#[path = "dom_event_listener_invocation.rs"]
mod event_listener_invocation;
#[path = "dom_event_listeners.rs"]
mod event_listeners;
use event_listeners::{Listener, ListenerTargets};
use interaction::scroll::core::frame::event_handler_registry::EventHandlerRegistryTargets;
pub use services::{
    CloseWatcherHost, DOMBindingsHost, DOMSelectionHost, DOMWindowViewConverter,
    DOMWindowViewProjector, DocumentFocusState, FrameProperty, FramePropertyGetter,
    TextControlSelectionHost,
};
#[path = "viewport_scroll.rs"]
mod viewport_scroll;
pub use viewport_scroll::{ViewportScrollBehavior, ViewportScrollState};
#[path = "dom_native_mutation_observers.rs"]
mod native_mutation_observers;
pub use native_mutation_observers::DOMNativeMutationObservers;

const GLOBAL_NAMES: &[&str] = &[
    "document",
    "__domInvoke",
    "__domInstall",
    "__domWrap",
    "window",
    "event",
    "self",
    "parent",
    "top",
    "addEventListener",
    "removeEventListener",
    "dispatchEvent",
    "onload",
    "onerror",
    "onclick",
    "oninput",
    "onchange",
    "onkeydown",
    "onkeyup",
];

#[derive(Clone)]
struct NodeCollection {
    root: HostObjectId,
    kind: String,
    query: String,
    snapshot: Vec<HostObjectId>,
    membership_cache: RefCell<Option<CollectionMembershipCache>>,
}
#[derive(Clone)]
struct CollectionMembershipCache {
    key: (dom::persistent_document::DOMOwnerHandle, u64),
    ids: Rc<Vec<HostObjectId>>,
}
// Rc<Vec> keeps the original miss allocation without copying IDs into Rc<[Id]>.
// The caller owns this view; no RefCell cache guard escapes the traversal.
enum CollectionNodes<'a> {
    Borrowed(&'a [HostObjectId]),
    Shared(Rc<Vec<HostObjectId>>),
    Owned(Vec<HostObjectId>),
}
impl std::ops::Deref for CollectionNodes<'_> {
    type Target = [HostObjectId];
    fn deref(&self) -> &[HostObjectId] {
        match self {
            Self::Borrowed(ids) => ids,
            Self::Shared(ids) => ids.as_slice(),
            Self::Owned(ids) => ids.as_slice(),
        }
    }
}
impl CollectionNodes<'_> {
    fn into_owned(self) -> Vec<HostObjectId> {
        match self {
            Self::Borrowed(ids) => ids.to_vec(),
            Self::Shared(ids) => ids.as_ref().clone(),
            Self::Owned(ids) => ids,
        }
    }
}
struct CollectionIterator {
    collection: HostObjectId,
    index: usize,
    done: bool,
}
#[derive(Clone)]
enum ListenerCallback {
    // Window/global targets live for the realm. Native DOM targets instead
    // own their callback through the engine-traced owner cell.
    Rooted(JavaScriptFunction),
    Traced(javascript::javascript_runtime::WeakJavaScriptFunction),
}
impl ListenerCallback {
    fn Resolve(&self) -> Option<JavaScriptFunction> {
        match self {
            Self::Rooted(callback) => Some(callback.clone()),
            Self::Traced(callback) => callback.Upgrade(),
        }
    }
    fn Matches(&self, callback: &JavaScriptFunction) -> bool {
        self.Resolve().as_ref() == Some(callback)
    }
}
/// Read-only access to the original Document's live listener registrations.
/// Retaining this port does not borrow the JavaScript bindings across dispatch
/// or freeze registrations before synchronous listener reentry.
#[derive(Clone)]
pub struct DOMEventListenerInterest {
    document: dom::persistent_document::DOMOwnerHandle,
    listeners: ListenerTargets,
    registry: Rc<EventHandlerRegistryTargets>,
}

impl DOMEventListenerInterest {
    /// Frame-wide source EventHandlerRegistry fact, updated by registration.
    /// This query is constant time and never inspects DOM or listener vectors.
    pub fn HasPointerRawUpdateEventHandlers(&self) -> bool {
        self.registry.HasPointerRawUpdateEventHandlers()
    }
    pub fn DocumentIdentity(&self) -> dom::persistent_document::DOMOwnerHandle {
        self.document
    }

    // Chromium: core/dom/events/event_target.h:414-419;
    // core/dom/events/event_listener_map.cc:56-62.
    pub fn HasEventListeners(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target: HostObjectId,
        event_type: &str,
    ) -> bool {
        document == self.document
            && self
                .listeners
                .borrow()
                .get(&target)
                .is_some_and(|target| target.HasEventListenersForType(event_type))
    }

    // Chromium: core/dom/events/event_target.h:432-438;
    // core/dom/events/event_listener_map.cc:64-76.
    pub fn HasCapturingEventListeners(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target: HostObjectId,
        event_type: &str,
    ) -> bool {
        document == self.document
            && self
                .listeners
                .borrow()
                .get(&target)
                .is_some_and(|target| target.HasCapturingEventListeners(event_type))
    }
}

impl interaction::event::EventListenerInterest for DOMEventListenerInterest {
    fn HasPointerEventHandlers(&self) -> bool {
        self.registry.HasPointerEventHandlers()
    }
    fn HasPointerRawUpdateEventHandlers(&self) -> bool {
        self.registry.HasPointerRawUpdateEventHandlers()
    }
    fn HasTouchEventHandlers(&self) -> bool {
        self.registry.HasTouchEventHandlers()
    }
    fn HasBlockingTouchHandler(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
    ) -> bool {
        document == self.document
            && self
                .listeners
                .borrow()
                .get(&target_node_id)
                .is_some_and(|target| {
                    self.registry
                        .HasBlockingTouchHandler(&target.AsEventTargetRef())
                })
    }
    fn HasAttributeEventListener(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
        event_type: &str,
    ) -> bool {
        document == self.document
            && self
                .listeners
                .borrow()
                .get(&target_node_id)
                .is_some_and(|target| target.GetAttributeEventListener(event_type).is_some())
    }
    fn HasEventListeners(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
        event_type: &str,
    ) -> bool {
        DOMEventListenerInterest::HasEventListeners(self, document, target_node_id, event_type)
    }

    fn HasCapturingEventListeners(
        &self,
        document: dom::persistent_document::DOMOwnerHandle,
        target_node_id: u64,
        event_type: &str,
    ) -> bool {
        DOMEventListenerInterest::HasCapturingEventListeners(
            self,
            document,
            target_node_id,
            event_type,
        )
    }
}

// Scoped ownership adapter for source DOMBindingsHost::change_focus.
pub type FocusChangeHandler = Rc<dyn Fn(HostObjectId, bool, &mut dyn JavaScriptHostRuntime)>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerCaptureOperation {
    Set,
    Release,
    Has,
}
pub type PointerCaptureHandler = Rc<
    dyn Fn(
        PointerCaptureOperation,
        interaction::input::public::common::input::pointer_id::PointerId,
        HostObjectId,
    ) -> Result<bool, interaction::PointerCaptureError>,
>;
/// Canvas draw commands request a rendering opportunity without fabricating a
/// DOM attribute/text mutation. Page supplies the scheduler-facing callback.
pub type CanvasDirtyHandler = Rc<dyn Fn(HostObjectId)>;
// A source Emit can synchronously reenter JavaScript (image events and
// mutation notifications). Execute it after releasing the host borrow.
pub type ScopedDOMMutationEmitter = Rc<
    dyn Fn(&DOMMutation, &mut dyn JavaScriptHostRuntime) -> Result<(), dom::error::DOMException>,
>;
/// Lifetime-only binding to the original Document's native execution owners.
/// Implementations must not retain this binding, Window, realm or executor.
/// The old Window host retains this lease when a new Window is installed.
pub trait NativeDOMDocumentOwner {
    fn DocumentIdentity(&self) -> dom::persistent_document::DOMOwnerHandle;
}
pub struct DOMJavaScriptBindings {
    document: Rc<RefCell<DOM>>,
    form_validation_state: Rc<interaction::input_type::DefaultActionState>,
    native_document_owner: Option<Rc<dyn NativeDOMDocumentOwner>>,
    object_namespace: WeakJavaScriptHostObjectNamespace,
    services: services::DOMServices,
    emit_mutation: Rc<RefCell<DOMMutationEmitter>>,
    scoped_emit_mutation: Option<ScopedDOMMutationEmitter>,
    prototypes: HashMap<String, JavaScriptValue>,
    method_wrappers: HashMap<String, JavaScriptFunction>,
    runtime_realm: WeakJavaScriptRealm,
    current_event: Rc<RefCell<Option<HostValue>>>,
    callback_context:
        Rc<RefCell<Option<javascript::javascript_runtime::JavaScriptCallbackContext>>>,
    cssom_world: Rc<RefCell<Option<Rc<style::ProductionCSSDOMWrapperWorld>>>>,
    native_mutation_observers: Rc<DOMNativeMutationObservers>,
    current_script: HostObjectId,
    focused_node: Option<u64>,
    write_document: Option<Box<dyn FnMut(&str) -> bool>>,
    change_focus: Option<FocusChangeHandler>,
    pointer_capture: Option<PointerCaptureHandler>,
    collections: Rc<RefCell<HashMap<HostObjectId, NodeCollection>>>,
    // cpp: NodeListsNodeData / LiveNodeList collection caching. Index by the
    // actual owner and collection key, never scan every retained collection.
    live_collection_ids: Rc<RefCell<LiveCollectionIds>>,
    iterators: Rc<RefCell<HashMap<HostObjectId, CollectionIterator>>>,
    records: Rc<RefCell<HashMap<HostObjectId, HashMap<String, HostValue>>>>,
    native_record_classes: Rc<RefCell<HashMap<HostObjectId, &'static str>>>,
    task_attribution_tracker:
        Rc<std::cell::OnceCell<Rc<foundation::task_attribution::TaskAttributionTracker>>>,
    host_graph: Rc<DOMHostGraph>,
    next_collection_id: Rc<Cell<HostObjectId>>,
    native_abort: Rc<abort_signals::AbortBindings>,
    window_native_bindings: Rc<abort_signals::AbortBindings>,
    listeners: ListenerTargets,
    event_handler_registry: Rc<EventHandlerRegistryTargets>,
    next_listener_slot: Cell<u64>,
    custom_element_backup_queued: bool,
    selector_service: Rc<RefCell<style::persistent_selector::PersistentSelectorService>>,
    // Context registration aliases the canonical DOM element state.
    canvas_elements: RefCell<HashMap<HostObjectId, Rc<RefCell<canvas::HTMLCanvasElementState>>>>,
    webgl_contexts: RefCell<HashMap<HostObjectId, webgl::WebGLRenderingContextState>>,
    canvas_webgl_ids: RefCell<HashMap<HostObjectId, HostObjectId>>,
    webgl_canvas_ids: RefCell<HashMap<HostObjectId, HostObjectId>>,
    next_webgl_id: Cell<HostObjectId>,
    canvas_dirty: Option<CanvasDirtyHandler>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DOMHostMemoryDiagnostics {
    pub collections: usize,
    pub iterators: usize,
    pub result_records: usize,
    pub record_fields: usize,
    pub listener_targets: usize,
    pub listener_entries: usize,
    pub events: usize,
    pub event_identities: usize,
}
impl DOMJavaScriptBindings {
    pub fn SetCloseWatcherHost(&self, host: CloseWatcherHost) {
        self.native_abort.BindCloseWatcherHost(host);
    }
    pub(crate) fn BindAbortTaskScheduler(&self, scheduler: abort_signals::AbortTaskScheduler) {
        self.native_abort.BindScheduler(scheduler);
    }
    pub fn EventListenerInterest(&self) -> DOMEventListenerInterest {
        DOMEventListenerInterest {
            document: self.document.borrow().GetDocument().RootHandle(),
            listeners: self.listeners.clone(),
            registry: self.event_handler_registry.clone(),
        }
    }

    /// Document's original SelectorQuery cache, shared with native consumers.
    pub fn SelectorQueryOwner(
        &self,
    ) -> Rc<RefCell<style::persistent_selector::PersistentSelectorService>> {
        self.selector_service.clone()
    }
}
impl DOMJavaScriptBindings {
    pub fn HasPointerRawUpdateEventHandlers(&self) -> bool {
        self.event_handler_registry
            .HasPointerRawUpdateEventHandlers()
    }
    pub fn SetCanvasDirtyHandler(&mut self, handler: CanvasDirtyHandler) {
        self.canvas_dirty = Some(handler);
    }

    fn NotifyCanvasDirty(&self, context_id: HostObjectId) {
        if let (Some(handler), Some(dom_node_id)) = (
            self.canvas_dirty.as_ref(),
            self.webgl_canvas_ids.borrow().get(&context_id).copied(),
        ) {
            handler(dom_node_id);
        }
    }

    /// Publish immutable front-buffer snapshots at the Page lifecycle boundary.
    /// Cross-origin texture/image uploads will flip `origin_clean` when those
    /// WebGL entry points are added; the current vertical slice accepts only
    /// same-document shader/buffer input and is therefore origin-clean.
    pub fn SnapshotCanvasPaintResources(&self) -> Vec<canvas::CanvasPaintResource> {
        self.document
            .borrow()
            .GetDocument()
            .SnapshotCanvasPaintResources()
    }

    pub fn MemoryDiagnostics(&self) -> DOMHostMemoryDiagnostics {
        let records = self.records.borrow();
        let listeners = self.listeners.borrow();
        let (events, event_identities) = self.services.MemoryCounts();
        DOMHostMemoryDiagnostics {
            collections: self.collections.borrow().len(),
            iterators: self.iterators.borrow().len(),
            result_records: records.len(),
            record_fields: records.values().map(HashMap::len).sum(),
            listener_targets: listeners
                .values()
                .filter(|target| target.HasEventListeners())
                .count(),
            listener_entries: listeners
                .values()
                .map(|target| {
                    target
                        .EventTypes()
                        .iter()
                        .map(|kind| target.NumberOfEventListeners(kind))
                        .sum::<usize>()
                })
                .sum(),
            events,
            event_identities,
        }
    }
    fn selector_error(error: style::persistent_selector::PersistentSelectorError) -> HostResult {
        match error {
            style::persistent_selector::PersistentSelectorError::Syntax(error) => {
                HostResult::Failure(JavaScriptExceptionKind::kSyntaxError, error.message.Utf8())
            }
            style::persistent_selector::PersistentSelectorError::Unsupported(message) => {
                HostResult::Failure(
                    JavaScriptExceptionKind::kRuntimeError,
                    format!("NotSupportedError: {}", message),
                )
            }
        }
    }
    /// EventListenerProperties::blocking_event_listeners analogue used when
    /// deciding whether wheel input may run on the compositor thread.
    pub fn HasBlockingListener(&self, kind: &str) -> bool {
        if kind == "wheel" {
            return self.event_handler_registry.HasBlockingWheelEventHandlers();
        }
        self.listeners
            .borrow()
            .values()
            .any(|target| event_listeners::HasBlockingListener(target, kind))
    }

    /// Targets whose active listeners can cancel this event. Geometry belongs
    /// to Page/PrePaint; bindings expose identities only.
    pub fn BlockingListenerTargets(&self, kind: &str) -> Vec<HostObjectId> {
        self.listeners
            .borrow()
            .iter()
            .filter_map(|(id, target)| {
                (if kind == "wheel" {
                    self.event_handler_registry
                        .HasBlockingWheelHandler(&target.AsEventTargetRef())
                } else {
                    event_listeners::HasBlockingListener(target, kind)
                })
                .then_some(*id)
            })
            .collect()
    }
    pub fn SetBlockingWheelInvalidation(&self, emit: Rc<dyn Fn(Option<u64>)>) {
        self.event_handler_registry
            .SetBlockingWheelInvalidation(emit);
    }

    // cpp: webapi/dom_bindings.cc:236-240
    pub fn new(document: Rc<RefCell<DOM>>, emit_mutation: DOMMutationEmitter) -> Self {
        Self::WithHost(document, emit_mutation, DOMBindingsHost::default())
    }
    pub fn WithHost(
        document: Rc<RefCell<DOM>>,
        mut emit_mutation: DOMMutationEmitter,
        mut host: DOMBindingsHost,
    ) -> Self {
        // This is the script binding mutation entry. Frame-owner hooks retain
        // its explicit InContext fact through released-borrow notification
        // drains; parser/host mutations enter DOM without this scope.
        let mutation_document = Rc::downgrade(&document);
        let emit_mutation: DOMMutationEmitter = Box::new(move |mutation| {
            let _context = mutation_document.upgrade().map(|document| {
                document
                    .borrow()
                    .GetDocument()
                    .EnterFrameOwnerCreationContext(
                        dom::frame_owner_loading::FrameOwnerCreationContext::Script,
                    )
            });
            emit_mutation(mutation);
        });
        // Binding this Document to its Window supplies Document constructor's
        // dom_window_ branch (document.cc:1010), on the canonical owner.
        document
            .borrow()
            .GetDocument()
            .SetReadyState(dom::document_parser_lifecycle::DocumentReadyState::Loading);
        let collections = Rc::new(RefCell::new(HashMap::new()));
        let live_collection_ids = Rc::new(RefCell::new(HashMap::new()));
        let iterators = Rc::new(RefCell::new(HashMap::new()));
        let records = Rc::new(RefCell::new(HashMap::new()));
        let listeners = Rc::new(RefCell::new(HashMap::new()));
        if host.event_clock.is_none() {
            host.event_clock = Some(Rc::new(services::StandaloneEventClock::new()));
        }
        let event_clock = host.event_clock.clone();
        let services = services::DOMServices::new(host);
        let native_record_classes = Rc::new(RefCell::new(HashMap::new()));
        let host_graph = Rc::new(DOMHostGraph::new(
            document.clone(),
            collections.clone(),
            live_collection_ids.clone(),
            iterators.clone(),
            records.clone(),
            native_record_classes.clone(),
            listeners.clone(),
            services.events.clone(),
        ));
        let native_mutation_observers =
            DOMNativeMutationObservers::new(&document, host_graph.clone());
        let next_collection_id = Rc::new(Cell::new(1 << 62));
        let current_event = Rc::new(RefCell::new(None));
        let exception_ids = next_collection_id.clone();
        let exception_classes = native_record_classes.clone();
        let exception_records = records.clone();
        let exception_graph = Rc::downgrade(&host_graph);
        let native_abort = abort_signals::AbortBindings::new(
            &document,
            &host_graph,
            next_collection_id.clone(),
            native_record_classes.clone(),
            listeners.clone(),
            services.events.clone(),
            current_event.clone(),
            services.event_state.clone(),
            event_clock,
            Rc::new(move |name, message| {
                Self::CreateDOMExceptionRecord(
                    &exception_ids,
                    &exception_classes,
                    &exception_records,
                    &exception_graph
                        .upgrade()
                        .expect("live DOMException binding graph"),
                    HostValue::String(name.into()),
                    HostValue::String(message.into()),
                )
            }),
        );
        assert!(host_graph.abort_bindings.set(native_abort.clone()).is_ok());
        let form_validation_state = Rc::new(interaction::input_type::DefaultActionState::default());
        form_validation_state
            .SetValidationPatternEngine(crate::form_validation::CreateValidationPatternEngine());
        interaction::adapter::page_form_validation::InstallStandaloneInputValueSanitizer(
            &document,
            form_validation_state.clone(),
        );
        Self {
            form_validation_state,
            native_mutation_observers,
            native_document_owner: None,
            object_namespace: Default::default(),
            services,
            document,
            emit_mutation: Rc::new(RefCell::new(emit_mutation)),
            scoped_emit_mutation: None,
            prototypes: HashMap::new(),
            method_wrappers: HashMap::new(),
            runtime_realm: WeakJavaScriptRealm::default(),
            current_event,
            callback_context: Default::default(),
            cssom_world: Default::default(),
            current_script: 0,
            focused_node: None,
            write_document: None,
            change_focus: None,
            pointer_capture: None,
            collections,
            live_collection_ids,
            iterators,
            records,
            native_record_classes,
            task_attribution_tracker: Default::default(),
            host_graph,
            next_collection_id,
            window_native_bindings: native_abort.clone(),
            native_abort,
            listeners,
            event_handler_registry: Rc::new(EventHandlerRegistryTargets::default()),
            next_listener_slot: Cell::new(1 << 61),
            custom_element_backup_queued: false,
            selector_service: Rc::new(RefCell::new(Default::default())),
            canvas_elements: RefCell::new(HashMap::new()),
            webgl_contexts: RefCell::new(HashMap::new()),
            canvas_webgl_ids: RefCell::new(HashMap::new()),
            webgl_canvas_ids: RefCell::new(HashMap::new()),
            next_webgl_id: Cell::new(1 << 60),
            canvas_dirty: None,
        }
    }

    fn NewListenerSlot(&self) -> u64 {
        let slot = self.next_listener_slot.get();
        assert!(slot < 1 << 62, "listener reference slots exhausted");
        self.next_listener_slot.set(slot + 1);
        slot
    }
    fn StoreListenerCallback(
        &self,
        target: HostObjectId,
        slot: u64,
        callback: &JavaScriptFunction,
    ) -> ListenerCallback {
        self.host_graph
            .ListenerReference(target, slot, Some(callback.clone()));
        ListenerCallback::Traced(callback.Downgrade())
    }
    fn RemoveListenerReference(&self, listener: &Listener) {
        if matches!(listener.callback, ListenerCallback::Traced(_)) {
            self.host_graph
                .ListenerReference(listener.target, listener.reference_slot, None);
        }
    }

    // cpp: webapi/dom_bindings.h:33
    // scoped runtime is a Rust reborrow adapter.
    pub fn BindFormValidationState(
        &mut self,
        state: Rc<interaction::input_type::DefaultActionState>,
    ) {
        self.form_validation_state = state;
    }
    pub fn SetFormValidationHandler(&mut self, handler: Option<services::FormValidationHandler>) {
        self.services.host.validate_form = handler;
    }
    pub fn SetDocumentEditorCommandHandler(
        &mut self,
        handler: Option<services::DocumentEditorCommandHandler>,
    ) {
        self.services.host.document_editor_command = handler;
    }
    pub fn SetDocumentEditorCommandExecutor(
        &mut self,
        handler: Option<services::DocumentEditorCommandExecutor>,
    ) {
        self.services.host.execute_document_editor_command = handler;
    }
    pub fn SetDOMSelectionHost(&mut self, host: Option<services::DOMSelectionHost>) {
        self.services.host.selection = host;
    }
    pub fn SetTextControlSelectionHost(
        &mut self,
        host: Option<services::TextControlSelectionHost>,
    ) {
        self.services.host.text_control_selection = host;
    }
    pub fn SetFocusChangeHandler(&mut self, handler: Option<FocusChangeHandler>) {
        self.change_focus = handler;
    }
    pub fn SetDOMEventTargetResolver(&mut self, resolver: services::DOMEventTargetResolver) {
        self.services.host.resolve_event_target = Some(resolver);
    }

    pub fn SetPointerCaptureHandler(&mut self, handler: Option<PointerCaptureHandler>) {
        self.pointer_capture = handler;
    }

    pub fn SetScopedMutationEmitter(&mut self, emitter: ScopedDOMMutationEmitter) {
        let document = Rc::downgrade(&self.document);
        self.scoped_emit_mutation = Some(Rc::new(move |mutation, runtime| {
            let _context = document.upgrade().map(|document| {
                document
                    .borrow()
                    .GetDocument()
                    .EnterFrameOwnerCreationContext(
                        dom::frame_owner_loading::FrameOwnerCreationContext::Script,
                    )
            });
            emitter(mutation, runtime)
        }));
    }

    pub fn BindNativeDocumentOwner(&mut self, owner: Rc<dyn NativeDOMDocumentOwner>) {
        assert!(
            self.native_document_owner.is_none(),
            "one native owner for this Window binding"
        );
        assert_eq!(
            owner.DocumentIdentity(),
            self.document.borrow().GetDocument().RootHandle()
        );
        self.native_document_owner = Some(owner);
    }

    pub fn DisassociateWindowDocument(&mut self) {
        let id = self
            .document
            .borrow()
            .GetDocument()
            .Node(self.document.borrow().GetDocument().Root())
            .Id();
        self.host_graph.SetRoot(id, 0, false);
    }
    pub(crate) fn DocumentURLForWindow(&self) -> String {
        self.document
            .borrow()
            .GetDocument()
            .DocumentURL()
            .to_owned()
    }
    pub fn BindHostObjectNamespace(&mut self, namespace: &JavaScriptHostObjectNamespace) {
        assert!(
            self.object_namespace.Upgrade().is_none(),
            "one namespace for these original bindings"
        );
        self.object_namespace = namespace.Downgrade();
        self.native_abort.BindNamespace(namespace);
        self.native_mutation_observers
            .BindHostObjectNamespace(namespace);
    }
    pub fn HostObjectNamespace(&self) -> Option<JavaScriptHostObjectNamespace> {
        self.object_namespace.Upgrade()
    }
    /// Reusing an existing Context keeps its installed, original IDL objects;
    /// it does not evaluate bootstrap or consult replaceable author globals.
    pub fn CopyWindowIDLBindingsForReuse(&mut self, previous: &Self) {
        self.window_native_bindings = previous.window_native_bindings.clone();
        let window_target = previous.ListenerTarget(0);
        self.listeners.borrow_mut().insert(0, window_target);
        self.current_event = previous.current_event.clone();
        self.native_abort
            .BindCurrentEvent(self.current_event.clone());
        if let Some(callback) = previous.method_wrappers.get("__callEventHandler") {
            self.native_abort.BindHandlerWrapper(callback);
        }
        if let Some(callback) = previous.method_wrappers.get("__inputDataTransferFileValue") {
            self.native_abort.BindInputFileProjection(callback);
        }
        assert!(self.prototypes.is_empty() && self.method_wrappers.is_empty());
        self.prototypes = previous.prototypes.clone();
        self.method_wrappers = previous.method_wrappers.clone();
        if let Some(delivery) = self.method_wrappers.get("__deliverMutationObserver") {
            // The retained IDL wrapper table owns this callback. The observer
            // keeps only its weak identity, as in the initial installation.
            self.native_mutation_observers.BindScriptDelivery(delivery);
        }
        *self.services.event_state.borrow_mut() = previous.services.event_state.borrow().clone();
    }

    // cpp: webapi/dom_bindings.h:50
    // The caller supplies a full or current-context runtime during delivery.
    // Retain only the source binding's validity, avoiding a host/realm cycle.
    pub fn BindRuntime(&mut self, realm: &JavaScriptRealm) {
        self.runtime_realm = realm.Downgrade();
        self.native_abort.BindRuntime(realm);
        self.native_mutation_observers.BindRuntime(realm);
    }

    pub fn NativeMutationObservers(&self) -> Rc<DOMNativeMutationObservers> {
        self.native_mutation_observers.clone()
    }

    pub fn SetReadyState(&mut self, state: String) {
        use dom::document_parser_lifecycle::DocumentReadyState as S;
        let state = match state.as_str() {
            "loading" => S::Loading,
            "interactive" => S::Interactive,
            "complete" => S::Complete,
            _ => panic!("source Document ready state"),
        };
        self.document.borrow().GetDocument().SetReadyState(state);
    }

    pub fn SetCurrentScript(&mut self, id: HostObjectId) {
        if self.current_script != id {
            self.host_graph.SetRoot(self.current_script, 5, false);
            self.host_graph.SetRoot(id, 5, true);
        }
        self.current_script = id;
    }

    // cpp: webapi/dom_bindings.h:58
    pub fn SetFocusedNode(&mut self, id: Option<u64>) {
        if self.focused_node != id {
            if let Some(old) = self.focused_node {
                self.host_graph.SetRoot(old, 3, false);
            }
            if let Some(id) = id {
                self.host_graph.SetRoot(id, 3, true);
            }
        }
        self.focused_node = id;
    }

    // Rust callback adapter for DOMBindingsHost::write_document.
    pub fn SetDocumentWriteHandler(&mut self, handler: Box<dyn FnMut(&str) -> bool>) {
        self.write_document = Some(handler);
    }

    fn value(value: HostValue) -> HostResult {
        HostResult {
            value,
            ..Default::default()
        }
    }

    fn unhandled() -> HostResult {
        HostResult {
            handled: false,
            ..Default::default()
        }
    }

    fn popup_error(error: &dom::error::DOMException) -> HostResult {
        let name = match error.kind {
            dom::error::DOMExceptionKind::NotSupportedError => "NotSupportedError",
            dom::error::DOMExceptionKind::InvalidNodeType => "InvalidNodeTypeError",
            dom::error::DOMExceptionKind::IndexSizeError => "IndexSizeError",
            dom::error::DOMExceptionKind::NotFoundError => "NotFoundError",
            dom::error::DOMExceptionKind::InvalidStateError => "InvalidStateError",
            _ => return Self::type_error(error.message),
        };
        Self::value(HostValue::Record(vec![
            ("exceptionName".into(), HostValue::String(name.into())),
            ("message".into(), HostValue::String(error.message.into())),
        ]))
    }
    fn type_error(message: &str) -> HostResult {
        HostResult::Failure(JavaScriptExceptionKind::kTypeError, message)
    }

    fn node(&self, id: HostObjectId) -> Option<usize> {
        self.document.borrow().GetDocument().FindNodeById(id)
    }

    fn method(&self, receiver: HostObjectId, name: &str) -> HostResult {
        if let Some(wrapper) = self.method_wrappers.get(name) {
            return Self::value(HostValue::JavaScriptFunction(wrapper.clone()));
        }
        Self::value(HostValue::Method(HostMethodRef {
            receiver,
            name: name.into(),
        }))
    }

    fn get(&self, receiver: HostObjectId, member: &str) -> HostResult {
        if receiver == 0 && member == "event" {
            return Self::value(self.current_event.borrow().clone().unwrap_or_default());
        }
        // cpp: webapi/dom_bindings.cc:269-300
        if let Some(record) = self.records.borrow().get(&receiver) {
            return record
                .get(member)
                .cloned()
                .map_or_else(Self::unhandled, Self::value);
        }
        if self.iterators.borrow().contains_key(&receiver) {
            return if member == "next" {
                self.method(receiver, member)
            } else {
                Self::unhandled()
            };
        }
        // cpp: webapi/dom_bindings.cc:278-285
        if is_event_handler(member) {
            return self
                .listeners
                .borrow()
                .get(&receiver)
                .and_then(|target| target.GetAttributeEventListener(&member[2..]))
                .and_then(|listener| Listener::From(listener.as_ref()).callback.Resolve())
                .map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |callback| Self::value(HostValue::JavaScriptFunction(callback)),
                );
        }
        if let Some(collection) = self.collections.borrow().get(&receiver) {
            if member == "item"
                || (member == "namedItem"
                    && collection.kind != "snapshot"
                    && collection.kind != "childNodes"
                    && collection.kind != "labels")
            {
                let class = if matches!(
                    collection.kind.as_str(),
                    "snapshot" | "childNodes" | "labels"
                ) {
                    "NodeList"
                } else {
                    "HTMLCollection"
                };
                if self
                    .method_wrappers
                    .contains_key(&format!("__{class}_{member}"))
                {
                    // DOM-only source wrappers live on the real prototype.
                    // Preserve Reflect getter/error frames and own/prototype
                    // overrides; argument coercion runs before host traversal.
                    return Self::unhandled();
                }
                // Preserve the raw host protocol before bootstrap installation.
                return Self::value(HostValue::Method(HostMethodRef {
                    receiver,
                    name: member.into(),
                }));
            }
            let nodes = self.collection_nodes(collection);
            if member == "length" {
                return Self::value(HostValue::Number(nodes.len() as f64));
            }
            // std::from_chars accepts decimal digits without leading +/-/space.
            if !member.is_empty() && member.bytes().all(|c| c.is_ascii_digit()) {
                if let Ok(index) = member.parse::<usize>() {
                    if let Some(&id) = nodes.get(index) {
                        return Self::value(HostValue::Object(HostObjectRef { id }));
                    }
                }
            }
            return Self::unhandled();
        }
        // cpp: webapi/dom_bindings.cc:301-322
        if receiver == 0 {
            // cpp: webapi/dom_bindings.cc:302-309
            if matches!(
                member,
                "addEventListener" | "removeEventListener" | "dispatchEvent"
            ) {
                if let Some(wrapper) = self.method_wrappers.get(&format!("__window_{member}")) {
                    return Self::value(HostValue::JavaScriptFunction(wrapper.clone()));
                }
                return Self::value(HostValue::Method(HostMethodRef {
                    receiver: 0,
                    name: member.into(),
                }));
            }
            if matches!(member, "__domInvoke" | "__domInstall" | "__domWrap") {
                return self.method(0, member);
            }
            if member == "document" {
                let root = self.document.borrow().GetDocument().Root();
                let id = self.document.borrow().GetDocument().Node(root).Id();
                return Self::value(HostValue::Object(HostObjectRef { id }));
            }
            if matches!(member, "window" | "self" | "parent" | "top") {
                return Self::value(HostValue::Object(HostObjectRef { id: 0 }));
            }
            if let Some(wrapper) = self.method_wrappers.get(member) {
                return Self::value(HostValue::JavaScriptFunction(wrapper.clone()));
            }
            return Self::unhandled();
        }
        if let Some(result) = self.GetBoundEvent(receiver, member) {
            return result;
        }
        let document = self.document.borrow();
        let document = document.GetDocument();
        let Some(index) = document.FindNodeById(receiver) else {
            return Self::type_error("host object no longer exists");
        };
        let node = document.Node(index);
        // cpp: webapi/dom_bindings.cc:384-385
        if matches!(member, "childNodes" | "children") {
            return Self::value(HostValue::Object(self.collection(receiver, member, "")));
        }
        if member == "options" && node.IsHTMLElement("select") {
            return Self::value(HostValue::Object(self.collection(receiver, "options", "")));
        }
        if is_method(node.Type(), member) {
            return self.method(receiver, member);
        }
        // cpp: webapi/dom_bindings.cc:360-388
        if node.Type() == DOMNodeType::kDocument {
            match member {
                "location" | "cookie" if !document.HasDOMWindow() && index == document.Root() => {
                    return Self::value(if member == "location" {
                        HostValue::Null(JavaScriptNull)
                    } else {
                        HostValue::String(String::new())
                    });
                }
                "URL" | "documentURI" => {
                    return Self::value(HostValue::String(document.DocumentURL().to_owned()));
                }
                "domain" => {
                    return Self::value(HostValue::String(if document.HasDOMWindow() {
                        document.SecurityContext().Origin().Domain()
                    } else {
                        String::new()
                    }));
                }
                "activeElement" => {
                    let focused = if let Some(read) = &self.services.host.read_document_focus {
                        read(document.RootHandle().Identity())
                            .and_then(|focus| focus.active_element)
                    } else {
                        self.focused_node
                    };
                    if let Some(id) = focused {
                        return Self::value(HostValue::Object(HostObjectRef { id }));
                    }
                    return self.get(receiver, "body");
                }
                "readyState" => {
                    return Self::value(HostValue::String(document.ReadyState().Name().into()));
                }
                "currentScript" => {
                    return if self.current_script == 0 {
                        Self::value(HostValue::Null(JavaScriptNull))
                    } else {
                        Self::value(HostValue::Object(HostObjectRef {
                            id: self.current_script,
                        }))
                    };
                }
                "defaultView" => {
                    return Self::value(
                        if document.HasDOMWindow()
                            && receiver == document.Node(document.Root()).Id()
                        {
                            HostValue::Object(HostObjectRef { id: 0 })
                        } else {
                            HostValue::Null(JavaScriptNull)
                        },
                    );
                }
                "fullscreenEnabled" => return Self::value(HostValue::Boolean(true)),
                "fullscreen" => {
                    return Self::value(HostValue::Boolean(document.HasFullscreenElements()));
                }
                "fullscreenElement" => {
                    return Self::node_reference(document, document.FullscreenElement());
                }
                "documentElement" | "scrollingElement" | "head" | "body" => {
                    let root = node
                        .Children()
                        .iter()
                        .copied()
                        .find(|&child| document.Node(child).Type() == DOMNodeType::kElement);
                    // Documents produced by this browser currently use the
                    // standards-mode scrolling model: the root element owns
                    // the viewport scroll offset.
                    if member == "documentElement" || member == "scrollingElement" {
                        return Self::node_reference(document, root);
                    }
                    if let Some(root) = root {
                        for &child in document.Node(root).Children() {
                            if document.Node(child).IsHTMLElement(member)
                                || (member == "body"
                                    && document.Node(child).IsHTMLElement("frameset"))
                            {
                                return Self::node_reference(document, Some(child));
                            }
                        }
                    }
                    return Self::node_reference(document, None);
                }
                _ => {}
            }
        }
        // cpp: webapi/dom_bindings.cc:391-411
        if node.IsHTMLElement("img") {
            let loader = document.EnsureHTMLImageLoader(index);
            let selection = document.ResponsiveImageSelectionFor(index);
            let image = loader.GetContent().and_then(|content| content.Metadata());
            match member {
                "complete" => {
                    return Self::value(HostValue::Boolean(loader.ImageComplete()));
                }
                "naturalWidth" => {
                    return Self::value(HostValue::Number(image.map_or(0.0, |image| {
                        image
                            .NaturalSizeForCandidate(
                                true,
                                selection.and_then(|selection| selection.density),
                            )
                            .0
                    })));
                }
                "naturalHeight" => {
                    return Self::value(HostValue::Number(image.map_or(0.0, |image| {
                        image
                            .NaturalSizeForCandidate(
                                true,
                                selection.and_then(|selection| selection.density),
                            )
                            .1
                    })));
                }
                "currentSrc" => {
                    return Self::value(HostValue::String(document.ImageCurrentSrc(index)));
                }
                _ => {}
            }
        }
        // cpp: webapi/dom_bindings.cc:412-442
        if node.Type() == DOMNodeType::kElement {
            match member {
                "className" | "id" => {
                    let attribute = if member == "className" { "class" } else { "id" };
                    let text = node
                        .FindAttribute(attribute)
                        .map_or("", |attribute| &attribute.value);
                    return Self::value(HostValue::String(text.into()));
                }
                "tagName" => {
                    let name = if node.Namespace() == DOMNamespace::kHTML {
                        node.Name().to_ascii_uppercase()
                    } else {
                        node.Name().into()
                    };
                    return Self::value(HostValue::String(name));
                }
                "localName" => return Self::value(HostValue::String(node.Name().into())),
                "namespaceURI" => {
                    return Self::value(match node.Namespace() {
                        DOMNamespace::kHTML => {
                            HostValue::String("http://www.w3.org/1999/xhtml".into())
                        }
                        DOMNamespace::kSVG => {
                            HostValue::String("http://www.w3.org/2000/svg".into())
                        }
                        DOMNamespace::kMathML => {
                            HostValue::String("http://www.w3.org/1998/Math/MathML".into())
                        }
                        DOMNamespace::kNone => HostValue::Null(JavaScriptNull),
                    });
                }
                _ => {}
            }
        }
        if member == "ownerDocument" {
            return Self::value(if node.Type() == DOMNodeType::kDocument {
                HostValue::Null(JavaScriptNull)
            } else {
                HostValue::Object(HostObjectRef {
                    id: node
                        .OwnerDocumentNode()
                        .expect("DOM node has no owner document")
                        .Id(),
                })
            });
        }
        if member == "content" && node.IsHTMLElement("template") {
            return Self::node_reference(
                document,
                Some(
                    document
                        .TemplateContents(index)
                        .expect("template has no contents"),
                ),
            );
        }
        // cpp: webapi/dom_bindings.cc:443-477
        match member {
            "nodeType" => Self::value(HostValue::Number(if node.IsCDATASection() {
                4.
            } else {
                node_type_number(node.Type())
            })),
            "nodeName" => Self::value(HostValue::String(if node.IsCDATASection() {
                "#cdata-section".into()
            } else {
                node_name(node.Type(), node.Name(), node.Namespace())
            })),
            "textContent"
                if matches!(
                    node.Type(),
                    DOMNodeType::kDocument | DOMNodeType::kDocumentType
                ) =>
            {
                Self::value(HostValue::Null(JavaScriptNull))
            }
            "textContent" => Self::value(HostValue::String(text_content(document, index))),
            "data" if is_character_data(node.Type()) => {
                Self::value(HostValue::String(node.Data().into()))
            }
            "target" if node.Type() == DOMNodeType::kProcessingInstruction => {
                Self::value(HostValue::String(node.Name().into()))
            }
            "nodeValue" => Self::value(if is_character_data(node.Type()) {
                HostValue::String(node.Data().into())
            } else {
                HostValue::Null(JavaScriptNull)
            }),
            "innerHTML"
                if matches!(
                    node.Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                ) =>
            {
                Self::value(HostValue::String(inner_html(document, index)))
            }
            "outerHTML" if node.Type() == DOMNodeType::kElement => {
                Self::value(HostValue::String(outer_html(document, index)))
            }
            "parentNode" => Self::node_reference(document, node.Parent()),
            "firstChild" => Self::node_reference(document, node.Children().first().copied()),
            "lastChild" => Self::node_reference(document, node.Children().last().copied()),
            "id" => Self::value(HostValue::String(
                node.FindAttribute("id")
                    .map_or("", |attribute| &attribute.value)
                    .into(),
            )),
            "value" if is_control(node) => {
                Self::value(HostValue::String16(document.ControlValueUTF16(index)))
            }
            "value" if node.IsHTMLElement("option") => {
                Self::value(HostValue::String(document.OptionValue(index)))
            }
            "selected" if node.IsHTMLElement("option") => {
                Self::value(HostValue::Boolean(document.OptionSelected(index)))
            }
            "checked" if node.IsHTMLElement("input") => {
                Self::value(HostValue::Boolean(document.ControlChecked(index)))
            }
            "indeterminate" if node.IsHTMLElement("input") => {
                Self::value(HostValue::Boolean(document.ControlIndeterminate(index)))
            }
            _ => Self::unhandled(),
        }
    }

    fn node_reference(document: &Document, node: Option<usize>) -> HostResult {
        Self::value(node.map_or(HostValue::Null(JavaScriptNull), |node| {
            HostValue::Object(HostObjectRef {
                id: document.Node(node).Id(),
            })
        }))
    }

    fn IsEventTargetReceiver(&self, receiver: u64) -> bool {
        receiver == 0
            || self.node(receiver).is_some()
            || self.native_abort.NativeTarget(receiver).is_some()
    }
    fn SetEventHandler(&self, receiver: u64, event_type: &str, arguments: &[HostValue]) {
        let target = self.ListenerTarget(receiver);
        let previous = target.GetAttributeEventListener(event_type);
        let listener = if let Some(HostValue::JavaScriptFunction(callback)) = arguments.first() {
            let slot = self.NewListenerSlot();
            Some(
                Rc::new(Listener::new(
                    receiver,
                    self.StoreListenerCallback(receiver, slot, callback),
                    slot,
                    true,
                    self.callback_context.borrow().clone(),
                    self.callback_context.clone(),
                    Rc::downgrade(&self.native_abort),
                ))
                    as interaction::dispatch::core::dom::events::event_listener::EventListenerRef,
            )
        } else {
            None
        };
        target.SetAttributeEventListener(event_type, listener);
        // Replacing the callback keeps the RegisteredEventListener's place
        // in every source snapshot while updating its one traced edge.
        if let Some(previous) = previous {
            self.RemoveListenerReference(Listener::From(previous.as_ref()));
        }
    }
    fn set(&mut self, receiver: HostObjectId, member: &str, arguments: &[HostValue]) -> HostResult {
        if member == "selected" {
            let Some(index) = self.node(receiver) else {
                return Self::type_error("Invalid option receiver");
            };
            if !self
                .document
                .borrow()
                .GetDocument()
                .Node(index)
                .IsHTMLElement("option")
            {
                return Self::type_error("selected requires an option");
            }
            let Some(HostValue::Boolean(value)) = arguments.first() else {
                return Self::type_error("selected requires a boolean");
            };
            (self.emit_mutation.borrow_mut())(&DOMMutation {
                mutation_type: DOMMutationType::kSetOptionSelected,
                target_node_id: receiver,
                bool_value: *value,
                ..Default::default()
            });
            return Self::value(HostValue::Boolean(*value));
        }
        // cpp: webapi/dom_bindings.cc:479-495
        if is_event_handler(member) {
            self.SetEventHandler(receiver, &member[2..], arguments);
            return HostResult::default();
        }
        // cpp: webapi/dom_bindings.cc:499-555
        if receiver == 0 {
            return Self::unhandled();
        }
        let Some(index) = self.node(receiver) else {
            return Self::unhandled();
        };
        let owner = self.document.borrow();
        let node = owner.GetDocument().Node(index);
        if member == "checked" {
            if !node.IsHTMLElement("input") {
                return Self::type_error("checked is only writable on input elements");
            }
            let Some(HostValue::Boolean(value)) = arguments.first() else {
                return Self::type_error("checked requires a boolean");
            };
            let value = *value;
            drop(owner);
            (self.emit_mutation.borrow_mut())(&DOMMutation {
                mutation_type: DOMMutationType::kSetControlChecked,
                target_node_id: receiver,
                bool_value: value,
                ..Default::default()
            });
            return Self::value(HostValue::Boolean(value));
        }
        if member == "indeterminate" {
            if !node.IsHTMLElement("input") {
                return Self::type_error("indeterminate is only writable on input elements");
            }
            let Some(HostValue::Boolean(value)) = arguments.first() else {
                return Self::type_error("indeterminate requires a boolean");
            };
            let value = *value;
            drop(owner);
            (self.emit_mutation.borrow_mut())(&DOMMutation {
                mutation_type: DOMMutationType::kSetControlIndeterminate,
                target_node_id: receiver,
                bool_value: value,
                ..Default::default()
            });
            return Self::value(HostValue::Boolean(value));
        }
        if !matches!(
            member,
            "textContent" | "data" | "nodeValue" | "innerHTML" | "value" | "id" | "className"
        ) {
            return Self::unhandled();
        }
        // value is a WebIDL DOMString, so isolated UTF-16 surrogates must
        // survive both the JavaScript bridge and the DOM mutation transaction.
        if member == "value" && is_control(node) {
            if let Some(HostValue::String16(units)) = arguments.first() {
                let units = units.clone();
                drop(owner);
                (self.emit_mutation.borrow_mut())(&DOMMutation {
                    mutation_type: DOMMutationType::kSetControlValue,
                    target_node_id: receiver,
                    utf16_value: Some(units.clone()),
                    ..Default::default()
                });
                return Self::value(HostValue::String16(units));
            }
        }
        let Some(value) = dom_string_argument(arguments.first()) else {
            return Self::type_error(&format!("{member} requires a string-convertible value"));
        };
        let option_value = member == "value" && node.IsHTMLElement("option");
        let mutation_type = match member {
            "textContent" | "data" | "nodeValue" => {
                if matches!(
                    node.Type(),
                    DOMNodeType::kDocument | DOMNodeType::kDocumentType
                ) || (member != "textContent" && !is_character_data(node.Type()))
                {
                    return Self::value(HostValue::String(value));
                }
                DOMMutationType::kSetTextContent
            }
            "innerHTML" => {
                if !matches!(
                    node.Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                ) {
                    return Self::type_error(
                        "innerHTML is only writable on elements and fragments",
                    );
                }
                DOMMutationType::kSetInnerHTML
            }
            "value" => {
                if option_value {
                    DOMMutationType::kSetAttribute
                } else {
                    if !is_control(node) {
                        return Self::type_error("value is only writable on form controls");
                    }
                    DOMMutationType::kSetControlValue
                }
            }
            "id" | "className" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("id is only writable on elements");
                }
                DOMMutationType::kSetAttribute
            }
            _ => return Self::type_error("property is not writable"),
        };
        drop(owner);
        let mut trace = browser_tracing::span("dom", "DOMMutationHost");
        trace.set("mutation_type", mutation_type as u8 as f64);
        (self.emit_mutation.borrow_mut())(&DOMMutation {
            mutation_type,
            target_node_id: receiver,
            name: match member {
                "id" => "id",
                "className" => "class",
                "value" if option_value => "value",
                _ => "",
            }
            .into(),
            value: value.clone(),
            utf16_value: match arguments.first() {
                Some(HostValue::String16(units)) => Some(units.clone()),
                _ => None,
            },
            ..Default::default()
        });
        Self::value(HostValue::String(value))
    }

    fn call(
        &mut self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> HostResult {
        if member == "__formElements" {
            let valid = {
                let owner = self.document.borrow();
                let document = owner.GetDocument();
                document
                    .FindNodeById(receiver)
                    .is_some_and(|i| document.Node(i).IsHTMLElement("form"))
            };
            return if valid {
                Self::value(HostValue::Object(self.collection(
                    receiver,
                    "formControls",
                    "",
                )))
            } else {
                Self::type_error("Illegal invocation")
            };
        }
        if matches!(
            member,
            "querySelector" | "querySelectorAll" | "matches" | "closest"
        ) {
            if let Err(error) = interaction::adapter::page_form_validation::RefreshControlValidity(
                &self.document,
                &self.form_validation_state,
            ) {
                return Self::type_error(&format!("Form validation service required: {error:?}"));
            }
        }
        if member.starts_with("__internals") {
            return self.CallElementInternals(receiver, member, arguments);
        }
        if matches!(
            member,
            "__controlValidity" | "__validityBrand" | "setCustomValidity"
        ) {
            return self.CallFormValidation(receiver, member, arguments);
        }
        if matches!(
            member,
            "__domExceptionName" | "__domExceptionMessage" | "__domExceptionCode"
        ) {
            if self.native_record_classes.borrow().get(&receiver) != Some(&"DOMException") {
                return Self::type_error("Illegal invocation");
            }
            let field = match member {
                "__domExceptionName" => "name",
                "__domExceptionMessage" => "message",
                _ => "code",
            };
            return self.get(receiver, field);
        }
        if receiver == 0 && member == "__createDOMException" {
            let message = match arguments.first() {
                Some(HostValue::String(s)) => s.as_str(),
                _ => return Self::type_error("DOMException message conversion required"),
            };
            let name = match arguments.get(1) {
                Some(HostValue::String(s)) => s.as_str(),
                _ => return Self::type_error("DOMException name conversion required"),
            };
            return Self::value(self.CreateNativeDOMException(name, message));
        }
        if let Some(context) = self.webgl_contexts.borrow().get(&receiver).cloned() {
            let number = |index: usize| match arguments.get(index) {
                Some(HostValue::Number(value)) => Some(*value),
                _ => None,
            };
            let object = |index: usize| number(index).map(|value| value as u32);
            return match member {
                "createShader" => object(0)
                    .and_then(|kind| context.CreateShader(kind))
                    .map_or_else(
                        || Self::value(HostValue::Null(JavaScriptNull)),
                        |id| Self::value(HostValue::Number(f64::from(id))),
                    ),
                "shaderSource" => match (object(0), arguments.get(1)) {
                    (Some(shader), Some(HostValue::String(source))) => {
                        context.ShaderSource(shader, source.clone());
                        HostResult::default()
                    }
                    _ => Self::type_error("shaderSource requires shader and source"),
                },
                "compileShader" => match object(0) {
                    Some(shader) => {
                        context.CompileShader(shader);
                        HostResult::default()
                    }
                    None => Self::type_error("compileShader requires shader"),
                },
                "getShaderParameter" => match (object(0), object(1)) {
                    (Some(shader), Some(parameter)) => Self::value(HostValue::Boolean(
                        context.ShaderParameter(shader, parameter),
                    )),
                    _ => Self::type_error("getShaderParameter requires shader and pname"),
                },
                "getShaderInfoLog" => match object(0) {
                    Some(shader) => Self::value(HostValue::String(context.ShaderInfoLog(shader))),
                    None => Self::type_error("getShaderInfoLog requires shader"),
                },
                "createProgram" => {
                    Self::value(HostValue::Number(f64::from(context.CreateProgram())))
                }
                "attachShader" => match (object(0), object(1)) {
                    (Some(program), Some(shader)) => {
                        context.AttachShader(program, shader);
                        HostResult::default()
                    }
                    _ => Self::type_error("attachShader requires program and shader"),
                },
                "linkProgram" => match object(0) {
                    Some(program) => {
                        context.LinkProgram(program);
                        HostResult::default()
                    }
                    None => Self::type_error("linkProgram requires program"),
                },
                "getProgramParameter" => match (object(0), object(1)) {
                    (Some(program), Some(parameter))
                        if matches!(
                            parameter,
                            webgl::ACTIVE_UNIFORMS | webgl::ACTIVE_ATTRIBUTES
                        ) =>
                    {
                        Self::value(HostValue::Number(f64::from(
                            context.ProgramParameterInteger(program, parameter),
                        )))
                    }
                    (Some(program), Some(parameter)) => Self::value(HostValue::Boolean(
                        context.ProgramParameter(program, parameter),
                    )),
                    _ => Self::type_error("getProgramParameter requires program and pname"),
                },
                "getProgramInfoLog" => match object(0) {
                    Some(program) => {
                        Self::value(HostValue::String(context.ProgramInfoLog(program)))
                    }
                    None => Self::type_error("getProgramInfoLog requires program"),
                },
                "deleteShader" => match object(0) {
                    Some(shader) => {
                        context.DeleteShader(shader);
                        HostResult::default()
                    }
                    None => Self::type_error("deleteShader requires shader"),
                },
                "getActiveUniform" | "getActiveAttrib" => match (object(0), object(1)) {
                    (Some(program), Some(index)) => {
                        let info = if member == "getActiveUniform" {
                            context.ActiveUniform(program, index)
                        } else {
                            context.ActiveAttrib(program, index)
                        };
                        info.map_or_else(
                            || Self::value(HostValue::Null(JavaScriptNull)),
                            |info| {
                                Self::value(HostValue::Record(vec![
                                    ("name".into(), HostValue::String(info.name)),
                                    ("size".into(), HostValue::Number(f64::from(info.size))),
                                    ("type".into(), HostValue::Number(f64::from(info.kind))),
                                ]))
                            },
                        )
                    }
                    _ => Self::type_error("active-info query requires program and index"),
                },
                "getUniformLocation" => match (object(0), arguments.get(1)) {
                    (Some(program), Some(HostValue::String(name))) => {
                        context.GetUniformLocation(program, name).map_or_else(
                            || Self::value(HostValue::Null(JavaScriptNull)),
                            |location| Self::value(HostValue::Number(f64::from(location))),
                        )
                    }
                    _ => Self::type_error("getUniformLocation requires program and name"),
                },
                "useProgram" => match object(0) {
                    Some(program) => {
                        context.UseProgram(program);
                        HostResult::default()
                    }
                    None => Self::type_error("useProgram requires program"),
                },
                "getAttribLocation" => match (object(0), arguments.get(1)) {
                    (Some(program), Some(HostValue::String(name))) => Self::value(
                        HostValue::Number(f64::from(context.GetAttribLocation(program, name))),
                    ),
                    _ => Self::type_error("getAttribLocation requires program and name"),
                },
                "createBuffer" => Self::value(HostValue::Number(f64::from(context.CreateBuffer()))),
                "bindBuffer" => match (object(0), arguments.get(1)) {
                    (Some(target), Some(HostValue::Number(buffer))) => {
                        context.BindBuffer(target, Some(*buffer as u32));
                        HostResult::default()
                    }
                    (Some(target), Some(HostValue::Null(_))) => {
                        context.BindBuffer(target, None);
                        HostResult::default()
                    }
                    _ => Self::type_error("bindBuffer requires target and buffer"),
                },
                "bufferData" => match (object(0), arguments.get(1), object(2)) {
                    (Some(target), Some(HostValue::Bytes(bytes)), Some(usage)) => {
                        context.BufferData(target, bytes, usage);
                        HostResult::default()
                    }
                    _ => Self::type_error("bufferData requires target, bytes and usage"),
                },
                "createTexture" => {
                    Self::value(HostValue::Number(f64::from(context.CreateTexture())))
                }
                "bindTexture" => match (object(0), arguments.get(1)) {
                    (Some(target), Some(HostValue::Number(texture))) => {
                        context.BindTexture(target, Some(*texture as u32));
                        HostResult::default()
                    }
                    (Some(target), Some(HostValue::Null(_))) => {
                        context.BindTexture(target, None);
                        HostResult::default()
                    }
                    _ => Self::type_error("bindTexture requires target and texture"),
                },
                "texParameteri" => match (object(0), object(1), number(2)) {
                    (Some(target), Some(name), Some(value)) => {
                        context.TexParameteri(target, name, value as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("texParameteri requires target, pname and value"),
                },
                "texImage2D" => match (
                    object(0),
                    number(1),
                    number(2),
                    number(3),
                    number(4),
                    number(5),
                    object(6),
                    object(7),
                    arguments.get(8),
                ) {
                    (
                        Some(target),
                        Some(level),
                        Some(internal_format),
                        Some(width),
                        Some(height),
                        Some(border),
                        Some(format),
                        Some(kind),
                        Some(pixels),
                    ) => {
                        let pixels = match pixels {
                            HostValue::Bytes(bytes) => Some(bytes.as_ref()),
                            HostValue::Null(_) => None,
                            _ => {
                                return Self::type_error("texImage2D pixels must be bytes or null");
                            }
                        };
                        context.TexImage2D(
                            target,
                            level as i32,
                            internal_format as i32,
                            width as i32,
                            height as i32,
                            border as i32,
                            format,
                            kind,
                            pixels,
                        );
                        HostResult::default()
                    }
                    _ => Self::type_error("texImage2D requires nine arguments"),
                },
                "enableVertexAttribArray" => match object(0) {
                    Some(index) => {
                        context.EnableVertexAttribArray(index);
                        HostResult::default()
                    }
                    None => Self::type_error("enableVertexAttribArray requires index"),
                },
                "vertexAttrib2fv" => match (object(0), number(1), number(2)) {
                    (Some(index), Some(x), Some(y)) => {
                        context.VertexAttrib2fv(index, &[x as f32, y as f32]);
                        HostResult::default()
                    }
                    _ => Self::type_error("vertexAttrib2fv requires index and two values"),
                },
                "createVertexArray" => {
                    Self::value(HostValue::Number(f64::from(context.CreateVertexArray())))
                }
                "bindVertexArray" => match arguments.first() {
                    Some(HostValue::Number(vertex_array)) => {
                        context.BindVertexArray(Some(*vertex_array as u32));
                        HostResult::default()
                    }
                    Some(HostValue::Null(_)) => {
                        context.BindVertexArray(None);
                        HostResult::default()
                    }
                    _ => Self::type_error("bindVertexArray requires a vertex array or null"),
                },
                "vertexAttribPointer" => match (
                    object(0),
                    number(1),
                    object(2),
                    arguments.get(3),
                    number(4),
                    number(5),
                ) {
                    (
                        Some(index),
                        Some(size),
                        Some(kind),
                        Some(HostValue::Boolean(normalized)),
                        Some(stride),
                        Some(offset),
                    ) => {
                        context.VertexAttribPointer(
                            index,
                            size as i32,
                            kind,
                            *normalized,
                            stride as i32,
                            offset as usize,
                        );
                        HostResult::default()
                    }
                    _ => Self::type_error("vertexAttribPointer requires six arguments"),
                },
                "viewport" => match (number(0), number(1), number(2), number(3)) {
                    (Some(x), Some(y), Some(width), Some(height)) => {
                        context.Viewport(x as i32, y as i32, width as i32, height as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("viewport requires four numbers"),
                },
                "clearColor" => match (number(0), number(1), number(2), number(3)) {
                    (Some(r), Some(g), Some(b), Some(a)) => {
                        context.ClearColor(r as f32, g as f32, b as f32, a as f32);
                        HostResult::default()
                    }
                    _ => Self::type_error("clearColor requires four numbers"),
                },
                "clearDepth" => match number(0) {
                    Some(depth) => {
                        context.ClearDepth(depth);
                        HostResult::default()
                    }
                    None => Self::type_error("clearDepth requires a number"),
                },
                "clearStencil" => match number(0) {
                    Some(stencil) => {
                        context.ClearStencil(stencil as i32);
                        HostResult::default()
                    }
                    None => Self::type_error("clearStencil requires a number"),
                },
                "enable" | "disable" | "depthFunc" | "frontFace" | "cullFace" => match object(0) {
                    Some(value) => {
                        match member {
                            "enable" => context.Enable(value),
                            "disable" => context.Disable(value),
                            "depthFunc" => context.DepthFunc(value),
                            "frontFace" => context.FrontFace(value),
                            "cullFace" => context.CullFace(value),
                            _ => unreachable!(),
                        }
                        HostResult::default()
                    }
                    None => Self::type_error("WebGL state call requires an enum"),
                },
                "scissor" => match (number(0), number(1), number(2), number(3)) {
                    (Some(x), Some(y), Some(width), Some(height)) => {
                        context.Scissor(x as i32, y as i32, width as i32, height as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("scissor requires four numbers"),
                },
                "depthMask" => match arguments.first() {
                    Some(HostValue::Boolean(enabled)) => {
                        context.DepthMask(*enabled);
                        HostResult::default()
                    }
                    _ => Self::type_error("depthMask requires a boolean"),
                },
                "colorMask" => match arguments {
                    [HostValue::Boolean(red), HostValue::Boolean(green), HostValue::Boolean(blue), HostValue::Boolean(alpha), ..] =>
                    {
                        context.ColorMask(*red, *green, *blue, *alpha);
                        HostResult::default()
                    }
                    _ => Self::type_error("colorMask requires four booleans"),
                },
                "blendEquation" => match object(0) {
                    Some(mode) => {
                        context.BlendEquation(mode);
                        HostResult::default()
                    }
                    None => Self::type_error("blendEquation requires a mode"),
                },
                "blendFuncSeparate" => match (object(0), object(1), object(2), object(3)) {
                    (
                        Some(source_rgb),
                        Some(destination_rgb),
                        Some(source_alpha),
                        Some(destination_alpha),
                    ) => {
                        context.BlendFuncSeparate(
                            source_rgb,
                            destination_rgb,
                            source_alpha,
                            destination_alpha,
                        );
                        HostResult::default()
                    }
                    _ => Self::type_error("blendFuncSeparate requires four enums"),
                },
                "bindFramebuffer" => match (object(0), arguments.get(1)) {
                    (Some(_), Some(HostValue::Null(_))) => {
                        context.BindDefaultFramebuffer();
                        HostResult::default()
                    }
                    _ => Self::type_error("only the default framebuffer is supported"),
                },
                "drawBuffers" => match object(0) {
                    Some(_) => {
                        context.DrawDefaultBuffer();
                        HostResult::default()
                    }
                    None => Self::type_error("drawBuffers requires a buffer enum"),
                },
                "clear" => match object(0) {
                    Some(mask) => {
                        context.Clear(mask);
                        self.NotifyCanvasDirty(receiver);
                        HostResult::default()
                    }
                    None => Self::type_error("clear requires mask"),
                },
                "drawArrays" => match (object(0), number(1), number(2)) {
                    (Some(mode), Some(first), Some(count)) => {
                        context.DrawArrays(mode, first as i32, count as i32);
                        self.NotifyCanvasDirty(receiver);
                        HostResult::default()
                    }
                    _ => Self::type_error("drawArrays requires mode, first and count"),
                },
                "drawElements" => match (object(0), number(1), object(2), number(3)) {
                    (Some(mode), Some(count), Some(kind), Some(offset)) => {
                        context.DrawElements(mode, count as i32, kind, offset as usize);
                        self.NotifyCanvasDirty(receiver);
                        HostResult::default()
                    }
                    _ => Self::type_error("drawElements requires mode, count, type and offset"),
                },
                "uniformMatrix2fv" | "uniformMatrix3fv" | "uniformMatrix4fv" => {
                    match (object(0), arguments.get(1), arguments.get(2)) {
                        (
                            Some(location),
                            Some(HostValue::Boolean(transpose)),
                            Some(HostValue::Bytes(bytes)),
                        ) => match webgl_f32_values(bytes) {
                            Some(values) => {
                                match member {
                                    "uniformMatrix2fv" => {
                                        context.UniformMatrix2fv(location, *transpose, &values)
                                    }
                                    "uniformMatrix3fv" => {
                                        context.UniformMatrix3fv(location, *transpose, &values)
                                    }
                                    "uniformMatrix4fv" => {
                                        context.UniformMatrix4fv(location, *transpose, &values)
                                    }
                                    _ => unreachable!(),
                                }
                                HostResult::default()
                            }
                            None => Self::type_error("uniform matrix data is not Float32-aligned"),
                        },
                        _ => Self::type_error(
                            "uniform matrix call requires location, transpose and values",
                        ),
                    }
                }
                "uniform1fv" | "uniform2fv" | "uniform3fv" | "uniform4fv" => {
                    match (object(0), arguments.get(1)) {
                        (Some(location), Some(HostValue::Bytes(bytes))) => {
                            match webgl_f32_values(bytes) {
                                Some(values) => {
                                    match member {
                                        "uniform1fv" => context.Uniform1fv(location, &values),
                                        "uniform2fv" => context.Uniform2fv(location, &values),
                                        "uniform3fv" => context.Uniform3fv(location, &values),
                                        "uniform4fv" => context.Uniform4fv(location, &values),
                                        _ => unreachable!(),
                                    }
                                    HostResult::default()
                                }
                                None => Self::type_error("uniform data is not Float32-aligned"),
                            }
                        }
                        _ => Self::type_error("uniform vector call requires location and values"),
                    }
                }
                "uniform2f" => match (object(0), number(1), number(2)) {
                    (Some(location), Some(x), Some(y)) => {
                        context.Uniform2f(location, x as f32, y as f32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform2f requires location and two values"),
                },
                "uniform3f" => match (object(0), number(1), number(2), number(3)) {
                    (Some(location), Some(x), Some(y), Some(z)) => {
                        context.Uniform3f(location, x as f32, y as f32, z as f32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform3f requires location and three values"),
                },
                "uniform4f" => match (object(0), number(1), number(2), number(3), number(4)) {
                    (Some(location), Some(x), Some(y), Some(z), Some(w)) => {
                        context.Uniform4f(location, x as f32, y as f32, z as f32, w as f32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform4f requires location and four values"),
                },
                "uniform1f" => match (object(0), number(1)) {
                    (Some(location), Some(value)) => {
                        context.Uniform1f(location, value as f32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform1f requires location and value"),
                },
                "uniform1i" => match (object(0), number(1)) {
                    (Some(location), Some(value)) => {
                        context.Uniform1i(location, value as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform1i requires location and value"),
                },
                "uniform2i" => match (object(0), number(1), number(2)) {
                    (Some(location), Some(x), Some(y)) => {
                        context.Uniform2i(location, x as i32, y as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform2i requires location and two values"),
                },
                "uniform3i" => match (object(0), number(1), number(2), number(3)) {
                    (Some(location), Some(x), Some(y), Some(z)) => {
                        context.Uniform3i(location, x as i32, y as i32, z as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform3i requires location and three values"),
                },
                "uniform4i" => match (object(0), number(1), number(2), number(3), number(4)) {
                    (Some(location), Some(x), Some(y), Some(z), Some(w)) => {
                        context.Uniform4i(location, x as i32, y as i32, z as i32, w as i32);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform4i requires location and four values"),
                },
                "uniform1iv" | "uniform2iv" | "uniform3iv" | "uniform4iv" => {
                    match (object(0), arguments.get(1)) {
                        (Some(location), Some(HostValue::Bytes(bytes))) => {
                            match webgl_i32_values(bytes) {
                                Some(values) => {
                                    match member {
                                        "uniform1iv" => context.Uniform1iv(location, &values),
                                        "uniform2iv" => context.Uniform2iv(location, &values),
                                        "uniform3iv" => context.Uniform3iv(location, &values),
                                        "uniform4iv" => context.Uniform4iv(location, &values),
                                        _ => unreachable!(),
                                    }
                                    HostResult::default()
                                }
                                None => Self::type_error("uniform data is not Int32-aligned"),
                            }
                        }
                        _ => Self::type_error("uniform vector call requires location and values"),
                    }
                }
                "uniform1ui" => match (object(0), object(1)) {
                    (Some(location), Some(value)) => {
                        context.Uniform1ui(location, value);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform1ui requires location and value"),
                },
                "uniform2ui" => match (object(0), object(1), object(2)) {
                    (Some(location), Some(x), Some(y)) => {
                        context.Uniform2ui(location, x, y);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform2ui requires location and two values"),
                },
                "uniform3ui" => match (object(0), object(1), object(2), object(3)) {
                    (Some(location), Some(x), Some(y), Some(z)) => {
                        context.Uniform3ui(location, x, y, z);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform3ui requires location and three values"),
                },
                "uniform4ui" => match (object(0), object(1), object(2), object(3), object(4)) {
                    (Some(location), Some(x), Some(y), Some(z), Some(w)) => {
                        context.Uniform4ui(location, x, y, z, w);
                        HostResult::default()
                    }
                    _ => Self::type_error("uniform4ui requires location and four values"),
                },
                "uniform1uiv" | "uniform2uiv" | "uniform3uiv" | "uniform4uiv" => {
                    match (object(0), arguments.get(1)) {
                        (Some(location), Some(HostValue::Bytes(bytes))) => {
                            match webgl_u32_values(bytes) {
                                Some(values) => {
                                    match member {
                                        "uniform1uiv" => context.Uniform1uiv(location, &values),
                                        "uniform2uiv" => context.Uniform2uiv(location, &values),
                                        "uniform3uiv" => context.Uniform3uiv(location, &values),
                                        "uniform4uiv" => context.Uniform4uiv(location, &values),
                                        _ => unreachable!(),
                                    }
                                    HostResult::default()
                                }
                                None => Self::type_error("uniform data is not Uint32-aligned"),
                            }
                        }
                        _ => Self::type_error("uniform vector call requires location and values"),
                    }
                }
                "getShaderPrecisionFormat" => Self::value(HostValue::Record(vec![
                    ("rangeMin".into(), HostValue::Number(127.0)),
                    ("rangeMax".into(), HostValue::Number(127.0)),
                    ("precision".into(), HostValue::Number(23.0)),
                ])),
                "getParameter" => match object(0) {
                    Some(webgl::VENDOR) => Self::value(HostValue::String("WebKit".into())),
                    Some(webgl::RENDERER) => Self::value(HostValue::String("WebKit WebGL".into())),
                    Some(webgl::VERSION) => Self::value(HostValue::String(
                        if context.ContextType() == canvas::CanvasContextType::WebGL2 {
                            "WebGL 2.0 (OpenGL ES 3.0 Chromium)"
                        } else {
                            "WebGL 1.0 (OpenGL ES 2.0 Chromium)"
                        }
                        .into(),
                    )),
                    Some(webgl::SHADING_LANGUAGE_VERSION) => Self::value(HostValue::String(
                        if context.ContextType() == canvas::CanvasContextType::WebGL2 {
                            "WebGL GLSL ES 3.00 (OpenGL ES GLSL ES 3.0 Chromium)"
                        } else {
                            "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"
                        }
                        .into(),
                    )),
                    Some(parameter @ (webgl::VIEWPORT | webgl::SCISSOR_BOX)) => {
                        let bytes = context
                            .IntegerParameters(parameter, 4)
                            .into_iter()
                            .flat_map(i32::to_ne_bytes)
                            .collect::<Vec<_>>();
                        Self::value(HostValue::Bytes(bytes.into()))
                    }
                    Some(
                        parameter @ (webgl::ALIASED_POINT_SIZE_RANGE
                        | webgl::ALIASED_LINE_WIDTH_RANGE),
                    ) => {
                        let bytes = context
                            .FloatParameters(parameter, 2)
                            .into_iter()
                            .flat_map(f32::to_ne_bytes)
                            .collect::<Vec<_>>();
                        Self::value(HostValue::Bytes(bytes.into()))
                    }
                    Some(parameter) => Self::value(HostValue::Number(f64::from(
                        context.IntegerParameter(parameter),
                    ))),
                    None => Self::type_error("getParameter requires pname"),
                },
                "getExtension" => Self::value(HostValue::Null(JavaScriptNull)),
                "getContextAttributes" => Self::value(HostValue::Record(vec![
                    ("alpha".into(), HostValue::Boolean(true)),
                    ("depth".into(), HostValue::Boolean(true)),
                    ("stencil".into(), HostValue::Boolean(true)),
                    ("antialias".into(), HostValue::Boolean(true)),
                    ("premultipliedAlpha".into(), HostValue::Boolean(true)),
                    ("preserveDrawingBuffer".into(), HostValue::Boolean(false)),
                    // canvas_context_creation_attributes_core.h:34-39 keeps
                    // the requested value even if backend admission disables
                    // the effective low-latency mode used by input routing.
                    (
                        "desynchronized".into(),
                        HostValue::Boolean(context.CreationAttributes().desynchronized_specified),
                    ),
                    (
                        "failIfMajorPerformanceCaveat".into(),
                        HostValue::Boolean(false),
                    ),
                    (
                        "powerPreference".into(),
                        HostValue::String("default".into()),
                    ),
                ])),
                "getError" => Self::value(HostValue::Number(f64::from(context.GetError()))),
                "__snapshot" => context.Snapshot().map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |snapshot| Self::value(HostValue::Bytes(snapshot.pixels.into())),
                ),
                _ => Self::unhandled(),
            };
        }
        // Opt-in attribution of read/query work hidden inside JS callbacks.
        // Recording observes this native binding only; it never calls JS.
        enum CallProfile {
            Off,
            All,
            Layout,
        }
        static PROFILE_CALLS: std::sync::OnceLock<CallProfile> = std::sync::OnceLock::new();
        let mode = PROFILE_CALLS.get_or_init(|| {
            match std::env::var("BROWSER_PROFILE_DOM_CALLS").as_deref() {
                Ok("1") => CallProfile::All,
                Ok("layout") => CallProfile::Layout,
                _ => CallProfile::Off,
            }
        });
        let selected = matches!(mode, CallProfile::All)
            || (matches!(mode, CallProfile::Layout)
                && matches!(
                    member,
                    "clientRects" | "metric" | "scrollOffset" | "computedStyle"
                ));
        let _profile = selected.then(|| {
            let trace = browser_tracing::span("dom", "DOM.Call");
            if browser_tracing::enabled() {
                let name = match (member, arguments) {
                    (
                        "__domAttributeGetter",
                        [_, HostValue::String(interface), HostValue::String(attribute), ..],
                    ) => format!("{interface}.{attribute}"),
                    (
                        "__domAttributeSet",
                        [HostValue::String(interface), HostValue::String(attribute), ..],
                    ) => format!("{interface}.{attribute}"),
                    (
                        "metric" | "computedStyle" | "scrollOffset",
                        [HostValue::String(property), ..],
                    ) => format!("{member}.{property}"),
                    _ => member.to_owned(),
                };
                browser_tracing::instant_with_text(
                    "dom",
                    "DOM.CallName",
                    &name,
                    &[("receiver", receiver as f64)],
                );
            }
            trace
        });
        if member == "__domAttributeGetter" {
            return self.AttributeGetter(arguments);
        }
        if matches!(member, "__domHasInterface" | "__domAttributeSet") {
            return self.AttributeCall(receiver, member, arguments);
        }
        if member == "__domCollectionBrand" {
            return self.collections.borrow().get(&receiver).map_or_else(
                || Self::type_error("Illegal invocation"),
                |collection| {
                    Self::value(HostValue::String(
                        if collection.kind == "options" {
                            "HTMLOptionsCollection"
                        } else if matches!(
                            collection.kind.as_str(),
                            "snapshot" | "childNodes" | "labels"
                        ) {
                            "NodeList"
                        } else {
                            "HTMLCollection"
                        }
                        .into(),
                    ))
                },
            );
        }
        if let Some(result) = self.CallViewportScroll(receiver, member, arguments) {
            return result;
        }
        if let Some(result) = self.CallDOMService(receiver, member, arguments) {
            return result;
        }
        if matches!(
            member,
            "__eventTargetBrand" | "__eventTargetHandlerGet" | "__eventTargetHandlerSet"
        ) {
            if !self.IsEventTargetReceiver(receiver) {
                return Self::type_error("Illegal EventTarget invocation");
            }
            if member == "__eventTargetBrand" {
                return Self::value(HostValue::Boolean(true));
            }
            let Some(HostValue::String(kind)) = arguments.first() else {
                return Self::type_error("EventHandler requires an event type");
            };
            if member == "__eventTargetHandlerSet" {
                self.SetEventHandler(receiver, kind, &arguments[1..]);
                return HostResult::default();
            }
            return self
                .ListenerTarget(receiver)
                .GetAttributeEventListener(kind)
                .and_then(|listener| Listener::From(listener.as_ref()).callback.Resolve())
                .map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |callback| Self::value(HostValue::JavaScriptFunction(callback)),
                );
        }
        if member == "__abortGetAttributeListener" {
            return self.get(receiver, "onabort");
        }
        if member == "__abortSetAttributeListener" {
            return self.set(receiver, "onabort", arguments);
        }
        // cpp: webapi/dom_bindings.cc:743-775
        if matches!(member, "addEventListener" | "removeEventListener") {
            if !self.IsEventTargetReceiver(receiver) {
                return Self::type_error("Illegal EventTarget invocation");
            }
            let Some(HostValue::String(kind)) = arguments.first() else {
                return Self::type_error("EventTarget requires an event type");
            };
            let callback = match arguments.get(1) {
                None | Some(HostValue::Null(_)) | Some(HostValue::Undefined(_)) => {
                    return HostResult::default()
                }
                Some(HostValue::JavaScriptFunction(callback)) => callback,
                _ => return Self::type_error("EventTarget callback conversion required"),
            };
            let capture = match arguments.get(2) {
                None => false,
                Some(HostValue::Boolean(capture)) => *capture,
                _ => return Self::type_error("listener options currently require a boolean"),
            };
            let target = self.ListenerTarget(receiver);
            let reference_slot = self.NewListenerSlot();
            let listener = Rc::new(Listener::new(
                receiver,
                ListenerCallback::Rooted(callback.clone()),
                reference_slot,
                false,
                self.callback_context.borrow().clone(),
                self.callback_context.clone(),
                Rc::downgrade(&self.native_abort),
            ));
            if member == "removeEventListener" {
                target.removeEventListener(kind, Some(listener.as_ref()), capture);
            } else {
                use interaction::dispatch::core::dom::events::add_event_listener_options_resolved::AddEventListenerOptionsResolved;
                let mut options = AddEventListenerOptionsResolved::new();
                options.setCapture(capture);
                match arguments.get(3) {
                    None => {}
                    Some(HostValue::Boolean(once)) => options.setOnce(*once),
                    _ => return Self::type_error("bad_variant_access"),
                }
                match arguments.get(4) {
                    None | Some(HostValue::Undefined(_)) => {}
                    Some(HostValue::Boolean(passive)) => options.setPassive(*passive),
                    _ => return Self::type_error("bad_variant_access"),
                }
                let listener = Rc::new(Listener::new(
                    receiver,
                    self.StoreListenerCallback(receiver, reference_slot, callback),
                    reference_slot,
                    false,
                    self.callback_context.borrow().clone(),
                    self.callback_context.clone(),
                    Rc::downgrade(&self.native_abort),
                ));
                match arguments.get(5) {
                    None | Some(HostValue::Undefined(_)) | Some(HostValue::Null(_)) => {}
                    Some(signal) => {
                        let Some(signal) = self.native_abort.SignalFromValue(signal) else {
                            self.RemoveListenerReference(&listener);
                            return Self::type_error("signal is not an AbortSignal");
                        };
                        options.setSignal(signal);
                    }
                }
                if !target.addEventListenerWithOptions(kind, Some(listener.clone()), options) {
                    self.RemoveListenerReference(&listener);
                }
            }
            return HostResult::default();
        }
        // cpp: webapi/dom_bindings.cc:650-665
        if receiver == 0 && member == "cssSupports" {
            let (Some(HostValue::String(property)), Some(HostValue::String(value))) =
                (arguments.first(), arguments.get(1))
            else {
                return Self::value(HostValue::Boolean(false));
            };
            let supported = !value.is_empty()
                && match property.as_str() {
                    "color" | "background-color" | "border-color" => {
                        value == "currentcolor"
                            || layoutng_assembly::css_color_parser::ParseCSSColor(value).is_some()
                    }
                    "width" | "height" | "font-size" | "margin-left" | "padding-left" => {
                        value == "auto"
                            || css_parser::length_percentage_parser::ParseLengthPercentage(
                                value, 16.0,
                            )
                            .is_some()
                    }
                    "display" => matches!(
                        value.as_str(),
                        "none"
                            | "block"
                            | "inline"
                            | "flex"
                            | "grid"
                            | "inline-block"
                            | "contents"
                            | "table"
                    ),
                    _ => false,
                };
            return Self::value(HostValue::Boolean(supported));
        }
        // cpp: webapi/dom_bindings.cc:701-742
        if let Some(state) = self.iterators.borrow_mut().get_mut(&receiver) {
            if member == "@@iterator" {
                return Self::value(HostValue::Object(HostObjectRef { id: receiver }));
            }
            if member == "next" {
                let collections = self.collections.borrow();
                let nodes = self.collection_nodes(&collections[&state.collection]);
                let done = state.done || state.index >= nodes.len();
                state.done = done;
                let value = if done {
                    HostValue::default()
                } else {
                    let id = nodes[state.index];
                    state.index += 1;
                    HostValue::Object(HostObjectRef { id })
                };
                let id = self.allocate_collection_id();
                self.StoreRecord(
                    id,
                    HashMap::from([
                        ("done".into(), HostValue::Boolean(done)),
                        ("value".into(), value),
                    ]),
                );
                return Self::value(HostValue::Object(HostObjectRef { id }));
            }
            return Self::type_error("unknown iterator method");
        }
        if let Some(collection) = self.collections.borrow().get(&receiver) {
            if member == "@@iterator" {
                let id = self.allocate_collection_id();
                self.host_graph.Iterator(id, receiver);
                self.iterators.borrow_mut().insert(
                    id,
                    CollectionIterator {
                        collection: receiver,
                        index: 0,
                        done: false,
                    },
                );
                return Self::value(HostValue::Object(HostObjectRef { id }));
            }
            let nodes = self.collection_nodes(collection);
            if member == "item" {
                if let Some(HostValue::Number(index)) = arguments.first() {
                    if *index >= 0.0 && *index < nodes.len() as f64 {
                        return Self::value(HostValue::Object(HostObjectRef {
                            id: nodes[*index as usize],
                        }));
                    }
                }
                return Self::value(HostValue::Null(JavaScriptNull));
            }
            if member == "namedItem" {
                if let Some(HostValue::String(name)) = arguments
                    .first()
                    .filter(|a| matches!(a,HostValue::String(s) if !s.is_empty()))
                {
                    let owner = self.document.borrow();
                    let tree = owner.GetDocument();
                    for &id in nodes.iter() {
                        let node = tree.Node(tree.FindNodeById(id).unwrap());
                        if node.FindAttribute("id").is_some_and(|a| a.value == *name)
                            || node.FindAttribute("name").is_some_and(|a| a.value == *name)
                        {
                            return Self::value(HostValue::Object(HostObjectRef { id }));
                        }
                    }
                }
                return Self::value(HostValue::Null(JavaScriptNull));
            }
            return Self::type_error("unknown collection method");
        }
        // cpp: webapi/dom_bindings.cc:737-759
        if receiver == 0 {
            return match member {
                "hasAttributes"
                | "getAttributeNames"
                | "__validateImportNode"
                | "importNode"
                | "__slotBrand"
                | "__slotName"
                | "__setSlotName"
                | "assignedNodes"
                | "assignedElements"
                | "assign" => Self::type_error("Illegal invocation"),
                "__domInstall" => {
                    if let [HostValue::String(name), HostValue::JavaScriptValue(value), ..] =
                        arguments
                    {
                        self.prototypes.insert(name.clone(), value.clone());
                        HostResult::default()
                    } else {
                        Self::type_error("prototype registration requires name and object")
                    }
                }
                "__domWrap" => {
                    if let [HostValue::String(name), HostValue::JavaScriptFunction(value), ..] =
                        arguments
                    {
                        self.method_wrappers.insert(name.clone(), value.clone());
                        if name == "__callEventHandler" {
                            self.native_abort.BindHandlerWrapper(value);
                        }
                        if name == "__inputDataTransferFileValue" {
                            self.native_abort.BindInputFileProjection(value);
                        }
                        if name == "__deliverMutationObserver" {
                            self.native_mutation_observers.BindScriptDelivery(value);
                        }
                        if name == "__eventState" {
                            *self.services.event_state.borrow_mut() = Some(value.clone());
                        }
                        HostResult::default()
                    } else {
                        Self::type_error("method registration requires name and function")
                    }
                }
                "__domInvoke" => {
                    if let [HostValue::Object(object), HostValue::String(name), rest @ ..] =
                        arguments
                    {
                        self.call(object.id, name, rest)
                    } else {
                        Self::type_error("Illegal invocation")
                    }
                }
                _ => Self::unhandled(),
            };
        }
        // cpp: webapi/dom_bindings.cc:816-823
        if member == "write" {
            let is_document = self.node(receiver).is_some_and(|index| {
                self.document.borrow().GetDocument().Node(index).Type() == DOMNodeType::kDocument
            });
            if is_document {
                // Document::write cc:4820-4825 rejects XML before requesting
                // a parser insertion point (including SVGDocument).
                let html = {
                    let owner = self.document.borrow();
                    let d = owner.GetDocument();
                    d.DocumentType(d.Root()) == dom::persistent_document::DOMDocumentType::kHTML
                };
                if !html {
                    return Self::popup_error(&dom::error::DOMException {
                        kind: dom::error::DOMExceptionKind::InvalidStateError,
                        message: "Only HTML documents support write().",
                    });
                }
                let Some(HostValue::String(markup)) = arguments.first() else {
                    return Self::type_error("document.write requires a string");
                };
                if self
                    .document
                    .borrow()
                    .GetDocument()
                    .ShouldIgnoreOpensAndWrites()
                {
                    return HostResult::default();
                }
                let _write_scope = self.document.borrow().GetDocument().EnterDocumentWrite();
                let _frame_creation_context = self
                    .document
                    .borrow()
                    .GetDocument()
                    .EnterFrameOwnerCreationContext(
                        dom::frame_owner_loading::FrameOwnerCreationContext::Script,
                    );
                if self
                    .write_document
                    .as_mut()
                    .is_some_and(|write| write(markup))
                {
                    return HostResult::default();
                }
                return Self::type_error(
                    "document.write requires an active parser insertion point",
                );
            }
        }
        if member.starts_with("__ce") {
            return self.CustomElementCall(receiver, member, arguments);
        }
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let Some(index) = document.FindNodeById(receiver) else {
            return Self::type_error("host object no longer exists");
        };
        let node = document.Node(index);
        match member {
            // Source: element.cc:10091-10131
            "setPointerCapture" | "releasePointerCapture" | "hasPointerCapture" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("pointer capture requires an element receiver");
                }
                let Some(HostValue::Number(pointer_id)) = arguments.first() else {
                    return Self::type_error("pointer capture requires a pointer id");
                };
                let Some(handler) = self.pointer_capture.clone() else {
                    return if member == "hasPointerCapture" {
                        Self::value(HostValue::Boolean(false))
                    } else {
                        HostResult::default()
                    };
                };
                let operation = match member {
                    "setPointerCapture" => PointerCaptureOperation::Set,
                    "releasePointerCapture" => PointerCaptureOperation::Release,
                    _ => PointerCaptureOperation::Has,
                };
                let pointer_id = *pointer_id as i32;
                drop(owner);
                match handler(operation, pointer_id, receiver) {
                    Ok(value) => Self::value(HostValue::Boolean(value)),
                    Err(error) => {
                        let (name, message) = match error {
                            interaction::PointerCaptureError::NoActivePointer => (
                                "NotFoundError",
                                "No active pointer with the given id is found.",
                            ),
                            interaction::PointerCaptureError::InvalidState => {
                                ("InvalidStateError", "InvalidStateError")
                            }
                        };
                        Self::value(HostValue::Record(vec![
                            ("exceptionName".into(), HostValue::String(name.into())),
                            ("message".into(), HostValue::String(message.into())),
                        ]))
                    }
                }
            }
            "__canvasDimension" | "__canvasSetDimension" | "getContext"
                if node.IsHTMLElement("canvas") =>
            {
                if member == "getContext" {
                    let Some(HostValue::String(context_id)) = arguments.first() else {
                        return Self::type_error("getContext requires a context identifier");
                    };
                    let Some(context_type) = canvas::CanvasContextType::FromContextId(context_id)
                    else {
                        return Self::value(HostValue::Null(JavaScriptNull));
                    };
                    if !matches!(
                        context_type,
                        canvas::CanvasContextType::WebGL1 | canvas::CanvasContextType::WebGL2
                    ) {
                        return Self::value(HostValue::Null(JavaScriptNull));
                    }
                    if !cfg!(feature = "webgl_vertical_slice") {
                        // Do not make standards feature detection succeed for
                        // the intentionally incomplete triangle-only slice.
                        return Self::value(HostValue::Null(JavaScriptNull));
                    }
                    if let Some(id) = self.canvas_webgl_ids.borrow().get(&receiver).copied() {
                        let same_family = self
                            .webgl_contexts
                            .borrow()
                            .get(&id)
                            .is_some_and(|context| context.ContextType() == context_type);
                        return if same_family {
                            Self::value(HostValue::Object(HostObjectRef { id }))
                        } else {
                            Self::value(HostValue::Null(JavaScriptNull))
                        };
                    }
                    let mut factory = webgl::WebGLContextFactory::default();
                    let desynchronized = matches!(arguments.get(1), Some(HostValue::Boolean(true)));
                    let attributes = canvas::CanvasContextCreationAttributesCore {
                        desynchronized,
                        desynchronized_specified: desynchronized,
                        ..Default::default()
                    };
                    let canvas_owner = document.CanvasElementState(index);
                    let created = canvas_owner
                        .borrow_mut()
                        .GetCanvasRenderingContext(&mut factory, context_id, attributes)
                        .is_some();
                    let Some(context) = created.then(|| factory.TakeCreated()).flatten() else {
                        return Self::value(HostValue::Null(JavaScriptNull));
                    };
                    self.canvas_elements
                        .borrow_mut()
                        .insert(receiver, canvas_owner);
                    let id = self.next_webgl_id.get();
                    self.next_webgl_id.set(id + 1);
                    self.webgl_contexts.borrow_mut().insert(id, context);
                    self.canvas_webgl_ids.borrow_mut().insert(receiver, id);
                    self.webgl_canvas_ids.borrow_mut().insert(id, receiver);
                    return Self::value(HostValue::Object(HostObjectRef { id }));
                }
                let Some(HostValue::String(dimension)) = arguments.first() else {
                    return Self::type_error("canvas dimension name is required");
                };
                let default = match dimension.as_str() {
                    "width" => 300_u32,
                    "height" => 150_u32,
                    _ => return Self::type_error("invalid canvas dimension"),
                };
                if member == "__canvasDimension" {
                    let value = node
                        .FindAttribute(dimension)
                        .and_then(|attribute| parse_canvas_dimension(&attribute.value))
                        // html_canvas_element.cc:842-859. The content
                        // attribute remains untouched, but dimensions above
                        // signed 31-bit range expose the element default.
                        .filter(|value| *value <= i32::MAX as u32)
                        .unwrap_or(default);
                    return Self::value(HostValue::Number(f64::from(value)));
                }
                let Some(HostValue::Number(value)) = arguments.get(1) else {
                    return Self::type_error("canvas dimension value is required");
                };
                if !value.is_finite() || *value < 0.0 || *value > u32::MAX as f64 {
                    return Self::type_error("canvas dimension is outside unsigned long range");
                }
                let value = *value as u32;
                // The IDL setter stores the dimension default for an unsigned
                // value outside Blink's 31-bit canvas range. Direct
                // setAttribute still preserves its original oversized text;
                // the getter branch above exposes the default in either case.
                let value = if value > i32::MAX as u32 {
                    default
                } else {
                    value
                };
                drop(owner);
                (self.emit_mutation.borrow_mut())(&DOMMutation {
                    mutation_type: DOMMutationType::kSetAttribute,
                    target_node_id: receiver,
                    name: dimension.clone(),
                    value: value.to_string(),
                    ..Default::default()
                });
                Self::value(HostValue::Number(f64::from(value)))
            }
            "isEqualNode" => {
                let other = match arguments.first() {
                    Some(HostValue::Object(other)) => document.FindNodeById(other.id),
                    Some(HostValue::Null(_)) | Some(HostValue::Undefined(_)) => None,
                    _ => return Self::type_error("isEqualNode requires a Node or null"),
                };
                Self::value(HostValue::Boolean(
                    other.is_some_and(|other| document.IsEqualNode(index, document, other)),
                ))
            }
            "__htmlConstructionBrand" => {
                let Some(HostValue::String(tag)) = arguments.first() else {
                    return Self::type_error("Invalid HTML constructor interface");
                };
                let compatible = node.Type() == DOMNodeType::kElement
                    && node.Namespace() == DOMNamespace::kHTML
                    && if tag.is_empty() {
                        dom::custom_elements::IsValidCustomElementName(node.Name())
                    } else {
                        node.Name() == tag
                    };
                if !compatible {
                    return Self::type_error("Illegal HTML constructor interface");
                }
                HostResult::default()
            }
            "getTotalLength"
                if node.Type() == DOMNodeType::kElement
                    && node.Namespace() == DOMNamespace::kSVG
                    && node.Name() == "path" =>
            {
                let Some(data) = node
                    .FindAttribute("d")
                    .map(|attribute| attribute.value.as_str())
                else {
                    return Self::value(HostValue::Number(0.0));
                };
                let Some(parsed) = dom::svg_path_parser::ParseSVGPathDefault(data) else {
                    return Self::value(HostValue::Number(0.0));
                };
                use skia::src::core::{
                    SkContourMeasure::SkContourMeasureIter, SkPathBuilder::SkPathBuilder,
                };
                let mut path = SkPathBuilder::default();
                for command in parsed.commands {
                    match command.verb {
                        PaintPathVerb::kMoveTo => {
                            path.moveToXY(command.point.x as f32, command.point.y as f32);
                        }
                        PaintPathVerb::kLineTo => {
                            path.lineToXY(command.point.x as f32, command.point.y as f32);
                        }
                        PaintPathVerb::kQuadraticTo => {
                            path.quadToXY(
                                command.control1.x as f32,
                                command.control1.y as f32,
                                command.point.x as f32,
                                command.point.y as f32,
                            );
                        }
                        PaintPathVerb::kCubicTo => {
                            path.cubicToXY(
                                command.control1.x as f32,
                                command.control1.y as f32,
                                command.control2.x as f32,
                                command.control2.y as f32,
                                command.point.x as f32,
                                command.point.y as f32,
                            );
                        }
                        PaintPathVerb::kClose => {
                            path.close();
                        }
                        PaintPathVerb::kConicTo => {
                            return Self::type_error("invalid SVG path geometry");
                        }
                    }
                }
                let path = path.detach(None);
                let mut contours = SkContourMeasureIter::new(&path, false, 1.0);
                let mut length = 0.0_f32;
                loop {
                    let contour = contours.next();
                    if !contour.is_some() {
                        break;
                    }
                    length += contour.length();
                }
                Self::value(HostValue::Number(f64::from(length)))
            }
            "contains" => {
                let Some(HostValue::Object(other)) = arguments.first() else {
                    return Self::value(HostValue::Boolean(false));
                };
                let Some(mut other) = document.FindNodeById(other.id) else {
                    return Self::value(HostValue::Boolean(false));
                };
                loop {
                    if other == index {
                        return Self::value(HostValue::Boolean(true));
                    }
                    let Some(parent) = document.Node(other).Parent() else {
                        return Self::value(HostValue::Boolean(false));
                    };
                    other = parent;
                }
            }
            "__sibling" => {
                let [HostValue::Boolean(reverse), HostValue::Boolean(elements), ..] = arguments
                else {
                    return Self::type_error(
                        "sibling lookup requires direction and element filter",
                    );
                };
                let Some(parent) = node.Parent() else {
                    return Self::node_reference(document, None);
                };
                // Blink Node::nextSibling/previousSibling read sibling pointers
                // (core/dom/node.h). This arena stores children in a Vec; scan
                // IDs directly instead of enumerating a JS NodeList and creating
                // an iterator result and wrapper lookup for every sibling.
                let siblings = document.Node(parent).Children();
                let Some(position) = siblings.iter().position(|&sibling| sibling == index) else {
                    return Self::node_reference(document, None);
                };
                let matches = |sibling: &usize| {
                    !*elements || document.Node(*sibling).Type() == DOMNodeType::kElement
                };
                let sibling = if *reverse {
                    siblings[..position].iter().rev().copied().find(matches)
                } else {
                    siblings[position + 1..].iter().copied().find(matches)
                };
                Self::node_reference(document, sibling)
            }
            // cpp: webapi/dom_bindings.cc:833-846
            "stylesheetConstruct" => {
                if node.Type() != DOMNodeType::kDocument {
                    return Self::type_error("CSSStyleSheet constructor requires a Document");
                }
                drop(owner);
                let mut owner = self.document.borrow_mut();
                let document = owner.GetDocumentMut();
                let sheet = style::ParseCSS("");
                let identity = match document.RegisterConstructedStyleSheet(sheet) {
                    Ok(identity) => identity,
                    Err(_) => return Self::type_error("Could not construct CSSStyleSheet"),
                };
                Self::value(HostValue::Number(identity as f64))
            }
            "stylesheetReplaceSync" => {
                let (Some(HostValue::Number(identity)), Some(HostValue::String(text))) =
                    (arguments.first(), arguments.get(1))
                else {
                    return Self::type_error("replaceSync requires a constructed sheet and text");
                };
                let raw_identity = *identity;
                let identity = raw_identity as u64;
                if identity == 0 || identity as f64 != raw_identity {
                    return Self::type_error("Invalid constructed stylesheet identity");
                }
                let replacement = style::ParseCSS(text);
                drop(owner);
                let mut owner = self.document.borrow_mut();
                let changed = match owner
                    .GetDocumentMut()
                    .ReplaceConstructedStyleSheet(identity, replacement)
                {
                    Ok(changed) => changed,
                    Err(_) => return Self::type_error("Unknown constructed stylesheet"),
                };
                drop(owner);
                if changed {
                    if let Some(invalidate) = self.services.host.invalidate_style.as_mut() {
                        invalidate();
                    }
                }
                HostResult::default()
            }
            "adoptedStyleSheetCount" | "adoptedStyleSheetIdentity" | "setAdoptedStyleSheets" => {
                if node.Type() != DOMNodeType::kDocument
                    && (node.Type() != DOMNodeType::kDocumentFragment
                        || document.ShadowHost(index).is_none())
                {
                    return Self::type_error(
                        "adoptedStyleSheets requires a Document or ShadowRoot",
                    );
                }
                if member == "adoptedStyleSheetCount" {
                    return Self::value(HostValue::Number(
                        document.AdoptedStyleSheetsForTreeScope(index).len() as f64,
                    ));
                }
                if member == "adoptedStyleSheetIdentity" {
                    let Some(HostValue::Number(position)) = arguments.first() else {
                        return Self::type_error("adopted stylesheet index is required");
                    };
                    let sheets = document.AdoptedStyleSheetsForTreeScope(index);
                    let position = *position as usize;
                    return Self::value(HostValue::Number(
                        sheets.get(position).map_or(0, |sheet| sheet.Identity()) as f64,
                    ));
                }
                let mut identities = Vec::with_capacity(arguments.len());
                for value in arguments {
                    let HostValue::Number(identity) = value else {
                        return Self::type_error(
                            "adoptedStyleSheets requires CSSStyleSheet objects",
                        );
                    };
                    let converted = *identity as u64;
                    if converted == 0 || converted as f64 != *identity {
                        return Self::type_error("Invalid constructed stylesheet identity");
                    }
                    identities.push(converted);
                }
                drop(owner);
                let mut owner = self.document.borrow_mut();
                let changed = match owner
                    .GetDocumentMut()
                    .SetAdoptedStyleSheets(index, identities)
                {
                    Ok(changed) => changed,
                    Err(_) => {
                        return Self::type_error("Cannot adopt stylesheet in this tree scope");
                    }
                };
                drop(owner);
                if changed {
                    if let Some(invalidate) = self.services.host.invalidate_style.as_mut() {
                        invalidate();
                    }
                }
                HostResult::default()
            }
            "stylesheetCount" => Self::value(HostValue::Number(
                document.StyleSheetsForNode(index).len() as f64,
            )),
            "stylesheetOwner"
            | "stylesheetRuleCount"
            | "stylesheetRuleText"
            | "constructedStylesheetRuleCount"
            | "constructedStylesheetRuleText" => {
                let constructed = member.starts_with("constructed");
                let sheets = document.StyleSheetsForNode(index);
                let Some(HostValue::Number(sheet_index)) = arguments.first() else {
                    return Self::value(HostValue::Null(Default::default()));
                };
                let sheet = if constructed {
                    document.ConstructedStyleSheet(*sheet_index as u64)
                } else {
                    (*sheet_index >= 0.0)
                        .then(|| sheets.get(*sheet_index as usize))
                        .flatten()
                };
                let Some(sheet) = sheet else {
                    return Self::value(HostValue::Null(Default::default()));
                };
                if member == "stylesheetOwner" {
                    return Self::value(if sheet.owner_node_id != 0 {
                        HostValue::Object(HostObjectRef {
                            id: sheet.owner_node_id,
                        })
                    } else {
                        HostValue::Null(Default::default())
                    });
                }
                if member.ends_with("RuleCount") {
                    return Self::value(HostValue::Number(sheet.rules.len() as f64));
                }
                let Some(HostValue::Number(rule_index)) = arguments.get(1) else {
                    return Self::value(HostValue::String(String::new()));
                };
                if *rule_index < 0.0 || *rule_index >= sheet.rules.len() as f64 {
                    return Self::value(HostValue::String(String::new()));
                }
                let rule = &sheet.rules[*rule_index as usize];
                let mut text = rule.selector_text.clone() + " {";
                for declaration in &rule.declarations {
                    text.push_str(&declaration.property);
                    text.push(':');
                    text.push_str(&declaration.value);
                    text.push(';');
                }
                text.push('}');
                Self::value(HostValue::String(text))
            }
            // cpp: webapi/dom_bindings.cc:847-869
            "stylesheetInsertRule"
            | "stylesheetDeleteRule"
            | "constructedStylesheetInsertRule"
            | "constructedStylesheetDeleteRule" => {
                let constructed = member.starts_with("constructed");
                let sheets = document.StyleSheetsForNode(index);
                let (Some(HostValue::Number(sheet_index)), Some(HostValue::Number(rule_index))) =
                    (arguments.first(), arguments.get(1))
                else {
                    return Self::type_error("Invalid stylesheet index");
                };
                if *sheet_index < 0.0 || *rule_index < 0.0 {
                    return Self::type_error("Invalid stylesheet index");
                }
                let sheet = if constructed {
                    document.ConstructedStyleSheet(*sheet_index as u64)
                } else {
                    sheets.get(*sheet_index as usize)
                };
                let Some(sheet) = sheet else {
                    return Self::type_error("Invalid stylesheet index");
                };
                if *rule_index > sheet.rules.len() as f64
                    || (member.ends_with("DeleteRule") && *rule_index == sheet.rules.len() as f64)
                {
                    return HostResult::Failure(
                        javascript::javascript_runtime::JavaScriptExceptionKind::kRangeError,
                        "CSS rule index out of bounds",
                    );
                }
                let index = *rule_index as usize;
                let sheet = if member.ends_with("InsertRule") {
                    let text = match arguments.get(2) {
                        Some(HostValue::String(text)) => text.as_str(),
                        _ => "",
                    };
                    style::InsertCSSStyleSheetRule(sheet, index, text)
                } else {
                    style::DeleteCSSStyleSheetRule(sheet, index)
                };
                let sheet = match sheet {
                    Ok(sheet) => sheet,
                    Err(style::CSSStyleSheetRuleMutationError::IndexSize) => {
                        return HostResult::Failure(
                            javascript::javascript_runtime::JavaScriptExceptionKind::kRangeError,
                            "CSS rule index out of bounds",
                        );
                    }
                    Err(style::CSSStyleSheetRuleMutationError::Syntax) => {
                        return HostResult::Failure(
                            javascript::javascript_runtime::JavaScriptExceptionKind::kSyntaxError,
                            "Expected one CSS style rule",
                        );
                    }
                    Err(style::CSSStyleSheetRuleMutationError::MissingTypedBacking) => {
                        return HostResult::Failure(
                            javascript::javascript_runtime::JavaScriptExceptionKind::kTypeError,
                            "Stylesheet has no current parsed owner",
                        );
                    }
                };
                drop(owner);
                if constructed {
                    let mut owner = self.document.borrow_mut();
                    let changed = match owner
                        .GetDocumentMut()
                        .ReplaceConstructedStyleSheet(*sheet_index as u64, sheet)
                    {
                        Ok(changed) => changed,
                        Err(_) => return Self::type_error("Unknown constructed stylesheet"),
                    };
                    drop(owner);
                    if changed {
                        if let Some(invalidate) = self.services.host.invalidate_style.as_mut() {
                            invalidate();
                        }
                    }
                } else if let Some(emit) = self.services.host.emit_style_sheet.as_mut() {
                    emit(sheet);
                }
                Self::value(HostValue::Number(index as f64))
            }
            // cpp: webapi/dom_bindings.cc:779-804
            "serializeNode" => {
                let mut output = String::new();
                serialize_node(document, index, &mut output);
                Self::value(HostValue::String(output))
            }
            "__slotBrand" | "__slotName" | "__setSlotName" | "assignedNodes"
            | "assignedElements" | "assign" => {
                if !node.IsHTMLElement("slot") {
                    return Self::type_error("HTMLSlotElement method requires a slot receiver");
                }
                if member == "__slotBrand" {
                    return HostResult::default();
                }
                if member == "__slotName" {
                    return Self::value(HostValue::String(
                        node.FindAttribute("name")
                            .map_or("", |a| a.value.as_str())
                            .into(),
                    ));
                }
                if member == "__setSlotName" {
                    let Some(HostValue::String(value)) = arguments.first() else {
                        return Self::type_error("slot name requires a DOMString");
                    };
                    let mutation = DOMMutation {
                        mutation_type: DOMMutationType::kSetAttribute,
                        target_node_id: receiver,
                        name: "name".into(),
                        value: value.clone(),
                        ..Default::default()
                    };
                    drop(owner);
                    (self.emit_mutation.borrow_mut())(&mutation);
                    return HostResult::default();
                }
                if member == "assign" {
                    // Web IDL converts the entire variadic union before any mutation.
                    let mut node_ids = Vec::with_capacity(arguments.len());
                    for argument in arguments {
                        let HostValue::Object(object) = argument else {
                            return Self::type_error("assign accepts only Element or Text nodes");
                        };
                        let Some(child) = document.FindNodeById(object.id) else {
                            return Self::type_error("assign accepts only Element or Text nodes");
                        };
                        if !matches!(
                            document.Node(child).Type(),
                            DOMNodeType::kElement | DOMNodeType::kText
                        ) {
                            return Self::type_error("assign accepts only Element or Text nodes");
                        }
                        node_ids.push(object.id);
                    }
                    let mutation = DOMMutation {
                        mutation_type: DOMMutationType::kAssignSlotNodes,
                        target_node_id: receiver,
                        node_ids,
                        ..Default::default()
                    };
                    drop(owner);
                    (self.emit_mutation.borrow_mut())(&mutation);
                    return HostResult::default();
                }
                let nodes = if matches!(arguments.first(), Some(HostValue::Boolean(true))) {
                    document.FlattenedAssignedNodes(index)
                } else {
                    document.AssignedNodes(index)
                };
                let nodes: Vec<_> = nodes
                    .into_iter()
                    .filter(|&child| {
                        member != "assignedElements"
                            || document.Node(child).Type() == DOMNodeType::kElement
                    })
                    .collect();
                let mut fields = Vec::with_capacity(nodes.len() + 1);
                fields.push(("length".into(), HostValue::Number(nodes.len() as f64)));
                fields.extend(nodes.into_iter().enumerate().map(|(position, child)| {
                    (
                        position.to_string(),
                        HostValue::Object(HostObjectRef {
                            id: document.Node(child).Id(),
                        }),
                    )
                }));
                Self::value(HostValue::Record(fields))
            }
            "hasAttributes" | "getAttributeNames" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("attribute methods require an element receiver");
                }
                // Blink Element::hasAttributes / getAttributeNamesForBindings.
                // Read canonical attribute order and qualified names directly;
                // the snapshot crosses the existing immutable value boundary.
                if member == "hasAttributes" {
                    return Self::value(HostValue::Boolean(!node.Attributes().is_empty()));
                }
                let mut fields = Vec::with_capacity(node.Attributes().len() + 1);
                fields.push((
                    "length".into(),
                    HostValue::Number(node.Attributes().len() as f64),
                ));
                fields.extend(
                    node.Attributes()
                        .iter()
                        .enumerate()
                        .map(|(position, attribute)| {
                            let name = if attribute.prefix.is_empty() {
                                attribute.local_name.clone()
                            } else {
                                format!("{}:{}", attribute.prefix, attribute.local_name)
                            };
                            (position.to_string(), HostValue::String(name))
                        }),
                );
                Self::value(HostValue::Record(fields))
            }
            "attributeList" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("attributes requires an element receiver");
                }
                let list = self.allocate_collection_id();
                let mut fields = HashMap::from([(
                    "length".into(),
                    HostValue::Number(node.Attributes().len() as f64),
                )]);
                for (position, attribute) in node.Attributes().iter().enumerate() {
                    let id = self.allocate_collection_id();
                    let name = if attribute.prefix.is_empty() {
                        attribute.local_name.clone()
                    } else {
                        format!("{}:{}", attribute.prefix, attribute.local_name)
                    };
                    self.StoreRecord(
                        id,
                        HashMap::from([
                            ("name".into(), HostValue::String(name)),
                            (
                                "localName".into(),
                                HostValue::String(attribute.local_name.clone()),
                            ),
                            (
                                "namespaceURI".into(),
                                if attribute.namespace_uri.is_empty() {
                                    HostValue::Null(JavaScriptNull)
                                } else {
                                    HostValue::String(attribute.namespace_uri.clone())
                                },
                            ),
                            (
                                "prefix".into(),
                                if attribute.prefix.is_empty() {
                                    HostValue::Null(JavaScriptNull)
                                } else {
                                    HostValue::String(attribute.prefix.clone())
                                },
                            ),
                            ("value".into(), HostValue::String(attribute.value.clone())),
                        ]),
                    );
                    fields.insert(
                        position.to_string(),
                        HostValue::Object(HostObjectRef { id }),
                    );
                }
                self.StoreRecord(list, fields);
                Self::value(HostValue::Object(HostObjectRef { id: list }))
            }
            "importNode" | "__validateImportNode" => {
                if node.Type() != DOMNodeType::kDocument {
                    return Self::type_error("importNode requires a document receiver");
                }
                let Some(HostValue::Object(source)) = arguments.first() else {
                    return Self::type_error("importNode requires a Node");
                };
                let Some(source_index) = document.FindNodeById(source.id) else {
                    return Self::type_error("importNode requires a Node");
                };
                let supported = document.Node(source_index).Type() != DOMNodeType::kDocument
                    && document.ShadowHost(source_index).is_none();
                if member == "__validateImportNode" {
                    return Self::value(HostValue::Boolean(supported));
                }
                if !supported {
                    return Self::type_error("Document and ShadowRoot nodes cannot be imported");
                }
                let mutation = DOMMutation {
                    mutation_type: DOMMutationType::kImportNode,
                    target_node_id: receiver,
                    source_node_id: source.id,
                    child_node_id: document.NextNodeId(),
                    bool_value: matches!(arguments.get(1), Some(HostValue::Boolean(true))),
                    ..Default::default()
                };
                let id = mutation.child_node_id;
                drop(owner);
                (self.emit_mutation.borrow_mut())(&mutation);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            // cpp: webapi/dom_bindings.cc:805-811,817-827
            "createHTMLDocument" | "cloneNode" => {
                let mut mutation = DOMMutation {
                    mutation_type: if member == "cloneNode" {
                        DOMMutationType::kCloneNode
                    } else {
                        DOMMutationType::kCreateDocument
                    },
                    target_node_id: receiver,
                    child_node_id: document.NextNodeId(),
                    bool_value: if member == "cloneNode" {
                        matches!(arguments.first(), Some(HostValue::Boolean(true)))
                    } else {
                        !arguments.is_empty()
                    },
                    ..Default::default()
                };
                if let Some(HostValue::String(title)) = arguments.first() {
                    mutation.value = title.clone();
                }
                let id = mutation.child_node_id;
                drop(owner);
                (self.emit_mutation.borrow_mut())(&mutation);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            "hasFocus" => Self::value(HostValue::Boolean(
                self.services
                    .host
                    .read_document_focus
                    .as_ref()
                    .and_then(|read| read(document.RootHandle().Identity()))
                    .is_some_and(|focus| focus.has_focus),
            )),
            // cpp: element.cc:7768-7888. The DOM owner allocates the
            // ShadowRoot so style/tree-scope identity is not confined to the
            // JavaScript wrapper's WeakMap.
            "__shadowReferenceTargetGet" | "__shadowReferenceTargetSet" => {
                if document.ShadowHost(index).is_none() {
                    return Self::type_error("referenceTarget requires a ShadowRoot");
                }
                if member == "__shadowReferenceTargetGet" {
                    return Self::value(
                        document
                            .ShadowReferenceTarget(index)
                            .map_or(HostValue::Null(JavaScriptNull), |v| {
                                HostValue::String(v.into())
                            }),
                    );
                }
                let reference_target = match arguments.first() {
                    Some(HostValue::String(value)) => Some(value.clone()),
                    Some(HostValue::Null(_)) => None,
                    _ => return Self::type_error("referenceTarget requires a nullable DOMString"),
                };
                drop(owner);
                (self.emit_mutation.borrow_mut())(&DOMMutation {
                    mutation_type: DOMMutationType::kSetShadowReferenceTarget,
                    target_node_id: receiver,
                    reference_target,
                    ..Default::default()
                });
                HostResult::default()
            }
            "attachShadow" => {
                let (Some(HostValue::String(mode)), Some(HostValue::String(slot_assignment))) =
                    (arguments.first(), arguments.get(1))
                else {
                    return Self::type_error("shadow root mode is required");
                };
                if !matches!(mode.as_str(), "open" | "closed") {
                    return Self::type_error("shadow root mode must be open or closed");
                }
                if !matches!(slot_assignment.as_str(), "named" | "manual") {
                    return Self::type_error("shadow root slot assignment must be named or manual");
                }
                let mutation = DOMMutation {
                    mutation_type: DOMMutationType::kAttachShadowRoot,
                    target_node_id: receiver,
                    child_node_id: document.NextNodeId(),
                    name: slot_assignment.clone(),
                    value: mode.clone(),
                    bool_value: matches!(arguments.get(2), Some(HostValue::Boolean(true))),
                    reference_target: match arguments.get(3) {
                        Some(HostValue::String(value)) => Some(value.clone()),
                        _ => None,
                    },
                    ..Default::default()
                };
                let id = mutation.child_node_id;
                drop(owner);
                (self.emit_mutation.borrow_mut())(&mutation);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            // cpp: webapi/dom_bindings.cc:861-883
            "createElement"
            | "createElementNS"
            | "createTextNode"
            | "createComment"
            | "createDocumentFragment" => {
                let mut mutation = DOMMutation {
                    mutation_type: DOMMutationType::kCreateElement,
                    target_node_id: if node.Type() == DOMNodeType::kDocument {
                        receiver
                    } else {
                        node.OwnerDocumentNode().unwrap().Id()
                    },
                    child_node_id: document.NextNodeId(),
                    ..Default::default()
                };
                match member {
                    "createTextNode" | "createComment" => {
                        mutation.mutation_type = if member == "createTextNode" {
                            DOMMutationType::kCreateText
                        } else {
                            DOMMutationType::kCreateComment
                        };
                        match arguments.first() {
                            Some(HostValue::String(text)) => mutation.value = text.clone(),
                            Some(HostValue::String16(units)) => {
                                mutation.value = String::from_utf16_lossy(units);
                                mutation.utf16_value = Some(units.clone());
                            }
                            _ => {}
                        }
                    }
                    "createDocumentFragment" => {
                        mutation.mutation_type = DOMMutationType::kCreateFragment
                    }
                    _ => {
                        let Some(HostValue::String(name)) =
                            arguments.get(usize::from(member == "createElementNS"))
                        else {
                            return Self::type_error("element name is required");
                        };
                        if name.is_empty() {
                            return Self::type_error("element name is required");
                        }
                        mutation.is_value =
                            match arguments.get(if member == "createElementNS" { 2 } else { 1 }) {
                                Some(HostValue::String16(value)) => Some(value.clone()),
                                Some(HostValue::String(value)) => {
                                    Some(value.encode_utf16().collect::<Vec<_>>().into())
                                }
                                _ => None,
                            };
                        mutation.name = if member == "createElement" {
                            name.to_ascii_lowercase()
                        } else {
                            name.clone()
                        };
                        if member == "createElementNS" {
                            mutation.bool_value = true;
                            if let Some(HostValue::String(ns)) = arguments.first() {
                                mutation.namespace_uri = ns.clone();
                            }
                            if let Some((prefix, local_name)) = name.split_once(':') {
                                mutation.prefix = prefix.into();
                                mutation.name = local_name.into();
                            }
                        }
                    }
                }
                let id = mutation.child_node_id;
                drop(owner);
                (self.emit_mutation.borrow_mut())(&mutation);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            // cpp: webapi/dom_bindings.cc:884-898
            "appendChild" | "insertBefore" | "removeChild" => {
                let Some(HostValue::Object(child)) = arguments.first() else {
                    return Self::type_error("child must be a Node");
                };
                let child = *child;
                let before_node_id = if member == "insertBefore" {
                    if let Some(HostValue::Object(before)) = arguments.get(1) {
                        before.id
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mutation = DOMMutation {
                    mutation_type: match member {
                        "appendChild" => DOMMutationType::kAppendChild,
                        "insertBefore" => DOMMutationType::kInsertBefore,
                        _ => DOMMutationType::kRemoveChild,
                    },
                    target_node_id: receiver,
                    child_node_id: child.id,
                    before_node_id,
                    ..Default::default()
                };
                drop(owner);
                if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (self.emit_mutation.borrow_mut())(&mutation)
                })) {
                    if let Some(error) = payload.downcast_ref::<dom::error::DOMException>() {
                        return Self::type_error(error.message);
                    }
                    std::panic::resume_unwind(payload);
                }
                Self::value(HostValue::Object(child))
            }
            // cpp: webapi/dom_bindings.cc:899-921
            "getStyle" | "setStyle" | "hasStyle" => {
                let Some(HostValue::String(property)) = arguments.first() else {
                    return Self::type_error("style property name is required");
                };
                if member == "hasStyle" {
                    // Source: CSSStyleDeclaration's named-property getter in
                    // core/css/css_style_declaration.idl:38-44 and generated
                    // css_property_names.cc:1886-1895.  `property in style`
                    // tests the exposed CSS property table, not the current
                    // declaration value.
                    return Self::value(HostValue::Boolean(
                        style::css_property_names::FindProperty(
                            property.to_ascii_lowercase().as_bytes(),
                        )
                        .is_some(),
                    ));
                }
                let inline_style = node.InlineStyle();
                let standard_mode = node.Namespace() == DOMNamespace::kHTML
                    && document.GetCompatibilityMode()
                        != dom::persistent_document::CompatibilityMode::kQuirks;
                let cssom_context =
                    self.services
                        .host
                        .cssom_document
                        .as_ref()
                        .map(|(source, policy)| {
                            source.CSSOMContextForElement(
                                document,
                                index,
                                policy.Get(),
                                self.cssom_world.borrow().clone(),
                            )
                        });
                if member == "getStyle" {
                    if let Some(context) = &cssom_context {
                        return Self::value(HostValue::String(
                            style::GetInlineCSSDeclarationWithContext(
                                &inline_style,
                                property,
                                context,
                            ),
                        ));
                    }
                    return Self::value(HostValue::String(style::GetInlineCSSDeclaration(
                        &inline_style,
                        property,
                        standard_mode,
                    )));
                }
                let Some(HostValue::String(value)) = arguments.get(1) else {
                    return Self::type_error("style value is required");
                };
                let priority = match arguments.get(2) {
                    Some(HostValue::String(priority)) => priority.as_str(),
                    _ => "",
                };
                let result = cssom_context.as_ref().map_or_else(
                    || {
                        style::SetInlineCSSDeclaration(
                            &inline_style,
                            property,
                            value,
                            priority,
                            standard_mode,
                        )
                    },
                    |context| {
                        style::SetInlineCSSDeclarationWithContext(
                            &inline_style,
                            property,
                            value,
                            priority,
                            context,
                        )
                    },
                );
                let Some(result) = result else {
                    return HostResult::default();
                };
                let (mutation_type, value, inline_style_independent, inline_style) = match result {
                    style::CSSInlineDeclarationMutation::Changed(style) => (
                        DOMMutationType::kSetAttribute,
                        style.SourceText().to_owned(),
                        false,
                        Some(style),
                    ),
                    style::CSSInlineDeclarationMutation::ChangedIndependent(style) => (
                        DOMMutationType::kSetAttribute,
                        style.SourceText().to_owned(),
                        true,
                        Some(style),
                    ),
                    style::CSSInlineDeclarationMutation::Unchanged => {
                        if document.CustomElementDefinitionForElement(index).is_none() {
                            return HostResult::default();
                        }
                        (
                            DOMMutationType::kNotifyInlineStyleUnchanged,
                            String::new(),
                            false,
                            None,
                        )
                    }
                };
                drop(owner);
                (self.emit_mutation.borrow_mut())(&DOMMutation {
                    mutation_type,
                    target_node_id: receiver,
                    name: "style".into(),
                    value,
                    inline_style_independent,
                    inline_style,
                    ..Default::default()
                });
                HostResult::default()
            }
            // cpp: webapi/dom_bindings.cc:922-949
            "querySelectorAll" => {
                let Some(HostValue::String(selector)) = arguments.first() else {
                    return Self::type_error("querySelectorAll requires a selector");
                };
                let mut service = self.selector_service.borrow_mut();
                service.interaction.focused_node_id = self.focused_node;
                let result = service.QueryAll(document, index, selector);
                drop(service);
                let nodes = match result {
                    Ok(nodes) => nodes,
                    Err(error) => return Self::selector_error(error),
                };
                let id = self.allocate_collection_id();
                let collection = NodeCollection {
                    root: receiver,
                    kind: "snapshot".into(),
                    query: selector.clone(),
                    snapshot: nodes
                        .into_iter()
                        .map(|node| document.Node(node).Id())
                        .collect(),
                    membership_cache: RefCell::new(None),
                };
                self.StoreCollection(id, collection);
                Self::value(HostValue::Object(HostObjectRef { id }))
            }
            "getElementsByTagName" | "getElementsByClassName" => {
                let Some(HostValue::String(query)) = arguments.first() else {
                    return Self::type_error(&format!("{member} requires a string"));
                };
                Self::value(HostValue::Object(self.collection(
                    receiver,
                    if member == "querySelectorAll" {
                        "snapshot"
                    } else {
                        member
                    },
                    query,
                )))
            }
            "querySelector" | "matches" | "closest" => {
                let Some(HostValue::String(selector)) = arguments.first() else {
                    return Self::type_error(&format!("{member} requires a selector"));
                };
                let mut service = self.selector_service.borrow_mut();
                service.interaction.focused_node_id = self.focused_node;
                if member == "matches" {
                    return match service.Matches(document, index, selector) {
                        Ok(value) => Self::value(HostValue::Boolean(value)),
                        Err(error) => Self::selector_error(error),
                    };
                }
                let result = if member == "closest" {
                    service.Closest(document, index, selector)
                } else {
                    service.QueryFirst(document, index, selector)
                };
                let found = match result {
                    Ok(found) => found,
                    Err(error) => return Self::selector_error(error),
                };
                found.map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |node| {
                        Self::value(HostValue::Object(HostObjectRef {
                            id: document.Node(node).Id(),
                        }))
                    },
                )
            }
            // cpp: html_dialog_element.cc:148-224,552-616;
            //      html_element.cc:1927-1935, Show/HidePopoverInternal.
            "showModal" | "close" | "showPopover" | "hidePopover" => {
                if matches!(member, "showModal" | "close") && !node.IsHTMLElement("dialog") {
                    return Self::type_error("dialog method requires a dialog receiver");
                }
                let mutation = DOMMutation {
                    mutation_type: if matches!(member, "showModal" | "close") {
                        DOMMutationType::kSetDialogModal
                    } else {
                        DOMMutationType::kSetPopoverOpen
                    },
                    target_node_id: receiver,
                    bool_value: matches!(member, "showModal" | "showPopover"),
                    utf16_value: if member == "close" {
                        match arguments.first() {
                            Some(HostValue::String16(value)) => Some(value.clone()),
                            Some(HostValue::String(value)) => {
                                Some(value.encode_utf16().collect::<Vec<_>>().into())
                            }
                            _ => None,
                        }
                    } else {
                        None
                    },
                    source_node_id: if member == "showPopover" {
                        match arguments.first() {
                            Some(HostValue::Object(source)) => source.id,
                            _ => 0,
                        }
                    } else {
                        0
                    },
                    ..Default::default()
                };
                drop(owner);
                if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (self.emit_mutation.borrow_mut())(&mutation)
                })) {
                    if let Some(error) = payload.downcast_ref::<dom::error::DOMException>() {
                        return Self::popup_error(error);
                    }
                    std::panic::resume_unwind(payload);
                }
                HostResult::default()
            }
            // cpp: fullscreen/element_fullscreen.idl;
            //      fullscreen/document_fullscreen.idl. The DOM host owns the
            // document fullscreen stack; platform/window presentation stays
            // outside this platform-neutral lifecycle.
            "requestFullscreen" | "exitFullscreen" => {
                if member == "requestFullscreen" && node.Type() != DOMNodeType::kElement {
                    return Self::type_error("requestFullscreen requires an element receiver");
                }
                if member == "exitFullscreen" && node.Type() != DOMNodeType::kDocument {
                    return Self::type_error("exitFullscreen requires a document receiver");
                }
                let mutation = DOMMutation {
                    mutation_type: if member == "requestFullscreen" {
                        DOMMutationType::kRequestFullscreen
                    } else {
                        DOMMutationType::kExitFullscreen
                    },
                    target_node_id: receiver,
                    ..Default::default()
                };
                drop(owner);
                if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (self.emit_mutation.borrow_mut())(&mutation)
                })) {
                    if let Some(error) = payload.downcast_ref::<dom::error::DOMException>() {
                        return Self::type_error(error.message);
                    }
                    std::panic::resume_unwind(payload);
                }
                HostResult::default()
            }
            // cpp: webapi/dom_bindings.cc:950-962
            "getElementById" => {
                if !matches!(
                    node.Type(),
                    DOMNodeType::kDocument | DOMNodeType::kDocumentFragment
                ) {
                    return Self::type_error(
                        "getElementById requires a document or fragment receiver",
                    );
                }
                let Some(HostValue::String(id)) = arguments.first() else {
                    return Self::type_error("getElementById requires a string");
                };
                let found = document.FindElementByIdInTreeScope(index, id);
                found.map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |found| {
                        Self::value(HostValue::Object(HostObjectRef {
                            id: document.Node(found).Id(),
                        }))
                    },
                )
            }
            // cpp: webapi/dom_bindings.cc:963-995
            "getAttribute" | "hasAttribute" | "removeAttribute" | "setAttribute"
            | "setAttributeNS" => {
                if node.Type() != DOMNodeType::kElement {
                    return Self::type_error("attribute method requires an element receiver");
                }
                let namespaced = member == "setAttributeNS";
                let name_argument = usize::from(namespaced);
                let name = if matches!(member, "setAttribute" | "setAttributeNS") {
                    dom_string_argument(arguments.get(name_argument))
                } else {
                    match arguments.first() {
                        Some(HostValue::String(name)) => Some(name.clone()),
                        _ => None,
                    }
                };
                let Some(name) = name else {
                    return Self::type_error(
                        if matches!(member, "setAttribute" | "setAttributeNS") {
                            "setAttribute requires string-convertible name and value"
                        } else {
                            "attribute method requires a name"
                        },
                    );
                };
                let (prefix, local_name) = if namespaced {
                    let mut parts = name.split(':');
                    let first = parts.next().unwrap_or_default();
                    match (parts.next(), parts.next()) {
                        (None, _) if !first.is_empty() => (String::new(), first.to_owned()),
                        (Some(local), None) if !first.is_empty() && !local.is_empty() => {
                            (first.to_owned(), local.to_owned())
                        }
                        _ => {
                            return Self::type_error(
                                "setAttributeNS requires a valid qualified name",
                            );
                        }
                    }
                } else {
                    (String::new(), name.clone())
                };
                let namespace_uri = if namespaced {
                    match arguments.first() {
                        Some(HostValue::Null(_)) | None => String::new(),
                        Some(value) => dom_string_argument(Some(value)).unwrap_or_default(),
                    }
                } else {
                    String::new()
                };
                if namespaced
                    && ((!prefix.is_empty() && namespace_uri.is_empty())
                        || (prefix == "xml"
                            && namespace_uri != "http://www.w3.org/XML/1998/namespace")
                        || ((prefix == "xmlns" || name == "xmlns")
                            && namespace_uri != "http://www.w3.org/2000/xmlns/")
                        || (namespace_uri == "http://www.w3.org/2000/xmlns/"
                            && prefix != "xmlns"
                            && name != "xmlns"))
                {
                    return Self::type_error(
                        "setAttributeNS namespace does not match qualified name",
                    );
                }
                let attribute = node
                    .Attributes()
                    .iter()
                    .find(|attribute| {
                        attribute.local_name == local_name
                            && attribute.namespace_uri == namespace_uri
                    })
                    .map(|attribute| attribute.value.clone());
                if member == "getAttribute" {
                    let value = node
                        .Attributes()
                        .iter()
                        .position(|attribute| {
                            attribute.local_name == local_name
                                && attribute.namespace_uri == namespace_uri
                        })
                        .map(|i| node.AttributesUTF16()[i].value.clone());
                    return Self::value(
                        value.map_or(HostValue::Null(JavaScriptNull), HostValue::String16),
                    );
                }
                if member == "hasAttribute" {
                    return Self::value(HostValue::Boolean(attribute.is_some()));
                }
                let utf16_value = if matches!(member, "setAttribute" | "setAttributeNS") {
                    match arguments.get(if namespaced { 2 } else { 1 }) {
                        Some(HostValue::String16(units)) => Some(units.clone()),
                        _ => None,
                    }
                } else {
                    None
                };
                let value = if let Some(units) = &utf16_value {
                    String::from_utf16_lossy(units)
                } else if matches!(member, "setAttribute" | "setAttributeNS") {
                    let Some(value) =
                        dom_string_argument(arguments.get(if namespaced { 2 } else { 1 }))
                    else {
                        return Self::type_error(
                            "setAttribute requires string-convertible name and value",
                        );
                    };
                    value
                } else {
                    String::new()
                };
                // The emitter synchronously applies the mutation to this
                // arena; release the read borrow before invoking it.
                drop(owner);
                let mut trace = browser_tracing::span("dom", "DOMMutationHost");
                trace.set(
                    "mutation_type",
                    if matches!(member, "setAttribute" | "setAttributeNS") {
                        DOMMutationType::kSetAttribute as u8 as f64
                    } else {
                        DOMMutationType::kRemoveAttribute as u8 as f64
                    },
                );
                (self.emit_mutation.borrow_mut())(&DOMMutation {
                    mutation_type: if matches!(member, "setAttribute" | "setAttributeNS") {
                        DOMMutationType::kSetAttribute
                    } else {
                        DOMMutationType::kRemoveAttribute
                    },
                    target_node_id: receiver,
                    name: local_name,
                    prefix,
                    value,
                    utf16_value,
                    namespace_uri,
                    ..Default::default()
                });
                HostResult::default()
            }
            _ => Self::type_error("host method is not installed"),
        }
    }
}

impl Drop for DOMJavaScriptBindings {
    fn drop(&mut self) {
        self.native_abort.ContextDestroyed();
    }
}

impl JavaScriptHostBindings for DOMJavaScriptBindings {
    fn NativeObject(&self, id: HostObjectId) -> Option<Rc<dyn std::any::Any>> {
        self.native_abort.NativeObject(id).or_else(|| {
            self.services
                .events
                .borrow()
                .NativeEvent(id)
                .map(|event| Rc::new(event) as Rc<dyn std::any::Any>)
        })
    }
    fn NativeObjectOwnerIdentity(
        &self,
    ) -> Option<javascript::javascript_runtime::HostObjectOwnerIdentity> {
        Some(javascript::javascript_runtime::HostObjectOwnerIdentity(
            self.document.borrow().GetDocument().RootHandle().Identity(),
        ))
    }
    fn DocumentHostObjectId(&self) -> Option<HostObjectId> {
        let owner = self.document.borrow();
        Some(owner.GetDocument().Node(owner.GetDocument().Root()).Id())
    }
    fn BindTaskAttributionTracker(
        &mut self,
        tracker: Rc<foundation::task_attribution::TaskAttributionTracker>,
    ) {
        if let Some(old) = self.task_attribution_tracker.get() {
            assert!(Rc::ptr_eq(old, &tracker), "same Window/VM task tracker");
        } else {
            assert!(self.task_attribution_tracker.set(tracker).is_ok());
        }
    }
    fn ArgumentPolicy(
        &self,
        receiver: HostObjectId,
        member: &str,
    ) -> javascript::javascript_runtime::HostArgumentPolicy {
        use javascript::javascript_runtime::HostArgumentPolicy as P;
        match member {
            "execCommand" => P::NativeIdentity,
            member if member.starts_with("__trustedTypes") => P::NativeIdentity,
            "__abortControllerAbort" | "__abortStatic" => P::Identity,
            "__formDataFileRemember"
            | "__formDataAppendFile"
            | "__formDataSetFile"
            | "__formDataEventCreate" => P::NativeIdentity,
            "__inputDataTransferTypesRemember" => P::Identity,
            "__inputEventConvertView" => P::Identity,
            "__navigationCreateNavigateEvent" | "__navigationCreateEntryChangeEvent" => {
                P::NativeIdentity
            }
            member if member.starts_with("__navigation") => P::Identity,
            "__domInvoke" if receiver == 0 => P::Forwarded {
                receiver: 0,
                member: 1,
                arguments: 2,
            },
            _ => P::Values,
        }
    }
    fn ObjectTraceSource(
        &self,
    ) -> Option<Rc<dyn javascript::javascript_runtime::HostObjectTraceSource>> {
        Some(self.host_graph.clone())
    }

    fn PrepareInvocation(&mut self, call: &HostCall<'_>) -> Option<HostContinuation> {
        if call.symbol == HostSymbol::kNone && call.operation == HostOperation::kGet {
            if let Some(get) = self
                .native_abort
                .PrepareInputIndexedGet(call.receiver, call.member)
            {
                return Some(get);
            }
        }
        // Reads never dispatch JS or emit a mutation through this seam. Leave
        // their receiver validation to get(), instead of looking up every DOM
        // Proxy receiver once here and again in the actual property read.
        if call.symbol != HostSymbol::kNone || call.operation == HostOperation::kGet {
            return None;
        }
        // Ordinary properties are resolved by the JavaScript engine. Do not
        // prepare a native mutation until an installed IDL setter is selected.
        if call.operation == HostOperation::kSet && self.UsesOrdinaryNodeProperties(call.receiver) {
            return None;
        }
        let (id, member) = if call.operation == HostOperation::kCall
            && call.receiver == 0
            && call.member == "__domInvoke"
        {
            if let [HostValue::Object(o), HostValue::String(n), ..] = call.arguments {
                (o.id, n.as_str())
            } else {
                return None;
            }
        } else {
            (call.receiver, call.member)
        };
        if call.operation == HostOperation::kCall {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            if let Some(continuation) = self.PrepareRangeSelection(id, member, arguments) {
                return Some(continuation);
            }
            if let Some(continuation) = self.PrepareTextControlSelection(id, member, arguments) {
                return Some(continuation);
            }
            if let Some(continuation) = self.PrepareEditorCommand(id, member, arguments) {
                return Some(continuation);
            }
        }
        if call.operation == HostOperation::kCall && member == "__inputEventConvertView" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let view = arguments.first().cloned().unwrap_or_default();
            let current = self
                .services
                .host
                .resolve_event_target
                .as_ref()
                .and_then(|resolve| resolve(0))
                .and_then(|target| target.AsAbstractView())
                .unwrap_or_else(|| self.ListenerTarget(0) as _);
            let convert = self.services.host.convert_window_view.clone();
            let bindings = self.native_abort.clone();
            return Some(Box::new(move |runtime| {
                let global = runtime.GlobalObject();
                let owner = if runtime.IsSameObject(&view, &global) {
                    Some(current)
                } else {
                    convert.as_ref().and_then(|convert| convert(&view, runtime))
                };
                match owner {
                    Some(owner) => Self::value(bindings.BindInputView(owner, view)),
                    None => Self::type_error("UIEvent view requires a Window"),
                }
            }));
        }
        if call.operation == HostOperation::kCall
            && matches!(member, "__frameContentWindow" | "__frameContentDocument")
        {
            // HTMLFrameOwnerElement::contentWindow/contentDocument. Release
            // the bindings and DOM borrows before lazily initializing the
            // child's LocalWindowProxy (which can execute binding bootstrap).
            let valid = self.node(id).is_some_and(|index| {
                self.document
                    .borrow()
                    .GetDocument()
                    .Node(index)
                    .IsHTMLElement("iframe")
            });
            if !valid {
                return Some(Box::new(|_| Self::type_error("Illegal invocation")));
            }
            let property = if member == "__frameContentWindow" {
                FrameProperty::ContentWindow
            } else {
                FrameProperty::ContentDocument
            };
            let getter = self.services.host.read_frame.clone();
            return Some(Box::new(move |runtime| {
                getter.as_ref().map_or_else(
                    || Self::value(HostValue::Null(JavaScriptNull)),
                    |getter| getter(id, property, runtime),
                )
            }));
        }
        if call.operation == HostOperation::kCall
            && (member.starts_with("__abort")
                || member.starts_with("__closeWatcher")
                || member.starts_with("__navigation")
                || member.starts_with("__trustedTypes")
                || member.starts_with("__formData")
                || (member.starts_with("__inputDataTransfer")
                    && member != "__inputDataTransferSetDragImage")
                || member == "__eventTargetCreate")
            && !matches!(
                member,
                "__abortGetAttributeListener" | "__abortSetAttributeListener"
            )
        {
            let args = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            return self.native_abort.Prepare(id, member, args);
        }
        if id == 0 && call.operation == HostOperation::kCall && member == "__createDOMException" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let message = arguments.first()?.clone();
            let name = arguments.get(1)?.clone();
            if !matches!(&message, HostValue::String(_) | HostValue::String16(_))
                || !matches!(&name, HostValue::String(_) | HostValue::String16(_))
            {
                return None;
            }
            let value = self.CreateNativeDOMExceptionValues(name.clone(), message.clone());
            return Some(Box::new(move |runtime| {
                runtime.CaptureExceptionStack(&value, &name, &message)
            }));
        }
        if call.operation == HostOperation::kCall && member == "__domCreateGetter" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            return Some(Self::PrepareGetter(arguments));
        }
        if call.operation == HostOperation::kCall && member == "__ceIsProxy" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let value = arguments.first().cloned().unwrap_or_default();
            return Some(Box::new(move |runtime| {
                Self::value(HostValue::Boolean(runtime.IsProxy(&value)))
            }));
        }
        if call.operation == HostOperation::kCall && member == "__ceReportException" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let thrown = arguments.first().cloned().unwrap_or_default();
            return Some(Box::new(move |runtime| runtime.ReportException(&thrown)));
        }
        // This internal getter only reads the tree. Validate its receiver in
        // call(), without a second node lookup or preparing a mutation emitter.
        if call.operation == HostOperation::kCall
            && matches!(
                member,
                "__sibling" | "__domHasInterface" | "__domAttributeGetter"
            )
        {
            return None;
        }
        // cpp: webapi/dom_bindings.cc:634-653
        // Synthetic dispatch can call back into this host. Prepare under the
        // host borrow, then enter Interaction on the active runtime after it
        // has been released (including Window's delegating host borrow).
        if call.operation == HostOperation::kCall && member == "dispatchEvent" {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            return Some(self.PrepareSyntheticDispatch(id, arguments));
        }
        // Preserve call routing: collections/bound events/invalid objects must
        // go through Invoke; source Node(receiver) precedes focus/blur.
        // cpp: webapi/dom_bindings.cc:773-774
        if self.node(id).is_none() {
            return None;
        }
        // Attribute writes only need the two-phase continuation when applying
        // the mutation can reenter JavaScript. Blink similarly skips mutation
        // observer record delivery when no observer has attribute interest.
        // Keep image request attributes on the scoped path because a cached
        // resource may synchronously dispatch its completion event.
        if call.operation == HostOperation::kCall
            && matches!(member, "setAttribute" | "removeAttribute")
            && !self
                .document
                .borrow()
                .GetDocument()
                .HasMutationObserversOfType(dom::mutation_observers::MutationType::Attributes)
            && !self
                .document
                .borrow()
                .GetDocument()
                .HasCustomElementDefinitions()
        {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let resource_request = arguments.first().is_some_and(|value| {
                matches!(value, HostValue::String(name) if matches!(name.to_ascii_lowercase().as_str(), "src" | "srcset" | "sizes" | "referrerpolicy"))
            });
            if !resource_request {
                return None;
            }
        }
        // cpp: webapi/dom_bindings.cc:818-821
        // Copy callback out of bindings
        //
        // caller executes only after host/window/DOM borrows are released.
        if call.operation == HostOperation::kCall && matches!(member, "focus" | "blur") {
            let handler = self.change_focus.clone();
            let focus = member == "focus";
            return Some(Box::new(move |runtime| {
                if let Some(handler) = handler {
                    handler(id, focus, runtime);
                }
                HostResult::default()
            }));
        }
        if call.operation == HostOperation::kCall
            && matches!(member, "checkValidity" | "reportValidity")
        {
            let allowed = {
                let owner = self.document.borrow();
                let d = owner.GetDocument();
                d.FindNodeById(id).is_some_and(|i| {
                    d.FormControlClass(i).is_some()
                        || d.ElementInternalsTargetIsFormAssociated(i)
                        || d.Node(i).IsHTMLElement("form")
                })
            };
            if !allowed {
                return Some(Box::new(|_| Self::type_error("Illegal invocation")));
            }
            let handler = self.services.host.validate_form.clone();
            let report = member == "reportValidity";
            return Some(Box::new(move |runtime| match handler {
                Some(handler) => match handler(id, report, runtime) {
                    Ok(value) => Self::value(HostValue::Boolean(value)),
                    Err(error) => {
                        Self::type_error(&format!("Form validation service required: {error:?}"))
                    }
                },
                None => Self::type_error("Native validation dispatcher required"),
            }));
        }
        if call.operation == HostOperation::kCall && matches!(member, "submit" | "requestSubmit") {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            let submitter = arguments.first().and_then(|value| {
                if let HostValue::Object(object) = value {
                    Some(object.id)
                } else {
                    None
                }
            });
            let handler = self.services.host.submit_form.clone();
            let dispatch_event = member == "requestSubmit";
            return Some(Box::new(move |runtime| {
                if let Some(handler) = handler {
                    handler(id, submitter, dispatch_event, runtime);
                }
                HostResult::default()
            }));
        }
        let custom_elements_active = self
            .document
            .borrow()
            .GetDocument()
            .HasCustomElementDefinitions();
        // CEReactions is an attribute of the selected IDL accessor, not of the
        // general host setter bridge (checked/indeterminate/data have no scope).
        let has_reaction_scope = member != "__domAttributeSet" || {
            let arguments = if call.receiver == 0 && call.member == "__domInvoke" {
                &call.arguments[2..]
            } else {
                call.arguments
            };
            matches!(arguments.get(3), Some(HostValue::Boolean(true)))
        };
        let reactions = (custom_elements_active && has_reaction_scope)
            .then(|| {
                self.method_wrappers
                    .get("__customElementReactions")
                    .cloned()
            })
            .flatten();
        let emitter = match self.scoped_emit_mutation.clone() {
            Some(emitter) => emitter,
            None if reactions.is_some() => {
                let emitter = self.emit_mutation.clone();
                Rc::new(
                    move |mutation: &DOMMutation, _: &mut dyn JavaScriptHostRuntime| {
                        (emitter.borrow_mut())(mutation);
                        Ok(())
                    },
                )
            }
            None => return None,
        };
        let emits = match call.operation {
            HostOperation::kSet => matches!(
                member,
                "checked"
                    | "files"
                    | "textContent"
                    | "data"
                    | "nodeValue"
                    | "innerHTML"
                    | "value"
                    | "id"
                    | "className"
            ),
            HostOperation::kCall => matches!(
                member,
                "__domAttributeSet"
                    | "createHTMLDocument"
                    | "cloneNode"
                    | "importNode"
                    | "createElement"
                    | "createElementNS"
                    | "createTextNode"
                    | "createComment"
                    | "createDocumentFragment"
                    | "attachShadow"
                    | "appendChild"
                    | "insertBefore"
                    | "removeChild"
                    | "setStyle"
                    | "setAttribute"
                    | "setAttributeNS"
                    | "removeAttribute"
                    | "__setSlotName"
                    | "__canvasSetDimension"
                    | "assign"
                    | "showModal"
                    | "close"
                    | "showPopover"
                    | "hidePopover"
                    | "requestFullscreen"
                    | "exitFullscreen"
            ),
            _ => false,
        };
        if !emits {
            return None;
        }
        // Each of these source Set/Call branches checks arguments, constructs
        // one mutation, emits it, then returns a value independent of the DOM.
        // Reuse that exact argument/return-value logic; no tree mutation or
        // connected-subtree preparation occurs under the bindings borrow.
        let mutation = Rc::new(RefCell::new(None));
        let pending = mutation.clone();
        let original = std::mem::replace(
            &mut self.emit_mutation,
            Rc::new(RefCell::new(Box::new(move |m: &DOMMutation| {
                assert!(
                    pending.borrow().is_none(),
                    "a DOM host invocation emits one mutation"
                );
                *pending.borrow_mut() = Some(m.clone());
            }))),
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.Invoke(call)));
        self.emit_mutation = original;
        let result = result.unwrap_or_else(|error| std::panic::resume_unwind(error));
        let mutation = mutation.borrow_mut().take();
        let catches_errors = matches!(
            member,
            "attachShadow"
                | "appendChild"
                | "insertBefore"
                | "removeChild"
                | "requestFullscreen"
                | "exitFullscreen"
                | "showModal"
                | "close"
                | "showPopover"
                | "hidePopover"
        );
        let popup_method = matches!(
            member,
            "showModal" | "close" | "showPopover" | "hidePopover"
        );
        let document = self.document.clone();
        let reaction_document = {
            let owner = document.borrow();
            HostValue::Object(HostObjectRef {
                id: owner.GetDocument().Node(owner.GetDocument().Root()).Id(),
            })
        };
        Some(Box::new(move |runtime| {
            if reactions.is_some() {
                document
                    .borrow_mut()
                    .GetDocumentMut()
                    .PushCustomElementReactionScope();
            }
            let mut outcome = result;
            if let Some(mutation) = mutation {
                let emitted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    emitter(&mutation, runtime)
                }));
                match emitted {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) if catches_errors => {
                        outcome = if popup_method {
                            Self::popup_error(&error)
                        } else {
                            Self::type_error(error.message)
                        };
                    }
                    Ok(Err(error)) => std::panic::panic_any(error),
                    Err(error) if catches_errors => {
                        if let Some(error) = error.downcast_ref::<std::io::Error>() {
                            outcome = Self::type_error(&error.to_string());
                        } else {
                            std::panic::resume_unwind(error);
                        }
                    }
                    Err(error) => std::panic::resume_unwind(error),
                }
            }
            dom::attribute_lifecycle::DrainAttributeLifecycle(&document);
            if let Some(callback) = reactions {
                let invoked = runtime.Call(&callback, &HostValue::default(), &[reaction_document]);
                let popped = document
                    .borrow_mut()
                    .GetDocumentMut()
                    .PopCustomElementReactionScope();
                assert!(popped, "CEReactions scope must be drained before pop");
                if let Some(error) = invoked.exception {
                    return HostResult {
                        exception: Some(error),
                        ..HostResult::default()
                    };
                }
            }
            outcome
        }))
    }
    // cpp: webapi/dom_bindings.cc:243-264
    fn GlobalNames(&self) -> Vec<String> {
        GLOBAL_NAMES.iter().map(|name| (*name).into()).collect()
    }

    fn InvokeWithRuntime(
        &mut self,
        call: &HostCall<'_>,
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        struct WorldScope {
            slot: Rc<RefCell<Option<Rc<style::ProductionCSSDOMWrapperWorld>>>>,
            previous: Option<Rc<style::ProductionCSSDOMWrapperWorld>>,
        }
        impl Drop for WorldScope {
            fn drop(&mut self) {
                *self.slot.borrow_mut() = self.previous.take();
            }
        }
        struct CallbackScope {
            slot: Rc<RefCell<Option<javascript::javascript_runtime::JavaScriptCallbackContext>>>,
            previous: Option<javascript::javascript_runtime::JavaScriptCallbackContext>,
        }
        impl Drop for CallbackScope {
            fn drop(&mut self) {
                *self.slot.borrow_mut() = self.previous.take();
            }
        }
        let slot = self.callback_context.clone();
        let previous = slot.borrow_mut().replace(runtime.CallbackContext());
        let _callback_scope = CallbackScope { slot, previous };
        let slot = self.cssom_world.clone();
        // This binding is installed only for the actual main-world ScriptState;
        // no isolated-world constructor is admitted by BindRuntime.
        let previous = slot
            .borrow_mut()
            .replace(Rc::new(style::ProductionCSSDOMWrapperWorld {
                identity: runtime.RealmIdentity(),
                world_id: runtime.CallbackContext().WorldId(),
            }));
        let _scope = WorldScope { slot, previous };
        self.Invoke(call)
    }
    fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
        if self.DocumentHostObjectId() == Some(call.receiver)
            && call.operation == HostOperation::kGet
            && matches!(
                call.member,
                "URL" | "documentURI" | "domain" | "location" | "cookie"
            )
        {
            // An old Document remains in its original namespace after the
            // Window's current Document changes. Its URL and detached-Window
            // state must not be read from the new Window's equal native id.
            return self.get(call.receiver, call.member);
        }
        // Blink installs regular IDL attributes/methods on interface prototypes.
        // The wrapper's existing Reflect path must select the own/inherited
        // descriptor and carry the actual ECMAScript receiver. Only genuine
        // exotic records, collections, bound events and globals use projection.
        if matches!(call.operation, HostOperation::kGet | HostOperation::kSet)
            && (self.UsesOrdinaryNodeProperties(call.receiver))
        {
            return Self::unhandled();
        }
        if call.symbol != HostSymbol::kNone {
            if call.operation == HostOperation::kGet
                && call.symbol == HostSymbol::kIterator
                && (self.collections.borrow().contains_key(&call.receiver)
                    || self.iterators.borrow().contains_key(&call.receiver))
            {
                return Self::value(HostValue::Method(HostMethodRef {
                    receiver: call.receiver,
                    name: "@@iterator".into(),
                }));
            }
            return Self::unhandled();
        }
        match call.operation {
            HostOperation::kGet => self.get(call.receiver, call.member),
            HostOperation::kSet => self.set(call.receiver, call.member, call.arguments),
            HostOperation::kCall => self.call(call.receiver, call.member, call.arguments),
            HostOperation::kConstruct => Self::type_error("DOM constructors are not installed yet"),
        }
    }

    fn UsesOrdinaryProperties(&self, id: HostObjectId) -> bool {
        if self.DocumentHostObjectId() == Some(id) {
            return false;
        }
        // Blink's ordinary IDL Node/Element wrappers use their installed
        // prototype descriptors. Collections and other exotic records still
        // require host property interception.
        (self.native_record_classes.borrow().contains_key(&id)
            && !self.native_abort.IsInputIndexedObject(id))
            || self.webgl_contexts.borrow().contains_key(&id)
            || self.UsesOrdinaryNodeProperties(id)
    }

    fn PrototypeFor(&mut self, id: HostObjectId) -> HostResult {
        if let Some(name) = self.services.EventInterfaceName(id) {
            return self
                .prototypes
                .get(&name)
                .cloned()
                .map_or_else(Self::unhandled, |value| {
                    Self::value(HostValue::JavaScriptValue(value))
                });
        }
        if let Some(name) = self.native_record_classes.borrow().get(&id) {
            return self
                .prototypes
                .get(*name)
                .cloned()
                .map_or_else(Self::unhandled, |value| {
                    Self::value(HostValue::JavaScriptValue(value))
                });
        }
        if self.webgl_contexts.borrow().contains_key(&id) {
            let prototype = self
                .webgl_contexts
                .borrow()
                .get(&id)
                .map(webgl::WebGLRenderingContextState::ContextType)
                .map_or("WebGLRenderingContext", |kind| match kind {
                    canvas::CanvasContextType::WebGL2 => "WebGL2RenderingContext",
                    _ => "WebGLRenderingContext",
                });
            return self
                .prototypes
                .get(prototype)
                .cloned()
                .map_or_else(Self::unhandled, |value| {
                    Self::value(HostValue::JavaScriptValue(value))
                });
        }
        if let Some(collection) = self.collections.borrow().get(&id) {
            let name = if collection.kind == "options" {
                "HTMLOptionsCollection"
            } else if matches!(
                collection.kind.as_str(),
                "snapshot" | "childNodes" | "labels"
            ) {
                "NodeList"
            } else {
                "HTMLCollection"
            };
            return self
                .prototypes
                .get(name)
                .cloned()
                .map_or_else(Self::unhandled, |value| {
                    Self::value(HostValue::JavaScriptValue(value))
                });
        }
        // cpp: webapi/dom_bindings.cc:1043-1094
        let document = self.document.borrow();
        let document = document.GetDocument();
        let Some(index) = document.FindNodeById(id) else {
            return Self::unhandled();
        };
        let node = document.Node(index);
        let prototype = match node.Type() {
            DOMNodeType::kDocument => "Document",
            DOMNodeType::kElement => match node.Namespace() {
                DOMNamespace::kHTML => match node.Name() {
                    "input" => "HTMLInputElement",
                    "textarea" => "HTMLTextAreaElement",
                    "button" => "HTMLButtonElement",
                    "form" => "HTMLFormElement",
                    "select" => "HTMLSelectElement",
                    "option" => "HTMLOptionElement",
                    "label" => "HTMLLabelElement",
                    "details" => "HTMLDetailsElement",
                    "pre" | "listing" | "xmp" => "HTMLPreElement",
                    "dialog" => "HTMLDialogElement",
                    "script" => "HTMLScriptElement",
                    "style" => "HTMLStyleElement",
                    "link" => "HTMLLinkElement",
                    "meta" => "HTMLMetaElement",
                    "iframe" => "HTMLIFrameElement",
                    "img" => "HTMLImageElement",
                    "canvas" => "HTMLCanvasElement",
                    "template" => "HTMLTemplateElement",
                    "slot" => "HTMLSlotElement",
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "HTMLHeadingElement",
                    "div" => "HTMLDivElement",
                    "li" => "HTMLLIElement",
                    "a" => "HTMLAnchorElement",
                    "video" => "HTMLVideoElement",
                    "audio" => "HTMLAudioElement",
                    _ => "HTMLElement",
                },
                DOMNamespace::kSVG => match node.Name() {
                    "path" => "SVGPathElement",
                    _ => "SVGElement",
                },
                DOMNamespace::kMathML => "MathMLElement",
                DOMNamespace::kNone => "Element",
            },
            DOMNodeType::kText => "Text",
            DOMNodeType::kComment => "Comment",
            DOMNodeType::kProcessingInstruction => "ProcessingInstruction",
            DOMNodeType::kDocumentFragment if document.ShadowHost(index).is_some() => "ShadowRoot",
            DOMNodeType::kDocumentFragment => "DocumentFragment",
            _ => "Node",
        };
        self.prototypes
            .get(prototype)
            .cloned()
            .map_or_else(Self::unhandled, |value| {
                Self::value(HostValue::JavaScriptValue(value))
            })
    }
}

fn is_method(node_type: DOMNodeType, member: &str) -> bool {
    if member == "isEqualNode" {
        return true;
    }
    if matches!(member, "addEventListener" | "removeEventListener") {
        return matches!(node_type, DOMNodeType::kDocument | DOMNodeType::kElement);
    }
    if matches!(
        member,
        "querySelector" | "querySelectorAll" | "getElementsByTagName" | "getElementsByClassName"
    ) {
        return matches!(
            node_type,
            DOMNodeType::kDocument | DOMNodeType::kElement | DOMNodeType::kDocumentFragment
        );
    }
    if matches!(member, "matches" | "closest") {
        return node_type == DOMNodeType::kElement;
    }
    if matches!(
        member,
        "setPointerCapture" | "releasePointerCapture" | "hasPointerCapture"
    ) {
        return node_type == DOMNodeType::kElement;
    }
    if member == "getElementById" {
        return node_type == DOMNodeType::kDocument;
    }
    if member == "requestFullscreen" {
        return node_type == DOMNodeType::kElement;
    }
    if member == "exitFullscreen" {
        return node_type == DOMNodeType::kDocument;
    }
    if matches!(
        member,
        "getAttribute" | "hasAttribute" | "setAttribute" | "setAttributeNS" | "removeAttribute"
    ) {
        return node_type == DOMNodeType::kElement;
    }
    false
}

// cpp: webapi/dom_bindings.cc:189-197
fn is_event_handler(name: &str) -> bool {
    matches!(
        name,
        "onload"
            | "onerror"
            | "onabort"
            | "onnavigate"
            | "onnavigatesuccess"
            | "onnavigateerror"
            | "oncurrententrychange"
            | "ondispose"
            | "oninput"
            | "onchange"
            | "onclick"
            | "onkeydown"
            | "onkeyup"
            | "onkeypress"
            | "onfocus"
            | "onblur"
            | "onsubmit"
            | "onreset"
            | "onmousedown"
            | "onmouseup"
            | "onmousemove"
            | "onreadystatechange"
            | "onfullscreenchange"
            | "onfullscreenerror"
    )
}

fn is_character_data(node_type: DOMNodeType) -> bool {
    matches!(
        node_type,
        DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
    )
}

/// HTML's non-negative-integer parser used by canvas width/height reflection.
fn parse_canvas_dimension(input: &str) -> Option<u32> {
    canvas::ParseCanvasDimension(input)
}

fn webgl_f32_values(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect(),
    )
}

fn webgl_i32_values(bytes: &[u8]) -> Option<Vec<i32>> {
    if bytes.len() % std::mem::size_of::<i32>() != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|chunk| i32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect(),
    )
}

fn webgl_u32_values(bytes: &[u8]) -> Option<Vec<u32>> {
    if bytes.len() % std::mem::size_of::<u32>() != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect(),
    )
}

impl DOMJavaScriptBindings {
    fn StoreCollection(&self, id: HostObjectId, collection: NodeCollection) {
        self.host_graph.Collection(id, &collection);
        self.collections.borrow_mut().insert(id, collection);
    }
    fn StaticNodeList(&self, nodes: Vec<HostObjectId>) -> HostObjectRef {
        let id = self.allocate_collection_id();
        self.StoreCollection(
            id,
            NodeCollection {
                root: 0,
                kind: "snapshot".into(),
                query: String::new(),
                snapshot: nodes,
                membership_cache: RefCell::new(None),
            },
        );
        HostObjectRef { id }
    }
    /// DOMException.cc:192-208,214-217,264-274. Native construction uses the installed
    /// prototype identity, never the replaceable global DOMException function.
    pub(super) fn CreateNativeDOMException(&self, name: &str, message: &str) -> HostValue {
        self.CreateNativeDOMExceptionValues(
            HostValue::String(name.into()),
            HostValue::String(message.into()),
        )
    }
    fn CreateNativeDOMExceptionValues(&self, name: HostValue, message: HostValue) -> HostValue {
        Self::CreateDOMExceptionRecord(
            &self.next_collection_id,
            &self.native_record_classes,
            &self.records,
            &self.host_graph,
            name,
            message,
        )
    }
    fn CreateDOMExceptionRecord(
        next_id: &Cell<u64>,
        classes: &RefCell<HashMap<u64, &'static str>>,
        records: &RefCell<HashMap<u64, HashMap<String, HostValue>>>,
        graph: &DOMHostGraph,
        name: HostValue,
        message: HostValue,
    ) -> HostValue {
        let store_record = |id, fields| {
            graph.Record(id, &fields);
            records.borrow_mut().insert(id, fields);
        };
        let decoded_name = match &name {
            HostValue::String(value) => Some(value.clone()),
            HostValue::String16(value) => String::from_utf16(value).ok(),
            _ => unreachable!("DOMException name was converted by Web IDL"),
        };
        let code = match decoded_name.as_deref().unwrap_or("") {
            "IndexSizeError" => 1,
            "HierarchyRequestError" => 3,
            "WrongDocumentError" => 4,
            "InvalidCharacterError" => 5,
            "NoModificationAllowedError" => 7,
            "NotFoundError" => 8,
            "NotSupportedError" => 9,
            "InUseAttributeError" => 10,
            "InvalidStateError" => 11,
            "SyntaxError" => 12,
            "InvalidModificationError" => 13,
            "NamespaceError" => 14,
            "InvalidAccessError" => 15,
            "TypeMismatchError" => 17,
            "SecurityError" => 18,
            "NetworkError" => 19,
            "AbortError" => 20,
            "URLMismatchError" => 21,
            "QuotaExceededError" => 22,
            "TimeoutError" => 23,
            "InvalidNodeTypeError" => 24,
            "DataCloneError" => 25,
            _ => 0,
        };
        let id = {
            let id = next_id.get();
            next_id.set(id + 1);
            id
        };
        classes.borrow_mut().insert(id, "DOMException");
        store_record(
            id,
            HashMap::from([
                ("name".into(), name),
                ("message".into(), message),
                ("code".into(), HostValue::Number(f64::from(code))),
            ]),
        );
        HostValue::Object(HostObjectRef { id })
    }
    fn StoreRecord(&self, id: HostObjectId, fields: HashMap<String, HostValue>) {
        self.host_graph.Record(id, &fields);
        self.records.borrow_mut().insert(id, fields);
    }
    fn allocate_collection_id(&self) -> HostObjectId {
        let id = self.next_collection_id.get();
        self.next_collection_id.set(id + 1);
        id
    }
    // cpp: webapi/dom_bindings.cc:990-1006
    fn collection(&self, root: HostObjectId, kind: &str, query: &str) -> HostObjectRef {
        if kind != "snapshot" {
            if let Some(id) = self
                .live_collection_ids
                .borrow()
                .get(&root)
                .and_then(|kinds| kinds.get(kind))
                .and_then(|queries| queries.get(query))
            {
                return HostObjectRef { id: *id };
            }
        }
        let mut collection = NodeCollection {
            root,
            kind: kind.into(),
            query: query.into(),
            snapshot: Vec::new(),
            membership_cache: RefCell::new(None),
        };
        if kind == "snapshot" {
            collection.kind = "query".into();
            collection.snapshot = self.collection_nodes(&collection).into_owned();
            collection.kind = "snapshot".into();
        }
        let id = self.allocate_collection_id();
        self.StoreCollection(id, collection);
        if kind != "snapshot" {
            self.live_collection_ids
                .borrow_mut()
                .entry(root)
                .or_default()
                .entry(kind.to_owned())
                .or_default()
                .insert(query.to_owned(), id);
        }
        HostObjectRef { id }
    }
    // cpp: webapi/dom_bindings.cc:1008-1037
    fn collection_nodes<'a>(&self, collection: &'a NodeCollection) -> CollectionNodes<'a> {
        if collection.kind == "snapshot" {
            return CollectionNodes::Borrowed(&collection.snapshot);
        }
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let cache_key = matches!(
            collection.kind.as_str(),
            "childNodes"
                | "children"
                | "getElementsByTagName"
                | "getElementsByClassName"
                | "options"
                | "labels"
        )
        .then(|| document.CollectionMembershipKey())
        .flatten();
        if let Some(key) = cache_key {
            if let Some(cache) = collection
                .membership_cache
                .borrow()
                .as_ref()
                .filter(|cache| cache.key == key)
            {
                return CollectionNodes::Shared(cache.ids.clone());
            }
        }
        let Some(root) = document.FindNodeById(collection.root) else {
            return CollectionNodes::Owned(Vec::new());
        };
        if collection.kind == "formControls" {
            return CollectionNodes::Shared(document.ListedFormControlNodeIds(root));
        }
        if collection.kind == "query" {
            let indices = self
                .selector_service
                .borrow_mut()
                .QueryAll(document, root, &collection.query)
                .expect("internal snapshot selector is validated before construction");
            return CollectionNodes::Owned(
                indices.into_iter().map(|i| document.Node(i).Id()).collect(),
            );
        }
        fn visit(
            document: &Document,
            index: usize,
            collection: &NodeCollection,
            result: &mut Vec<HostObjectId>,
        ) {
            for &child in document.Node(index).Children() {
                let node = document.Node(child);
                let element = node.Type() == DOMNodeType::kElement;
                let matched = match collection.kind.as_str() {
                    "childNodes" => true,
                    "children" => element,
                    "getElementsByTagName" => {
                        element
                            && (collection.query == "*"
                                || node.Name() == collection.query
                                || (node.Namespace() == DOMNamespace::kHTML
                                    && node.Name() == collection.query.to_ascii_lowercase()))
                    }
                    "getElementsByClassName" => element && has_classes(node, &collection.query),
                    _ => false,
                };
                if matched {
                    result.push(node.Id());
                }
                if collection.kind != "childNodes" && collection.kind != "children" {
                    visit(document, child, collection, result);
                }
            }
        }
        let mut result = Vec::new();
        if collection.kind == "options" {
            result = document.SelectOptionNodeIds(root);
        } else if collection.kind == "labels" {
            result = document
                .ElementInternalsLabels(root)
                .into_iter()
                .map(|label| document.Node(label).Id())
                .collect();
        } else {
            visit(document, root, collection, &mut result);
        }
        if let Some(key) = cache_key {
            let ids = Rc::new(result);
            *collection.membership_cache.borrow_mut() = Some(CollectionMembershipCache {
                key,
                ids: ids.clone(),
            });
            CollectionNodes::Shared(ids)
        } else {
            // Overflow, unknown kinds and transient query snapshots keep fresh traversal.
            collection.membership_cache.borrow_mut().take();
            CollectionNodes::Owned(result)
        }
    }
}
// cpp: webapi/dom_bindings.cc:199-213
fn has_classes(node: &dom::persistent_document::DOMNode, query: &str) -> bool {
    let Some(classes) = node.FindAttribute("class") else {
        return false;
    };
    let mut any = false;
    for word in query
        .split(|c: char| matches!(c, ' ' | '\t'..='\r'))
        .filter(|s| !s.is_empty())
    {
        any = true;
        if !classes
            .value
            .split_ascii_whitespace()
            .any(|candidate| candidate == word)
        {
            return false;
        }
    }
    any
}

fn is_control(node: &dom::persistent_document::DOMNode) -> bool {
    node.IsHTMLElement("input") || node.IsHTMLElement("textarea") || node.IsHTMLElement("select")
}

// cpp: webapi/dom_bindings.cc:55-64
fn append_descendant_text(document: &Document, node: usize, output: &mut String) {
    let node = document.Node(node);
    if node.Type() == DOMNodeType::kText {
        output.push_str(node.Data());
        return;
    }
    for &child in node.Children() {
        append_descendant_text(document, child, output);
    }
}

// cpp: webapi/dom_bindings.cc:66-75
fn text_content(document: &Document, node: usize) -> String {
    if is_character_data(document.Node(node).Type()) {
        return document.Node(node).Data().into();
    }
    let mut output = String::new();
    append_descendant_text(document, node, &mut output);
    output
}

// cpp: webapi/dom_bindings.cc:77-85
fn append_escaped(text: &str, output: &mut String, attribute: bool) {
    for character in text.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' if !attribute => output.push_str("&gt;"),
            '"' if attribute => output.push_str("&quot;"),
            _ => output.push(character),
        }
    }
}

// cpp: webapi/dom_bindings.cc:87-92
fn is_void_html_element(name: &str) -> bool {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ]
    .contains(&name)
}

// cpp: webapi/dom_bindings.cc:94-130
fn serialize_node(document: &Document, index: usize, output: &mut String) {
    let node = document.Node(index);
    if node.IsCDATASection() && !document.IsInHTMLDocument(index) {
        output.push_str("<![CDATA[");
        output.push_str(node.Data());
        output.push_str("]]>");
        return;
    }
    if node.Type() == DOMNodeType::kText {
        append_escaped(node.Data(), output, false);
        return;
    }
    if node.Type() == DOMNodeType::kComment {
        output.push_str("<!--");
        output.push_str(node.Data());
        output.push_str("-->");
        return;
    }
    if matches!(
        node.Type(),
        DOMNodeType::kDocument | DOMNodeType::kDocumentFragment
    ) {
        for &child in node.Children() {
            serialize_node(document, child, output);
        }
        return;
    }
    if node.Type() != DOMNodeType::kElement {
        return;
    }
    output.push('<');
    output.push_str(node.Name());
    for attribute in node.Attributes() {
        output.push(' ');
        if !attribute.prefix.is_empty() {
            output.push_str(&attribute.prefix);
            output.push(':');
        }
        output.push_str(&attribute.local_name);
        output.push_str("=\"");
        append_escaped(&attribute.value, output, true);
        output.push('"');
    }
    output.push('>');
    if node.Namespace() == DOMNamespace::kHTML && is_void_html_element(node.Name()) {
        return;
    }
    let contents = document.TemplateContents(index).unwrap_or(index);
    for &child in document.Node(contents).Children() {
        serialize_node(document, child, output);
    }
    output.push_str("</");
    output.push_str(node.Name());
    output.push('>');
}

// cpp: webapi/dom_bindings.cc:132-138
fn inner_html(document: &Document, index: usize) -> String {
    let mut output = String::new();
    let contents = document.TemplateContents(index).unwrap_or(index);
    for &child in document.Node(contents).Children() {
        serialize_node(document, child, &mut output);
    }
    output
}

// DOM Parsing and Serialization: Element.outerHTML serializes the element
// itself, whereas innerHTML serializes only its child fragment.
fn outer_html(document: &Document, index: usize) -> String {
    let mut output = String::new();
    serialize_node(document, index, &mut output);
    output
}

// cpp: webapi/dom_bindings.cc:140-161
fn node_type_number(node_type: DOMNodeType) -> f64 {
    match node_type {
        DOMNodeType::kElement => 1.0,
        DOMNodeType::kText => 3.0,
        DOMNodeType::kProcessingInstruction => 7.0,
        DOMNodeType::kComment => 8.0,
        DOMNodeType::kDocument => 9.0,
        DOMNodeType::kDocumentType => 10.0,
        DOMNodeType::kDocumentFragment => 11.0,
    }
}

// cpp: webapi/dom_bindings.cc:163-191
fn node_name(node_type: DOMNodeType, name: &str, namespace: DOMNamespace) -> String {
    match node_type {
        DOMNodeType::kDocument => "#document".into(),
        DOMNodeType::kText => "#text".into(),
        DOMNodeType::kComment => "#comment".into(),
        DOMNodeType::kDocumentFragment => "#document-fragment".into(),
        DOMNodeType::kElement if namespace == DOMNamespace::kHTML => name.to_ascii_uppercase(),
        _ => name.into(),
    }
}

// cpp: webapi/dom_bindings.cc:35-53
fn dom_string_argument(argument: Option<&HostValue>) -> Option<String> {
    match argument? {
        HostValue::String(value) => Some(value.clone()),
        HostValue::String16(value) => Some(String::from_utf16_lossy(value)),
        HostValue::Boolean(value) => Some(value.to_string()),
        HostValue::Number(value) => Some(dom_number_string(*value)),
        HostValue::Null(_) => Some("null".into()),
        HostValue::Undefined(_) => Some("undefined".into()),
        _ => None,
    }
}

// std::ostringstream uses defaultfloat with six significant decimal digits.
// Round before selecting fixed/scientific notation, including exponent carry.
fn dom_number_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Infinity"
        } else {
            "Infinity"
        }
        .into();
    }
    let rounded = format!("{value:.5e}");
    let (mantissa, exponent) = rounded.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    if (-4..6).contains(&exponent) {
        let fixed = format!("{value:.precision$}", precision = (5 - exponent) as usize);
        if fixed.contains('.') {
            fixed.trim_end_matches('0').trim_end_matches('.').into()
        } else {
            fixed
        }
    } else {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!("{mantissa}e{exponent:+03}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DOMBootstrapSource;
    use dom::dom_mutation::ApplyDOMMutations;
    use dom::persistent_document::{DOMAttribute, DOMSlotAssignmentMode};
    use javascript::javascript_runtime::JavaScriptRuntime;
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;

    #[test]
    fn attach_shadow_publishes_tree_scope_and_slot_assignment_to_retained_dom() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<html><body><x-host id='host'><span id='light'></span></x-host>\
             <x-host id='manual-host'><span id='manual-light' slot='named'></span></x-host>\
             </body></html>",
        )));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                )
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        let script = runtime.Evaluate(
            &realm,
            r#"
                    function check(value) { if (!value) throw new Error('shadow owner'); }
                    const host = document.getElementById('host');
                    const light = document.getElementById('light');
                    const root = host.attachShadow({mode:'open'});
                    const slot = document.createElement('slot');
                    slot.setAttribute('name','named');
                    root.appendChild(slot);
                    light.setAttribute('slot','named');
                    check(host.shadowRoot === root && root.host === host);
                    check(root.parentNode === null && root.querySelector('slot') === slot);

                    const manualHost = document.getElementById('manual-host');
                    const manualRoot = manualHost.attachShadow({mode:'open',slotAssignment:'manual'});
                    const manualSlot = document.createElement('slot');
                    manualSlot.setAttribute('name','named');
                    manualRoot.appendChild(manualSlot);
                    check(manualRoot.slotAssignment === 'manual');
                "#,
            "attach-shadow.js",
        );
        assert!(script.Succeeded(), "{:?}", script.exception);

        let owner = document.borrow();
        let tree = owner.GetDocument();
        let host = (0..tree.NodeCount())
            .find(|&index| {
                tree.Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "host")
            })
            .unwrap();
        let light = (0..tree.NodeCount())
            .find(|&index| {
                tree.Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "light")
            })
            .unwrap();
        let root = tree.ShadowRoot(host).expect("native shadow root owner");
        assert_eq!(tree.ShadowHost(root), Some(host));
        assert_eq!(
            tree.OwnerShadowHost(tree.Node(root).Children()[0]),
            Some(host)
        );
        assert_eq!(
            tree.AssignedSlot(light),
            tree.Node(root).Children().first().copied()
        );

        let manual_host = (0..tree.NodeCount())
            .find(|&index| {
                tree.Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "manual-host")
            })
            .unwrap();
        let manual_light = (0..tree.NodeCount())
            .find(|&index| {
                tree.Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "manual-light")
            })
            .unwrap();
        let manual_root = tree.ShadowRoot(manual_host).unwrap();
        assert_eq!(
            tree.ShadowRootSlotAssignment(manual_root),
            Some(DOMSlotAssignmentMode::kManual)
        );
        assert_eq!(tree.AssignedSlot(manual_light), None);
    }

    #[test]
    fn blocking_listener_targets_report_only_live_non_passive_registrations() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<html><body><div id='passive'></div><div id='blocking'></div></body></html>",
        )));
        let blocking_id = {
            let owner = document.borrow();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .find_map(|index| {
                    document
                        .Node(index)
                        .FindAttribute("id")
                        .filter(|attribute| attribute.value == "blocking")
                        .map(|_| document.Node(index).Id())
                })
                .unwrap()
        };
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                )
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        assert!(runtime
            .Evaluate(
                &realm,
                r#"
                    const passiveWheel = () => {};
                    const blockingWheel = () => {};
                    document.getElementById('passive').addEventListener(
                        'wheel', passiveWheel, { passive: true });
                    document.getElementById('blocking').addEventListener(
                        'wheel', blockingWheel);
                "#,
                "blocking-wheel-targets.js",
            )
            .Succeeded());
        assert_eq!(
            bindings.borrow().BlockingListenerTargets("wheel"),
            vec![blocking_id]
        );
        assert!(runtime
            .Evaluate(
                &realm,
                "document.getElementById('blocking').removeEventListener('wheel', blockingWheel);",
                "remove-blocking-wheel-target.js",
            )
            .Succeeded());
        assert!(bindings
            .borrow()
            .BlockingListenerTargets("wheel")
            .is_empty());
        assert!(runtime
            .Evaluate(
                &realm,
                "window.addEventListener('wheel', blockingWheel);",
                "window-blocking-wheel-target.js",
            )
            .Succeeded());
        assert_eq!(bindings.borrow().BlockingListenerTargets("wheel"), vec![0]);
    }

    #[test]
    fn sibling_getters_follow_live_tree_without_creating_collections() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<html><body><div id='box'>alpha<!-- comment --><span id='left'></span>omega<b id='right'></b></div></body></html>",
        )));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                )
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        let collections = bindings.borrow().collections.borrow().len();
        assert!(runtime.Evaluate(&realm, r#"
            function check(value) { if (!value) throw new Error('sibling query'); }
            const box = document.getElementById('box');
            const left = document.getElementById('left'), right = document.getElementById('right');
            const first = box.firstChild, comment = first.nextSibling, text = left.nextSibling;
            check(first.previousSibling === null && first.previousElementSibling === null);
            check(comment.nodeType === 8 && comment.previousSibling === first && comment.nextSibling === left);
            check(first.nextElementSibling === left && text.previousElementSibling === left && text.nextElementSibling === right);
            check(left.previousElementSibling === null && left.nextElementSibling === right);
            check(right.previousSibling === text && right.nextSibling === null && right.nextElementSibling === null);
        "#, "sibling-query.js").Succeeded());
        assert_eq!(bindings.borrow().collections.borrow().len(), collections);
        assert!(runtime.Evaluate(&realm, r#"
            const inserted = document.createElement('i');
            check(inserted.nextSibling === null && inserted.previousSibling === null);
            box.insertBefore(inserted, left);
            check(comment.nextSibling === inserted && left.previousSibling === inserted && first.nextElementSibling === inserted);
            box.removeChild(inserted);
            check(inserted.previousSibling === null && inserted.nextElementSibling === null && comment.nextSibling === left);
            box.appendChild(left);
            check(left.previousSibling === right && right.nextElementSibling === left && left.nextSibling === null);
        "#, "sibling-mutation.js").Succeeded());
    }

    #[test]
    fn parsed_dom_properties_mutations_and_template_serialization_match_source() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            r#"<!doctype html><html><head></head><body><div id="box">one<!-- c -->two<span>three</span></div><input id="check" type="checkbox" checked value="seed"><textarea id="area">initial</textarea><select id="choice"><option>A</option><option selected value="b">B</option></select><img id="missing" src="x.png"><img id="blank"><template id="tpl"><span data-quote="&quot;&amp;&lt;&gt;">nested &amp;&lt;&gt;</span><!-- c --><br></template><svg id="vector"></svg></body></html>"#,
        )));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        let result = runtime.Evaluate(&realm, r#"
            function check(condition, message) { if (!condition) throw new Error(message); }
            check(document.documentElement.tagName === 'HTML' && document.head.tagName === 'HEAD' && document.body.tagName === 'BODY', 'document structure');
            check(document.activeElement === document.body && document.defaultView === globalThis && document.ownerDocument === null && document.textContent === null, 'document properties');
            const box = document.getElementById('box');
            check(box.parentNode === document.body && box.ownerDocument === document && box.namespaceURI === 'http://www.w3.org/1999/xhtml', 'node ownership');
            check(box.textContent === 'onetwothree' && box.nodeValue === null, 'descendant text excludes comments');
            const text = box.firstChild;
            text.data = 'changed';
            check(text.nodeValue === 'changed' && box.textContent === 'changedtwothree', 'character data mutation');
            box.nodeValue = 'ignored';
            check(box.textContent === 'changedtwothree', 'element nodeValue setter');
            box.textContent = 'new';
            check(box.firstChild === box.lastChild && box.firstChild.parentNode === box && box.innerHTML === 'new', 'replace child text');
            const control = document.getElementById('check');
            check(control.checked === true && control.value === 'seed', 'initial input state');
            control.checked = false; control.value = 'changed value';
            check(control.checked === false && control.hasAttribute('checked') && control.value === 'changed value' && control.getAttribute('value') === 'seed', 'live control state differs from attribute');
            const area = document.getElementById('area'), choice = document.getElementById('choice');
            check(area.value === 'initial' && choice.value === 'b', 'initial textarea/select');
            area.value = 'edited'; choice.value = 'chosen';
            check(area.value === 'edited' && choice.value === 'chosen' && area.textContent === 'initial', 'live textarea/select state');
            const missing = document.getElementById('missing'), blank = document.getElementById('blank');
            check(missing.complete === false && missing.naturalWidth === 0 && missing.naturalHeight === 0 && missing.currentSrc === 'x.png' && blank.complete === true, 'unloaded images');
            const template = document.getElementById('tpl');
            check(template.content.nodeType === 11 && template.content.ownerDocument === document && template.textContent === '', 'template ownership');
            check(template.innerHTML === '<span data-quote="&quot;&amp;&lt;>">nested &amp;&lt;&gt;</span><!-- c --><br>', 'template serialization');
            check(template.outerHTML === '<template id="tpl"><span data-quote="&quot;&amp;&lt;>">nested &amp;&lt;&gt;</span><!-- c --><br></template>', 'outerHTML serializes the element and template contents');
            check(document.getElementById('vector').outerHTML === '<svg id="vector"></svg>', 'SVG outerHTML serialization');
            check(document.getElementById('vector').namespaceURI === 'http://www.w3.org/2000/svg', 'SVG namespace');
            box.setAttribute('rounded', 1.23456789); box.setAttribute('scientific', 1000000); box.setAttribute('large-fixed', 100000); box.setAttribute('infinity', Infinity);
            check(box.getAttribute('rounded') === '1.23457' && box.getAttribute('scientific') === '1e+06' && box.getAttribute('large-fixed') === '100000' && box.getAttribute('infinity') === 'Infinity', 'source number formatting');
        "#, "properties.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        let check_id = {
            let mut owner = document.borrow_mut();
            owner.GetDocumentMut().SetImageResource(
                "x.png".into(),
                dom::ImageResourceMetadata {
                    id: 1,
                    natural_width: 17.0,
                    natural_height: 11.0,
                    ..Default::default()
                },
            );
            let tree = owner.GetDocument();
            tree.Node(find_element_by_id(tree, tree.Root(), "check").unwrap())
                .Id()
        };
        bindings.borrow_mut().SetFocusedNode(Some(check_id));
        let result = runtime.Evaluate(&realm,
            "check(document.activeElement === control && missing.complete && missing.naturalWidth === 17 && missing.naturalHeight === 11, 'focused node and loaded image')", "properties.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    #[test]
    fn snapshot_collection_reads_borrow_fixed_ids_without_copying() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML("<body></body>")));
        let bindings = DOMJavaScriptBindings::new(document, Box::new(|_| {}));
        let snapshot = NodeCollection {
            root: 0,
            kind: "snapshot".into(),
            query: String::new(),
            snapshot: (1..=1024).collect(),
            membership_cache: RefCell::new(None),
        };
        for _ in 0..16 {
            let nodes = bindings.collection_nodes(&snapshot);
            assert!(
                matches!(nodes, CollectionNodes::Borrowed(_)),
                "immutable snapshot must not materialize N ids per read"
            );
            assert_eq!(nodes.as_ptr(), snapshot.snapshot.as_ptr());
            assert_eq!(nodes.len(), 1024);
            assert_eq!(nodes[0], 1);
            assert_eq!(nodes[1023], 1024);
        }
        let live = NodeCollection {
            root: u64::MAX,
            kind: "children".into(),
            query: String::new(),
            snapshot: vec![7],
            membership_cache: RefCell::new(None),
        };
        assert!(
            matches!(bindings.collection_nodes(&live),CollectionNodes::Owned(ref nodes) if nodes.is_empty()),
            "live collection must recompute, never consume a fixed snapshot"
        );
    }
    #[test]
    fn snapshot_iterator_retains_detached_ids_and_sticky_done() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a></p><p id=b></p></main>",
        )));
        let emit = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
            const root=document.getElementById('root'), snap=root.querySelectorAll('p'), live=root.children;
            const a=snap[0],b=snap[1],iter=snap[Symbol.iterator]();
            if(iter.next().value!==a)throw Error('first iterator id');
            root.removeChild(a);root.removeChild(b);
            if(live.length!==0 || snap.length!==2 || snap.item(0)!==a || snap[1]!==b)throw Error('detached snapshot');
            if(iter.next().value!==b || !iter.next().done)throw Error('detached iterator');
            root.appendChild(a);root.appendChild(b);root.appendChild(document.createElement('p'));
            if(live.length!==3 || !iter.next().done || snap.length!==2)throw Error('done or live snapshot');
            if(Array.from(snap).map(n=>n.id).join('|')!=='a|b')throw Error('snapshot iterable order');
            // Only the static list retains this removed node. A native ID is
            // insufficient: StaticNodeList::Trace must preserve its wrapper.
            const retained=(()=>{
                const p=document.createElement('p');p.authorState=42;root.appendChild(p);
                const list=root.querySelectorAll('p:last-child');p.remove();return list;
            })();
            for(let i=0;i<40000;i++){const cycle={};cycle.self=cycle;}
            if(retained.length!==1 || retained[0].authorState!==42)throw Error('snapshot wrapper lifetime');
        "#,"snapshot.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
    // Artifact: insert inside dom_bindings.rs's existing tests module.
    #[test]
    fn live_membership_cache_reuses_only_matching_arena_revision_and_reads_fresh_names() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main><p class=one id=a></p><p id=b></p></main>",
        )));
        let bindings = DOMJavaScriptBindings::new(document.clone(), Box::new(|_| {}));
        let collection = NodeCollection {
            root: 1,
            kind: "getElementsByClassName".into(),
            query: "one".into(),
            snapshot: vec![],
            membership_cache: RefCell::new(None),
        };
        let CollectionNodes::Shared(first) = bindings.collection_nodes(&collection) else {
            panic!("cacheable live membership")
        };
        let CollectionNodes::Shared(second) = bindings.collection_nodes(&collection) else {
            panic!("repeat cacheable live membership")
        };
        assert!(Rc::ptr_eq(&first, &second));
        assert_eq!(first.len(), 1);
        {
            let mut owner = document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let root = tree.Root();
            let index = find_element_by_id(tree, root, "b").unwrap();
            tree.SetAttribute(
                index,
                DOMAttribute {
                    local_name: "class".into(),
                    value: "one".into(),
                    ..Default::default()
                },
            );
        }
        let CollectionNodes::Shared(third) = bindings.collection_nodes(&collection) else {
            panic!("updated cacheable membership")
        };
        assert!(!Rc::ptr_eq(&first, &third));
        assert_eq!(third.len(), 2);
        assert_eq!(first.len(), 1);
        {
            let mut owner = document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let root = tree.Root();
            let a = find_element_by_id(tree, root, "a").unwrap();
            tree.Remove(a);
        }
        assert_eq!(bindings.collection_nodes(&collection).len(), 1);
        let mut other = html::html_parser::ParseHTML(
            "<main><p class=one></p><p class=one></p><p class=one></p></main>",
        );
        document.borrow_mut().SwapDocument(other.GetDocumentMut());
        assert_eq!(
            bindings.collection_nodes(&collection).len(),
            3,
            "arena identity must prevent cross-document ID reuse"
        );
        document.borrow_mut().SwapDocument(other.GetDocumentMut());
        assert_eq!(
            bindings.collection_nodes(&collection).len(),
            1,
            "restored arena must recompute after foreign cache publication"
        );
    }
    #[test]
    fn live_membership_cache_preserves_mutation_between_reads_iterators_and_argument_coercion() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a name=first class=one></p><p id=b></p></main>",
        )));
        let emit = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
        function check(ok,message){if(!ok)throw Error(message)}
        const root=document.getElementById('root'),a=document.getElementById('a'),b=document.getElementById('b');
        const kids=root.children,tags=root.getElementsByTagName('P'),classes=root.getElementsByClassName('one');
        const snap=root.querySelectorAll('p'),nodes=root.childNodes;
        check(kids.length===2 && tags.length===2 && classes.length===1,'warm memberships');
        root.removeChild(a);check(kids.item(0)===b && kids.length===1 && classes.length===0,'remove between length/item');
        root.insertBefore(a,b);check(kids.item(0)===a && tags.item(1)===b,'insert current order');
        const iterator=kids[Symbol.iterator]();check(iterator.next().value===a,'iterator first');
        root.removeChild(b);const c=document.createElement('p');c.id='c';root.appendChild(c);
        check(iterator.next().value===c && iterator.next().done,'iterator membership mutation');
        root.appendChild(b);check(iterator.next().done,'sticky completed iterator');
        check(kids.item({valueOf(){root.removeChild(a);return 0;}})===c,'argument coercion changes membership before item');
        c.setAttribute('name','fresh');check(kids.namedItem('fresh')===c,'fresh name');
        c.setAttribute('name','new');check(kids.namedItem('fresh')===null && kids.namedItem('new')===c,'name changed after warm lookup');
        check(kids.namedItem({toString(){c.id='coerced';return 'coerced'}})===c,'name coercion side effect');
        b.className='one';check(classes.length===1 && classes[0]===b,'class change invalidation');
        const fragment=document.createDocumentFragment();fragment.appendChild(c);
        check(kids.length===1 && fragment.children.item(0)===c,'reparent roots');
        fragment.appendChild(document.createTextNode('text'));check(fragment.childNodes.length===2,'text child membership');
        // getElementsByTagName is an Element/Document operation, not a
        // DocumentFragment member. Warm the detached Element's live collection
        // and retain the original two-descendant assertion through ParentNode.
        const detachedTags=c.getElementsByTagName('P');check(detachedTags.length===0,'warm detached membership');
        c.appendChild(a);check(tags.length===1 && detachedTags.length===1 && fragment.querySelectorAll('p').length===2,'detached subtree insertion');
        check(snap.length===2 && snap[0]===a && snap[1]===b,'fixed snapshot identity retains detached nodes');
        root.textContent='new';check(kids.length===0 && nodes.length===1 && nodes[0].nodeType===3,'text replacement');
        let coercion=false;try{kids.item({valueOf(){throw Error('coercion')}})}catch(e){coercion=e.message==='coercion'}
        check(coercion,'coercion exception is not suppressed');
    "#,"live-membership-cache.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    // Insert in existing DOM binding tests. All existing mutation assertions retained.
    #[test]
    fn collection_webidl_receiver_conversion_exceptions_and_prototype_semantics() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a></p><p id=b></p></main>",
        )));
        let emit = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        assert!(runtime
            .Evaluate(&realm, DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
        function check(ok,label){if(!ok)throw Error(label)}
        function typeError(run,label){let seen=false;try{run()}catch(e){seen=e instanceof TypeError}check(seen,label)}
        const root=document.getElementById('root'),a=document.getElementById('a'),b=document.getElementById('b');
        const live=root.children, snap=root.querySelectorAll('p');
        const htmlItem=HTMLCollection.prototype.item,nodeItem=NodeList.prototype.item,named=HTMLCollection.prototype.namedItem;
        check(live.item===htmlItem && live.item===live.item && snap.item===nodeItem && htmlItem!==nodeItem,'per-interface prototype identity');
        check(htmlItem.name==='item' && htmlItem.length===1 && named.name==='namedItem' && named.length===1,'method descriptors');
        check(!Object.hasOwn(htmlItem,'prototype') && !Object.hasOwn(live,'item'),'nonconstructor prototype method');
        const d=Object.getOwnPropertyDescriptor(HTMLCollection.prototype,'item');check(d.writable && d.configurable && d.enumerable,'prototype method attributes');
        let coerced=0;const bad={valueOf(){coerced++;return 0},toString(){coerced++;return 'a'}};
        typeError(()=>htmlItem(bad),'detached receiver');
        typeError(()=>htmlItem.call(snap,bad),'wrong interface');
        typeError(()=>nodeItem.call(live,bad),'opposite wrong interface');
        typeError(()=>htmlItem.call(Object.create(HTMLCollection.prototype),bad),'prototype impostor');
        typeError(()=>named.call(snap,bad),'named wrong interface');
        check(coerced===0,'receiver errors precede argument coercion');
        typeError(()=>live.item(),'required item argument');typeError(()=>live.namedItem(),'required named argument');
        for(const value of [0,0.9,'0',null,undefined,NaN,Infinity,-Infinity,4294967296,-4294967296])check(live.item(value)===a,'unsigned long conversion '+value);
        check(live.item(4294967297)===b && live.item(-1)===null,'modulo and negative wrap');
        typeError(()=>live.item(1n),'BigInt ToNumber');typeError(()=>live.item(Symbol('index')),'Symbol ToNumber');
        let seenHint='';check(live.namedItem({[Symbol.toPrimitive](hint){seenHint=hint;b.id='fresh';return 'fresh'}})===b && seenHint==='string','DOMString hint and mutation');
        typeError(()=>live.namedItem(Symbol('name')),'primitive symbol DOMString');
        typeError(()=>live.namedItem({[Symbol.toPrimitive](){return Symbol('name')}}),'object symbol DOMString');
        let error=false;try{live.item({valueOf(){throw Error('coercion')}})}catch(e){error=e.message==='coercion' && e.stack.includes('collection-webidl.js')}
        check(error,'coercion failure and user source stack');
        const oldString=String;globalThis.String=()=>{throw Error('must not replace IDL intrinsic')};
        try{check(live.namedItem({toString(){return 'fresh'}})===b,'captured original String')}finally{globalThis.String=oldString}
        HTMLCollection.prototype.item=function(){return 'overridden'};
        check(live.item(0)==='overridden','prototype override');HTMLCollection.prototype.item=htmlItem;
        Object.defineProperty(live,'item',{value:()=>17,configurable:true});check(live.item(0)===17,'own override');delete live.item;
        Object.defineProperty(HTMLCollection.prototype,'item',{get(){throw Error('prototype getter')},configurable:true});
        let getter=false;try{live.item}catch(e){getter=e.message==='prototype getter' && e.stack.includes('collection-webidl.js')}
        check(getter,'Reflect getter error');Object.defineProperty(HTMLCollection.prototype,'item',d);
        let primitiveCalls=0;check(live.item({valueOf(){primitiveCalls++;root.removeChild(a);return 0}})===b && primitiveCalls===1,'coercion before membership lookup once');
        check(snap.item(0)===a && snap.item(1)===b,'fixed snapshot ID retention after coercion mutation');
    "#,"collection-webidl.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    #[test]
    fn selectors_and_collections_keep_live_identity_and_snapshot_semantics() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<main id=root><p id=a name=first class='one two'>text</p><!--comment--><p id=b class=two></p><section><span class=leaf></span></section><input id=field placeholder=hint><svg viewBox='0 0 10 10'><linearGradient id='gradient'/></svg></main>",
        )));
        let emit_document = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        let bootstrap = runtime.Evaluate(&realm, DOMBootstrapSource(), "dom-webidl");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
        let result = runtime.Evaluate(&realm,r#"
            function check(value,label){if(!value)throw Error(label)}
            const root=document.querySelector('#root'), a=root.querySelector('p.one'), b=root.querySelector('#b');
            check(root === document.getElementById('root') && root.querySelector('#root') === null,'query excludes receiver');
            check(a.matches('main > p.one.two:first-child') && b.matches('p + p:nth-child(2)') && b.closest('main') === root,'combinators');
            check(root.matches(':has(> section)') && root.matches(':has(> section .leaf)') && !root.matches(':has(> .leaf)') && root.matches(':has(.leaf)'),'has source traversal');
            check(a.matches(':is(#a,.absent):not(.absent):where(p)') && !a.matches('p::before'),'selector lists and pseudo');
            check(document.querySelector(':root') === document.documentElement,'document root');
            const detached=document.createElement('div');check(!detached.matches(':root') && detached.matches(':first-child') && detached.matches(':last-child'),'detached source semantics');
            const field=document.getElementById('field');field.value='live';check(!field.matches(':placeholder-shown'),'placeholder reads live control value');field.setAttribute('value','attribute');check(!field.matches(':placeholder-shown'),'placeholder attribute');
            const svg=document.querySelector('svg');check(svg.matches('[viewBox]') && svg.matches('[viewbox]'),'SVG attribute selectors normalize in HTML documents');
            check(document.querySelector('linearGradient').id === 'gradient' && document.querySelector('lineargradient').id === 'gradient','SVG type selectors normalize in HTML documents');
            const live=root.children, childNodes=root.childNodes;
            check(live instanceof HTMLCollection && childNodes instanceof NodeList && root.children === live && root.childNodes === childNodes,'live identity and prototypes');
            check(live.length === 5 && childNodes.length === 6 && childNodes[1].nodeType === 8,'children node kinds');
            check(live.item(0.9) === a && live.item(-1) === null && live.namedItem('first') === a && live.namedItem('') === null,'collection methods');
            const snapshot=root.querySelectorAll('p'), tags=root.getElementsByTagName('P'), classes=root.getElementsByClassName('one two');
            check(snapshot instanceof NodeList && snapshot.length === 2 && tags.length === 2 && classes.length === 1,'query collections');
            let visits=0;Array.prototype.forEach.call(snapshot,()=>visits++);
            check(visits===2 && (0 in snapshot) && !(2 in snapshot),'host indices participate in HasProperty');
            check(Array.prototype.map.call(snapshot,n=>n.id).join(',')==='a,b','array methods traverse host collection');
            check(root.getElementsByTagName('P') === tags && root.querySelectorAll('p') !== snapshot,'cache and snapshots');
            const scripts=document.scripts, script=document.createElement('script');script.id='live-script';
            check(scripts instanceof HTMLCollection && scripts === document.scripts && scripts.length === 0,'document scripts collection');
            root.appendChild(script);
            check(scripts.length === 1 && scripts[0] === script && scripts.namedItem('live-script') === script,'document scripts insertion');
            root.removeChild(script);check(scripts.length === 0,'document scripts removal');
            check(document.images === document.getElementsByTagName('img') && document.forms === document.getElementsByTagName('form') && document.plugins === document.embeds,'document collection filters');
            const added=document.createElement('p');added.className='one two';root.appendChild(added);
            check(live.length === 6 && tags.length === 3 && classes.length === 2 && snapshot.length === 2,'live mutation');
            root.removeChild(a);check(snapshot[0] === a && tags.length === 2 && classes.length === 1,'snapshot retains detached node');
            const iterator=live[Symbol.iterator]();check(iterator[Symbol.iterator]() === iterator,'iterator identity');
            while(!iterator.next().done){}
            root.appendChild(a);check(iterator.next().done,'iterator remains done');
            check(Array.from(live).length === live.length && Array.from(snapshot).length === 2,'iterability');
            const fragment=document.createDocumentFragment();fragment.appendChild(document.createElement('aside'));
            check(fragment.querySelector('aside') === fragment.firstChild && fragment.children.length === 1,'fragment query');
            const clone=root.cloneNode(true);check(clone !== root && clone.children.length === root.children.length && clone.querySelector('p') !== root.querySelector('p'),'deep clone');
            const shallow=a.cloneNode(false);check(shallow.childNodes.length === 0 && shallow.className === a.className,'shallow clone');
            const text=document.createTextNode('text'), comment=document.createComment('note');
            fragment.insertBefore(text,fragment.firstChild);fragment.appendChild(comment);check(fragment.firstChild === text && fragment.lastChild === comment,'create and insert node kinds');
            let invalid=false;try{root.appendChild(root)}catch(e){invalid=e instanceof TypeError}check(invalid && root.parentNode !== null,'invalid cycle');
            a.style.setProperty('color','red');a.style.setProperty('color','blue','important');check(a.getAttribute('style') === 'color: blue !important;' && a.style.getPropertyValue('color') === 'blue','inline style');
            check('transform' in a.style && 'perspective' in a.style && !('notARealProperty' in a.style),'CSS declaration named properties');
            const separate=document.implementation.createHTMLDocument('title');check(separate.defaultView === null && separate.head.firstChild.textContent === 'title' && !separate.hasFocus(),'detached document');
            a.className='one\u000btwo';check(!a.matches('.one.two[class~=two]') && root.getElementsByClassName('one\u000btwo').length >= 1 && !a.matches('\u00a0p'),'source CSS and HTML whitespace sets');
            true
        "#,"collections.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
    }

    #[test]
    fn bootstrap_dom_script_mutation_is_synchronously_visible() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let input_id = {
            let mut owner = document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let root = tree.Root();
            let html = tree.CreateElementDefault(DOMNamespace::kHTML, "html".into());
            tree.AppendChild(root, html);
            let body = tree.CreateElementDefault(DOMNamespace::kHTML, "body".into());
            tree.AppendChild(html, body);
            let input = tree.CreateElementDefault(DOMNamespace::kHTML, "input".into());
            tree.SetAttribute(
                input,
                DOMAttribute {
                    local_name: "id".into(),
                    value: "kw".into(),
                    ..Default::default()
                },
            );
            tree.AppendChild(body, input);
            tree.Node(input).Id()
        };
        let emit_document = document.clone();
        let host = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    emit_document.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        let bootstrap = runtime.Evaluate(&realm, DOMBootstrapSource(), "browser:dom-webidl");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
        let script = runtime.Evaluate(
            &realm,
            "const field = document.getElementById('kw');\
             field.className = 'night';\
             document instanceof Document && field instanceof HTMLInputElement && field.getAttribute('class') === 'night'",
            "page.js",
        );
        assert!(script.Succeeded(), "{:?}", script.exception);
        assert_eq!(
            script
                .value
                .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
        let owner = document.borrow();
        let tree = owner.GetDocument();
        let input = tree.FindNodeById(input_id).unwrap();
        assert_eq!(
            tree.Node(input).FindAttribute("class").unwrap().value,
            "night"
        );
    }
}
