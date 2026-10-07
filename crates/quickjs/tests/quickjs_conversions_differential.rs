const CONVERSION_FUNCTIONS: &[&str] = &[
    "JS_ToBoolFree",
    "JS_ToBool",
    "JS_IsHTMLDDA",
    "skip_spaces",
    "to_digit",
    "is_digit",
    "is_safe_integer",
    "js_dbuf_init",
    "js_dbuf_bytecode_init",
    "js_realloc_bytecode_rt",
];
#[test]
fn official_c_truth_conversion_space_scanning_and_buffer_policy_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = gc_source(&c);
    oracle.push_str("#include \"libunicode.h\"\n");
    oracle.push_str("#define MAX_SAFE_INTEGER (((int64_t)1 << 53) - 1)\n");
    source::append_functions(&mut oracle, &c, CONVERSION_FUNCTIONS);
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_conversions_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-conv-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("conv.c");
    let executable = directory.join("conv");
    std::fs::write(&path, oracle).unwrap();
    let mut command = include!("quickjs_oracle_config.rs");
    command.args(["-std=c11", "-O2", "-I"]).arg(&upstream);
    if cfg!(feature = "short-opcodes") {
        command.arg("-DSHORT_OPCODES=1");
    }
    command
        .arg(&path)
        .arg(upstream.join("cutils.c"))
        .arg(upstream.join("libunicode.c"))
        .arg("-o")
        .arg(&executable);
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&executable).output().unwrap();
    let _ = std::fs::remove_dir_all(directory);
    assert!(
        c.status.success(),
        "C conversion oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        assert_eq!(JS_InitAtoms(rt), 0);
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        let mut rng = 0xabcdef0123456789u64;
        for i in 0..32768 {
            let bits = weak_rng(&mut rng);
            let vs = [
                JS_NewInt32(&mut ctx, bits as i32),
                JS_MKVAL(JS_TAG_BOOL, bits as i32),
                JS_NULL,
                JS_UNDEFINED,
                JS_EXCEPTION,
                __JS_NewShortBigInt(&mut ctx, bits as i64),
                __JS_NewFloat64(&mut ctx, f64::from_bits(bits)),
                JS_MKVAL(JS_TAG_UNINITIALIZED, 0),
            ];
            for v in vs {
                num(&mut out, JS_ToBool(&mut ctx, v) as u64, 4);
                num(&mut out, JS_ToBoolFree(&mut ctx, v) as u64, 4);
            }
            for n in [
                is_safe_integer(f64::from_bits(bits)),
                is_digit(bits as i32),
                to_digit(bits as i32),
            ] {
                num(&mut out, n as u64, 4);
            }
            let units = [bits as u16, 1, 2, 0, 4];
            let p = make_string(rt, &units[..i % 6], (i % 2) as u32);
            let mut v = JS_MKPTR(JS_TAG_STRING, p.cast());
            num(&mut out, JS_ToBool(&mut ctx, v) as u64, 4);
            num(&mut out, (*js_rc(p.cast())).ref_count as u64, 4);
            num(&mut out, JS_ToBoolFree(&mut ctx, v) as u64, 4);
            let rope = js_mallocz_rt(rt, size_of::<JSStringRope>()).cast::<JSStringRope>();
            (*js_rc(rope.cast())).ref_count = 1;
            (*rope).left = JS_MKPTR(
                JS_TAG_STRING,
                make_string(rt, &units[..i % 3], (i % 2) as u32).cast(),
            );
            (*rope).right = JS_MKPTR(
                JS_TAG_STRING,
                make_string(rt, &units[..i % 3], (i % 2) as u32).cast(),
            );
            (*rope).len = ((i % 3) * 2) as u32;
            v = JS_MKPTR(JS_TAG_STRING_ROPE, rope.cast());
            num(&mut out, JS_ToBool(&mut ctx, v) as u64, 4);
            num(&mut out, (*js_rc(rope.cast())).ref_count as u64, 4);
            num(&mut out, JS_ToBoolFree(&mut ctx, v) as u64, 4);
            let b = js_mallocz_rt(rt, size_of::<JSBigInt>() + 8 * size_of::<js_limb_t>())
                .cast::<JSBigInt>();
            (*js_rc(b.cast())).ref_count = 1;
            (*b).len = 8;
            for j in 0..8 {
                *ptr::addr_of_mut!((*b).tab).cast::<js_limb_t>().add(j) = if i % 3 == 0 {
                    0
                } else {
                    weak_rng(&mut rng) as js_limb_t
                };
            }
            v = JS_MKPTR(JS_TAG_BIG_INT, b.cast());
            num(&mut out, JS_ToBool(&mut ctx, v) as u64, 4);
            num(&mut out, (*js_rc(b.cast())).ref_count as u64, 4);
            num(&mut out, JS_ToBoolFree(&mut ctx, v) as u64, 4);
            let o = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
            (*js_rc(o.cast())).ref_count = 100;
            (*o).set_is_HTMLDDA((i % 2) as u8);
            v = JS_MKPTR(JS_TAG_OBJECT, o.cast());
            num(&mut out, JS_ToBool(&mut ctx, v) as u64, 4);
            num(&mut out, (*js_rc(o.cast())).ref_count as u64, 4);
            num(&mut out, JS_ToBoolFree(&mut ctx, v) as u64, 4);
            num(&mut out, (*js_rc(o.cast())).ref_count as u64, 4);
            js_free_rt(rt, o.cast());
            let atom = __JS_NewAtomInit(rt, c"symbol".as_ptr(), 6, JS_ATOM_TYPE_SYMBOL);
            let p = *(*rt).atom_array.add(atom as usize);
            v = JS_MKPTR(JS_TAG_SYMBOL, p.cast());
            num(&mut out, JS_ToBool(&mut ctx, v) as u64, 4);
            num(&mut out, (*js_rc(p.cast())).ref_count as u64, 4);
            num(&mut out, JS_ToBoolFree(&mut ctx, v) as u64, 4);
        }
        for cp in 0..0x110000 {
            let mut text = [0u8; 32];
            text[0] = b' ';
            let n = super::super::cutils::unicode_to_utf8(text.as_mut_ptr().add(1), cp) as usize;
            text[n + 1] = b'\t';
            text[n + 2] = b'x';
            num(&mut out, skip_spaces(text.as_ptr().cast()) as u64, 4);
        }
        for c in -1024..=1024 {
            num(&mut out, to_digit(c) as u64, 4);
            num(&mut out, is_digit(c) as u64, 4);
        }
        for trial in 0..64 {
            let mut db: super::super::cutils_header::DynBuf = core::mem::zeroed();
            js_dbuf_bytecode_init(&mut ctx, &mut db);
            num(&mut out, (db.opaque == rt.cast()) as u64, 4);
            num(
                &mut out,
                (db.realloc_func.map(|f| f as usize)
                    == Some(js_realloc_bytecode_rt as *const () as usize)) as u64,
                4,
            );
            num(&mut out, db.size as u64, 4);
            num(&mut out, db.allocated_size as u64, 4);
            num(&mut out, db.error as u64, 4);
            num(&mut out, db.buf.is_null() as u64, 4);
            let mut p = js_realloc_bytecode_rt(rt.cast(), ptr::null_mut(), 32);
            num(&mut out, (!p.is_null()) as u64, 4);
            if !p.is_null() {
                ptr::write_bytes(p.cast::<u8>(), trial as u8, 32);
                num(
                    &mut out,
                    js_realloc_bytecode_rt(rt.cast(), p, i32::MAX as usize / 2 + 1).is_null()
                        as u64,
                    4,
                );
                num(&mut out, *p.cast::<u8>() as u64, 4);
                p = js_realloc_bytecode_rt(rt.cast(), p, 1024);
                num(&mut out, (!p.is_null()) as u64, 4);
                num(&mut out, *p.cast::<u8>() as u64, 4);
                js_realloc_bytecode_rt(rt.cast(), p, 0);
            }
            js_dbuf_init(&mut ctx, &mut db);
            num(&mut out, (db.opaque == rt.cast()) as u64, 4);
            num(
                &mut out,
                (db.realloc_func.map(|f| f as usize)
                    == Some(js_realloc_dbuf_rt as *const () as usize)) as u64,
                4,
            );
        }
        cleanup(&mut out, rt, &mut h);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "conversion C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c truth/space/buffer parity: 32768 values, 1114112 codepoints, {} bytes",
        out.len()
    );
}
