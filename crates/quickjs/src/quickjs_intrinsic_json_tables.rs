// quickjs.c:50165..50175. Exact built-in JSON property order. MIT.
static mut js_json_funcs: [JSCFunctionListEntry; 5] = [
    JS_CFUNC_DEF(c"isRawJSON".as_ptr(), 1, Some(js_json_isRawJSON)),
    JS_CFUNC_DEF(c"parse".as_ptr(), 2, Some(js_json_parse)),
    JS_CFUNC_DEF(c"rawJSON".as_ptr(), 1, Some(js_json_rawJSON)),
    JS_CFUNC_DEF(c"stringify".as_ptr(), 3, Some(js_json_stringify)),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"JSON".as_ptr(), JS_PROP_CONFIGURABLE),
];
static mut js_json_obj: [JSCFunctionListEntry; 1] = [
    JS_OBJECT_DEF(c"JSON".as_ptr(), ptr::addr_of_mut!(js_json_funcs).cast(), 5, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE),
];
