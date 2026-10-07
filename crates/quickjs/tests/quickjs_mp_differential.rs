use super::*;
#[path = "quickjs_test_source.rs"]
mod source;
fn next(rng: &mut u64) -> u64 {
    *rng ^= *rng << 13;
    *rng ^= *rng >> 7;
    *rng ^= *rng << 17;
    *rng
}
fn num(out: &mut Vec<u8>, n: u64, size: usize) {
    out.extend_from_slice(&n.to_le_bytes()[..size]);
}
fn dump(out: &mut Vec<u8>, r: js_limb_t, p: &[js_limb_t]) {
    num(out, r as u64, size_of::<js_limb_t>());
    for &v in p {
        num(out, v as u64, size_of::<js_limb_t>());
    }
}
#[test]
fn official_c_bigint_limb_arithmetic_matches() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    let begin = c.find("#define ADDC(").unwrap();
    let end = begin + c[begin..].find("static JSBigInt *js_bigint_new(").unwrap();
    oracle.push_str(&c[begin..end]);
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "js_bigint_set_si",
            "js_bigint_set_si64",
            "js_bigint_set_short",
            "js_bigint_sign",
            "js_bigint_get_si_sat",
        ],
    );
    oracle.push_str(include_str!("quickjs_mp_oracle.c"));
    let directory = std::env::temp_dir().join(format!("quickjs-mp-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("mp.c");
    let executable = directory.join("mp");
    std::fs::write(&path, oracle).unwrap();
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(upstream)
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&executable).output().unwrap();
    let _ = std::fs::remove_dir_all(directory);
    assert!(
        c.status.success(),
        "C arithmetic oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    let mut rng = 0x123456789abcdef0u64;
    unsafe {
        for step in 0..16384usize {
            let mut a = [0 as js_limb_t; 34];
            let mut b = a;
            let v = next(&mut rng) as js_limb_t;
            let n = step % 33;
            let shift = 1 + step % (JS_LIMB_BITS - 1);
            for i in 0..34 {
                a[i] = next(&mut rng) as js_limb_t;
                b[i] = next(&mut rng) as js_limb_t;
                match step % 16 {
                    0 => a[i] = 0,
                    1 => a[i] = js_limb_t::MAX,
                    2 => b[i] = js_limb_t::MAX,
                    3 => b[i] = 0,
                    4 => a[i] = 1 << (i % JS_LIMB_BITS),
                    _ => {}
                }
            }
            num(&mut out, js_limb_safe_clz(v) as u64, 4);
            if v != 0 {
                num(&mut out, js_limb_clz(v) as u64, 4);
            }
            for op in 0..11 {
                for alias in 0..3 {
                    let mut x = a;
                    let mut y = b;
                    let mut r = [js_limb_t::from_ne_bytes([0xa5; size_of::<js_limb_t>()]); 68];
                    let p = if alias == 0 {
                        r.as_mut_ptr()
                    } else if alias == 1 {
                        x.as_mut_ptr()
                    } else {
                        y.as_mut_ptr()
                    };
                    let divisor = v | 1;
                    let rem = b[0] % divisor;
                    let ret = match op {
                        0 => mp_add(
                            p,
                            x.as_ptr(),
                            y.as_ptr(),
                            n as js_limb_t,
                            (step & 1) as js_limb_t,
                        ),
                        1 => mp_sub(p, x.as_ptr(), y.as_ptr(), n as i32, (step & 1) as js_limb_t),
                        2 => mp_neg(p, x.as_ptr(), n as i32),
                        3 => mp_mul1(p, x.as_ptr(), n as js_limb_t, v, b[0]),
                        4 => mp_div1(p, x.as_ptr(), n as js_limb_t, divisor, rem),
                        5 => mp_add_mul1(p, x.as_ptr(), n as js_limb_t, v),
                        6 => mp_sub_mul1(p, x.as_ptr(), n as js_limb_t, v),
                        7 => mp_shl(p, x.as_ptr(), n as i32, shift as i32),
                        8 => mp_shr(p, x.as_ptr(), n as i32, shift as i32, v),
                        9 => {
                            let divisor = v | (1 << (JS_LIMB_BITS - 1));
                            mp_div1norm(p, x.as_ptr(), n as js_limb_t, divisor, b[0] % divisor)
                        }
                        10 => {
                            let divisor = v | (1 << (JS_LIMB_BITS - 1));
                            udiv1norm(p, a[0] % divisor, b[0], divisor, udiv1norm_init(divisor))
                        }
                        _ => unreachable!(),
                    };
                    dump(&mut out, ret, core::slice::from_raw_parts(p, n + 2));
                }
            }
            let na = 1 + step % 32;
            let mut nb = 1 + (step / 32) % 32;
            let mut r = [js_limb_t::from_ne_bytes([0xa5; size_of::<js_limb_t>()]); 68];
            mp_mul_basecase(
                r.as_mut_ptr(),
                a.as_ptr(),
                na as js_limb_t,
                b.as_ptr(),
                nb as js_limb_t,
            );
            dump(&mut out, 0, &r[..na + nb + 2]);
            nb = 1 + step % na;
            b[nb - 1] |= 1 << (JS_LIMB_BITS - 1);
            if step % 16 == 5 {
                a[..nb].copy_from_slice(&b[..nb]);
            }
            if step % 16 == 6 {
                a[..na].fill(js_limb_t::MAX);
            }
            r.fill(js_limb_t::from_ne_bytes([0xa5; size_of::<js_limb_t>()]));
            mp_divnorm(
                r.as_mut_ptr(),
                a.as_mut_ptr(),
                na as js_limb_t,
                b.as_ptr(),
                nb as js_limb_t,
            );
            dump(&mut out, 0, &r[..na - nb + 2]);
            dump(&mut out, 0, &a[..na + 2]);
        }
        for _ in 0..4096 {
            let v = next(&mut rng) as i64;
            let mut buf: JSBigIntBuf = core::mem::zeroed();
            let p = js_bigint_set_si64(&mut buf, v);
            num(&mut out, (*p).len as u64, 4);
            for i in 0..(*p).len {
                num(
                    &mut out,
                    *ptr::addr_of!((*p).tab).cast::<js_limb_t>().add(i as usize) as u64,
                    size_of::<js_limb_t>(),
                );
            }
            num(&mut out, js_bigint_sign(p) as u64, 4);
            num(
                &mut out,
                js_bigint_get_si_sat(p) as u64,
                size_of::<js_limb_t>(),
            );
            let p = js_bigint_set_short(&mut buf, JS_MKVAL(JS_TAG_SHORT_BIG_INT, v as i32));
            num(&mut out, (*p).len as u64, 4);
            num(
                &mut out,
                *ptr::addr_of!((*p).tab).cast::<js_limb_t>() as u64,
                size_of::<js_limb_t>(),
            );
            #[repr(C)]
            struct Big {
                len: i32,
                tab: [js_limb_t; 4],
            }
            let big = Big {
                len: 4,
                tab: [
                    next(&mut rng) as js_limb_t,
                    next(&mut rng) as js_limb_t,
                    next(&mut rng) as js_limb_t,
                    next(&mut rng) as js_limb_t,
                ],
            };
            let p = ptr::from_ref(&big).cast();
            num(&mut out, js_bigint_sign(p) as u64, 4);
            num(
                &mut out,
                js_bigint_get_si_sat(p) as u64,
                size_of::<js_limb_t>(),
            );
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "BigInt limb mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c BigInt limb parity: 573440 arithmetic calls, {} bytes",
        out.len()
    );
}
include!("quickjs_bigint_float_differential.rs");
