//! Browser-side ownership and callback adapter over official public QuickJS APIs.
//! This module owns references; it neither reads engine layouts nor implements JS.
use quickjs::quickjs_atom::{JS_ATOM_Symbol_iterator, JS_ATOM_Symbol_toStringTag};
use quickjs::{quickjs::*, quickjs_header::*};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    ffi::{c_char, c_void, CStr, CString},
    fmt, ptr,
    rc::{Rc, Weak},
    sync::OnceLock,
};

// Process-local, trusted browser bootstrap bytecode only. QuickJS bytecode is
// engine-version/architecture specific; never persist it or accept it from a page.
// Name AND exact source are checked. At most the two built-in WebIDL programs
// are retained, with 1 MiB per entry and no realm/global object references.
struct BootstrapBytecode {
    name: String,
    source: String,
    bytes: Rc<Vec<u8>>,
}
thread_local! {
    static BOOTSTRAP_BYTECODE: RefCell<Vec<BootstrapBytecode>> = const { RefCell::new(Vec::new()) };
}

pub type JsResult<T> = Result<T, Value>;
pub type Gc<T> = Rc<T>;
#[derive(Clone, Copy)]
pub enum SourceType {
    Script,
    Module,
}
#[derive(Clone, Copy)]
pub enum ErrorKind {
    Syntax,
    Type,
    Range,
}
#[derive(Clone, Copy)]
pub enum WellKnownSymbol {
    Iterator,
    ToStringTag,
}
pub trait HostCallable {
    fn call(
        &self,
        ctx: &mut Context,
        this: &Value,
        args: &[Value],
        data: &[Value],
        new_target: &Value,
    ) -> JsResult<Value>;
}
pub trait RejectionTracker {
    fn on_rejection(
        &mut self,
        ctx: &mut Context,
        promise: &Gc<JsObject>,
        reason: &Value,
        handled: bool,
    );
}
pub trait ModuleLoader {
    fn resolve(&mut self, base: &str, specifier: &str) -> Result<String, String>;
    fn load(&mut self, specifier: &str) -> Result<String, String>;
    fn compiled(&self, _specifier: &str) -> Option<Value> {
        None
    }
    fn request_dynamic(&mut self, _base: &str, _specifier: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn poll_dynamic(&mut self, _url: &str) -> Result<Option<()>, String> {
        Ok(None)
    }
}

struct PendingDynamicImport {
    resolve: JSValue,
    reject: JSValue,
    attributes: JSValue,
    base: String,
    specifier: String,
    url: String,
}
struct NativeHolder {
    callable: RefCell<Option<Rc<dyn HostCallable>>>,
    data_count: usize,
}
static NATIVE_HOLDER_CLASS: OnceLock<JSClassID> = OnceLock::new();
fn native_holder_class() -> JSClassID {
    *NATIVE_HOLDER_CLASS.get_or_init(|| unsafe {
        let mut id = 0;
        JS_NewClassID(&mut id);
        id
    })
}
unsafe fn native_holder_finalizer(_: *mut JSRuntime, value: JSValue) {
    let opaque = JS_GetOpaque(value, native_holder_class()).cast::<Rc<NativeHolder>>();
    if !opaque.is_null() {
        drop(Box::from_raw(opaque));
    }
}
struct CallbackState {
    owner: Weak<EngineOwner>,
    functions: RefCell<Vec<Weak<NativeHolder>>>,
    function_sweep_threshold: Cell<usize>,
    tracker: RefCell<Option<Box<dyn RejectionTracker>>>,
    tracker_generation: Cell<u64>,
    tracker_running: Cell<bool>,
    rejection_events: RefCell<VecDeque<(Value, Value, bool)>>,
    loader: RefCell<Option<Box<dyn ModuleLoader>>>,
    pending_dynamic_imports: RefCell<Vec<PendingDynamicImport>>,
    modules: RefCell<HashMap<String, Value>>,
    prepared_modules: RefCell<HashMap<usize, Value>>,
    last_exception_location: RefCell<Option<JSExceptionLocation>>,
    intrinsics: HashMap<&'static str, JSValue>,
    proxy_class: JSClassID,
}
struct EngineOwner {
    rt: *mut JSRuntime,
    ctx: *mut JSContext,
    callbacks: *mut CallbackState,
}
impl Drop for EngineOwner {
    fn drop(&mut self) {
        unsafe {
            JS_SetModuleEmbeddingHooks(self.rt, None, None, ptr::null_mut());
            JS_SetHostPromiseRejectionTracker(self.rt, None, ptr::null_mut());
            JS_SetModuleLoaderFunc(self.rt, None, None, ptr::null_mut());
            JS_SetContextOpaque(self.ctx, ptr::null_mut());
            let data = Box::from_raw(self.callbacks);
            for pending in data.pending_dynamic_imports.borrow_mut().drain(..) {
                JS_FreeValue(self.ctx, pending.resolve);
                JS_FreeValue(self.ctx, pending.reject);
                JS_FreeValue(self.ctx, pending.attributes);
            }
            for value in data.intrinsics.values() {
                JS_FreeValue(self.ctx, *value);
            }
            drop(data);
            JS_FreeContext(self.ctx);
            JS_FreeRuntime(self.rt);
        }
    }
}
pub struct OwnedRaw {
    raw: JSValue,
    owner: Rc<EngineOwner>,
}
impl Drop for OwnedRaw {
    fn drop(&mut self) {
        unsafe { JS_FreeValueRT(self.owner.rt, self.raw) }
    }
}
#[derive(Clone)]
pub struct JsString(Rc<OwnedRaw>);
#[derive(Clone)]
pub struct JsSymbol(Rc<OwnedRaw>);
// Gc<JsObject> already supplies shared ownership. Keeping another Rc around
// its OwnedRaw doubles native argument/result allocation without sharing it
// independently; release the JS reference with the last outer Gc instead.
pub struct JsObject(OwnedRaw);
impl JsObject {
    pub(super) fn identity(&self) -> (usize, usize) {
        (
            self.0.owner.rt as usize,
            JS_VALUE_GET_PTR(self.0.raw) as usize,
        )
    }
}
impl PartialEq for JsObject {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.owner.rt == rhs.0.owner.rt
            && JS_VALUE_GET_PTR(self.0.raw) == JS_VALUE_GET_PTR(rhs.0.raw)
    }
}
impl Eq for JsObject {}
impl JsString {
    pub fn to_string_lossy(&self) -> String {
        unsafe {
            let mut len = 0;
            let bytes = JS_ToCStringLen2(self.0.owner.ctx, &mut len, self.0.raw, 0);
            if bytes.is_null() {
                let error = JS_GetException(self.0.owner.ctx);
                JS_FreeValue(self.0.owner.ctx, error);
                return String::new();
            }
            let text =
                String::from_utf8_lossy(std::slice::from_raw_parts(bytes.cast(), len)).into_owned();
            JS_FreeCString(self.0.owner.ctx, bytes);
            text
        }
    }
}
#[derive(Clone)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    Str(JsString),
    Object(Gc<JsObject>),
    Symbol(JsSymbol),
    Other(Rc<OwnedRaw>),
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undefined => f.write_str("undefined"),
            Self::Null => f.write_str("null"),
            Self::Bool(v) => v.fmt(f),
            Self::Int(v) => v.fmt(f),
            Self::Float(v) => v.fmt(f),
            Self::Str(v) => v.to_string_lossy().fmt(f),
            Self::Object(_) => f.write_str("[object]"),
            Self::Symbol(_) => f.write_str("[symbol]"),
            Self::Other(_) => f.write_str("[value]"),
        }
    }
}
impl Default for Value {
    fn default() -> Self {
        Self::Undefined
    }
}
impl Value {
    pub fn number(n: f64) -> Self {
        if n.is_finite() && n == n as i32 as f64 && !(n == 0.0 && n.is_sign_negative()) {
            Self::Int(n as i32)
        } else {
            Self::Float(n)
        }
    }
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Int(v) => Some(*v as f64),
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        self.as_number()
    }
    pub fn as_boolean(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    pub fn as_string(&self) -> Option<&JsString> {
        if let Self::Str(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_object(&self) -> Option<&Gc<JsObject>> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }
    fn owned(&self) -> Option<&OwnedRaw> {
        match self {
            Self::Str(v) => Some(&v.0),
            Self::Object(v) => Some(&v.0),
            Self::Symbol(v) => Some(&v.0),
            Self::Other(v) => Some(v),
            _ => None,
        }
    }
    fn raw(&self) -> JSValue {
        match self {
            Self::Undefined => JS_UNDEFINED,
            Self::Null => JS_NULL,
            Self::Bool(v) => JS_NewBool(ptr::null_mut(), *v as i32),
            Self::Int(v) => JS_NewInt32(ptr::null_mut(), *v),
            Self::Float(v) => JS_NewFloat64(ptr::null_mut(), *v),
            _ => self.owned().unwrap().raw,
        }
    }
    fn checked(&self, ctx: &mut Context) -> JsResult<JSValue> {
        if self.owned().is_some_and(|v| v.owner.rt != ctx.owner.rt) {
            Err(ctx.type_error("Value belongs to another realm"))
        } else {
            Ok(self.raw())
        }
    }
    unsafe fn take(owner: &Rc<EngineOwner>, raw: JSValue) -> Self {
        match JS_VALUE_GET_NORM_TAG(raw) {
            JS_TAG_UNDEFINED => Self::Undefined,
            JS_TAG_NULL => Self::Null,
            JS_TAG_BOOL => Self::Bool(JS_VALUE_GET_BOOL(raw) != 0),
            JS_TAG_INT => Self::Int(JS_VALUE_GET_INT(raw)),
            JS_TAG_FLOAT64 => Self::Float(JS_VALUE_GET_FLOAT64(raw)),
            JS_TAG_OBJECT => Self::Object(Rc::new(JsObject(OwnedRaw {
                raw,
                owner: owner.clone(),
            }))),
            tag => {
                let owned = Rc::new(OwnedRaw {
                    raw,
                    owner: owner.clone(),
                });
                // Public JS_IsString includes both flat strings and the
                // official rope representation used by long concatenations.
                if JS_IsString(raw) != 0 {
                    return Self::Str(JsString(owned));
                }
                match tag {
                    JS_TAG_SYMBOL => Self::Symbol(JsSymbol(owned)),
                    _ => Self::Other(owned),
                }
            }
        }
    }
    pub fn to_boolean(&self) -> bool {
        unsafe {
            JS_ToBool(
                self.owned().map_or(ptr::null_mut(), |v| v.owner.ctx),
                self.raw(),
            ) != 0
        }
    }
    pub fn strict_equals(a: &Self, b: &Self) -> bool {
        if let (Some(x), Some(y)) = (a.owned(), b.owned()) {
            if x.owner.rt != y.owner.rt {
                return false;
            }
        }
        unsafe {
            JS_StrictEq(
                a.owned()
                    .or(b.owned())
                    .map_or(ptr::null_mut(), |v| v.owner.ctx),
                a.raw(),
                b.raw(),
            ) != 0
        }
    }
}
#[derive(Clone)]
pub enum PropKey {
    Str(JsString),
    Index(u32),
    Sym(JsSymbol),
}
impl PropKey {
    pub fn from_string(s: &JsString) -> Self {
        Self::Str(s.clone())
    }
}
#[derive(Clone, Copy)]
pub struct PropFlags(i32);
impl PropFlags {
    pub const CONFIGURABLE: Self = Self(JS_PROP_CONFIGURABLE);
    pub const ENUMERABLE: Self = Self(JS_PROP_ENUMERABLE);
    pub const C_W: Self = Self(JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE);
    pub const C_W_E: Self = Self(JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE | JS_PROP_ENUMERABLE);
    pub fn union(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
pub enum Property {
    Data(Value, PropFlags),
    Accessor(Option<Value>, Option<Value>, PropFlags),
}
impl Property {
    pub fn data(v: Value, flags: PropFlags) -> Self {
        Self::Data(v, flags)
    }
    pub fn accessor(get: Option<Value>, set: Option<Value>, flags: PropFlags) -> Self {
        Self::Accessor(get, set, flags)
    }
}
pub struct WellKnown {
    owner: Rc<EngineOwner>,
}
impl WellKnown {
    pub fn get(&self, which: WellKnownSymbol) -> JsSymbol {
        unsafe {
            let raw = JS_AtomToValue(
                self.owner.ctx,
                match which {
                    WellKnownSymbol::Iterator => JS_ATOM_Symbol_iterator,
                    WellKnownSymbol::ToStringTag => JS_ATOM_Symbol_toStringTag,
                },
            );
            match Value::take(&self.owner, raw) {
                Value::Symbol(s) => s,
                _ => panic!("well-known symbol"),
            }
        }
    }
}
pub fn same_symbol(a: &JsSymbol, b: &JsSymbol) -> bool {
    Value::strict_equals(&Value::Symbol(a.clone()), &Value::Symbol(b.clone()))
}

pub struct Context {
    owner: Rc<EngineOwner>,
    primary: bool,
    pub well_known: WellKnown,
}
impl Drop for Context {
    fn drop(&mut self) {
        if self.primary {
            unsafe {
                let data = &*self.owner.callbacks;
                JS_SetHostPromiseRejectionTracker(self.owner.rt, None, ptr::null_mut());
                data.tracker.borrow_mut().take();
                data.rejection_events.borrow_mut().clear();
                // Public opaque finalizers release captures when JS functions
                // die. Retire still-live closures here to break host-owned
                // values retaining this runtime during realm teardown.
                for holder in data
                    .functions
                    .borrow_mut()
                    .drain(..)
                    .filter_map(|v| v.upgrade())
                {
                    holder.callable.borrow_mut().take();
                }
                data.modules.borrow_mut().clear();
                data.prepared_modules.borrow_mut().clear();
                data.loader.borrow_mut().take();
            }
        }
    }
}
impl Context {
    pub fn new() -> Self {
        Self::with_native_stack_budget(1024 * 1024)
    }
    pub fn with_native_stack_budget(bytes: usize) -> Self {
        assert!(
            bytes > 0,
            "a native stack budget must retain overflow protection"
        );
        unsafe {
            let rt = JS_NewRuntime();
            assert!(!rt.is_null(), "QuickJS runtime allocation");
            JS_SetMaxStackSize(rt, bytes);
            JS_EnableExceptionMetadata(rt, true);
            JS_SetExceptionMetadataStackReadPolicy(rt, true);
            let ctx = JS_NewContext(rt);
            if ctx.is_null() {
                JS_FreeRuntime(rt);
                panic!("QuickJS context allocation");
            }
            let class = JSClassDef {
                class_name: c"BrowserNativeCapture".as_ptr(),
                finalizer: Some(native_holder_finalizer),
                gc_mark: None,
                call: None,
                exotic: ptr::null_mut(),
            };
            assert_eq!(
                JS_NewClass(rt, native_holder_class(), &class),
                0,
                "host capture class"
            );
            let callbacks = Box::into_raw(Box::new(CallbackState {
                owner: Weak::new(),
                functions: RefCell::new(Vec::new()),
                function_sweep_threshold: Cell::new(64),
                tracker: RefCell::new(None),
                tracker_generation: Cell::new(0),
                tracker_running: Cell::new(false),
                rejection_events: RefCell::new(VecDeque::new()),
                loader: RefCell::new(None),
                pending_dynamic_imports: RefCell::new(Vec::new()),
                modules: RefCell::new(HashMap::new()),
                prepared_modules: RefCell::new(HashMap::new()),
                last_exception_location: RefCell::new(None),
                intrinsics: HashMap::new(),
                proxy_class: 0,
            }));
            let owner = Rc::new(EngineOwner { rt, ctx, callbacks });
            (*callbacks).owner = Rc::downgrade(&owner);
            JS_SetContextOpaque(ctx, callbacks.cast());
            JS_SetModuleEmbeddingHooks(
                rt,
                Some(dynamic_import),
                Some(consume_internal_module_promise),
                callbacks.cast(),
            );
            let global = JS_GetGlobalObject(ctx);
            for name in ["Proxy", "Error", "SyntaxError", "TypeError", "RangeError"] {
                let key = CString::new(name).unwrap();
                let value = JS_GetPropertyStr(ctx, global, key.as_ptr());
                assert_eq!(JS_IsException(value), 0);
                (*callbacks).intrinsics.insert(name, value);
            }
            JS_FreeValue(ctx, global);
            let mut context = Self {
                well_known: WellKnown {
                    owner: owner.clone(),
                },
                owner,
                primary: true,
            };
            let target = context.new_object();
            let handler = context.new_object();
            let probe = context
                .new_proxy(target, handler)
                .expect("proxy class probe");
            (*callbacks).proxy_class = JS_GetClassID(probe.0.raw);
            context
        }
    }
    unsafe fn borrowed(ctx: *mut JSContext) -> Self {
        let data = &*(JS_GetContextOpaque(ctx) as *const CallbackState);
        let owner = data.owner.upgrade().expect("live host context");
        Self {
            well_known: WellKnown {
                owner: owner.clone(),
            },
            owner,
            primary: false,
        }
    }
    fn data(&self) -> &CallbackState {
        unsafe { &*self.owner.callbacks }
    }
    fn accept(&mut self, raw: JSValue) -> JsResult<Value> {
        unsafe {
            if JS_IsException(raw) != 0 {
                Err(self.take_exception())
            } else {
                Ok(Value::take(&self.owner, raw))
            }
        }
    }
    pub fn validate_value(&mut self, value: &Value) -> JsResult<()> {
        value.checked(self).map(|_| ())
    }
    fn take_exception(&mut self) -> Value {
        unsafe {
            self.data()
                .last_exception_location
                .replace(JS_GetExceptionMetadata(self.owner.ctx));
            Value::take(&self.owner, JS_GetException(self.owner.ctx))
        }
    }
    pub fn take_exception_location(&mut self) -> Option<JSExceptionLocation> {
        self.data().last_exception_location.borrow_mut().take()
    }
    pub fn error_creation_location(&self, value: &Value) -> Option<JSExceptionLocation> {
        if value
            .owned()
            .is_some_and(|owned| !Rc::ptr_eq(&owned.owner, &self.owner))
        {
            return None;
        }
        unsafe { JS_GetErrorCreationMetadata(self.owner.ctx, value.raw()) }
    }
    fn key_atom(&mut self, key: &PropKey) -> JsResult<JSAtom> {
        unsafe {
            let raw = match key {
                PropKey::Str(s) => Value::Str(s.clone()).checked(self)?,
                PropKey::Sym(s) => Value::Symbol(s.clone()).checked(self)?,
                PropKey::Index(n) => JS_NewUint32(self.owner.ctx, *n),
            };
            let atom = JS_ValueToAtom(self.owner.ctx, raw);
            if atom == JS_ATOM_NULL as u32 {
                Err(self.take_exception())
            } else {
                Ok(atom)
            }
        }
    }
    pub fn intern(&mut self, text: &str) -> JsString {
        unsafe {
            let raw = JS_NewStringLen(self.owner.ctx, text.as_ptr().cast(), text.len());
            match self.accept(raw).expect("host string allocation") {
                Value::Str(v) => v,
                _ => unreachable!(),
            }
        }
    }
    pub fn new_array_buffer_copy(&mut self, bytes: &[u8]) -> JsResult<Value> {
        unsafe {
            self.accept(JS_NewArrayBufferCopy(
                self.owner.ctx,
                bytes.as_ptr(),
                bytes.len(),
            ))
        }
    }
    pub fn global(&mut self) -> Gc<JsObject> {
        unsafe {
            match self.accept(JS_GetGlobalObject(self.owner.ctx)).unwrap() {
                Value::Object(o) => o,
                _ => unreachable!(),
            }
        }
    }
    pub fn global_value(&mut self, name: &str) -> Value {
        let global = Value::Object(self.global());
        let key = PropKey::from_string(&self.intern(name));
        self.get_property(&global, &key).unwrap_or(Value::Undefined)
    }
    pub fn new_object(&mut self) -> Gc<JsObject> {
        unsafe {
            match self
                .accept(JS_NewObject(self.owner.ctx))
                .expect("host object allocation")
            {
                Value::Object(o) => o,
                _ => unreachable!(),
            }
        }
    }
    // Fresh data snapshots use fallible public QuickJS operations. Defining
    // own data properties bypasses inherited setters, including __proto__.
    pub fn try_new_object(&mut self) -> JsResult<Gc<JsObject>> {
        unsafe {
            match self.accept(JS_NewObject(self.owner.ctx))? {
                Value::Object(object) => Ok(object),
                _ => unreachable!(),
            }
        }
    }
    pub fn define_data_value(
        &mut self,
        object: &Gc<JsObject>,
        name: &str,
        value: Value,
        flags: PropFlags,
    ) -> JsResult<()> {
        self.validate_value(&Value::Object(object.clone()))?;
        self.validate_value(&value)?;
        unsafe {
            let atom = JS_NewAtomLen(self.owner.ctx, name.as_ptr().cast(), name.len());
            if atom == JS_ATOM_NULL as u32 {
                return Err(self.take_exception());
            }
            let result = JS_DefinePropertyValue(
                self.owner.ctx,
                object.0.raw,
                atom,
                JS_DupValue(self.owner.ctx, value.raw()),
                flags.0,
            );
            JS_FreeAtom(self.owner.ctx, atom);
            if result < 0 {
                Err(self.take_exception())
            } else if result == 0 {
                Err(self.type_error("Cannot define snapshot property"))
            } else {
                Ok(())
            }
        }
    }
    pub fn is_callable(&self, value: &Value) -> bool {
        if value.owned().is_some_and(|v| v.owner.rt != self.owner.rt) {
            return false;
        }
        unsafe { JS_IsFunction(self.owner.ctx, value.raw()) != 0 }
    }
    pub fn is_proxy(&self, value: &Value) -> bool {
        value.is_object()
            && !value.owned().is_some_and(|v| v.owner.rt != self.owner.rt)
            && unsafe { JS_GetClassID(value.raw()) == self.data().proxy_class }
    }
    pub fn get_property(&mut self, obj: &Value, key: &PropKey) -> JsResult<Value> {
        let raw = obj.checked(self)?;
        let atom = self.key_atom(key)?;
        unsafe {
            let result = JS_GetProperty(self.owner.ctx, raw, atom);
            JS_FreeAtom(self.owner.ctx, atom);
            self.accept(result)
        }
    }
    pub fn has_property(&mut self, obj: &Gc<JsObject>, key: &PropKey) -> JsResult<bool> {
        self.validate_value(&Value::Object(obj.clone()))?;
        let atom = self.key_atom(key)?;
        unsafe {
            let n = JS_HasProperty(self.owner.ctx, obj.0.raw, atom);
            JS_FreeAtom(self.owner.ctx, atom);
            if n < 0 {
                Err(self.take_exception())
            } else {
                Ok(n != 0)
            }
        }
    }
    pub fn to_property_key(&mut self, key: &Value) -> JsResult<PropKey> {
        let raw = key.checked(self)?;
        unsafe {
            match self.accept(JS_ToPropertyKey(self.owner.ctx, raw))? {
                Value::Str(s) => Ok(PropKey::Str(s)),
                Value::Symbol(s) => Ok(PropKey::Sym(s)),
                _ => unreachable!(),
            }
        }
    }
    pub fn to_js_string(&mut self, value: &Value) -> JsResult<JsString> {
        let raw = value.checked(self)?;
        unsafe {
            match self.accept(JS_ToString(self.owner.ctx, raw))? {
                Value::Str(s) => Ok(s),
                _ => unreachable!(),
            }
        }
    }
    pub fn to_rust_string(&mut self, value: &Value) -> JsResult<String> {
        self.to_js_string(value).map(|s| s.to_string_lossy())
    }
    pub fn format_exception(&mut self, value: &Value) -> String {
        self.to_rust_string(value)
            .unwrap_or_else(|_| format!("{value:?}"))
    }
    pub fn set_prototype_of(
        &mut self,
        obj: &Gc<JsObject>,
        proto: Option<Gc<JsObject>>,
    ) -> JsResult<bool> {
        self.validate_value(&Value::Object(obj.clone()))?;
        if let Some(value) = &proto {
            self.validate_value(&Value::Object(value.clone()))?;
        }
        unsafe {
            let n = JS_SetPrototype(
                self.owner.ctx,
                obj.0.raw,
                proto.as_ref().map_or(JS_NULL, |v| v.0.raw),
            );
            if n < 0 {
                Err(self.take_exception())
            } else {
                Ok(n != 0)
            }
        }
    }
    pub fn define_value(&mut self, obj: &Gc<JsObject>, name: &str, value: Value, flags: PropFlags) {
        let key = PropKey::from_string(&self.intern(name));
        self.define_own(obj, key, Property::Data(value, flags));
    }
    pub fn define_own(&mut self, obj: &Gc<JsObject>, key: PropKey, property: Property) -> bool {
        if self.validate_value(&Value::Object(obj.clone())).is_err() {
            return false;
        }
        let valid = match &property {
            Property::Data(value, _) => self.validate_value(value),
            Property::Accessor(get, set, _) => get
                .as_ref()
                .map_or(Ok(()), |value| self.validate_value(value))
                .and_then(|_| {
                    set.as_ref()
                        .map_or(Ok(()), |value| self.validate_value(value))
                }),
        };
        if valid.is_err() {
            return false;
        }
        let Ok(atom) = self.key_atom(&key) else {
            return false;
        };
        unsafe {
            let n = match property {
                Property::Data(v, flags) => JS_DefinePropertyValue(
                    self.owner.ctx,
                    obj.0.raw,
                    atom,
                    JS_DupValue(self.owner.ctx, v.raw()),
                    flags.0,
                ),
                Property::Accessor(g, s, flags) => JS_DefinePropertyGetSet(
                    self.owner.ctx,
                    obj.0.raw,
                    atom,
                    JS_DupValue(self.owner.ctx, g.as_ref().map_or(JS_UNDEFINED, Value::raw)),
                    JS_DupValue(self.owner.ctx, s.as_ref().map_or(JS_UNDEFINED, Value::raw)),
                    flags.0,
                ),
            };
            JS_FreeAtom(self.owner.ctx, atom);
            if n < 0 {
                let _ = self.take_exception();
            }
            n > 0
        }
    }
    pub fn new_proxy(
        &mut self,
        target: Gc<JsObject>,
        handler: Gc<JsObject>,
    ) -> JsResult<Gc<JsObject>> {
        self.validate_value(&Value::Object(target.clone()))?;
        self.validate_value(&Value::Object(handler.clone()))?;
        unsafe {
            let ctor = self.data().intrinsics["Proxy"];
            let mut args = [target.0.raw, handler.0.raw];
            let value = JS_CallConstructor(self.owner.ctx, ctor, 2, args.as_mut_ptr());
            match self.accept(value)? {
                Value::Object(o) => Ok(o),
                _ => unreachable!(),
            }
        }
    }
    pub fn call(&mut self, function: Value, this: Value, args: &[Value]) -> JsResult<Value> {
        let f = function.checked(self)?;
        let this = this.checked(self)?;
        // Proxy Reflect fallbacks and browser callbacks usually have only a
        // few arguments. Keep their borrowed raw values on this call's stack;
        // larger calls retain the original checked, ordered Vec path.
        let mut inline_args = [JS_UNDEFINED; 8];
        let mut heap_args;
        let raw_args = if args.len() <= inline_args.len() {
            for (slot, value) in inline_args.iter_mut().zip(args) {
                *slot = value.checked(self)?;
            }
            &mut inline_args[..args.len()]
        } else {
            heap_args = args
                .iter()
                .map(|v| v.checked(self))
                .collect::<JsResult<Vec<_>>>()?;
            heap_args.as_mut_slice()
        };
        unsafe {
            self.accept(JS_Call(
                self.owner.ctx,
                f,
                this,
                raw_args.len() as i32,
                raw_args.as_mut_ptr(),
            ))
        }
    }
    pub fn eval(&mut self, source: &str) -> JsResult<Value> {
        self.eval_named(source, SourceType::Script, "<eval>")
    }
    /// qjsc.c output_object_code / quickjs-libc.c js_std_eval_binary contract.
    /// Compilation may be reused; every realm still executes the entire script.
    pub fn eval_bootstrap_named(&mut self, source: &str, name: &str) -> JsResult<Value> {
        if !matches!(name, "browser:dom-webidl" | "browser:window-webidl")
            || source.len() > 256 * 1024
        {
            return self.eval_named(source, SourceType::Script, name);
        }
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        let started = profile.then(std::time::Instant::now);
        let cached = BOOTSTRAP_BYTECODE.with(|cache| {
            cache
                .borrow()
                .iter()
                .find(|entry| entry.name == name && entry.source == source)
                .map(|entry| entry.bytes.clone())
        });
        let hit = cached.is_some();
        // These API calls own their JSValue. JS_EvalFunction consumes exactly
        // the compiled reference, and serialized bytes are copied before free.
        let compiled = unsafe {
            if let Some(bytes) = cached {
                let raw = JS_ReadObject(
                    self.owner.ctx,
                    bytes.as_ptr(),
                    bytes.len(),
                    JS_READ_OBJ_BYTECODE,
                );
                if JS_IsException(raw) != 0 {
                    return Err(self.take_exception());
                }
                raw
            } else {
                let mut code = source.as_bytes().to_vec();
                code.push(0);
                let filename = CString::new(name).expect("static bootstrap name");
                let raw = JS_Eval(
                    self.owner.ctx,
                    code.as_ptr().cast(),
                    source.len(),
                    filename.as_ptr(),
                    JS_EVAL_TYPE_GLOBAL | JS_EVAL_FLAG_COMPILE_ONLY,
                );
                if JS_IsException(raw) != 0 {
                    return Err(self.take_exception());
                }
                let mut length = 0;
                let buffer =
                    JS_WriteObject(self.owner.ctx, &mut length, raw, JS_WRITE_OBJ_BYTECODE);
                if !buffer.is_null() {
                    if length <= 1024 * 1024 {
                        let bytes = Rc::new(std::slice::from_raw_parts(buffer, length).to_vec());
                        BOOTSTRAP_BYTECODE.with(|cache| {
                            let mut cache = cache.borrow_mut();
                            cache.retain(|entry| entry.name != name);
                            cache.push(BootstrapBytecode {
                                name: name.into(),
                                source: source.into(),
                                bytes,
                            });
                        });
                    }
                    js_free(self.owner.ctx, buffer.cast());
                } else {
                    // Caching is optional; an unsupported serialized feature
                    // must not turn a valid bootstrap into a page error.
                    JS_FreeValue(self.owner.ctx, JS_GetException(self.owner.ctx));
                }
                raw
            }
        };
        if let Some(start) = started {
            eprintln!(
                "javascript-bootstrap-code-profile name={name} hit={hit} prepare_ms={:.3}",
                start.elapsed().as_secs_f64() * 1000.
            );
        }
        unsafe { self.accept(JS_EvalFunction(self.owner.ctx, compiled)) }
    }

    pub fn eval_named(&mut self, source: &str, kind: SourceType, name: &str) -> JsResult<Value> {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        self.eval_named_profiled(source, kind, name, profile)
    }
    fn eval_named_profiled(
        &mut self,
        source: &str,
        kind: SourceType,
        name: &str,
        profile: bool,
    ) -> JsResult<Value> {
        let started = profile.then(std::time::Instant::now);
        if profile && source.len() > 100_000 {
            eprintln!(
                "javascript-evaluate-start source={name:?} bytes={}",
                source.len()
            );
        }
        let mut code = source.as_bytes().to_vec();
        code.push(0);
        let name = CString::new(name).map_err(|_| self.type_error("Source name contains NUL"))?;
        let result = unsafe {
            // Official quickjs.c __JS_EvalInternal and JS_EvalFunction use the
            // same global this/closure inputs for global scripts. Splitting
            // those calls only attributes time; it retains the entire compile
            // and execution cost and does not cache, postpone or cancel work.
            // Modules retain their ordinary resolution/evaluation path.
            let split = profile && matches!(kind, SourceType::Script);
            let compile_started = split.then(std::time::Instant::now);
            let compiled_or_result = JS_Eval(
                self.owner.ctx,
                code.as_ptr().cast(),
                source.len(),
                name.as_ptr(),
                match kind {
                    SourceType::Script => JS_EVAL_TYPE_GLOBAL,
                    SourceType::Module => JS_EVAL_TYPE_MODULE,
                } | if split { JS_EVAL_FLAG_COMPILE_ONLY } else { 0 },
            );
            if let Some(compile_started) = compile_started {
                let compile_ms = compile_started.elapsed().as_secs_f64() * 1000.0;
                let execute_started = std::time::Instant::now();
                let compile_failed = JS_IsException(compiled_or_result) != 0;
                // JS_EvalFunction consumes the successful compiled reference.
                let value = if compile_failed {
                    compiled_or_result
                } else {
                    JS_EvalFunction(self.owner.ctx, compiled_or_result)
                };
                let execute_ms = if compile_failed {
                    0.0
                } else {
                    execute_started.elapsed().as_secs_f64() * 1000.0
                };
                eprintln!("javascript-evaluate-phase-profile source={:?} bytes={} compile_ms={compile_ms:.3} execute_ms={execute_ms:.3} compile_failed={compile_failed}", name.to_string_lossy(), source.len());
                self.accept(value)
            } else {
                self.accept(compiled_or_result)
            }
        };
        if let Some(started) = started {
            let elapsed = started.elapsed().as_secs_f64() * 1000.0;
            if elapsed > 50.0 {
                eprintln!(
                    "javascript-evaluate-profile source={:?} bytes={} ms={elapsed:.3}",
                    name.to_string_lossy(),
                    source.len()
                );
            }
        }
        result
    }
    pub fn make_error(&mut self, kind: ErrorKind, message: &str) -> Value {
        self.make_named_error(
            match kind {
                ErrorKind::Syntax => "SyntaxError",
                ErrorKind::Type => "TypeError",
                ErrorKind::Range => "RangeError",
            },
            message,
        )
    }
    pub fn make_plain_error(&mut self, message: &str) -> Value {
        self.make_named_error("Error", message)
    }
    fn make_named_error(&mut self, name: &'static str, message: &str) -> Value {
        let raw = self.data().intrinsics[name];
        let function = unsafe { Value::take(&self.owner, JS_DupValue(self.owner.ctx, raw)) };
        let message = Value::Str(self.intern(message));
        self.call(function, Value::Undefined, &[message])
            .unwrap_or_else(|e| e)
    }
    pub fn type_error(&mut self, message: &str) -> Value {
        self.make_error(ErrorKind::Type, message)
    }
    pub fn range_error(&mut self, message: &str) -> Value {
        self.make_error(ErrorKind::Range, message)
    }
    pub fn new_host_function(
        &mut self,
        callable: Rc<dyn HostCallable>,
        data: Vec<Value>,
        name: &str,
        length: u32,
    ) -> Gc<JsObject> {
        for value in &data {
            value.checked(self).expect("native capture realm");
        }
        let holder = Rc::new(NativeHolder {
            callable: RefCell::new(Some(callable)),
            data_count: data.len(),
        });
        {
            let mut registry = self.data().functions.borrow_mut();
            let threshold = &self.data().function_sweep_threshold;
            if registry.len() >= threshold.get() {
                registry.retain(|entry| entry.strong_count() != 0);
                // Scanning every 64 registrations is quadratic when thousands
                // of DOM proxy traps stay alive. Amortize against live entries;
                // dead-only churn retains the original 64-entry bound.
                threshold.set(registry.len().saturating_mul(2).max(64));
            }
            registry.push(Rc::downgrade(&holder));
        }
        unsafe {
            let capture = JS_NewObjectClass(self.owner.ctx, native_holder_class() as i32);
            if JS_IsException(capture) != 0 {
                panic!("native capture allocation: {:?}", self.take_exception());
            }
            JS_SetOpaque(capture, Box::into_raw(Box::new(holder)).cast());
            // Store JS captures in the engine's real CFunctionData array, whose
            // GC marker and finalizer already own and trace those references.
            let mut values = Vec::with_capacity(data.len() + 1);
            values.push(capture);
            for value in &data {
                values.push(value.checked(self).expect("native capture realm"));
            }
            let raw = JS_NewCFunctionData(
                self.owner.ctx,
                Some(native_dispatch),
                length as i32,
                0,
                values.len() as i32,
                values.as_mut_ptr(),
            );
            JS_FreeValue(self.owner.ctx, capture);
            let object = match self.accept(raw).expect("native callback allocation") {
                Value::Object(o) => o,
                _ => unreachable!(),
            };
            let value = Value::Str(self.intern(name));
            self.define_value(&object, "name", value, PropFlags::CONFIGURABLE);
            object
        }
    }
    pub fn new_native_function(
        &mut self,
        callback: fn(&mut Context, &Value, &[Value], i32) -> JsResult<Value>,
        name: &str,
        length: u32,
        magic: i32,
    ) -> Gc<JsObject> {
        self.new_host_function(
            Rc::new(NativeCallback { callback, magic }),
            Vec::new(),
            name,
            length,
        )
    }
    pub fn set_rejection_tracker(&mut self, tracker: Option<Box<dyn RejectionTracker>>) {
        self.data()
            .tracker_generation
            .set(self.data().tracker_generation.get().wrapping_add(1));
        self.data().tracker.replace(tracker);
        unsafe {
            JS_SetHostPromiseRejectionTracker(
                self.owner.rt,
                Some(rejection_dispatch),
                self.owner.callbacks.cast(),
            );
        }
    }
    pub fn enqueue_job(&mut self, function: Value, args: Vec<Value>) -> JsResult<()> {
        let mut raw = vec![function.checked(self)?];
        // JS_EnqueueJob duplicates the raw arguments. Keep their owning values
        // alive until that duplication has finished.
        for value in &args {
            raw.push(value.checked(self)?);
        }
        unsafe {
            if JS_EnqueueJob(
                self.owner.ctx,
                Some(host_job),
                raw.len() as i32,
                raw.as_mut_ptr(),
            ) < 0
            {
                Err(self.take_exception())
            } else {
                Ok(())
            }
        }
    }
    pub fn run_jobs_errors(&mut self) -> Vec<Value> {
        self.run_jobs_errors_profiled(std::env::var_os("BROWSER_PROFILE_INPUT").is_some())
    }
    fn run_jobs_errors_profiled(&mut self, profile: bool) -> Vec<Value> {
        self.resume_dynamic_imports();
        let tracing = browser_tracing::enabled();
        let mut trace = browser_tracing::span("javascript", "JobDrain");
        let mut traced_jobs = 0usize;
        let started = profile.then(std::time::Instant::now);
        let mut jobs = 0usize;
        let mut execute_ms = 0.0f64;
        let mut max_ms = 0.0f64;
        let mut over_16ms = 0usize;
        let mut errors = Vec::new();
        unsafe {
            loop {
                let mut ctx = ptr::null_mut();
                let job_started = profile.then(std::time::Instant::now);
                let n = JS_ExecutePendingJob(self.owner.rt, &mut ctx);
                let elapsed = job_started.map(|time| time.elapsed().as_secs_f64() * 1000.0);
                if n == 0 {
                    break;
                }
                if tracing {
                    traced_jobs += 1;
                }
                if n < 0 {
                    errors.push(Value::take(&self.owner, JS_GetException(ctx)));
                }
                if let Some(elapsed) = elapsed {
                    jobs += 1;
                    execute_ms += elapsed;
                    max_ms = max_ms.max(elapsed);
                    over_16ms += usize::from(elapsed >= 16.0);
                    // Every job is executed and included in sum/max/count. Bound
                    // diagnostic output by logging only jobs taking at least 1 ms.
                    if elapsed >= 1.0 {
                        eprintln!("javascript-pending-job-profile index={jobs} execute_ms={elapsed:.3} failed={}", n < 0);
                    }
                }
            }
        }
        trace.set("jobs", traced_jobs as f64);
        trace.set("errors", errors.len() as f64);
        if let Some(started) = started {
            if jobs > 0 {
                // Drain total includes per-job logs, exception ownership and
                // the final empty-queue check. The enclosing checkpoint timer
                // also includes this summary's output and error conversion.
                eprintln!("javascript-job-drain-profile jobs={jobs} errors={} execute_sum_ms={execute_ms:.3} max_ms={max_ms:.3} over_16ms={over_16ms} total_ms={:.3}", errors.len(), started.elapsed().as_secs_f64() * 1000.0);
            }
        }
        errors
    }

    fn resume_dynamic_imports(&mut self) {
        let data = self.data();
        let pending = std::mem::take(&mut *data.pending_dynamic_imports.borrow_mut());
        let mut remaining = Vec::new();
        for import in pending {
            let state = data
                .loader
                .borrow_mut()
                .as_mut()
                .ok_or_else(|| "No module loader".to_owned())
                .and_then(|loader| loader.poll_dynamic(&import.url));
            match state {
                Ok(None) => remaining.push(import),
                Ok(Some(())) => unsafe {
                    let base = CString::new(import.base.as_str()).unwrap();
                    let specifier = CString::new(import.specifier.as_str()).unwrap();
                    JS_HostResumeDynamicImport(
                        self.owner.ctx,
                        import.resolve,
                        import.reject,
                        base.as_ptr(),
                        specifier.as_ptr(),
                        import.attributes,
                    );
                    JS_FreeValue(self.owner.ctx, import.resolve);
                    JS_FreeValue(self.owner.ctx, import.reject);
                    JS_FreeValue(self.owner.ctx, import.attributes);
                },
                Err(message) => unsafe {
                    let mut context = Context::borrowed(self.owner.ctx);
                    let error = context.make_plain_error(&message);
                    let mut reason = error.raw();
                    let result =
                        JS_Call(self.owner.ctx, import.reject, JS_UNDEFINED, 1, &mut reason);
                    JS_FreeValue(self.owner.ctx, result);
                    JS_FreeValue(self.owner.ctx, import.resolve);
                    JS_FreeValue(self.owner.ctx, import.reject);
                    JS_FreeValue(self.owner.ctx, import.attributes);
                },
            }
        }
        data.pending_dynamic_imports.borrow_mut().extend(remaining);
    }
    pub fn with_module_loader<R>(
        &mut self,
        loader: Box<dyn ModuleLoader>,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.data().loader.replace(Some(loader));
        unsafe {
            JS_SetModuleLoaderFunc(
                self.owner.rt,
                Some(module_normalize),
                Some(module_load),
                self.owner.callbacks.cast(),
            );
        }
        operation(self)
    }
    /// Requests come from QuickJS's compiled record, never a source scanner.
    pub fn module_requests(&mut self, module: &Value) -> JsResult<Vec<String>> {
        module.checked(self)?;
        if JS_VALUE_GET_TAG(module.raw()) != JS_TAG_MODULE {
            return Err(self.type_error("Invalid module"));
        }
        let pointer = JS_VALUE_GET_PTR(module.raw()).cast::<JSModuleDef>();
        let count = unsafe { JS_GetModuleRequestCount(self.owner.ctx, pointer) };
        if count < 0 {
            return Err(self.take_exception());
        }
        let mut requests = Vec::new();
        for index in 0..count {
            let atom = unsafe { JS_GetModuleRequestName(self.owner.ctx, pointer, index) };
            if atom == JS_ATOM_NULL as JSAtom {
                return Err(self.take_exception());
            }
            let raw = unsafe { JS_AtomToString(self.owner.ctx, atom) };
            unsafe {
                JS_FreeAtom(self.owner.ctx, atom);
            }
            let value = self.accept(raw)?;
            requests.push(self.to_rust_string(&value)?);
        }
        Ok(requests)
    }
    pub fn prepare_module_graph(&mut self, root_name: &str, root: &Value) -> JsResult<Vec<Value>> {
        fn visit(
            ctx: &mut Context,
            name: &str,
            module: &Value,
            seen: &mut HashMap<usize, Value>,
        ) -> JsResult<()> {
            module.checked(ctx)?;
            if JS_VALUE_GET_TAG(module.raw()) != JS_TAG_MODULE {
                return Err(ctx.type_error("Invalid module"));
            }
            let identity = JS_VALUE_GET_PTR(module.raw()) as usize;
            if ctx.data().prepared_modules.borrow().contains_key(&identity)
                || seen.contains_key(&identity)
            {
                return Ok(());
            }
            seen.insert(identity, module.clone());
            let mut children = Vec::new();
            for specifier in ctx.module_requests(module)? {
                let resolved = ctx
                    .data()
                    .loader
                    .borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "Module resolver is unavailable".to_owned())
                    .and_then(|loader| loader.resolve(name, &specifier));
                let url = resolved.map_err(|message| ctx.type_error(&message))?;
                let compiled = ctx
                    .data()
                    .loader
                    .borrow()
                    .as_ref()
                    .and_then(|loader| loader.compiled(&url));
                let child = match compiled.or_else(|| ctx.lookup_module(&url)) {
                    Some(module) => {
                        module.checked(ctx)?;
                        ctx.data()
                            .modules
                            .borrow_mut()
                            .insert(url.clone(), module.clone());
                        module
                    }
                    None => {
                        let source = ctx
                            .data()
                            .loader
                            .borrow_mut()
                            .as_mut()
                            .ok_or_else(|| "Module resolver is unavailable".to_owned())
                            .and_then(|loader| loader.load(&url));
                        let source = source.map_err(|message| ctx.type_error(&message))?;
                        ctx.compile_module(&url, &source)?
                    }
                };
                children.push((url, child));
            }
            for (url, module) in children {
                visit(ctx, &url, &module, seen)?;
            }
            Ok(())
        }
        let mut seen = HashMap::new();
        visit(self, root_name, root, &mut seen)?;
        Ok(seen.into_values().collect())
    }
    pub fn mark_module_graph_prepared(&mut self, modules: Vec<Value>) {
        self.data().prepared_modules.borrow_mut().extend(
            modules
                .into_iter()
                .map(|module| (JS_VALUE_GET_PTR(module.raw()) as usize, module)),
        );
    }
    pub fn lookup_module(&self, name: &str) -> Option<Value> {
        self.data().modules.borrow().get(name).cloned()
    }
    pub fn compile_module(&mut self, name: &str, source: &str) -> JsResult<Value> {
        let value = self.compile_module_unregistered(name, source)?;
        self.data()
            .modules
            .borrow_mut()
            .insert(name.to_owned(), value.clone());
        Ok(value)
    }
    pub fn compile_module_unregistered(&mut self, name: &str, source: &str) -> JsResult<Value> {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        let started = profile.then(std::time::Instant::now);
        if profile {
            eprintln!(
                "javascript-module-compile-start source={name:?} bytes={}",
                source.len()
            );
        }
        let result = unsafe {
            let mut code = source.as_bytes().to_vec();
            code.push(0);
            let filename =
                CString::new(name).map_err(|_| self.type_error("Module URL contains NUL"))?;
            let raw = JS_Eval(
                self.owner.ctx,
                code.as_ptr().cast(),
                source.len(),
                filename.as_ptr(),
                JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY | JS_EVAL_FLAG_HOST_NO_RESOLVE,
            );
            let value = self.accept(raw)?;
            set_import_meta(self.owner.ctx, JS_VALUE_GET_PTR(value.raw()).cast(), name)?;
            Ok(value)
        };
        if let Some(started) = started {
            eprintln!(
                "javascript-module-compile-profile source={name:?} bytes={} ms={:.3}",
                source.len(),
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        result
    }
    /// Hydrate only trusted in-process compiler output on this context owner.
    /// The module remains unresolved and unevaluated, as in CompileModule.
    pub fn hydrate_module_unregistered(
        &mut self,
        name: &str,
        bytes: &[u8],
        metadata: &JSCompiledSourceMetadata,
    ) -> JsResult<Value> {
        let profile = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
        let started = profile.then(std::time::Instant::now);
        let result = unsafe {
            let raw = JS_ReadObject(
                self.owner.ctx,
                bytes.as_ptr(),
                bytes.len(),
                JS_READ_OBJ_BYTECODE,
            );
            let value = self.accept(raw)?;
            if JS_VALUE_GET_TAG(value.raw()) != JS_TAG_MODULE {
                return Err(self.type_error("Compiler output is not a module"));
            }
            if !JS_ImportCompiledSourceMetadata(self.owner.ctx, value.raw(), metadata) {
                return Err(self.type_error("Compiler source metadata is unavailable"));
            }
            set_import_meta(self.owner.ctx, JS_VALUE_GET_PTR(value.raw()).cast(), name)?;
            Ok(value)
        };
        if let Some(started) = started {
            eprintln!(
                "javascript-module-hydrate-profile source={name:?} bytes={} ms={:.3}",
                bytes.len(),
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        result
    }
    pub fn resolve_module(&mut self, module: &Value) -> JsResult<()> {
        module.checked(self)?;
        unsafe {
            if JS_ResolveModule(self.owner.ctx, module.raw()) < 0 {
                Err(self.take_exception())
            } else {
                Ok(())
            }
        }
    }
    pub fn evaluate_module(&mut self, module: &Value) -> JsResult<Value> {
        module.checked(self)?;
        unsafe {
            self.accept(JS_EvalFunction(
                self.owner.ctx,
                JS_DupValue(self.owner.ctx, module.raw()),
            ))
        }
    }
}
struct NativeCallback {
    callback: fn(&mut Context, &Value, &[Value], i32) -> JsResult<Value>,
    magic: i32,
}
impl HostCallable for NativeCallback {
    fn call(
        &self,
        ctx: &mut Context,
        this: &Value,
        args: &[Value],
        _: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        (self.callback)(ctx, this, args, self.magic)
    }
}
unsafe fn native_dispatch(
    ctx: *mut JSContext,
    this: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
    _magic: i32,
    data: *mut JSValue,
) -> JSValue {
    let mut context = Context::borrowed(ctx);
    let opaque = JS_GetOpaque(*data, native_holder_class()).cast::<Rc<NativeHolder>>();
    if opaque.is_null() {
        return JS_ThrowTypeError(ctx, c"Invalid native capture".as_ptr());
    }
    let holder = &*opaque;
    let function = holder.callable.borrow().as_ref().cloned();
    let Some(function) = function else {
        return JS_ThrowTypeError(ctx, c"Native realm has been released".as_ptr());
    };
    let captures = (0..holder.data_count)
        .map(|i| Value::take(&context.owner, JS_DupValue(ctx, *data.add(i + 1))))
        .collect::<Vec<_>>();
    let this = Value::take(&context.owner, JS_DupValue(ctx, this));
    // These are still owned, duplicated values for the entire native call,
    // including reentry. Only the temporary argument container is inline.
    let mut inline_args: [Value; 8] = std::array::from_fn(|_| Value::Undefined);
    let heap_args;
    let args = if argc as usize <= inline_args.len() {
        for (i, slot) in inline_args.iter_mut().take(argc as usize).enumerate() {
            *slot = Value::take(&context.owner, JS_DupValue(ctx, *argv.add(i)));
        }
        &inline_args[..argc as usize]
    } else {
        heap_args = (0..argc as usize)
            .map(|i| Value::take(&context.owner, JS_DupValue(ctx, *argv.add(i))))
            .collect::<Vec<_>>();
        heap_args.as_slice()
    };
    match function.call(&mut context, &this, args, &captures, &Value::Undefined) {
        Ok(value) => match value.checked(&mut context) {
            Ok(raw) => JS_DupValue(ctx, raw),
            Err(error) => JS_Throw(ctx, JS_DupValue(ctx, error.raw())),
        },
        Err(value) => match value.checked(&mut context) {
            Ok(raw) => JS_Throw(ctx, JS_DupValue(ctx, raw)),
            Err(error) => JS_Throw(ctx, JS_DupValue(ctx, error.raw())),
        },
    }
}
unsafe fn rejection_dispatch(
    ctx: *mut JSContext,
    promise: JSValueConst,
    reason: JSValueConst,
    handled: JS_BOOL,
    opaque: *mut c_void,
) {
    let data = &*(opaque as *const CallbackState);
    let mut context = Context::borrowed(ctx);
    let promise = Value::take(&context.owner, JS_DupValue(ctx, promise));
    let reason = Value::take(&context.owner, JS_DupValue(ctx, reason));
    data.rejection_events
        .borrow_mut()
        .push_back((promise, reason, handled != 0));
    // A tracker may evaluate JS and synchronously trigger another rejection.
    // Release registry borrows before calling it and drain nested notifications
    // after the current call returns, preserving their order.
    if data.tracker_running.replace(true) {
        return;
    }
    struct Reset<'a>(&'a Cell<bool>);
    impl Drop for Reset<'_> {
        fn drop(&mut self) {
            self.0.set(false);
        }
    }
    let _reset = Reset(&data.tracker_running);
    loop {
        let event = data.rejection_events.borrow_mut().pop_front();
        let Some((promise, reason, handled)) = event else {
            break;
        };
        let generation = data.tracker_generation.get();
        let tracker = data.tracker.borrow_mut().take();
        if let (Some(mut tracker), Some(promise)) = (tracker, promise.as_object()) {
            tracker.on_rejection(&mut context, promise, &reason, handled);
            if data.tracker_generation.get() == generation {
                data.tracker.replace(Some(tracker));
            }
        }
    }
}
unsafe fn host_job(ctx: *mut JSContext, argc: i32, argv: *mut JSValueConst) -> JSValue {
    JS_Call(ctx, *argv, JS_UNDEFINED, argc - 1, argv.add(1))
}
unsafe fn dynamic_import(
    ctx: *mut JSContext,
    resolve: JSValueConst,
    reject: JSValueConst,
    base: JSValueConst,
    specifier: JSValueConst,
    attributes: JSValueConst,
    opaque: *mut c_void,
) -> i32 {
    let data = &*opaque.cast::<CallbackState>();
    let base_ptr = JS_ToCString(ctx, base);
    let specifier_ptr = JS_ToCString(ctx, specifier);
    if base_ptr.is_null() || specifier_ptr.is_null() {
        if !base_ptr.is_null() {
            JS_FreeCString(ctx, base_ptr);
        }
        if !specifier_ptr.is_null() {
            JS_FreeCString(ctx, specifier_ptr);
        }
        return -1;
    }
    let base_string = CStr::from_ptr(base_ptr).to_string_lossy().into_owned();
    let specifier_string = CStr::from_ptr(specifier_ptr).to_string_lossy().into_owned();
    JS_FreeCString(ctx, base_ptr);
    JS_FreeCString(ctx, specifier_ptr);
    let request = data
        .loader
        .borrow_mut()
        .as_mut()
        .ok_or_else(|| "Not supported".to_owned())
        .and_then(|loader| loader.request_dynamic(&base_string, &specifier_string));
    match request {
        Ok(Some(url)) => {
            data.pending_dynamic_imports
                .borrow_mut()
                .push(PendingDynamicImport {
                    resolve: JS_DupValue(ctx, resolve),
                    reject: JS_DupValue(ctx, reject),
                    attributes: JS_DupValue(ctx, attributes),
                    base: base_string,
                    specifier: specifier_string,
                    url,
                });
            0
        }
        Ok(None) => {
            let message = format!("Unresolved dynamic module import: {specifier_string}");
            let mut context = Context::borrowed(ctx);
            let error = context.make_plain_error(&message);
            let mut reason = error.raw();
            let result = JS_Call(ctx, reject, JS_UNDEFINED, 1, &mut reason);
            if JS_IsException(result) != 0 {
                return -1;
            }
            JS_FreeValue(ctx, result);
            0
        }
        Err(message) => {
            let mut context = Context::borrowed(ctx);
            let error = context.make_plain_error(&message);
            let mut reason = error.raw();
            let result = JS_Call(ctx, reject, JS_UNDEFINED, 1, &mut reason);
            if JS_IsException(result) != 0 {
                return -1;
            }
            JS_FreeValue(ctx, result);
            0
        }
    }
}
unsafe fn consume_internal_module_promise(
    ctx: *mut JSContext,
    _: *mut JSModuleDef,
    promise: JSValueConst,
    _: *mut c_void,
) {
    let _ = JS_MarkPromiseHandled(ctx, promise);
}
unsafe fn module_normalize(
    ctx: *mut JSContext,
    base: *const c_char,
    specifier: *const c_char,
    opaque: *mut c_void,
) -> *mut c_char {
    let data = &*(opaque as *const CallbackState);
    let base = CStr::from_ptr(base).to_string_lossy();
    let specifier = CStr::from_ptr(specifier).to_string_lossy();
    let resolved = data
        .loader
        .borrow_mut()
        .as_mut()
        .ok_or_else(|| "Module loader is unavailable".to_owned())
        .and_then(|loader| loader.resolve(&base, &specifier));
    match resolved {
        Ok(url) => match CString::new(url) {
            Ok(url) => js_strdup(ctx, url.as_ptr()),
            Err(_) => {
                JS_ThrowTypeError(ctx, c"Module URL contains NUL".as_ptr());
                ptr::null_mut()
            }
        },
        Err(message) => {
            JS_ThrowTypeError(ctx, format_args!("{message}"));
            ptr::null_mut()
        }
    }
}
unsafe fn set_import_meta(
    ctx: *mut JSContext,
    module: *mut JSModuleDef,
    url: &str,
) -> JsResult<()> {
    let mut context = Context::borrowed(ctx);
    let meta = JS_GetImportMeta(ctx, module);
    if JS_IsException(meta) != 0 {
        return Err(context.take_exception());
    }
    let value = JS_NewStringLen(ctx, url.as_ptr().cast(), url.len());
    let ret = JS_DefinePropertyValueStr(
        ctx,
        meta,
        c"url".as_ptr(),
        value,
        JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE | JS_PROP_ENUMERABLE,
    );
    JS_FreeValue(ctx, meta);
    if ret < 0 {
        Err(context.take_exception())
    } else {
        Ok(())
    }
}
unsafe fn module_load(
    ctx: *mut JSContext,
    name: *const c_char,
    opaque: *mut c_void,
) -> *mut JSModuleDef {
    let data = &*(opaque as *const CallbackState);
    let name = CStr::from_ptr(name).to_string_lossy().into_owned();
    if let Some(module) = data.modules.borrow().get(&name) {
        return JS_VALUE_GET_PTR(module.raw()).cast();
    }
    let source = data
        .loader
        .borrow_mut()
        .as_mut()
        .ok_or_else(|| "Module loader is unavailable".to_owned())
        .and_then(|loader| loader.load(&name));
    match source {
        Ok(source) => {
            let mut code = source.as_bytes().to_vec();
            code.push(0);
            let filename = CString::new(name.as_bytes()).unwrap();
            let raw = JS_Eval(
                ctx,
                code.as_ptr().cast(),
                source.len(),
                filename.as_ptr(),
                JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY | JS_EVAL_FLAG_HOST_NO_RESOLVE,
            );
            if JS_IsException(raw) != 0 {
                return ptr::null_mut();
            }
            let module = JS_VALUE_GET_PTR(raw).cast();
            if let Err(error) = set_import_meta(ctx, module, &name) {
                JS_FreeValue(ctx, raw);
                JS_Throw(ctx, JS_DupValue(ctx, error.raw()));
                return ptr::null_mut();
            }
            let context = Context::borrowed(ctx);
            data.modules
                .borrow_mut()
                .insert(name.clone(), Value::take(&context.owner, raw));
            module
        }
        Err(message) => {
            JS_ThrowTypeError(ctx, format_args!("{message}"));
            ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_compile_execute_profile_matches_ordinary_eval() {
        for source in [
            "this === globalThis && (function(){return this})() === globalThis",
            "'use strict'; this === globalThis && (function(){return this})() === undefined",
            "var audit=0; let lexical=7; function update(){audit+=lexical}; update(); audit",
            "globalThis.audit=1; Promise.resolve().then(()=>audit++); audit",
            "globalThis.audit=2; eval('audit+=3'); audit",
            "globalThis.audit=1;\nthrow new TypeError('phase failure')",
            "globalThis.audit=1;\nlet broken = ;",
        ] {
            let mut outcomes = Vec::new();
            for profile in [false, true] {
                let mut context = Context::new();
                let result = context.eval_named_profiled(
                    source,
                    SourceType::Script,
                    "phase-test.js",
                    profile,
                );
                let outcome = match result {
                    Ok(value) => (true, context.to_rust_string(&value).unwrap(), None),
                    Err(error) => {
                        let location = context.take_exception_location();
                        let key = PropKey::from_string(&context.intern("stack"));
                        let value = context.get_property(&error, &key).unwrap();
                        (false, context.to_rust_string(&value).unwrap(), location)
                    }
                };
                let before_jobs = context
                    .eval("typeof audit === 'undefined' ? 'unset' : String(audit)")
                    .unwrap();
                let before_jobs = context.to_rust_string(&before_jobs).unwrap();
                assert!(context.run_jobs_errors().is_empty());
                let after_jobs = context
                    .eval("typeof audit === 'undefined' ? 'unset' : String(audit)")
                    .unwrap();
                let after_jobs = context.to_rust_string(&after_jobs).unwrap();
                outcomes.push((outcome, before_jobs, after_jobs));
            }
            assert_eq!(outcomes[0], outcomes[1], "phase split changed {source:?}");
        }
        // SourceType::Module remains on JS_Eval's normal module path.
        let mut outcomes = Vec::new();
        for profile in [false, true] {
            let mut context = Context::new();
            let value = context
                .eval_named_profiled(
                    "globalThis.audit = this === undefined; export const answer=42;",
                    SourceType::Module,
                    "phase-module.js",
                    profile,
                )
                .unwrap();
            let returned = context.to_rust_string(&value).unwrap();
            assert!(context.run_jobs_errors().is_empty());
            let value = context.eval("audit").unwrap();
            outcomes.push((returned, value.as_boolean()));
        }
        assert_eq!(outcomes[0], outcomes[1]);
        assert_eq!(outcomes[0].1, Some(true));
    }

    #[test]
    fn pending_job_profile_preserves_fifo_nested_jobs_and_failure_continuation() {
        let mut outcomes = Vec::new();
        for profile in [false, true] {
            let mut context = Context::new();
            context.eval("globalThis.order=[]").unwrap();
            for source in [
                "()=>{order.push('first'); Promise.resolve().then(()=>order.push('nested'))}",
                "()=>{order.push('throw'); throw new TypeError('job failure')}",
                "()=>order.push('last')",
            ] {
                let callback = context.eval(source).unwrap();
                context.enqueue_job(callback, Vec::new()).unwrap();
            }
            let errors = context.run_jobs_errors_profiled(profile);
            assert_eq!(errors.len(), 1);
            let failure = context.to_rust_string(&errors[0]).unwrap();
            let value = context.eval("order.join(',')").unwrap();
            let order = context.to_rust_string(&value).unwrap();
            assert_eq!(order, "first,throw,last,nested");
            assert_eq!(failure, "TypeError: job failure");
            assert!(context.run_jobs_errors_profiled(profile).is_empty());
            outcomes.push((order, failure));
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }

    #[test]
    fn rope_concatenation_is_a_string_at_the_public_host_boundary() {
        let mut context = Context::new();
        let value = context.eval("'a'.repeat(600)+'b'.repeat(600)").unwrap();
        assert_eq!(
            value.as_string().unwrap().to_string_lossy(),
            format!("{}{}", "a".repeat(600), "b".repeat(600))
        );
    }

    #[test]
    fn exported_values_keep_their_runtime_alive_until_last_release() {
        let (value, owner) = {
            let mut context = Context::new();
            let value = context.eval("'kept value'").unwrap();
            (value, Rc::downgrade(&context.owner))
        };
        assert!(owner.upgrade().is_some());
        assert_eq!(value.as_string().unwrap().to_string_lossy(), "kept value");
        drop(value);
        assert!(owner.upgrade().is_none());
    }

    struct NestedTracker {
        seen: Rc<RefCell<Vec<String>>>,
    }
    impl RejectionTracker for NestedTracker {
        fn on_rejection(&mut self, ctx: &mut Context, _: &Gc<JsObject>, reason: &Value, _: bool) {
            self.seen
                .borrow_mut()
                .push(ctx.to_rust_string(reason).unwrap());
            if self.seen.borrow().len() == 1 {
                ctx.eval("Promise.reject('nested')").unwrap();
            }
        }
    }
    #[test]
    fn rejection_tracker_can_reenter_js_without_losing_nested_notifications() {
        let mut context = Context::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        context.set_rejection_tracker(Some(Box::new(NestedTracker { seen: seen.clone() })));
        context.eval("Promise.reject('outer')").unwrap();
        assert_eq!(&*seen.borrow(), &["outer", "nested"]);
    }

    #[test]
    fn queued_arguments_keep_their_only_reference_until_enqueue_duplicates_them() {
        let mut context = Context::new();
        let function = context
            .eval("value=>{globalThis.audit=value.secret}")
            .unwrap();
        let value = context.eval("({secret:'must survive'})").unwrap();
        context.enqueue_job(function, vec![value]).unwrap();
        for _ in 0..100 {
            context.eval("({secret:'reuse'})").unwrap();
        }
        assert!(context.run_jobs_errors().is_empty());
        assert_eq!(
            context
                .eval("audit")
                .unwrap()
                .as_string()
                .unwrap()
                .to_string_lossy(),
            "must survive"
        );
    }

    struct DropTracked(Rc<std::cell::Cell<usize>>);
    impl Drop for DropTracked {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    impl HostCallable for DropTracked {
        fn call(
            &self,
            _: &mut Context,
            _: &Value,
            _: &[Value],
            _: &[Value],
            _: &Value,
        ) -> JsResult<Value> {
            Ok(Value::Undefined)
        }
    }
    #[test]
    fn collected_native_functions_release_their_host_captures_before_realm_teardown() {
        let count = Rc::new(std::cell::Cell::new(0));
        let mut context = Context::new();
        for _ in 0..1000 {
            let function = context.new_host_function(
                Rc::new(DropTracked(count.clone())),
                vec![],
                "temporary",
                0,
            );
            drop(function);
        }
        assert_eq!(count.get(), 1000);
        assert!(context.data().functions.borrow().len() <= 64);
        drop(context);
        assert_eq!(count.get(), 1000);
    }
    #[test]
    fn native_capture_registry_remains_bounded_with_live_functions_and_dead_churn() {
        let count = Rc::new(Cell::new(0));
        let mut context = Context::new();
        let retained: Vec<_> = (0..1024)
            .map(|_| {
                context.new_host_function(
                    Rc::new(DropTracked(count.clone())),
                    vec![],
                    "retained",
                    0,
                )
            })
            .collect();
        for _ in 0..10000 {
            drop(context.new_host_function(
                Rc::new(DropTracked(count.clone())),
                vec![],
                "temporary",
                0,
            ));
            assert!(context.data().functions.borrow().len() <= 2 * 1024 + 64);
        }
        assert_eq!(count.get(), 10000);
        assert!(context
            .call(Value::Object(retained[0].clone()), Value::Undefined, &[])
            .is_ok());
        drop(retained);
        for _ in 0..3000 {
            drop(context.new_host_function(
                Rc::new(DropTracked(count.clone())),
                vec![],
                "retire",
                0,
            ));
        }
        assert!(context.data().functions.borrow().len() <= 64);
        assert_eq!(count.get(), 14024);
        drop(context);
        assert_eq!(count.get(), 14024);
    }

    #[test]
    fn engine_marks_native_capture_values_in_an_unreachable_js_cycle() {
        let count = Rc::new(std::cell::Cell::new(0));
        let mut context = Context::new();
        {
            let object = context.new_object();
            let function = context.new_host_function(
                Rc::new(DropTracked(count.clone())),
                vec![Value::Object(object.clone())],
                "cyclic",
                0,
            );
            context.define_value(
                &object,
                "callback",
                Value::Object(function),
                PropFlags::C_W_E,
            );
        }
        assert_eq!(count.get(), 0);
        unsafe {
            JS_RunGC(context.owner.rt);
        }
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn public_host_seam_rejects_foreign_objects_at_all_ownership_entry_points() {
        let mut source = Context::new();
        let foreign = source.eval("({secret:17})").unwrap();
        let foreign_object = foreign.as_object().unwrap().clone();
        let mut target = Context::new();
        let local = target.new_object();
        let key = PropKey::from_string(&target.intern("foreign"));
        assert!(!target.define_own(
            &local,
            key.clone(),
            Property::Data(foreign.clone(), PropFlags::C_W_E)
        ));
        assert!(target.has_property(&foreign_object, &key).is_err());
        assert!(target
            .set_prototype_of(&local, Some(foreign_object.clone()))
            .is_err());
        assert!(target.set_prototype_of(&foreign_object, None).is_err());
        assert!(target
            .new_proxy(foreign_object.clone(), local.clone())
            .is_err());
        assert!(target.new_proxy(local.clone(), foreign_object).is_err());
        assert!(target.resolve_module(&foreign).is_err());
        assert!(target.evaluate_module(&foreign).is_err());
        drop(source);
        assert_eq!(target.eval("1+2").unwrap().as_number(), Some(3.0));
    }

    #[test]
    fn inline_native_arguments_preserve_reentry_order_receiver_and_exceptions() {
        struct Forward(Value);
        impl HostCallable for Forward {
            fn call(
                &self,
                ctx: &mut Context,
                this: &Value,
                args: &[Value],
                _: &[Value],
                _: &Value,
            ) -> JsResult<Value> {
                ctx.call(self.0.clone(), this.clone(), args)
            }
        }
        let mut ctx = Context::new();
        let receiver = ctx.eval("({marker:17})").unwrap();
        let callback = ctx
            .eval(
                r#"(function(){
            if(this.marker!==17)throw Error('receiver');
            for(let i=0;i<arguments.length;i++){
                if(arguments[i].marker!==i)throw Error('argument order');
            }
            if(arguments.length===9)throw new RangeError('nine');
            return arguments.length;
        })"#,
            )
            .unwrap();
        let forward =
            Value::Object(ctx.new_host_function(Rc::new(Forward(callback)), vec![], "forward", 0));
        for len in [0, 8, 9] {
            let args: Vec<_> = (0..len)
                .map(|i| ctx.eval(&format!("({{marker:{i}}})")).unwrap())
                .collect();
            let result = ctx.call(forward.clone(), receiver.clone(), &args);
            if len == 9 {
                assert_eq!(
                    ctx.format_exception(&result.unwrap_err()),
                    "RangeError: nine"
                );
            } else {
                assert_eq!(result.unwrap().as_number(), Some(len as f64));
            }
        }
        assert_eq!(ctx.eval("1+2").unwrap().as_number(), Some(3.0));
    }

    struct ForeignReturn(Value);
    impl HostCallable for ForeignReturn {
        fn call(
            &self,
            _: &mut Context,
            _: &Value,
            _: &[Value],
            _: &[Value],
            _: &Value,
        ) -> JsResult<Value> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn native_callback_cannot_transfer_an_object_between_runtimes() {
        let mut source = Context::new();
        let foreign = source.eval("({secret: 17})").unwrap();
        let mut target = Context::new();
        let callback =
            target.new_host_function(Rc::new(ForeignReturn(foreign)), Vec::new(), "foreign", 0);
        let global = target.global();
        target.define_value(
            &global,
            "foreign",
            Value::Object(callback),
            PropFlags::C_W_E,
        );
        let result = target
            .eval("try { foreign(); 'bad' } catch(e) { e.name+':'+e.message }")
            .unwrap();
        assert_eq!(
            result.as_string().unwrap().to_string_lossy(),
            "TypeError:Value belongs to another realm"
        );
        assert_eq!(target.eval("1+2").unwrap().as_number(), Some(3.0));
    }
}
