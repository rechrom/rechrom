//! Original quickjs-libc.c Worker and shared-memory host algorithms.
//! POSIX services and detached threads remain outside the engine crate.
use super::{quickjs_libc::*, quickjs_libc_state::*};
use quickjs::{list::*, quickjs::*, quickjs_header::*};
use std::{
    ffi::{c_char, c_void},
    mem::{size_of, MaybeUninit},
    ptr,
    sync::{
        atomic::{AtomicI32, Ordering},
        Mutex, OnceLock,
    },
};

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
unsafe fn host_errno() -> i32 {
    *libc::__error()
}
#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn host_errno() -> i32 {
    *libc::__errno_location()
}

// C:2284..2348. Keep its exact retry conditions, including signal's OR test.
pub unsafe fn js_waker_init(w: *mut JSWaker) -> i32 {
    let mut fds = [0; 2];
    if libc::pipe(fds.as_mut_ptr()) < 0 {
        return -1;
    }
    (*w).read_fd = fds[0];
    (*w).write_fd = fds[1];
    0
}
pub unsafe fn js_waker_signal(w: *mut JSWaker) {
    loop {
        let ret = libc::write((*w).write_fd, c"".as_ptr().cast(), 1);
        if ret == 1 {
            break;
        }
        if ret < 0 && (host_errno() != libc::EAGAIN || host_errno() != libc::EINTR) {
            break;
        }
    }
}
pub unsafe fn js_waker_clear(w: *mut JSWaker) {
    let mut buf = [0u8; 16];
    loop {
        let ret = libc::read((*w).read_fd, buf.as_mut_ptr().cast(), buf.len());
        if ret >= 0 {
            break;
        }
        if host_errno() != libc::EAGAIN && host_errno() != libc::EINTR {
            break;
        }
    }
}
pub unsafe fn js_waker_close(w: *mut JSWaker) {
    libc::close((*w).read_fd);
    libc::close((*w).write_fd);
    (*w).read_fd = -1;
    (*w).write_fd = -1;
}

// C:3465..3485. uint64_t buf[0] aligns the C header to 8 bytes.
#[repr(C)]
pub struct JSWorkerData {
    pub recv_pipe: *mut JSWorkerMessagePipe,
    pub send_pipe: *mut JSWorkerMessagePipe,
    pub msg_handler: *mut JSWorkerMessageHandler,
}
#[repr(C)]
pub struct WorkerFuncArgs {
    pub filename: *mut c_char,
    pub basename: *mut c_char,
    pub recv_pipe: *mut JSWorkerMessagePipe,
    pub send_pipe: *mut JSWorkerMessagePipe,
    pub strip_flags: i32,
}
#[repr(C, align(8))]
struct JSSABHeader {
    ref_count: AtomicI32,
}
static WORKER_CLASS_ID: OnceLock<JSClassID> = OnceLock::new();
static WORKER_NEW_CONTEXT: Mutex<Option<unsafe fn(*mut JSRuntime) -> *mut JSContext>> =
    Mutex::new(None);
