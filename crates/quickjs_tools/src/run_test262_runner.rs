// run-test262.c:1667..1770. Statistics retain every original averaged field.
pub struct MemoryStats {
    pub count: i32,
    pub all: JSMemoryUsage,
    pub avg: JSMemoryUsage,
    pub min: JSMemoryUsage,
    pub max: JSMemoryUsage,
    pub min_filename: Vec<u8>,
    pub max_filename: Vec<u8>,
}
impl Default for MemoryStats {
    fn default() -> Self {
        unsafe {
            Self {
                count: 0,
                all: std::mem::zeroed(),
                avg: std::mem::zeroed(),
                min: std::mem::zeroed(),
                max: std::mem::zeroed(),
                min_filename: Vec::new(),
                max_filename: Vec::new(),
            }
        }
    }
}
pub struct RunState {
    pub execution: RunnerExecutionState,
    pub config: std::sync::Mutex<RunnerConfig>,
    pub dump_memory: i32,
    pub stats: std::sync::Mutex<MemoryStats>,
    pub test_count: std::sync::atomic::AtomicI32,
    pub test_failed: std::sync::atomic::AtomicI32,
    pub test_skipped: std::sync::atomic::AtomicI32,
}
pub unsafe fn update_stats(rt: *mut JSRuntime, filename: &[u8], state: &RunState) {
    let mut stats = std::mem::zeroed();
    JS_ComputeMemoryUsage(rt, &mut stats);
    let mut s = state.stats.lock().unwrap_or_else(|e| e.into_inner());
    let first = s.count == 0;
    s.count += 1;
    if first {
        s.all = std::ptr::read(&stats);
        s.avg = std::ptr::read(&stats);
        s.min = std::ptr::read(&stats);
        s.max = std::ptr::read(&stats);
        s.min_filename = filename.to_vec();
        s.max_filename = filename.to_vec();
    } else {
        if s.max.malloc_size < stats.malloc_size {
            s.max = std::ptr::read(&stats);
            s.max_filename = filename.to_vec();
        }
        if s.min.malloc_size > stats.malloc_size {
            s.min = std::ptr::read(&stats);
            s.min_filename = filename.to_vec();
        }
        macro_rules! update {
            ($field:ident) => {
                s.all.$field = s.all.$field.wrapping_add(stats.$field);
                s.avg.$field = s.all.$field / s.count as i64;
            };
        }
        update!(malloc_count);
        update!(malloc_size);
        update!(memory_used_count);
        update!(memory_used_size);
        update!(atom_count);
        update!(atom_size);
        update!(str_count);
        update!(str_size);
        update!(obj_count);
        update!(obj_size);
        update!(prop_count);
        update!(prop_size);
        update!(shape_count);
        update!(shape_size);
        update!(js_func_count);
        update!(js_func_size);
        update!(js_func_code_size);
        update!(js_func_pc2line_count);
        update!(js_func_pc2line_size);
        update!(c_func_count);
        update!(array_count);
        update!(fast_array_count);
        update!(fast_array_elements);
    }
}
fn runner_fatal(exit: i32, text: &[u8]) -> RunnerError {
    let mut message = b"run-test262: ".to_vec();
    message.extend_from_slice(text);
    message.push(b'\n');
    RunnerError { exit, message }
}
pub unsafe fn run_test_buf(
    tls: *mut ThreadLocalStorage,
    filename: &[u8],
    harness: &[u8],
    includes: &NameList,
    buf: &[u8],
    error_type: Option<&[u8]>,
    eval_flags: i32,
    is_negative: bool,
    is_async: bool,
    can_block: bool,
    state: &RunState,
) -> Result<i32, RunnerError> {
    use std::sync::atomic::Ordering::SeqCst;
    let rt = JS_NewRuntime();
    if rt.is_null() {
        return Err(runner_fatal(1, b"JS_NewRuntime failure"));
    }
    JS_SetRuntimeOpaque(rt, tls.cast());
    let ctx = JS_NewContext(rt);
    if ctx.is_null() {
        JS_FreeRuntime(rt);
        return Err(runner_fatal(1, b"JS_NewContext failure"));
    }
    let name = CString::new(filename).unwrap();
    JS_SetRuntimeInfo(rt, name.as_ptr());
    JS_SetCanBlock(rt, can_block as i32);
    JS_SetModuleLoaderFunc2(
        rt,
        None,
        Some(js_module_loader_test),
        None,
        name.as_ptr().cast_mut().cast(),
    );
    add_helpers(ctx);
    for path in &includes.array {
        if eval_file(ctx, harness, path, JS_EVAL_TYPE_GLOBAL, &state.execution)? != 0 {
            let mut text = b"error including ".to_vec();
            text.extend_from_slice(path);
            text.extend_from_slice(b" for ");
            text.extend_from_slice(filename);
            return Err(runner_fatal(1, &text));
        }
    }
    let ret = (eval_buf(
        ctx,
        buf,
        name.as_ptr(),
        true,
        is_negative,
        error_type,
        &state.execution,
        eval_flags,
        is_async,
    ) != 0) as i32;
    if state.dump_memory != 0 {
        update_stats(rt, filename, state);
    }
    js_agent_free(ctx);
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
    state.test_count.fetch_add(1, SeqCst);
    if ret != 0 {
        state.test_failed.fetch_add(1, SeqCst);
        runner_write(state.execution.outfile, b"  FAILED\n");
    }
    Ok(ret)
}
// run-test262.c:1772..2017. Preparation lives in prepare_test, from this same
// original function; each selected strictness mode runs a fresh original runtime.
pub unsafe fn run_test(
    tls: *mut ThreadLocalStorage,
    filename: &[u8],
    index: i32,
    state: &RunState,
) -> Result<i32, RunnerError> {
    use std::sync::atomic::Ordering::SeqCst;
    let buf = load_file(filename)?;
    let test = {
        let mut config = state.config.lock().unwrap_or_else(|e| e.into_inner());
        prepare_test(&mut config, filename, &buf)
    };
    runner_write(runner_stdout(), &test.diagnostics);
    if !test.warnings.is_empty() {
        libc::fflush(runner_stdout());
        runner_write(runner_stderr(), &test.warnings);
    }
    if !state.execution.outfile.is_null() {
        if let Some(line) = test_report_line(&test, filename, index) {
            runner_write(state.execution.outfile, &line);
        }
    }
    if test.skip || !test.use_strict && !test.use_nostrict {
        state.test_skipped.fetch_add(1, SeqCst);
        return Ok(-2);
    }
    extern "C" {
        fn clock() -> libc::clock_t;
    }
    let start = clock();
    let flags = if test.is_module {
        JS_EVAL_TYPE_MODULE
    } else {
        JS_EVAL_TYPE_GLOBAL
    };
    let mut ret = 0;
    if test.use_nostrict {
        ret = run_test_buf(
            tls,
            filename,
            &test.harness,
            &test.include_list,
            &buf,
            test.error_type.as_deref(),
            flags,
            test.is_negative,
            test.is_async,
            test.can_block,
            state,
        )?;
    }
    if test.use_strict {
        ret |= run_test_buf(
            tls,
            filename,
            &test.harness,
            &test.include_list,
            &buf,
            test.error_type.as_deref(),
            flags | JS_EVAL_FLAG_STRICT,
            test.is_negative,
            test.is_async,
            test.can_block,
            state,
        )?;
    }
    let elapsed = clock().wrapping_sub(start);
    if !state.execution.outfile.is_null() && index >= 0 && elapsed as i64 >= crate::quickjs_libc::CLOCKS_PER_SEC / 10 {
        runner_write(
            state.execution.outfile,
            format!(" time: {} ms\n", elapsed as i64 * 1000 / crate::quickjs_libc::CLOCKS_PER_SEC).as_bytes(),
        );
    }
    Ok(ret)
}
// run-test262.c:2020..2090. test262-harness/eshost already supplies its harness.
pub unsafe fn run_test262_harness_test(
    tls: *mut ThreadLocalStorage,
    filename: &[u8],
    is_module: bool,
    can_block: bool,
) -> Result<i32, RunnerError> {
    set_test262_outfile(runner_stdout());
    let rt = JS_NewRuntime();
    if rt.is_null() {
        return Err(runner_fatal(1, b"JS_NewRuntime failure"));
    }
    JS_SetRuntimeOpaque(rt, tls.cast());
    let ctx = JS_NewContext(rt);
    if ctx.is_null() {
        JS_FreeRuntime(rt);
        return Err(runner_fatal(1, b"JS_NewContext failure"));
    }
    let name = CString::new(filename).unwrap();
    JS_SetRuntimeInfo(rt, name.as_ptr());
    JS_SetCanBlock(rt, can_block as i32);
    JS_SetModuleLoaderFunc2(
        rt,
        None,
        Some(js_module_loader_test),
        None,
        name.as_ptr().cast_mut().cast(),
    );
    add_helpers(ctx);
    let mut buf = load_file(filename)?;
    let len = buf.len();
    buf.push(0);
    let res = JS_Eval(
        ctx,
        buf.as_ptr().cast(),
        len,
        name.as_ptr(),
        if is_module {
            JS_EVAL_TYPE_MODULE
        } else {
            JS_EVAL_TYPE_GLOBAL
        },
    );
    let mut ret_code = 0;
    if JS_IsException(res) != 0 {
        crate::quickjs_libc::js_std_dump_error(ctx);
        ret_code = 1;
    } else {
        let promise = if is_module {
            res
        } else {
            JS_FreeValue(ctx, res);
            JS_UNDEFINED
        };
        loop {
            let ret = JS_ExecutePendingJob(JS_GetRuntime(ctx), std::ptr::null_mut());
            if ret < 0 {
                crate::quickjs_libc::js_std_dump_error(ctx);
                ret_code = 1;
            } else if ret == 0 {
                break;
            }
        }
        if is_module && JS_PromiseState(ctx, promise) == JS_PROMISE_REJECTED {
            JS_Throw(ctx, JS_PromiseResult(ctx, promise));
            crate::quickjs_libc::js_std_dump_error(ctx);
            ret_code = 1;
        }
        JS_FreeValue(ctx, promise);
    }
    js_agent_free(ctx);
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
    Ok(ret_code)
}

// run-test262.c:136..139,236..268. Original host output/termination helpers.
pub fn atomic_inc(value: &std::sync::atomic::AtomicI32) {
    value.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}
pub unsafe fn warning(message: &[u8]) {
    libc::fflush(runner_stdout());
    runner_write(runner_stderr(), b"run-test262: ");
    runner_write(runner_stderr(), message);
    runner_write(runner_stderr(), b"\n");
}
pub unsafe fn fatal(exit: i32, message: &[u8]) -> ! {
    warning(message);
    std::process::exit(exit)
}
pub unsafe fn perror_exit(exit: i32, name: &[u8]) -> ! {
    libc::fflush(runner_stdout());
    runner_write(runner_stderr(), b"run-test262: ");
    let name = CString::new(name).unwrap();
    libc::perror(name.as_ptr());
    std::process::exit(exit)
}
pub unsafe fn help() -> ! {
    runner_write(runner_stdout(), runner_help().as_bytes());
    std::process::exit(1)
}
