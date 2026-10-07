#[test]
fn official_c_string_comparison_widening_and_rope_hash_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    source::append_functions(&mut oracle, &c, ATOM_FUNCTIONS);
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "js_string_eq",
            "js_string_compare",
            "copy_str16",
            "JS_IsEmptyString",
            "count_ascii",
        ],
    );
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_strings_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-strings-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("strings.c");
    let executable = directory.join("strings");
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
        "C strings oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        let mut rng = 0x123456789abcdef0u64;
        for step in 0..32768usize {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let mut a = [0u16; 128];
            let mut b = a;
            let n = step % 65;
            let w1 = ((step >> 1) & 1) as u32;
            let w2 = ((step >> 2) & 1) as u32;
            for i in 0..128 {
                a[i] = (rng >> ((i % 8) * 8)).wrapping_add((i * 263) as u64) as u16;
                b[i] = if step % 3 == 0 {
                    a[i]
                } else {
                    a[i].wrapping_add((step % 257) as u16)
                };
                if w1 == 0 {
                    a[i] &= 255;
                }
                if w2 == 0 {
                    b[i] &= 255;
                }
            }
            let p = make_string(rt, &a[..n], w1);
            let q = make_string(rt, &b[..n + step % 3], w2);
            let pos1 = if n != 0 { step % n } else { 0 };
            let pos2 = if n != 0 { (step / 65) % n } else { 0 };
            let len = n - pos1.max(pos2);
            for v in [
                js_string_memcmp(p, pos1 as i32, q, pos2 as i32, len as i32),
                js_string_eq(ptr::null_mut(), p, q),
                js_string_eq(ptr::null_mut(), p, p),
                js_string_compare(ptr::null_mut(), p, q),
                js_string_compare(ptr::null_mut(), q, p),
                JS_IsEmptyString(JS_MKPTR(JS_TAG_STRING, p.cast())),
                JS_IsEmptyString(JS_UNDEFINED),
            ] {
                num(&mut out, v as u64, 4);
            }
            let mut dst = [0xa5a5u16; 128];
            copy_str16(dst.as_mut_ptr().add(3), p, pos1 as i32, len as i32);
            for v in dst {
                num(&mut out, v as u64, 2);
            }
            num(&mut out, hash_string(p, rng as u32) as u64, 4);
            let mut rope: JSStringRope = core::mem::zeroed();
            rope.left = JS_MKPTR(JS_TAG_STRING, p.cast());
            rope.right = JS_MKPTR(JS_TAG_STRING, q.cast());
            let mut root: JSStringRope = core::mem::zeroed();
            root.left = JS_MKPTR(JS_TAG_STRING_ROPE, ptr::from_mut(&mut rope).cast());
            root.right = JS_MKPTR(JS_TAG_STRING, p.cast());
            num(
                &mut out,
                hash_string_rope(
                    JS_MKPTR(JS_TAG_STRING_ROPE, ptr::from_mut(&mut root).cast()),
                    rng as u32,
                ) as u64,
                4,
            );
            num(&mut out, count_ascii(string_data8(p), n << w1) as u64, 4);
            js_free_string(rt, p);
            js_free_string(rt, q);
        }
        num(&mut out, h.calls as u64, 4);
        num(&mut out, h.live as u64, 4);
        num(&mut out, h.trace, 8);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "string C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c string parity: 32768 mixed-width fixtures, {} bytes",
        out.len()
    );
}
