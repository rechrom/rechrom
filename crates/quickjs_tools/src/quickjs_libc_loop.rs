// POSIX host state/loop from official quickjs-libc.c; MIT, see LICENSE.
// Included into quickjs_libc so original private helper scope is preserved.
use super::{quickjs_libc_state::*, quickjs_libc_worker::*};
use quickjs::list::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub static OS_PENDING_SIGNALS: AtomicU64 = AtomicU64::new(0);
static OS_POLL_ENABLED: AtomicBool = AtomicBool::new(false);
// The original process-global pointer only ever holds js_os_poll on POSIX.
pub fn js_os_enable_poll() { OS_POLL_ENABLED.store(true, Ordering::Relaxed); }
pub unsafe fn interrupt_handler(_rt: *mut JSRuntime, _opaque: *mut c_void) -> i32 {
    ((OS_PENDING_SIGNALS.load(Ordering::Relaxed) >> libc::SIGINT) & 1) as i32
}

// quickjs-libc.c:2004..2012,2068..2073,2168..2173.
pub unsafe fn free_rw_handler(rt: *mut JSRuntime, handler: *mut JSOSRWHandler) {
    list_del(ptr::addr_of_mut!((*handler).link));
    JS_FreeValueRT(rt, (*handler).rw_func[0]);
    JS_FreeValueRT(rt, (*handler).rw_func[1]);
    js_free_rt(rt, handler.cast());
}
pub unsafe fn free_sh(rt: *mut JSRuntime, handler: *mut JSOSSignalHandler) {
    list_del(ptr::addr_of_mut!((*handler).link));
    JS_FreeValueRT(rt, (*handler).func);
    js_free_rt(rt, handler.cast());
}
pub unsafe fn free_timer(rt: *mut JSRuntime, timer: *mut JSOSTimer) {
    list_del(ptr::addr_of_mut!((*timer).link));
    JS_FreeValueRT(rt, (*timer).func);
    js_free_rt(rt, timer.cast());
}

// quickjs-libc.c:2265..2276. Retain before calling: it may delete its own node.
pub unsafe fn call_handler(ctx: *mut JSContext, function: JSValueConst) {
    let retained = JS_DupValue(ctx, function);
    let value = JS_Call(ctx, retained, JS_UNDEFINED, 0, ptr::null_mut());
    JS_FreeValue(ctx, retained);
    if JS_IsException(value) != 0 { js_std_dump_error(ctx); }
    JS_FreeValue(ctx, value);
}

