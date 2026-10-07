// Official QuickJS Promise/async class callback table, exact class order. MIT.
static js_async_class_def: [JSClassShortDef; 9] = [
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_Promise, finalizer: Some(js_promise_finalizer), gc_mark: Some(js_promise_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_PromiseResolveFunction, finalizer: Some(js_promise_resolve_function_finalizer), gc_mark: Some(js_promise_resolve_function_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_PromiseRejectFunction, finalizer: Some(js_promise_resolve_function_finalizer), gc_mark: Some(js_promise_resolve_function_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_AsyncFunction, finalizer: Some(js_bytecode_function_finalizer), gc_mark: Some(js_bytecode_function_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_AsyncFunctionResolve, finalizer: Some(js_async_function_resolve_finalizer), gc_mark: Some(js_async_function_resolve_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_AsyncFunctionReject, finalizer: Some(js_async_function_resolve_finalizer), gc_mark: Some(js_async_function_resolve_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_empty_string, finalizer: Some(js_async_from_sync_iterator_finalizer), gc_mark: Some(js_async_from_sync_iterator_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_AsyncGeneratorFunction, finalizer: Some(js_bytecode_function_finalizer), gc_mark: Some(js_bytecode_function_mark) },
    JSClassShortDef { class_name: crate::quickjs_atom::JS_ATOM_AsyncGenerator, finalizer: Some(js_async_generator_finalizer), gc_mark: Some(js_async_generator_mark) },
];
const PROMISE_MAGIC_all: i32 = 0;
const PROMISE_MAGIC_allSettled: i32 = 1;
const PROMISE_MAGIC_any: i32 = 2;
