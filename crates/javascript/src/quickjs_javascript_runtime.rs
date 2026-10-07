//! Pure Rust QuickJS backend for the engine-neutral browser host boundary.
#![allow(non_snake_case)]

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::javascript_runtime::*;
#[path = "quickjs_module_worker.rs"]
mod module_worker;
#[path = "quickjs_native.rs"]
mod native;
pub use native::Value as JsValue;
use native::*;

#[path = "quickjs_console.rs"]
mod console;
#[path = "quickjs_intl.rs"]
mod intl;
#[path = "quickjs_regexp.rs"]
mod regexp;

struct Wrapper {
    target: Gc<JsObject>,
    proxy: Gc<JsObject>,
    prototype_installed: bool,
}

struct State {
    host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    global: Value,
    wrappers: HashMap<HostObjectId, Wrapper>,
    // QuickJS object identity, scoped to its runtime. Forward entries strongly
    // retain proxies, so GC cannot recycle an address while indexed here.
    wrapper_ids: HashMap<(usize, usize), HostObjectId>,
    fallbacks: HashMap<String, Value>,
    reflect_get: Value,
    reflect_set: Value,
    // Realm-local callable; its per-call graph state remains inside the helper.
    clone_copier: Option<Value>,
    hooks: Rc<Hooks>,
}

#[derive(Default)]
struct Hooks {
    checkpoint_running: Cell<bool>,
    rejections: RefCell<Vec<(Gc<JsObject>, Value)>>,
    exceptions: RefCell<Vec<JavaScriptException>>,
}

struct Tracker(Rc<Hooks>);
impl RejectionTracker for Tracker {
    fn on_rejection(
        &mut self,
        _: &mut Context,
        promise: &Gc<JsObject>,
        reason: &Value,
        handled: bool,
    ) {
        let mut pending = self.0.rejections.borrow_mut();
        pending.retain(|(p, _)| p != promise);
        if !handled {
            pending.push((promise.clone(), reason.clone()));
        }
    }
}

pub struct QuickJsRealm {
    pub context: Context,
    state: Rc<RefCell<State>>,
}

impl Drop for QuickJsRealm {
    fn drop(&mut self) {
        // Host callables retain their state; release cached JS handles before
        // the context's heap is destroyed so that this does not form a cycle.
        let mut state = self.state.borrow_mut();
        state.wrapper_ids.clear();
        state.wrappers.clear();
        state.fallbacks.clear();
        state.global = Value::Undefined;
        state.reflect_get = Value::Undefined;
        state.reflect_set = Value::Undefined;
        state.clone_copier = None;
        state.hooks.rejections.borrow_mut().clear();
    }
}

pub struct QuickJsJavaScriptRuntime {
    realms: Vec<JavaScriptRealm>,
    native_stack_budget: usize,
    module_worker: Option<module_worker::Worker>,
    module_worker_unavailable: bool,
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
            realms: Vec::new(),
            native_stack_budget: 1024 * 1024,
            module_worker: None,
            module_worker_unavailable: false,
            module_compilation_wake: None,
        }
    }
}
impl QuickJsJavaScriptRuntime {
    pub fn new() -> Self {
        Self::default()
    }
    /// Configure the embedding's native stack allowance through QuickJS's
    /// public API. The caller must leave room for the host's own stack frames
    /// and select a budget below the actual stack available on its thread.
    /// The default remains 1 MiB for threads whose stack size is unknown.
    pub fn with_native_stack_budget(bytes: usize) -> Self {
        assert!(
            bytes > 0,
            "a native stack budget must retain overflow protection"
        );
        Self {
            realms: Vec::new(),
            native_stack_budget: bytes,
            module_worker: None,
            module_worker_unavailable: false,
            module_compilation_wake: None,
        }
    }
}

