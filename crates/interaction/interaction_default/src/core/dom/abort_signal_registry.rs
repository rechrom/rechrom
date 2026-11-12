// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/abort_signal_registry.h:25-65; .cc:20-99.
use super::abort_signal::{AbortSignal, AlgorithmHandle};
use crate::bindings::core::v8::script_promise::ScriptNativeObject;
use interaction_dispatch::core::dom::events::event_listener::EventListener;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};
/// Publishes Trace metadata only; it cannot enter a realm or run script.
pub trait AbortSignalRegistryObserver {
    fn RegistryChanged(&self, registry: Rc<AbortSignalRegistry>);
}
pub struct AbortSignalRegistry {
    self_: Weak<Self>,
    observer: Weak<dyn AbortSignalRegistryObserver>,
    destroyed: Cell<bool>,
    event_listener_signals:
        RefCell<HashMap<usize, (Weak<dyn EventListener>, Option<Rc<AlgorithmHandle>>)>>,
    signals_registered_for_abort: RefCell<HashMap<usize, Rc<AbortSignal>>>,
}
impl AbortSignalRegistry {
    pub fn new(observer: Weak<dyn AbortSignalRegistryObserver>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            observer,
            destroyed: Cell::new(false),
            event_listener_signals: RefCell::new(HashMap::new()),
            signals_registered_for_abort: RefCell::new(HashMap::new()),
        })
    }
    fn TraceChanged(&self) {
        if let Some(observer) = self.observer.upgrade() {
            observer.RegistryChanged(
                self.self_
                    .upgrade()
                    .expect("live execution-context registry"),
            );
        }
    }
    pub fn RegisterAbortAlgorithm(
        &self,
        listener: &Rc<dyn EventListener>,
        handle: Option<Rc<AlgorithmHandle>>,
    ) {
        if self.destroyed.get() {
            return;
        }
        self.event_listener_signals.borrow_mut().insert(
            Rc::as_ptr(listener) as *const () as usize,
            (Rc::downgrade(listener), handle),
        );
        self.TraceChanged();
    }
    pub fn RegisterSignal(&self, signal: Rc<AbortSignal>) {
        if !self.destroyed.get() {
            self.signals_registered_for_abort
                .borrow_mut()
                .insert(signal.Identity(), signal);
            self.TraceChanged();
        }
    }
    pub fn UnregisterSignal(&self, signal: &AbortSignal) {
        if !self.destroyed.get() {
            self.signals_registered_for_abort
                .borrow_mut()
                .remove(&signal.Identity());
            self.TraceChanged();
        }
    }
    /// Weak collection processing belongs to the Window's existing GC checkpoint.
    pub fn SweepExpiredListeners(&self) {
        self.event_listener_signals
            .borrow_mut()
            .retain(|_, (listener, _)| listener.strong_count() != 0);
        self.TraceChanged();
    }
    pub fn ContextDestroyed(&self) {
        self.destroyed.set(true);
        self.event_listener_signals.borrow_mut().clear();
        self.TraceChanged();
    }
}
impl ScriptNativeObject for AbortSignalRegistry {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "AbortSignalRegistry"
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        for signal in self.signals_registered_for_abort.borrow().values() {
            visit(signal.clone());
        }
        // Native listeners have no VM wrapper and are weak through their
        // source Rc lifetime. JSBasedEventListener uses the ephemeron below.
        for (listener, handle) in self.event_listener_signals.borrow().values() {
            if listener
                .upgrade()
                .is_some_and(|listener| listener.ScriptBindingIdentity().is_none())
            {
                if let Some(handle) = handle {
                    visit(handle.clone());
                }
            }
        }
    }
    fn VisitEphemerons(&self, visit: &mut dyn FnMut(u64, Rc<dyn ScriptNativeObject>)) {
        for (listener, handle) in self.event_listener_signals.borrow().values() {
            if let (Some(key), Some(handle)) = (
                listener
                    .upgrade()
                    .and_then(|listener| listener.ScriptBindingIdentity()),
                handle,
            ) {
                visit(key, handle.clone());
            }
        }
    }
    fn WeakKeysReleased(&self, keys: &[u64]) {
        self.event_listener_signals
            .borrow_mut()
            .retain(|_, (listener, _)| {
                listener.upgrade().is_some_and(|listener| {
                    listener
                        .ScriptBindingIdentity()
                        .is_none_or(|key| !keys.contains(&key))
                })
            });
    }
}
