// Copyright 2018 The Chromium Authors. BSD-style license.
//! Chromium bindings/core/v8/js_based_event_listener.h:31-96;
//! js_based_event_listener.cc:50-201; js_event_listener.cc:46-60.
//! The live binding lends ScriptState facts and synchronous scopes. This
//! algorithm owns no runtime, callback table, execution context, or event copy.
use crate::core::dom::events::event::EventRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptState {
    pub identity: u64,
    pub world_id: i32,
}

pub trait JSListenerInvocation {
    fn IsExecutionTerminating(&self) -> bool;
    fn CanBeDispatchedInWorld(&self, event: &EventRef) -> bool;
    fn ListenerObjectIsObject(&self) -> bool;
    fn ScriptStateOrReportError(&self) -> Option<ScriptState>;
    fn ContextIsValid(&self, state: ScriptState) -> bool;
    fn WithScriptState(&self, state: ScriptState, invoke: &mut dyn FnMut());
    fn EventTargetScriptState(&self, world_id: i32) -> Option<ScriptState>;
    fn ShouldAllowAccess(&self, listener: ScriptState, target: ScriptState) -> bool;
    /// ToV8<Event> and Window::CurrentEvent are scoped after the access check.
    fn EnterCurrentEvent(&self, event: &EventRef) -> Box<dyn CurrentEventScope + '_>;
    /// JSEventListener::InvokeInternal performs the source runnability check,
    /// reports exceptions through the actual realm, and returns whether thrown.
    fn InvokeInternal(&self, event: &EventRef) -> bool;
}
/// A stack-scoped restore guard. It may borrow this invocation, but cannot be
/// retained as an Any value or outlive the synchronous listener call.
pub trait CurrentEventScope {}

pub fn Invoke(invocation: &dyn JSListenerInvocation, event: &EventRef) {
    if invocation.IsExecutionTerminating() || !invocation.CanBeDispatchedInWorld(event) {
        return;
    }
    if !invocation.ListenerObjectIsObject() {
        return;
    }
    let Some(listener) = invocation.ScriptStateOrReportError() else {
        return;
    };
    if !invocation.ContextIsValid(listener) {
        return;
    }
    invocation.WithScriptState(listener, &mut || {
        let Some(target) = invocation.EventTargetScriptState(listener.world_id) else {
            return;
        };
        debug_assert_eq!(listener.world_id, target.world_id);
        if !invocation.ShouldAllowAccess(listener, target) {
            return;
        }
        let _current_event = invocation.EnterCurrentEvent(event);
        if invocation.InvokeInternal(event) {
            event
                .borrow_mut()
                .BaseEventMut()
                .LegacySetDidListenersThrowFlag();
        }
    });
}