// quickjs-libc.c:2517..2528.
unsafe fn js_poll_expand(state: *mut JSThreadState) -> i32 {
    let size = ((*state).poll_fds_size + (*state).poll_fds_size / 2).max(16);
    let data: *mut libc::pollfd = libc::realloc((*state).poll_fds.cast(), size as usize * std::mem::size_of::<libc::pollfd>()).cast();
    if data.is_null() { return -1; }
    (*state).poll_fds = data;
    (*state).poll_fds_size = size;
    0
}
// quickjs-libc.c:2530..2545.
unsafe fn js_poll_add_poll_fd(state: *mut JSThreadState, count: &mut i32, fd: i32, events: i32) -> i32 {
    if *count >= (*state).poll_fds_size && js_poll_expand(state) != 0 { return -1; }
    let poll = (*state).poll_fds.add(*count as usize);
    *count += 1;
    (*poll).fd = fd;
    (*poll).events = events as i16;
    (*poll).revents = 0;
    0
}
// quickjs-libc.c:2547..2657. Dispatch one callback, then rebuild lists/fd indices.
pub unsafe fn js_os_poll(ctx: *mut JSContext) -> i32 {
    let rt = JS_GetRuntime(ctx);
    let state = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    if (*state).recv_pipe.is_null() && OS_PENDING_SIGNALS.load(Ordering::Relaxed) != 0 {
        for link in ListIter::new(ptr::addr_of_mut!((*state).os_signal_handlers), false, false) {
            let handler = link.cast::<JSOSSignalHandler>();
            let mask = 1u64 << (*handler).sig_num;
            if OS_PENDING_SIGNALS.load(Ordering::Relaxed) & mask != 0 {
                OS_PENDING_SIGNALS.fetch_and(!mask, Ordering::Relaxed);
                call_handler(ctx, (*handler).func);
                return 0;
            }
        }
    }
    if list_empty(ptr::addr_of_mut!((*state).os_rw_handlers)) != 0 &&
       list_empty(ptr::addr_of_mut!((*state).os_timers)) != 0 &&
       list_empty(ptr::addr_of_mut!((*state).port_list)) != 0 { return -1; }
    let mut delay = -1;
    if list_empty(ptr::addr_of_mut!((*state).os_timers)) == 0 {
        let now = get_time_ms();
        delay = 10000;
        for link in ListIter::new(ptr::addr_of_mut!((*state).os_timers), false, false) {
            let timer = link.cast::<JSOSTimer>();
            let remaining = (*timer).timeout.wrapping_sub(now);
            if remaining <= 0 {
                let function = (*timer).func;
                (*timer).func = JS_UNDEFINED;
                free_timer(rt, timer);
                call_handler(ctx, function);
                JS_FreeValue(ctx, function);
                return 0;
            } else if remaining < delay as i64 { delay = remaining as i32; }
        }
    }
    let mut count = 0;
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_rw_handlers), false, false) {
        let handler = link.cast::<JSOSRWHandler>();
        let mut events = 0;
        if JS_IsNull((*handler).rw_func[0]) == 0 { events |= libc::POLLIN as i32; }
        if JS_IsNull((*handler).rw_func[1]) == 0 { events |= libc::POLLOUT as i32; }
        if events != 0 {
            (*handler).poll_fd_index = count;
            if js_poll_add_poll_fd(state, &mut count, (*handler).fd, events) != 0 { return -1; }
        }
    }
    for link in ListIter::new(ptr::addr_of_mut!((*state).port_list), false, false) {
        let port = link.cast::<JSWorkerMessageHandler>();
        if JS_IsNull((*port).on_message_func) == 0 {
            (*port).poll_fd_index = count;
            if js_poll_add_poll_fd(state, &mut count, (*(*port).recv_pipe).waker.read_fd, libc::POLLIN as i32) != 0 { return -1; }
        }
    }
    let ready = libc::poll((*state).poll_fds, count as libc::nfds_t, delay);
    if ready > 0 {
        for link in ListIter::new(ptr::addr_of_mut!((*state).os_rw_handlers), false, false) {
            let handler = link.cast::<JSOSRWHandler>();
            let events = (*(*state).poll_fds.add((*handler).poll_fd_index as usize)).revents;
            if JS_IsNull((*handler).rw_func[0]) == 0 && events & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL | libc::POLLIN) != 0 {
                call_handler(ctx, (*handler).rw_func[0]); return 0;
            }
            if JS_IsNull((*handler).rw_func[1]) == 0 && events & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL | libc::POLLOUT) != 0 {
                call_handler(ctx, (*handler).rw_func[1]); return 0;
            }
        }
        for link in ListIter::new(ptr::addr_of_mut!((*state).port_list), false, false) {
            let port = link.cast::<JSWorkerMessageHandler>();
            if JS_IsNull((*port).on_message_func) == 0 && (*(*state).poll_fds.add((*port).poll_fd_index as usize)).revents != 0 &&
                handle_posted_message(rt, ctx, port) != 0 { return 0; }
        }
    }
    0
}

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
        sab_alloc: Some(js_sab_alloc), sab_free: Some(js_sab_free),
        sab_dup: Some(js_sab_dup), sab_opaque: ptr::null_mut(),
    };
    JS_SetSharedArrayBufferFunctions(rt, &functions);
}
// quickjs-libc.c:4165..4210. Ports belong to Worker finalizers after unlinking.
pub unsafe fn js_std_free_handlers(rt: *mut JSRuntime) {
    let state = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_rw_handlers), false, true) { free_rw_handler(rt, link.cast()); }
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_signal_handlers), false, true) { free_sh(rt, link.cast()); }
    for link in ListIter::new(ptr::addr_of_mut!((*state).os_timers), false, true) { free_timer(rt, link.cast()); }
    for link in ListIter::new(ptr::addr_of_mut!((*state).rejected_promise_list), false, true) {
        let entry = link.cast::<JSRejectedPromiseEntry>();
        JS_FreeValueRT(rt, (*entry).promise);
        JS_FreeValueRT(rt, (*entry).reason);
        libc::free(entry.cast());
    }
    js_free_message_pipe((*state).recv_pipe);
    js_free_message_pipe((*state).send_pipe);
    for link in ListIter::new(ptr::addr_of_mut!((*state).port_list), false, true) {
        (*link).prev = ptr::null_mut(); (*link).next = ptr::null_mut();
    }
    libc::free((*state).poll_fds.cast());
    libc::free(state.cast());
    JS_SetRuntimeOpaque(rt, ptr::null_mut());
}

