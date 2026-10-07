use super::*;
struct Host {
    calls: i32,
    fail: i32,
    stack_calls: i32,
    stack_fail: i32,
    allocs: std::collections::HashSet<usize>,
}
unsafe fn stack_check(o: *mut c_void, _n: usize) -> i32 {
    let h = &mut *o.cast::<Host>();
    h.stack_calls += 1;
    (h.stack_calls == h.stack_fail) as i32
}
unsafe fn timeout(_o: *mut c_void) -> i32 {
    0
}
unsafe fn realloc(o: *mut c_void, p: *mut c_void, n: usize) -> *mut c_void {
    let h = &mut *o.cast::<Host>();
    h.calls += 1;
    if n != 0 && h.calls == h.fail {
        return ptr::null_mut();
    }
    let q = dbuf_default_realloc(ptr::null_mut(), p, n);
    if n == 0 || !q.is_null() {
        h.allocs.remove(&(p as usize));
    }
    if !q.is_null() {
        h.allocs.insert(q as usize);
    }
    q
}
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn num(&mut self) -> u32 {
        let v = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        v
    }
    fn bytes(&mut self, n: usize) -> &'a [u8] {
        let b = &self.data[self.pos..self.pos + n];
        self.pos += n;
        b
    }
}
#[test]
fn official_c_compile_syntax_limits_and_failures_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = std::env::temp_dir().join(format!(
        "quickjs-regexp-compile-oracle-{}",
        std::process::id()
    ));
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(root.join("../../vendor/quickjs-2026-06-04"))
        .arg(root.join("tests/regexp_compile_oracle.c"))
        .arg(root.join("../../vendor/quickjs-2026-06-04/cutils.c"))
        .arg(root.join("../../vendor/quickjs-2026-06-04/libunicode.c"))
        .arg("-o")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&file)
        .arg(root.join("tests/regexp_compile_cases.tsv"))
        .output()
        .unwrap();
    let _ = std::fs::remove_file(file);
    assert!(
        c.status.success(),
        "C oracle crashed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut r = Reader {
        data: &c.stdout,
        pos: 0,
    };
    let mut count = 0;
    for (record, line) in include_str!("regexp_compile_cases.tsv").lines().enumerate() {
        let (options, text) = line.split_once('\t').unwrap();
        let options: Vec<i32> = options.split(',').map(|s| s.parse().unwrap()).collect();
        let mut pattern: Vec<u8> = text
            .as_bytes()
            .chunks_exact(2)
            .map(|b| u8::from_str_radix(core::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect();
        let pattern_len = pattern.len();
        pattern.extend_from_slice(&[0; 8]);
        assert_eq!(r.num(), record as u32);
        let expected_len = r.num();
        let expected_calls = r.num();
        let expected_stack_calls = r.num();
        let expected_allocs = r.num();
        let error_len = r.num();
        let expected_error = r.bytes(error_len as usize);
        let expected_bc = r.bytes(expected_len as usize);
        let mut h = Host {
            calls: 0,
            fail: options[1],
            stack_calls: 0,
            stack_fail: options[2],
            allocs: Default::default(),
        };
        let mut host = LREHost {
            opaque: (&mut h as *mut Host).cast(),
            check_stack_overflow: stack_check,
            check_timeout: timeout,
            realloc,
        };
        let mut error = [0 as c_char; 128];
        let mut len = 0;
        unsafe {
            let bc = lre_compile(
                &mut len,
                error.as_mut_ptr(),
                options[3],
                pattern.as_ptr().cast(),
                pattern_len,
                options[0],
                (&mut host as *mut LREHost).cast(),
            );
            assert_eq!(
                (
                    len as u32,
                    h.calls as u32,
                    h.stack_calls as u32,
                    h.allocs.len() as u32
                ),
                (
                    expected_len,
                    expected_calls,
                    expected_stack_calls,
                    expected_allocs
                ),
                "record={record}, options={options:?}, pattern={:?}",
                String::from_utf8_lossy(&pattern[..pattern_len])
            );
            assert_eq!(
                core::ffi::CStr::from_ptr(error.as_ptr()).to_bytes(),
                expected_error,
                "error record={record}, options={options:?}, pattern={:?}",
                String::from_utf8_lossy(&pattern[..pattern_len])
            );
            if !bc.is_null() {
                assert_eq!(
                    core::slice::from_raw_parts(bc, len as usize),
                    expected_bc,
                    "bytecode record={record}, options={options:?}, pattern={:?}",
                    String::from_utf8_lossy(&pattern[..pattern_len])
                );
            } else {
                assert_eq!(expected_len, 0);
            }
            // Match C's lifetime during comparison, then reclaim fixture-owned storage.
            for p in h.allocs {
                dbuf_default_realloc(ptr::null_mut(), p as *mut c_void, 0);
            }
        }
        count += 1;
    }
    assert_eq!(r.pos, c.stdout.len());
    eprintln!(
        "regexp compiler boundary C parity: {count} cases, {} fixture bytes",
        c.stdout.len()
    );
}

#[test]
fn failed_string_set_bytecode_allocation_returns_memory_error() {
    // Official C dereferences the failed allocation for this input. There is no
    // defined C result to mirror; Rust rejects it before patching the buffer.
    for pattern in [r"[\q{ab|abc|x|}]", r"[\q{ab|abc|}]|a"] {
        let pattern = std::ffi::CString::new(pattern).unwrap();
        let mut h = Host {
            calls: 0,
            fail: 1,
            stack_calls: 0,
            stack_fail: 0,
            allocs: Default::default(),
        };
        let mut host = LREHost {
            opaque: (&mut h as *mut Host).cast(),
            check_stack_overflow: stack_check,
            check_timeout: timeout,
            realloc,
        };
        let mut error = [0 as c_char; 128];
        let mut len = 123;
        unsafe {
            let bc = lre_compile(
                &mut len,
                error.as_mut_ptr(),
                128,
                pattern.as_ptr(),
                pattern.as_bytes().len(),
                LRE_FLAG_UNICODE_SETS,
                (&mut host as *mut LREHost).cast(),
            );
            assert!(bc.is_null());
            assert_eq!(len, 0);
            assert_eq!(
                core::ffi::CStr::from_ptr(error.as_ptr()).to_bytes(),
                b"out of memory"
            );
            assert!(h.allocs.is_empty());
        }
    }
}
