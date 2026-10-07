include!("quickjs_parser_compiler_cases.rs");
include!("quickjs_parser_module_cases.rs");
unsafe fn dump_compiled(out: &mut Vec<u8>, val: JSValue) {
    if JS_VALUE_GET_TAG(val) == JS_TAG_MODULE {
        num(out, JS_TAG_MODULE as u64, 4);
        let m = JS_VALUE_GET_PTR(val).cast::<JSModuleDef>();
        for n in [(*m).module_name as i32, (*m).has_tla as i32, (*m).resolved as i32, (*m).func_created as i32, (*m).status as i32, (*m).req_module_entries_count, (*m).export_entries_count, (*m).star_export_entries_count, (*m).import_entries_count] { num(out, n as u64, 4); }
        for i in 0..(*m).req_module_entries_count as usize {
            let r = &*(*m).req_module_entries.add(i); num(out, r.module_name as u64, 4); dump_compiled(out, r.attributes);
        }
        for i in 0..(*m).export_entries_count as usize {
            let e = &*(*m).export_entries.add(i);
            for n in [e.export_type as i32, e.local_name as i32, e.export_name as i32, if e.export_type == JS_EXPORT_TYPE_LOCAL { e.u.local.var_idx } else { e.u.req_module_idx }] { num(out, n as u64, 4); }
        }
        for i in 0..(*m).star_export_entries_count as usize { num(out, (*(*m).star_export_entries.add(i)).req_module_idx as u64, 4); }
        for i in 0..(*m).import_entries_count as usize {
            let e = &*(*m).import_entries.add(i);
            for n in [e.var_idx, e.is_star, e.import_name as i32, e.req_module_idx] { num(out, n as u64, 4); }
        }
        dump_compiled(out, (*m).func_obj); return;
    }
    if JS_VALUE_GET_TAG(val) == JS_TAG_OBJECT {
        num(out, JS_TAG_OBJECT as u64, 4);
        let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
        num(out, (*p).class_id as u64, 4); num(out, (*p).extensible() as u64, 4); num(out, (*p).fast_array() as u64, 4);
        let sh = (*p).shape; num(out, (*sh).prop_count as u64, 4);
        for i in 0..(*sh).prop_count as usize {
            let prs = &*get_shape_prop(sh).add(i); num(out, prs.atom as u64, 4); num(out, prs.flags() as u64, 4);
            if prs.atom != 0 { assert_eq!(prs.flags() as i32 & JS_PROP_TMASK, 0); dump_compiled(out, (*(*p).prop.add(i)).u.value); }
        }
        if (*p).fast_array() != 0 { let a = (*p).u.array; num(out, a.count as u64, 4); for i in 0..a.count as usize { dump_compiled(out, *a.u.values.add(i)); } }
        return;
    }
    if JS_VALUE_GET_TAG(val) != JS_TAG_FUNCTION_BYTECODE {
        dump_val(out, val); return;
    }
    num(out, JS_TAG_FUNCTION_BYTECODE as u64, 4);
    let b = JS_VALUE_GET_PTR(val).cast::<JSFunctionBytecode>();
    for n in [(*b).js_mode as i32, (*b).has_prototype() as i32, (*b).has_simple_parameter_list() as i32,
        (*b).is_derived_class_constructor() as i32, (*b).need_home_object() as i32, (*b).func_kind() as i32,
        (*b).new_target_allowed() as i32, (*b).super_call_allowed() as i32, (*b).super_allowed() as i32,
        (*b).arguments_allowed() as i32, (*b).has_debug() as i32, (*b).read_only_bytecode() as i32,
        (*b).is_direct_or_indirect_eval() as i32, (*b).byte_code_len, (*b).func_name as i32,
        (*b).arg_count as i32, (*b).var_count as i32, (*b).defined_arg_count as i32, (*b).stack_size as i32,
        (*b).var_ref_count as i32, (*b).cpool_count, (*b).closure_var_count] { num(out, n as u64, 4); }
    out.extend_from_slice(core::slice::from_raw_parts((*b).byte_code_buf, (*b).byte_code_len as usize));
    num(out, (!(*b).vardefs.is_null()) as u64, 4);
    if !(*b).vardefs.is_null() {
        for i in 0..(*b).arg_count as usize + (*b).var_count as usize {
            let v = &*(*b).vardefs.add(i);
            for n in [v.var_name as i32, v.scope_next, v.is_const() as i32, v.is_lexical() as i32,
                v.is_captured() as i32, v.has_scope() as i32, v.var_kind() as i32, v.var_ref_idx as i32] { num(out, n as u64, 4); }
        }
    }
    for i in 0..(*b).closure_var_count as usize {
        let v = &*(*b).closure_var.add(i);
        for n in [v.closure_type() as i32, v.is_lexical() as i32, v.is_const() as i32, v.var_kind() as i32,
            v.var_idx as i32, v.var_name as i32] { num(out, n as u64, 4); }
    }
    if (*b).has_debug() != 0 {
        for n in [(*b).debug.filename as i32, (*b).debug.source_len, (*b).debug.pc2line_len, (!(*b).debug.source.is_null()) as i32] { num(out, n as u64, 4); }
        if (*b).debug.pc2line_len != 0 { out.extend_from_slice(core::slice::from_raw_parts((*b).debug.pc2line_buf, (*b).debug.pc2line_len as usize)); }
        if !(*b).debug.source.is_null() { out.extend_from_slice(core::slice::from_raw_parts((*b).debug.source.cast(), (*b).debug.source_len as usize)); }
    }
    for i in 0..(*b).cpool_count as usize { dump_compiled(out, *(*b).cpool.add(i)); }
}
pub(super) unsafe fn compiler_bytecode_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
    // Bytecode retains its realm with allocator-owned reference counts.
    // The lexer fixture's stack context has no malloc block header.
    let base = ctx;
    let ctx = js_mallocz_rt((*base).rt, size_of::<JSContext>()).cast::<JSContext>();
    assert!(!ctx.is_null());
    core::ptr::copy_nonoverlapping(base, ctx, 1);
    init_list_head(core::ptr::addr_of_mut!((*ctx).loaded_modules));
    (*js_rc(ctx.cast())).ref_count = 1;
    (*ctx).eval_internal = Some(__JS_EvalInternal);
    for strip in 0..4 { for strict in 0..2 { for (i, source) in COMPILER_CASES.iter().enumerate() {
        mark(out, format!("bytecode strip {strip} strict {strict} case {i}"));
        (*(*ctx).rt).strip_flags = strip;
        let mut input = source.to_vec(); input.resize(source.len() + 8, 0);
        let val = JS_Eval(ctx, input.as_ptr().cast(), source.len(), c"compiler.js".as_ptr(), JS_EVAL_FLAG_COMPILE_ONLY | if strict != 0 { JS_EVAL_FLAG_STRICT } else { 0 });
        dump_compiled(out, val); dump_exception(out, ctx); JS_FreeValue(ctx, val);
        num(out, (*h).calls as u64, 4); num(out, (*h).live as u64, 4); num(out, (*h).trace, 8);
    } } }
    for strip in 0..4 { for (i, source) in MODULE_CASES.iter().enumerate() {
        mark(out, format!("module strip {strip} case {i}"));
        (*(*ctx).rt).strip_flags = strip;
        let mut input = source.to_vec(); input.resize(source.len() + 8, 0);
        let val = JS_Eval(ctx, input.as_ptr().cast(), source.len(), c"module.js".as_ptr(), JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
        dump_compiled(out, val); dump_exception(out, ctx); JS_FreeValue(ctx, val);
        js_free_modules(ctx, JS_FREE_MODULE_ALL);
        num(out, (*h).calls as u64, 4); num(out, (*h).live as u64, 4); num(out, (*h).trace, 8);
    } }
    (*(*ctx).rt).strip_flags = 0;
    assert_eq!((*js_rc(ctx.cast())).ref_count, 1);
    js_free_rt((*ctx).rt, ctx.cast());
}
