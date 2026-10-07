// quickjs.c:2067..2124,2216..2219,2593..2649,2856..2860. MIT.
pub unsafe fn JS_NewRuntime2(mf: *const JSMallocFunctions, opaque: *mut c_void) -> *mut JSRuntime {
    let mut ms: JSMallocState = core::mem::zeroed();
    ms.opaque = opaque;
    ms.malloc_limit = usize::MAX;
    let rt = (*mf).js_malloc.unwrap()(&mut ms, size_of::<JSRuntime>()).cast::<JSRuntime>();
    if rt.is_null() {
        return ptr::null_mut();
    }
    ptr::write_bytes(rt, 0, 1);
    js_malloc_init(&mut (*rt).malloc_ctx);
    (*rt).malloc_ctx.mf = ptr::read(mf);
    (*rt).malloc_ctx.malloc_state = ms;
    (*rt).malloc_gc_threshold = 256 * 1024;
    init_list_head(&mut (*rt).context_list);
    init_list_head(&mut (*rt).gc_obj_list);
    init_list_head(&mut (*rt).gc_zero_ref_count_list);
    (*rt).gc_phase = JS_GC_PHASE_NONE as u8;
    init_list_head(&mut (*rt).weakref_list);
    init_list_head(&mut (*rt).job_list);
    let mut fail = JS_InitAtoms(rt) != 0;
    if !fail {
        fail = init_class_range(
            rt,
            js_std_class_def.as_ptr(),
            JS_CLASS_OBJECT as i32,
            js_std_class_def.len() as i32,
        ) < 0;
    }
    if !fail {
        (*(*rt).class_array.add(JS_CLASS_ARGUMENTS as usize)).exotic =
            ptr::addr_of_mut!(js_arguments_exotic_methods);
        (*(*rt).class_array.add(JS_CLASS_MAPPED_ARGUMENTS as usize)).exotic =
            ptr::addr_of_mut!(js_arguments_exotic_methods);
        (*(*rt).class_array.add(JS_CLASS_STRING as usize)).exotic =
            ptr::addr_of_mut!(js_string_exotic_methods);
        (*(*rt).class_array.add(JS_CLASS_MODULE_NS as usize)).exotic =
            ptr::addr_of!(js_module_ns_exotic_methods);
        (*(*rt).class_array.add(JS_CLASS_C_FUNCTION as usize)).call = Some(js_call_c_function);
        (*(*rt).class_array.add(JS_CLASS_C_FUNCTION_DATA as usize)).call =
            Some(js_c_function_data_call);
        (*(*rt).class_array.add(JS_CLASS_BOUND_FUNCTION as usize)).call =
            Some(js_call_bound_function);
        (*(*rt).class_array.add(JS_CLASS_GENERATOR_FUNCTION as usize)).call =
            Some(js_generator_function_call);
        fail = init_shape_hash(rt) != 0;
    }
    if fail {
        JS_FreeRuntime(rt);
        return ptr::null_mut();
    }
    (*rt).stack_size = 1024 * 1024; // JS_DEFAULT_STACK_SIZE, quickjs.h.
    JS_UpdateStackTop(rt);
    (*rt).current_exception = JS_UNINITIALIZED;
    rt
}
pub unsafe fn JS_NewRuntime() -> *mut JSRuntime {
    JS_NewRuntime2(&def_malloc_funcs, ptr::null_mut())
}
pub unsafe fn JS_UpdateStackTop(rt: *mut JSRuntime) {
    (*rt).stack_top = js_get_stack_pointer();
    update_stack_limit(rt);
}
pub unsafe fn JS_NewContextRaw(rt: *mut JSRuntime) -> *mut JSContext {
    let ctx = js_mallocz_rt(rt, size_of::<JSContext>()).cast::<JSContext>();
    if ctx.is_null() {
        return ptr::null_mut();
    }
    (*js_rc(ctx.cast())).ref_count = 1;
    add_gc_object(rt, &mut (*ctx).header, JS_GC_OBJ_TYPE_JS_CONTEXT);
    (*ctx).class_proto = js_malloc_rt(rt, size_of::<JSValue>() * (*rt).class_count as usize).cast();
    if (*ctx).class_proto.is_null() {
        // Original ordering retained: add_gc_object precedes this allocation.
        js_free_rt(rt, ctx.cast());
        return ptr::null_mut();
    }
    (*ctx).rt = rt;
    list_add_tail(&mut (*ctx).link, &mut (*rt).context_list);
    for i in 0..(*rt).class_count {
        *(*ctx).class_proto.add(i as usize) = JS_NULL;
    }
    (*ctx).array_ctor = JS_NULL;
    (*ctx).iterator_ctor = JS_NULL;
    (*ctx).regexp_ctor = JS_NULL;
    (*ctx).promise_ctor = JS_NULL;
    init_list_head(&mut (*ctx).loaded_modules);
    if JS_AddIntrinsicBasicObjects(ctx) != 0 {
        JS_FreeContext(ctx);
        return ptr::null_mut();
    }
    ctx
}
pub unsafe fn JS_NewContext(rt: *mut JSRuntime) -> *mut JSContext {
    let ctx = JS_NewContextRaw(rt);
    if ctx.is_null() {
        return ptr::null_mut();
    }
    if JS_AddIntrinsicBaseObjects(ctx) != 0
        || JS_AddIntrinsicDate(ctx) != 0
        || JS_AddIntrinsicEval(ctx) != 0
        || JS_AddIntrinsicStringNormalize(ctx) != 0
        || JS_AddIntrinsicRegExp(ctx) != 0
        || JS_AddIntrinsicJSON(ctx) != 0
        || JS_AddIntrinsicProxy(ctx) != 0
        || JS_AddIntrinsicMapSet(ctx) != 0
        || JS_AddIntrinsicTypedArrays(ctx) != 0
        || JS_AddIntrinsicPromise(ctx) != 0
        || JS_AddIntrinsicWeakRef(ctx) != 0
    {
        JS_FreeContext(ctx);
        return ptr::null_mut();
    }
    ctx
}
