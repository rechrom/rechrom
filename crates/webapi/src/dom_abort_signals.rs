//! Native Web IDL projection of AbortController/AbortSignal.
//! Chromium core/dom/abort_controller.cc:16-56; abort_signal.cc:45-443;
//! abort_signal_registry.cc:20-99. All abort state/algorithms remain native.
use super::*;
use interaction::{
    default::core::dom::{
        abort_controller::AbortController,
        abort_signal::{
            AbortSignal, AbortSignalContext, AbortSignalInvocation, AbortSignalTimeoutTask,
            AlgorithmHandle, CancellableAbortTask,
        },
        abort_signal_registry::{AbortSignalRegistry, AbortSignalRegistryObserver},
    },
    dispatch::{
        bindings::script_invocation::{ScriptNativeObject, ScriptValue, Value},
        core::dom::events::{
            event::{EventClock, EventRef},
            event_listener::EventListenerRef,
            event_target::{EventTarget, ScopedEventListenerInvoker},
        },
    },
};
use javascript::javascript_runtime::{HostObjectTraceChange as C, WeakJavaScriptValue};
use std::{any::Any, collections::HashSet, rc::Weak};

#[path = "dom_close_watcher.rs"]
mod close_watcher;
#[path = "dom_form_data.rs"]
mod form_data;
#[path = "dom_input_event_data_transfer.rs"]
mod input_event_data_transfer;
#[path = "dom_navigation.rs"]
mod navigation;
#[path = "dom_trusted_types.rs"]
mod trusted_types;

enum StoredValue {
    Rooted(HostValue),
    Traced(WeakJavaScriptValue),
}
struct AbortValue(RefCell<StoredValue>);
impl ScriptValue for AbortValue {}
fn value(host: HostValue) -> Value {
    Rc::new(AbortValue(RefCell::new(StoredValue::Rooted(host))))
}
fn host(value: &Value) -> HostValue {
    match &*(value.as_ref() as &dyn Any)
        .downcast_ref::<AbortValue>()
        .expect("native abort script value")
        .0
        .borrow()
    {
        StoredValue::Rooted(value) => value.clone(),
        StoredValue::Traced(value) => {
            HostValue::JavaScriptValue(value.Upgrade().expect("traced AbortSignal reason"))
        }
    }
}

pub(super) type AbortTaskScheduler = Rc<
    dyn Fn(
        u64,
        bool,
        Box<dyn FnOnce(&mut dyn JavaScriptHostRuntime)>,
    ) -> Rc<dyn CancellableAbortTask>,
