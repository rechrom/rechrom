use super::*;

unsafe fn probe_finalized(rt: *mut JSRuntime, _: JSValue) {
    let count = JS_GetRuntimeOpaque(rt).cast::<u32>();
    *count += 1;
}

#[test]
fn allocation_gc_preserves_live_roots_and_collects_unreachable_cycles_like_c() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("../../vendor/quickjs-2026-06-04");
    let folder = std::env::temp_dir().join(format!("quickjs-force-gc-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let executable = folder.join("c");
    let mut compiler = include!("quickjs_oracle_config.rs");
    compiler
        .args(["-std=gnu11", "-O2", "-DCONFIG_VERSION=\"2026-06-04\""])
        .arg("-I")
        .arg(&source)
        .arg(root.join("tests/quickjs_force_gc_oracle.c"));
    for name in ["cutils.c", "libunicode.c", "libregexp.c", "dtoa.c"] {
        compiler.arg(source.join(name));
    }
    let built = compiler
        .arg("-lm")
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let expected = std::process::Command::new(&executable).output().unwrap();
    assert!(
        expected.status.success(),
        "{}",
        String::from_utf8_lossy(&expected.stderr)
    );
    let actual = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| unsafe {
            let rt = JS_NewRuntime();
            assert!(!rt.is_null());
            let ctx = JS_NewContext(rt);
            assert!(!ctx.is_null());
            // Callbacks own access during collection; retain only a raw pointer.
            let count = Box::into_raw(Box::new(0u32));
            JS_SetRuntimeOpaque(rt, count.cast());
            let mut id = 0;
            JS_NewClassID(&mut id);
            let def = JSClassDef {
                class_name: c"RootProbe".as_ptr(),
                finalizer: Some(probe_finalized),
                gc_mark: None,
                call: None,
                exotic: ptr::null_mut(),
            };
            assert_eq!(JS_NewClass(rt, id, &def), 0);
            JS_SetGCThreshold(rt, usize::MAX);
            let mut roots = [JS_UNDEFINED; 16];
            for value in &mut roots {
                *value = JS_NewObjectClass(ctx, id as i32);
                assert_eq!(JS_IsException(*value), 0);
                assert!(
                    JS_SetPropertyStr(ctx, *value, c"self".as_ptr(), JS_DupValue(ctx, *value)) >= 0
                );
                assert_eq!(*count, 0, "externally retained cycle collected prematurely");
            }
            for value in &roots[..8] {
                JS_FreeValue(ctx, *value);
            }
            let trigger = JS_NewObject(ctx);
            assert_eq!(JS_IsException(trigger), 0);
            let mut out = format!("{}\n", *count);
            for value in &roots[8..] {
                JS_FreeValue(ctx, *value);
            }
            JS_FreeValue(ctx, trigger);
            JS_RunGC(rt);
            out += &format!("{}\n", *count);
            JS_FreeContext(ctx);
            JS_FreeRuntime(rt);
            out += &format!("{}\n", *count);
            drop(Box::from_raw(count));
            out.into_bytes()
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(
        actual, expected.stdout,
        "allocation GC differs from official C"
    );
    assert_eq!(
        actual,
        if cfg!(feature = "force-gc-at-malloc") {
            b"8\n16\n16\n".as_slice()
        } else {
            b"0\n16\n16\n".as_slice()
        },
        "forced/default configuration not selected"
    );
    let _ = std::fs::remove_dir_all(folder);
}
