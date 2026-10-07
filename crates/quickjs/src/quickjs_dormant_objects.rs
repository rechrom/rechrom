// Original quickjs.c #if 0 helpers: 5785-5803 and 57511-57516. MIT.
// Preserve source algorithms without installing any additional JavaScript properties.
unsafe fn JS_GetObjectData(ctx: *mut JSContext, obj: JSValueConst) -> JSValue {
    if JS_VALUE_GET_TAG(obj) == JS_TAG_OBJECT {
        let p = JS_VALUE_GET_OBJ(obj);
        match (*p).class_id as u32 {
            JS_CLASS_NUMBER | JS_CLASS_STRING | JS_CLASS_BOOLEAN | JS_CLASS_SYMBOL
            | JS_CLASS_DATE | JS_CLASS_BIG_INT => return JS_DupValue(ctx, (*p).u.object_data),
            _ => {},
        }
    }
    JS_UNDEFINED
}
unsafe fn js_typed_array___create(
    ctx: *mut JSContext, this_val: JSValueConst, argc: i32, argv: *mut JSValueConst,
) -> JSValue {
    js_typed_array_create(ctx, *argv, (argc - 1).max(0), argv.add(1))
}
