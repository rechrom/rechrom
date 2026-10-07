// quickjs.c:52968-53059. Exact immutable Map/Set metadata. MIT.
static mut js_map_funcs: [JSCFunctionListEntry; 2] = [
    JS_CFUNC_MAGIC_DEF(c"groupBy".as_ptr(), (2) as i32, Some(js_object_groupBy), (1) as i32),
    JS_CGETSET_DEF(c"[Symbol.species]".as_ptr(), Some(js_get_this), None),
];

static mut js_map_proto_funcs: [JSCFunctionListEntry; 14] = [
    JS_CFUNC_MAGIC_DEF(c"set".as_ptr(), (2) as i32, Some(js_map_set), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"get".as_ptr(), (1) as i32, Some(js_map_get), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"getOrInsert".as_ptr(), (2) as i32, Some(js_map_getOrInsert), ((JS_CLASS_MAP << 1) | 0) as i32),
    JS_CFUNC_MAGIC_DEF(c"getOrInsertComputed".as_ptr(), (2) as i32, Some(js_map_getOrInsert), ((JS_CLASS_MAP << 1) | 1) as i32),
    JS_CFUNC_MAGIC_DEF(c"has".as_ptr(), (1) as i32, Some(js_map_has), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"delete".as_ptr(), (1) as i32, Some(js_map_delete), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"clear".as_ptr(), (0) as i32, Some(js_map_clear), (0) as i32),
    JS_CGETSET_MAGIC_DEF(c"size".as_ptr(), Some(js_map_get_size), None, (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"forEach".as_ptr(), (1) as i32, Some(js_map_forEach), (0) as i32),
    JS_CFUNC_MAGIC_DEF(c"values".as_ptr(), (0) as i32, Some(js_create_map_iterator), (((JS_ITERATOR_KIND_VALUE as i32) << 2) | 0) as i32),
    JS_CFUNC_MAGIC_DEF(c"keys".as_ptr(), (0) as i32, Some(js_create_map_iterator), (((JS_ITERATOR_KIND_KEY as i32) << 2) | 0) as i32),
    JS_CFUNC_MAGIC_DEF(c"entries".as_ptr(), (0) as i32, Some(js_create_map_iterator), (((JS_ITERATOR_KIND_KEY_AND_VALUE as i32) << 2) | 0) as i32),
    JS_ALIAS_DEF(c"[Symbol.iterator]".as_ptr(), c"entries".as_ptr()),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Map".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];

static mut js_set_proto_funcs: [JSCFunctionListEntry; 18] = [
    JS_CFUNC_MAGIC_DEF(c"add".as_ptr(), (1) as i32, Some(js_map_set), (MAGIC_SET) as i32),
    JS_CFUNC_MAGIC_DEF(c"has".as_ptr(), (1) as i32, Some(js_map_has), (MAGIC_SET) as i32),
    JS_CFUNC_MAGIC_DEF(c"delete".as_ptr(), (1) as i32, Some(js_map_delete), (MAGIC_SET) as i32),
    JS_CFUNC_MAGIC_DEF(c"clear".as_ptr(), (0) as i32, Some(js_map_clear), (MAGIC_SET) as i32),
    JS_CGETSET_MAGIC_DEF(c"size".as_ptr(), Some(js_map_get_size), None, (MAGIC_SET) as i32),
    JS_CFUNC_MAGIC_DEF(c"forEach".as_ptr(), (1) as i32, Some(js_map_forEach), (MAGIC_SET) as i32),
    JS_CFUNC_DEF(c"isDisjointFrom".as_ptr(), (1) as i32, Some(js_set_isDisjointFrom)),
    JS_CFUNC_DEF(c"isSubsetOf".as_ptr(), (1) as i32, Some(js_set_isSubsetOf)),
    JS_CFUNC_DEF(c"isSupersetOf".as_ptr(), (1) as i32, Some(js_set_isSupersetOf)),
    JS_CFUNC_DEF(c"intersection".as_ptr(), (1) as i32, Some(js_set_intersection)),
    JS_CFUNC_DEF(c"difference".as_ptr(), (1) as i32, Some(js_set_difference)),
    JS_CFUNC_DEF(c"symmetricDifference".as_ptr(), (1) as i32, Some(js_set_symmetricDifference)),
    JS_CFUNC_DEF(c"union".as_ptr(), (1) as i32, Some(js_set_union)),
    JS_CFUNC_MAGIC_DEF(c"values".as_ptr(), (0) as i32, Some(js_create_map_iterator), (((JS_ITERATOR_KIND_KEY as i32) << 2) | MAGIC_SET) as i32),
    JS_ALIAS_DEF(c"keys".as_ptr(), c"values".as_ptr()),
    JS_ALIAS_DEF(c"[Symbol.iterator]".as_ptr(), c"values".as_ptr()),
    JS_CFUNC_MAGIC_DEF(c"entries".as_ptr(), (0) as i32, Some(js_create_map_iterator), (((JS_ITERATOR_KIND_KEY_AND_VALUE as i32) << 2) | MAGIC_SET) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Set".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];

static mut js_weak_map_proto_funcs: [JSCFunctionListEntry; 7] = [
    JS_CFUNC_MAGIC_DEF(c"set".as_ptr(), (2) as i32, Some(js_map_set), (MAGIC_WEAK) as i32),
    JS_CFUNC_MAGIC_DEF(c"get".as_ptr(), (1) as i32, Some(js_map_get), (MAGIC_WEAK) as i32),
    JS_CFUNC_MAGIC_DEF(c"getOrInsert".as_ptr(), (2) as i32, Some(js_map_getOrInsert), ((JS_CLASS_WEAKMAP << 1) | 0) as i32),
    JS_CFUNC_MAGIC_DEF(c"getOrInsertComputed".as_ptr(), (2) as i32, Some(js_map_getOrInsert), ((JS_CLASS_WEAKMAP << 1) | 1) as i32),
    JS_CFUNC_MAGIC_DEF(c"has".as_ptr(), (1) as i32, Some(js_map_has), (MAGIC_WEAK) as i32),
    JS_CFUNC_MAGIC_DEF(c"delete".as_ptr(), (1) as i32, Some(js_map_delete), (MAGIC_WEAK) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"WeakMap".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];

static mut js_weak_set_proto_funcs: [JSCFunctionListEntry; 4] = [
    JS_CFUNC_MAGIC_DEF(c"add".as_ptr(), (1) as i32, Some(js_map_set), (MAGIC_SET | MAGIC_WEAK) as i32),
    JS_CFUNC_MAGIC_DEF(c"has".as_ptr(), (1) as i32, Some(js_map_has), (MAGIC_SET | MAGIC_WEAK) as i32),
    JS_CFUNC_MAGIC_DEF(c"delete".as_ptr(), (1) as i32, Some(js_map_delete), (MAGIC_SET | MAGIC_WEAK) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"WeakSet".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];

static mut js_map_iterator_proto_funcs: [JSCFunctionListEntry; 2] = [
    JS_ITERATOR_NEXT_DEF(c"next".as_ptr(), (0) as i32, Some(js_map_iterator_next), (0) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Map Iterator".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];

static mut js_set_iterator_proto_funcs: [JSCFunctionListEntry; 2] = [
    JS_ITERATOR_NEXT_DEF(c"next".as_ptr(), (0) as i32, Some(js_map_iterator_next), (MAGIC_SET) as i32),
    JS_PROP_STRING_DEF(c"[Symbol.toStringTag]".as_ptr(), c"Set Iterator".as_ptr(), (JS_PROP_CONFIGURABLE) as i32),
];
static mut js_map_proto_funcs_ptr: [*const JSCFunctionListEntry; 6] = [
    ptr::addr_of!(js_map_proto_funcs).cast(),
    ptr::addr_of!(js_set_proto_funcs).cast(),
    ptr::addr_of!(js_weak_map_proto_funcs).cast(),
    ptr::addr_of!(js_weak_set_proto_funcs).cast(),
    ptr::addr_of!(js_map_iterator_proto_funcs).cast(),
    ptr::addr_of!(js_set_iterator_proto_funcs).cast(),
];
static js_map_proto_funcs_count: [u8; 6] = [14, 18, 7, 4, 2, 2];
