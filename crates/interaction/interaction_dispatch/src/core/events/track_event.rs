//! Chromium core/html/track/track_event.cc:31-64, track_event.h:35-61.
//! The event retains the original Audio/Video/TextTrack owner, not copied data.
use crate::core::dom::events::event::{Event, EventClock, EventInterface, EventTargetRef};
use std::rc::Rc;
pub trait TrackEventTrack {
    fn Target(&self) -> EventTargetRef;
}
pub struct TrackEvent {
    event: Event,
    track: Rc<dyn TrackEventTrack>,
}
impl TrackEvent {
    pub fn Create(kind: &str, track: Rc<dyn TrackEventTrack>, clock: &dyn EventClock) -> Self {
        Self {
            event: Event::CreateWithType(kind, clock),
            track,
        }
    }
    pub fn track(&self) -> Rc<dyn TrackEventTrack> {
        self.track.clone()
    }
}
impl EventInterface for TrackEvent {
    fn BaseEvent(&self) -> &Event {
        &self.event
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        &mut self.event
    }
    fn InterfaceName(&self) -> &str {
        "TrackEvent"
    }
    fn AsTrackEvent(&self) -> Option<&Self> {
        Some(self)
    }
}
