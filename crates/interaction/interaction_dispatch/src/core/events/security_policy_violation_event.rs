//! Chromium security_policy_violation_event.{h:38-96,cc:34-91};
//! content_security_policy.cc:1234-1369,1474-1502.
use crate::core::dom::events::{
    event::{Event, EventClock, EventInterface},
    event_init::EventInit,
};
pub use foundation::content_security_policy::CSPViolationEventData as SecurityPolicyViolationEventData;
use std::ops::{Deref, DerefMut};
pub struct SecurityPolicyViolationEvent {
    event: Event,
    data: SecurityPolicyViolationEventData,
}
impl SecurityPolicyViolationEvent {
    pub fn new(data: SecurityPolicyViolationEventData, clock: &dyn EventClock) -> Self {
        Self::WithType("securitypolicyviolation", data, clock)
    }
    pub fn WithType(
        event_type: &str,
        data: SecurityPolicyViolationEventData,
        clock: &dyn EventClock,
    ) -> Self {
        Self {
            event: Event::CreateWithInitializer(
                event_type,
                &EventInit {
                    bubbles: true,
                    cancelable: false,
                    composed: true,
                },
                clock,
            ),
            data,
        }
    }
    pub fn data(&self) -> &SecurityPolicyViolationEventData {
        &self.data
    }
}
impl Deref for SecurityPolicyViolationEvent {
    type Target = Event;
    fn deref(&self) -> &Event {
        &self.event
    }
}
impl DerefMut for SecurityPolicyViolationEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.event
    }
}
impl EventInterface for SecurityPolicyViolationEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "SecurityPolicyViolationEvent"
    }
    fn AsSecurityPolicyViolationEvent(&self) -> Option<&Self> {
        Some(self)
    }
}
