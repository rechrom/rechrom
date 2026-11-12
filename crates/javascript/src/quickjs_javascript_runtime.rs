//! Pure Rust QuickJS backend for the engine-neutral browser host boundary.
#![allow(non_snake_case)]

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::javascript_runtime::*;
#[path = "quickjs_adapter.rs"]
mod adapter;
#[path = "quickjs_module_worker.rs"]
mod module_worker;
#[path = "quickjs_native_microtasks.rs"]
mod native_microtasks;
pub use adapter::Value as JsValue;
use adapter::*;

#[path = "quickjs_console.rs"]
mod console;
#[path = "quickjs_intl.rs"]
mod intl;
#[path = "quickjs_regexp.rs"]
mod regexp;
#[path = "script_regexp.rs"]
pub mod script_regexp;

#[path = "quickjs_bridge_profile.rs"]
mod bridge_profile;
#[path = "quickjs_host_graph.rs"]
mod host_graph;
#[path = "quickjs_host_namespaces.rs"]
mod host_namespaces;
use host_graph::{HostGraph, SynchronizeHostGraph};
enum WrapperSlot {
    Strong(JsObject),
    Weak(WeakObject),
}

#[cfg(test)]
mod array_buffer_host_bridge_tests {
    use super::*;

    struct Host(Rc<RefCell<Vec<u8>>>);
    impl JavaScriptHostBindings for Host {
        fn GlobalNames(&self) -> Vec<String> {
            vec!["acceptBytes".into()]
        }
        fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
            match (call.receiver, call.operation, call.member) {
                (0, HostOperation::kGet, "acceptBytes") => HostResult {
                    value: HostValue::Method(HostMethodRef {
                        receiver: 0,
                        name: "acceptBytes".into(),
                    }),
                    ..Default::default()
                },
                (0, HostOperation::kCall, "acceptBytes") => {
                    if let [HostValue::Bytes(bytes)] = call.arguments {
                        *self.0.borrow_mut() = bytes.to_vec();
                        HostResult::default()
                    } else {
                        HostResult::Failure(
                            JavaScriptExceptionKind::kTypeError,
                            "expected ArrayBuffer bytes",
                        )
                    }
                }
                _ => HostResult {
                    handled: false,
                    ..Default::default()
                },
            }
        }
    }

    #[test]
    fn array_buffer_crosses_host_boundary_as_exact_bytes() {
        let bytes = Rc::new(RefCell::new(Vec::new()));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host(bytes.clone()))));
        let result = runtime.Evaluate(
            &realm,
            "acceptBytes(new Uint8Array([0,1,127,128,255]).buffer)",
            "array-buffer-host-bridge.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(&*bytes.borrow(), &[0, 1, 127, 128, 255]);
    }
}
impl WrapperSlot {
    fn get(&self) -> Option<JsObject> {
        match self {
            Self::Strong(object) => Some(object.clone()),
            Self::Weak(object) => object.upgrade(),
        }
    }
    fn identity(&self) -> (usize, usize) {
        match self {
            Self::Strong(object) => object.identity(),
            Self::Weak(object) => object.identity(),
        }
    }
}
struct Wrapper {
    target: WrapperSlot,
    object: WrapperSlot,
    prototype_installed: bool,
}

struct State {
    host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    realms: std::rc::Weak<RefCell<Vec<JavaScriptRealm>>>,
    agent_identity: usize,
    document_object: Option<HostObjectId>,
    object_host: Option<Rc<RefCell<dyn JavaScriptHostBindings>>>,
    global_host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    namespaces: Rc<host_namespaces::NamespaceRegistry>,
    global: Value,
    wrappers: HashMap<HostObjectId, Wrapper>,
    host_graph: HostGraph,
    // Realm-local identity. Weak slots retain tombstones until removal, so
    // a collected wrapper's address cannot alias a new object in this index.
    wrapper_ids: HashMap<(usize, usize), HostObjectId>,
    fallbacks: HashMap<String, Value>,
    reflect_get: Value,
    reflect_set: Value,
    hooks: Rc<Hooks>,
}

#[derive(Default)]
struct Hooks {
    wrappers: Rc<RefCell<host_namespaces::WrapperOrigins>>,
    task_tracker: Option<Rc<foundation::task_attribution::TaskAttributionTracker>>,
    script_tool_invocation: Cell<Option<[u64; 2]>>,
    checkpoint_running: Cell<bool>,
    checkpoint_suppression: Cell<usize>,
    rejections: RefCell<Vec<(JsObject, Value)>>,
    exceptions: RefCell<Vec<JavaScriptException>>,
}
struct ScriptToolScope {
    hooks: Rc<Hooks>,
    previous: Option<[u64; 2]>,
}
impl JavaScriptScriptToolInvocationScope for ScriptToolScope {}
impl Drop for ScriptToolScope {
    fn drop(&mut self) {
        self.hooks.script_tool_invocation.set(self.previous);
    }
}

struct Tracker(Rc<Hooks>);
impl RejectionTracker for Tracker {
    fn on_rejection(&mut self, _: &mut Context, promise: &JsObject, reason: &Value, handled: bool) {
        let mut pending = self.0.rejections.borrow_mut();
        pending.retain(|(p, _)| p != promise);
        if !handled {
            pending.push((promise.clone(), reason.clone()));
        }
    }
}

struct RealmRuntimeData {
    context: WeakContext,
    state: std::rc::Weak<RefCell<State>>,
}
pub struct QuickJsRealm {
    pub context: Context,
    state: Rc<RefCell<State>>,
}

impl Drop for QuickJsRealm {
    fn drop(&mut self) {
        // Host callables retain their state; release cached JS handles before
        // the context's heap is destroyed so that this does not form a cycle.
        for namespace in host_namespaces::States(&self.state) {
            let mut namespace = namespace.borrow_mut();
            namespace.wrapper_ids.clear();
            namespace.wrappers.clear();
            namespace.host_graph.Clear();
            namespace.fallbacks.clear();
            namespace.global = Value::Undefined;
            namespace.reflect_get = Value::Undefined;
            namespace.reflect_set = Value::Undefined;
        }
        host_namespaces::Clear(&self.state);
        let mut state = self.state.borrow_mut();
        state.wrapper_ids.clear();
        state.wrappers.clear();
        state.host_graph.Clear();
        state.fallbacks.clear();
        state.global = Value::Undefined;
        state.reflect_get = Value::Undefined;
        state.reflect_set = Value::Undefined;
        state.hooks.rejections.borrow_mut().clear();
    }
}

struct PreparedClassicScript {
    realm: JavaScriptRealm,
    worker: module_worker::Job,
    outcome: Option<module_worker::Outcome>,
}
pub struct QuickJsJavaScriptRuntime {
    realms: Rc<RefCell<Vec<JavaScriptRealm>>>,
    stack_budget: usize,
    compilation_executor: Option<std::sync::Arc<dyn executor::Executor>>,
    module_worker: Option<module_worker::Worker>,
    module_worker_unavailable: bool,
    prepared_classic_scripts: Vec<PreparedClassicScript>,
    module_compilation_wake: Option<JavaScriptModuleCompilationWake>,
}

struct QuickJsModuleCompilationJob {
    realm: JavaScriptRealm,
    worker: module_worker::Job,
    completed: bool,
}
impl Default for QuickJsJavaScriptRuntime {
    fn default() -> Self {
        Self {
            realms: Default::default(),
            stack_budget: 1024 * 1024,
            compilation_executor: Some(executor::inline()),
            module_worker: None,
            module_worker_unavailable: false,
            prepared_classic_scripts: Vec::new(),
            module_compilation_wake: None,
        }
    }
}
impl QuickJsJavaScriptRuntime {
    pub fn new() -> Self {
        Self::default()
    }
    /// Configure the embedding's maximum stack size through QuickJS's
    /// public API. The caller must leave room for the host's own stack frames
    /// and select a budget below the actual stack available on its thread.
    /// The default remains 1 MiB for threads whose stack size is unknown.
    pub fn with_max_stack_size(bytes: usize) -> Self {
        assert!(
            bytes > 0,
            "the maximum stack size must retain overflow protection"
        );
        Self {
            realms: Default::default(),
            stack_budget: bytes,
            compilation_executor: Some(executor::inline()),
            module_worker: None,
            module_worker_unavailable: false,
            prepared_classic_scripts: Vec::new(),
            module_compilation_wake: None,
        }
    }
}

impl JavaScriptFunctionStorage for JsObject {
    fn Downgrade(&self) -> WeakJavaScriptFunction {
        WeakJavaScriptFunction::new(WeakObject::new(self))
    }
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn IsSameFunction(&self, other: &dyn JavaScriptFunctionStorage) -> bool {
        other
            .AsAny()
            .downcast_ref::<Self>()
            .is_some_and(|v| self == v)
    }
}

// Native script fields keep objects weak in Rust; the corresponding host
// graph ValueReference is the sole strong edge into the VM. Primitive values
// cannot close object cycles and retain their immutable VM payload directly.
enum WeakQuickJsValue {
    Object(WeakObject),
    Primitive(Value),
}
impl JavaScriptValueStorage for Value {
    fn Downgrade(&self) -> WeakJavaScriptValue {
        WeakJavaScriptValue::new(match self {
            Value::Object(object) => WeakQuickJsValue::Object(WeakObject::new(object)),
            value => WeakQuickJsValue::Primitive(value.clone()),
        })
    }
}
impl WeakJavaScriptValueStorage for WeakQuickJsValue {
    fn Upgrade(&self) -> Option<JavaScriptValue> {
        let value = match self {
            Self::Object(object) => Value::Object(object.upgrade()?),
            Self::Primitive(value) => value.clone(),
        };
        Some(JavaScriptValue::new_traced(value))
    }
}

impl WeakJavaScriptFunctionStorage for WeakObject {
    fn Upgrade(&self) -> Option<JavaScriptFunction> {
        self.upgrade().map(JavaScriptFunction::new)
    }
}

fn from_js(value: &Value, context: &Context, state: &Rc<RefCell<State>>) -> HostValue {
    if let Some(bytes) = context.array_buffer_copy(value) {
        return HostValue::Bytes(bytes.into());
    }
    from_js_identity(value, context, state)
}
fn native_wrapper_argument(
    origin: Rc<RefCell<State>>,
    id: HostObjectId,
    value: &Value,
) -> HostValue {
    let native = host_namespaces::ObjectHost(&origin, id)
        .borrow()
        .NativeObject(id);
    let namespace = JavaScriptHostObjectNamespace::new(origin);
    let value_ = HostValue::Object(HostObjectRef { id });
    if let Some(native) = native {
        HostValue::Retained(
            Rc::new(JavaScriptNativeObjectReference::new(
                namespace.Downgrade(),
                native,
                JavaScriptValue::new_traced(value.clone()),
            )),
            Box::new(value_),
        )
    } else {
        namespace.Wrap(value_)
    }
}
fn from_js_argument(
    value: &Value,
    context: &Context,
    state: &Rc<RefCell<State>>,
    identity: bool,
) -> HostValue {
    if let Value::Object(object) = value {
        if let Some((origin, id)) = host_namespaces::WrapperOrigin(state, object) {
            return native_wrapper_argument(origin, id, value);
        }
    }
    if identity {
        from_js_identity(value, context, state)
    } else {
        from_js(value, context, state)
    }
}
fn from_js_identity(value: &Value, context: &Context, state: &Rc<RefCell<State>>) -> HostValue {
    let state = host_namespaces::Active(state);
    if let Value::Object(object) = value {
        if let Some((origin, id)) = host_namespaces::WrapperOrigin(&state, object) {
            return if Rc::ptr_eq(&origin, &state) {
                HostValue::Object(HostObjectRef { id })
            } else {
                native_wrapper_argument(origin, id, value)
            };
        }
    }
    match value {
        Value::Undefined => HostValue::default(),
        Value::Null => HostValue::Null(Default::default()),
        Value::Bool(v) => HostValue::Boolean(*v),
        Value::Int(v) => HostValue::Number(*v as f64),
        Value::Float(v) => HostValue::Number(*v),
        Value::Str(v) => {
            let units = v.to_utf16();
            match String::from_utf16(&units) {
                Ok(value) => HostValue::String(value),
                Err(_) => HostValue::String16(units.into()),
            }
        }
        Value::Object(o) => {
            let state = state.borrow();
            if Value::strict_equals(value, &state.global) {
                return HostValue::Object(HostObjectRef { id: 0 });
            }
            if let Some(&id) = state.wrapper_ids.get(&o.identity()) {
                return HostValue::Object(HostObjectRef { id });
            }
            if context.is_callable(value) {
                HostValue::JavaScriptFunction(JavaScriptFunction::new(o.clone()))
            } else {
                HostValue::JavaScriptValue(JavaScriptValue::new_traced(value.clone()))
            }
        }
        _ => HostValue::JavaScriptValue(JavaScriptValue::new_traced(value.clone())),
    }
}
fn host_arguments(
    ctx: &Context,
    state: &Rc<RefCell<State>>,
    method: &HostMethodRef,
    arguments: &[Value],
) -> Vec<HostValue> {
    let host = host_namespaces::ObjectHost(state, method.receiver);
    let policy = host.borrow().ArgumentPolicy(method.receiver, &method.name);
    let (mut values, policy, start) = match policy {
        HostArgumentPolicy::Forwarded {
            receiver,
            member,
            arguments: start,
        } => {
            let prefix: Vec<_> = arguments
                .iter()
                .take(start)
                .map(|value| from_js_argument(value, ctx, state, true))
                .collect();
            let policy = match (prefix.get(receiver), prefix.get(member)) {
                (Some(receiver), Some(HostValue::String(member))) => {
                    let (origin, value) = host_namespaces::TaggedIn(state, receiver)
                        .unwrap_or_else(|| (state.clone(), receiver));
                    if let HostValue::Object(receiver) = value {
                        host_namespaces::ObjectHost(&origin, receiver.id)
                            .borrow()
                            .ArgumentPolicy(receiver.id, member)
                    } else {
                        HostArgumentPolicy::Values
                    }
                }
                _ => HostArgumentPolicy::Values,
            };
            (prefix, policy, start)
        }
        policy => (Vec::with_capacity(arguments.len()), policy, 0),
    };
    values.extend(arguments.iter().skip(start).map(|value| match policy {
        HostArgumentPolicy::Identity => {
            HostValue::JavaScriptValue(JavaScriptValue::new_traced(value.clone()))
        }
        HostArgumentPolicy::NativeIdentity => {
            if let Value::Object(object) = value {
                if let Some((origin, id)) = host_namespaces::WrapperOrigin(state, object) {
                    let native = host_namespaces::ObjectHost(&origin, id)
                        .borrow()
                        .NativeObject(id)
                        .unwrap_or_else(|| Rc::new(HostObjectRef { id }));
                    let namespace = JavaScriptHostObjectNamespace::new(origin);
                    return HostValue::Retained(
                        Rc::new(JavaScriptNativeObjectReference::new(
                            namespace.Downgrade(),
                            native,
                            JavaScriptValue::new_traced(value.clone()),
                        )),
                        Box::new(HostValue::Object(HostObjectRef { id })),
                    );
                }
            }
            HostValue::JavaScriptValue(JavaScriptValue::new_traced(value.clone()))
        }
        _ => from_js_argument(value, ctx, state, false),
    }));
    values
}

fn error(ctx: &mut Context, exception: JavaScriptException) -> Value {
    let kind = match exception.kind {
        JavaScriptExceptionKind::kSyntaxError => ErrorKind::Syntax,
        JavaScriptExceptionKind::kTypeError => ErrorKind::Type,
        JavaScriptExceptionKind::kRangeError => ErrorKind::Range,
        JavaScriptExceptionKind::kRuntimeError => return ctx.make_plain_error(&exception.message),
    };
    ctx.make_error(kind, &exception.message)
}

fn exception(ctx: &mut Context, thrown: &Value, source: &str) -> JavaScriptException {
    let location = ctx.take_exception_location();
    exception_at(ctx, thrown, source, location)
}
fn exception_at(
    ctx: &mut Context,
    thrown: &Value,
    source: &str,
    location: Option<quickjs::quickjs::JSExceptionLocation>,
) -> JavaScriptException {
    let message = ctx.format_exception(thrown);
    let get = |ctx: &mut Context, name: &str| {
        let key = PropKey::from_string(&ctx.intern(name));
        ctx.get_property(thrown, &key).unwrap_or(Value::Undefined)
    };
    let stack_value = get(ctx, "stack");
    let stack = stack_value
        .as_string()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let line = get(ctx, "lineNumber").as_f64().unwrap_or(0.0) as usize;
    let column = get(ctx, "columnNumber")
        .as_f64()
        .map_or(usize::MAX, |n| n.max(1.0) as usize - 1);
    let kind = if message.starts_with("SyntaxError:") {
        JavaScriptExceptionKind::kSyntaxError
    } else if message.starts_with("TypeError:") {
        JavaScriptExceptionKind::kTypeError
    } else if message.starts_with("RangeError:") {
        JavaScriptExceptionKind::kRangeError
    } else {
        JavaScriptExceptionKind::kRuntimeError
    };
    JavaScriptException {
        kind,
        message,
        stack,
        source_name: location
            .as_ref()
            .map_or_else(|| source.into(), |location| location.filename.clone()),
        line: location.as_ref().map_or(line, |location| location.line),
        column: location.as_ref().map_or(column, |location| location.column),
        source_line: location.map_or_else(String::new, |location| location.source_line),
    }
}

fn result(ctx: &mut Context, value: JsResult<Value>, source: &str) -> JavaScriptResult {
    match value {
        Ok(value) => JavaScriptResult {
            value: JavaScriptValue::new_traced(value),
            exception: None,
        },
        Err(thrown) => JavaScriptResult {
            exception: Some(exception(ctx, &thrown, source)),
            ..Default::default()
        },
    }
}

fn key_name(ctx: &mut Context, key: &Value) -> JsResult<Option<(String, HostSymbol)>> {
    // A real Proxy trap already receives a property key. Preserve ownership
    // validation, while avoiding redundant property-key conversion and Rc allocation for
    // its string inputs. Symbols and generic callers keep
    // the original observable ToPropertyKey conversion below.
    ctx.validate_value(key)?;
    match key {
        Value::Str(string) => return Ok(Some((string.to_string_lossy(), HostSymbol::kNone))),
        _ => {}
    }
    let key = ctx.to_property_key(key)?;
    Ok(match key {
        PropKey::Str(s) => Some((s.to_string_lossy(), HostSymbol::kNone)),
        PropKey::Index(n) => Some((n.to_string(), HostSymbol::kNone)),
        PropKey::Sym(s) if same_symbol(&s, &ctx.well_known.get(WellKnownSymbol::Iterator)) => {
            Some((String::new(), HostSymbol::kIterator))
        }
        _ => None,
    })
}

