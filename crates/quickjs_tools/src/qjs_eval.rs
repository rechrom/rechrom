// qjs.c:49..76. Bellard/Gordon, MIT. Included after the host await helper
// is registered; no substitute promise loop or exception handler is used.
pub unsafe fn eval_buf(
    ctx: *mut JSContext,
    buffer: *const c_void,
    len: i32,
    filename: *const std::ffi::c_char,
    eval_flags: i32,
) -> i32 {
    let value = if eval_flags & JS_EVAL_TYPE_MASK == JS_EVAL_TYPE_MODULE {
        let mut val = quickjs::quickjs::JS_Eval(
            ctx,
            buffer.cast(),
            len as usize,
            filename,
            eval_flags | JS_EVAL_FLAG_COMPILE_ONLY,
        );
        if JS_IsException(val) == 0 {
            crate::quickjs_libc::js_module_set_import_meta(ctx, val, 1, 1);
            val = quickjs::quickjs::JS_EvalFunction(ctx, val);
        }
        crate::quickjs_libc::js_std_await(ctx, val)
    } else {
        quickjs::quickjs::JS_Eval(ctx, buffer.cast(), len as usize, filename, eval_flags)
    };
    let result = if JS_IsException(value) != 0 {
        crate::quickjs_libc::js_std_dump_error(ctx);
        -1
    } else {
        0
    };
    JS_FreeValue(ctx, value);
    result
}
// qjs.c:78..104. File failures are reported by the host caller with exit(1).
pub unsafe fn eval_file_bytes(
    ctx: *mut JSContext,
    filename: *const std::ffi::c_char,
    mut module: i32,
    strict: i32,
) -> Result<i32, ByteCliError> {
    let mut len = 0;
    let buf = crate::quickjs_libc::js_load_file(ctx, &mut len, filename);
    if buf.is_null() {
        let error = std::io::Error::last_os_error();
        let text = error.to_string();
        let suffix = error
            .raw_os_error()
            .map(|code| format!(" (os error {code})"));
        let text = suffix
            .as_ref()
            .and_then(|suffix| text.strip_suffix(suffix))
            .unwrap_or(&text);
        let mut message = CStr::from_ptr(filename).to_bytes().to_vec();
        message.extend_from_slice(b": ");
        message.extend_from_slice(text.as_bytes());
        message.push(b'\n');
        return Err(ByteCliError::new(message, 1));
    }
    if module < 0 {
        module = (CStr::from_ptr(filename).to_bytes().ends_with(b".mjs")
            || quickjs::quickjs::JS_DetectModule(buf.cast(), len) != 0) as i32;
    }
    let mut flags = if module != 0 {
        JS_EVAL_TYPE_MODULE
    } else {
        JS_EVAL_TYPE_GLOBAL
    };
    if module == 0 && strict != 0 {
        flags |= JS_EVAL_FLAG_STRICT;
    }
    let result = eval_buf(ctx, buf.cast(), len as i32, filename, flags);
    quickjs::quickjs::js_free(ctx, buf.cast());
    Ok(result)
}

// UTF-8-domain embedding adapter; the CLI uses the byte-preserving entry point.
pub unsafe fn eval_file(
    ctx: *mut JSContext,
    filename: *const std::ffi::c_char,
    module: i32,
    strict: i32,
) -> Result<i32, CliError> {
    eval_file_bytes(ctx, filename, module, strict).map_err(|e| CliError {
        message: String::from_utf8_lossy(&e.message).into_owned(),
        help: e.help,
        exit: e.exit,
    })
}
