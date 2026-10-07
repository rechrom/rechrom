//! Native ImageResourceObserver mixin receiver for translated owner classes.
#![allow(non_snake_case)]
use foundation::String as BlinkString;
use std::{cell::Cell, ffi::c_void};

// cpp: core/loader/resource/image_resource_observer.h:38-114
pub struct ImageResourceObserverVTable {
    pub DebugName: fn(*mut c_void) -> BlinkString,
}
#[repr(C)]
pub struct ImageResourceObserver {
    vtable: &'static ImageResourceObserverVTable,
    receiver: Cell<*mut c_void>,
}
impl ImageResourceObserver {
    /// Derived native owners embed this mixin and bind their stable address
    /// before registering it with a StyleImage. No observer token is invented.
    pub fn new_for_derived(vtable: &'static ImageResourceObserverVTable) -> Self {
        Self {
            vtable,
            receiver: Cell::new(std::ptr::null_mut()),
        }
    }
    pub fn BindReceiver(&self, receiver: *mut c_void) {
        assert!(!receiver.is_null());
        let old = self.receiver.get();
        assert!(
            old.is_null() || old == receiver,
            "native image observer owner moved"
        );
        self.receiver.set(receiver);
    }
    pub fn DebugName(&self) -> BlinkString {
        let receiver = self.receiver.get();
        assert!(!receiver.is_null());
        (self.vtable.DebugName)(receiver)
    }
}
