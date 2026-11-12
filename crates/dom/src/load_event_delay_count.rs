//! Document's canonical load-event delay count and source RAII lease.
//! Chromium increment_load_event_delay_count.h:20-54; document.cc:8856-8882.
use std::{cell::Cell, rc::Rc};
#[derive(Default)]
pub struct LoadEventDelayCounter {
    count: Cell<u32>,
}
impl LoadEventDelayCounter {
    pub fn Count(&self) -> u32 {
        self.count.get()
    }
    pub fn Increment(&self) {
        self.count.set(
            self.Count()
                .checked_add(1)
                .expect("Document load delay count"),
        );
    }
    pub fn Decrement(&self) -> bool {
        assert_ne!(self.Count(), 0, "balanced Document load delay");
        self.count.set(self.Count() - 1);
        self.Count() == 0
    }
    pub fn Acquire(self: &Rc<Self>) -> LoadEventDelayLease {
        self.Increment();
        LoadEventDelayLease {
            counter: self.clone(),
        }
    }
}
pub struct LoadEventDelayLease {
    counter: Rc<LoadEventDelayCounter>,
}
impl Drop for LoadEventDelayLease {
    fn drop(&mut self) {
        self.counter.Decrement();
    }
}
