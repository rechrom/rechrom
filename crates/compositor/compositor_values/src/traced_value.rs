//! Typed trace values (base/trace_event/traced_value.h) produced by settings
//! `AsValue`. Moved from compositor_scheduler, which re-exports it.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum TraceValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Dictionary(BTreeMap<String, TraceValue>),
}

// Moved from compositor_scheduler::services with the value (coherence).
impl From<scroll_animation::TimeDelta> for TraceValue {
    fn from(value: scroll_animation::TimeDelta) -> Self {
        Self::Integer(value.InMicroseconds())
    }
}
