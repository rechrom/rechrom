// quickjs.c:49002-49032. Original RegExp descriptors and order. MIT.
static mut js_regexp_funcs: [JSCFunctionListEntry; 2] = [
    JS_CFUNC_DEF(c"escape".as_ptr(), (1) as i32, Some(js_regexp_escape)),
    JS_CGETSET_DEF(c"[Symbol.species]".as_ptr(), Some(js_get_this), None),
];

static mut js_regexp_proto_funcs: [JSCFunctionListEntry; 19] = [
    JS_CGETSET_DEF(c"flags".as_ptr(), Some(js_regexp_get_flags), None),
    JS_CGETSET_DEF(c"source".as_ptr(), Some(js_regexp_get_source), None),
    JS_CGETSET_MAGIC_DEF(c"global".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_GLOBAL) as i32),
    JS_CGETSET_MAGIC_DEF(c"ignoreCase".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_IGNORECASE) as i32),
    JS_CGETSET_MAGIC_DEF(c"multiline".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_MULTILINE) as i32),
    JS_CGETSET_MAGIC_DEF(c"dotAll".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_DOTALL) as i32),
    JS_CGETSET_MAGIC_DEF(c"unicode".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_UNICODE) as i32),
    JS_CGETSET_MAGIC_DEF(c"unicodeSets".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_UNICODE_SETS) as i32),
    JS_CGETSET_MAGIC_DEF(c"sticky".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_STICKY) as i32),
    JS_CGETSET_MAGIC_DEF(c"hasIndices".as_ptr(), Some(js_regexp_get_flag), None, (LRE_FLAG_INDICES) as i32),
    JS_CFUNC_DEF(c"exec".as_ptr(), (1) as i32, Some(js_regexp_exec)),
    JS_CFUNC_DEF(c"compile".as_ptr(), (2) as i32, Some(js_regexp_compile)),
    JS_CFUNC_DEF(c"test".as_ptr(), (1) as i32, Some(js_regexp_test)),
    JS_CFUNC_DEF(c"toString".as_ptr(), (0) as i32, Some(js_regexp_toString)),
    JS_CFUNC_DEF(c"[Symbol.replace]".as_ptr(), (2) as i32, Some(js_regexp_Symbol_replace)),
    JS_CFUNC_DEF(c"[Symbol.match]".as_ptr(), (1) as i32, Some(js_regexp_Symbol_match)),
    JS_CFUNC_DEF(c"[Symbol.matchAll]".as_ptr(), (1) as i32, Some(js_regexp_Symbol_matchAll)),
    JS_CFUNC_DEF(c"[Symbol.search]".as_ptr(), (1) as i32, Some(js_regexp_Symbol_search)),
    JS_CFUNC_DEF(c"[Symbol.split]".as_ptr(), (2) as i32, Some(js_regexp_Symbol_split)),
];

static mut js_regexp_string_iterator_proto_funcs: [JSCFunctionListEntry; 2] = [
    JS_ITERATOR_NEXT_DEF(c"next".as_ptr(), (0) as i32, Some(js_regexp_string_iterator_next), (0) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"RegExp String Iterator".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];
