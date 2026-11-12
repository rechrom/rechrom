//! Live QuickJS binding ports for JSBasedEventListener's source lifecycle.
use super::*;
use interaction::dispatch::{
    bindings::js_based_event_listener::{
        self as source, CurrentEventScope, JSListenerInvocation, ScriptState,
    },
    core::dom::events::event::{DOMWrapperWorld, EventRef},
};
use javascript::javascript_runtime::JavaScriptCallbackContext;

struct World(i32);
impl DOMWrapperWorld for World {
    fn WorldId(&self) -> i32 {
        self.0
    }
}

struct RestoreCurrentEvent {
    slot: Rc<RefCell<Option<HostValue>>>,
    previous: Option<HostValue>,
}
impl CurrentEventScope for RestoreCurrentEvent {}
impl Drop for RestoreCurrentEvent {
    fn drop(&mut self) {
        *self.slot.borrow_mut() = self.previous.take();
    }
}

pub(super) struct Invocation<'a, 'runtime> {
    pub listener: &'a Listener,
    pub callback: &'a JavaScriptFunction,
    pub runtime: &'a RefCell<&'runtime mut dyn JavaScriptHostRuntime>,
    pub receiver: HostValue,
    pub argument: HostValue,
    pub handler_wrapper: Option<JavaScriptFunction>,
    pub current_event: Rc<RefCell<Option<HostValue>>>,
    pub report_error: RefCell<&'a mut dyn FnMut(&mut dyn JavaScriptHostRuntime, &HostValue)>,
}
impl Invocation<'_, '_> {
    fn ActiveContext(&self) -> JavaScriptCallbackContext {
        self.runtime.borrow().CallbackContext()
    }
    fn Report(&self, message: &str) {
        let mut runtime = self.runtime.borrow_mut();
        let error = runtime.NewError(JavaScriptExceptionKind::kRuntimeError, message);
        (self.report_error.borrow_mut())(&mut **runtime, &error);
    }
    pub(super) fn Invoke(&self, event: &EventRef) {
        source::Invoke(self, event);
    }
}
impl JSListenerInvocation for Invocation<'_, '_> {
    fn IsExecutionTerminating(&self) -> bool {
        self.runtime.borrow().IsExecutionTerminating()
    }
    fn CanBeDispatchedInWorld(&self, event: &EventRef) -> bool {
        self.listener.context.as_ref().is_some_and(|context| {
            event
                .borrow()
                .CanBeDispatchedInWorld(&World(context.WorldId()))
        })
    }
    fn ListenerObjectIsObject(&self) -> bool {
        self.runtime
            .borrow_mut()
            .IsObject(&HostValue::JavaScriptFunction(self.callback.clone()))
    }
    fn ScriptStateOrReportError(&self) -> Option<ScriptState> {
        let context = self.listener.context.as_ref()?;
        // This backend has one Context per VM. An unavailable/foreign callback
        // realm cannot be replaced by the event target's currently active one.
        if context.Identity() != self.ActiveContext().Identity() {
            self.Report("SecurityError: EventListener callback realm is not accessible");
            return None;
        }
        Some(ScriptState {
            identity: context.Identity(),
            world_id: context.WorldId(),
        })
    }
    fn ContextIsValid(&self, state: ScriptState) -> bool {
        self.listener
            .context
            .as_ref()
            .is_some_and(|context| context.ContextIsValid() && context.Identity() == state.identity)
    }
    fn WithScriptState(&self, state: ScriptState, invoke: &mut dyn FnMut()) {
        if self.ActiveContext().Identity() == state.identity {
            invoke();
        }
    }
    fn EventTargetScriptState(&self, world_id: i32) -> Option<ScriptState> {
        let context = self.ActiveContext();
        (context.ContextIsValid() && context.WorldId() == world_id).then(|| ScriptState {
            identity: context.Identity(),
            world_id,
        })
    }
    fn ShouldAllowAccess(&self, listener: ScriptState, target: ScriptState) -> bool {
        let allowed = listener == target;
        if !allowed {
            self.Report("SecurityError: EventListener target realm is not accessible");
        }
        allowed
    }
    fn EnterCurrentEvent(&self, event: &EventRef) -> Box<dyn CurrentEventScope + '_> {
        let previous = self.current_event.borrow().clone();
        // NodeEventContext retained this exact source invocation-target fact
        // before listener reentry. Shadow-tree listeners keep window.event.
        if !event.borrow().BaseEvent().invocationTargetInShadowTree() {
            *self.current_event.borrow_mut() = Some(self.argument.clone());
        }
        Box::new(RestoreCurrentEvent {
            slot: self.current_event.clone(),
            previous,
        })
    }
    fn InvokeInternal(&self, event: &EventRef) -> bool {
        if self
            .listener
            .context
            .as_ref()
            .is_some_and(|context| context.IsPaused())
            && !event
                .borrow()
                .ShouldDispatchEvenWhenExecutionContextIsPaused()
        {
            self.Report("EventListener cannot run while its execution context is paused");
            return true;
        }
        let result = if let Some(wrapper) = self.handler_wrapper.as_ref() {
            self.runtime.borrow_mut().CallValue(
                &HostValue::JavaScriptFunction(wrapper.clone()),
                &self.receiver,
                &[
                    HostValue::JavaScriptFunction(self.callback.clone()),
                    self.argument.clone(),
                ],
            )
        } else {
            self.runtime.borrow_mut().CallValue(
                &HostValue::JavaScriptFunction(self.callback.clone()),
                &self.receiver,
                &[self.argument.clone()],
            )
        };
        if let Some(error) = result.thrown {
            let mut runtime = self.runtime.borrow_mut();
            (self.report_error.borrow_mut())(&mut **runtime, &error);
            true
        } else if let Some(error) = result.exception {
            self.Report(&error.message);
            true
        } else {
            false
        }
    }
}