#[derive(Clone)]
enum Action {
    Get(HostObjectId),
    Set(HostObjectId),
    Has(HostObjectId),
    GlobalGet(String),
    GlobalSet(String),
    Method(HostMethodRef),
    Callback(HostMethodRef, bool),
}
struct HostFunction {
    state: Rc<RefCell<State>>,
    action: Action,
}

fn invoke(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    call: &HostCall<'_>,
    mut arguments: Vec<HostValue>,
    receiver_origin: Option<(Rc<RefCell<State>>, HostObjectId)>,
) -> HostResult {
    let (mut origin, receiver) = receiver_origin.unwrap_or_else(|| (state.clone(), call.receiver));
    let mut host = host_namespaces::ObjectHost(&origin, receiver);
    let policy = host.borrow().ArgumentPolicy(call.receiver, call.member);
    // Ordinary IDL descriptors forward their actual receiver. Resolve that
    // wrapper's originating namespace before consulting the native binding,
    // never interpreting a new Document's equal numeric id in the old host.
    if let HostArgumentPolicy::Forwarded { receiver, .. } = policy {
        if let Some(value) = arguments.get(receiver) {
            let (owner, value) =
                host_namespaces::TaggedIn(state, value).unwrap_or_else(|| (state.clone(), value));
            if let HostValue::Object(receiver) = value {
                if receiver.id != 0 {
                    origin = owner;
                    host = host_namespaces::ObjectHost(&origin, receiver.id);
                }
            }
        }
    }
    let argument_namespace = host.borrow().ArgumentNamespace(&HostCall {
        arguments: &arguments,
        ..*call
    });
    let runtime_origin = argument_namespace
        .as_ref()
        .map_or_else(|| origin.clone(), host_namespaces::StateFor);
    assert!(
        Rc::ptr_eq(
            &origin.borrow().namespaces,
            &runtime_origin.borrow().namespaces
        ),
        "same Context argument namespace"
    );
    let global = origin.borrow().global_host.clone();
    if argument_namespace.is_some()
        || !Rc::ptr_eq(&host, &global)
        || !host_namespaces::IsWindow(&origin)
    {
        for value in &mut arguments {
            host_namespaces::Localize(value, &runtime_origin);
        }
    }
    let local = HostCall {
        arguments: &arguments,
        ..*call
    };
    let _namespace = host_namespaces::Enter(&runtime_origin);
    let continuation = host.borrow_mut().PrepareInvocation(&local);
    let mut runtime = HostRuntime {
        context: ctx,
        state: &runtime_origin,
    };
    let mut answer = if let Some(run) = continuation {
        run(&mut runtime)
    } else {
        host.borrow_mut().InvokeWithRuntime(&local, &mut runtime)
    };
    if argument_namespace.is_some() || !Rc::ptr_eq(&host, &global) {
        let namespace = JavaScriptHostObjectNamespace::new(runtime_origin);
        answer.value = namespace.Wrap(answer.value);
        answer.thrown = answer.thrown.map(|value| namespace.Wrap(value));
    } else {
        host_namespaces::RecordGlobalResult(state, &answer.value);
    }
    answer
}

fn host_result(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    result: HostResult,
) -> JsResult<Value> {
    let _host_roots = SynchronizeHostGraph(ctx, state)?;
    if let Some(thrown) = result.thrown {
        return Err(to_js_value(ctx, state, &thrown)?);
    }
    if let Some(e) = result.exception {
        Err(error(ctx, e))
    } else {
        to_js_value(ctx, state, &result.value)
    }
}

impl HostCallable for HostFunction {
    fn call(
        &self,
        ctx: &mut Context,
        this: &Value,
        args: &[Value],
        data: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        let _boundary = bridge_profile::enter(bridge_profile::Phase::Call);
        let state = &self.state;
        let _namespace = host_namespaces::Enter(state);
        match &self.action {
            Action::Callback(method, capture_receiver) => {
                let receiver_origin = this
                    .as_object()
                    .and_then(|object| host_namespaces::WrapperOrigin(state, object));
                let data = if *capture_receiver { &data[1..] } else { data };
                let values = {
                    let _conversion = bridge_profile::enter(bridge_profile::Phase::Arguments);
                    let mut values = Vec::with_capacity(1 + data.len() + args.len());
                    values.push(from_js_argument(this, ctx, state, false));
                    values.extend(
                        data.iter()
                            .chain(args)
                            .map(|value| from_js_argument(value, ctx, state, false)),
                    );
                    values
                };
                let answer = invoke(
                    ctx,
                    state,
                    &HostCall {
                        receiver: method.receiver,
                        operation: HostOperation::kCall,
                        member: &method.name,
                        symbol: HostSymbol::kNone,
                        arguments: &[],
                    },
                    values,
                    receiver_origin,
                );
                host_result(ctx, state, answer)
            }
            Action::Method(method) => {
                let values: Vec<_> = {
                    let _conversion = bridge_profile::enter(bridge_profile::Phase::Arguments);
                    host_arguments(ctx, state, method, args)
                };
                let answer = invoke(
                    ctx,
                    state,
                    &HostCall {
                        receiver: method.receiver,
                        operation: HostOperation::kCall,
                        member: &method.name,
                        symbol: HostSymbol::kNone,
                        arguments: &[],
                    },
                    values,
                    None,
                );
                host_result(ctx, state, answer)
            }
            Action::GlobalGet(name) => {
                let answer = invoke(
                    ctx,
                    state,
                    &HostCall {
                        receiver: 0,
                        operation: HostOperation::kGet,
                        member: name,
                        symbol: HostSymbol::kNone,
                        arguments: &[],
                    },
                    Vec::new(),
                    None,
                );
                if answer.handled || answer.exception.is_some() {
                    host_result(ctx, state, answer)
                } else {
                    let _host_roots = SynchronizeHostGraph(ctx, state)?;
                    Ok(state
                        .borrow()
                        .fallbacks
                        .get(name)
                        .cloned()
                        .unwrap_or(Value::Undefined))
                }
            }
            Action::GlobalSet(name) => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                let host_value = {
                    let _conversion = bridge_profile::enter(bridge_profile::Phase::Arguments);
                    from_js_argument(&value, ctx, state, false)
                };
                let answer = invoke(
                    ctx,
                    state,
                    &HostCall {
                        receiver: 0,
                        operation: HostOperation::kSet,
                        member: name,
                        symbol: HostSymbol::kNone,
                        arguments: &[],
                    },
                    vec![host_value],
                    None,
                );
                let _host_roots = SynchronizeHostGraph(ctx, state)?;
                if let Some(thrown) = answer.thrown {
                    return Err(to_js_value(ctx, state, &thrown)?);
                }
                if let Some(e) = answer.exception {
                    return Err(error(ctx, e));
                }
                if !answer.handled {
                    state.borrow_mut().fallbacks.insert(name.clone(), value);
                }
                Ok(Value::Undefined)
            }
            Action::Get(id) | Action::Set(id) | Action::Has(id) => {
                let mut _host_roots = Vec::new();
                let target = args.first().cloned().unwrap_or(Value::Undefined);
                let key = args.get(1).cloned().unwrap_or(Value::Undefined);
                let set = matches!(self.action, Action::Set(_));
                let has = matches!(self.action, Action::Has(_));
                let value = args.get(2).cloned().unwrap_or(Value::Undefined);
                let receiver = args
                    .get(if set { 3 } else { 2 })
                    .cloned()
                    .unwrap_or(Value::Undefined);
                if let Some((name, symbol)) = key_name(ctx, &key)? {
                    let set_value = set.then(|| {
                        let _conversion = bridge_profile::enter(bridge_profile::Phase::Arguments);
                        from_js_argument(&value, ctx, state, false)
                    });
                    let values = set_value.into_iter().collect();
                    let answer = invoke(
                        ctx,
                        state,
                        &HostCall {
                            receiver: *id,
                            operation: if set {
                                HostOperation::kSet
                            } else {
                                HostOperation::kGet
                            },
                            member: &name,
                            symbol,
                            arguments: &[],
                        },
                        values,
                        None,
                    );
                    _host_roots.extend(SynchronizeHostGraph(ctx, state)?);
                    if let Some(thrown) = answer.thrown {
                        return Err(to_js_value(ctx, state, &thrown)?);
                    }
                    if let Some(e) = answer.exception {
                        return Err(error(ctx, e));
                    }
                    if answer.handled {
                        return if has || set {
                            Ok(Value::Bool(true))
                        } else {
                            to_js_value(ctx, state, &answer.value)
                        };
                    }
                }
                if has {
                    let property = ctx.to_property_key(&key)?;
                    return Ok(Value::Bool(ctx.has_property(
                        target.as_object().expect("proxy target"),
                        &property,
                    )?));
                }
                let function = {
                    let s = state.borrow();
                    if set {
                        s.reflect_set.clone()
                    } else {
                        s.reflect_get.clone()
                    }
                };
                if set {
                    ctx.call(function, Value::Undefined, &[target, key, value, receiver])
                } else {
                    ctx.call(function, Value::Undefined, &[target, key, receiver])
                }
            }
        }
    }
}

fn function(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    action: Action,
    name: &str,
    length: u32,
) -> Value {
    Value::Object(ctx.new_host_function(
        Rc::new(HostFunction {
            state: state.clone(),
            action,
        }),
        Vec::new(),
        name,
        length,
    ))
}

fn install_prototype(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    id: HostObjectId,
) -> JsResult<()> {
    let entry = state
        .borrow()
        .wrappers
        .get(&id)
        .and_then(|w| w.target.get().map(|target| (target, w.prototype_installed)));
    let Some((target, false)) = entry else {
        return Ok(());
    };
    let host = host_namespaces::ObjectHost(state, id);
    let answer = host.borrow_mut().PrototypeFor(id);
    if answer.handled && answer.exception.is_none() {
        let value = to_js(ctx, state, &answer.value)?;
        if value.is_null() || value.is_object() {
            if ctx.set_prototype_of(&target, value.as_object().cloned())? {
                if let Some(w) = state.borrow_mut().wrappers.get_mut(&id) {
                    w.prototype_installed = true;
                }
            }
        }
    }
    Ok(())
}

fn to_js(ctx: &mut Context, state: &Rc<RefCell<State>>, value: &HostValue) -> JsResult<Value> {
    let state = host_namespaces::Active(state);
    let state = &state;
    let _host_roots = SynchronizeHostGraph(ctx, state)?;
    to_js_value(ctx, state, value)
}

// The caller holds the synchronized host roots across the whole conversion.
// Recursive records and callback arguments must not poll the host per object.
fn to_js_value(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    value: &HostValue,
) -> JsResult<Value> {
    let _conversion = bridge_profile::enter(bridge_profile::Phase::Return);
    if let HostValue::Retained(owner, _) = value {
        if owner
            .downcast_ref::<JavaScriptNativeObjectReference>()
            .is_some_and(|reference| reference.Namespace().Upgrade().is_none())
        {
            return Err(ctx.type_error("Native object belongs to a destroyed Context"));
        }
        if owner
            .downcast_ref::<WeakJavaScriptHostObjectNamespace>()
            .is_some_and(|namespace| namespace.Upgrade().is_none())
        {
            return Err(ctx.type_error("Native object belongs to a destroyed Context"));
        }
    }
    if matches!(value, HostValue::Retained(owner, _) if owner.is::<HostObjectOwnerIdentity>())
        && host_namespaces::TaggedIn(state, value).is_none()
    {
        return Err(ctx.type_error("Native object owner does not belong to this Context"));
    }
    if let Some((origin, value)) = host_namespaces::TaggedIn(state, value) {
        if origin.borrow().agent_identity != ctx.AgentIdentity() {
            return Err(ctx.type_error("Native object namespace belongs to another Agent"));
        }
        let _namespace = host_namespaces::Enter(&origin);
        let roots = SynchronizeHostGraph(ctx, &origin)?;
        let value = to_js_value(ctx, &origin, value);
        drop(roots);
        return value;
    }
    if let HostValue::Object(object) = value {
        let origin = host_namespaces::ObjectState(state, object.id);
        if !Rc::ptr_eq(&origin, state) {
            let _namespace = host_namespaces::Enter(&origin);
            return to_js_value(ctx, &origin, value);
        }
    }
    let value = match value {
        HostValue::Undefined(_) => Value::Undefined,
        HostValue::Null(_) => Value::Null,
        HostValue::Boolean(v) => Value::Bool(*v),
        HostValue::Number(v) => Value::number(*v),
        HostValue::String(v) => Value::Str(ctx.intern(v)),
        HostValue::String16(v) => Value::Str(ctx.intern_utf16(v)),
        HostValue::Bytes(v) => ctx.new_array_buffer_copy(v)?,
        HostValue::Retained(_, value) => to_js_value(ctx, state, value)?,
        HostValue::Array(items) => {
            let array = ctx.try_new_array()?;
            for (index, item) in items.iter().enumerate() {
                let item = to_js_value(ctx, state, item)?;
                ctx.define_data_value(&array, &index.to_string(), item, PropFlags::C_W_E)?;
            }
            Value::Object(array)
        }
        HostValue::Record(fields) => {
            let object = ctx.try_new_object()?;
            for (name, value) in fields {
                let value = to_js_value(ctx, state, value)?;
                ctx.define_data_value(&object, name, value, PropFlags::C_W_E)?;
            }
            Value::Object(object)
        }
        HostValue::JavaScriptValue(v) => v
            .Implementation::<Value>()
            .cloned()
            .ok_or_else(|| ctx.type_error("Value belongs to another engine"))?,
        HostValue::JavaScriptFunction(v) => Value::Object(
            v.Implementation::<JsObject>()
                .cloned()
                .ok_or_else(|| ctx.type_error("Function belongs to another engine"))?,
        ),
        HostValue::Method(m) => {
            // A bound host method retains its receiver through engine-marked
            // captures, even if the caller stores the method separately.
            let capture = state
                .borrow()
                .wrappers
                .get(&m.receiver)
                .and_then(|wrapper| wrapper.object.get());
            Value::Object(
                ctx.new_host_function(
                    Rc::new(HostFunction {
                        state: state.clone(),
                        action: Action::Method(m.clone()),
                    }),
                    capture
                        .map(|object| vec![Value::Object(object)])
                        .unwrap_or_default(),
                    &m.name,
                    0,
                ),
            )
        }
        HostValue::Object(o) if o.id == 0 => state.borrow().global.clone(),
        HostValue::Object(o) => {
            let existing = state
                .borrow()
                .wrappers
                .get(&o.id)
                .and_then(|w| w.object.get());
            if let Some(object) = existing {
                install_prototype(ctx, state, o.id)?;
                return Ok(Value::Object(object));
            }
            let managed = state.borrow().host_graph.Manages(o.id);
            // A host cell is already an ordinary traced object. Surface it
            // directly as the wrapper target instead of creating a second
            // object and a cycle for every Node (DOMDataStore inline storage).
            let mut cells = Vec::new();
            let target = if managed {
                state.borrow_mut().host_graph.Cell(ctx, o.id, &mut cells)?
            } else {
                ctx.new_object()
            };
            let host = host_namespaces::ObjectHost(state, o.id);
            let ordinary = host.borrow().UsesOrdinaryProperties(o.id);
            let object = if ordinary {
                // Ordinary host wrappers resolve installed/author descriptors
                // directly in the engine, as Blink's V8 instance templates do.
                // Identity and the host interface brand remain in the same
                // realm-local registry; neither depends on the prototype.
                target.clone()
            } else {
                let handler = ctx.new_object();
                for (name, action, length) in [
                    ("get", Action::Get(o.id), 3),
                    ("set", Action::Set(o.id), 4),
                    ("has", Action::Has(o.id), 2),
                ] {
                    let callback = function(ctx, state, action, name, length);
                    ctx.define_value(&handler, name, callback, PropFlags::C_W_E);
                }
                ctx.new_proxy(target.clone(), handler)?
            };
            {
                let mut state = state.borrow_mut();
                if let Some(old) = state.wrappers.remove(&o.id) {
                    state
                        .namespaces
                        .RemoveWrapperIdentity(old.object.identity());
                    state.wrapper_ids.remove(&old.object.identity());
                }
                state.wrapper_ids.insert(object.identity(), o.id);
                state.wrappers.insert(
                    o.id,
                    Wrapper {
                        target: if managed {
                            WrapperSlot::Weak(WeakObject::new(&target))
                        } else {
                            WrapperSlot::Strong(target.clone())
                        },
                        object: if managed {
                            WrapperSlot::Weak(WeakObject::new(&object))
                        } else {
                            WrapperSlot::Strong(object.clone())
                        },
                        prototype_installed: false,
                    },
                );
            }
            host_namespaces::RegisterWrapper(state, &object, o.id);
            if managed {
                let attached = host_graph::AttachWrapper(ctx, &object, &target);
                if let Err(error) = attached {
                    let mut state = state.borrow_mut();
                    if let Some(old) = state.wrappers.remove(&o.id) {
                        state
                            .namespaces
                            .RemoveWrapperIdentity(old.object.identity());
                        state.wrapper_ids.remove(&old.object.identity());
                    }
                    return Err(error);
                }
            }
            install_prototype(ctx, state, o.id)?;
            Value::Object(object)
        }
    };
    ctx.validate_value(&value)?;
    Ok(value)
}

