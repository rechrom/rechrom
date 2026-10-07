// quickjs.c:51408-51419,51447-51461. Original Symbol descriptors/order. MIT.
static mut js_symbol_proto_funcs: [JSCFunctionListEntry; 5] = [
    JS_CFUNC_DEF(c"toString".as_ptr(), (0) as i32, Some(js_symbol_toString)),
    JS_CFUNC_DEF(c"valueOf".as_ptr(), (0) as i32, Some(js_symbol_valueOf)),
    JS_CFUNC_DEF(c"[Symbol.toPrimitive]".as_ptr(), (1) as i32, Some(js_symbol_valueOf)),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Symbol".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
    JS_CGETSET_DEF(c"description".as_ptr(), Some(js_symbol_get_description), None),
];

static mut js_symbol_funcs: [JSCFunctionListEntry; 15] = [
    JS_CFUNC_DEF(c"for".as_ptr(), (1) as i32, Some(js_symbol_for)),
    JS_CFUNC_DEF(c"keyFor".as_ptr(), (1) as i32, Some(js_symbol_keyFor)),
    JS_PROP_ATOM_DEF(c"toPrimitive".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_toPrimitive) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"iterator".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_iterator) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"match".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_match) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"matchAll".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_matchAll) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"replace".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_replace) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"search".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_search) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"split".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_split) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"toStringTag".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_toStringTag) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"isConcatSpreadable".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_isConcatSpreadable) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"hasInstance".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_hasInstance) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"species".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_species) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"unscopables".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_unscopables) as i32, (0) as i32),
    JS_PROP_ATOM_DEF(c"asyncIterator".as_ptr(), (crate::quickjs_atom::JS_ATOM_Symbol_asyncIterator) as i32, (0) as i32),
];
