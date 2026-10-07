// Full C engine is the reference for the staged property/error/scalar group.
// Test boundaries reject VM calls and intrinsic autoinit; GC uses production.
mod property_runtime {
    use super::*;
    include!("../src/quickjs_context_alloc.rs");
    include!("../src/quickjs_strings.rs");
    include!("../src/quickjs_string_conversions.rs");
    include!("../src/quickjs_string_ropes.rs");
    include!("../src/quickjs_atom_strings.rs");
    include!("../src/quickjs_header_runtime_wrappers.rs");
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
    unsafe fn JS_RunGC(rt: *mut JSRuntime) {
        super::JS_RunGC(rt)
    }
    unsafe fn js_instantiate_prototype(
        _ctx: *mut JSContext,
        _p: *mut JSObject,
        _atom: JSAtom,
        _opaque: *mut c_void,
    ) -> JSValue {
        panic!("intrinsic autoinit is outside this oracle")
    }
    unsafe fn js_module_ns_autoinit(
        _ctx: *mut JSContext,
        _p: *mut JSObject,
        _atom: JSAtom,
        _opaque: *mut c_void,
    ) -> JSValue {
        panic!("intrinsic autoinit is outside this oracle")
    }
    include!("../src/quickjs_property_writes.rs");
    include!("../src/quickjs_global_properties.rs");
    include!("../src/quickjs_private_properties.rs");
    include!("../src/quickjs_property_names.rs");
    include!("../src/quickjs_string_object_length.rs");
    include!("../src/quickjs_vm_support.rs");
    include!("../src/quickjs_intrinsic_functions.rs");
    include!("../src/quickjs_object_operations.rs");
    include!("../src/quickjs_intrinsic_objects.rs");
    include!("../src/quickjs_intrinsic_object_proto_table.rs");
    include!("../src/quickjs_intrinsic_object_table.rs");
    unsafe fn JS_Call(_ctx: *mut JSContext,_func: JSValueConst,_this: JSValueConst,_argc: i32,_argv: *mut JSValueConst) -> JSValue {panic!("VM calls excluded from Object oracle")}
    unsafe fn js_map_constructor(_ctx: *mut JSContext,_new: JSValueConst,_argc: i32,_argv: *mut JSValueConst,_magic: i32) -> JSValue {panic!("Map grouping excluded from Object oracle")}
    unsafe fn js_map_get(_ctx: *mut JSContext,_this: JSValueConst,_argc: i32,_argv: *mut JSValueConst,_magic: i32) -> JSValue {panic!("Map grouping excluded from Object oracle")}
    unsafe fn js_map_set(_ctx: *mut JSContext,_this: JSValueConst,_argc: i32,_argv: *mut JSValueConst,_magic: i32) -> JSValue {panic!("Map grouping excluded from Object oracle")}
    unsafe fn js_array_push(_ctx: *mut JSContext,_this: JSValueConst,_argc: i32,_argv: *mut JSValueConst,_magic: i32) -> JSValue {panic!("iterator grouping excluded from Object oracle")}
    unsafe fn js_create_from_ctor(_ctx: *mut JSContext,_ctor: JSValueConst,_class: i32) -> JSValue {panic!("constructor realm calls excluded from Object oracle")}
    unsafe fn JS_GetIterator(_ctx: *mut JSContext,_obj: JSValueConst,_async: i32) -> JSValue {panic!("iterator calls excluded from Object oracle")}
    unsafe fn JS_IteratorNext(_ctx: *mut JSContext,_obj: JSValueConst,_next: JSValueConst,_argc: i32,_argv: *mut JSValueConst,_done: *mut i32) -> JSValue {panic!("iterator calls excluded from Object oracle")}
    unsafe fn JS_IteratorClose(_ctx: *mut JSContext,_obj: JSValueConst,_exception: i32) -> i32 {panic!("iterator calls excluded from Object oracle")}
    unsafe fn JS_Invoke(_ctx: *mut JSContext,_obj: JSValueConst,_atom: JSAtom,_argc: i32,_argv: *mut JSValueConst) -> JSValue {panic!("VM call excluded from Object oracle")}
    include!("quickjs_writes_conversions_fixtures.rs");
    include!("quickjs_names_private_global_fixtures.rs");
    include!("quickjs_objects_ropes_fixtures.rs");
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
        for class in [JS_CLASS_NUMBER,JS_CLASS_BOOLEAN,JS_CLASS_BIG_INT,JS_CLASS_SYMBOL,JS_CLASS_STRING] {classes[class as usize].finalizer=Some(js_object_data_finalizer);}
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
    // Construct test-owned rope payloads. This checks production comparison and
    // indexing independently of the untranslated rope constructor/rebalancer.
    unsafe fn test_rope(ctx: *mut JSContext, left: JSValue, right: JSValue) -> JSValue {
        let r = js_malloc(ctx, core::mem::size_of::<JSStringRope>()).cast::<JSStringRope>();
        assert!(!r.is_null());
        (*js_rc(r.cast())).ref_count = 1;
        (*r).left = left;
        (*r).right = right;
        (*r).len = string_rope_get_len(left) + string_rope_get_len(right);
        (*r).is_wide_char = 1;
        (*r).depth = 1;
        JS_MKPTR(JS_TAG_STRING_ROPE, r.cast())
    }
    unsafe fn supplemental(out: &mut Vec<u8>, ctx: *mut JSContext) {
        let base = JS_NewObject(ctx);
        let child = JS_NewObject(ctx);
        let key = __JS_AtomFromUInt32(3);
        num(
            out,
            JS_DefinePropertyValue(ctx, base, key, JS_NewInt32(ctx, 23), JS_PROP_C_W_E) as u64,
            4,
        );
        num(out, JS_SetPrototype(ctx, child, base) as u64, 4);
        let v = JS_GetProperty(ctx, child, key);
        dump_val(out, v);
        JS_FreeValue(ctx, v);
        num(out, JS_HasProperty(ctx, child, key) as u64, 4);
        for obj in [base, child] {
            let mut d: JSPropertyDescriptor = core::mem::zeroed();
            let ret = JS_GetOwnProperty(ctx, &mut d, obj, key);
            num(out, ret as u64, 4);
            if ret > 0 {
                num(out, d.flags as u64, 4);
                dump_val(out, d.value);
                dump_val(out, d.getter);
                dump_val(out, d.setter);
                JS_FreeValue(ctx, d.value);
                JS_FreeValue(ctx, d.getter);
                JS_FreeValue(ctx, d.setter);
            }
        }
        for (obj, proto, flags) in [
            (child, child, 1),
            (base, child, 0),
            (base, child, 1),
            (child, JS_TRUE, 0),
            (JS_NewInt32(ctx, 3), JS_NULL, 1),
            (JS_NULL, JS_NULL, 1),
        ] {
            num(
                out,
                JS_SetPrototypeInternal(ctx, obj, proto, flags) as u64,
                4,
            );
            dump_exception(out, ctx);
        }
        let v = JS_GetPrototype(ctx, child);
        dump_val(out, v);
        JS_FreeValue(ctx, v);
        JS_SetImmutablePrototype(ctx, child);
        num(out, JS_SetPrototype(ctx, child, JS_NULL) as u64, 4);
        dump_exception(out, ctx);
        num(out, JS_PreventExtensions(ctx, base) as u64, 4);
        num(out, JS_IsExtensible(ctx, base) as u64, 4);
        num(out, JS_SetPrototype(ctx, base, child) as u64, 4);
        dump_exception(out, ctx);
        for obj in [child, JS_NULL, JS_UNDEFINED, JS_EXCEPTION, JS_TRUE] {
            let v = JS_GetPropertyInternal(ctx, obj, __JS_AtomFromUInt32(77), obj, 1);
            dump_val(out, v);
            JS_FreeValue(ctx, v);
            dump_exception(out, ctx);
        }
        JS_FreeValue(ctx, child);
        JS_FreeValue(ctx, base);
        let arr = JS_NewArray(ctx);
        for i in 0..12 {
            JS_DefinePropertyValueUint32(
                ctx,
                arr,
                i,
                JS_NewInt32(ctx, i as i32 * 9),
                JS_PROP_C_W_E,
            );
        }
        for idx in [-1i64, 0, 7, 11, 12, 2147483648, 4294967295, 4294967296] {
            let mut v = JS_UNDEFINED;
            num(out, JS_TryGetPropertyInt64(ctx, arr, idx, &mut v) as u64, 4);
            dump_val(out, v);
            JS_FreeValue(ctx, v);
            let v = JS_GetPropertyInt64(ctx, arr, idx);
            dump_val(out, v);
            JS_FreeValue(ctx, v);
        }
        JS_FreeValue(ctx, arr);
        let global = JS_NewObjectProtoClass(ctx, JS_NULL, JS_CLASS_GLOBAL_OBJECT);
        let gp = JS_VALUE_GET_PTR(global).cast::<JSObject>();
        (*gp).u.global_object.uninitialized_vars = JS_NewObject(ctx);
        JS_DefinePropertyValue(ctx, global, key, JS_NewInt32(ctx, 11), JS_PROP_C_W_E);
        let mut pr = ptr::null_mut();
        find_own_property(&mut pr, gp, key);
        let vr = (*pr).u.var_ref;
        (*js_rc(vr.cast())).ref_count += 1;
        num(out, delete_property(ctx, gp, key) as u64, 4);
        dump_object(out, gp);
        dump_val(out, *(*vr).pvalue);
        dump_object(
            out,
            JS_VALUE_GET_PTR((*gp).u.global_object.uninitialized_vars).cast(),
        );
        num(
            out,
            JS_DefinePropertyValue(ctx, global, key, JS_NewInt32(ctx, 12), JS_PROP_C_W_E) as u64,
            4,
        );
        dump_val(out, *(*vr).pvalue);
        dump_object(out, gp);
        free_var_ref((*ctx).rt, vr);
        JS_FreeValue(ctx, global);
        let empty = JS_NewStringLen(ctx, c"".as_ptr(), 0);
        let a = JS_NewStringLen(ctx, c"a".as_ptr(), 1);
        let b = JS_NewStringLen(ctx, c"b".as_ptr(), 1);
        let wide = js_new_string16_len(ctx, [97u16, 98].as_ptr(), 2);
        let rope = test_rope(ctx, JS_DupValue(ctx, a), JS_DupValue(ctx, b));
        let nested = test_rope(ctx, JS_DupValue(ctx, empty), JS_DupValue(ctx, rope));
        let sym = JS_NewSymbolFromAtom(ctx, crate::quickjs_atom::JS_ATOM_name, JS_ATOM_TYPE_SYMBOL);
        let sym2 =
            JS_NewSymbolFromAtom(ctx, crate::quickjs_atom::JS_ATOM_name, JS_ATOM_TYPE_SYMBOL);
        let obj = JS_NewObject(ctx);
        let obj2 = JS_NewObject(ctx);
        let vals = [
            JS_NULL,
            JS_UNDEFINED,
            JS_FALSE,
            JS_TRUE,
            JS_NewInt32(ctx, 0),
            JS_NewInt32(ctx, -1),
            __JS_NewFloat64(ctx, -0.0),
            __JS_NewFloat64(ctx, f64::from_bits(0x7ff8000000001234)),
            JS_NAN,
            __JS_NewFloat64(ctx, 0.5),
            __JS_NewShortBigInt(ctx, -1),
            __JS_NewShortBigInt(ctx, 0),
            empty,
            a,
            b,
            wide,
            rope,
            nested,
            sym,
            JS_DupValue(ctx, sym),
            sym2,
            obj,
            JS_DupValue(ctx, obj),
            obj2,
        ];
        for x in vals {
            for y in vals {
                num(out, JS_StrictEq(ctx, x, y) as u64, 4);
                num(out, JS_SameValue(ctx, x, y) as u64, 4);
                num(out, JS_SameValueZero(ctx, x, y) as u64, 4);
                if tag_is_string(JS_VALUE_GET_TAG(x) as u32) != 0
                    && tag_is_string(JS_VALUE_GET_TAG(y) as u32) != 0
                {
                    num(out, js_string_rope_compare(ctx, x, y, 0) as u64, 4);
                    num(out, js_string_rope_compare(ctx, x, y, 1) as u64, 4);
                }
            }
        }
        for string in [wide, rope, nested] {
            for idx in 0..4 {
                let v = JS_GetPropertyUint32(ctx, string, idx);
                dump_val(out, v);
                JS_FreeValue(ctx, v);
            }
            let v = JS_GetProperty(ctx, string, crate::quickjs_atom::JS_ATOM_length);
            dump_val(out, v);
            JS_FreeValue(ctx, v);
        }
        for v in vals {
            JS_FreeValue(ctx, v);
        }
        let f = JS_NewObjectProtoClass(ctx, JS_NULL, JS_CLASS_C_FUNCTION);
        JS_DefinePropertyValue(
            ctx,
            f,
            crate::quickjs_atom::JS_ATOM_name,
            JS_NewStringLen(ctx, c"native_fn".as_ptr(), 9),
            JS_PROP_C_W_E,
        );
        let mut sf: JSStackFrame = core::mem::zeroed();
        sf.cur_func = f;
        (*(*ctx).rt).current_stack_frame = &mut sf;
        let error = JS_NewError(ctx);
        build_backtrace(ctx, error, c"filename.js".as_ptr(), 12, 3, 0);
        dump_object(out, JS_VALUE_GET_PTR(error).cast());
        JS_FreeValue(ctx, error);
        let error = JS_NewError(ctx);
        build_backtrace(
            ctx,
            error,
            ptr::null(),
            0,
            0,
            JS_BACKTRACE_FLAG_SKIP_FIRST_LEVEL,
        );
        dump_object(out, JS_VALUE_GET_PTR(error).cast());
        JS_FreeValue(ctx, error);
        (*(*ctx).rt).current_stack_frame = ptr::null_mut();
        JS_FreeValue(ctx, f);
    }
    unsafe fn numeric_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
        // Allocator callbacks mutate this state through rt.opaque. A live &mut
        // Host across engine calls would incorrectly promise exclusive access.
        let mut rng = 0x123456789abcdef0u64;
        for _ in 0..16384 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let v = __JS_NewFloat64(ctx, f64::from_bits(rng));
            let mut n = 0;
            num(out, JS_ToInt32(ctx, &mut n, v) as u64, 4);
            num(out, n as u64, 4);
        }
        for text in [
            c"+Infinity",
            c"-0x1",
            c"0_1",
            c"00_1",
            c"1_2_3",
            c".1",
            c"1.e2",
            c"1e+",
            c"077",
            c"078",
            c"0b11",
            c"0o77",
            c"0xG",
            c"--1",
            c" 1",
            c"1__2",
            c"_1",
            c"1_",
            c"1e1_2",
            c"1e_2",
        ] {
            for radix in [0, 2, 8, 10, 16, 36] {
                for mask in 0..32 {
                    let flags = (if mask & 1 != 0 { ATOD_INT_ONLY } else { 0 })
                        | (if mask & 2 != 0 {
                            ATOD_ACCEPT_BIN_OCT
                        } else {
                            0
                        })
                        | (if mask & 4 != 0 {
                            ATOD_ACCEPT_LEGACY_OCTAL
                        } else {
                            0
                        })
                        | (if mask & 8 != 0 {
                            ATOD_ACCEPT_UNDERSCORES
                        } else {
                            0
                        })
                        | (if mask & 16 != 0 {
                            ATOD_ACCEPT_PREFIX_AFTER_SIGN
                        } else {
                            0
                        });
                    let mut end = ptr::null();
                    let v = js_atof(ctx, text.as_ptr(), &mut end, radix, flags);
                    num(out, end.offset_from(text.as_ptr()) as u64, 4);
                    dump_val(out, v);
                    JS_FreeValue(ctx, v);
                }
            }
        }
        for text in [
            c"0n",
            c"1234567890123456789012345678901234567890n",
            c"-0x123456789abcdef0123456789n",
            c"0b10101010101010101010101010101010101010101010101010101010101010101n",
            c"077n",
            c"1.5n",
            c"1e2n",
        ] {
            for mask in 0..16 {
                let flags = ATOD_ACCEPT_SUFFIX
                    | (if mask & 1 != 0 { ATOD_TYPE_BIG_INT } else { 0 })
                    | (if mask & 2 != 0 {
                        ATOD_ACCEPT_BIN_OCT
                    } else {
                        0
                    })
                    | (if mask & 4 != 0 {
                        ATOD_ACCEPT_LEGACY_OCTAL
                    } else {
                        0
                    })
                    | (if mask & 8 != 0 {
                        ATOD_ACCEPT_PREFIX_AFTER_SIGN
                    } else {
                        0
                    });
                let mut end = ptr::null();
                let v = js_atof(ctx, text.as_ptr(), &mut end, 0, flags);
                num(out, end.offset_from(text.as_ptr()) as u64, 4);
                dump_val(out, v);
                JS_FreeValue(ctx, v);
                dump_exception(out, ctx);
            }
        }
        num(out, (*h).calls as u64, 4);
        for radix in [2, 8, 10, 16] {
            for len in [1, 19, 20, 65, 128] {
                for negative in 0..2 {
                    let mut text = Vec::new();
                    if negative != 0 {
                        text.push(b'-');
                    }
                    for i in 0..len {
                        text.push(digits[((i * 7 + 1) % radix) as usize]);
                    }
                    text.push(0);
                    num(out, (*h).calls as u64, 4);
                    let b = js_bigint_from_string(ctx, text.as_ptr().cast(), radix);
                    assert!(!b.is_null());
                    let v = JS_CompactBigInt(ctx, b);
                    dump_val(out, v);
                    for base in 2..=36 {
                        for n in [radix, len, negative, base] {
                            num(out, n as u64, 4);
                        }
                        num(out, (*h).calls as u64, 4);
                        let s = js_bigint_to_string1(ctx, v, base);
                        dump_val(out, s);
                        JS_FreeValue(ctx, s);
                    }
                    let s = JS_ToString(ctx, v);
                    dump_val(out, s);
                    JS_FreeValue(ctx, s);
                    JS_FreeValue(ctx, v);
                }
            }
        }
        for n in [
            0,
            1,
            2147483648,
            9223372036854775807,
            9223372036854775808,
            18446744073709551615,
        ] {
            let v = JS_NewBigUint64(ctx, n);
            dump_val(out, v);
            let s = JS_ToString(ctx, v);
            dump_val(out, s);
            JS_FreeValue(ctx, s);
            JS_FreeValue(ctx, v);
        }
        for fail in 0..64 {
            num(out, fail as u64, 4);
            num(out, (*h).calls as u64, 4);
            (*h).fail = if fail == 0 { 0 } else { (*h).calls + fail };
            let text=c"-123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890";
            let b = js_bigint_from_string(ctx, text.as_ptr(), 10);
            let v = if b.is_null() {
                JS_EXCEPTION
            } else {
                JS_CompactBigInt(ctx, b)
            };
            dump_val(out, v);
            dump_exception(out, ctx);
            if JS_IsException(v) == 0 {
                for radix in 2..=36 {
                    let s = js_bigint_to_string1(ctx, v, radix);
                    dump_val(out, s);
                    JS_FreeValue(ctx, s);
                    dump_exception(out, ctx);
                }
            }
            JS_FreeValue(ctx, v);
            (*h).fail = 0;
        }
    }
    unsafe fn fixtures(out: &mut Vec<u8>) {
        for trial in 0..128 {
            let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
            let rt = &mut *rt as *mut JSRuntime;
            let mut ctx: JSContext = core::mem::zeroed();
            let ctx = &mut ctx as *mut JSContext;
            let mut h = host(0);
            let mut classes: Vec<JSClass> = (0..JS_CLASS_INIT_COUNT)
                .map(|_| core::mem::zeroed())
                .collect();
            let mut prototypes = vec![JS_NULL; JS_CLASS_INIT_COUNT as usize];
            setup(rt, ctx, &mut h, &mut classes, &mut prototypes);
            let obj = JS_NewObject(ctx);
            let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
            let getter = JS_NewObjectProtoClass(ctx, JS_NULL, JS_CLASS_C_FUNCTION);
            if trial > 0 {
                h.fail = h.calls + trial;
            }
            for step in 0..512 {
                let atom = __JS_AtomFromUInt32((step % 32) as u32);
                let val = JS_NewInt32(ctx, step);
                let flags = if step % 5 == 0 { JS_PROP_THROW } else { 0 };
                let ret = match step % 8 {
                    0 => JS_DefinePropertyValue(ctx, obj, atom, val, JS_PROP_C_W_E | flags),
                    1 => JS_DefineProperty(
                        ctx,
                        obj,
                        atom,
                        val,
                        JS_UNDEFINED,
                        JS_UNDEFINED,
                        JS_PROP_HAS_VALUE | flags,
                    ),
                    2 => JS_DefineProperty(
                        ctx,
                        obj,
                        atom,
                        JS_UNDEFINED,
                        getter,
                        JS_UNDEFINED,
                        JS_PROP_HAS_GET | JS_PROP_CONFIGURABLE | JS_PROP_HAS_CONFIGURABLE | flags,
                    ),
                    3 => JS_DefinePropertyValue(ctx, obj, atom, val, JS_PROP_CONFIGURABLE | flags),
                    4 => delete_property(ctx, p, atom),
                    5 => JS_DefineProperty(
                        ctx,
                        obj,
                        atom,
                        JS_UNDEFINED,
                        JS_UNDEFINED,
                        JS_UNDEFINED,
                        JS_PROP_HAS_WRITABLE | flags,
                    ),
                    6 => JS_DefineProperty(
                        ctx,
                        obj,
                        atom,
                        val,
                        JS_UNDEFINED,
                        JS_UNDEFINED,
                        JS_PROP_HAS_VALUE | JS_PROP_HAS_ENUMERABLE | JS_PROP_ENUMERABLE | flags,
                    ),
                    _ => JS_DefinePropertyValue(ctx, obj, atom, val, flags),
                };
                num(out, ret as u64, 4);
                dump_exception(out, ctx);
                if step % 64 == 0 {
                    dump_object(out, p);
                }
            }
            h.fail = 0;
            (*p).set_extensible(0);
            num(
                out,
                JS_DefinePropertyValueStr(
                    ctx,
                    obj,
                    c"new".as_ptr(),
                    JS_NewInt32(ctx, 1),
                    JS_PROP_THROW,
                ) as u64,
                4,
            );
            dump_exception(out, ctx);
            num(
                out,
                JS_DefinePropertyValue(ctx, JS_NULL, __JS_AtomFromUInt32(0), JS_NewInt32(ctx, 1), 0)
                    as u64,
                4,
            );
            dump_exception(out, ctx);
            JS_FreeValue(ctx, getter);
            JS_FreeValue(ctx, obj);
            let arr = JS_NewArray(ctx);
            let p = JS_VALUE_GET_PTR(arr).cast::<JSObject>();
            for i in 0..64 {
                num(
                    out,
                    JS_DefinePropertyValueUint32(
                        ctx,
                        arr,
                        i,
                        JS_NewInt32(ctx, i as i32),
                        JS_PROP_C_W_E,
                    ) as u64,
                    4,
                );
            }
            num(
                out,
                JS_DefinePropertyValueUint32(
                    ctx,
                    arr,
                    7,
                    JS_NewInt32(ctx, 70),
                    JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE,
                ) as u64,
                4,
            );
            dump_object(out, p);
            for len in [50.0, 8.0, 7.0, 0.0, -1.0, 1.5, 4294967296.0] {
                let v = JS_NewFloat64(ctx, len);
                num(
                    out,
                    JS_DefinePropertyValue(
                        ctx,
                        arr,
                        crate::quickjs_atom::JS_ATOM_length,
                        v,
                        JS_PROP_WRITABLE | JS_PROP_THROW,
                    ) as u64,
                    4,
                );
                dump_exception(out, ctx);
                dump_object(out, p);
            }
            JS_FreeValue(ctx, arr);
            // Original scalar conversion, numeric formatting and parser.
            for text in [
                c"".as_ptr(),
                c"  ".as_ptr(),
                c"0x10".as_ptr(),
                c"-0x10".as_ptr(),
                c"0b101".as_ptr(),
                c"Infinity".as_ptr(),
                c"-0".as_ptr(),
                c"1.25e2".as_ptr(),
                c"1e+".as_ptr(),
                c"123abc".as_ptr(),
            ] {
                let v =
                    JS_NewStringLen(ctx, text, core::ffi::CStr::from_ptr(text).to_bytes().len());
                let numval = JS_ToNumber(ctx, v);
                dump_val(out, numval);
                let string = JS_ToString(ctx, numval);
                dump_val(out, string);
                JS_FreeValue(ctx, string);
                JS_FreeValue(ctx, numval);
                JS_FreeValue(ctx, v);
            }
            for n in [
                f64::NEG_INFINITY,
                -0.0,
                0.0,
                1.5,
                4294967295.0,
                4294967296.0,
                9007199254740991.0,
                f64::INFINITY,
            ] {
                let v = JS_NewFloat64(ctx, n);
                let mut u = 0;
                num(out, JS_ToUint32(ctx, &mut u, v) as u64, 4);
                num(out, u as u64, 4);
                let string = JS_ToString(ctx, v);
                dump_val(out, string);
                JS_FreeValue(ctx, string);
            }
            supplemental(out, ctx);
            if trial == 0 {
                numeric_fixtures(out, ctx, &mut h);
                writes_conversions_fixtures(out, ctx, &mut h);
                names_private_global_fixtures(out, ctx, &mut h);
                objects_ropes_fixtures(out, ctx, &mut h);
            }
            num(out, h.calls as u64, 4);
            num(out, h.live as u64, 4);
            num(out, h.trace, 8);
            teardown(out, rt, ctx, &mut h);
        }
    }
    #[test]
    fn official_full_c_property_arrays_scalars_and_real_oom_exceptions_match() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let upstream = root.join("../../vendor/quickjs-2026-06-04");
        let directory =
            std::env::temp_dir().join(format!("quickjs-properties-oracle-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("props.c");
        let executable = directory.join("props");
        let mut oracle =
            String::from("#include \"quickjs.c\"\n#undef malloc\n#undef free\n#undef realloc\n");
        oracle.push_str(
            include_str!("quickjs_atoms_oracle.c")
                .split("int main(void){")
                .next()
                .unwrap(),
        );
        let (before,main)=include_str!("quickjs_properties_oracle.c").split_once("int main(void){").unwrap();
        oracle.push_str(before);
        oracle.push_str(include_str!("quickjs_writes_conversions_oracle.c"));
        oracle.push_str(include_str!("quickjs_names_private_global_oracle.c"));
        oracle.push_str(include_str!("quickjs_objects_ropes_oracle.c"));
        oracle.push_str("int main(void){");
        oracle.push_str(main);
        std::fs::write(&path, oracle).unwrap();
        let result = include!("quickjs_oracle_config.rs")
            .args(["-std=gnu11", "-O2", "-DCONFIG_VERSION=\"2026-06-04\"", "-I"])
            .arg(&upstream)
            .arg(&path)
            .arg(upstream.join("cutils.c"))
            .arg(upstream.join("libunicode.c"))
            .arg(upstream.join("libregexp.c"))
            .arg(upstream.join("dtoa.c"))
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
        assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
        let mut out = Vec::new();
        unsafe {
            fixtures(&mut out);
        }
        if let Some(i) = out.iter().zip(&c.stdout).position(|(a, b)| a != b) {
            panic!(
                "full C property/error oracle differs at byte {i}: Rust {:?}, C {:?}; lengths {} / {}",
                &out[i.saturating_sub(16)..(i+32).min(out.len())], &c.stdout[i.saturating_sub(16)..(i+32).min(c.stdout.len())],out.len(),c.stdout.len()
            );
        }
        assert_eq!(out.len(), c.stdout.len());
        let _ = std::fs::remove_dir_all(directory);
        eprintln!(
            "full C properties/arrays/scalars/OOM: {} matching bytes",
            out.len()
        );
    }
}
