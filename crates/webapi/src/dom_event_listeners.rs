//! Web IDL callback storage for the translated EventTarget listener owner.
//!
//! Chromium: core/dom/events/event_target.cc:453-839,960-1096;
//! bindings/core/v8/js_event_listener.h:30-45. Registration flags, snapshots,
//! removal and dispatch ordering belong exclusively to EventTarget. This
//! adapter retains only callback identity and the existing binding trace edge.
//!
//! Listener invocation borrows the callback's original ScriptState and the live
//! runtime through `dom_event_listener_invocation`; native AbortSignal owns its
//! algorithms and the registry's weak-key entries through `dom_abort_signals`.

use super::*;
use interaction::dispatch::core::dom::events::{
    add_event_listener_options_resolved::AbortAlgorithmHandle,
    event::{EventRef, EventTargetKind},
    event_listener::{EventListener, EventListenerRef, ExecutionContext},
    event_target::{EventTarget, EventTargetHost, ScriptForbiddenScope, TaskAttributionState},
    registered_event_listener::RegisteredEventListener,
};
use interaction::scroll::core::frame::event_handler_registry::EventHandlerRegistryTarget;
use std::{any::Any, rc::Weak};

pub(super) type ListenerTargets = Rc<RefCell<HashMap<HostObjectId, Rc<EventTarget>>>>;

pub(super) struct Listener {
    pub target: HostObjectId,
    pub callback: ListenerCallback,
    pub reference_slot: u64,
    pub attribute: bool,
    pub context: Option<javascript::javascript_runtime::JavaScriptCallbackContext>,
    pub(super) bindings: Weak<super::abort_signals::AbortBindings>,
    current_context: Rc<RefCell<Option<javascript::javascript_runtime::JavaScriptCallbackContext>>>,
}

impl Listener {
    pub(super) fn new(
        target: HostObjectId,
        callback: ListenerCallback,
        reference_slot: u64,
        attribute: bool,
        context: Option<javascript::javascript_runtime::JavaScriptCallbackContext>,
        current_context: Rc<
            RefCell<Option<javascript::javascript_runtime::JavaScriptCallbackContext>>,
        >,
        bindings: Weak<super::abort_signals::AbortBindings>,
    ) -> Self {
        Self {
            target,
            callback,
            reference_slot,
            attribute,
            context,
            current_context,
            bindings,
        }
    }

    pub(super) fn From(listener: &dyn EventListener) -> &Self {
        (listener as &dyn Any)
            .downcast_ref()
            .expect("DOM callback binding")
    }
}

impl EventListener for Listener {
    fn ScriptBindingIdentity(&self) -> Option<u64> {
        Some(self.reference_slot)
    }
    fn Invoke(&self, _: Option<&dyn ExecutionContext>, _: &EventRef) {
        panic!("DOM JavaScript callbacks require the live scoped runtime invoker")
    }

    fn Matches(&self, other: &dyn EventListener) -> bool {
        let Some(other) = (other as &dyn Any).downcast_ref::<Self>() else {
            return false;
        };
        // JSEventListener and JSEventHandler are distinct native classes even
        // when they reference the same callback object.
        self.attribute == other.attribute
            && self
                .callback
                .Resolve()
                .is_some_and(|callback| other.callback.Matches(&callback))
    }

    fn IsEventHandler(&self) -> bool {
        self.attribute
    }
    fn IsJSBasedEventListener(&self) -> bool {
        true
    }
    fn BelongsToTheCurrentWorld(&self, context: Option<&dyn ExecutionContext>) -> bool {
        let Some(listener) = self.context.as_ref() else {
            return false;
        };
        if let Some(current) = self.current_context.borrow().as_ref() {
            return listener.ContextIsValid()
                && current.Identity() == listener.Identity()
                && current.WorldId() == listener.WorldId();
        }
        context.is_some_and(|context| context.IsWindow() && context.IsParsingDocument())
            && listener.WorldId() == 0
    }
}

pub(super) type NativeEventContext = Rc<RefCell<Weak<RefCell<DOM>>>>;
struct ListenerContext {
    document: Weak<RefCell<DOM>>,
    window: Option<NativeEventContext>,
}
impl ListenerContext {
    fn Document(&self) -> Option<Rc<RefCell<DOM>>> {
        self.window.as_ref().map_or_else(
            || self.document.upgrade(),
            |window| window.borrow().upgrade(),
        )
    }
}
impl ExecutionContext for ListenerContext {
    fn IsParsingDocument(&self) -> bool {
        self.Document().is_some_and(|owner| {
            owner
                .borrow()
                .GetDocument()
                .GetDocumentParser()
                .is_some_and(|parser| parser.IsParsing())
        })
    }
    fn IsContextDestroyed(&self) -> bool {
        self.Document()
            .is_none_or(|owner| owner.borrow().GetDocument().IsContextDestroyed())
    }
    fn IsWindow(&self) -> bool {
        true
    }
    fn IsInFencedFrameTree(&self) -> bool {
        false
    }
    fn DeprecateUnloadOptOutEnabled(&self) -> bool {
        false
    }
    fn UnloadFeatureEnabled(&self) -> bool {
        true
    }
}

