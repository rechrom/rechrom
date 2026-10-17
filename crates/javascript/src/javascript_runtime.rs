#![allow(non_snake_case, non_camel_case_types)]

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// cpp: javascript/javascript_runtime.h:17-17
pub type HostObjectId = u64;

// cpp: javascript/javascript_runtime.h:19-25
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JavaScriptUndefined;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JavaScriptNull;

// cpp: javascript/javascript_runtime.h:27-75
#[derive(Clone, Default)]
pub struct JavaScriptValue {
    storage: Option<Rc<dyn Any>>,
}

impl JavaScriptValue {
    pub fn new<T: Any>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(storage)),
        }
    }

    pub fn IsValid(&self) -> bool {
        self.storage.is_some()
    }

    pub fn Implementation<T: Any>(&self) -> Option<&T> {
        self.storage.as_ref()?.downcast_ref::<T>()
    }
}

impl PartialEq for JavaScriptValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Clone, Default)]
pub struct JavaScriptFunction {
    storage: Option<Rc<dyn JavaScriptFunctionStorage>>,
}

pub trait JavaScriptFunctionStorage: Any {
    fn AsAny(&self) -> &dyn Any;
    fn IsSameFunction(&self, other: &dyn JavaScriptFunctionStorage) -> bool;
    fn Downgrade(&self) -> WeakJavaScriptFunction;
}

/// A non-owning callback identity. Its native owner must publish the actual
/// function edge through HostObjectTraceSource before releasing a strong handle.
#[derive(Clone, Default)]
pub struct WeakJavaScriptFunction {
    storage: Option<Rc<dyn WeakJavaScriptFunctionStorage>>,
}
pub trait WeakJavaScriptFunctionStorage: Any {
    fn Upgrade(&self) -> Option<JavaScriptFunction>;
}
impl WeakJavaScriptFunction {
    pub fn new<T: WeakJavaScriptFunctionStorage>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(storage)),
        }
    }
    pub fn Upgrade(&self) -> Option<JavaScriptFunction> {
        self.storage.as_ref()?.Upgrade()
    }
}
impl std::fmt::Debug for JavaScriptFunction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaScriptFunction")
            .field("valid", &self.IsValid())
            .finish_non_exhaustive()
    }
}

impl JavaScriptFunction {
    pub fn new<T: JavaScriptFunctionStorage>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(storage)),
        }
    }

    pub fn IsValid(&self) -> bool {
        self.storage.is_some()
    }
    pub fn Downgrade(&self) -> WeakJavaScriptFunction {
        self.storage
            .as_ref()
            .map_or_else(WeakJavaScriptFunction::default, |storage| {
                storage.Downgrade()
            })
    }

    pub fn Implementation<T: Any>(&self) -> Option<&T> {
        self.storage.as_ref()?.AsAny().downcast_ref::<T>()
    }
}

impl PartialEq for JavaScriptFunction {
    fn eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b) || a.IsSameFunction(b.as_ref()),
            _ => false,
        }
    }
}

#[derive(Clone, Default)]
pub struct JavaScriptRealm {
    storage: Option<Rc<RefCell<dyn Any>>>,
}

// Non-owning source runtime binding. The engine owns the realm; a Window
// retains only its validity, avoiding Window -> realm -> host -> Window cycles.
#[derive(Clone, Default)]
pub struct WeakJavaScriptRealm {
    storage: Option<std::rc::Weak<RefCell<dyn Any>>>,
}
impl WeakJavaScriptRealm {
    pub fn IsValid(&self) -> bool {
        self.storage.as_ref().is_some_and(|s| s.strong_count() != 0)
    }
}
impl JavaScriptRealm {
    pub fn Downgrade(&self) -> WeakJavaScriptRealm {
        WeakJavaScriptRealm {
            storage: self.storage.as_ref().map(Rc::downgrade),
        }
    }
    pub fn new<T: Any>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(RefCell::new(storage))),
        }
    }

    pub fn IsValid(&self) -> bool {
        self.storage.is_some()
    }

    pub fn WithImplementation<T: Any, R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let storage = self.storage.as_ref()?;
        let mut storage = storage.borrow_mut();
        Some(f(storage.downcast_mut::<T>()?))
    }
}

