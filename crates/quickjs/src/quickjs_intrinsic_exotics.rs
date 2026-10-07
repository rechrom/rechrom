// quickjs.c:8552..8565,16141..16166,44951..45038. MIT.
unsafe fn js_arguments_define_own_property(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    val: JSValueConst,
    getter: JSValueConst,
    setter: JSValueConst,
    flags: i32,
) -> i32 {
    let p = JS_VALUE_GET_PTR(this_obj).cast::<JSObject>();
    let mut idx = 0;
    if (*p).fast_array() != 0
        && JS_AtomIsArrayIndex(ctx, &mut idx, prop) != 0
        && idx < (*p).u.array.count
    {
        if convert_fast_array_to_array(ctx, p) != 0 {
            return -1;
        }
    }
    JS_DefineProperty(
        ctx,
        this_obj,
        prop,
        val,
        getter,
        setter,
        flags | JS_PROP_NO_EXOTIC,
    )
}
const fn empty_exotic_methods() -> JSClassExoticMethods {
    JSClassExoticMethods {
        get_own_property: None,
        get_own_property_names: None,
        delete_property: None,
        define_own_property: None,
        has_property: None,
        get_property: None,
        set_property: None,
        get_prototype: None,
        set_prototype: None,
        is_extensible: None,
        prevent_extensions: None,
    }
}
static mut js_arguments_exotic_methods: JSClassExoticMethods = JSClassExoticMethods {
    define_own_property: Some(js_arguments_define_own_property),
    ..empty_exotic_methods()
};

unsafe fn js_string_get_own_property(
    ctx: *mut JSContext,
    desc: *mut JSPropertyDescriptor,
    obj: JSValueConst,
    prop: JSAtom,
) -> i32 {
    if __JS_AtomIsTaggedInt(prop) != 0 {
        let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        if JS_VALUE_GET_TAG((*p).u.object_data) == JS_TAG_STRING {
            let p1 = JS_VALUE_GET_PTR((*p).u.object_data).cast::<JSString>();
            let idx = __JS_AtomToUInt32(prop);
            if idx < (*p1).len() {
                if !desc.is_null() {
                    let ch = string_get(p1, idx as i32);
                    (*desc).flags = JS_PROP_ENUMERABLE;
                    (*desc).value = js_new_string_char(ctx, ch as u16);
                    (*desc).getter = JS_UNDEFINED;
                    (*desc).setter = JS_UNDEFINED;
                }
                return 1;
            }
        }
    }
    0
}
unsafe fn js_string_define_own_property(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    val: JSValueConst,
    getter: JSValueConst,
    setter: JSValueConst,
    flags: i32,
) -> i32 {
    if __JS_AtomIsTaggedInt(prop) != 0 {
        let idx = __JS_AtomToUInt32(prop);
        let p = JS_VALUE_GET_PTR(this_obj).cast::<JSObject>();
        if JS_VALUE_GET_TAG((*p).u.object_data) == JS_TAG_STRING {
            let p1 = JS_VALUE_GET_PTR((*p).u.object_data).cast::<JSString>();
            if idx < (*p1).len() {
                let mut fail = check_define_prop_flags(JS_PROP_ENUMERABLE, flags) == 0;
                if flags & JS_PROP_HAS_VALUE != 0 {
                    if JS_VALUE_GET_TAG(val) != JS_TAG_STRING {
                        fail = true;
                    } else {
                        let p2 = JS_VALUE_GET_PTR(val).cast::<JSString>();
                        if (*p2).len() != 1 || string_get(p1, idx as i32) != string_get(p2, 0) {
                            fail = true;
                        }
                    }
                }
                if fail {
                    return JS_ThrowTypeErrorOrFalse(ctx, flags, c"property is not configurable".as_ptr());
                }
                return 1;
            }
        }
    }
    JS_DefineProperty(
        ctx,
        this_obj,
        prop,
        val,
        getter,
        setter,
        flags | JS_PROP_NO_EXOTIC,
    )
}
unsafe fn js_string_delete_property(ctx: *mut JSContext, obj: JSValueConst, prop: JSAtom) -> i32 {
    if __JS_AtomIsTaggedInt(prop) != 0
        && __JS_AtomToUInt32(prop) < js_string_obj_get_length(ctx, obj)
    {
        return 0;
    }
    1
}
static mut js_string_exotic_methods: JSClassExoticMethods = JSClassExoticMethods {
    get_own_property: Some(js_string_get_own_property),
    define_own_property: Some(js_string_define_own_property),
    delete_property: Some(js_string_delete_property),
    ..empty_exotic_methods()
};
