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
    weak_factory: Option<fn(&dyn Any) -> WeakJavaScriptValue>,
}

/// A script value held by a traced native field. The owner publishes a
/// ValueReference edge before relinquishing the strong temporary handle.
#[derive(Clone, Default)]
pub struct WeakJavaScriptValue {
    storage: Option<Rc<dyn WeakJavaScriptValueStorage>>,
}
pub trait WeakJavaScriptValueStorage: Any {
    fn Upgrade(&self) -> Option<JavaScriptValue>;
}
pub trait JavaScriptValueStorage: Any {
    fn Downgrade(&self) -> WeakJavaScriptValue;
}
impl WeakJavaScriptValue {
    pub fn new<T: WeakJavaScriptValueStorage>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(storage)),
        }
    }
    pub fn Upgrade(&self) -> Option<JavaScriptValue> {
        self.storage.as_ref()?.Upgrade()
    }
}

impl JavaScriptValue {
    pub fn new<T: Any>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(storage)),
            weak_factory: None,
        }
    }

    pub fn new_traced<T: JavaScriptValueStorage>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(storage)),
            weak_factory: Some(|value| {
                value
                    .downcast_ref::<T>()
                    .expect("script value storage")
                    .Downgrade()
            }),
        }
    }
    pub fn Downgrade(&self) -> Option<WeakJavaScriptValue> {
        Some((self.weak_factory?)(self.storage.as_deref()?))
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
impl std::fmt::Debug for JavaScriptValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaScriptValue")
            .field("valid", &self.IsValid())
            .finish_non_exhaustive()
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

/// A non-owning callback identity. Its host owner must publish the actual
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
    native_microtasks: Option<Rc<dyn JavaScriptAgentMicrotasks>>,
    host_runtime_data: Option<Rc<dyn Any>>,
}

pub type JavaScriptNativeMicrotask = Box<dyn FnOnce(Option<&mut dyn JavaScriptHostRuntime>)>;
#[derive(Clone, Copy, Debug)]
pub enum NativeMicrotaskEnqueueError {
    RealmDestroyed,
    AllocationFailed,
}
/// The document Agent queue. A realm binds its Promise queue; a scriptless
/// Agent has only native delegates and lends no runtime at its checkpoint.
/// Enqueuing never executes script or borrows the enclosing ScriptEngine.
pub trait JavaScriptAgentMicrotasks {
    fn Enqueue(&self, task: JavaScriptNativeMicrotask) -> Result<(), NativeMicrotaskEnqueueError>;
}