struct Loader {
    host: Option<Rc<dyn JavaScriptModuleResolver>>,
    sources: HashMap<String, String>,
    resolutions: HashMap<(String, String), String>,
    compiled: HashMap<String, Value>,
}
impl ModuleLoader for Loader {
    fn resolve(&mut self, base: &str, specifier: &str) -> Result<String, String> {
        let key = (base.to_owned(), specifier.to_owned());
        if let Some(url) = self.resolutions.get(&key) {
            return Ok(url.clone());
        }
        let source = self
            .host
            .as_ref()
            .ok_or_else(|| format!("Unresolved module import: {specifier}"))?
            .ResolveModule(specifier, base)
            .map_err(|e| format!("Failed to load module {specifier} from {base}: {e}"))?
            .ok_or_else(|| format!("Unresolved module import: {specifier} from {base}"))?;
        if let Some(module) = source.module {
            let value = module
                .Implementation::<Value>()
                .cloned()
                .ok_or_else(|| "Module belongs to another engine".to_owned())?;
            self.compiled.insert(source.url.clone(), value);
        }
        self.sources.insert(source.url.clone(), source.source);
        self.resolutions.insert(key, source.url.clone());
        Ok(source.url)
    }
    fn load(&mut self, specifier: &str) -> Result<String, String> {
        self.sources
            .get(specifier)
            .cloned()
            .ok_or_else(|| format!("Unresolved module import: {specifier}"))
    }
    fn compiled(&self, specifier: &str) -> Option<Value> {
        self.compiled.get(specifier).cloned()
    }
    fn request_dynamic(&mut self, base: &str, specifier: &str) -> Result<Option<String>, String> {
        self.host
            .as_ref()
            .ok_or_else(|| format!("Unresolved dynamic module import: {specifier}"))?
            .RequestDynamicModule(specifier, base)
            .map_err(|error| error.to_string())
    }
    fn poll_dynamic(&mut self, url: &str) -> Result<Option<()>, String> {
        let Some(source) = self
            .host
            .as_ref()
            .ok_or_else(|| "No dynamic module resolver".to_owned())?
            .PollDynamicModule(url)
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        if let Some(module) = source.module {
            let value = module
                .Implementation::<Value>()
                .cloned()
                .ok_or_else(|| "Module belongs to another engine".to_owned())?;
            self.compiled.insert(source.url.clone(), value);
        }
        self.sources.insert(source.url.clone(), source.source);
        Ok(Some(()))
    }
}

fn create_realm(
    mut context: Context,
    host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    realms: &Rc<RefCell<Vec<JavaScriptRealm>>>,
    shared_hooks: Option<Rc<Hooks>>,
) -> JavaScriptRealm {
    let profiling = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
    let realm_started = profiling.then(std::time::Instant::now);
    if let Some(start) = realm_started {
        eprintln!(
            "javascript-realm-profile phase=context ms={:.3}",
            start.elapsed().as_secs_f64() * 1000.
        );
    }
    let services_started = profiling.then(std::time::Instant::now);
    host.borrow_mut()
        .BindTaskAttributionTracker(context.task_attribution_tracker());
    let hooks = shared_hooks.unwrap_or_else(|| {
        Rc::new(Hooks {
            task_tracker: Some(context.task_attribution_tracker()),
            ..Default::default()
        })
    });
    context.set_rejection_tracker(Some(Box::new(Tracker(hooks.clone()))));
    intl::install(&mut context).expect("ICU Locale installation");
    console::install(&mut context).expect("console installation");
    regexp::install(&mut context).expect("RegExp legacy state installation");
    if let Some(start) = services_started {
        eprintln!(
            "javascript-realm-profile phase=services ms={:.3}",
            start.elapsed().as_secs_f64() * 1000.
        );
    }
    let bindings_started = profiling.then(std::time::Instant::now);
    let reflect = context.global_value("Reflect");
    let get_key = PropKey::from_string(&context.intern("get"));
    let set_key = PropKey::from_string(&context.intern("set"));
    let reflect_get = context
        .get_property(&reflect, &get_key)
        .expect("Reflect.get");
    let reflect_set = context
        .get_property(&reflect, &set_key)
        .expect("Reflect.set");
    let global = context.global();
    let state = Rc::new(RefCell::new(State {
        host: host.clone(),
        realms: Rc::downgrade(realms),
        agent_identity: context.AgentIdentity(),
        document_object: host.borrow().DocumentHostObjectId(),
        object_host: None,
        global_host: host.clone(),
        namespaces: Rc::new(host_namespaces::NamespaceRegistry::WithWrappers(
            hooks.wrappers.clone(),
        )),
        global: Value::Object(global.clone()),
        wrappers: HashMap::new(),
        host_graph: HostGraph::new(host.borrow().ObjectTraceSource()),
        wrapper_ids: HashMap::new(),
        fallbacks: HashMap::new(),
        reflect_get,
        reflect_set,
        hooks,
    }));
    host_namespaces::Initialize(&state);
    let names = host.borrow().GlobalNames();
    for name in names {
        let answer = host.borrow_mut().Invoke(&HostCall {
            receiver: 0,
            operation: HostOperation::kGet,
            member: &name,
            symbol: HostSymbol::kNone,
            arguments: &[],
        });
        host_namespaces::RecordGlobalResult(&state, &answer.value);
        if answer.exception.is_none() {
            if let Ok(value) = to_js(&mut context, &state, &answer.value) {
                state.borrow_mut().fallbacks.insert(name.clone(), value);
            }
        }
        let getter = function(
            &mut context,
            &state,
            Action::GlobalGet(name.clone()),
            &format!("get {name}"),
            0,
        );
        let setter = function(
            &mut context,
            &state,
            Action::GlobalSet(name.clone()),
            &format!("set {name}"),
            1,
        );
        let key = PropKey::from_string(&context.intern(&name));
        context.define_own(
            &global,
            key,
            Property::accessor(
                Some(getter),
                Some(setter),
                PropFlags::CONFIGURABLE.union(PropFlags::ENUMERABLE),
            ),
        );
    }
    if let Some(start) = bindings_started {
        eprintln!(
            "javascript-realm-profile phase=host-bindings ms={:.3}",
            start.elapsed().as_secs_f64() * 1000.
        );
    }
    context.defer_cycle_collection();
    let native_microtasks = native_microtasks::NativeMicrotasks::new(&context, &state);
    let data = Rc::new(RealmRuntimeData {
        context: context.Downgrade(),
        state: Rc::downgrade(&state),
    });
    let realm =
        JavaScriptRealm::WithNativeMicrotasks(QuickJsRealm { context, state }, native_microtasks)
            .WithHostRuntimeData(data);
    realms.borrow_mut().push(realm.clone());
    realm
}

impl JavaScriptRuntime for QuickJsJavaScriptRuntime {
    fn SetScriptExecutionPaused(
        &mut self,
        realm: &JavaScriptRealm,
        paused: bool,
    ) -> Result<(), JavaScriptException> {
        if !self.realms.borrow().contains(realm) {
            return Err(JavaScriptException {
                message: "foreign runtime realm".into(),
                ..Default::default()
            });
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|realm| {
                realm.context.callback_context().SetPaused(paused)
            })
            .ok_or_else(|| JavaScriptException {
                message: "callback context was destroyed".into(),
                ..Default::default()
            })
    }
    fn SelectDocumentHostObjectNamespace(
        &mut self,
        realm: &JavaScriptRealm,
        namespace: &JavaScriptHostObjectNamespace,
    ) {
        assert!(self.realms.borrow().contains(realm), "same runtime realm");
        realm
            .WithImplementation::<QuickJsRealm, _>(|realm| {
                host_namespaces::SelectDocument(&realm.state, namespace);
            })
            .expect("QuickJS realm");
    }
    fn CreateHostObjectNamespace(
        &mut self,
        realm: &JavaScriptRealm,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
        initial: bool,
    ) -> Result<JavaScriptHostObjectNamespace, JavaScriptException> {
        if !self.realms.borrow().contains(realm) {
            return Err(JavaScriptException {
                message: "foreign realm host namespace".into(),
                ..Default::default()
            });
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|realm| {
                host.borrow_mut().BindTaskAttributionTracker(
                    realm
                        .state
                        .borrow()
                        .hooks
                        .task_tracker
                        .clone()
                        .expect("real VM task attribution tracker"),
                );
                host_namespaces::Create(&realm.state, host, initial)
            })
            .expect("QuickJS realm identity validated")
    }
    fn WithHostRuntime(
        &mut self,
        realm: &JavaScriptRealm,
        run: &mut dyn FnMut(&mut dyn JavaScriptHostRuntime),
    ) -> Result<(), JavaScriptException> {
        if !self.realms.borrow().contains(realm) {
            return Err(JavaScriptException {
                message: "realm belongs to another runtime".into(),
                ..JavaScriptException::default()
            });
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let mut runtime = HostRuntime {
                    context: &mut r.context,
                    state: &r.state,
                };
                run(&mut runtime);
            })
            .ok_or_else(|| JavaScriptException {
                message: "invalid QuickJS realm".into(),
                ..JavaScriptException::default()
            })
    }

    fn GarbageCollectionRequested(&self) -> bool {
        self.realms.borrow().iter().any(|realm| {
            realm
                .WithImplementation::<QuickJsRealm, _>(|realm| {
                    realm.context.cycle_collection_requested()
                })
                .unwrap_or(false)
        })
    }

    fn CollectGarbage(&mut self) {
        for realm in self.realms.borrow().iter() {
            realm.WithImplementation::<QuickJsRealm, _>(|realm| {
                realm.context.collect_cycles();
            });
        }
    }

    fn MemoryDiagnostics(&self, realm: &JavaScriptRealm) -> Option<JavaScriptMemoryDiagnostics> {
        realm.WithImplementation::<QuickJsRealm, _>(|realm| {
            let usage = realm.context.memory_usage();
            let state = realm.state.borrow();
            JavaScriptMemoryDiagnostics {
                malloc_bytes: usage.malloc_size,
                used_bytes: usage.memory_used_size,
                objects: usage.obj_count,
                strings: usage.str_count,
                string_bytes: usage.str_size,
                atoms: usage.atom_count,
                properties: usage.prop_count,
                property_bytes: usage.prop_size,
                shapes: usage.shape_count,
                function_bytes: usage.js_func_size,
                code_bytes: usage.js_func_code_size,
                array_elements: usage.fast_array_elements,
                binary_bytes: usage.binary_object_size,
                host_wrappers: state.wrappers.len(),
                host_identities: state.wrapper_ids.len(),
                host_fallbacks: state.fallbacks.len(),
            }
        })
    }
    fn CreateRealm(&mut self, host: Rc<RefCell<dyn JavaScriptHostBindings>>) -> JavaScriptRealm {
        create_realm(
            Context::with_max_stack_size(self.stack_budget),
            host,
            &self.realms,
            None,
        )
    }

    fn PrepareClassicScript(&mut self, realm: &JavaScriptRealm, source: &str, name: &str) -> bool {
        if !self.realms.borrow().contains(realm)
            || self.module_worker_unavailable
            || self.compilation_executor.is_none()
        {
            return true;
        }
        if let Some(job) = self.prepared_classic_scripts.iter_mut().find(|job| {
            &job.realm == realm && job.worker.name == name && job.worker.source.as_str() == source
        }) {
            if job.outcome.is_none() {
                job.outcome = job.worker.poll();
            }
            return job.outcome.is_some();
        }
        if self.module_worker.is_none() {
            let mut worker = module_worker::Worker::with_executor(
                self.compilation_executor.as_ref().unwrap().clone(),
            );
            worker.set_wake(self.module_compilation_wake.clone());
            self.module_worker = Some(worker);
        }
        let worker =
            self.module_worker
                .as_ref()
                .unwrap()
                .begin_classic(source, name, self.stack_budget);
        self.prepared_classic_scripts.push(PreparedClassicScript {
            realm: realm.clone(),
            worker,
            outcome: None,
        });
        false
    }
    fn Evaluate(&mut self, realm: &JavaScriptRealm, source: &str, name: &str) -> JavaScriptResult {
        let mut trace = browser_tracing::span("javascript", "ScriptEvaluate");
        trace.set("source_bytes", source.len() as f64);
        if !self.realms.borrow().contains(realm) {
            return JavaScriptResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                "realm belongs to another runtime",
                "",
                0,
                0,
            );
        }
        let prepared = self
            .prepared_classic_scripts
            .iter()
            .position(|job| {
                &job.realm == realm
                    && job.worker.name == name
                    && job.worker.source.as_str() == source
            })
            .map(|index| self.prepared_classic_scripts.swap_remove(index));
        let compiled = prepared.and_then(|job| job.outcome);
        trace.set(
            "background_compiled",
            matches!(&compiled, Some(module_worker::Outcome::Compiled { .. })) as u8 as f64,
        );
        let boundary = bridge_profile::Measurement::start();
        let answer = realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let _host_roots = match SynchronizeHostGraph(&mut r.context, &r.state) {
                    Ok(roots) => roots,
                    Err(error) => return result(&mut r.context, Err(error), name),
                };
                let value = if let Some(module_worker::Outcome::Compiled { bytes, metadata }) =
                    compiled
                {
                    let mut hydrate = browser_tracing::span("javascript", "ClassicScriptHydrate");
                    hydrate.set("bytecode_bytes", bytes.len() as f64);
                    let script = r.context.hydrate_classic_script(&bytes, &metadata);
                    drop(hydrate);
                    match script {
                        Ok(script) => r.context.evaluate_classic_script(&script),
                        // Hydration executes no page code. Preserve the owner
                        // compiler's error semantics when importing fails.
                        Err(_) => r.context.eval_named(source, SourceType::Script, name),
                    }
                } else {
                    r.context.eval_named(source, SourceType::Script, name)
                };
                result(&mut r.context, value, name)
            })
            .expect("invalid QuickJS realm");
        bridge_profile::Measurement::finish(boundary, &mut trace);
        answer
    }

    fn EvaluateBootstrap(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        name: &str,
    ) -> JavaScriptResult {
        let mut trace = browser_tracing::span("javascript", "BootstrapEvaluate");
        trace.set("source_bytes", source.len() as f64);
        if !self.realms.borrow().contains(realm) {
            return JavaScriptResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                "realm belongs to another runtime",
                "",
                0,
                0,
            );
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let _host_roots = match SynchronizeHostGraph(&mut r.context, &r.state) {
                    Ok(roots) => roots,
                    Err(error) => return result(&mut r.context, Err(error), name),
                };
                let value = r.context.eval_bootstrap_named(source, name);
                result(&mut r.context, value, name)
            })
            .expect("invalid QuickJS realm")
    }

    fn SupportsModules(&self) -> bool {
        true
    }
    fn CompileModule(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        name: &str,
    ) -> Result<JavaScriptModuleCompilation, JavaScriptException> {
        if !self.realms.borrow().contains(realm) {
            return Err(JavaScriptResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                "realm belongs to another runtime",
                "",
                0,
                0,
            )
            .exception
            .unwrap());
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let ctx = &mut r.context;
                let compiled = ctx
                    .compile_module_unregistered(name, source)
                    .and_then(|module| {
                        ctx.module_requests(&module)
                            .map(|requests| JavaScriptModuleCompilation {
                                module: JavaScriptValue::new_traced(module),
                                requests,
                            })
                    });
                compiled.map_err(|error| result(ctx, Err(error), name).exception.unwrap())
            })
            .expect("invalid QuickJS realm")
    }
    fn BeginCompileModule(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        name: &str,
    ) -> Option<JavaScriptModuleCompilationJob> {
        if !self.realms.borrow().contains(realm)
            || self.module_worker_unavailable
            || self.compilation_executor.is_none()
        {
            return None;
        }
        if self.module_worker.is_none() {
            let mut worker = module_worker::Worker::with_executor(
                self.compilation_executor.as_ref().unwrap().clone(),
            );
            worker.set_wake(self.module_compilation_wake.clone());
            self.module_worker = Some(worker);
        }
        let worker = self
            .module_worker
            .as_ref()
            .unwrap()
            .begin(source, name, self.stack_budget);
        Some(JavaScriptModuleCompilationJob::new(
            QuickJsModuleCompilationJob {
                realm: realm.clone(),
                worker,
                completed: false,
            },
        ))
    }
    fn PollCompileModule(
        &mut self,
        realm: &JavaScriptRealm,
        job: &mut JavaScriptModuleCompilationJob,
    ) -> JavaScriptModuleCompilationPoll {
        let failure = |message: &str| {
            JavaScriptModuleCompilationPoll::Ready(Err(JavaScriptResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                message,
                "",
                0,
                0,
            )
            .exception
            .unwrap()))
        };
        if !self.realms.borrow().contains(realm) {
            return failure("realm belongs to another runtime");
        }
        let Some(job) = job.ImplementationMut::<QuickJsModuleCompilationJob>() else {
            return failure("module compilation job belongs to another engine");
        };
        if &job.realm != realm {
            return failure("module compilation job belongs to another realm");
        }
        if job.completed {
            return failure("module compilation job was already consumed");
        }
        let Some(outcome) = job.worker.poll() else {
            return JavaScriptModuleCompilationPoll::Pending;
        };
        job.completed = true;
        let compiled = match outcome {
            module_worker::Outcome::MainFallback => {
                // Error formatting observes this realm's Error prototype and
                // getters. A worker error must run the original owner path.
                self.CompileModule(realm, &job.worker.source, &job.worker.name)
            }
            module_worker::Outcome::Compiled { bytes, metadata } => realm
                .WithImplementation::<QuickJsRealm, _>(|r| {
                    let ctx = &mut r.context;
                    let compiled = ctx
                        .hydrate_module_unregistered(&job.worker.name, &bytes, &metadata)
                        .and_then(|module| {
                            ctx.module_requests(&module).map(|requests| {
                                JavaScriptModuleCompilation {
                                    module: JavaScriptValue::new_traced(module),
                                    requests,
                                }
                            })
                        });
                    compiled.map_err(|error| {
                        result(ctx, Err(error), &job.worker.name).exception.unwrap()
                    })
                })
                .expect("invalid QuickJS realm"),
        };
        JavaScriptModuleCompilationPoll::Ready(compiled)
    }
    fn SetCompilationExecutor(&mut self, executor: Option<std::sync::Arc<dyn executor::Executor>>) {
        assert!(
            self.module_worker.is_none(),
            "configure compilation before submitting work"
        );
        self.compilation_executor = executor;
    }
    fn SetModuleCompilationWake(&mut self, wake: Option<JavaScriptModuleCompilationWake>) {
        self.module_compilation_wake = wake.clone();
        if let Some(worker) = &mut self.module_worker {
            worker.set_wake(wake);
        }
    }
    fn EvaluateCompiledModule(
        &mut self,
        realm: &JavaScriptRealm,
        module: &JavaScriptValue,
        name: &str,
        resolver: Option<Rc<dyn JavaScriptModuleResolver>>,
    ) -> JavaScriptResult {
        let _trace = browser_tracing::span("javascript", "CompiledModuleEvaluate");
        if !self.realms.borrow().contains(realm) {
            return JavaScriptResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                "realm belongs to another runtime",
                "",
                0,
                0,
            );
        }
        let Some(module) = module.Implementation::<Value>().cloned() else {
            return JavaScriptResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "Module belongs to another engine",
                name,
                0,
                0,
            );
        };
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let _host_roots = match SynchronizeHostGraph(&mut r.context, &r.state) {
                    Ok(roots) => roots,
                    Err(error) => return result(&mut r.context, Err(error), name),
                };
                let ctx = &mut r.context;
                let value = ctx.with_module_loader(
                    Box::new(Loader {
                        host: resolver,
                        sources: HashMap::new(),
                        resolutions: HashMap::new(),
                        compiled: HashMap::new(),
                    }),
                    |ctx| {
                        let prepared = ctx.prepare_module_graph(name, &module)?;
                        ctx.resolve_module(&module)?;
                        let value = ctx.evaluate_module(&module)?;
                        ctx.mark_module_graph_prepared(prepared);
                        Ok(value)
                    },
                );
                result(ctx, value, name)
            })
            .expect("invalid QuickJS realm")
    }
    fn EvaluateModule(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        name: &str,
        loader: Option<Rc<dyn JavaScriptModuleResolver>>,
    ) -> JavaScriptResult {
        let mut trace = browser_tracing::span("javascript", "ModuleEvaluate");
        trace.set("source_bytes", source.len() as f64);
        if !self.realms.borrow().contains(realm) {
            return JavaScriptResult::Failure(
                JavaScriptExceptionKind::kRuntimeError,
                "realm belongs to another runtime",
                "",
                0,
                0,
            );
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let _host_roots = match SynchronizeHostGraph(&mut r.context, &r.state) {
                    Ok(roots) => roots,
                    Err(error) => return result(&mut r.context, Err(error), name),
                };
                let ctx = &mut r.context;
                let loader = Loader {
                    host: loader,
                    sources: HashMap::new(),
                    resolutions: HashMap::new(),
                    compiled: HashMap::new(),
                };
                let value = ctx.with_module_loader(Box::new(loader), |ctx| {
                    let module = match ctx.lookup_module(name) {
                        Some(module) => module,
                        None => ctx.compile_module(name, source)?,
                    };
                    let prepared = ctx.prepare_module_graph(name, &module)?;
                    ctx.resolve_module(&module)?;
                    let value = ctx.evaluate_module(&module)?;
                    ctx.mark_module_graph_prepared(prepared);
                    Ok(value)
                });
                result(ctx, value, name)
            })
            .expect("invalid QuickJS realm")
    }

    fn Call(
        &mut self,
        realm: &JavaScriptRealm,
        callback: &JavaScriptFunction,
        receiver: &HostValue,
        args: &[HostValue],
    ) -> JavaScriptResult {
        if !self.realms.borrow().contains(realm) {
            return JavaScriptResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "value is not callable in this realm",
                "",
                0,
                0,
            );
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                call(&mut r.context, &r.state, callback, receiver, args)
            })
            .expect("invalid QuickJS realm")
    }
    fn Clone(&mut self, realm: &JavaScriptRealm, value: &HostValue) -> HostResult {
        if !self.realms.borrow().contains(realm) {
            return HostResult::Failure(JavaScriptExceptionKind::kTypeError, "Invalid realm");
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| clone_value(&mut r.context, &r.state, value))
            .expect("invalid QuickJS realm")
    }
    fn EnqueueMicrotask(
        &mut self,
        realm: &JavaScriptRealm,
        callback: &JavaScriptFunction,
    ) -> HostResult {
        if !self.realms.borrow().contains(realm) {
            return HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "Invalid microtask callback",
            );
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| enqueue(&mut r.context, callback))
            .expect("invalid QuickJS realm")
    }
    fn PerformMicrotaskCheckpoint(&mut self) {
        // A microtask can synchronously initialize a newly inserted frame.
        // Keep the registry unborrowed while its existing Agent queue runs.
        let realms = self.realms.borrow().clone();
        for realm in &realms {
            realm
                .WithImplementation::<QuickJsRealm, _>(|r| checkpoint(&mut r.context, &r.state))
                .expect("invalid QuickJS realm");
        }
    }
    fn TakePendingExceptions(&mut self, realm: &JavaScriptRealm) -> Vec<JavaScriptException> {
        if !self.realms.borrow().contains(realm) {
            return Vec::new();
        }
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| take_exceptions(&mut r.context, &r.state))
            .unwrap_or_default()
    }
}

