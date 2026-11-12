//! Native Navigation Web IDL projection. Algorithms and retained event state
//! stay in interaction; every operation borrows the currently executing VM.
use super::*;
use interaction::default::core::navigation_api::{
    navigate_event::{
        NavigateEventOwner, NavigationController, NavigationFocusReset, NavigationHandler,
        NavigationHistoryBehavior, NavigationInterceptOptions, NavigationNavigateOptions,
        NavigationScrollBehavior,
    },
    navigation_api::{
        NavigationApi, NavigationOptions, NavigationReloadOptions, NavigationRestoreCallback,
        NavigationType,
    },
};
use interaction::dispatch::bindings::script_invocation::{
    NativeMicrotask, PromiseReaction, ScriptExceptionMessage, ScriptInvocation, ScriptPromise,
    ScriptPromiseCapability, ScriptSourceLocation,
};
use javascript::javascript_runtime::{JavaScriptNativeFunction, JavaScriptUndefined};

pub(super) struct NavigationInvocation<'a, 'runtime>(pub(super) AbortInvocation<'a, 'runtime>);
impl NavigationInvocation<'_, '_> {
    fn Result(&self, result: HostResult) -> Result<Value, Value> {
        if let Some(thrown) = result.thrown {
            return Err(value(thrown));
        }
        if let Some(error) = result.exception {
            return Err(value(
                self.0
                    .runtime
                    .borrow_mut()
                    .NewError(error.kind, &error.message),
            ));
        }
        Ok(value(result.value))
    }
    fn Function(
        &self,
        callback: Rc<dyn JavaScriptNativeFunction>,
    ) -> Result<JavaScriptFunction, Value> {
        let result = self.0.runtime.borrow_mut().CreateNativeFunction(callback);
        let function = self.Result(result)?;
        match host(&function) {
            HostValue::JavaScriptFunction(function) => Ok(function),
            _ => unreachable!("CreateNativeFunction returns a native function"),
        }
    }
    fn Callback(&self, action: Action) -> Result<JavaScriptFunction, Value> {
        let captures = match &action {
            Action::Reaction(reaction) => reaction.Captures(),
            Action::Microtask(task) => task.Captures(),
        };
        let captures = captures
            .into_iter()
            .map(|owner| host(&self.NativeObjectValue(owner)))
            .collect();
        self.Function(Rc::new(NavigationCallback {
            owner: self.0.owner.self_.clone(),
            action,
            captures,
        }))
    }
    fn SerializationError(&self, error: javascript::JavaScriptSerializationError) -> Value {
        match error {
            javascript::JavaScriptSerializationError::Exception { value: thrown, .. } => {
                value(thrown)
            }
            javascript::JavaScriptSerializationError::DataCloneError(message) => {
                self.DOMException("DataCloneError", &message)
            }
        }
    }
}
impl ScriptInvocation for NavigationInvocation<'_, '_> {
    fn ExceptionMessage(&self, v: &Value) -> ScriptExceptionMessage {
        let message = self.0.runtime.borrow_mut().ExceptionMessage(&host(v));
        ScriptExceptionMessage {
            message: message.message,
            location: message.location.map(location),
        }
    }
    fn TaskAttributionTracker(
        &self,
    ) -> Option<Rc<foundation::task_attribution::TaskAttributionTracker>> {
        Some(self.0.runtime.borrow().TaskAttributionTracker())
    }
    fn IsInContext(&self) -> bool {
        self.0.runtime.borrow().IsInContext()
    }
    fn EventListenerInvoker(&self) -> &dyn ScopedEventListenerInvoker {
        self
    }
    fn ContextIsValid(&self) -> bool {
        !self.0.owner.destroyed.get() && self.0.runtime.borrow().CallbackContext().ContextIsValid()
    }
    fn WorldIdentity(&self) -> u64 {
        self.0.runtime.borrow().CallbackContext().Identity()
    }
    fn WorldId(&self) -> i32 {
        self.0.runtime.borrow().CallbackContext().WorldId()
    }
    fn CaptureStackSourceLocation(&self) -> Option<ScriptSourceLocation> {
        self.0
            .runtime
            .borrow()
            .CaptureSourceLocation(0)
            .map(location)
    }
    fn ScriptToolInvocationId(&self) -> Option<[u64; 2]> {
        self.0.runtime.borrow().ScriptToolInvocationId()
    }
    fn WithWorld(&self, world: u64, callback: &mut dyn FnMut(&dyn ScriptInvocation)) {
        // Navigation's bindings are invoked in their originating ScriptState.
        // Another realm must be lent by its own Window before reaching here.
        if self.WorldIdentity() == world && self.ContextIsValid() {
            callback(self);
        }
    }
    fn WithRunMicrotasksScope(&self, callback: &mut dyn FnMut()) {
        let scope = self.0.runtime.borrow_mut().DeferMicrotaskCheckpoint();
        let outermost = scope.IsOutermost();
        callback();
        drop(scope);
        if outermost {
            self.0.runtime.borrow_mut().PerformMicrotaskCheckpoint();
        }
    }
    fn Undefined(&self) -> Value {
        value(HostValue::default())
    }
    fn Null(&self) -> Value {
        value(HostValue::Null(JavaScriptNull))
    }
    fn IsUndefined(&self, v: &Value) -> bool {
        self.0.runtime.borrow_mut().IsUndefined(&host(v))
    }
    fn IsNull(&self, v: &Value) -> bool {
        self.0.runtime.borrow_mut().IsNull(&host(v))
    }
    fn IsCallable(&self, v: &Value) -> bool {
        self.0.runtime.borrow_mut().IsCallable(&host(v))
    }
    fn IsObject(&self, v: &Value) -> bool {
        self.0.runtime.borrow_mut().IsObject(&host(v))
    }
    fn String(&self, v: &[u16]) -> Value {
        value(HostValue::String16(v.into()))
    }
    fn ToString(&self, v: &Value) -> Result<Vec<u16>, Value> {
        let result = self.0.runtime.borrow_mut().ToString(&host(v));
        let result = self.Result(result)?;
        Ok(self
            .PrimitiveString(&result)
            .expect("ToString returns string"))
    }
    fn ToInt32(&self, v: &Value) -> Result<i32, Value> {
        let result = self.0.runtime.borrow_mut().ToInt32(&host(v));
        match host(&self.Result(result)?) {
            HostValue::Number(v) => Ok(v as i32),
            _ => unreachable!(),
        }
    }
    fn Get(&self, v: &Value, property: &str) -> Result<Value, Value> {
        let result = self.0.runtime.borrow_mut().GetProperty(&host(v), property);
        self.Result(result)
    }
    fn TypeError(&self, message: &[u16]) -> Value {
        let result = self.0.runtime.borrow_mut().NewTypeError(message);
        value(result.thrown.unwrap_or(result.value))
    }
    fn EvaluateClassicScript(&self, source: &[u16], base_url: &str) -> Result<Value, Value> {
        let result = self
            .0
            .runtime
            .borrow_mut()
            .EvaluateClassicScript(source, base_url);
        self.Result(result)
    }
    fn PrimitiveString(&self, v: &Value) -> Option<Vec<u16>> {
        self.0
            .runtime
            .borrow_mut()
            .PrimitiveString(&host(v))
            .map(|s| s.to_vec())
    }
    fn DOMException(&self, name: &'static str, message: &str) -> Value {
        AbortSignalInvocation::DOMException(&self.0, name, message)
    }
    fn Call(&self, function: &Value, receiver: &Value, args: &[Value]) -> Result<Value, Value> {
        let args = args.iter().map(host).collect::<Vec<_>>();
        let result = self
            .0
            .runtime
            .borrow_mut()
            .CallValue(&host(function), &host(receiver), &args);
        self.Result(result)
    }
    fn ReportException(&self, v: &Value) {
        self.0.runtime.borrow_mut().ReportException(&host(v));
    }
    fn NewPromiseCapability(&self) -> Result<ScriptPromiseCapability, Value> {
        let result = self.0.runtime.borrow_mut().NewPromiseCapability();
        let capability = result.map_err(|error| {
            value(
                self.0
                    .runtime
                    .borrow_mut()
                    .NewError(error.kind, &error.message),
            )
        })?;
        Ok(ScriptPromiseCapability {
            promise: ScriptPromise(value(capability.promise)),
            resolve: value(HostValue::JavaScriptFunction(capability.resolve)),
            reject: value(HostValue::JavaScriptFunction(capability.reject)),
        })
    }
    fn PromiseResolve(&self, v: Value) -> Result<ScriptPromise, Value> {
        let result = self.0.runtime.borrow_mut().PromiseResolve(&host(&v));
        self.Result(result).map(ScriptPromise)
    }
    fn PerformPromiseThen(
        &self,
        promise: &ScriptPromise,
        fulfilled: Rc<dyn PromiseReaction>,
        rejected: Rc<dyn PromiseReaction>,
    ) -> Result<(), Value> {
        let fulfilled = self.Callback(Action::Reaction(fulfilled))?;
        let rejected = self.Callback(Action::Reaction(rejected))?;
        let result = self.0.runtime.borrow_mut().PerformPromiseThen(
            &host(&promise.0),
            &fulfilled,
            &rejected,
        );
        self.Result(result).map(|_| ())
    }
    fn MarkPromiseHandled(&self, promise: &ScriptPromise) {
        let result = self
            .0
            .runtime
            .borrow_mut()
            .MarkPromiseHandled(&host(&promise.0));
        assert!(self.Result(result).is_ok(), "native navigation Promise");
    }
    fn EnqueueMicrotask(&self, task: Rc<dyn NativeMicrotask>) -> Result<(), Value> {
        let callback = self.Callback(Action::Microtask(task))?;
        let result = self.0.runtime.borrow_mut().EnqueueMicrotask(&callback);
        self.Result(result).map(|_| ())
    }
    fn NativeObjectValue(&self, object: Rc<dyn ScriptNativeObject>) -> Value {
        let id = self.0.owner.EnsureObject(object.clone());
        self.0.owner.Trace(object);
        value(self.0.owner.NativeValue(id))
    }
    fn TraceNativeObject(&self, object: Rc<dyn ScriptNativeObject>) {
        self.0.owner.Trace(object);
    }
    fn KeepAlive(&self, values: &[Value]) -> Box<dyn Any> {
        Box::new(values.iter().map(host).collect::<Vec<_>>())
    }
    fn SerializeForStorage(
        &self,
        v: &Value,
    ) -> Result<Rc<dyn navigation_history::SerializedScriptValue>, Value> {
        let result = self.0.runtime.borrow_mut().SerializeForStorage(&host(v));
        result
            .map(|v| {
                Rc::new(NavigationSerializedValue(v))
                    as Rc<dyn navigation_history::SerializedScriptValue>
            })
            .map_err(|e| self.SerializationError(e))
    }
    fn Deserialize(
        &self,
        v: &Rc<dyn navigation_history::SerializedScriptValue>,
    ) -> Result<Value, Value> {
        let storage = javascript::SerializedScriptValue::FromStorageBytes(v.StorageBytes());
        let result = self.0.runtime.borrow_mut().Deserialize(&storage);
        result.map(value).map_err(|e| self.SerializationError(e))
    }
}
fn location(v: javascript::javascript_runtime::JavaScriptSourceLocation) -> ScriptSourceLocation {
    ScriptSourceLocation {
        url: v.source_name,
        line: v.line as u32,
        column: v.column as u32,
    }
}
struct NavigationSerializedValue(javascript::SerializedScriptValue);
impl navigation_history::SerializedScriptValue for NavigationSerializedValue {
    fn StorageBytes(&self) -> std::sync::Arc<[u8]> {
        self.0.StorageBytes()
    }
}
enum Action {
    Reaction(Rc<dyn PromiseReaction>),
    Microtask(Rc<dyn NativeMicrotask>),
}
struct NavigationCallback {
    owner: Weak<AbortBindings>,
    action: Action,
    captures: Vec<HostValue>,
}
impl JavaScriptNativeFunction for NavigationCallback {
    fn CapturedValues(&self) -> Vec<HostValue> {
        self.captures.clone()
    }
    fn Call(
        &self,
        runtime: &mut dyn JavaScriptHostRuntime,
        _: &HostValue,
        args: &[HostValue],
    ) -> HostResult {
        let Some(owner) = self.owner.upgrade().filter(|o| !o.destroyed.get()) else {
            return HostResult::default();
        };
        let i = owner.NavigationInvocation(runtime);
        match &self.action {
            Action::Reaction(reaction) => {
                reaction.React(&i, value(args.first().cloned().unwrap_or_default()))
            }
            Action::Microtask(task) => task.Run(&i),
        }
        HostResult::default()
    }
}
impl ScopedEventListenerInvoker for NavigationInvocation<'_, '_> {
    fn ScriptInvocation(&self) -> Option<&dyn ScriptInvocation> {
        Some(self)
    }
    fn Invoke(
        &self,
        listener: &EventListenerRef,
        context: Option<
            &dyn interaction::dispatch::core::dom::events::event_listener::ExecutionContext,
        >,
        event: &EventRef,
    ) {
        let selected = Listener::From(listener.as_ref());
        let binding = if selected.target == 0 {
            self.0.owner.current_document_bindings.borrow().upgrade()
        } else {
            selected.bindings.upgrade()
        };
        if let Some(binding) =
            binding.filter(|binding| !std::ptr::eq(binding.as_ref(), self.0.owner))
        {
            // The listener keeps its source Window's native owner. Borrow the
            // same current invoker's host runtime while selecting that owner's
            // original realm and namespace; never borrow ScriptEngine again.
            let Some(realm) = binding.realm.borrow().Upgrade() else {
                return;
            };
            let namespace = binding.namespace.borrow().Upgrade();
            let mut runtime = self.0.runtime.borrow_mut();
            if let Err(error) = runtime.WithRealm(&realm, &mut |runtime| {
                let _namespace = namespace
                    .as_ref()
                    .map(|namespace| runtime.EnterHostObjectNamespace(namespace));
                let invocation = binding.NavigationInvocation(runtime);
                invocation.Invoke(listener, context, event);
            }) {
                let exception = runtime.NewError(error.kind, &error.message);
                runtime.ReportException(&exception);
            }
            return;
        }
        if let Some(native) = event.ClassOwner::<NavigateEventOwner>() {
            let existing = self
                .0
                .owner
                .identities
                .borrow()
                .get(&native.ScriptIdentity())
                .copied();
            let id = self.0.owner.events.borrow_mut().BindNative(
                event,
                Listener::From(listener.as_ref()).target,
                || existing.unwrap_or_else(|| self.0.owner.Allocate()),
            );
            self.0.owner.StoreAt(id, native.clone());
            self.0.owner.Trace(native);
        }
        if let Some(typed) = event.Typed::<interaction::default::core::navigation_api::navigation_current_entry_change_event::NavigationCurrentEntryChangeEvent>() {
            let from = typed.borrow().from.clone();
            let id = self.0.owner.events.borrow_mut().BindNative(event, Listener::From(listener.as_ref()).target, || self.0.owner.Allocate());
            let from = from.map(|from| { let id = self.0.owner.EnsureObject(from.clone()); self.0.owner.Trace(from); id });
            self.0.owner.Publish([C::Reference { owner: id, slot: 0, target: from }]);
        }
        if let Some(error) =
            event.Typed::<interaction::dispatch::core::events::error_event::ErrorEvent>()
        {
            let existing = self
                .0
                .owner
                .identities
                .borrow()
                .get(&error.ScriptIdentity())
                .copied();
            let id = self.0.owner.events.borrow_mut().BindNative(
                event,
                Listener::From(listener.as_ref()).target,
                || existing.unwrap_or_else(|| self.0.owner.Allocate()),
            );
            self.0.owner.StoreAt(id, error.clone());
            self.0.owner.Trace(error);
        }
        self.0.Invoke(listener, context, event);
    }
}
impl AbortBindings {
    pub(super) fn NavigationInvocation<'a, 'r>(
        &'a self,
        runtime: &'r mut dyn JavaScriptHostRuntime,
    ) -> NavigationInvocation<'a, 'r> {
        self.NavigationInvocationWithSynthetic(runtime, None)
    }
    pub(super) fn NavigationInvocationWithSynthetic<'a, 'r>(
        &'a self,
        runtime: &'r mut dyn JavaScriptHostRuntime,
        synthetic: Option<(HostValue, HostObjectId)>,
    ) -> NavigationInvocation<'a, 'r> {
        NavigationInvocation(AbortInvocation {
            owner: self,
            runtime: RefCell::new(runtime),
            native_events: RefCell::new(Vec::new()),
            synthetic,
        })
    }
}