impl JavaScriptFunctionStorage for Gc<JsObject> {
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

fn from_js(value: &Value, context: &Context, state: &Rc<RefCell<State>>) -> HostValue {
    match value {
        Value::Undefined => HostValue::default(),
        Value::Null => HostValue::Null(Default::default()),
        Value::Bool(v) => HostValue::Boolean(*v),
        Value::Int(v) => HostValue::Number(*v as f64),
        Value::Float(v) => HostValue::Number(*v),
        Value::Str(v) => HostValue::String(v.to_string_lossy()),
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
                HostValue::JavaScriptValue(JavaScriptValue::new(value.clone()))
            }
        }
        _ => HostValue::JavaScriptValue(JavaScriptValue::new(value.clone())),
    }
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
            value: JavaScriptValue::new(value),
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
}
struct HostFunction {
    state: Rc<RefCell<State>>,
    action: Action,
}

fn invoke(ctx: &mut Context, state: &Rc<RefCell<State>>, call: &HostCall<'_>) -> HostResult {
    let host = state.borrow().host.clone();
    let continuation = host.borrow_mut().PrepareInvocation(call);
    let mut runtime = HostRuntime {
        context: ctx,
        state,
    };
    if let Some(run) = continuation {
        run(&mut runtime)
    } else {
        host.borrow_mut().InvokeWithRuntime(call, &mut runtime)
    }
}

fn host_result(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
    result: HostResult,
) -> JsResult<Value> {
    if let Some(e) = result.exception {
        Err(error(ctx, e))
    } else {
        to_js(ctx, state, &result.value)
    }
}

