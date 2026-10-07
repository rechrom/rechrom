use quickjs::{quickjs::*, quickjs_header::*};
use quickjs_tools::run_test262::*;
fn main() {
    unsafe {
        let tls = Box::into_raw(Box::new(std::mem::zeroed::<ThreadLocalStorage>()));
        init_thread_local_storage(tls);
        let rt = JS_NewRuntime();
        JS_SetRuntimeOpaque(rt, tls.cast());
        JS_SetCanBlock(rt, 1);
        let ctx = JS_NewContext(rt);
        assert!(!ctx.is_null());
        set_test262_outfile({
            extern "C" {
                static mut __stdoutp: *mut std::ffi::c_void;
            }
            __stdoutp.cast()
        });
        add_helpers(ctx);
        let filename = std::ffi::CString::new(std::env::args().nth(1).unwrap()).unwrap();
        let mut len = 0;
        let buf = quickjs_tools::quickjs_libc::js_load_file(ctx, &mut len, filename.as_ptr());
        assert!(!buf.is_null());
        let value = JS_Eval(ctx, buf.cast(), len, filename.as_ptr(), JS_EVAL_TYPE_GLOBAL);
        js_free(ctx, buf.cast());
        let failed = JS_IsException(value) != 0;
        if failed {
            quickjs_tools::quickjs_libc::js_std_dump_error(ctx);
        }
        JS_FreeValue(ctx, value);
        js_agent_free(ctx);
        {
            extern "C" {
                fn printf(format: *const std::ffi::c_char, ...) -> i32;
            }
            printf(c"async=%d\n".as_ptr(), (*tls).async_done);
        }
        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
        free_thread_local_storage(tls);
        drop(Box::from_raw(tls));
        if failed {
            std::process::exit(4);
        }
    }
}
