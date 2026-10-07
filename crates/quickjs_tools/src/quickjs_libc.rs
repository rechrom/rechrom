//! Host layer translated from quickjs-libc.c (Bellard/Gordon, MIT).
//! Original source spans are recorded per function. OS services stay in this
//! crate; the engine is accessed through its public QuickJS API.
use quickjs::{quickjs::*, quickjs_header::*};
use std::{
    alloc::{alloc, dealloc, Layout},
    ffi::{c_char, c_void, CStr},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    ptr,
};

#[cfg(unix)]
include!("quickjs_libc_loop.rs");
#[cfg(unix)]
include!("quickjs_libc_stdio.rs");
#[cfg(unix)]
include!("quickjs_libc_os.rs");
include!("quickjs_libc_loader.rs");
include!("host_args.rs");

#[cfg(windows)]
use super::quickjs_libc_windows::crt as libc;
#[cfg(windows)]
pub use super::quickjs_libc_windows::{
    get_time_ms, get_time_ns, js_console_log, js_init_module_os, js_init_module_std, js_load_file,
    js_os_now, js_print, js_std_add_helpers, js_std_await, js_std_dump_error, js_std_dump_error1,
    js_std_eval_binary, js_std_eval_binary_json_module, js_std_free_handlers, js_std_getenv,
    js_std_getenviron, js_std_init_handlers, js_std_loop, js_std_promise_rejection_tracker,
    js_std_setenv, js_std_unsetenv,
};
#[cfg(windows)]
pub use super::quickjs_libc_windows_worker::js_std_set_worker_new_context_func;
#[cfg(unix)]
pub use super::quickjs_libc_worker::js_std_set_worker_new_context_func;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
unsafe fn set_host_errno(error: i32) {
    *libc::__error() = error;
}
#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn set_host_errno(error: i32) {
    *libc::__errno_location() = error;
}
#[cfg(windows)]
unsafe fn set_host_errno(error: i32) {
    *libc::_errno() = error;
}

#[cfg(unix)]
fn filename_path(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}
#[cfg(not(unix))]
fn filename_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}
#[cfg(unix)]
fn path_bytes(path: &std::path::Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().to_vec()
}
#[cfg(not(unix))]
fn path_bytes(path: &std::path::Path) -> Vec<u8> {
    path.to_string_lossy().into_owned().into_bytes()
}

/// quickjs-libc.c:395-438. Read an exact seek-measured file into a terminated
/// buffer. For a nonnull context the caller frees it with js_free. The original
/// null-context malloc/free pair maps to Rust's allocator and the helper below.
#[cfg(not(windows))]
pub unsafe fn js_load_file(
    ctx: *mut JSContext,
    pbuf_len: *mut usize,
    filename: *const c_char,
) -> *mut u8 {
    let mut file = match File::open(filename_path(CStr::from_ptr(filename).to_bytes())) {
        Ok(file) => file,
        Err(_) => return ptr::null_mut(),
    };
    let length = match file.seek(SeekFrom::End(0)) {
        Ok(n) if n < isize::MAX as u64 => n as usize,
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "linux",
            target_os = "android"
        ))]
        Ok(n) if n == isize::MAX as u64 => {
            set_host_errno(libc::EISDIR);
            return ptr::null_mut();
        }
        _ => return ptr::null_mut(),
    };
    if file.seek(SeekFrom::Start(0)).is_err() {
        return ptr::null_mut();
    }
    let buffer = if ctx.is_null() {
        alloc(Layout::array::<u8>(length + 1).unwrap())
    } else {
        js_malloc(ctx, length + 1).cast::<u8>()
    };
    if buffer.is_null() {
        return ptr::null_mut();
    }
    if file
        .read_exact(std::slice::from_raw_parts_mut(buffer, length))
        .is_err()
    {
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "linux",
            target_os = "android"
        ))]
        set_host_errno(libc::EIO);
        js_free_file_buffer(ctx, buffer, length);
        return ptr::null_mut();
    }
    *buffer.add(length) = 0;
    *pbuf_len = length;
    buffer
}

/// Allocator pairing for the translated null-context file-loading path.
pub unsafe fn js_free_file_buffer(ctx: *mut JSContext, buffer: *mut u8, length: usize) {
    if buffer.is_null() {
        return;
    }
    if ctx.is_null() {
        dealloc(buffer, Layout::array::<u8>(length + 1).unwrap());
    } else {
        js_free(ctx, buffer.cast());
    }
}

