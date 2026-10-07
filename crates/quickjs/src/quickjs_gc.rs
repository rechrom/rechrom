// quickjs.c reference release, GC object destruction and traversal. MIT.
// Class finalizers/markers remain injected through the original class table.
type JSFreeModuleEnum = u32;
const JS_FREE_MODULE_ALL: JSFreeModuleEnum = 0;
const JS_FREE_MODULE_NOT_RESOLVED: JSFreeModuleEnum = 1;
unsafe fn js_free_modules(ctx: *mut JSContext, flag: JSFreeModuleEnum) {
    for el in ListIter::new(&mut (*ctx).loaded_modules, false, true) {
        let m = el
            .cast::<u8>()
            .sub(offset_of!(JSModuleDef, link))
            .cast::<JSModuleDef>();
        if flag == JS_FREE_MODULE_ALL || (flag == JS_FREE_MODULE_NOT_RESOLVED && (*m).resolved == 0)
        {
            list_del(&mut (*m).link);
            (*m).link.prev = ptr::null_mut();
            (*m).link.next = ptr::null_mut();
            JS_FreeValue(ctx, JS_MKPTR(JS_TAG_MODULE, m.cast()));
        }
    }
}
pub unsafe fn JS_FreeContext(ctx: *mut JSContext) {
    let rt = (*ctx).rt;
    (*js_rc(ctx.cast())).ref_count -= 1;
    if (*js_rc(ctx.cast())).ref_count > 0 {
        return;
    }
    assert_eq!((*js_rc(ctx.cast())).ref_count, 0);
    js_free_modules(ctx, JS_FREE_MODULE_ALL);
    JS_FreeValue(ctx, (*ctx).global_obj);
    JS_FreeValue(ctx, (*ctx).global_var_obj);
    JS_FreeValue(ctx, (*ctx).throw_type_error);
    JS_FreeValue(ctx, (*ctx).eval_obj);
    JS_FreeValue(ctx, (*ctx).array_proto_values);
    for i in 0..JS_NATIVE_ERROR_COUNT {
        JS_FreeValue(ctx, (*ctx).native_error_proto[i]);
    }
    let mut i = 0;
    while i < (*rt).class_count {
        JS_FreeValue(ctx, *(*ctx).class_proto.add(i as usize));
        i += 1;
    }
    js_free_rt(rt, (*ctx).class_proto.cast());
    JS_FreeValue(ctx, (*ctx).iterator_ctor);
    JS_FreeValue(ctx, (*ctx).async_iterator_proto);
    JS_FreeValue(ctx, (*ctx).promise_ctor);
    JS_FreeValue(ctx, (*ctx).array_ctor);
    JS_FreeValue(ctx, (*ctx).regexp_ctor);
    JS_FreeValue(ctx, (*ctx).function_ctor);
    JS_FreeValue(ctx, (*ctx).function_proto);
    js_free_shape_null(rt, (*ctx).array_shape);
    js_free_shape_null(rt, (*ctx).arguments_shape);
    js_free_shape_null(rt, (*ctx).mapped_arguments_shape);
    js_free_shape_null(rt, (*ctx).regexp_shape);
    js_free_shape_null(rt, (*ctx).regexp_result_shape);
    list_del(&mut (*ctx).link);
    remove_gc_object(&mut (*ctx).header);
    js_free_rt(rt, ctx.cast());
}
unsafe fn js_free_shape0(rt: *mut JSRuntime, sh: *mut JSShape) {
    assert_eq!((*js_rc(sh.cast())).ref_count, 0);
    if (*sh).is_hashed != 0 {
        js_shape_hash_unlink(rt, sh);
    }
    if !(*sh).proto.is_null() {
        JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, (*sh).proto.cast()));
    }
    let pr = get_shape_prop(sh);
    let mut i = 0;
    while i < (*sh).prop_count {
        JS_FreeAtomRT(rt, (*pr.add(i as usize)).atom);
        i += 1;
    }
    remove_gc_object(&mut (*sh).header);
    js_free_rt(rt, sh.cast());
}
unsafe fn js_free_shape(rt: *mut JSRuntime, sh: *mut JSShape) {
    (*js_rc(sh.cast())).ref_count -= 1;
    if (*js_rc(sh.cast())).ref_count <= 0 {
        js_free_shape0(rt, sh);
    }
}
unsafe fn js_free_shape_null(rt: *mut JSRuntime, sh: *mut JSShape) {
    if !sh.is_null() {
        js_free_shape(rt, sh);
    }
}
unsafe fn js_autoinit_free(_rt: *mut JSRuntime, pr: *mut JSProperty) {
    JS_FreeContext(js_autoinit_get_realm(pr));
}
unsafe fn js_autoinit_mark(rt: *mut JSRuntime, pr: *mut JSProperty, mark_func: JS_MarkFunc) {
    mark_func(rt, ptr::addr_of_mut!((*js_autoinit_get_realm(pr)).header));
}
unsafe fn free_property(rt: *mut JSRuntime, pr: *mut JSProperty, prop_flags: i32) {
    match prop_flags & JS_PROP_TMASK {
        JS_PROP_GETSET => {
            if !(*pr).u.getset.getter.is_null() {
                JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.getter.cast()));
            }
            if !(*pr).u.getset.setter.is_null() {
                JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.setter.cast()));
            }
        }
        JS_PROP_VARREF => free_var_ref(rt, (*pr).u.var_ref),
        JS_PROP_AUTOINIT => js_autoinit_free(rt, pr),
        _ => JS_FreeValueRT(rt, (*pr).u.value),
    }
}
unsafe fn free_var_ref(rt: *mut JSRuntime, var_ref: *mut JSVarRef) {
    if var_ref.is_null() {
        return;
    }
    assert!((*js_rc(var_ref.cast())).ref_count > 0);
    (*js_rc(var_ref.cast())).ref_count -= 1;
    if (*js_rc(var_ref.cast())).ref_count == 0 {
        if (*var_ref).is_detached != 0 {
            JS_FreeValueRT(rt, (*var_ref).u.value);
        } else {
            let sf = (*var_ref).u.attached.stack_frame;
            let index = (*var_ref).u.attached.var_ref_idx as usize;
            assert_eq!(*(*sf).var_refs.add(index), var_ref);
            *(*sf).var_refs.add(index) = ptr::null_mut();
            if (*sf).js_mode & JS_MODE_ASYNC != 0 {
                let async_func = sf
                    .cast::<u8>()
                    .sub(offset_of!(JSAsyncFunctionState, frame))
                    .cast();
                async_func_free(rt, async_func);
            }
        }
        remove_gc_object(&mut (*var_ref).header);
        js_free_rt(rt, var_ref.cast());
    }
}
unsafe fn async_func_free_frame(rt: *mut JSRuntime, s: *mut JSAsyncFunctionState) {
    js_host_free_frame_diagnostics(rt, ptr::addr_of_mut!((*s).frame));
    let sf = ptr::addr_of_mut!((*s).frame);
    assert!(!(*sf).cur_sp.is_null());
    let mut sp = (*sf).arg_buf;
    while sp < (*sf).cur_sp {
        JS_FreeValueRT(rt, *sp);
        sp = sp.add(1);
    }
    JS_FreeValueRT(rt, (*sf).cur_func);
    JS_FreeValueRT(rt, (*s).this_val);
}
unsafe fn __async_func_free(rt: *mut JSRuntime, s: *mut JSAsyncFunctionState) {
    if (*s).is_completed == 0 {
        async_func_free_frame(rt, s);
    }
    JS_FreeValueRT(rt, (*s).resolving_funcs[0]);
    JS_FreeValueRT(rt, (*s).resolving_funcs[1]);
    remove_gc_object(&mut (*s).header);
    if (*rt).gc_phase == JS_GC_PHASE_REMOVE_CYCLES as u8 && (*js_rc(s.cast())).ref_count != 0 {
        list_add_tail(&mut (*s).header.link, &mut (*rt).gc_zero_ref_count_list);
    } else {
        js_free_rt(rt, s.cast());
    }
}
unsafe fn async_func_free(rt: *mut JSRuntime, s: *mut JSAsyncFunctionState) {
    (*js_rc(s.cast())).ref_count -= 1;
    if (*js_rc(s.cast())).ref_count == 0 && (*rt).gc_phase != JS_GC_PHASE_REMOVE_CYCLES as u8 {
        list_del(&mut (*s).header.link);
        list_add(&mut (*s).header.link, &mut (*rt).gc_zero_ref_count_list);
        if (*rt).gc_phase == JS_GC_PHASE_NONE as u8 {
            free_zero_refcount(rt);
        }
    }
}
unsafe fn close_var_ref(rt: *mut JSRuntime, sf: *mut JSStackFrame, var_ref: *mut JSVarRef) {
    if (*sf).js_mode & JS_MODE_ASYNC != 0 {
        async_func_free(
            rt,
            sf.cast::<u8>()
                .sub(offset_of!(JSAsyncFunctionState, frame))
                .cast(),
        );
    }
    (*var_ref).u.value = JS_DupValueRT(rt, *(*var_ref).pvalue);
    (*var_ref).pvalue = ptr::addr_of_mut!((*var_ref).u.value);
    (*var_ref).is_detached = 1;
}
unsafe fn close_var_refs(rt: *mut JSRuntime, b: *mut JSFunctionBytecode, sf: *mut JSStackFrame) {
    let mut i = 0;
    while i < (*b).var_ref_count {
        let var_ref = *(*sf).var_refs.add(i as usize);
        if !var_ref.is_null() {
            close_var_ref(rt, sf, var_ref);
        }
        i += 1;
    }
}
unsafe fn close_lexical_var(
    ctx: *mut JSContext,
    b: *mut JSFunctionBytecode,
    sf: *mut JSStackFrame,
    var_idx: i32,
) {
    let var_ref_idx =
        (*(*b).vardefs.add((*b).arg_count as usize + var_idx as usize)).var_ref_idx as usize;
    let var_ref = *(*sf).var_refs.add(var_ref_idx);
    if !var_ref.is_null() {
        close_var_ref((*ctx).rt, sf, var_ref);
        *(*sf).var_refs.add(var_ref_idx) = ptr::null_mut();
    }
}
unsafe fn js_free_module_def(rt: *mut JSRuntime, m: *mut JSModuleDef) {
    JS_FreeAtomRT(rt, (*m).module_name);
    let mut i = 0;
    while i < (*m).req_module_entries_count {
        let rme = (*m).req_module_entries.add(i as usize);
        JS_FreeAtomRT(rt, (*rme).module_name);
        JS_FreeValueRT(rt, (*rme).attributes);
        i += 1;
    }
    js_free_rt(rt, (*m).req_module_entries.cast());
    let mut i = 0;
    while i < (*m).export_entries_count {
        let me = (*m).export_entries.add(i as usize);
        if (*me).export_type == JS_EXPORT_TYPE_LOCAL {
            free_var_ref(rt, (*me).u.local.var_ref);
        }
        JS_FreeAtomRT(rt, (*me).export_name);
        JS_FreeAtomRT(rt, (*me).local_name);
        i += 1;
    }
    js_free_rt(rt, (*m).export_entries.cast());
    js_free_rt(rt, (*m).star_export_entries.cast());
    let mut i = 0;
    while i < (*m).import_entries_count {
        JS_FreeAtomRT(rt, (*(*m).import_entries.add(i as usize)).import_name);
        i += 1;
    }
    js_free_rt(rt, (*m).import_entries.cast());
    js_free_rt(rt, (*m).async_parent_modules.cast());
    JS_FreeValueRT(rt, (*m).module_ns);
    JS_FreeValueRT(rt, (*m).func_obj);
    JS_FreeValueRT(rt, (*m).eval_exception);
    JS_FreeValueRT(rt, (*m).meta_obj);
    JS_FreeValueRT(rt, (*m).promise);
    JS_FreeValueRT(rt, (*m).resolving_funcs[0]);
    JS_FreeValueRT(rt, (*m).resolving_funcs[1]);
    JS_FreeValueRT(rt, (*m).private_value);
    if !(*m).link.next.is_null() {
        list_del(&mut (*m).link);
    }
    remove_gc_object(&mut (*m).header);
    if (*rt).gc_phase == JS_GC_PHASE_REMOVE_CYCLES as u8 && (*js_rc(m.cast())).ref_count != 0 {
        list_add_tail(&mut (*m).header.link, &mut (*rt).gc_zero_ref_count_list);
    } else {
        js_free_rt(rt, m.cast());
    }
}
unsafe fn free_bytecode_atoms(
    rt: *mut JSRuntime,
    bc_buf: *const u8,
    bc_len: i32,
    use_short_opcodes: JS_BOOL,
) {
    use super::quickjs_opcode as op;
    let mut pos = 0;
    while pos < bc_len {
        let opcode = *bc_buf.add(pos as usize) as usize;
        let oi = if use_short_opcodes != 0 {
            op::short_opcode_info(opcode)
        } else {
            &op::opcode_info[opcode]
        };
        let len = oi.size as i32;
        match oi.fmt as i32 {
            op::OP_FMT_atom
            | op::OP_FMT_atom_u8
            | op::OP_FMT_atom_u16
            | op::OP_FMT_atom_label_u8
            | op::OP_FMT_atom_label_u16 => {
                if pos + 1 + 4 <= bc_len {
                    let atom = super::cutils_header::get_u32(bc_buf.add(pos as usize + 1));
                    JS_FreeAtomRT(rt, atom);
                }
            }
            _ => {}
        }
        pos += len;
    }
}
unsafe fn free_function_bytecode(rt: *mut JSRuntime, b: *mut JSFunctionBytecode) {
    js_host_free_bytecode_diagnostics(rt, b);
    if !(*b).byte_code_buf.is_null() {
        free_bytecode_atoms(rt, (*b).byte_code_buf, (*b).byte_code_len, 1);
    }
    if !(*b).vardefs.is_null() {
        let mut i = 0;
        while i < (*b).arg_count as usize + (*b).var_count as usize {
            JS_FreeAtomRT(rt, (*(*b).vardefs.add(i)).var_name);
            i += 1;
        }
    }
    let mut i = 0;
    while i < (*b).cpool_count {
        JS_FreeValueRT(rt, *(*b).cpool.add(i as usize));
        i += 1;
    }
    let mut i = 0;
    while i < (*b).closure_var_count {
        JS_FreeAtomRT(rt, (*(*b).closure_var.add(i as usize)).var_name);
        i += 1;
    }
    if !(*b).realm.is_null() {
        JS_FreeContext((*b).realm);
    }
    JS_FreeAtomRT(rt, (*b).func_name);
    if (*b).has_debug() != 0 {
        JS_FreeAtomRT(rt, (*b).debug.filename);
        js_free_rt(rt, (*b).debug.pc2line_buf.cast());
        js_free_rt(rt, (*b).debug.source.cast());
    }
    remove_gc_object(&mut (*b).header);
    if (*rt).gc_phase == JS_GC_PHASE_REMOVE_CYCLES as u8 && (*js_rc(b.cast())).ref_count != 0 {
        list_add_tail(&mut (*b).header.link, &mut (*rt).gc_zero_ref_count_list);
    } else {
        js_free_rt(rt, b.cast());
    }
}
unsafe fn free_object(rt: *mut JSRuntime, p: *mut JSObject) {
    js_host_free_object_diagnostics(rt, p);
    (*p).set_free_mark(1);
    let sh = (*p).shape;
    let pr = get_shape_prop(sh);
    let mut i = 0;
    while i < (*sh).prop_count {
        free_property(
            rt,
            (*p).prop.add(i as usize),
            (*pr.add(i as usize)).flags() as i32,
        );
        i += 1;
    }
    js_free_rt(rt, (*p).prop.cast());
    js_free_shape(rt, sh);
    (*p).shape = ptr::null_mut();
    (*p).prop = ptr::null_mut();
    if let Some(finalizer) = (*(*rt).class_array.add((*p).class_id as usize)).finalizer {
        finalizer(rt, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
    }
    (*p).class_id = 0;
    (*p).u.opaque = ptr::null_mut();
    (*p).u.func.var_refs = ptr::null_mut();
    (*p).u.func.home_object = ptr::null_mut();
    remove_gc_object(&mut (*p).header);
    if (*rt).gc_phase == JS_GC_PHASE_REMOVE_CYCLES as u8 {
        if (*js_rc(p.cast())).ref_count == 0 && (*p).weakref_count == 0 {
            js_free_rt(rt, p.cast());
        } else {
            list_add_tail(&mut (*p).header.link, &mut (*rt).gc_zero_ref_count_list);
        }
    } else if (*p).weakref_count == 0 {
        js_free_rt(rt, p.cast());
    } else {
        (*js_rc(p.cast())).gc_obj_type_and_mark &= 0x7f;
    }
}
unsafe fn free_gc_object(rt: *mut JSRuntime, gp: *mut JSGCObjectHeader) {
    match ((*js_rc(gp.cast())).gc_obj_type_and_mark & 0x7f) as u32 {
        JS_GC_OBJ_TYPE_JS_OBJECT => free_object(rt, gp.cast()),
        JS_GC_OBJ_TYPE_FUNCTION_BYTECODE => free_function_bytecode(rt, gp.cast()),
        JS_GC_OBJ_TYPE_ASYNC_FUNCTION => __async_func_free(rt, gp.cast()),
        JS_GC_OBJ_TYPE_MODULE => js_free_module_def(rt, gp.cast()),
        _ => std::process::abort(),
    }
}
unsafe fn free_zero_refcount(rt: *mut JSRuntime) {
    (*rt).gc_phase = JS_GC_PHASE_DECREF as u8;
    loop {
        let el = (*rt).gc_zero_ref_count_list.next;
        if el == ptr::addr_of_mut!((*rt).gc_zero_ref_count_list) {
            break;
        }
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        assert_eq!((*js_rc(p.cast())).ref_count, 0);
        free_gc_object(rt, p);
    }
    (*rt).gc_phase = JS_GC_PHASE_NONE as u8;
}
pub unsafe fn __JS_FreeValueRT(rt: *mut JSRuntime, v: JSValue) {
    match JS_VALUE_GET_TAG(v) {
        JS_TAG_STRING => {
            let p = JS_VALUE_GET_PTR(v).cast::<JSString>();
            if (*p).atom_type() != 0 {
                JS_FreeAtomStruct(rt, p);
            } else {
                js_free_rt(rt, p.cast());
            }
        }
        JS_TAG_STRING_ROPE => {
            let p = JS_VALUE_GET_PTR(v).cast::<JSStringRope>();
            JS_FreeValueRT(rt, (*p).left);
            JS_FreeValueRT(rt, (*p).right);
            js_free_rt(rt, p.cast());
        }
        JS_TAG_OBJECT | JS_TAG_FUNCTION_BYTECODE | JS_TAG_MODULE => {
            let p = JS_VALUE_GET_PTR(v).cast::<JSGCObjectHeader>();
            if (*rt).gc_phase != JS_GC_PHASE_REMOVE_CYCLES as u8 {
                list_del(&mut (*p).link);
                list_add(&mut (*p).link, &mut (*rt).gc_zero_ref_count_list);
                (*js_rc(p.cast())).gc_obj_type_and_mark |= 0x80;
                if (*rt).gc_phase == JS_GC_PHASE_NONE as u8 {
                    free_zero_refcount(rt);
                }
            }
        }
        JS_TAG_BIG_INT => js_free_rt(rt, JS_VALUE_GET_PTR(v)),
        JS_TAG_SYMBOL => JS_FreeAtomStruct(rt, JS_VALUE_GET_PTR(v).cast()),
        _ => std::process::abort(),
    }
}
pub unsafe fn __JS_FreeValue(ctx: *mut JSContext, v: JSValue) {
    __JS_FreeValueRT((*ctx).rt, v);
}
unsafe fn set_value(ctx: *mut JSContext, pval: *mut JSValue, new_val: JSValue) {
    let old_val = *pval;
    *pval = new_val;
    JS_FreeValue(ctx, old_val);
}
pub unsafe fn JS_SetClassProto(ctx: *mut JSContext, class_id: JSClassID, obj: JSValue) {
    assert!(class_id < (*(*ctx).rt).class_count as u32);
    set_value(ctx, (*ctx).class_proto.add(class_id as usize), obj);
}
pub unsafe fn JS_GetClassProto(ctx: *mut JSContext, class_id: JSClassID) -> JSValue {
    assert!(class_id < (*(*ctx).rt).class_count as u32);
    JS_DupValue(ctx, *(*ctx).class_proto.add(class_id as usize))
}
pub unsafe fn JS_Throw(ctx: *mut JSContext, obj: JSValue) -> JSValue {
    js_host_exception_begin((*ctx).rt);
    let rt = (*ctx).rt;
    JS_FreeValue(ctx, (*rt).current_exception);
    (*rt).current_exception = obj;
    (*rt).current_exception_is_uncatchable = 0;
    JS_EXCEPTION
}
pub unsafe fn JS_GetException(ctx: *mut JSContext) -> JSValue {
    js_host_exception_taken((*ctx).rt);
    let val = (*(*ctx).rt).current_exception;
    (*(*ctx).rt).current_exception = JS_UNINITIALIZED;
    val
}
pub unsafe fn JS_HasException(ctx: *mut JSContext) -> JS_BOOL {
    (JS_VALUE_GET_TAG((*(*ctx).rt).current_exception) != JS_TAG_UNINITIALIZED) as i32
}
pub unsafe fn JS_IsLiveObject(_rt: *mut JSRuntime, obj: JSValueConst) -> JS_BOOL {
    if JS_IsObject(obj) == 0 {
        0
    } else {
        ((*JS_VALUE_GET_PTR(obj).cast::<JSObject>()).free_mark() == 0) as i32
    }
}
pub unsafe fn JS_FreeRuntime(rt: *mut JSRuntime) {
    js_host_clear_exception_diagnostics(rt);
    js_clear_module_embedding_hooks(rt);
    JS_FreeValueRT(rt, (*rt).current_exception);
    for el in ListIter::new(&mut (*rt).job_list, false, true) {
        let e = el
            .cast::<u8>()
            .sub(offset_of!(JSJobEntry, link))
            .cast::<JSJobEntry>();
        let mut i = 0;
        while i < (*e).argc {
            JS_FreeValueRT(
                rt,
                *ptr::addr_of!((*e).argv).cast::<JSValue>().add(i as usize),
            );
            i += 1;
        }
        JS_FreeContext((*e).realm);
        js_free_rt(rt, e.cast());
    }
    init_list_head(&mut (*rt).job_list);
    // quickjs.c: don't remove weak objects during shutdown, which would enqueue
    // new FinalizationRegistry jobs. Use the original shared GC entry point.
    JS_RunGCInternal(rt, 0);
    assert!(list_empty(&mut (*rt).gc_obj_list) != 0);
    assert!(list_empty(&mut (*rt).weakref_list) != 0);
    let mut i = 0;
    while i < (*rt).class_count {
        let cl = (*rt).class_array.add(i as usize);
        if (*cl).class_id != 0 {
            JS_FreeAtomRT(rt, (*cl).class_name);
        }
        i += 1;
    }
    js_free_rt(rt, (*rt).class_array.cast());
    let mut i = 0;
    while i < (*rt).atom_size {
        let p = *(*rt).atom_array.add(i as usize);
        if atom_is_free(p) == 0 {
            js_free_rt(rt, p.cast());
        }
        i += 1;
    }
    js_free_rt(rt, (*rt).atom_array.cast());
    js_free_rt(rt, (*rt).atom_hash.cast());
    js_free_rt(rt, (*rt).shape_hash.cast());
    let mut ms = ptr::read(ptr::addr_of!((*rt).malloc_ctx.malloc_state));
    (*rt).malloc_ctx.mf.js_free.unwrap()(&mut ms, rt.cast());
}
