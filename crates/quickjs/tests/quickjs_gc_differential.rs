const GC_FUNCTIONS: &[&str] = &[
    "JS_MarkContext",
    "js_free_modules",
    "JS_FreeContext",
    "js_free_shape0",
    "js_free_shape",
    "js_free_shape_null",
    "js_autoinit_free",
    "js_autoinit_mark",
    "free_property",
    "free_var_ref",
    "async_func_free_frame",
    "__async_func_free",
    "async_func_free",
    "close_var_ref",
    "close_var_refs",
    "close_lexical_var",
    "js_mark_module_def",
    "js_free_module_def",
    "free_bytecode_atoms",
    "free_function_bytecode",
    "free_object",
    "free_gc_object",
    "free_zero_refcount",
    "__JS_FreeValueRT",
    "__JS_FreeValue",
    "mark_children",
    "gc_decref_child",
    "gc_decref",
    "gc_scan_incref_child",
    "gc_scan_incref_child2",
    "gc_scan",
    "gc_free_cycles",
    "JS_Throw",
    "JS_GetException",
    "JS_HasException",
    "JS_IsLiveObject",
    "JS_DupContext",
    "JS_SetUncatchableException",
    "JS_MarkValue",
    "add_gc_object",
    "remove_gc_object",
    "get_shape_size",
    "get_shape_prop",
    "shape_hash",
    "get_shape_hash",
    "js_shape_hash_unlink",
    "js_autoinit_get_realm",
    "js_autoinit_get_id",
];
fn gc_source(c: &str) -> String {
    let mut oracle = source::source_prefix(c);
    let a = c.find("typedef enum {\n    JS_CLOSURE_LOCAL").unwrap();
    let b = a + c[a..].find("enum {\n    __JS_ATOM_NULL").unwrap();
    oracle.push_str(&c[a..b]);
    let op = c.find("typedef enum OPCodeFormat").unwrap();
    let op_end = op + c[op..].find("static int JS_InitAtoms").unwrap();
    oracle.push_str(&c[op..op_end]);
    let oi = c.find("typedef struct JSOpCode {").unwrap();
    let oi_end = oi + c[oi..].find("static __exception int next_token").unwrap();
    oracle.push_str(&c[oi..oi_end]);
    oracle.push_str("typedef enum JSFreeModuleEnum { JS_FREE_MODULE_ALL, JS_FREE_MODULE_NOT_RESOLVED } JSFreeModuleEnum;\n");
    source::append_functions(&mut oracle, c, ATOM_FUNCTIONS);
    source::append_functions(&mut oracle, c, GC_FUNCTIONS);
    oracle
}
fn gc_id(
    objects: &[*mut JSObject; 64],
    shapes: &[*mut JSShape; 64],
    p: *mut JSGCObjectHeader,
) -> u32 {
    for i in 0..64 {
        if p == objects[i].cast() {
            return i as u32 + 1;
        }
        if p == shapes[i].cast() {
            return i as u32 + 65;
        }
    }
    u32::MAX
}
struct GCLog {
    out: *mut Vec<u8>,
    objects: [*mut JSObject; 64],
    shapes: [*mut JSShape; 64],
}
unsafe fn record_finalizer(rt: *mut JSRuntime, v: JSValue) {
    let log = &*(*rt).user_opaque.cast::<GCLog>();
    let p = JS_VALUE_GET_PTR(v).cast::<JSObject>();
    let out = &mut *log.out;
    for n in [
        gc_id(&log.objects, &log.shapes, p.cast()),
        (*rt).gc_phase as u32,
        (*p).free_mark() as u32,
        (*p).shape.is_null() as u32,
        (*p).prop.is_null() as u32,
        JS_IsLiveObject(rt, v) as u32,
    ] {
        num(out, n as u64, 4);
    }
}
unsafe fn dump_gc_list(out: &mut Vec<u8>, head: *mut list_head, log: &GCLog) {
    num(out, ListIter::new(head, false, false).count() as u64, 4);
    for el in ListIter::new(head, false, false) {
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        let rc = &*js_rc(p.cast());
        for n in [
            gc_id(&log.objects, &log.shapes, p),
            ((rc.gc_obj_type_and_mark & 127) as u32),
            rc.ref_count as u32,
            (rc.gc_obj_type_and_mark >> 7) as u32,
        ] {
            num(out, n as u64, 4);
        }
    }
}
unsafe fn gc_snapshot(out: &mut Vec<u8>, rt: *mut JSRuntime, h: &Host, log: &GCLog) {
    num(out, (*rt).gc_phase as u64, 4);
    dump_gc_list(out, &mut (*rt).gc_obj_list, log);
    dump_gc_list(out, &mut (*rt).tmp_obj_list, log);
    dump_gc_list(out, &mut (*rt).gc_zero_ref_count_list, log);
    num(out, h.calls as u64, 4);
    num(out, h.live as u64, 4);
    num(out, h.trace, 8);
}
// Collection invokes callbacks which mutate the allocator host and the output
// log. Do not hold Rust shared/unique borrows of that state across those calls;
// unlike C pointers, such references allow optimized reads to be reused.
unsafe fn collect(out: *mut Vec<u8>, rt: *mut JSRuntime, h: *const Host, log: *const GCLog) {
    gc_decref(rt);
    gc_snapshot(&mut *out, rt, &*h, &*log);
    gc_scan(rt);
    gc_snapshot(&mut *out, rt, &*h, &*log);
    gc_free_cycles(rt);
    gc_snapshot(&mut *out, rt, &*h, &*log);
}
#[test]
fn official_c_reference_release_cycle_collection_and_zombies_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = gc_source(&c);
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_gc_oracle.c"));
    let directory = std::env::temp_dir().join(format!("quickjs-gc-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("gc.c");
    let executable = directory.join("gc");
    std::fs::write(&path, oracle).unwrap();
    let mut command = include!("quickjs_oracle_config.rs");
    command.args(["-std=c11", "-O2", "-I"]);
    command.arg(upstream);
    if cfg!(feature = "short-opcodes") {
        command.arg("-DSHORT_OPCODES=1");
    }
    command.arg(&path).arg("-o").arg(&executable);
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
        "C GC oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        init_list_head(&mut (*rt).gc_obj_list);
        init_list_head(&mut (*rt).gc_zero_ref_count_list);
        init_list_head(&mut (*rt).tmp_obj_list);
        init_list_head(&mut (*rt).context_list);
        assert_eq!(JS_InitAtoms(rt), 0);
        (*rt).current_exception = JS_UNINITIALIZED;
        let mut classes: Vec<JSClass> = (0..=JS_CLASS_INIT_COUNT)
            .map(|_| core::mem::zeroed())
            .collect();
        (*rt).class_count = (JS_CLASS_INIT_COUNT + 1) as i32;
        (*rt).class_array = classes.as_mut_ptr();
        classes[JS_CLASS_INIT_COUNT as usize].finalizer = Some(record_finalizer);
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        let mut names = [0; 16];
        for i in 0..16 {
            let text = std::ffi::CString::new(format!("property-{i}")).unwrap();
            names[i] = __JS_NewAtomInit(
                rt,
                text.as_ptr(),
                text.as_bytes().len() as i32,
                JS_ATOM_TYPE_STRING,
            );
        }
        let mut rng = 0x123456789abcdef0u64;
        let mut log = GCLog {
            out: &mut out,
            objects: [ptr::null_mut(); 64],
            shapes: [ptr::null_mut(); 64],
        };
        (*rt).user_opaque = ptr::from_mut(&mut log).cast();
        for trial in 0..512 {
            for i in 0..64 {
                let p = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
                log.objects[i] = p;
                (*js_rc(p.cast())).ref_count = 1;
                add_gc_object(rt, &mut (*p).header, JS_GC_OBJ_TYPE_JS_OBJECT);
                (*p).class_id = JS_CLASS_INIT_COUNT as u16;
                (*p).weakref_count = if trial % 4 == 0 && i % 7 == 0 { 1 } else { 0 };
                let sh = js_mallocz_rt(rt, get_shape_size(4, 4)).cast::<JSShape>();
                log.shapes[i] = sh;
                (*js_rc(sh.cast())).ref_count = 1;
                add_gc_object(rt, &mut (*sh).header, JS_GC_OBJ_TYPE_SHAPE);
                (*sh).prop_hash_mask = 3;
                (*sh).prop_size = 4;
                (*sh).prop_count = 4;
                (*p).shape = sh;
                (*p).prop = js_mallocz_rt(rt, size_of::<JSProperty>() * 4).cast();
            }
            for i in 0..64 {
                let pr = get_shape_prop(log.shapes[i]);
                for j in 0..4 {
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    (*pr.add(j)).atom = JS_DupAtomRT(rt, names[(i + j) % 16]);
                    let kind = rng % 4;
                    let target = ((rng >> 32) & 63) as usize;
                    let property = (*log.objects[i]).prop.add(j);
                    match kind {
                        0 => {
                            (*property).u.value = JS_DupValueRT(
                                rt,
                                JS_MKPTR(JS_TAG_OBJECT, log.objects[target].cast()),
                            )
                        }
                        1 => (*property).u.value = JS_NewInt32(&mut ctx, rng as i32),
                        2 => {
                            (*pr.add(j)).set_flags(JS_PROP_GETSET as u32);
                            (*property).u.getset.getter = log.objects[target];
                            JS_DupValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, log.objects[target].cast()));
                            if rng & 8 != 0 {
                                (*property).u.getset.setter = log.objects[(target + 1) & 63];
                                JS_DupValueRT(
                                    rt,
                                    JS_MKPTR(JS_TAG_OBJECT, log.objects[(target + 1) & 63].cast()),
                                );
                            }
                        }
                        _ => {
                            let units = [rng as u16, (rng >> 16) as u16, 1, 2, 3];
                            let s = make_string(rt, &units, ((rng >> 12) & 1) as u32);
                            (*property).u.value = JS_MKPTR(JS_TAG_STRING, s.cast());
                        }
                    }
                }
                if (trial + i) % 3 == 0 {
                    (*log.shapes[i]).proto = log.objects[(i + 13) & 63];
                    JS_DupValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, (*log.shapes[i]).proto.cast()));
                }
            }
            gc_snapshot(&mut out, rt, &h, &log);
            for i in (0..64).step_by(2) {
                JS_FreeValue(&mut ctx, JS_MKPTR(JS_TAG_OBJECT, log.objects[i].cast()));
            }
            collect(ptr::addr_of_mut!(out), rt, ptr::addr_of!(h), ptr::addr_of!(log));
            for i in (1..64).step_by(2) {
                JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, log.objects[i].cast()));
            }
            collect(ptr::addr_of_mut!(out), rt, ptr::addr_of!(h), ptr::addr_of!(log));
            if trial % 4 == 0 {
                for i in (0..64).step_by(7) {
                    let p = log.objects[i];
                    for n in [
                        JS_IsLiveObject(rt, JS_MKPTR(JS_TAG_OBJECT, p.cast())) as u32,
                        (*p).class_id as u32,
                        (*p).u.opaque.is_null() as u32,
                        (*js_rc(p.cast())).ref_count as u32,
                        ((*js_rc(p.cast())).gc_obj_type_and_mark >> 7) as u32,
                    ] {
                        num(&mut out, n as u64, 4);
                    }
                    js_free_rt(rt, p.cast());
                }
            }
            num(&mut out, list_empty(&mut (*rt).gc_obj_list) as u64, 4);
            num(&mut out, list_empty(&mut (*rt).tmp_obj_list) as u64, 4);
            snapshot(&mut out, rt, &h);
        }
        for i in 0..2048 {
            num(&mut out, JS_HasException(&mut ctx) as u64, 4);
            let val = JS_NewInt32(&mut ctx, i);
            num(
                &mut out,
                JS_VALUE_GET_TAG(JS_Throw(&mut ctx, val)) as u64,
                4,
            );
            JS_SetUncatchableException(&mut ctx, 1);
            let val = JS_NewInt32(&mut ctx, i + 1);
            num(
                &mut out,
                JS_VALUE_GET_TAG(JS_Throw(&mut ctx, val)) as u64,
                4,
            );
            num(&mut out, (*rt).current_exception_is_uncatchable as u64, 4);
            num(
                &mut out,
                JS_VALUE_GET_INT(JS_GetException(&mut ctx)) as u64,
                4,
            );
            num(&mut out, JS_HasException(&mut ctx) as u64, 4);
        }
        for name in names {
            JS_FreeAtomRT(rt, name);
        }
        cleanup(&mut out, rt, &mut h);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        let dump = std::env::temp_dir().join(format!("quickjs-gc-difference-{}", std::process::id()));
        std::fs::create_dir_all(&dump).unwrap();
        std::fs::write(dump.join("rust.bin"), &out).unwrap();
        std::fs::write(dump.join("c.bin"), &c.stdout).unwrap();
        panic!(
            "GC C/Rust mismatch at byte {i}: Rust={}, C={}; dumps: {}",
            out[i], c.stdout[i], dump.display()
        );
    }
    eprintln!(
        "quickjs.c reference/cycle GC parity: 512 graphs, 32768 objects, {} bytes",
        out.len()
    );
}
