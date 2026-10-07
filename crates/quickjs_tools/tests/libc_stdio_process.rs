use quickjs::{quickjs::*, quickjs_header::*};
use quickjs_tools::quickjs_libc::*;
use std::{ffi::CString, ptr};
fn main() {
    unsafe {
        let name = CString::new(std::env::args().nth(1).expect("module path")).unwrap();
        let rt = JS_NewRuntime();
        assert!(!rt.is_null());
        js_std_init_handlers(rt);
        let ctx = JS_NewContext(rt);
        assert!(!ctx.is_null());
        js_init_module_std(ctx, c"std".as_ptr());
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
