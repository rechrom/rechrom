// Full C engine is the reference for the staged property/error/scalar group.
// Test boundaries reject VM/ToPrimitive, intrinsic autoinit,
// typed-array element writes and weak-GC execution.
mod intrinsic_runtime {
    use super::*;
    include!("../src/quickjs_context_alloc.rs");
    include!("../src/quickjs_strings.rs");
    include!("../src/quickjs_string_conversions.rs");
    include!("../src/quickjs_atom_strings.rs");
    include!("../src/quickjs_shape_construction.rs");
    include!("../src/quickjs_object_construction.rs");
    include!("../src/quickjs_properties.rs");
    include!("../src/quickjs_property_helpers.rs");
    include!("../src/quickjs_property_conversions.rs");
    include!("../src/quickjs_scalar_conversions.rs");
    include!("../src/quickjs_errors.rs");
    include!("../src/quickjs_prototypes.rs");
    include!("../src/quickjs_property_reads.rs");
    include!("../src/quickjs_bigint_construction.rs");
    include!("../src/quickjs_numeric_conversions.rs");
    include!("../src/quickjs_bigint_conversions.rs");
    include!("../src/quickjs_object_conversions.rs");
    include!("../src/quickjs_intrinsic_functions.rs");
    include!("../src/quickjs_string_object_length.rs");
    include!("../src/quickjs_intrinsic_exotics.rs");

    include!("../src/quickjs_runtime_init_jobs_gc.rs");
    include!("../src/quickjs_runtime_init_allocator.rs");
    include!("../src/quickjs_intrinsic_weakref.rs");
    include!("../src/quickjs_intrinsic_errors.rs");
    include!("../src/quickjs_intrinsic_function_builtins.rs");
    const JS_MAX_LOCAL_VARS: i32 = 65534;
    unsafe fn JS_Call(
        _ctx: *mut JSContext,
        _f: JSValueConst,
        _t: JSValueConst,
        _n: i32,
        _a: *mut JSValueConst,
    ) -> JSValue {
        panic!("JS call is outside this intrinsic oracle")
    }
    unsafe fn JS_CallConstructor2(
        _ctx: *mut JSContext,
        _f: JSValueConst,
        _t: JSValueConst,
        _n: i32,
        _a: *mut JSValueConst,
    ) -> JSValue {
        panic!("JS constructor call is outside this intrinsic oracle")
    }
    unsafe fn JS_EvalObject(
        _ctx: *mut JSContext,
        _t: JSValueConst,
        _s: JSValueConst,
        _f: i32,
        _scope: i32,
    ) -> JSValue {
        panic!("dynamic Function compilation is outside this intrinsic oracle")
    }
    unsafe fn JS_GetIterator(_ctx: *mut JSContext, _v: JSValueConst, _async: i32) -> JSValue {
        panic!("iterator protocol is outside this intrinsic oracle")
    }
    unsafe fn JS_IteratorNext(
        _ctx: *mut JSContext,
        _v: JSValueConst,
        _m: JSValueConst,
        _n: i32,
        _a: *mut JSValueConst,
        _d: *mut i32,
    ) -> JSValue {
        panic!("iterator protocol is outside this intrinsic oracle")
    }
    unsafe fn JS_IteratorClose(_ctx: *mut JSContext, _v: JSValueConst, _exception: i32) -> i32 {
        panic!("iterator protocol is outside this intrinsic oracle")
    }
    unsafe fn JS_ConcatString(_ctx: *mut JSContext, _a: JSValue, _b: JSValue) -> JSValue {
        panic!("full rope concat is outside this intrinsic oracle")
    }
    unsafe fn JS_OrdinaryIsInstanceOf(
        _ctx: *mut JSContext,
        _a: JSValueConst,
        _b: JSValueConst,
    ) -> i32 {
        panic!("instanceof is outside this intrinsic oracle")
    }
    unsafe fn JS_ThrowTypeErrorRevokedProxy(ctx: *mut JSContext) -> JSValue {
        JS_ThrowTypeError(ctx, c"revoked proxy".as_ptr())
    }
    unsafe fn JS_VALUE_GET_OBJ(v: JSValueConst) -> *mut JSObject {
        JS_VALUE_GET_PTR(v).cast()
    }
    unsafe fn js_module_ns_autoinit(
        _ctx: *mut JSContext,
        _p: *mut JSObject,
        _atom: JSAtom,
        _opaque: *mut c_void,
    ) -> JSValue {
        panic!("intrinsic autoinit is outside this oracle")
    }

