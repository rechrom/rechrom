// Shared C4133..4402 algorithms, Windows state has no POSIX pollfd allocation.
use crt as libc;
// quickjs-libc.c:4133..4163.
pub unsafe fn js_std_init_handlers(rt: *mut JSRuntime) {
    let state = libc::calloc(1, std::mem::size_of::<JSThreadState>()).cast::<JSThreadState>();
    if state.is_null() {
        eprint!("Could not allocate memory for the worker");
        std::process::exit(1);
    }
    init_list_head(ptr::addr_of_mut!((*state).os_rw_handlers));
    init_list_head(ptr::addr_of_mut!((*state).os_signal_handlers));
    init_list_head(ptr::addr_of_mut!((*state).os_timers));
    init_list_head(ptr::addr_of_mut!((*state).port_list));
    init_list_head(ptr::addr_of_mut!((*state).rejected_promise_list));
    (*state).next_timer_id = 1;
    JS_SetRuntimeOpaque(rt, state.cast());
    let functions = JSSharedArrayBufferFunctions {
        sab_alloc: Some(js_sab_alloc),
        sab_free: Some(js_sab_free),
        sab_dup: Some(js_sab_dup),
        sab_opaque: ptr::null_mut(),
    };
    JS_SetSharedArrayBufferFunctions(rt, &functions);
}
// quickjs-libc.c:4165..4210. Ports belong to Worker finalizers after unlinking.
pub unsafe fn js_std_free_handlers(rt: *mut JSRuntime) {
    let state = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_rw_handlers), false, true) {
        free_rw_handler(rt, link.cast());
    }
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_signal_handlers), false, true) {
        free_sh(rt, link.cast());
    }
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_timers), false, true) {
        free_timer(rt, link.cast());
    }
    for link in ListIter::new(
        ptr::addr_of_mut!((*state).rejected_promise_list),
        false,
        true,
    ) {
        let entry = link.cast::<JSRejectedPromiseEntry>();
        JS_FreeValueRT(rt, (*entry).promise);
        JS_FreeValueRT(rt, (*entry).reason);
        libc::free(entry.cast());
    }
    js_free_message_pipe((*state).recv_pipe);
    js_free_message_pipe((*state).send_pipe);
    for link in ListIter::new(ptr::addr_of_mut!((*state).port_list), false, true) {
        (*link).prev = ptr::null_mut();
        (*link).next = ptr::null_mut();
    }
    libc::free(state.cast());
    JS_SetRuntimeOpaque(rt, ptr::null_mut());
}

