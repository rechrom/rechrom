const CLASS_MARK_FUNCTIONS: &[&str] = &[
    "js_array_mark",
    "js_object_data_mark",
    "js_c_function_mark",
    "js_bytecode_function_mark",
    "js_bound_function_mark",
    "js_for_in_iterator_mark",
    "js_c_function_data_mark",
    "js_global_object_mark",
];
struct MarkLog {
    out: *mut Vec<u8>,
    pool: [*mut JSGCObjectHeader; 32],
}
unsafe fn trace_mark(rt: *mut JSRuntime, p: *mut JSGCObjectHeader) {
    let log = &mut *(*rt).user_opaque.cast::<MarkLog>();
    let id = log.pool.iter().position(|&x| x == p).map_or(0, |i| i + 1);
    num(&mut *log.out, id as u64, 4);
}
fn fixture_mark_value(pool: &[*mut JSGCObjectHeader; 32], index: usize) -> JSValue {
    match index % 8 {
        0 => JS_MKPTR(JS_TAG_OBJECT, pool[index % 4].cast()),
        1 => JS_MKPTR(JS_TAG_FUNCTION_BYTECODE, pool[4].cast()),
        2 => JS_MKPTR(JS_TAG_MODULE, pool[8].cast()),
        3 => JS_MKPTR(JS_TAG_STRING, 16usize as *mut c_void),
        4 => JS_UNDEFINED,
        5 => JS_MKVAL(JS_TAG_SHORT_BIG_INT, 123),
        6 => JS_MKVAL(JS_TAG_INT, 456),
        _ => JS_NULL,
    }
}
#[test]
fn official_c_all_gc_type_and_class_mark_traces_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = weak_source(&c);
    let a = c.find("typedef struct JSCFunctionDataRecord {").unwrap();
    let b = a + c[a..]
        .find("static void js_c_function_data_finalizer")
        .unwrap();
    oracle.push_str(&c[a..b]);
    source::append_functions(&mut oracle, &c, CLASS_MARK_FUNCTIONS);
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_gc_marks_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-marks-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("marks.c");
    let executable = directory.join("marks");
    std::fs::write(&path, oracle).unwrap();
    let mut command = include!("quickjs_oracle_config.rs");
    command.args(["-std=c11", "-O2", "-I"]).arg(upstream);
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
        "C marks oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        init_list_head(&mut (*rt).gc_obj_list);
        init_list_head(&mut (*rt).context_list);
        let mut log = MarkLog {
            out: &mut out,
            pool: [ptr::null_mut(); 32],
        };
        (*rt).user_opaque = ptr::from_mut(&mut log).cast();
        for i in 0..4 {
            log.pool[i] = js_mallocz_rt(rt, size_of::<JSObject>()).cast();
            (*js_rc(log.pool[i].cast())).gc_obj_type_and_mark = JS_GC_OBJ_TYPE_JS_OBJECT as u8;
        }
        for (i, size, tag) in [
            (
                4,
                size_of::<JSFunctionBytecode>(),
                JS_GC_OBJ_TYPE_FUNCTION_BYTECODE,
            ),
            (5, size_of::<JSVarRef>(), JS_GC_OBJ_TYPE_VAR_REF),
            (
                6,
                size_of::<JSAsyncFunctionState>(),
                JS_GC_OBJ_TYPE_ASYNC_FUNCTION,
            ),
            (7, size_of::<JSContext>(), JS_GC_OBJ_TYPE_JS_CONTEXT),
            (8, size_of::<JSModuleDef>(), JS_GC_OBJ_TYPE_MODULE),
            (9, get_shape_size(4, 4), JS_GC_OBJ_TYPE_SHAPE),
        ] {
            log.pool[i] = js_mallocz_rt(rt, size).cast();
            (*js_rc(log.pool[i].cast())).gc_obj_type_and_mark = tag as u8;
        }
        let pool = log.pool;
        let p = pool[0].cast::<JSObject>();
        let b = pool[4].cast::<JSFunctionBytecode>();
        let vr = pool[5].cast::<JSVarRef>();
        let af = pool[6].cast::<JSAsyncFunctionState>();
        let ctx = pool[7].cast::<JSContext>();
        let m = pool[8].cast::<JSModuleDef>();
        let sh = pool[9].cast::<JSShape>();
        let mut classes: Vec<JSClass> = (0..JS_CLASS_INIT_COUNT)
            .map(|_| core::mem::zeroed())
            .collect();
        (*rt).class_array = classes.as_mut_ptr();
        (*rt).class_count = JS_CLASS_INIT_COUNT as i32;
        (*ctx).rt = rt;
        init_list_head(&mut (*ctx).loaded_modules);
        (*ctx).class_proto =
            js_mallocz_rt(rt, (*rt).class_count as usize * size_of::<JSValue>()).cast();
        (*p).shape = sh;
        (*sh).prop_hash_mask = 3;
        (*sh).prop_size = 4;
        (*sh).prop_count = 4;
        (*p).class_id = JS_CLASS_OBJECT as u16;
        let mut props: [JSProperty; 4] = core::mem::zeroed();
        (*p).prop = props.as_mut_ptr();
        let prs = get_shape_prop(sh);
        for j in 0..4 {
            (*prs.add(j)).atom = j as u32 + 1;
        }
        let mut values = [JS_UNDEFINED; 16];
        let mut refs = [vr, ptr::null_mut(), vr, ptr::null_mut()];
        let mut req: [JSReqModuleEntry; 4] = core::mem::zeroed();
        let mut exports: [JSExportEntry; 4] = core::mem::zeroed();
        (*b).cpool = values.as_mut_ptr();
        (*b).closure_var_count = 4;
        (*b).realm = ctx;
        (*vr).pvalue = ptr::addr_of_mut!((*vr).u.value);
        (*af).frame.arg_buf = values.as_mut_ptr();
        (*af).frame.cur_sp = values.as_mut_ptr().add(16);
        (*m).req_module_entries = req.as_mut_ptr();
        (*m).req_module_entries_count = 4;
        (*m).export_entries = exports.as_mut_ptr();
        (*m).export_entries_count = 4;
        for t in 0..4096usize {
            num(&mut out, t as u64, 4);
            for j in 0..16 {
                values[j] = fixture_mark_value(&pool, t + j);
            }
            (*b).cpool_count = (t % 16) as i32;
            (*sh).proto = if t % 2 != 0 {
                pool[1].cast()
            } else {
                ptr::null_mut()
            };
            props[0].u.value = fixture_mark_value(&pool, t);
            (*prs.add(1)).set_flags(JS_PROP_GETSET as u32);
            props[1].u.getset.getter = if t % 2 != 0 {
                pool[2].cast()
            } else {
                ptr::null_mut()
            };
            props[1].u.getset.setter = if t % 3 != 0 {
                pool[3].cast()
            } else {
                ptr::null_mut()
            };
            (*prs.add(2)).set_flags(JS_PROP_VARREF as u32);
            props[2].u.var_ref = vr;
            (*prs.add(3)).set_flags(JS_PROP_AUTOINIT as u32);
            props[3].u.init.realm_and_id = ctx as usize | t % 4;
            (*vr).is_detached = (t % 2) as u8;
            if (*vr).is_detached != 0 {
                (*vr).u.value = fixture_mark_value(&pool, t + 1);
                (*vr).pvalue = ptr::addr_of_mut!((*vr).u.value);
            } else {
                (*vr).u.attached.stack_frame = &mut (*af).frame;
                (*vr).pvalue = values.as_mut_ptr();
            }
            (*af).is_completed = (t % 3 == 0) as i32;
            (*af).frame.js_mode = if t % 4 != 0 { JS_MODE_ASYNC } else { 0 };
            (*af).frame.cur_func = fixture_mark_value(&pool, t + 2);
            (*af).frame.cur_sp = if t % 5 != 0 {
                values.as_mut_ptr().add(16)
            } else {
                ptr::null_mut()
            };
            (*af).this_val = fixture_mark_value(&pool, t + 3);
            (*af).resolving_funcs[0] = fixture_mark_value(&pool, t + 4);
            (*af).resolving_funcs[1] = fixture_mark_value(&pool, t + 5);
            (*ctx).global_obj = fixture_mark_value(&pool, t);
            (*ctx).global_var_obj = fixture_mark_value(&pool, t + 1);
            (*ctx).throw_type_error = fixture_mark_value(&pool, t + 2);
            (*ctx).eval_obj = fixture_mark_value(&pool, t + 3);
            (*ctx).array_proto_values = fixture_mark_value(&pool, t + 4);
            for j in 0..JS_NATIVE_ERROR_COUNT {
                (*ctx).native_error_proto[j] = fixture_mark_value(&pool, t + j);
            }
            for j in 0..(*rt).class_count as usize {
                *(*ctx).class_proto.add(j) = fixture_mark_value(&pool, t + j);
            }
            (*ctx).iterator_ctor = fixture_mark_value(&pool, t + 5);
            (*ctx).async_iterator_proto = fixture_mark_value(&pool, t + 6);
            (*ctx).promise_ctor = fixture_mark_value(&pool, t + 7);
            (*ctx).array_ctor = fixture_mark_value(&pool, t + 8);
            (*ctx).regexp_ctor = fixture_mark_value(&pool, t + 9);
            (*ctx).function_ctor = fixture_mark_value(&pool, t + 10);
            (*ctx).function_proto = fixture_mark_value(&pool, t + 11);
            (*ctx).array_shape = sh;
            (*ctx).arguments_shape = if t % 2 != 0 { sh } else { ptr::null_mut() };
            (*ctx).mapped_arguments_shape = if t % 3 != 0 { sh } else { ptr::null_mut() };
            (*ctx).regexp_shape = if t % 4 != 0 { sh } else { ptr::null_mut() };
            (*ctx).regexp_result_shape = if t % 5 != 0 { sh } else { ptr::null_mut() };
            for j in 0..4 {
                req[j].attributes = fixture_mark_value(&pool, t + j);
                exports[j].export_type = if j % 2 != 0 {
                    JS_EXPORT_TYPE_INDIRECT
                } else {
                    JS_EXPORT_TYPE_LOCAL
                };
                exports[j].u.local.var_ref = if j % 3 != 0 { vr } else { ptr::null_mut() };
            }
            (*m).module_ns = fixture_mark_value(&pool, t);
            (*m).func_obj = fixture_mark_value(&pool, t + 1);
            (*m).eval_exception = fixture_mark_value(&pool, t + 2);
            (*m).meta_obj = fixture_mark_value(&pool, t + 3);
            (*m).promise = fixture_mark_value(&pool, t + 4);
            (*m).resolving_funcs[0] = fixture_mark_value(&pool, t + 5);
            (*m).resolving_funcs[1] = fixture_mark_value(&pool, t + 6);
            (*m).private_value = fixture_mark_value(&pool, t + 7);
            list_add_tail(&mut (*m).link, &mut (*ctx).loaded_modules);
            for j in 0..10 {
                if j == 0 || j >= 4 {
                    num(&mut out, 100 + j as u64, 4);
                    mark_children(rt, pool[j], trace_mark);
                }
            }
            list_del(&mut (*m).link);
            let val = JS_MKPTR(JS_TAG_OBJECT, p.cast());
            num(&mut out, 200, 4);
            (*p).u.array.u.values = values.as_mut_ptr();
            (*p).u.array.count = (t % 16) as u32;
            js_array_mark(rt, val, Some(trace_mark));
            num(&mut out, 201, 4);
            (*p).u.object_data = fixture_mark_value(&pool, t);
            js_object_data_mark(rt, val, Some(trace_mark));
            num(&mut out, 202, 4);
            (*p).u.cfunc.realm = if t % 2 != 0 { ctx } else { ptr::null_mut() };
            js_c_function_mark(rt, val, Some(trace_mark));
            num(&mut out, 203, 4);
            (*p).u.func.function_bytecode = if t % 2 != 0 { b } else { ptr::null_mut() };
            (*p).u.func.var_refs = if t % 3 != 0 {
                refs.as_mut_ptr()
            } else {
                ptr::null_mut()
            };
            (*p).u.func.home_object = if t % 4 != 0 {
                pool[3].cast()
            } else {
                ptr::null_mut()
            };
            js_bytecode_function_mark(rt, val, Some(trace_mark));
            num(&mut out, 204, 4);
            let mut buf = [0 as js_limb_t; 32];
            let bf = buf.as_mut_ptr().cast::<JSBoundFunction>();
            (*bf).func_obj = fixture_mark_value(&pool, t);
            (*bf).this_val = fixture_mark_value(&pool, t + 1);
            (*bf).argc = (t % 8) as i32;
            for j in 0..(*bf).argc as usize {
                *ptr::addr_of_mut!((*bf).argv).cast::<JSValue>().add(j) = values[j];
            }
            (*p).u.bound_function = bf;
            js_bound_function_mark(rt, val, Some(trace_mark));
            num(&mut out, 205, 4);
            let mut it: JSForInIterator = core::mem::zeroed();
            it.obj = fixture_mark_value(&pool, t);
            (*p).u.for_in_iterator = &mut it;
            js_for_in_iterator_mark(rt, val, Some(trace_mark));
            num(&mut out, 206, 4);
            let cd = buf.as_mut_ptr().cast::<JSCFunctionDataRecord>();
            (*cd).data_len = (t % 8) as u8;
            for j in 0..(*cd).data_len as usize {
                *ptr::addr_of_mut!((*cd).data).cast::<JSValue>().add(j) = values[j];
            }
            (*p).class_id = JS_CLASS_C_FUNCTION_DATA as u16;
            (*p).u.opaque = cd.cast();
            js_c_function_data_mark(rt, val, Some(trace_mark));
            num(&mut out, 207, 4);
            (*p).u.global_object.uninitialized_vars = fixture_mark_value(&pool, t);
            js_global_object_mark(rt, val, Some(trace_mark));
            num(&mut out, 208, 4);
            let mut fr: JSFinalizationRegistryData = core::mem::zeroed();
            let mut entries: [JSFinRecEntry; 4] = core::mem::zeroed();
            init_list_head(&mut fr.entries);
            fr.realm = ctx;
            fr.cb = fixture_mark_value(&pool, t);
            for j in 0..4 {
                entries[j].held_val = fixture_mark_value(&pool, t + j);
                list_add_tail(&mut entries[j].link, &mut fr.entries);
            }
            (*p).class_id = JS_CLASS_FINALIZATION_REGISTRY as u16;
            (*p).u.opaque = ptr::from_mut(&mut fr).cast();
            js_finrec_mark(rt, val, Some(trace_mark));
            num(&mut out, 209, 4);
            let mut ms: JSMapState = core::mem::zeroed();
            let mut mr: [JSMapRecord; 4] = core::mem::zeroed();
            init_list_head(&mut ms.records);
            ms.is_weak = (t % 2) as i32;
            for j in 0..4 {
                mr[j].key = fixture_mark_value(&pool, t + j);
                mr[j].value = fixture_mark_value(&pool, t + j + 1);
                list_add_tail(&mut mr[j].link, &mut ms.records);
            }
            (*p).u.map_state = &mut ms;
            js_map_mark(rt, val, Some(trace_mark));
            (*p).class_id = JS_CLASS_OBJECT as u16;
        }
        js_free_rt(rt, (*ctx).class_proto.cast());
        for p in &pool[..10] {
            js_free_rt(rt, (*p).cast());
        }
        assert!(h.live >= 0);
        let remaining: Vec<_> = h.allocs.keys().copied().collect();
        for p in remaining {
            host_free(&mut (*rt).malloc_ctx.malloc_state, p as *mut c_void);
        }
        assert_eq!(h.live, 0);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "mark C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c GC mark trace parity: 4096 fixtures, 7 GC types, 10 class payloads, {} bytes",
        out.len()
    );
}
