// Optional embedding APIs. No C API, default algorithm or internal C layout changes.
// A host may compile an unresolved module graph, choose dynamic import policy,
// and explicitly consume the real synchronous module-internal Promise.
use std::sync::Mutex;
// Opt-in ScriptOrigin host metadata. Out-of-line storage leaves the official
// bytecode layout and ordinary JS_Eval behavior unchanged. Each nested
// function shares its source Script's captured module-resolution base URL.
static SCRIPT_BASE_URLS: std::sync::LazyLock<
    Mutex<std::collections::HashMap<(usize, usize), String>>,
> = std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

pub unsafe fn JS_SetCompiledScriptBaseURL(
    ctx: *mut JSContext,
    script: JSValueConst,
    base_url: &str,
) -> i32 {
    if JS_VALUE_GET_TAG(script) != JS_TAG_FUNCTION_BYTECODE {
        return -1;
    }
    let mut pending = vec![JS_VALUE_GET_PTR(script).cast::<JSFunctionBytecode>()];
    let mut urls = SCRIPT_BASE_URLS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    while let Some(bytecode) = pending.pop() {
        urls.insert(((*ctx).rt as usize, bytecode as usize), base_url.to_owned());
        for index in 0..(*bytecode).cpool_count {
            let value = *(*bytecode).cpool.add(index as usize);
            if JS_VALUE_GET_TAG(value) == JS_TAG_FUNCTION_BYTECODE {
                pending.push(JS_VALUE_GET_PTR(value).cast());
            }
        }
    }
    0
}
unsafe fn js_host_active_script_base_url(ctx: *mut JSContext) -> Option<String> {
    let urls = SCRIPT_BASE_URLS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if urls.is_empty() {
        return None;
    }
    let mut frame = (*(*ctx).rt).current_stack_frame;
    while !frame.is_null() {
        let bytecode = JS_GetFunctionBytecode((*frame).cur_func);
        if !bytecode.is_null() {
            if let Some(url) = urls.get(&((*ctx).rt as usize, bytecode as usize)) {
                return Some(url.clone());
            }
            if (*bytecode).is_direct_or_indirect_eval() == 0 {
                return None;
            }
        }
        frame = (*frame).prev_frame;
    }
    None
}
unsafe fn js_host_inherit_script_base_url(ctx: *mut JSContext, bytecode: *mut JSFunctionBytecode) {
    if let Some(url) = js_host_active_script_base_url(ctx) {
        SCRIPT_BASE_URLS
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(((*ctx).rt as usize, bytecode as usize), url);
    }
}
unsafe fn js_host_free_script_base_url(rt: *mut JSRuntime, bytecode: *mut JSFunctionBytecode) {
    SCRIPT_BASE_URLS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&(rt as usize, bytecode as usize));
}

/// Has an effect only with MODULE | COMPILE_ONLY. Ordinary JS_Eval follows C.
pub const JS_EVAL_FLAG_HOST_NO_RESOLVE: i32 = 1 << 24;
pub type JSHostDynamicImportFunc = unsafe fn(
    ctx: *mut JSContext,
    resolve: JSValueConst,
    reject: JSValueConst,
    base: JSValueConst,
    specifier: JSValueConst,
    attributes: JSValueConst,
    opaque: *mut c_void,
) -> i32;
pub type JSHostModuleInternalPromiseFunc = unsafe fn(
    ctx: *mut JSContext,
    module: *mut JSModuleDef,
    promise: JSValueConst,
    opaque: *mut c_void,
);
#[derive(Clone, Copy)]
struct ModuleEmbeddingHooks {
    runtime: usize,
    dynamic_import: Option<JSHostDynamicImportFunc>,
    internal_promise: Option<JSHostModuleInternalPromiseFunc>,
    opaque: usize,
}
// Out-of-line opt-in storage preserves official sizeof/runtime memory accounting.
// Borrowed callbacks are copied before invocation: hosts may reenter the engine.
static MODULE_EMBEDDING_HOOKS: Mutex<Vec<ModuleEmbeddingHooks>> = Mutex::new(Vec::new());

/// Callbacks receive borrowed values. A dynamic hook runs after the official
/// specifier and import-attributes coercion and owns settling/scheduling the
/// supplied real Promise through its resolve/reject functions. None retains C.
/// The internal hook observes only the Promise synchronously consumed by the
/// module evaluator, never user-created or public module-evaluation Promises.
pub unsafe fn JS_SetModuleEmbeddingHooks(
    runtime: *mut JSRuntime,
    dynamic_import: Option<JSHostDynamicImportFunc>,
    internal_promise: Option<JSHostModuleInternalPromiseFunc>,
    opaque: *mut c_void,
) {
    let mut hooks = MODULE_EMBEDDING_HOOKS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    hooks.retain(|entry| entry.runtime != runtime as usize);
    if dynamic_import.is_some() || internal_promise.is_some() {
        hooks.push(ModuleEmbeddingHooks {
            runtime: runtime as usize,
            dynamic_import,
            internal_promise,
            opaque: opaque as usize,
        });
    }
}

