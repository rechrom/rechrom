// quickjs.c:40889-40912. Original Object static registration order.
// Read through addr_of!; entries and their raw metadata pointers are immutable.
static mut js_object_funcs: [JSCFunctionListEntry; 23] = [
    JS_CFUNC_DEF(c"create".as_ptr(), 2, Some(js_object_create)),
    JS_CFUNC_MAGIC_DEF(
        c"getPrototypeOf".as_ptr(),
        1,
        Some(js_object_getPrototypeOf),
        0,
    ),
    JS_CFUNC_DEF(
        c"setPrototypeOf".as_ptr(),
        2,
        Some(js_object_setPrototypeOf),
    ),
    JS_CFUNC_MAGIC_DEF(
        c"defineProperty".as_ptr(),
        3,
        Some(js_object_defineProperty),
        0,
    ),
    JS_CFUNC_DEF(
        c"defineProperties".as_ptr(),
        2,
        Some(js_object_defineProperties),
    ),
    JS_CFUNC_DEF(
        c"getOwnPropertyNames".as_ptr(),
        1,
        Some(js_object_getOwnPropertyNames),
    ),
    JS_CFUNC_DEF(
        c"getOwnPropertySymbols".as_ptr(),
        1,
        Some(js_object_getOwnPropertySymbols),
    ),
    JS_CFUNC_MAGIC_DEF(c"groupBy".as_ptr(), 2, Some(js_object_groupBy), 0),
    JS_CFUNC_MAGIC_DEF(
        c"keys".as_ptr(),
        1,
        Some(js_object_keys),
        JS_ITERATOR_KIND_KEY as i32,
    ),
    JS_CFUNC_MAGIC_DEF(
        c"values".as_ptr(),
        1,
        Some(js_object_keys),
        JS_ITERATOR_KIND_VALUE as i32,
    ),
    JS_CFUNC_MAGIC_DEF(
        c"entries".as_ptr(),
        1,
        Some(js_object_keys),
        JS_ITERATOR_KIND_KEY_AND_VALUE as i32,
    ),
    JS_CFUNC_MAGIC_DEF(c"isExtensible".as_ptr(), 1, Some(js_object_isExtensible), 0),
    JS_CFUNC_MAGIC_DEF(
        c"preventExtensions".as_ptr(),
        1,
        Some(js_object_preventExtensions),
        0,
    ),
    JS_CFUNC_MAGIC_DEF(
        c"getOwnPropertyDescriptor".as_ptr(),
        2,
        Some(js_object_getOwnPropertyDescriptor),
        0,
    ),
    JS_CFUNC_DEF(
        c"getOwnPropertyDescriptors".as_ptr(),
        1,
        Some(js_object_getOwnPropertyDescriptors),
    ),
    JS_CFUNC_DEF(c"is".as_ptr(), 2, Some(js_object_is)),
    JS_CFUNC_DEF(c"assign".as_ptr(), 2, Some(js_object_assign)),
    JS_CFUNC_MAGIC_DEF(c"seal".as_ptr(), 1, Some(js_object_seal), 0),
    JS_CFUNC_MAGIC_DEF(c"freeze".as_ptr(), 1, Some(js_object_seal), 1),
    JS_CFUNC_MAGIC_DEF(c"isSealed".as_ptr(), 1, Some(js_object_isSealed), 0),
    JS_CFUNC_MAGIC_DEF(c"isFrozen".as_ptr(), 1, Some(js_object_isSealed), 1),
    JS_CFUNC_DEF(c"fromEntries".as_ptr(), 1, Some(js_object_fromEntries)),
    JS_CFUNC_DEF(c"hasOwn".as_ptr(), 2, Some(js_object_hasOwn)),
];