impl PartialEq for JavaScriptRealm {
    fn eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

// cpp: javascript/javascript_runtime.h:77-115
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostObjectRef {
    pub id: HostObjectId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostMethodRef {
    pub receiver: HostObjectId,
    pub name: String,
}

/// An ordinary native function. The callback receives the actual JavaScript
/// `this`, followed by captured data and call arguments. Captured values must
/// be owned and traced by the JavaScript engine, just like closure captures.
/// A bound callback receiver is retained through the same engine-owned graph.
#[derive(Clone)]
pub struct HostFunctionBinding {
    pub name: String,
    pub length: u32,
    pub callback: HostMethodRef,
    pub data: Vec<HostValue>,
}

#[derive(Clone)]
pub enum HostValue {
    Undefined(JavaScriptUndefined),
    Null(JavaScriptNull),
    Boolean(bool),
    Number(f64),
    String(String),
    /// Immutable binary payload materialized as an ArrayBuffer in the target
    /// realm.  This keeps Fetch/XHR bytes lossless across the host boundary.
    Bytes(Arc<[u8]>),
    Object(HostObjectRef),
    // Owned internal data snapshot; materialized in the current realm,
    // without persistent host identity or proxy/getter registration.
    Record(Vec<(String, HostValue)>),
    Method(HostMethodRef),
    JavaScriptValue(JavaScriptValue),
    JavaScriptFunction(JavaScriptFunction),
}

impl Default for HostValue {
    fn default() -> Self {
        Self::Undefined(JavaScriptUndefined)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HostOperation {
    #[default]
    kGet,
    kSet,
    kCall,
    kConstruct,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HostSymbol {
    #[default]
    kNone,
    kIterator,
}

pub struct HostCall<'a> {
    pub receiver: HostObjectId,
    pub operation: HostOperation,
    pub member: &'a str,
    pub symbol: HostSymbol,
    pub arguments: &'a [HostValue],
}

// cpp: javascript/javascript_runtime.h:117-197
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JavaScriptExceptionKind {
    kSyntaxError,
    kTypeError,
    kRangeError,
    #[default]
    kRuntimeError,
}

#[derive(Clone, Debug, Default)]
pub struct JavaScriptException {
    pub kind: JavaScriptExceptionKind,
    pub message: String,
    pub source_name: String,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
    pub stack: String,
}

pub struct HostResult {
    pub value: HostValue,
    pub exception: Option<JavaScriptException>,
    pub handled: bool,
}

impl Default for HostResult {
    fn default() -> Self {
        Self {
            value: HostValue::default(),
            exception: None,
            handled: true,
        }
    }
}

impl HostResult {
    pub fn Succeeded(&self) -> bool {
        self.exception.is_none()
    }

    pub fn Failure(kind: JavaScriptExceptionKind, message: impl Into<String>) -> Self {
        Self {
            exception: Some(JavaScriptException {
                kind,
                message: message.into(),
                ..Default::default()
            }),
            handled: true,
            ..Default::default()
        }
    }
}

pub type HostContinuation = Box<dyn FnOnce(&mut dyn JavaScriptHostRuntime) -> HostResult>;

/// Immutable native ownership facts consumed on the realm's owner thread.
/// Slots name native fields, not JS properties; roots name actual host activity.
/// The host publishes changes before reentering JS. GC never calls the source.
#[derive(Clone, Debug)]
pub enum HostObjectTraceChange {
    Object {
        id: HostObjectId,
    },
    /// Replace an owner's native reference vector (for example a static list).
    ReferenceList {
        owner: HostObjectId,
        targets: std::rc::Rc<[HostObjectId]>,
    },
    Reference {
        owner: HostObjectId,
        slot: u64,
        target: Option<HostObjectId>,
    },
    /// Transfer a callback edge to an existing native owner. This temporary
    /// strong handle is released after the engine commits the traced edge.
    FunctionReference {
        owner: HostObjectId,
        slot: u64,
        function: Option<JavaScriptFunction>,
    },
    Root {
        id: HostObjectId,
        reason: u64,
        rooted: bool,
    },
    /// Native storage was retired after Released. IDs must not be reused;
    /// owners remove incoming native references before publishing this fact.
    Retire {
        id: HostObjectId,
    },
}
pub trait HostObjectTraceSource {
    fn TakeChanges(&self) -> Vec<HostObjectTraceChange>;
    /// Owner-thread notification, never a GC callback. Owners may retire
    /// ephemeral host records; native DOM storage remains with its own owner.
    fn Released(&self, _ids: &[HostObjectId]) {}
}

pub trait JavaScriptHostBindings {
    /// None preserves ownership for hosts which have not supplied native
    /// lifetime facts. The JavaScript backend never invents a DOM graph.
    fn ObjectTraceSource(&self) -> Option<std::rc::Rc<dyn HostObjectTraceSource>> {
        None
    }

    // Prototype: prepare under a short host borrow; execute after releasing it.
    fn PrepareInvocation(&mut self, _call: &HostCall<'_>) -> Option<HostContinuation> {
        None
    }

    fn GlobalNames(&self) -> Vec<String>;
    fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult;
    // Rust reborrow adapter for a host calling its bound engine while that
    // engine is already executing a host call. Avoids aliasing a runtime or
    // reborrowing the realm's RefCell. The services operate in the current realm.
    fn InvokeWithRuntime(
        &mut self,
        call: &HostCall<'_>,
        _runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        self.Invoke(call)
    }
    /// True when installed JavaScript descriptors implement all property
    /// semantics. Such host wrappers are ordinary objects; indexed/named or
    /// otherwise intercepted objects keep the engine's host traps.
    fn UsesOrdinaryProperties(&self, _id: HostObjectId) -> bool {
        false
    }
    fn PrototypeFor(&mut self, _id: HostObjectId) -> HostResult {
        HostResult {
            handled: false,
            ..Default::default()
        }
    }
}

pub trait JavaScriptHostRuntime {
    fn CreateHostFunction(&mut self, binding: &HostFunctionBinding) -> HostResult;
    /// Report an author exception through the realm's existing error channel,
    /// without throwing it from a host algorithm that invokes callbacks.
    fn ReportException(&mut self, value: &HostValue) -> HostResult;
    /// Native object classification used by Web IDL sequence conversion.
    fn IsProxy(&mut self, value: &HostValue) -> bool;
    // The current context is already executing a host continuation. Source
    // Page::Checkpoint must drain/report without borrowing its realm again.
    fn PerformMicrotaskCheckpoint(&mut self) -> Vec<JavaScriptException>;
    fn Call(
        &mut self,
        function: &JavaScriptFunction,
        receiver: &HostValue,
        arguments: &[HostValue],
    ) -> JavaScriptResult;
    /// Web IDL string conversion; called after the host borrow is released.
    fn ToString(&mut self, value: &HostValue) -> HostResult;
    fn Clone(&mut self, value: &HostValue) -> HostResult;
    fn EnqueueMicrotask(&mut self, callback: &JavaScriptFunction) -> HostResult;
}

#[derive(Default)]
pub struct JavaScriptResult {
    pub value: JavaScriptValue,
    pub exception: Option<JavaScriptException>,
}

impl JavaScriptResult {
    pub fn Succeeded(&self) -> bool {
        self.exception.is_none()
    }

    pub fn Failure(
        kind: JavaScriptExceptionKind,
        message: impl Into<String>,
        source_name: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            exception: Some(JavaScriptException {
                kind,
                message: message.into(),
                source_name: source_name.into(),
                line,
                column,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct JavaScriptModuleSource {
    pub url: String,
    pub source: String,
    /// A record compiled in this realm. Source-only embedders may leave it None.
    pub module: Option<JavaScriptValue>,
}

pub trait JavaScriptModuleResolver {
    /// Resolve an import against prepared modules. This callback must not fetch
    /// resources or wait for network: graph loading belongs to the embedder.
    fn ResolveModule(
        &self,
        specifier: &str,
        referrer_url: &str,
    ) -> std::io::Result<Option<JavaScriptModuleSource>>;

    /// Start an asynchronously evaluated `import()` request without waiting
    /// for transport. The returned URL is a stable key for polling.
    fn RequestDynamicModule(
        &self,
        _specifier: &str,
        _referrer_url: &str,
    ) -> std::io::Result<Option<String>> {
        Ok(None)
    }

    /// Return `None` while the requested graph is still loading.
    fn PollDynamicModule(&self, _url: &str) -> std::io::Result<Option<JavaScriptModuleSource>> {
        Ok(None)
    }
}

/// Engine-owner-thread job handle. Only owned compiler payload crosses threads.
pub struct JavaScriptModuleCompilationJob {
    storage: Box<dyn Any>,
}
impl JavaScriptModuleCompilationJob {
    pub fn new<T: Any>(storage: T) -> Self {
        Self {
            storage: Box::new(storage),
        }
    }
    pub fn ImplementationMut<T: Any>(&mut self) -> Option<&mut T> {
        self.storage.downcast_mut()
    }
}
pub type JavaScriptModuleCompilationWake = std::sync::Arc<dyn Fn() + Send + Sync>;
pub enum JavaScriptModuleCompilationPoll {
    Pending,
    Ready(Result<JavaScriptModuleCompilation, JavaScriptException>),
}

pub struct JavaScriptModuleCompilation {
    pub module: JavaScriptValue,
    pub requests: Vec<String>,
}

// cpp: javascript/javascript_runtime.h:215-267
/// Read-only runtime accounting. Fields overlap; this is not an additive RSS total.
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaScriptMemoryDiagnostics {
    pub malloc_bytes: i64,
    pub used_bytes: i64,
    pub objects: i64,
    pub strings: i64,
    pub string_bytes: i64,
    pub atoms: i64,
    pub properties: i64,
    pub property_bytes: i64,
    pub shapes: i64,
    pub function_bytes: i64,
    pub code_bytes: i64,
    pub array_elements: i64,
    pub binary_bytes: i64,
    pub host_wrappers: usize,
    pub host_identities: usize,
    pub host_fallbacks: usize,
}
pub trait JavaScriptRuntime {
    fn MemoryDiagnostics(&self, _realm: &JavaScriptRealm) -> Option<JavaScriptMemoryDiagnostics> {
        None
    }
    /// True when the engine's cycle collector should be scheduled by the
    /// embedding. Engines with an internal collector keep the default false.
    fn GarbageCollectionRequested(&self) -> bool {
        false
    }
    /// Runs an embedding-scheduled cycle collection on the realm owner.
    fn CollectGarbage(&mut self) {}
    fn CreateRealm(&mut self, host: Rc<RefCell<dyn JavaScriptHostBindings>>) -> JavaScriptRealm;
    fn Evaluate(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        source_name: &str,
    ) -> JavaScriptResult;
    /// Prepare an external classic script without executing it. False owns
    /// pending compiler work; the host returns to its event loop and retries
    /// after the compilation wake. Evaluate still runs on this realm owner.
    /// Other backends retain their ordinary synchronous Evaluate behavior.
    fn PrepareClassicScript(
        &mut self,
        _realm: &JavaScriptRealm,
        _source: &str,
        _source_name: &str,
    ) -> bool {
        true
    }
    /// Trusted embedder initialization only; page-provided source uses Evaluate.
    fn EvaluateBootstrap(
        &mut self,
        realm: &JavaScriptRealm,
        source: &str,
        source_name: &str,
    ) -> JavaScriptResult {
        self.Evaluate(realm, source, source_name)
    }
    fn SupportsModules(&self) -> bool {
        false
    }
    /// Compile without resolving imports or executing user code. The record is
    /// realm-owned and can be retained across asynchronous dependency loading.
    fn CompileModule(
        &mut self,
        _realm: &JavaScriptRealm,
        _source: &str,
        source_name: &str,
    ) -> Result<JavaScriptModuleCompilation, JavaScriptException> {
        Err(JavaScriptResult::Failure(
            JavaScriptExceptionKind::kRuntimeError,
            "Module compilation is unavailable",
            source_name,
            0,
            0,
        )
        .exception
        .unwrap())
    }
    /// None preserves the existing synchronous compiler for other engines.
    /// Some owns actual pending work; the caller must poll until real Ready.
    fn BeginCompileModule(
        &mut self,
        _realm: &JavaScriptRealm,
        _source: &str,
        _source_name: &str,
    ) -> Option<JavaScriptModuleCompilationJob> {
        None
    }
    fn PollCompileModule(
        &mut self,
        _realm: &JavaScriptRealm,
        _job: &mut JavaScriptModuleCompilationJob,
    ) -> JavaScriptModuleCompilationPoll {
        JavaScriptModuleCompilationPoll::Ready(Err(JavaScriptResult::Failure(
            JavaScriptExceptionKind::kRuntimeError,
            "Module compilation job is unavailable",
            "",
            0,
            0,
        )
        .exception
        .unwrap()))
    }
    /// Configure compile-only execution before submitting scripts. None keeps
    /// compilation on the realm owner; the injected executor may also be inline.
    fn SetCompilationExecutor(
        &mut self,
        _executor: Option<std::sync::Arc<dyn executor::Executor>>,
    ) {
    }
    /// A worker may invoke only this Send + Sync host work notification; it
    /// must never borrow a realm, run JS, or run a microtask in the callback.
    fn SetModuleCompilationWake(&mut self, _wake: Option<JavaScriptModuleCompilationWake>) {}
    fn EvaluateCompiledModule(
        &mut self,
        _realm: &JavaScriptRealm,
        _module: &JavaScriptValue,
        source_name: &str,
        _resolver: Option<Rc<dyn JavaScriptModuleResolver>>,
    ) -> JavaScriptResult {
        JavaScriptResult::Failure(
            JavaScriptExceptionKind::kRuntimeError,
            "Compiled module evaluation is unavailable",
            source_name,
            0,
            0,
        )
    }
    fn EvaluateModule(
        &mut self,
        _realm: &JavaScriptRealm,
        _source: &str,
        source_name: &str,
        _loader: Option<Rc<dyn JavaScriptModuleResolver>>,
    ) -> JavaScriptResult {
        JavaScriptResult::Failure(
            JavaScriptExceptionKind::kRuntimeError,
            "ECMAScript modules are unavailable",
            source_name,
            0,
            0,
        )
    }
    fn Call(
        &mut self,
        realm: &JavaScriptRealm,
        function: &JavaScriptFunction,
        receiver: &HostValue,
        arguments: &[HostValue],
    ) -> JavaScriptResult;
    fn Clone(&mut self, _realm: &JavaScriptRealm, _value: &HostValue) -> HostResult {
        HostResult::Failure(
            JavaScriptExceptionKind::kTypeError,
            "This engine does not support structured serialization",
        )
    }
    fn EnqueueMicrotask(
        &mut self,
        _realm: &JavaScriptRealm,
        _function: &JavaScriptFunction,
    ) -> HostResult {
        HostResult::Failure(
            JavaScriptExceptionKind::kRuntimeError,
            "Microtask enqueue is unavailable",
        )
    }
    fn PerformMicrotaskCheckpoint(&mut self);
    fn TakePendingExceptions(&mut self, _realm: &JavaScriptRealm) -> Vec<JavaScriptException> {
        Vec::new()
    }
}
