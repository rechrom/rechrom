use super::*;
fn num(o: &mut Vec<u8>, v: u32) {
    o.extend(v.to_le_bytes());
}
fn num64(o: &mut Vec<u8>, v: u64) {
    o.extend(v.to_le_bytes());
}
fn rng(s: &mut u32) -> u32 {
    *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
    *s
}
unsafe fn printcase(o: &mut Vec<u8>, bits: u64, radix: i32, digits: i32, flags: i32) {
    let d = uint64_as_float64(bits);
    let mut buf = [0x7fu8; 2048];
    let mut temp = JSDTOATempMem::default();
    let n = js_dtoa(buf.as_mut_ptr().cast(), d, radix, digits, flags, &mut temp);
    let bound = js_dtoa_max_len(d, radix, digits, flags);
    assert!(n>=0&&n<=bound,"dtoa bound too small, radix={radix}, bits={bits:x}, flags={flags}, digits={digits}, len={n}, bound={bound}");
    num(o, bound as u32);
    num(o, n as u32);
    o.extend(&buf[..n as usize + 1]);
    num(o, buf[n as usize + 1] as u32);
    let mut parse = JSATODTempMem::default();
    let mut next = buf.as_ptr().cast::<c_char>();
    let value = js_atod(buf.as_ptr().cast(), &mut next, radix, 0, &mut parse);
    num64(o, float64_as_uint64(value));
    num(o, next.offset_from(buf.as_ptr().cast()) as u32);
}
unsafe fn parsecase(o: &mut Vec<u8>, text: &str, radix: i32, flags: i32) {
    let text = std::ffi::CString::new(text).unwrap();
    let mut temp = JSATODTempMem::default();
    let mut next = text.as_ptr();
    let value = js_atod(text.as_ptr(), &mut next, radix, flags, &mut temp);
    num64(o, float64_as_uint64(value));
    num(o, next.offset_from(text.as_ptr()) as u32);
}
unsafe fn dump() -> Vec<u8> {
    let mut o = Vec::new();
    let mut state = 0x12345678;
    let bounds = [
        0,
        0x8000000000000000,
        1,
        2,
        0x8000000000000001,
        0xfffffffffffff,
        0x10000000000000,
        0x10000000000001,
        0x3ff0000000000000,
        0x3fefffffffffffff,
        0x3ff0000000000001,
        0x7fefffffffffffff,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff0000000000001,
        0xfff8000000000000,
    ];
    for radix in 2..=36 {
        for bits in bounds {
            for exp in [0, 4, 8] {
                for sign in [0, 16] {
                    printcase(&mut o, bits, radix, 0, exp | sign);
                }
            }
        }
        for _ in 0..1500 {
            let bits = ((rng(&mut state) as u64) << 32) | rng(&mut state) as u64;
            for exp in [0, 4, 8] {
                printcase(&mut o, bits, radix, 0, exp);
            }
        }
    }
    let digits = [0, 1, 2, 6, 17, 50, 100, 101];
    for _ in 0..500 {
        let bits = ((rng(&mut state) as u64) << 32) | rng(&mut state) as u64;
        for digit in digits {
            for exp in [0, 4, 8] {
                for fmt in 1..=2 {
                    if digit != 0 || fmt == 2 {
                        printcase(&mut o, bits, 10, digit, exp | fmt);
                    }
                }
            }
        }
    }
    for i in 0..65536 {
        let bits = float64_as_uint64(fromfp16(i as u16));
        printcase(&mut o, bits, 10, 0, 0);
        printcase(&mut o, bits, 10, 2, 2);
        printcase(&mut o, bits, 10, 10, 1);
    }
    let mut buf = [0u8; 128];
    for _ in 0..10000 {
        let a = rng(&mut state);
        let b = ((rng(&mut state) as u64) << 32) | rng(&mut state) as u64;
        let n = u32toa(buf.as_mut_ptr().cast(), a);
        num(&mut o, n as u32);
        o.extend(&buf[..n]);
        let n = i32toa(buf.as_mut_ptr().cast(), a as i32);
        num(&mut o, n as u32);
        o.extend(&buf[..n]);
        let n = u64toa(buf.as_mut_ptr().cast(), b);
        num(&mut o, n as u32);
        o.extend(&buf[..n]);
        let n = i64toa(buf.as_mut_ptr().cast(), b as i64);
        num(&mut o, n as u32);
        o.extend(&buf[..n]);
        for radix in 2..=36 {
            let n = u64toa_radix(buf.as_mut_ptr().cast(), b, radix);
            num(&mut o, n as u32);
            o.extend(&buf[..n]);
            let n = i64toa_radix(buf.as_mut_ptr().cast(), b as i64, radix);
            num(&mut o, n as u32);
            o.extend(&buf[..n]);
        }
    }
    for line in include_str!("dtoa_parse_cases.tsv").lines() {
        let mut parts = line.splitn(3, '\t');
        let radix = parts.next().unwrap().parse().unwrap();
        let flags = parts.next().unwrap().parse().unwrap();
        parsecase(&mut o, parts.next().unwrap(), radix, flags);
    }
    for radix in 2..=36 {
        for a in -2048..=2047 {
            num(&mut o, mul_log2_radix(a, radix) as u32);
        }
    }
    o
}
#[test]
fn entire_dtoa_algorithm_matches_official_c() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = std::env::temp_dir().join(format!("quickjs-dtoa-oracle-{}", std::process::id()));
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(root.join("../../vendor/quickjs-2026-06-04"))
        .arg(root.join("tests/dtoa_oracle.c"))
        .arg(root.join("../../vendor/quickjs-2026-06-04/cutils.c"))
        .args(["-lm", "-o"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&file)
        .arg(root.join("tests/dtoa_parse_cases.tsv"))
        .output()
        .unwrap();
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
    eprintln!("dtoa official C parity: {} compared bytes", rust.len());
}
