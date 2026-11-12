// Copyright 2010 Google Inc. All rights reserved.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/scoped_event_queue.h:41-77
// Source: third_party/blink/renderer/core/dom/events/scoped_event_queue.cc:44-97
// The document execution sequence supplies dispatch, retaining original events.
use super::{event::EventRef, event_dispatcher::ScopedEventQueue};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub struct ScopedEventQueueOwner {
    queued_events: RefCell<Vec<EventRef>>,
    scoping_level: Cell<u32>,
}
impl ScopedEventQueueOwner {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            queued_events: RefCell::new(Vec::new()),
            scoping_level: Cell::new(0),
        })
    }
    // Source: scoped_event_queue.cc:60-65. Callback binding carries only the
    // current execution scope; this owner makes the source queue decision.
    pub fn EnqueueEvent(&self, event: EventRef, dispatch: &dyn Fn(EventRef)) {
        if self.ShouldQueueEvents() {
            self.queued_events.borrow_mut().push(event);
        } else {
            self.DispatchEvent(event, dispatch);
        }
    }
    pub fn ShouldQueueEvents(&self) -> bool {
        self.scoping_level.get() > 0
    }
    pub fn IncrementScopingLevel(&self) {
        self.scoping_level.set(
            self.scoping_level
                .get()
                .checked_add(1)
                .expect("event queue scope overflow"),
        );
    }
    pub fn DecrementScopingLevel(&self, dispatch: &dyn Fn(EventRef)) {
        let level = self.scoping_level.get();
        assert_ne!(level, 0, "balanced event queue scopes");
        self.scoping_level.set(level - 1);
        if level == 1 {
            // Swap before callbacks: reentry gets the live, empty queue and
            // unscoped events dispatch synchronously between these entries.
            let events = std::mem::take(&mut *self.queued_events.borrow_mut());
            for event in events {
                self.DispatchEvent(event, dispatch);
            }
        }
    }
    fn DispatchEvent(&self, event: EventRef, dispatch: &dyn Fn(EventRef)) {
        assert!(
            event.borrow().BaseEvent().RawTarget().is_some(),
            "scoped event has its retained target"
        );
        dispatch(event);
    }
}

// The queue owns source scoping and pending Event identities. The embedding's
// current synchronous dispatch scope supplies the callback separately, so a
// queue never retains a VM borrow or creates an owner/callback reference cycle.
pub struct ScopedEventQueueBinding<'a> {
    owner: Rc<ScopedEventQueueOwner>,
    dispatch: Rc<dyn Fn(EventRef) + 'a>,
}
impl<'a> ScopedEventQueueBinding<'a> {
    pub fn new(owner: Rc<ScopedEventQueueOwner>, dispatch: Rc<dyn Fn(EventRef) + 'a>) -> Rc<Self> {
        Rc::new(Self { owner, dispatch })
    }
    pub fn Scope(self: &Rc<Self>) -> EventQueueScope<'a> {
        self.owner.IncrementScopingLevel();
        EventQueueScope(self.clone())
    }
}
impl ScopedEventQueue for ScopedEventQueueBinding<'_> {
    fn EnqueueEvent(&self, event: EventRef) {
        self.owner.EnqueueEvent(event, self.dispatch.as_ref());
    }
}
pub struct EventQueueScope<'a>(Rc<ScopedEventQueueBinding<'a>>);
impl Drop for EventQueueScope<'_> {
    fn drop(&mut self) {
        self.0.owner.DecrementScopingLevel(self.0.dispatch.as_ref());
    }
}

impl Drop for ScopedEventQueueOwner {
    fn drop(&mut self) {
        debug_assert_eq!(self.scoping_level.get(), 0);
        debug_assert!(self.queued_events.get_mut().is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dom::events::event::{Event, EventTarget, EventTargetKind};
    use interaction_input::base::time::time::TimeTicks;
    struct Node;
    impl EventTarget for Node {
        fn Kind(&self) -> EventTargetKind {
            EventTargetKind::Node
        }
    }
    fn event(kind: &str) -> EventRef {
        let mut event = Event::Uninitialized(TimeTicks::default());
        event.SetType(kind);
        event.SetTarget(Some(Rc::new(Node)));
        EventRef::new(event)
    }
    #[test]
    fn outer_scope_retains_identity_and_flushes_without_a_live_queue_borrow() {
        let dispatched = Rc::new(RefCell::new(Vec::<EventRef>::new()));
        let queue_slot = Rc::new(RefCell::new(None::<Rc<ScopedEventQueueBinding>>));
        let events = dispatched.clone();
        let slot = queue_slot.clone();
        let queue = ScopedEventQueueBinding::new(
            ScopedEventQueueOwner::new(),
            Rc::new(move |event| {
                let reenter = event.borrow().BaseEvent().r#type() == "first";
                events.borrow_mut().push(event);
                if reenter {
                    slot.borrow()
                        .as_ref()
                        .unwrap()
                        .EnqueueEvent(super::tests::event("reentrant"));
                }
            }),
        );
        *queue_slot.borrow_mut() = Some(queue.clone());
        let first = event("first");
        let last = event("last");
        {
            let _outer = queue.Scope();
            queue.EnqueueEvent(first.clone());
            {
                let _inner = queue.Scope();
                queue.EnqueueEvent(last.clone());
            }
            assert!(dispatched.borrow().is_empty());
        }
        let dispatched = dispatched.borrow();
        assert!(dispatched[0].IsSameEvent(&first));
        assert_eq!(dispatched[1].borrow().BaseEvent().r#type(), "reentrant");
        assert!(dispatched[2].IsSameEvent(&last));
        queue_slot.borrow_mut().take();
    }
}