>;
pub(super) struct AbortBindings {
    self_: Weak<Self>,
    document: event_listeners::NativeEventContext,
    current_document_bindings: RefCell<Weak<AbortBindings>>,
    graph: Weak<DOMHostGraph>,
    next_id: Rc<Cell<u64>>,
    classes: Rc<RefCell<HashMap<u64, &'static str>>>,
    listeners: ListenerTargets,
    events: Rc<RefCell<services::BoundEvents>>,
    current_event: RefCell<Rc<RefCell<Option<HostValue>>>>,
    namespace: RefCell<WeakJavaScriptHostObjectNamespace>,
    realm: RefCell<WeakJavaScriptRealm>,
    clock: Option<Rc<dyn EventClock>>,
    scheduler: std::cell::OnceCell<AbortTaskScheduler>,
    exception: Rc<dyn Fn(&str, &str) -> HostValue>,
    objects: RefCell<HashMap<u64, Rc<dyn ScriptNativeObject>>>,
    identities: RefCell<HashMap<usize, u64>>,
    registry: std::cell::OnceCell<Rc<AbortSignalRegistry>>,
    handler_wrapper: RefCell<Option<JavaScriptFunction>>,
    input_file_projection: RefCell<Option<JavaScriptFunction>>,
    foreign_bindings: RefCell<HashMap<usize, WeakJavaScriptValue>>,
    event_state: Rc<RefCell<Option<JavaScriptFunction>>>,
    destroyed: Cell<bool>,
    form_files: RefCell<HashMap<usize, Weak<form_data::NativeFormFile>>>,
    form_data_constructor: RefCell<Option<super::services::FormDataConstructor>>,
    navigation_time_stamp: RefCell<
        Option<Rc<dyn interaction::dispatch::core::dom::events::event::EventTimeStampContext>>,
    >,
    navigation: RefCell<
        Option<Rc<interaction::default::core::navigation_api::navigation_api::NavigationApi>>,
    >,
    history: RefCell<Option<Rc<interaction::default::core::frame::history::History>>>,
    trusted_types: RefCell<HashMap<i32, Rc<trusted_types::NativeTrustedTypeFactory>>>,
    close_watcher_host: RefCell<Option<services::CloseWatcherHost>>,
}
impl AbortBindings {
    pub(super) fn new(
        document: &Rc<RefCell<DOM>>,
        graph: &Rc<DOMHostGraph>,
        next_id: Rc<Cell<u64>>,
        classes: Rc<RefCell<HashMap<u64, &'static str>>>,
        listeners: ListenerTargets,
        events: Rc<RefCell<services::BoundEvents>>,
        current_event: Rc<RefCell<Option<HostValue>>>,
        event_state: Rc<RefCell<Option<JavaScriptFunction>>>,
        clock: Option<Rc<dyn EventClock>>,
        exception: Rc<dyn Fn(&str, &str) -> HostValue>,
    ) -> Rc<Self> {
        let owner = Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            document: Rc::new(RefCell::new(Rc::downgrade(document))),
            current_document_bindings: RefCell::new(self_.clone()),
            graph: Rc::downgrade(graph),
            next_id,
            classes,
            listeners,
            events,
            current_event: RefCell::new(current_event),
            namespace: Default::default(),
            realm: Default::default(),
            clock,
            scheduler: Default::default(),
            exception,
            objects: Default::default(),
            identities: Default::default(),
            registry: Default::default(),
            handler_wrapper: Default::default(),
            input_file_projection: Default::default(),
            foreign_bindings: Default::default(),
            event_state,
            destroyed: false.into(),
            form_files: Default::default(),
            form_data_constructor: Default::default(),
            navigation: Default::default(),
            history: Default::default(),
            trusted_types: Default::default(),
            close_watcher_host: Default::default(),
            navigation_time_stamp: Default::default(),
        });
        let observer: Rc<dyn AbortSignalRegistryObserver> = owner.clone();
        assert!(owner
            .registry
            .set(AbortSignalRegistry::new(Rc::downgrade(&observer)))
            .is_ok());
        owner
    }
    pub(super) fn NativeFromValue<T: Any>(&self, v: &HostValue) -> Option<Rc<T>> {
        match v {
            HostValue::Object(v) => {
                let native: Rc<dyn Any> = self.objects.borrow().get(&v.id)?.clone();
                native.downcast().ok()
            }
            HostValue::Retained(owner, _) => owner
                .downcast_ref::<javascript::javascript_runtime::JavaScriptNativeObjectReference>()?
                .NativeOwner::<T>(),
            _ => None,
        }
    }
    fn Allocate(&self) -> u64 {
        let id = self.next_id.get();
        self.next_id
            .set(id.checked_add(1).expect("native host identity"));
        id
    }
    fn Publish(&self, changes: impl IntoIterator<Item = C>) {
        if let Some(graph) = self.graph.upgrade() {
            graph.Publish(changes);
        }
    }
    fn StoreAt(&self, id: u64, object: Rc<dyn ScriptNativeObject>) {
        self.identities
            .borrow_mut()
            .insert(object.ScriptIdentity(), id);
        if matches!(
            object.ScriptInterfaceName(),
            "AbortSignal"
                | "CloseWatcher"
                | "AbortController"
                | "EventTarget"
                | "History"
                | "Navigation"
                | "NavigationHistoryEntry"
                | "NavigationTransition"
                | "NavigationDestination"
                | "NavigationActivation"
                | "NavigationPrecommitController"
                | "NavigationDeferPageSwapController"
                | "NavigateEvent"
                | "FormData"
                | "FormDataFile"
                | "DataTransfer"
                | "DataTransferItemList"
                | "DataTransferItem"
                | "FileList"
                | "InputDeviceCapabilities"
                | "TrustedHTML"
                | "TrustedScript"
                | "TrustedScriptURL"
                | "TrustedTypePolicy"
                | "TrustedTypePolicyFactory"
                | "Range"
        ) {
            self.classes
                .borrow_mut()
                .insert(id, object.ScriptInterfaceName());
        }
        if let Some(target) = object.EventTarget() {
            self.listeners.borrow_mut().insert(id, target);
        }
        self.objects.borrow_mut().insert(id, object);
        self.Publish([C::Object { id }]);
    }
    pub(super) fn EnsureObject(&self, object: Rc<dyn ScriptNativeObject>) -> u64 {
        if let Some(&id) = self.identities.borrow().get(&object.ScriptIdentity()) {
            return id;
        }
        let target_id = object.EventTarget().and_then(|target| {
            self.listeners
                .borrow()
                .iter()
                .find_map(|(id, stored)| Rc::ptr_eq(stored, &target).then_some(*id))
        });
        let id = target_id.unwrap_or_else(|| self.Allocate());
        self.StoreAt(id, object);
        id
    }
    fn Trace(&self, object: Rc<dyn ScriptNativeObject>) {
        self.TraceVisited(object, &mut HashSet::new());
    }
    fn TraceVisited(&self, object: Rc<dyn ScriptNativeObject>, visited: &mut HashSet<usize>) {
        if !visited.insert(object.ScriptIdentity()) {
            return;
        }
        let id = self.EnsureObject(object.clone());
        let mut slot = 0;
        object.VisitScriptValues(&mut |value| {
            let current = host(&value);
            match current {
                HostValue::JavaScriptValue(current) => {
                    self.Publish([C::ValueReference {
                        owner: id,
                        slot,
                        value: Some(current.clone()),
                    }]);
                    if let Some(weak) = current.Downgrade() {
                        *(value.as_ref() as &dyn Any)
                            .downcast_ref::<AbortValue>()
                            .expect("abort reason")
                            .0
                            .borrow_mut() = StoredValue::Traced(weak);
                    }
                }
                HostValue::Retained(reference, _) => {
                    if let Some(reference) = reference.downcast_ref::<javascript::javascript_runtime::JavaScriptNativeObjectReference>() {
                        let current = reference.BindingCell().clone();
                        self.Publish([C::ValueReference { owner: id, slot, value: Some(current.clone()) }]);
                        if let Some(weak) = current.Downgrade() {
                            *(value.as_ref() as &dyn Any).downcast_ref::<AbortValue>().expect("native script field").0.borrow_mut() = StoredValue::Traced(weak);
                        }
                    }
                }
                HostValue::Object(target) => self.Publish([C::Reference {
                    owner: id,
                    slot,
                    target: Some(target.id),
                }]),
                _ => {}
            }
            slot += 1;
        });
        let mut natives = Vec::new();
        object.VisitNativeObjects(&mut |value| natives.push(value));
        self.FormDataNativeObjects(&object, &mut natives);
        self.InputDataTransferNativeObjects(&object, &mut natives);
        let mut targets = Vec::new();
        for native in natives {
            let foreign = self
                .foreign_bindings
                .borrow()
                .get(&native.ScriptIdentity())
                .cloned();
            if let Some(foreign) = foreign {
                self.Publish([C::ValueReference {
                    owner: id,
                    slot,
                    value: Some(
                        foreign.Upgrade().expect(
                            "foreign native source Member retains its original binding cell",
                        ),
                    ),
                }]);
                slot += 1;
            } else {
                targets.push(self.EnsureObject(native.clone()));
                self.TraceVisited(native, visited);
            }
        }
        self.Publish([C::ReferenceList {
            owner: id,
            targets: targets.into(),
        }]);
        object.VisitEphemerons(&mut |key, native| {
            let target = self.EnsureObject(native.clone());
            self.Publish([C::Ephemeron {
                owner: id,
                slot: key,
                entry: Some((key, target)),
            }]);
            self.TraceVisited(native, visited);
        });
    }
    pub(super) fn RebindForWindowReuse(
        &self,
        document: &Rc<RefCell<DOM>>,
        current: &Rc<AbortBindings>,
    ) {
        assert!(
            !self.destroyed.get(),
            "reuse a live Window execution context"
        );
        *self.document.borrow_mut() = Rc::downgrade(document);
        *self.current_document_bindings.borrow_mut() = Rc::downgrade(current);
    }
    pub(super) fn NativeContext(&self) -> event_listeners::NativeEventContext {
        self.document.clone()
    }
    pub(super) fn NativeValue(&self, id: u64) -> HostValue {
        self.namespace
            .borrow()
            .Upgrade()
            .expect("original native Window namespace")
            .Wrap(HostValue::Object(HostObjectRef { id }))
    }
    pub(super) fn BindCurrentEvent(&self, slot: Rc<RefCell<Option<HostValue>>>) {
        *self.current_event.borrow_mut() = slot;
    }
    pub(super) fn BindRuntime(&self, realm: &JavaScriptRealm) {
        *self.realm.borrow_mut() = realm.Downgrade();
    }
    pub(super) fn BindNamespace(&self, namespace: &JavaScriptHostObjectNamespace) {
        *self.namespace.borrow_mut() = namespace.Downgrade();
    }
    pub(super) fn BindScheduler(&self, scheduler: AbortTaskScheduler) {
        assert!(self.scheduler.set(scheduler).is_ok());
    }
    pub(super) fn BindHandlerWrapper(&self, callback: &JavaScriptFunction) {
        *self.handler_wrapper.borrow_mut() = Some(callback.clone());
    }
    pub(super) fn BindInputFileProjection(&self, callback: &JavaScriptFunction) {
        *self.input_file_projection.borrow_mut() = Some(callback.clone());
    }
    pub(super) fn RegisterAbortAlgorithm(
        &self,
        listener: &EventListenerRef,
        handle: Rc<dyn interaction::dispatch::core::dom::events::add_event_listener_options_resolved::AbortAlgorithmHandle>,
    ) {
        let handle: Rc<dyn Any> = handle;
        self.registry.get().unwrap().RegisterAbortAlgorithm(
            listener,
            Some(
                handle
                    .downcast::<AlgorithmHandle>()
                    .expect("native AbortSignal algorithm handle"),
            ),
        );
    }
    pub(super) fn Signal(&self, id: u64) -> Option<Rc<AbortSignal>> {
        let object = self.objects.borrow().get(&id)?.clone();
        let object: Rc<dyn Any> = object;
        object.downcast().ok()
    }
    pub(super) fn NativeObject(&self, id: u64) -> Option<Rc<dyn Any>> {
        let object = self.objects.borrow().get(&id)?.clone();
        Some(object)
    }
    pub(super) fn SignalFromValue(&self, value: &HostValue) -> Option<Rc<AbortSignal>> {
        if let HostValue::Object(value) = value {
            return self.Signal(value.id);
        }
        let HostValue::Retained(owner, _) = value else {
            return None;
        };
        let reference = owner
            .downcast_ref::<javascript::javascript_runtime::JavaScriptNativeObjectReference>()?;
        reference.Namespace().Upgrade()?;
        let signal = reference.NativeOwner::<AbortSignal>()?;
        self.foreign_bindings
            .borrow_mut()
            .insert(signal.Identity(), reference.BindingCell().Downgrade()?);
        Some(signal)
    }
    pub(super) fn NativeTarget(&self, id: u64) -> Option<Rc<EventTarget>> {
        self.objects.borrow().get(&id)?.EventTarget()
    }
    fn Context(&self, id: u64) -> Rc<dyn AbortSignalContext> {
        Rc::new(SignalContext {
            owner: self.self_.clone(),
            id,
            clock: self
                .clock
                .as_ref()
                .expect("native AbortSignal requires the embedding EventClock")
                .clone(),
        })
    }
    pub(super) fn Prepare(
        self: &Rc<Self>,
        receiver: u64,
        member: &str,
        args: &[HostValue],
    ) -> Option<HostContinuation> {
        if !member.starts_with("__abort")
            && !member.starts_with("__closeWatcher")
            && !member.starts_with("__navigation")
            && !member.starts_with("__trustedTypes")
            && !member.starts_with("__formData")
            && !member.starts_with("__inputDataTransfer")
            && member != "__eventTargetCreate"
        {
            return None;
        }
        let owner = self.clone();
        let member = member.to_owned();
        let args = args.to_vec();
        Some(Box::new(move |runtime| {
            if member.starts_with("__closeWatcher") {
                owner.InvokeCloseWatcher(receiver, &member, &args, runtime)
            } else if member.starts_with("__trustedTypes") {
                owner.InvokeTrustedTypes(receiver, &member, &args, runtime)
            } else if member.starts_with("__inputDataTransfer") {
                owner.InvokeInputDataTransfer(receiver, &member, &args, runtime)
            } else if member.starts_with("__formData") {
                owner.InvokeFormData(receiver, &member, &args, runtime)
            } else if member.starts_with("__navigation") {
                owner.InvokeNavigation(receiver, &member, &args, runtime)
            } else {
                owner.Invoke(receiver, &member, &args, runtime)
            }
        }))
    }
    fn Invoke(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let error = || {
            HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "Illegal AbortSignal invocation",
            )
        };
        if member == "__eventTargetCreate" && receiver == 0 && !self.destroyed.get() {
            let id = self.Allocate();
            let target =
                event_listeners::CreateGenericTarget(self.document.clone(), self.graph.clone(), id);
            self.StoreAt(id, target.clone());
            self.Trace(target);
            return HostResult {
                value: HostValue::Object(HostObjectRef { id }),
                ..Default::default()
            };
        }
        if self.destroyed.get() || self.clock.is_none() {
            return HostResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                "native AbortSignal execution context is unavailable",
            );
        }
        let invoker = AbortInvocation {
            owner: self,
            runtime: RefCell::new(runtime),
            native_events: RefCell::new(Vec::new()),
            synthetic: None,
        };
        match member {
            "__abortControllerCreate" if receiver == 0 => {
                let signal_id = self.Allocate();
                let controller = AbortController::new(self.Context(signal_id));
                let id = self.Allocate();
                self.StoreAt(signal_id, controller.Signal().clone());
                self.StoreAt(id, controller.clone());
                self.Trace(controller);
                HostResult {
                    value: HostValue::Object(HostObjectRef { id }),
                    ..Default::default()
                }
            }
            "__abortControllerSignal" | "__abortControllerAbort" => {
                let Some(object) = self.objects.borrow().get(&receiver).cloned() else {
                    return error();
                };
                let Some(controller) = object.AsAny().downcast_ref::<AbortController>() else {
                    return error();
                };
                if member == "__abortControllerSignal" {
                    let id = self.EnsureObject(controller.Signal().clone());
                    HostResult {
                        value: HostValue::Object(HostObjectRef { id }),
                        ..Default::default()
                    }
                } else {
                    controller
                        .Signal()
                        .SignalAbort(&invoker, value(args.first().cloned().unwrap_or_default()));
                    HostResult::default()
                }
            }
            "__abortStatic" | "__abortTimeout" | "__abortAny" if receiver == 0 => {
                let id = self.Allocate();
                let signal = if member == "__abortStatic" {
                    let reason = value(args.first().cloned().unwrap_or_default());
                    if invoker.IsUndefined(&reason) {
                        AbortSignal::StaticAbortWithoutReason(self.Context(id), &invoker)
                    } else {
                        AbortSignal::StaticAbort(self.Context(id), reason)
                    }
                } else if member == "__abortTimeout" {
                    if self.scheduler.get().is_none() {
                        return HostResult::Failure(
                            JavaScriptExceptionKind::kRuntimeError,
                            "native AbortSignal timer runner is unavailable",
                        );
                    }
                    let Some(HostValue::Number(milliseconds)) = args.first() else {
                        return error();
                    };
                    AbortSignal::Timeout(self.Context(id), *milliseconds as u64)
                } else {
                    let sources = args
                        .iter()
                        .map(|arg| self.SignalFromValue(arg))
                        .collect::<Option<Vec<_>>>();
                    let Some(sources) = sources else {
                        return error();
                    };
                    AbortSignal::Composite(self.Context(id), &sources)
                };
                self.StoreAt(id, signal.clone());
                self.Trace(signal);
                HostResult {
                    value: HostValue::Object(HostObjectRef { id }),
                    ..Default::default()
                }
            }
            "__abortSignalBrand"
            | "__abortSignalAborted"
            | "__abortSignalReason"
            | "__abortSignalThrow" => {
                let Some(signal) = self.Signal(receiver) else {
                    return error();
                };
                match member {
                    "__abortSignalBrand" => HostResult {
                        value: HostValue::Boolean(true),
                        ..Default::default()
                    },
                    "__abortSignalAborted" => HostResult {
                        value: HostValue::Boolean(signal.Aborted()),
                        ..Default::default()
                    },
                    "__abortSignalReason" => HostResult {
                        value: host(&signal.Reason(&invoker)),
                        ..Default::default()
                    },
                    _ => match signal.ThrowIfAborted() {
                        Ok(()) => HostResult::default(),
                        Err(reason) => HostResult {
                            thrown: Some(host(&reason)),
                            ..Default::default()
                        },
                    },
                }
            }
            _ => error(),
        }
    }
    pub(super) fn PrepareNativeDispatch(
        self: &Rc<Self>,
        receiver: u64,
        kind: String,
        bubbles: bool,
        cancelable: bool,
        composed: bool,
        argument: HostValue,
        native: Option<EventRef>,
        initialized: bool,
        constructed: bool,
    ) -> HostContinuation {
        let owner = self.clone();
        Box::new(move |runtime| {
            if !constructed && matches!(argument, HostValue::Retained(_, _)) {
                return HostResult::Failure(
                    JavaScriptExceptionKind::kTypeError,
                    "native Event wrapper belongs to another document namespace",
                );
            }
            let Some(target) = owner.NativeTarget(receiver) else {
                return HostResult::Failure(
                    JavaScriptExceptionKind::kTypeError,
                    "Illegal EventTarget invocation",
                );
            };
            let Some(clock) = owner.clock.as_ref() else {
                return HostResult::Failure(
                    JavaScriptExceptionKind::kRuntimeError,
                    "native EventTarget clock is unavailable",
                );
            };
            use interaction::dispatch::core::dom::events::{
                event::{Event, EventInit},
                event_target::ExceptionState,
            };
            let event = native.unwrap_or_else(|| {
                EventRef::new(if initialized {
                    Event::CreateWithInitializer(
                        &kind,
                        &EventInit {
                            bubbles,
                            cancelable,
                            composed,
                        },
                        clock.as_ref(),
                    )
                } else {
                    Event::Create(clock.as_ref())
                })
            });
            struct Exception(Option<&'static str>);
            impl ExceptionState for Exception {
                fn ThrowInvalidStateError(&mut self, message: &'static str) {
                    self.0 = Some(message);
                }
            }
            let invocation = AbortInvocation {
                owner: &owner,
                runtime: RefCell::new(runtime),
                native_events: Default::default(),
                synthetic: constructed.then_some((argument, receiver)),
            };
            let mut exception = Exception(None);
            let result =
                target.dispatchEventForBindingsWithInvoker(&event, &mut exception, &invocation);
            if let Some(message) = exception.0 {
                return HostResult {
                    thrown: Some(host(&invocation.DOMException("InvalidStateError", message))),
                    ..Default::default()
                };
            }
            // Native event construction/target assignment also occurs when the
            // target has no listeners; retain one wrapper for subsequent redispatch.
            let id = owner
                .events
                .borrow_mut()
                .BindNative(&event, receiver, || owner.Allocate());
            if !invocation.native_events.borrow().contains(&id) {
                invocation.native_events.borrow_mut().push(id);
            }
            HostResult {
                value: HostValue::Boolean(result),
                ..Default::default()
            }
        })
    }
    pub(super) fn Retire(&self, id: u64) {
        let object = self.objects.borrow_mut().remove(&id);
        if let Some(object) = object {
            self.identities
                .borrow_mut()
                .remove(&object.ScriptIdentity());
        }
    }
    pub(super) fn WeakKeysReleased(&self, ids: &[u64]) {
        let registry = self.registry.get().unwrap();
        let registry_id = self
            .identities
            .borrow()
            .get(&registry.ScriptIdentity())
            .copied();
        let Some(registry_id) = registry_id else {
            return;
        };
        registry.WeakKeysReleased(ids);
        for &key in ids {
            self.Publish([C::Ephemeron {
                owner: registry_id,
                slot: key,
                entry: None,
            }]);
        }
    }
    pub(super) fn ContextDestroyed(&self) {
        for factory in self
            .trusted_types
            .borrow_mut()
            .drain()
            .map(|(_, factory)| factory)
        {
            if let Some(&id) = self.identities.borrow().get(&factory.ScriptIdentity()) {
                self.Publish([C::Root {
                    id,
                    reason: 12,
                    rooted: false,
                }]);
            }
        }
        if let Some(history) = self.history.borrow_mut().take() {
            if let Some(&id) = self.identities.borrow().get(&history.ScriptIdentity()) {
                self.Publish([C::Root {
                    id,
                    reason: 9,
                    rooted: false,
                }]);
            }
        }
        if let Some(navigation) = self.navigation.borrow_mut().take() {
            if let Some(&id) = self.identities.borrow().get(&navigation.ScriptIdentity()) {
                self.Publish([C::Root {
                    id,
                    reason: 9,
                    rooted: false,
                }]);
            }
        }
        if self.destroyed.replace(true) {
            return;
        }
        // ExecutionContext teardown destroys registered watchers without
        // firing close, releasing their AbortSignal and traced wrapper roots.
        if let Some(host) = self.close_watcher_host.borrow_mut().take() {
            let watchers: Vec<_> = self.objects.borrow().values().filter_map(|object| {
                object.AsAny().downcast_ref::<interaction::default::core::html::closewatcher::close_watcher::NativeCloseWatcher>().map(|watcher| watcher.watcher)
            }).collect();
            for watcher in watchers {
                (host.destroy)(watcher);
            }
        }
        let signals: Vec<_> = self
            .objects
            .borrow()
            .values()
            .filter_map(|object| object.AsAny().downcast_ref::<AbortSignal>())
            .map(|signal| signal.Identity())
            .collect();
        for identity in signals {
            let id = self.identities.borrow().get(&identity).copied();
            if let Some(id) = id {
                if let Some(signal) = self.Signal(id) {
                    signal.ContextDestroyed();
                }
            }
        }
        self.registry.get().unwrap().ContextDestroyed();
    }
}
impl AbortSignalRegistryObserver for AbortBindings {
    fn RegistryChanged(&self, registry: Rc<AbortSignalRegistry>) {
        let id = self.EnsureObject(registry.clone());
        self.Publish([C::Root {
            id,
            reason: 8,
            rooted: !self.destroyed.get(),
        }]);
        self.Trace(registry);
    }
}
struct SignalContext {
    owner: Weak<AbortBindings>,
    id: u64,
    clock: Rc<dyn EventClock>,
}
impl AbortSignalContext for SignalContext {
    fn IsContextDestroyed(&self) -> bool {
        self.owner
            .upgrade()
            .is_none_or(|owner| owner.destroyed.get())
    }
    fn Clock(&self) -> &dyn EventClock {
        self.clock.as_ref()
    }
    fn CreateEventTarget(&self, signal: Weak<AbortSignal>) -> Rc<EventTarget> {
        let owner = self.owner.upgrade().expect("live abort context");
        event_listeners::CreateAbortTarget(
            self.id,
            owner.document.clone(),
            owner.graph.clone(),
            signal,
        )
    }
    fn Registry(&self) -> Rc<AbortSignalRegistry> {
        self.owner
            .upgrade()
            .expect("live abort context")
            .registry
            .get()
            .unwrap()
            .clone()
    }
    fn TraceSignal(&self, signal: Rc<AbortSignal>) {
        if let Some(owner) = self.owner.upgrade() {
            owner.Trace(signal);
        }
    }
    fn PostTimeout(
        &self,
        ms: u64,
        high_nesting: bool,
        task: Rc<dyn AbortSignalTimeoutTask>,
    ) -> Rc<dyn CancellableAbortTask> {
        let owner = self.owner.upgrade().expect("live abort timeout context");
        let captures = task.Captures();
        for capture in &captures {
            if capture.AsAny().is::<AbortSignal>() {
                owner.StoreAt(self.id, capture.clone());
            }
        }
        let roots: Vec<_> = captures
            .into_iter()
            .map(|capture| owner.EnsureObject(capture))
            .collect();
        for &id in &roots {
            owner.Publish([C::Root {
                id,
                reason: 10,
                rooted: true,
            }]);
        }
        let lease = Rc::new(TaskRoots {
            owner: owner.self_.clone(),
            ids: roots,
            active: true.into(),
        });
        let captured_roots = lease.clone();
        let weak = owner.self_.clone();
        let token = owner.scheduler.get().expect("native timeout runner")(
            ms,
            high_nesting,
            Box::new(move |runtime| {
                let _roots = captured_roots;
                if let Some(owner) = weak.upgrade().filter(|owner| !owner.destroyed.get()) {
                    let namespace = owner.namespace.borrow().Upgrade();
                    let _namespace = namespace
                        .as_ref()
                        .map(|namespace| runtime.EnterHostObjectNamespace(namespace));
                    let invocation = AbortInvocation {
                        owner: &owner,
                        runtime: RefCell::new(runtime),
                        native_events: Default::default(),
                        synthetic: None,
                    };
                    task.Run(&invocation);
                }
                _roots.Release();
            }),
        );
        Rc::new(TimeoutToken {
            token,
            roots: lease,
        })
    }
}
struct TaskRoots {
    owner: Weak<AbortBindings>,
    ids: Vec<u64>,
    active: Cell<bool>,
}
impl TaskRoots {
    fn Release(&self) {
        if self.active.replace(false) {
            if let Some(owner) = self.owner.upgrade() {
                for &id in &self.ids {
                    owner.Publish([C::Root {
                        id,
                        reason: 10,
                        rooted: false,
                    }]);
                }
            }
        }
    }
}
struct TimeoutToken {
    token: Rc<dyn CancellableAbortTask>,
    roots: Rc<TaskRoots>,
}
impl CancellableAbortTask for TimeoutToken {
    fn Cancel(&self) {
        self.token.Cancel();
        self.roots.Release();
    }
}

