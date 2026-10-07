// run-test262.c:1362..1558. Original negative, async and known-error reporting.
// FILE services are host-only; the engine retains its platform-neutral writer.
pub struct RunnerExecutionState {
    pub outfile: *mut libc::FILE,
    pub error_out: *mut libc::FILE,
    pub verbose: bool,
    pub error_file: Option<Vec<u8>>,
    pub new_errors: std::sync::atomic::AtomicI32,
    pub changed_errors: std::sync::atomic::AtomicI32,
    pub fixed_errors: std::sync::atomic::AtomicI32,
}
// POSIX FILE streams serialize each native call exactly as the original host.
unsafe impl Send for RunnerExecutionState {}
unsafe impl Sync for RunnerExecutionState {}
fn runner_atoi(bytes: &[u8]) -> i32 {
    let mut p = 0;
    while bytes.get(p).is_some_and(|c| c_space(*c)) {
        p += 1;
    }
    let negative = bytes.get(p) == Some(&b'-');
    if matches!(bytes.get(p), Some(b'+' | b'-')) {
        p += 1;
    }
    let mut n = 0i32;
    while let Some(c) = bytes.get(p).filter(|c| c.is_ascii_digit()) {
        n = n.wrapping_mul(10).wrapping_add((*c - b'0') as i32);
        p += 1;
    }
    if negative {
        n.wrapping_neg()
    } else {
        n
    }
}
unsafe fn runner_write(file: *mut libc::FILE, bytes: &[u8]) {
    if !file.is_null() {
        libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), file);
    }
}
unsafe fn runner_report(
    file: *mut libc::FILE,
    filename: &[u8],
    line: i32,
    strict: &[u8],
    parts: &[&[u8]],
) {
    let mut bytes = filename.to_vec();
    bytes.extend_from_slice(format!(":{line}: ").as_bytes());
    bytes.extend_from_slice(strict);
    for part in parts {
        bytes.extend_from_slice(part);
    }
    bytes.push(b'\n');
    runner_write(file, &bytes);
}
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
extern "C" {
    #[link_name = "__stdoutp"]
    static mut RUNNER_STDOUT: *mut libc::FILE;
    #[link_name = "__stderrp"]
    static mut RUNNER_STDERR: *mut libc::FILE;
}
#[cfg(any(target_os = "linux", target_os = "android"))]
extern "C" {
    #[link_name = "stdout"]
    static mut RUNNER_STDOUT: *mut libc::FILE;
    #[link_name = "stderr"]
    static mut RUNNER_STDERR: *mut libc::FILE;
}
unsafe fn runner_stdout() -> *mut libc::FILE {
    #[cfg(unix)] { RUNNER_STDOUT }
    #[cfg(windows)] { crate::quickjs_libc_windows::stdio_stdout() }
}
unsafe fn runner_stderr() -> *mut libc::FILE {
    #[cfg(unix)] { RUNNER_STDERR }
    #[cfg(windows)] { crate::quickjs_libc_windows::stdio_stderr() }
}
pub unsafe fn eval_buf(
    ctx: *mut JSContext,
    buf: &[u8],
    filename: *const c_char,
    is_test: bool,
    is_negative: bool,
    error_type: Option<&[u8]>,
    state: &RunnerExecutionState,
    eval_flags: i32,
    is_async: bool,
) -> i32 {
    use std::sync::atomic::Ordering::SeqCst;
    let tls = JS_GetRuntimeOpaque(JS_GetRuntime(ctx)).cast::<ThreadLocalStorage>();
    let (mut pos, pos_line) = skip_comments(buf, 1);
    let mut error_line = pos_line;
    let mut has_error_line = false;
    let mut exception = JS_UNDEFINED;
    let mut error_name = std::ptr::null();
    let ret_promise = eval_flags & JS_EVAL_TYPE_MODULE != 0;
    (*tls).async_done = 0;
    let mut terminated = buf.to_vec();
    terminated.push(0);
    let mut res = JS_Eval(
        ctx,
        terminated.as_ptr().cast(),
        buf.len(),
        filename,
        eval_flags,
    );
    if (is_async || ret_promise) && JS_IsException(res) == 0 {
        let promise = if ret_promise {
            res
        } else {
            JS_FreeValue(ctx, res);
            JS_UNDEFINED
        };
        loop {
            let ret = JS_ExecutePendingJob(JS_GetRuntime(ctx), std::ptr::null_mut());
            if ret < 0 {
                res = JS_EXCEPTION;
                break;
            }
            if ret == 0 {
                res = if is_async {
                    if (*tls).async_done != 1 {
                        JS_ThrowTypeError(ctx, c"$DONE() not called".as_ptr())
                    } else {
                        JS_UNDEFINED
                    }
                } else {
                    match JS_PromiseState(ctx, promise) {
                        JS_PROMISE_FULFILLED => JS_UNDEFINED,
                        JS_PROMISE_REJECTED => JS_Throw(ctx, JS_PromiseResult(ctx, promise)),
                        _ => JS_ThrowTypeError(ctx, c"promise is pending".as_ptr()),
                    }
                };
                break;
            }
        }
        JS_FreeValue(ctx, promise);
    }
    let strict = if eval_flags & JS_EVAL_FLAG_STRICT != 0 {
        &b"strict mode: "[..]
    } else {
        &b""[..]
    };
    let name = CStr::from_ptr(filename).to_bytes();
    let ret;
    if JS_IsException(res) != 0 {
        exception = JS_GetException(ctx);
        let is_error = JS_IsError(ctx, exception) != 0;
        if !state.outfile.is_null() {
            if !is_error {
                runner_write(state.outfile, strict);
                runner_write(state.outfile, b"Throw: ");
            }
            js_print(ctx, JS_NULL, 1, &mut exception);
        }
        if is_error {
            let error_name_val = JS_GetPropertyStr(ctx, exception, c"name".as_ptr());
            error_name = JS_ToCString(ctx, error_name_val);
            let stack = JS_GetPropertyStr(ctx, exception, c"stack".as_ptr());
            if JS_IsUndefined(stack) == 0 {
                let text = JS_ToCString(ctx, stack);
                if !text.is_null() {
                    let bytes = CStr::from_ptr(text).to_bytes();
                    runner_write(state.outfile, bytes);
                    if let Some(p) = substring(bytes, name, 0) {
                        if byte(bytes, p + name.len()) == b':' {
                            error_line = runner_atoi(&bytes[p + name.len() + 1..]);
                            has_error_line = true;
                        }
                    }
                    JS_FreeCString(ctx, text);
                }
            }
            JS_FreeValue(ctx, stack);
            JS_FreeValue(ctx, error_name_val);
        }
        ret = if is_negative {
            if let Some(expected) = error_type {
                let text = JS_ToCString(ctx, exception);
                let bytes = CStr::from_ptr(text).to_bytes();
                let class = &bytes[..bytes.iter().position(|&c| c == b':').unwrap_or(bytes.len())];
                let ret = if class == expected { 0 } else { -1 };
                JS_FreeCString(ctx, text);
                ret
            } else {
                0
            }
        } else {
            -1
        };
    } else {
        ret = if is_negative { -1 } else { 0 };
    }
    if state.verbose && is_test {
        let old = state
            .error_file
            .as_ref()
            .and_then(|s| find_error(Some(s), name, eval_flags & JS_EVAL_FLAG_STRICT != 0));
        let mut msg_val = JS_UNDEFINED;
        let mut msg = std::ptr::null();
        if JS_IsUndefined(exception) == 0 {
            msg_val = JS_ToString(ctx, exception);
            msg = JS_ToCString(ctx, msg_val);
        }
        let message = if msg.is_null() {
            None
        } else {
            Some(CStr::from_ptr(msg).to_bytes())
        };
        if is_negative {
            if ret == 0 {
                if let (Some(message), Some((old, _))) = (message, old.as_ref()) {
                    if old == b"expected error"
                        || old.starts_with(b"unexpected error type:")
                        || old == message
                    {
                        if !has_error_line {
                            let (_, p, line) = longest_match(buf, message, pos, pos_line);
                            if let Some(p) = p {
                                pos = p;
                            }
                            if let Some(line) = line {
                                error_line = line;
                            }
                        }
                        runner_report(
                            runner_stdout(),
                            name,
                            error_line,
                            strict,
                            &[b"OK, now has error ", message],
                        );
                        state.fixed_errors.fetch_add(1, SeqCst);
                    }
                }
            } else if old.is_none() {
                if let Some(message) = message {
                    runner_report(
                        state.error_out,
                        name,
                        error_line,
                        strict,
                        &[b"unexpected error type: ", message],
                    );
                } else {
                    runner_report(
                        state.error_out,
                        name,
                        error_line,
                        strict,
                        &[b"expected error"],
                    );
                }
                state.new_errors.fetch_add(1, SeqCst);
            }
        } else if let Some(message) = message {
            if old.as_ref().is_none_or(|(old, _)| old != message) {
                if !has_error_line {
                    let p = skip_prefix(message, b"Test262 Error: ");
                    let find = if substring(p, b"Test case returned non-true value!", 0).is_some() {
                        &b"runTestCase"[..]
                    } else {
                        p
                    };
                    let (_, p, line) = longest_match(buf, find, pos, pos_line);
                    if let Some(p) = p {
                        pos = p;
                    }
                    if let Some(line) = line {
                        error_line = line;
                    }
                }
                runner_report(
                    state.error_out,
                    name,
                    error_line,
                    strict,
                    &[
                        if state.error_file.is_some() {
                            b"unexpected error: "
                        } else {
                            b""
                        },
                        message,
                    ],
                );
                if let Some((old, line)) = old
                    .as_ref()
                    .filter(|(old, line)| old != message || error_line != *line)
                {
                    runner_report(
                        runner_stdout(),
                        name,
                        *line,
                        strict,
                        &[b"previous error: ", old],
                    );
                    state.changed_errors.fetch_add(1, SeqCst);
                } else {
                    state.new_errors.fetch_add(1, SeqCst);
                }
            }
        } else if let Some((old, line)) = old {
            runner_report(
                runner_stdout(),
                name,
                line,
                strict,
                &[b"OK, fixed error: ", &old],
            );
            state.fixed_errors.fetch_add(1, SeqCst);
        }
        JS_FreeValue(ctx, msg_val);
        JS_FreeCString(ctx, msg);
    }
    let _ = pos;
    JS_FreeCString(ctx, error_name);
    JS_FreeValue(ctx, exception);
    JS_FreeValue(ctx, res);
    ret
}
// run-test262.c:1560..1580. Missing files retain original fatal loading behavior.
pub unsafe fn eval_file(
    ctx: *mut JSContext,
    base: &[u8],
    path: &[u8],
    eval_flags: i32,
    state: &RunnerExecutionState,
) -> Result<i32, RunnerError> {
    let filename = compose_path(Some(base), path);
    let buf = load_file(&filename)?;
    let name = CString::new(filename.clone()).unwrap();
    if eval_buf(
        ctx,
        &buf,
        name.as_ptr(),
        false,
        false,
        None,
        state,
        eval_flags,
        false,
    ) != 0
    {
        libc::fflush(runner_stdout());
        let mut warning = b"run-test262: error evaluating ".to_vec();
        warning.extend_from_slice(&filename);
        warning.push(b'\n');
        runner_write(runner_stderr(), &warning);
        return Ok(1);
    }
    Ok(0)
}
