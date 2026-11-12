// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: platform/bindings/script_state.h:80-171; bindings/core/v8/
// script_promise.h:194-229, script_promise_resolver.h:79-104,178-209.
// These opaque values retain VM identity. The invocation is a borrowed stack
// scope, not a service which may be stored, upgraded, or used after dispatch.
use std::{any::Any, rc::Rc};
pub trait ScriptValue: Any {}
pub use navigation_history::SerializedScriptValue;
pub type Value = Rc<dyn ScriptValue>;
#[derive(Clone, Debug)]
pub struct ScriptSourceLocation {
    pub url: String,
    pub line: u32,
    pub column: u32,
}
#[derive(Clone, Debug)]
pub struct ScriptExceptionMessage {
    pub message: Vec<u16>,
    pub location: Option<ScriptSourceLocation>,
}
/// Source Trace edges, visited on the owner thread before script reentry.
/// The collector consumes published edges and never invokes this interface.
pub trait ScriptNativeObject: Any {
    fn AsAny(&self) -> &dyn Any;
    fn ScriptIdentity(&self) -> usize {
        self as *const Self as *const () as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str;
    fn VisitScriptValues(&self, _visit: &mut dyn FnMut(Value)) {}
    fn VisitNativeObjects(&self, _visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {}
    /// Conditional Member edges in a source weak-key collection. The key is
    /// the exact binding cell of the native key, never its target or callback.
    fn VisitEphemerons(&self, _visit: &mut dyn FnMut(u64, Rc<dyn ScriptNativeObject>)) {}
    /// Owner-thread weak collection cleanup after the collector releases keys.
    /// This must neither invoke script nor touch the VM.
    fn WeakKeysReleased(&self, _keys: &[u64]) {}
    fn EventTarget(&self) -> Option<Rc<crate::core::dom::events::event_target::EventTarget>> {
        None
    }
    fn DidAddEventListener(&self, _event_type: &str) {}
    fn DidRemoveEventListener(&self, _event_type: &str) {}
    fn WrapperReleased(&self) {
        if let Some(target) = self.EventTarget() {
            target.RemoveAllEventListeners();
        }
    }
}
#[derive(Clone)]
pub struct ScriptPromise(pub Value);
#[derive(Clone)]
pub struct ScriptPromiseCapability {
    pub promise: ScriptPromise,
    pub resolve: Value,
    pub reject: Value,
}
pub trait PromiseReaction {
    fn React(&self, invocation: &dyn ScriptInvocation, value: Value);
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>> {
        Vec::new()
    }
}
pub trait NativeMicrotask {
    fn Run(&self, invocation: &dyn ScriptInvocation);
    fn Captures(&self) -> Vec<Rc<dyn ScriptNativeObject>> {
        Vec::new()
    }
}
pub trait ScriptInvocation {
    fn ExceptionMessage(&self, value: &Value) -> ScriptExceptionMessage;
    fn TaskAttributionTracker(
        &self,
    ) -> Option<Rc<foundation::task_attribution::TaskAttributionTracker>>;
    fn IsInContext(&self) -> bool;
    fn EventListenerInvoker(
        &self,
    ) -> &dyn crate::core::dom::events::event_target::ScopedEventListenerInvoker;
    fn ContextIsValid(&self) -> bool;
    /// ScriptState identity, distinguishing isolated worlds in the same Agent.
    fn WorldIdentity(&self) -> u64;
    /// DOMWrapperWorld identifier, distinct from the ScriptState identity.
    fn WorldId(&self) -> i32;
    /// None means the current isolate has no JS frame. The owning source
    /// CaptureSourceLocation still performs its Document/parser fallback.
    fn CaptureStackSourceLocation(&self) -> Option<ScriptSourceLocation>;
    fn ScriptToolInvocationId(&self) -> Option<[u64; 2]>;
    /// Lends another live world of this execution context. An unavailable world
    /// does not run the callback. No realm/runtime reference may escape it.
    fn WithWorld(&self, world: u64, callback: &mut dyn FnMut(&dyn ScriptInvocation));
    fn WithRunMicrotasksScope(&self, callback: &mut dyn FnMut());
    fn Undefined(&self) -> Value;
    fn Null(&self) -> Value;
    fn IsUndefined(&self, value: &Value) -> bool;
    fn String(&self, value: &[u16]) -> Value;
    fn ToString(&self, value: &Value) -> Result<Vec<u16>, Value>;
    fn ToInt32(&self, value: &Value) -> Result<i32, Value>;
    fn IsNull(&self, value: &Value) -> bool;
    fn IsCallable(&self, value: &Value) -> bool;
    fn IsObject(&self, value: &Value) -> bool;
    fn Get(&self, value: &Value, property: &str) -> Result<Value, Value>;
    fn TypeError(&self, message: &[u16]) -> Value;
    fn EvaluateClassicScript(&self, source: &[u16], base_url: &str) -> Result<Value, Value>;
    fn PrimitiveString(&self, value: &Value) -> Option<Vec<u16>>;
    fn DOMException(&self, name: &'static str, message: &str) -> Value;
    fn Call(&self, function: &Value, receiver: &Value, args: &[Value]) -> Result<Value, Value>;
    fn ReportException(&self, value: &Value);
    fn NewPromiseCapability(&self) -> Result<ScriptPromiseCapability, Value>;
    fn PromiseResolve(&self, value: Value) -> Result<ScriptPromise, Value>;
    fn PerformPromiseThen(
        &self,
        promise: &ScriptPromise,
        fulfilled: Rc<dyn PromiseReaction>,
        rejected: Rc<dyn PromiseReaction>,
    ) -> Result<(), Value>;
    fn MarkPromiseHandled(&self, promise: &ScriptPromise);
    fn EnqueueMicrotask(&self, task: Rc<dyn NativeMicrotask>) -> Result<(), Value>;
    /// Bindings project a concrete native class identity without making the VM
    /// or event-dispatch package depend on that class's engine.
    fn NativeObjectValue(&self, object: Rc<dyn ScriptNativeObject>) -> Value;
    /// A native field changed. Bindings publish its source Trace edges before
    /// JS reentry; this does not create another native state owner.
    fn TraceNativeObject(&self, object: Rc<dyn ScriptNativeObject>);
    /// Strong stack roots for a source-local HeapVector moved out of its
    /// traced field before invoking callbacks. The guard stores no runtime.
    fn KeepAlive(&self, values: &[Value]) -> Box<dyn Any>;
    fn SerializeForStorage(&self, value: &Value) -> Result<Rc<dyn SerializedScriptValue>, Value>;
    fn Deserialize(&self, value: &Rc<dyn SerializedScriptValue>) -> Result<Value, Value>;
}
/// A scriptless Window still has the native event/default-action lifecycle.
/// These values carry a precise failure if a branch actually needs the VM.
pub enum NoScriptValue {
    Undefined,
    Null,
    DOMException { name: &'static str, message: String },
    CapabilityUnavailable(&'static str),
    NativeObject(Rc<dyn ScriptNativeObject>),
    String(Vec<u16>),
}
impl ScriptValue for NoScriptValue {}
pub struct NoScriptInvocation<'a> {
    pub invoker: &'a dyn crate::core::dom::events::event_target::ScopedEventListenerInvoker,
    pub context_is_valid: &'a dyn Fn() -> bool,
}
impl NoScriptInvocation<'_> {
    fn Unavailable(&self, operation: &'static str) -> Value {
        Rc::new(NoScriptValue::CapabilityUnavailable(operation))
    }
}
impl ScriptInvocation for NoScriptInvocation<'_> {
    fn ExceptionMessage(&self, value: &Value) -> ScriptExceptionMessage {
        let message = match (value.as_ref() as &dyn Any)
            .downcast_ref::<NoScriptValue>()
            .expect("scriptless native value")
        {
            NoScriptValue::Undefined => "undefined".encode_utf16().collect(),
            NoScriptValue::Null => "null".encode_utf16().collect(),
            NoScriptValue::String(value) => value.clone(),
            NoScriptValue::DOMException { name, message } => {
                format!("{name}: {message}").encode_utf16().collect()
            }
            NoScriptValue::CapabilityUnavailable(operation) => {
                format!("Required script capability: {operation}")
                    .encode_utf16()
                    .collect()
            }
            NoScriptValue::NativeObject(value) => {
                format!("[object {}]", value.ScriptInterfaceName())
                    .encode_utf16()
                    .collect()
            }
        };
        let mut result: Vec<u16> = "Uncaught ".encode_utf16().collect();
        result.extend(message);
        ScriptExceptionMessage {
            message: result,
            location: None,
        }
    }
    fn TaskAttributionTracker(
        &self,
    ) -> Option<Rc<foundation::task_attribution::TaskAttributionTracker>> {
        None
    }
    fn IsInContext(&self) -> bool {
        false
    }

    fn EvaluateClassicScript(&self, _: &[u16], _: &str) -> Result<Value, Value> {
        Err(self.Unavailable("EvaluateClassicScript"))
    }
    fn PrimitiveString(&self, value: &Value) -> Option<Vec<u16>> {
        match (value.as_ref() as &dyn Any).downcast_ref::<NoScriptValue>() {
            Some(NoScriptValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    }
    fn String(&self, value: &[u16]) -> Value {
        Rc::new(NoScriptValue::String(value.to_vec()))
    }
    fn ToString(&self, value: &Value) -> Result<Vec<u16>, Value> {
        match (value.as_ref() as &dyn Any).downcast_ref::<NoScriptValue>() {
            Some(NoScriptValue::String(value)) => Ok(value.clone()),
            _ => Err(self.Unavailable("ToString")),
        }
    }
    fn ToInt32(&self, _: &Value) -> Result<i32, Value> {
        Err(self.Unavailable("ToInt32"))
    }
    fn IsNull(&self, value: &Value) -> bool {
        matches!(
            (value.as_ref() as &dyn Any).downcast_ref::<NoScriptValue>(),
            Some(NoScriptValue::Null)
        )
    }
    fn IsCallable(&self, _: &Value) -> bool {
        false
    }
    fn IsObject(&self, value: &Value) -> bool {
        matches!(
            (value.as_ref() as &dyn Any).downcast_ref::<NoScriptValue>(),
            Some(NoScriptValue::NativeObject(_))
        )
    }
    fn Get(&self, _: &Value, _: &str) -> Result<Value, Value> {
        Err(self.Unavailable("Get"))
    }
    fn TypeError(&self, _: &[u16]) -> Value {
        self.Unavailable("TypeError")
    }
    fn EventListenerInvoker(
        &self,
    ) -> &dyn crate::core::dom::events::event_target::ScopedEventListenerInvoker {
        self.invoker
    }
    fn ContextIsValid(&self) -> bool {
        (self.context_is_valid)()
    }
    fn WorldIdentity(&self) -> u64 {
        0
    }
    fn WorldId(&self) -> i32 {
        0
    }
    fn CaptureStackSourceLocation(&self) -> Option<ScriptSourceLocation> {
        None
    }
    fn ScriptToolInvocationId(&self) -> Option<[u64; 2]> {
        None
    }
    fn WithWorld(&self, world: u64, callback: &mut dyn FnMut(&dyn ScriptInvocation)) {
        if world == 0 && self.ContextIsValid() {
            callback(self);
        }
    }
    fn WithRunMicrotasksScope(&self, callback: &mut dyn FnMut()) {
        callback();
    }
    fn Undefined(&self) -> Value {
        Rc::new(NoScriptValue::Undefined)
    }
    fn Null(&self) -> Value {
        Rc::new(NoScriptValue::Null)
    }
    fn IsUndefined(&self, value: &Value) -> bool {
        matches!(
            (value.as_ref() as &dyn Any).downcast_ref::<NoScriptValue>(),
            Some(NoScriptValue::Undefined)
        )
    }
    fn DOMException(&self, name: &'static str, message: &str) -> Value {
        Rc::new(NoScriptValue::DOMException {
            name,
            message: message.into(),
        })
    }
    fn Call(&self, _: &Value, _: &Value, _: &[Value]) -> Result<Value, Value> {
        Err(self.Unavailable("Call"))
    }
    fn ReportException(&self, _: &Value) {
        panic!("scriptless invocation cannot report an author exception")
    }
    fn NewPromiseCapability(&self) -> Result<ScriptPromiseCapability, Value> {
        Err(self.Unavailable("NewPromiseCapability"))
    }
    fn PromiseResolve(&self, _: Value) -> Result<ScriptPromise, Value> {
        Err(self.Unavailable("PromiseResolve"))
    }
    fn PerformPromiseThen(
        &self,
        _: &ScriptPromise,
        _: Rc<dyn PromiseReaction>,
        _: Rc<dyn PromiseReaction>,
    ) -> Result<(), Value> {
        Err(self.Unavailable("PerformPromiseThen"))
    }
    fn MarkPromiseHandled(&self, _: &ScriptPromise) {
        unreachable!("scriptless invocation cannot create a Promise")
    }
    fn EnqueueMicrotask(&self, _: Rc<dyn NativeMicrotask>) -> Result<(), Value> {
        Err(self.Unavailable("EnqueueMicrotask"))
    }
    fn NativeObjectValue(&self, object: Rc<dyn ScriptNativeObject>) -> Value {
        Rc::new(NoScriptValue::NativeObject(object))
    }
    fn TraceNativeObject(&self, _: Rc<dyn ScriptNativeObject>) {}
    fn KeepAlive(&self, values: &[Value]) -> Box<dyn Any> {
        Box::new(values.to_vec())
    }
    fn SerializeForStorage(&self, _: &Value) -> Result<Rc<dyn SerializedScriptValue>, Value> {
        Err(self.Unavailable("SerializeForStorage"))
    }
    fn Deserialize(&self, _: &Rc<dyn SerializedScriptValue>) -> Result<Value, Value> {
        Err(self.Unavailable("Deserialize"))
    }
}
