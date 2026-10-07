// Manual official bytecode executed by the combined real VM/Promise graph.
// RawContext's real BasicObjects is used; the full BaseObjects remains outside
// this fixture. The small Function constructor object is real native code.
unsafe fn async_dump_scalar(out: &mut Vec<u8>, value: JSValue) {
    let tag = JS_VALUE_GET_NORM_TAG(value); out.extend_from_slice(&tag.to_le_bytes());
    let bits = if tag == JS_TAG_FLOAT64 { JS_VALUE_GET_FLOAT64(value).to_bits() } else { JS_VALUE_GET_INT(value) as u32 as u64 };
    out.extend_from_slice(&bits.to_le_bytes());
}
unsafe fn async_dump_result(out: &mut Vec<u8>, ctx: *mut JSContext, value: JSValue, object: bool) {
    if object && JS_IsObject(value) != 0 {
        let v = JS_GetProperty(ctx, value, crate::quickjs_atom::JS_ATOM_value);
        let d = JS_GetProperty(ctx, value, crate::quickjs_atom::JS_ATOM_done);
        async_dump_scalar(out, v); async_dump_scalar(out, d); JS_FreeValue(ctx, v); JS_FreeValue(ctx, d);
    } else { async_dump_scalar(out, value); }
}
unsafe fn async_dump_promise(out: &mut Vec<u8>, ctx: *mut JSContext, p: JSValue, object: bool) {
    assert!(JS_IsException(p) == 0);
    out.extend_from_slice(&(JS_PromiseState(ctx, p) as i32).to_le_bytes());
    let r = JS_PromiseResult(ctx, p); async_dump_result(out, ctx, r, object); JS_FreeValue(ctx, r);
}
unsafe fn async_drain(rt: *mut JSRuntime) {
    let mut job_ctx = ptr::null_mut(); let mut count = 0;
    loop { let n = JS_ExecutePendingJob(rt, &mut job_ctx); assert!(n >= 0); if n == 0 { break; } count += 1; assert!(count < 100); }
}
unsafe fn async_fixture_function(ctx: *mut JSContext, kind: u32, code: &[u8]) -> JSValue {
    let b = js_mallocz(ctx, size_of::<JSFunctionBytecode>() + code.len()).cast::<JSFunctionBytecode>(); assert!(!b.is_null());
    (*js_rc(b.cast())).ref_count = 1;
    add_gc_object((*ctx).rt, &mut (*b).header, JS_GC_OBJ_TYPE_FUNCTION_BYTECODE);
    (*b).realm = JS_DupContext(ctx); (*b).stack_size = 8; (*b).set_func_kind(kind as u8);
    (*b).func_name = JS_DupAtom(ctx, crate::quickjs_atom::JS_ATOM_empty_string);
    (*b).byte_code_buf = b.add(1).cast(); (*b).byte_code_len = code.len() as i32;
    ptr::copy_nonoverlapping(code.as_ptr(), (*b).byte_code_buf, code.len());
    let f = js_closure(ctx, JS_MKPTR(JS_TAG_FUNCTION_BYTECODE, b.cast()), ptr::null_mut(), ptr::null_mut(), 0);
    assert!(JS_IsException(f) == 0); f
}
fn async_push(code: &mut Vec<u8>, n: i32) { code.push(OP_push_i32 as u8); code.extend_from_slice(&n.to_le_bytes()); }
pub unsafe fn vm_async_combined_fixture() -> Vec<u8> {
    let mut out = Vec::new(); let rt = JS_NewRuntime(); assert!(!rt.is_null());
    let ctx = JS_NewContextRaw(rt); assert!(!ctx.is_null());
    let ft = JSCFunctionType { generic_magic: Some(js_function_constructor) };
    (*ctx).function_ctor = JS_NewCFunction2(ctx, ft.generic, c"Function".as_ptr(), 1, JS_CFUNC_constructor_or_func_magic, JS_FUNC_NORMAL as i32);
    assert!(JS_IsException((*ctx).function_ctor) == 0); assert_eq!(JS_AddIntrinsicPromise(ctx), 0);
    let mut arg = JS_NewInt32(ctx, 37);
    for i in 0..64i32 {
        let mut code = vec![OP_initial_yield as u8]; async_push(&mut code, i - 17); code.extend_from_slice(&[OP_yield as u8, OP_drop as u8, OP_drop as u8]); async_push(&mut code, i + 29); code.push(OP_return_async as u8);
        let f = async_fixture_function(ctx, JS_FUNC_GENERATOR, &code);
        for start in [GEN_MAGIC_NEXT, GEN_MAGIC_RETURN, GEN_MAGIC_THROW] {
            let g = JS_Call(ctx, f, JS_UNDEFINED, 0, ptr::null_mut()); assert!(JS_IsException(g) == 0);
            for k in 0..3 {
                let mut done = 0; let v = js_generator_next(ctx, g, 1, &mut arg, &mut done, if k == 0 { start } else { GEN_MAGIC_NEXT });
                out.extend_from_slice(&done.to_le_bytes()); async_dump_scalar(&mut out, v);
                if JS_IsException(v) != 0 { let e = JS_GetException(ctx); async_dump_scalar(&mut out, e); JS_FreeValue(ctx, e); }
                JS_FreeValue(ctx, v);
            }
            JS_FreeValue(ctx, g);
        }
        JS_FreeValue(ctx, f);
        for await_value in [false, true] {
            let mut code = Vec::new(); async_push(&mut code, i - 17); if await_value { code.push(OP_await as u8); } code.push(OP_return_async as u8);
            let f = async_fixture_function(ctx, JS_FUNC_ASYNC, &code);
            let p = JS_Call(ctx, f, JS_UNDEFINED, 0, ptr::null_mut()); async_dump_promise(&mut out, ctx, p, false); async_drain(rt); async_dump_promise(&mut out, ctx, p, false);
            JS_FreeValue(ctx, p); JS_FreeValue(ctx, f);
        }
        for await_value in [false, true] {
            let mut code = vec![OP_initial_yield as u8]; async_push(&mut code, i - 17);
            if await_value { code.extend_from_slice(&[OP_await as u8, OP_drop as u8]); async_push(&mut code, i + 19); }
            code.extend_from_slice(&[OP_yield as u8, OP_drop as u8, OP_drop as u8]); async_push(&mut code, i + 29); code.push(OP_return_async as u8);
            let f = async_fixture_function(ctx, JS_FUNC_ASYNC_GENERATOR, &code); let g = JS_Call(ctx, f, JS_UNDEFINED, 0, ptr::null_mut()); assert!(JS_IsException(g) == 0);
            for _ in 0..3 { let p = js_async_generator_next(ctx, g, 1, &mut arg, GEN_MAGIC_NEXT); async_dump_promise(&mut out, ctx, p, true); async_drain(rt); async_dump_promise(&mut out, ctx, p, true); JS_FreeValue(ctx, p); }
            JS_FreeValue(ctx, g); JS_FreeValue(ctx, f);
        }
    }
    assert!((*rt).current_stack_frame.is_null());
    JS_FreeContext(ctx); JS_FreeRuntime(rt); out
}
