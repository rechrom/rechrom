// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/abort_signal.h:41-209; .cc:45-443;
// abort_signal_composition_manager.cc:19-161. Weak linked collections keep
// insertion order. No binding-side aborted flag or algorithm snapshot owner.
use super::abort_signal_registry::AbortSignalRegistry;
use crate::bindings::core::v8::script_promise::{ScriptInvocation, ScriptNativeObject, Value};
use interaction_dispatch::core::dom::events::{
    event::{Event, EventClock, EventRef, EventTargetRef},
    event_target::EventTarget,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalType {
    Controller,
    Aborted,
    Timeout,
    Composite,
}
pub trait CancellableAbortTask {
    fn Cancel(&self);
}
/// The synchronous VM operations used by AbortSignal's source algorithm.
/// Keeping this port separate lets bindings lend their live invocation without
/// manufacturing unrelated Promise or serialization capabilities.
pub trait AbortSignalInvocation {
    fn IsUndefined(&self, value: &Value) -> bool;
    fn Undefined(&self) -> Value;
    fn DOMException(&self, name: &'static str, message: &str) -> Value;
    fn KeepAlive(&self, values: &[Value]) -> Box<dyn std::any::Any>;
    fn EventListenerInvoker(
        &self,
    ) -> &dyn interaction_dispatch::core::dom::events::event_target::ScopedEventListenerInvoker;
}
pub trait AbortSignalTimeoutTask {
    fn Run(&self, invocation: &dyn AbortSignalInvocation);
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>>;
}
pub struct ScriptAbortInvocation<'a>(pub &'a dyn ScriptInvocation);
impl AbortSignalInvocation for ScriptAbortInvocation<'_> {
    fn IsUndefined(&self, value: &Value) -> bool {
        self.0.IsUndefined(value)
    }
    fn Undefined(&self) -> Value {
        self.0.Undefined()
    }
    fn DOMException(&self, name: &'static str, message: &str) -> Value {
        self.0.DOMException(name, message)
    }
    fn KeepAlive(&self, values: &[Value]) -> Box<dyn std::any::Any> {
        self.0.KeepAlive(values)
    }
    fn EventListenerInvoker(
        &self,
    ) -> &dyn interaction_dispatch::core::dom::events::event_target::ScopedEventListenerInvoker
    {
        self.0.EventListenerInvoker()
    }
}
pub trait AbortSignalContext {
    fn IsContextDestroyed(&self) -> bool;
    fn Clock(&self) -> &dyn EventClock;
    fn CreateEventTarget(&self, signal: Weak<AbortSignal>) -> Rc<EventTarget>;
    fn Registry(&self) -> Rc<AbortSignalRegistry>;
    /// Publish changed source Member fields before entering script. This must
    /// only enqueue tracing metadata and cannot borrow a running VM.
    fn TraceSignal(&self, signal: Rc<AbortSignal>);
    /// The source runner is JavascriptTimerImmediate for zero, otherwise
    /// JavascriptTimerDelayedHighNesting; its scheduling policy stays native.
    fn PostTimeout(
        &self,
        milliseconds: u64,
        high_nesting: bool,
        task: Rc<dyn AbortSignalTimeoutTask>,
    ) -> Rc<dyn CancellableAbortTask>;
}
pub trait AbortAlgorithm {
    fn Run(&self, invocation: &dyn AbortSignalInvocation);
    fn VisitScriptValues(&self, _visit: &mut dyn FnMut(Value)) {}
    fn VisitNativeObjects(&self, _visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {}
}
struct OnceAlgorithm(RefCell<Option<Box<dyn FnOnce()>>>);
impl AbortAlgorithm for OnceAlgorithm {
    fn Run(&self, _: &dyn AbortSignalInvocation) {
        let callback = self.0.borrow_mut().take();
        if let Some(callback) = callback {
            callback();
        }
    }
}
pub struct AlgorithmHandle {
    algorithm: Rc<dyn AbortAlgorithm>,
    _signal: Rc<AbortSignal>,
}
impl ScriptNativeObject for AlgorithmHandle {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "AbortSignalAlgorithmHandle"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        self.algorithm.VisitScriptValues(visit);
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        visit(self._signal.clone());
        self.algorithm.VisitNativeObjects(visit);
    }
}
impl interaction_dispatch::core::dom::events::add_event_listener_options_resolved::AbortAlgorithmHandle for AlgorithmHandle {}
pub struct AbortSignal {
    self_: Weak<Self>,
    pub target: Rc<EventTarget>,
    context: Rc<dyn AbortSignalContext>,
    signal_type: SignalType,
    reason: RefCell<Option<Value>>,
    running: Cell<bool>,
    algorithms: RefCell<Vec<Weak<AlgorithmHandle>>>,
    settled: Cell<bool>,
    sources: RefCell<Vec<Weak<AbortSignal>>>,
    dependents: RefCell<Vec<Weak<AbortSignal>>>,
    timeout_task: RefCell<Option<Rc<dyn CancellableAbortTask>>>,
}
impl AbortSignal {
    pub fn new(context: Rc<dyn AbortSignalContext>, signal_type: SignalType) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            target: context.CreateEventTarget(self_.clone()),
            context,
            signal_type,
            reason: RefCell::new(None),
            running: Cell::new(false),
            algorithms: RefCell::new(Vec::new()),
            settled: Cell::new(false),
            sources: RefCell::new(Vec::new()),
            dependents: RefCell::new(Vec::new()),
            timeout_task: RefCell::new(None),
        })
    }
    pub fn Composite(context: Rc<dyn AbortSignalContext>, sources: &[Rc<AbortSignal>]) -> Rc<Self> {
        let result = Self::new(context, SignalType::Composite);
        for source in sources {
            if let Some(reason) = source.reason.borrow().clone() {
                *result.reason.borrow_mut() = Some(reason);
                break;
            }
        }
        if !result.Aborted() {
            for source in sources {
                if source.signal_type == SignalType::Composite {
                    let flattened: Vec<_> = source
                        .sources
                        .borrow()
                        .iter()
                        .filter_map(Weak::upgrade)
                        .collect();
                    for source in flattened {
                        result.AddSource(&source);
                    }
                } else {
                    result.AddSource(source);
                }
            }
        }
        if result.sources.borrow().is_empty() {
            result.Settle();
        }
        let _ = result.context.Registry();
        result
    }
    pub fn StaticAbort(context: Rc<dyn AbortSignalContext>, reason: Value) -> Rc<Self> {
        let result = Self::new(context, SignalType::Aborted);
        *result.reason.borrow_mut() = Some(reason);
        result.Settle();
        result
    }
    pub fn StaticAbortWithoutReason(
        context: Rc<dyn AbortSignalContext>,
        invocation: &dyn AbortSignalInvocation,
    ) -> Rc<Self> {
        Self::StaticAbort(
            context,
            invocation.DOMException("AbortError", "signal is aborted without reason"),
        )
    }
    pub fn Timeout(context: Rc<dyn AbortSignalContext>, milliseconds: u64) -> Rc<Self> {
        let result = Self::new(context, SignalType::Timeout);
        if result.context.IsContextDestroyed() {
            return result;
        }
        let handle = result.context.PostTimeout(
            milliseconds,
            milliseconds != 0,
            Rc::new(TimeoutTask(result.clone())),
        );
        *result.timeout_task.borrow_mut() = Some(handle);
        result
    }
    pub fn Identity(&self) -> usize {
        self.self_.as_ptr() as usize
    }
    fn TraceChanged(&self) {
        self.context
            .TraceSignal(self.self_.upgrade().expect("live signal"));
    }
    pub fn Aborted(&self) -> bool {
        self.reason.borrow().is_some()
    }
    pub fn Reason(&self, invocation: &dyn AbortSignalInvocation) -> Value {
        self.reason
            .borrow()
            .clone()
            .unwrap_or_else(|| invocation.Undefined())
    }
    pub fn ThrowIfAborted(&self) -> Result<(), Value> {
        match self.reason.borrow().clone() {
            Some(reason) => Err(reason),
            None => Ok(()),
        }
    }
    pub fn CanAbort(&self) -> bool {
        !self.Aborted() && !self.settled.get()
    }
    pub fn AddScopedAlgorithm(
        &self,
        algorithm: Rc<dyn AbortAlgorithm>,
    ) -> Option<Rc<AlgorithmHandle>> {
        if self.Aborted() || self.settled.get() {
            return None;
        }
        assert!(!self.running.get());
        let handle = Rc::new(AlgorithmHandle {
            algorithm,
            _signal: self.self_.upgrade().expect("live signal"),
        });
        self.algorithms.borrow_mut().push(Rc::downgrade(&handle));
        Some(handle)
    }
    pub fn RemoveAlgorithm(&self, handle: &Rc<AlgorithmHandle>) {
        if self.Aborted() || self.settled.get() {
            return;
        }
        assert!(!self.running.get());
        self.algorithms
            .borrow_mut()
            .retain(|h| !h.ptr_eq(&Rc::downgrade(handle)));
    }
    fn AddSource(&self, source: &Rc<AbortSignal>) {
        if source.settled.get() {
            return;
        }
        debug_assert!(source.signal_type != SignalType::Composite);
        debug_assert_ne!(self.Identity(), source.Identity());
        if self
            .sources
            .borrow()
            .iter()
            .any(|s| s.ptr_eq(&Rc::downgrade(source)))
        {
            return;
        }
        self.sources.borrow_mut().push(Rc::downgrade(source));
        assert!(!source.Aborted());
        source.dependents.borrow_mut().push(self.self_.clone());
    }
    pub fn SignalAbort(&self, invocation: &dyn AbortSignalInvocation, reason: Value) {
        if self.Aborted() {
            return;
        }
        assert_ne!(self.signal_type, SignalType::Composite);
        self.SetAbortReason(invocation, reason);
        let reason = self.reason.borrow().clone().expect("abort reason");
        let mut dependents = Vec::new();
        for signal in self.dependents.borrow().iter().filter_map(Weak::upgrade) {
            if !signal.Aborted() {
                signal.SetAbortReason(invocation, reason.clone());
                dependents.push(signal);
            }
        }
        self.RunAbortSteps(invocation);
        for signal in dependents {
            signal.RunAbortSteps(invocation);
            signal.Settle();
        }
        self.Settle();
    }
    fn SetAbortReason(&self, invocation: &dyn AbortSignalInvocation, reason: Value) {
        assert!(!self.Aborted());
        let reason = if invocation.IsUndefined(&reason) {
            invocation.DOMException("AbortError", "signal is aborted with undefined reason")
        } else {
            reason
        };
        *self.reason.borrow_mut() = Some(reason);
        self.TraceChanged();
    }
    fn RunAbortSteps(&self, invocation: &dyn AbortSignalInvocation) {
        assert!(!self.running.replace(true));
        struct Reset<'a>(&'a Cell<bool>);
        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _scope = Reset(&self.running);
        let algorithms: Vec<_> = self
            .algorithms
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        let mut captures = Vec::new();
        for handle in &algorithms {
            handle
                .algorithm
                .VisitScriptValues(&mut |value| captures.push(value));
        }
        let _roots = invocation.KeepAlive(&captures);
        for handle in algorithms {
            handle.algorithm.Run(invocation);
        }
        let event = EventRef::new(Event::CreateWithType("abort", self.context.Clock()));
        if self.target.GetExecutionContext().is_none() {
            return;
        }
        event.borrow_mut().BaseEventMut().SetTrusted(true);
        let target: EventTargetRef = self.target.clone();
        self.target.DispatchEventInternalWithTarget(
            &event,
            target,
            Some(invocation.EventListenerInvoker()),
        );
    }
    fn Settle(&self) {
        debug_assert!(!self.settled.get());
        self.settled.set(true);
        assert!(!self.running.get());
        self.algorithms.borrow_mut().clear();
        if self.signal_type == SignalType::Composite {
            if !self.context.IsContextDestroyed() {
                self.context.Registry().UnregisterSignal(self);
            }
            self.sources.borrow_mut().clear();
        } else {
            let dependents: Vec<_> = self
                .dependents
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .collect();
            for signal in dependents {
                if signal.settled.get() {
                    continue;
                }
                signal
                    .sources
                    .borrow_mut()
                    .retain(|s| !s.ptr_eq(&self.self_));
                if signal
                    .sources
                    .borrow()
                    .iter()
                    .all(|s| s.strong_count() == 0)
                {
                    signal.Settle();
                }
            }
            self.dependents.borrow_mut().clear();
        }
    }
    pub fn DetachFromController(&self) {
        if self.Aborted() {
            return;
        }
        assert!(!self.running.get());
        self.Settle();
    }
    pub fn ListenerChanged(&self, event_type: &str, added: bool) {
        if self.signal_type != SignalType::Composite
            || event_type != "abort"
            || self.settled.get()
            || self.context.IsContextDestroyed()
        {
            return;
        }
        if !added && self.target.HasEventListenersForType(event_type) {
            return;
        }
        if added {
            self.context
                .Registry()
                .RegisterSignal(self.self_.upgrade().expect("live composite signal"));
        } else {
            self.context.Registry().UnregisterSignal(self);
        }
    }
    pub fn ContextDestroyed(&self) {
        self.target.RemoveAllEventListeners();
        let task = self.timeout_task.borrow_mut().take();
        if let Some(task) = task {
            task.Cancel();
        }
    }
}
struct TimeoutTask(Rc<AbortSignal>);
impl AbortSignalTimeoutTask for TimeoutTask {
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>> {
        vec![self.0.clone()]
    }
    fn Run(&self, invocation: &dyn AbortSignalInvocation) {
        let reason = invocation.DOMException("TimeoutError", "signal timed out");
        self.0.SignalAbort(invocation, reason);
    }
}
impl interaction_dispatch::core::dom::events::add_event_listener_options_resolved::AbortSignal
    for AbortSignal
{
    fn aborted(&self) -> bool {
        self.Aborted()
    }
    fn AddAlgorithm(&self,callback:Box<dyn FnOnce()>)->Option<Rc<dyn interaction_dispatch::core::dom::events::add_event_listener_options_resolved::AbortAlgorithmHandle>>{
        self.AddScopedAlgorithm(Rc::new(OnceAlgorithm(RefCell::new(Some(callback))))).map(|h|h as Rc<dyn interaction_dispatch::core::dom::events::add_event_listener_options_resolved::AbortAlgorithmHandle>)
    }
}

impl crate::bindings::core::v8::script_promise::ScriptNativeObject for AbortSignal {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "AbortSignal"
    }
    fn EventTarget(&self) -> Option<Rc<EventTarget>> {
        Some(self.target.clone())
    }
    fn DidAddEventListener(&self, kind: &str) {
        self.ListenerChanged(kind, true);
    }
    fn DidRemoveEventListener(&self, kind: &str) {
        self.ListenerChanged(kind, false);
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        if let Some(value) = self.reason.borrow().clone() {
            visit(value);
        }
    }
}
