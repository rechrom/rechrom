// quickjs.c:31936..32104. Compiler function allocation and ownership. MIT.
use crate::cutils::dbuf_free;
unsafe fn js_new_function_def(
    ctx: *mut JSContext,
    parent: *mut JSFunctionDef,
    is_eval: i32,
    is_func_expr: i32,
    filename: *const c_char,
    source_ptr: *const u8,
    get_line_col_cache: *mut GetLineColCache,
) -> *mut JSFunctionDef {
    let fd = js_mallocz(ctx, size_of::<JSFunctionDef>()).cast::<JSFunctionDef>();
    if fd.is_null() {
        return ptr::null_mut();
    }
    (*fd).ctx = ctx;
    init_list_head(&mut (*fd).child_list);
    (*fd).parent = parent;
    (*fd).parent_cpool_idx = -1;
    if !parent.is_null() {
        list_add_tail(&mut (*fd).link, &mut (*parent).child_list);
        (*fd).js_mode = (*parent).js_mode;
        (*fd).parent_scope_level = (*parent).scope_level;
    }
    (*fd).set_strip_debug((((*(*ctx).rt).strip_flags & JS_STRIP_DEBUG as u8) != 0) as u8);
    (*fd).set_strip_source(
        ((*(*ctx).rt).strip_flags & (JS_STRIP_DEBUG | JS_STRIP_SOURCE) as u8 != 0) as u8,
    );
    (*fd).is_eval = is_eval;
    (*fd).is_func_expr = is_func_expr;
    js_dbuf_bytecode_init(ctx, &mut (*fd).byte_code);
    (*fd).last_opcode_pos = -1;
    (*fd).func_name = JS_ATOM_NULL as u32;
    (*fd).var_object_idx = -1;
    (*fd).arg_var_object_idx = -1;
    (*fd).arguments_var_idx = -1;
    (*fd).arguments_arg_idx = -1;
    (*fd).func_var_idx = -1;
    (*fd).eval_ret_idx = -1;
    (*fd).this_var_idx = -1;
    (*fd).new_target_var_idx = -1;
    (*fd).this_active_func_var_idx = -1;
    (*fd).home_object_var_idx = -1;
    (*fd).scopes = (*fd).def_scope_array.as_mut_ptr();
    (*fd).scope_size = 4;
    (*fd).scope_count = 1;
    (*(*fd).scopes).first = -1;
    (*(*fd).scopes).parent = -1;
    (*fd).scope_level = 0;
    (*fd).scope_first = -1;
    (*fd).body_scope = -1;
    (*fd).filename = JS_NewAtom(ctx, filename);
    (*fd).source_pos = source_ptr.offset_from((*get_line_col_cache).buf_start) as u32;
    (*fd).get_line_col_cache = get_line_col_cache;
    js_dbuf_init(ctx, &mut (*fd).pc2line);
    (*fd).last_opcode_source_ptr = source_ptr;
    fd
}
unsafe fn js_free_function_def(ctx: *mut JSContext, fd: *mut JSFunctionDef) {
    let head = ptr::addr_of_mut!((*fd).child_list);
    let mut el = (*head).next;
    while el != head {
        let next = (*el).next;
        let child = el
            .cast::<u8>()
            .sub(offset_of!(JSFunctionDef, link))
            .cast::<JSFunctionDef>();
        js_free_function_def(ctx, child);
        el = next;
    }
    free_bytecode_atoms(
        (*ctx).rt,
        (*fd).byte_code.buf,
        (*fd).byte_code.size as i32,
        (*fd).use_short_opcodes,
    );
    dbuf_free(&mut (*fd).byte_code);
    js_free(ctx, (*fd).jump_slots.cast());
    js_free(ctx, (*fd).label_slots.cast());
    js_free(ctx, (*fd).line_number_slots.cast());
    for i in 0..(*fd).cpool_count {
        JS_FreeValue(ctx, *(*fd).cpool.offset(i as isize));
    }
    js_free(ctx, (*fd).cpool.cast());
    JS_FreeAtom(ctx, (*fd).func_name);
    for i in 0..(*fd).var_count {
        JS_FreeAtom(ctx, (*(*fd).vars.offset(i as isize)).var_name);
    }
    js_free(ctx, (*fd).vars.cast());
    for i in 0..(*fd).arg_count {
        JS_FreeAtom(ctx, (*(*fd).args.offset(i as isize)).var_name);
    }
    js_free(ctx, (*fd).args.cast());
    for i in 0..(*fd).global_var_count {
        JS_FreeAtom(ctx, (*(*fd).global_vars.offset(i as isize)).var_name);
    }
    js_free(ctx, (*fd).global_vars.cast());
    for i in 0..(*fd).closure_var_count {
        JS_FreeAtom(ctx, (*(*fd).closure_var.offset(i as isize)).var_name);
    }
    js_free(ctx, (*fd).closure_var.cast());
    if (*fd).scopes != (*fd).def_scope_array.as_mut_ptr() {
        js_free(ctx, (*fd).scopes.cast());
    }
    JS_FreeAtom(ctx, (*fd).filename);
    dbuf_free(&mut (*fd).pc2line);
    js_free(ctx, (*fd).source.cast());
    if !(*fd).parent.is_null() {
        list_del(&mut (*fd).link);
    }
    js_free(ctx, fd.cast());
}