fn call(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    callback: &JavaScriptFunction,
    receiver: &HostValue,
    arguments: &[HostValue],
) -> JavaScriptResult {
    // Browser task owners enclose this callback and its later checkpoint in
    // their own task span. Reentrant event/observer calls remain nested here.
    let mut trace = browser_tracing::span("javascript", "Callback");
    trace.set("argc", arguments.len() as f64);
    let boundary = bridge_profile::Measurement::start();
    let value = (|| {
        let _host_roots = SynchronizeHostGraph(ctx, state)?;
        let function = callback
            .Implementation::<JsObject>()
            .ok_or_else(|| ctx.type_error("Invalid callback"))?;
        // Pin the host receiver before argument allocation can collect a
        // just-popped host reaction/event task's former root.
        let receiver = to_js_value(ctx, state, receiver)?;
        let args = arguments
            .iter()
            .map(|v| to_js_value(ctx, state, v))
            .collect::<JsResult<Vec<_>>>()?;
        ctx.call(Value::Object(function.clone()), receiver, &args)
    })();
    trace.set("failed", u8::from(value.is_err()) as f64);
    bridge_profile::Measurement::finish(boundary, &mut trace);
    result(ctx, value, "")
}

fn serialization_error(error: adapter::CloneFailure) -> crate::JavaScriptSerializationError {
    match error {
        adapter::CloneFailure::DataClone(message) => {
            crate::JavaScriptSerializationError::DataCloneError(message.into())
        }
        adapter::CloneFailure::Exception(value) => crate::JavaScriptSerializationError::Exception {
            // Keep the original engine value, including ArrayBuffer/host wrappers;
            // generic from_js may copy binary bytes and must not be used here.
            value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
            // Reporting metadata must not Get(stack)/ToString(author exception).
            exception: JavaScriptException::default(),
        },
    }
}
fn serialize_for_storage(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    value: &HostValue,
) -> Result<crate::SerializedScriptValue, crate::JavaScriptSerializationError> {
    let _roots =
        SynchronizeHostGraph(ctx, state).map_err(|value| serialization_error(value.into()))?;
    let value =
        to_js_value(ctx, state, value).map_err(|value| serialization_error(value.into()))?;
    ctx.serialize_for_storage(&value)
        .map_err(serialization_error)
}
fn deserialize_storage(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    storage: &crate::SerializedScriptValue,
) -> Result<HostValue, crate::JavaScriptSerializationError> {
    let _roots =
        SynchronizeHostGraph(ctx, state).map_err(|value| serialization_error(value.into()))?;
    let value = ctx
        .deserialize_storage(storage)
        .map_err(serialization_error)?;
    Ok(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
        value,
    )))
}

fn clone_value(ctx: &mut Context, state: &Rc<RefCell<State>>, value: &HostValue) -> HostResult {
    let answer = serialize_for_storage(ctx, state, value)
        .and_then(|storage| deserialize_storage(ctx, state, &storage));
    match answer {
        Ok(value) => HostResult {
            value,
            ..Default::default()
        },
        Err(crate::JavaScriptSerializationError::DataCloneError(message)) => {
            HostResult::Failure(JavaScriptExceptionKind::kTypeError, message)
        }
        Err(crate::JavaScriptSerializationError::Exception { value, exception }) => HostResult {
            thrown: Some(value),
            exception: Some(exception),
            ..Default::default()
        },
    }
}

fn enqueue(ctx: &mut Context, callback: &JavaScriptFunction) -> HostResult {
    let _trace = browser_tracing::span("javascript", "MicrotaskEnqueue");
    let Some(function) = callback.Implementation::<JsObject>() else {
        return HostResult::Failure(
            JavaScriptExceptionKind::kTypeError,
            "Invalid microtask callback",
        );
    };
    match ctx.enqueue_job(Value::Object(function.clone()), Vec::new()) {
        Ok(()) => HostResult::default(),
        Err(thrown) => HostResult {
            exception: Some(exception(ctx, &thrown, "")),
            ..Default::default()
        },
    }
}
fn checkpoint(ctx: &mut Context, state: &Rc<RefCell<State>>) {
    let hooks = state.borrow().hooks.clone();
    if hooks.checkpoint_suppression.get() != 0 {
        return;
    }
    let _host_roots = match SynchronizeHostGraph(ctx, state) {
        Ok(roots) => roots,
        Err(error) => {
            let reported = exception(ctx, &error, "");
            hooks.exceptions.borrow_mut().push(reported);
            return;
        }
    };
    if hooks.checkpoint_running.replace(true) {
        return;
    }
    let mut trace = browser_tracing::span("javascript", "MicrotaskCheckpoint");
    let boundary = bridge_profile::Measurement::start();
    let started = std::env::var_os("BROWSER_PROFILE_INPUT")
        .is_some()
        .then(std::time::Instant::now);
    for e in ctx.run_jobs_errors() {
        let location = ctx.error_creation_location(&e);
        let _ = ctx.take_exception_location();
        let mut e = exception_at(ctx, &e, "undefined", location);
        e.kind = JavaScriptExceptionKind::kRuntimeError;
        hooks.exceptions.borrow_mut().push(e);
    }
    hooks.checkpoint_running.set(false);
    bridge_profile::Measurement::finish(boundary, &mut trace);
    if let Some(started) = started {
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        if elapsed >= 1.0 {
            // Includes every job, the drain logs and exception conversion. The
            // existing caller/task timer includes this diagnostic log as well.
            eprintln!("javascript-microtask-checkpoint-profile total_ms={elapsed:.3}");
        }
    }
}
fn take_exceptions(ctx: &mut Context, state: &Rc<RefCell<State>>) -> Vec<JavaScriptException> {
    let hooks = state.borrow().hooks.clone();
    let mut output = std::mem::take(&mut *hooks.exceptions.borrow_mut());
    let rejections = std::mem::take(&mut *hooks.rejections.borrow_mut());
    for (_, reason) in rejections {
        let location = ctx.error_creation_location(&reason);
        let mut e = exception_at(ctx, &reason, "undefined", location);
        e.kind = JavaScriptExceptionKind::kRuntimeError;
        e.message = format!("Unhandled rejection: {}", e.message);
        output.push(e);
    }
    output
}

