// quickjs.c:41286..41461. Error constructors, prototype and methods. MIT.
unsafe fn iterator_to_array(ctx: *mut JSContext, items: JSValueConst) -> JSValue {
    use crate::quickjs_atom::JS_ATOM_next;
    let iter = JS_GetIterator(ctx, items, 0);
    let mut next_method = JS_UNDEFINED;
    let mut r = JS_UNDEFINED;
    let mut fail = JS_IsException(iter) != 0;
    if !fail {
        next_method = JS_GetProperty(ctx, iter, JS_ATOM_next);
        fail = JS_IsException(next_method) != 0;
    }
    if !fail {
        r = JS_NewArray(ctx);
        fail = JS_IsException(r) != 0;
    }
    if !fail {
        let mut k: i64 = 0;
        loop {
            let mut done = 0;
            let v = JS_IteratorNext(ctx, iter, next_method, 0, ptr::null_mut(), &mut done);
            if JS_IsException(v) != 0 {
                JS_IteratorClose(ctx, iter, 1);
                fail = true;
                break;
            }
            if done != 0 {
                break;
            }
            if JS_DefinePropertyValueInt64(ctx, r, k, v, JS_PROP_C_W_E | JS_PROP_THROW) < 0 {
                JS_IteratorClose(ctx, iter, 1);
                fail = true;
                break;
            }
            k = k.wrapping_add(1);
        }
    }
    if fail {
        JS_FreeValue(ctx, r);
        r = JS_EXCEPTION;
    }
    JS_FreeValue(ctx, next_method);
    JS_FreeValue(ctx, iter);
    r
}
unsafe fn js_error_constructor(
    ctx: *mut JSContext,
    mut new_target: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
    magic: i32,
) -> JSValue {
    use crate::quickjs_atom::{JS_ATOM_cause, JS_ATOM_errors, JS_ATOM_message};
    if JS_IsUndefined(new_target) != 0 {
        new_target = JS_GetActiveFunction(ctx);
    }
    let mut proto = JS_GetProperty(ctx, new_target, crate::quickjs_atom::JS_ATOM_prototype);
    if JS_IsException(proto) != 0 {
        return proto;
    }
    if JS_IsObject(proto) == 0 {
        JS_FreeValue(ctx, proto);
        let realm = JS_GetFunctionRealm(ctx, new_target);
        if realm.is_null() {
            return JS_EXCEPTION;
        }
        let proto1 = if magic < 0 {
            *(*realm).class_proto.add(JS_CLASS_ERROR as usize)
        } else {
            (*realm).native_error_proto[magic as usize]
        };
        proto = JS_DupValue(ctx, proto1);
    }
    let obj = JS_NewObjectProtoClass(ctx, proto, JS_CLASS_ERROR);
    JS_FreeValue(ctx, proto);
    if JS_IsException(obj) != 0 {
        return obj;
    }
    let mut arg_index = (magic == JS_AGGREGATE_ERROR) as i32;
    let message = *argv.add(arg_index as usize);
    arg_index += 1;
    let mut fail = false;
    if JS_IsUndefined(message) == 0 {
        let msg = JS_ToString(ctx, message);
        fail = JS_IsException(msg) != 0;
        if !fail {
            JS_DefinePropertyValue(
                ctx,
                obj,
                JS_ATOM_message,
                msg,
                JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
            );
        }
    }
    if !fail && arg_index < argc {
        let options = *argv.add(arg_index as usize);
        if JS_IsObject(options) != 0 {
            let present = JS_HasProperty(ctx, options, JS_ATOM_cause);
            if present < 0 {
                fail = true;
            } else if present != 0 {
                let cause = JS_GetProperty(ctx, options, JS_ATOM_cause);
                fail = JS_IsException(cause) != 0;
                if !fail {
                    JS_DefinePropertyValue(
                        ctx,
                        obj,
                        JS_ATOM_cause,
                        cause,
                        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
                    );
                }
            }
        }
    }
    if !fail && magic == JS_AGGREGATE_ERROR {
        let errors = iterator_to_array(ctx, *argv);
        fail = JS_IsException(errors) != 0;
        if !fail {
            JS_DefinePropertyValue(
                ctx,
                obj,
                JS_ATOM_errors,
                errors,
                JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
            );
        }
    }
    if fail {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    build_backtrace(
        ctx,
        obj,
        ptr::null(),
        0,
        0,
        JS_BACKTRACE_FLAG_SKIP_FIRST_LEVEL,
    );
    obj
}
unsafe fn js_error_toString(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    use crate::quickjs_atom::{JS_ATOM_Error, JS_ATOM_message};
    if JS_IsObject(this_val) == 0 {
        return JS_ThrowTypeErrorNotAnObject(ctx);
    }
    let n = JS_GetProperty(ctx, this_val, crate::quickjs_atom::JS_ATOM_name);
    let mut name = if JS_IsUndefined(n) != 0 {
        JS_AtomToString(ctx, JS_ATOM_Error)
    } else {
        JS_ToStringFree(ctx, n)
    };
    if JS_IsException(name) != 0 {
        return JS_EXCEPTION;
    }
    let m = JS_GetProperty(ctx, this_val, JS_ATOM_message);
    let msg = if JS_IsUndefined(m) != 0 {
        JS_AtomToString(ctx, JS_ATOM_empty_string)
    } else {
        JS_ToStringFree(ctx, m)
    };
    if JS_IsException(msg) != 0 {
        JS_FreeValue(ctx, name);
        return JS_EXCEPTION;
    }
    if JS_IsEmptyString(name) == 0 && JS_IsEmptyString(msg) == 0 {
        name = JS_ConcatString3(ctx, c"".as_ptr(), name, c": ".as_ptr());
    }
    JS_ConcatString(ctx, name, msg)
}
unsafe fn js_error_isError(
    ctx: *mut JSContext,
    _this_val: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    JS_NewBool(ctx, JS_IsError(ctx, *argv))
}
unsafe fn js_aggregate_error_constructor(ctx: *mut JSContext, errors: JSValueConst) -> JSValue {
    let obj = JS_NewObjectProtoClass(
        ctx,
        (*ctx).native_error_proto[JS_AGGREGATE_ERROR as usize],
        JS_CLASS_ERROR,
    );
    if JS_IsException(obj) != 0 {
        return obj;
    }
    JS_DefinePropertyValue(
        ctx,
        obj,
        crate::quickjs_atom::JS_ATOM_errors,
        JS_DupValue(ctx, errors),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    );
    obj
}
static mut js_error_proto_funcs: [JSCFunctionListEntry; 3] = [
    JS_CFUNC_DEF(c"toString".as_ptr(), 0, Some(js_error_toString)),
    JS_PROP_STRING_DEF(
        c"name".as_ptr(),
        c"Error".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
];
static mut js_error_funcs: [JSCFunctionListEntry; 1] =
    [JS_CFUNC_DEF(c"isError".as_ptr(), 1, Some(js_error_isError))];
static mut js_native_error_proto_funcs: [JSCFunctionListEntry; 16] = [
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_EvalError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_RangeError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_ReferenceError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_SyntaxError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_TypeError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_URIError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_InternalError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_ATOM_DEF(
        c"name".as_ptr(),
        crate::quickjs_atom::JS_ATOM_AggregateError as i32,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
    JS_PROP_STRING_DEF(
        c"message".as_ptr(),
        c"".as_ptr(),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ),
];
