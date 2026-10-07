// quickjs.c:11063-11140,39790-39845. MIT. Staged with original call path.
const HINT_STRING: i32 = 0;
const HINT_NUMBER: i32 = 1;
const HINT_NONE: i32 = 2;
const HINT_FORCE_ORDINARY: i32 = 1 << 4;
unsafe fn JS_ToPrimitiveFree(ctx: *mut JSContext, val: JSValue, mut hint: i32) -> JSValue {
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT {
        return val;
    }
    let force_ordinary = hint & HINT_FORCE_ORDINARY;
    hint &= !HINT_FORCE_ORDINARY;
    if force_ordinary == 0 {
        let method = JS_GetProperty(ctx, val, crate::quickjs_atom::JS_ATOM_Symbol_toPrimitive);
        if JS_IsException(method) != 0 {
            JS_FreeValue(ctx, val);
            return JS_EXCEPTION;
        }
        if JS_IsUndefined(method) == 0 && JS_IsNull(method) == 0 {
            let atom = match hint {
                HINT_STRING => crate::quickjs_atom::JS_ATOM_string,
                HINT_NUMBER => crate::quickjs_atom::JS_ATOM_number,
                _ => crate::quickjs_atom::JS_ATOM_default,
            };
            let mut arg = JS_AtomToString(ctx, atom);
            let ret = JS_CallFree(ctx, method, val, 1, &mut arg);
            JS_FreeValue(ctx, arg);
            if JS_IsException(ret) != 0 {
                JS_FreeValue(ctx, val);
                return JS_EXCEPTION;
            }
            JS_FreeValue(ctx, val);
            if JS_VALUE_GET_TAG(ret) != JS_TAG_OBJECT {
                return ret;
            }
            JS_FreeValue(ctx, ret);
            return JS_ThrowTypeError(ctx, c"toPrimitive".as_ptr());
        }
    }
    if hint != HINT_STRING {
        hint = HINT_NUMBER;
    }
    for i in 0..2 {
        let name = if i ^ hint == 0 {
            crate::quickjs_atom::JS_ATOM_toString
        } else {
            crate::quickjs_atom::JS_ATOM_valueOf
        };
        let method = JS_GetProperty(ctx, val, name);
        if JS_IsException(method) != 0 {
            JS_FreeValue(ctx, val);
            return JS_EXCEPTION;
        }
        if JS_IsFunction(ctx, method) != 0 {
            let ret = JS_CallFree(ctx, method, val, 0, ptr::null_mut());
            if JS_IsException(ret) != 0 {
                JS_FreeValue(ctx, val);
                return JS_EXCEPTION;
            }
            if JS_VALUE_GET_TAG(ret) != JS_TAG_OBJECT {
                JS_FreeValue(ctx, val);
                return ret;
            }
            JS_FreeValue(ctx, ret);
        } else {
            JS_FreeValue(ctx, method);
        }
    }
    JS_ThrowTypeError(ctx, c"toPrimitive".as_ptr());
    JS_FreeValue(ctx, val);
    JS_EXCEPTION
}
unsafe fn JS_ToPrimitive(ctx: *mut JSContext, val: JSValueConst, hint: i32) -> JSValue {
    JS_ToPrimitiveFree(ctx, JS_DupValue(ctx, val), hint)
}
unsafe fn JS_ToObject(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    let class = match JS_VALUE_GET_NORM_TAG(val) {
        JS_TAG_OBJECT | JS_TAG_EXCEPTION => return JS_DupValue(ctx, val),
        JS_TAG_SHORT_BIG_INT | JS_TAG_BIG_INT => JS_CLASS_BIG_INT,
        JS_TAG_INT | JS_TAG_FLOAT64 => JS_CLASS_NUMBER,
        JS_TAG_STRING | JS_TAG_STRING_ROPE => {
            let str = JS_ToString(ctx, val);
            if JS_IsException(str) != 0 {
                return JS_EXCEPTION;
            }
            let obj = JS_NewObjectClass(ctx, JS_CLASS_STRING as i32);
            if JS_IsException(obj) == 0 {
                JS_DefinePropertyValue(
                    ctx,
                    obj,
                    crate::quickjs_atom::JS_ATOM_length,
                    JS_NewInt32(
                        ctx,
                        (*JS_VALUE_GET_PTR(str).cast::<JSString>()).len() as i32,
                    ),
                    0,
                );
                JS_SetObjectData(ctx, obj, JS_DupValue(ctx, str));
            }
            JS_FreeValue(ctx, str);
            return obj;
        }
        JS_TAG_BOOL => JS_CLASS_BOOLEAN,
        JS_TAG_SYMBOL => JS_CLASS_SYMBOL,
        _ => return JS_ThrowTypeError(ctx, c"cannot convert to object".as_ptr()),
    };
    let obj = JS_NewObjectClass(ctx, class as i32);
    if JS_IsException(obj) == 0 {
        JS_SetObjectData(ctx, obj, JS_DupValue(ctx, val));
    }
    obj
}
unsafe fn JS_ToObjectFree(ctx: *mut JSContext, val: JSValue) -> JSValue {
    let obj = JS_ToObject(ctx, val);
    JS_FreeValue(ctx, val);
    obj
}
