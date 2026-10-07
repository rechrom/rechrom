// quickjs.c:51210-51222,51299-51305. Immutable Proxy metadata. MIT.
static mut js_proxy_exotic_methods: JSClassExoticMethods = JSClassExoticMethods {
    get_own_property: Some(js_proxy_get_own_property),
    define_own_property: Some(js_proxy_define_own_property),
    delete_property: Some(js_proxy_delete_property),
    get_own_property_names: Some(js_proxy_get_own_property_names),
    has_property: Some(js_proxy_has),
    get_property: Some(js_proxy_get),
    set_property: Some(js_proxy_set),
    get_prototype: Some(js_proxy_get_prototype),
    set_prototype: Some(js_proxy_set_prototype),
    is_extensible: Some(js_proxy_is_extensible),
    prevent_extensions: Some(js_proxy_prevent_extensions),
};
static mut js_proxy_funcs: [JSCFunctionListEntry; 1] = [
    JS_CFUNC_DEF(c"revocable".as_ptr(), (2) as i32, Some(js_proxy_revocable)),
];
static js_proxy_class_def: [JSClassShortDef; 1] = [JSClassShortDef {
    class_name: crate::quickjs_atom::JS_ATOM_Object,
    finalizer: Some(js_proxy_finalizer), gc_mark: Some(js_proxy_mark),
}];
