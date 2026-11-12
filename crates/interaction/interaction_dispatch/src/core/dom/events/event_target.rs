/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2008 Apple Inc. All rights reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2007, 2008 Nikolas Zimmermann <zimmermann@kde.org>
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY EXPRESS
 * OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL APPLE COMPUTER, INC. OR CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_target.h:1-449
// Source: third_party/blink/renderer/core/dom/events/event_target.cc:1-1176
// Rust modules replace includes, namespaces, GC and wrapper declarations.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

use super::add_event_listener_options_resolved::{
    AbortAlgorithmHandle, AbortSignalRef, AddEventListenerOptionsResolved,
};
use super::event::{Event, EventRef, EventTargetKind, EventTargetRef, PassiveMode, PhaseType};
use super::event_dispatch_forbidden_scope::EventDispatchForbiddenScope;
use super::event_dispatch_result::DispatchEventResult;
use super::event_listener::{EventListener, EventListenerRef, ExecutionContext};
use super::event_listener_map::{EventListenerMap, EventListenerVector};
use super::registered_event_listener::{
    OptionsForMatching, RegisteredEventListener, RegisteredEventListenerRef,
};

// Source: event_target.h:81-90
#[macro_export]
macro_rules! define_static_attribute_event_listener {
    ($get:ident, $set:ident, $event_type:expr) => {
        pub fn $get(
            target: &$crate::core::dom::events::event_target::EventTarget,
        ) -> Option<$crate::core::dom::events::event_listener::EventListenerRef> {
            target.GetAttributeEventListener($event_type)
        }
        pub fn $set(
            target: &$crate::core::dom::events::event_target::EventTarget,
            listener: Option<$crate::core::dom::events::event_listener::EventListenerRef>,
        ) {
            target.SetAttributeEventListener($event_type, listener);
        }
    };
}

// Source: event_target.h:92-100
#[macro_export]
macro_rules! define_window_attribute_event_listener {
    ($get:ident, $set:ident, $event_type:expr) => {
        pub fn $get(&self) -> Option<$crate::core::dom::events::event_listener::EventListenerRef> {
            self.GetDocumentForWindowEventHandler()
                .GetWindowAttributeEventListener($event_type)
        }
        pub fn $set(
            &self,
            listener: Option<$crate::core::dom::events::event_listener::EventListenerRef>,
        ) {
            self.GetDocumentForWindowEventHandler()
                .SetWindowAttributeEventListener($event_type, listener);
        }
    };
}

// Source: event_target.cc:873-884; local_dom_window.cc:2365-2379
// A package boundary may retain the JavaScript listener map in WebAPI. This
// port receives the canonical Event after the source target/phase assignment;
// listener selection and its registration order stay with that map's owner.
pub trait ExternalEventListenerInvoker {
    fn FireEventListeners(&self, event: &EventRef) -> DispatchEventResult;
}

// Source: event_target.h:50-65,142-162,351-400
// Ports retain the context/target/task owners supplied by the embedding.
// Listener selection, ordering, cancellation and passive policy stay below.
/// Scoped binding invocation receives exactly the listener selected below.
/// The caller may borrow its current script runtime for this synchronous call.
pub trait ScopedEventListenerInvoker {
    /// The current VM invocation, borrowed only for this dispatch stack. Native
    /// invokers without a VM scope return None; this never implies Continue.
    fn ScriptInvocation(
        &self,
    ) -> Option<&dyn crate::bindings::script_invocation::ScriptInvocation> {
        None
    }
    fn Invoke(
        &self,
        listener: &EventListenerRef,
        context: Option<&dyn ExecutionContext>,
        event: &EventRef,
    );
}

pub trait ScriptForbiddenScope {}
pub trait TaskAttributionState: std::any::Any {}
impl TaskAttributionState for foundation::task_attribution::TaskAttributionInfo {}
pub trait EventTargetHost {
    /// EventTarget.cc:1163-1167. The scope is provided by the actual executing
    /// Window's performance owner; dispatch never imports the default package.
    fn BeginNavigationEventTiming(&self, event: &EventRef) -> Box<dyn std::any::Any>;
    /// The source-local listener HeapVector is a stack root during dispatch,
    /// including entries removed by an earlier callback in that same vector.
    fn KeepAliveListeners(&self, _listeners: &[EventListenerRef]) -> Box<dyn std::any::Any> {
        Box::new(())
    }
    fn InterfaceName(&self) -> &str;
    fn GetExecutionContext(&self) -> Option<Rc<dyn ExecutionContext>>;
    fn Kind(&self) -> EventTargetKind;
    fn IsMainSequence(&self) -> bool;
    fn ClearCurrentTargetAfterDispatchEnabled(&self) -> bool;
    fn IsDocumentNode(&self) -> bool {
        false
    }
    fn IsDocumentElement(&self) -> bool {
        false
    }
    fn IsDocumentBody(&self) -> bool {
        false
    }
    fn IsLocalDOMWindow(&self) -> bool {
        false
    }
    fn IsMessagePort(&self) -> bool {
        false
    }
    fn IsServiceWorker(&self) -> bool {
        false
    }
    fn IsPseudoElement(&self) -> bool {
        false
    }
    /// Source event.cc:368-369; supplied by the live Node/SVGElement owner.
    fn IsSVGElement(&self) -> bool {
        false
    }
    fn CorrespondingElement(&self) -> Option<EventTargetRef> {
        None
    }
    fn IsWindowOrWorkerGlobalScope(&self) -> bool {
        false
    }
    fn KeepEventInNode(&self, _event: &Event) -> bool {
        false
    }
    fn ResetEventQueueStatus(&self, _event_type: &str) {}
    fn AddedEventListener(&self, _event_type: &str, _listener: &RegisteredEventListener) {}
    fn RemovedEventListener(&self, _event_type: &str, _listener: &RegisteredEventListener) {}
    /// Node::RemoveAllEventListeners, node.cc:3129-3148, updates its registry
    /// before clearing the base listener map.
    fn WillRemoveAllEventListeners(&self) {}
    /// LocalDOMWindow removes its registry membership after the base clear,
    /// local_dom_window.cc:2382-2405.
    fn DidRemoveAllEventListeners(&self) {}
    fn RegisterAbortAlgorithm(
        &self,
        listener: EventListenerRef,
        handle: Rc<dyn AbortAlgorithmHandle>,
    );
    fn EnterScriptForbiddenScope(&self) -> Box<dyn ScriptForbiddenScope>;
    fn PostEventTask(&self, task_type: u32, task: Box<dyn FnOnce()>);
    fn CaptureCurrentTaskState(&self) -> Option<Rc<dyn TaskAttributionState>>;
    // A derived source EventTarget (Node/Window) supplies its translated
    // DispatchEventInternal override, retaining this same target identity.
    fn DispatchEventInternal(
        &self,
        _target: &EventTarget,
        _event: &EventRef,
    ) -> Option<DispatchEventResult> {
        None
    }
}

