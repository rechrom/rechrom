// quickjs.c:30228..30236. Module namespace has-property behavior. MIT.
unsafe fn js_module_ns_has(_ctx: *mut JSContext, obj: JSValueConst, atom: JSAtom) -> i32 {
    (!find_own_property1(JS_VALUE_GET_PTR(obj).cast(), atom).is_null()) as i32
}
static js_module_ns_exotic_methods: JSClassExoticMethods = JSClassExoticMethods {
    get_own_property: None,
    get_own_property_names: None,
    delete_property: None,
    define_own_property: None,
    has_property: Some(js_module_ns_has),
    get_property: None,
    set_property: None,
    get_prototype: None,
    set_prototype: None,
    is_extensible: None,
    prevent_extensions: None,
};
