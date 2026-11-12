// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: bindings/core/v8/promise_all.h:74-211. The empty list still resolves
// on the SAME Agent microtask queue, never synchronously or a Page task queue.
use super::script_promise::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
type Fulfill = Box<dyn FnOnce(&dyn ScriptInvocation, Vec<Value>)>;
type Reject = Box<dyn FnOnce(&dyn ScriptInvocation, Value)>;
struct PromiseAll {
    owners: Vec<Rc<dyn ScriptNativeObject>>,
    pending: Cell<usize>,
    settled: Cell<bool>,
    values: RefCell<Vec<Option<Value>>>,
    resolve: RefCell<Option<Fulfill>>,
    reject: RefCell<Option<Reject>>,
    undefined: bool,
}
impl PromiseAll {
    fn Fulfilled(&self, invocation: &dyn ScriptInvocation, index: usize, value: Value) {
        if self.settled.get() {
            return;
        }
        if !self.undefined {
            self.values.borrow_mut()[index] = Some(value);
        }
        let remaining = self
            .pending
            .get()
            .checked_sub(1)
            .expect("pending PromiseAll reaction");
        self.pending.set(remaining);
        if remaining == 0 {
            self.Resolve(invocation);
        }
    }
    fn Resolve(&self, invocation: &dyn ScriptInvocation) {
        debug_assert_eq!(self.pending.get(), 0);
        self.settled.set(true);
        let callback = self
            .resolve
            .borrow_mut()
            .take()
            .expect("PromiseAll resolve callback");
        let values = if self.undefined {
            Vec::new()
        } else {
            std::mem::take(&mut *self.values.borrow_mut())
                .into_iter()
                .map(|v| v.expect("fulfilled promise value"))
                .collect()
        };
        callback(invocation, values);
    }
    fn Rejected(&self, invocation: &dyn ScriptInvocation, value: Value) {
        if self.settled.replace(true) {
            return;
        }
        let callback = self
            .reject
            .borrow_mut()
            .take()
            .expect("PromiseAll reject callback");
        callback(invocation, value);
    }
}
struct Fulfilled {
    all: Rc<PromiseAll>,
    index: usize,
}
impl PromiseReaction for Fulfilled {
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>> {
        vec![self.all.clone()]
    }
    fn React(&self, i: &dyn ScriptInvocation, v: Value) {
        self.all.Fulfilled(i, self.index, v);
        i.TraceNativeObject(self.all.clone());
    }
}
struct Rejected(Rc<PromiseAll>);
impl PromiseReaction for Rejected {
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>> {
        vec![self.0.clone()]
    }
    fn React(&self, i: &dyn ScriptInvocation, v: Value) {
        self.0.Rejected(i, v);
        i.TraceNativeObject(self.0.clone());
    }
}
struct Empty(Rc<PromiseAll>);
impl NativeMicrotask for Empty {
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>> {
        vec![self.0.clone()]
    }
    fn Run(&self, i: &dyn ScriptInvocation) {
        self.0.Resolve(i);
    }
}
pub fn WaitForAll(
    invocation: &dyn ScriptInvocation,
    owners: Vec<Rc<dyn ScriptNativeObject>>,
    promises: &[ScriptPromise],
    undefined: bool,
    resolve: Fulfill,
    reject: Reject,
) -> Result<(), Value> {
    let all = Rc::new(PromiseAll {
        owners,
        pending: Cell::new(promises.len()),
        settled: Cell::new(false),
        values: RefCell::new(vec![None; promises.len()]),
        resolve: RefCell::new(Some(resolve)),
        reject: RefCell::new(Some(reject)),
        undefined,
    });
    if promises.is_empty() {
        invocation.EnqueueMicrotask(Rc::new(Empty(all)))?;
        return Ok(());
    }
    for (index, promise) in promises.iter().enumerate() {
        invocation.PerformPromiseThen(
            promise,
            Rc::new(Fulfilled {
                all: all.clone(),
                index,
            }),
            Rc::new(Rejected(all.clone())),
        )?;
    }
    Ok(())
}

impl ScriptNativeObject for PromiseAll {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "InternalPromiseAll"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        for value in self.values.borrow().iter().flatten() {
            visit(value.clone());
        }
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        for owner in &self.owners {
            visit(owner.clone());
        }
    }
}
