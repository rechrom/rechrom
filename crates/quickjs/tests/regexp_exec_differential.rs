use super::*;
struct Host {
    calls: i32,
    fail: i32,
    polls: i32,
    timeout: i32,
}
unsafe fn stack_check(_o: *mut c_void, _n: usize) -> i32 {
    0
}
unsafe fn timeout(o: *mut c_void) -> i32 {
    let h = &mut *o.cast::<Host>();
    h.polls += 1;
    h.timeout
}
unsafe fn realloc(o: *mut c_void, p: *mut c_void, n: usize) -> *mut c_void {
    let h = &mut *o.cast::<Host>();
    h.calls += 1;
    if n != 0 && h.calls == h.fail {
        return ptr::null_mut();
    }
    dbuf_default_realloc(ptr::null_mut(), p, n)
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
fn official_c_bytecode_executes_identically_in_rust() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = std::env::temp_dir().join(format!("quickjs-regexp-oracle-{}", std::process::id()));
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(root.join("../../vendor/quickjs-2026-06-04"))
        .arg(root.join("tests/regexp_exec_oracle.c"))
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
        .arg(root.join("tests/regexp_exec_cases.tsv"))
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
    let mut executed = 0usize;
    let cases: Vec<_> = include_str!("regexp_exec_cases.tsv").lines().collect();
    let unhex = |s: &str| {
        s.as_bytes()
            .chunks_exact(2)
            .map(|b| u8::from_str_radix(core::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect::<Vec<u8>>()
    };
    for record in 0..include_str!("regexp_exec_cases.tsv").lines().count() {
        assert_eq!(r.num(), record as u32);
        let bc_len = r.num() as usize;
        let expected_bc;
        let expected_error;
        if bc_len == 0 {
            let n = r.num() as usize;
            expected_error = r.bytes(n);
            expected_bc = &[][..];
        } else {
            expected_bc = r.bytes(bc_len);
            expected_error = &[][..];
        }
        let fields: Vec<_> = cases[record].split('\t').collect();
        let pattern = std::ffi::CString::new(unhex(fields[1])).unwrap();
        let mut h = Host {
            calls: 0,
            fail: 0,
            polls: 0,
            timeout: 0,
        };
        let mut host = LREHost {
            opaque: (&mut h as *mut Host).cast(),
            check_stack_overflow: stack_check,
            check_timeout: timeout,
            realloc,
        };
        let mut len = 0;
        let mut error = [0 as c_char; 128];
        let rust_bc = unsafe {
            lre_compile(
                &mut len,
                error.as_mut_ptr(),
                128,
                pattern.as_ptr(),
                pattern.as_bytes().len(),
                fields[0].parse().unwrap(),
                (&mut host as *mut LREHost).cast(),
            )
        };
        let bc = if rust_bc.is_null() {
            Vec::new()
        } else {
            unsafe { core::slice::from_raw_parts(rust_bc, len as usize).to_vec() }
        };
        unsafe {
            assert_eq!(
                core::ffi::CStr::from_ptr(error.as_ptr()).to_bytes(),
                expected_error,
                "compile error record={record}, pattern={pattern:?}, flags={}",
                fields[0]
            );
            if !rust_bc.is_null() {
                lre_realloc((&mut host as *mut LREHost).cast(), rust_bc.cast(), 0);
            }
        }
        assert_eq!(
            &bc, expected_bc,
            "compiled bytecode record={record}, pattern={pattern:?}, flags={}",
            fields[0]
        );
        if bc_len == 0 {
            continue;
        }
        for mode in 0..2 {
            assert_eq!(r.num(), mode);
            let length = r.num() as usize;
            let bytes = r.bytes(length << mode);
            // UTF-16 requires aligned storage; preserve native-endian original units.
            let subject: Vec<u16> = if mode == 1 {
                bytes
                    .chunks_exact(2)
                    .map(|b| u16::from_ne_bytes([b[0], b[1]]))
                    .collect()
            } else {
                Vec::new()
            };
            let data = if mode == 1 {
                subject.as_ptr().cast::<u8>()
            } else {
                bytes.as_ptr()
            };
            for index in 0..=length {
                for fail in 0..3 {
                    let expected = r.num() as i32;
                    let expected_calls = r.num() as i32;
                    let expected_polls = r.num() as i32;
                    let mut h = Host {
                        calls: 0,
                        fail,
                        polls: 0,
                        timeout: 1,
                    };
                    let mut host = LREHost {
                        opaque: (&mut h as *mut Host).cast(),
                        check_stack_overflow: stack_check,
                        check_timeout: timeout,
                        realloc,
                    };
                    unsafe {
                        let mut captures =
                            vec![ptr::null_mut(); lre_get_alloc_count(bc.as_ptr()) as usize];
                        let ret = lre_exec(
                            captures.as_mut_ptr(),
                            bc.as_ptr(),
                            data,
                            index as i32,
                            length as i32,
                            mode as i32,
                            (&mut host as *mut LREHost).cast(),
                        );
                        assert_eq!(
                            (ret, h.calls, h.polls),
                            (expected, expected_calls, expected_polls),
                            "record={record}, mode={mode}, index={index}, fail={fail}"
                        );
                        for i in 0..2 * lre_get_capture_count(bc.as_ptr()) as usize {
                            let expected = r.num();
                            let actual = if captures[i].is_null() {
                                u32::MAX
                            } else {
                                (captures[i] as usize).wrapping_sub(data as usize) as u32
                            };
                            assert_eq!(actual,expected,"capture {i}, record={record}, mode={mode}, index={index}, fail={fail}");
                        }
                    }
                    executed += 1;
                }
            }
        }
    }
    let escapes = [
        "b",
        "f",
        "n",
        "r",
        "t",
        "v",
        "x00",
        "xFE",
        "xG1",
        "x1",
        "u0041",
        "uD83D\\uDE00",
        "uD83D\\uXXXX",
        "u{10ffff}",
        "u{110000}",
        "u{}",
        "u{00000000000000001}",
        "0",
        "00",
        "07",
        "377",
        "400",
        "777",
        "09",
        "q",
        "",
        "cA",
    ];
    for mode in 0..3 {
        for escape in escapes {
            let text = std::ffi::CString::new(escape).unwrap();
            let bytes = text.as_bytes_with_nul();
            let mut p = bytes.as_ptr();
            unsafe {
                assert_eq!(lre_parse_escape(&mut p, mode), r.num() as i32);
                assert_eq!(p.offset_from(bytes.as_ptr()) as u32, r.num());
            }
        }
    }
    assert_eq!(r.pos, c.stdout.len());
    eprintln!(
        "regexp compiler + execution C parity: {} compilations, {executed} runs, {} fixture bytes",
        cases.len(),
        c.stdout.len()
    );
}
