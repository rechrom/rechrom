use super::*;
fn num(o: &mut Vec<u8>, v: u32) {
    o.extend(v.to_le_bytes());
}
fn rng(s: &mut u32) -> u32 {
    *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
    *s
}
unsafe fn range(o: &mut Vec<u8>, cr: &CharRange, ret: i32) {
    num(o, ret as u32);
    num(o, cr.len as u32);
    num(o, cr.size as u32);
    for i in 0..cr.len {
        num(o, *cr.points.add(i as usize));
    }
}
unsafe fn callback(opaque: *mut c_void, buf: *const u32, len: i32) {
    let o = &mut *opaque.cast::<Vec<u8>>();
    num(o, len as u32);
    for i in 0..len {
        num(o, *buf.add(i as usize));
    }
}
unsafe fn names(o: &mut Vec<u8>, table: &[u8], kind: i32) {
    let mut p = 0;
    while table[p] != 0 {
        let mut q = p;
        while table[q] != 0 && table[q] != b',' {
            q += 1;
        }
        let name = std::ffi::CString::new(&table[p..q]).unwrap();
        let mut cr = CharRange::default();
        cr_init(&mut cr, ptr::null_mut(), None);
        let ret = match kind {
            0 => unicode_script(&mut cr, name.as_ptr(), 0),
            1 => unicode_script(&mut cr, name.as_ptr(), 1),
            2 => unicode_general_category(&mut cr, name.as_ptr()),
            3 => unicode_prop(&mut cr, name.as_ptr()),
            _ => {
                unicode_sequence_prop(name.as_ptr(), callback, (o as *mut Vec<u8>).cast(), &mut cr)
            }
        };
        range(o, &cr, ret);
        cr_free(&cr);
        p = q + 1;
    }
}
struct Allocator {
    calls: i32,
    fail: i32,
    last: *mut c_void,
}
unsafe fn failrealloc(opaque: *mut c_void, p: *mut c_void, n: usize) -> *mut c_void {
    let a = &mut *opaque.cast::<Allocator>();
    a.calls += 1;
    if n != 0 && a.calls == a.fail {
        return ptr::null_mut();
    }
    let r = dbuf_default_realloc(ptr::null_mut(), p, n);
    if n != 0 {
        a.last = r;
    } else if p == a.last {
        a.last = ptr::null_mut();
    }
    r
}
unsafe fn dump() -> Vec<u8> {
    let mut o = Vec::new();
    let mut state = 0x12345678;
    for c in 0..=0x10ffff {
        for type_ in 0..3 {
            let mut res = [0xa5a5a5a5; 3];
            num(&mut o, lre_case_conv(&mut res, c, type_) as u32);
            for x in res {
                num(&mut o, x);
            }
        }
        for x in [
            lre_canonicalize(c, 0),
            lre_canonicalize(c, 1),
            lre_is_cased(c),
            lre_is_case_ignorable(c),
            lre_is_id_start(c),
            lre_is_id_continue(c),
            lre_is_space_non_ascii(c),
            lre_is_space(c),
            lre_js_is_ident_first(c),
            lre_js_is_ident_next(c),
            unicode_get_cc(c),
        ] {
            num(&mut o, x as u32);
        }
        for compat in 0..2 {
            let mut res = [0; 18];
            let n = unicode_decomp_char(&mut res, c, compat);
            num(&mut o, n as u32);
            for i in 0..n {
                num(&mut o, res[i as usize]);
            }
        }
    }
    for c in 0..=0x10ffffu32 {
        for type_ in 0..4 {
            let mut out = ptr::null_mut();
            let n = unicode_normalize(&mut out, &c, 1, type_, ptr::null_mut(), None);
            num(&mut o, n as u32);
            for i in 0..n {
                num(&mut o, *out.add(i as usize));
            }
            dbuf_default_realloc(ptr::null_mut(), out.cast(), 0);
        }
    }
    for idx in unicode_comp_table {
        let d_idx = (idx as u32) >> 6;
        let offset = (idx as u32) & 63;
        let v = unicode_decomp_table1[d_idx as usize];
        let code = v >> 14;
        let len = (v >> 7) & 127;
        let type_ = (v >> 1) & 63;
        let mut pair = [0; 2];
        unicode_decomp_entry(&mut pair, code + offset, d_idx as usize, code, len, type_);
        num(&mut o, compose_pair(pair[0], pair[1]));
    }
    for _ in 0..10000 {
        let mut src = [0; 16];
        let len = (rng(&mut state) % 17) as usize;
        for i in 0..len {
            let a = rng(&mut state);
            src[i] = if a % 4 == 0 {
                0x300 + a % 112
            } else if a % 4 == 1 {
                0x1100 + a % 256
            } else {
                a % 0x110000
            };
        }
        for type_ in 0..4 {
            let mut out = ptr::null_mut();
            let n = unicode_normalize(
                &mut out,
                src.as_ptr(),
                len as i32,
                type_,
                ptr::null_mut(),
                None,
            );
            num(&mut o, n as u32);
            for i in 0..n {
                num(&mut o, *out.add(i as usize));
            }
            dbuf_default_realloc(ptr::null_mut(), out.cast(), 0);
        }
    }
    names(&mut o, &unicode_script_name_table, 0);
    names(&mut o, &unicode_script_name_table, 1);
    names(&mut o, &unicode_gc_name_table, 2);
    names(&mut o, &unicode_prop_name_table, 3);
    names(&mut o, &unicode_sequence_prop_name_table, 4);
    for i in 0..256 {
        for x in [
            lre_ctype_bits[i] as i32,
            lre_is_space_byte(i as u8),
            lre_is_id_start_byte(i as u8),
            lre_is_id_continue_byte(i as u8),
            lre_is_word_byte(i as u8),
        ] {
            num(&mut o, x as u32);
        }
    }
    for _ in 0..1000 {
        let (mut a, mut b, mut v) = ([0; 16], [0; 16], 0);
        for i in 0..16 {
            v += rng(&mut state) % 100;
            a[i] = v;
        }
        v = 0;
        for i in 0..16 {
            v += rng(&mut state) % 100;
            b[i] = v;
        }
        for op in 0..4 {
            let mut cr = CharRange::default();
            cr_init(&mut cr, ptr::null_mut(), None);
            let ret = cr_op(&mut cr, a.as_ptr(), 16, b.as_ptr(), 16, op);
            range(&mut o, &cr, ret);
            let ret = cr_invert(&mut cr);
            range(&mut o, &cr, ret);
            let ret = cr_op1(&mut cr, a.as_ptr(), 16, op);
            range(&mut o, &cr, ret);
            cr_free(&cr);
        }
        for unicode in 0..2 {
            let mut cr = CharRange::default();
            cr_init(&mut cr, ptr::null_mut(), None);
            for i in (0..16).step_by(2) {
                cr_add_interval(&mut cr, a[i], a[i + 1]);
            }
            let ret = cr_regexp_canonicalize(&mut cr, unicode);
            range(&mut o, &cr, ret);
            cr_free(&cr);
        }
    }
    for unicode in 0..2 {
        let mut cr = CharRange::default();
        cr_init(&mut cr, ptr::null_mut(), None);
        cr_add_interval(&mut cr, 0, 0x110000);
        let ret = cr_regexp_canonicalize(&mut cr, unicode);
        range(&mut o, &cr, ret);
        cr_free(&cr);
    }
    for fail in 1..40 {
        let mut cr = CharRange::default();
        let mut a = Allocator {
            calls: 0,
            fail,
            last: ptr::null_mut(),
        };
        cr_init(
            &mut cr,
            (&mut a as *mut Allocator).cast(),
            Some(failrealloc),
        );
        let ret = unicode_prop(&mut cr, c"Alphabetic".as_ptr());
        range(&mut o, &cr, ret);
        cr_free(&cr);
        num(&mut o, a.calls as u32);
    }
    for fail in 1..5 {
        let src = [0xfb03, 0xac01, 0x1f82, 0x301];
        let mut out = ptr::null_mut();
        let mut a = Allocator {
            calls: 0,
            fail,
            last: ptr::null_mut(),
        };
        let n = unicode_normalize(
            &mut out,
            src.as_ptr(),
            4,
            UNICODE_NFKC,
            (&mut a as *mut Allocator).cast(),
            Some(failrealloc),
        );
        num(&mut o, n as u32);
        num(&mut o, (!out.is_null()) as u32);
        for i in 0..n {
            num(&mut o, *out.add(i as usize));
        }
        if !a.last.is_null() {
            dbuf_default_realloc(ptr::null_mut(), a.last, 0);
        }
        num(&mut o, a.calls as u32);
    }
    o
}
#[test]
fn entire_unicode_algorithm_matches_official_c() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = std::env::temp_dir().join(format!("quickjs-unicode-oracle-{}", std::process::id()));
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(root.join("../../vendor/quickjs-2026-06-04"))
        .arg(root.join("tests/unicode_oracle.c"))
        .arg(root.join("../../vendor/quickjs-2026-06-04/cutils.c"))
        .args(["-o"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut failure = std::process::Command::new(&file)
        .arg("--script-allocation-failure")
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    assert_stalls(&mut failure);
    let mut failure = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "libunicode::tests::script_allocation_failure_probe",
        ])
        .env("QUICKJS_SCRIPT_FAILURE_PROBE", "1")
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    assert_stalls(&mut failure);
    let c = std::process::Command::new(&file).output().unwrap();
    let _ = std::fs::remove_file(file);
    assert!(
        c.status.success(),
        "C oracle crashed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let rust = unsafe { dump() };
    assert_eq!(c.stdout.len(), rust.len(), "serialized lengths differ");
    if let Some(i) = c.stdout.iter().zip(&rust).position(|(a, b)| a != b) {
        panic!(
            "C/Rust mismatch at byte {i}: C={}, Rust={}",
            c.stdout[i], rust[i]
        );
    }
    eprintln!("Unicode official C parity: {} compared bytes", rust.len());
}
fn assert_stalls(child: &mut std::process::Child) {
    let until = std::time::Instant::now() + std::time::Duration::from_millis(300);
    while std::time::Instant::now() < until {
        assert!(
            child.try_wait().unwrap().is_none(),
            "source failure branch unexpectedly returned"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}
#[test]
fn script_allocation_failure_probe() {
    if std::env::var_os("QUICKJS_SCRIPT_FAILURE_PROBE").is_none() {
        return;
    }
    unsafe {
        let mut cr = CharRange::default();
        let mut a = Allocator {
            calls: 0,
            fail: 1,
            last: ptr::null_mut(),
        };
        cr_init(
            &mut cr,
            (&mut a as *mut Allocator).cast(),
            Some(failrealloc),
        );
        unicode_script(&mut cr, c"Latin".as_ptr(), 0);
    }
    panic!("expected upstream non-returning failure path");
}