// Source: event_target.cc:841-852
pub trait ExceptionState {
    fn ThrowInvalidStateError(&mut self, message: &'static str);
}

// Source: event_target.h:164-186; event_target.cc:533-564,737-758
// V8 projects its union and callback into these existing typed values.
pub enum AddEventListenerOptionsOrBoolean {
    Boolean(bool),
    Options(AddEventListenerOptionsResolved),
}
pub enum EventListenerOptionsOrBoolean {
    Boolean(bool),
    Options { capture: bool },
}

// Source: event_target.cc:87-105
fn EventPassiveMode(listener: &RegisteredEventListener) -> PassiveMode {
    if !listener.Passive() {
        return if listener.PassiveSpecified() {
            PassiveMode::kNotPassive
        } else {
            PassiveMode::kNotPassiveDefault
        };
    }
    if listener.PassiveForcedForDocumentTarget() {
        return PassiveMode::kPassiveForcedDocumentLevel;
    }
    if listener.PassiveSpecified() {
        return PassiveMode::kPassive;
    }
    PassiveMode::kPassiveDefault
}
// Source: event_target.cc:107-120
fn IsTouchScrollBlockingEvent(event_type: &str) -> bool {
    matches!(event_type, "touchstart" | "touchmove")
}
fn IsWheelScrollBlockingEvent(event_type: &str) -> bool {
    matches!(event_type, "mousewheel" | "wheel")
}
fn IsScrollBlockingEvent(event_type: &str) -> bool {
    IsTouchScrollBlockingEvent(event_type) || IsWheelScrollBlockingEvent(event_type)
}
// Source: event_target.cc:122-219
// TODO: blocked-event PerformanceMonitor/UseCounter instrumentation. These
// source helpers do not select listeners or change event cancellation.

// Source: event_target.cc:221-244
pub trait ObservableScriptState {
    fn ContextIsValid(&self) -> bool;
    fn EnterScope(&self) -> Box<dyn ObservableScriptScope>;
}
pub trait ObservableScriptScope {}
pub trait ObservableSubscriber {
    fn active(&self) -> bool;
    fn signal(&self) -> AbortSignalRef;
    // The binding converts the same EventRef to its ScriptValue projection.
    fn next(&self, event: &EventRef);
}
#[derive(Clone, Default)]
pub struct ObservableEventListenerOptions {
    pub capture: Option<bool>,
    pub passive: Option<bool>,
}
struct ObservableEventListener {
    subscriber_: Rc<dyn ObservableSubscriber>,
    script_state_: Rc<dyn ObservableScriptState>,
}
impl ObservableEventListener {
    // Source: event_target.cc:246-276
    fn Create(
        subscriber: Rc<dyn ObservableSubscriber>,
        script_state: Rc<dyn ObservableScriptState>,
        event_type: &str,
        target: &EventTarget,
        options: &ObservableEventListenerOptions,
    ) {
        assert!(subscriber.active());
        let mut resolved = AddEventListenerOptionsResolved::default();
        if let Some(capture) = options.capture {
            resolved.setCapture(capture);
        }
        if let Some(passive) = options.passive {
            resolved.setPassive(passive);
        }
        resolved.setSignal(subscriber.signal());
        let listener: EventListenerRef = Rc::new(Self {
            subscriber_: subscriber,
            script_state_: script_state,
        });
        target.addEventListenerWithOptions(event_type, Some(listener), resolved);
    }
}
impl EventListener for ObservableEventListener {
    // Source: event_target.cc:278-288
    fn Invoke(&self, _context: Option<&dyn ExecutionContext>, event: &EventRef) {
        debug_assert!(self.script_state_.ContextIsValid());
        let _scope = self.script_state_.EnterScope();
        self.subscriber_.next(event);
    }
    fn Matches(&self, other: &dyn EventListener) -> bool {
        self.Identity() == other.Identity()
    }
    fn IsNativeEventListener(&self) -> bool {
        true
    }
}
// Source: event_target.cc:290-295
// Rc owners provide the GC tracing edges for subscriber and script state.

// Source: event_target.cc:297-336
pub struct ObservableSubscribeDelegate {
    event_target_: Weak<EventTarget>,
    event_type_: String,
    options_: ObservableEventListenerOptions,
}
impl ObservableSubscribeDelegate {
    // Source: event_target.cc:338-371
    pub fn OnSubscribe(
        &self,
        subscriber: Rc<dyn ObservableSubscriber>,
        script_state: Rc<dyn ObservableScriptState>,
    ) {
        assert!(script_state.ContextIsValid());
        let Some(target) = self.event_target_.upgrade() else {
            return;
        };
        if subscriber.signal().aborted() {
            return;
        }
        ObservableEventListener::Create(
            subscriber,
            script_state,
            &self.event_type_,
            &target,
            &self.options_,
        );
    }
}
// Source: event_target.cc:373-378
// Weak/Rc retain exactly the source weak target and strong options owners.

// Source: event_target.h:102-113; event_target.cc:382-388
#[derive(Default)]
pub struct EventTargetData {
    pub event_listener_map: EventListenerMap,
}

// Source: event_target.h:136-143,402; event_target.cc:390-392
// A weak self pointer expresses source WrapWeakPersistent(this) without
// changing the target's identity or extending its lifetime.
pub struct EventTarget {
    host_: Rc<dyn EventTargetHost>,
    data_: RefCell<Option<EventTargetData>>,
    self_: Weak<EventTarget>,
}
impl EventTarget {
    pub fn new(host: Rc<dyn EventTargetHost>) -> Rc<Self> {
        Rc::new_cyclic(|weak| Self {
            host_: host,
            data_: RefCell::new(None),
            self_: weak.clone(),
        })
    }
    pub fn InterfaceName(&self) -> &str {
        self.host_.InterfaceName()
    }
    pub fn GetExecutionContext(&self) -> Option<Rc<dyn ExecutionContext>> {
        self.host_.GetExecutionContext()
    }
    pub fn AsEventTargetRef(&self) -> EventTargetRef {
        self.self_.upgrade().expect("live EventTarget owner")
    }
    // Source: event_target.cc:394-418
    pub fn ToNode(&self) -> Option<EventTargetRef> {
        (self.host_.Kind() == EventTargetKind::Node).then(|| self.AsEventTargetRef())
    }
    pub fn ToDOMWindow(&self) -> Option<EventTargetRef> {
        (self.host_.Kind() == EventTargetKind::Window).then(|| self.AsEventTargetRef())
    }
    pub fn ToLocalDOMWindow(&self) -> Option<EventTargetRef> {
        self.host_
            .IsLocalDOMWindow()
            .then(|| self.AsEventTargetRef())
    }
    pub fn ToMessagePort(&self) -> Option<EventTargetRef> {
        self.host_.IsMessagePort().then(|| self.AsEventTargetRef())
    }
    pub fn ToServiceWorker(&self) -> Option<EventTargetRef> {
        self.host_
            .IsServiceWorker()
            .then(|| self.AsEventTargetRef())
    }
    pub fn ResetEventQueueStatus(&self, event_type: &str) {
        self.host_.ResetEventQueueStatus(event_type);
    }
    // Source: event_target.cc:420-431
    pub fn Create(host: Rc<dyn EventTargetHost>) -> Rc<super::event_target_impl::EventTargetImpl> {
        super::event_target_impl::EventTargetImpl::new(host)
    }
    // Source: event_target.cc:433-451
    pub fn IsTopLevelNode(&self) -> bool {
        if self.ToLocalDOMWindow().is_some() {
            return true;
        }
        if self.ToNode().is_none() {
            return false;
        }
        self.host_.IsDocumentNode() || self.host_.IsDocumentElement() || self.host_.IsDocumentBody()
    }
    // Source: event_target.cc:453-518
    fn SetDefaultAddEventListenerOptions(
        &self,
        event_type: &str,
        options: &mut AddEventListenerOptionsResolved,
    ) {
        options.SetPassiveSpecified(options.hasPassive());
        if !IsScrollBlockingEvent(event_type) {
            if !options.hasPassive() {
                options.setPassive(false);
            }
            return;
        }
        // TODO: event_target.cc:465-473 passive UseCounter instrumentation.
        if IsTouchScrollBlockingEvent(event_type) && !options.hasPassive() && self.IsTopLevelNode()
        {
            options.setPassive(true);
            options.SetPassiveForcedForDocumentTarget(true);
            return;
        }
        if IsWheelScrollBlockingEvent(event_type) && self.IsTopLevelNode() && !options.hasPassive()
        {
            // TODO: event_target.cc:484-497 document-wheel UseCounter.
            options.setPassive(true);
            options.SetPassiveForcedForDocumentTarget(true);
            return;
        }
        if !options.hasPassive() {
            options.setPassive(false);
        }
        // TODO: event_target.cc:507-517 discouraged non-passive API warning.
    }
    // Source: event_target.cc:520-525
    pub fn when(
        &self,
        event_type: &str,
        options: ObservableEventListenerOptions,
    ) -> ObservableSubscribeDelegate {
        ObservableSubscribeDelegate {
            event_target_: self.self_.clone(),
            event_type_: event_type.into(),
            options_: options,
        }
    }
    // Source: event_target.cc:527-564
    // Binding callback conversion supplies the same EventListener owner.
    pub fn addEventListenerForBindings(
        &self,
        event_type: &str,
        listener: Option<EventListenerRef>,
        options: AddEventListenerOptionsOrBoolean,
    ) -> bool {
        match options {
            AddEventListenerOptionsOrBoolean::Boolean(capture) => {
                self.addEventListener(event_type, listener, capture)
            }
            AddEventListenerOptionsOrBoolean::Options(options) => {
                self.addEventListenerWithOptions(event_type, listener, options)
            }
        }
    }
    // Source: event_target.cc:566-580
    pub fn addEventListener(
        &self,
        event_type: &str,
        listener: Option<EventListenerRef>,
        use_capture: bool,
    ) -> bool {
        let mut options = AddEventListenerOptionsResolved::default();
        options.setCapture(use_capture);
        self.SetDefaultAddEventListenerOptions(event_type, &mut options);
        self.AddEventListenerInternal(event_type, listener, &options)
    }
    pub fn addEventListenerWithOptions(
        &self,
        event_type: &str,
        listener: Option<EventListenerRef>,
        mut options: AddEventListenerOptionsResolved,
    ) -> bool {
        self.SetDefaultAddEventListenerOptions(event_type, &mut options);
        self.AddEventListenerInternal(event_type, listener, &options)
    }
    // Source: event_target.cc:582-680
    pub fn AddEventListenerInternal(
        &self,
        event_type: &str,
        listener: Option<EventListenerRef>,
        options: &AddEventListenerOptionsResolved,
    ) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        if options.signal().is_some_and(|signal| signal.aborted()) {
            return false;
        }
        let Some(context) = self.GetExecutionContext() else {
            return false;
        };
        if matches!(event_type, "unload" | "beforeunload")
            && context.IsWindow()
            && context.IsInFencedFrameTree()
        {
            // TODO: event_target.cc:604-605 console error instrumentation.
            return false;
        }
        if event_type == "unload"
            && !context.DeprecateUnloadOptOutEnabled()
            && !context.UnloadFeatureEnabled()
        {
            return false;
        }
        // TODO: event_target.cc:623-644 touch UseCounter/activity logger.
        let mut registered = None;
        let added = self.EnsureEventTargetData().event_listener_map.Add(
            event_type,
            listener.clone(),
            options,
            &mut registered,
        );
        if added {
            let registered = registered.expect("added listener must be registered");
            if let Some(signal) = options.signal() {
                let target = self.self_.clone();
                let listener_weak = Rc::downgrade(&listener);
                let event_type = event_type.to_owned();
                let capture = options.capture();
                let handle = signal.AddAlgorithm(Box::new(move || {
                    if let (Some(target), Some(listener)) =
                        (target.upgrade(), listener_weak.upgrade())
                    {
                        target.removeEventListener(&event_type, Some(listener.as_ref()), capture);
                    }
                }));
                if let Some(handle) = handle {
                    self.host_.RegisterAbortAlgorithm(listener, handle);
                }
                // TODO: event_target.cc:670-674 abort-signal UseCounter.
            }
            self.AddedEventListener(event_type, &registered);
        }
        added
    }
    // Source: event_target.cc:682-729
    pub fn AddedEventListener(&self, event_type: &str, registered: &RegisteredEventListener) {
        // TODO: source default body counts document/worker WebFeatures.
        self.host_.AddedEventListener(event_type, registered);
    }
    // Source: event_target.cc:731-772
    pub fn removeEventListenerForBindings(
        &self,
        event_type: &str,
        listener: Option<&dyn EventListener>,
        options: EventListenerOptionsOrBoolean,
    ) -> bool {
        let capture = match options {
            EventListenerOptionsOrBoolean::Boolean(value)
            | EventListenerOptionsOrBoolean::Options { capture: value } => value,
        };
        self.removeEventListener(event_type, listener, capture)
    }
    pub fn removeEventListener(
        &self,
        event_type: &str,
        listener: Option<&dyn EventListener>,
        use_capture: bool,
    ) -> bool {
        self.RemoveEventListenerInternal(
            event_type,
            listener,
            &OptionsForMatching::new(use_capture),
        )
    }
    // Source: event_target.cc:774-799
    pub fn RemoveEventListenerInternal(
        &self,
        event_type: &str,
        listener: Option<&dyn EventListener>,
        options: &OptionsForMatching,
    ) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        let mut registered = None;
        let removed = {
            let mut data = self.data_.borrow_mut();
            let Some(data) = data.as_mut() else {
                return false;
            };
            data.event_listener_map
                .Remove(event_type, listener, options, &mut registered)
        };
        if !removed {
            return false;
        }
        self.host_.RemovedEventListener(
            event_type,
            &registered.expect("removed listener must be retained"),
        );
        true
    }
    // Source: event_target.cc:801-814
    fn GetAttributeRegisteredEventListener(
        &self,
        event_type: &str,
    ) -> Option<RegisteredEventListenerRef> {
        let vector = self.GetEventListeners(event_type)?;
        let context = self.GetExecutionContext()?;
        let registered = vector
            .borrow()
            .iter()
            .find(|registered| {
                let listener = registered.Callback();
                listener.as_ref().is_some_and(|listener| {
                    listener.IsEventHandler()
                        && listener.BelongsToTheCurrentWorld(Some(context.as_ref()))
                })
            })
            .cloned();
        registered
    }
    // Source: event_target.cc:816-839
    pub fn SetAttributeEventListener(
        &self,
        event_type: &str,
        listener: Option<EventListenerRef>,
    ) -> bool {
        let registered = self.GetAttributeRegisteredEventListener(event_type);
        let Some(listener) = listener else {
            if let Some(registered) = registered {
                if let Some(callback) = registered.Callback() {
                    self.removeEventListener(event_type, Some(callback.as_ref()), false);
                }
            }
            return false;
        };
        if let Some(registered) = registered {
            registered.SetCallback(Some(listener));
            return true;
        }
        self.addEventListener(event_type, Some(listener), false)
    }
    pub fn GetAttributeEventListener(&self, event_type: &str) -> Option<EventListenerRef> {
        self.GetAttributeRegisteredEventListener(event_type)
            .and_then(|registered| registered.Callback())
    }
    // Source: event_target.cc:841-864
    pub fn dispatchEventForBindings(
        &self,
        event: &EventRef,
        exceptions: &mut dyn ExceptionState,
    ) -> bool {
        self.DispatchEventForBindingsInternal(event, exceptions, None)
    }
    /// A constructible native target borrows the current binding invocation;
    /// it retains neither a runtime nor a second listener-selection path.
    pub fn dispatchEventForBindingsWithInvoker(
        &self,
        event: &EventRef,
        exceptions: &mut dyn ExceptionState,
        invoker: &dyn ScopedEventListenerInvoker,
    ) -> bool {
        debug_assert_eq!(self.host_.Kind(), EventTargetKind::Other);
        self.DispatchEventForBindingsInternal(event, exceptions, Some(invoker))
    }
    fn DispatchEventForBindingsInternal(
        &self,
        event: &EventRef,
        exceptions: &mut dyn ExceptionState,
        invoker: Option<&dyn ScopedEventListenerInvoker>,
    ) -> bool {
        if !event.borrow().BaseEvent().WasInitialized() {
            exceptions.ThrowInvalidStateError("The event provided is uninitialized.");
            return false;
        }
        if event.borrow().BaseEvent().IsBeingDispatched() {
            exceptions.ThrowInvalidStateError("The event is already being dispatched.");
            return false;
        }
        if self.GetExecutionContext().is_none() {
            return false;
        }
        event.borrow_mut().BaseEventMut().SetTrusted(false);
        let result = if let Some(invoker) = invoker {
            self.DispatchEventInternalWithTarget(event, self.AsEventTargetRef(), Some(invoker))
        } else {
            self.DispatchEventInternal(event)
        };
        result != DispatchEventResult::kCanceledByEventHandler
    }
    // Source: event_target.cc:866-883
    pub fn DispatchEvent(&self, event: &EventRef) -> DispatchEventResult {
        if self.GetExecutionContext().is_none() {
            return DispatchEventResult::kCanceledBeforeDispatch;
        }
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        self.DispatchEventInternal(event)
    }
    /// Native direct-target dispatch with the embedding's existing listener
    /// projection. Keep ExecutionContext, trusted status and cleanup decisions
    /// on this canonical EventTarget, as in DispatchEvent above.
    pub fn DispatchEventWithExternalListeners(
        &self,
        event: &EventRef,
        listeners: &dyn ExternalEventListenerInvoker,
    ) -> DispatchEventResult {
        if self.GetExecutionContext().is_none() {
            return DispatchEventResult::kCanceledBeforeDispatch;
        }
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        Self::DispatchEventAtTarget(
            event,
            self.AsEventTargetRef(),
            self.AsEventTargetRef(),
            self.host_.ClearCurrentTargetAfterDispatchEnabled(),
            listeners,
        )
    }
    pub fn DispatchEventInternal(&self, event: &EventRef) -> DispatchEventResult {
        if let Some(result) = self.host_.DispatchEventInternal(self, event) {
            return result;
        }
        self.DispatchEventInternalWithTarget(event, self.AsEventTargetRef(), None)
    }

    /// Node/Window adapters retain their canonical interface target identity.
    /// The source direct-target phase and cleanup below remain unchanged.
    pub fn DispatchEventInternalWithTarget(
        &self,
        event: &EventRef,
        target: EventTargetRef,
        invoker: Option<&dyn ScopedEventListenerInvoker>,
    ) -> DispatchEventResult {
        self.DispatchEventInternalWithCurrentTarget(event, target.clone(), target, invoker)
    }

    // Source: local_dom_window.cc:2365-2379. The load caller passes Document
    // as target while the same Window remains the invocation/currentTarget.
    pub fn DispatchWindowEventWithTarget(
        &self,
        event: &EventRef,
        window: EventTargetRef,
        legacy_target: EventTargetRef,
        invoker: Option<&dyn ScopedEventListenerInvoker>,
    ) -> DispatchEventResult {
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        self.DispatchEventInternalWithCurrentTarget(event, legacy_target, window, invoker)
    }

    fn DispatchEventInternalWithCurrentTarget(
        &self,
        event: &EventRef,
        target: EventTargetRef,
        current_target: EventTargetRef,
        invoker: Option<&dyn ScopedEventListenerInvoker>,
    ) -> DispatchEventResult {
        struct LocalListeners<'a> {
            target: &'a EventTarget,
            invoker: Option<&'a dyn ScopedEventListenerInvoker>,
        }
        impl ExternalEventListenerInvoker for LocalListeners<'_> {
            fn FireEventListeners(&self, event: &EventRef) -> DispatchEventResult {
                self.target
                    .FireEventListenersWithInvoker(event, self.invoker)
            }
        }
        Self::DispatchEventAtTarget(
            event,
            target,
            current_target,
            self.host_.ClearCurrentTargetAfterDispatchEnabled(),
            &LocalListeners {
                target: self,
                invoker,
            },
        )
    }

    // Source: event_target.cc:873-884. The lifecycle is shared with ordinary
    // EventTarget dispatch, including Window's explicit currentTarget override.
    // The external callback cannot choose target, phase, or cleanup order.
    pub fn DispatchEventAtTarget(
        event: &EventRef,
        target: EventTargetRef,
        current_target: EventTargetRef,
        clear_current_target: bool,
        listeners: &dyn ExternalEventListenerInvoker,
    ) -> DispatchEventResult {
        {
            let mut event = event.borrow_mut();
            event.SetTarget(Some(target));
            event.BaseEventMut().SetCurrentTarget(Some(current_target));
            event.BaseEventMut().SetEventPhase(PhaseType::kAtTarget);
        }
        let result = listeners.FireEventListeners(event);
        let mut event = event.borrow_mut();
        if clear_current_target {
            event.BaseEventMut().SetCurrentTarget(None);
        }
        event.BaseEventMut().SetEventPhase(PhaseType::kNone);
        result
    }
    // Source: event_target.cc:885-890
    pub fn EnsureEventTargetData(&self) -> RefMut<'_, EventTargetData> {
        RefMut::map(self.data_.borrow_mut(), |data| {
            data.get_or_insert_with(EventTargetData::default)
        })
    }
    // Source: event_target.h:228-229
    pub fn GetEventTargetData(&self) -> Option<Ref<'_, EventTargetData>> {
        Ref::filter_map(self.data_.borrow(), |data| data.as_ref()).ok()
    }

    // Source: event_target.cc:960-1002
    pub fn FireEventListeners(&self, event: &EventRef) -> DispatchEventResult {
        self.FireEventListenersWithInvoker(event, None)
    }

    pub fn FireEventListenersWithInvoker(
        &self,
        event: &EventRef,
        invoker: Option<&dyn ScopedEventListenerInvoker>,
    ) -> DispatchEventResult {
        debug_assert!(!EventDispatchForbiddenScope::IsEventDispatchForbidden(
            self.host_.IsMainSequence()
        ));
        debug_assert!(event.borrow().BaseEvent().WasInitialized());
        if self.GetEventTargetData().is_none() {
            return DispatchEventResult::kNotCanceled;
        }
        let event_type = event.borrow().BaseEvent().r#type().to_owned();
        let legacy = LegacyType(&event_type);
        let legacy_listeners = (!legacy.is_empty())
            .then(|| self.GetEventListeners(legacy))
            .flatten();
        let listeners = self.GetEventListeners(&event_type);
        let trusted = event.borrow().BaseEvent().isTrusted();
        let fired = if let Some(listeners) = listeners {
            let snapshot = listeners.borrow().clone();
            self.FireEventListenersSnapshot(event, snapshot, invoker)
        } else if trusted {
            if let Some(legacy_listeners) = legacy_listeners {
                event.borrow_mut().BaseEventMut().SetType(legacy);
                let snapshot = legacy_listeners.borrow().clone();
                let fired = self.FireEventListenersSnapshot(event, snapshot, invoker);
                event.borrow_mut().BaseEventMut().SetType(event_type);
                fired
            } else {
                false
            }
        } else {
            false
        };
        if fired {
            event.borrow_mut().DoneDispatchingEventAtCurrentTarget();
            // TODO: event_target.cc:996-999 Editor::CountEvent/CountLegacyEvents.
        }
        Self::GetDispatchEventResult(event.borrow().BaseEvent())
    }
    // Source: event_target.cc:1004-1096
    fn FireEventListenersSnapshot(
        &self,
        event: &EventRef,
        entry: Vec<RegisteredEventListenerRef>,
        invoker: Option<&dyn ScopedEventListenerInvoker>,
    ) -> bool {
        let callback_roots: Vec<_> = entry.iter().filter_map(|entry| entry.Callback()).collect();
        let _listener_roots = self.host_.KeepAliveListeners(&callback_roots);
        let Some(context) = self.GetExecutionContext() else {
            return false;
        };
        // TODO: event_target.cc:1018-1028 firing counts/blocked-event timing.
        let mut fired_listener = false;
        {
            let _no_script = self.host_.EnterScriptForbiddenScope();
            for registered in &entry {
                if registered.IsAnimationTrigger()
                    && registered.ShouldFire(event.borrow().BaseEvent())
                {
                    let listener = registered.Callback().expect("registered callback");
                    if registered.Once() {
                        let event_type = event.borrow().BaseEvent().r#type().to_owned();
                        self.removeEventListener(
                            &event_type,
                            Some(listener.as_ref()),
                            registered.Capture(),
                        );
                    }
                    if let Some(invoker) = invoker {
                        invoker.Invoke(&listener, Some(context.as_ref()), event);
                    } else {
                        listener.Invoke(Some(context.as_ref()), event);
                    }
                }
            }
        }
        for registered in entry {
            if registered.IsAnimationTrigger() {
                continue;
            }
            if registered.Removed() {
                continue;
            }
            if event.borrow().BaseEvent().ImmediatePropagationStopped() {
                break;
            }
            if !registered.ShouldFire(event.borrow().BaseEvent()) {
                continue;
            }
            let listener = registered.Callback().expect("registered callback");
            if registered.Once() {
                let event_type = event.borrow().BaseEvent().r#type().to_owned();
                self.removeEventListener(
                    &event_type,
                    Some(listener.as_ref()),
                    registered.Capture(),
                );
            }
            event
                .borrow_mut()
                .BaseEventMut()
                .SetHandlingPassive(EventPassiveMode(&registered));
            // TODO: event_target.cc:1076 UserCallback probe. JS listener bindings
            // handle/report their own exceptions just as source Invoke does.
            if let Some(invoker) = invoker {
                invoker.Invoke(&listener, Some(context.as_ref()), event);
            } else {
                listener.Invoke(Some(context.as_ref()), event);
            }
            fired_listener = true;
            // TODO: event_target.cc:1083-1091 blocked listener warning.
            event
                .borrow_mut()
                .BaseEventMut()
                .SetHandlingPassive(PassiveMode::kNotPassive);
        }
        fired_listener
    }
    // Source: event_target.cc:1098-1104
    pub fn GetDispatchEventResult(event: &Event) -> DispatchEventResult {
        if event.defaultPrevented() {
            return DispatchEventResult::kCanceledByEventHandler;
        }
        if event.DefaultHandled() {
            return DispatchEventResult::kCanceledByDefaultEventHandler;
        }
        DispatchEventResult::kNotCanceled
    }
    // Source: event_target.cc:1106-1120
    pub fn GetEventListeners(&self, event_type: &str) -> Option<EventListenerVector> {
        self.GetEventTargetData()?
            .event_listener_map
            .Find(event_type)
    }
    // Source: event_target.cc:1122-1130
    pub fn NumberOfEventListeners(&self, event_type: &str) -> usize {
        self.GetEventListeners(event_type)
            .map_or(0, |listeners| listeners.borrow().len())
    }
    pub fn EventTypes(&self) -> Vec<String> {
        self.GetEventTargetData()
            .map_or_else(Vec::new, |data| data.event_listener_map.EventTypes())
    }
    // Source: event_target.cc:1132-1136
    pub fn RemoveAllEventListeners(&self) {
        self.host_.WillRemoveAllEventListeners();
        if let Some(data) = self.data_.borrow_mut().as_mut() {
            data.event_listener_map.Clear();
        }
        self.host_.DidRemoveAllEventListeners();
    }
    // Source: event_target.cc:1138-1148
    pub fn EnqueueEvent(&self, event: EventRef, task_type: u32) {
        let Some(context) = self.GetExecutionContext() else {
            return;
        };
        let target = self.self_.upgrade().expect("enqueued target owner");
        let task_state = self.host_.CaptureCurrentTaskState();
        // TODO: event_target.cc:1142 AsyncTaskContext Schedule instrumentation.
        self.host_.PostEventTask(
            task_type,
            Box::new(move || target.DispatchEnqueuedEvent(event, context, task_state)),
        );
    }
    // Source: event_target.cc:1150-1169
    fn DispatchEnqueuedEvent(
        &self,
        event: EventRef,
        _context: Rc<dyn ExecutionContext>,
        _task_state: Option<Rc<dyn TaskAttributionState>>,
    ) {
        if self.GetExecutionContext().is_none() {
            // TODO: event_target.cc:1155 AsyncTaskContext Cancel instrumentation.
            return;
        }
        let event_type = event.borrow().BaseEvent().r#type().to_owned();
        self.ResetEventQueueStatus(&event_type);
        // TODO: event_target.cc:1159-1162 AsyncTask/TaskScope instrumentation.
        let _event_timing = self.host_.BeginNavigationEventTiming(&event);
        self.DispatchEvent(&event);
    }
    // Source: event_target.h:220-222
    pub fn KeepEventInNode(&self, event: &Event) -> bool {
        self.host_.KeepEventInNode(event)
    }
    pub fn IsWindowOrWorkerGlobalScope(&self) -> bool {
        self.host_.IsWindowOrWorkerGlobalScope()
    }
    // Source: event_target.h:407-445
    pub fn HasEventListeners(&self) -> bool {
        self.GetEventTargetData()
            .is_some_and(|data| !data.event_listener_map.IsEmpty())
    }
    pub fn HasEventListenersForType(&self, event_type: &str) -> bool {
        self.GetEventTargetData()
            .is_some_and(|data| data.event_listener_map.Contains(event_type))
    }
    pub fn HasAnyEventListeners(&self, event_types: &[String]) -> bool {
        event_types
            .iter()
            .any(|event_type| self.HasEventListenersForType(event_type))
    }
    pub fn HasAnyCapturingEventListeners(&self) -> bool {
        self.GetEventTargetData()
            .is_some_and(|data| data.event_listener_map.ContainsAnyCapturing())
    }
    pub fn HasCapturingEventListeners(&self, event_type: &str) -> bool {
        self.GetEventTargetData()
            .is_some_and(|data| data.event_listener_map.ContainsCapturing(event_type))
    }
    pub fn HasJSBasedEventListeners(&self, event_type: &str) -> bool {
        self.GetEventTargetData().is_some_and(|data| {
            data.event_listener_map
                .ContainsJSBasedEventListeners(event_type)
        })
    }
}

