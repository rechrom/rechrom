// quickjs.c:56152..56282. Exact ordering of the minimum intrinsic graph. MIT.
unsafe fn JS_AddIntrinsicBasicObjects(ctx: *mut JSContext) -> i32 {
    let object_proto = JS_NewObjectProtoClassAlloc(ctx, JS_NULL, JS_CLASS_OBJECT, 11 + 1);
    *(*ctx).class_proto.add(JS_CLASS_OBJECT as usize) = object_proto;
    if JS_IsException(object_proto) != 0 {
        return -1;
    }
    JS_SetImmutablePrototype(ctx, object_proto);
    (*ctx).function_proto = JS_NewCFunction3(
        ctx,
        Some(js_function_proto),
        c"".as_ptr(),
        0,
        JS_CFUNC_generic,
        0,
        object_proto,
        8 + 3 + 2,
    );
    if JS_IsException((*ctx).function_proto) != 0 {
        return -1;
    }
    *(*ctx).class_proto.add(JS_CLASS_BYTECODE_FUNCTION as usize) =
        JS_DupValue(ctx, (*ctx).function_proto);
    (*ctx).global_obj = JS_NewObjectProtoClassAlloc(ctx, object_proto, JS_CLASS_GLOBAL_OBJECT, 64);
    if JS_IsException((*ctx).global_obj) != 0 {
        return -1;
    }
    {
        let obj = JS_NewObjectProtoClassAlloc(ctx, JS_NULL, JS_CLASS_OBJECT, 4);
        let p = JS_VALUE_GET_PTR((*ctx).global_obj).cast::<JSObject>();
        (*p).u.global_object.uninitialized_vars = obj;
    }
    (*ctx).global_var_obj = JS_NewObjectProtoClassAlloc(ctx, JS_NULL, JS_CLASS_OBJECT, 16);
    if JS_IsException((*ctx).global_var_obj) != 0 {
        return -1;
    }
    let ft = JSCFunctionType {
        generic_magic: Some(js_error_constructor),
    };
    let obj = JS_NewCConstructor(
        ctx,
        JS_CLASS_ERROR as i32,
        c"Error".as_ptr(),
        ft.generic,
        1,
        JS_CFUNC_constructor_or_func_magic,
        -1,
        JS_UNDEFINED,
        ptr::addr_of!(js_error_funcs).cast(),
        1,
        ptr::addr_of!(js_error_proto_funcs).cast(),
        3,
        0,
    );
    if JS_IsException(obj) != 0 {
        return -1;
    }
    for i in 0..JS_NATIVE_ERROR_COUNT {
        let mut buf = [0 as c_char; ATOM_GET_STR_BUF_SIZE];
        let name = JS_AtomGetStr(
            ctx,
            buf.as_mut_ptr(),
            buf.len() as i32,
            crate::quickjs_atom::JS_ATOM_EvalError + i as u32,
        );
        let n_args = 1 + (i as i32 == JS_AGGREGATE_ERROR) as i32;
        let funcs = ptr::addr_of!(js_native_error_proto_funcs)
            .cast::<JSCFunctionListEntry>()
            .add(2 * i);
        let func_obj = JS_NewCConstructor(
            ctx,
            -1,
            name,
            ft.generic,
            n_args,
            JS_CFUNC_constructor_or_func_magic,
            i as i32,
            obj,
            ptr::null(),
            0,
            funcs,
            2,
            0,
        );
        if JS_IsException(func_obj) != 0 {
            JS_FreeValue(ctx, obj);
            return -1;
        }
        (*ctx).native_error_proto[i] =
            JS_GetProperty(ctx, func_obj, crate::quickjs_atom::JS_ATOM_prototype);
        JS_FreeValue(ctx, func_obj);
        if JS_IsException((*ctx).native_error_proto[i]) != 0 {
            JS_FreeValue(ctx, obj);
            return -1;
        }
    }
    JS_FreeValue(ctx, obj);
    let obj = JS_NewCConstructor(
        ctx,
        JS_CLASS_ARRAY as i32,
        c"Array".as_ptr(),
        Some(js_array_constructor),
        1,
        JS_CFUNC_constructor_or_func,
        0,
        JS_UNDEFINED,
        ptr::addr_of!(js_array_funcs).cast(),
        4,
        ptr::addr_of!(js_array_proto_funcs).cast(),
        40,
        JS_NEW_CTOR_PROTO_CLASS,
    );
    if JS_IsException(obj) != 0 {
        return -1;
    }
    (*ctx).array_ctor = obj;
    (*JS_VALUE_GET_PTR(*(*ctx).class_proto.add(JS_CLASS_ARRAY as usize)).cast::<JSObject>())
        .set_is_std_array_prototype(1);
    (*ctx).array_shape = js_new_shape2(
        ctx,
        get_proto_obj(*(*ctx).class_proto.add(JS_CLASS_ARRAY as usize)),
        JS_PROP_INITIAL_HASH_SIZE,
        1,
    );
    if (*ctx).array_shape.is_null() {
        return -1;
    }
    if add_shape_property(
        ctx,
        &mut (*ctx).array_shape,
        ptr::null_mut(),
        crate::quickjs_atom::JS_ATOM_length,
        JS_PROP_WRITABLE | JS_PROP_LENGTH,
    ) != 0
    {
        return -1;
    }
    (*ctx).arguments_shape = js_new_shape2(
        ctx,
        get_proto_obj(object_proto),
        JS_PROP_INITIAL_HASH_SIZE,
        3,
    );
    if (*ctx).arguments_shape.is_null() {
        return -1;
    }
    for (atom, flags) in [
        (
            crate::quickjs_atom::JS_ATOM_length,
            JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        ),
        (
            crate::quickjs_atom::JS_ATOM_Symbol_iterator,
            JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        ),
        (crate::quickjs_atom::JS_ATOM_callee, JS_PROP_GETSET),
    ] {
        if add_shape_property(
            ctx,
            &mut (*ctx).arguments_shape,
            ptr::null_mut(),
            atom,
            flags,
        ) != 0
        {
            return -1;
        }
    }
    (*ctx).mapped_arguments_shape = js_new_shape2(
        ctx,
        get_proto_obj(object_proto),
        JS_PROP_INITIAL_HASH_SIZE,
        3,
    );
    if (*ctx).mapped_arguments_shape.is_null() {
        return -1;
    }
    for atom in [
        crate::quickjs_atom::JS_ATOM_length,
        crate::quickjs_atom::JS_ATOM_Symbol_iterator,
        crate::quickjs_atom::JS_ATOM_callee,
    ] {
        if add_shape_property(
            ctx,
            &mut (*ctx).mapped_arguments_shape,
            ptr::null_mut(),
            atom,
            JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        ) != 0
        {
            return -1;
        }
    }
    0
}
