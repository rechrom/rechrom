unsafe fn js_function_proto(
    _ctx: *mut JSContext,
    _this_val: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    JS_UNDEFINED
}
// quickjs.c:16107..16140,40938..41282. Function constructor/prototype. MIT.
unsafe fn js_throw_type_error(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    let b = JS_GetFunctionBytecode(this_val);
    if b.is_null()
        || (*b).js_mode & JS_MODE_STRICT as u8 != 0
        || (*b).has_prototype() == 0
        || argc >= 1
    {
        return JS_ThrowTypeError(ctx, c"invalid property access".as_ptr());
    }
    JS_UNDEFINED
}
unsafe fn js_function_proto_fileName(ctx: *mut JSContext, this_val: JSValueConst) -> JSValue {
    let b = JS_GetFunctionBytecode(this_val);
    if !b.is_null() && (*b).has_debug() != 0 {
        return JS_AtomToString(ctx, (*b).debug.filename);
    }
    JS_UNDEFINED
}
unsafe fn js_function_proto_lineNumber(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    is_col: i32,
) -> JSValue {
    let b = JS_GetFunctionBytecode(this_val);
    if !b.is_null() && (*b).has_debug() != 0 {
        let mut col = 0;
        let line = find_line_num(ctx, b, u32::MAX, &mut col);
        return JS_NewInt32(ctx, if is_col != 0 { col } else { line });
    }
    JS_UNDEFINED
}
unsafe fn js_function_constructor(
    ctx: *mut JSContext,
    new_target: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
    magic: i32,
) -> JSValue {
    let func_kind = magic as JSFunctionKindEnum;
    let mut b: StringBuffer = core::mem::zeroed();
    string_buffer_init(ctx, &mut b, 0);
    string_buffer_putc8(&mut b, b'(' as u32);
    if func_kind == JS_FUNC_ASYNC || func_kind == JS_FUNC_ASYNC_GENERATOR {
        string_buffer_puts8(&mut b, c"async ".as_ptr());
    }
    string_buffer_puts8(&mut b, c"function".as_ptr());
    if func_kind == JS_FUNC_GENERATOR || func_kind == JS_FUNC_ASYNC_GENERATOR {
        string_buffer_putc8(&mut b, b'*' as u32);
    }
    string_buffer_puts8(&mut b, c" anonymous(".as_ptr());
    let n = argc - 1;
    for i in 0..n {
        if i != 0 {
            string_buffer_putc8(&mut b, b',' as u32);
        }
        if string_buffer_concat_value(&mut b, *argv.add(i as usize)) != 0 {
            string_buffer_free(&mut b);
            return JS_EXCEPTION;
        }
    }
    string_buffer_puts8(&mut b, c"\n) {\n".as_ptr());
    if n >= 0 && string_buffer_concat_value(&mut b, *argv.add(n as usize)) != 0 {
        string_buffer_free(&mut b);
        return JS_EXCEPTION;
    }
    string_buffer_puts8(&mut b, c"\n})".as_ptr());
    let s = string_buffer_end(&mut b);
    if JS_IsException(s) != 0 {
        return JS_EXCEPTION;
    }
    let obj = JS_EvalObject(ctx, (*ctx).global_obj, s, JS_EVAL_TYPE_INDIRECT, -1);
    JS_FreeValue(ctx, s);
    if JS_IsException(obj) != 0 {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    if JS_IsUndefined(new_target) == 0 {
        let mut proto = JS_GetProperty(ctx, new_target, crate::quickjs_atom::JS_ATOM_prototype);
        if JS_IsException(proto) != 0 {
            JS_FreeValue(ctx, obj);
            return JS_EXCEPTION;
        }
        if JS_IsObject(proto) == 0 {
            JS_FreeValue(ctx, proto);
            let realm = JS_GetFunctionRealm(ctx, new_target);
            if realm.is_null() {
                JS_FreeValue(ctx, obj);
                return JS_EXCEPTION;
            }
            proto = JS_DupValue(
                ctx,
                *(*realm)
                    .class_proto
                    .add(func_kind_to_class_id[func_kind as usize] as usize),
            );
        }
        let ret = JS_SetPrototypeInternal(ctx, obj, proto, 1);
        JS_FreeValue(ctx, proto);
        if ret < 0 {
            JS_FreeValue(ctx, obj);
            return JS_EXCEPTION;
        }
    }
    obj
}
unsafe fn js_get_length32(ctx: *mut JSContext, pres: *mut u32, obj: JSValueConst) -> i32 {
    let len_val = JS_GetProperty(ctx, obj, crate::quickjs_atom::JS_ATOM_length);
    if JS_IsException(len_val) != 0 {
        *pres = 0;
        return -1;
    }
    JS_ToUint32Free(ctx, pres, len_val)
}
unsafe fn js_get_length64(ctx: *mut JSContext, pres: *mut i64, obj: JSValueConst) -> i32 {
    let len_val = JS_GetProperty(ctx, obj, crate::quickjs_atom::JS_ATOM_length);
    if JS_IsException(len_val) != 0 {
        *pres = 0;
        return -1;
    }
    JS_ToLengthFree(ctx, pres, len_val)
}
unsafe fn free_arg_list(ctx: *mut JSContext, tab: *mut JSValue, len: u32) {
    for i in 0..len {
        JS_FreeValue(ctx, *tab.add(i as usize));
    }
    js_free(ctx, tab.cast());
}
unsafe fn build_arg_list(
    ctx: *mut JSContext,
    plen: *mut u32,
    array_arg: JSValueConst,
) -> *mut JSValue {
    if JS_VALUE_GET_TAG(array_arg) != JS_TAG_OBJECT {
        JS_ThrowTypeError(ctx, c"not a object".as_ptr());
        return ptr::null_mut();
    }
    let mut len64 = 0;
    if js_get_length64(ctx, &mut len64, array_arg) != 0 {
        return ptr::null_mut();
    }
    if len64 > JS_MAX_LOCAL_VARS as i64 {
        JS_ThrowRangeError(
            ctx,
            format_args!(
                "too many arguments in function call (only {} allowed)",
                JS_MAX_LOCAL_VARS
            ),
        );
        return ptr::null_mut();
    }
    let len = len64 as u32;
    let tab = js_mallocz(ctx, size_of::<JSValue>() * len.max(1) as usize).cast::<JSValue>();
    if tab.is_null() {
        return ptr::null_mut();
    }
    let p = JS_VALUE_GET_PTR(array_arg).cast::<JSObject>();
    if matches!(
        (*p).class_id as u32,
        JS_CLASS_ARRAY | JS_CLASS_ARGUMENTS | JS_CLASS_MAPPED_ARGUMENTS
    ) && (*p).fast_array() != 0
        && len == (*p).u.array.count
    {
        if (*p).class_id as u32 == JS_CLASS_MAPPED_ARGUMENTS {
            for i in 0..len {
                *tab.add(i as usize) =
                    JS_DupValue(ctx, *(**(*p).u.array.u.var_refs.add(i as usize)).pvalue);
            }
        } else {
            for i in 0..len {
                *tab.add(i as usize) = JS_DupValue(ctx, *(*p).u.array.u.values.add(i as usize));
            }
        }
    } else {
        for i in 0..len {
            let ret = JS_GetPropertyUint32(ctx, array_arg, i);
            if JS_IsException(ret) != 0 {
                free_arg_list(ctx, tab, i);
                return ptr::null_mut();
            }
            *tab.add(i as usize) = ret;
        }
    }
    *plen = len;
    tab
}
unsafe fn js_function_apply(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
    magic: i32,
) -> JSValue {
    if check_function(ctx, this_val) != 0 {
        return JS_EXCEPTION;
    }
    let this_arg = *argv;
    let array_arg = *argv.add(1);
    if matches!(JS_VALUE_GET_TAG(array_arg), JS_TAG_UNDEFINED | JS_TAG_NULL) && magic != 2 {
        return JS_Call(ctx, this_val, this_arg, 0, ptr::null_mut());
    }
    let mut len = 0;
    let tab = build_arg_list(ctx, &mut len, array_arg);
    if tab.is_null() {
        return JS_EXCEPTION;
    }
    let ret = if magic & 1 != 0 {
        JS_CallConstructor2(ctx, this_val, this_arg, len as i32, tab)
    } else {
        JS_Call(ctx, this_val, this_arg, len as i32, tab)
    };
    free_arg_list(ctx, tab, len);
    ret
}
unsafe fn js_function_call(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    if argc <= 0 {
        JS_Call(ctx, this_val, JS_UNDEFINED, 0, ptr::null_mut())
    } else {
        JS_Call(ctx, this_val, *argv, argc - 1, argv.add(1))
    }
}
unsafe fn js_function_bind(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    if check_function(ctx, this_val) != 0 {
        return JS_EXCEPTION;
    }
    let func_obj = JS_NewObjectProtoClass(ctx, (*ctx).function_proto, JS_CLASS_BOUND_FUNCTION);
    if JS_IsException(func_obj) != 0 {
        return JS_EXCEPTION;
    }
    let p = JS_VALUE_GET_PTR(func_obj).cast::<JSObject>();
    (*p).set_is_constructor(JS_IsConstructor(ctx, this_val) as u8);
    let arg_count = (argc - 1).max(0);
    let bf = js_malloc(
        ctx,
        size_of::<JSBoundFunction>() + arg_count as usize * size_of::<JSValue>(),
    )
    .cast::<JSBoundFunction>();
    if bf.is_null() {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    (*bf).func_obj = JS_DupValue(ctx, this_val);
    (*bf).this_val = JS_DupValue(ctx, *argv);
    (*bf).argc = arg_count;
    for i in 0..arg_count {
        *ptr::addr_of_mut!((*bf).argv)
            .cast::<JSValue>()
            .add(i as usize) = JS_DupValue(ctx, *argv.add(i as usize + 1));
    }
    (*p).u.bound_function = bf;
    let ret = JS_GetOwnProperty(
        ctx,
        ptr::null_mut(),
        this_val,
        crate::quickjs_atom::JS_ATOM_length,
    );
    if ret < 0 {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    let len_val = if ret == 0 {
        JS_NewInt32(ctx, 0)
    } else {
        let v = JS_GetProperty(ctx, this_val, crate::quickjs_atom::JS_ATOM_length);
        if JS_IsException(v) != 0 {
            JS_FreeValue(ctx, func_obj);
            return JS_EXCEPTION;
        }
        if JS_VALUE_GET_TAG(v) == JS_TAG_INT {
            let len1 = JS_VALUE_GET_INT(v);
            JS_NewInt32(
                ctx,
                if len1 <= arg_count {
                    0
                } else {
                    len1 - arg_count
                },
            )
        } else if JS_VALUE_GET_NORM_TAG(v) == JS_TAG_FLOAT64 {
            let mut d = JS_VALUE_GET_FLOAT64(v);
            if d.is_nan() {
                d = 0.0;
            } else {
                d = d.trunc();
                if d <= arg_count as f64 {
                    d = 0.0;
                } else {
                    d -= arg_count as f64;
                }
            }
            JS_NewFloat64(ctx, d)
        } else {
            JS_FreeValue(ctx, v);
            JS_NewInt32(ctx, 0)
        }
    };
    JS_DefinePropertyValue(
        ctx,
        func_obj,
        crate::quickjs_atom::JS_ATOM_length,
        len_val,
        JS_PROP_CONFIGURABLE,
    );
    let mut name1 = JS_GetProperty(ctx, this_val, crate::quickjs_atom::JS_ATOM_name);
    if JS_IsException(name1) != 0 {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    if JS_IsString(name1) == 0 {
        JS_FreeValue(ctx, name1);
        name1 = JS_AtomToString(ctx, JS_ATOM_empty_string);
    }
    name1 = JS_ConcatString3(ctx, c"bound ".as_ptr(), name1, c"".as_ptr());
    if JS_IsException(name1) != 0 {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    JS_DefinePropertyValue(
        ctx,
        func_obj,
        crate::quickjs_atom::JS_ATOM_name,
        name1,
        JS_PROP_CONFIGURABLE,
    );
    func_obj
}
unsafe fn js_function_toString(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    if check_function(ctx, this_val) != 0 {
        return JS_EXCEPTION;
    }
    let p = JS_VALUE_GET_PTR(this_val).cast::<JSObject>();
    let mut func_kind = JS_FUNC_NORMAL;
    if js_class_has_bytecode((*p).class_id as u32) != 0 {
        let b = (*p).u.func.function_bytecode;
        if (*b).has_debug() != 0 && !(*b).debug.source.is_null() {
            return JS_NewStringLen(ctx, (*b).debug.source, (*b).debug.source_len as usize);
        }
        func_kind = (*b).func_kind() as u32;
    }
    let pref = match func_kind {
        JS_FUNC_GENERATOR => c"function *".as_ptr(),
        JS_FUNC_ASYNC => c"async function ".as_ptr(),
        JS_FUNC_ASYNC_GENERATOR => c"async function *".as_ptr(),
        _ => c"function ".as_ptr(),
    };
    let mut name = JS_GetProperty(ctx, this_val, crate::quickjs_atom::JS_ATOM_name);
    if JS_IsUndefined(name) != 0 {
        name = JS_AtomToString(ctx, JS_ATOM_empty_string);
    }
    JS_ConcatString3(ctx, pref, name, c"() {\n    [native code]\n}".as_ptr())
}
unsafe fn js_function_hasInstance(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let ret = JS_OrdinaryIsInstanceOf(ctx, *argv, this_val);
    if ret < 0 {
        JS_EXCEPTION
    } else {
        JS_NewBool(ctx, ret)
    }
}
static mut js_function_proto_funcs: [JSCFunctionListEntry; 8] = [
    JS_CFUNC_DEF(c"call".as_ptr(), 1, Some(js_function_call)),
    JS_CFUNC_MAGIC_DEF(c"apply".as_ptr(), 2, Some(js_function_apply), 0),
    JS_CFUNC_DEF(c"bind".as_ptr(), 1, Some(js_function_bind)),
    JS_CFUNC_DEF(c"toString".as_ptr(), 0, Some(js_function_toString)),
    JS_CFUNC_DEF(
        c"[Symbol.hasInstance]".as_ptr(),
        1,
        Some(js_function_hasInstance),
    ),
    JS_CGETSET_DEF(c"fileName".as_ptr(), Some(js_function_proto_fileName), None),
    JS_CGETSET_MAGIC_DEF(
        c"lineNumber".as_ptr(),
        Some(js_function_proto_lineNumber),
        None,
        0,
    ),
    JS_CGETSET_MAGIC_DEF(
        c"columnNumber".as_ptr(),
        Some(js_function_proto_lineNumber),
        None,
        1,
    ),
];