// Source: event_target.cc:892-909
fn LegacyType(event_type: &str) -> &'static str {
    match event_type {
        "transitionend" => "webkitTransitionEnd",
        "animationstart" => "webkitAnimationStart",
        "animationend" => "webkitAnimationEnd",
        "animationiteration" => "webkitAnimationIteration",
        "wheel" => "mousewheel",
        _ => "",
    }
}
// Source: event_target.cc:911-958
// TODO: CountLegacyEvents is UseCounter-only instrumentation.

// Source: event_target.h:136-150
impl super::event::EventTarget for EventTarget {
    fn AsAbstractView(&self) -> Option<crate::core::events::ui_event::AbstractViewRef> {
        (self.host_.Kind() == EventTargetKind::Window)
            .then(|| self.self_.upgrade().expect("live native Window") as _)
    }
    fn AsNativeEventTarget(&self) -> Option<Rc<Self>> {
        (self.host_.Kind() == EventTargetKind::Other)
            .then(|| self.self_.upgrade().expect("live native EventTargetImpl"))
    }
    fn Kind(&self) -> EventTargetKind {
        self.host_.Kind()
    }
    fn IsPseudoElement(&self) -> bool {
        self.host_.IsPseudoElement()
    }
    fn IsSVGElement(&self) -> bool {
        self.host_.IsSVGElement()
    }
    fn CorrespondingElement(&self) -> Option<EventTargetRef> {
        self.host_.CorrespondingElement()
    }
}
// Source: event_target.cc:1171-1174
// Rust Rc/RefCell owners provide ScriptWrappable/data GC tracing edges.