// quickjs-libc.c:4227..4269. Match by actual promise identity, never by reason.
unsafe fn find_rejected_promise(
    ctx: *mut JSContext,
    state: *mut JSThreadState,
    promise: JSValueConst,
) -> *mut JSRejectedPromiseEntry {
    for link in ListIter::new(
        ptr::addr_of_mut!((*state).rejected_promise_list),
        false,
        false,
    ) {
        let entry = link.cast::<JSRejectedPromiseEntry>();
        if JS_SameValue(ctx, (*entry).promise, promise) != 0 {
            return entry;
        }
    }
    ptr::null_mut()
}
pub unsafe fn js_std_promise_rejection_tracker(
    ctx: *mut JSContext,
    promise: JSValueConst,
    reason: JSValueConst,
    handled: JS_BOOL,
    _opaque: *mut c_void,
) {
    let state = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<JSThreadState>();
    let mut entry = find_rejected_promise(ctx, state, promise);
    if handled == 0 {
        if entry.is_null() {
            entry = libc::malloc(std::mem::size_of::<JSRejectedPromiseEntry>()).cast();
            if !entry.is_null() {
                (*entry).promise = JS_DupValue(ctx, promise);
                (*entry).reason = JS_DupValue(ctx, reason);
                list_add_tail(
                    ptr::addr_of_mut!((*entry).link),
                    ptr::addr_of_mut!((*state).rejected_promise_list),
                );
            }
        }
    } else if !entry.is_null() {
        JS_FreeValue(ctx, (*entry).promise);
        JS_FreeValue(ctx, (*entry).reason);
        list_del(ptr::addr_of_mut!((*entry).link));
        libc::free(entry.cast());
    }
}
// quickjs-libc.c:4275..4289.
pub unsafe fn js_std_promise_rejection_check(ctx: *mut JSContext) {
    let state = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<JSThreadState>();
    if list_empty(ptr::addr_of_mut!((*state).rejected_promise_list)) == 0 {
        for link in ListIter::new(
            ptr::addr_of_mut!((*state).rejected_promise_list),
            false,
            false,
        ) {
            eprint!("Possibly unhandled promise rejection: ");
            js_std_dump_error1(ctx, (*link.cast::<JSRejectedPromiseEntry>()).reason);
        }
        std::process::exit(1);
    }
}
// quickjs-libc.c:4292..4312.
pub unsafe fn js_std_loop(ctx: *mut JSContext) {
    loop {
        loop {
            let result = JS_ExecutePendingJob(JS_GetRuntime(ctx), ptr::null_mut());
            if result <= 0 {
                if result < 0 {
                    js_std_dump_error(ctx);
                }
                break;
            }
        }
        js_std_promise_rejection_check(ctx);
        if !OS_POLL_ENABLED.load(Ordering::Relaxed) || js_os_poll(ctx) != 0 {
            break;
        }
    }
}
// quickjs-libc.c:4317..4351. Consume the input exactly once on every exit path.
pub unsafe fn js_std_await(ctx: *mut JSContext, object: JSValue) -> JSValue {
    loop {
        match JS_PromiseState(ctx, object) {
            JS_PROMISE_FULFILLED => {
                let result = JS_PromiseResult(ctx, object);
                JS_FreeValue(ctx, object);
                return result;
            }
            JS_PROMISE_REJECTED => {
                let result = JS_Throw(ctx, JS_PromiseResult(ctx, object));
                JS_FreeValue(ctx, object);
                return result;
            }
            JS_PROMISE_PENDING => {
                let result = JS_ExecutePendingJob(JS_GetRuntime(ctx), ptr::null_mut());
                if result < 0 {
                    js_std_dump_error(ctx);
                }
                if result == 0 {
                    js_std_promise_rejection_check(ctx);
                    if OS_POLL_ENABLED.load(Ordering::Relaxed) {
                        js_os_poll(ctx);
                    }
                }
            }
            _ => return object,
        }
    }
}
// quickjs-libc.c:4353..4384.
pub unsafe fn js_std_eval_binary(
    ctx: *mut JSContext,
    buffer: *const u8,
    length: usize,
    load_only: i32,
) {
    let object = JS_ReadObject(ctx, buffer, length, JS_READ_OBJ_BYTECODE);
    if JS_IsException(object) != 0 {
        js_std_dump_error(ctx);
        std::process::exit(1);
    }
    if load_only != 0 {
        if JS_VALUE_GET_TAG(object) == JS_TAG_MODULE {
            js_module_set_import_meta(ctx, object, 0, 0);
        }
        JS_FreeValue(ctx, object);
        return;
    }
    let value = if JS_VALUE_GET_TAG(object) == JS_TAG_MODULE {
        if JS_ResolveModule(ctx, object) < 0 {
            JS_FreeValue(ctx, object);
            js_std_dump_error(ctx);
            std::process::exit(1);
        }
        js_module_set_import_meta(ctx, object, 0, 1);
        js_std_await(ctx, JS_EvalFunction(ctx, object))
    } else {
        JS_EvalFunction(ctx, object)
    };
    if JS_IsException(value) != 0 {
        js_std_dump_error(ctx);
        std::process::exit(1);
    }
    JS_FreeValue(ctx, value);
}
// quickjs-libc.c:4386..4402.
pub unsafe fn js_std_eval_binary_json_module(
    ctx: *mut JSContext,
    buffer: *const u8,
    length: usize,
    name: *const c_char,
) {
    let object = JS_ReadObject(ctx, buffer, length, 0);
    if JS_IsException(object) != 0 || create_json_module(ctx, name, object).is_null() {
        js_std_dump_error(ctx);
        std::process::exit(1);
    }
}
