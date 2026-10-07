// quickjs.c public/internal call wrappers and constructor realm selection. MIT.
// c: quickjs.c:20600
pub unsafe fn JS_Call(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    this_obj: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    JS_CallInternal(
        ctx,
        func_obj,
        this_obj,
        JS_UNDEFINED,
        argc,
        argv,
        JS_CALL_FLAG_COPY_ARGV,
    )
}
// c: quickjs.c:20607
unsafe fn JS_CallFree(
    ctx: *mut JSContext,
    func_obj: JSValue,
    this_obj: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let res = JS_CallInternal(
        ctx,
        func_obj,
        this_obj,
        JS_UNDEFINED,
        argc,
        argv,
        JS_CALL_FLAG_COPY_ARGV,
    );
    JS_FreeValue(ctx, func_obj);
    res
}
// c: quickjs.c:20618
unsafe fn JS_GetFunctionRealm(ctx: *mut JSContext, func_obj: JSValueConst) -> *mut JSContext {
    if JS_VALUE_GET_TAG(func_obj) != JS_TAG_OBJECT {
        return ctx;
    }
    let p = JS_VALUE_GET_OBJ(func_obj);
    match (*p).class_id as u32 {
        JS_CLASS_C_FUNCTION => (*p).u.cfunc.realm,
        JS_CLASS_BYTECODE_FUNCTION
        | JS_CLASS_GENERATOR_FUNCTION
        | JS_CLASS_ASYNC_FUNCTION
        | JS_CLASS_ASYNC_GENERATOR_FUNCTION => (*(*p).u.func.function_bytecode).realm,
        JS_CLASS_PROXY => {
            let s = (*p).u.opaque.cast::<JSProxyData>();
            if s.is_null() {
                ctx
            } else if (*s).is_revoked != 0 {
                JS_ThrowTypeErrorRevokedProxy(ctx);
                ptr::null_mut()
            } else {
                JS_GetFunctionRealm(ctx, (*s).target)
            }
        }
        JS_CLASS_BOUND_FUNCTION => JS_GetFunctionRealm(ctx, (*(*p).u.bound_function).func_obj),
        _ => ctx,
    }
}
// c: quickjs.c:20666
unsafe fn js_create_from_ctor(ctx: *mut JSContext, ctor: JSValueConst, class_id: i32) -> JSValue {
    let proto = if JS_IsUndefined(ctor) != 0 {
        JS_DupValue(ctx, *(*ctx).class_proto.add(class_id as usize))
    } else {
        let mut proto = JS_GetProperty(ctx, ctor, JS_ATOM_prototype);
        if JS_IsException(proto) != 0 {
            return proto;
        }
        if JS_IsObject(proto) == 0 {
            JS_FreeValue(ctx, proto);
            let realm = JS_GetFunctionRealm(ctx, ctor);
            if realm.is_null() {
                return JS_EXCEPTION;
            }
            proto = JS_DupValue(ctx, *(*realm).class_proto.add(class_id as usize));
        }
        proto
    };
    let obj = JS_NewObjectProtoClass(ctx, proto, class_id as u32);
    JS_FreeValue(ctx, proto);
    obj
}
// c: quickjs.c:20692
unsafe fn JS_CallConstructorInternal(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    new_target: JSValueConst,
    argc: i32,
    argv: *mut JSValue,
    flags: i32,
) -> JSValue {
    if js_poll_interrupts(ctx) != 0 {
        return JS_EXCEPTION;
    }
    let flags = flags | JS_CALL_FLAG_CONSTRUCTOR;
    if JS_VALUE_GET_TAG(func_obj) != JS_TAG_OBJECT {
        return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
    }
    let p = JS_VALUE_GET_OBJ(func_obj);
    if (*p).is_constructor() == 0 {
        return JS_ThrowTypeErrorNotAConstructor(ctx, func_obj);
    }
    if (*p).class_id as u32 != JS_CLASS_BYTECODE_FUNCTION {
        return match (*(*ctx).rt)
            .class_array
            .add((*p).class_id as usize)
            .as_ref()
            .unwrap()
            .call
        {
            Some(call_func) => call_func(ctx, func_obj, new_target, argc, argv, flags),
            None => JS_ThrowTypeError(ctx, c"not a function".as_ptr()),
        };
    }
    let b = (*p).u.func.function_bytecode;
    if (*b).is_derived_class_constructor() != 0 {
        JS_CallInternal(ctx, func_obj, JS_UNDEFINED, new_target, argc, argv, flags)
    } else {
        let obj = js_create_from_ctor(ctx, new_target, JS_CLASS_OBJECT as i32);
        if JS_IsException(obj) != 0 {
            return JS_EXCEPTION;
        }
        let ret = JS_CallInternal(ctx, func_obj, obj, new_target, argc, argv, flags);
        if JS_VALUE_GET_TAG(ret) == JS_TAG_OBJECT || JS_IsException(ret) != 0 {
            JS_FreeValue(ctx, obj);
            ret
        } else {
            JS_FreeValue(ctx, ret);
            obj
        }
    }
}
// c: quickjs.c:20740
pub unsafe fn JS_CallConstructor2(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    new_target: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    JS_CallConstructorInternal(
        ctx,
        func_obj,
        new_target,
        argc,
        argv,
        JS_CALL_FLAG_COPY_ARGV,
    )
}
// c: quickjs.c:20749
pub unsafe fn JS_CallConstructor(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    JS_CallConstructorInternal(ctx, func_obj, func_obj, argc, argv, JS_CALL_FLAG_COPY_ARGV)
}
// c: quickjs.c:20757
pub unsafe fn JS_Invoke(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    atom: JSAtom,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let func_obj = JS_GetProperty(ctx, this_val, atom);
    if JS_IsException(func_obj) != 0 {
        return func_obj;
    }
    JS_CallFree(ctx, func_obj, this_val, argc, argv)
}
// c: quickjs.c:20767
unsafe fn JS_InvokeFree(
    ctx: *mut JSContext,
    this_val: JSValue,
    atom: JSAtom,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let res = JS_Invoke(ctx, this_val, atom, argc, argv);
    JS_FreeValue(ctx, this_val);
    res
}