// quickjs-libc.c:441-463.
pub unsafe fn js_loadScript(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let filename = JS_ToCString(ctx, *argv);
    if filename.is_null() {
        return JS_EXCEPTION;
    }
    let mut length = 0;
    let buffer = js_load_file(ctx, &mut length, filename);
    if buffer.is_null() {
        JS_ThrowReferenceError(
            ctx,
            format_args!(
                "could not load '{}'",
                CStr::from_ptr(filename).to_string_lossy()
            ),
        );
        JS_FreeCString(ctx, filename);
        return JS_EXCEPTION;
    }
    let value = JS_Eval(ctx, buffer.cast(), length, filename, JS_EVAL_TYPE_GLOBAL);
    js_free(ctx, buffer.cast());
    JS_FreeCString(ctx, filename);
    value
}

// quickjs-libc.c:466-484.
pub unsafe fn js_std_loadFile(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let filename = JS_ToCString(ctx, *argv);
    if filename.is_null() {
        return JS_EXCEPTION;
    }
    let mut length = 0;
    let buffer = js_load_file(ctx, &mut length, filename);
    JS_FreeCString(ctx, filename);
    if buffer.is_null() {
        return JS_NULL;
    }
    let value = JS_NewStringLen(ctx, buffer.cast(), length);
    js_free(ctx, buffer.cast());
    value
}

// quickjs-libc.c:548-599. Preserve unescaped file:// URLs, realpath failures,
// source PATH_MAX truncation and writable/configurable/enumerable properties.
pub unsafe fn js_module_set_import_meta(
    ctx: *mut JSContext,
    func_val: JSValueConst,
    use_realpath: JS_BOOL,
    is_main: JS_BOOL,
) -> i32 {
    assert_eq!(JS_VALUE_GET_TAG(func_val), JS_TAG_MODULE);
    let module = JS_VALUE_GET_PTR(func_val).cast::<JSModuleDef>();
    let atom = JS_GetModuleName(ctx, module);
    let name = JS_AtomToCString(ctx, atom);
    JS_FreeAtom(ctx, atom);
    if name.is_null() {
        return -1;
    }
    let original = CStr::from_ptr(name).to_bytes();
    let mut url = if original.contains(&b':') {
        original.to_vec()
    } else {
        let mut bytes = b"file://".to_vec();
        #[cfg(not(windows))]
        if use_realpath != 0 {
            match std::fs::canonicalize(filename_path(original)) {
                Ok(path) => bytes.extend_from_slice(&path_bytes(&path)),
                Err(_) => {
                    JS_ThrowTypeError(ctx, c"realpath failure".as_ptr());
                    JS_FreeCString(ctx, name);
                    return -1;
                }
            }
        } else {
            bytes.extend_from_slice(original);
        }
        #[cfg(windows)]
        {
            let _ = use_realpath;
            bytes.extend_from_slice(original);
        }
        bytes
    };
    #[cfg(unix)]
    const PATH_MAX: usize = libc::PATH_MAX as usize;
    #[cfg(windows)]
    const PATH_MAX: usize = super::quickjs_libc_windows::WIN_PATH_MAX;
    url.truncate(PATH_MAX + 16 - 1);
    JS_FreeCString(ctx, name);
    let meta = JS_GetImportMeta(ctx, module);
    if JS_IsException(meta) != 0 {
        return -1;
    }
    JS_DefinePropertyValueStr(
        ctx,
        meta,
        c"url".as_ptr(),
        JS_NewStringLen(ctx, url.as_ptr().cast(), url.len()),
        JS_PROP_C_W_E,
    );
    JS_DefinePropertyValueStr(
        ctx,
        meta,
        c"main".as_ptr(),
        JS_NewBool(ctx, is_main),
        JS_PROP_C_W_E,
    );
    JS_FreeValue(ctx, meta);
    0
}

// quickjs-libc.c:601-607.
unsafe fn json_module_init(ctx: *mut JSContext, module: *mut JSModuleDef) -> i32 {
    let value = JS_GetModulePrivateValue(ctx, module);
    JS_SetModuleExport(ctx, module, c"default".as_ptr(), value);
    0
}
// quickjs-libc.c:609-621.
pub unsafe fn create_json_module(
    ctx: *mut JSContext,
    name: *const c_char,
    value: JSValue,
) -> *mut JSModuleDef {
    let module = JS_NewCModule(ctx, name, Some(json_module_init));
    if module.is_null() {
        JS_FreeValue(ctx, value);
        return ptr::null_mut();
    }
    JS_AddModuleExport(ctx, module, c"default".as_ptr());
    JS_SetModulePrivateValue(ctx, module, value);
    module
}