/// Resume a host-deferred dynamic import after its module graph is available.
/// The resolve/reject functions and attributes are borrowed for this call.
pub unsafe fn JS_HostResumeDynamicImport(
    ctx: *mut JSContext,
    resolve: JSValueConst,
    reject: JSValueConst,
    base: *const std::ffi::c_char,
    specifier: *const std::ffi::c_char,
    attributes: JSValueConst,
) {
    let mut resolving = [resolve, reject];
    JS_LoadModuleInternal(ctx, base, specifier, resolving.as_mut_ptr(), attributes);
}
unsafe fn js_clear_module_embedding_hooks(runtime: *mut JSRuntime) {
    SCRIPT_BASE_URLS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .retain(|(owner, _), _| *owner != runtime as usize);
    JS_SetModuleEmbeddingHooks(runtime, None, None, ptr::null_mut());
}
fn js_module_embedding_hooks(runtime: *mut JSRuntime) -> Option<ModuleEmbeddingHooks> {
    MODULE_EMBEDDING_HOOKS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .iter()
        .find(|entry| entry.runtime == runtime as usize)
        .copied()
}
fn js_host_should_resolve_module(flags: i32) -> bool {
    let compile_module = flags & (JS_EVAL_TYPE_MASK | JS_EVAL_FLAG_COMPILE_ONLY)
        == JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY;
    !compile_module || flags & JS_EVAL_FLAG_HOST_NO_RESOLVE == 0
}
unsafe fn js_host_dynamic_import_or_enqueue(
    ctx: *mut JSContext,
    argc: i32,
    argv: *mut JSValue,
) -> i32 {
    if let Some(hooks) = js_module_embedding_hooks((*ctx).rt) {
        if let Some(callback) = hooks.dynamic_import {
            let base_url = js_host_active_script_base_url(ctx);
            let base = if let Some(base_url) = base_url {
                JS_NewStringLen(ctx, base_url.as_ptr().cast(), base_url.len())
            } else {
                JS_DupValue(ctx, *argv.add(2))
            };
            if JS_IsException(base) != 0 {
                return -1;
            }
            let result = callback(
                ctx,
                *argv,
                *argv.add(1),
                base,
                *argv.add(3),
                *argv.add(4),
                hooks.opaque as *mut c_void,
            );
            JS_FreeValue(ctx, base);
            return result;
        }
    }
    JS_EnqueueJob(ctx, Some(js_dynamic_import_job), argc, argv)
}
unsafe fn js_host_observe_internal_module_promise(
    ctx: *mut JSContext,
    module: *mut JSModuleDef,
    promise: JSValueConst,
) {
    if JS_IsException(promise) == 0 {
        if let Some(hooks) = js_module_embedding_hooks((*ctx).rt) {
            if let Some(callback) = hooks.internal_promise {
                callback(ctx, module, promise, hooks.opaque as *mut c_void);
            }
        }
    }
}

/// Count the static module requests without loading or resolving dependencies.
pub unsafe fn JS_GetModuleRequestCount(ctx: *mut JSContext, module: *mut JSModuleDef) -> i32 {
    if module.is_null() {
        JS_ThrowTypeError(ctx, c"invalid module".as_ptr());
        return -1;
    }
    (*module).req_module_entries_count
}
/// Returns an owned atom; release it with JS_FreeAtom. Invalid indexes throw.
pub unsafe fn JS_GetModuleRequestName(
    ctx: *mut JSContext,
    module: *mut JSModuleDef,
    index: i32,
) -> JSAtom {
    if module.is_null() || index < 0 || index >= (*module).req_module_entries_count {
        JS_ThrowRangeError(ctx, c"invalid module request index".as_ptr());
        return crate::quickjs_atom::JS_ATOM_NULL as JSAtom;
    }
    JS_DupAtom(
        ctx,
        (*(*module).req_module_entries.add(index as usize)).module_name,
    )
}
/// Mark a Promise synchronously consumed by an embedding as handled, using the
/// same rejection-tracker notification as perform_promise_then. No reactions,
/// values, or jobs are added; this does not deduplicate different Promises.
pub unsafe fn JS_MarkPromiseHandled(ctx: *mut JSContext, promise: JSValueConst) -> i32 {
    let state = JS_GetOpaque2(ctx, promise, JS_CLASS_PROMISE as JSClassID).cast::<JSPromiseData>();
    if state.is_null() {
        return -1;
    }
    if (*state).is_handled == 0 {
        let runtime = (*ctx).rt;
        if (*state).promise_state == JS_PROMISE_REJECTED {
            if let Some(callback) = (*runtime).host_promise_rejection_tracker {
                callback(
                    ctx,
                    promise,
                    (*state).promise_result,
                    1,
                    (*runtime).host_promise_rejection_tracker_opaque,
                );
            }
        }
        (*state).is_handled = 1;
    }
    0
}

/// Embedding equivalent of PerformPromiseThen. The existing intrinsic owns
/// reaction allocation and the Agent job queue. No author `then` getter or
/// Promise constructor is consulted and no result Promise is exposed.
pub unsafe fn JS_PerformPromiseThen(
    ctx: *mut JSContext,
    promise: JSValueConst,
    fulfilled: JSValueConst,
    rejected: JSValueConst,
) -> i32 {
    if JS_GetOpaque2(ctx, promise, JS_CLASS_PROMISE as JSClassID).is_null() {
        return -1;
    }
    let mut handlers = [fulfilled, rejected];
    let mut capability = [JS_UNDEFINED; 2];
    perform_promise_then(ctx, promise, handlers.as_mut_ptr(), capability.as_mut_ptr())
}
