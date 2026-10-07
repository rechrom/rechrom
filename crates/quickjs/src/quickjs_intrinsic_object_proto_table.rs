// quickjs.c:40914-40926. Original Object.prototype registration order.
// Raw-pointer metadata is read through addr_of!; the table is never mutated.
static mut js_object_proto_funcs: [JSCFunctionListEntry; 11] = [
    JS_CFUNC_DEF(c"toString".as_ptr(), 0, Some(js_object_toString)),
    JS_CFUNC_DEF(
        c"toLocaleString".as_ptr(),
        0,
        Some(js_object_toLocaleString),
    ),
    JS_CFUNC_DEF(c"valueOf".as_ptr(), 0, Some(js_object_valueOf)),
    JS_CFUNC_DEF(
        c"hasOwnProperty".as_ptr(),
        1,
        Some(js_object_hasOwnProperty),
    ),
    JS_CFUNC_DEF(c"isPrototypeOf".as_ptr(), 1, Some(js_object_isPrototypeOf)),
    JS_CFUNC_DEF(
        c"propertyIsEnumerable".as_ptr(),
        1,
        Some(js_object_propertyIsEnumerable),
    ),
    JS_CGETSET_DEF(
        c"__proto__".as_ptr(),
        Some(js_object_get___proto__),
        Some(js_object_set___proto__),
    ),
    JS_CFUNC_MAGIC_DEF(
        c"__defineGetter__".as_ptr(),
        2,
        Some(js_object___defineGetter__),
        0,
    ),
    JS_CFUNC_MAGIC_DEF(
        c"__defineSetter__".as_ptr(),
        2,
        Some(js_object___defineGetter__),
        1,
    ),
    JS_CFUNC_MAGIC_DEF(
        c"__lookupGetter__".as_ptr(),
        1,
        Some(js_object___lookupGetter__),
        0,
    ),
    JS_CFUNC_MAGIC_DEF(
        c"__lookupSetter__".as_ptr(),
        1,
        Some(js_object___lookupGetter__),
        1,
    ),
];
