use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// Source: include/private/SkIDChangeListener.h:25-31
// Rust inheritance substitution: subclasses carry this native base state and implement changed().
pub trait SkIDChangeListener: Send + Sync {
    fn state(&self) -> &SkIDChangeListenerState;
    fn changed(&self);

    // Source: include/private/SkIDChangeListener.h:37-37
    #[inline]
    fn markShouldDeregister(&self) {
        self.state()
            .fShouldDeregister
            .store(true, Ordering::Relaxed);
    }

    // Source: include/private/SkIDChangeListener.h:40-40
    #[inline]
    fn shouldDeregister(&self) -> bool {
        self.state().fShouldDeregister.load(Ordering::Acquire)
    }
}
// Source: include/private/SkIDChangeListener.h:68-69
pub struct List {
    fListeners: Mutex<Vec<Arc<dyn SkIDChangeListener>>>,
}

// Source: include/private/SkIDChangeListener.h:73-73
pub struct SkIDChangeListenerState {
    fShouldDeregister: AtomicBool,
}

// Source: src/core/SkIDChangeListener.cpp:21-21
impl Default for SkIDChangeListenerState {
    fn default() -> Self {
        Self {
            fShouldDeregister: AtomicBool::new(false),
        }
    }
}

// Source: src/core/SkIDChangeListener.cpp:27-27
impl Default for List {
    fn default() -> Self {
        Self {
            fListeners: Mutex::new(Vec::new()),
        }
    }
}

// Source: src/core/SkIDChangeListener.cpp:29-37
impl Drop for List {
    fn drop(&mut self) {
        let listeners = self.fListeners.get_mut().unwrap_or_else(|e| e.into_inner());
        for listener in listeners {
            if !listener.shouldDeregister() {
                listener.changed();
            }
        }
    }
}

// Source: src/core/SkIDChangeListener.cpp:39-53
impl List {
    pub fn add(&self, listener: Option<Arc<dyn SkIDChangeListener>>) {
        let Some(listener) = listener else {
            return;
        };
        crate::SkASSERT!(@source b"src/core/SkIDChangeListener.cpp\0".as_ptr(),43; @condition b"!listener->shouldDeregister()\0".as_ptr(); !listener.shouldDeregister());
        let mut listeners = self.fListeners.lock().unwrap_or_else(|e| e.into_inner());
        let mut i = 0;
        while i < listeners.len() {
            if listeners[i].shouldDeregister() {
                listeners.swap_remove(i);
            } else {
                i += 1;
            }
        }
        listeners.push(listener);
    }
}

// Source: src/core/SkIDChangeListener.cpp:55-58
impl List {
    pub fn count(&self) -> i32 {
        let listeners = self.fListeners.lock().unwrap_or_else(|e| e.into_inner());
        listeners.len() as i32
    }
}

// Source: src/core/SkIDChangeListener.cpp:60-68
impl List {
    pub fn changed(&self) {
        let mut listeners = self.fListeners.lock().unwrap_or_else(|e| e.into_inner());
        for listener in listeners.iter() {
            if !listener.shouldDeregister() {
                listener.changed();
            }
        }
        listeners.clear();
    }
}

// Source: src/core/SkIDChangeListener.cpp:70-73
impl List {
    pub fn reset(&self) {
        self.fListeners
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}
