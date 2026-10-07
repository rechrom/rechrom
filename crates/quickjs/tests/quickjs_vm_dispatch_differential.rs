// Executes the actual translated VM, using manually assembled C bytecode.
// The unavailable parser/intrinsics/slow operators stay explicit test-only
// boundaries; this fixture covers primitive fast paths and native dispatch.
unsafe fn vm_dump(out: &mut Vec<u8>, rt: *mut JSRuntime, value: JSValue) {
    let tag = JS_VALUE_GET_NORM_TAG(value);
    out.extend_from_slice(&tag.to_le_bytes());
    let bits = if tag == JS_TAG_FLOAT64 { JS_VALUE_GET_FLOAT64(value).to_bits() }
        else { JS_VALUE_GET_INT(value) as u32 as u64 };
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(&((*rt).current_stack_frame.is_null() as u32).to_le_bytes());
}
unsafe fn vm_native_generic(ctx: *mut JSContext, this_val: JSValueConst, argc: i32, argv: *mut JSValueConst) -> JSValue {
    let sf = (*(*ctx).rt).current_stack_frame;
    let mut n = argc * 100_000 + (*sf).arg_count * 1000 + JS_VALUE_GET_INT(this_val) * 10;
    for i in 0..3 { n += JS_VALUE_GET_INT(*argv.add(i)); }
    JS_NewInt32(ctx, n)
}
unsafe fn vm_native_magic(ctx: *mut JSContext, this_val: JSValueConst, argc: i32, argv: *mut JSValueConst, magic: i32) -> JSValue {
    let n = vm_native_generic(ctx, this_val, argc, argv);
    JS_NewInt32(ctx, JS_VALUE_GET_INT(n) + magic)
}
unsafe fn vm_native_getter(ctx: *mut JSContext, this_val: JSValueConst) -> JSValue { JS_NewInt32(ctx, JS_VALUE_GET_INT(this_val) + 31) }
unsafe fn vm_native_setter(ctx: *mut JSContext, this_val: JSValueConst, v: JSValueConst) -> JSValue { JS_NewInt32(ctx, JS_VALUE_GET_INT(this_val) + JS_VALUE_GET_INT(v) + 17) }
unsafe fn vm_native_getter_magic(ctx: *mut JSContext, this_val: JSValueConst, magic: i32) -> JSValue { JS_NewInt32(ctx, JS_VALUE_GET_INT(this_val) + magic) }
unsafe fn vm_native_setter_magic(ctx: *mut JSContext, this_val: JSValueConst, v: JSValueConst, magic: i32) -> JSValue { JS_NewInt32(ctx, JS_VALUE_GET_INT(this_val) + JS_VALUE_GET_INT(v) + magic) }
unsafe fn vm_native_f_f(x: f64) -> f64 { let y = x * x - 1.0; if y.is_nan() { f64::from_bits(0x7ff8000000000000) } else { y } }
unsafe fn vm_native_f_f_f(x: f64, y: f64) -> f64 { let z = x * x - y; if z.is_nan() { f64::from_bits(0x7ff8000000000000) } else { z } }
unsafe fn vm_native_iterator(ctx: *mut JSContext, this_val: JSValueConst, argc: i32, argv: *mut JSValueConst, done: *mut i32, magic: i32) -> JSValue { *done = 2; vm_native_magic(ctx, this_val, argc, argv, magic) }
unsafe fn vm_native_bound(ctx: *mut JSContext, this_val: JSValueConst, argc: i32, argv: *mut JSValueConst) -> JSValue {
    let marker = if JS_IsObject(this_val) != 0 { (*JS_VALUE_GET_OBJ(this_val)).class_id as i32 } else { JS_VALUE_GET_INT(this_val) };
    let sf = (*(*ctx).rt).current_stack_frame;
    let mut n = argc * 100000 + (*sf).arg_count * 1000 + marker * 10;
    for i in 0..argc { n += JS_VALUE_GET_INT(*argv.offset(i as isize)) * (i + 1); }
    JS_NewInt32(ctx, n)
}
unsafe fn vm_native_data(ctx: *mut JSContext, this_val: JSValueConst, argc: i32, argv: *mut JSValueConst, magic: i32, data: *mut JSValue) -> JSValue {
    let mut n = argc * 100000 + magic + JS_VALUE_GET_INT(this_val) * 10
        + ((*(*ctx).rt).current_stack_frame.is_null() as i32) * 1000000
        + JS_VALUE_GET_INT(*data) * 3 + JS_VALUE_GET_INT(*data.add(1)) * 5;
    for i in 0..3 { n += JS_VALUE_GET_INT(*argv.add(i)) * (i as i32 + 1); }
    JS_NewInt32(ctx, n)
}
pub unsafe fn vm_dispatch_fixture() -> Vec<u8> {
    let mut out = Vec::new();
    let mut rt: JSRuntime = core::mem::zeroed();
    let mut ctx: JSContext = core::mem::zeroed();
    let mut b: JSFunctionBytecode = core::mem::zeroed();
    let mut p: JSObject = core::mem::zeroed();
    ctx.rt = &mut rt;
    rt.current_exception = JS_UNINITIALIZED;
    let mut classes: Vec<JSClass> = (0..JS_CLASS_INIT_COUNT).map(|_| core::mem::zeroed()).collect();
    classes[JS_CLASS_C_FUNCTION as usize].call = Some(js_call_c_function);
    rt.class_array = classes.as_mut_ptr();
    rt.class_count = classes.len() as i32;
    p.class_id = JS_CLASS_BYTECODE_FUNCTION as u16;
    p.u.func.function_bytecode = &mut b;
    b.realm = &mut ctx;
    b.stack_size = 8;
    b.var_count = 2;
    let function = JS_MKPTR(JS_TAG_OBJECT, ptr::addr_of_mut!(p).cast());
    let mut seed = 0x8473_9823u32;
    for i in 0..32768u32 {
        seed ^= seed.wrapping_shl(13); seed ^= seed.wrapping_shr(17); seed ^= seed.wrapping_shl(5);
        let x = if i % 16 == 0 { i as i32 - 100 } else { seed as i32 };
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let y = seed as i32;
        for opcode in [OP_add, OP_sub, OP_mul, OP_shl, OP_sar, OP_and, OP_xor, OP_or, OP_div, OP_mod, OP_shr, OP_lt, OP_lte, OP_gt, OP_gte, OP_eq, OP_neq, OP_strict_eq, OP_strict_neq] {
            let mut code = vec![OP_push_i32 as u8]; code.extend_from_slice(&x.to_le_bytes());
            code.push(OP_push_i32 as u8); code.extend_from_slice(&y.to_le_bytes());
            code.extend_from_slice(&[opcode as u8, OP_return as u8]);
            b.byte_code_buf = code.as_mut_ptr(); b.byte_code_len = code.len() as i32;
            ctx.interrupt_counter = 10000;
            let v = JS_Call(&mut ctx, function, JS_UNDEFINED, 0, ptr::null_mut());
            vm_dump(&mut out, &mut rt, v);
            JS_FreeValue(&mut ctx, v);
        }
        for opcode in [OP_neg, OP_not, OP_lnot] {
            let mut code = vec![OP_push_i32 as u8]; code.extend_from_slice(&x.to_le_bytes());
            code.extend_from_slice(&[opcode as u8, OP_return as u8]);
            b.byte_code_buf = code.as_mut_ptr(); b.byte_code_len = code.len() as i32;
            ctx.interrupt_counter = 10000;
            let v = JS_Call(&mut ctx, function, JS_UNDEFINED, 0, ptr::null_mut());
            vm_dump(&mut out, &mut rt, v);
            JS_FreeValue(&mut ctx, v);
        }
        let mut code = vec![OP_push_i32 as u8]; code.extend_from_slice(&x.to_le_bytes());
        code.extend_from_slice(&[OP_put_loc as u8, 0, 0, OP_inc_loc as u8, 0, OP_dec_loc as u8, 0, OP_get_loc as u8, 0, 0, OP_return as u8]);
        b.byte_code_buf = code.as_mut_ptr(); b.byte_code_len = code.len() as i32;
        ctx.interrupt_counter = 10000;
        let v = JS_Call(&mut ctx, function, JS_UNDEFINED, 0, ptr::null_mut());
        vm_dump(&mut out, &mut rt, v);
        JS_FreeValue(&mut ctx, v);
    }
    #[cfg(feature = "short-opcodes")]
    {
        let mut codes: Vec<Vec<u8>> = (OP_push_minus1..=OP_push_7).map(|op| vec![op as u8, OP_return as u8]).collect();
        codes.extend([
            vec![OP_push_i8 as u8, 128, OP_return as u8],
            vec![OP_push_i16 as u8, 0, 128, OP_return as u8],
            vec![OP_push_7 as u8, OP_put_loc0 as u8, OP_get_loc0 as u8, OP_return as u8],
            vec![OP_push_i8 as u8, 238, OP_put_loc8 as u8, 0, OP_get_loc8 as u8, 0, OP_return as u8],
            vec![OP_push_0 as u8, OP_if_false8 as u8, 3, OP_push_1 as u8, OP_return as u8, OP_push_7 as u8, OP_return as u8],
            vec![OP_push_1 as u8, OP_if_false8 as u8, 3, OP_push_1 as u8, OP_return as u8, OP_push_7 as u8, OP_return as u8],
        ]);
        for code in &mut codes {
            b.byte_code_buf = code.as_mut_ptr(); b.byte_code_len = code.len() as i32; ctx.interrupt_counter = 10000;
            let v = JS_Call(&mut ctx, function, JS_UNDEFINED, 0, ptr::null_mut());
            vm_dump(&mut out, &mut rt, v); JS_FreeValue(&mut ctx, v);
        }
    }
    p.class_id = JS_CLASS_C_FUNCTION as u16;
    p.u.cfunc.realm = &mut ctx;
    p.u.cfunc.length = 3;
    p.u.cfunc.magic = -173;
    let mut args = [JS_NewInt32(&mut ctx, 11), JS_NewInt32(&mut ctx, 31), JS_NewInt32(&mut ctx, -17), JS_NewInt32(&mut ctx, 9), JS_NewInt32(&mut ctx, 27)];
    for cproto in 0..=JS_CFUNC_iterator_next {
        p.u.cfunc.cproto = cproto as u8;
        p.u.cfunc.c_function = match cproto {
            JS_CFUNC_generic | JS_CFUNC_constructor | JS_CFUNC_constructor_or_func => JSCFunctionType { generic: Some(vm_native_generic) },
            JS_CFUNC_generic_magic | JS_CFUNC_constructor_magic | JS_CFUNC_constructor_or_func_magic => JSCFunctionType { generic_magic: Some(vm_native_magic) },
            JS_CFUNC_f_f => JSCFunctionType { f_f: Some(vm_native_f_f) },
            JS_CFUNC_f_f_f => JSCFunctionType { f_f_f: Some(vm_native_f_f_f) },
            JS_CFUNC_getter => JSCFunctionType { getter: Some(vm_native_getter) },
            JS_CFUNC_setter => JSCFunctionType { setter: Some(vm_native_setter) },
            JS_CFUNC_getter_magic => JSCFunctionType { getter_magic: Some(vm_native_getter_magic) },
            JS_CFUNC_setter_magic => JSCFunctionType { setter_magic: Some(vm_native_setter_magic) },
            JS_CFUNC_iterator_next => JSCFunctionType { iterator_next: Some(vm_native_iterator) },
            _ => unreachable!(),
        };
        for argc in 0..=5 {
            ctx.interrupt_counter = 10000;
            let v = js_call_c_function(&mut ctx, function, JS_NewInt32(&mut ctx, 4), argc, args.as_mut_ptr(), JS_CALL_FLAG_CONSTRUCTOR);
            vm_dump(&mut out, &mut rt, v);
            JS_FreeValue(&mut ctx, v);
        }
    }
    p.set_is_constructor(1);
    p.u.cfunc.cproto = JS_CFUNC_constructor_or_func as u8;
    p.u.cfunc.c_function = JSCFunctionType { generic: Some(vm_native_bound) };
    let mut storage = vec![0u64; (size_of::<JSBoundFunction>() + 2 * size_of::<JSValue>()).div_ceil(8)];
    let bf = storage.as_mut_ptr().cast::<JSBoundFunction>();
    (*bf).func_obj = function; (*bf).this_val = JS_NewInt32(&mut ctx, 19); (*bf).argc = 2;
    *(*bf).argv.as_mut_ptr() = JS_NewInt32(&mut ctx, -7);
    *(*bf).argv.as_mut_ptr().add(1) = JS_NewInt32(&mut ctx, 18);
    // SameValue duplicates/releases object references, so these fixture objects
    // carry the original real arena refcount header, with one retained owner.
    #[repr(C)] struct FixtureObject { header: JSMallocBlockHeader, object: JSObject }
    let mut bound_store: FixtureObject = core::mem::zeroed();
    let bound = ptr::addr_of_mut!(bound_store.object); (*js_rc(bound.cast())).ref_count = 1;
    (*bound).class_id = JS_CLASS_BOUND_FUNCTION as u16; (*bound).u.bound_function = bf;
    let bound_val = JS_MKPTR(JS_TAG_OBJECT, bound.cast());
    let mut other_store: FixtureObject = core::mem::zeroed();
    let other = ptr::addr_of_mut!(other_store.object); (*js_rc(other.cast())).ref_count = 1;
    (*other).class_id = JS_CLASS_OBJECT as u16;
    let other_val = JS_MKPTR(JS_TAG_OBJECT, other.cast());
    for argc in 0..=5 {
        for new_target in [JS_UNDEFINED, bound_val, other_val] {
            ctx.interrupt_counter = 10000;
            let flags = if JS_IsUndefined(new_target) != 0 { 0 } else { JS_CALL_FLAG_CONSTRUCTOR };
            let v = js_call_bound_function(&mut ctx, bound_val, new_target, argc, args.as_mut_ptr(), flags);
            vm_dump(&mut out, &mut rt, v); JS_FreeValue(&mut ctx, v);
        }
    }
    let mut data_storage = vec![0u64; (size_of::<JSCFunctionDataRecord>() + 2 * size_of::<JSValue>()).div_ceil(8)];
    let data = data_storage.as_mut_ptr().cast::<JSCFunctionDataRecord>();
    (*data).func = Some(vm_native_data); (*data).length = 3; (*data).magic = (-173i32) as u16; (*data).data_len = 2;
    *(*data).data.as_mut_ptr() = JS_NewInt32(&mut ctx, 72); *(*data).data.as_mut_ptr().add(1) = JS_NewInt32(&mut ctx, -11);
    classes[JS_CLASS_C_FUNCTION_DATA as usize].call = Some(js_c_function_data_call);
    p.class_id = JS_CLASS_C_FUNCTION_DATA as u16; p.u.opaque = data.cast();
    for argc in 0..=5 {
        ctx.interrupt_counter = 10000;
        let v = JS_Call(&mut ctx, function, JS_NewInt32(&mut ctx, 4), argc, args.as_mut_ptr());
        vm_dump(&mut out, &mut rt, v); JS_FreeValue(&mut ctx, v);
    }
    out
}
