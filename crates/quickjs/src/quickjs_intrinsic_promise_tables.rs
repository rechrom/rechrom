// Exact official quickjs.c Promise/AsyncIterator table order and descriptors. MIT.
static mut js_promise_funcs: [JSCFunctionListEntry; 9] = [
    JS_CFUNC_MAGIC_DEF(c"resolve".as_ptr(), 1, Some(js_promise_resolve), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"reject".as_ptr(), 1, Some(js_promise_resolve), (1) as i32),
    JS_CFUNC_MAGIC_DEF(c"all".as_ptr(), 1, Some(js_promise_all), (PROMISE_MAGIC_all) as i32),
    JS_CFUNC_MAGIC_DEF(c"allSettled".as_ptr(), 1, Some(js_promise_all), (PROMISE_MAGIC_allSettled) as i32),
    JS_CFUNC_MAGIC_DEF(c"any".as_ptr(), 1, Some(js_promise_all), (PROMISE_MAGIC_any) as i32),
    JS_CFUNC_DEF(c"try".as_ptr(), 1, Some(js_promise_try)),
    JS_CFUNC_DEF(c"race".as_ptr(), 1, Some(js_promise_race)),
    JS_CFUNC_DEF(c"withResolvers".as_ptr(), 0, Some(js_promise_withResolvers)),
    JS_CGETSET_DEF(c"[Symbol.species]".as_ptr(), Some(js_get_this), None),
];
static mut js_promise_proto_funcs: [JSCFunctionListEntry; 4] = [
    JS_CFUNC_DEF(c"then".as_ptr(), 2, Some(js_promise_then)),
    JS_CFUNC_DEF(c"catch".as_ptr(), 1, Some(js_promise_catch)),
    JS_CFUNC_DEF(c"finally".as_ptr(), 1, Some(js_promise_finally)),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Promise".as_ptr(), JS_PROP_CONFIGURABLE),
];
static mut js_async_function_proto_funcs: [JSCFunctionListEntry; 1] = [
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"AsyncFunction".as_ptr(), JS_PROP_CONFIGURABLE),
];
static mut js_async_iterator_proto_funcs: [JSCFunctionListEntry; 1] = [
    JS_CFUNC_DEF(c"[Symbol.asyncIterator]".as_ptr(), 0, Some(js_iterator_proto_iterator)),
];
static mut js_async_from_sync_iterator_proto_funcs: [JSCFunctionListEntry; 3] = [
    JS_CFUNC_MAGIC_DEF(c"next".as_ptr(), 1, Some(js_async_from_sync_iterator_next), (GEN_MAGIC_NEXT) as i32),
    JS_CFUNC_MAGIC_DEF(c"return".as_ptr(), 1, Some(js_async_from_sync_iterator_next), (GEN_MAGIC_RETURN) as i32),
    JS_CFUNC_MAGIC_DEF(c"throw".as_ptr(), 1, Some(js_async_from_sync_iterator_next), (GEN_MAGIC_THROW) as i32),
];
static mut js_async_generator_function_proto_funcs: [JSCFunctionListEntry; 1] = [
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"AsyncGeneratorFunction".as_ptr(), JS_PROP_CONFIGURABLE),
];
static mut js_async_generator_proto_funcs: [JSCFunctionListEntry; 4] = [
    JS_CFUNC_MAGIC_DEF(c"next".as_ptr(), 1, Some(js_async_generator_next), (GEN_MAGIC_NEXT) as i32),
    JS_CFUNC_MAGIC_DEF(c"return".as_ptr(), 1, Some(js_async_generator_next), (GEN_MAGIC_RETURN) as i32),
    JS_CFUNC_MAGIC_DEF(c"throw".as_ptr(), 1, Some(js_async_generator_next), (GEN_MAGIC_THROW) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"AsyncGenerator".as_ptr(), JS_PROP_CONFIGURABLE),
];
