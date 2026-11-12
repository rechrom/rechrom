//! Canonical native execution-context lifetime and weak synchronous observers.
//! Source: execution_context.cc177-179, ExecutionContextLifecycleObserver.
//! The read-only atomic handle also serves native drag/text-check requests.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
/// Observers may clear their own source state; no Page or context is passed in
/// and notification cannot reactivate this lifetime.
pub trait NativeExecutionContextObserver {
    fn ContextDestroyed(&self);
}
pub struct NativeExecutionContextObservers {
    alive: Arc<AtomicBool>,
    observers: RefCell<Vec<Weak<dyn NativeExecutionContextObserver>>>,
}
impl NativeExecutionContextObservers {
    pub fn Register(&self, observer: &Rc<dyn NativeExecutionContextObserver>) {
        if !self.alive.load(Ordering::Acquire) {
            observer.ContextDestroyed();
            return;
        }
        let mut observers = self.observers.borrow_mut();
        observers.retain(|entry| entry.strong_count() != 0);
        if !observers.iter().any(|entry| {
            entry
                .upgrade()
                .is_some_and(|old| Rc::ptr_eq(&old, observer))
        }) {
            observers.push(Rc::downgrade(observer));
        }
    }
    pub fn Unregister(&self, observer: &Rc<dyn NativeExecutionContextObserver>) {
        self.observers.borrow_mut().retain(|entry| {
            entry
                .upgrade()
                .is_some_and(|old| !Rc::ptr_eq(&old, observer))
        });
    }
    fn NotifyContextDestroyed(&self) {
        let observers = std::mem::take(&mut *self.observers.borrow_mut());
        for observer in observers {
            if let Some(observer) = observer.upgrade() {
                observer.ContextDestroyed();
            }
        }
    }
}
/// ExecutionContext.cc:94-115,153-179,323-341. Hidden visibility is
/// independent of paused/frozen execution and must not be inferred from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionContextLifecycle {
    Running,
    Paused,
    Frozen,
    Destroyed,
}
/// Value delivered by LocalFrame::OnPageLifecycleStateUpdated (cc3213-3223).
/// Context destruction is a separate Document shutdown notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameLifecycleState {
    Running,
    Paused,
    Frozen,
}
/// State observers are distinct from destruction-only observers, as in
/// ExecutionContext.cc:157-173, HeapObserverList.h:100-118. Notification is
/// synchronous; registration mutation is prohibited while iterating.
pub trait ExecutionContextLifecycleStateObserver {
    fn ContextLifecycleStateChanged(&self, state: ExecutionContextLifecycle);
}
#[derive(Default)]
pub struct ExecutionContextLifecycleStateObservers {
    iterating: Cell<bool>,
    observers: RefCell<Vec<Weak<dyn ExecutionContextLifecycleStateObserver>>>,
}
impl ExecutionContextLifecycleStateObservers {
    pub fn Register(&self, observer: &Rc<dyn ExecutionContextLifecycleStateObserver>) {
        assert!(
            !self.iterating.get(),
            "source HeapObserverList forbids mutation during notification"
        );
        let mut entries = self.observers.borrow_mut();
        entries.retain(|e| e.strong_count() != 0);
        if !entries
            .iter()
            .any(|e| e.upgrade().is_some_and(|old| Rc::ptr_eq(&old, observer)))
        {
            entries.push(Rc::downgrade(observer));
        }
    }
    pub fn Unregister(&self, observer: &Rc<dyn ExecutionContextLifecycleStateObserver>) {
        assert!(
            !self.iterating.get(),
            "source HeapObserverList forbids mutation during notification"
        );
        self.observers
            .borrow_mut()
            .retain(|e| e.upgrade().is_some_and(|old| !Rc::ptr_eq(&old, observer)));
    }
    fn Notify(&self, state: ExecutionContextLifecycle) {
        struct Reset<'a>(&'a Cell<bool>, bool);
        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.0.set(self.1);
            }
        }
        let previous = self.iterating.replace(true);
        let _reset = Reset(&self.iterating, previous);
        let entries = self.observers.borrow();
        for observer in entries.iter() {
            if let Some(observer) = observer.upgrade() {
                observer.ContextLifecycleStateChanged(state);
            }
        }
    }
}
#[must_use = "deliver synchronous lifecycle callbacks after releasing the DOM borrow"]
pub struct ExecutionContextLifecycleChange {
    observers: Rc<ExecutionContextLifecycleStateObservers>,
    state: ExecutionContextLifecycle,
}
impl ExecutionContextLifecycleChange {
    pub fn Notify(self) {
        self.observers.Notify(self.state);
    }
}
pub(crate) struct NativeExecutionContextLifetime {
    lifecycle: Cell<ExecutionContextLifecycle>,
    lifecycle_observers: Rc<ExecutionContextLifecycleStateObservers>,
    alive: Arc<AtomicBool>,
    observers: Rc<NativeExecutionContextObservers>,
}
impl NativeExecutionContextLifetime {
    pub fn new() -> Self {
        let alive = Arc::new(AtomicBool::new(true));
        Self {
            lifecycle: Cell::new(ExecutionContextLifecycle::Running),
            lifecycle_observers: Rc::new(ExecutionContextLifecycleStateObservers::default()),
            observers: Rc::new(NativeExecutionContextObservers {
                alive: alive.clone(),
                observers: Default::default(),
            }),
            alive,
        }
    }
    pub fn Lifecycle(&self) -> ExecutionContextLifecycle {
        self.lifecycle.get()
    }
    pub fn SetLifecycle(
        &self,
        state: ExecutionContextLifecycle,
    ) -> Option<ExecutionContextLifecycleChange> {
        assert!(
            self.lifecycle.get() != ExecutionContextLifecycle::Destroyed,
            "destroyed context cannot resume"
        );
        if state == ExecutionContextLifecycle::Destroyed {
            self.ContextDestroyed();
        } else if self.lifecycle.replace(state) != state {
            return Some(ExecutionContextLifecycleChange {
                observers: self.lifecycle_observers.clone(),
                state,
            });
        }
        None
    }
    pub fn LifecycleObservers(&self) -> Rc<ExecutionContextLifecycleStateObservers> {
        self.lifecycle_observers.clone()
    }
    pub fn ReadOnlyHandle(&self) -> Arc<AtomicBool> {
        self.alive.clone()
    }
    pub fn Observers(&self) -> Rc<NativeExecutionContextObservers> {
        self.observers.clone()
    }
    pub fn ContextDestroyed(&self) {
        self.lifecycle.set(ExecutionContextLifecycle::Destroyed);
        if self.alive.swap(false, Ordering::AcqRel) {
            self.observers.NotifyContextDestroyed();
        }
    }
}
impl Drop for NativeExecutionContextLifetime {
    fn drop(&mut self) {
        self.ContextDestroyed();
    }
}