// quickjs-libc.c:4227..4269. Match by actual promise identity, never by reason.
unsafe fn find_rejected_promise(ctx: *mut JSContext, state: *mut JSThreadState, promise: JSValueConst) -> *mut JSRejectedPromiseEntry {
    for link in ListIter::new(ptr::addr_of_mut!((*state).rejected_promise_list), false, false) {
        let entry = link.cast::<JSRejectedPromiseEntry>();
        if JS_SameValue(ctx, (*entry).promise, promise) != 0 { return entry; }
    }
    ptr::null_mut()
}
pub unsafe fn js_std_promise_rejection_tracker(ctx: *mut JSContext, promise: JSValueConst, reason: JSValueConst, handled: JS_BOOL, _opaque: *mut c_void) {
    let state = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<JSThreadState>();
    let mut entry = find_rejected_promise(ctx, state, promise);
    if handled == 0 {
        if entry.is_null() {
            entry = libc::malloc(std::mem::size_of::<JSRejectedPromiseEntry>()).cast();
            if !entry.is_null() {
                (*entry).promise = JS_DupValue(ctx, promise);
                (*entry).reason = JS_DupValue(ctx, reason);
                list_add_tail(ptr::addr_of_mut!((*entry).link), ptr::addr_of_mut!((*state).rejected_promise_list));
            }
        }
    } else if !entry.is_null() {
        JS_FreeValue(ctx, (*entry).promise); JS_FreeValue(ctx, (*entry).reason);
        list_del(ptr::addr_of_mut!((*entry).link)); libc::free(entry.cast());
    }
}
// quickjs-libc.c:4275..4289.
pub unsafe fn js_std_promise_rejection_check(ctx: *mut JSContext) {
    let state = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<JSThreadState>();
    if list_empty(ptr::addr_of_mut!((*state).rejected_promise_list)) == 0 {
        for link in ListIter::new(ptr::addr_of_mut!((*state).rejected_promise_list), false, false) {
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
                if result < 0 { js_std_dump_error(ctx); }
                break;
            }
        }
        js_std_promise_rejection_check(ctx);
        if !OS_POLL_ENABLED.load(Ordering::Relaxed) || js_os_poll(ctx) != 0 { break; }
    }
}
// quickjs-libc.c:4317..4351. Consume the input exactly once on every exit path.
pub unsafe fn js_std_await(ctx: *mut JSContext, object: JSValue) -> JSValue {
    loop {
        match JS_PromiseState(ctx, object) {
            JS_PROMISE_FULFILLED => {
                let result = JS_PromiseResult(ctx, object); JS_FreeValue(ctx, object); return result;
            }
            JS_PROMISE_REJECTED => {
                let result = JS_Throw(ctx, JS_PromiseResult(ctx, object)); JS_FreeValue(ctx, object); return result;
            }
            JS_PROMISE_PENDING => {
                let result = JS_ExecutePendingJob(JS_GetRuntime(ctx), ptr::null_mut());
                if result < 0 { js_std_dump_error(ctx); }
                if result == 0 {
                    js_std_promise_rejection_check(ctx);
                    if OS_POLL_ENABLED.load(Ordering::Relaxed) { js_os_poll(ctx); }
                }
            }
            _ => return object,
        }
    }
}
// quickjs-libc.c:4353..4384.
pub unsafe fn js_std_eval_binary(ctx: *mut JSContext, buffer: *const u8, length: usize, load_only: i32) {
    let object = JS_ReadObject(ctx, buffer, length, JS_READ_OBJ_BYTECODE);
    if JS_IsException(object) != 0 { js_std_dump_error(ctx); std::process::exit(1); }
    if load_only != 0 {
        if JS_VALUE_GET_TAG(object) == JS_TAG_MODULE { js_module_set_import_meta(ctx, object, 0, 0); }
        JS_FreeValue(ctx, object);
        return;
    }
    let value = if JS_VALUE_GET_TAG(object) == JS_TAG_MODULE {
        if JS_ResolveModule(ctx, object) < 0 {
            JS_FreeValue(ctx, object); js_std_dump_error(ctx); std::process::exit(1);
        }
        js_module_set_import_meta(ctx, object, 0, 1);
        js_std_await(ctx, JS_EvalFunction(ctx, object))
    } else { JS_EvalFunction(ctx, object) };
    if JS_IsException(value) != 0 { js_std_dump_error(ctx); std::process::exit(1); }
    JS_FreeValue(ctx, value);
}
// quickjs-libc.c:4386..4402.
pub unsafe fn js_std_eval_binary_json_module(ctx: *mut JSContext, buffer: *const u8, length: usize, name: *const c_char) {
    let object = JS_ReadObject(ctx, buffer, length, 0);
    if JS_IsException(object) != 0 || create_json_module(ctx, name, object).is_null() {
        js_std_dump_error(ctx); std::process::exit(1);
    }
}