struct AbortInvocation<'a, 'runtime> {
    owner: &'a AbortBindings,
    runtime: RefCell<&'runtime mut dyn JavaScriptHostRuntime>,
    native_events: RefCell<Vec<u64>>,
    synthetic: Option<(HostValue, u64)>,
}
impl AbortSignalInvocation for AbortInvocation<'_, '_> {
    fn IsUndefined(&self, value: &Value) -> bool {
        self.runtime.borrow_mut().IsUndefined(&host(value))
    }
    fn Undefined(&self) -> Value {
        value(HostValue::default())
    }
    fn DOMException(&self, name: &'static str, message: &str) -> Value {
        let error = (self.owner.exception)(name, message);
        let captured = self.runtime.borrow_mut().CaptureExceptionStack(
            &error,
            &HostValue::String(name.into()),
            &HostValue::String(message.into()),
        );
        assert!(
            captured.thrown.is_none() && captured.exception.is_none(),
            "native AbortSignal DOMException stack capture"
        );
        value(captured.value)
    }
    fn KeepAlive(&self, values: &[Value]) -> Box<dyn Any> {
        Box::new(values.iter().map(host).collect::<Vec<_>>())
    }
    fn EventListenerInvoker(&self) -> &dyn ScopedEventListenerInvoker {
        self
    }
}
impl ScopedEventListenerInvoker for AbortInvocation<'_, '_> {
    fn Invoke(
        &self,
        selected: &EventListenerRef,
        _: Option<&dyn interaction::dispatch::core::dom::events::event_listener::ExecutionContext>,
        event: &EventRef,
    ) {
        let listener = Listener::From(selected.as_ref());
        let Some(callback) = listener.callback.Resolve() else {
            return;
        };
        let id = self
            .owner
            .events
            .borrow_mut()
            .BindNative(event, listener.target, || self.owner.Allocate());
        if !self.native_events.borrow().contains(&id) {
            self.native_events.borrow_mut().push(id);
        }
        use interaction::dispatch::core::dom::events::event::PassiveMode;
        let passive = !matches!(
            event.borrow().BaseEvent().HandlingPassive(),
            PassiveMode::kNotPassive | PassiveMode::kNotPassiveDefault
        );
        self.owner.events.borrow_mut().SetNativePassive(id, passive);
        if let (Some((argument, target)), Some(update)) = (
            self.synthetic.as_ref(),
            self.owner.event_state.borrow().clone(),
        ) {
            use interaction::dispatch::core::dom::events::event::{EventTargetKind, PhaseType};
            let (target, phase) = {
                let owner = event.borrow();
                let base = owner.BaseEvent();
                let target = base
                    .target()
                    .and_then(|event_target| match event_target.Kind() {
                        EventTargetKind::Window => Some(0),
                        EventTargetKind::Node => event_target.NodeId(),
                        EventTargetKind::Other => None,
                    })
                    .unwrap_or(*target);
                let phase = match base.eventPhase() {
                    PhaseType::kNone => 0,
                    PhaseType::kCapturingPhase => 1,
                    PhaseType::kAtTarget => 2,
                    PhaseType::kBubblingPhase => 3,
                };
                (target, phase)
            };
            self.runtime.borrow_mut().CallValue(
                &HostValue::JavaScriptFunction(update),
                &HostValue::Object(HostObjectRef { id: 0 }),
                &[
                    argument.clone(),
                    HostValue::Object(HostObjectRef { id: target }),
                    HostValue::Object(HostObjectRef {
                        id: listener.target,
                    }),
                    HostValue::Number(phase as f64),
                    HostValue::Boolean(passive),
                    HostValue::Object(HostObjectRef { id }),
                ],
            );
        }
        let wrapper = if listener.attribute {
            self.owner.handler_wrapper.borrow().clone()
        } else {
            None
        };
        let mut report = |runtime: &mut dyn JavaScriptHostRuntime, error: &HostValue| {
            runtime.ReportException(error);
        };
        event_listener_invocation::Invocation {
            listener,
            callback: &callback,
            runtime: &self.runtime,
            receiver: HostValue::Object(HostObjectRef {
                id: listener.target,
            }),
            argument: self
                .synthetic
                .as_ref()
                .map(|(argument, _)| argument.clone())
                .unwrap_or(HostValue::Object(HostObjectRef { id })),
            handler_wrapper: wrapper,
            current_event: self.owner.current_event.borrow().clone(),
            report_error: RefCell::new(&mut report),
        }
        .Invoke(event);
    }
}
impl Drop for AbortInvocation<'_, '_> {
    fn drop(&mut self) {
        for &id in self.native_events.borrow().iter() {
            self.owner.events.borrow_mut().EndNative(id);
            if let (Some((argument, target)), Some(update)) = (
                self.synthetic.as_ref(),
                self.owner.event_state.borrow().clone(),
            ) {
                self.runtime.borrow_mut().CallValue(
                    &HostValue::JavaScriptFunction(update),
                    &HostValue::Object(HostObjectRef { id: 0 }),
                    &[
                        argument.clone(),
                        HostValue::Object(HostObjectRef { id: *target }),
                        HostValue::Null(JavaScriptNull),
                        HostValue::Number(0.0),
                        HostValue::Boolean(false),
                        HostValue::Object(HostObjectRef { id }),
                    ],
                );
            }
        }
    }
}
