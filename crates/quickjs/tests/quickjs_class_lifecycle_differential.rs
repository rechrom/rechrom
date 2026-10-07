const ASYNC_LIFECYCLE_FUNCTIONS: &[&str] = &[
    "JS_PromiseState",
    "JS_PromiseResult",
    "promise_reaction_data_free",
    "JS_SetHostPromiseRejectionTracker",
    "js_promise_resolve_function_free_resolved",
    "js_promise_resolve_function_finalizer",
    "js_promise_resolve_function_mark",
    "js_promise_finalizer",
    "js_promise_mark",
    "js_async_function_resolve_finalizer",
    "js_async_function_resolve_mark",
    "js_async_generator_free",
    "js_async_generator_finalizer",
    "js_async_generator_mark",
    "js_async_from_sync_iterator_finalizer",
    "js_async_from_sync_iterator_mark",
];
const LIFECYCLE_FUNCTIONS: &[&str] = &[
    "js_mapped_arguments_finalizer",
    "js_mapped_arguments_mark",
    "free_generator_stack_rt",
    "free_generator_stack",
    "js_generator_finalizer",
    "js_generator_mark",
    "js_array_iterator_finalizer",
    "js_array_iterator_mark",
    "js_regexp_finalizer",
    "js_regexp_string_iterator_finalizer",
    "js_regexp_string_iterator_mark",
    "js_proxy_finalizer",
    "js_proxy_mark",
    "js_map_iterator_finalizer",
    "js_map_iterator_mark",
    "js_array_buffer_finalizer",
    "js_typed_array_finalizer",
    "js_typed_array_mark",
];
struct LifecycleLog {
    out: *mut Vec<u8>,
    free_calls: u32,
    sab_calls: u32,
}
unsafe fn lifecycle_array_free(rt: *mut JSRuntime, opaque: *mut c_void, data: *mut c_void) {
    let log = &mut *(*rt).user_opaque.cast::<LifecycleLog>();
    log.free_calls += 1;
    num(&mut *log.out, opaque as usize as u64, 4);
    js_free_rt(rt, data);
}
unsafe fn lifecycle_sab_free(opaque: *mut c_void, data: *mut c_void) {
    let rt = opaque.cast::<JSRuntime>();
    let log = &mut *(*rt).user_opaque.cast::<LifecycleLog>();
    log.sab_calls += 1;
    num(&mut *log.out, 0xf00d, 4);
    js_free_rt(rt, data);
}
unsafe fn lifecycle_mark(rt: *mut JSRuntime, p: *mut JSGCObjectHeader) {
    let log = &mut *(*rt).user_opaque.cast::<LifecycleLog>();
    num(
        &mut *log.out,
        ((*js_rc(p.cast())).gc_obj_type_and_mark & 127) as u64,
        4,
    );
}
unsafe fn lifecycle_string(rt: *mut JSRuntime, n: usize) -> JSValue {
    JS_MKPTR(
        JS_TAG_STRING,
        make_string(rt, &[1, 2, 3, 4, 5][..n % 6], (n % 2) as u32).cast(),
    )
}
#[test]
fn official_c_class_payload_destruction_and_buffer_view_order_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = weak_source(&c);
    for (start, end) in [
        (
            "typedef enum JSGeneratorStateEnum {",
            "static void free_generator_stack_rt",
        ),
        (
            "typedef struct JSArrayIteratorData {",
            "static void js_array_iterator_finalizer",
        ),
        (
            "typedef struct JSRegExpStringIteratorData {",
            "static void js_regexp_string_iterator_finalizer",
        ),
        (
            "typedef struct JSMapIteratorData {",
            "static void js_map_iterator_finalizer",
        ),
    ] {
        let a = c.find(start).unwrap();
        let b = a + c[a..].find(end).unwrap();
        oracle.push_str(&c[a..b]);
    }
    for (start, end) in [
        (
            "typedef struct JSPromiseData {",
            "JSPromiseStateEnum JS_PromiseState",
        ),
        (
            "typedef enum JSAsyncGeneratorStateEnum {",
            "static void js_async_generator_free",
        ),
        (
            "typedef struct JSAsyncFromSyncIteratorData {",
            "static void js_async_from_sync_iterator_finalizer",
        ),
    ] {
        let a = c.find(start).unwrap();
        let b = a + c[a..].find(end).unwrap();
        oracle.push_str(&c[a..b]);
    }
    source::append_functions(&mut oracle, &c, LIFECYCLE_FUNCTIONS);
    source::append_functions(&mut oracle, &c, ASYNC_LIFECYCLE_FUNCTIONS);
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_class_lifecycle_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-lifecycle-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("lifecycle.c");
    let executable = directory.join("lifecycle");
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
        "C lifecycle oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        assert_eq!(JS_InitAtoms(rt), 0);
        init_list_head(&mut (*rt).gc_obj_list);
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        (*rt).sab_funcs.sab_opaque = rt.cast();
        let mut log = LifecycleLog {
            out: &mut out,
            free_calls: 0,
            sab_calls: 0,
        };
        (*rt).user_opaque = ptr::from_mut(&mut log).cast();
        for n in [
            size_of::<JSGeneratorData>(),
            size_of::<JSArrayIteratorData>(),
            size_of::<JSRegExpStringIteratorData>(),
            size_of::<JSMapIteratorData>(),
        ] {
            num(&mut out, n as u64, 4);
        }
        for n in [
            size_of::<JSPromiseData>(),
            size_of::<JSPromiseFunctionDataResolved>(),
            size_of::<JSPromiseFunctionData>(),
            size_of::<JSPromiseReactionData>(),
            size_of::<JSAsyncGeneratorRequest>(),
            size_of::<JSAsyncGeneratorData>(),
            size_of::<JSAsyncFromSyncIteratorData>(),
        ] {
            num(&mut out, n as u64, 4);
        }
        for t in 0..4096usize {
            let mut holder: JSObject = core::mem::zeroed();
            let val = JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut holder).cast());
            num(&mut out, t as u64, 4);
            let mut refs = [ptr::null_mut(); 4];
            for j in 0..4 {
                let vr = js_mallocz_rt(rt, size_of::<JSVarRef>()).cast::<JSVarRef>();
                refs[j] = vr;
                (*js_rc(vr.cast())).ref_count = 1;
                add_gc_object(rt, &mut (*vr).header, JS_GC_OBJ_TYPE_VAR_REF);
                (*vr).is_detached = 1;
                (*vr).u.value = lifecycle_string(rt, t + j);
                (*vr).pvalue = ptr::addr_of_mut!((*vr).u.value);
            }
            holder.u.array.u.var_refs = js_malloc_rt(rt, 4 * size_of::<*mut JSVarRef>()).cast();
            ptr::copy_nonoverlapping(refs.as_ptr(), holder.u.array.u.var_refs, 4);
            holder.u.array.count = 4;
            js_mapped_arguments_mark(rt, val, Some(lifecycle_mark));
            js_mapped_arguments_finalizer(rt, val);
            num(&mut out, list_empty(&mut (*rt).gc_obj_list) as u64, 4);
            let af =
                js_mallocz_rt(rt, size_of::<JSAsyncFunctionState>()).cast::<JSAsyncFunctionState>();
            (*js_rc(af.cast())).ref_count = 2;
            (*js_rc(af.cast())).gc_obj_type_and_mark = JS_GC_OBJ_TYPE_ASYNC_FUNCTION as u8;
            let g = js_mallocz_rt(rt, size_of::<JSGeneratorData>()).cast::<JSGeneratorData>();
            (*g).state = (t % 5) as u32;
            (*g).func_state = if t % 2 != 0 { af } else { ptr::null_mut() };
            holder.class_id = JS_CLASS_GENERATOR as u16;
            holder.u.generator_data = g;
            js_generator_mark(rt, val, Some(lifecycle_mark));
            free_generator_stack_rt(rt, g);
            num(&mut out, (*g).state as u64, 4);
            num(&mut out, (*g).func_state.is_null() as u64, 4);
            num(&mut out, (*js_rc(af.cast())).ref_count as u64, 4);
            js_generator_finalizer(rt, val);
            js_free_rt(rt, af.cast());
            let ai =
                js_mallocz_rt(rt, size_of::<JSArrayIteratorData>()).cast::<JSArrayIteratorData>();
            (*ai).obj = lifecycle_string(rt, t);
            holder.u.array_iterator_data = ai;
            js_array_iterator_mark(rt, val, Some(lifecycle_mark));
            js_array_iterator_finalizer(rt, val);
            holder.u.regexp.pattern = if t % 2 != 0 {
                ptr::null_mut()
            } else {
                JS_VALUE_GET_PTR(lifecycle_string(rt, t)).cast()
            };
            holder.u.regexp.bytecode = if t % 3 != 0 {
                ptr::null_mut()
            } else {
                JS_VALUE_GET_PTR(lifecycle_string(rt, t + 1)).cast()
            };
            js_regexp_finalizer(rt, val);
            let ri = js_mallocz_rt(rt, size_of::<JSRegExpStringIteratorData>())
                .cast::<JSRegExpStringIteratorData>();
            (*ri).iterating_regexp = lifecycle_string(rt, t);
            (*ri).iterated_string = lifecycle_string(rt, t + 1);
            holder.u.regexp_string_iterator_data = ri;
            js_regexp_string_iterator_mark(rt, val, Some(lifecycle_mark));
            js_regexp_string_iterator_finalizer(rt, val);
            let proxy = js_mallocz_rt(rt, size_of::<JSProxyData>()).cast::<JSProxyData>();
            (*proxy).target = lifecycle_string(rt, t);
            (*proxy).handler = lifecycle_string(rt, t + 1);
            holder.class_id = JS_CLASS_PROXY as u16;
            holder.u.proxy_data = proxy;
            js_proxy_mark(rt, val, Some(lifecycle_mark));
            js_proxy_finalizer(rt, val);
            let mut buffer = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
            (*js_rc(buffer.cast())).ref_count = 100;
            (*buffer).set_free_mark((t % 2) as u8);
            let mr = js_mallocz_rt(rt, size_of::<JSMapRecord>()).cast::<JSMapRecord>();
            (*mr).ref_count = 1;
            (*mr).empty = 1;
            let mut records: list_head = core::mem::zeroed();
            init_list_head(&mut records);
            list_add_tail(&mut (*mr).link, &mut records);
            let mi = js_mallocz_rt(rt, size_of::<JSMapIteratorData>()).cast::<JSMapIteratorData>();
            (*mi).obj = JS_MKPTR(JS_TAG_OBJECT, buffer.cast());
            (*mi).cur_record = if t % 3 != 0 { mr } else { ptr::null_mut() };
            holder.u.map_iterator_data = mi;
            js_map_iterator_mark(rt, val, Some(lifecycle_mark));
            js_map_iterator_finalizer(rt, val);
            num(&mut out, (*js_rc(buffer.cast())).ref_count as u64, 4);
            num(&mut out, list_empty(&mut records) as u64, 4);
            if list_empty(&mut records) == 0 {
                js_free_rt(rt, mr.cast());
            }
            js_free_rt(rt, buffer.cast());
            buffer = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
            (*js_rc(buffer.cast())).ref_count = 100;
            let ab = js_mallocz_rt(rt, size_of::<JSArrayBuffer>()).cast::<JSArrayBuffer>();
            (*ab).shared = (t % 2) as u8;
            (*ab).data = js_malloc_rt(rt, 17).cast();
            (*ab).opaque = (t + 1) as *mut c_void;
            (*ab).free_func = if t % 3 != 0 {
                Some(lifecycle_array_free)
            } else {
                None
            };
            init_list_head(&mut (*ab).array_list);
            (*rt).sab_funcs.sab_free = if t % 4 != 0 {
                Some(lifecycle_sab_free)
            } else {
                None
            };
            (*buffer).u.array_buffer = ab;
            let mut views: [JSObject; 4] = core::mem::zeroed();
            let mut tas = [ptr::null_mut(); 4];
            for j in 0..4 {
                let ta = js_mallocz_rt(rt, size_of::<JSTypedArray>()).cast::<JSTypedArray>();
                tas[j] = ta;
                (*ta).obj = &mut views[j];
                (*ta).buffer = buffer;
                (*ta).offset = 123;
                (*ta).length = 456;
                views[j].class_id = if j % 2 != 0 {
                    JS_CLASS_DATAVIEW as u16
                } else {
                    JS_CLASS_UINT8_ARRAY as u16
                };
                views[j].u.array.u.ptr = 16usize as *mut c_void;
                views[j].u.array.count = 789;
                views[j].u.typed_array = ta;
                list_add_tail(&mut (*ta).link, &mut (*ab).array_list);
                js_typed_array_mark(
                    rt,
                    JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut views[j]).cast()),
                    Some(lifecycle_mark),
                );
            }
            if t % 2 != 0 {
                js_typed_array_finalizer(
                    rt,
                    JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut views[3]).cast()),
                );
                tas[3] = ptr::null_mut();
            }
            let data = (*ab).data;
            let callback_will_free = ((*ab).shared != 0 && (*rt).sab_funcs.sab_free.is_some())
                || (*ab).free_func.is_some();
            js_array_buffer_finalizer(rt, JS_MKPTR(JS_TAG_OBJECT, buffer.cast()));
            if !callback_will_free {
                js_free_rt(rt, data.cast());
            }
            for j in 0..4 {
                let ta = tas[j];
                if !ta.is_null() {
                    for n in [
                        (*ta).link.next.is_null() as u32,
                        (*ta).link.prev.is_null() as u32,
                        (*ta).offset,
                        (*ta).length,
                    ] {
                        num(&mut out, n as u64, 4);
                    }
                    if j % 2 == 0 {
                        num(&mut out, views[j].u.array.count as u64, 4);
                        num(&mut out, views[j].u.array.u.ptr.is_null() as u64, 4);
                    }
                    js_typed_array_finalizer(
                        rt,
                        JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut views[j]).cast()),
                    );
                }
            }
            num(&mut out, (*js_rc(buffer.cast())).ref_count as u64, 4);
            js_free_rt(rt, buffer.cast());
            num(&mut out, log.free_calls as u64, 4);
            num(&mut out, log.sab_calls as u64, 4);

            let held = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
            (*js_rc(held.cast())).ref_count = 100;
            let ps = js_mallocz_rt(rt, size_of::<JSPromiseData>()).cast::<JSPromiseData>();
            (*ps).promise_state = (t % 3) as i32;
            (*ps).promise_result = async_owned(rt, held, t);
            for i in 0..2 {
                init_list_head(&mut (*ps).promise_reactions[i]);
                for j in 0..t % 5 {
                    let rd = js_mallocz_rt(rt, size_of::<JSPromiseReactionData>())
                        .cast::<JSPromiseReactionData>();
                    (*rd).resolving_funcs[0] = async_owned(rt, held, t + j);
                    (*rd).resolving_funcs[1] = async_owned(rt, held, t + j + 1);
                    (*rd).handler = async_owned(rt, held, t + j + 2);
                    list_add_tail(&mut (*rd).link, &mut (*ps).promise_reactions[i]);
                }
            }
            holder.class_id = JS_CLASS_PROMISE as u16;
            holder.u.promise_data = ps;
            num(&mut out, JS_PromiseState(&mut ctx, val) as u64, 4);
            let result = JS_PromiseResult(&mut ctx, val);
            num(&mut out, JS_VALUE_GET_TAG(result) as u64, 4);
            JS_FreeValue(&mut ctx, result);
            js_promise_mark(rt, val, Some(lifecycle_mark));
            js_promise_finalizer(rt, val);
            let sr = js_mallocz_rt(rt, size_of::<JSPromiseFunctionDataResolved>())
                .cast::<JSPromiseFunctionDataResolved>();
            (*sr).ref_count = 2;
            for j in 0..2 {
                let pf = js_mallocz_rt(rt, size_of::<JSPromiseFunctionData>())
                    .cast::<JSPromiseFunctionData>();
                (*pf).presolved = sr;
                (*pf).promise = async_owned(rt, held, t + j);
                holder.u.promise_function_data = pf;
                js_promise_resolve_function_mark(rt, val, Some(lifecycle_mark));
                js_promise_resolve_function_finalizer(rt, val);
            }
            let af =
                js_mallocz_rt(rt, size_of::<JSAsyncFunctionState>()).cast::<JSAsyncFunctionState>();
            (*js_rc(af.cast())).ref_count = 100;
            (*js_rc(af.cast())).gc_obj_type_and_mark = JS_GC_OBJ_TYPE_ASYNC_FUNCTION as u8;
            let ag =
                js_mallocz_rt(rt, size_of::<JSAsyncGeneratorData>()).cast::<JSAsyncGeneratorData>();
            (*ag).func_state = if t % 2 != 0 { af } else { ptr::null_mut() };
            init_list_head(&mut (*ag).queue);
            for j in 0..t % 5 {
                let req = js_mallocz_rt(rt, size_of::<JSAsyncGeneratorRequest>())
                    .cast::<JSAsyncGeneratorRequest>();
                (*req).result = async_owned(rt, held, t + j);
                (*req).promise = async_owned(rt, held, t + j + 1);
                (*req).resolving_funcs[0] = async_owned(rt, held, t + j + 2);
                (*req).resolving_funcs[1] = async_owned(rt, held, t + j + 3);
                list_add_tail(&mut (*req).link, &mut (*ag).queue);
            }
            holder.class_id = JS_CLASS_ASYNC_GENERATOR as u16;
            holder.u.async_generator_data = ag;
            js_async_generator_mark(rt, val, Some(lifecycle_mark));
            js_async_generator_finalizer(rt, val);
            holder.u.async_function_data = if t % 3 != 0 { af } else { ptr::null_mut() };
            js_async_function_resolve_mark(rt, val, Some(lifecycle_mark));
            js_async_function_resolve_finalizer(rt, val);
            num(&mut out, (*js_rc(af.cast())).ref_count as u64, 4);
            js_free_rt(rt, af.cast());
            let asi = js_mallocz_rt(rt, size_of::<JSAsyncFromSyncIteratorData>())
                .cast::<JSAsyncFromSyncIteratorData>();
            (*asi).sync_iter = async_owned(rt, held, t);
            (*asi).next_method = async_owned(rt, held, t + 1);
            holder.class_id = JS_CLASS_ASYNC_FROM_SYNC_ITERATOR as u16;
            holder.u.async_from_sync_iterator_data = asi;
            js_async_from_sync_iterator_mark(rt, val, Some(lifecycle_mark));
            js_async_from_sync_iterator_finalizer(rt, val);
            holder.class_id = 0;
            num(&mut out, JS_PromiseState(&mut ctx, val) as u64, 4);
            num(
                &mut out,
                JS_IsUndefined(JS_PromiseResult(&mut ctx, val)) as u64,
                4,
            );
            JS_SetHostPromiseRejectionTracker(rt, None, t as *mut c_void);
            num(
                &mut out,
                (*rt).host_promise_rejection_tracker.is_none() as u64,
                4,
            );
            num(
                &mut out,
                (*rt).host_promise_rejection_tracker_opaque as usize as u64,
                4,
            );
            num(&mut out, (*js_rc(held.cast())).ref_count as u64, 4);
            js_free_rt(rt, held.cast());
            snapshot(&mut out, rt, &h);
        }
        cleanup(&mut out, rt, &mut h);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "lifecycle C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c class lifecycle parity: 4096 destructor fixtures, {} bytes",
        out.len()
    );
}

unsafe fn async_owned(rt: *mut JSRuntime, held: *mut JSObject, n: usize) -> JSValue {
    if n % 3 != 0 {
        lifecycle_string(rt, n)
    } else {
        JS_DupValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, held.cast()))
    }
}
