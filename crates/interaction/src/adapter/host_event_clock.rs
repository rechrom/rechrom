//! Embedding clock shared by Page dispatch and queued native input. Source
//! TimeTicks remains a value; this adapter alone maps the host Instant domain.
use interaction_input::base::time::time::TimeTicks;
use std::{sync::OnceLock, time::Instant};

pub struct HostEventClock;
pub trait HostInputClock: Send + Sync {
    fn TimeForInstant(&self, instant: Instant) -> TimeTicks;
}
impl HostEventClock {
    pub fn TimeForInstant(instant: Instant) -> TimeTicks {
        static ORIGIN: OnceLock<Instant> = OnceLock::new();
        let origin = *ORIGIN.get_or_init(Instant::now);
        let micros = if instant >= origin {
            i64::try_from(instant.duration_since(origin).as_micros()).unwrap_or(i64::MAX - 1)
        } else {
            -i64::try_from(origin.duration_since(instant).as_micros()).unwrap_or(i64::MAX - 1)
        };
        TimeTicks::FromInternalValue(micros.saturating_add(1))
    }
}
impl interaction_dispatch::core::dom::events::event::EventClock for HostEventClock {
    fn Now(&self) -> TimeTicks {
        Self::TimeForInstant(Instant::now())
    }
}
impl HostInputClock for HostEventClock {
    fn TimeForInstant(&self, instant: Instant) -> TimeTicks {
        HostEventClock::TimeForInstant(instant)
    }
}