unsafe fn worker_class_id() -> JSClassID {
    *WORKER_CLASS_ID.get_or_init(|| {
        let mut id = 0;
        JS_NewClassID(&mut id);
        id
    })
}
// C:3486..3490, seq_cst matches C's unspecified/default atomic order.
pub unsafe fn atomic_add_int(p: *const AtomicI32, v: i32) -> i32 {
    (*p).fetch_add(v, Ordering::SeqCst).wrapping_add(v)
}
// C:3493..3520. Messages may outlive either runtime: use the common host heap.
pub unsafe fn js_sab_alloc(_opaque: *mut c_void, size: usize) -> *mut c_void {
    let Some(total) = size.checked_add(size_of::<JSSABHeader>()) else {
        return ptr::null_mut();
    };
    let sab = libc::malloc(total).cast::<JSSABHeader>();
    if sab.is_null() {
        return ptr::null_mut();
    }
    ptr::write(
        sab,
        JSSABHeader {
            ref_count: AtomicI32::new(1),
        },
    );
    sab.cast::<u8>().add(size_of::<JSSABHeader>()).cast()
}
pub unsafe fn js_sab_free(_opaque: *mut c_void, p: *mut c_void) {
    let sab = p
        .cast::<u8>()
        .sub(size_of::<JSSABHeader>())
        .cast::<JSSABHeader>();
    let count = atomic_add_int(ptr::addr_of!((*sab).ref_count), -1);
    assert!(count >= 0);
    if count == 0 {
        libc::free(sab.cast());
    }
}
pub unsafe fn js_sab_dup(_opaque: *mut c_void, p: *mut c_void) {
    let sab = p
        .cast::<u8>()
        .sub(size_of::<JSSABHeader>())
        .cast::<JSSABHeader>();
    atomic_add_int(ptr::addr_of!((*sab).ref_count), 1);
}
// C:3521..3537.
pub unsafe fn js_new_message_pipe() -> *mut JSWorkerMessagePipe {
    let ps = libc::malloc(size_of::<JSWorkerMessagePipe>()).cast::<JSWorkerMessagePipe>();
    if ps.is_null() {
        return ps;
    }
    if js_waker_init(ptr::addr_of_mut!((*ps).waker)) != 0 {
        libc::free(ps.cast());
        return ptr::null_mut();
    }
    ptr::write(ptr::addr_of_mut!((*ps).ref_count), AtomicI32::new(1));
    init_list_head(ptr::addr_of_mut!((*ps).msg_queue));
    libc::pthread_mutex_init(ptr::addr_of_mut!((*ps).mutex), ptr::null());
    ps
}
pub unsafe fn js_dup_message_pipe(ps: *mut JSWorkerMessagePipe) -> *mut JSWorkerMessagePipe {
    atomic_add_int(ptr::addr_of!((*ps).ref_count), 1);
    ps
}
// C:3544..3555.
pub unsafe fn js_free_message(msg: *mut JSWorkerMessage) {
    for i in 0..(*msg).sab_tab_len {
        js_sab_free(ptr::null_mut(), (*(*msg).sab_tab.add(i)).cast());
    }
    libc::free((*msg).sab_tab.cast());
    libc::free((*msg).data.cast());
    libc::free(msg.cast());
}
// C:3556..3576.
pub unsafe fn js_free_message_pipe(ps: *mut JSWorkerMessagePipe) {
    if ps.is_null() {
        return;
    }
    let count = atomic_add_int(ptr::addr_of!((*ps).ref_count), -1);
    assert!(count >= 0);
    if count == 0 {
        for link in ListIter::new(ptr::addr_of_mut!((*ps).msg_queue), false, true) {
            js_free_message(link.cast());
        }
        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*ps).mutex));
        js_waker_close(ptr::addr_of_mut!((*ps).waker));
        libc::free(ps.cast());
    }
}
// C:3578..3589.
pub unsafe fn js_free_port(rt: *mut JSRuntime, port: *mut JSWorkerMessageHandler) {
    if !port.is_null() {
        js_free_message_pipe((*port).recv_pipe);
        JS_FreeValueRT(rt, (*port).on_message_func);
        if !(*port).link.prev.is_null() {
            list_del(ptr::addr_of_mut!((*port).link));
        }
        js_free_rt(rt, port.cast());
    }
}
// C:2357..2409. Snapshot the callback because it may remove/free its own port.
pub unsafe fn handle_posted_message(
    _rt: *mut JSRuntime,
    ctx: *mut JSContext,
    port: *mut JSWorkerMessageHandler,
) -> i32 {
    let ps = (*port).recv_pipe;
    libc::pthread_mutex_lock(ptr::addr_of_mut!((*ps).mutex));
    if list_empty(ptr::addr_of_mut!((*ps).msg_queue)) != 0 {
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*ps).mutex));
        return 0;
    }
    let msg = (*ps).msg_queue.next.cast::<JSWorkerMessage>();
    list_del(ptr::addr_of_mut!((*msg).link));
    if list_empty(ptr::addr_of_mut!((*ps).msg_queue)) != 0 {
        js_waker_clear(ptr::addr_of_mut!((*ps).waker));
    }
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*ps).mutex));
    let data = JS_ReadObject(
        ctx,
        (*msg).data,
        (*msg).data_len,
        JS_READ_OBJ_SAB | JS_READ_OBJ_REFERENCE,
    );
    js_free_message(msg);
    if JS_IsException(data) != 0 {
        js_std_dump_error(ctx);
        return 1;
    }
    let mut obj = JS_NewObject(ctx);
    if JS_IsException(obj) != 0 {
        JS_FreeValue(ctx, data);
        js_std_dump_error(ctx);
        return 1;
    }
    JS_DefinePropertyValueStr(ctx, obj, c"data".as_ptr(), data, JS_PROP_C_W_E);
    let func = JS_DupValue(ctx, (*port).on_message_func);
    let value = JS_Call(ctx, func, JS_UNDEFINED, 1, &mut obj);
    JS_FreeValue(ctx, obj);
    JS_FreeValue(ctx, func);
    if JS_IsException(value) != 0 {
        js_std_dump_error(ctx);
    } else {
        JS_FreeValue(ctx, value);
    }
    1
}
// C:3590..3618.
unsafe fn js_worker_finalizer(rt: *mut JSRuntime, value: JSValue) {
    let worker = JS_GetOpaque(value, worker_class_id()).cast::<JSWorkerData>();
    if !worker.is_null() {
        js_free_message_pipe((*worker).recv_pipe);
        js_free_message_pipe((*worker).send_pipe);
        js_free_port(rt, (*worker).msg_handler);
        js_free_rt(rt, worker.cast());
    }
}
unsafe fn js_worker_mark(rt: *mut JSRuntime, value: JSValueConst, mark: Option<JS_MarkFunc>) {
    let worker = JS_GetOpaque(value, worker_class_id()).cast::<JSWorkerData>();
    if !worker.is_null() {
        let port = (*worker).msg_handler;
        if !port.is_null() {
            JS_MarkValue(
                rt,
                (*port).on_message_func,
                mark.expect("GC supplies its mark callback"),
            );
        }
    }
}
// C:3620..3668. A detached POSIX thread owns the argument block and its pipes.
extern "C" fn worker_func(opaque: *mut c_void) -> *mut c_void {
    unsafe {
        let args = opaque.cast::<WorkerFuncArgs>();
        let rt = JS_NewRuntime();
        if rt.is_null() {
            eprint!("JS_NewRuntime failure");
            std::process::exit(1);
        }
        JS_SetStripInfo(rt, (*args).strip_flags);
        js_std_init_handlers(rt);
        JS_SetModuleLoaderFunc2(
            rt,
            None,
            Some(js_module_loader),
            Some(js_module_check_attributes),
            ptr::null_mut(),
        );
        let ts = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
        (*ts).recv_pipe = (*args).recv_pipe;
        (*ts).send_pipe = (*args).send_pipe;
        let new_context = *WORKER_NEW_CONTEXT.lock().unwrap_or_else(|e| e.into_inner());
        // Caller supplies the context factory, just as the C tool's function pointer.
        let ctx = new_context
            .expect("js_std_set_worker_new_context_func must precede Worker construction")(
            rt
        );
        if ctx.is_null() {
            eprint!("JS_NewContext failure");
        }
        JS_SetCanBlock(rt, 1);
        js_std_add_helpers(ctx, -1, ptr::null_mut());
        let value = JS_LoadModule(ctx, (*args).basename, (*args).filename);
        libc::free((*args).filename.cast());
        libc::free((*args).basename.cast());
        libc::free(args.cast());
        let value = js_std_await(ctx, value);
        if JS_IsException(value) != 0 {
            js_std_dump_error(ctx);
        }
        JS_FreeValue(ctx, value);
        js_std_loop(ctx);
        JS_FreeContext(ctx);
        js_std_free_handlers(rt);
        JS_FreeRuntime(rt);
        ptr::null_mut()
    }
}
// C:3670..3700.
pub unsafe fn js_worker_ctor_internal(
    ctx: *mut JSContext,
    new_target: JSValueConst,
    recv_pipe: *mut JSWorkerMessagePipe,
    send_pipe: *mut JSWorkerMessagePipe,
) -> JSValue {
    let class = worker_class_id();
    let proto = if JS_IsUndefined(new_target) != 0 {
        JS_GetClassProto(ctx, class)
    } else {
        JS_GetPropertyStr(ctx, new_target, c"prototype".as_ptr())
    };
    if JS_IsException(proto) != 0 {
        return JS_EXCEPTION;
    }
    let obj = JS_NewObjectProtoClass(ctx, proto, class);
    JS_FreeValue(ctx, proto);
    if JS_IsException(obj) != 0 {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    let s = js_mallocz(ctx, size_of::<JSWorkerData>()).cast::<JSWorkerData>();
    if s.is_null() {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    (*s).recv_pipe = js_dup_message_pipe(recv_pipe);
    (*s).send_pipe = js_dup_message_pipe(send_pipe);
    JS_SetOpaque(obj, s.cast());
    obj
}
// C:3702..3781. Allocation/pipe ownership passes to the detached thread only
// after pthread_create succeeds; the JS Worker retains the inverse pair.
pub unsafe fn js_worker_ctor(
    ctx: *mut JSContext,
    new_target: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let rt = JS_GetRuntime(ctx);
    let ts = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    if !(*ts).recv_pipe.is_null() {
        return JS_ThrowTypeError(ctx, c"cannot create a worker inside a worker".as_ptr());
    }
    if WORKER_NEW_CONTEXT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_none()
    {
        return JS_ThrowTypeError(ctx, c"worker context factory is not configured".as_ptr());
    }
    let atom = JS_GetScriptOrModuleName(ctx, 1);
    if atom == JS_ATOM_NULL as u32 {
        return JS_ThrowTypeError(
            ctx,
            c"could not determine calling script or module name".as_ptr(),
        );
    }
    let basename = JS_AtomToCString(ctx, atom);
    JS_FreeAtom(ctx, atom);
    if basename.is_null() {
        return JS_EXCEPTION;
    }
    let filename = JS_ToCString(ctx, *argv);
    if filename.is_null() {
        JS_FreeCString(ctx, basename);
        return JS_EXCEPTION;
    }
    let mut args = ptr::null_mut::<WorkerFuncArgs>();
    let mut obj = JS_UNDEFINED;
    let success = (|| {
        args = libc::calloc(1, size_of::<WorkerFuncArgs>()).cast();
        if args.is_null() {
            JS_ThrowOutOfMemory(ctx);
            return false;
        }
        (*args).filename = libc::strdup(filename);
        (*args).basename = libc::strdup(basename);
        // C forgot to test its strdup results; handle host OOM before a new thread
        // can dereference a NULL filename. Normal allocation behavior is unchanged.
        if (*args).filename.is_null() || (*args).basename.is_null() {
            JS_ThrowOutOfMemory(ctx);
            return false;
        }
        (*args).recv_pipe = js_new_message_pipe();
        if (*args).recv_pipe.is_null() {
            JS_ThrowOutOfMemory(ctx);
            return false;
        }
        (*args).send_pipe = js_new_message_pipe();
        if (*args).send_pipe.is_null() {
            JS_ThrowOutOfMemory(ctx);
            return false;
        }
        (*args).strip_flags = JS_GetStripInfo(rt);
        obj = js_worker_ctor_internal(ctx, new_target, (*args).send_pipe, (*args).recv_pipe);
        if JS_IsException(obj) != 0 {
            return false;
        }
        let mut attr = MaybeUninit::<libc::pthread_attr_t>::uninit();
        libc::pthread_attr_init(attr.as_mut_ptr());
        let mut attr = attr.assume_init();
        libc::pthread_attr_setdetachstate(&mut attr, libc::PTHREAD_CREATE_DETACHED);
        // Explicit tool-thread provisioning: Darwin's 512KiB pthread default cannot
        // safely contain the official 1MiB JS limit with translated native frames.
        // Keep the engine's limit intact and provision this host thread to 16MiB.
        let ret = libc::pthread_attr_setstacksize(&mut attr, 16 * 1024 * 1024);
        let ret = if ret == 0 {
            let mut tid = MaybeUninit::<libc::pthread_t>::uninit();
            libc::pthread_create(tid.as_mut_ptr(), &attr, worker_func, args.cast())
        } else {
            ret
        };
        libc::pthread_attr_destroy(&mut attr);
        if ret != 0 {
            JS_ThrowTypeError(ctx, c"could not create worker".as_ptr());
            return false;
        }
        true
    })();
    JS_FreeCString(ctx, basename);
    JS_FreeCString(ctx, filename);
    if success {
        obj
    } else {
        if !args.is_null() {
            libc::free((*args).filename.cast());
            libc::free((*args).basename.cast());
            js_free_message_pipe((*args).recv_pipe);
            js_free_message_pipe((*args).send_pipe);
            libc::free(args.cast());
        }
        JS_FreeValue(ctx, obj);
        JS_EXCEPTION
    }
}
// C:3783..3850. Copy out of the per-runtime allocator before crossing a thread.
pub unsafe fn js_worker_postMessage(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let worker = JS_GetOpaque2(ctx, this_val, worker_class_id()).cast::<JSWorkerData>();
    if worker.is_null() {
        return JS_EXCEPTION;
    }
    let mut data_len = 0;
    let mut sab_tab_len = 0;
    let mut sab_tab = ptr::null_mut();
    let data = JS_WriteObject2(
        ctx,
        &mut data_len,
        *argv,
        JS_WRITE_OBJ_SAB | JS_WRITE_OBJ_REFERENCE,
        &mut sab_tab,
        &mut sab_tab_len,
    );
    if data.is_null() {
        return JS_EXCEPTION;
    }
    let msg = libc::malloc(size_of::<JSWorkerMessage>()).cast::<JSWorkerMessage>();
    let success = (|| {
        if msg.is_null() {
            return false;
        }
        (*msg).data = ptr::null_mut();
        (*msg).sab_tab = ptr::null_mut();
        (*msg).data = libc::malloc(data_len).cast();
        if (*msg).data.is_null() {
            return false;
        }
        ptr::copy_nonoverlapping(data, (*msg).data, data_len);
        (*msg).data_len = data_len;
        if sab_tab_len > 0 {
            (*msg).sab_tab = libc::malloc(size_of::<*mut u8>() * sab_tab_len).cast();
            if (*msg).sab_tab.is_null() {
                return false;
            }
            ptr::copy_nonoverlapping(sab_tab, (*msg).sab_tab, sab_tab_len);
        }
        (*msg).sab_tab_len = sab_tab_len;
        true
    })();
    js_free(ctx, data.cast());
    js_free(ctx, sab_tab.cast());
    if !success {
        if !msg.is_null() {
            libc::free((*msg).data.cast());
            libc::free((*msg).sab_tab.cast());
            libc::free(msg.cast());
        }
        return JS_EXCEPTION;
    }
    for i in 0..(*msg).sab_tab_len {
        js_sab_dup(ptr::null_mut(), (*(*msg).sab_tab.add(i)).cast());
    }
    let ps = (*worker).send_pipe;
    libc::pthread_mutex_lock(ptr::addr_of_mut!((*ps).mutex));
    if list_empty(ptr::addr_of_mut!((*ps).msg_queue)) != 0 {
        js_waker_signal(ptr::addr_of_mut!((*ps).waker));
    }
    list_add_tail(
        ptr::addr_of_mut!((*msg).link),
        ptr::addr_of_mut!((*ps).msg_queue),
    );
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*ps).mutex));
    JS_UNDEFINED
}
// C:3852..3899. Port removal can happen reentrantly in the callback itself.
pub unsafe fn js_worker_set_onmessage(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    func: JSValueConst,
) -> JSValue {
    let rt = JS_GetRuntime(ctx);
    let ts = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    let worker = JS_GetOpaque2(ctx, this_val, worker_class_id()).cast::<JSWorkerData>();
    if worker.is_null() {
        return JS_EXCEPTION;
    }
    let mut port = (*worker).msg_handler;
    if JS_IsNull(func) != 0 {
        if !port.is_null() {
            js_free_port(rt, port);
            (*worker).msg_handler = ptr::null_mut();
        }
    } else {
        if JS_IsFunction(ctx, func) == 0 {
            return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
        }
        if port.is_null() {
            port = js_mallocz(ctx, size_of::<JSWorkerMessageHandler>()).cast();
            if port.is_null() {
                return JS_EXCEPTION;
            }
            (*port).recv_pipe = js_dup_message_pipe((*worker).recv_pipe);
            (*port).on_message_func = JS_NULL;
            list_add_tail(
                ptr::addr_of_mut!((*port).link),
                ptr::addr_of_mut!((*ts).port_list),
            );
            (*worker).msg_handler = port;
        }
        JS_FreeValue(ctx, (*port).on_message_func);
        (*port).on_message_func = JS_DupValue(ctx, func);
    }
    JS_UNDEFINED
}
pub unsafe fn js_worker_get_onmessage(ctx: *mut JSContext, this_val: JSValueConst) -> JSValue {
    let worker = JS_GetOpaque2(ctx, this_val, worker_class_id()).cast::<JSWorkerData>();
    if worker.is_null() {
        return JS_EXCEPTION;
    }
    let port = (*worker).msg_handler;
    if port.is_null() {
        JS_NULL
    } else {
        JS_DupValue(ctx, (*port).on_message_func)
    }
}
// C:3901..3904, exact 2-entry native prototype table.
const js_worker_proto_funcs: [JSCFunctionListEntry; 2] = [
    JS_CFUNC_DEF(c"postMessage".as_ptr(), 1, Some(js_worker_postMessage)),
    JS_CGETSET_DEF(
        c"onmessage".as_ptr(),
        Some(js_worker_get_onmessage),
        Some(js_worker_set_onmessage),
    ),
];
// C:3908..3914. The function pointer avoids linking a whole context factory
// into an embedding host that chooses a reduced set of intrinsics.
pub unsafe fn js_std_set_worker_new_context_func(
    func: Option<unsafe fn(*mut JSRuntime) -> *mut JSContext>,
) {
    *WORKER_NEW_CONTEXT.lock().unwrap_or_else(|e| e.into_inner()) = func;
}
// C:4016..4043, worker block of js_os_init. Root handles normal OS exports.
pub unsafe fn js_worker_init(ctx: *mut JSContext, m: *mut JSModuleDef) {
    let rt = JS_GetRuntime(ctx);
    let ts = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    let class = worker_class_id();
    let definition = JSClassDef {
        class_name: c"Worker".as_ptr(),
        finalizer: Some(js_worker_finalizer),
        gc_mark: Some(js_worker_mark),
        call: None,
        exotic: ptr::null_mut(),
    };
    JS_NewClass(rt, class, &definition);
    let proto = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(
        ctx,
        proto,
        js_worker_proto_funcs.as_ptr(),
        js_worker_proto_funcs.len() as i32,
    );
    let obj = JS_NewCFunction2(
        ctx,
        Some(js_worker_ctor),
        c"Worker".as_ptr(),
        1,
        JS_CFUNC_constructor,
        0,
    );
    JS_SetConstructor(ctx, obj, proto);
    JS_SetClassProto(ctx, class, proto);
    if !(*ts).recv_pipe.is_null() && !(*ts).send_pipe.is_null() {
        JS_DefinePropertyValueStr(
            ctx,
            obj,
            c"parent".as_ptr(),
            js_worker_ctor_internal(ctx, JS_UNDEFINED, (*ts).recv_pipe, (*ts).send_pipe),
            JS_PROP_C_W_E,
        );
    }
    JS_SetModuleExport(ctx, m, c"Worker".as_ptr(), obj);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    unsafe fn eval(ctx: *mut JSContext, source: &str) -> JSValue {
        let mut bytes = source.as_bytes().to_vec();
        bytes.push(0);
        let v = JS_Eval(
            ctx,
            bytes.as_ptr().cast(),
            source.len(),
            c"worker-gate.js".as_ptr(),
            JS_EVAL_TYPE_GLOBAL,
        );
        if JS_IsException(v) != 0 {
            let e = JS_GetException(ctx);
            let p = JS_ToCString(ctx, e);
            let message = CStr::from_ptr(p).to_string_lossy().into_owned();
            JS_FreeCString(ctx, p);
            JS_FreeValue(ctx, e);
            panic!("{message}");
        }
        v
    }
    unsafe fn text(ctx: *mut JSContext, v: JSValue) -> String {
        let p = JS_ToCString(ctx, v);
        assert!(!p.is_null());
        let s = CStr::from_ptr(p).to_string_lossy().into_owned();
        JS_FreeCString(ctx, p);
        s
    }
    unsafe fn register_class(ctx: *mut JSContext) {
        let class = worker_class_id();
        let definition = JSClassDef {
            class_name: c"Worker".as_ptr(),
            finalizer: Some(js_worker_finalizer),
            gc_mark: Some(js_worker_mark),
            call: None,
            exotic: ptr::null_mut(),
        };
        assert_eq!(JS_NewClass(JS_GetRuntime(ctx), class, &definition), 0);
        let proto = JS_NewObject(ctx);
        JS_SetPropertyFunctionList(
            ctx,
            proto,
            js_worker_proto_funcs.as_ptr(),
            js_worker_proto_funcs.len() as i32,
        );
        JS_SetClassProto(ctx, class, proto);
    }
    // Only the queue seam is under test here. Complete worker construction and
    // std/os initialization are covered by the independent tool-process gate.
    unsafe fn context() -> (*mut JSRuntime, *mut JSContext, *mut JSThreadState) {
        let rt = JS_NewRuntime();
        let ctx = JS_NewContext(rt);
        assert!(!rt.is_null() && !ctx.is_null());
        let state = libc::calloc(1, size_of::<JSThreadState>()).cast::<JSThreadState>();
        assert!(!state.is_null());
        for head in [
            ptr::addr_of_mut!((*state).os_rw_handlers),
            ptr::addr_of_mut!((*state).os_signal_handlers),
            ptr::addr_of_mut!((*state).os_timers),
            ptr::addr_of_mut!((*state).port_list),
            ptr::addr_of_mut!((*state).rejected_promise_list),
        ] {
            init_list_head(head);
        }
        (*state).next_timer_id = 1;
        JS_SetRuntimeOpaque(rt, state.cast());
        JS_SetSharedArrayBufferFunctions(
            rt,
            &JSSharedArrayBufferFunctions {
                sab_alloc: Some(js_sab_alloc),
                sab_free: Some(js_sab_free),
                sab_dup: Some(js_sab_dup),
                sab_opaque: ptr::null_mut(),
            },
        );
        register_class(ctx);
        (rt, ctx, state)
    }
    unsafe fn free_context(rt: *mut JSRuntime, ctx: *mut JSContext, state: *mut JSThreadState) {
        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
        libc::free(state.cast());
    }
    #[test]
    fn shared_buffer_refs_are_atomic_aligned_and_survive_other_threads() {
        unsafe {
            let p = js_sab_alloc(ptr::null_mut(), 64);
            assert!(!p.is_null());
            assert_eq!((p as usize) % 8, 0);
            *(p.cast::<u64>()) = 0x0123456789abcdef;
            let mut threads = Vec::new();
            for _ in 0..8 {
                js_sab_dup(ptr::null_mut(), p);
                let addr = p as usize;
                threads.push(std::thread::spawn(move || {
                    for _ in 0..10000 {
                        js_sab_dup(ptr::null_mut(), addr as *mut c_void);
                        js_sab_free(ptr::null_mut(), addr as *mut c_void);
                    }
                    assert_eq!(*(addr as *const u64), 0x0123456789abcdef);
                    js_sab_free(ptr::null_mut(), addr as *mut c_void);
                }));
            }
            for thread in threads {
                thread.join().unwrap();
            }
            let h = p
                .cast::<u8>()
                .sub(size_of::<JSSABHeader>())
                .cast::<JSSABHeader>();
            assert_eq!((*h).ref_count.load(Ordering::SeqCst), 1);
            js_sab_free(ptr::null_mut(), p);
        }
    }
    #[test]
    fn fifo_waker_and_reentrant_port_removal_preserve_serialized_cycles() {
        unsafe {
            let (ra, a, sa) = context();
            let (rb, b, sb) = context();
            let ab = js_new_message_pipe();
            let ba = js_new_message_pipe();
            let sender = js_worker_ctor_internal(a, JS_UNDEFINED, ba, ab);
            let receiver = js_worker_ctor_internal(b, JS_UNDEFINED, ab, ba);
            let global = JS_GetGlobalObject(b);
            JS_SetPropertyStr(b, global, c"receiver".as_ptr(), JS_DupValue(b, receiver));
            JS_FreeValue(b, global);
            let mut v=eval(b,"var received=[];(ev)=>{received.push(ev.data.i);if(ev.data.self!==ev.data)throw 'cycle';if(ev.data.i===2)receiver.onmessage=null}");
            assert_eq!(JS_IsException(js_worker_set_onmessage(b, receiver, v)), 0);
            JS_FreeValue(b, v);
            for i in 0..3 {
                v = eval(a, &format!("(()=>{{let v={{i:{i}}};v.self=v;return v}})()"));
                assert_eq!(
                    JS_IsException(js_worker_postMessage(a, sender, 1, &mut v)),
                    0
                );
                JS_FreeValue(a, v);
            }
            let worker = JS_GetOpaque(receiver, worker_class_id()).cast::<JSWorkerData>();
            let mut fd = libc::pollfd {
                fd: (*ab).waker.read_fd,
                events: libc::POLLIN,
                revents: 0,
            };
            assert_eq!(libc::poll(&mut fd, 1, 0), 1);
            for _ in 0..3 {
                assert_eq!(handle_posted_message(rb, b, (*worker).msg_handler), 1);
            }
            assert!((*worker).msg_handler.is_null());
            fd.revents = 0;
            assert_eq!(libc::poll(&mut fd, 1, 0), 0);
            v = eval(b, "JSON.stringify(received)");
            assert_eq!(text(b, v), "[0,1,2]");
            JS_FreeValue(b, v);
            JS_FreeValue(a, sender);
            JS_FreeValue(b, receiver);
            js_free_message_pipe(ab);
            js_free_message_pipe(ba);
            free_context(ra, a, sa);
            free_context(rb, b, sb);
        }
    }
    #[test]
    fn queued_sab_remains_live_after_sender_runtime_is_freed() {
        unsafe {
            let (ra, a, sa) = context();
            let (rb, b, sb) = context();
            let ab = js_new_message_pipe();
            let ba = js_new_message_pipe();
            let sender = js_worker_ctor_internal(a, JS_UNDEFINED, ba, ab);
            let receiver = js_worker_ctor_internal(b, JS_UNDEFINED, ab, ba);
            let f=eval(b,"ev=>{let v=ev.data;if(v.self!==v||v.a.buffer!==v.b.buffer)throw 'identity';v.a[0]+=5;globalThis.received=JSON.stringify([v.a[0],v.b[0],v.a.length])}");
            assert_eq!(JS_IsException(js_worker_set_onmessage(b, receiver, f)), 0);
            JS_FreeValue(b, f);
            let mut v=eval(a,"(()=>{let s=new SharedArrayBuffer(16),a=new Int32Array(s);a[0]=37;let v={a,b:new Int32Array(s)};v.self=v;return v})()");
            assert_eq!(
                JS_IsException(js_worker_postMessage(a, sender, 1, &mut v)),
                0
            );
            JS_FreeValue(a, v);
            JS_FreeValue(a, sender);
            free_context(ra, a, sa);
            let worker = JS_GetOpaque(receiver, worker_class_id()).cast::<JSWorkerData>();
            assert_eq!(handle_posted_message(rb, b, (*worker).msg_handler), 1);
            v = eval(b, "received");
            assert_eq!(text(b, v), "[42,42,4]");
            JS_FreeValue(b, v);
            assert_eq!(handle_posted_message(rb, b, (*worker).msg_handler), 0);
            JS_FreeValue(b, receiver);
            js_free_message_pipe(ab);
            js_free_message_pipe(ba);
            free_context(rb, b, sb);
        }
    }
}
