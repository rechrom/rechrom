// run-test262.c:55..85,225..234,515..943 (QuickJS 2026-06-04).
// Bellard/Gordon MIT. Included in run_test262's namespace. Engine/JSValue
// ownership is unchanged; POSIX threads/mutexes/condition variables are host-only.
use quickjs::list::{init_list_head, list_add_tail, list_del, list_empty, list_head, ListIter};
#[repr(C)]
pub struct ThreadLocalStorage {
    pub agent_mutex: libc::pthread_mutex_t,
    pub agent_cond: libc::pthread_cond_t,
    pub agent_list: list_head,
    pub report_mutex: libc::pthread_mutex_t,
    pub report_list: list_head,
    pub async_done: i32,
}
#[repr(C)]
pub struct Test262Agent {
    pub link: list_head,
    pub tls: *mut ThreadLocalStorage,
    pub tid: libc::pthread_t,
    pub script: *mut c_char,
    pub broadcast_func: JSValue,
    pub broadcast_pending: i32,
    pub broadcast_sab: JSValue,
    pub broadcast_sab_buf: *mut u8,
    pub broadcast_sab_size: usize,
    pub broadcast_val: i32,
}
#[repr(C)]
pub struct AgentReport {
    pub link: list_head,
    pub str_: *mut c_char,
}
static TEST262_OUTFILE: std::sync::atomic::AtomicPtr<libc::FILE> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
/// Same process-wide optional FILE stream as the C runner. Caller owns the
/// stream and must retain it until all agents have joined.
pub fn set_test262_outfile(file: *mut libc::FILE) {
    TEST262_OUTFILE.store(file, std::sync::atomic::Ordering::Release);
}
pub unsafe fn init_thread_local_storage(tls: *mut ThreadLocalStorage) {
    std::ptr::write_bytes(tls, 0, 1);
    libc::pthread_mutex_init(&mut (*tls).agent_mutex, std::ptr::null());
    libc::pthread_cond_init(&mut (*tls).agent_cond, std::ptr::null());
    init_list_head(&mut (*tls).agent_list);
    libc::pthread_mutex_init(&mut (*tls).report_mutex, std::ptr::null());
    init_list_head(&mut (*tls).report_list);
}
/// After js_agent_free: release unread reports and native synchronization.
/// C retains unread reports until process termination; the embedding can clean
/// them between runner instances without changing observable agent behavior.
pub unsafe fn free_thread_local_storage(tls: *mut ThreadLocalStorage) {
    for el in ListIter::new(&mut (*tls).report_list, false, true) {
        let rep = el.cast::<AgentReport>();
        list_del(el);
        libc::free((*rep).str_.cast());
        libc::free(rep.cast());
    }
    libc::pthread_mutex_destroy(&mut (*tls).report_mutex);
    libc::pthread_cond_destroy(&mut (*tls).agent_cond);
    libc::pthread_mutex_destroy(&mut (*tls).agent_mutex);
}
unsafe fn js_print_value_write(opaque: *mut c_void, buf: *const c_char, len: usize) {
    libc::fwrite(buf.cast(), 1, len, opaque.cast());
}
pub unsafe fn js_print(ctx: *mut JSContext, _: JSValue, argc: i32, argv: *mut JSValue) -> JSValue {
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    let file = TEST262_OUTFILE.load(std::sync::atomic::Ordering::Acquire);
    for i in 0..argc {
        if i != 0 && !file.is_null() {
            libc::fputc(b' ' as i32, file);
        }
        let v = *argv.add(i as usize);
        if JS_IsString(v) != 0 {
            let mut len = 0;
            let s = JS_ToCStringLen(ctx, &mut len, v);
            if s.is_null() {
                return JS_EXCEPTION;
            }
            let bytes = CStr::from_ptr(s).to_bytes();
            // Protect the shared completion counter without changing its C layout.
            libc::pthread_mutex_lock(&mut (*tls).report_mutex);
            if bytes == b"Test262:AsyncTestComplete" {
                (*tls).async_done += 1;
            } else if bytes.starts_with(b"Test262:AsyncTestFailure") {
                (*tls).async_done = 2;
            }
            libc::pthread_mutex_unlock(&mut (*tls).report_mutex);
            if !file.is_null() {
                libc::fwrite(s.cast(), 1, len, file);
            }
            JS_FreeCString(ctx, s);
        } else if !file.is_null() {
            JS_PrintValue(
                ctx,
                Some(js_print_value_write),
                file.cast(),
                v,
                std::ptr::null(),
            );
        }
    }
    if !file.is_null() {
        libc::fputc(b'\n' as i32, file);
    }
    JS_UNDEFINED
}
unsafe fn js_detachArrayBuffer(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    JS_DetachArrayBuffer(ctx, *argv);
    JS_UNDEFINED
}
unsafe fn js_evalScript(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    let mut len = 0;
    let s = JS_ToCStringLen(ctx, &mut len, *argv);
    if s.is_null() {
        return JS_EXCEPTION;
    }
    let ret = JS_Eval(ctx, s, len, c"<evalScript>".as_ptr(), JS_EVAL_TYPE_GLOBAL);
    JS_FreeCString(ctx, s);
    ret
}
extern "C" fn agent_start(arg: *mut c_void) -> *mut c_void {
    unsafe {
        let agent = arg.cast::<Test262Agent>();
        let tls = (*agent).tls;
        let rt = JS_NewRuntime();
        if rt.is_null() {
            eprintln!("JS_NewRuntime failure");
            std::process::exit(1);
        }
        JS_SetRuntimeOpaque(rt, tls.cast());
        let ctx = JS_NewContext(rt);
        if ctx.is_null() {
            JS_FreeRuntime(rt);
            eprintln!("JS_NewContext failure");
            std::process::exit(1);
        }
        JS_SetContextOpaque(ctx, agent.cast());
        JS_SetRuntimeInfo(rt, c"agent".as_ptr());
        JS_SetCanBlock(rt, 1);
        add_helpers(ctx);
        let val = JS_Eval(
            ctx,
            (*agent).script,
            libc::strlen((*agent).script),
            c"<evalScript>".as_ptr(),
            JS_EVAL_TYPE_GLOBAL,
        );
        libc::free((*agent).script.cast());
        (*agent).script = std::ptr::null_mut();
        if JS_IsException(val) != 0 {
            crate::quickjs_libc::js_std_dump_error(ctx);
        }
        JS_FreeValue(ctx, val);
        loop {
            let ret = JS_ExecutePendingJob(rt, std::ptr::null_mut());
            if ret < 0 {
                crate::quickjs_libc::js_std_dump_error(ctx);
                break;
            }
            if ret == 0 {
                if JS_IsUndefined((*agent).broadcast_func) != 0 {
                    break;
                }
                libc::pthread_mutex_lock(&mut (*tls).agent_mutex);
                while (*agent).broadcast_pending == 0 {
                    libc::pthread_cond_wait(&mut (*tls).agent_cond, &mut (*tls).agent_mutex);
                }
                (*agent).broadcast_pending = 0;
                libc::pthread_cond_signal(&mut (*tls).agent_cond);
                libc::pthread_mutex_unlock(&mut (*tls).agent_mutex);
                let mut args = [
                    JS_NewArrayBuffer(
                        ctx,
                        (*agent).broadcast_sab_buf,
                        (*agent).broadcast_sab_size,
                        None,
                        std::ptr::null_mut(),
                        1,
                    ),
                    JS_NewInt32(ctx, (*agent).broadcast_val),
                ];
                let val = JS_Call(
                    ctx,
                    (*agent).broadcast_func,
                    JS_UNDEFINED,
                    2,
                    args.as_mut_ptr(),
                );
                for arg in args {
                    JS_FreeValue(ctx, arg);
                }
                if JS_IsException(val) != 0 {
                    crate::quickjs_libc::js_std_dump_error(ctx);
                }
                JS_FreeValue(ctx, val);
                JS_FreeValue(ctx, (*agent).broadcast_func);
                (*agent).broadcast_func = JS_UNDEFINED;
            }
        }
        JS_FreeValue(ctx, (*agent).broadcast_func);
        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
        std::ptr::null_mut()
    }
}
unsafe fn js_agent_start(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    if !JS_GetContextOpaque(ctx).is_null() {
        return JS_ThrowTypeError(ctx, c"cannot be called inside an agent".as_ptr());
    }
    let script = JS_ToCString(ctx, *argv);
    if script.is_null() {
        return JS_EXCEPTION;
    }
    let agent = libc::calloc(1, std::mem::size_of::<Test262Agent>()).cast::<Test262Agent>();
    if agent.is_null() {
        JS_FreeCString(ctx, script);
        return JS_ThrowOutOfMemory(ctx);
    }
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    (*agent).tls = tls;
    (*agent).broadcast_func = JS_UNDEFINED;
    (*agent).broadcast_sab = JS_UNDEFINED;
    (*agent).script = libc::strdup(script);
    JS_FreeCString(ctx, script);
    if (*agent).script.is_null() {
        libc::free(agent.cast());
        return JS_ThrowOutOfMemory(ctx);
    }
    list_add_tail(&mut (*agent).link, &mut (*tls).agent_list);
    let mut attr: libc::pthread_attr_t = std::mem::zeroed();
    libc::pthread_attr_init(&mut attr);
    // C requests 2 MiB. Translated Rust dispatch frames need additional native
    // headroom; preserve the engine's independent default 1 MiB JS budget.
    libc::pthread_attr_setstacksize(&mut attr, 16 << 20);
    let ret = libc::pthread_create(&mut (*agent).tid, &attr, agent_start, agent.cast());
    libc::pthread_attr_destroy(&mut attr);
    if ret != 0 {
        list_del(&mut (*agent).link);
        libc::free((*agent).script.cast());
        libc::free(agent.cast());
        return JS_ThrowInternalError(ctx, c"cannot create agent thread".as_ptr());
    }
    JS_UNDEFINED
}
pub unsafe fn js_agent_free(ctx: *mut JSContext) {
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    for el in ListIter::new(&mut (*tls).agent_list, false, true) {
        let agent = el.cast::<Test262Agent>();
        libc::pthread_join((*agent).tid, std::ptr::null_mut());
        JS_FreeValue(ctx, (*agent).broadcast_sab);
        list_del(el);
        libc::free(agent.cast());
    }
}
unsafe fn js_agent_leaving(ctx: *mut JSContext, _: JSValue, _: i32, _: *mut JSValue) -> JSValue {
    if JS_GetContextOpaque(ctx).is_null() {
        return JS_ThrowTypeError(ctx, c"must be called inside an agent".as_ptr());
    }
    JS_UNDEFINED
}
unsafe fn is_broadcast_pending(tls: *mut ThreadLocalStorage) -> bool {
    ListIter::new(&mut (*tls).agent_list, false, false)
        .any(|el| (*el.cast::<Test262Agent>()).broadcast_pending != 0)
}
unsafe fn js_agent_broadcast(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    if !JS_GetContextOpaque(ctx).is_null() {
        return JS_ThrowTypeError(ctx, c"cannot be called inside an agent".as_ptr());
    }
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    let sab = *argv;
    let mut size = 0;
    let buf = JS_GetArrayBuffer(ctx, &mut size, sab);
    if buf.is_null() {
        return JS_EXCEPTION;
    }
    let mut val = 0;
    if JS_ToInt32(ctx, &mut val, *argv.add(1)) != 0 {
        return JS_EXCEPTION;
    }
    libc::pthread_mutex_lock(&mut (*tls).agent_mutex);
    for el in ListIter::new(&mut (*tls).agent_list, false, false) {
        let agent = el.cast::<Test262Agent>();
        (*agent).broadcast_pending = 1;
        (*agent).broadcast_sab = JS_DupValue(ctx, sab);
        (*agent).broadcast_sab_buf = buf;
        (*agent).broadcast_sab_size = size;
        (*agent).broadcast_val = val;
    }
    libc::pthread_cond_broadcast(&mut (*tls).agent_cond);
    while is_broadcast_pending(tls) {
        libc::pthread_cond_wait(&mut (*tls).agent_cond, &mut (*tls).agent_mutex);
    }
    libc::pthread_mutex_unlock(&mut (*tls).agent_mutex);
    JS_UNDEFINED
}
unsafe fn js_agent_receiveBroadcast(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    argv: *mut JSValue,
) -> JSValue {
    let agent = JS_GetContextOpaque(ctx).cast::<Test262Agent>();
    if agent.is_null() {
        return JS_ThrowTypeError(ctx, c"must be called inside an agent".as_ptr());
    }
    if JS_IsFunction(ctx, *argv) == 0 {
        return JS_ThrowTypeError(ctx, c"expecting function".as_ptr());
    }
    JS_FreeValue(ctx, (*agent).broadcast_func);
    (*agent).broadcast_func = JS_DupValue(ctx, *argv);
    JS_UNDEFINED
}
unsafe fn js_agent_sleep(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    let mut duration = 0;
    if JS_ToUint32(ctx, &mut duration, *argv) != 0 {
        return JS_EXCEPTION;
    }
    libc::usleep(duration.wrapping_mul(1000));
    JS_UNDEFINED
}
unsafe fn get_clock_ms() -> i64 {
    let mut ts: libc::timespec = std::mem::zeroed();
    libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
    (ts.tv_sec as u64)
        .wrapping_mul(1000)
        .wrapping_add((ts.tv_nsec / 1_000_000) as u64) as i64
}
unsafe fn js_agent_monotonicNow(
    ctx: *mut JSContext,
    _: JSValue,
    _: i32,
    _: *mut JSValue,
) -> JSValue {
    JS_NewInt64(ctx, get_clock_ms())
}
unsafe fn js_agent_getReport(ctx: *mut JSContext, _: JSValue, _: i32, _: *mut JSValue) -> JSValue {
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    libc::pthread_mutex_lock(&mut (*tls).report_mutex);
    let rep = if list_empty(&mut (*tls).report_list) != 0 {
        std::ptr::null_mut()
    } else {
        let p = (*tls).report_list.next;
        list_del(p);
        p.cast::<AgentReport>()
    };
    libc::pthread_mutex_unlock(&mut (*tls).report_mutex);
    if rep.is_null() {
        JS_NULL
    } else {
        let v = JS_NewString(ctx, (*rep).str_);
        libc::free((*rep).str_.cast());
        libc::free(rep.cast());
        v
    }
}
unsafe fn js_agent_report(ctx: *mut JSContext, _: JSValue, _: i32, argv: *mut JSValue) -> JSValue {
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    let s = JS_ToCString(ctx, *argv);
    if s.is_null() {
        return JS_EXCEPTION;
    }
    let rep = libc::malloc(std::mem::size_of::<AgentReport>()).cast::<AgentReport>();
    if rep.is_null() {
        JS_FreeCString(ctx, s);
        return JS_ThrowOutOfMemory(ctx);
    }
    (*rep).str_ = libc::strdup(s);
    JS_FreeCString(ctx, s);
    if (*rep).str_.is_null() {
        libc::free(rep.cast());
        return JS_ThrowOutOfMemory(ctx);
    }
    libc::pthread_mutex_lock(&mut (*tls).report_mutex);
    list_add_tail(&mut (*rep).link, &mut (*tls).report_list);
    libc::pthread_mutex_unlock(&mut (*tls).report_mutex);
    JS_UNDEFINED
}
const JS_AGENT_FUNCS: [JSCFunctionListEntry; 8] = [
    JS_CFUNC_DEF(c"start".as_ptr(), 1, Some(js_agent_start)),
    JS_CFUNC_DEF(c"getReport".as_ptr(), 0, Some(js_agent_getReport)),
    JS_CFUNC_DEF(c"broadcast".as_ptr(), 2, Some(js_agent_broadcast)),
    JS_CFUNC_DEF(c"report".as_ptr(), 1, Some(js_agent_report)),
    JS_CFUNC_DEF(c"leaving".as_ptr(), 0, Some(js_agent_leaving)),
    JS_CFUNC_DEF(
        c"receiveBroadcast".as_ptr(),
        1,
        Some(js_agent_receiveBroadcast),
    ),
    JS_CFUNC_DEF(c"sleep".as_ptr(), 1, Some(js_agent_sleep)),
    JS_CFUNC_DEF(c"monotonicNow".as_ptr(), 0, Some(js_agent_monotonicNow)),
];
unsafe fn js_new_agent(ctx: *mut JSContext) -> JSValue {
    let agent = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(ctx, agent, JS_AGENT_FUNCS.as_ptr(), 8);
    agent
}
unsafe fn js_createRealm(ctx: *mut JSContext, _: JSValue, _: i32, _: *mut JSValue) -> JSValue {
    let ctx1 = JS_NewContext(JS_GetRuntime(ctx));
    if ctx1.is_null() {
        return JS_ThrowOutOfMemory(ctx);
    }
    let ret = add_helpers1(ctx1);
    JS_FreeContext(ctx1);
    ret
}
unsafe fn js_IsHTMLDDA(_: *mut JSContext, _: JSValue, _: i32, _: *mut JSValue) -> JSValue {
    JS_NULL
}
unsafe fn js_gc(ctx: *mut JSContext, _: JSValue, _: i32, _: *mut JSValue) -> JSValue {
    JS_RunGC(JS_GetRuntime(ctx));
    JS_UNDEFINED
}
pub unsafe fn add_helpers1(ctx: *mut JSContext) -> JSValue {
    let global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(
        ctx,
        global,
        c"print".as_ptr(),
        JS_NewCFunction(ctx, Some(js_print), c"print".as_ptr(), 1),
    );
    let obj = JS_NewObject(ctx);
    for (name, len, callback) in [
        (c"detachArrayBuffer", 1, js_detachArrayBuffer as JSCFunction),
        (c"evalScript", 1, js_evalScript as JSCFunction),
        (
            c"codePointRange",
            2,
            js_string_codePointRange as JSCFunction,
        ),
    ] {
        JS_SetPropertyStr(
            ctx,
            obj,
            name.as_ptr(),
            JS_NewCFunction(ctx, Some(callback), name.as_ptr(), len),
        );
    }
    JS_SetPropertyStr(ctx, obj, c"agent".as_ptr(), js_new_agent(ctx));
    JS_SetPropertyStr(ctx, obj, c"global".as_ptr(), JS_DupValue(ctx, global));
    JS_SetPropertyStr(
        ctx,
        obj,
        c"createRealm".as_ptr(),
        JS_NewCFunction(ctx, Some(js_createRealm), c"createRealm".as_ptr(), 0),
    );
    let html = JS_NewCFunction(ctx, Some(js_IsHTMLDDA), c"IsHTMLDDA".as_ptr(), 0);
    JS_SetIsHTMLDDA(ctx, html);
    JS_SetPropertyStr(ctx, obj, c"IsHTMLDDA".as_ptr(), html);
    JS_SetPropertyStr(
        ctx,
        obj,
        c"gc".as_ptr(),
        JS_NewCFunction(ctx, Some(js_gc), c"gc".as_ptr(), 0),
    );
    JS_SetPropertyStr(ctx, global, c"$262".as_ptr(), JS_DupValue(ctx, obj));
    JS_FreeValue(ctx, global);
    obj
}
pub unsafe fn add_helpers(ctx: *mut JSContext) {
    JS_FreeValue(ctx, add_helpers1(ctx));
}
