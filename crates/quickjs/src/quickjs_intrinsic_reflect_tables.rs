// quickjs.c:50330-50349. Original Reflect descriptors and order. MIT.
static mut js_reflect_funcs: [JSCFunctionListEntry; 14] = [
    JS_CFUNC_DEF(c"apply".as_ptr(), (3) as i32, Some(js_reflect_apply)),
    JS_CFUNC_DEF(c"construct".as_ptr(), (2) as i32, Some(js_reflect_construct)),
    JS_CFUNC_MAGIC_DEF(c"defineProperty".as_ptr(), (3) as i32, Some(js_object_defineProperty), (1) as i32),
    JS_CFUNC_DEF(c"deleteProperty".as_ptr(), (2) as i32, Some(js_reflect_deleteProperty)),
    JS_CFUNC_DEF(c"get".as_ptr(), (2) as i32, Some(js_reflect_get)),
    JS_CFUNC_MAGIC_DEF(c"getOwnPropertyDescriptor".as_ptr(), (2) as i32, Some(js_object_getOwnPropertyDescriptor), (1) as i32),
    JS_CFUNC_MAGIC_DEF(c"getPrototypeOf".as_ptr(), (1) as i32, Some(js_object_getPrototypeOf), (1) as i32),
    JS_CFUNC_DEF(c"has".as_ptr(), (2) as i32, Some(js_reflect_has)),
    JS_CFUNC_MAGIC_DEF(c"isExtensible".as_ptr(), (1) as i32, Some(js_object_isExtensible), (1) as i32),
    JS_CFUNC_DEF(c"ownKeys".as_ptr(), (1) as i32, Some(js_reflect_ownKeys)),
    JS_CFUNC_MAGIC_DEF(c"preventExtensions".as_ptr(), (1) as i32, Some(js_object_preventExtensions), (1) as i32),
    JS_CFUNC_DEF(c"set".as_ptr(), (3) as i32, Some(js_reflect_set)),
    JS_CFUNC_DEF(c"setPrototypeOf".as_ptr(), (2) as i32, Some(js_reflect_setPrototypeOf)),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Reflect".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];
static mut js_reflect_obj: [JSCFunctionListEntry; 1] = [
    JS_OBJECT_DEF(c"Reflect".as_ptr(), ptr::addr_of!(js_reflect_funcs).cast(), 14, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE),
];