struct Handler {
    callback: Value,
    task_state: RefCell<Option<Rc<dyn Any>>>,
}
impl NavigationHandler for Handler {
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        visit(self.callback.clone());
    }
    fn SetTaskState(&self, state: Option<Rc<dyn Any>>) {
        *self.task_state.borrow_mut() = state;
    }
    fn Invoke(
        &self,
        i: &dyn ScriptInvocation,
        event: &Rc<NavigateEventOwner>,
        controller: Option<Rc<NavigationController>>,
    ) -> Option<ScriptPromise> {
        let state = self.task_state.borrow().clone().and_then(|state| {
            state
                .downcast::<foundation::task_attribution::TaskAttributionInfo>()
                .ok()
        });
        let _task_scope = i.TaskAttributionTracker().and_then(|tracker| {
            tracker.SetCurrentTaskStateIfTopLevel(
                state,
                i.IsInContext(),
                foundation::task_attribution::TaskScopeType::Callback,
            )
        });
        let receiver = i.NativeObjectValue(event.clone());
        let args = controller
            .map(|c| vec![i.NativeObjectValue(c)])
            .unwrap_or_default();
        match i
            .Call(&self.callback, &receiver, &args)
            .and_then(|v| i.PromiseResolve(v))
        {
            Ok(promise) => Some(promise),
            Err(error) => {
                i.ReportException(&error);
                None
            }
        }
    }
}
struct Restore {
    callback: Value,
    navigation: Weak<NavigationApi>,
}
impl NavigationRestoreCallback for Restore {
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        visit(self.callback.clone());
    }
    fn Invoke(&self, i: &dyn ScriptInvocation, api: &NavigationApi) {
        let receiver = self
            .navigation
            .upgrade()
            .map(|api| i.NativeObjectValue(api))
            .unwrap_or_else(|| i.Undefined());
        if let Err(error) = i.Call(&self.callback, &receiver, &[]) {
            i.ReportException(&error);
        }
        let _ = api;
    }
}
fn history(v: Option<&HostValue>) -> NavigationHistoryBehavior {
    match v {
        Some(HostValue::String(s)) if s == "push" => NavigationHistoryBehavior::Push,
        Some(HostValue::String(s)) if s == "replace" => NavigationHistoryBehavior::Replace,
        _ => NavigationHistoryBehavior::Auto,
    }
}
fn kind(v: NavigationType) -> HostValue {
    HostValue::String(
        match v {
            NavigationType::Push => "push",
            NavigationType::Replace => "replace",
            NavigationType::Reload => "reload",
            NavigationType::Traverse => "traverse",
        }
        .into(),
    )
}
fn optional(v: Option<&HostValue>) -> Option<Value> {
    v.filter(|v| !matches!(v, HostValue::Undefined(_)))
        .cloned()
        .map(value)
}
fn nav_options(args: &[HostValue], start: usize) -> NavigationNavigateOptions {
    NavigationNavigateOptions {
        history: history(args.get(start)),
        state: args
            .get(start + 1)
            .cloned()
            .map(value)
            .filter(|_| matches!(args.get(start + 2), Some(HostValue::Boolean(true)))),
        info: optional(args.get(start + 3)),
    }
}
impl AbortBindings {
    pub(super) fn InvokeNavigation(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let i = self.NavigationInvocation(runtime);
        // Identity policy preserves state/info/callback identity. Project only
        // Web IDL's already-converted primitive metadata after borrowing the VM.
        let mut args = args
            .iter()
            .map(|arg| {
                let v = value(arg.clone());
                if i.IsUndefined(&v) {
                    HostValue::default()
                } else if let Some(units) = i.PrimitiveString(&v) {
                    String::from_utf16(&units)
                        .map(HostValue::String)
                        .unwrap_or_else(|_| HostValue::String16(units.into()))
                } else {
                    arg.clone()
                }
            })
            .collect::<Vec<_>>();
        let presence = match member {
            "__navigationNavigate" | "__navigationControllerRedirect" => Some(3),
            "__navigationReload" => Some(1),
            _ => None,
        };
        if let Some(index) = presence {
            if let Some(arg) = args.get_mut(index) {
                *arg = HostValue::Boolean(i.ToInt32(&value(arg.clone())).is_ok_and(|n| n != 0));
            }
        }
        let metadata: &[usize] = match member {
            "__navigationCreateNavigateEvent" => &[4, 5, 6, 10, 12, 13, 14],
            "__navigationCreateEntryChangeEvent" => &[3, 4, 5],
            _ => &[],
        };
        for &index in metadata {
            if let Some(arg) = args.get_mut(index) {
                *arg = HostValue::Boolean(i.ToInt32(&value(arg.clone())).is_ok_and(|n| n != 0));
            }
        }
        let args = args.as_slice();
        let result: Result<HostValue, Value> = (|| {
            if member == "__navigationHistoryWindow" && receiver == 0 {
                return Ok(self
                    .history
                    .borrow()
                    .as_ref()
                    .map(|history| host(&i.NativeObjectValue(history.clone())))
                    .unwrap_or_default());
            }
            if member.starts_with("__navigationHistory") {
                let history = self
                    .objects
                    .borrow()
                    .get(&receiver)
                    .cloned()
                    .and_then(|owner| {
                        (owner as Rc<dyn Any>)
                            .downcast::<interaction::default::core::frame::history::History>()
                            .ok()
                    })
                    .ok_or_else(|| {
                        i.TypeError(
                            &"Illegal History invocation"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        )
                    })?;
                let result = match member {
                    "__navigationHistoryState" => Ok(host(&history.StateHelper(&i)?)),
                    "__navigationHistoryLength" => {
                        Ok(HostValue::Number(history.Length(&i)? as f64))
                    }
                    "__navigationHistoryScrollRestoration" => Ok(HostValue::String(
                        if history.ScrollRestoration(&i)?
                            == navigation_history::ScrollRestorationType::Manual
                        {
                            "manual"
                        } else {
                            "auto"
                        }
                        .into(),
                    )),
                    "__navigationHistorySetScrollRestoration" => history
                        .SetScrollRestoration(
                            &i,
                            if string_arg(args.first()) == "manual" {
                                navigation_history::ScrollRestorationType::Manual
                            } else {
                                navigation_history::ScrollRestorationType::Auto
                            },
                        )
                        .map(|_| HostValue::default()),
                    "__navigationHistoryGo" => history
                        .Go(
                            &i,
                            i.ToInt32(&value(args.first().cloned().unwrap_or_default()))?,
                        )
                        .map(|_| HostValue::default()),
                    "__navigationHistoryPushState" | "__navigationHistoryReplaceState" => {
                        let url = args
                            .get(1)
                            .filter(|arg| !i.IsNull(&value((*arg).clone())))
                            .map(|arg| string_arg(Some(arg)));
                        history
                            .StateObjectAdded(
                                &i,
                                &value(args.first().cloned().unwrap_or_default()),
                                url.as_deref(),
                                member == "__navigationHistoryReplaceState",
                            )
                            .map(|_| HostValue::default())
                    }
                    _ => Err(
                        i.TypeError(&"Unknown History member".encode_utf16().collect::<Vec<_>>())
                    ),
                };
                self.Trace(history);
                return result;
            }
            if member == "__navigationCreateNavigateEvent" && receiver == 0 {
                use interaction::default::core::navigation_api::navigate_event::{
                    NavigateEventConstructorInit, NavigateEventData,
                };
                use interaction::default::core::navigation_api::navigation_destination::NavigationDestination;
                let destination = args
                    .get(2)
                    .and_then(|v| self.NativeFromValue::<NavigationDestination>(v))
                    .ok_or_else(|| {
                        i.TypeError(
                            &"destination must be a NavigationDestination"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        )
                    })?;
                let signal = args
                    .get(3)
                    .and_then(|v| self.SignalFromValue(v))
                    .ok_or_else(|| {
                        i.TypeError(
                            &"signal must be an AbortSignal"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        )
                    })?;
                let api = self
                    .navigation
                    .borrow()
                    .clone()
                    .expect("Window Navigation owner before script");
                let b = |n| matches!(args.get(n), Some(HostValue::Boolean(true)));
                let source_element = args.get(11).filter(|v| !i.IsNull(&value((*v).clone()))).cloned().map(|v| interaction::default::core::navigation_api::navigate_event::NavigateEventSourceElement::Script(value(v)));
                let form_data = args
                    .get(7)
                    .filter(|v| !i.IsNull(&value((*v).clone())))
                    .cloned()
                    .map(value);
                let download_request = args
                    .get(8)
                    .filter(|v| !i.IsNull(&value((*v).clone())))
                    .map(|v| string_arg(Some(v)));
                let event = api.CreateUntrustedNavigateEvent(
                    &i,
                    &string_arg(args.first()),
                    NavigateEventConstructorInit {
                        event_init: interaction::dispatch::core::dom::events::event::EventInit {
                            bubbles: b(12),
                            cancelable: b(13),
                            composed: b(14),
                        },
                        navigation_type: navigation_type(&string_arg(args.get(1))),
                        destination,
                        signal,
                        data: NavigateEventData {
                            can_intercept: b(4),
                            user_initiated: b(5),
                            hash_change: b(6),
                            form_data,
                            download_request,
                            has_ua_visual_transition: b(10),
                            source_element,
                        },
                        info: args.get(9).cloned().map(value),
                    },
                );
                let id = self
                    .events
                    .borrow_mut()
                    .BindConstructedNative(&event.EventRef(), || self.Allocate());
                self.StoreAt(id, event.clone());
                self.Trace(event);
                return Ok(self.NativeValue(id));
            }
            if member == "__navigationCreateEntryChangeEvent" && receiver == 0 {
                use interaction::default::core::navigation_api::navigation_history_entry::NavigationHistoryEntryOwner;
                use interaction::default::core::navigation_api::navigation_current_entry_change_event::NavigationCurrentEntryChangeEvent;
                let from = args
                    .get(2)
                    .and_then(|v| self.NativeFromValue::<NavigationHistoryEntryOwner>(v))
                    .ok_or_else(|| {
                        i.TypeError(
                            &"from must be a NavigationHistoryEntry"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        )
                    })?;
                let b = |n| matches!(args.get(n), Some(HostValue::Boolean(true)));
                let native = EventRef::new(NavigationCurrentEntryChangeEvent { event: interaction::dispatch::core::dom::events::event::Event::CreateWithInitializer(&string_arg(args.first()), &interaction::dispatch::core::dom::events::event::EventInit { bubbles: b(3), cancelable: b(4), composed: b(5) }, self.clock.as_ref().expect("Window EventClock").as_ref()), from: Some(from.clone()), navigation_type: args.get(1).filter(|v| !i.IsNull(&value((*v).clone()))).map(|v| navigation_type(&string_arg(Some(v)))) });
                let id = self
                    .events
                    .borrow_mut()
                    .BindConstructedNative(&native, || self.Allocate());
                let from_id = self.EnsureObject(from.clone());
                self.Trace(from);
                self.Publish([C::Reference {
                    owner: id,
                    slot: 0,
                    target: Some(from_id),
                }]);
                return Ok(self.NativeValue(id));
            }
            if member == "__navigationWindow" && receiver == 0 {
                return Ok(self
                    .navigation
                    .borrow()
                    .as_ref()
                    .map(|api| host(&i.NativeObjectValue(api.clone())))
                    .unwrap_or(HostValue::Undefined(JavaScriptUndefined)));
            }
            if member == "__navigationEventBase" {
                let native = self.events.borrow().NativeEvent(receiver).ok_or_else(|| {
                    i.TypeError(
                        &"Illegal Event invocation"
                            .encode_utf16()
                            .collect::<Vec<_>>(),
                    )
                })?;
                let event = native.borrow();
                let base = event.BaseEvent();
                return Ok(match string_arg(args.first()).as_str() {
                    "type" => HostValue::String(base.r#type().into()),
                    "bubbles" => HostValue::Boolean(base.bubbles()),
                    "cancelable" => HostValue::Boolean(base.cancelable()),
                    "composed" => HostValue::Boolean(base.composed()),
                    "isTrusted" => HostValue::Boolean(base.isTrusted()),
                    "defaultPrevented" => HostValue::Boolean(base.defaultPrevented()),
                    "timeStamp" => HostValue::Number(
                        base.timeStamp(self.navigation_time_stamp.borrow().as_deref()),
                    ),
                    _ => HostValue::default(),
                });
            }
            if member.starts_with("__navigationError") {
                let native = self
                    .events
                    .borrow()
                    .NativeEvent(receiver)
                    .expect("native ErrorEvent");
                let typed = native
                    .Typed::<interaction::dispatch::core::events::error_event::ErrorEvent>()
                    .expect("ErrorEvent");
                let event = typed.borrow();
                return Ok(match member {
                    "__navigationErrorMessage" => HostValue::String16(
                        (0..event.message().length())
                            .map(|n| event.message().CodeUnitAt(n))
                            .collect::<Vec<_>>()
                            .into(),
                    ),
                    "__navigationErrorFilename" => HostValue::String(event.filename().into()),
                    "__navigationErrorLine" => HostValue::Number(event.lineno() as f64),
                    "__navigationErrorColumn" => HostValue::Number(event.colno() as f64),
                    "__navigationErrorValue" => host(&event.Error(&i)),
                    _ => HostValue::default(),
                });
            }
            if member.starts_with("__navigationPopState") {
                let native = self.events.borrow().NativeEvent(receiver).ok_or_else(|| {
                    i.TypeError(
                        &"Illegal PopStateEvent invocation"
                            .encode_utf16()
                            .collect::<Vec<_>>(),
                    )
                })?;
                let typed = native
                    .Typed::<interaction::dispatch::core::events::pop_state_event::PopStateEvent>()
                    .ok_or_else(|| {
                        i.TypeError(
                            &"Illegal PopStateEvent invocation"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        )
                    })?;
                let event = typed.borrow();
                return Ok(match member {
                    "__navigationPopStateState" => {
                        let history = self.history.borrow().clone().filter(|history| {
                            history.IsSameAsCurrentState(&event.SerializedState())
                        });
                        if let Some(history) = history {
                            let state = history.StateHelper(&i)?;
                            self.Trace(history);
                            host(&state)
                        } else {
                            host(&event.State(&i)?)
                        }
                    }
                    "__navigationPopStateHasUAVisualTransition" => {
                        HostValue::Boolean(event.hasUAVisualTransition())
                    }
                    _ => HostValue::default(),
                });
            }
            if member.starts_with("__navigationHashChange") {
                let native = self.events.borrow().NativeEvent(receiver).ok_or_else(|| {
                    i.TypeError(
                        &"Illegal HashChangeEvent invocation"
                            .encode_utf16()
                            .collect::<Vec<_>>(),
                    )
                })?;
                let typed = native.Typed::<interaction::dispatch::core::events::hash_change_event::HashChangeEvent>().ok_or_else(|| {
                    i.TypeError(&"Illegal HashChangeEvent invocation".encode_utf16().collect::<Vec<_>>())
                })?;
                let event = typed.borrow();
                return Ok(match member {
                    "__navigationHashChangeOldURL" => HostValue::String(event.oldURL().into()),
                    "__navigationHashChangeNewURL" => HostValue::String(event.newURL().into()),
                    _ => HostValue::default(),
                });
            }
            if member == "__navigationEntryChangeFrom" || member == "__navigationEntryChangeType" {
                let native = self
                    .events
                    .borrow()
                    .NativeEvent(receiver)
                    .expect("native currententrychange event");
                let typed = native.Typed::<interaction::default::core::navigation_api::navigation_current_entry_change_event::NavigationCurrentEntryChangeEvent>().expect("NavigationCurrentEntryChangeEvent");
                let event = typed.borrow();
                return Ok(if member == "__navigationEntryChangeType" {
                    event
                        .navigation_type
                        .map(kind)
                        .unwrap_or(HostValue::Null(JavaScriptNull))
                } else {
                    event
                        .from
                        .clone()
                        .map(|v| host(&i.NativeObjectValue(v)))
                        .unwrap_or(HostValue::Null(JavaScriptNull))
                });
            }
            let object = self
                .objects
                .borrow()
                .get(&receiver)
                .cloned()
                .ok_or_else(|| {
                    i.TypeError(
                        &"Illegal Navigation invocation"
                            .encode_utf16()
                            .collect::<Vec<_>>(),
                    )
                })?;
            if let Some(api) = object.AsAny().downcast_ref::<NavigationApi>() {
                let entry = |e: Option<Rc<dyn interaction::default::core::navigation_api::navigation_api::NavigationHistoryEntry>>| e.map(|e| host(&i.NativeObjectValue(e))).unwrap_or(HostValue::Null(JavaScriptNull));
                let navigation_result = match member {
                    "__navigationNavigate" => {
                        Some(api.Navigate(&i, &string_arg(args.first()), nav_options(args, 1))?)
                    }
                    "__navigationReload" => Some(api.Reload(
                        &i,
                        NavigationReloadOptions {
                            state:
                                args.first().cloned().map(value).filter(|_| {
                                    matches!(args.get(1), Some(HostValue::Boolean(true)))
                                }),
                            info: optional(args.get(2)),
                        },
                    )?),
                    "__navigationTraverse" => Some(api.TraverseTo(
                        &i,
                        &string_arg(args.first()),
                        NavigationOptions {
                            info: optional(args.get(1)),
                        },
                    )?),
                    "__navigationBack" => Some(api.Back(
                        &i,
                        NavigationOptions {
                            info: optional(args.first()),
                        },
                    )?),
                    "__navigationForward" => Some(api.Forward(
                        &i,
                        NavigationOptions {
                            info: optional(args.first()),
                        },
                    )?),
                    _ => None,
                };
                if let Some(r) = navigation_result {
                    return Ok(HostValue::Record(vec![
                        ("committed".into(), host(&r.committed.0)),
                        ("finished".into(), host(&r.finished.0)),
                    ]));
                }
                return Ok(match member {
                    "__navigationCurrentEntry" => entry(api.CurrentEntry()),
                    "__navigationEntries" => {
                        let entries = api.Entries();
                        let mut result =
                            vec![("length".into(), HostValue::Number(entries.len() as f64))];
                        result.extend(
                            entries
                                .into_iter()
                                .enumerate()
                                .map(|(n, e)| (n.to_string(), entry(Some(e)))),
                        );
                        HostValue::Record(result)
                    }
                    "__navigationCanGoBack" => HostValue::Boolean(api.CanGoBack()),
                    "__navigationCanGoForward" => HostValue::Boolean(api.CanGoForward()),
                    "__navigationActivation" => api
                        .Activation()
                        .map(|v| host(&i.NativeObjectValue(v)))
                        .unwrap_or(HostValue::Null(JavaScriptNull)),
                    "__navigationTransition" => api
                        .Transition()
                        .map(|v| host(&i.NativeObjectValue(v)))
                        .unwrap_or(HostValue::Null(JavaScriptNull)),
                    "__navigationUpdateCurrentEntry" => {
                        api.UpdateCurrentEntry(
                            &i,
                            value(args.first().cloned().unwrap_or_default()),
                        )?;
                        HostValue::default()
                    }
                    _ => {
                        return Err(i.TypeError(
                            &"Unknown Navigation member"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        ))
                    }
                });
            }
            if let Some(event) = object.AsAny().downcast_ref::<NavigateEventOwner>() {
                return Ok(match member {
                    "__navigationEventType" => kind(event.NavigationType()),
                    "__navigationEventDestination" => {
                        host(&i.NativeObjectValue(event.destination.clone()))
                    }
                    "__navigationEventSignal" => host(&i.NativeObjectValue(event.Signal())),
                    "__navigationEventInfo" => host(&event.Info()),
                    "__navigationEventCanIntercept" => HostValue::Boolean(event.data.can_intercept),
                    "__navigationEventUserInitiated" => {
                        HostValue::Boolean(event.data.user_initiated)
                    }
                    "__navigationEventHashChange" => HostValue::Boolean(event.data.hash_change),
                    "__navigationEventHasUAVisualTransition" => {
                        HostValue::Boolean(event.data.has_ua_visual_transition)
                    }
                    "__navigationEventFormData" => event
                        .data
                        .form_data
                        .as_ref()
                        .map(host)
                        .unwrap_or(HostValue::Null(JavaScriptNull)),
                    "__navigationEventDownloadRequest" => event
                        .data
                        .download_request
                        .clone()
                        .map(HostValue::String)
                        .unwrap_or(HostValue::Null(JavaScriptNull)),
                    "__navigationEventSourceElement" => match event.data.source_element.as_ref() {
                        Some(interaction::default::core::navigation_api::navigate_event::NavigateEventSourceElement::Script(element)) => host(element),
                        Some(interaction::default::core::navigation_api::navigate_event::NavigateEventSourceElement::Native(target)) => self.listeners.borrow().iter().find_map(|(id, owner)| { let candidate = owner.AsEventTargetRef(); Rc::ptr_eq(&candidate, target).then_some(HostValue::Object(HostObjectRef { id: *id })) }).unwrap_or(HostValue::Null(JavaScriptNull)),
                        None => HostValue::Null(JavaScriptNull),
                    },
                    "__navigationEventIntercept" => {
                        let handler = |v: Option<&HostValue>| {
                            optional(v).map(|v| {
                                Rc::new(Handler {
                                    callback: v,
                                    task_state: Default::default(),
                                }) as Rc<dyn NavigationHandler>
                            })
                        };
                        event.Intercept(
                            &i,
                            NavigationInterceptOptions {
                                handler: handler(args.first()),
                                precommit_handler: handler(args.get(1)),
                                focus_reset: match args.get(2) {
                                    Some(HostValue::String(v)) => Some(if v == "manual" {
                                        NavigationFocusReset::Manual
                                    } else {
                                        NavigationFocusReset::AfterTransition
                                    }),
                                    _ => None,
                                },
                                scroll: match args.get(3) {
                                    Some(HostValue::String(v)) => Some(if v == "manual" {
                                        NavigationScrollBehavior::Manual
                                    } else {
                                        NavigationScrollBehavior::AfterTransition
                                    }),
                                    _ => None,
                                },
                            },
                        )?;
                        HostValue::default()
                    }
                    "__navigationEventScroll" => {
                        event.Scroll(&i)?;
                        HostValue::default()
                    }
                    "__navigationEventDeferPageSwap" => {
                        event.DeferPageSwap(
                            &i,
                            Rc::new(Handler {
                                callback: value(args.first().cloned().unwrap_or_default()),
                                task_state: Default::default(),
                            }),
                        )?;
                        HostValue::default()
                    }
                    _ => {
                        return Err(i.TypeError(
                            &"Unknown NavigateEvent member"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        ))
                    }
                });
            }
            if let Some(controller) = object.AsAny().downcast_ref::<NavigationController>() {
                match member {
                    "__navigationControllerRedirect" => {
                        controller.Redirect(&i, &string_arg(args.first()), nav_options(args, 1))?
                    }
                    "__navigationControllerAddHandler" => controller.AddHandler(
                        &i,
                        Rc::new(Handler {
                            callback: value(args.first().cloned().unwrap_or_default()),
                            task_state: Default::default(),
                        }),
                    )?,
                    "__navigationControllerRestore" => controller.AddRestoreCallback(
                        &i,
                        Rc::new(Restore {
                            callback: value(args.first().cloned().unwrap_or_default()),
                            navigation: self
                                .navigation
                                .borrow()
                                .as_ref()
                                .map(Rc::downgrade)
                                .unwrap_or_default(),
                        }),
                    )?,
                    _ => {
                        return Err(i.TypeError(
                            &"Unknown navigation controller member"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        ))
                    }
                }
                return Ok(HostValue::default());
            }
            if let Some(entry) = object.AsAny().downcast_ref::<interaction::default::core::navigation_api::navigation_history_entry::NavigationHistoryEntryOwner>() {
                use interaction::default::core::navigation_api::navigation_api::NavigationHistoryEntry;
                return Ok(match member {
                    "__navigationEntryKey" => HostValue::String(entry.Key()), "__navigationEntryId" => HostValue::String(entry.Id()), "__navigationEntryURL" => entry.Url().map(HostValue::String).unwrap_or(HostValue::Null(JavaScriptNull)), "__navigationEntryIndex" => HostValue::Number(entry.Index() as f64), "__navigationEntrySameDocument" => HostValue::Boolean(entry.SameDocument()), "__navigationEntryGetState" => match entry.SerializedState() { Some(state) => host(&i.Deserialize(&state)?), None => HostValue::default() },
                    _ => return Err(i.TypeError(&"Unknown NavigationHistoryEntry member".encode_utf16().collect::<Vec<_>>())),
                });
            }
            if let Some(destination) = object.AsAny().downcast_ref::<interaction::default::core::navigation_api::navigation_destination::NavigationDestination>() {
                return Ok(match member {
                    "__navigationDestinationKey" => HostValue::String(destination.Key()), "__navigationDestinationId" => HostValue::String(destination.Id()), "__navigationDestinationURL" => HostValue::String(destination.URL()), "__navigationDestinationIndex" => HostValue::Number(destination.Index() as f64), "__navigationDestinationSameDocument" => HostValue::Boolean(destination.SameDocument()), "__navigationDestinationGetState" => destination.GetState(&i)?.as_ref().map(host).unwrap_or_default(),
                    _ => return Err(i.TypeError(&"Unknown NavigationDestination member".encode_utf16().collect::<Vec<_>>())),
                });
            }
            if let Some(activation) = object.AsAny().downcast_ref::<interaction::default::core::navigation_api::navigation_activation::NavigationActivation>() {
                return Ok(match member {
                    "__navigationActivationType" => kind(activation.NavigationType()), "__navigationActivationEntry" => activation.Entry().map(|v| host(&i.NativeObjectValue(v))).unwrap_or(HostValue::Null(JavaScriptNull)), "__navigationActivationFrom" => activation.From().map(|v| host(&i.NativeObjectValue(v))).unwrap_or(HostValue::Null(JavaScriptNull)),
                    _ => return Err(i.TypeError(&"Unknown NavigationActivation member".encode_utf16().collect::<Vec<_>>())),
                });
            }
            if let Some(transition) = object.AsAny().downcast_ref::<interaction::default::core::navigation_api::navigation_transition::NavigationTransitionOwner>() {
                use interaction::default::core::navigation_api::navigation_api::NavigationTransition;
                return Ok(match member {
                    "__navigationTransitionType" => kind(transition.NavigationType()), "__navigationTransitionFrom" => transition.From().map(|v| host(&i.NativeObjectValue(v))).unwrap_or(HostValue::Null(JavaScriptNull)), "__navigationTransitionDestination" => host(&i.NativeObjectValue(transition.Destination())), "__navigationTransitionCommitted" => host(&transition.Committed(&i)?.0), "__navigationTransitionFinished" => host(&transition.Finished(&i)?.0),
                    _ => return Err(i.TypeError(&"Unknown NavigationTransition member".encode_utf16().collect::<Vec<_>>())),
                });
            }
            Err(i.TypeError(
                &"Illegal Navigation invocation"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            ))
        })();
        if let Some(object) = self.objects.borrow().get(&receiver).cloned() {
            self.Trace(object);
        }
        match result {
            Ok(value) => HostResult {
                value,
                ..Default::default()
            },
            Err(error) => HostResult {
                thrown: Some(host(&error)),
                ..Default::default()
            },
        }
    }
}
fn string_arg(v: Option<&HostValue>) -> String {
    match v {
        Some(HostValue::String(v)) => v.clone(),
        Some(HostValue::String16(v)) => String::from_utf16_lossy(v),
        _ => String::new(),
    }
}
impl DOMJavaScriptBindings {
    /// ClearForReuse changes only this Window's current Document. Its native
    /// namespace, tracing graph, object IDs and EventTarget owners stay live.
    pub fn RebindForWindowReuse(&self) {
        self.window_native_bindings
            .RebindForWindowReuse(&self.document, &self.native_abort);
    }
    pub fn NavigationWindowEventTarget(&self) -> Rc<EventTarget> {
        self.ListenerTarget(0)
    }
    pub fn BindNavigationTimeStampContext(
        &self,
        context: Rc<dyn interaction::dispatch::core::dom::events::event::EventTimeStampContext>,
    ) {
        *self
            .window_native_bindings
            .navigation_time_stamp
            .borrow_mut() = Some(context);
    }
    /// Assemble the native Window owner before exposing its Web IDL getter.
    pub fn InstallNavigationApi(&mut self, api: Rc<NavigationApi>) {
        let id = self.window_native_bindings.EnsureObject(api.clone());
        let old = self
            .window_native_bindings
            .navigation
            .borrow_mut()
            .replace(api.clone());
        assert!(old.is_none(), "one Navigation owner per Window");
        self.window_native_bindings.Publish([C::Root {
            id,
            reason: 9,
            rooted: true,
        }]);
        self.window_native_bindings.Trace(api);
    }
    pub fn InstallHistory(
        &mut self,
        history: Rc<interaction::default::core::frame::history::History>,
    ) {
        let id = self.window_native_bindings.EnsureObject(history.clone());
        assert!(
            self.window_native_bindings
                .history
                .borrow_mut()
                .replace(history.clone())
                .is_none(),
            "one History owner per Window"
        );
        self.window_native_bindings.Publish([C::Root {
            id,
            reason: 9,
            rooted: true,
        }]);
        self.window_native_bindings.Trace(history);
    }
    pub fn RunNavigationInvocation(
        bindings: &Rc<RefCell<Self>>,
        runtime: &mut dyn JavaScriptHostRuntime,
        callback: &mut dyn FnMut(&dyn ScriptInvocation),
    ) {
        let (owner, synthetic) = {
            let bindings = bindings.borrow();
            (
                bindings.window_native_bindings.clone(),
                bindings.services.ActiveSyntheticDispatch(),
            )
        };
        let invocation = owner.NavigationInvocationWithSynthetic(runtime, synthetic);
        callback(&invocation);
    }
    pub fn WithNavigationInvocation(
        &self,
        runtime: &mut dyn JavaScriptHostRuntime,
        callback: &mut dyn FnMut(&dyn ScriptInvocation),
    ) {
        let invocation = self.window_native_bindings.NavigationInvocation(runtime);
        callback(&invocation);
    }
    pub fn CreateNavigationEventTarget(&self) -> Rc<EventTarget> {
        let id = self.window_native_bindings.Allocate();
        let target = event_listeners::CreateGenericTarget(
            self.window_native_bindings.document.clone(),
            self.window_native_bindings.graph.clone(),
            id,
        )
        .EventTarget()
        .expect("native EventTarget base");
        self.window_native_bindings
            .listeners
            .borrow_mut()
            .insert(id, target.clone());
        target
    }
    pub fn NavigationAbortContext(&self) -> Rc<dyn AbortSignalContext> {
        self.window_native_bindings
            .Context(self.window_native_bindings.Allocate())
    }
}

fn navigation_type(v: &str) -> NavigationType {
    match v {
        "reload" => NavigationType::Reload,
        "replace" => NavigationType::Replace,
        "traverse" => NavigationType::Traverse,
        _ => NavigationType::Push,
    }
}

impl DOMJavaScriptBindings {
    /// Lend Navigation's native listener invoker to the existing synchronous
    /// Interaction focus transaction. No runtime or DOM borrow crosses a call.
    pub fn DispatchNavigationDOMListeners(
        bindings: &Rc<RefCell<Self>>,
        invocation: &interaction::event::EventListenerInvocation<'_>,
        script: &dyn ScriptInvocation,
    ) -> interaction::event::EventListenerResult {
        let target = {
            let this = bindings.borrow();
            let target = this
                .listeners
                .borrow()
                .get(&invocation.current_target_node_id)
                .cloned();
            target
        };
        let (Some(target), Some(event)) = (target, invocation.event.canonical.as_ref()) else {
            return Default::default();
        };
        target.FireEventListenersWithInvoker(event, Some(script.EventListenerInvoker()));
        let event = event.borrow();
        let event = event.BaseEvent();
        interaction::event::EventListenerResult {
            prevent_default: event.defaultPrevented(),
            stop_propagation: event.PropagationStopped(),
            stop_immediate_propagation: event.ImmediatePropagationStopped(),
        }
    }
}
