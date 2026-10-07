//! Loaded as a unit-test module to also exercise the translated static helpers.
use super::*;
fn num(o: &mut Vec<u8>, v: u32) {
    o.extend(v.to_le_bytes());
}
fn num64(o: &mut Vec<u8>, v: u64) {
    o.extend(v.to_le_bytes());
}
fn rng(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state
}
unsafe fn utf(o: &mut Vec<u8>, c: u32) {
    let mut b = [0xa5u8; 8];
    let n = unicode_to_utf8(b.as_mut_ptr(), c);
    num(o, n as u32);
    o.extend(&b[..6]);
    if n != 0 {
        decode(o, &b, n);
    }
}
unsafe fn decode(o: &mut Vec<u8>, b: &[u8; 8], len: i32) {
    let mut p = b.as_ptr().add(7);
    num(o, unicode_from_utf8(b.as_ptr(), len, &mut p) as u32);
    num(o, p.offset_from(b.as_ptr()) as u32);
}
unsafe fn bufstate(o: &mut Vec<u8>, s: &DynBuf, result: i32) {
    num(o, result as u32);
    num64(o, s.size as u64);
    num64(o, s.allocated_size as u64);
    num(o, s.error as u32);
    if s.size != 0 {
        o.extend(core::slice::from_raw_parts(s.buf, s.size));
    }
}
struct Allocator {
    calls: i32,
    fail: i32,
}
unsafe fn failrealloc(opaque: *mut c_void, p: *mut c_void, n: usize) -> *mut c_void {
    let a = &mut *opaque.cast::<Allocator>();
    a.calls += 1;
    if n != 0 && a.calls == a.fail {
        return ptr::null_mut();
    }
    dbuf_default_realloc(ptr::null_mut(), p, n)
}
struct CmpCtx {
    hash: u64,
    calls: u32,
    direction: i32,
}
unsafe fn compare(a: *const c_void, b: *const c_void, opaque: *mut c_void) -> i32 {
    let s = &mut *opaque.cast::<CmpCtx>();
    let (x, y) = (*a.cast::<u8>(), *b.cast::<u8>());
    s.calls += 1;
    s.hash = s.hash.wrapping_mul(1099511628211).wrapping_add(x as u64);
    s.hash = s.hash.wrapping_mul(1099511628211).wrapping_add(y as u64);
    ((x > y) as i32 - (x < y) as i32) * s.direction
}
unsafe fn sortcase(
    o: &mut Vec<u8>,
    state: &mut u32,
    n: usize,
    size: usize,
    offset: usize,
    distribution: i32,
    direction: i32,
    heap: bool,
) {
    #[repr(align(16))]
    struct Aligned([u8; 8192]);
    let mut b = Aligned([0x77; 8192]);
    let p = b.0.as_mut_ptr().add(offset);
    for i in 0..n {
        let v = match distribution {
            0 => rng(state) as u8,
            1 => i as u8,
            2 => (n - i) as u8,
            3 => 5,
            _ => (i % 7) as u8,
        };
        for j in 0..size {
            *p.add(i * size + j) = v.wrapping_add(j as u8);
        }
    }
    let mut ctx = CmpCtx {
        hash: 1469598103934665603,
        calls: 0,
        direction,
    };
    if heap {
        heapsortx(p, n, size, compare, (&mut ctx as *mut CmpCtx).cast());
    } else {
        rqsort(p.cast(), n, size, compare, (&mut ctx as *mut CmpCtx).cast());
    }
    num(o, ctx.calls);
    num64(o, ctx.hash);
    o.extend(&b.0[..n * size + offset + 16]);
}
unsafe fn dump() -> Vec<u8> {
    let mut o = Vec::new();
    let mut state = 0x12345678;
    for i in 0u32..65536 {
        let d = fromfp16(i as u16);
        num64(&mut o, float64_as_uint64(d));
        num(&mut o, tofp16(d) as u32);
        num(&mut o, isfp16nan(i as u16) as u32);
        num(&mut o, isfp16zero(i as u16) as u32);
        if i < 0x7bff {
            let next = fromfp16((i + 1) as u16);
            let mid = (d + next) * 0.5;
            num(&mut o, tofp16(mid) as u32);
            num(&mut o, tofp16(f64::from_bits(mid.to_bits() - 1)) as u32);
            num(&mut o, tofp16(f64::from_bits(mid.to_bits() + 1)) as u32);
        }
    }
    for _ in 0..100000 {
        let b = ((rng(&mut state) as u64) << 32) | rng(&mut state) as u64;
        num(&mut o, tofp16(f64::from_bits(b)) as u32);
    }
    for i in 0..=0x10ffff {
        utf(&mut o, i);
    }
    for i in [
        0x1fffff, 0x200000, 0x3ffffff, 0x4000000, 0x7fffffff, 0x80000000, 0xffffffff,
    ] {
        utf(&mut o, i);
    }
    for _ in 0..10000 {
        utf(&mut o, rng(&mut state));
    }
    for i in 0..65536 {
        let b = [(i >> 8) as u8, i as u8, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        for len in 1..=6 {
            decode(&mut o, &b, len);
        }
    }
    for fail in 0..=6 {
        let mut s = DynBuf::default();
        let mut a = Allocator { calls: 0, fail };
        dbuf_init2(&mut s, (&mut a as *mut Allocator).cast(), Some(failrealloc));
        let r = dbuf_put(&mut s, ptr::null(), 0);
        bufstate(&mut o, &s, r);
        let r = dbuf_putc(&mut s, 0xff);
        bufstate(&mut o, &s, r);
        let r = dbuf_put_u16(&mut s, 0x82fe);
        bufstate(&mut o, &s, r);
        let r = dbuf_put_u32(&mut s, 0x89abcdef);
        bufstate(&mut o, &s, r);
        let r = dbuf_put_u64(&mut s, 0x0123456789abcdef);
        bufstate(&mut o, &s, r);
        if s.size >= 7 {
            let r = dbuf_put_self(&mut s, 0, 7);
            bufstate(&mut o, &s, r);
        }
        let r = dbuf_claim(&mut s, 40);
        bufstate(&mut o, &s, r);
        let r = dbuf_putstr(&mut s, c"abc".as_ptr());
        bufstate(&mut o, &s, r);
        dbuf_set_error(&mut s);
        let r = dbuf_claim(&mut s, 0);
        bufstate(&mut o, &s, r);
        dbuf_free(&mut s);
        num(&mut o, a.calls as u32);
        num(
            &mut o,
            (s.buf.is_null() && s.realloc_func.is_none() && s.opaque.is_null()) as u32,
        );
        dbuf_free(&mut s);
    }
    let mut s = DynBuf::default();
    dbuf_init(&mut s);
    let r = dbuf_printf_args(&mut s, format_args!("value:{:08x}/{}", 0x12, "héllo"));
    bufstate(&mut o, &s, r);
    let text = "x".repeat(400);
    let r = dbuf_printf_args(&mut s, format_args!("[{text}]"));
    bufstate(&mut o, &s, r);
    dbuf_free(&mut s);
    for kind in 0..3 {
        dbuf_init(&mut s);
        s.size = if kind == 0 { usize::MAX - 1 } else { 0 };
        s.allocated_size = if kind == 1 { usize::MAX - 1 } else { 0 };
        s.error = (kind == 2) as i32;
        num(
            &mut o,
            dbuf_claim(&mut s, if kind == 0 { 5 } else { usize::MAX }) as u32,
        );
        num(&mut o, s.error as u32);
        s.buf = ptr::null_mut();
        dbuf_free(&mut s);
    }
    for size in 1..=32 {
        for offset in 0..16 {
            for dist in 0..5 {
                for heap in [false, true] {
                    sortcase(
                        &mut o,
                        &mut state,
                        127,
                        size,
                        offset,
                        dist,
                        if dist % 2 != 0 { -1 } else { 1 },
                        heap,
                    );
                }
            }
        }
    }
    for n in 0..16 {
        sortcase(&mut o, &mut state, n, 16, 0, 4, 1, false);
    }
    for k in -10..=300 {
        num(&mut o, from_hex(k) as u32);
    }
    for c in 0..=0x10ffff {
        num(&mut o, is_surrogate(c) as u32);
        num(&mut o, is_hi_surrogate(c) as u32);
        num(&mut o, is_lo_surrogate(c) as u32);
        if c >= 0x10000 {
            num(
                &mut o,
                from_surrogate(get_hi_surrogate(c), get_lo_surrogate(c)),
            );
        }
    }
    for _ in 0..10000 {
        let a = rng(&mut state) | 1;
        let b = rng(&mut state);
        let x = ((a as u64) << 32) | b as u64;
        num(&mut o, clz32(a) as u32);
        num(&mut o, ctz32(a) as u32);
        num(&mut o, clz64(x) as u32);
        num(&mut o, ctz64(x) as u32);
        num(&mut o, max_int(a as i32, b as i32) as u32);
        num(&mut o, min_int(a as i32, b as i32) as u32);
        num(&mut o, max_uint32(a, b));
        num(&mut o, min_uint32(a, b));
        num64(&mut o, max_int64(x as i64, !x as i64) as u64);
        num64(&mut o, min_int64(x as i64, !x as i64) as u64);
        num(&mut o, bswap16(a as u16) as u32);
        num(&mut o, bswap32(a));
        num64(&mut o, bswap64(x));
        let mut buf = [0xa5u8; 24];
        let p = buf.as_mut_ptr();
        put_u64(p.add(1), x);
        put_u32(p.add(3), b);
        put_u16(p.add(7), a as u16);
        put_u8(p.add(9), a as u8);
        o.extend(buf);
        num64(&mut o, get_u64(p.add(1)));
        num64(&mut o, get_i64(p.add(1)) as u64);
        num(&mut o, get_u32(p.add(3)));
        num(&mut o, get_i32(p.add(3)) as u32);
        num(&mut o, get_u16(p.add(7)));
        num(&mut o, get_i16(p.add(7)) as u32);
        num(&mut o, get_u8(p.add(9)));
        num(&mut o, get_i8(p.add(9)) as u32);
    }
    let str = c"abcdef".as_ptr();
    for size in -1..12 {
        let mut b = [0x7fu8; 16];
        pstrcpy(b.as_mut_ptr().cast(), size, str);
        o.extend(b);
    }
    for prefix in [c"", c"a", c"abc", c"abcdef", c"abcdefg", c"abX"] {
        let mut p = str.add(6);
        num(&mut o, strstart(str, prefix.as_ptr(), &mut p) as u32);
        num(&mut o, p.offset_from(str) as u32);
        num(&mut o, has_suffix(str, prefix.as_ptr()) as u32);
    }
    let mut b = [0u8; 16];
    b[..3].copy_from_slice(b"abc");
    num(
        &mut o,
        (pstrcat(b.as_mut_ptr().cast(), 5, c"defgh".as_ptr()) == b.as_mut_ptr().cast()) as u32,
    );
    o.extend(b);
    memcpy_no_ub(ptr::null_mut(), ptr::null(), 0);
    o
}
#[test]
fn entire_cutils_algorithm_matches_official_c() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = std::env::temp_dir().join(format!("quickjs-cutils-oracle-{}", std::process::id()));
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(root.join("../../vendor/quickjs-2026-06-04"))
        .arg(root.join("tests/cutils_oracle.c"))
        .args(["-lm", "-o"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&file).output().unwrap();
    let _ = std::fs::remove_file(file);
    assert!(c.status.success(), "C oracle crashed");
    let rust = unsafe { dump() };
    assert_eq!(c.stdout.len(), rust.len(), "serialized length differs");
    if let Some(i) = c.stdout.iter().zip(&rust).position(|(a, b)| a != b) {
        panic!(
            "C/Rust mismatch at byte {i}: C={}, Rust={}",
            c.stdout[i], rust[i]
        );
    }
    eprintln!("cutils official C parity: {} compared bytes", rust.len());
}
#[test]
fn printf_two_pass_errors_and_buffer_nul() {
    unsafe {
        let mut s = DynBuf::default();
        dbuf_init(&mut s);
        let mut calls = Vec::new();
        assert_eq!(
            dbuf_printf(&mut s, |p, cap| {
                calls.push(cap);
                let n = 140usize;
                for i in 0..n.min(cap - 1) {
                    *p.add(i) = b'x';
                }
                *p.add(n.min(cap - 1)) = 0;
                n as i32
            }),
            0
        );
        assert_eq!(calls, [128, 141]);
        assert_eq!(s.size, 140);
        assert_eq!(*s.buf.add(140), 0);
        let old_size = s.size;
        assert_eq!(dbuf_printf(&mut s, |_, _| -1), -1);
        assert_eq!(s.size, old_size);
        dbuf_free(&mut s);
    }
}
