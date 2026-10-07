// quickjs.c:8552-8566. MIT. Shared by property names and String exotics.
unsafe fn js_string_obj_get_length(_ctx: *mut JSContext, obj: JSValueConst) -> u32 {
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    let val = (*p).u.object_data;
    if JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
        (*JS_VALUE_GET_PTR(val).cast::<JSString>()).len()
    } else {
        0
    }
}