    unsafe fn JS_SetPropertyValue(
        _ctx: *mut JSContext,
        _obj: JSValueConst,
        _prop: JSValue,
        _val: JSValue,
        _flags: i32,
    ) -> i32 {
        panic!("typed-array element writes are outside this oracle")
    }
    unsafe fn JS_CallFree(
        _ctx: *mut JSContext,
        _func: JSValue,
        _this: JSValueConst,
        _argc: i32,
        _argv: *mut JSValueConst,
    ) -> JSValue {
        panic!("VM call is outside this oracle")
    }
    unsafe fn callable(
        _ctx: *mut JSContext,
        _func: JSValueConst,
        _this: JSValueConst,
        _argc: i32,
        _argv: *mut JSValueConst,
        _flags: i32,
    ) -> JSValue {
        panic!("VM call is outside this oracle")
    }
    unsafe fn setup(
        rt: *mut JSRuntime,
        ctx: *mut JSContext,
        h: &mut Host,
        classes: &mut [JSClass],
        prototypes: &mut [JSValue],
    ) {
        initialize(rt, h);
        init_list_head(&mut (*rt).context_list);
        init_list_head(&mut (*rt).gc_obj_list);
        init_list_head(&mut (*rt).gc_zero_ref_count_list);
        init_list_head(&mut (*rt).weakref_list);
        (*rt).malloc_gc_threshold = usize::MAX;
        (*rt).current_exception = JS_UNINITIALIZED;
        assert_eq!(JS_InitAtoms(rt), 0);
        assert_eq!(init_shape_hash(rt), 0);
        (*rt).class_array = classes.as_mut_ptr();
        (*rt).class_count = classes.len() as i32;
        classes[JS_CLASS_ARRAY as usize].finalizer = Some(js_array_finalizer);
        classes[JS_CLASS_GLOBAL_OBJECT as usize].finalizer = Some(js_global_object_finalizer);
        classes[JS_CLASS_C_FUNCTION as usize].call = Some(callable);
        (*ctx).rt = rt;
        (*ctx).class_proto = prototypes.as_mut_ptr();
        (*ctx).native_error_proto.fill(JS_NULL);
        init_list_head(&mut (*ctx).loaded_modules);
        let arr = JS_NewObjectProtoClass(ctx, JS_NULL, JS_CLASS_ARRAY);
        assert_eq!(JS_IsException(arr), 0);
        (*ctx).array_shape = js_dup_shape((*JS_VALUE_GET_PTR(arr).cast::<JSObject>()).shape);
        JS_FreeValue(ctx, arr);
    }
    unsafe fn teardown(out: &mut Vec<u8>, rt: *mut JSRuntime, ctx: *mut JSContext, h: &mut Host) {
        JS_FreeValue(ctx, (*rt).current_exception);
        (*rt).current_exception = JS_UNINITIALIZED;
        js_free_shape(rt, (*ctx).array_shape);
        assert_eq!((*rt).shape_hash_count, 0);
        assert_ne!(list_empty(&mut (*rt).gc_obj_list), 0);
        js_free_rt(rt, (*rt).shape_hash.cast());
        cleanup(out, rt, h);
        assert_eq!(h.live, 0);
    }
    unsafe fn dump_val(out: &mut Vec<u8>, v: JSValue) {
        num(out, JS_VALUE_GET_NORM_TAG(v) as u64, 4);
        match JS_VALUE_GET_NORM_TAG(v) {
            JS_TAG_STRING => {
                let s = JS_VALUE_GET_PTR(v).cast::<JSString>();
                num(out, (*s).len() as u64, 4);
                for i in 0..(*s).len() {
                    num(out, string_get(s, i as i32) as u64, 4);
                }
            }
            JS_TAG_FLOAT64 => num(out, JS_VALUE_GET_FLOAT64(v).to_bits(), 8),
            JS_TAG_SHORT_BIG_INT => num(out, JS_VALUE_GET_SHORT_BIG_INT(v) as u64, 8),
            JS_TAG_BIG_INT => {
                let b = JS_VALUE_GET_PTR(v).cast::<JSBigInt>();
                num(out, (*b).len as u64, 4);
                for i in 0..(*b).len as usize {
                    num(out, *bigint_tab(b).add(i) as u64, 8);
                }
            }
            JS_TAG_OBJECT => num(
                out,
                (*JS_VALUE_GET_PTR(v).cast::<JSObject>()).class_id as u64,
                4,
            ),
            _ => num(out, JS_VALUE_GET_INT(v) as u64, 4),
        }
    }
    unsafe fn dump_exception(out: &mut Vec<u8>, ctx: *mut JSContext) {
        let has = JS_HasException(ctx);
        num(out, has as u64, 4);
        if has == 0 {
            return;
        }
        let ex = JS_GetException(ctx);
        dump_val(out, ex);
        if JS_IsObject(ex) != 0 {
            let p = JS_VALUE_GET_PTR(ex).cast::<JSObject>();
            for atom in [
                crate::quickjs_atom::JS_ATOM_message,
                crate::quickjs_atom::JS_ATOM_stack,
            ] {
                let mut pr = ptr::null_mut();
                let prs = find_own_property(&mut pr, p, atom);
                num(out, (!prs.is_null()) as u64, 4);
                if !prs.is_null() {
                    num(out, (*prs).flags() as u64, 4);
                    dump_val(out, (*pr).u.value);
                }
            }
        }
        JS_FreeValue(ctx, ex);
    }
    unsafe fn dump_object(out: &mut Vec<u8>, p: *mut JSObject) {
        let sh = (*p).shape;
        for n in [
            (*p).class_id as u32,
            (*p).extensible() as u32,
            (*p).fast_array() as u32,
            (*sh).is_hashed as u32,
            (*sh).prop_count as u32,
            (*sh).deleted_prop_count as u32,
            (*sh).prop_size as u32,
            (*sh).prop_hash_mask,
        ] {
            num(out, n as u64, 4);
        }
        for i in 0..(*sh).prop_count as usize {
            let prs = get_shape_prop(sh).add(i);
            let pr = (*p).prop.add(i);
            num(out, (*prs).atom as u64, 4);
            num(out, (*prs).flags() as u64, 4);
            match (*prs).flags() as i32 & JS_PROP_TMASK {
                JS_PROP_GETSET => {
                    num(out, !(*pr).u.getset.getter.is_null() as u64, 4);
                    num(out, !(*pr).u.getset.setter.is_null() as u64, 4);
                }
                JS_PROP_VARREF => {
                    let vr = (*pr).u.var_ref;
                    num(out, (*js_rc(vr.cast())).ref_count as u64, 4);
                    num(out, (*vr).is_const as u64, 4);
                    dump_val(out, *(*vr).pvalue);
                }
                _ => dump_val(out, (*pr).u.value),
            }
        }
        if (*p).fast_array() != 0 {
            num(out, (*p).u.array.count as u64, 4);
            num(out, (*p).u.array.u1.size as u64, 4);
            for i in 0..(*p).u.array.count {
                dump_val(out, *(*p).u.array.u.values.add(i as usize));
            }
        }
    }
    unsafe fn js_instantiate_prototype(
        ctx: *mut JSContext,
        p: *mut JSObject,
        _atom: JSAtom,
        _opaque: *mut c_void,
    ) -> JSValue {
        let this_val = JS_MKPTR(JS_TAG_OBJECT, p.cast());
        let obj = JS_NewObject(ctx);
        if JS_IsException(obj) != 0 {
            return JS_EXCEPTION;
        }
        set_cycle_flag(ctx, obj);
        set_cycle_flag(ctx, this_val);
        if JS_DefinePropertyValue(
            ctx,
            obj,
            crate::quickjs_atom::JS_ATOM_constructor,
            JS_DupValue(ctx, this_val),
            JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        ) < 0
        {
            JS_FreeValue(ctx, obj);
            return JS_EXCEPTION;
        }
        obj
    }
    unsafe fn fn_generic(
        _ctx: *mut JSContext,
        _this: JSValueConst,
        _argc: i32,
        _argv: *mut JSValueConst,
    ) -> JSValue {
        JS_UNDEFINED
    }
    unsafe fn fn_data(
        _ctx: *mut JSContext,
        _this: JSValueConst,
        _argc: i32,
        _argv: *mut JSValueConst,
        _magic: i32,
        _data: *mut JSValue,
    ) -> JSValue {
        JS_UNDEFINED
    }
    unsafe fn fn_get(_ctx: *mut JSContext, _this: JSValueConst) -> JSValue {
        JS_UNDEFINED
    }
    unsafe fn fn_set(_ctx: *mut JSContext, _this: JSValueConst, _val: JSValueConst) -> JSValue {
        JS_UNDEFINED
    }
    unsafe fn dump_func(out: &mut Vec<u8>, v: JSValue) {
        dump_object(out, JS_VALUE_GET_PTR(v).cast());
        let p = JS_VALUE_GET_PTR(v).cast::<JSObject>();
        num(out, (*p).is_constructor() as u64, 4);
        num(out, (*p).u.cfunc.length as u64, 4);
        num(out, (*p).u.cfunc.cproto as u64, 4);
        num(out, (*p).u.cfunc.magic as u64, 4);
        num(out, (*js_rc((*p).u.cfunc.realm.cast())).ref_count as u64, 4);
    }
    unsafe fn init_fixtures(out: &mut Vec<u8>) {
        let mut rt_box = Box::<JSRuntime>::new(core::mem::zeroed());
        let rt = &mut *rt_box;
        let mut initial: JSContext = core::mem::zeroed();
        let mut h = Host {
            calls: 0,
            fail: 0,
            live: 0,
            trace: 1469598103934665603,
            allocs: Default::default(),
        };
        let mut classes = Vec::new();
        classes.resize_with(JS_CLASS_INIT_COUNT as usize, || core::mem::zeroed());
        let mut protos = vec![JS_NULL; JS_CLASS_INIT_COUNT as usize];
        setup(rt, &mut initial, &mut h, &mut classes, &mut protos);
        let ctx = js_mallocz_rt(rt, size_of::<JSContext>()).cast::<JSContext>();
        num(out, h.trace, 8);
        ptr::copy_nonoverlapping(&initial, ctx, 1);
        init_list_head(&mut (*ctx).loaded_modules);
        (*js_rc(ctx.cast())).ref_count = 1000;
        (*ctx).function_proto = JS_NULL;
        classes[JS_CLASS_C_FUNCTION as usize].finalizer = Some(js_c_function_finalizer);
        classes[JS_CLASS_C_FUNCTION_DATA as usize].finalizer = Some(js_c_function_data_finalizer);
        let names = [
            c"",
            c"f",
            c"abc",
            c"雪",
            c"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789abcdefghi",
        ];
        for name in names {
            for cproto in 0..13 {
                for length in [0, 1, 255, 256] {
                    for magic in [-32769, -1, 0, 65535] {
                        for nfields in [0, 8] {
                            let v = JS_NewCFunction3(
                                ctx,
                                Some(fn_generic),
                                name.as_ptr(),
                                length,
                                cproto,
                                magic,
                                JS_NULL,
                                nfields,
                            );
                            assert_eq!(JS_IsException(v), 0);
                            dump_func(out, v);
                            JS_FreeValue(ctx, v);
                        }
                    }
                }
            }
        }
        let mut data = [
            JS_NewInt32(ctx, 123),
            JS_NewStringLen(ctx, c"data".as_ptr(), 4),
            JS_NULL,
        ];
        for n in 0..4 {
            let v = JS_NewCFunctionData(ctx, Some(fn_data), 257, -1, n, data.as_mut_ptr());
            dump_object(out, JS_VALUE_GET_PTR(v).cast());
            let s = (*JS_VALUE_GET_PTR(v).cast::<JSObject>())
                .u
                .c_function_data_record;
            num(out, (*s).length as u64, 4);
            num(out, (*s).data_len as u64, 4);
            num(out, (*s).magic as u64, 4);
            for i in 0..n {
                dump_val(
                    out,
                    *ptr::addr_of!((*s).data).cast::<JSValue>().add(i as usize),
                );
            }
            JS_FreeValue(ctx, v);
        }
        for v in data {
            JS_FreeValue(ctx, v);
        }
        let sub = [JS_PROP_INT32_DEF(c"answer".as_ptr(), 42, JS_PROP_C_W_E)];
        let fields = [
            JS_CFUNC_DEF(c"run".as_ptr(), 2, Some(fn_generic)),
            JS_PROP_STRING_DEF(c"str".as_ptr(), c"value".as_ptr(), JS_PROP_C_W_E),
            JS_PROP_INT32_DEF(c"int".as_ptr(), -17, JS_PROP_C_W_E),
            JS_PROP_INT64_DEF(c"large".as_ptr(), 9007199254740991, JS_PROP_C_W_E),
            JS_PROP_DOUBLE_DEF(c"dbl".as_ptr(), -0.0, JS_PROP_C_W_E),
            JS_PROP_UNDEFINED_DEF(c"undef".as_ptr(), JS_PROP_C_W_E),
            JS_PROP_BOOL_DEF(c"bool".as_ptr(), 1, JS_PROP_C_W_E),
            JS_PROP_ATOM_DEF(
                c"atom".as_ptr(),
                crate::quickjs_atom::JS_ATOM_name as i32,
                JS_PROP_C_W_E,
            ),
            JS_OBJECT_DEF(c"nested".as_ptr(), sub.as_ptr(), 1, JS_PROP_C_W_E),
            JS_CGETSET_DEF(c"access".as_ptr(), Some(fn_get), Some(fn_set)),
            JS_ALIAS_DEF(c"alias".as_ptr(), c"str".as_ptr()),
        ];
        let obj = JS_NewObject(ctx);
        num(
            out,
            JS_SetPropertyFunctionList(ctx, obj, fields.as_ptr(), fields.len() as i32) as u64,
            4,
        );
        for e in fields {
            let atom = find_atom(ctx, e.name);
            if e.def_type as i32 != JS_DEF_CGETSET {
                let v = JS_GetProperty(ctx, obj, atom);
                dump_val(out, v);
                if JS_IsObject(v) != 0 {
                    dump_object(out, JS_VALUE_GET_PTR(v).cast());
                }
                JS_FreeValue(ctx, v);
            }
            JS_FreeAtom(ctx, atom);
        }
        dump_object(out, JS_VALUE_GET_PTR(obj).cast());
        JS_FreeValue(ctx, obj);
        // Constructor flags and existing/parent prototypes, then break the
        // deliberate constructor cycle by deleting its configurable back-link.
        (*ctx).global_obj = JS_NewObject(ctx);
        for flags in 0..16 {
            if flags & JS_NEW_CTOR_PROTO_EXIST != 0 {
                protos[JS_CLASS_OBJECT as usize] =
                    JS_NewObjectProtoClass(ctx, JS_NULL, JS_CLASS_OBJECT);
            }
            let ctor = JS_NewCConstructor(
                ctx,
                JS_CLASS_OBJECT as i32,
                c"Custom".as_ptr(),
                Some(fn_generic),
                2,
                JS_CFUNC_constructor_or_func,
                7,
                JS_UNDEFINED,
                ptr::null(),
                0,
                ptr::null(),
                0,
                flags,
            );
            assert_eq!(JS_IsException(ctor), 0);
            dump_func(out, ctor);
            let proto = JS_GetProperty(ctx, ctor, crate::quickjs_atom::JS_ATOM_prototype);
            dump_object(out, JS_VALUE_GET_PTR(proto).cast());
            num(
                out,
                delete_property(
                    ctx,
                    JS_VALUE_GET_PTR(proto).cast(),
                    crate::quickjs_atom::JS_ATOM_constructor,
                ) as u64,
                4,
            );
            if flags & JS_NEW_CTOR_NO_GLOBAL == 0 {
                let a = JS_NewAtom(ctx, c"Custom".as_ptr());
                delete_property(ctx, JS_VALUE_GET_PTR((*ctx).global_obj).cast(), a);
                JS_FreeAtom(ctx, a);
            }
            JS_FreeValue(ctx, proto);
            JS_FreeValue(ctx, ctor);
            JS_FreeValue(ctx, protos[JS_CLASS_OBJECT as usize]);
            protos[JS_CLASS_OBJECT as usize] = JS_NULL;
        }
        JS_FreeValue(ctx, (*ctx).global_obj);
        jobs_weak_fixtures(out, ctx, &mut classes);
        num(out, (*js_rc(ctx.cast())).ref_count as u64, 4);
        assert_eq!((*js_rc(ctx.cast())).ref_count, 1000);
        num(out, h.calls as u64, 4);
        num(out, h.live as u64, 4);
        num(out, h.trace, 8);
        let shape = (*ctx).array_shape;
        js_free_shape(rt, shape);
        js_free_rt(rt, ctx.cast());
        assert_eq!((*rt).shape_hash_count, 0);
        assert_ne!(list_empty(&mut (*rt).gc_obj_list), 0);
        js_free_rt(rt, (*rt).shape_hash.cast());
        cleanup(out, rt, &mut h);
        assert_eq!(h.live, 0);
    }
    #[test]
    fn official_c_native_constructors_function_lists_and_autoinit_match() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let upstream = root.join("../../vendor/quickjs-2026-06-04");
        let dir = std::env::temp_dir().join(format!(
            "quickjs-runtime-init-oracle-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let mut oracle =
            String::from("#include \"quickjs.c\"\n#undef malloc\n#undef free\n#undef realloc\n");
        oracle.push_str(
            include_str!("quickjs_atoms_oracle.c")
                .split("int main(void){")
                .next()
                .unwrap(),
        );
        oracle.push_str(
            include_str!("quickjs_properties_oracle.c")
                .split("static JSValue test_rope")
                .next()
                .unwrap(),
        );
        oracle.push_str(include_str!("quickjs_runtime_init_functions_oracle.c"));
        let src = dir.join("init.c");
        let exe = dir.join("init");
        std::fs::write(&src, oracle).unwrap();
        let build = include!("quickjs_oracle_config.rs")
            .args(["-std=gnu11", "-O2", "-DCONFIG_VERSION=\"2026-06-04\"", "-I"])
            .arg(&upstream)
            .arg(&src)
            .arg(upstream.join("cutils.c"))
            .arg(upstream.join("libunicode.c"))
            .arg(upstream.join("libregexp.c"))
            .arg(upstream.join("dtoa.c"))
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let c = std::process::Command::new(&exe).output().unwrap();
        assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
        let mut out = Vec::new();
        unsafe {
            init_fixtures(&mut out);
        }
        if let Some(i) = out.iter().zip(&c.stdout).position(|(a, b)| a != b) {
            panic!(
                "constructor oracle mismatch byte {i}, lengths {} / {}, Rust {:?}, C {:?}",
                out.len(),
                c.stdout.len(),
                &out[i.saturating_sub(16)..(i + 24).min(out.len())],
                &c.stdout[i.saturating_sub(16)..(i + 24).min(c.stdout.len())]
            );
        }
        assert_eq!(out.len(), c.stdout.len());
        std::fs::remove_dir_all(dir).unwrap();
        eprintln!(
            "native constructors/function-list/autoinit: {} matching bytes",
            out.len()
        );
    }

    unsafe fn JS_GetFunctionRealm(ctx: *mut JSContext, func_obj: JSValueConst) -> *mut JSContext {
        if JS_VALUE_GET_TAG(func_obj) != JS_TAG_OBJECT {
            return ctx;
        }
        let p = JS_VALUE_GET_OBJ(func_obj);
        match (*p).class_id as u32 {
            JS_CLASS_C_FUNCTION => (*p).u.cfunc.realm,
            JS_CLASS_BYTECODE_FUNCTION
            | JS_CLASS_GENERATOR_FUNCTION
            | JS_CLASS_ASYNC_FUNCTION
            | JS_CLASS_ASYNC_GENERATOR_FUNCTION => (*(*p).u.func.function_bytecode).realm,
            JS_CLASS_PROXY => {
                let s = (*p).u.opaque.cast::<JSProxyData>();
                if s.is_null() {
                    ctx
                } else if (*s).is_revoked != 0 {
                    JS_ThrowTypeErrorRevokedProxy(ctx);
                    ptr::null_mut()
                } else {
                    JS_GetFunctionRealm(ctx, (*s).target)
                }
            }
            JS_CLASS_BOUND_FUNCTION => JS_GetFunctionRealm(ctx, (*(*p).u.bound_function).func_obj),
            _ => ctx,
        }
    }

    unsafe fn js_create_from_ctor(
        ctx: *mut JSContext,
        ctor: JSValueConst,
        class_id: i32,
    ) -> JSValue {
        let proto = if JS_IsUndefined(ctor) != 0 {
            JS_DupValue(ctx, *(*ctx).class_proto.add(class_id as usize))
        } else {
            let mut proto = JS_GetProperty(ctx, ctor, crate::quickjs_atom::JS_ATOM_prototype);
            if JS_IsException(proto) != 0 {
                return proto;
            }
            if JS_IsObject(proto) == 0 {
                JS_FreeValue(ctx, proto);
                let realm = JS_GetFunctionRealm(ctx, ctor);
                if realm.is_null() {
                    return JS_EXCEPTION;
                }
                proto = JS_DupValue(ctx, *(*realm).class_proto.add(class_id as usize));
            }
            proto
        };
        let obj = JS_NewObjectProtoClass(ctx, proto, class_id as u32);
        JS_FreeValue(ctx, proto);
        obj
    }

    unsafe fn JS_GetActiveFunction(ctx: *mut JSContext) -> JSValueConst {
        (*(*(*ctx).rt).current_stack_frame).cur_func
    }

    const func_kind_to_class_id: [u16; 4] = [
        JS_CLASS_BYTECODE_FUNCTION as u16,
        JS_CLASS_GENERATOR_FUNCTION as u16,
        JS_CLASS_ASYNC_FUNCTION as u16,
        JS_CLASS_ASYNC_GENERATOR_FUNCTION as u16,
    ];
    unsafe fn oracle_job(ctx: *mut JSContext, argc: i32, argv: *mut JSValueConst) -> JSValue {
        let count = (*ctx).user_opaque.cast::<i32>();
        *count += 1;
        if argc > 0 && JS_VALUE_GET_INT(*argv) < 0 {
            return JS_Throw(ctx, JS_NewInt32(ctx, -123));
        }
        JS_NewInt32(ctx, argc)
    }
    unsafe fn jobs_weak_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, classes: &mut [JSClass]) {
        let rt = (*ctx).rt;
        init_list_head(&mut (*rt).job_list);
        let mut count = 0i32;
        (*ctx).user_opaque = (&mut count as *mut i32).cast();
        for i in -5..20 {
            let mut args = [
                JS_NewInt32(ctx, i),
                JS_NewStringLen(ctx, c"job".as_ptr(), 3),
            ];
            num(
                out,
                JS_EnqueueJob(ctx, Some(oracle_job), 2, args.as_mut_ptr()) as u64,
                4,
            );
            for v in args {
                JS_FreeValue(ctx, v);
            }
        }
        num(out, JS_IsJobPending(rt) as u64, 4);
        for _ in 0..26 {
            let mut selected = ptr::null_mut();
            num(out, JS_ExecutePendingJob(rt, &mut selected) as u64, 4);
            num(out, !selected.is_null() as u64, 4);
            num(out, count as u64, 4);
            dump_exception(out, ctx);
        }
        classes[JS_CLASS_WEAK_REF as usize].finalizer = Some(js_weakref_finalizer);
        classes[JS_CLASS_FINALIZATION_REGISTRY as usize].finalizer = Some(js_finrec_finalizer);
        classes[JS_CLASS_FINALIZATION_REGISTRY as usize].gc_mark = Some(js_finrec_mark);
        for _ in 0..20 {
            let target = JS_NewObject(ctx);
            let mut args = [target];
            let wr = js_weakref_constructor(ctx, JS_TRUE, 1, args.as_mut_ptr());
            assert_eq!(JS_IsException(wr), 0);
            let v = js_weakref_deref(ctx, wr, 0, ptr::null_mut());
            dump_val(out, v);
            JS_FreeValue(ctx, v);
            JS_FreeValue(ctx, target);
            JS_RunGC(rt);
            let v = js_weakref_deref(ctx, wr, 0, ptr::null_mut());
            dump_val(out, v);
            JS_FreeValue(ctx, v);
            JS_FreeValue(ctx, wr);
        }
        let cb = JS_NewCFunction2(
            ctx,
            Some(fn_generic),
            c"cb".as_ptr(),
            1,
            JS_CFUNC_generic,
            0,
        );
        let mut args = [cb];
        let registry = js_finrec_constructor(ctx, JS_TRUE, 1, args.as_mut_ptr());
        assert_eq!(JS_IsException(registry), 0);
        let token = JS_NewObject(ctx);
        for i in 0..8 {
            let target = JS_NewObject(ctx);
            let mut args = [target, JS_NewInt32(ctx, i), token];
            let ret = js_finrec_register(ctx, registry, 3, args.as_mut_ptr());
            dump_val(out, ret);
            JS_FreeValue(ctx, ret);
            JS_FreeValue(ctx, target);
        }
        let mut args = [token];
        let v = js_finrec_unregister(ctx, registry, 1, args.as_mut_ptr());
        dump_val(out, v);
        JS_FreeValue(ctx, v);
        let v = js_finrec_unregister(ctx, registry, 1, args.as_mut_ptr());
        dump_val(out, v);
        JS_FreeValue(ctx, v);
        for i in 0..8 {
            let target = JS_NewObject(ctx);
            let mut args = [target, JS_NewInt32(ctx, i), token];
            let v = js_finrec_register(ctx, registry, 3, args.as_mut_ptr());
            dump_val(out, v);
            JS_FreeValue(ctx, v);
            JS_FreeValue(ctx, target);
        }
        JS_FreeValue(ctx, token);
        JS_RunGC(rt);
        num(out, JS_IsJobPending(rt) as u64, 4);
        for el in ListIter::new(&mut (*rt).job_list, false, true) {
            let e = el
                .cast::<u8>()
                .sub(offset_of!(JSJobEntry, link))
                .cast::<JSJobEntry>();
            num(out, (*e).argc as u64, 4);
            for i in 0..(*e).argc {
                let v = *ptr::addr_of!((*e).argv).cast::<JSValue>().add(i as usize);
                dump_val(out, v);
                JS_FreeValue(ctx, v);
            }
            JS_FreeContext((*e).realm);
            list_del(&mut (*e).link);
            js_free_rt(rt, e.cast());
        }
        JS_FreeValue(ctx, registry);
        JS_FreeValue(ctx, cb);
        (*ctx).user_opaque = ptr::null_mut();
    }
}
