//! Page's existing event contract projects the canonical translated owner.
//!
//! Keep a single binding type for native managers and ordinary Page dispatch;
//! EventProjection carries its flags and subclass values into source dispatch.
pub use super::event::{
    Event, EventListenerDispatcher, EventListenerInterest, EventListenerInterestRef,
    EventListenerInvocation, EventListenerResult, EventPhase, EventType, EventTypeName, MakeEvent,
    MakeSyntheticEvent, NextEventIdentity,
};