// quickjs-libc.c:625-653. Inspect only enumerable string keys, never their values.
pub unsafe fn js_module_check_attributes(
    ctx: *mut JSContext,
    _opaque: *mut c_void,
    attributes: JSValueConst,
) -> i32 {
    let mut table = ptr::null_mut();
    let mut count = 0;
    if JS_GetOwnPropertyNames(
        ctx,
        &mut table,
        &mut count,
        attributes,
        JS_GPN_ENUM_ONLY | JS_GPN_STRING_MASK,
    ) != 0
    {
        return -1;
    }
    let mut result = 0;
    for i in 0..count {
        let mut length = 0;
        let text = JS_AtomToCStringLen(ctx, &mut length, (*table.add(i as usize)).atom);
        if text.is_null() {
            result = -1;
            break;
        }
        if std::slice::from_raw_parts(text.cast::<u8>(), length) != b"type" {
            JS_ThrowTypeError(
                ctx,
                format_args!(
                    "import attribute '{}' is not supported",
                    CStr::from_ptr(text).to_string_lossy()
                ),
            );
            result = -1;
        }
        JS_FreeCString(ctx, text);
        if result != 0 {
            break;
        }
    }
    JS_FreePropertyEnum(ctx, table, count);
    result
}

// quickjs-libc.c:656-682. Retain original property access and conversion order.
pub unsafe fn js_module_test_json(ctx: *mut JSContext, attributes: JSValueConst) -> i32 {
    if JS_IsUndefined(attributes) != 0 {
        return 0;
    }
    let value = JS_GetPropertyStr(ctx, attributes, c"type".as_ptr());
    if JS_IsString(value) == 0 {
        return 0;
    }
    let mut length = 0;
    let text = JS_ToCStringLen(ctx, &mut length, value);
    JS_FreeValue(ctx, value);
    if text.is_null() {
        return 0;
    }
    let result = match std::slice::from_raw_parts(text.cast::<u8>(), length) {
        b"json" => 1,
        b"json5" => 2,
        _ => 0,
    };
    JS_FreeCString(ctx, text);
    result
}

// quickjs-libc.c:4212-4216. Share libc's stream with std.FILE/printf so
// buffering and ordering follow the original host rather than Rust stdout.
#[cfg(unix)]
pub unsafe fn js_std_dump_error1(ctx: *mut JSContext, error: JSValueConst) {
    JS_PrintValue(
        ctx,
        Some(stdio_print_value_write),
        stdio_stderr().cast(),
        error,
        ptr::null(),
    );
    libc::fputc(b'\n' as i32, stdio_stderr());
}
// quickjs-libc.c:4218-4225.
#[cfg(unix)]
pub unsafe fn js_std_dump_error(ctx: *mut JSContext) {
    let error = JS_GetException(ctx);
    js_std_dump_error1(ctx, error);
    JS_FreeValue(ctx, error);
}

// quickjs-libc.c:737-745.
pub unsafe fn js_std_exit(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let mut status = 0;
    if JS_ToInt32(ctx, &mut status, *argv) != 0 {
        status = -1;
    }
    std::process::exit(status);
}

// quickjs-libc.c:747-760. POSIX environment bytes are not assumed to be UTF-8.
#[cfg(unix)]
pub unsafe fn js_std_getenv(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    let value = libc::getenv(name);
    JS_FreeCString(ctx, name);
    if value.is_null() {
        JS_UNDEFINED
    } else {
        JS_NewString(ctx, value)
    }
}

// quickjs-libc.c:784-800. Upstream deliberately ignores setenv's return value.
#[cfg(unix)]
pub unsafe fn js_std_setenv(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    let value = JS_ToCString(ctx, *argv.add(1));
    if value.is_null() {
        JS_FreeCString(ctx, name);
        return JS_EXCEPTION;
    }
    libc::setenv(name, value, 1);
    JS_FreeCString(ctx, name);
    JS_FreeCString(ctx, value);
    JS_UNDEFINED
}

// quickjs-libc.c:802-812.
#[cfg(unix)]
pub unsafe fn js_std_unsetenv(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let name = JS_ToCString(ctx, *argv);
    if name.is_null() {
        return JS_EXCEPTION;
    }
    libc::unsetenv(name);
    JS_FreeCString(ctx, name);
    JS_UNDEFINED
}

