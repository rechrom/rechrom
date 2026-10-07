// Included in the limb test module, sharing its independent source extractor.
#[repr(C)]
struct BigFloatFixture {
    len: i32,
    tab: [js_limb_t; 128],
}
fn normalize_fixture(a: &mut BigFloatFixture) {
    while a.len > 1 {
        let n = a.len as usize;
        let v = a.tab[n - 1];
        if (v != 0 && v != js_limb_t::MAX) || (v & 1) != (a.tab[n - 2] >> (JS_LIMB_BITS - 1)) {
            break;
        }
        a.len -= 1;
    }
}
#[test]
fn official_c_bigint_float_rounding_and_comparison_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "js_bigint_sign",
            "js_bigint_get_mant_exp",
            "shr_rndn",
            "js_bigint_to_float64",
            "js_bigint_float64_cmp",
            "js_bigint_cmp",
        ],
    );
    oracle.push_str(include_str!("quickjs_bigint_float_oracle.c"));
    let directory = std::env::temp_dir().join(format!(
        "quickjs-bigint-float-oracle-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("float.c");
    let executable = directory.join("float");
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
        "C BigInt float oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    let mut rng = 0x123456789abcdef0u64;
    let patterns = [
        0,
        0x8000000000000000,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff8000000000000,
        0x7ff0000000000001,
        1,
        0x8000000000000001,
        0x0010000000000000,
        0x8010000000000000,
        0x3ff0000000000000,
        0xbff0000000000000,
        0x7fefffffffffffff,
        0xffefffffffffffff,
    ];
    unsafe {
        for step in 0..32768usize {
            let mut a = BigFloatFixture {
                len: (1 + step % 128) as i32,
                tab: [0; 128],
            };
            let mut b = BigFloatFixture {
                len: (1 + (step / 128) % 128) as i32,
                tab: [0; 128],
            };
            for i in 0..a.len as usize {
                a.tab[i] = next(&mut rng) as js_limb_t;
            }
            for i in 0..b.len as usize {
                b.tab[i] = next(&mut rng) as js_limb_t;
            }
            match step % 8 {
                0 => {
                    a.tab.fill(0);
                    a.tab[(step / 8) % a.len as usize] = 1 << ((step / 1024) % JS_LIMB_BITS);
                }
                1 => {
                    a.tab[..a.len as usize].fill(js_limb_t::MAX);
                    a.tab[0] = a.tab[0].wrapping_sub((step % 2048) as js_limb_t);
                }
                2 => {
                    a.len = 1;
                    a.tab[0] = if (step / 8) & 1 != 0 { 0 } else { 1 };
                }
                3 => {
                    a.len = 2;
                    a.tab[0] = ((1u64 << 53) + (step % 2048) as u64) as js_limb_t;
                    a.tab[1] = 0;
                }
                _ => {}
            }
            normalize_fixture(&mut a);
            normalize_fixture(&mut b);
            num(&mut out, a.len as u64, 4);
            num(&mut out, b.len as u64, 4);
            let ap = ptr::from_ref(&a).cast::<JSBigInt>();
            let bp = ptr::from_ref(&b).cast::<JSBigInt>();
            let mut e = 0;
            if a.len != 1 || a.tab[0] != 0 {
                num(
                    &mut out,
                    js_bigint_get_mant_exp(ptr::null_mut(), &mut e, ap),
                    8,
                );
                num(&mut out, e as u64, 4);
            }
            let f = js_bigint_to_float64(ptr::null_mut(), ap).to_bits();
            num(&mut out, f, 8);
            num(&mut out, js_bigint_cmp(ptr::null_mut(), ap, bp) as u64, 4);
            num(&mut out, js_bigint_cmp(ptr::null_mut(), ap, ap) as u64, 4);
            for k in 0..18 {
                let bits = match k {
                    0..=13 => patterns[k],
                    14 => f,
                    15 => f.wrapping_add(1),
                    16 => f.wrapping_sub(1),
                    _ => next(&mut rng),
                };
                num(
                    &mut out,
                    js_bigint_float64_cmp(ptr::null_mut(), ap, f64::from_bits(bits)) as u64,
                    4,
                );
            }
            let v = next(&mut rng);
            for n in 1..=30 {
                num(&mut out, shr_rndn(v, n), 8);
            }
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "BigInt float mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c BigInt float parity: 32768 integers, 589824 comparisons, {} bytes",
        out.len()
    );
}