struct HostRuntime<'a> {
    context: &'a mut Context,
    state: &'a Rc<RefCell<State>>,
}
struct CheckpointScope(Rc<Hooks>);
impl JavaScriptMicrotaskCheckpointScope for CheckpointScope {
    fn IsOutermost(&self) -> bool {
        self.0.checkpoint_suppression.get() == 1
    }
}
impl Drop for CheckpointScope {
    fn drop(&mut self) {
        self.0.checkpoint_suppression.set(
            self.0
                .checkpoint_suppression
                .get()
                .checked_sub(1)
                .expect("active microtask scope"),
        );
    }
}
struct NativeFunction {
    state: std::rc::Weak<RefCell<State>>,
    callback: Rc<dyn JavaScriptNativeFunction>,
}
impl HostCallable for NativeFunction {
    fn call(
        &self,
        context: &mut Context,
        receiver: &Value,
        arguments: &[Value],
        _: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        let state = self
            .state
            .upgrade()
            .ok_or_else(|| context.type_error("Native function execution context was destroyed"))?;
        let _roots = SynchronizeHostGraph(context, &state)?;
        // Opaque VM handles preserve identity; from_js would copy ArrayBuffers.
        let receiver = HostValue::JavaScriptValue(JavaScriptValue::new_traced(receiver.clone()));
        let arguments: Vec<_> = arguments
            .iter()
            .map(|v| HostValue::JavaScriptValue(JavaScriptValue::new_traced(v.clone())))
            .collect();
        let result = self.callback.Call(
            &mut HostRuntime {
                context,
                state: &state,
            },
            &receiver,
            &arguments,
        );
        host_result(context, &state, result)
    }
}
impl JavaScriptHostRuntime for HostRuntime<'_> {
    fn CreateChildRealm(
        &mut self,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    ) -> Result<JavaScriptRealm, JavaScriptException> {
        let (realms, hooks) = {
            let state = self.state.borrow();
            (state.realms.upgrade(), state.hooks.clone())
        };
        let realms = realms.ok_or_else(|| JavaScriptException {
            message: "the Window Agent has been destroyed".into(),
            ..Default::default()
        })?;
        Ok(create_realm(
            self.context.new_child(),
            host,
            &realms,
            Some(hooks),
        ))
    }
    fn WithRealm(
        &mut self,
        realm: &JavaScriptRealm,
        run: &mut dyn FnMut(&mut dyn JavaScriptHostRuntime),
    ) -> Result<(), JavaScriptException> {
        let data =
            realm
                .HostRuntimeData::<RealmRuntimeData>()
                .ok_or_else(|| JavaScriptException {
                    message: "invalid child realm".into(),
                    ..Default::default()
                })?;
        let mut context = data.context.Upgrade().ok_or_else(|| JavaScriptException {
            message: "child realm has been destroyed".into(),
            ..Default::default()
        })?;
        if !context.same_agent(self.context) {
            return Err(JavaScriptException {
                message: "realm belongs to another Window Agent".into(),
                ..Default::default()
            });
        }
        let state = data.state.upgrade().ok_or_else(|| JavaScriptException {
            message: "child Window bindings have been destroyed".into(),
            ..Default::default()
        })?;
        run(&mut HostRuntime {
            context: &mut context,
            state: &state,
        });
        Ok(())
    }

    fn GlobalObject(&mut self) -> HostValue {
        HostValue::JavaScriptValue(JavaScriptValue::new_traced(
            self.state.borrow().global.clone(),
        ))
    }
    fn IsSameObject(&mut self, left: &HostValue, right: &HostValue) -> bool {
        match (
            to_js_value(self.context, self.state, left),
            to_js_value(self.context, self.state, right),
        ) {
            (Ok(Value::Object(left)), Ok(Value::Object(right))) => left == right,
            _ => false,
        }
    }
    fn CreateHostObjectNamespace(
        &mut self,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
        initial: bool,
    ) -> Result<JavaScriptHostObjectNamespace, JavaScriptException> {
        host.borrow_mut()
            .BindTaskAttributionTracker(self.context.task_attribution_tracker());
        host_namespaces::Create(self.state, host, initial)
    }
    fn SelectDocumentHostObjectNamespace(&mut self, namespace: &JavaScriptHostObjectNamespace) {
        host_namespaces::SelectDocument(self.state, namespace);
    }
    fn CallbackContext(&self) -> JavaScriptCallbackContext {
        self.context.callback_context()
    }
    fn IsExecutionTerminating(&self) -> bool {
        self.context.is_execution_terminating()
    }
    fn EnterHostObjectNamespace(
        &mut self,
        namespace: &JavaScriptHostObjectNamespace,
    ) -> Box<dyn JavaScriptHostObjectNamespaceScope> {
        let namespace = host_namespaces::StateFor(namespace);
        assert!(
            Rc::ptr_eq(
                &self.state.borrow().namespaces,
                &namespace.borrow().namespaces
            ),
            "same realm host namespace"
        );
        Box::new(host_namespaces::Enter(&namespace))
    }
    fn EvaluateClassicScript(&mut self, source: &[u16], base_url: &str) -> HostResult {
        let result = (|| {
            let _roots = SynchronizeHostGraph(self.context, self.state)?;
            self.context.eval_classic_utf16(source, base_url)
        })();
        match result {
            Ok(value) => HostResult {
                value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
                ..Default::default()
            },
            Err(value) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    value,
                ))),
                ..Default::default()
            },
        }
    }
    fn PrimitiveString(&mut self, value: &HostValue) -> Option<Rc<[u16]>> {
        match value {
            HostValue::String(value) => Some(value.encode_utf16().collect::<Vec<_>>().into()),
            HostValue::String16(value) => Some(value.clone()),
            HostValue::JavaScriptValue(value) => match value.Implementation::<Value>()? {
                Value::Str(value) => Some(value.to_utf16().into()),
                _ => None,
            },
            _ => None,
        }
    }
    fn IsObject(&mut self, value: &HostValue) -> bool {
        match value {
            HostValue::Object(_)
            | HostValue::JavaScriptFunction(_)
            | HostValue::Method(_)
            | HostValue::Bytes(_)
            | HostValue::Record(_) => true,
            HostValue::JavaScriptValue(value) => value
                .Implementation::<Value>()
                .is_some_and(|value| matches!(value, Value::Object(_))),
            HostValue::Retained(_, value) => self.IsObject(value),
            _ => false,
        }
    }
    fn IsNull(&mut self, value: &HostValue) -> bool {
        match value {
            HostValue::Null(_) => true,
            HostValue::JavaScriptValue(value) => value
                .Implementation::<Value>()
                .is_some_and(|value| matches!(value, Value::Null)),
            _ => false,
        }
    }
    fn IsCallable(&mut self, value: &HostValue) -> bool {
        match value {
            HostValue::JavaScriptFunction(_) | HostValue::Method(_) => true,
            HostValue::JavaScriptValue(value) => value
                .Implementation::<Value>()
                .is_some_and(|value| self.context.is_callable(value)),
            _ => false,
        }
    }
    fn GetProperty(&mut self, value: &HostValue, property: &str) -> HostResult {
        let result: Result<Value, Value> = (|| {
            let _roots = SynchronizeHostGraph(self.context, self.state)?;
            let value = to_js_value(self.context, self.state, value)?;
            let key = PropKey::Str(self.context.intern(property));
            self.context.get_property(&value, &key)
        })();
        match result {
            Ok(value) => HostResult {
                value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
                ..Default::default()
            },
            Err(value) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    value,
                ))),
                ..Default::default()
            },
        }
    }
    fn SetProperty(
        &mut self,
        value: &HostValue,
        property: &str,
        assigned: &HostValue,
    ) -> HostResult {
        let result = (|| {
            let _roots = SynchronizeHostGraph(self.context, self.state)?;
            let value = to_js_value(self.context, self.state, value)?;
            let assigned = to_js_value(self.context, self.state, assigned)?;
            let key = Value::Str(self.context.intern(property));
            let reflect_set = self.state.borrow().reflect_set.clone();
            self.context
                .call(reflect_set, Value::Undefined, &[value, key, assigned])
        })();
        match result {
            Ok(value) => HostResult {
                value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
                ..Default::default()
            },
            Err(value) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    value,
                ))),
                ..Default::default()
            },
        }
    }
    fn NewTypeError(&mut self, message: &[u16]) -> HostResult {
        let message = Value::Str(self.context.intern_utf16(message));
        match self
            .context
            .make_intrinsic_error_value("TypeError", message)
        {
            Ok(value) => HostResult {
                value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
                ..Default::default()
            },
            Err(value) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    value,
                ))),
                ..Default::default()
            },
        }
    }
    fn NewError(&mut self, kind: JavaScriptExceptionKind, message: &str) -> HostValue {
        let value = match kind {
            JavaScriptExceptionKind::kSyntaxError => {
                self.context.make_error(ErrorKind::Syntax, message)
            }
            JavaScriptExceptionKind::kTypeError => {
                self.context.make_error(ErrorKind::Type, message)
            }
            JavaScriptExceptionKind::kRangeError => {
                self.context.make_error(ErrorKind::Range, message)
            }
            JavaScriptExceptionKind::kRuntimeError => self.context.make_plain_error(message),
        };
        HostValue::JavaScriptValue(JavaScriptValue::new_traced(value))
    }
    fn CaptureExceptionStack(
        &mut self,
        value: &HostValue,
        name: &HostValue,
        message: &HostValue,
    ) -> HostResult {
        let result: Result<Value, Value> = (|| {
            let _roots = SynchronizeHostGraph(self.context, self.state)?;
            let object = to_js_value(self.context, self.state, value)?;
            let object = object
                .as_object()
                .ok_or_else(|| {
                    self.context
                        .type_error("exception wrapper must be an object")
                })?
                .clone();
            let message = to_js_value(self.context, self.state, message)?;
            let error = self.context.make_error_value(message)?;
            let error_object = error
                .as_object()
                .expect("intrinsic Error returns an object")
                .clone();
            let name = to_js_value(self.context, self.state, name)?;
            self.context
                .define_data_value(&error_object, "name", name, PropFlags::C_W)?;
            let key = PropKey::Str(self.context.intern("stack"));
            let stack = self.context.get_property(&error, &key)?;
            self.context
                .define_data_value(&object, "stack", stack, PropFlags::C_W)?;
            Ok(Value::Object(object))
        })();
        match result {
            Ok(value) => HostResult {
                value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
                ..Default::default()
            },
            Err(value) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    value,
                ))),
                ..Default::default()
            },
        }
    }
    fn DeferMicrotaskCheckpoint(&mut self) -> Box<dyn JavaScriptMicrotaskCheckpointScope> {
        let hooks = self.state.borrow().hooks.clone();
        hooks.checkpoint_suppression.set(
            hooks
                .checkpoint_suppression
                .get()
                .checked_add(1)
                .expect("microtask scope depth"),
        );
        Box::new(CheckpointScope(hooks))
    }
    fn RealmIdentity(&self) -> u64 {
        self.context.Identity()
    }
    fn IsUndefined(&mut self, value: &HostValue) -> bool {
        to_js_value(self.context, self.state, value).is_ok_and(|v| matches!(v, Value::Undefined))
    }
    fn CreateNativeFunction(&mut self, callback: Rc<dyn JavaScriptNativeFunction>) -> HostResult {
        let captures = callback.CapturedValues();
        let _roots = match SynchronizeHostGraph(self.context, self.state) {
            Ok(roots) => roots,
            Err(value) => {
                return HostResult {
                    thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                        value,
                    ))),
                    ..Default::default()
                };
            }
        };
        let data = match captures
            .iter()
            .map(|v| to_js_value(self.context, self.state, v))
            .collect::<JsResult<Vec<_>>>()
        {
            Ok(data) => data,
            Err(value) => {
                return HostResult {
                    thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                        value,
                    ))),
                    ..Default::default()
                };
            }
        };
        let function = self.context.new_host_function(
            Rc::new(NativeFunction {
                state: Rc::downgrade(self.state),
                callback,
            }),
            data,
            "",
            1,
        );
        HostResult {
            value: HostValue::JavaScriptFunction(JavaScriptFunction::new(function)),
            ..Default::default()
        }
    }
    fn CallValue(
        &mut self,
        function: &HostValue,
        receiver: &HostValue,
        arguments: &[HostValue],
    ) -> HostResult {
        let answer = (|| {
            let _roots = SynchronizeHostGraph(self.context, self.state)?;
            let function = to_js_value(self.context, self.state, function)?;
            let receiver = to_js_value(self.context, self.state, receiver)?;
            let arguments = arguments
                .iter()
                .map(|v| to_js_value(self.context, self.state, v))
                .collect::<JsResult<Vec<_>>>()?;
            self.context.call(function, receiver, &arguments)
        })();
        match answer {
            Ok(value) => HostResult {
                value: HostValue::JavaScriptValue(JavaScriptValue::new_traced(value)),
                ..Default::default()
            },
            Err(value) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    value,
                ))),
                ..Default::default()
            },
        }
    }
    fn NewPromiseCapability(&mut self) -> Result<JavaScriptPromiseCapability, JavaScriptException> {
        let (promise, resolve, reject) = self
            .context
            .new_promise_capability()
            .map_err(|e| exception(self.context, &e, ""))?;
        Ok(JavaScriptPromiseCapability {
            promise: from_js(&promise, self.context, self.state),
            resolve: JavaScriptFunction::new(
                resolve
                    .as_object()
                    .expect("Promise resolving function")
                    .clone(),
            ),
            reject: JavaScriptFunction::new(
                reject
                    .as_object()
                    .expect("Promise rejecting function")
                    .clone(),
            ),
        })
    }
    fn PromiseResolve(&mut self, value: &HostValue) -> HostResult {
        let value = to_js_value(self.context, self.state, value)
            .and_then(|v| self.context.promise_resolve(&v));
        match value {
            Ok(value) => HostResult {
                value: from_js(&value, self.context, self.state),
                ..HostResult::default()
            },
            Err(error) => HostResult {
                exception: Some(exception(self.context, &error, "")),
                ..HostResult::default()
            },
        }
    }
    fn PerformPromiseThen(
        &mut self,
        promise: &HostValue,
        fulfilled: &JavaScriptFunction,
        rejected: &JavaScriptFunction,
    ) -> HostResult {
        let answer = (|| {
            let promise = to_js_value(self.context, self.state, promise)?;
            let fulfilled = to_js_value(
                self.context,
                self.state,
                &HostValue::JavaScriptFunction(fulfilled.clone()),
            )?;
            let rejected = to_js_value(
                self.context,
                self.state,
                &HostValue::JavaScriptFunction(rejected.clone()),
            )?;
            self.context
                .perform_promise_then(&promise, &fulfilled, &rejected)
        })();
        match answer {
            Ok(()) => HostResult::default(),
            Err(error) => HostResult {
                exception: Some(exception(self.context, &error, "")),
                ..HostResult::default()
            },
        }
    }
    fn MarkPromiseHandled(&mut self, promise: &HostValue) -> HostResult {
        let answer = to_js_value(self.context, self.state, promise)
            .and_then(|promise| self.context.mark_promise_handled(&promise));
        match answer {
            Ok(()) => HostResult::default(),
            Err(error) => HostResult {
                exception: Some(exception(self.context, &error, "")),
                ..HostResult::default()
            },
        }
    }

    fn ExceptionMessage(&mut self, value: &HostValue) -> JavaScriptExceptionMessage {
        let value = to_js_value(self.context, self.state, value).expect("same VM exception");
        let location = self
            .context
            .error_creation_location(&value)
            .or_else(|| self.context.capture_current_source_location(0))
            .map(|location| JavaScriptSourceLocation {
                source_name: location.filename,
                line: location.line as usize,
                column: location.column as usize,
            });
        JavaScriptExceptionMessage {
            message: self.context.exception_message(&value),
            location,
        }
    }
    fn TaskAttributionTracker(&self) -> Rc<foundation::task_attribution::TaskAttributionTracker> {
        self.context.task_attribution_tracker()
    }
    fn IsInContext(&self) -> bool {
        self.context.is_in_context()
    }
    fn CaptureSourceLocation(&self, skip_script_frames: usize) -> Option<JavaScriptSourceLocation> {
        self.context
            .capture_current_source_location(skip_script_frames)
            .map(|location| JavaScriptSourceLocation {
                source_name: location.filename,
                line: location.line,
                column: location.column,
            })
    }
    fn ScriptToolInvocationId(&self) -> Option<[u64; 2]> {
        self.state.borrow().hooks.script_tool_invocation.get()
    }
    fn EnterScriptToolInvocation(
        &mut self,
        id: [u64; 2],
    ) -> Box<dyn JavaScriptScriptToolInvocationScope> {
        assert_ne!(id, [0, 0], "script tool invocation identity is not empty");
        let hooks = self.state.borrow().hooks.clone();
        let previous = hooks.script_tool_invocation.replace(Some(id));
        Box::new(ScriptToolScope { hooks, previous })
    }

    fn CreateHostFunction(&mut self, binding: &HostFunctionBinding) -> HostResult {
        let created = (|| -> JsResult<JsObject> {
            let _roots = SynchronizeHostGraph(self.context, self.state)?;
            let mut data = binding
                .data
                .iter()
                .map(|value| {
                    let value = to_js_value(self.context, self.state, value)?;
                    self.context.validate_value(&value)?;
                    Ok(value)
                })
                .collect::<JsResult<Vec<_>>>()?;
            let capture_receiver = binding.callback.receiver != 0;
            if capture_receiver {
                data.insert(
                    0,
                    to_js_value(
                        self.context,
                        self.state,
                        &HostValue::Object(HostObjectRef {
                            id: binding.callback.receiver,
                        }),
                    )?,
                );
            }
            Ok(self.context.new_host_function(
                Rc::new(HostFunction {
                    state: self.state.clone(),
                    action: Action::Callback(binding.callback.clone(), capture_receiver),
                }),
                data,
                &binding.name,
                binding.length,
            ))
        })();
        match created {
            Ok(function) => HostResult {
                value: HostValue::JavaScriptFunction(JavaScriptFunction::new(function)),
                ..Default::default()
            },
            Err(thrown) => HostResult {
                exception: Some(exception(self.context, &thrown, "")),
                ..Default::default()
            },
        }
    }
    fn IsProxy(&mut self, value: &HostValue) -> bool {
        to_js(self.context, self.state, value).is_ok_and(|value| self.context.is_proxy(&value))
    }
    fn ReportException(&mut self, value: &HostValue) -> HostResult {
        let value = match to_js(self.context, self.state, value) {
            Ok(value) => value,
            Err(value) => value,
        };
        let location = self.context.error_creation_location(&value);
        let reported = exception_at(self.context, &value, "", location);
        self.state
            .borrow()
            .hooks
            .exceptions
            .borrow_mut()
            .push(reported);
        HostResult::default()
    }
    fn Call(
        &mut self,
        callback: &JavaScriptFunction,
        receiver: &HostValue,
        args: &[HostValue],
    ) -> JavaScriptResult {
        call(self.context, self.state, callback, receiver, args)
    }
    fn ToString(&mut self, value: &HostValue) -> HostResult {
        let _host_roots = match SynchronizeHostGraph(self.context, self.state) {
            Ok(roots) => roots,
            Err(error) => {
                return HostResult {
                    thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                        error,
                    ))),
                    ..Default::default()
                };
            }
        };
        let converted = to_js_value(self.context, self.state, value)
            .and_then(|value| self.context.to_js_string(&value));
        match converted {
            Ok(text) => HostResult {
                value: {
                    let units = text.to_utf16();
                    match String::from_utf16(&units) {
                        Ok(text) => HostValue::String(text),
                        Err(_) => HostValue::String16(units.into()),
                    }
                },
                ..Default::default()
            },
            Err(thrown) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    thrown,
                ))),
                ..Default::default()
            },
        }
    }
    fn ToInt32(&mut self, value: &HostValue) -> HostResult {
        let _host_roots = match SynchronizeHostGraph(self.context, self.state) {
            Ok(roots) => roots,
            Err(error) => {
                return HostResult {
                    thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                        error,
                    ))),
                    ..Default::default()
                };
            }
        };
        match to_js_value(self.context, self.state, value)
            .and_then(|value| self.context.to_int32(&value))
        {
            Ok(value) => HostResult {
                value: HostValue::Number(f64::from(value)),
                ..Default::default()
            },
            Err(error) => HostResult {
                thrown: Some(HostValue::JavaScriptValue(JavaScriptValue::new_traced(
                    error,
                ))),
                ..Default::default()
            },
        }
    }
    fn SerializeForStorage(
        &mut self,
        value: &HostValue,
    ) -> Result<crate::SerializedScriptValue, crate::JavaScriptSerializationError> {
        serialize_for_storage(self.context, self.state, value)
    }
    fn Deserialize(
        &mut self,
        value: &crate::SerializedScriptValue,
    ) -> Result<HostValue, crate::JavaScriptSerializationError> {
        deserialize_storage(self.context, self.state, value)
    }
    fn Clone(&mut self, value: &HostValue) -> HostResult {
        clone_value(self.context, self.state, value)
    }
    fn EnqueueMicrotask(&mut self, callback: &JavaScriptFunction) -> HostResult {
        enqueue(self.context, callback)
    }
    fn PerformMicrotaskCheckpoint(&mut self) -> Vec<JavaScriptException> {
        checkpoint(self.context, self.state);
        take_exceptions(self.context, self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::javascript_runtime::{HostMethodRef, HostObjectRef};

    #[test]
    fn host_function_bindings_preserve_receiver_captures_and_realm_ownership() {
        struct Source(RefCell<Vec<HostObjectTraceChange>>);
        impl HostObjectTraceSource for Source {
            fn TakeChanges(&self) -> Vec<HostObjectTraceChange> {
                std::mem::take(&mut *self.0.borrow_mut())
            }
        }
        struct CallbackHost(Rc<Source>);
        impl JavaScriptHostBindings for CallbackHost {
            fn ObjectTraceSource(&self) -> Option<Rc<dyn HostObjectTraceSource>> {
                Some(self.0.clone())
            }
            fn GlobalNames(&self) -> Vec<String> {
                Vec::new()
            }
            fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
                assert_eq!(call.member, "callback");
                assert_eq!(call.arguments.len(), 3);
                HostResult {
                    value: HostValue::Record(vec![
                        ("receiver".into(), call.arguments[0].clone()),
                        ("captured".into(), call.arguments[1].clone()),
                        ("argument".into(), call.arguments[2].clone()),
                        ("boundID".into(), HostValue::Number(call.receiver as f64)),
                    ]),
                    ..Default::default()
                }
            }
        }
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(CallbackHost(Rc::new(Source(
            RefCell::new(vec![HostObjectTraceChange::Object { id: 77 }]),
        ))))));
        realm.WithImplementation::<QuickJsRealm, _>(|r| {
            let capture = r.context.eval("({held:7})").unwrap();
            let weak = WeakObject::new(capture.as_object().unwrap());
            let binding = HostFunctionBinding {
                name: "bound".into(), length: 1,
                callback: HostMethodRef { receiver: 0, name: "callback".into() },
                data: vec![HostValue::JavaScriptValue(JavaScriptValue::new_traced(capture))],
            };
            let created = HostRuntime { context: &mut r.context, state: &r.state }.CreateHostFunction(&binding);
            assert!(created.exception.is_none());
            let function = to_js(&mut r.context, &r.state, &created.value).unwrap();
            let global = r.context.global();
            r.context.define_value(&global, "bound", function, PropFlags::C_W_E);
            drop(created);
            drop(binding);
            assert!(weak.upgrade().is_some(), "engine capture must retain data");
            assert_eq!(r.context.eval("(()=>{const receiver={},argument={};const out=bound.call(receiver,argument);return bound.name==='bound'&&bound.length===1&&out.receiver===receiver&&out.argument===argument&&out.captured.held===7})()").unwrap().as_boolean(), Some(true));
            r.context.eval("delete globalThis.bound").unwrap();
            assert!(weak.upgrade().is_none(), "released function must release captures");

            let receiver = to_js(&mut r.context, &r.state, &HostValue::Object(HostObjectRef { id: 77 })).unwrap();
            let weak_receiver = WeakObject::new(receiver.as_object().unwrap());
            let binding = HostFunctionBinding {
                name: "boundReceiver".into(), length: 1,
                callback: HostMethodRef { receiver: 77, name: "callback".into() },
                data: vec![HostValue::Number(2.0)],
            };
            let created = HostRuntime { context: &mut r.context, state: &r.state }.CreateHostFunction(&binding);
            assert!(created.exception.is_none());
            let function = to_js(&mut r.context, &r.state, &created.value).unwrap();
            r.context.define_value(&global, "boundReceiver", function, PropFlags::C_W_E);
            drop(created);
            drop(receiver);
            r.context.collect_garbage();
            assert!(weak_receiver.upgrade().is_some(), "bound host receiver must remain reachable");
            assert_eq!(r.context.eval("boundReceiver.call({},4).boundID===77").unwrap().as_boolean(), Some(true));
            r.context.eval("delete globalThis.boundReceiver").unwrap();
            r.context.collect_garbage();
            assert!(weak_receiver.upgrade().is_none(), "bound receiver must be released with the function");

            let mut foreign = Context::new();
            let binding = HostFunctionBinding {
                name: "foreign".into(), length: 0,
                callback: HostMethodRef { receiver: 0, name: "callback".into() },
                data: vec![HostValue::JavaScriptValue(JavaScriptValue::new_traced(foreign.eval("({})").unwrap()))],
            };
            let created = HostRuntime { context: &mut r.context, state: &r.state }.CreateHostFunction(&binding);
            assert!(created.exception.is_some(), "foreign captures must be rejected");
        });
    }

    #[test]
    fn trusted_bootstrap_bytecode_preserves_realm_execution_and_source_identity() {
        let bootstrap = "(()=>{ const privateState={value:0}; globalThis.next=()=>++privateState.value; globalThis.boots=(globalThis.boots||0)+1; })();";
        for _ in 0..3 {
            let mut runtime = QuickJsJavaScriptRuntime::new();
            let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
            let result = runtime.EvaluateBootstrap(&realm, bootstrap, "browser:dom-webidl");
            assert!(result.Succeeded(), "{:?}", result.exception);
            let result = runtime.Evaluate(
                &realm,
                "boots===1 && next()===1 && next()===2",
                "test:bootstrap-state",
            );
            assert_eq!(
                result.value.Implementation::<Value>().unwrap().as_boolean(),
                Some(true)
            );
            // Reusing compiled code must not reuse execution or captured objects.
            assert!(runtime
                .EvaluateBootstrap(&realm, bootstrap, "browser:dom-webidl")
                .Succeeded());
            let result =
                runtime.Evaluate(&realm, "boots===2 && next()===1", "test:bootstrap-repeat");
            assert_eq!(
                result.value.Implementation::<Value>().unwrap().as_boolean(),
                Some(true)
            );
            for source in ["globalThis.answer=41", "globalThis.answer=42"] {
                assert!(runtime
                    .EvaluateBootstrap(&realm, source, "browser:window-webidl")
                    .Succeeded());
            }
            let result = runtime.Evaluate(&realm, "answer===42", "test:bootstrap-source");
            assert_eq!(
                result.value.Implementation::<Value>().unwrap().as_boolean(),
                Some(true)
            );
        }
        for _ in 0..2 {
            let mut runtime = QuickJsJavaScriptRuntime::new();
            let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
            let result = runtime.EvaluateBootstrap(
                &realm,
                "\nthrow new TypeError('bootstrap failure')",
                "browser:window-webidl",
            );
            let error = result.exception.expect("cached bootstrap still throws");
            assert!(error.message.contains("bootstrap failure"));
            assert_eq!(error.source_name, "browser:window-webidl");
            assert_eq!(error.line, 2);
        }
    }

    #[test]
    fn module_compilation_exposes_requests_without_execution_and_retains_realm_records() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let child = runtime
            .CompileModule(&realm, "export const x=3;", "https://test/child.js")
            .unwrap_or_else(|e| panic!("{}", e.message));
        let root = runtime.CompileModule(&realm,
            "// import './fake.js';\nconst text=\"export * from './fake.js'\"; import {x} from './child.js'; globalThis.answer=x;",
            "https://test/root.js").unwrap_or_else(|e| panic!("{}",e.message));
        assert_eq!(root.requests, vec!["./child.js"]);
        assert!(runtime
            .Evaluate(
                &realm,
                "if(typeof answer!=='undefined') throw Error('compiled code ran');",
                "assert.js"
            )
            .Succeeded());
        struct Prepared {
            record: JavaScriptValue,
            calls: Cell<usize>,
        }
        impl JavaScriptModuleResolver for Prepared {
            fn ResolveModule(
                &self,
                specifier: &str,
                referrer: &str,
            ) -> std::io::Result<Option<JavaScriptModuleSource>> {
                assert_eq!(specifier, "./child.js");
                assert_eq!(referrer, "https://test/root.js");
                self.calls.set(self.calls.get() + 1);
                Ok(Some(JavaScriptModuleSource {
                    url: "https://test/child.js".into(),
                    source: String::new(),
                    module: Some(self.record.clone()),
                }))
            }
        }
        let resolver = Rc::new(Prepared {
            record: child.module,
            calls: Cell::new(0),
        });
        let result = runtime.EvaluateCompiledModule(
            &realm,
            &root.module,
            "https://test/root.js",
            Some(resolver.clone()),
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(resolver.calls.get(), 1);
        assert!(runtime
            .Evaluate(
                &realm,
                "if(answer!==3) throw Error('compiled record was not linked');",
                "assert.js"
            )
            .Succeeded());
        let other = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(!runtime
            .EvaluateCompiledModule(&other, &root.module, "https://test/root.js", None)
            .Succeeded());
    }

    #[test]
    fn foreign_runtime_cannot_evaluate_call_or_clone_in_another_runtime_realm() {
        let mut source = QuickJsJavaScriptRuntime::new();
        let realm = source.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let value = source.Evaluate(&realm, "()=>17", "source.js");
        let function = JavaScriptFunction::new(
            value
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        );
        let mut target = QuickJsJavaScriptRuntime::new();
        assert_eq!(
            target
                .Evaluate(&realm, "globalThis.changed=true", "foreign.js")
                .exception
                .unwrap()
                .message,
            "realm belongs to another runtime"
        );
        assert_eq!(
            target
                .Call(&realm, &function, &HostValue::default(), &[])
                .exception
                .unwrap()
                .message,
            "value is not callable in this realm"
        );
        assert_eq!(
            target
                .Clone(&realm, &HostValue::Number(1.0))
                .exception
                .unwrap()
                .message,
            "Invalid realm"
        );
        assert_eq!(
            source
                .Evaluate(&realm, "typeof changed", "check.js")
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            "undefined"
        );
    }

    #[test]
    fn parser_exception_keeps_source_location_and_runtime_stack() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let syntax = runtime
            .Evaluate(&realm, "\nlet broken = ;", "https://test/broken.js")
            .exception
            .unwrap();
        assert_eq!(syntax.kind, JavaScriptExceptionKind::kSyntaxError);
        assert_eq!(syntax.source_name, "https://test/broken.js");
        assert_eq!(syntax.line, 2);
        assert_ne!(syntax.column, usize::MAX);
        assert!(syntax.stack.contains("https://test/broken.js:2:"));
        let thrown = runtime
            .Evaluate(
                &realm,
                "function nested(){throw new TypeError('boom')}nested()",
                "https://test/call.js",
            )
            .exception
            .unwrap();
        assert_eq!(thrown.kind, JavaScriptExceptionKind::kTypeError);
        assert_eq!(thrown.message, "TypeError: boom");
        assert!(thrown.stack.contains("nested (https://test/call.js:"));
    }

    #[test]
    fn intl_locale_maximizes_script_and_region() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.Evaluate(&realm,
            "const locale=new Intl.Locale('zh').maximize(); locale.language+':'+locale.script+':'+locale.region",
            "intl-locale.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            "zh:Hans:CN"
        );
        let result = runtime.Evaluate(&realm, r#"
            JSON.stringify([
                Intl.NumberFormat.supportedLocalesOf(['en', 'de-DE', 'zh-CN', 'zz-ZZ']),
                new Intl.NumberFormat('de-DE', {minimumFractionDigits:2, maximumFractionDigits:2}).format(1234.5),
                new Intl.NumberFormat('en', {maximumFractionDigits:0}).format(-1.5),
                new Intl.NumberFormat('de-DE', {minimumFractionDigits:2, maximumFractionDigits:2}).formatToParts(1234.5).map(({type,value}) => [type,value]),
                new Intl.NumberFormat('en').formatToParts(-Infinity).map(({type,value}) => [type,value]),
                new Intl.DateTimeFormat('en-US', {timeZone:'UTC'}).format(Date.UTC(2026,9,15)),
                new Intl.PluralRules('ar').select(2),
                new Intl.PluralRules('en', {type:'ordinal'}).select(2),
                new Intl.PluralRules('en', {minimumFractionDigits:2}).select(1)
            ])
        "#, "intl-format.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            r#"[["en","de-DE","zh-CN"],"1.234,50","-2",[["integer","1"],["group","."],["integer","234"],["decimal",","],["fraction","50"]],[["minusSign","-"],["infinity","∞"]],"10/15/2026","two","two","other"]"#
        );
    }

    #[test]
    fn intl_segmenter_exposes_utf16_segments_and_word_status() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.Evaluate(
            &realm,
            r#"
                const graphemes = [...new Intl.Segmenter('en').segment('A🇺🇸e\u0301')];
                const words = [...new Intl.Segmenter('en', {granularity:'word'}).segment('Hi, 42!')];
                JSON.stringify([
                    graphemes.map(x => [x.segment, x.index]),
                    words.map(x => [x.segment, x.index, x.isWordLike]),
                    new Intl.Segmenter().segment('a😀b').containing(2).segment,
                    new Intl.Segmenter('de', {granularity:'sentence'}).resolvedOptions()
                ])
            "#,
            "intl-segmenter.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            r#"[[["A",0],["🇺🇸",1],["é",5]],[["Hi",0,true],[",",2,false],[" ",3,false],["42",4,true],["!",6,false]],"😀",{"locale":"de","granularity":"sentence"}]"#
        );
    }

    #[test]
    fn binary_operands_preserve_values_before_rhs_updates() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.Evaluate(
            &realm,
            r#"
            JSON.stringify([
                (()=>{let n=1;return n + ++n})(),
                (()=>{var n=1;return n + ++n})(),
                (()=>{let n=2,offsets=[0,0,0,7];return n+offsets[++n]})(),
                (()=>{let n=1;return n === ++n})(),
                (()=>{let n=1;return n | (n=4)})(),
                (()=>{let n=1;return n - (n=4)})()
            ])
        "#,
            "binary-evaluation-order.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            "[3,3,9,false,5,-3]"
        );
    }

    #[test]
    fn large_bundle_register_frame_is_not_a_recursion_overflow() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let mut script = String::new();
        for i in 0..12_000 {
            script.push_str(&format!("let bundle_value_{i}={i};"));
        }
        script.push_str("bundle_value_11999");
        let result = runtime.Evaluate(&realm, &script, "large-bundle.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_number(),
            Some(11999.0)
        );
        let recursive = runtime.Evaluate(
            &realm,
            "(function recurse(){return recurse()})()",
            "recursive.js",
        );
        assert!(recursive.exception.is_some(), "recursion remains bounded");
    }

    #[test]
    fn super_method_arrow_preserves_lexical_receiver() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.Evaluate(&realm, "class Base { Cv(){} wa(){return ()=>a=>this;} } class Child extends Base {wa(){var a=super.wa();return b=>a(b);}} const child=new Child; child.wa()()(0) === child && (()=>{const o={make(){return ()=>()=>()=>this;}};return o.make()()()()===o})() && (()=>{function f(){return ()=>()=>this};const o={};return f.call(o)()()===o})()", "lexical-receiver.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
    }

    #[test]
    fn structured_clone_preserves_graph_and_rejects_executables() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let original = runtime.Evaluate(&realm, "(()=>{let buffer=new ArrayBuffer(8);let bytes=new Uint8Array(buffer);bytes[3]=42;let a={bytes,view:new DataView(buffer),date:new Date(123),regex:/abc/gi,sparse:new Array(3)};a.self=a;a.map=new Map([[a,new Set([a])]]);a.sparse[2]=a;return a})()", "clone-input.js");
        let cloned = runtime.Clone(&realm, &HostValue::JavaScriptValue(original.value));
        assert!(cloned.exception.is_none(), "{:?}", cloned.exception);
        let check = runtime.Evaluate(&realm, "a=>a.self===a && a.map.get(a).has(a) && a.bytes[3]===42 && a.view.buffer===a.bytes.buffer && a.date.getTime()===123 && a.regex.source==='abc' && a.regex.flags==='gi' && !(0 in a.sparse) && a.sparse[2]===a", "clone-check.js");
        let function = JavaScriptFunction::new(
            check
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        );
        let checked = runtime.Call(&realm, &function, &HostValue::default(), &[cloned.value]);
        assert_eq!(
            checked
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
        for source in [
            "()=>{}",
            "Symbol('x')",
            "new WeakMap()",
            "new Proxy({}, {})",
            "({nested:()=>{}})",
        ] {
            let value = runtime.Evaluate(&realm, source, "not-cloneable.js");
            assert!(
                runtime
                    .Clone(&realm, &HostValue::JavaScriptValue(value.value))
                    .exception
                    .is_some(),
                "{source}"
            );
        }
    }

    struct EmptyHost;
    impl JavaScriptHostBindings for EmptyHost {
        fn GlobalNames(&self) -> Vec<String> {
            Vec::new()
        }
        fn Invoke(&mut self, _call: &HostCall<'_>) -> HostResult {
            HostResult::default()
        }
    }

    #[test]
    fn child_context_can_initialize_synchronously_during_parent_host_call() {
        struct FrameHost(Rc<RefCell<Option<JavaScriptRealm>>>);
        impl JavaScriptHostBindings for FrameHost {
            fn GlobalNames(&self) -> Vec<String> {
                vec!["childWindow".into()]
            }
            fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
                if call.member == "childWindow" && call.operation == HostOperation::kGet {
                    return HostResult {
                        value: HostValue::Method(HostMethodRef {
                            receiver: 0,
                            name: "childWindow".into(),
                        }),
                        ..Default::default()
                    };
                }
                HostResult::default()
            }
            fn PrepareInvocation(&mut self, call: &HostCall<'_>) -> Option<HostContinuation> {
                if call.member != "childWindow" || call.operation != HostOperation::kCall {
                    return None;
                }
                let slot = self.0.clone();
                Some(Box::new(move |runtime| {
                    let realm = runtime
                        .CreateChildRealm(Rc::new(RefCell::new(EmptyHost)))
                        .unwrap();
                    let mut result = HostResult::default();
                    runtime.WithRealm(&realm, &mut |child| {
                        result = child.EvaluateClassicScript(
                            &"var marker={}; var jobs=[]; Promise.resolve().then(()=>jobs.push('child'));".encode_utf16().collect::<Vec<_>>(),
                            "test:initial-about-blank",
                        );
                        assert!(result.exception.is_none() && result.thrown.is_none());
                        result.value = child.GlobalObject();
                    }).unwrap();
                    *slot.borrow_mut() = Some(realm);
                    result
                }))
            }
        }
        let child = Rc::new(RefCell::new(None));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let parent = runtime.CreateRealm(Rc::new(RefCell::new(FrameHost(child.clone()))));
        let result = runtime.Evaluate(&parent, r#"
            var child=childWindow();
            if(child===globalThis || child.Array===Array || !(new child.Array instanceof child.Array))
                throw Error('child global or intrinsic identity');
            child.marker.parent=globalThis;
            if(child.marker.parent!==globalThis || child.jobs.length!==0)
                throw Error('identity or premature child microtask');
            Promise.resolve().then(()=>child.jobs.push('parent'));
        "#, "test:parent-host-reentry");
        assert!(result.Succeeded(), "{:?}", result.exception);
        runtime.PerformMicrotaskCheckpoint();
        let result = runtime.Evaluate(
            &parent,
            "if(child.jobs.join(',')!=='child,parent')throw Error(child.jobs);",
            "test:shared-agent-checkpoint",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        runtime.CollectGarbage();
        let result = runtime.Evaluate(
            child.borrow().as_ref().unwrap(),
            "if(marker.parent.child!==globalThis)throw Error('child lifetime');",
            "test:child-gc-identity",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
    }

    #[test]
    fn modules_preserve_live_bindings_cycles_cache_meta_and_checkpoint_order() {
        use crate::javascript_runtime::{JavaScriptModuleResolver, JavaScriptModuleSource};
        struct Loader(RefCell<Vec<(String, String)>>);
        impl JavaScriptModuleResolver for Loader {
            fn ResolveModule(
                &self,
                specifier: &str,
                referrer: &str,
            ) -> std::io::Result<Option<JavaScriptModuleSource>> {
                self.0
                    .borrow_mut()
                    .push((specifier.into(), referrer.into()));
                let (url, source) = match specifier {
                    "a" => (
                        "https://test/a.js",
                        "import {getB} from 'b'; export let a=2; export function change(){a=7}; export const read=()=>getB()+a; globalThis.order.push('a:'+import.meta.url);",
                    ),
                    "b" => (
                        "https://test/b.js",
                        "import {a} from 'a'; export function getB(){return a*3}; globalThis.order.push('b');",
                    ),
                    _ => return Ok(None),
                };
                Ok(Some(JavaScriptModuleSource {
                    module: None,
                    url: url.into(),
                    source: source.into(),
                }))
            }
        }
        let loader = Rc::new(Loader(RefCell::new(Vec::new())));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(runtime
            .Evaluate(
                &realm,
                "var order=[]; Promise.resolve().then(()=>order.push('queued'));",
                "classic"
            )
            .Succeeded());
        let result=runtime.EvaluateModule(&realm,
            "import {read,change} from 'a'; order.push('root:'+import.meta.url); if(read()!==8)throw Error('cycle');change();if(read()!==28)throw Error('live binding');Promise.resolve().then(()=>order.push('module-job'));",
            "https://test/root.js",Some(loader.clone()));
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert!(runtime.Evaluate(&realm,"if(order.join('|')!=='b|a:https://test/a.js|root:https://test/root.js')throw Error(order.join('|'))", "before-checkpoint").Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime.TakePendingExceptions(&realm).is_empty());
        assert!(runtime
            .Evaluate(
                &realm,
                "if(order.slice(-2).join('|')!=='queued|module-job')throw Error(order.join('|'))",
                "after-checkpoint"
            )
            .Succeeded());
        let count = loader.0.borrow().len();
        assert!(runtime
            .EvaluateModule(
                &realm,
                "throw Error('must reuse compiled root')",
                "https://test/root.js",
                Some(loader.clone())
            )
            .Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        assert_eq!(loader.0.borrow().len(), count);
        let result = runtime.EvaluateModule(
            &realm,
            "import 'absent'",
            "https://test/missing.js",
            Some(loader.clone()),
        );
        assert_eq!(
            result.exception.unwrap().message,
            "TypeError: Unresolved module import: absent from https://test/missing.js"
        );
        runtime.PerformMicrotaskCheckpoint();
        assert!(
            runtime.TakePendingExceptions(&realm).is_empty(),
            "internal graph promise must not report a second error"
        );
        assert!(runtime
            .EvaluateModule(
                &realm,
                "await Promise.resolve(); order.push('awaited');",
                "https://test/await.js",
                Some(loader)
            )
            .Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime.TakePendingExceptions(&realm).is_empty());
        assert!(runtime.Evaluate(&realm,"if(order.at(-1)!=='awaited')throw Error('top-level await');import('ignored').catch(e=>order.push(e.message));", "dynamic").Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime
            .Evaluate(
                &realm,
                "if(order.at(-1)!=='Not supported')throw Error(order.at(-1))",
                "dynamic-result"
            )
            .Succeeded());
        let mut foreign = QuickJsJavaScriptRuntime::new();
        assert_eq!(
            foreign
                .EvaluateModule(&realm, "", "foreign", None)
                .exception
                .unwrap()
                .message,
            "realm belongs to another runtime"
        );
    }

    #[test]
    fn embedding_module_policy_preserves_coercion_order_and_distinct_rejections() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.Evaluate(
            &realm,
            r#"
            var order=[];
            var specifier={toString(){order.push('coerce');return 'missing'}};
            import(specifier).catch(e=>order.push(e.message));
            Promise.resolve().then(()=>order.push('following'));
            var bad={toString(){throw new Error('coercion failed')}};
            var returned;
            try { returned=import(bad); order.push('returned-promise'); }
            catch(e) { throw new Error('dynamic import synchronously threw'); }
            if(!(returned instanceof Promise)) throw new Error('not a Promise');
            returned.catch(e=>order.push(e.message));
            import('missing',{with:{type:42}}).catch(e=>order.push(e.name));
            Promise.reject('same'); Promise.reject('same');
        "#,
            "policy.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        runtime.PerformMicrotaskCheckpoint();
        let errors = runtime.TakePendingExceptions(&realm);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors
            .iter()
            .all(|error| error.message == "Unhandled rejection: same"));
        let result = runtime.Evaluate(&realm,
            "if(order.join('|')!=='coerce|returned-promise|Not supported|following|coercion failed|TypeError')throw Error(order.join('|'))",
            "policy-check.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        let result = runtime.EvaluateModule(
            &realm,
            "throw new Error('module rejection');",
            "https://test/throws.js",
            None,
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        runtime.PerformMicrotaskCheckpoint();
        let errors = runtime.TakePendingExceptions(&realm);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(
            errors[0].message,
            "Unhandled rejection: Error: module rejection"
        );
    }

    #[test]
    fn dynamic_import_waits_for_host_graph_and_resolves_namespace() {
        use crate::javascript_runtime::{JavaScriptModuleResolver, JavaScriptModuleSource};
        struct DynamicLoader {
            available: std::cell::Cell<bool>,
        }
        impl JavaScriptModuleResolver for DynamicLoader {
            fn ResolveModule(
                &self,
                specifier: &str,
                _referrer: &str,
            ) -> std::io::Result<Option<JavaScriptModuleSource>> {
                if specifier != "./chunk.js" || !self.available.get() {
                    return Ok(None);
                }
                Ok(Some(JavaScriptModuleSource {
                    module: None,
                    url: "https://test/chunk.js".into(),
                    source: "export const answer=42".into(),
                }))
            }
            fn RequestDynamicModule(
                &self,
                specifier: &str,
                _referrer: &str,
            ) -> std::io::Result<Option<String>> {
                Ok((specifier == "./chunk.js").then(|| "https://test/chunk.js".into()))
            }
            fn PollDynamicModule(
                &self,
                url: &str,
            ) -> std::io::Result<Option<JavaScriptModuleSource>> {
                if url != "https://test/chunk.js" || !self.available.get() {
                    return Ok(None);
                }
                Ok(Some(JavaScriptModuleSource {
                    module: None,
                    url: url.into(),
                    source: "export const answer=42".into(),
                }))
            }
        }
        let loader = Rc::new(DynamicLoader {
            available: std::cell::Cell::new(false),
        });
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.EvaluateModule(
            &realm,
            "globalThis.dynamicAnswer='pending'; import('./chunk.js').then(m=>dynamicAnswer=m.answer)",
            "https://test/root.js",
            Some(loader.clone()),
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime
            .Evaluate(&realm, "dynamicAnswer==='pending'", "pending-check.js")
            .value
            .Implementation::<JsValue>()
            .unwrap()
            .as_boolean()
            .unwrap());
        loader.available.set(true);
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime
            .Evaluate(&realm, "dynamicAnswer===42", "resolved-check.js")
            .value
            .Implementation::<JsValue>()
            .unwrap()
            .as_boolean()
            .unwrap());
    }

    #[test]
    fn module_loading_prepares_direct_requests_first_and_retries_failed_graphs() {
        use crate::javascript_runtime::{JavaScriptModuleResolver, JavaScriptModuleSource};
        struct Loader {
            requests: RefCell<Vec<String>>,
            available: std::cell::Cell<bool>,
        }
        impl JavaScriptModuleResolver for Loader {
            fn ResolveModule(
                &self,
                specifier: &str,
                _referrer: &str,
            ) -> std::io::Result<Option<JavaScriptModuleSource>> {
                self.requests.borrow_mut().push(specifier.into());
                let source = match specifier {
                    "a" => "import 'leaf'; globalThis.order.push('a');",
                    "b" => "globalThis.order.push('b');",
                    "leaf" => "globalThis.order.push('leaf');",
                    "retry" if !self.available.get() => {
                        return Err(std::io::Error::other("not yet available"));
                    }
                    "retry" => "globalThis.order.push('retry');",
                    _ => return Ok(None),
                };
                Ok(Some(JavaScriptModuleSource {
                    module: None,
                    url: format!("https://test/{specifier}.js"),
                    source: source.into(),
                }))
            }
        }
        let loader = Rc::new(Loader {
            requests: RefCell::new(Vec::new()),
            available: std::cell::Cell::new(false),
        });
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(runtime
            .Evaluate(&realm, "var order=[]", "initial.js")
            .Succeeded());
        let result = runtime.EvaluateModule(
            &realm,
            "import 'a'; import 'b';",
            "https://test/root.js",
            Some(loader.clone()),
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(*loader.requests.borrow(), ["a", "b", "leaf"]);
        let failed = runtime.EvaluateModule(
            &realm,
            "import 'retry'; order.push('root-retry');",
            "https://test/retry-root.js",
            Some(loader.clone()),
        );
        assert_eq!(
            failed.exception.unwrap().message,
            "TypeError: Failed to load module retry from https://test/retry-root.js: not yet available"
        );
        loader.available.set(true);
        let result = runtime.EvaluateModule(
            &realm,
            "throw Error('cached source must be used')",
            "https://test/retry-root.js",
            Some(loader.clone()),
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        let result = runtime.Evaluate(
            &realm,
            "if(order.join('|')!=='leaf|a|b|retry|root-retry')throw Error(order.join('|'))",
            "retry-check.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            *loader.requests.borrow(),
            ["a", "b", "leaf", "retry", "retry"]
        );
        let weak = Rc::downgrade(&loader);
        drop(loader);
        assert!(
            weak.upgrade().is_none(),
            "host loader retained outside evaluation scope"
        );
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime.TakePendingExceptions(&realm).is_empty());
    }

    #[test]
    fn checkpoint_reports_uncaught_callbacks_and_only_unhandled_rejections_per_realm() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let other = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(runtime.Evaluate(&realm,
            "var order = []; var handled = Promise.reject('handled');\
             Promise.resolve().then(() => handled.catch(() => {}));\
             Promise.resolve().then(() => { order.push('promise'); throw new Error('unhandled'); });",
            "jobs.js").Succeeded());
        assert!(runtime
            .Evaluate(&other, "Promise.reject('other realm')", "other.js")
            .Succeeded());
        let throwing = runtime.Evaluate(
            &realm,
            "() => { order.push('throwing'); throw new TypeError('callback failure'); }",
            "jobs.js",
        );
        let callback = JavaScriptFunction::new(
            throwing
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        );
        assert!(runtime.EnqueueMicrotask(&realm, &callback).Succeeded());
        let mut foreign_runtime = QuickJsJavaScriptRuntime::new();
        assert!(!foreign_runtime
            .EnqueueMicrotask(&realm, &callback)
            .Succeeded());
        assert!(foreign_runtime.TakePendingExceptions(&realm).is_empty());
        let following = runtime.Evaluate(&realm, "() => order.push('following')", "jobs.js");
        let callback = JavaScriptFunction::new(
            following
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        );
        assert!(runtime.EnqueueMicrotask(&realm, &callback).Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        let errors = runtime.TakePendingExceptions(&realm);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].message.contains("callback failure"));
        assert_eq!(errors[0].kind, JavaScriptExceptionKind::kRuntimeError);
        assert_eq!(errors[1].message, "Unhandled rejection: Error: unhandled");
        assert!(runtime.TakePendingExceptions(&realm).is_empty());
        let errors = runtime.TakePendingExceptions(&other);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Unhandled rejection: other realm");
        let order = runtime.Evaluate(&realm, "order.join(',')", "jobs.js");
        assert_eq!(
            order
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            "promise,throwing,following"
        );
    }

    #[test]
    fn realm_evaluation_function_call_and_microtask_checkpoint() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let result = runtime.Evaluate(
            &realm,
            "let total = 1; Promise.resolve().then(() => total = 7); total",
            "test.js",
        );
        assert!(result.Succeeded());
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_number(),
            Some(1.0)
        );
        runtime.PerformMicrotaskCheckpoint();
        let result = runtime.Evaluate(&realm, "total", "test.js");
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_number(),
            Some(7.0)
        );
        let function = runtime.Evaluate(&realm, "(x) => x + total", "test.js");
        let object = function
            .value
            .Implementation::<JsValue>()
            .unwrap()
            .as_object()
            .unwrap();
        let function = JavaScriptFunction::new(object.clone());
        let called = runtime.Call(
            &realm,
            &function,
            &HostValue::Object(HostObjectRef { id: 0 }),
            &[HostValue::Number(5.0)],
        );
        assert_eq!(
            called
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_number(),
            Some(12.0)
        );
        let same_function = JavaScriptFunction::new(object.clone());
        assert!(function == same_function);
    }

    struct TestDom {
        class_name: Rc<RefCell<String>>,
        dynamic_global: Rc<RefCell<String>>,
    }

    impl JavaScriptHostBindings for TestDom {
        fn GlobalNames(&self) -> Vec<String> {
            vec![
                "document".to_owned(),
                "dynamic".to_owned(),
                "fallback".to_owned(),
            ]
        }

        fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
            let value = match (call.receiver, call.operation, call.member) {
                (0, HostOperation::kGet, "document") => HostValue::Object(HostObjectRef { id: 1 }),
                (0, HostOperation::kGet, "dynamic") => {
                    HostValue::String(self.dynamic_global.borrow().clone())
                }
                (1, HostOperation::kGet, "querySelector") => HostValue::Method(HostMethodRef {
                    receiver: 1,
                    name: "querySelector".into(),
                }),
                (1, HostOperation::kCall, "querySelector") => {
                    HostValue::Object(HostObjectRef { id: 2 })
                }
                (2, HostOperation::kGet, "className") => {
                    HostValue::String(self.class_name.borrow().clone())
                }
                (2, HostOperation::kSet, "className") => {
                    if let [HostValue::String(value)] = call.arguments {
                        *self.class_name.borrow_mut() = value.clone();
                    }
                    HostValue::default()
                }
                _ => {
                    return HostResult {
                        handled: false,
                        ..Default::default()
                    };
                }
            };
            HostResult {
                value,
                ..Default::default()
            }
        }
    }

    #[test]
    fn host_objects_keep_identity_and_route_properties_methods_and_expandos() {
        let class_name = Rc::new(RefCell::new(String::from("before")));
        let dynamic_global = Rc::new(RefCell::new(String::from("first")));
        let host = Rc::new(RefCell::new(TestDom {
            class_name: class_name.clone(),
            dynamic_global: dynamic_global.clone(),
        }));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(host);
        let script = "const a = document.querySelector('#x');\
                      const b = document.querySelector('#x');\
                      a.className = 'after'; a.extra = 7;\
                      a === b && b.className === 'after' && b.extra === 7";
        let result = runtime.Evaluate(&realm, script, "host.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
        assert_eq!(&*class_name.borrow(), "after");
        let long = runtime.Evaluate(
            &realm,
            "document.querySelector('#x').className = 'x'.repeat(4096)+' '+'y'.repeat(4096)",
            "host-rope.js",
        );
        assert!(long.Succeeded(), "{:?}", long.exception);
        assert_eq!(
            &*class_name.borrow(),
            &format!("{} {}", "x".repeat(4096), "y".repeat(4096))
        );
        assert_eq!(
            long.value
                .Implementation::<JsValue>()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            *class_name.borrow()
        );
        let result = runtime.Evaluate(
            &realm,
            "Reflect = null; const c = document.querySelector('#x'); c.another = 11; c.another",
            "host.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_number(),
            Some(11.0)
        );
        *dynamic_global.borrow_mut() = String::from("second");
        let result = runtime.Evaluate(
            &realm,
            "dynamic === 'second' && (fallback = 5) === 5 && fallback === 5",
            "host.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            result
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_boolean(),
            Some(true)
        );
    }
}

