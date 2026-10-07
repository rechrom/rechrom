// quickjs.c native call dispatch. Bellard/Gordon, MIT.
const JS_CALL_FLAG_COPY_ARGV: i32 = 1 << 1;
const JS_CALL_FLAG_GENERATOR: i32 = 1 << 2;
// c: quickjs.c:16782
unsafe fn js_create_iterator_result(ctx: *mut JSContext, val: JSValue, done: JS_BOOL) -> JSValue {
    let obj = JS_NewObject(ctx);
    if JS_IsException(obj) != 0 {
        JS_FreeValue(ctx, val);
        return obj;
    }
    if JS_DefinePropertyValue(ctx, obj, JS_ATOM_value, val, JS_PROP_C_W_E) < 0
        || JS_DefinePropertyValue(ctx, obj, JS_ATOM_done, JS_NewBool(ctx, done), JS_PROP_C_W_E) < 0
    {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    obj
}
// c: quickjs.c:17588
unsafe fn js_call_c_function(
    mut ctx: *mut JSContext,
    func_obj: JSValueConst,
    mut this_obj: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
    flags: i32,
) -> JSValue {
    let rt = (*ctx).rt;
    let p = JS_VALUE_GET_OBJ(func_obj);
    let cproto = (*p).u.cfunc.cproto as JSCFunctionEnum;
    let arg_count = (*p).u.cfunc.length as i32;
    if js_check_stack_overflow(rt, core::mem::size_of::<JSValue>() * arg_count as usize) != 0 {
        return JS_ThrowStackOverflow(ctx);
    }
    let mut sf: JSStackFrame = core::mem::zeroed();
    sf.prev_frame = (*rt).current_stack_frame;
    (*rt).current_stack_frame = &mut sf;
    ctx = (*p).u.cfunc.realm;
    sf.js_mode = 0;
    sf.cur_func = func_obj;
    sf.arg_count = argc;
    let mut arg_buf = argv;
    // The C length field is uint8_t. A fixed local buffer replaces its
    // bounded alloca without a host allocator call or observable OOM.
    let mut padded = [JS_UNDEFINED; 255];
    if argc < arg_count {
        for i in 0..argc {
            padded[i as usize] = *argv.add(i as usize);
        }
        arg_buf = padded.as_mut_ptr();
        sf.arg_count = arg_count;
    }
    sf.arg_buf = arg_buf;
    let func = (*p).u.cfunc.c_function;
    let magic = (*p).u.cfunc.magic as i32;
    let constructor_only = cproto == JS_CFUNC_constructor || cproto == JS_CFUNC_constructor_magic;
    let ctor_or_func =
        cproto == JS_CFUNC_constructor_or_func || cproto == JS_CFUNC_constructor_or_func_magic;
    let ret_val = if constructor_only && flags & JS_CALL_FLAG_CONSTRUCTOR == 0 {
        JS_ThrowTypeError(ctx, c"must be called with new".as_ptr())
    } else {
        if ctor_or_func && flags & JS_CALL_FLAG_CONSTRUCTOR == 0 {
            this_obj = JS_UNDEFINED;
        }
        match cproto {
            JS_CFUNC_constructor | JS_CFUNC_constructor_or_func | JS_CFUNC_generic => {
                func.generic.expect("registered native function")(ctx, this_obj, argc, arg_buf)
            }
            JS_CFUNC_constructor_magic
            | JS_CFUNC_constructor_or_func_magic
            | JS_CFUNC_generic_magic => func
                .generic_magic
                .expect("registered native magic function")(
                ctx, this_obj, argc, arg_buf, magic
            ),
            JS_CFUNC_getter => func.getter.expect("registered getter")(ctx, this_obj),
            JS_CFUNC_setter => func.setter.expect("registered setter")(ctx, this_obj, *arg_buf),
            JS_CFUNC_getter_magic => {
                func.getter_magic.expect("registered getter")(ctx, this_obj, magic)
            }
            JS_CFUNC_setter_magic => {
                func.setter_magic.expect("registered setter")(ctx, this_obj, *arg_buf, magic)
            }
            JS_CFUNC_f_f => {
                let mut d1 = 0.0;
                if JS_ToFloat64(ctx, &mut d1, *arg_buf) != 0 {
                    JS_EXCEPTION
                } else {
                    JS_NewFloat64(ctx, func.f_f.expect("registered f_f")(d1))
                }
            }
            JS_CFUNC_f_f_f => {
                let (mut d1, mut d2) = (0.0, 0.0);
                if JS_ToFloat64(ctx, &mut d1, *arg_buf) != 0
                    || JS_ToFloat64(ctx, &mut d2, *arg_buf.add(1)) != 0
                {
                    JS_EXCEPTION
                } else {
                    JS_NewFloat64(ctx, func.f_f_f.expect("registered f_f_f")(d1, d2))
                }
            }
            JS_CFUNC_iterator_next => {
                let mut done = 0;
                let ret = func.iterator_next.expect("registered iterator_next")(
                    ctx, this_obj, argc, arg_buf, &mut done, magic,
                );
                if JS_IsException(ret) == 0 && done != 2 {
                    js_create_iterator_result(ctx, ret, done)
                } else {
                    ret
                }
            }
            _ => std::process::abort(),
        }
    };
    (*rt).current_stack_frame = sf.prev_frame;
    ret_val
}
// C alloca buffers have no runtime allocator accounting. Vec uses the Rust
// allocator for the corresponding temporary contiguous buffer; references
// in it are borrowed and never duplicated/freed merely for allocation.
// c: quickjs.c:17717
unsafe fn js_call_bound_function(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    this_obj: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
    flags: i32,
) -> JSValue {
    let p = JS_VALUE_GET_OBJ(func_obj);
    let bf = (*p).u.bound_function;
    let arg_count = (*bf).argc.wrapping_add(argc);
    if js_check_stack_overflow(
        (*ctx).rt,
        core::mem::size_of::<JSValue>().wrapping_mul(arg_count as usize),
    ) != 0
    {
        return JS_ThrowStackOverflow(ctx);
    }
    let mut arg_buf = Vec::<JSValue>::new();
    if arg_buf.try_reserve_exact(arg_count as usize).is_err() {
        return JS_ThrowOutOfMemory(ctx);
    }
    for i in 0..(*bf).argc {
        arg_buf.push(*(*bf).argv.as_ptr().add(i as usize));
    }
    for i in 0..argc {
        arg_buf.push(*argv.add(i as usize));
    }
    if flags & JS_CALL_FLAG_CONSTRUCTOR != 0 {
        let new_target = if js_same_value(ctx, func_obj, this_obj) != 0 {
            (*bf).func_obj
        } else {
            this_obj
        };
        JS_CallConstructor2(
            ctx,
            (*bf).func_obj,
            new_target,
            arg_count,
            arg_buf.as_mut_ptr(),
        )
    } else {
        JS_Call(
            ctx,
            (*bf).func_obj,
            (*bf).this_val,
            arg_count,
            arg_buf.as_mut_ptr(),
        )
    }
}
// c: quickjs.c:6024. Data callbacks keep the caller's realm/frame in C.
// The C record's u8 length bounds argument padding to 255 slots.
unsafe fn js_c_function_data_call(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    this_val: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
    _flags: i32,
) -> JSValue {
    let s = JS_GetOpaque(func_obj, JS_CLASS_C_FUNCTION_DATA).cast::<JSCFunctionDataRecord>();
    let mut padded = [JS_UNDEFINED; 255];
    let arg_buf = if argc < (*s).length as i32 {
        for i in 0..argc as usize { padded[i] = *argv.add(i); }
        padded.as_mut_ptr()
    } else { argv };
    (*s).func.expect("C function data callback")(ctx, this_val, argc, arg_buf, (*s).magic as i32, (*s).data.as_mut_ptr())
}
