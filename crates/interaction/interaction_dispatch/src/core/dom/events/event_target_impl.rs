// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_target_impl.h:1-37
// Source: third_party/blink/renderer/core/dom/events/event_target_impl.cc:1-28

use std::ops::Deref;
use std::rc::Rc;

use super::add_event_listener_options_resolved::AbortAlgorithmHandle;
use super::event::{EventTargetKind, EventTargetRef};
use super::event_listener::{EventListenerRef, ExecutionContext};
use super::event_target::{
    EventTarget, EventTargetHost, ScriptForbiddenScope, TaskAttributionState,
};

// Source: event_target_impl.h:17-32
// The constructible subclass embeds the same base owner. Its context client
// delegates to the embedding's live execution-context owner and services.
pub struct EventTargetImpl {
    target_: Rc<EventTarget>,
}

struct EventTargetImplHost {
    context_client_: Rc<dyn EventTargetHost>,
}
impl EventTargetHost for EventTargetImplHost {
    fn KeepAliveListeners(&self, listeners: &[EventListenerRef]) -> Box<dyn std::any::Any> {
        self.context_client_.KeepAliveListeners(listeners)
    }
    fn AddedEventListener(
        &self,
        kind: &str,
        listener: &super::registered_event_listener::RegisteredEventListener,
    ) {
        self.context_client_.AddedEventListener(kind, listener);
    }
    fn RemovedEventListener(
        &self,
        kind: &str,
        listener: &super::registered_event_listener::RegisteredEventListener,
    ) {
        self.context_client_.RemovedEventListener(kind, listener);
    }
    fn WillRemoveAllEventListeners(&self) {
        self.context_client_.WillRemoveAllEventListeners();
    }
    fn DidRemoveAllEventListeners(&self) {
        self.context_client_.DidRemoveAllEventListeners();
    }
    fn BeginNavigationEventTiming(&self, event: &super::event::EventRef) -> Box<dyn std::any::Any> {
        self.context_client_.BeginNavigationEventTiming(event)
    }
    // Source: event_target_impl.cc:12-14
    fn InterfaceName(&self) -> &str {
        "EventTargetImpl"
    }
    // Source: event_target_impl.cc:16-18
    fn GetExecutionContext(&self) -> Option<Rc<dyn ExecutionContext>> {
        self.context_client_.GetExecutionContext()
    }
    fn Kind(&self) -> EventTargetKind {
        EventTargetKind::Other
    }
    fn IsMainSequence(&self) -> bool {
        self.context_client_.IsMainSequence()
    }
    fn ClearCurrentTargetAfterDispatchEnabled(&self) -> bool {
        self.context_client_
            .ClearCurrentTargetAfterDispatchEnabled()
    }
    fn RegisterAbortAlgorithm(
        &self,
        listener: EventListenerRef,
        handle: Rc<dyn AbortAlgorithmHandle>,
    ) {
        self.context_client_
            .RegisterAbortAlgorithm(listener, handle);
    }
    fn EnterScriptForbiddenScope(&self) -> Box<dyn ScriptForbiddenScope> {
        self.context_client_.EnterScriptForbiddenScope()
    }
    fn PostEventTask(&self, task_type: u32, task: Box<dyn FnOnce()>) {
        self.context_client_.PostEventTask(task_type, task);
    }
    fn CaptureCurrentTaskState(&self) -> Option<Rc<dyn TaskAttributionState>> {
        self.context_client_.CaptureCurrentTaskState()
    }
}

impl EventTargetImpl {
    // Source: event_target_impl.cc:25-26
    // The supplied context client is ExecutionContext::From(script_state).
    pub fn new(context_client: Rc<dyn EventTargetHost>) -> Rc<Self> {
        Rc::new(Self {
            target_: EventTarget::new(Rc::new(EventTargetImplHost {
                context_client_: context_client,
            })),
        })
    }
    // Source: event_target_impl.cc:12-18
    pub fn InterfaceName(&self) -> &str {
        self.target_.InterfaceName()
    }
    pub fn GetExecutionContext(&self) -> Option<Rc<dyn ExecutionContext>> {
        self.target_.GetExecutionContext()
    }
    pub fn AsEventTargetRef(&self) -> EventTargetRef {
        self.target_.AsEventTargetRef()
    }
}
impl Deref for EventTargetImpl {
    type Target = EventTarget;
    fn deref(&self) -> &Self::Target {
        &self.target_
    }
}
impl crate::bindings::script_invocation::ScriptNativeObject for EventTargetImpl {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "EventTarget"
    }
    fn EventTarget(&self) -> Option<Rc<EventTarget>> {
        Some(self.target_.clone())
    }
}
// Source: event_target_impl.h:28,32; event_target_impl.cc:20-23
// Rc's base and context owners retain the same GC tracing edges; Drop supplies
// the default destructor. The wrapper is never a second dispatch identity.