struct ListenerTargetHost {
    context: Rc<ListenerContext>,
    graph: Weak<DOMHostGraph>,
    id: HostObjectId,
    registry: std::cell::OnceCell<EventHandlerRegistryTarget>,
    signal: Option<Weak<interaction::default::core::dom::abort_signal::AbortSignal>>,
    native_other: bool,
}
struct NoScriptScope;
impl ScriptForbiddenScope for NoScriptScope {}

impl EventTargetHost for ListenerTargetHost {
    fn BeginNavigationEventTiming(&self, _: &EventRef) -> Box<dyn Any> {
        Box::new(())
    }
    fn InterfaceName(&self) -> &str {
        if self.signal.is_some() {
            return "AbortSignal";
        }
        if self.native_other {
            return "EventTarget";
        }
        if self.id == 0 {
            "Window"
        } else {
            "Node"
        }
    }
    fn GetExecutionContext(&self) -> Option<Rc<dyn ExecutionContext>> {
        (!self.context.IsContextDestroyed())
            .then(|| self.context.clone() as Rc<dyn ExecutionContext>)
    }
    fn Kind(&self) -> EventTargetKind {
        if self.signal.is_some() || self.native_other {
            return EventTargetKind::Other;
        }
        if self.id == 0 {
            EventTargetKind::Window
        } else {
            EventTargetKind::Node
        }
    }
    fn IsMainSequence(&self) -> bool {
        true
    }
    fn ClearCurrentTargetAfterDispatchEnabled(&self) -> bool {
        true
    }
    fn IsLocalDOMWindow(&self) -> bool {
        self.id == 0
    }
    fn IsDocumentNode(&self) -> bool {
        let Some(owner) = self.context.Document() else {
            return false;
        };
        let owner = owner.borrow();
        let document = owner.GetDocument();
        document
            .FindNodeById(self.id)
            .is_some_and(|node| document.Node(node).Type() == DOMNodeType::kDocument)
    }
    fn IsDocumentElement(&self) -> bool {
        self.IsTopLevelElement(false)
    }
    fn IsDocumentBody(&self) -> bool {
        self.IsTopLevelElement(true)
    }
    fn KeepAliveListeners(&self, listeners: &[EventListenerRef]) -> Box<dyn Any> {
        // Root the actual VM callbacks, including once/removed entries, before
        // EventTarget can revoke traced native edges and enter author script.
        let cells = self.graph.upgrade().map(|graph| {
            graph.KeepListenerCells(
                listeners
                    .iter()
                    .filter_map(|listener| listener.ScriptBindingIdentity())
                    .collect(),
            )
        });
        Box::new((
            cells,
            listeners
                .iter()
                .filter_map(|listener| Listener::From(listener.as_ref()).callback.Resolve())
                .collect::<Vec<_>>(),
        ))
    }
    fn AddedEventListener(&self, event_type: &str, registered: &RegisteredEventListener) {
        if let Some(signal) = self.signal.as_ref() {
            if let Some(signal) = signal.upgrade() {
                signal.ListenerChanged(event_type, true);
            }
            return;
        }
        if self.native_other {
            return;
        }
        self.registry
            .get()
            .expect("retained target registry")
            .DidAddEventHandler(event_type, &registered.Options());
    }
    fn WillRemoveAllEventListeners(&self) {
        if self.id != 0 {
            if let Some(graph) = self.graph.upgrade() {
                graph.ClearListenerReferences(self.id);
            }
        }
        if self.id != 0 && self.signal.is_none() && !self.native_other {
            self.registry
                .get()
                .expect("retained target registry")
                .DidRemoveAllEventHandlers();
        }
    }
    fn DidRemoveAllEventListeners(&self) {
        if self.id == 0 {
            if let Some(graph) = self.graph.upgrade() {
                graph.ClearListenerReferences(self.id);
            }
        }
        if let Some(signal) = self.signal.as_ref() {
            if let Some(signal) = signal.upgrade() {
                signal.ListenerChanged("abort", false);
            }
            return;
        }
        if self.native_other {
            return;
        }
        if self.id == 0 {
            self.registry
                .get()
                .expect("retained target registry")
                .DidRemoveAllEventHandlers();
        }
    }
    fn RemovedEventListener(&self, event_type: &str, registered: &RegisteredEventListener) {
        if let Some(signal) = self.signal.as_ref() {
            if let Some(signal) = signal.upgrade() {
                signal.ListenerChanged(event_type, false);
            }
        } else if !self.native_other {
            self.registry
                .get()
                .expect("retained target registry")
                .DidRemoveEventHandler(event_type, &registered.Options());
        }
        if let (Some(graph), Some(listener)) = (self.graph.upgrade(), registered.Callback()) {
            let listener = Listener::From(listener.as_ref());
            if matches!(listener.callback, ListenerCallback::Traced(_)) {
                graph.ListenerReference(listener.target, listener.reference_slot, None);
            }
        }
    }
    fn RegisterAbortAlgorithm(
        &self,
        listener: EventListenerRef,
        handle: Rc<dyn AbortAlgorithmHandle>,
    ) {
        let graph = self.graph.upgrade().expect("live binding graph");
        graph
            .abort_bindings
            .get()
            .expect("native abort binding owner")
            .RegisterAbortAlgorithm(&listener, handle);
    }
    fn EnterScriptForbiddenScope(&self) -> Box<dyn ScriptForbiddenScope> {
        // These Web IDL bindings cannot register native animation-trigger
        // callbacks; EventTarget's script-forbidden pass has no such entries.
        Box::new(NoScriptScope)
    }
    fn PostEventTask(&self, _: u32, _: Box<dyn FnOnce()>) {
        panic!("queued DOM events require the Document task-runner port")
    }
    fn CaptureCurrentTaskState(&self) -> Option<Rc<dyn TaskAttributionState>> {
        None
    }
}

