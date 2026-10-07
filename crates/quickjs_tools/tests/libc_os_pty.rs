//! Actual production host with real POSIX tty, fork and signal services.
use quickjs::{quickjs::*, quickjs_header::*};
use quickjs_tools::{quickjs_libc::*, quickjs_libc_state::*};
use std::{
    ffi::{c_void, CString},
    mem::{align_of, offset_of, size_of},
    ptr,
};
include!("../../quickjs/tests/quickjs_diagnostic_allocator.rs");
static mut LIVE: usize = 0;
static mut FAIL_AFTER: usize = usize::MAX;
unsafe fn gate_fail() -> bool {
    if FAIL_AFTER == usize::MAX {
        return false;
    }
    FAIL_AFTER -= 1;
    if FAIL_AFTER == 0 {
        FAIL_AFTER = usize::MAX;
        true
    } else {
        false
    }
}
unsafe fn gate_alloc(s: *mut JSMallocState, n: usize) -> *mut c_void {
    if gate_fail() {
        return ptr::null_mut();
    }
    let p = diagnostic_malloc(s, n);
    if !p.is_null() {
        LIVE += 1;
    }
    p
}
unsafe fn gate_free(s: *mut JSMallocState, p: *mut c_void) {
    if !p.is_null() {
        LIVE -= 1;
    }
    diagnostic_free(s, p)
}
unsafe fn gate_realloc(s: *mut JSMallocState, p: *mut c_void, n: usize) -> *mut c_void {
    if p.is_null() {
        return gate_alloc(s, n);
    }
    if n == 0 {
        gate_free(s, p);
        return ptr::null_mut();
    }
    if gate_fail() {
        return ptr::null_mut();
    }
    diagnostic_realloc(s, p, n)
}
static GATE_ALLOCATOR: JSMallocFunctions = JSMallocFunctions {
    js_malloc: Some(gate_alloc),
    js_free: Some(gate_free),
    js_realloc: Some(gate_realloc),
    js_malloc_usable_size: Some(diagnostic_capacity),
};
macro_rules! layout {
    ($t:ty,$f:ident) => {
        println!(
            "layout:{}:{}:{}:{}",
            stringify!($t),
            size_of::<$t>(),
            align_of::<$t>(),
            offset_of!($t, $f)
        );
    };
}
unsafe fn oom_gates() {
    let source=c"(()=>{let env={};for(let i=0;i<80;i++)env['host_key_'+i]={toString(){return 'value_'+i}};return [['/not-a-quickjs-file','hello'],{env,file:'/not-a-quickjs-file',cwd:'/tmp',usePath:false}]})()";
    for i in 1..=150 {
        let rt = JS_NewRuntime2(&GATE_ALLOCATOR, ptr::null_mut());
        js_std_init_handlers(rt);
        let ctx = JS_NewContext(rt);
        assert!(!ctx.is_null());
        let pair = JS_Eval(
            ctx,
            source.as_ptr(),
            source.to_bytes().len(),
            c"exec-oom.js".as_ptr(),
            JS_EVAL_TYPE_GLOBAL,
        );
        assert_eq!(JS_IsException(pair), 0);
        let mut args = [
            JS_GetPropertyUint32(ctx, pair, 0),
            JS_GetPropertyUint32(ctx, pair, 1),
        ];
        FAIL_AFTER = i;
        let v = js_os_exec(ctx, JS_UNDEFINED, 2, args.as_mut_ptr());
        FAIL_AFTER = usize::MAX;
        let exception = JS_IsException(v);
        let mut status = -999;
        if exception != 0 {
            let e = JS_GetException(ctx);
            JS_FreeValue(ctx, e);
        } else {
            JS_ToInt32(ctx, &mut status, v);
        }
        JS_FreeValue(ctx, v);
        JS_FreeValue(ctx, args[0]);
        JS_FreeValue(ctx, args[1]);
        JS_FreeValue(ctx, pair);
        JS_FreeContext(ctx);
        js_std_free_handlers(rt);
        JS_FreeRuntime(rt);
        let live = LIVE;
        println!("oom:{i}:{exception}:{status}:{live}");
        assert_eq!(live, 0);
    }
}
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--exec-oom") {
        unsafe {
            oom_gates();
        }
        return;
    }
    unsafe {
        layout!(JSOSRWHandler, rw_func);
        layout!(JSOSSignalHandler, func);
        layout!(JSOSTimer, func);
        layout!(JSWorkerMessage, sab_tab_len);
        layout!(JSWaker, write_fd);
        layout!(JSWorkerMessagePipe, waker);
        layout!(JSWorkerMessageHandler, poll_fd_index);
        layout!(JSRejectedPromiseEntry, reason);
        layout!(JSThreadState, poll_fds_size);
        let rt = JS_NewRuntime2(&GATE_ALLOCATOR, ptr::null_mut());
        assert!(!rt.is_null());
        js_std_init_handlers(rt);
        let ctx = JS_NewContext(rt);
        assert!(!ctx.is_null());
        js_init_module_std(ctx, c"std".as_ptr());
        js_init_module_os(ctx, c"os".as_ptr());
        JS_SetModuleLoaderFunc2(
            rt,
            None,
            Some(js_module_loader),
            Some(js_module_check_attributes),
            ptr::null_mut(),
        );
        js_std_add_helpers(ctx, 0, ptr::null_mut());
        let name = CString::new(std::env::args().nth(1).unwrap()).unwrap();
        let mut len = 0;
        let source = js_load_file(ctx, &mut len, name.as_ptr());
        assert!(!source.is_null());
        let value = JS_Eval(ctx, source.cast(), len, name.as_ptr(), JS_EVAL_TYPE_MODULE);
        js_free(ctx, source.cast());
        let value = js_std_await(ctx, value);
        if JS_IsException(value) != 0 {
            js_std_dump_error(ctx);
            std::process::exit(4);
        }
        JS_FreeValue(ctx, value);
        js_std_loop(ctx);
        let mut tty: libc::termios = std::mem::zeroed();
        assert_eq!(libc::tcgetattr(0, &mut tty), 0);
        println!(
            "raw:{}:{}:{}:{}:{}:{}:{}",
            (tty.c_lflag & libc::ECHO != 0) as u8,
            (tty.c_lflag & libc::ICANON != 0) as u8,
            (tty.c_lflag & libc::ISIG != 0) as u8,
            (tty.c_oflag & libc::OPOST != 0) as u8,
            (tty.c_iflag & libc::ICRNL != 0) as u8,
            tty.c_cc[libc::VMIN],
            tty.c_cc[libc::VTIME]
        );
        JS_FreeContext(ctx);
        js_std_free_handlers(rt);
        JS_FreeRuntime(rt);
        let live = LIVE;
        println!("live:{live}");
        assert_eq!(live, 0);
    }
}
