// Full C engine is the reference for the staged property/error/scalar group.
// Test boundaries reject VM/ToPrimitive, intrinsic autoinit,
// typed-array element writes. GC uses the production collector.
mod parser_runtime {
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

    include!("../src/quickjs_parser_types.rs");
    include!("../src/quickjs_parser_lexer.rs");
    include!("../src/quickjs_compiler_scopes.rs");
    include!("../src/quickjs_parser_lookahead.rs");
    include!("../src/quickjs_compiler_functions.rs");
    include!("../src/quickjs_parser_json_lexer.rs");
    include!("../src/quickjs_compiler_lvalues.rs");
    include!("../src/quickjs_compiler_flow.rs");
    #[cfg(parser_full_source)]
    mod full_compiler_source {
        use super::*;
        include!("../src/quickjs_string_object_length.rs");
        include!("../src/quickjs_property_names.rs");
        include!("../src/quickjs_header_runtime_wrappers.rs");
        include!("quickjs_parser_execution_boundaries.rs");
    include!("../src/quickjs_parser_format.rs");
    include!("../src/quickjs_parser_c_adapters.rs");
    include!("../src/quickjs_compiler_patterns.rs");
    include!("../src/quickjs_parser_expressions.rs");
    include!("../src/quickjs_parser_generated_literals.rs");
    include!("../src/quickjs_parser_generated_destructuring.rs");
    include!("../src/quickjs_parser_generated_postfix.rs");
    include!("../src/quickjs_parser_generated_unary.rs");
    include!("../src/quickjs_parser_generated_assignment.rs");
    include!("../src/quickjs_parser_generated_statements.rs");
    include!("../src/quickjs_parser_generated_modules.rs");
    include!("../src/quickjs_compiler_generated_passes.rs");
    include!("../src/quickjs_parser_generated_functions.rs");
    include!("../src/quickjs_parser_generated_eval.rs");
    include!("quickjs_parser_bytecode_fixtures.rs");
    }
    const OP_SPECIAL_OBJECT_ARGUMENTS: i32 = 0;
    const OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS: i32 = 1;
    const OP_SPECIAL_OBJECT_THIS_FUNC: i32 = 2;
    const OP_SPECIAL_OBJECT_NEW_TARGET: i32 = 3;
    const OP_SPECIAL_OBJECT_HOME_OBJECT: i32 = 4;
    const OP_SPECIAL_OBJECT_VAR_OBJECT: i32 = 5;
    const OP_SPECIAL_OBJECT_IMPORT_META: i32 = 6;
    const HINT_STRING: i32 = 0;
    const HINT_NUMBER: i32 = 1;
    unsafe fn JS_ToPrimitive(_ctx: *mut JSContext, _v: JSValueConst, _hint: i32) -> JSValue {
        panic!("VM/ToPrimitive path is outside this oracle")
    }
    unsafe fn JS_ToPrimitiveFree(_ctx: *mut JSContext, _v: JSValue, _hint: i32) -> JSValue {
        panic!("VM/ToPrimitive path is outside this oracle")
    }
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
    unsafe fn js_module_ns_autoinit(_ctx: *mut JSContext, _p: *mut JSObject, _atom: JSAtom, _opaque: *mut c_void) -> JSValue { panic!("module namespace autoinit outside lexer oracle") }
    unsafe fn JS_InstantiateFunctionListItem2(
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
    include!("quickjs_parser_cases.rs");
    std::thread_local! {static MARKS:std::cell::RefCell<Vec<(usize,String)>>=std::cell::RefCell::new(Vec::new());}
    fn mark(out: &Vec<u8>, s: String) {
        MARKS.with(|m| m.borrow_mut().push((out.len(), s)));
    }

    unsafe fn lvalue_flow_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
        let input = b"lvalue input\0";
        let mut cache: GetLineColCache = core::mem::zeroed();
        cache.ptr = input.as_ptr();
        cache.buf_start = input.as_ptr();
        let dynamic = JS_NewAtom(ctx, c"owned_lvalue".as_ptr());
        for mask in 0..4 {
            for op in [
                OP_scope_get_var,
                OP_get_field,
                OP_scope_get_private_field,
                OP_get_array_el,
                OP_get_super_value,
                OP_push_i32,
            ] {
                for name in [
                    crate::quickjs_atom::JS_ATOM_name,
                    JS_ATOM_arguments,
                    JS_ATOM_eval,
                    JS_ATOM_this,
                    JS_ATOM_new_target,
                    dynamic,
                ] {
                    for keep in 0..2 {
                        for special in 0..5 {
                            let fd = js_new_function_def(
                                ctx,
                                ptr::null_mut(),
                                0,
                                0,
                                c"lvalue.js".as_ptr(),
                                input.as_ptr(),
                                &mut cache,
                            );
                            assert!(!fd.is_null());
                            (*fd).js_mode = if mask & 1 != 0 {
                                JS_MODE_STRICT as u8
                            } else {
                                0
                            };
                            let mut s: JSParseState = core::mem::zeroed();
                            s.ctx = ctx;
                            s.cur_func = fd;
                            s.buf_start = input.as_ptr();
                            s.token.ptr = s.buf_start;
                            s.filename = c"lvalue.js".as_ptr();
                            push_scope(&mut s);
                            if mask & 2 != 0 {
                                assert!(add_scope_var(ctx, fd, JS_ATOM__with_, JS_VAR_NORMAL) >= 0);
                            }
                            emit_op(&mut s, op as u8);
                            if op == OP_scope_get_var
                                || op == OP_get_field
                                || op == OP_scope_get_private_field
                            {
                                emit_atom(&mut s, name);
                                if op != OP_get_field {
                                    emit_u16(&mut s, (*fd).scope_level as u16);
                                }
                            } else if op == OP_push_i32 {
                                emit_u32(&mut s, 7);
                            }
                            let mut opcode = 0;
                            let mut scope = 0;
                            let mut result_name = 0;
                            let mut label = 0;
                            let mut depth = 0;
                            let ret = get_lvalue(
                                &mut s,
                                &mut opcode,
                                &mut scope,
                                &mut result_name,
                                &mut label,
                                &mut depth,
                                keep,
                                TOK_INC,
                            );
                            num(out, ret as u64, 4);
                            for n in [opcode, scope, result_name as i32, label, depth] {
                                num(out, n as u64, 4);
                            }
                            dump_exception(out, ctx);
                            if ret == 0 {
                                emit_op(&mut s, OP_inc as u8);
                                put_lvalue(&mut s, opcode, scope, result_name, label, special, 0);
                            }
                            num(out, (*fd).byte_code.size as u64, 4);
                            out.extend_from_slice(core::slice::from_raw_parts(
                                (*fd).byte_code.buf,
                                (*fd).byte_code.size,
                            ));
                            num(out, (*fd).last_opcode_pos as u64, 4);
                            num(out, (*fd).label_count as u64, 4);
                            js_free_function_def(ctx, fd);
                            num(out, (*h).calls as u64, 4);
                            num(out, (*h).live as u64, 4);
                            num(out, (*h).trace, 8);
                        }
                    }
                }
            }
        }
        JS_FreeAtom(ctx, dynamic);
        for kind in 0..4 {
            for derived in 0..2 {
                for hasval in 0..2 {
                    for iter in 0..2 {
                        let fd = js_new_function_def(
                            ctx,
                            ptr::null_mut(),
                            0,
                            0,
                            c"flow.js".as_ptr(),
                            input.as_ptr(),
                            &mut cache,
                        );
                        assert!(!fd.is_null());
                        (*fd).func_kind = kind;
                        (*fd).is_derived_class_constructor = derived;
                        let mut s: JSParseState = core::mem::zeroed();
                        s.ctx = ctx;
                        s.cur_func = fd;
                        s.buf_start = input.as_ptr();
                        s.token.ptr = s.buf_start;
                        s.filename = c"flow.js".as_ptr();
                        push_scope(&mut s);
                        let mut envs: [BlockEnv; 3] = core::mem::zeroed();
                        for i in 0..3 {
                            let lb = new_label(&mut s);
                            let lc = new_label(&mut s);
                            push_break_entry(
                                fd,
                                &mut envs[i],
                                crate::quickjs_atom::JS_ATOM_name,
                                lb,
                                lc,
                                4,
                            );
                            envs[i].set_has_iterator(iter);
                            if i == 1 {
                                envs[i].label_finally = new_label(&mut s);
                            }
                            envs[i].set_is_regular_stmt((i == 2) as u8);
                        }
                        for name in [
                            0,
                            crate::quickjs_atom::JS_ATOM_name,
                            crate::quickjs_atom::JS_ATOM_length,
                        ] {
                            for is_cont in 0..2 {
                                num(out, emit_break(&mut s, name, is_cont) as u64, 4);
                                dump_exception(out, ctx);
                                let fresh = new_label(&mut s);
                                emit_label(&mut s, fresh);
                            }
                        }
                        emit_return(&mut s, hasval);
                        for _ in 0..3 {
                            pop_break_entry(fd);
                        }
                        num(out, (*fd).byte_code.size as u64, 4);
                        out.extend_from_slice(core::slice::from_raw_parts(
                            (*fd).byte_code.buf,
                            (*fd).byte_code.size,
                        ));
                        num(out, (*fd).last_opcode_pos as u64, 4);
                        num(out, (*fd).label_count as u64, 4);
                        js_free_function_def(ctx, fd);
                        num(out, (*h).calls as u64, 4);
                        num(out, (*h).live as u64, 4);
                        num(out, (*h).trace, 8);
                    }
                }
            }
        }
    }
    unsafe fn json_lexer_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
        for ext in 0..2 {
            for (case, source) in PARSER_CASES.iter().enumerate() {
                let mut input = source.to_vec();
                input.extend_from_slice(&[0; 8]);
                let mut s: JSParseState = core::mem::zeroed();
                js_parse_init(
                    ctx,
                    &mut s,
                    input.as_ptr().cast(),
                    source.len(),
                    c"json.js".as_ptr(),
                );
                s.ext_json = ext;
                num(out, ext as u64, 4);
                num(out, case as u64, 4);
                for _ in 0..source.len() + 2 {
                    let ret = json_next_token(&mut s);
                    num(out, ret as u64, 4);
                    num(out, s.token.val as u64, 4);
                    num(out, s.token.ptr.offset_from(s.buf_start) as u64, 4);
                    num(out, s.buf_ptr.offset_from(s.buf_start) as u64, 4);
                    num(out, s.last_ptr.offset_from(s.buf_start) as u64, 4);
                    if ret == 0 {
                        match s.token.val {
                            TOK_NUMBER => dump_val(out, s.token.u.num.val),
                            TOK_STRING => {
                                dump_val(out, s.token.u.str.str);
                                num(out, s.token.u.str.sep as u64, 4);
                            }
                            TOK_IDENT => num(out, s.token.u.ident.atom as u64, 4),
                            _ => {}
                        }
                    }
                    dump_exception(out, ctx);
                    if ret < 0 || s.token.val == TOK_EOF {
                        break;
                    }
                }
                free_token(&mut s, ptr::addr_of_mut!(s.token));
                mark(out, format!("json {ext} {case}"));
                num(out, (*h).calls as u64, 4);
                num(out, (*h).live as u64, 4);
                num(out, (*h).trace, 8);
            }
        }
    }
    unsafe fn compiler_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
        let input = b"compiler source filename bytes\0";
        let mut cache: GetLineColCache = core::mem::zeroed();
        cache.ptr = input.as_ptr();
        cache.buf_start = input.as_ptr();
        for trial in 0..64 {
            num(out, trial as u64, 4);
            let fd = js_new_function_def(
                ctx,
                ptr::null_mut(),
                (trial & 1) as i32,
                0,
                c"compiler.js".as_ptr(),
                input.as_ptr(),
                &mut cache,
            );
            assert!(!fd.is_null());
            (*fd).is_global_var = ((trial >> 1) & 1) as i32;
            (*fd).eval_type = if trial & 4 != 0 {
                JS_EVAL_TYPE_MODULE
            } else {
                JS_EVAL_TYPE_GLOBAL
            };
            (*fd).js_mode = if trial & 8 != 0 {
                JS_MODE_STRICT as u8
            } else {
                0
            };
            let child = js_new_function_def(
                ctx,
                fd,
                0,
                1,
                c"child.js".as_ptr(),
                input.as_ptr().add(5),
                &mut cache,
            );
            assert!(!child.is_null());
            num(out, (*child).parent_scope_level as u64, 4);
            num(out, (*child).js_mode as u64, 4);
            num(out, (*child).source_pos as u64, 4);
            let mut s: JSParseState = core::mem::zeroed();
            s.ctx = ctx;
            s.filename = c"compiler.js".as_ptr();
            s.buf_start = input.as_ptr();
            s.token.ptr = s.buf_start;
            s.cur_func = fd;
            num(out, push_scope(&mut s) as u64, 4);
            (*fd).body_scope = (*fd).scope_level;
            let atoms = [
                crate::quickjs_atom::JS_ATOM_arguments,
                crate::quickjs_atom::JS_ATOM_name,
                crate::quickjs_atom::JS_ATOM_length,
                crate::quickjs_atom::JS_ATOM_this,
            ];
            for atom in atoms {
                num(out, add_arg(ctx, fd, atom) as u64, 4);
            }
            num(out, add_arguments_arg(ctx, fd) as u64, 4);
            num(out, add_func_var(ctx, fd, atoms[1]) as u64, 4);
            num(out, add_arguments_var(ctx, fd) as u64, 4);
            (*h).fail = if trial == 0 { 0 } else { (*h).calls + trial };
            for step in 0..128 {
                let atom = atoms[(step % 4) as usize];
                let r = match step % 8 {
                    0 => push_scope(&mut s),
                    1 => define_var(&mut s, fd, atom, (step % 7) as u32),
                    2 => add_private_class_field(
                        &mut s,
                        fd,
                        atom,
                        JS_VAR_PRIVATE_FIELD,
                        (step & 1) as i32,
                    ),
                    3 => {
                        let label = new_label(&mut s);
                        if label >= 0 {
                            emit_goto(&mut s, OP_goto as i32, label);
                            emit_label(&mut s, label);
                        }
                        label
                    }
                    4 => {
                        emit_source_pos(&mut s, input.as_ptr().add((step % 20) as usize));
                        emit_push_const(&mut s, JS_NewInt32(ctx, step), 0)
                    }
                    5 => {
                        if (*fd).scope_level > (*fd).body_scope {
                            pop_scope(&mut s);
                        }
                        0
                    }
                    6 => find_lexical_decl(ctx, fd, atom, (*fd).scope_first, 1),
                    _ => find_var(ctx, fd, atom),
                };
                num(out, r as u64, 4);
                dump_exception(out, ctx);
                if r < 0 && crate::cutils_header::dbuf_error(&(*fd).byte_code) != 0 {
                    break;
                }
            }
            (*h).fail = 0;
            num(out, (*fd).byte_code.size as u64, 4);
            out.extend_from_slice(core::slice::from_raw_parts(
                (*fd).byte_code.buf,
                (*fd).byte_code.size,
            ));
            for n in [
                (*fd).last_opcode_pos,
                (*fd).scope_level,
                (*fd).scope_first,
                (*fd).scope_count,
                (*fd).scope_size,
                (*fd).var_count,
                (*fd).var_size,
                (*fd).arg_count,
                (*fd).arg_size,
                (*fd).global_var_count,
                (*fd).global_var_size,
                (*fd).cpool_count,
                (*fd).cpool_size,
                (*fd).label_count,
                (*fd).label_size,
            ] {
                num(out, n as u64, 4);
            }
            for i in 0..(*fd).scope_count {
                let scope = &*(*fd).scopes.offset(i as isize);
                num(out, scope.parent as u64, 4);
                num(out, scope.first as u64, 4);
            }
            for i in 0..(*fd).var_count {
                let v = &*(*fd).vars.offset(i as isize);
                for n in [
                    v.var_name as i32,
                    v.scope_level,
                    v.scope_next,
                    v.flags as i32,
                    v.var_ref_idx as i32,
                    v.func_pool_idx,
                ] {
                    num(out, n as u64, 4);
                }
            }
            for i in 0..(*fd).global_var_count {
                let v = &*(*fd).global_vars.offset(i as isize);
                for n in [
                    v.cpool_idx,
                    v.force_init() as i32,
                    v.is_lexical() as i32,
                    v.is_const() as i32,
                    v.scope_level,
                    v.var_name as i32,
                ] {
                    num(out, n as u64, 4);
                }
            }
            js_free_function_def(ctx, fd);
            num(out, (*h).calls as u64, 4);
            num(out, (*h).live as u64, 4);
            num(out, (*h).trace, 8);
        }
    }
    unsafe fn fixtures(out: &mut Vec<u8>) {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut ctx: JSContext = core::mem::zeroed();
        let ctx = &mut ctx as *mut JSContext;
        let mut h = host(0);
        let mut classes: Vec<JSClass> = (0..JS_CLASS_INIT_COUNT)
            .map(|_| core::mem::zeroed())
            .collect();
        let mut protos = vec![JS_NULL; JS_CLASS_INIT_COUNT as usize];
        setup(rt, ctx, &mut h, &mut classes, &mut protos);
        for n in [
            size_of::<BlockEnv>(),
            size_of::<JSGlobalVar>(),
            size_of::<JSVarDef>(),
            size_of::<JSFunctionDef>(),
            size_of::<JSToken>(),
            size_of::<JSParseState>(),
            offset_of!(JSFunctionDef, func_name),
            offset_of!(JSFunctionDef, debug_flags),
            offset_of!(JSFunctionDef, filename),
        ] {
            num(out, n as u64, 4);
        }
        for mode in 0..16 {
            for (case, source) in PARSER_CASES.iter().enumerate() {
                let mut input = source.to_vec();
                input.extend_from_slice(&[0; 8]);
                let mut fd: JSFunctionDef = core::mem::zeroed();
                let mut parent: JSFunctionDef = core::mem::zeroed();
                fd.js_mode = if mode & 1 != 0 {
                    JS_MODE_STRICT as u8
                } else {
                    0
                };
                fd.func_kind = ((mode >> 1) & 3) as u8;
                fd.func_type = if mode & 8 != 0 {
                    JS_PARSE_FUNC_ARROW as u8
                } else {
                    JS_PARSE_FUNC_EXPR as u8
                };
                parent.func_kind = JS_FUNC_ASYNC_GENERATOR as u8;
                fd.parent = &mut parent;
                let mut s: JSParseState = core::mem::zeroed();
                s.ctx = ctx;
                s.filename = c"lexer.js".as_ptr();
                s.cur_func = &mut fd;
                s.buf_start = input.as_ptr();
                s.buf_ptr = s.buf_start;
                s.buf_end = s.buf_start.add(source.len());
                s.token.val = TOK_EOF;
                s.token.ptr = s.buf_start;
                s.is_module = (mode == 15) as i32;
                s.allow_html_comments = (s.is_module == 0) as i32;
                num(
                    out,
                    JS_DetectModule(input.as_ptr().cast(), source.len()) as u64,
                    4,
                );
                for no_lf in 0..2 {
                    let mut p = input.as_ptr();
                    num(out, simple_next_token(&mut p, no_lf) as u64, 4);
                    num(out, p.offset_from(input.as_ptr()) as u64, 4);
                }
                let mut cache: GetLineColCache = core::mem::zeroed();
                cache.ptr = input.as_ptr();
                cache.buf_start = input.as_ptr();
                for i in 0..32 {
                    let index = (i * 193) % (source.len() + 1);
                    let mut col = 0;
                    num(
                        out,
                        get_line_col_cached(&mut cache, &mut col, input.as_ptr().add(index)) as u64,
                        4,
                    );
                    num(out, col as u64, 4);
                }
                if source.starts_with(b"/")
                    && !source.starts_with(b"//")
                    && !source.starts_with(b"/*")
                {
                    let ret = js_parse_regexp(&mut s);
                    num(out, ret as u64, 4);
                    num(out, s.buf_ptr.offset_from(s.buf_start) as u64, 4);
                    if ret == 0 {
                        dump_val(out, s.token.u.regexp.body);
                        dump_val(out, s.token.u.regexp.flags);
                    }
                    dump_exception(out, ctx);
                    free_token(&mut s, ptr::addr_of_mut!(s.token));
                    s.token.val = TOK_EOF;
                    s.buf_ptr = s.buf_start;
                }
                if source.starts_with(b"(") || source.starts_with(b"[") || source.starts_with(b"{")
                {
                    let ret = next_token(&mut s);
                    assert_eq!(ret, 0);
                    let mut bits = 0;
                    num(
                        out,
                        js_parse_skip_parens_token(&mut s, &mut bits, (mode & 1) as i32) as u64,
                        4,
                    );
                    num(out, bits as u64, 4);
                    num(out, s.token.val as u64, 4);
                    num(out, s.buf_ptr.offset_from(s.buf_start) as u64, 4);
                    dump_exception(out, ctx);
                    free_token(&mut s, ptr::addr_of_mut!(s.token));
                    s.token.val = TOK_EOF;
                    s.buf_ptr = s.buf_start;
                }
                num(out, mode as u64, 4);
                num(out, case as u64, 4);
                for _ in 0..source.len() + 2 {
                    let ret = next_token(&mut s);
                    num(out, ret as u64, 4);
                    num(out, s.token.val as u64, 4);
                    num(out, s.token.ptr.offset_from(s.buf_start) as u64, 4);
                    num(out, s.buf_ptr.offset_from(s.buf_start) as u64, 4);
                    num(out, s.last_ptr.offset_from(s.buf_start) as u64, 4);
                    num(out, s.got_lf as u64, 4);
                    if ret == 0 {
                        match s.token.val {
                            TOK_NUMBER => dump_val(out, s.token.u.num.val),
                            TOK_STRING | TOK_TEMPLATE => {
                                dump_val(out, s.token.u.str.str);
                                num(out, s.token.u.str.sep as u64, 4);
                            }
                            TOK_IDENT | TOK_PRIVATE_NAME => {
                                num(out, s.token.u.ident.atom as u64, 4);
                                num(out, s.token.u.ident.has_escape as u64, 4);
                                num(out, s.token.u.ident.is_reserved as u64, 4);
                            }
                            n if (TOK_FIRST_KEYWORD..=TOK_LAST_KEYWORD).contains(&n) => {
                                num(out, s.token.u.ident.atom as u64, 4);
                                num(out, s.token.u.ident.has_escape as u64, 4);
                                num(out, s.token.u.ident.is_reserved as u64, 4);
                            }
                            _ => {}
                        }
                    }
                    dump_exception(out, ctx);
                    if ret < 0 || s.token.val == TOK_EOF {
                        break;
                    }
                }
                free_token(&mut s, ptr::addr_of_mut!(s.token));
                num(out, h.calls as u64, 4);
                num(out, h.live as u64, 4);
                num(out, h.trace, 8);
            }
        }
        mark(out, "before JSON".into());
        json_lexer_fixtures(out, ctx, &mut h);
        compiler_fixtures(out, ctx, &mut h);
        lvalue_flow_fixtures(out, ctx, &mut h);
        #[cfg(parser_full_source)]
        full_compiler_source::compiler_bytecode_fixtures(out, ctx, &mut h);
        teardown(out, rt, ctx, &mut h);
    }
    #[test]
    fn official_full_c_javascript_lexer_tokens_layouts_and_errors_match() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let upstream = root.join("../../vendor/quickjs-2026-06-04");
        let directory =
            std::env::temp_dir().join(format!("quickjs-parser-oracle-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("lexer.c");
        let executable = directory.join("lexer");
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
                .split("static void dump_object")
                .next()
                .unwrap(),
        );
        oracle.push_str(include_str!("quickjs_parser_cases.h"));
        let mut parser_oracle = include_str!("quickjs_parser_oracle.c").to_string();
        #[cfg(parser_full_source)] {
            let bytecode_oracle = include_str!("quickjs_parser_bytecode_oracle.c").replace("#include \"quickjs_parser_compiler_cases.h\"", include_str!("quickjs_parser_compiler_cases.h")).replace("#include \"quickjs_parser_module_cases.h\"", include_str!("quickjs_parser_module_cases.h"));
            parser_oracle = parser_oracle.replace("int main(void){", &(bytecode_oracle + "\nint main(void){"));
            parser_oracle = parser_oracle.replace("lvalue_flow_fixtures(&ctx,&h);", "lvalue_flow_fixtures(&ctx,&h);compiler_bytecode_fixtures(&ctx,&h);");
        }
        oracle.push_str(&parser_oracle);
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
            MARKS.with(|m| {
                eprintln!(
                    "near checkpoints {:?}",
                    m.borrow()
                        .iter()
                        .filter(|(pos, _)| *pos >= i.saturating_sub(5000) && *pos <= i + 5000)
                        .collect::<Vec<_>>()
                )
            });
            panic!(
                "parser C oracle differs at byte {i}: Rust {:?}, C {:?}; lengths {} / {}",
                &out[i.saturating_sub(16)..(i + 32).min(out.len())],
                &c.stdout[i.saturating_sub(16)..(i + 32).min(c.stdout.len())],
                out.len(),
                c.stdout.len()
            );
        }
        assert_eq!(out.len(), c.stdout.len());
        let _ = std::fs::remove_dir_all(directory);
        eprintln!("full C lexer: {} matching bytes", out.len());
    }
}