// Non-owning source runtime binding. The engine owns the realm; a Window
// retains only its validity, avoiding Window -> realm -> host -> Window cycles.
#[derive(Clone, Default)]
pub struct WeakJavaScriptRealm {
    storage: Option<std::rc::Weak<RefCell<dyn Any>>>,
    native_microtasks: Option<std::rc::Weak<dyn JavaScriptAgentMicrotasks>>,
    host_runtime_data: Option<std::rc::Weak<dyn Any>>,
}
impl WeakJavaScriptRealm {
    pub fn IsValid(&self) -> bool {
        self.storage.as_ref().is_some_and(|s| s.strong_count() != 0)
    }
    /// Upgrade the original Context identity, without selecting the frame's
    /// current realm. Its native Agent queue is the same shared queue.
    pub fn Upgrade(&self) -> Option<JavaScriptRealm> {
        Some(JavaScriptRealm {
            storage: Some(self.storage.as_ref()?.upgrade()?),
            host_runtime_data: self
                .host_runtime_data
                .as_ref()
                .and_then(std::rc::Weak::upgrade),
            native_microtasks: match &self.native_microtasks {
                Some(queue) => Some(queue.upgrade()?),
                None => None,
            },
        })
    }
}
impl JavaScriptRealm {
    pub fn Downgrade(&self) -> WeakJavaScriptRealm {
        WeakJavaScriptRealm {
            storage: self.storage.as_ref().map(Rc::downgrade),
            native_microtasks: self.native_microtasks.as_ref().map(Rc::downgrade),
            host_runtime_data: self.host_runtime_data.as_ref().map(Rc::downgrade),
        }
    }
    pub fn new<T: Any>(storage: T) -> Self {
        Self {
            storage: Some(Rc::new(RefCell::new(storage))),
            native_microtasks: None,
            host_runtime_data: None,
        }
    }
    pub fn WithNativeMicrotasks<T: Any>(
        storage: T,
        queue: Rc<dyn JavaScriptAgentMicrotasks>,
    ) -> Self {
        Self {
            storage: Some(Rc::new(RefCell::new(storage))),
            native_microtasks: Some(queue),
            host_runtime_data: None,
        }
    }
    pub(crate) fn WithHostRuntimeData(mut self, data: Rc<dyn Any>) -> Self {
        self.host_runtime_data = Some(data);
        self
    }
    pub(crate) fn HostRuntimeData<T: Any>(&self) -> Option<&T> {
        self.host_runtime_data.as_deref()?.downcast_ref()
    }
    pub fn NativeMicrotasks(&self) -> Option<Rc<dyn JavaScriptAgentMicrotasks>> {
        self.native_microtasks.clone()
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

/// An ordinary host function. The callback receives the actual JavaScript
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
    /// JavaScript/WTF-16 code units. Use for strings containing lone surrogates.
    /// Ordinary UTF-8 callers may continue constructing String.
    String16(Rc<[u16]>),
    /// Immutable binary payload materialized as an ArrayBuffer in the target
    /// realm.  This keeps Fetch/XHR bytes lossless across the host boundary.
    Bytes(Arc<[u8]>),
    Object(HostObjectRef),
    // Owned internal data snapshot; materialized in the current realm,
    // without persistent host identity or proxy/getter registration.
    Record(Vec<(String, HostValue)>),
    /// Web IDL sequence materialized with the realm's native Array creation.
    Array(Vec<HostValue>),
    /// Materialize the value while retaining its existing native lifetime
    /// owner. Used to transfer MutationRecord node roots into JS atomically.
    Retained(Rc<dyn Any>, Box<HostValue>),
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
    /// Transient original author exception. The current invocation consumes this
    /// rooted engine value; it is never serialized or installed as a realm root.
    pub thrown: Option<HostValue>,
    pub exception: Option<JavaScriptException>,
    pub handled: bool,
}

impl Default for HostResult {
    fn default() -> Self {
        Self {
            value: HostValue::default(),
            thrown: None,
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

/// One native host-object namespace within an existing Context. Native ids
/// are local to their owning bindings; wrapper identity never uses an id from
/// another Document's host. This is an engine handle, not serialized state.
#[derive(Clone)]
pub struct JavaScriptHostObjectNamespace {
    storage: Rc<dyn Any>,
}
#[derive(Clone, Default)]
pub struct WeakJavaScriptHostObjectNamespace {
    storage: Option<std::rc::Weak<dyn Any>>,
}
/// A native wrapper crossing a host-object namespace boundary. The transfer
/// carries the original binding cell and native owner; receivers never look up
/// an equal numeric id in their own document or reconstitute native state.
pub struct JavaScriptNativeObjectReference {
    namespace: WeakJavaScriptHostObjectNamespace,
    native: Rc<dyn Any>,
    binding: JavaScriptValue,
}
impl JavaScriptNativeObjectReference {
    pub(crate) fn new(
        namespace: WeakJavaScriptHostObjectNamespace,
        native: Rc<dyn Any>,
        binding: JavaScriptValue,
    ) -> Self {
        Self {
            namespace,
            native,
            binding,
        }
    }
    pub fn Namespace(&self) -> &WeakJavaScriptHostObjectNamespace {
        &self.namespace
    }
    pub fn NativeOwner<T: Any>(&self) -> Option<Rc<T>> {
        self.native.clone().downcast().ok()
    }
    pub fn BindingCell(&self) -> &JavaScriptValue {
        &self.binding
    }
}
impl JavaScriptHostObjectNamespace {
    pub(crate) fn new<T: Any>(storage: Rc<T>) -> Self {
        Self { storage }
    }
    pub(crate) fn ImplementationOwner<T: Any>(&self) -> Option<Rc<T>> {
        self.storage.clone().downcast().ok()
    }
    pub fn Downgrade(&self) -> WeakJavaScriptHostObjectNamespace {
        WeakJavaScriptHostObjectNamespace {
            storage: Some(Rc::downgrade(&self.storage)),
        }
    }
    pub fn Wrap(&self, value: HostValue) -> HostValue {
        HostValue::Retained(Rc::new(self.Downgrade()), Box::new(value))
    }
}
impl WeakJavaScriptHostObjectNamespace {
    pub fn Upgrade(&self) -> Option<JavaScriptHostObjectNamespace> {
        Some(JavaScriptHostObjectNamespace {
            storage: self.storage.as_ref()?.upgrade()?,
        })
    }
}
/// Canonical native owner identity carried by a retained cross-document
/// reference. This is not a document token or a VM/object-local numeric id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HostObjectOwnerIdentity(pub u128);
pub trait JavaScriptHostObjectNamespaceScope {}

/// Immutable host ownership facts consumed on the realm's owner thread.
/// Slots name host fields, not JS properties; roots name actual host activity.
/// The host publishes changes before reentering JS. GC never calls the source.
#[derive(Clone, Debug)]
pub enum HostObjectTraceChange {
    Object {
        id: HostObjectId,
    },
    /// Replace an owner's host reference vector (for example a static list).
    ReferenceList {
        owner: HostObjectId,
        targets: std::rc::Rc<[HostObjectId]>,
    },
    Reference {
        owner: HostObjectId,
        slot: u64,
        target: Option<HostObjectId>,
    },
    /// Clear a generated native-field group without walking the source owner
    /// again. Other native fields and the backend wrapper edge are retained.
    ClearReferenceRange {
        owner: HostObjectId,
        first_slot: u64,
        end_slot: u64,
    },
    /// Transfer a callback edge to an existing host owner. This temporary
    /// strong handle is released after the engine commits the traced edge.
    FunctionReference {
        owner: HostObjectId,
        slot: u64,
        function: Option<JavaScriptFunction>,
    },
    /// Arbitrary script value held in a native MemberScriptValue field. The
    /// engine installs a real GC edge, then releases this temporary handle.
    ValueReference {
        owner: HostObjectId,
        slot: u64,
        value: Option<JavaScriptValue>,
    },
    /// Conditional native ownership: retaining owner does not retain key.
    /// Target is retained only while key is live, matching WeakMember-key
    /// HeapHashMap/ephemeron semantics (not a weak-key Rust refcount probe).
    Ephemeron {
        owner: HostObjectId,
        slot: u64,
        entry: Option<(HostObjectId, HostObjectId)>,
    },
    Root {
        id: HostObjectId,
        reason: u64,
        rooted: bool,
    },
    /// Host storage was retired after Released. IDs must not be reused;
    /// owners remove incoming host references before publishing this fact.
    Retire {
        id: HostObjectId,
    },
}
pub trait HostObjectTraceSource {
    fn TakeChanges(&self) -> Vec<HostObjectTraceChange>;
    /// Owner-thread notification, never a GC callback. Owners may retire
    /// ephemeral host records; host DOM storage remains with its own owner.
    fn Released(&self, _ids: &[HostObjectId]) {}
}

/// Immutable ABI metadata. IDL `any` retains the original JS object, including
/// ArrayBuffer; byte-consuming interfaces keep the existing Bytes projection.
#[derive(Clone, Copy, Debug, Default)]
pub enum HostArgumentPolicy {
    #[default]
    Values,
    Identity,
    /// Preserve the original VM value except that native wrapper arguments
    /// additionally lend their original typed owner for Web IDL conversion.
    NativeIdentity,
    Forwarded {
        receiver: usize,
        member: usize,
        arguments: usize,
    },
}
pub trait JavaScriptHostBindings {
    /// Generated wrapper conversion lends the exact native owner as opaque
    /// identity. The JavaScript engine neither interprets nor mutates it.
    fn NativeObject(&self, _id: HostObjectId) -> Option<Rc<dyn Any>> {
        None
    }
    fn NativeObjectOwnerIdentity(&self) -> Option<HostObjectOwnerIdentity> {
        None
    }
    fn DocumentHostObjectId(&self) -> Option<HostObjectId> {
        None
    }
    fn IsWindowHost(&self) -> bool {
        false
    }
    /// The Window owns global dispatch while a selected native member may
    /// belong to its current Document's host namespace.
    fn ArgumentNamespace(&self, _call: &HostCall<'_>) -> Option<JavaScriptHostObjectNamespace> {
        None
    }
    fn BindTaskAttributionTracker(
        &mut self,
        _tracker: Rc<foundation::task_attribution::TaskAttributionTracker>,
    ) {
    }
    /// Window-owned objects (Location, Performance, etc.) stay in the global
    /// namespace when the same Window acquires a new Document host.
    fn OwnsGlobalHostObject(&self, _id: HostObjectId) -> bool {
        false
    }
    fn ArgumentPolicy(&self, _receiver: HostObjectId, _member: &str) -> HostArgumentPolicy {
        HostArgumentPolicy::Values
    }
    /// None preserves ownership for hosts which have not supplied ownership
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

/// A location captured from the active VM stack. Lines are one-based and
/// columns are zero-based UTF-16, matching the existing exception boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaScriptSourceLocation {
    pub source_name: String,
    pub line: usize,
    pub column: usize,
}
/// Native Promise handles in this exact realm/Agent. These handles carry VM
/// identity; no Rust resolver or second job queue substitutes for the engine.
#[derive(Clone)]
pub struct JavaScriptPromiseCapability {
    pub promise: HostValue,
    pub resolve: JavaScriptFunction,
    pub reject: JavaScriptFunction,
}
/// VM-owned native callback. It is invoked with the current Context on the
/// existing stack and cannot reacquire or store a realm runtime.
pub trait JavaScriptNativeFunction {
    /// Captures become private VM-traced function data, not permanent Rust
    /// roots. Native callbacks may publish mutable native-cell edges separately.
    fn CapturedValues(&self) -> Vec<HostValue> {
        Vec::new()
    }
    fn Call(
        &self,
        runtime: &mut dyn JavaScriptHostRuntime,
        receiver: &HostValue,
        arguments: &[HostValue],
    ) -> HostResult;
}
/// A scope changes only the Agent's checkpoint policy. It retains no Context
/// or runtime pointer, so native source may release the runtime borrow before
/// executing callbacks and reacquire it after this guard is dropped.
pub trait JavaScriptMicrotaskCheckpointScope {
    fn IsOutermost(&self) -> bool;
}
/// Stack task attribution carries no runtime or realm pointer.
pub trait JavaScriptScriptToolInvocationScope {}
/// ScriptState lifetime and execution policy of the callback's original realm.
/// This read-only handle contains no VM/runtime pointer and cannot enter script.
#[derive(Clone)]
pub struct JavaScriptCallbackContext(Rc<JavaScriptCallbackContextState>);
struct JavaScriptCallbackContextState {
    identity: u64,
    world_id: i32,
    valid: std::cell::Cell<bool>,
    paused: std::cell::Cell<bool>,
    terminating: std::cell::Cell<bool>,
}
impl JavaScriptCallbackContext {
    pub(crate) fn new(identity: u64, world_id: i32) -> Self {
        Self(Rc::new(JavaScriptCallbackContextState {
            identity,
            world_id,
            valid: true.into(),
            paused: false.into(),
            terminating: false.into(),
        }))
    }
    pub fn Identity(&self) -> u64 {
        self.0.identity
    }
    pub fn WorldId(&self) -> i32 {
        self.0.world_id
    }
    pub fn ContextIsValid(&self) -> bool {
        self.0.valid.get()
    }
    pub fn IsPaused(&self) -> bool {
        self.0.paused.get()
    }
    pub fn IsExecutionTerminating(&self) -> bool {
        self.0.terminating.get()
    }
    pub(crate) fn Invalidate(&self) {
        self.0.valid.set(false);
    }
    pub(crate) fn SetPaused(&self, value: bool) {
        self.0.paused.set(value);
    }
    pub(crate) fn SetTerminating(&self) {
        self.0.terminating.set(true);
    }
}
#[derive(Clone, Debug)]
pub struct JavaScriptExceptionMessage {
    pub message: Vec<u16>,
    pub location: Option<JavaScriptSourceLocation>,
}
pub trait JavaScriptHostRuntime {
    /// A local child shares this Agent's VM and job queue, with a fresh global
    /// and ScriptState. These primitives can be used inside a host callback;
    /// they never reacquire the executing realm's RefCell.
    fn CreateChildRealm(
        &mut self,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    ) -> Result<JavaScriptRealm, JavaScriptException>;
    fn WithRealm(
        &mut self,
        realm: &JavaScriptRealm,
        run: &mut dyn FnMut(&mut dyn JavaScriptHostRuntime),
    ) -> Result<(), JavaScriptException>;
    fn GlobalObject(&mut self) -> HostValue;
    /// Native wrapper identity check. This never invokes Proxy traps or author
    /// conversion hooks, including when the operands belong to sibling realms.
    fn IsSameObject(&mut self, left: &HostValue, right: &HostValue) -> bool;
    fn CreateHostObjectNamespace(
        &mut self,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
        initial: bool,
    ) -> Result<JavaScriptHostObjectNamespace, JavaScriptException>;
    fn SelectDocumentHostObjectNamespace(&mut self, namespace: &JavaScriptHostObjectNamespace);
    fn CallbackContext(&self) -> JavaScriptCallbackContext;
    fn IsExecutionTerminating(&self) -> bool;
    fn ExceptionMessage(&mut self, value: &HostValue) -> JavaScriptExceptionMessage;
    fn TaskAttributionTracker(&self) -> Rc<foundation::task_attribution::TaskAttributionTracker>;
    fn IsInContext(&self) -> bool;
    fn EnterHostObjectNamespace(
        &mut self,
        namespace: &JavaScriptHostObjectNamespace,
    ) -> Box<dyn JavaScriptHostObjectNamespaceScope>;
    fn NewError(&mut self, kind: JavaScriptExceptionKind, message: &str) -> HostValue;
    fn DeferMicrotaskCheckpoint(&mut self) -> Box<dyn JavaScriptMicrotaskCheckpointScope>;
    fn RealmIdentity(&self) -> u64;
    fn IsUndefined(&mut self, value: &HostValue) -> bool;
    fn IsNull(&mut self, value: &HostValue) -> bool;
    fn IsCallable(&mut self, value: &HostValue) -> bool;
    fn IsObject(&mut self, value: &HostValue) -> bool;
    fn GetProperty(&mut self, value: &HostValue, property: &str) -> HostResult;
    fn SetProperty(
        &mut self,
        value: &HostValue,
        property: &str,
        assigned: &HostValue,
    ) -> HostResult;
    fn NewTypeError(&mut self, message: &[u16]) -> HostResult;
    fn EvaluateClassicScript(&mut self, source: &[u16], base_url: &str) -> HostResult;
    fn PrimitiveString(&mut self, value: &HostValue) -> Option<Rc<[u16]>>;
    fn CreateNativeFunction(&mut self, callback: Rc<dyn JavaScriptNativeFunction>) -> HostResult;
    /// Capture the current VM stack onto an existing native exception wrapper.
    /// The WebAPI owner supplies its immutable name/message and class identity.
    fn CaptureExceptionStack(
        &mut self,
        value: &HostValue,
        name: &HostValue,
        message: &HostValue,
    ) -> HostResult;
    /// Preserves the original thrown value, including objects and binary data.
    /// The metadata-only JavaScriptResult remains for legacy call consumers.
    fn CallValue(
        &mut self,
        function: &HostValue,
        receiver: &HostValue,
        arguments: &[HostValue],
    ) -> HostResult;
    fn NewPromiseCapability(&mut self) -> Result<JavaScriptPromiseCapability, JavaScriptException>;
    /// ScriptPromise::FromV8Value: retain native Promise identity; otherwise
    /// adopt with a new native resolver, without a constructor-property read.
    fn PromiseResolve(&mut self, value: &HostValue) -> HostResult;
    fn PerformPromiseThen(
        &mut self,
        promise: &HostValue,
        fulfilled: &JavaScriptFunction,
        rejected: &JavaScriptFunction,
    ) -> HostResult;
    fn MarkPromiseHandled(&mut self, promise: &HostValue) -> HostResult;
    fn CaptureSourceLocation(&self, skip_script_frames: usize) -> Option<JavaScriptSourceLocation>;
    /// Current scheduler ScriptToolContext, if installed on this Agent task.
    fn ScriptToolInvocationId(&self) -> Option<[u64; 2]>;
    fn EnterScriptToolInvocation(
        &mut self,
        id: [u64; 2],
    ) -> Box<dyn JavaScriptScriptToolInvocationScope>;
    fn CreateHostFunction(&mut self, binding: &HostFunctionBinding) -> HostResult;
    /// Report an author exception through the realm's existing error channel,
    /// without throwing it from a host algorithm that invokes callbacks.
    fn ReportException(&mut self, value: &HostValue) -> HostResult;
    /// Host object classification used by Web IDL sequence conversion.
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
    /// Web IDL long conversion, preserving VM conversion exceptions.
    fn ToInt32(&mut self, value: &HostValue) -> HostResult;
    fn SerializeForStorage(
        &mut self,
        value: &HostValue,
    ) -> Result<crate::SerializedScriptValue, crate::JavaScriptSerializationError>;
    fn Deserialize(
        &mut self,
        value: &crate::SerializedScriptValue,
    ) -> Result<HostValue, crate::JavaScriptSerializationError>;
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
    fn SetScriptExecutionPaused(
        &mut self,
        _realm: &JavaScriptRealm,
        _paused: bool,
    ) -> Result<(), JavaScriptException> {
        Err(JavaScriptException {
            message: "runtime does not support script pause policy".into(),
            ..Default::default()
        })
    }
    /// Register the initial Document host on the existing realm, or create a
    /// separate host namespace for a replacement Document without a new realm.
    fn SelectDocumentHostObjectNamespace(
        &mut self,
        realm: &JavaScriptRealm,
        namespace: &JavaScriptHostObjectNamespace,
    );
    fn CreateHostObjectNamespace(
        &mut self,
        realm: &JavaScriptRealm,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
        initial: bool,
    ) -> Result<JavaScriptHostObjectNamespace, JavaScriptException>;
    /// Lends the current realm's host primitive scope to a native task. Callers
    /// already executing JS use their provided JavaScriptHostRuntime instead.
    /// The callback cannot retain or later upgrade this stack-only borrow.
    fn WithHostRuntime(
        &mut self,
        _realm: &JavaScriptRealm,
        _run: &mut dyn FnMut(&mut dyn JavaScriptHostRuntime),
    ) -> Result<(), JavaScriptException> {
        Err(JavaScriptException {
            message: "native realm invocation is unavailable".into(),
            ..JavaScriptException::default()
        })
    }

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
    fn CreateChildRealm(
        &mut self,
        parent: &JavaScriptRealm,
        host: Rc<RefCell<dyn JavaScriptHostBindings>>,
    ) -> Result<JavaScriptRealm, JavaScriptException> {
        let mut result = None;
        self.WithHostRuntime(parent, &mut |runtime| {
            result = Some(runtime.CreateChildRealm(host.clone()));
        })?;
        result.expect("host runtime callback")
    }
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
