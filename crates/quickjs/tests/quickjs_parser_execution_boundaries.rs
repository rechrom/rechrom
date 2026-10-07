// Test-only boundaries for the parser/compiler oracle. These paths are
// deliberately not substituted in production. Any fixture entering them
// fails, rather than claiming JavaScript execution before the VM connects.
unsafe fn JS_Call(_ctx: *mut JSContext, _func: JSValueConst, _this: JSValueConst, _argc: i32, _argv: *mut JSValueConst) -> JSValue {
    panic!("execution outside parser compile-only oracle")
}
unsafe fn JS_NewCFunctionData(_ctx: *mut JSContext, _func: Option<JSCFunctionData>, _length: i32, _magic: i32, _data_len: i32, _data: *mut JSValueConst) -> JSValue {
    panic!("C-function allocation outside parser compile-only oracle")
}
unsafe fn JS_NewPromiseCapability(_ctx: *mut JSContext, _funcs: *mut JSValue) -> JSValue {
    panic!("promise execution outside parser compile-only oracle")
}
unsafe fn js_promise_then(_ctx: *mut JSContext, _this: JSValueConst, _argc: i32, _argv: *mut JSValueConst) -> JSValue {
    panic!("promise execution outside parser compile-only oracle")
}
unsafe fn js_closure(_ctx: *mut JSContext, _bfunc: JSValue, _refs: *mut *mut JSVarRef, _sf: *mut JSStackFrame, _eval: JS_BOOL) -> JSValue {
    panic!("closure execution outside parser compile-only oracle")
}
unsafe fn js_closure2(_ctx: *mut JSContext, _func: JSValue, _bc: *mut JSFunctionBytecode, _refs: *mut *mut JSVarRef, _sf: *mut JSStackFrame, _eval: JS_BOOL, _m: *mut JSModuleDef) -> JSValue {
    panic!("closure execution outside parser compile-only oracle")
}
unsafe fn js_async_function_call(_ctx: *mut JSContext, _func: JSValue, _this: JSValue, _argc: i32, _argv: *mut JSValue, _flags: i32) -> JSValue {
    panic!("async execution outside parser compile-only oracle")
}
unsafe fn JS_EnqueueJob(_ctx: *mut JSContext, _func: Option<JSJobFunc>, _argc: i32, _argv: *mut JSValueConst) -> i32 {
    panic!("job execution outside parser compile-only oracle")
}

unsafe fn js_free_desc(_ctx: *mut JSContext, _desc: *mut JSPropertyDescriptor) { panic!("exotic descriptor outside parser oracle") }
