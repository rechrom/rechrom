//! Native ErrorEvent constructor, error_event.cc:89-101,108-121;
//! error_event.h:83-109. Script-origin errors retain their realm/value through
//! separate bindings; this constructor is the source's world=null,error=null
//! browser-generated error (e.g. ResizeObserver loop limit).
use crate::{
    bindings::script_invocation::ScriptSourceLocation,
    core::dom::events::event::{Event, EventClock, EventInit, EventInterface},
};
use foundation::String;
use std::ops::{Deref, DerefMut};
pub struct ErrorEvent {
    event: Event,
    message: String,
    location: ScriptSourceLocation,
    unsanitized_message: Option<String>,
    error: Option<crate::bindings::script_invocation::Value>,
    world: Option<i32>,
}
impl ErrorEvent {
    pub fn Create(message: String, location: ScriptSourceLocation, clock: &dyn EventClock) -> Self {
        Self {
            event: Event::CreateWithInitializer(
                "error",
                &EventInit {
                    bubbles: false,
                    cancelable: true,
                    composed: false,
                },
                clock,
            ),
            message,
            location,
            unsanitized_message: None,
            error: None,
            world: None,
        }
    }
    /// ErrorEvent.cc:102-106,108-121: retain the original thrown value/world.
    pub fn CreateForScript(
        message: String,
        location: ScriptSourceLocation,
        error: crate::bindings::script_invocation::Value,
        world: i32,
        clock: &dyn EventClock,
    ) -> Self {
        let mut event = Self::Create(message, location, clock);
        event.error = Some(error);
        event.world = Some(world);
        event
    }
    pub fn Error(
        &self,
        invocation: &dyn crate::bindings::script_invocation::ScriptInvocation,
    ) -> crate::bindings::script_invocation::Value {
        if self.world == Some(invocation.WorldId()) {
            self.error.clone().unwrap_or_else(|| invocation.Null())
        } else {
            invocation.Null()
        }
    }
    pub fn message(&self) -> &String {
        &self.message
    }
    pub fn filename(&self) -> &str {
        &self.location.url
    }
    pub fn lineno(&self) -> u32 {
        self.location.line
    }
    pub fn colno(&self) -> u32 {
        self.location.column
    }
    pub fn Location(&self) -> &ScriptSourceLocation {
        &self.location
    }
    pub fn SetUnsanitizedMessage(&mut self, message: String) {
        debug_assert!(self
            .unsanitized_message
            .as_ref()
            .is_none_or(|message| message.empty()));
        self.unsanitized_message = Some(message);
    }
    pub fn MessageForConsole(&self) -> &String {
        self.unsanitized_message
            .as_ref()
            .filter(|s| !s.empty())
            .unwrap_or(&self.message)
    }
}
impl Deref for ErrorEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for ErrorEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
impl EventInterface for ErrorEvent {
    // error_event.cc:112-114: script errors remain in their originating world.
    fn CanBeDispatchedInWorld(
        &self,
        world: &dyn crate::core::dom::events::event::DOMWrapperWorld,
    ) -> bool {
        self.world.is_none_or(|id| id == world.WorldId())
    }
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "ErrorEvent"
    }
    fn IsErrorEvent(&self) -> bool {
        true
    }
    fn AsErrorEvent(&self) -> Option<&ErrorEvent> {
        Some(self)
    }
}

/// ExecutionContext::DispatchErrorEvent, cc:283-319. The original Window
/// execution context retains this guard and pending exception list. Native
/// browser errors use kDoNotSanitize and a null world/script exception value.
#[derive(Default)]
pub struct ErrorEventDispatchState {
    in_dispatch: std::cell::Cell<bool>,
    pending: std::cell::RefCell<Vec<crate::core::dom::events::event::EventRef>>,
}
pub trait ErrorEventDispatchOperations {
    fn ErrorEventTarget(&self) -> Option<crate::core::dom::events::event::EventTargetRef>;
    fn DispatchEvent(
        &self,
        target: crate::core::dom::events::event::EventTargetRef,
        event: crate::core::dom::events::event::EventRef,
    );
    fn ExceptionThrown(&self, event: &crate::core::dom::events::event::EventRef);
}
impl ErrorEventDispatchState {
    pub fn Dispatch(
        &self,
        event: crate::core::dom::events::event::EventRef,
        operations: &dyn ErrorEventDispatchOperations,
    ) {
        if self.in_dispatch.get() {
            self.pending.borrow_mut().push(event);
            return;
        }
        let handled = if let Some(target) = operations.ErrorEventTarget() {
            self.in_dispatch.set(true);
            struct Reset<'a>(&'a std::cell::Cell<bool>);
            impl Drop for Reset<'_> {
                fn drop(&mut self) {
                    self.0.set(false);
                }
            }
            {
                let _reset = Reset(&self.in_dispatch);
                operations.DispatchEvent(target, event.clone());
            }
            event.borrow().BaseEvent().defaultPrevented()
        } else {
            false
        };
        if !handled {
            operations.ExceptionThrown(&event);
        }
        let pending = std::mem::take(&mut *self.pending.borrow_mut());
        for nested in pending {
            operations.ExceptionThrown(&nested);
        }
    }
}

impl crate::bindings::script_invocation::ScriptNativeObject for std::cell::RefCell<ErrorEvent> {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "ErrorEvent"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(crate::bindings::script_invocation::Value)) {
        if let Some(error) = self.borrow().error.clone() {
            visit(error);
        }
    }
}