impl HostCallable for HostFunction {
    fn call(
        &self,
        ctx: &mut Context,
        _: &Value,
        args: &[Value],
        _: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        let state = &self.state;
        match &self.action {
            Action::Method(method) => {
                let values: Vec<_> = args.iter().map(|v| from_js(v, ctx, state)).collect();
                let answer = invoke(
                    ctx,
                    state,
                    &HostCall {
                        receiver: method.receiver,
                        operation: HostOperation::kCall,
                        member: &method.name,
                        symbol: HostSymbol::kNone,
                        arguments: &values,
                    },
                );
                host_result(ctx, state, answer)
            }
            Action::GlobalGet(name) => {
                let host = state.borrow().host.clone();
                let answer = host.borrow_mut().Invoke(&HostCall {
                    receiver: 0,
                    operation: HostOperation::kGet,
                    member: name,
                    symbol: HostSymbol::kNone,
                    arguments: &[],
                });
                if answer.handled || answer.exception.is_some() {
                    host_result(ctx, state, answer)
                } else {
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
                let host_value = from_js(&value, ctx, state);
                let answer = invoke(
                    ctx,
                    state,
                    &HostCall {
                        receiver: 0,
                        operation: HostOperation::kSet,
                        member: name,
                        symbol: HostSymbol::kNone,
                        arguments: &[host_value],
                    },
                );
                if let Some(e) = answer.exception {
                    return Err(error(ctx, e));
                }
                if !answer.handled {
                    state.borrow_mut().fallbacks.insert(name.clone(), value);
                }
                Ok(Value::Undefined)
            }
            Action::Get(id) | Action::Set(id) | Action::Has(id) => {
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
                    let set_value = set.then(|| from_js(&value, ctx, state));
                    let values = set_value.as_ref().map(std::slice::from_ref).unwrap_or(&[]);
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
                            arguments: values,
                        },
                    );
                    if let Some(e) = answer.exception {
                        return Err(error(ctx, e));
                    }
                    if answer.handled {
                        return if has || set {
                            Ok(Value::Bool(true))
                        } else {
                            to_js(ctx, state, &answer.value)
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
        .map(|w| (w.target.clone(), w.prototype_installed));
    let Some((target, false)) = entry else {
        return Ok(());
    };
    let host = state.borrow().host.clone();
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
    let value = match value {
        HostValue::Undefined(_) => Value::Undefined,
        HostValue::Null(_) => Value::Null,
        HostValue::Boolean(v) => Value::Bool(*v),
        HostValue::Number(v) => Value::number(*v),
        HostValue::String(v) => Value::Str(ctx.intern(v)),
        HostValue::Record(fields) => {
            let object = ctx.try_new_object()?;
            for (name, value) in fields {
                let value = to_js(ctx, state, value)?;
                ctx.define_data_value(&object, name, value, PropFlags::C_W_E)?;
            }
            Value::Object(object)
        }
        HostValue::JavaScriptValue(v) => v
            .Implementation::<Value>()
            .cloned()
            .ok_or_else(|| ctx.type_error("Value belongs to another engine"))?,
        HostValue::JavaScriptFunction(v) => Value::Object(
            v.Implementation::<Gc<JsObject>>()
                .cloned()
                .ok_or_else(|| ctx.type_error("Function belongs to another engine"))?,
        ),
        HostValue::Method(m) => function(ctx, state, Action::Method(m.clone()), &m.name, 0),
        HostValue::Object(o) if o.id == 0 => state.borrow().global.clone(),
        HostValue::Object(o) => {
            let existing = state.borrow().wrappers.get(&o.id).map(|w| w.proxy.clone());
            if let Some(proxy) = existing {
                install_prototype(ctx, state, o.id)?;
                return Ok(Value::Object(proxy));
            }
            let target = ctx.new_object();
            let handler = ctx.new_object();
            for (name, action, length) in [
                ("get", Action::Get(o.id), 3),
                ("set", Action::Set(o.id), 4),
                ("has", Action::Has(o.id), 2),
            ] {
                let callback = function(ctx, state, action, name, length);
                ctx.define_value(&handler, name, callback, PropFlags::C_W_E);
            }
            let proxy = ctx.new_proxy(target.clone(), handler)?;
            {
                let mut state = state.borrow_mut();
                state.wrapper_ids.insert(proxy.identity(), o.id);
                state.wrappers.insert(
                    o.id,
                    Wrapper {
                        target,
                        proxy: proxy.clone(),
                        prototype_installed: false,
                    },
                );
            }
            install_prototype(ctx, state, o.id)?;
            Value::Object(proxy)
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
}

impl JavaScriptRuntime for QuickJsJavaScriptRuntime {
    fn CreateRealm(&mut self, host: Rc<RefCell<dyn JavaScriptHostBindings>>) -> JavaScriptRealm {
        let profiling = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        let realm_started = profiling.then(std::time::Instant::now);
        let mut context = Context::with_native_stack_budget(self.native_stack_budget);
        if let Some(start) = realm_started {
            eprintln!(
                "javascript-realm-profile phase=context ms={:.3}",
                start.elapsed().as_secs_f64() * 1000.
            );
        }
        let services_started = profiling.then(std::time::Instant::now);
        let hooks = Rc::new(Hooks::default());
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
            global: Value::Object(global.clone()),
            wrappers: HashMap::new(),
            wrapper_ids: HashMap::new(),
            fallbacks: HashMap::new(),
            reflect_get,
            reflect_set,
            clone_copier: None,
            hooks,
        }));
        let names = host.borrow().GlobalNames();
        for name in names {
            let answer = host.borrow_mut().Invoke(&HostCall {
                receiver: 0,
                operation: HostOperation::kGet,
                member: &name,
                symbol: HostSymbol::kNone,
                arguments: &[],
            });
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
        let realm = JavaScriptRealm::new(QuickJsRealm { context, state });
        self.realms.push(realm.clone());
        realm
    }

    fn Evaluate(&mut self, realm: &JavaScriptRealm, source: &str, name: &str) -> JavaScriptResult {
        let mut trace = browser_tracing::span("javascript", "ScriptEvaluate");
        trace.set("source_bytes", source.len() as f64);
        if !self.realms.contains(realm) {
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
                let value = r.context.eval_named(source, SourceType::Script, name);
                result(&mut r.context, value, name)
            })
            .expect("invalid QuickJS realm")
    }

    fn EvaluateBootstrap(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        name: &str,
    ) -> JavaScriptResult {
        let mut trace = browser_tracing::span("javascript", "BootstrapEvaluate");
        trace.set("source_bytes", source.len() as f64);
        if !self.realms.contains(realm) {
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
        if !self.realms.contains(realm) {
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
                                module: JavaScriptValue::new(module),
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
        if !self.realms.contains(realm) || self.module_worker_unavailable {
            return None;
        }
        if self.module_worker.is_none() {
            let Some(mut worker) = module_worker::Worker::new(self.native_stack_budget) else {
                self.module_worker_unavailable = true;
                return None;
            };
            worker.set_wake(self.module_compilation_wake.clone());
            self.module_worker = Some(worker);
        }
        let worker =
            self.module_worker
                .as_ref()
                .unwrap()
                .begin(source, name, self.native_stack_budget);
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
        if !self.realms.contains(realm) {
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
                                    module: JavaScriptValue::new(module),
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
        if !self.realms.contains(realm) {
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
        if !self.realms.contains(realm) {
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
        if !self.realms.contains(realm) {
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
        if !self.realms.contains(realm) {
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
        if !self.realms.contains(realm) {
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
        for realm in &self.realms {
            realm
                .WithImplementation::<QuickJsRealm, _>(|r| checkpoint(&mut r.context, &r.state))
                .expect("invalid QuickJS realm");
        }
    }
    fn TakePendingExceptions(&mut self, realm: &JavaScriptRealm) -> Vec<JavaScriptException> {
        if !self.realms.contains(realm) {
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
    let value = (|| {
        let function = callback
            .Implementation::<Gc<JsObject>>()
            .ok_or_else(|| ctx.type_error("Invalid callback"))?;
        let args = arguments
            .iter()
            .map(|v| to_js(ctx, state, v))
            .collect::<JsResult<Vec<_>>>()?;
        let receiver = to_js(ctx, state, receiver)?;
        ctx.call(Value::Object(function.clone()), receiver, &args)
    })();
    trace.set("failed", u8::from(value.is_err()) as f64);
    result(ctx, value, "")
}

fn clone_value(ctx: &mut Context, state: &Rc<RefCell<State>>, value: &HostValue) -> HostResult {
    // Structured serialization is a browser service, independent of which
    // ECMAScript backend executes the graph-copy algorithm.
    fn is_proxy(ctx: &mut Context, _: &Value, args: &[Value], _: i32) -> JsResult<Value> {
        Ok(Value::Bool(
            args.first().is_some_and(|value| ctx.is_proxy(value)),
        ))
    }
    let answer = (|| {
        let input = to_js(ctx, state, value)?;
        let proxy_check = Value::Object(ctx.new_native_function(is_proxy, "", 1, 0));
        // Copy the handle out before executing JS: getters may reenter Clone.
        let cached = state.borrow().clone_copier.clone();
        let copier = match cached {
            Some(copier) => copier,
            None => {
                let copier = ctx.eval(include_str!("structured_clone.js"))?;
                state.borrow_mut().clone_copier = Some(copier.clone());
                copier
            }
        };
        // The unchanged helper creates its seen Map and output graph per call.
        ctx.call(copier, Value::Undefined, &[input, proxy_check])
    })();
    match answer {
        Ok(v) => HostResult {
            value: from_js(&v, ctx, state),
            ..Default::default()
        },
        Err(e) => HostResult {
            exception: Some(exception(ctx, &e, "")),
            ..Default::default()
        },
    }
}

fn enqueue(ctx: &mut Context, callback: &JavaScriptFunction) -> HostResult {
    let _trace = browser_tracing::span("javascript", "MicrotaskEnqueue");
    let Some(function) = callback.Implementation::<Gc<JsObject>>() else {
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
    if hooks.checkpoint_running.replace(true) {
        return;
    }
    let _trace = browser_tracing::span("javascript", "MicrotaskCheckpoint");
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
impl JavaScriptHostRuntime for HostRuntime<'_> {
    fn Call(
        &mut self,
        callback: &JavaScriptFunction,
        receiver: &HostValue,
        args: &[HostValue],
    ) -> JavaScriptResult {
        call(self.context, self.state, callback, receiver, args)
    }
    fn ToString(&mut self, value: &HostValue) -> HostResult {
        let converted = to_js(self.context, self.state, value)
            .and_then(|value| self.context.to_rust_string(&value));
        match converted {
            Ok(text) => HostResult {
                value: HostValue::String(text),
                ..Default::default()
            },
            Err(thrown) => HostResult {
                exception: Some(exception(self.context, &thrown, "")),
                ..Default::default()
            },
        }
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
                    "a" => ("https://test/a.js", "import {getB} from 'b'; export let a=2; export function change(){a=7}; export const read=()=>getB()+a; globalThis.order.push('a:'+import.meta.url);"),
                    "b" => ("https://test/b.js", "import {a} from 'a'; export function getB(){return a*3}; globalThis.order.push('b');"),
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
                        return Err(std::io::Error::other("not yet available"))
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
        assert_eq!(failed.exception.unwrap().message,
            "TypeError: Failed to load module retry from https://test/retry-root.js: not yet available");
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
                    }
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
    }
    impl JavaScriptHostBindings for Host {
        fn GlobalNames(&self) -> Vec<String> {
            vec![]
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult {
                handled: false,
                ..Default::default()
            }
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
        })));
        let js = realm
            .WithImplementation::<QuickJsRealm, _>(|r| {
                let expected = to_js(
                    &mut r.context,
                    &r.state,
                    &HostValue::Object(HostObjectRef { id: 17 }),
                )
                .unwrap();
                for id in 100..1100 {
                    to_js(
                        &mut r.context,
                        &r.state,
                        &HostValue::Object(HostObjectRef { id }),
                    )
                    .unwrap();
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
        let mut runtime = QuickJsJavaScriptRuntime::with_native_stack_budget(8 * 1024 * 1024);
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
                JavaScriptValue::new(r.context.eval("({foreign:true})").unwrap())
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
mod structured_clone_callable_cache_tests {
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
    fn realm_state(realm: &JavaScriptRealm) -> Rc<RefCell<State>> {
        realm
            .WithImplementation::<QuickJsRealm, _>(|r| r.state.clone())
            .unwrap()
    }
    #[test]
    fn clone_callable_is_lazy_reused_isolated_and_released() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let first = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let second = runtime.CreateRealm(Rc::new(RefCell::new(EmptyHost)));
        let first_state = realm_state(&first);
        let second_state = realm_state(&second);
        assert!(first_state.borrow().clone_copier.is_none());
        assert!(second_state.borrow().clone_copier.is_none());
        assert!(runtime.Clone(&first, &HostValue::Number(7.)).Succeeded());
        let identity = first_state
            .borrow()
            .clone_copier
            .as_ref()
            .unwrap()
            .as_object()
            .unwrap()
            .identity();
        for _ in 0..3 {
            assert!(runtime.Clone(&first, &HostValue::Number(9.)).Succeeded())
        }
        let executable = runtime.Evaluate(&first, "()=>{}", "clone-failure.js");
        assert!(runtime
            .Clone(&first, &HostValue::JavaScriptValue(executable.value))
            .exception
            .is_some());
        assert_eq!(
            first_state
                .borrow()
                .clone_copier
                .as_ref()
                .unwrap()
                .as_object()
                .unwrap()
                .identity(),
            identity
        );
        assert!(runtime.Clone(&second, &HostValue::Number(11.)).Succeeded());
        assert_ne!(
            second_state
                .borrow()
                .clone_copier
                .as_ref()
                .unwrap()
                .as_object()
                .unwrap()
                .identity(),
            identity
        );
        drop(first);
        drop(second);
        drop(runtime);
        assert!(first_state.borrow().clone_copier.is_none());
        assert!(second_state.borrow().clone_copier.is_none());
    }
}