#[cfg(test)]
mod loading_wrapper_timing {
    use super::*;
    struct EmptyHost;
    impl JavaScriptHostBindings for EmptyHost {
        fn GlobalNames(&self) -> Vec<String> {
            vec![]
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult::default()
        }
    }
    #[test]
    #[ignore = "manual Debug DOM wrapper identity scaling"]
    fn profile_loading_wrapper_identity() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        realm.WithImplementation::<QuickJsRealm,_>(|r| {
            let create = std::time::Instant::now();
            for id in 1..=6000 { to_js(&mut r.context,&r.state,&HostValue::Object(HostObjectRef{id})).unwrap(); }
            eprintln!("wrapper-create-bench count=6000 ms={:.3}",create.elapsed().as_secs_f64()*1000.0);
            let started = std::time::Instant::now();
            for i in 0..20000 {
                let id = 1 + (i*997)%6000;
                let js = to_js(&mut r.context,&r.state,&HostValue::Object(HostObjectRef{id})).unwrap();
                assert!(matches!(from_js(&js,&r.context,&r.state),HostValue::Object(HostObjectRef{id: actual}) if actual==id));
            }
            eprintln!("wrapper-identity-bench calls=20000 ms={:.3}",started.elapsed().as_secs_f64()*1000.0);
        }).unwrap();
    }
}

