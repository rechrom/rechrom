//! Chromium interaction_effects_monitor.h:28-80; .cc:17-72.
use super::{
    soft_navigation_context::SoftNavigationContext,
    soft_navigation_heuristics::SoftNavigationHeuristics,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
pub trait InteractionEffectsObserver {
    fn OnContentfulPaint(&self, new_painted_area: u64);
}
pub struct InteractionEffectsMonitor {
    self_: Weak<Self>,
    heuristics: RefCell<Option<Weak<SoftNavigationHeuristics>>>,
    observer: RefCell<Option<Weak<dyn InteractionEffectsObserver>>>,
    total_painted_area: Cell<u64>,
    min_context_id: Cell<u64>,
    interaction_count: Cell<u32>,
}
impl InteractionEffectsMonitor {
    pub fn new(heuristics: Option<&Rc<SoftNavigationHeuristics>>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            heuristics: RefCell::new(heuristics.map(Rc::downgrade)),
            observer: RefCell::new(None),
            total_painted_area: Cell::new(0),
            min_context_id: Cell::new(0),
            interaction_count: Cell::new(0),
        })
    }
    pub fn StartMonitoring(&self, observer: &Rc<dyn InteractionEffectsObserver>) {
        let Some(owner) = self.heuristics.borrow().as_ref().and_then(Weak::upgrade) else {
            return;
        };
        assert!(self.observer.borrow().is_none());
        self.total_painted_area.set(0);
        *self.observer.borrow_mut() = Some(Rc::downgrade(observer));
        self.min_context_id
            .set(SoftNavigationContext::NextContextId());
        owner.RegisterInteractionEffectsMonitor(self.self_.upgrade().expect("live monitor"));
    }
    pub fn StopMonitoring(&self) {
        let Some(owner) = self.heuristics.borrow().as_ref().and_then(Weak::upgrade) else {
            return;
        };
        self.observer.borrow_mut().take();
        owner.UnregisterInteractionEffectsMonitor(self);
    }
    pub fn Shutdown(&self) {
        self.heuristics.borrow_mut().take();
        self.observer.borrow_mut().take();
    }
    pub fn TotalPaintedArea(&self) -> u64 {
        self.total_painted_area.get()
    }
    pub fn InteractionCount(&self) -> u32 {
        self.interaction_count.get()
    }
    pub fn OnSoftNavigationContextCreated(&self) {
        self.interaction_count
            .set(self.interaction_count.get().wrapping_add(1));
    }
    pub fn OnContentfulPaint(&self, context: &SoftNavigationContext, new_painted_area: u64) {
        if context.ContextId() < self.min_context_id.get() {
            return;
        }
        let observer = self
            .observer
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .expect("registered observer lives until StopMonitoring");
        self.total_painted_area
            .set(self.total_painted_area.get().wrapping_add(new_painted_area));
        observer.OnContentfulPaint(new_painted_area);
    }
}
impl Drop for InteractionEffectsMonitor {
    fn drop(&mut self) {
        assert!(self.observer.borrow().is_none());
    }
}
