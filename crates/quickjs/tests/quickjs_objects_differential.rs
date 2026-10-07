unsafe fn object_fn_a(
    _ctx: *mut JSContext,
    _t: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    JS_UNDEFINED
}
unsafe fn object_fn_b(
    _ctx: *mut JSContext,
    _t: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    JS_NULL
}
#[test]
fn official_c_object_execution_layouts_flags_and_property_lookup_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    let a = c.find("typedef enum {\n    JS_CLOSURE_LOCAL").unwrap();
    let b = a + c[a..].find("enum {\n    __JS_ATOM_NULL").unwrap();
    oracle.push_str(&c[a..b]);
    source::append_functions(&mut oracle, &c, ATOM_FUNCTIONS);
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "get_shape_size",
            "get_shape_prop",
            "JS_GetClassID",
            "JS_IsFunction",
            "JS_IsCFunction",
            "JS_IsConstructor",
            "JS_SetConstructorBit",
            "JS_IsError",
            "JS_SetUncatchableException",
            "JS_SetOpaque",
            "JS_GetOpaque",
            "JS_SetIsHTMLDDA",
            "JS_IsHTMLDDA",
            "find_own_property1",
            "find_own_property",
            "js_autoinit_get_realm",
            "js_autoinit_get_id",
            "JS_DupContext",
            "JS_IsJobPending",
        ],
    );
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_objects_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-objects-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("quickjs_object_layouts.inc"),
        include_str!("quickjs_object_layouts.inc"),
    )
    .unwrap();
    let path = directory.join("objects.c");
    let executable = directory.join("objects");
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
        "C object oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    include!("quickjs_object_layouts.rs");
    for v in 0..256 {
        let mut c: JSClosureVar = unsafe { core::mem::zeroed() };
        c.set_closure_type(v);
        c.set_is_lexical((v >> 3) as u8);
        c.set_is_const((v >> 4) as u8);
        c.set_var_kind((v >> 2) as u8);
        num(&mut out, c.closure_flags as u64, 1);
        num(&mut out, c.var_kind_storage as u64, 1);
        for n in [
            c.closure_type(),
            c.is_lexical() as u32,
            c.is_const() as u32,
            c.var_kind() as u32,
        ] {
            num(&mut out, n as u64, 4);
        }
        let mut d: JSBytecodeVarDef = unsafe { core::mem::zeroed() };
        d.set_is_const(v as u8);
        d.set_is_lexical((v >> 1) as u8);
        d.set_is_captured((v >> 2) as u8);
        d.set_has_scope((v >> 3) as u8);
        d.set_var_kind((v >> 4) as u8);
        num(&mut out, d.var_flags as u64, 1);
        let mut b: JSFunctionBytecode = unsafe { core::mem::zeroed() };
        b.set_has_prototype(v as u8);
        b.set_has_simple_parameter_list((v >> 1) as u8);
        b.set_is_derived_class_constructor((v >> 2) as u8);
        b.set_need_home_object((v >> 3) as u8);
        b.set_func_kind((v >> 4) as u8);
        b.set_new_target_allowed((v >> 6) as u8);
        b.set_super_call_allowed((v >> 7) as u8);
        b.set_super_allowed(!v as u8);
        b.set_arguments_allowed((!v >> 1) as u8);
        b.set_has_debug((!v >> 2) as u8);
        b.set_read_only_bytecode((!v >> 3) as u8);
        b.set_is_direct_or_indirect_eval((!v >> 4) as u8);
        num(&mut out, b.function_flags[0] as u64, 1);
        num(&mut out, b.function_flags[1] as u64, 1);
        num(&mut out, b.func_kind() as u64, 4);
        num(&mut out, b.has_debug() as u64, 4);
        let mut o: JSObject = unsafe { core::mem::zeroed() };
        o.set_is_std_array_prototype(v as u8);
        o.set_extensible((v >> 1) as u8);
        o.set_free_mark((v >> 2) as u8);
        o.set_is_exotic((v >> 3) as u8);
        o.set_fast_array((v >> 4) as u8);
        o.set_is_constructor((v >> 5) as u8);
        o.set_has_immutable_prototype((v >> 6) as u8);
        o.set_tmp_mark((v >> 7) as u8);
        o.set_is_HTMLDDA(v as u8);
        num(&mut out, o.object_flags[0] as u64, 1);
        num(&mut out, o.object_flags[1] as u64, 1);
        let mut m: JSModuleDef = unsafe { core::mem::zeroed() };
        m.has_tla = v as i8;
        m.resolved = !v as i8;
        m.func_created = (v as i32 - 128) as i8;
        m.status = v as u8;
        m.eval_has_exception = v as i8;
        for n in [
            m.has_tla as i32,
            m.resolved as i32,
            m.func_created as i32,
            m.status as i32,
            m.eval_has_exception as i32,
        ] {
            num(&mut out, n as u64, 4);
        }
    }
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        let mut classes: Vec<JSClass> = (0..JS_CLASS_INIT_COUNT)
            .map(|i| {
                let mut c: JSClass = core::mem::zeroed();
                if i & 1 != 0 {
                    c.call = Some(test_call);
                }
                c
            })
            .collect();
        (*rt).class_count = JS_CLASS_INIT_COUNT as i32;
        (*rt).class_array = classes.as_mut_ptr();
        let mut proxy: JSProxyData = core::mem::zeroed();
        let mut object: JSObject = core::mem::zeroed();
        let mut rng = 0x123456789abcdef0u64;
        for step in 0..16384 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            object.class_id = (step % JS_CLASS_INIT_COUNT) as u16;
            object.u.opaque = ptr::null_mut();
            if object.class_id as u32 == JS_CLASS_PROXY {
                proxy.is_func = (step % 256) as u8;
                object.u.proxy_data = &mut proxy;
            }
            if object.class_id as u32 == JS_CLASS_C_FUNCTION {
                object.u.cfunc.c_function.generic = if step & 1 != 0 {
                    Some(object_fn_a)
                } else {
                    Some(object_fn_b)
                };
                object.u.cfunc.magic = rng as i16;
            }
            let val = if step % 7 == 0 {
                JS_UNDEFINED
            } else {
                JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut object).cast())
            };
            num(&mut out, JS_GetClassID(val) as u64, 4);
            num(&mut out, JS_IsFunction(&mut ctx, val) as u64, 4);
            for f in [
                Some(object_fn_a as JSCFunction),
                Some(object_fn_b as JSCFunction),
                None,
            ] {
                num(
                    &mut out,
                    JS_IsCFunction(&mut ctx, val, f, rng as i16 as i32) as u64,
                    4,
                );
            }
            num(
                &mut out,
                JS_SetConstructorBit(&mut ctx, val, rng as i32) as u64,
                4,
            );
            num(&mut out, JS_IsConstructor(&mut ctx, val) as u64, 4);
            num(&mut out, JS_IsError(&mut ctx, val) as u64, 4);
            JS_SetUncatchableException(&mut ctx, rng as i32);
            num(&mut out, (*rt).current_exception_is_uncatchable as u64, 4);
            JS_SetIsHTMLDDA(&mut ctx, val);
            num(&mut out, JS_IsHTMLDDA(&mut ctx, val) as u64, 4);
            JS_SetOpaque(val, rng as usize as *mut c_void);
            num(
                &mut out,
                JS_GetOpaque(val, object.class_id as u32) as usize as u64,
                8,
            );
            num(
                &mut out,
                JS_GetOpaque(val, object.class_id as u32 + 1) as usize as u64,
                8,
            );
        }
        let mut h = host(0);
        initialize(rt, &mut h);
        let heap_ctx = js_mallocz_rt(rt, size_of::<JSContext>()).cast::<JSContext>();
        (*js_rc(heap_ctx.cast())).ref_count = 1;
        for _ in 0..1024 {
            num(&mut out, (JS_DupContext(heap_ctx) == heap_ctx) as u64, 4);
            num(&mut out, (*js_rc(heap_ctx.cast())).ref_count as u64, 4);
        }
        js_free_rt(rt, heap_ctx.cast());
        init_list_head(&mut (*rt).job_list);
        let mut jobs: Vec<list_head> = (0..32).map(|_| core::mem::zeroed()).collect();
        for j in &mut jobs {
            num(&mut out, JS_IsJobPending(rt) as u64, 4);
            list_add_tail(j, &mut (*rt).job_list);
        }
        for j in &mut jobs {
            list_del(j);
            num(&mut out, JS_IsJobPending(rt) as u64, 4);
        }
        for n in 0..=256 {
            let mut hash_size = 4;
            while hash_size < n {
                hash_size *= 2;
            }
            let sh = js_mallocz_rt(rt, get_shape_size(hash_size, n)).cast::<JSShape>();
            (*sh).prop_hash_mask = (hash_size - 1) as u32;
            (*sh).prop_size = n as i32;
            (*sh).prop_count = n as i32;
            let pr = get_shape_prop(sh);
            let values = js_mallocz_rt(rt, size_of::<JSProperty>() * n).cast::<JSProperty>();
            object.shape = sh;
            object.prop = values;
            let table = ptr::addr_of_mut!((*sh).hash_table).cast::<u32>();
            for i in 0..n {
                let atom = (1 + (i * 263) % 129) as u32;
                (*pr.add(i)).atom = atom;
                let k = (atom & (*sh).prop_hash_mask) as usize;
                (*pr.add(i)).set_hash_next(*table.add(k));
                *table.add(k) = i as u32 + 1;
                (*values.add(i)).u.value = JS_NewInt32(&mut ctx, i as i32);
            }
            for atom in 0..512 {
                let a = find_own_property1(&mut object, atom);
                let mut v = 1usize as *mut JSProperty;
                let b = find_own_property(&mut v, &mut object, atom);
                num(
                    &mut out,
                    if a.is_null() {
                        0
                    } else {
                        1 + a.offset_from(pr) as u64
                    },
                    4,
                );
                num(
                    &mut out,
                    if b.is_null() {
                        0
                    } else {
                        1 + b.offset_from(pr) as u64
                    },
                    4,
                );
                num(
                    &mut out,
                    if v.is_null() {
                        0
                    } else {
                        1 + v.offset_from(values) as u64
                    },
                    4,
                );
                if !v.is_null() {
                    num(&mut out, JS_VALUE_GET_INT((*v).u.value) as u64, 4);
                }
            }
            js_free_rt(rt, sh.cast());
            js_free_rt(rt, values.cast());
        }
        for n in 0..256 {
            let mut p: JSProperty = core::mem::zeroed();
            p.u.init.realm_and_id = 0x123456789abcdef0u64 as usize | (n & 3);
            num(&mut out, js_autoinit_get_realm(&mut p) as usize as u64, 8);
            num(&mut out, js_autoinit_get_id(&mut p) as u64, 4);
        }
        num(&mut out, h.calls as u64, 4);
        num(&mut out, h.live as u64, 4);
        num(&mut out, h.trace, 8);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "object C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!("quickjs.c object/execution layouts parity: 16384 objects, 131584 property lookups, {} bytes",out.len());
}