#[cfg(test)]
mod wrapper_identity_tests {
    use super::*;
    struct Host {
        prototypes: Rc<Cell<usize>>,
        source: Option<Rc<dyn HostObjectTraceSource>>,
    }
    impl JavaScriptHostBindings for Host {
        fn ObjectTraceSource(&self) -> Option<Rc<dyn HostObjectTraceSource>> {
            self.source.clone()
        }
        fn GlobalNames(&self) -> Vec<String> {
            vec![]
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult {
                handled: false,
                ..Default::default()
            }
        }
        fn UsesOrdinaryProperties(&self, id: HostObjectId) -> bool {
            id == 17
        }
        fn PrototypeFor(&mut self, _: HostObjectId) -> HostResult {
            self.prototypes.set(self.prototypes.get() + 1);
            HostResult {
                handled: true,
                value: HostValue::Null(Default::default()),
                ..Default::default()
            }
        }
    }
    #[test]
    fn host_identity_survives_fresh_js_handles_and_is_scoped_to_each_realm() {
        let calls = Rc::new(Cell::new(0));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host {
            prototypes: calls.clone(),
            source: None,
        })));
        let js = realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let expected = to_js(
                    &mut r.context,
                    &r.state,
                    &HostValue::Object(HostObjectRef { id: 17 }),
                )
                .unwrap();
                assert!(
                    !r.context.is_proxy(&expected),
                    "ordinary wrapper needs no host traps"
                );
                for id in 100..1100 {
                    let intercepted = to_js(
                        &mut r.context,
                        &r.state,
                        &HostValue::Object(HostObjectRef { id }),
                    )
                    .unwrap();
                    assert!(
                        r.context.is_proxy(&intercepted),
                        "exotic host semantics stay intercepted"
                    );
                }
                let global = r.context.global();
                r.context
                    .define_value(&global, "wrapper", expected.clone(), PropFlags::C_W_E);
                let fresh = r.context.eval("wrapper").unwrap();
                assert!(Value::strict_equals(&expected, &fresh));
                assert!(matches!(
                    from_js(&fresh, &r.context, &r.state),
                    HostValue::Object(HostObjectRef { id: 17 })
                ));
                for _ in 0..10 {
                    let again = to_js(
                        &mut r.context,
                        &r.state,
                        &HostValue::Object(HostObjectRef { id: 17 }),
                    )
                    .unwrap();
                    assert!(Value::strict_equals(&again, &fresh));
                }
                assert_eq!(
                    calls.get(),
                    1001,
                    "prototype installation occurs once per wrapper"
                );
                expected
            })
            .unwrap();
        let second = runtime.CreateRealm(Rc::new(RefCell::new(Host {
            prototypes: calls.clone(),
            source: None,
        })));
        second
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let local = to_js(
                    &mut r.context,
                    &r.state,
                    &HostValue::Object(HostObjectRef { id: 17 }),
                )
                .unwrap();
                assert!(!Value::strict_equals(&js, &local));
                assert!(
                    matches!(
                        from_js(&js, &r.context, &r.state),
                        HostValue::JavaScriptValue(_)
                    ),
                    "foreign identity must not be treated as a local host object"
                );
                assert!(matches!(
                    from_js(&local, &r.context, &r.state),
                    HostValue::Object(HostObjectRef { id: 17 })
                ));
            })
            .unwrap();

        struct TraceSource(RefCell<Vec<HostObjectTraceChange>>);
        impl HostObjectTraceSource for TraceSource {
            fn TakeChanges(&self) -> Vec<HostObjectTraceChange> {
                std::mem::take(&mut *self.0.borrow_mut())
            }
        }
        use HostObjectTraceChange as C;
        let source = Rc::new(TraceSource(RefCell::new(vec![
            C::Object { id: 17 },
            C::Root {
                id: 17,
                reason: 0,
                rooted: true,
            },
        ])));
        let managed = runtime.CreateRealm(Rc::new(RefCell::new(Host {
            prototypes: calls,
            source: Some(source.clone()),
        })));
        managed
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let value = to_js(
                    &mut r.context,
                    &r.state,
                    &HostValue::Object(HostObjectRef { id: 17 }),
                )
                .unwrap();
                let weak = WeakObject::new(value.as_object().unwrap());
                drop(value);
                source.0.borrow_mut().push(C::Root {
                    id: 17,
                    reason: 0,
                    rooted: false,
                });
                let scope = SynchronizeHostGraph(&mut r.context, &r.state).unwrap();
                assert!(
                    weak.upgrade().is_some(),
                    "former host root died before callback/return conversion"
                );
                drop(scope);
                assert!(
                    weak.upgrade().is_none(),
                    "former host root survived its conversion scope"
                );
                source.0.borrow_mut().extend([
                    C::Root {
                        id: 19,
                        reason: 0,
                        rooted: true,
                    },
                    C::Reference {
                        owner: 19,
                        slot: 1,
                        target: Some(17),
                    },
                ]);
                let value = to_js(
                    &mut r.context,
                    &r.state,
                    &HostValue::Object(HostObjectRef { id: 17 }),
                )
                .unwrap();
                let weak = WeakObject::new(value.as_object().unwrap());
                drop(value);
                source.0.borrow_mut().push(C::Reference {
                    owner: 19,
                    slot: 1,
                    target: None,
                });
                let scope = SynchronizeHostGraph(&mut r.context, &r.state).unwrap();
                assert!(
                    weak.upgrade().is_some(),
                    "removed host edge died before return conversion"
                );
                drop(scope);
                assert!(
                    weak.upgrade().is_none(),
                    "removed host edge survived its conversion scope"
                );
            })
            .unwrap();
    }
}