// quickjs-libc.c:816-851. Atom/value ownership and DefineProperty flags remain
// the original API calls; enumerate the host's current byte-valued environment.
#[cfg(unix)]
pub unsafe fn js_std_getenviron(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    use std::os::unix::ffi::OsStrExt;
    let object = JS_NewObject(ctx);
    if JS_IsException(object) != 0 {
        return JS_EXCEPTION;
    }
    for (name, value) in std::env::vars_os() {
        let name = name.as_bytes();
        let value = value.as_bytes();
        let atom = JS_NewAtomLen(ctx, name.as_ptr().cast(), name.len());
        if atom == 0 {
            JS_FreeValue(ctx, object);
            return JS_EXCEPTION;
        }
        let result = JS_DefinePropertyValue(
            ctx,
            object,
            atom,
            JS_NewStringLen(ctx, value.as_ptr().cast(), value.len()),
            JS_PROP_C_W_E,
        );
        JS_FreeAtom(ctx, atom);
        if result < 0 {
            JS_FreeValue(ctx, object);
            return JS_EXCEPTION;
        }
    }
    object
}

// quickjs-libc.c:853-858.
pub unsafe fn js_std_gc(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    JS_RunGC(JS_GetRuntime(ctx));
    JS_UNDEFINED
}

// quickjs-libc.c:865-878.
pub unsafe fn get_bool_option(
    ctx: *mut JSContext,
    result: &mut i32,
    object: JSValueConst,
    name: *const c_char,
) -> i32 {
    let value = JS_GetPropertyStr(ctx, object, name);
    if JS_IsException(value) != 0 {
        return -1;
    }
    if JS_IsUndefined(value) == 0 {
        *result = JS_ToBool(ctx, value);
    }
    JS_FreeValue(ctx, value);
    0
}

// Platform-selected clocks and the original portable POSIX fallback.
#[cfg(unix)]
include!("quickjs_libc_clock.rs");

// quickjs-libc.c:2162-2166.
#[cfg(unix)]
pub unsafe fn js_os_now(
    ctx: *mut JSContext,
    _this: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    JS_NewFloat64(ctx, get_time_ns() as f64 / 1e6)
}

// quickjs-libc.c:4063-4087. Preserve engine formatting of nonstring values.
#[cfg(unix)]
pub unsafe fn js_print(
    ctx: *mut JSContext,
    _this: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    for i in 0..argc {
        if i != 0 {
            libc::fputc(b' ' as i32, stdio_stdout());
        }
        let value = *argv.add(i as usize);
        if JS_IsString(value) != 0 {
            let mut length = 0;
            let text = JS_ToCStringLen(ctx, &mut length, value);
            if text.is_null() {
                return JS_EXCEPTION;
            }
            libc::fwrite(text.cast(), 1, length, stdio_stdout());
            JS_FreeCString(ctx, text);
        } else {
            JS_PrintValue(
                ctx,
                Some(stdio_print_value_write),
                stdio_stdout().cast(),
                value,
                ptr::null(),
            );
        }
    }
    libc::fputc(b'\n' as i32, stdio_stdout());
    JS_UNDEFINED
}
// quickjs-libc.c:4089-4096.
#[cfg(unix)]
pub unsafe fn js_console_log(
    ctx: *mut JSContext,
    this: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let value = js_print(ctx, this, argc, argv);
    libc::fflush(stdio_stdout());
    value
}

// quickjs-libc.c:4098-4131 (Linux/macOS host clock configuration).
#[cfg(unix)]
pub unsafe fn js_std_add_helpers(ctx: *mut JSContext, argc: i32, argv: *mut *mut c_char) {
    let global = JS_GetGlobalObject(ctx);
    let console = JS_NewObject(ctx);
    JS_SetPropertyStr(
        ctx,
        console,
        c"log".as_ptr(),
        JS_NewCFunction(ctx, Some(js_console_log), c"log".as_ptr(), 1),
    );
    JS_SetPropertyStr(ctx, global, c"console".as_ptr(), console);
    let performance = JS_NewObject(ctx);
    JS_SetPropertyStr(
        ctx,
        performance,
        c"now".as_ptr(),
        JS_NewCFunction(ctx, Some(js_os_now), c"now".as_ptr(), 0),
    );
    JS_SetPropertyStr(ctx, global, c"performance".as_ptr(), performance);
    if argc >= 0 {
        let args = JS_NewArray(ctx);
        for i in 0..argc {
            JS_SetPropertyUint32(
                ctx,
                args,
                i as u32,
                JS_NewString(ctx, *argv.add(i as usize)),
            );
        }
        JS_SetPropertyStr(ctx, global, c"scriptArgs".as_ptr(), args);
    }
    JS_SetPropertyStr(
        ctx,
        global,
        c"print".as_ptr(),
        JS_NewCFunction(ctx, Some(js_print), c"print".as_ptr(), 1),
    );
    JS_SetPropertyStr(
        ctx,
        global,
        c"__loadScript".as_ptr(),
        JS_NewCFunction(ctx, Some(js_loadScript), c"__loadScript".as_ptr(), 1),
    );
    JS_FreeValue(ctx, global);
}
