const DEBUG_FUNCTIONS: &[&str] = &[
    "dbuf_put_leb128",
    "dbuf_put_sleb128",
    "get_leb128",
    "get_sleb128",
    "find_line_num",
    "get_prop_flags",
    "check_define_prop_flags",
];
#[test]
fn official_c_varints_source_locations_and_descriptor_flags_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = gc_source(&c);
    source::append_functions(&mut oracle, &c, DEBUG_FUNCTIONS);
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_debug_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-debug-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("debug.c");
    let executable = directory.join("debug");
    std::fs::write(&path, oracle).unwrap();
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(&upstream)
        .arg(&path)
        .arg(upstream.join("cutils.c"))
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
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
    let mut out = Vec::new();
    unsafe {
        let mut rng = 0x987654321abcdef0u64;
        for i in 0..32768 {
            let bits = weak_rng(&mut rng);
            let mut db: crate::cutils_header::DynBuf = core::mem::zeroed();
            crate::cutils::dbuf_init(&mut db);
            dbuf_put_leb128(&mut db, bits as u32);
            dbuf_put_sleb128(&mut db, (bits >> 32) as i32);
            num(&mut out, db.size as u64, 4);
            out.extend_from_slice(core::slice::from_raw_parts(db.buf, db.size));
            for size in 0..=db.size {
                let mut u = 0xa5a5a5a5;
                let mut v = 0xa5a5a5a5u32 as i32;
                num(
                    &mut out,
                    get_leb128(&mut u, db.buf, db.buf.add(size)) as u64,
                    4,
                );
                num(&mut out, u as u64, 4);
                num(
                    &mut out,
                    get_sleb128(&mut v, db.buf, db.buf.add(size)) as u64,
                    4,
                );
                num(&mut out, v as u64, 4);
            }
            crate::cutils::dbuf_free(&mut db);
            let mut bytes = [0u8; 128];
            let size = i % 129;
            for b in &mut bytes[..size] {
                *b = weak_rng(&mut rng) as u8;
            }
            for offset in 0..=size {
                let mut u = 0;
                num(
                    &mut out,
                    get_leb128(&mut u, bytes.as_ptr().add(offset), bytes.as_ptr().add(size)) as u64,
                    4,
                );
                num(&mut out, u as u64, 4);
            }
            let mut b: JSFunctionBytecode = core::mem::zeroed();
            b.set_has_debug((i % 2) as u8);
            b.debug.pc2line_buf = bytes.as_mut_ptr();
            b.debug.pc2line_len = size as i32;
            for pc in [0, 1, 63, 255, 65535, u32::MAX, bits as u32] {
                let mut col = 0xa5a5a5a5u32 as i32;
                let line = find_line_num(ptr::null_mut(), &mut b, pc, &mut col);
                num(&mut out, line as u64, 4);
                num(&mut out, col as u64, 4);
            }
        }
        // All flag combinations permitted in JS_DefineProperty's input mask.
        for prop_flags in 0..64 {
            for flags in 0..16384 {
                num(
                    &mut out,
                    check_define_prop_flags(prop_flags, flags) as u64,
                    4,
                );
                num(&mut out, get_prop_flags(flags, prop_flags) as u64, 4);
            }
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(a, b)| a != b) {
        panic!(
            "debug/flags differ at byte {i}: Rust {}, C {}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "varints, source locations, descriptor flags: {} matching bytes",
        out.len()
    );
}
