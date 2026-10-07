// run-test262.c:2092..2150. Same 50 ms condition wait and final snapshot.
pub unsafe fn pthread_cond_timedwait2(
    cond: *mut libc::pthread_cond_t,
    mutex: *mut libc::pthread_mutex_t,
    timeout: i32,
) -> i32 {
    let mut ts = std::mem::zeroed();
    libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts);
    ts.tv_sec += timeout as libc::time_t / 1000;
    ts.tv_nsec += (timeout as libc::c_long % 1000) * 1_000_000;
    if ts.tv_nsec >= 1_000_000_000 {
        ts.tv_nsec -= 1_000_000_000;
        ts.tv_sec += 1;
    }
    libc::pthread_cond_timedwait(cond, mutex, &ts)
}
struct RunnerProgress {
    exit: std::sync::Mutex<bool>,
    cond: std::sync::Condvar,
}
fn show_progress(
    state: std::sync::Arc<RunState>,
    progress: std::sync::Arc<RunnerProgress>,
    compact: bool,
) {
    use std::sync::atomic::Ordering::SeqCst;
    let mut exit = progress.exit.lock().unwrap_or_else(|e| e.into_inner());
    let mut last_skipped = 0;
    let mut last_failed = 0;
    let mut dots = 0;
    loop {
        exit = progress
            .cond
            .wait_timeout(exit, std::time::Duration::from_millis(50))
            .unwrap_or_else(|e| e.into_inner())
            .0;
        let failed = state.test_failed.load(SeqCst);
        let count = state.test_count.load(SeqCst);
        let skipped = state.test_skipped.load(SeqCst);
        let mut bytes = Vec::new();
        if compact {
            let mut c = b'.';
            if skipped > last_skipped {
                c = b'-';
            }
            if failed > last_failed {
                c = b'!';
            }
            last_skipped = skipped;
            last_failed = failed;
            bytes.push(c);
            dots += 1;
            if *exit || dots % 60 == 0 {
                bytes.extend_from_slice(format!(" {failed}/{count}/{skipped}\n").as_bytes());
            }
        } else {
            bytes.extend_from_slice(format!("{failed}/{count}/{skipped}\x1b[K\r").as_bytes());
        }
        unsafe {
            runner_write(runner_stderr(), &bytes);
            libc::fflush(runner_stderr());
        }
        if *exit {
            break;
        }
    }
}
// C:2154..2162. EXCLUDE wins over either index boundary.
pub fn include_exclude_or_skip(
    index: usize,
    tests: &NameList,
    excluded: &NameList,
    start: i32,
    stop: i32,
) -> i32 {
    if namelist_find(excluded, &tests.array[index]) >= 0 {
        1
    } else if (index as i32) < start || stop >= 0 && (index as i32) > stop {
        2
    } else {
        0
    }
}
// run-test262.c:2165..2193. Thread partition is prepared by the caller;
// each thread retains its own original TLS, test clocks and agent lifecycle.
pub unsafe fn run_test_dir_list(cases: Vec<(usize, Vec<u8>)>, s: &RunState, threshold: i32) {
    let mut tls = std::mem::zeroed();
    init_thread_local_storage(&mut tls);
    for (index, filename) in cases {
        let start = if threshold != 0 {
            get_clock_ms() as i32
        } else {
            0
        };
        if let Err(error) = run_test(&mut tls, &filename, index as i32, &s) {
            libc::fflush(runner_stdout());
            runner_write(runner_stderr(), &error.message);
            std::process::exit(error.exit);
        }
        if threshold != 0 {
            let elapsed = (get_clock_ms() as i32).wrapping_sub(start);
            if elapsed >= threshold {
                let mut line = b"\n".to_vec();
                line.extend_from_slice(&filename);
                line.extend_from_slice(format!(" ({elapsed} ms)\n").as_bytes());
                runner_write(runner_stderr(), &line);
            }
        }
    }
    free_thread_local_storage(&mut tls);
}
unsafe fn runner_open_file(name: &[u8], mode: &CStr) -> Result<*mut libc::FILE, RunnerError> {
    let filename = CString::new(name).unwrap();
    let file = libc::fopen(filename.as_ptr(), mode.as_ptr());
    if file.is_null() {
        Err(RunnerError::io(name, io::Error::last_os_error()))
    } else {
        Ok(file)
    }
}
unsafe fn runner_print_stream(opaque: *mut c_void, buffer: *const c_char, len: usize) -> usize {
    libc::fwrite(buffer.cast(), 1, len, opaque.cast())
}
unsafe fn dump_runner_memory(stats: &JSMemoryUsage) {
    let mut stream = QuickJSPrintStream {
        write_func: runner_print_stream,
        write_opaque: runner_stdout().cast(),
    };
    JS_DumpMemoryUsage(&mut stream, stats, std::ptr::null_mut());
}
// run-test262.c:2231..2529. Per-invocation state replaces C globals; execution,
// partitions, report rules, counters, progress and resource ownership are kept.
pub unsafe fn run_runner(argv: &[Vec<u8>]) -> Result<i32, RunnerCliError> {
    use std::{
        ptr,
        sync::{
            atomic::{AtomicI32, Ordering::SeqCst},
            Arc, Condvar, Mutex,
        },
    };
    let mut tls: ThreadLocalStorage = std::mem::zeroed();
    init_thread_local_storage(&mut tls);
    #[cfg(not(windows))]
    libc::setenv(c"TZ".as_ptr(), c"America/Los_Angeles".as_ptr(), 1);
    let mut options = parse_runner_options(argv)?;
    runner_write(runner_stdout(), &options.diagnostics);
    if options.is_test262_harness {
        // Original C dereferences argv[optind] when no test exists despite a
        // populated config list. Diagnose this undefined-input domain safely.
        let filename = argv
            .get(options.optind)
            .ok_or_else(|| runner_fatal(1, b"missing test filename"))?;
        let ret =
            run_test262_harness_test(&mut tls, filename, options.is_module, options.can_block)?;
        free_thread_local_storage(&mut tls);
        return Ok(ret);
    }
    if options.nthreads == 0 {
        options.nthreads = cpu_count();
        if options.nthreads >= 8 {
            options.nthreads -= 1;
        }
    }
    options.nthreads = options.nthreads.max(1);
    let mut error_file = None;
    let mut error_out = runner_stdout();
    if let Some(filename) = &options.config.error_filename {
        error_file = Some(load_file(filename)?);
        if options.only_check_errors {
            namelist_free(&mut options.config.test_list);
            namelist_add_from_error_file(
                &mut options.config.test_list,
                error_file.as_ref().unwrap(),
            );
        }
        if options.update_errors {
            error_file = None;
            error_out = runner_open_file(filename, c"w")?;
        }
    }
    let mut excluded_count = update_exclude_dirs(
        &mut options.config.test_list,
        &mut options.config.exclude_list,
        &mut options.config.exclude_dir_list,
    );
    extern "C" {
        fn clock() -> libc::clock_t;
    }
    let start_clock = clock();
    if options.count_skipped_features {
        let length = options
            .config
            .harness_skip_features
            .as_ref()
            .ok_or_else(|| runner_fatal(1, b"missing skip_features configuration"))?
            .len();
        options.config.harness_skip_features_count = Some(vec![0; length]);
    }
    let mut start_index = 0;
    let mut stop_index = -1;
    let outfile;
    if options.is_dir_list {
        if options.optind < argv.len()
            && !argv[options.optind]
                .first()
                .is_some_and(|c| c.is_ascii_digit())
        {
            let filename = &argv[options.optind];
            options.optind += 1;
            namelist_load(&mut options.config.test_list, filename)
                .map_err(|e| RunnerError::io(filename, e))?;
        }
        if options.optind < argv.len() {
            start_index = runner_atoi(&argv[options.optind]);
            options.optind += 1;
            if options.optind < argv.len() {
                stop_index = runner_atoi(&argv[options.optind]);
                options.optind += 1;
            }
        }
        outfile = match options.config.report_filename.as_deref() {
            None | Some(b"none") => ptr::null_mut(),
            _ if options.nthreads > 1 => ptr::null_mut(),
            Some(b"-") => runner_stdout(),
            Some(name) => runner_open_file(name, c"wb")?,
        };
        namelist_sort(&mut options.config.test_list);
        namelist_sort(&mut options.config.exclude_list);
    } else {
        outfile = runner_stdout();
    }
    set_test262_outfile(outfile);
    let mut selected = Vec::new();
    let mut skipped_count = 0;
    if options.is_dir_list {
        for index in 0..options.config.test_list.array.len() {
            match include_exclude_or_skip(
                index,
                &options.config.test_list,
                &options.config.exclude_list,
                start_index,
                stop_index,
            ) {
                1 => excluded_count += 1,
                2 => skipped_count += 1,
                _ => selected.push((index, options.config.test_list.array[index].clone())),
            }
        }
    }
    let state = Arc::new(RunState {
        execution: RunnerExecutionState {
            outfile,
            error_out,
            verbose: options.config.verbose,
            error_file,
            new_errors: AtomicI32::new(0),
            changed_errors: AtomicI32::new(0),
            fixed_errors: AtomicI32::new(0),
        },
        config: Mutex::new(options.config),
        dump_memory: options.dump_memory,
        stats: Mutex::new(MemoryStats::default()),
        test_count: AtomicI32::new(0),
        test_failed: AtomicI32::new(0),
        test_skipped: AtomicI32::new(skipped_count),
    });
    if options.is_dir_list {
        let progress = Arc::new(RunnerProgress {
            exit: Mutex::new(false),
            cond: Condvar::new(),
        });
        let s = state.clone();
        let p = progress.clone();
        let compact = options.compact;
        let reporter = std::thread::spawn(move || show_progress(s, p, compact));
        let mut threads = Vec::new();
        for thread_index in 0..options.nthreads as usize {
            let cases = selected
                .iter()
                .filter(|(index, _)| index % options.nthreads as usize == thread_index)
                .cloned()
                .collect::<Vec<_>>();
            let s = state.clone();
            let threshold = options.slow_test_threshold;
            // Rust native frames need more headroom; JS's original 1 MiB
            // runtime stack limit stays unchanged, as with agent/Worker hosts.
            let thread = std::thread::Builder::new()
                .stack_size(16 << 20)
                .spawn(move || unsafe { run_test_dir_list(cases, &s, threshold) })
                .map_err(|e| runner_fatal(1, e.to_string().as_bytes()))?;
            threads.push(thread);
        }
        for thread in threads {
            thread
                .join()
                .map_err(|_| runner_fatal(1, b"test thread panicked"))?;
        }
        *progress.exit.lock().unwrap_or_else(|e| e.into_inner()) = true;
        progress.cond.notify_one();
        reporter
            .join()
            .map_err(|_| runner_fatal(1, b"progress thread panicked"))?;
        if !outfile.is_null() && outfile != runner_stdout() {
            libc::fclose(outfile);
            set_test262_outfile(ptr::null_mut());
        }
    } else {
        for filename in &argv[options.optind..] {
            run_test(&mut tls, filename, -1, &state)?;
        }
    }
    let elapsed = clock().wrapping_sub(start_clock);
    if options.dump_memory != 0 {
        let stats = state.stats.lock().unwrap_or_else(|e| e.into_inner());
        if options.dump_memory > 1 && stats.count > 1 {
            let mut text = b"\nMininum memory statistics for ".to_vec();
            text.extend_from_slice(&stats.min_filename);
            text.extend_from_slice(b":\n\n");
            runner_write(runner_stdout(), &text);
            dump_runner_memory(&stats.min);
            let mut text = b"\nMaximum memory statistics for ".to_vec();
            text.extend_from_slice(&stats.max_filename);
            text.extend_from_slice(b":\n\n");
            runner_write(runner_stdout(), &text);
            dump_runner_memory(&stats.max);
        }
        runner_write(
            runner_stdout(),
            format!("\nAverage memory statistics for {} tests:\n\n", stats.count).as_bytes(),
        );
        dump_runner_memory(&stats.avg);
        runner_write(runner_stdout(), b"\n");
    }
    if options.count_skipped_features {
        let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
        let features = config.harness_skip_features.as_ref().unwrap();
        let counts = config.harness_skip_features_count.as_ref().unwrap();
        let mut displayed = false;
        for i in 0..features.len() {
            if counts[i] != 0 {
                if !displayed {
                    displayed = true;
                    runner_write(runner_stdout(), b"SKIPPED FEATURE                  COUNT\n");
                }
                let mut n = 0;
                let mut line = Vec::new();
                while n < 30 {
                    let c = byte(features, i + n);
                    if is_word_sep(c) {
                        break;
                    }
                    line.push(c);
                    n += 1;
                }
                line.resize(30, b' ');
                line.extend_from_slice(format!(" {:7}\n", counts[i]).as_bytes());
                runner_write(runner_stdout(), &line);
            }
        }
        runner_write(runner_stdout(), b"\n");
    }
    let new = state.execution.new_errors.load(SeqCst);
    let changed = state.execution.changed_errors.load(SeqCst);
    let fixed = state.execution.fixed_errors.load(SeqCst);
    if options.is_dir_list {
        let count = state.test_count.load(SeqCst);
        let failed = state.test_failed.load(SeqCst);
        let skipped = state.test_skipped.load(SeqCst);
        let mut line = format!(
            "Result: {failed}/{count} error{}",
            if count != 1 { "s" } else { "" }
        );
        if excluded_count != 0 {
            line.push_str(&format!(", {excluded_count} excluded"));
        }
        if skipped != 0 {
            line.push_str(&format!(", {skipped} skipped"));
        }
        if state.execution.error_file.is_some() {
            for (n, text) in [(new, "new"), (changed, "changed"), (fixed, "fixed")] {
                if n != 0 {
                    line.push_str(&format!(", {n} {text}"));
                }
            }
        }
        line.push('\n');
        runner_write(runner_stderr(), line.as_bytes());
        if options.show_timings {
            runner_write(
                runner_stderr(),
                format!(
                    "Total user time: {:.3}s (nthreads={})\n",
                    elapsed as f64 / crate::quickjs_libc::CLOCKS_PER_SEC as f64,
                    options.nthreads
                )
                .as_bytes(),
            );
        }
    }
    if error_out != runner_stdout() && !error_out.is_null() {
        libc::fclose(error_out);
    }
    free_thread_local_storage(&mut tls);
    Ok((new != 0 || changed != 0 || fixed != 0) as i32)
}
pub fn main_entry() -> i32 {
    let argv = match crate::quickjs_libc::args_bytes() {
        Ok(argv) => argv,
        Err(error) => { eprintln!("run-test262: {error}"); return 1; }
    };
    match unsafe { run_runner(&argv) } {
        Ok(code) => code,
        Err(error) => unsafe {
            libc::fflush(runner_stdout());
            runner_write(runner_stderr(), &error.error.message);
            if error.help {
                runner_write(runner_stdout(), runner_help().as_bytes());
            }
            error.error.exit
        },
    }
}
