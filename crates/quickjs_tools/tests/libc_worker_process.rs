//! The complete production tool host, including real loader and detached Worker.
use quickjs::{quickjs::*, quickjs_header::*};
use quickjs_tools::{quickjs_libc::*, quickjs_libc_worker::js_std_set_worker_new_context_func};
use std::{ffi::CString, ptr};
unsafe fn new_context(rt: *mut JSRuntime) -> *mut JSContext {
    let ctx = JS_NewContext(rt);
    if ctx.is_null() {
        return ctx;
    }
    js_init_module_std(ctx, c"std".as_ptr());
    js_init_module_os(ctx, c"os".as_ptr());
    ctx
}
fn main() {
    unsafe {
        let name = CString::new(std::env::args().nth(1).expect("parent module path")).unwrap();
        let rt = JS_NewRuntime();
        assert!(!rt.is_null());
        js_std_set_worker_new_context_func(Some(new_context));
        js_std_init_handlers(rt);
        let ctx = new_context(rt);
        assert!(!ctx.is_null());
        JS_SetModuleLoaderFunc2(
            rt,
            None,
            Some(js_module_loader),
            Some(js_module_check_attributes),
            ptr::null_mut(),
        );
        js_std_add_helpers(ctx, 0, ptr::null_mut());
        let mut size = 0;
        let source = js_load_file(ctx, &mut size, name.as_ptr());
        assert!(!source.is_null());
        let value = JS_Eval(ctx, source.cast(), size, name.as_ptr(), JS_EVAL_TYPE_MODULE);
        js_free(ctx, source.cast());
        let value = js_std_await(ctx, value);
        if JS_IsException(value) != 0 {
            js_std_dump_error(ctx);
            std::process::exit(4);
        }
        JS_FreeValue(ctx, value);
        js_std_loop(ctx);
        JS_FreeContext(ctx);
        js_std_free_handlers(rt);
        JS_FreeRuntime(rt);
    }
}