impl ListenerTargetHost {
    fn IsTopLevelElement(&self, body: bool) -> bool {
        let Some(owner) = self.context.Document() else {
            return false;
        };
        let owner = owner.borrow();
        let document = owner.GetDocument();
        let Some(index) = document.FindNodeById(self.id) else {
            return false;
        };
        let root = document
            .Node(index)
            .OwnerDocumentNode()
            .and_then(|owner| document.FindNodeById(owner.Id()))
            .and_then(|doc| {
                document
                    .Node(doc)
                    .Children()
                    .iter()
                    .copied()
                    .find(|&child| document.Node(child).Type() == DOMNodeType::kElement)
            });
        root.is_some_and(|root| {
            if !body {
                root == index
            } else {
                document
                    .Node(root)
                    .Children()
                    .iter()
                    .copied()
                    .find(|&child| {
                        document.Node(child).IsHTMLElement("body")
                            || document.Node(child).IsHTMLElement("frameset")
                    })
                    == Some(index)
            }
        })
    }
}

pub(super) fn HasBlockingListener(target: &EventTarget, kind: &str) -> bool {
    target.GetEventListeners(kind).is_some_and(|listeners| {
        listeners
            .borrow()
            .iter()
            .any(|listener| !listener.Removed() && !listener.Passive())
    })
}

pub(super) fn CreateGenericTarget(
    document: NativeEventContext,
    graph: Weak<DOMHostGraph>,
    id: u64,
) -> Rc<interaction::dispatch::core::dom::events::event_target_impl::EventTargetImpl> {
    interaction::dispatch::core::dom::events::event_target_impl::EventTargetImpl::new(Rc::new(
        ListenerTargetHost {
            context: Rc::new(ListenerContext {
                document: Weak::new(),
                window: Some(document),
            }),
            graph,
            id,
            registry: Default::default(),
            signal: None,
            native_other: true,
        },
    ))
}

pub(super) fn CreateAbortTarget(
    id: u64,
    document: NativeEventContext,
    graph: Weak<DOMHostGraph>,
    signal: Weak<interaction::default::core::dom::abort_signal::AbortSignal>,
) -> Rc<EventTarget> {
    EventTarget::new(Rc::new(ListenerTargetHost {
        context: Rc::new(ListenerContext {
            document: Weak::new(),
            window: Some(document),
        }),
        graph,
        id,
        registry: Default::default(),
        signal: Some(signal),
        native_other: false,
    }))
}

impl DOMJavaScriptBindings {
    pub(super) fn ListenerTarget(&self, id: HostObjectId) -> Rc<EventTarget> {
        self.listeners
            .borrow_mut()
            .entry(id)
            .or_insert_with(|| {
                let host = Rc::new(ListenerTargetHost {
                    context: Rc::new(ListenerContext {
                        document: Rc::downgrade(&self.document),
                        window: (id == 0).then(|| self.window_native_bindings.NativeContext()),
                    }),
                    graph: Rc::downgrade(&self.host_graph),
                    id,
                    registry: std::cell::OnceCell::new(),
                    signal: None,
                    native_other: false,
                });
                let target = EventTarget::new(host.clone());
                assert!(host
                    .registry
                    .set(
                        self.event_handler_registry.BindTargetWithNodeId(
                            &target.AsEventTargetRef(),
                            (id != 0).then_some(id)
                        )
                    )
                    .is_ok());
                target
            })
            .clone()
    }
}
