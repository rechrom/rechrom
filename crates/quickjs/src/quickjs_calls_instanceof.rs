// quickjs.c:8059..8155. Bellard/Gordon MIT.
// Return TRUE, FALSE, or -1 on exception, including exotic prototype chains.
unsafe fn JS_OrdinaryIsInstanceOf(ctx: *mut JSContext, val: JSValueConst, obj: JSValueConst) -> i32 {
    if JS_IsFunction(ctx, obj) == 0 { return 0; }
    let mut p = JS_VALUE_GET_OBJ(obj).cast_const();
    if (*p).class_id as u32 == JS_CLASS_BOUND_FUNCTION {
        return JS_IsInstanceOf(ctx, val, (*(*p).u.bound_function).func_obj);
    }
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT { return 0; }
    let obj_proto = JS_GetProperty(ctx, obj, JS_ATOM_prototype);
    let ret;
    if JS_VALUE_GET_TAG(obj_proto) != JS_TAG_OBJECT {
        if JS_IsException(obj_proto) == 0 {
            JS_ThrowTypeError(ctx, c"operand 'prototype' property is not an object".as_ptr());
        }
        ret = -1;
    } else {
        let proto = JS_VALUE_GET_OBJ(obj_proto).cast_const();
        p = JS_VALUE_GET_OBJ(val).cast_const();
        ret = loop {
            let proto1 = (*(*p).shape).proto.cast_const();
            if proto1.is_null() {
                if (*p).is_exotic() != 0 && (*p).fast_array() == 0 {
                    let mut obj1 = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast_mut().cast()));
                    break loop {
                        obj1 = JS_GetPrototypeFree(ctx, obj1);
                        if JS_IsException(obj1) != 0 { break -1; }
                        if JS_IsNull(obj1) != 0 { break 0; }
                        if proto == JS_VALUE_GET_OBJ(obj1).cast_const() {
                            JS_FreeValue(ctx, obj1); break 1;
                        }
                        if js_poll_interrupts(ctx) != 0 {
                            JS_FreeValue(ctx, obj1); break -1;
                        }
                    };
                }
                break 0;
            }
            p = proto1;
            if proto == p { break 1; }
        };
    }
    JS_FreeValue(ctx, obj_proto);
    ret
}
pub unsafe fn JS_IsInstanceOf(ctx: *mut JSContext, mut val: JSValueConst, obj: JSValueConst) -> i32 {
    if JS_IsObject(obj) != 0 {
        let method = JS_GetProperty(ctx, obj, crate::quickjs_atom::JS_ATOM_Symbol_hasInstance);
        if JS_IsException(method) != 0 { return -1; }
        if JS_IsNull(method) == 0 && JS_IsUndefined(method) == 0 {
            let ret = JS_CallFree(ctx, method, obj, 1, &mut val);
            return JS_ToBoolFree(ctx, ret);
        }
        if JS_IsFunction(ctx, obj) != 0 { return JS_OrdinaryIsInstanceOf(ctx, val, obj); }
    }
    JS_ThrowTypeError(ctx, c"invalid 'instanceof' right operand".as_ptr());
    -1
}
