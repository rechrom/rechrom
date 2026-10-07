// Exact official quickjs.c Number/Boolean table order and descriptors. MIT.
static mut js_number_funcs: [JSCFunctionListEntry; 14] = [
    JS_ALIAS_BASE_DEF(c"parseInt".as_ptr(), c"parseInt".as_ptr(), 0),
    JS_ALIAS_BASE_DEF(c"parseFloat".as_ptr(), c"parseFloat".as_ptr(), 0),
    JS_CFUNC_DEF(c"isNaN".as_ptr(), 1, Some(js_number_isNaN)),
    JS_CFUNC_DEF(c"isFinite".as_ptr(), 1, Some(js_number_isFinite)),
    JS_CFUNC_DEF(c"isInteger".as_ptr(), 1, Some(js_number_isInteger)),
    JS_CFUNC_DEF(c"isSafeInteger".as_ptr(), 1, Some(js_number_isSafeInteger)),
    JS_PROP_DOUBLE_DEF(c"MAX_VALUE".as_ptr(), 1.7976931348623157e+308, 0),
    JS_PROP_DOUBLE_DEF(c"MIN_VALUE".as_ptr(), 5e-324, 0),
    JS_PROP_DOUBLE_DEF(c"NaN".as_ptr(), f64::NAN, 0),
    JS_PROP_DOUBLE_DEF(c"NEGATIVE_INFINITY".as_ptr(), -f64::INFINITY, 0),
    JS_PROP_DOUBLE_DEF(c"POSITIVE_INFINITY".as_ptr(), f64::INFINITY, 0),
    JS_PROP_DOUBLE_DEF(c"EPSILON".as_ptr(), 2.220446049250313e-16, 0),
    JS_PROP_DOUBLE_DEF(c"MAX_SAFE_INTEGER".as_ptr(), 9007199254740991.0, 0),
    JS_PROP_DOUBLE_DEF(c"MIN_SAFE_INTEGER".as_ptr(), -9007199254740991.0, 0),
];
static mut js_number_proto_funcs: [JSCFunctionListEntry; 6] = [
    JS_CFUNC_DEF(c"toExponential".as_ptr(), 1, Some(js_number_toExponential)),
    JS_CFUNC_DEF(c"toFixed".as_ptr(), 1, Some(js_number_toFixed)),
    JS_CFUNC_DEF(c"toPrecision".as_ptr(), 1, Some(js_number_toPrecision)),
    JS_CFUNC_MAGIC_DEF(c"toString".as_ptr(), 1, Some(js_number_toString), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"toLocaleString".as_ptr(), 0, Some(js_number_toString), (1) as i32),
    JS_CFUNC_DEF(c"valueOf".as_ptr(), 0, Some(js_number_valueOf)),
];
static mut js_boolean_proto_funcs: [JSCFunctionListEntry; 2] = [
    JS_CFUNC_DEF(c"toString".as_ptr(), 0, Some(js_boolean_toString)),
    JS_CFUNC_DEF(c"valueOf".as_ptr(), 0, Some(js_boolean_valueOf)),
];
