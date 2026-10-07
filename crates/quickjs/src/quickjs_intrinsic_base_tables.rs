// Exact official quickjs.c Number/Boolean table order and descriptors. MIT.
static mut js_global_funcs: [JSCFunctionListEntry; 15] = [
    JS_CFUNC_DEF(c"parseInt".as_ptr(), 2, Some(js_parseInt)),
    JS_CFUNC_DEF(c"parseFloat".as_ptr(), 1, Some(js_parseFloat)),
    JS_CFUNC_DEF(c"isNaN".as_ptr(), 1, Some(js_global_isNaN)),
    JS_CFUNC_DEF(c"isFinite".as_ptr(), 1, Some(js_global_isFinite)),
    JS_CFUNC_MAGIC_DEF(c"decodeURI".as_ptr(), 1, Some(js_global_decodeURI), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"decodeURIComponent".as_ptr(), 1, Some(js_global_decodeURI), (1) as i32),
    JS_CFUNC_MAGIC_DEF(c"encodeURI".as_ptr(), 1, Some(js_global_encodeURI), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"encodeURIComponent".as_ptr(), 1, Some(js_global_encodeURI), (1) as i32),
    JS_CFUNC_DEF(c"escape".as_ptr(), 1, Some(js_global_escape)),
    JS_CFUNC_DEF(c"unescape".as_ptr(), 1, Some(js_global_unescape)),
    JS_PROP_DOUBLE_DEF(c"Infinity".as_ptr(), f64::INFINITY, 0),
    JS_PROP_DOUBLE_DEF(c"NaN".as_ptr(), f64::NAN, 0),
    JS_PROP_UNDEFINED_DEF(c"undefined".as_ptr(), 0),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"global".as_ptr(), JS_PROP_CONFIGURABLE),
    JS_CFUNC_DEF(c"eval".as_ptr(), 1, Some(js_global_eval)),
];
static mut js_generator_proto_funcs: [JSCFunctionListEntry; 4] = [
    JS_ITERATOR_NEXT_DEF(c"next".as_ptr(), 1, Some(js_generator_next), GEN_MAGIC_NEXT),
    JS_ITERATOR_NEXT_DEF(c"return".as_ptr(), 1, Some(js_generator_next), GEN_MAGIC_RETURN),
    JS_ITERATOR_NEXT_DEF(c"throw".as_ptr(), 1, Some(js_generator_next), GEN_MAGIC_THROW),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Generator".as_ptr(), JS_PROP_CONFIGURABLE),
];
static mut js_generator_function_proto_funcs: [JSCFunctionListEntry; 1] = [
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"GeneratorFunction".as_ptr(), JS_PROP_CONFIGURABLE),
];
