// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: bindings/core/v8/script_promise_resolver.h:79-104,178-209,
// 242-257; .cc:99-141. Active invocation path; no stored runtime borrow.
use super::script_promise::*;
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolverState {
    Pending,
    Resolved,
    Rejected,
}
pub struct ScriptPromiseResolver {
    capability: ScriptPromiseCapability,
    state: Cell<ResolverState>,
    detached: Cell<bool>,
}
impl ScriptPromiseResolver {
    pub fn new(invocation: &dyn ScriptInvocation) -> Result<Rc<Self>, Value> {
        Ok(Rc::new(Self {
            capability: invocation.NewPromiseCapability()?,
            state: Cell::new(ResolverState::Pending),
            detached: Cell::new(false),
        }))
    }
    pub fn Promise(&self) -> ScriptPromise {
        self.capability.promise.clone()
    }
    pub fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        visit(self.capability.promise.0.clone());
        visit(self.capability.resolve.clone());
        visit(self.capability.reject.clone());
    }
    pub fn Resolve(&self, invocation: &dyn ScriptInvocation, value: Value) {
        self.Settle(invocation, value, false);
    }
    pub fn Reject(&self, invocation: &dyn ScriptInvocation, value: Value) {
        self.Settle(invocation, value, true);
    }
    fn Settle(&self, invocation: &dyn ScriptInvocation, value: Value, rejected: bool) {
        if self.detached.get()
            || self.state.get() != ResolverState::Pending
            || !invocation.ContextIsValid()
        {
            return;
        }
        self.state.set(if rejected {
            ResolverState::Rejected
        } else {
            ResolverState::Resolved
        });
        let function = if rejected {
            &self.capability.reject
        } else {
            &self.capability.resolve
        };
        if invocation
            .Call(function, &invocation.Undefined(), &[value])
            .is_err()
        {
            panic!("native Promise resolver invocation failed");
        }
    }
    pub fn Detach(&self) {
        self.detached.set(true);
    }
}
