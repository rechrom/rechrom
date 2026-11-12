// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/abort_controller.h:20-52; .cc:16-56.
use super::abort_signal::{AbortSignal, AbortSignalContext, ScriptAbortInvocation, SignalType};
use crate::bindings::core::v8::script_promise::{ScriptInvocation, Value};
use std::rc::Rc;
pub struct AbortController {
    signal: Rc<AbortSignal>,
}
impl AbortController {
    pub fn new(context: Rc<dyn AbortSignalContext>) -> Rc<Self> {
        Rc::new(Self {
            signal: AbortSignal::new(context, SignalType::Controller),
        })
    }
    pub fn Signal(&self) -> &Rc<AbortSignal> {
        &self.signal
    }
    pub fn Abort(&self, invocation: &dyn ScriptInvocation, reason: Value) {
        self.signal
            .SignalAbort(&ScriptAbortInvocation(invocation), reason);
    }
    pub fn AbortWithoutReason(&self, invocation: &dyn ScriptInvocation) {
        self.Abort(
            invocation,
            invocation.DOMException("AbortError", "signal is aborted without reason"),
        );
    }
}
impl Drop for AbortController {
    fn drop(&mut self) {
        self.signal.DetachFromController();
    }
}

impl crate::bindings::core::v8::script_promise::ScriptNativeObject for AbortController {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "AbortController"
    }
    fn VisitNativeObjects(
        &self,
        visit: &mut dyn FnMut(
            Rc<dyn crate::bindings::core::v8::script_promise::ScriptNativeObject>,
        ),
    ) {
        visit(self.signal.clone());
    }
}