#[cfg(test)]
mod exception_loading_timing {
    use super::*;
    struct Host;
    impl JavaScriptHostBindings for Host {
        fn GlobalNames(&self) -> Vec<String> {
            vec![]
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult::default()
        }
    }
    #[test]
    #[ignore = "manual Debug caught exception diagnostics on a minified source"]
    fn profile_caught_exceptions_in_large_source() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host)));
        let mut source = format!("/*{}*/", "p".repeat(100_000));
        source.push_str("for(let i=0;i<300;i++){try{null.missing()}catch(e){}}");
        let started = std::time::Instant::now();
        assert!(runtime.Evaluate(&realm, &source, "minified.js").Succeeded());
        eprintln!(
            "caught-exception-bench bytes={} throws=300 ms={:.3}",
            source.len(),
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
}

#[cfg(test)]
mod vm_loading_timing {
    use super::*;
    struct Host;
    impl JavaScriptHostBindings for Host {
        fn GlobalNames(&self) -> Vec<String> {
            vec![]
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult::default()
        }
    }
    #[test]
    #[ignore = "manual Debug VM straight-line CFG timing"]
    fn profile_vm_basic_blocks() {
        let mut runtime = QuickJsJavaScriptRuntime::default();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host)));
        let started = std::time::Instant::now();
        let result=runtime.Evaluate(&realm,"function step(x){return x*3+1;}var sum=0;for(var i=0;i<200000;i++)sum+=step(i%100);if(sum!==29900000)throw Error(sum);","fixture:vm-blocks");
        assert!(result.Succeeded(), "{:?}", result.exception);
        eprintln!(
            "vm-basic-blocks-bench ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
}
#[cfg(test)]
mod background_module_tests {
    use super::*;
    struct EmptyHost;
    impl JavaScriptHostBindings for EmptyHost {
        fn GlobalNames(&self) -> Vec<String> {
            vec![]
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult::default()
        }
    }
    #[test]
    fn external_classic_preparation_preserves_owner_bindings_and_exceptions() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(runtime
            .Evaluate(&realm, "globalThis.counter=0;let existing=41;", "owner.js")
            .Succeeded());
        let prepare = |runtime: &mut QuickJsJavaScriptRuntime, source: &str, name: &str| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !runtime.PrepareClassicScript(&realm, source, name) {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        };
        let source = "counter++;globalThis.answer=existing+1;";
        prepare(&mut runtime, source, "external.js");
        assert!(runtime
            .Evaluate(
                &realm,
                "if(counter!==0)throw Error('prepare executed code')",
                "check.js"
            )
            .Succeeded());
        assert!(runtime.Evaluate(&realm, source, "external.js").Succeeded());
        assert!(runtime
            .Evaluate(
                &realm,
                "if(counter!==1||answer!==42)throw Error('owner bindings lost')",
                "check.js"
            )
            .Succeeded());
        let throwing = "counter++;throw new Error('compiled classic failure')";
        prepare(&mut runtime, throwing, "throwing.js");
        let error = runtime
            .Evaluate(&realm, throwing, "throwing.js")
            .exception
            .unwrap();
        assert!(error.message.contains("compiled classic failure"));
        assert_eq!(error.source_name, "throwing.js");
        assert!(runtime
            .Evaluate(
                &realm,
                "if(counter!==2)throw Error('script reran')",
                "check.js"
            )
            .Succeeded());
        let duplicate = "let existing=2;";
        prepare(&mut runtime, duplicate, "duplicate.js");
        assert!(!runtime
            .Evaluate(&realm, duplicate, "duplicate.js")
            .Succeeded());
        let malformed = "const = ;";
        prepare(&mut runtime, malformed, "syntax.js");
        assert!(!runtime.Evaluate(&realm, malformed, "syntax.js").Succeeded());
    }
    fn finish_background_compilation(
        runtime: &mut QuickJsJavaScriptRuntime,
        realm: &JavaScriptRealm,
        job: &mut JavaScriptModuleCompilationJob,
    ) -> Result<JavaScriptModuleCompilation, JavaScriptException> {
        let (sender, receiver) = std::sync::mpsc::channel();
        runtime.SetModuleCompilationWake(Some(std::sync::Arc::new(move || {
            let _ = sender.send(());
        })));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match runtime.PollCompileModule(realm, job) {
                JavaScriptModuleCompilationPoll::Ready(result) => return result,
                JavaScriptModuleCompilationPoll::Pending => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "compiler worker did not finish"
                    );
                    // The callback may have been installed after completion;
                    // bounded polling also handles that harmless race.
                    let _ = receiver.recv_timeout(std::time::Duration::from_millis(10));
                }
            }
        }
    }
    #[test]
    fn background_module_preserves_compile_only_import_meta_and_microtask_order() {
        let mut runtime = QuickJsJavaScriptRuntime::with_max_stack_size(8 * 1024 * 1024);
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(runtime
            .Evaluate(
                &realm,
                "globalThis.order=[];Promise.resolve().then(()=>order.push('queued'));",
                "classic.js"
            )
            .Succeeded());
        let mut job = runtime.BeginCompileModule(&realm,
            "order.push('module:'+import.meta.url);Promise.resolve().then(()=>order.push('module-job'));",
            "https://test/background.js").expect("compiler worker");
        let compiled = finish_background_compilation(&mut runtime, &realm, &mut job).unwrap();
        assert!(compiled.requests.is_empty());
        assert_eq!(
            runtime
                .Evaluate(&realm, "order.length", "check.js")
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_f64(),
            Some(0.0)
        );
        assert!(runtime
            .EvaluateCompiledModule(&realm, &compiled.module, "https://test/background.js", None)
            .Succeeded());
        assert!(runtime
            .Evaluate(
                &realm,
                "if(order.join('|')!=='module:https://test/background.js')throw Error(order);",
                "check.js"
            )
            .Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        assert!(runtime.Evaluate(&realm,
            "if(order.join('|')!=='module:https://test/background.js|queued|module-job')throw Error(order);", "check.js").Succeeded());
        assert!(matches!(
            runtime.PollCompileModule(&realm, &mut job),
            JavaScriptModuleCompilationPoll::Ready(Err(_))
        ));
    }
    #[test]
    fn background_module_metadata_survives_worker_heap_disposal_and_nested_throw() {
        let source = "const smile='😀';\nfunction inner(){\n const innerSmile='😀'; throw new Error('nested');\n}\nfunction outer(){inner()}\nglobalThis.throwNested=outer;";
        let name = "https://test/background-location.js";
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let ordinary = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let background = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let direct = runtime.CompileModule(&ordinary, source, name).unwrap();
        let mut job = runtime
            .BeginCompileModule(&background, source, name)
            .unwrap();
        let transferred =
            finish_background_compilation(&mut runtime, &background, &mut job).unwrap();
        // Outcome is sent only after the fresh worker runtime/context are freed.
        assert!(runtime
            .EvaluateCompiledModule(&ordinary, &direct.module, name, None)
            .Succeeded());
        assert!(runtime
            .EvaluateCompiledModule(&background, &transferred.module, name, None)
            .Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        // Module evaluation returns a promise. Invoke the installed nested
        // function at the same ordinary script boundary in both owner realms.
        let expected = runtime
            .Evaluate(&ordinary, "throwNested()", "nested-call.js")
            .exception
            .unwrap();
        let actual = runtime
            .Evaluate(&background, "throwNested()", "nested-call.js")
            .exception
            .unwrap();
        assert_eq!(
            (
                actual.kind,
                actual.message,
                actual.source_name,
                actual.line,
                actual.column,
                actual.source_line,
                actual.stack
            ),
            (
                expected.kind,
                expected.message,
                expected.source_name,
                expected.line,
                expected.column,
                expected.source_line,
                expected.stack
            )
        );
    }
    #[test]
    fn background_module_invalid_source_formats_error_in_modified_main_realm() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        assert!(runtime.Evaluate(&realm,
            "globalThis.formatted=0;Error.prototype.toString=function(){formatted++;return 'SyntaxError: owner-only';};",
            "setup.js").Succeeded());
        let mut job = runtime
            .BeginCompileModule(&realm, "export const = ;", "bad.js")
            .unwrap();
        let actual = finish_background_compilation(&mut runtime, &realm, &mut job)
            .err()
            .unwrap();
        assert_eq!(actual.message, "SyntaxError: owner-only");
        assert_eq!(
            runtime
                .Evaluate(&realm, "formatted", "check.js")
                .value
                .Implementation::<JsValue>()
                .unwrap()
                .as_f64(),
            Some(1.0)
        );
    }
    #[test]
    fn background_module_queue_cancellation_and_realm_identity_are_owned() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let other = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let mut jobs: Vec<_> = (0..8)
            .map(|i| {
                runtime
                    .BeginCompileModule(
                        &realm,
                        "globalThis.mustNotRun=true;export const x=1;",
                        &format!("queued-{i}.js"),
                    )
                    .unwrap()
            })
            .collect();
        assert!(matches!(
            runtime.PollCompileModule(&other, &mut jobs[0]),
            JavaScriptModuleCompilationPoll::Ready(Err(_))
        ));
        // Dropping queued work must not hydrate or evaluate it.
        jobs.remove(1);
        for mut job in jobs {
            assert!(finish_background_compilation(&mut runtime, &realm, &mut job).is_ok());
        }
        assert!(runtime
            .Evaluate(
                &realm,
                "if(typeof mustNotRun!=='undefined')throw Error('compile evaluated');",
                "check.js"
            )
            .Succeeded());
        let job = runtime
            .BeginCompileModule(&realm, "export let x=1;", "drop.js")
            .unwrap();
        drop(runtime);
        drop(job); // Runtime drop never joins the compiler thread.
    }
}

#[cfg(test)]
mod proxy_key_name_tests {
    use super::*;
    fn original_key_name(ctx: &mut Context, key: &Value) -> JsResult<Option<(String, HostSymbol)>> {
        let key = ctx.to_property_key(key)?;
        Ok(match key {
            PropKey::Str(s) => Some((s.to_string_lossy(), HostSymbol::kNone)),
            PropKey::Index(n) => Some((n.to_string(), HostSymbol::kNone)),
            PropKey::Sym(s) if same_symbol(&s, &ctx.well_known.get(WellKnownSymbol::Iterator)) => {
                Some((String::new(), HostSymbol::kIterator))
            }
            _ => None,
        })
    }
    #[test]
    fn proxy_key_names_match_original_conversion_for_canonical_and_generic_inputs() {
        let mut ctx = Context::new();
        for input in [
            "''",
            "'0'",
            "'00'",
            "'001'",
            "'-0'",
            "'2147483647'",
            "'4294967295'",
            "'😀'",
            "'\\ud800'",
            "'x\\u0000z'",
            "Symbol.iterator",
            "Symbol.toStringTag",
            "Symbol('x')",
            "0",
            "-0",
            "null",
            "true",
            "undefined",
        ] {
            let key = ctx.eval(input).unwrap();
            let expected = original_key_name(&mut ctx, &key).unwrap();
            let actual = key_name(&mut ctx, &key).unwrap();
            assert_eq!(actual, expected, "{input}");
        }
        let key=ctx.eval("globalThis.conversions=0;({[Symbol.toPrimitive](hint){conversions++;return 'converted';}})").unwrap();
        assert_eq!(
            original_key_name(&mut ctx, &key).unwrap(),
            key_name(&mut ctx, &key).unwrap()
        );
        assert_eq!(
            ctx.eval("conversions").unwrap().as_f64(),
            Some(2.0),
            "generic keys must retain conversion side effects"
        );
        let key = ctx
            .eval("({[Symbol.toPrimitive](){throw new RangeError('conversion failure');}})")
            .unwrap();
        let expected = original_key_name(&mut ctx, &key).err().unwrap();
        let actual = key_name(&mut ctx, &key).err().unwrap();
        assert_eq!(
            ctx.format_exception(&actual),
            ctx.format_exception(&expected)
        );
    }
    #[test]
    fn proxy_key_names_validate_foreign_strings_and_symbols_before_fast_path() {
        let mut source = Context::new();
        let mut target = Context::new();
        for input in ["'foreign'", "Symbol.iterator", "Symbol('foreign')"] {
            let key = source.eval(input).unwrap();
            let expected = original_key_name(&mut target, &key).err().unwrap();
            let actual = key_name(&mut target, &key).err().unwrap();
            assert_eq!(
                target.format_exception(&actual),
                target.format_exception(&expected),
                "{input}"
            );
        }
    }
}

#[cfg(test)]
mod host_record_snapshot_tests {
    use super::*;
    struct Host {
        calls: Rc<Cell<usize>>,
    }
    impl JavaScriptHostBindings for Host {
        fn GlobalNames(&self) -> Vec<String> {
            vec!["echo".into()]
        }
        fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
            if call.operation == HostOperation::kGet && call.member == "echo" {
                return HostResult {
                    value: HostValue::Method(HostMethodRef {
                        receiver: 0,
                        name: "echo".into(),
                    }),
                    ..Default::default()
                };
            }
            if call.operation == HostOperation::kCall && call.member == "echo" {
                self.calls.set(self.calls.get() + 1);
                assert!(
                    matches!(call.arguments.first(), Some(HostValue::JavaScriptValue(_))),
                    "ordinary snapshot callback arguments keep JS identity"
                );
                return HostResult {
                    value: call.arguments[0].clone(),
                    ..Default::default()
                };
            }
            HostResult {
                handled: false,
                ..Default::default()
            }
        }
    }
    #[test]
    fn owned_host_records_are_fresh_gc_values_with_data_fields_and_callback_identity() {
        let calls = Rc::new(Cell::new(0));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host {
            calls: calls.clone(),
        })));
        realm.WithImplementation::<QuickJsRealm,_>(|r| {
            r.context.eval("var inheritedSetterCalls=0; Object.defineProperty(Object.prototype,'width',{set(){inheritedSetterCalls++},configurable:true});").unwrap();
            let packet = HostValue::Record(vec![
                ("length".into(), HostValue::Number(1.0)),
                ("0".into(), HostValue::Record(vec![
                    ("x".into(), HostValue::Number(-0.0)),
                    ("width".into(), HostValue::Number(f64::INFINITY)),
                    ("nan".into(), HostValue::Number(f64::NAN)),
                    ("__proto__".into(), HostValue::Number(7.0)),
                    ("nul\0key".into(), HostValue::Number(11.0)),
                ])),
            ]);
            let wrappers = r.state.borrow().wrappers.len();
            let identities = r.state.borrow().wrapper_ids.len();
            let first = to_js(&mut r.context, &r.state, &packet).unwrap();
            let second = to_js(&mut r.context, &r.state, &packet).unwrap();
            assert!(!Value::strict_equals(&first, &second));
            assert!(matches!(from_js(&first, &r.context, &r.state), HostValue::JavaScriptValue(_)));
            let global = r.context.global();
            r.context.define_value(&global,"first",first,PropFlags::C_W_E);
            r.context.define_value(&global,"second",second,PropFlags::C_W_E);
            r.context.eval(r#"
                function check(v,m){if(!v)throw Error(m)}
                check(first!==second && first[0]!==second[0],'fresh nested identities');
                check(Object.getPrototypeOf(first)===Object.prototype,'ordinary snapshot');
                check(Object.is(first[0].x,-0) && first[0].width===Infinity && Number.isNaN(first[0].nan),'exact numeric values');
                check(first[0].__proto__===7 && first[0]['nul\0key']===11 && Object.getPrototypeOf(first[0])===Object.prototype,'own special names');
                const d=Object.getOwnPropertyDescriptor(first[0],'width');
                check(d.writable && d.configurable && d.enumerable && !d.get,'data descriptors');
                check(inheritedSetterCalls===0,'prototype setter never called');
                first[0].width=9;check(second[0].width===Infinity,'independent snapshot mutation');
                check(echo(first)===first,'callback roundtrip retains exact JS identity');
                delete Object.prototype.width;
            "#).unwrap();
            assert_eq!(r.state.borrow().wrappers.len(), wrappers);
            assert_eq!(r.state.borrow().wrapper_ids.len(), identities);
        }).unwrap();
        assert_eq!(calls.get(), 1);
    }
    #[test]
    fn owned_host_record_rejects_foreign_realm_values_without_registering_identity() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let host = || {
            Rc::new(RefCell::new(Host {
                calls: Rc::new(Cell::new(0)),
            }))
        };
        let first = runtime.CreateRealm(host());
        let second = runtime.CreateRealm(host());
        let foreign = second
            .WithImplementation::<QuickJsRealm, _>(|r| {
                JavaScriptValue::new_traced(r.context.eval("({foreign:true})").unwrap())
            })
            .unwrap();
        first
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let before = (
                    r.state.borrow().wrappers.len(),
                    r.state.borrow().wrapper_ids.len(),
                );
                let packet = HostValue::Record(vec![(
                    "foreign".into(),
                    HostValue::JavaScriptValue(foreign),
                )]);
                let error = to_js(&mut r.context, &r.state, &packet)
                    .expect_err("foreign ownership must fail");
                assert!(r.context.format_exception(&error).contains("another realm"));
                assert_eq!(
                    (
                        r.state.borrow().wrappers.len(),
                        r.state.borrow().wrapper_ids.len()
                    ),
                    before
                );
            })
            .unwrap();
    }
}

#[cfg(test)]
mod native_structured_clone_tests {
    use super::*;
    struct EmptyHost;
    impl JavaScriptHostBindings for EmptyHost {
        fn GlobalNames(&self) -> Vec<String> {
            Vec::new()
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult::default()
        }
    }
    #[test]
    fn clone_uses_native_slots_and_no_author_brand_or_collection_hooks() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let input=runtime.Evaluate(&realm,r#"(()=>{
            globalThis.cloneHookCalls=0;
            const input={map:new Map([[1,2]]),set:new Set([3]),date:new Date(123)};
            Object.defineProperty(input,Symbol.toStringTag,{get(){cloneHookCalls++;throw Error('brand')}});
            Map.prototype[Symbol.iterator]=Map.prototype.set=function(){cloneHookCalls++;throw Error('map')};
            Set.prototype[Symbol.iterator]=Set.prototype.add=function(){cloneHookCalls++;throw Error('set')};
            Date.prototype.getTime=function(){cloneHookCalls++;throw Error('date')};
            globalThis.Map=globalThis.Set=globalThis.Date=function(){cloneHookCalls++;throw Error('constructor')};
            return input;
        })()"#,"native-clone-input.js");
        assert!(input.Succeeded());
        let cloned = runtime.Clone(&realm, &HostValue::JavaScriptValue(input.value));
        assert!(cloned.Succeeded(), "{:?}", cloned.exception);
        let calls = runtime.Evaluate(&realm, "cloneHookCalls", "clone-hooks.js");
        assert_eq!(
            calls.value.Implementation::<JsValue>().unwrap().as_f64(),
            Some(0.0)
        );
    }
}
