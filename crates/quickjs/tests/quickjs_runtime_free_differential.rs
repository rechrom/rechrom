#[test]
fn official_c_runtime_job_context_cycle_and_allocator_shutdown_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let mut oracle =
        String::from("#include \"quickjs.c\"\n#undef malloc\n#undef free\n#undef realloc\n");
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_runtime_free_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-runtimefree-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("runtimefree.c");
    let executable = directory.join("runtimefree");
    std::fs::write(&path, oracle).unwrap();
    let mut command = include!("quickjs_oracle_config.rs");
    command
        .args(["-std=gnu11", "-O2", "-DCONFIG_VERSION=\"2026-06-04\"", "-I"])
        .arg(&upstream);
    if cfg!(feature = "short-opcodes") {
        command.arg("-DSHORT_OPCODES=1");
    }
    command.arg(&path);
    for file in ["cutils.c", "libunicode.c", "libregexp.c", "dtoa.c"] {
        command.arg(upstream.join(file));
    }
    command.arg("-o").arg(&executable);
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
        "C runtime-free oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        for trial in 0..512 {
            let mut h = host(0);
            let mut ms: JSMallocState = core::mem::zeroed();
            ms.opaque = ptr::from_mut(&mut h).cast();
            let rt = host_malloc(&mut ms, size_of::<JSRuntime>()).cast::<JSRuntime>();
            ptr::write_bytes(rt, 0, 1);
            initialize(rt, &mut h);
            for head in [
                ptr::addr_of_mut!((*rt).context_list),
                ptr::addr_of_mut!((*rt).gc_obj_list),
                ptr::addr_of_mut!((*rt).gc_zero_ref_count_list),
                ptr::addr_of_mut!((*rt).tmp_obj_list),
                ptr::addr_of_mut!((*rt).weakref_list),
                ptr::addr_of_mut!((*rt).job_list),
            ] {
                init_list_head(head);
            }
            assert_eq!(JS_InitAtoms(rt), 0);
            assert_eq!(init_shape_hash(rt), 0);
            (*rt).current_exception = JS_UNINITIALIZED;
            (*rt).class_count = JS_CLASS_INIT_COUNT as i32;
            (*rt).class_array =
                js_mallocz_rt(rt, size_of::<JSClass>() * (*rt).class_count as usize).cast();
            for j in 1..(*rt).class_count {
                let cl = (*rt).class_array.add(j as usize);
                (*cl).class_id = j as u32;
                (*cl).class_name =
                    __JS_NewAtomInit(rt, c"held-class".as_ptr(), 10, JS_ATOM_TYPE_STRING);
            }
            let ctx = js_mallocz_rt(rt, size_of::<JSContext>()).cast::<JSContext>();
            (*js_rc(ctx.cast())).ref_count = 1;
            (*ctx).rt = rt;
            init_list_head(&mut (*ctx).loaded_modules);
            list_add_tail(&mut (*ctx).link, &mut (*rt).context_list);
            add_gc_object(rt, &mut (*ctx).header, JS_GC_OBJ_TYPE_JS_CONTEXT);
            (*ctx).class_proto =
                js_mallocz_rt(rt, size_of::<JSValue>() * (*rt).class_count as usize).cast();
            let mut objects = [ptr::null_mut(); 4];
            for j in 0..4 {
                let p = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
                objects[j] = p;
                (*js_rc(p.cast())).ref_count = 1;
                add_gc_object(rt, &mut (*p).header, JS_GC_OBJ_TYPE_JS_OBJECT);
                (*p).class_id = JS_CLASS_OBJECT as u16;
                (*p).shape = js_mallocz_rt(rt, get_shape_size(4, 1)).cast();
                let sh = (*p).shape;
                (*js_rc(sh.cast())).ref_count = 1;
                add_gc_object(rt, &mut (*sh).header, JS_GC_OBJ_TYPE_SHAPE);
                (*sh).prop_hash_mask = 3;
                (*sh).prop_count = 1;
                (*sh).prop_size = 1;
                (*get_shape_prop(sh)).atom =
                    __JS_NewAtomInit(rt, c"edge".as_ptr(), 4, JS_ATOM_TYPE_STRING);
                (*p).prop = js_mallocz_rt(rt, size_of::<JSProperty>()).cast();
            }
            for j in 0..4 {
                (*(*objects[j]).prop).u.value =
                    JS_DupValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, objects[(j + 1) % 4].cast()));
            }
            *(*ctx).class_proto.add(1) =
                JS_DupValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, objects[0].cast()));
            for _ in 0..trial % 8 + 1 {
                let e = js_mallocz_rt(rt, size_of::<JSJobEntry>() + 4 * size_of::<JSValue>())
                    .cast::<JSJobEntry>();
                (*e).argc = 4;
                (*e).realm = JS_DupContext(ctx);
                for k in 0..4 {
                    *ptr::addr_of_mut!((*e).argv).cast::<JSValue>().add(k) =
                        JS_DupValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, objects[k].cast()));
                }
                list_add_tail(&mut (*e).link, &mut (*rt).job_list);
            }
            (*rt).current_exception = JS_MKPTR(
                JS_TAG_STRING,
                make_string(rt, &[1, 2, 3, 4, 5], (trial % 2) as u32).cast(),
            );
            JS_FreeContext(ctx);
            for p in objects {
                JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
            }
            JS_FreeRuntime(rt);
            num(&mut out, h.calls as u64, 4);
            num(&mut out, h.live as u64, 4);
            num(&mut out, h.trace, 8);
            assert_eq!(h.live, 0);
            assert!(h.allocs.is_empty());
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "runtime-free C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c runtime shutdown parity: 512 runtimes, pending jobs/contexts/cycles, {} bytes",
        out.len()
    );
}
