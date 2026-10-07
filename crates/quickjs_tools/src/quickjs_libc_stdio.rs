// quickjs-libc.c std/FILE/printf host functions (included in quickjs_libc).
// Native stdio/errno/popen/curl are tool services, never engine dependencies.
use quickjs::{
    cutils::{dbuf_free, dbuf_init2, dbuf_put, dbuf_putstr, unicode_from_utf8, unicode_to_utf8},
    cutils_header::{dbuf_error, DynBuf, UTF8_CHAR_LEN_MAX},
};
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
extern "C" {
    #[link_name = "__stdinp"]
    static mut STDIO_HOST_IN: *mut libc::FILE;
    #[link_name = "__stdoutp"]
    static mut STDIO_HOST_OUT: *mut libc::FILE;
    #[link_name = "__stderrp"]
    static mut STDIO_HOST_ERR: *mut libc::FILE;
}
#[cfg(any(target_os = "linux", target_os = "android"))]
extern "C" {
    #[link_name = "stdin"]
    static mut STDIO_HOST_IN: *mut libc::FILE;
    #[link_name = "stdout"]
    static mut STDIO_HOST_OUT: *mut libc::FILE;
    #[link_name = "stderr"]
    static mut STDIO_HOST_ERR: *mut libc::FILE;
}
unsafe fn stdio_stdin() -> *mut libc::FILE {
    STDIO_HOST_IN
}
unsafe fn stdio_stdout() -> *mut libc::FILE {
    STDIO_HOST_OUT
}
unsafe fn stdio_stderr() -> *mut libc::FILE {
    STDIO_HOST_ERR
}
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
unsafe fn stdio_errno_pointer() -> *mut i32 {
    libc::__error()
}
#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn stdio_errno_pointer() -> *mut i32 {
    libc::__errno_location()
}
// C:949..954. Shared return/errno adapter used by std and OS callbacks.
pub unsafe fn js_get_errno(ret: isize) -> isize {
    if ret == -1 {
        -(*stdio_errno_pointer() as isize)
    } else {
        ret
    }
}
unsafe fn stdio_bool_option(
    ctx: *mut JSContext,
    result: *mut i32,
    obj: JSValueConst,
    name: *const c_char,
) -> i32 {
    get_bool_option(ctx, &mut *result, obj, name)
}
unsafe fn stdio_realloc(opaque: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    js_realloc_rt(opaque.cast(), p, size)
}
// C:166..178.
unsafe fn js_std_dbuf_init(ctx: *mut JSContext, s: *mut DynBuf) {
    dbuf_init2(s, JS_GetRuntime(ctx).cast(), Some(stdio_realloc));
}
unsafe fn stdio_dbuf_putc(s: *mut DynBuf, c: u8) -> i32 {
    quickjs::cutils::__dbuf_putc(s, c as u8)
}
unsafe fn stdio_dbuf_printf_string(
    s: *mut DynBuf,
    fmt: *const c_char,
    value: *const c_char,
) -> i32 {
    quickjs::cutils::dbuf_printf(s, |out, cap| libc::snprintf(out.cast(), cap, fmt, value))
}
#[repr(C)]
struct JSSTDFile {
    f: *mut libc::FILE,
    close_in_finalizer: i32,
    is_popen: i32,
}
static STDIO_FILE_CLASS_ID: std::sync::OnceLock<JSClassID> = std::sync::OnceLock::new();
unsafe fn stdio_file_class_id() -> JSClassID {
    *STDIO_FILE_CLASS_ID.get_or_init(|| {
        let mut id = 0;
        JS_NewClassID(&mut id);
        id
    })
}
// C:181..393. Typed snprintf closures replace only the C variadic function
// pointer; DynBuf's actual allocator and 128-byte formatting growth stay exact.
pub unsafe fn js_printf_internal(
    ctx: *mut JSContext,
    argc: i32,
    argv: *mut JSValueConst,
    fp: *mut libc::FILE,
) -> JSValue {
    let mut dbuf = DynBuf::default();
    js_std_dbuf_init(ctx, &mut dbuf);
    let mut fmt_str = ptr::null();
    let success = (|| -> Result<(), ()> {
        if argc > 0 {
            let mut fmt_len = 0;
            fmt_str = JS_ToCStringLen(ctx, &mut fmt_len, *argv);
            if fmt_str.is_null() {
                return Err(());
            }
            let mut fmt = fmt_str.cast::<u8>();
            let end = fmt.add(fmt_len);
            let mut arg = 1;
            while fmt < end {
                let literal = fmt;
                while fmt < end && *fmt != b'%' {
                    fmt = fmt.add(1);
                }
                dbuf_put(&mut dbuf, literal, fmt.offset_from(literal) as usize);
                if fmt >= end {
                    break;
                }
                let mut f = [0i8; 32];
                let mut q = 0;
                macro_rules! invalid {
                    () => {{
                        JS_ThrowTypeError(
                            ctx,
                            c"invalid conversion specifier in format string".as_ptr(),
                        );
                        return Err(());
                    }};
                }
                macro_rules! missing {
                    () => {{
                        JS_ThrowReferenceError(
                            ctx,
                            c"missing argument for conversion specifier".as_ptr(),
                        );
                        return Err(());
                    }};
                }
                macro_rules! push {
                    ($ch:expr) => {{
                        if q >= f.len() - 1 {
                            invalid!();
                        }
                        f[q] = $ch as i8;
                        q += 1;
                    }};
                }
                push!(*fmt);
                fmt = fmt.add(1);
                loop {
                    let c = *fmt;
                    if matches!(c, b'0' | b'#' | b'+' | b'-' | b' ' | b'\'') {
                        push!(c);
                        fmt = fmt.add(1);
                    } else {
                        break;
                    }
                }
                if *fmt == b'*' {
                    if arg >= argc {
                        missing!();
                    }
                    let mut v = 0;
                    if JS_ToInt32(ctx, &mut v, *argv.offset(arg as isize)) != 0 {
                        return Err(());
                    }
                    arg += 1;
                    let len = libc::snprintf(f.as_mut_ptr().add(q), f.len() - q, c"%d".as_ptr(), v);
                    q += len.max(0) as usize;
                    fmt = fmt.add(1);
                } else {
                    while (*fmt).is_ascii_digit() {
                        push!(*fmt);
                        fmt = fmt.add(1);
                    }
                }
                if *fmt == b'.' {
                    push!(*fmt);
                    fmt = fmt.add(1);
                    if *fmt == b'*' {
                        if arg >= argc {
                            missing!();
                        }
                        let mut v = 0;
                        if JS_ToInt32(ctx, &mut v, *argv.offset(arg as isize)) != 0 {
                            return Err(());
                        }
                        arg += 1;
                        let len =
                            libc::snprintf(f.as_mut_ptr().add(q), f.len() - q, c"%d".as_ptr(), v);
                        q += len.max(0) as usize;
                        fmt = fmt.add(1);
                    } else {
                        while (*fmt).is_ascii_digit() {
                            push!(*fmt);
                            fmt = fmt.add(1);
                        }
                    }
                }
                let wide = *fmt == b'l';
                if wide {
                    fmt = fmt.add(1);
                }
                let c = *fmt;
                fmt = fmt.add(1);
                push!(c);
                f[q] = 0;
                match c {
                    b'c' => {
                        if arg >= argc {
                            missing!();
                        }
                        let v = *argv.offset(arg as isize);
                        arg += 1;
                        let mut value = 0;
                        if JS_IsString(v) != 0 {
                            let text = JS_ToCString(ctx, v);
                            if text.is_null() {
                                return Err(());
                            }
                            let mut next = ptr::null();
                            value =
                                unicode_from_utf8(text.cast(), UTF8_CHAR_LEN_MAX as i32, &mut next);
                            JS_FreeCString(ctx, text);
                        } else if JS_ToInt32(ctx, &mut value, v) != 0 {
                            return Err(());
                        }
                        if value as u32 > 0x10ffff {
                            value = 0xfffd;
                        }
                        let mut bytes = [0u8; UTF8_CHAR_LEN_MAX + 1];
                        let count = unicode_to_utf8(bytes.as_mut_ptr(), value as u32);
                        dbuf_put(&mut dbuf, bytes.as_ptr(), count as usize);
                    }
                    b'd' | b'i' | b'o' | b'u' | b'x' | b'X' => {
                        if arg >= argc {
                            missing!();
                        }
                        let mut value = 0i64;
                        if JS_ToInt64Ext(ctx, &mut value, *argv.offset(arg as isize)) != 0 {
                            return Err(());
                        }
                        arg += 1;
                        if wide {
                            if q >= f.len() - 2 {
                                invalid!();
                            }
                            f[q + 1] = f[q - 1];
                            f[q - 1] = b'l' as i8;
                            f[q] = b'l' as i8;
                            f[q + 2] = 0;
                            quickjs::cutils::dbuf_printf(&mut dbuf, |out, cap| {
                                libc::snprintf(
                                    out.cast(),
                                    cap,
                                    f.as_ptr(),
                                    value as libc::c_longlong,
                                )
                            });
                        } else {
                            quickjs::cutils::dbuf_printf(&mut dbuf, |out, cap| {
                                libc::snprintf(out.cast(), cap, f.as_ptr(), value as i32)
                            });
                        }
                    }
                    b's' => {
                        if arg >= argc {
                            missing!();
                        }
                        let value = JS_ToCString(ctx, *argv.offset(arg as isize));
                        arg += 1;
                        if value.is_null() {
                            return Err(());
                        }
                        stdio_dbuf_printf_string(&mut dbuf, f.as_ptr(), value);
                        JS_FreeCString(ctx, value);
                    }
                    b'e' | b'f' | b'g' | b'a' | b'E' | b'F' | b'G' | b'A' => {
                        if arg >= argc {
                            missing!();
                        }
                        let mut value = 0.0;
                        if JS_ToFloat64(ctx, &mut value, *argv.offset(arg as isize)) != 0 {
                            return Err(());
                        }
                        arg += 1;
                        quickjs::cutils::dbuf_printf(&mut dbuf, |out, cap| {
                            libc::snprintf(out.cast(), cap, f.as_ptr(), value)
                        });
                    }
                    b'%' => {
                        stdio_dbuf_putc(&mut dbuf, b'%');
                    }
                    _ => {
                        invalid!();
                    }
                }
            }
        }
        Ok(())
    })();
    JS_FreeCString(ctx, fmt_str);
    let result = if success.is_err() {
        JS_EXCEPTION
    } else if dbuf.error != 0 {
        JS_ThrowOutOfMemory(ctx)
    } else if !fp.is_null() {
        JS_NewInt32(ctx, libc::fwrite(dbuf.buf.cast(), 1, dbuf.size, fp) as i32)
    } else {
        JS_NewStringLen(ctx, dbuf.buf.cast(), dbuf.size)
    };
    dbuf_free(&mut dbuf);
    result
}
// C:879..930. Global eval options and interrupt-handler recursion state.
pub unsafe fn js_evalScript(
    ctx: *mut JSContext,
    _this: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let rt = JS_GetRuntime(ctx);
    let ts = JS_GetRuntimeOpaque(rt).cast::<JSThreadState>();
    let mut barrier = 0;
    let mut is_async = 0;
    if argc >= 2 {
        if get_bool_option(
            ctx,
            &mut barrier,
            *argv.add(1),
            c"backtrace_barrier".as_ptr(),
        ) != 0
            || get_bool_option(ctx, &mut is_async, *argv.add(1), c"async".as_ptr()) != 0
        {
            return JS_EXCEPTION;
        }
    }
    let mut len = 0;
    let source = JS_ToCStringLen(ctx, &mut len, *argv);
    if source.is_null() {
        return JS_EXCEPTION;
    }
    if (*ts).recv_pipe.is_null() {
        (*ts).eval_script_recurse += 1;
        if (*ts).eval_script_recurse == 1 {
            JS_SetInterruptHandler(rt, Some(interrupt_handler), ptr::null_mut());
        }
    }
    let flags = JS_EVAL_TYPE_GLOBAL
        | if barrier != 0 {
            JS_EVAL_FLAG_BACKTRACE_BARRIER
        } else {
            0
        }
        | if is_async != 0 { JS_EVAL_FLAG_ASYNC } else { 0 };
    let value = JS_Eval(ctx, source, len, c"<evalScript>".as_ptr(), flags);
    JS_FreeCString(ctx, source);
    if (*ts).recv_pipe.is_null() {
        (*ts).eval_script_recurse -= 1;
        if (*ts).eval_script_recurse == 0 {
            JS_SetInterruptHandler(rt, None, ptr::null_mut());
            OS_PENDING_SIGNALS
                .fetch_and(!(1u64 << libc::SIGINT), std::sync::atomic::Ordering::SeqCst);
            if JS_IsException(value) != 0 {
                JS_SetUncatchableException(ctx, 0);
            }
        }
    }
    value
}
// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:937. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_finalizer(mut rt: *mut JSRuntime, mut val: JSValue) -> () {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut s: *mut JSSTDFile = core::mem::zeroed();
    let mut vm_block: usize = 7;
    loop {
        match vm_block {
            // C line ?
            0 => {
                return ();
            }
            // C line 947
            1 => {
                let _ = js_free_rt(rt, ((s) as *mut c_void));
                vm_block = 0;
                continue;
            }
            // C line 943
            2 => {
                let _ = libc::pclose((*(s)).f);
                vm_block = 1;
                continue;
            }
            // C line 945
            3 => {
                let _ = libc::fclose((*(s)).f);
                vm_block = 1;
                continue;
            }
            // C line 942
            4 => {
                vm_block = if ((*(s)).is_popen) != 0 { 2 } else { 3 };
                continue;
            }
            // C line 941
            5 => {
                vm_block = if (((!((*(s)).f).is_null()) && (((*(s)).close_in_finalizer) != 0))
                    as i32)
                    != 0
                {
                    4
                } else {
                    1
                };
                continue;
            }
            // C line 940
            6 => {
                vm_block = if !(s).is_null() { 5 } else { 0 };
                continue;
            }
            // C line 939
            7 => {
                s = ((JS_GetOpaque(val, stdio_file_class_id())) as *mut JSSTDFile);
                vm_block = 6;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:958. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_strerror(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 3;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 964
            1 => {
                return JS_NewString(ctx, libc::strerror(err));
            }
            // C line 963
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 962
            3 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(err),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    2
                } else {
                    1
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:967. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_parseExtJSON(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut obj: JSValue = core::mem::zeroed();
    let mut str: *const c_char = core::mem::zeroed();
    let mut len: usize = core::mem::zeroed();
    let mut vm_block: usize = 6;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 979
            1 => {
                return obj;
            }
            // C line 978
            2 => {
                let _ = JS_FreeCString(ctx, str);
                vm_block = 1;
                continue;
            }
            // C line 977
            3 => {
                let _ = {
                    let assigned = JS_ParseJSON2(
                        ctx,
                        str,
                        len,
                        c"<input>".as_ptr(),
                        (1 as i32).wrapping_shl((0 as i32) as u32),
                    );
                    obj = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 976
            4 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 975
            5 => {
                vm_block = if (!(!(str).is_null()) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 974
            6 => {
                let _ = {
                    let assigned = JS_ToCStringLen(
                        ctx,
                        core::ptr::addr_of_mut!(len),
                        *(argv).offset((0 as i32) as isize),
                    );
                    str = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:982. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_new_std_file(
    mut ctx: *mut JSContext,
    mut f: *mut libc::FILE,
    mut close_in_finalizer: i32,
    mut is_popen: i32,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut s: *mut JSSTDFile = core::mem::zeroed();
    let mut obj: JSValue = core::mem::zeroed();
    let mut vm_block: usize = 12;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1000
            1 => {
                return obj;
            }
            // C line 999
            2 => {
                let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
                vm_block = 1;
                continue;
            }
            // C line 998
            3 => {
                let _ = {
                    let assigned = f;
                    (*(s)).f = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 997
            4 => {
                let _ = {
                    let assigned = is_popen;
                    (*(s)).is_popen = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 996
            5 => {
                let _ = {
                    let assigned = close_in_finalizer;
                    (*(s)).close_in_finalizer = assigned;
                    assigned
                };
                vm_block = 4;
                continue;
            }
            // C line 994
            6 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 993
            7 => {
                let _ = JS_FreeValue(ctx, obj);
                vm_block = 6;
                continue;
            }
            // C line 992
            8 => {
                vm_block = if (!(!(s).is_null()) as i32) != 0 {
                    7
                } else {
                    5
                };
                continue;
            }
            // C line 991
            9 => {
                let _ = {
                    let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSSTDFile>() as usize)))
                        as *mut JSSTDFile);
                    s = assigned;
                    assigned
                };
                vm_block = 8;
                continue;
            }
            // C line 990
            10 => {
                return obj;
            }
            // C line 989
            11 => {
                vm_block = if (JS_IsException(obj)) != 0 { 10 } else { 9 };
                continue;
            }
            // C line 988
            12 => {
                let _ = {
                    let assigned = JS_NewObjectClass(ctx, ((stdio_file_class_id()) as i32));
                    obj = assigned;
                    assigned
                };
                vm_block = 11;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1003. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_set_error_object(mut ctx: *mut JSContext, mut obj: JSValue, mut err: i32) -> () {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut vm_block: usize = 2;
    loop {
        match vm_block {
            // C line ?
            0 => {
                return ();
            }
            // C line 1006
            1 => {
                let _ = JS_SetPropertyStr(ctx, obj, c"errno".as_ptr(), JS_NewInt32(ctx, err));
                vm_block = 0;
                continue;
            }
            // C line 1005
            2 => {
                vm_block = if (!((JS_IsUndefined(obj)) != 0) as i32) != 0 {
                    1
                } else {
                    0
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1010. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_open(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut filename: *const c_char = core::mem::zeroed();
    let mut mode: *const c_char = core::mem::zeroed();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 24;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1043
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1042
            2 => {
                let _ = JS_FreeCString(ctx, mode);
                vm_block = 1;
                continue;
            }
            // C line ? labels: fail
            3 => {
                let _ = JS_FreeCString(ctx, filename);
                vm_block = 2;
                continue;
            }
            // C line 1039
            4 => {
                return js_new_std_file(
                    ctx,
                    f,
                    quickjs::cutils_header::TRUE,
                    quickjs::cutils_header::FALSE,
                );
            }
            // C line 1038
            5 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_NULL as i32) as i64),
                };
            }
            // C line 1037
            6 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1036
            7 => {
                let _ = JS_FreeCString(ctx, mode);
                vm_block = 6;
                continue;
            }
            // C line 1035
            8 => {
                let _ = JS_FreeCString(ctx, filename);
                vm_block = 7;
                continue;
            }
            // C line 1034
            9 => {
                let _ = js_set_error_object(ctx, *(argv).offset((2 as i32) as isize), err);
                vm_block = 8;
                continue;
            }
            // C line 1033
            10 => {
                vm_block = if (((argc) >= (3 as i32)) as i32) != 0 {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 1030
            11 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 1032
            12 => {
                let _ = {
                    let assigned = (0 as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 1029
            13 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    11
                } else {
                    12
                };
                continue;
            }
            // C line 1028
            14 => {
                let _ = {
                    let assigned = libc::fopen(filename, mode);
                    f = assigned;
                    assigned
                };
                vm_block = 13;
                continue;
            }
            // C line 1025
            15 => {
                vm_block = 3;
                continue;
            }
            // C line 1024
            16 => {
                let _ = JS_ThrowTypeError(ctx, c"invalid file mode".as_ptr());
                vm_block = 15;
                continue;
            }
            // C line 1023
            17 => {
                vm_block = if ((((*(mode).offset((libc::strspn(mode, c"rwa+b".as_ptr())) as isize))
                    as i32)
                    != (0 as i32)) as i32)
                    != 0
                {
                    16
                } else {
                    14
                };
                continue;
            }
            // C line 1022
            18 => {
                vm_block = 3;
                continue;
            }
            // C line 1021
            19 => {
                vm_block = if (!(!(mode).is_null()) as i32) != 0 {
                    18
                } else {
                    17
                };
                continue;
            }
            // C line 1020
            20 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((1 as i32) as isize));
                    mode = assigned;
                    assigned
                };
                vm_block = 19;
                continue;
            }
            // C line 1019
            21 => {
                vm_block = 3;
                continue;
            }
            // C line 1018
            22 => {
                vm_block = if (!(!(filename).is_null()) as i32) != 0 {
                    21
                } else {
                    20
                };
                continue;
            }
            // C line 1017
            23 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    filename = assigned;
                    assigned
                };
                vm_block = 22;
                continue;
            }
            // C line 1013
            24 => {
                mode = core::ptr::null_mut::<c_char>();
                vm_block = 23;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1046. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_popen(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut filename: *const c_char = core::mem::zeroed();
    let mut mode: *const c_char = core::mem::zeroed();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 24;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1079
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1078
            2 => {
                let _ = JS_FreeCString(ctx, mode);
                vm_block = 1;
                continue;
            }
            // C line ? labels: fail
            3 => {
                let _ = JS_FreeCString(ctx, filename);
                vm_block = 2;
                continue;
            }
            // C line 1075
            4 => {
                return js_new_std_file(
                    ctx,
                    f,
                    quickjs::cutils_header::TRUE,
                    quickjs::cutils_header::TRUE,
                );
            }
            // C line 1074
            5 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_NULL as i32) as i64),
                };
            }
            // C line 1073
            6 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1072
            7 => {
                let _ = JS_FreeCString(ctx, mode);
                vm_block = 6;
                continue;
            }
            // C line 1071
            8 => {
                let _ = JS_FreeCString(ctx, filename);
                vm_block = 7;
                continue;
            }
            // C line 1070
            9 => {
                let _ = js_set_error_object(ctx, *(argv).offset((2 as i32) as isize), err);
                vm_block = 8;
                continue;
            }
            // C line 1069
            10 => {
                vm_block = if (((argc) >= (3 as i32)) as i32) != 0 {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 1066
            11 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 1068
            12 => {
                let _ = {
                    let assigned = (0 as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 1065
            13 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    11
                } else {
                    12
                };
                continue;
            }
            // C line 1064
            14 => {
                let _ = {
                    let assigned = libc::popen(filename, mode);
                    f = assigned;
                    assigned
                };
                vm_block = 13;
                continue;
            }
            // C line 1061
            15 => {
                vm_block = 3;
                continue;
            }
            // C line 1060
            16 => {
                let _ = JS_ThrowTypeError(ctx, c"invalid file mode".as_ptr());
                vm_block = 15;
                continue;
            }
            // C line 1059
            17 => {
                vm_block = if ((((*(mode).offset((libc::strspn(mode, c"rw".as_ptr())) as isize))
                    as i32)
                    != (0 as i32)) as i32)
                    != 0
                {
                    16
                } else {
                    14
                };
                continue;
            }
            // C line 1058
            18 => {
                vm_block = 3;
                continue;
            }
            // C line 1057
            19 => {
                vm_block = if (!(!(mode).is_null()) as i32) != 0 {
                    18
                } else {
                    17
                };
                continue;
            }
            // C line 1056
            20 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((1 as i32) as isize));
                    mode = assigned;
                    assigned
                };
                vm_block = 19;
                continue;
            }
            // C line 1055
            21 => {
                vm_block = 3;
                continue;
            }
            // C line 1054
            22 => {
                vm_block = if (!(!(filename).is_null()) as i32) != 0 {
                    21
                } else {
                    20
                };
                continue;
            }
            // C line 1053
            23 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    filename = assigned;
                    assigned
                };
                vm_block = 22;
                continue;
            }
            // C line 1049
            24 => {
                mode = core::ptr::null_mut::<c_char>();
                vm_block = 23;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1082. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_fdopen(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut mode: *const c_char = core::mem::zeroed();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut fd: i32 = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 20;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1112
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line ? labels: fail
            2 => {
                let _ = JS_FreeCString(ctx, mode);
                vm_block = 1;
                continue;
            }
            // C line 1109
            3 => {
                return js_new_std_file(
                    ctx,
                    f,
                    quickjs::cutils_header::TRUE,
                    quickjs::cutils_header::FALSE,
                );
            }
            // C line 1108
            4 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_NULL as i32) as i64),
                };
            }
            // C line 1107
            5 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 1106
            6 => {
                let _ = JS_FreeCString(ctx, mode);
                vm_block = 5;
                continue;
            }
            // C line 1105
            7 => {
                let _ = js_set_error_object(ctx, *(argv).offset((2 as i32) as isize), err);
                vm_block = 6;
                continue;
            }
            // C line 1104
            8 => {
                vm_block = if (((argc) >= (3 as i32)) as i32) != 0 {
                    7
                } else {
                    6
                };
                continue;
            }
            // C line 1101
            9 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 8;
                continue;
            }
            // C line 1103
            10 => {
                let _ = {
                    let assigned = (0 as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 8;
                continue;
            }
            // C line 1100
            11 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    9
                } else {
                    10
                };
                continue;
            }
            // C line 1099
            12 => {
                let _ = {
                    let assigned = libc::fdopen(fd, mode);
                    f = assigned;
                    assigned
                };
                vm_block = 11;
                continue;
            }
            // C line 1096
            13 => {
                vm_block = 2;
                continue;
            }
            // C line 1095
            14 => {
                let _ = JS_ThrowTypeError(ctx, c"invalid file mode".as_ptr());
                vm_block = 13;
                continue;
            }
            // C line 1094
            15 => {
                vm_block = if ((((*(mode).offset((libc::strspn(mode, c"rwa+".as_ptr())) as isize))
                    as i32)
                    != (0 as i32)) as i32)
                    != 0
                {
                    14
                } else {
                    12
                };
                continue;
            }
            // C line 1093
            16 => {
                vm_block = 2;
                continue;
            }
            // C line 1092
            17 => {
                vm_block = if (!(!(mode).is_null()) as i32) != 0 {
                    16
                } else {
                    15
                };
                continue;
            }
            // C line 1091
            18 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((1 as i32) as isize));
                    mode = assigned;
                    assigned
                };
                vm_block = 17;
                continue;
            }
            // C line 1090
            19 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1089
            20 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(fd),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    19
                } else {
                    18
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1115. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_tmpfile(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 6;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1124
            1 => {
                return js_new_std_file(
                    ctx,
                    f,
                    quickjs::cutils_header::TRUE,
                    quickjs::cutils_header::FALSE,
                );
            }
            // C line 1123
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_NULL as i32) as i64),
                };
            }
            // C line 1122
            3 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1121
            4 => {
                let _ = js_set_error_object(
                    ctx,
                    *(argv).offset((0 as i32) as isize),
                    if !(f).is_null() {
                        (0 as i32)
                    } else {
                        *(stdio_errno_pointer())
                    },
                );
                vm_block = 3;
                continue;
            }
            // C line 1120
            5 => {
                vm_block = if (((argc) >= (1 as i32)) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 1119
            6 => {
                let _ = {
                    let assigned = libc::tmpfile();
                    f = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1127. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_sprintf(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut vm_block: usize = 1;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1130
            1 => {
                return js_printf_internal(ctx, argc, argv, core::ptr::null_mut::<libc::FILE>());
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1133. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_printf(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut vm_block: usize = 1;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1136
            1 => {
                return js_printf_internal(ctx, argc, argv, stdio_stdout());
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1139. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_get(mut ctx: *mut JSContext, mut obj: JSValue) -> *mut libc::FILE {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut s: *mut JSSTDFile = core::mem::zeroed();
    let mut vm_block: usize = 7;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1148
            1 => {
                return (*(s)).f;
            }
            // C line 1146
            2 => {
                return core::ptr::null_mut::<libc::FILE>();
            }
            // C line 1145
            3 => {
                let _ = JS_ThrowTypeError(ctx, c"invalid file handle".as_ptr());
                vm_block = 2;
                continue;
            }
            // C line 1144
            4 => {
                vm_block = if (!(!((*(s)).f).is_null()) as i32) != 0 {
                    3
                } else {
                    1
                };
                continue;
            }
            // C line 1143
            5 => {
                return core::ptr::null_mut::<libc::FILE>();
            }
            // C line 1142
            6 => {
                vm_block = if (!(!(s).is_null()) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1141
            7 => {
                s = ((JS_GetOpaque2(ctx, obj, stdio_file_class_id())) as *mut JSSTDFile);
                vm_block = 6;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1151. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_puts(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
    mut magic: i32,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut i: i32 = core::mem::zeroed();
    let mut str: *const c_char = core::mem::zeroed();
    let mut len: usize = core::mem::zeroed();
    let mut vm_block: usize = 14;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1174
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 1167
            2 => {
                vm_block = if (((i) < (argc)) as i32) != 0 { 8 } else { 1 };
                continue;
            }
            // C line ?
            3 => {
                let _ = {
                    let old = i;
                    i = (i).wrapping_add(1);
                    old
                };
                vm_block = 2;
                continue;
            }
            // C line 1172
            4 => {
                let _ = JS_FreeCString(ctx, str);
                vm_block = 3;
                continue;
            }
            // C line 1171
            5 => {
                let _ = libc::fwrite(((str) as *const c_void), ((1 as i32) as usize), len, f);
                vm_block = 4;
                continue;
            }
            // C line 1170
            6 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1169
            7 => {
                vm_block = if (!(!(str).is_null()) as i32) != 0 {
                    6
                } else {
                    5
                };
                continue;
            }
            // C line 1168
            8 => {
                let _ = {
                    let assigned = JS_ToCStringLen(
                        ctx,
                        core::ptr::addr_of_mut!(len),
                        *(argv).offset((i) as isize),
                    );
                    str = assigned;
                    assigned
                };
                vm_block = 7;
                continue;
            }
            // C line 1167
            9 => {
                let _ = {
                    let assigned = (0 as i32);
                    i = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 1160
            10 => {
                let _ = {
                    let assigned = stdio_stdout();
                    f = assigned;
                    assigned
                };
                vm_block = 9;
                continue;
            }
            // C line 1164
            11 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1163
            12 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    11
                } else {
                    9
                };
                continue;
            }
            // C line 1162
            13 => {
                let _ = {
                    let assigned = js_std_file_get(ctx, this_val);
                    f = assigned;
                    assigned
                };
                vm_block = 12;
                continue;
            }
            // C line 1159
            14 => {
                vm_block = if (((magic) == (0 as i32)) as i32) != 0 {
                    10
                } else {
                    13
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1177. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_close(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut s: *mut JSSTDFile = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 10;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1191
            1 => {
                return JS_NewInt32(ctx, err);
            }
            // C line 1190
            2 => {
                let _ = {
                    let assigned = core::ptr::null_mut::<libc::FILE>();
                    (*(s)).f = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1187
            3 => {
                let _ = {
                    let assigned = ((js_get_errno(((libc::pclose((*(s)).f)) as isize))) as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 1189
            4 => {
                let _ = {
                    let assigned = ((js_get_errno(((libc::fclose((*(s)).f)) as isize))) as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 1186
            5 => {
                vm_block = if ((*(s)).is_popen) != 0 { 3 } else { 4 };
                continue;
            }
            // C line 1185
            6 => {
                return JS_ThrowTypeError(ctx, c"invalid file handle".as_ptr());
            }
            // C line 1184
            7 => {
                vm_block = if (!(!((*(s)).f).is_null()) as i32) != 0 {
                    6
                } else {
                    5
                };
                continue;
            }
            // C line 1183
            8 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1182
            9 => {
                vm_block = if (!(!(s).is_null()) as i32) != 0 {
                    8
                } else {
                    7
                };
                continue;
            }
            // C line 1180
            10 => {
                s = ((JS_GetOpaque2(ctx, this_val, stdio_file_class_id())) as *mut JSSTDFile);
                vm_block = 9;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1194. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_printf(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 4;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1200
            1 => {
                return js_printf_internal(ctx, argc, argv, f);
            }
            // C line 1199
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1198
            3 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1197
            4 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 3;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1203. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn stdio_print_value_write(
    mut opaque: *mut c_void,
    mut buf: *const c_char,
    mut len: usize,
) -> () {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut fo: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 2;
    loop {
        match vm_block {
            // C line ?
            0 => {
                return ();
            }
            // C line 1206
            1 => {
                let _ = libc::fwrite(((buf) as *const c_void), ((1 as i32) as usize), len, fo);
                vm_block = 0;
                continue;
            }
            // C line 1205
            2 => {
                fo = ((opaque) as *mut libc::FILE);
                vm_block = 1;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1209. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_printObject(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut vm_block: usize = 2;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1213
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 1212
            2 => {
                let _ = JS_PrintValue(
                    ctx,
                    Some(stdio_print_value_write),
                    ((stdio_stdout()) as *mut c_void),
                    *(argv).offset((0 as i32) as isize),
                    core::ptr::null(),
                );
                vm_block = 1;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1216. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_flush(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 5;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1223
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 1222
            2 => {
                let _ = libc::fflush(f);
                vm_block = 1;
                continue;
            }
            // C line 1221
            3 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1220
            4 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    3
                } else {
                    2
                };
                continue;
            }
            // C line 1219
            5 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 4;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1226. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_tell(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
    mut is_bigint: i32,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut pos: i64 = core::mem::zeroed();
    let mut vm_block: usize = 7;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1239
            1 => {
                return JS_NewBigInt64(ctx, pos);
            }
            // C line 1241
            2 => {
                return JS_NewInt64(ctx, pos);
            }
            // C line 1238
            3 => {
                vm_block = if (is_bigint) != 0 { 1 } else { 2 };
                continue;
            }
            // C line 1236
            4 => {
                let _ = {
                    let assigned = libc::ftell(f);
                    pos = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 1232
            5 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1231
            6 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1229
            7 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 6;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1244. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_seek(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut pos: i64 = core::mem::zeroed();
    let mut whence: i32 = core::mem::zeroed();
    let mut ret: i32 = core::mem::zeroed();
    let mut vm_block: usize = 11;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1263
            1 => {
                return JS_NewInt32(ctx, ret);
            }
            // C line 1262
            2 => {
                let _ = {
                    let assigned = (*(stdio_errno_pointer())).wrapping_neg();
                    ret = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1261
            3 => {
                vm_block = if (((ret) < (0 as i32)) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1259
            4 => {
                let _ = {
                    let assigned = libc::fseek(f, pos, whence);
                    ret = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 1255
            5 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1254
            6 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(whence),
                    *(argv).offset((1 as i32) as isize),
                )) != 0
                {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1253
            7 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1252
            8 => {
                vm_block = if (JS_ToInt64Ext(
                    ctx,
                    core::ptr::addr_of_mut!(pos),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    7
                } else {
                    6
                };
                continue;
            }
            // C line 1251
            9 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1250
            10 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 1247
            11 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 10;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1266. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_eof(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 4;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1272
            1 => {
                return JS_NewBool(ctx, libc::feof(f));
            }
            // C line 1271
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1270
            3 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1269
            4 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 3;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1275. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_error(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 4;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1281
            1 => {
                return JS_NewBool(ctx, libc::ferror(f));
            }
            // C line 1280
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1279
            3 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1278
            4 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 3;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1284. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_clearerr(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 5;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1291
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 1290
            2 => {
                let _ = libc::clearerr(f);
                vm_block = 1;
                continue;
            }
            // C line 1289
            3 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1288
            4 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    3
                } else {
                    2
                };
                continue;
            }
            // C line 1287
            5 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 4;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1294. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_fileno(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 4;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1300
            1 => {
                return JS_NewInt32(ctx, libc::fileno(f));
            }
            // C line 1299
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1298
            3 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1297
            4 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 3;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1303. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_read_write(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
    mut magic: i32,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut pos: u64 = core::mem::zeroed();
    let mut len: u64 = core::mem::zeroed();
    let mut size: usize = core::mem::zeroed();
    let mut ret: usize = core::mem::zeroed();
    let mut buf: *mut u8 = core::mem::zeroed();
    let mut vm_block: usize = 16;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1326
            1 => {
                return JS_NewInt64(ctx, ((ret) as i64));
            }
            // C line 1323
            2 => {
                let _ = {
                    let assigned = libc::fwrite(
                        (((buf).offset(((pos) as isize))) as *const c_void),
                        ((1 as i32) as usize),
                        ((len) as usize),
                        f,
                    );
                    ret = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1325
            3 => {
                let _ = {
                    let assigned = libc::fread(
                        (((buf).offset(((pos) as isize))) as *mut c_void),
                        ((1 as i32) as usize),
                        ((len) as usize),
                        f,
                    );
                    ret = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1322
            4 => {
                vm_block = if (magic) != 0 { 2 } else { 3 };
                continue;
            }
            // C line 1321
            5 => {
                return JS_ThrowRangeError(ctx, c"read/write array buffer overflow".as_ptr());
            }
            // C line 1320
            6 => {
                vm_block = if ((((pos).wrapping_add(len)) > ((size) as u64)) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1319
            7 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1318
            8 => {
                vm_block = if (!(!(buf).is_null()) as i32) != 0 {
                    7
                } else {
                    6
                };
                continue;
            }
            // C line 1317
            9 => {
                let _ = {
                    let assigned = JS_GetArrayBuffer(
                        ctx,
                        core::ptr::addr_of_mut!(size),
                        *(argv).offset((0 as i32) as isize),
                    );
                    buf = assigned;
                    assigned
                };
                vm_block = 8;
                continue;
            }
            // C line 1316
            10 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1315
            11 => {
                vm_block = if (JS_ToIndex(
                    ctx,
                    core::ptr::addr_of_mut!(len),
                    *(argv).offset((2 as i32) as isize),
                )) != 0
                {
                    10
                } else {
                    9
                };
                continue;
            }
            // C line 1314
            12 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1313
            13 => {
                vm_block = if (JS_ToIndex(
                    ctx,
                    core::ptr::addr_of_mut!(pos),
                    *(argv).offset((1 as i32) as isize),
                )) != 0
                {
                    12
                } else {
                    11
                };
                continue;
            }
            // C line 1312
            14 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1311
            15 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    14
                } else {
                    13
                };
                continue;
            }
            // C line 1306
            16 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 15;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1330. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_getline(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut c: i32 = core::mem::zeroed();
    let mut dbuf: DynBuf = core::mem::zeroed();
    let mut obj: JSValue = core::mem::zeroed();
    let mut vm_block: usize = 19;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1362
            1 => {
                return obj;
            }
            // C line 1361
            2 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(dbuf));
                vm_block = 1;
                continue;
            }
            // C line 1360
            3 => {
                let _ = {
                    let assigned =
                        JS_NewStringLen(ctx, (((dbuf).buf) as *const c_char), (dbuf).size);
                    obj = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 1342
            4 => {
                vm_block = 15;
                continue;
            }
            // C line 1357
            5 => {
                return JS_ThrowOutOfMemory(ctx);
            }
            // C line 1356
            6 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(dbuf));
                vm_block = 5;
                continue;
            }
            // C line 1355
            7 => {
                vm_block = if (stdio_dbuf_putc(core::ptr::addr_of_mut!(dbuf), ((c) as u8))) != 0 {
                    6
                } else {
                    4
                };
                continue;
            }
            // C line 1354
            8 => {
                vm_block = 3;
                continue;
            }
            // C line 1353
            9 => {
                vm_block = if (((c) == (10 as i32)) as i32) != 0 {
                    8
                } else {
                    7
                };
                continue;
            }
            // C line 1348
            10 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_NULL as i32) as i64),
                };
            }
            // C line 1347
            11 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(dbuf));
                vm_block = 10;
                continue;
            }
            // C line 1350
            12 => {
                vm_block = 3;
                continue;
            }
            // C line 1345
            13 => {
                vm_block = if ((((dbuf).size) == ((0 as i32) as usize)) as i32) != 0 {
                    11
                } else {
                    12
                };
                continue;
            }
            // C line 1344
            14 => {
                vm_block = if (((c) == ((1 as i32).wrapping_neg())) as i32) != 0 {
                    13
                } else {
                    9
                };
                continue;
            }
            // C line 1343
            15 => {
                let _ = {
                    let assigned = libc::fgetc(f);
                    c = assigned;
                    assigned
                };
                vm_block = 14;
                continue;
            }
            // C line 1341
            16 => {
                let _ = js_std_dbuf_init(ctx, core::ptr::addr_of_mut!(dbuf));
                vm_block = 4;
                continue;
            }
            // C line 1339
            17 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1338
            18 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    17
                } else {
                    16
                };
                continue;
            }
            // C line 1333
            19 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 18;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1366. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_readAsString(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut c: i32 = core::mem::zeroed();
    let mut dbuf: DynBuf = core::mem::zeroed();
    let mut obj: JSValue = core::mem::zeroed();
    let mut max_size64: u64 = core::mem::zeroed();
    let mut max_size: usize = core::mem::zeroed();
    let mut max_size_val: JSValue = core::mem::zeroed();
    let mut vm_block: usize = 24;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1405
            1 => {
                return obj;
            }
            // C line 1404
            2 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(dbuf));
                vm_block = 1;
                continue;
            }
            // C line 1403
            3 => {
                let _ = {
                    let assigned =
                        JS_NewStringLen(ctx, (((dbuf).buf) as *const c_char), (dbuf).size);
                    obj = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 1393
            4 => {
                vm_block = if (((max_size) != ((0 as i32) as usize)) as i32) != 0 {
                    11
                } else {
                    3
                };
                continue;
            }
            // C line 1401
            5 => {
                let _ = {
                    let old = max_size;
                    max_size = (max_size).wrapping_sub(1);
                    old
                };
                vm_block = 4;
                continue;
            }
            // C line 1399
            6 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1398
            7 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(dbuf));
                vm_block = 6;
                continue;
            }
            // C line 1397
            8 => {
                vm_block = if (stdio_dbuf_putc(core::ptr::addr_of_mut!(dbuf), ((c) as u8))) != 0 {
                    7
                } else {
                    5
                };
                continue;
            }
            // C line 1396
            9 => {
                vm_block = 3;
                continue;
            }
            // C line 1395
            10 => {
                vm_block = if (((c) == ((1 as i32).wrapping_neg())) as i32) != 0 {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 1394
            11 => {
                let _ = {
                    let assigned = libc::fgetc(f);
                    c = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 1392
            12 => {
                let _ = js_std_dbuf_init(ctx, core::ptr::addr_of_mut!(dbuf));
                vm_block = 4;
                continue;
            }
            // C line 1389
            13 => {
                let _ = {
                    let assigned = ((max_size64) as usize);
                    max_size = assigned;
                    assigned
                };
                vm_block = 12;
                continue;
            }
            // C line 1388
            14 => {
                vm_block = if (((max_size64) < ((max_size) as u64)) as i32) != 0 {
                    13
                } else {
                    12
                };
                continue;
            }
            // C line 1387
            15 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1386
            16 => {
                vm_block =
                    if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(max_size64), max_size_val)) != 0 {
                        15
                    } else {
                        14
                    };
                continue;
            }
            // C line 1385
            17 => {
                vm_block = if (!((JS_IsUndefined(max_size_val)) != 0) as i32) != 0 {
                    16
                } else {
                    12
                };
                continue;
            }
            // C line 1384
            18 => {
                let _ = {
                    let assigned = (((1 as i32).wrapping_neg()) as usize);
                    max_size = assigned;
                    assigned
                };
                vm_block = 17;
                continue;
            }
            // C line 1381
            19 => {
                let _ = {
                    let assigned = *(argv).offset((0 as i32) as isize);
                    max_size_val = assigned;
                    assigned
                };
                vm_block = 18;
                continue;
            }
            // C line 1383
            20 => {
                let _ = {
                    let assigned = JSValue {
                        u: JSValueUnion {
                            uint64: (((0 as i32) as u32) as u64),
                        },
                        tag: ((JS_TAG_UNDEFINED as i32) as i64),
                    };
                    max_size_val = assigned;
                    assigned
                };
                vm_block = 18;
                continue;
            }
            // C line 1380
            21 => {
                vm_block = if (((argc) >= (1 as i32)) as i32) != 0 {
                    19
                } else {
                    20
                };
                continue;
            }
            // C line 1378
            22 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1377
            23 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    22
                } else {
                    21
                };
                continue;
            }
            // C line 1369
            24 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 23;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1408. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_getByte(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut vm_block: usize = 4;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1414
            1 => {
                return JS_NewInt32(ctx, libc::fgetc(f));
            }
            // C line 1413
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1412
            3 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 1411
            4 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 3;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1417. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_file_putByte(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut c: i32 = core::mem::zeroed();
    let mut vm_block: usize = 7;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1427
            1 => {
                return JS_NewInt32(ctx, c);
            }
            // C line 1426
            2 => {
                let _ = {
                    let assigned = libc::fputc(c, f);
                    c = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1425
            3 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1424
            4 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(c),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    3
                } else {
                    2
                };
                continue;
            }
            // C line 1423
            5 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1422
            6 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1420
            7 => {
                f = js_std_file_get(ctx, this_val);
                vm_block = 6;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1435. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn http_get_header_line(
    mut f: *mut libc::FILE,
    mut buf: *mut c_char,
    mut buf_size: usize,
    mut dbuf: *mut DynBuf,
) -> i32 {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut c: i32 = core::mem::zeroed();
    let mut p: *mut c_char = core::mem::zeroed();
    let mut vm_block: usize = 13;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1454
            1 => {
                return (0 as i32);
            }
            // C line 1453
            2 => {
                let _ = {
                    let assigned = ((0 as i32) as c_char);
                    *(p) = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1442
            3 => {
                vm_block = 12;
                continue;
            }
            // C line 1451
            4 => {
                vm_block = 2;
                continue;
            }
            // C line 1450
            5 => {
                vm_block = if (((c) == (10 as i32)) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 1449
            6 => {
                let _ = stdio_dbuf_putc(dbuf, ((c) as u8));
                vm_block = 5;
                continue;
            }
            // C line 1448
            7 => {
                vm_block = if !(dbuf).is_null() { 6 } else { 5 };
                continue;
            }
            // C line 1447
            8 => {
                let _ = {
                    let assigned = ((c) as c_char);
                    *({
                        let old = p;
                        p = (p).offset(1);
                        old
                    }) = assigned;
                    assigned
                };
                vm_block = 7;
                continue;
            }
            // C line 1446
            9 => {
                vm_block = if (((((p).offset_from(buf) as i64) as usize)
                    < ((buf_size).wrapping_sub(((1 as i32) as usize))))
                    as i32)
                    != 0
                {
                    8
                } else {
                    7
                };
                continue;
            }
            // C line 1445
            10 => {
                return (1 as i32).wrapping_neg();
            }
            // C line 1444
            11 => {
                vm_block = if (((c) < (0 as i32)) as i32) != 0 {
                    10
                } else {
                    9
                };
                continue;
            }
            // C line 1443
            12 => {
                let _ = {
                    let assigned = libc::fgetc(f);
                    c = assigned;
                    assigned
                };
                vm_block = 11;
                continue;
            }
            // C line 1441
            13 => {
                let _ = {
                    let assigned = buf;
                    p = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1457. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn http_get_status(mut buf: *const c_char) -> i32 {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut p: *const c_char = core::mem::zeroed();
    let mut vm_block: usize = 8;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1466
            1 => {
                return libc::atoi(p);
            }
            // C line 1464
            2 => {
                vm_block = if ((((*(p)) as i32) == (32 as i32)) as i32) != 0 {
                    3
                } else {
                    1
                };
                continue;
            }
            // C line 1465
            3 => {
                let _ = {
                    let old = p;
                    p = (p).offset(1);
                    old
                };
                vm_block = 2;
                continue;
            }
            // C line 1463
            4 => {
                return (0 as i32);
            }
            // C line 1462
            5 => {
                vm_block = if ((((*(p)) as i32) != (32 as i32)) as i32) != 0 {
                    4
                } else {
                    2
                };
                continue;
            }
            // C line 1460
            6 => {
                vm_block = if (((((((*(p)) as i32) != (32 as i32)) as i32) != 0)
                    && (((((*(p)) as i32) != (0 as i32)) as i32) != 0))
                    as i32)
                    != 0
                {
                    7
                } else {
                    5
                };
                continue;
            }
            // C line 1461
            7 => {
                let _ = {
                    let old = p;
                    p = (p).offset(1);
                    old
                };
                vm_block = 6;
                continue;
            }
            // C line 1459
            8 => {
                p = buf;
                vm_block = 6;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1469. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_std_urlGet(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut vm_local_storage = Vec::<u64>::new();
    let mut url: *const c_char = core::mem::zeroed();
    let mut cmd_buf: DynBuf = core::mem::zeroed();
    let mut data_buf_s: DynBuf = core::mem::zeroed();
    let mut data_buf: *mut DynBuf = core::mem::zeroed();
    let mut header_buf_s: DynBuf = core::mem::zeroed();
    let mut header_buf: *mut DynBuf = core::mem::zeroed();
    let mut buf: *mut c_char = core::mem::zeroed();
    let mut i: usize = core::mem::zeroed();
    let mut len: usize = core::mem::zeroed();
    let mut status: i32 = core::mem::zeroed();
    let mut response: JSValue = core::mem::zeroed();
    let mut ret_obj: JSValue = core::mem::zeroed();
    let mut options_obj: JSValue = core::mem::zeroed();
    let mut f: *mut libc::FILE = core::mem::zeroed();
    let mut binary_flag: i32 = core::mem::zeroed();
    let mut full_flag: i32 = core::mem::zeroed();
    let mut c: u8 = core::mem::zeroed();
    let mut vm_block: usize = 95;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1622
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1621
            2 => {
                let _ = JS_FreeValue(ctx, response);
                vm_block = 1;
                continue;
            }
            // C line 1620
            3 => {
                let _ = dbuf_free(header_buf);
                vm_block = 2;
                continue;
            }
            // C line 1619
            4 => {
                vm_block = if !(header_buf).is_null() { 3 } else { 2 };
                continue;
            }
            // C line 1618
            5 => {
                let _ = dbuf_free(data_buf);
                vm_block = 4;
                continue;
            }
            // C line 1617
            6 => {
                vm_block = if !(data_buf).is_null() { 5 } else { 4 };
                continue;
            }
            // C line 1616
            7 => {
                let _ = js_free(ctx, ((buf) as *mut c_void));
                vm_block = 6;
                continue;
            }
            // C line 1615
            8 => {
                let _ = libc::pclose(f);
                vm_block = 7;
                continue;
            }
            // C line 1614 labels: fail
            9 => {
                vm_block = if !(f).is_null() { 8 } else { 7 };
                continue;
            }
            // C line 1612
            10 => {
                return ret_obj;
            }
            // C line 1611
            11 => {
                let _ = dbuf_free(header_buf);
                vm_block = 10;
                continue;
            }
            // C line 1604
            12 => {
                let _ = JS_DefinePropertyValueStr(
                    ctx,
                    ret_obj,
                    c"status".as_ptr(),
                    JS_NewInt32(ctx, status),
                    ((((1 as i32).wrapping_shl((0 as i32) as u32))
                        | ((1 as i32).wrapping_shl((1 as i32) as u32)))
                        | ((1 as i32).wrapping_shl((2 as i32) as u32))),
                );
                vm_block = 11;
                continue;
            }
            // C line 1600
            13 => {
                let _ = JS_DefinePropertyValueStr(
                    ctx,
                    ret_obj,
                    c"responseHeaders".as_ptr(),
                    JS_NewStringLen(
                        ctx,
                        (((*(header_buf)).buf) as *mut c_char),
                        (*(header_buf)).size,
                    ),
                    ((((1 as i32).wrapping_shl((0 as i32) as u32))
                        | ((1 as i32).wrapping_shl((1 as i32) as u32)))
                        | ((1 as i32).wrapping_shl((2 as i32) as u32))),
                );
                vm_block = 12;
                continue;
            }
            // C line 1599
            14 => {
                vm_block = if (!((JS_IsNull(response)) != 0) as i32) != 0 {
                    13
                } else {
                    11
                };
                continue;
            }
            // C line 1596
            15 => {
                let _ = JS_DefinePropertyValueStr(
                    ctx,
                    ret_obj,
                    c"response".as_ptr(),
                    response,
                    ((((1 as i32).wrapping_shl((0 as i32) as u32))
                        | ((1 as i32).wrapping_shl((1 as i32) as u32)))
                        | ((1 as i32).wrapping_shl((2 as i32) as u32))),
                );
                vm_block = 14;
                continue;
            }
            // C line 1595
            16 => {
                vm_block = 9;
                continue;
            }
            // C line 1594
            17 => {
                vm_block = if (JS_IsException(ret_obj)) != 0 {
                    16
                } else {
                    15
                };
                continue;
            }
            // C line 1593
            18 => {
                let _ = {
                    let assigned = JS_NewObject(ctx);
                    ret_obj = assigned;
                    assigned
                };
                vm_block = 17;
                continue;
            }
            // C line 1609
            19 => {
                let _ = {
                    let assigned = response;
                    ret_obj = assigned;
                    assigned
                };
                vm_block = 11;
                continue;
            }
            // C line 1592
            20 => {
                vm_block = if (full_flag) != 0 { 18 } else { 19 };
                continue;
            }
            // C line 1590
            21 => {
                let _ = {
                    let assigned = core::ptr::null_mut::<DynBuf>();
                    data_buf = assigned;
                    assigned
                };
                vm_block = 20;
                continue;
            }
            // C line 1589
            22 => {
                let _ = dbuf_free(data_buf);
                vm_block = 21;
                continue;
            }
            // C line 1588
            23 => {
                let _ = {
                    let assigned = core::ptr::null_mut::<libc::FILE>();
                    f = assigned;
                    assigned
                };
                vm_block = 22;
                continue;
            }
            // C line 1587
            24 => {
                let _ = libc::pclose(f);
                vm_block = 23;
                continue;
            }
            // C line 1586
            25 => {
                let _ = {
                    let assigned = core::ptr::null_mut::<c_char>();
                    buf = assigned;
                    assigned
                };
                vm_block = 24;
                continue;
            }
            // C line ? labels: done
            26 => {
                let _ = js_free(ctx, ((buf) as *mut c_void));
                vm_block = 25;
                continue;
            }
            // C line 1583
            27 => {
                vm_block = 9;
                continue;
            }
            // C line 1582
            28 => {
                vm_block = if (JS_IsException(response)) != 0 {
                    27
                } else {
                    26
                };
                continue;
            }
            // C line 1577
            29 => {
                let _ = {
                    let assigned =
                        JS_NewArrayBufferCopy(ctx, (*(data_buf)).buf, (*(data_buf)).size);
                    response = assigned;
                    assigned
                };
                vm_block = 28;
                continue;
            }
            // C line 1580
            30 => {
                let _ = {
                    let assigned = JS_NewStringLen(
                        ctx,
                        (((*(data_buf)).buf) as *mut c_char),
                        (*(data_buf)).size,
                    );
                    response = assigned;
                    assigned
                };
                vm_block = 28;
                continue;
            }
            // C line 1576
            31 => {
                vm_block = if (binary_flag) != 0 { 29 } else { 30 };
                continue;
            }
            // C line 1575
            32 => {
                vm_block = 9;
                continue;
            }
            // C line 1574
            33 => {
                vm_block = if (dbuf_error(data_buf)) != 0 { 32 } else { 31 };
                continue;
            }
            // C line 1568
            34 => {
                vm_block = 38;
                continue;
            }
            // C line 1572
            35 => {
                let _ = dbuf_put(data_buf, ((buf) as *mut u8), len);
                vm_block = 34;
                continue;
            }
            // C line 1571
            36 => {
                vm_block = 33;
                continue;
            }
            // C line 1570
            37 => {
                vm_block = if (((len) == ((0 as i32) as usize)) as i32) != 0 {
                    36
                } else {
                    35
                };
                continue;
            }
            // C line 1569
            38 => {
                let _ = {
                    let assigned = libc::fread(
                        ((buf) as *mut c_void),
                        ((1 as i32) as usize),
                        ((4096 as i32) as usize),
                        f,
                    );
                    len = assigned;
                    assigned
                };
                vm_block = 37;
                continue;
            }
            // C line 1565
            39 => {
                let _ = {
                    (*(header_buf)).size =
                        ((*(header_buf)).size).wrapping_sub(((2 as i32) as usize));
                    (*(header_buf)).size
                };
                vm_block = 34;
                continue;
            }
            // C line 1564
            40 => {
                vm_block = 9;
                continue;
            }
            // C line 1563
            41 => {
                vm_block = if (dbuf_error(header_buf)) != 0 {
                    40
                } else {
                    39
                };
                continue;
            }
            // C line 1554
            42 => {
                vm_block = 47;
                continue;
            }
            // C line 1561
            43 => {
                vm_block = 41;
                continue;
            }
            // C line 1560
            44 => {
                vm_block = if (!((libc::strcmp(buf, c"\r\n".as_ptr())) != 0) as i32) != 0 {
                    43
                } else {
                    42
                };
                continue;
            }
            // C line 1558
            45 => {
                vm_block = 26;
                continue;
            }
            // C line ? labels: bad_header
            46 => {
                let _ = {
                    let assigned = JSValue {
                        u: JSValueUnion {
                            uint64: (((0 as i32) as u32) as u64),
                        },
                        tag: ((JS_TAG_NULL as i32) as i64),
                    };
                    response = assigned;
                    assigned
                };
                vm_block = 45;
                continue;
            }
            // C line 1555
            47 => {
                vm_block =
                    if (((http_get_header_line(f, buf, ((4096 as i32) as usize), header_buf))
                        < (0 as i32)) as i32)
                        != 0
                    {
                        46
                    } else {
                        44
                    };
                continue;
            }
            // C line 1550
            48 => {
                vm_block = 46;
                continue;
            }
            // C line 1549
            49 => {
                vm_block = if ((((!((full_flag) != 0) as i32) != 0)
                    && ((!(((((((status) >= (200 as i32)) as i32) != 0)
                        && ((((status) <= (299 as i32)) as i32) != 0))
                        as i32)
                        != 0) as i32)
                        != 0)) as i32)
                    != 0
                {
                    48
                } else {
                    42
                };
                continue;
            }
            // C line 1548
            50 => {
                let _ = {
                    let assigned = http_get_status(buf);
                    status = assigned;
                    assigned
                };
                vm_block = 49;
                continue;
            }
            // C line 1546
            51 => {
                vm_block = 46;
                continue;
            }
            // C line 1545
            52 => {
                let _ = {
                    let assigned = (0 as i32);
                    status = assigned;
                    assigned
                };
                vm_block = 51;
                continue;
            }
            // C line 1544
            53 => {
                vm_block = if (((http_get_header_line(
                    f,
                    buf,
                    ((4096 as i32) as usize),
                    core::ptr::null_mut::<DynBuf>(),
                )) < (0 as i32)) as i32)
                    != 0
                {
                    52
                } else {
                    50
                };
                continue;
            }
            // C line 1541
            54 => {
                vm_block = 9;
                continue;
            }
            // C line 1540
            55 => {
                vm_block = if (!(!(buf).is_null()) as i32) != 0 {
                    54
                } else {
                    53
                };
                continue;
            }
            // C line 1539
            56 => {
                let _ = {
                    let assigned = ((js_malloc(ctx, ((4096 as i32) as usize))) as *mut c_char);
                    buf = assigned;
                    assigned
                };
                vm_block = 55;
                continue;
            }
            // C line 1537
            57 => {
                let _ = js_std_dbuf_init(ctx, header_buf);
                vm_block = 56;
                continue;
            }
            // C line 1536
            58 => {
                let _ = js_std_dbuf_init(ctx, data_buf);
                vm_block = 57;
                continue;
            }
            // C line 1533
            59 => {
                return JS_ThrowTypeError(ctx, c"could not start curl".as_ptr());
            }
            // C line 1532
            60 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    59
                } else {
                    58
                };
                continue;
            }
            // C line 1531
            61 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(cmd_buf));
                vm_block = 60;
                continue;
            }
            // C line 1530
            62 => {
                let _ = {
                    let assigned = libc::popen((((cmd_buf).buf) as *mut c_char), c"r".as_ptr());
                    f = assigned;
                    assigned
                };
                vm_block = 61;
                continue;
            }
            // C line 1527
            63 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1526
            64 => {
                let _ = dbuf_free(core::ptr::addr_of_mut!(cmd_buf));
                vm_block = 63;
                continue;
            }
            // C line 1525
            65 => {
                vm_block = if (dbuf_error(core::ptr::addr_of_mut!(cmd_buf))) != 0 {
                    64
                } else {
                    62
                };
                continue;
            }
            // C line 1524
            66 => {
                let _ = stdio_dbuf_putc(core::ptr::addr_of_mut!(cmd_buf), ((0 as i32) as u8));
                vm_block = 65;
                continue;
            }
            // C line 1523
            67 => {
                let _ = dbuf_putstr(core::ptr::addr_of_mut!(cmd_buf), c"'".as_ptr());
                vm_block = 66;
                continue;
            }
            // C line 1522
            68 => {
                let _ = JS_FreeCString(ctx, url);
                vm_block = 67;
                continue;
            }
            // C line 1506
            69 => {
                vm_block = if ((((*(url).offset((i) as isize)) as i32) != (0 as i32)) as i32) != 0 {
                    77
                } else {
                    68
                };
                continue;
            }
            // C line ?
            70 => {
                let _ = {
                    let old = i;
                    i = (i).wrapping_add(1);
                    old
                };
                vm_block = 69;
                continue;
            }
            // C line 1519
            71 => {
                vm_block = 70;
                continue;
            }
            // C line ?
            72 => {
                let _ = stdio_dbuf_putc(core::ptr::addr_of_mut!(cmd_buf), c);
                vm_block = 71;
                continue;
            }
            // C line 1515
            73 => {
                let _ = stdio_dbuf_putc(core::ptr::addr_of_mut!(cmd_buf), ((92 as i32) as u8));
                vm_block = 72;
                continue;
            }
            // C line 1512
            74 => {
                vm_block = 70;
                continue;
            }
            // C line 1511
            75 => {
                let _ = dbuf_putstr(core::ptr::addr_of_mut!(cmd_buf), c"'\\''".as_ptr());
                vm_block = 74;
                continue;
            }
            // C line 1508
            76 => {
                vm_block = match ((c) as i32) {
                    x if x == (92 as i32) => 73,
                    x if x == (125 as i32) => 73,
                    x if x == (123 as i32) => 73,
                    x if x == (93 as i32) => 73,
                    x if x == (91 as i32) => 73,
                    x if x == (39 as i32) => 75,
                    _ => 72,
                };
                continue;
            }
            // C line 1507
            77 => {
                c = ((*(url).offset((i) as isize)) as u8);
                vm_block = 76;
                continue;
            }
            // C line 1506
            78 => {
                let _ = {
                    let assigned = ((0 as i32) as usize);
                    i = assigned;
                    assigned
                };
                vm_block = 69;
                continue;
            }
            // C line 1505
            79 => {
                let _ = stdio_dbuf_printf_string(
                    core::ptr::addr_of_mut!(cmd_buf),
                    c"%s '".as_ptr(),
                    c"curl -s -i --".as_ptr(),
                );
                vm_block = 78;
                continue;
            }
            // C line 1504
            80 => {
                let _ = js_std_dbuf_init(ctx, core::ptr::addr_of_mut!(cmd_buf));
                vm_block = 79;
                continue;
            }
            // C line 1500
            81 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line ? labels: fail_obj
            82 => {
                let _ = JS_FreeCString(ctx, url);
                vm_block = 81;
                continue;
            }
            // C line 1497
            83 => {
                vm_block = if (stdio_bool_option(
                    ctx,
                    core::ptr::addr_of_mut!(full_flag),
                    options_obj,
                    c"full".as_ptr(),
                )) != 0
                {
                    82
                } else {
                    80
                };
                continue;
            }
            // C line 1495
            84 => {
                vm_block = 82;
                continue;
            }
            // C line 1494
            85 => {
                vm_block = if (stdio_bool_option(
                    ctx,
                    core::ptr::addr_of_mut!(binary_flag),
                    options_obj,
                    c"binary".as_ptr(),
                )) != 0
                {
                    84
                } else {
                    83
                };
                continue;
            }
            // C line 1492
            86 => {
                let _ = {
                    let assigned = *(argv).offset((1 as i32) as isize);
                    options_obj = assigned;
                    assigned
                };
                vm_block = 85;
                continue;
            }
            // C line 1491
            87 => {
                vm_block = if (((argc) >= (2 as i32)) as i32) != 0 {
                    86
                } else {
                    80
                };
                continue;
            }
            // C line 1489
            88 => {
                let _ = {
                    let assigned = quickjs::cutils_header::FALSE;
                    full_flag = assigned;
                    assigned
                };
                vm_block = 87;
                continue;
            }
            // C line 1488
            89 => {
                let _ = {
                    let assigned = quickjs::cutils_header::FALSE;
                    binary_flag = assigned;
                    assigned
                };
                vm_block = 88;
                continue;
            }
            // C line 1486
            90 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1485
            91 => {
                vm_block = if (!(!(url).is_null()) as i32) != 0 {
                    90
                } else {
                    89
                };
                continue;
            }
            // C line 1484
            92 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    url = assigned;
                    assigned
                };
                vm_block = 91;
                continue;
            }
            // C line 1479
            93 => {
                response = JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
                vm_block = 92;
                continue;
            }
            // C line 1475
            94 => {
                header_buf = core::ptr::addr_of_mut!(header_buf_s);
                vm_block = 93;
                continue;
            }
            // C line 1474
            95 => {
                data_buf = core::ptr::addr_of_mut!(data_buf_s);
                vm_block = 94;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// C:1653..1669: exact errno properties, in declaration order.
const js_std_error_props: [JSCFunctionListEntry; 11] = [
    JS_PROP_INT32_DEF(c"EINVAL".as_ptr(), libc::EINVAL, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EIO".as_ptr(), libc::EIO, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EACCES".as_ptr(), libc::EACCES, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EEXIST".as_ptr(), libc::EEXIST, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"ENOSPC".as_ptr(), libc::ENOSPC, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"ENOSYS".as_ptr(), libc::ENOSYS, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EBUSY".as_ptr(), libc::EBUSY, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"ENOENT".as_ptr(), libc::ENOENT, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EPERM".as_ptr(), libc::EPERM, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EPIPE".as_ptr(), libc::EPIPE, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"EBADF".as_ptr(), libc::EBADF, JS_PROP_CONFIGURABLE),
];
const js_std_funcs: [JSCFunctionListEntry; 24] = [
    JS_CFUNC_DEF(c"exit".as_ptr(), 1, Some(js_std_exit)),
    JS_CFUNC_DEF(c"gc".as_ptr(), 0, Some(js_std_gc)),
    JS_CFUNC_DEF(c"evalScript".as_ptr(), 1, Some(js_evalScript)),
    JS_CFUNC_DEF(c"loadScript".as_ptr(), 1, Some(js_loadScript)),
    JS_CFUNC_DEF(c"getenv".as_ptr(), 1, Some(js_std_getenv)),
    JS_CFUNC_DEF(c"setenv".as_ptr(), 1, Some(js_std_setenv)),
    JS_CFUNC_DEF(c"unsetenv".as_ptr(), 1, Some(js_std_unsetenv)),
    JS_CFUNC_DEF(c"getenviron".as_ptr(), 1, Some(js_std_getenviron)),
    JS_CFUNC_DEF(c"urlGet".as_ptr(), 1, Some(js_std_urlGet)),
    JS_CFUNC_DEF(c"loadFile".as_ptr(), 1, Some(js_std_loadFile)),
    JS_CFUNC_DEF(c"strerror".as_ptr(), 1, Some(js_std_strerror)),
    JS_CFUNC_DEF(c"parseExtJSON".as_ptr(), 1, Some(js_std_parseExtJSON)),
    JS_CFUNC_DEF(c"open".as_ptr(), 2, Some(js_std_open)),
    JS_CFUNC_DEF(c"popen".as_ptr(), 2, Some(js_std_popen)),
    JS_CFUNC_DEF(c"fdopen".as_ptr(), 2, Some(js_std_fdopen)),
    JS_CFUNC_DEF(c"tmpfile".as_ptr(), 0, Some(js_std_tmpfile)),
    JS_CFUNC_MAGIC_DEF(c"puts".as_ptr(), 1, Some(js_std_file_puts), 0),
    JS_CFUNC_DEF(c"printf".as_ptr(), 1, Some(js_std_printf)),
    JS_CFUNC_DEF(c"sprintf".as_ptr(), 1, Some(js_std_sprintf)),
    JS_PROP_INT32_DEF(c"SEEK_SET".as_ptr(), libc::SEEK_SET, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"SEEK_CUR".as_ptr(), libc::SEEK_CUR, JS_PROP_CONFIGURABLE),
    JS_PROP_INT32_DEF(c"SEEK_END".as_ptr(), libc::SEEK_END, JS_PROP_CONFIGURABLE),
    JS_OBJECT_DEF(
        c"Error".as_ptr(),
        js_std_error_props.as_ptr(),
        js_std_error_props.len() as i32,
        JS_PROP_CONFIGURABLE,
    ),
    JS_CFUNC_DEF(c"__printObject".as_ptr(), 1, Some(js_std_file_printObject)),
];
const js_std_file_proto_funcs: [JSCFunctionListEntry; 17] = [
    JS_CFUNC_DEF(c"close".as_ptr(), 0, Some(js_std_file_close)),
    JS_CFUNC_MAGIC_DEF(c"puts".as_ptr(), 1, Some(js_std_file_puts), 1),
    JS_CFUNC_DEF(c"printf".as_ptr(), 1, Some(js_std_file_printf)),
    JS_CFUNC_DEF(c"flush".as_ptr(), 0, Some(js_std_file_flush)),
    JS_CFUNC_MAGIC_DEF(c"tell".as_ptr(), 0, Some(js_std_file_tell), 0),
    JS_CFUNC_MAGIC_DEF(c"tello".as_ptr(), 0, Some(js_std_file_tell), 1),
    JS_CFUNC_DEF(c"seek".as_ptr(), 2, Some(js_std_file_seek)),
    JS_CFUNC_DEF(c"eof".as_ptr(), 0, Some(js_std_file_eof)),
    JS_CFUNC_DEF(c"fileno".as_ptr(), 0, Some(js_std_file_fileno)),
    JS_CFUNC_DEF(c"error".as_ptr(), 0, Some(js_std_file_error)),
    JS_CFUNC_DEF(c"clearerr".as_ptr(), 0, Some(js_std_file_clearerr)),
    JS_CFUNC_MAGIC_DEF(c"read".as_ptr(), 3, Some(js_std_file_read_write), 0),
    JS_CFUNC_MAGIC_DEF(c"write".as_ptr(), 3, Some(js_std_file_read_write), 1),
    JS_CFUNC_DEF(c"getline".as_ptr(), 0, Some(js_std_file_getline)),
    JS_CFUNC_DEF(c"readAsString".as_ptr(), 0, Some(js_std_file_readAsString)),
    JS_CFUNC_DEF(c"getByte".as_ptr(), 0, Some(js_std_file_getByte)),
    JS_CFUNC_DEF(c"putByte".as_ptr(), 1, Some(js_std_file_putByte)),
];
// C:1710..1750. FILE wrappers own closeable handles; standard streams do not.
unsafe fn js_std_init(ctx: *mut JSContext, m: *mut JSModuleDef) -> i32 {
    let class = stdio_file_class_id();
    let definition = JSClassDef {
        class_name: c"FILE".as_ptr(),
        finalizer: Some(js_std_file_finalizer),
        gc_mark: None,
        call: None,
        exotic: ptr::null_mut(),
    };
    JS_NewClass(JS_GetRuntime(ctx), class, &definition);
    let proto = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(
        ctx,
        proto,
        js_std_file_proto_funcs.as_ptr(),
        js_std_file_proto_funcs.len() as i32,
    );
    JS_SetClassProto(ctx, class, proto);
    JS_SetModuleExportList(ctx, m, js_std_funcs.as_ptr(), js_std_funcs.len() as i32);
    JS_SetModuleExport(
        ctx,
        m,
        c"in".as_ptr(),
        js_new_std_file(ctx, stdio_stdin(), 0, 0),
    );
    JS_SetModuleExport(
        ctx,
        m,
        c"out".as_ptr(),
        js_new_std_file(ctx, stdio_stdout(), 0, 0),
    );
    JS_SetModuleExport(
        ctx,
        m,
        c"err".as_ptr(),
        js_new_std_file(ctx, stdio_stderr(), 0, 0),
    );
    0
}
pub unsafe fn js_init_module_std(ctx: *mut JSContext, name: *const c_char) -> *mut JSModuleDef {
    let m = JS_NewCModule(ctx, name, Some(js_std_init));
    if m.is_null() {
        return m;
    }
    JS_AddModuleExportList(ctx, m, js_std_funcs.as_ptr(), js_std_funcs.len() as i32);
    for name in [c"in", c"out", c"err"] {
        JS_AddModuleExport(ctx, m, name.as_ptr());
    }
    m
}