// Source: event_target.h:73-90,235-347
// Generated accessors preserve the header macros and their source event names.
macro_rules! attribute_event_listeners {
    ($(($get:ident, $set:ident, $name:literal)),* $(,)?) => {
        impl EventTarget { $(
            pub fn $get(&self) -> Option<EventListenerRef> { self.GetAttributeEventListener($name) }
            pub fn $set(&self, listener: Option<EventListenerRef>) { self.SetAttributeEventListener($name, listener); }
        )* }
    };
}

attribute_event_listeners! {
    (onabort, setOnabort, "abort"),
    (onanimationcancel, setOnanimationcancel, "animationcancel"),
    (onanimationend, setOnanimationend, "animationend"),
    (onanimationiteration, setOnanimationiteration, "animationiteration"),
    (onanimationstart, setOnanimationstart, "animationstart"),
    (onauxclick, setOnauxclick, "auxclick"),
    (onbeforefilter, setOnbeforefilter, "beforefilter"),
    (onbeforeinput, setOnbeforeinput, "beforeinput"),
    (onbeforematch, setOnbeforematch, "beforematch"),
    (onbeforetoggle, setOnbeforetoggle, "beforetoggle"),
    (onblur, setOnblur, "blur"),
    (oncancel, setOncancel, "cancel"),
    (oncanplay, setOncanplay, "canplay"),
    (oncanplaythrough, setOncanplaythrough, "canplaythrough"),
    (onchange, setOnchange, "change"),
    (onclick, setOnclick, "click"),
    (onclose, setOnclose, "close"),
    (oncommand, setOncommand, "command"),
    (oncomplete, setOncomplete, "complete"),
    (oncontentvisibilityautostatechange, setOncontentvisibilityautostatechange, "contentvisibilityautostatechange"),
    (oncontextmenu, setOncontextmenu, "contextmenu"),
    (oncontextlost, setOncontextlost, "contextlost"),
    (oncontextrestored, setOncontextrestored, "contextrestored"),
    (oncuechange, setOncuechange, "cuechange"),
    (ondblclick, setOndblclick, "dblclick"),
    (ondrag, setOndrag, "drag"),
    (ondragend, setOndragend, "dragend"),
    (ondragenter, setOndragenter, "dragenter"),
    (ondragleave, setOndragleave, "dragleave"),
    (ondragover, setOndragover, "dragover"),
    (ondragstart, setOndragstart, "dragstart"),
    (ondrop, setOndrop, "drop"),
    (ondurationchange, setOndurationchange, "durationchange"),
    (onemptied, setOnemptied, "emptied"),
    (onended, setOnended, "ended"),
    (onerror, setOnerror, "error"),
    (onfocus, setOnfocus, "focus"),
    (onformdata, setOnformdata, "formdata"),
    (ongotpointercapture, setOngotpointercapture, "gotpointercapture"),
    (oninput, setOninput, "input"),
    (oninvalid, setOninvalid, "invalid"),
    (onkeydown, setOnkeydown, "keydown"),
    (onkeypress, setOnkeypress, "keypress"),
    (onkeyup, setOnkeyup, "keyup"),
    (onload, setOnload, "load"),
    (onloadeddata, setOnloadeddata, "loadeddata"),
    (onloadedmetadata, setOnloadedmetadata, "loadedmetadata"),
    (onloadstart, setOnloadstart, "loadstart"),
    (onlostpointercapture, setOnlostpointercapture, "lostpointercapture"),
    (onmousedown, setOnmousedown, "mousedown"),
    (onmouseenter, setOnmouseenter, "mouseenter"),
    (onmouseleave, setOnmouseleave, "mouseleave"),
    (onmousemove, setOnmousemove, "mousemove"),
    (onmouseout, setOnmouseout, "mouseout"),
    (onmouseover, setOnmouseover, "mouseover"),
    (onmouseup, setOnmouseup, "mouseup"),
    (onmousewheel, setOnmousewheel, "mousewheel"),
    (onoverscrollcancel, setOnoverscrollcancel, "overscrollcancel"),
    (onoverscrollchanging, setOnoverscrollchanging, "overscrollchanging"),
    (onoverscrollend, setOnoverscrollend, "overscrollend"),
    (onoverscrollstart, setOnoverscrollstart, "overscrollstart"),
    (onpause, setOnpause, "pause"),
    (onplay, setOnplay, "play"),
    (onplaying, setOnplaying, "playing"),
    (onpointercancel, setOnpointercancel, "pointercancel"),
    (onpointerdown, setOnpointerdown, "pointerdown"),
    (onpointerenter, setOnpointerenter, "pointerenter"),
    (onpointerleave, setOnpointerleave, "pointerleave"),
    (onpointermove, setOnpointermove, "pointermove"),
    (onpointerout, setOnpointerout, "pointerout"),
    (onpointerover, setOnpointerover, "pointerover"),
    (onpointerrawupdate, setOnpointerrawupdate, "pointerrawupdate"),
    (onpointerup, setOnpointerup, "pointerup"),
    (onpopoverhide, setOnpopoverhide, "popoverhide"),
    (onpopovershow, setOnpopovershow, "popovershow"),
    (onprogress, setOnprogress, "progress"),
    (onratechange, setOnratechange, "ratechange"),
    (onreset, setOnreset, "reset"),
    (onresize, setOnresize, "resize"),
    (onscroll, setOnscroll, "scroll"),
    (onscrollend, setOnscrollend, "scrollend"),
    (onsecuritypolicyviolation, setOnsecuritypolicyviolation, "securitypolicyviolation"),
    (onseeked, setOnseeked, "seeked"),
    (onseeking, setOnseeking, "seeking"),
    (onselect, setOnselect, "select"),
    (onselectionchange, setOnselectionchange, "selectionchange"),
    (onselectstart, setOnselectstart, "selectstart"),
    (onslotchange, setOnslotchange, "slotchange"),
    (onscrollsnapchange, setOnscrollsnapchange, "scrollsnapchange"),
    (onscrollsnapchanging, setOnscrollsnapchanging, "scrollsnapchanging"),
    (onstalled, setOnstalled, "stalled"),
    (onsubmit, setOnsubmit, "submit"),
    (onsuspend, setOnsuspend, "suspend"),
    (ontimeupdate, setOntimeupdate, "timeupdate"),
    (ontoggle, setOntoggle, "toggle"),
    (ontouchcancel, setOntouchcancel, "touchcancel"),
    (ontouchend, setOntouchend, "touchend"),
    (ontouchmove, setOntouchmove, "touchmove"),
    (ontouchstart, setOntouchstart, "touchstart"),
    (ontransitioncancel, setOntransitioncancel, "transitioncancel"),
    (ontransitionend, setOntransitionend, "transitionend"),
    (ontransitionrun, setOntransitionrun, "transitionrun"),
    (ontransitionstart, setOntransitionstart, "transitionstart"),
    (onvolumechange, setOnvolumechange, "volumechange"),
    (onwaiting, setOnwaiting, "waiting"),
    (onwebkitanimationend, setOnwebkitanimationend, "webkitAnimationEnd"),
    (onwebkitanimationiteration, setOnwebkitanimationiteration, "webkitAnimationIteration"),
    (onwebkitanimationstart, setOnwebkitanimationstart, "webkitAnimationStart"),
    (onwebkittransitionend, setOnwebkittransitionend, "webkitTransitionEnd"),
    (onwheel, setOnwheel, "wheel"),
}

impl super::event_listener_map::EventListenerCopyTarget for EventTarget {
    fn addEventListener(
        &self,
        event_type: &str,
        listener: EventListenerRef,
        options: &AddEventListenerOptionsResolved,
    ) {
        self.addEventListenerWithOptions(event_type, Some(listener), options.clone());
    }
}
