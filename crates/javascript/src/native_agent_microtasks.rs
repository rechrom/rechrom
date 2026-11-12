//! Scriptless document Agent microtasks. This is the native-only branch of
//! the Agent queue, drained by the Page's existing task-end checkpoint. It is
//! not an event/task source and creates no JavaScript runtime or realm.
use crate::javascript_runtime::{
    JavaScriptAgentMicrotasks, JavaScriptNativeMicrotask, NativeMicrotaskEnqueueError,
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};
#[derive(Default)]
pub struct NativeAgentMicrotasks {
    tasks: RefCell<VecDeque<JavaScriptNativeMicrotask>>,
    running: Cell<bool>,
}
impl NativeAgentMicrotasks {
    pub fn new() -> Rc<Self> {
        Rc::new(Self::default())
    }
    pub fn PerformCheckpoint(&self) {
        if self.running.replace(true) {
            return;
        }
        struct Reset<'a>(&'a Cell<bool>);
        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _reset = Reset(&self.running);
        loop {
            let task = self.tasks.borrow_mut().pop_front();
            let Some(task) = task else {
                break;
            };
            task(None);
        }
    }
}
impl JavaScriptAgentMicrotasks for NativeAgentMicrotasks {
    fn Enqueue(&self, task: JavaScriptNativeMicrotask) -> Result<(), NativeMicrotaskEnqueueError> {
        self.tasks.borrow_mut().push_back(task);
        Ok(())
    }
}
