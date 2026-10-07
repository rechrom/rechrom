// run-test262.c:2196..2315. Byte argv and the original two-pass option scan.
pub struct RunnerOptions {
    pub config: RunnerConfig,
    pub optind: usize,
    pub is_dir_list: bool,
    pub only_check_errors: bool,
    pub is_test262_harness: bool,
    pub is_module: bool,
    pub can_block: bool,
    pub count_skipped_features: bool,
    pub dump_memory: i32,
    pub show_timings: bool,
    pub update_errors: bool,
    pub compact: bool,
    pub nthreads: i32,
    pub slow_test_threshold: i32,
    pub diagnostics: Vec<u8>,
}
pub struct RunnerCliError {
    pub error: RunnerError,
    pub help: bool,
}
impl From<RunnerError> for RunnerCliError {
    fn from(error: RunnerError) -> Self {
        Self { error, help: false }
    }
}
pub fn runner_help() -> &'static str {
    "run-test262 version 2026-06-04\nusage: run-test262 [options] {-f file ... | [dir_list] [index range]}\n-h             help\n-a             run tests in strict and nostrict modes\n-m             print memory usage summary\n-n             use new style harness\n-N             run test prepared by test262-harness+eshost\n-s             run tests in strict mode, skip @nostrict tests\n-E             only run tests from the error file\n-C             use compact progress indicator\n-t             show timings\n-u             update error file\n-v             verbose: output error messages\n-D duration    display tests taking more than 'duration' ms\n-T threads     number of parallel threads\n-c file        read configuration from 'file'\n-d dir         run all test files in directory tree 'dir'\n-e file        load the known errors from 'file'\n-f file        execute single test from 'file'\n-r file        set the report file name (default=none)\n-x file        exclude tests listed in 'file'\n--no-can-block set [[CanBlock]] to false (Atomics.wait will throw)\n"
}
pub fn get_opt_arg<'a>(option: &[u8], arg: Option<&'a Vec<u8>>) -> Result<&'a [u8], RunnerError> {
    arg.map(|a| a.as_slice()).ok_or_else(|| {
        let mut text = b"missing argument for option ".to_vec();
        text.extend_from_slice(option);
        runner_fatal(2, &text)
    })
}
pub fn parse_runner_options(argv: &[Vec<u8>]) -> Result<RunnerOptions, RunnerCliError> {
    let mut o = RunnerOptions {
        config: RunnerConfig::default(),
        optind: 1,
        is_dir_list: true,
        only_check_errors: false,
        is_test262_harness: false,
        is_module: false,
        can_block: true,
        count_skipped_features: false,
        dump_memory: 0,
        show_timings: false,
        update_errors: false,
        compact: { #[cfg(unix)] { unsafe { libc::isatty(2) == 0 } } #[cfg(windows)] { false } },
        nthreads: 0,
        slow_test_threshold: 0,
        diagnostics: Vec::new(),
    };
    let mut i = 1;
    let mut ignore = &b""[..];
    while i < argv.len() {
        let arg = &argv[i];
        if !arg.starts_with(b"-") {
            break;
        }
        i += 1;
        if substring(b"-c -d -e -x -f -r -E -D -T", arg, 0).is_some() {
            i += 1;
        }
        if substring(b"-d -f", arg, 0).is_some() {
            ignore = b"testdir";
        }
    }
    while o.optind < argv.len() {
        let arg = &argv[o.optind];
        if !arg.starts_with(b"-") {
            break;
        }
        o.optind += 1;
        match arg.as_slice() {
            b"-h" => {
                return Err(RunnerCliError {
                    error: RunnerError {
                        exit: 1,
                        message: Vec::new(),
                    },
                    help: true,
                })
            }
            b"-m" => o.dump_memory += 1,
            b"-n" => o.config.new_style = true,
            b"-s" => o.config.test_mode = TestMode::Strict,
            b"-a" => o.config.test_mode = TestMode::All,
            b"-t" => o.show_timings = true,
            b"-u" => o.update_errors = true,
            b"-v" => o.config.verbose = true,
            b"-C" => o.compact = true,
            b"-c" | b"-d" | b"-e" | b"-x" | b"-r" | b"-D" | b"-T" => {
                let value = get_opt_arg(arg, argv.get(o.optind))?;
                o.optind += 1;
                match arg.as_slice() {
                    b"-c" => load_config(&mut o.config, value, ignore, &mut o.diagnostics)?,
                    b"-d" => enumerate_tests(&mut o.config.test_list, value),
                    b"-e" => o.config.error_filename = Some(value.to_vec()),
                    b"-x" => namelist_load(&mut o.config.exclude_list, value)
                        .map_err(|e| RunnerError::io(value, e))?,
                    b"-r" => o.config.report_filename = Some(value.to_vec()),
                    b"-D" => o.slow_test_threshold = runner_atoi(value),
                    b"-T" => o.nthreads = runner_atoi(value),
                    _ => unreachable!(),
                }
            }
            b"-f" => o.is_dir_list = false,
            b"-E" => o.only_check_errors = true,
            b"-N" => o.is_test262_harness = true,
            b"--module" => o.is_module = true,
            b"--no-can-block" => o.can_block = false,
            b"--count_skipped_features" => o.count_skipped_features = true,
            _ => {
                let mut text = b"unknown option: ".to_vec();
                text.extend_from_slice(arg);
                return Err(runner_fatal(1, &text).into());
            }
        }
    }
    if o.optind >= argv.len() && o.config.test_list.array.is_empty() {
        return Err(RunnerCliError {
            error: RunnerError {
                exit: 1,
                message: Vec::new(),
            },
            help: true,
        });
    }
    Ok(o)
}
// run-test262.c:162..210, Linux physical core parser (1023-byte fgets chunks).
#[cfg(target_os = "linux")]
pub fn get_cpu_info_physical_cores() -> i32 {
    let data = match fs::read("/proc/cpuinfo") {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let mut cores = 0i32;
    let mut physical = -1i32;
    let mut p = 0;
    while p < data.len() {
        let max = (p + 1023).min(data.len());
        let end = data[p..max]
            .iter()
            .position(|&b| b == b'\n')
            .map(|n| p + n + 1)
            .unwrap_or(max);
        let mut line = &data[p..end];
        p = end;
        line = &line[..line.iter().position(|&c| c == 0).unwrap_or(line.len())];
        while line.last().is_some_and(|&c| c_space(c)) {
            line = &line[..line.len() - 1];
        }
        if line.starts_with(b"#") {
            continue;
        }
        let Some(colon) = line.iter().position(|&b| b == b':') else {
            continue;
        };
        let mut field = &line[..colon];
        while field.last().is_some_and(|&c| c_space(c)) {
            field = &field[..field.len() - 1];
        }
        let value = CString::new(str_strip(&line[colon + 1..])).unwrap();
        let n = unsafe { libc::strtol(value.as_ptr(), std::ptr::null_mut(), 0) as i32 };
        if field == b"cpu cores" {
            if cores == 0 {
                cores = n;
            }
        } else if field == b"physical id" {
            physical = physical.max(n);
        }
    }
    if cores <= 0 || physical < 0 {
        -1
    } else {
        cores.wrapping_mul(physical.wrapping_add(1))
    }
}
pub fn cpu_count() -> i32 {
    #[cfg(windows)]
    unsafe {
        use crate::quickjs_libc_windows::{GetCurrentProcess, GetProcessAffinityMask};
        let mut process_mask = 0usize;
        let mut system_mask = 0usize;
        if GetProcessAffinityMask(GetCurrentProcess(), &mut process_mask, &mut system_mask) != 0 {
            process_mask.count_ones() as i32
        } else { 0 }
    }

    #[cfg(target_os = "linux")]
    {
        { let count = get_cpu_info_physical_cores(); if count < 1 { unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) as i32 } } else { count } }
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) as i32 }
    }
}
