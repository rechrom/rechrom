// quickjs.c:7887-8074. MIT. Staged with original shape/error dependencies.
unsafe fn JS_SetImmutablePrototype(_ctx: *mut JSContext, obj: JSValueConst) {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return;
    }
    (*JS_VALUE_GET_PTR(obj).cast::<JSObject>()).set_has_immutable_prototype(1);
}
unsafe fn JS_SetPrototypeInternal(
    ctx: *mut JSContext,
    obj: JSValueConst,
    proto_val: JSValueConst,
    throw_flag: JS_BOOL,
) -> i32 {
    if (throw_flag != 0 && matches!(JS_VALUE_GET_TAG(obj), JS_TAG_NULL | JS_TAG_UNDEFINED))
        || (throw_flag == 0 && JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT)
    {
        JS_ThrowTypeErrorNotAnObject(ctx);
        return -1;
    }
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    let proto = if JS_VALUE_GET_TAG(proto_val) == JS_TAG_OBJECT {
        JS_VALUE_GET_PTR(proto_val).cast::<JSObject>()
    } else if JS_VALUE_GET_TAG(proto_val) == JS_TAG_NULL {
        ptr::null_mut()
    } else {
        JS_ThrowTypeErrorNotAnObject(ctx);
        return -1;
    };
    if throw_flag != 0 && JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return 1;
    }
    if (*p).is_exotic() != 0 {
        let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
        if !em.is_null() {
            if let Some(set_prototype) = (*em).set_prototype {
                let ret = set_prototype(ctx, obj, proto_val);
                if ret == 0 && throw_flag != 0 {
                    JS_ThrowTypeError(ctx, c"proxy: bad prototype".as_ptr());
                    return -1;
                }
                return ret;
            }
        }
    }
    let mut sh = (*p).shape;
    if (*sh).proto == proto {
        return 1;
    }
    if (*p).has_immutable_prototype() != 0 {
        if throw_flag != 0 {
            JS_ThrowTypeError(ctx, c"prototype is immutable".as_ptr());
            return -1;
        }
        return 0;
    }
    if (*p).extensible() == 0 {
        if throw_flag != 0 {
            JS_ThrowTypeError(ctx, c"object is not extensible".as_ptr());
            return -1;
        }
        return 0;
    }
    if !proto.is_null() {
        let mut p1 = proto;
        loop {
            if p1 == p {
                if throw_flag != 0 {
                    JS_ThrowTypeError(ctx, c"circular prototype chain".as_ptr());
                    return -1;
                }
                return 0;
            }
            p1 = (*(*p1).shape).proto;
            if p1.is_null() {
                break;
            }
        }
        JS_DupValue(ctx, proto_val);
    }
    if js_shape_prepare_update(ctx, p, ptr::null_mut()) != 0 {
        return -1;
    }
    sh = (*p).shape;
    if !(*sh).proto.is_null() {
        JS_FreeValue(ctx, JS_MKPTR(JS_TAG_OBJECT, (*sh).proto.cast()));
    }
    (*sh).proto = proto;
    (*p).set_is_std_array_prototype(0);
    1
}
pub unsafe fn JS_SetPrototype(
    ctx: *mut JSContext,
    obj: JSValueConst,
    proto_val: JSValueConst,
) -> i32 {
    JS_SetPrototypeInternal(ctx, obj, proto_val, 1)
}
unsafe fn JS_GetPrototypePrimitive(ctx: *mut JSContext, val: JSValueConst) -> JSValueConst {
    let class = match JS_VALUE_GET_NORM_TAG(val) {
        JS_TAG_SHORT_BIG_INT | JS_TAG_BIG_INT => JS_CLASS_BIG_INT,
        JS_TAG_INT | JS_TAG_FLOAT64 => JS_CLASS_NUMBER,
        JS_TAG_BOOL => JS_CLASS_BOOLEAN,
        JS_TAG_STRING | JS_TAG_STRING_ROPE => JS_CLASS_STRING,
        JS_TAG_SYMBOL => JS_CLASS_SYMBOL,
        _ => return JS_NULL,
    };
    *(*ctx).class_proto.add(class as usize)
}
pub unsafe fn JS_GetPrototype(ctx: *mut JSContext, obj: JSValueConst) -> JSValue {
    if JS_VALUE_GET_TAG(obj) == JS_TAG_OBJECT {
        let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        if (*p).is_exotic() != 0 {
            let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
            if !em.is_null() {
                if let Some(get_prototype) = (*em).get_prototype {
                    return get_prototype(ctx, obj);
                }
            }
        }
        let proto = (*(*p).shape).proto;
        if proto.is_null() {
            JS_NULL
        } else {
            JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, proto.cast()))
        }
    } else {
        JS_DupValue(ctx, JS_GetPrototypePrimitive(ctx, obj))
    }
}
unsafe fn JS_GetPrototypeFree(ctx: *mut JSContext, obj: JSValue) -> JSValue {
    let obj1 = JS_GetPrototype(ctx, obj);
    JS_FreeValue(ctx, obj);
    obj1
}
