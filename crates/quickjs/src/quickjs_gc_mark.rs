// quickjs.c GC traversal, refcount removal/scanning and cycle destruction. MIT.
#[repr(C)]
struct JSCFunctionDataRecord {
    func: Option<JSCFunctionData>,
    length: u8,
    data_len: u8,
    magic: u16,
    data: [JSValue; 0],
}
unsafe fn JS_MarkContext(rt: *mut JSRuntime, ctx: *mut JSContext, mark_func: JS_MarkFunc) {
    for el in ListIter::new(&mut (*ctx).loaded_modules, false, false) {
        let m = el
            .cast::<u8>()
            .sub(offset_of!(JSModuleDef, link))
            .cast::<JSModuleDef>();
        JS_MarkValue(rt, JS_MKPTR(JS_TAG_MODULE, m.cast()), mark_func);
    }
    JS_MarkValue(rt, (*ctx).global_obj, mark_func);
    JS_MarkValue(rt, (*ctx).global_var_obj, mark_func);
    JS_MarkValue(rt, (*ctx).throw_type_error, mark_func);
    JS_MarkValue(rt, (*ctx).eval_obj, mark_func);
    JS_MarkValue(rt, (*ctx).array_proto_values, mark_func);
    for i in 0..JS_NATIVE_ERROR_COUNT {
        JS_MarkValue(rt, (*ctx).native_error_proto[i], mark_func);
    }
    let mut i = 0;
    while i < (*rt).class_count {
        JS_MarkValue(rt, *(*ctx).class_proto.add(i as usize), mark_func);
        i += 1;
    }
    JS_MarkValue(rt, (*ctx).iterator_ctor, mark_func);
    JS_MarkValue(rt, (*ctx).async_iterator_proto, mark_func);
    JS_MarkValue(rt, (*ctx).promise_ctor, mark_func);
    JS_MarkValue(rt, (*ctx).array_ctor, mark_func);
    JS_MarkValue(rt, (*ctx).regexp_ctor, mark_func);
    JS_MarkValue(rt, (*ctx).function_ctor, mark_func);
    JS_MarkValue(rt, (*ctx).function_proto, mark_func);
    if !(*ctx).array_shape.is_null() {
        mark_func(rt, &mut (*(*ctx).array_shape).header);
    }
    if !(*ctx).arguments_shape.is_null() {
        mark_func(rt, &mut (*(*ctx).arguments_shape).header);
    }
    if !(*ctx).mapped_arguments_shape.is_null() {
        mark_func(rt, &mut (*(*ctx).mapped_arguments_shape).header);
    }
    if !(*ctx).regexp_shape.is_null() {
        mark_func(rt, &mut (*(*ctx).regexp_shape).header);
    }
    if !(*ctx).regexp_result_shape.is_null() {
        mark_func(rt, &mut (*(*ctx).regexp_result_shape).header);
    }
}
unsafe fn js_mark_module_def(rt: *mut JSRuntime, m: *mut JSModuleDef, mark_func: JS_MarkFunc) {
    let mut i = 0;
    while i < (*m).req_module_entries_count {
        JS_MarkValue(
            rt,
            (*(*m).req_module_entries.add(i as usize)).attributes,
            mark_func,
        );
        i += 1;
    }
    let mut i = 0;
    while i < (*m).export_entries_count {
        let me = (*m).export_entries.add(i as usize);
        if (*me).export_type == JS_EXPORT_TYPE_LOCAL && !(*me).u.local.var_ref.is_null() {
            mark_func(rt, &mut (*(*me).u.local.var_ref).header);
        }
        i += 1;
    }
    JS_MarkValue(rt, (*m).module_ns, mark_func);
    JS_MarkValue(rt, (*m).func_obj, mark_func);
    JS_MarkValue(rt, (*m).eval_exception, mark_func);
    JS_MarkValue(rt, (*m).meta_obj, mark_func);
    JS_MarkValue(rt, (*m).promise, mark_func);
    JS_MarkValue(rt, (*m).resolving_funcs[0], mark_func);
    JS_MarkValue(rt, (*m).resolving_funcs[1], mark_func);
    JS_MarkValue(rt, (*m).private_value, mark_func);
}
unsafe fn mark_children(rt: *mut JSRuntime, gp: *mut JSGCObjectHeader, mark_func: JS_MarkFunc) {
    match ((*js_rc(gp.cast())).gc_obj_type_and_mark & 0x7f) as u32 {
        JS_GC_OBJ_TYPE_JS_OBJECT => {
            let p = gp.cast::<JSObject>();
            let sh = (*p).shape;
            mark_func(rt, &mut (*sh).header);
            let mut prs = get_shape_prop(sh);
            let mut i = 0;
            while i < (*sh).prop_count {
                let pr = (*p).prop.add(i as usize);
                if (*prs).atom != JS_ATOM_NULL as u32 {
                    match (*prs).flags() as i32 & JS_PROP_TMASK {
                        JS_PROP_GETSET => {
                            if !(*pr).u.getset.getter.is_null() {
                                mark_func(rt, &mut (*(*pr).u.getset.getter).header);
                            }
                            if !(*pr).u.getset.setter.is_null() {
                                mark_func(rt, &mut (*(*pr).u.getset.setter).header);
                            }
                        }
                        JS_PROP_VARREF => mark_func(rt, &mut (*(*pr).u.var_ref).header),
                        JS_PROP_AUTOINIT => js_autoinit_mark(rt, pr, mark_func),
                        _ => JS_MarkValue(rt, (*pr).u.value, mark_func),
                    }
                }
                prs = prs.add(1);
                i += 1;
            }
            if (*p).class_id as u32 != JS_CLASS_OBJECT {
                if let Some(gc_mark) = (*(*rt).class_array.add((*p).class_id as usize)).gc_mark {
                    gc_mark(rt, JS_MKPTR(JS_TAG_OBJECT, p.cast()), Some(mark_func));
                }
            }
        }
        JS_GC_OBJ_TYPE_FUNCTION_BYTECODE => {
            let b = gp.cast::<JSFunctionBytecode>();
            let mut i = 0;
            while i < (*b).cpool_count {
                JS_MarkValue(rt, *(*b).cpool.add(i as usize), mark_func);
                i += 1;
            }
            if !(*b).realm.is_null() {
                mark_func(rt, &mut (*(*b).realm).header);
            }
        }
        JS_GC_OBJ_TYPE_VAR_REF => {
            let var_ref = gp.cast::<JSVarRef>();
            if (*var_ref).is_detached != 0 {
                JS_MarkValue(rt, *(*var_ref).pvalue, mark_func);
            } else {
                let sf = (*var_ref).u.attached.stack_frame;
                if (*sf).js_mode & JS_MODE_ASYNC != 0 {
                    let s = sf
                        .cast::<u8>()
                        .sub(offset_of!(JSAsyncFunctionState, frame))
                        .cast::<JSAsyncFunctionState>();
                    mark_func(rt, &mut (*s).header);
                }
            }
        }
        JS_GC_OBJ_TYPE_ASYNC_FUNCTION => {
            let s = gp.cast::<JSAsyncFunctionState>();
            let sf = ptr::addr_of_mut!((*s).frame);
            if (*s).is_completed == 0 {
                JS_MarkValue(rt, (*sf).cur_func, mark_func);
                JS_MarkValue(rt, (*s).this_val, mark_func);
                if !(*sf).cur_sp.is_null() {
                    let mut sp = (*sf).arg_buf;
                    while sp < (*sf).cur_sp {
                        JS_MarkValue(rt, *sp, mark_func);
                        sp = sp.add(1);
                    }
                }
            }
            JS_MarkValue(rt, (*s).resolving_funcs[0], mark_func);
            JS_MarkValue(rt, (*s).resolving_funcs[1], mark_func);
        }
        JS_GC_OBJ_TYPE_SHAPE => {
            let sh = gp.cast::<JSShape>();
            if !(*sh).proto.is_null() {
                mark_func(rt, &mut (*(*sh).proto).header);
            }
        }
        JS_GC_OBJ_TYPE_JS_CONTEXT => JS_MarkContext(rt, gp.cast(), mark_func),
        JS_GC_OBJ_TYPE_MODULE => js_mark_module_def(rt, gp.cast(), mark_func),
        _ => std::process::abort(),
    }
}
unsafe fn gc_decref_child(rt: *mut JSRuntime, p: *mut JSGCObjectHeader) {
    assert!((*js_rc(p.cast())).ref_count > 0);
    (*js_rc(p.cast())).ref_count -= 1;
    if (*js_rc(p.cast())).ref_count == 0 && (*js_rc(p.cast())).gc_obj_type_and_mark & 0x80 != 0 {
        list_del(&mut (*p).link);
        list_add_tail(&mut (*p).link, &mut (*rt).tmp_obj_list);
    }
}
unsafe fn gc_decref(rt: *mut JSRuntime) {
    init_list_head(&mut (*rt).tmp_obj_list);
    for el in ListIter::new(&mut (*rt).gc_obj_list, false, true) {
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        assert_eq!((*js_rc(p.cast())).gc_obj_type_and_mark & 0x80, 0);
        mark_children(rt, p, gc_decref_child);
        (*js_rc(p.cast())).gc_obj_type_and_mark |= 0x80;
        if (*js_rc(p.cast())).ref_count == 0 {
            list_del(&mut (*p).link);
            list_add_tail(&mut (*p).link, &mut (*rt).tmp_obj_list);
        }
    }
}
unsafe fn gc_scan_incref_child(rt: *mut JSRuntime, p: *mut JSGCObjectHeader) {
    (*js_rc(p.cast())).ref_count += 1;
    if (*js_rc(p.cast())).ref_count == 1 {
        list_del(&mut (*p).link);
        list_add_tail(&mut (*p).link, &mut (*rt).gc_obj_list);
        (*js_rc(p.cast())).gc_obj_type_and_mark &= 0x7f;
    }
}
unsafe fn gc_scan_incref_child2(_rt: *mut JSRuntime, p: *mut JSGCObjectHeader) {
    (*js_rc(p.cast())).ref_count += 1;
}
unsafe fn gc_scan(rt: *mut JSRuntime) {
    for el in ListIter::new(&mut (*rt).gc_obj_list, false, false) {
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        assert!((*js_rc(p.cast())).ref_count > 0);
        (*js_rc(p.cast())).gc_obj_type_and_mark &= 0x7f;
        mark_children(rt, p, gc_scan_incref_child);
    }
    for el in ListIter::new(&mut (*rt).tmp_obj_list, false, false) {
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        mark_children(rt, p, gc_scan_incref_child2);
    }
}
unsafe fn gc_free_cycles(rt: *mut JSRuntime) {
    (*rt).gc_phase = JS_GC_PHASE_REMOVE_CYCLES as u8;
    loop {
        let el = (*rt).tmp_obj_list.next;
        if el == ptr::addr_of_mut!((*rt).tmp_obj_list) {
            break;
        }
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        match ((*js_rc(p.cast())).gc_obj_type_and_mark & 0x7f) as u32 {
            JS_GC_OBJ_TYPE_JS_OBJECT
            | JS_GC_OBJ_TYPE_FUNCTION_BYTECODE
            | JS_GC_OBJ_TYPE_ASYNC_FUNCTION
            | JS_GC_OBJ_TYPE_MODULE => free_gc_object(rt, p),
            _ => {
                list_del(&mut (*p).link);
                list_add_tail(&mut (*p).link, &mut (*rt).gc_zero_ref_count_list);
            }
        }
    }
    (*rt).gc_phase = JS_GC_PHASE_NONE as u8;
    for el in ListIter::new(&mut (*rt).gc_zero_ref_count_list, false, true) {
        let p = el
            .cast::<u8>()
            .sub(offset_of!(JSGCObjectHeader, link))
            .cast::<JSGCObjectHeader>();
        let obj_type = ((*js_rc(p.cast())).gc_obj_type_and_mark & 0x7f) as u32;
        assert!(matches!(
            obj_type,
            JS_GC_OBJ_TYPE_JS_OBJECT
                | JS_GC_OBJ_TYPE_FUNCTION_BYTECODE
                | JS_GC_OBJ_TYPE_ASYNC_FUNCTION
                | JS_GC_OBJ_TYPE_MODULE
        ));
        if obj_type == JS_GC_OBJ_TYPE_JS_OBJECT && (*p.cast::<JSObject>()).weakref_count != 0 {
            (*js_rc(p.cast())).gc_obj_type_and_mark &= 0x7f;
        } else {
            js_free_rt(rt, p.cast());
        }
    }
    init_list_head(&mut (*rt).gc_zero_ref_count_list);
}
unsafe fn js_array_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let mut i = 0;
    while i < (*p).u.array.count {
        JS_FreeValueRT(rt, *(*p).u.array.u.values.add(i as usize));
        i += 1;
    }
    js_free_rt(rt, (*p).u.array.u.values.cast());
}
unsafe fn js_array_mark(rt: *mut JSRuntime, val: JSValueConst, mark_func: Option<JS_MarkFunc>) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let mut i = 0;
    while i < (*p).u.array.count {
        JS_MarkValue(
            rt,
            *(*p).u.array.u.values.add(i as usize),
            mark_func.unwrap(),
        );
        i += 1;
    }
}
unsafe fn js_object_data_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    JS_FreeValueRT(rt, (*p).u.object_data);
    (*p).u.object_data = JS_UNDEFINED;
}
unsafe fn js_object_data_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    JS_MarkValue(rt, (*p).u.object_data, mark_func.unwrap());
}
unsafe fn js_c_function_finalizer(_rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    if !(*p).u.cfunc.realm.is_null() {
        JS_FreeContext((*p).u.cfunc.realm);
    }
}
unsafe fn js_c_function_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    if !(*p).u.cfunc.realm.is_null() {
        mark_func.unwrap()(rt, &mut (*(*p).u.cfunc.realm).header);
    }
}
unsafe fn js_bytecode_function_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let p1 = (*p).u.func.home_object;
    if !p1.is_null() {
        JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_OBJECT, p1.cast()));
    }
    let b = (*p).u.func.function_bytecode;
    if !b.is_null() {
        let var_refs = (*p).u.func.var_refs;
        if !var_refs.is_null() {
            let mut i = 0;
            while i < (*b).closure_var_count {
                free_var_ref(rt, *var_refs.add(i as usize));
                i += 1;
            }
            js_free_rt(rt, var_refs.cast());
        }
        JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_FUNCTION_BYTECODE, b.cast()));
    }
}
unsafe fn js_bytecode_function_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let mark_func = mark_func.unwrap();
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let var_refs = (*p).u.func.var_refs;
    let b = (*p).u.func.function_bytecode;
    if !(*p).u.func.home_object.is_null() {
        JS_MarkValue(
            rt,
            JS_MKPTR(JS_TAG_OBJECT, (*p).u.func.home_object.cast()),
            mark_func,
        );
    }
    if !b.is_null() {
        if !var_refs.is_null() {
            let mut i = 0;
            while i < (*b).closure_var_count {
                let var_ref = *var_refs.add(i as usize);
                if !var_ref.is_null() {
                    mark_func(rt, &mut (*var_ref).header);
                }
                i += 1;
            }
        }
        JS_MarkValue(rt, JS_MKPTR(JS_TAG_FUNCTION_BYTECODE, b.cast()), mark_func);
    }
}
unsafe fn js_bound_function_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let bf = (*p).u.bound_function;
    JS_FreeValueRT(rt, (*bf).func_obj);
    JS_FreeValueRT(rt, (*bf).this_val);
    let mut i = 0;
    while i < (*bf).argc {
        JS_FreeValueRT(
            rt,
            *ptr::addr_of!((*bf).argv).cast::<JSValue>().add(i as usize),
        );
        i += 1;
    }
    js_free_rt(rt, bf.cast());
}
unsafe fn js_bound_function_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let mark_func = mark_func.unwrap();
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let bf = (*p).u.bound_function;
    JS_MarkValue(rt, (*bf).func_obj, mark_func);
    JS_MarkValue(rt, (*bf).this_val, mark_func);
    let mut i = 0;
    while i < (*bf).argc {
        JS_MarkValue(
            rt,
            *ptr::addr_of!((*bf).argv).cast::<JSValue>().add(i as usize),
            mark_func,
        );
        i += 1;
    }
}
unsafe fn js_for_in_iterator_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    let it = (*p).u.for_in_iterator;
    JS_FreeValueRT(rt, (*it).obj);
    if (*it).is_array == 0 {
        let mut i = 0;
        while i < (*it).atom_count {
            JS_FreeAtomRT(rt, (*(*it).tab_atom.add(i as usize)).atom);
            i += 1;
        }
        js_free_rt(rt, (*it).tab_atom.cast());
    }
    js_free_rt(rt, it.cast());
}
unsafe fn js_for_in_iterator_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    JS_MarkValue(rt, (*(*p).u.for_in_iterator).obj, mark_func.unwrap());
}
unsafe fn js_c_function_data_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s = JS_GetOpaque(val, JS_CLASS_C_FUNCTION_DATA).cast::<JSCFunctionDataRecord>();
    if !s.is_null() {
        let mut i = 0;
        while i < (*s).data_len {
            JS_FreeValueRT(
                rt,
                *ptr::addr_of!((*s).data).cast::<JSValue>().add(i as usize),
            );
            i += 1;
        }
        js_free_rt(rt, s.cast());
    }
}
unsafe fn js_c_function_data_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    let s = JS_GetOpaque(val, JS_CLASS_C_FUNCTION_DATA).cast::<JSCFunctionDataRecord>();
    if !s.is_null() {
        let mut i = 0;
        while i < (*s).data_len {
            JS_MarkValue(
                rt,
                *ptr::addr_of!((*s).data).cast::<JSValue>().add(i as usize),
                mark_func.unwrap(),
            );
            i += 1;
        }
    }
}
unsafe fn js_global_object_finalizer(rt: *mut JSRuntime, obj: JSValue) {
    JS_FreeValueRT(
        rt,
        (*JS_VALUE_GET_PTR(obj).cast::<JSObject>())
            .u
            .global_object
            .uninitialized_vars,
    );
}
unsafe fn js_global_object_mark(
    rt: *mut JSRuntime,
    val: JSValueConst,
    mark_func: Option<JS_MarkFunc>,
) {
    JS_MarkValue(
        rt,
        (*JS_VALUE_GET_PTR(val).cast::<JSObject>())
            .u
            .global_object
            .uninitialized_vars,
        mark_func.unwrap(),
    );
}
