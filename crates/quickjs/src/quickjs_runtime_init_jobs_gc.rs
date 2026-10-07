// quickjs.c:2263..2342,6510..6538,6815..6838,61027..61059. MIT.
unsafe fn JS_EnqueueJob2(
    ctx: *mut JSContext,
    job_func: Option<JSJobFunc>,
    argc: i32,
    argv: *mut JSValueConst,
    no_exception: i32,
) -> i32 {
    let rt = (*ctx).rt;
    let size =
        size_of::<JSJobEntry>().wrapping_add((argc as usize).wrapping_mul(size_of::<JSValue>()));
    let e = if no_exception != 0 {
        js_malloc_rt(rt, size)
    } else {
        js_malloc(ctx, size)
    }
    .cast::<JSJobEntry>();
    if e.is_null() {
        return -1;
    }
    (*e).realm = JS_DupContext(ctx);
    (*e).job_func = job_func;
    (*e).argc = argc;
    for i in 0..argc {
        *ptr::addr_of_mut!((*e).argv)
            .cast::<JSValue>()
            .add(i as usize) = JS_DupValue(ctx, *argv.add(i as usize));
    }
    list_add_tail(&mut (*e).link, &mut (*rt).job_list);
    0
}
pub unsafe fn JS_EnqueueJob(
    ctx: *mut JSContext,
    job_func: Option<JSJobFunc>,
    argc: i32,
    argv: *mut JSValueConst,
) -> i32 {
    JS_EnqueueJob2(ctx, job_func, argc, argv, 0)
}

pub unsafe fn JS_ExecutePendingJob(rt: *mut JSRuntime, pctx: *mut *mut JSContext) -> i32 {
    if list_empty(&mut (*rt).job_list) != 0 {
        if !pctx.is_null() {
            *pctx = ptr::null_mut();
        }
        return 0;
    }
    let e = (*rt)
        .job_list
        .next
        .cast::<u8>()
        .sub(offset_of!(JSJobEntry, link))
        .cast::<JSJobEntry>();
    list_del(&mut (*e).link);
    let ctx = (*e).realm;
    let argv = ptr::addr_of_mut!((*e).argv).cast::<JSValue>();
    let mut trace = browser_tracing::span("javascript", "PendingJob");
    trace.set("argc", (*e).argc as f64);
    let stack_profile = js_job_stack_profile_begin(rt, (*e).job_func.unwrap() as usize);
    let res = (*e).job_func.unwrap()(ctx, (*e).argc, argv);
    drop(stack_profile);
    for i in 0..(*e).argc {
        JS_FreeValue(ctx, *argv.add(i as usize));
    }
    let ret = if JS_IsException(res) != 0 { -1 } else { 1 };
    trace.set("failed", u8::from(ret < 0) as f64);
    JS_FreeValue(ctx, res);
    js_free(ctx, e.cast());
    if !pctx.is_null() {
        *pctx = if (*js_rc(ctx.cast())).ref_count > 1 {
            ctx
        } else {
            ptr::null_mut()
        };
    }
    JS_FreeContext(ctx);
    ret
}
unsafe fn js_finrec_job(ctx: *mut JSContext, _argc: i32, argv: *mut JSValueConst) -> JSValue {
    JS_Call(ctx, *argv, JS_UNDEFINED, 1, argv.add(1))
}
unsafe fn finrec_delete_weakref(rt: *mut JSRuntime, wh: *mut JSWeakRefHeader) {
    let frd = wh
        .cast::<u8>()
        .sub(offset_of!(JSFinalizationRegistryData, weakref_header))
        .cast::<JSFinalizationRegistryData>();
    for el in ListIter::new(&mut (*frd).entries, false, true) {
        let fre = el
            .cast::<u8>()
            .sub(offset_of!(JSFinRecEntry, link))
            .cast::<JSFinRecEntry>();
        if js_weakref_is_live((*fre).token) == 0 {
            js_weakref_free(rt, (*fre).token);
            (*fre).token = JS_UNDEFINED;
        }
        if js_weakref_is_live((*fre).target) == 0 {
            let mut args = [(*frd).cb, (*fre).held_val];
            JS_EnqueueJob2((*frd).realm, Some(js_finrec_job), 2, args.as_mut_ptr(), 1);
            js_weakref_free(rt, (*fre).target);
            js_weakref_free(rt, (*fre).token);
            JS_FreeValueRT(rt, (*fre).held_val);
            list_del(&mut (*fre).link);
            js_free_rt(rt, fre.cast());
        }
    }
}
unsafe fn gc_remove_weak_objects(rt: *mut JSRuntime) {
    (*rt).gc_phase = JS_GC_PHASE_DECREF as u8;
    for el in ListIter::new(&mut (*rt).weakref_list, false, false) {
        let wh = el
            .cast::<u8>()
            .sub(offset_of!(JSWeakRefHeader, link))
            .cast::<JSWeakRefHeader>();
        match (*wh).weakref_type {
            JS_WEAKREF_TYPE_MAP => map_delete_weakrefs(rt, wh),
            JS_WEAKREF_TYPE_WEAKREF => weakref_delete_weakref(rt, wh),
            JS_WEAKREF_TYPE_FINREC => finrec_delete_weakref(rt, wh),
            _ => std::process::abort(),
        }
    }
    (*rt).gc_phase = JS_GC_PHASE_NONE as u8;
    free_zero_refcount(rt);
}
unsafe fn JS_RunGCInternal(rt: *mut JSRuntime, remove_weak_objects: i32) {
    let mut trace = browser_tracing::span("gc", "QuickJSGC");
    trace.set("remove_weak_objects", remove_weak_objects as f64);
    if remove_weak_objects != 0 {
        let _phase = browser_tracing::span("gc", "QuickJSWeakObjects");
        gc_remove_weak_objects(rt);
    }
    {
        let _phase = browser_tracing::span("gc", "QuickJSDecref");
        gc_decref(rt);
    }
    {
        let _phase = browser_tracing::span("gc", "QuickJSScan");
        gc_scan(rt);
    }
    {
        let _phase = browser_tracing::span("gc", "QuickJSFreeCycles");
        gc_free_cycles(rt);
    }
}
pub unsafe fn JS_RunGC(rt: *mut JSRuntime) {
    JS_RunGCInternal(rt, 1);
}
