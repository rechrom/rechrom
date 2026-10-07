// quickjs.c closure reference capture and global declaration bindings. MIT.
// c: quickjs.c:17023
unsafe fn get_var_ref(
    ctx: *mut JSContext,
    sf: *mut JSStackFrame,
    var_idx: i32,
    is_arg: JS_BOOL,
) -> *mut JSVarRef {
    let p = JS_VALUE_GET_OBJ((*sf).cur_func);
    let b = (*p).u.func.function_bytecode;
    let (vd, pvalue) = if is_arg != 0 {
        (
            (*b).vardefs.add(var_idx as usize),
            (*sf).arg_buf.add(var_idx as usize),
        )
    } else {
        (
            (*b).vardefs.add((*b).arg_count as usize + var_idx as usize),
            (*sf).var_buf.add(var_idx as usize),
        )
    };
    assert!((*vd).is_captured() != 0);
    let var_ref_idx = (*vd).var_ref_idx;
    assert!(var_ref_idx < (*b).var_ref_count);
    let mut var_ref = *(*sf).var_refs.add(var_ref_idx as usize);
    if !var_ref.is_null() {
        assert_eq!((*var_ref).pvalue, pvalue);
        (*js_rc(var_ref.cast())).ref_count += 1;
        return var_ref;
    }
    var_ref = js_malloc(ctx, size_of::<JSVarRef>()).cast();
    if var_ref.is_null() {
        return ptr::null_mut();
    }
    (*js_rc(var_ref.cast())).ref_count = 1;
    add_gc_object((*ctx).rt, &mut (*var_ref).header, JS_GC_OBJ_TYPE_VAR_REF);
    (*var_ref).is_detached = 0;
    (*var_ref).is_lexical = 0;
    (*var_ref).is_const = 0;
    (*var_ref).u.attached = JSVarRefAttached {
        var_ref_idx,
        stack_frame: sf,
    };
    *(*sf).var_refs.add(var_ref_idx as usize) = var_ref;
    if (*sf).js_mode & JS_MODE_ASYNC != 0 {
        let async_func = sf
            .cast::<u8>()
            .sub(offset_of!(JSAsyncFunctionState, frame))
            .cast::<JSAsyncFunctionState>();
        (*js_rc(async_func.cast())).ref_count += 1;
    }
    (*var_ref).pvalue = pvalue;
    var_ref
}
// c: quickjs.c:17095
unsafe fn js_global_object_get_uninitialized_var(
    ctx: *mut JSContext,
    p1: *mut JSObject,
    atom: JSAtom,
) -> *mut JSVarRef {
    let p = JS_VALUE_GET_OBJ((*p1).u.global_object.uninitialized_vars);
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, atom);
    if !prs.is_null() {
        assert_eq!((*prs).flags() as i32 & JS_PROP_TMASK, JS_PROP_VARREF);
        let var_ref = (*pr).u.var_ref;
        (*js_rc(var_ref.cast())).ref_count += 1;
        return var_ref;
    }
    let var_ref = js_create_var_ref(ctx, 1);
    if var_ref.is_null() {
        return ptr::null_mut();
    }
    pr = add_property(ctx, p, atom, JS_PROP_C_W_E | JS_PROP_VARREF);
    if pr.is_null() {
        free_var_ref((*ctx).rt, var_ref);
        return ptr::null_mut();
    }
    (*pr).u.var_ref = var_ref;
    (*js_rc(var_ref.cast())).ref_count += 1;
    var_ref
}
// c: quickjs.c:17151
unsafe fn js_closure_define_global_var(
    ctx: *mut JSContext,
    cv: *mut JSClosureVar,
    is_direct_or_indirect_eval: JS_BOOL,
) -> *mut JSVarRef {
    let p;
    let mut flags;
    let mut pr = ptr::null_mut();
    let mut reused = ptr::null_mut();
    if (*cv).is_lexical() != 0 {
        p = JS_VALUE_GET_OBJ((*ctx).global_var_obj);
        flags = JS_PROP_ENUMERABLE | JS_PROP_CONFIGURABLE;
        if (*cv).is_const() == 0 {
            flags |= JS_PROP_WRITABLE;
        }
        let prs = find_own_property(&mut pr, p, (*cv).var_name);
        if !prs.is_null() {
            assert_eq!((*prs).flags() as i32 & JS_PROP_TMASK, JS_PROP_VARREF);
            let var_ref = (*pr).u.var_ref;
            (*js_rc(var_ref.cast())).ref_count += 1;
            return var_ref;
        }
        let p1 = JS_VALUE_GET_OBJ((*ctx).global_obj);
        let prs = find_own_property(&mut pr, p1, (*cv).var_name);
        if !prs.is_null() && (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_VARREF {
            let var_ref1 = js_create_var_ref(ctx, 0);
            if var_ref1.is_null() {
                return ptr::null_mut();
            }
            let var_ref = (*pr).u.var_ref;
            (*var_ref1).u.value = (*var_ref).u.value;
            (*var_ref).u.value = JS_UNINITIALIZED;
            (*pr).u.var_ref = var_ref1;
            reused = var_ref;
        }
    } else {
        p = JS_VALUE_GET_OBJ((*ctx).global_obj);
        flags = JS_PROP_ENUMERABLE | JS_PROP_WRITABLE;
        if is_direct_or_indirect_eval != 0 {
            flags |= JS_PROP_CONFIGURABLE;
        }
        loop {
            let prs = find_own_property(&mut pr, p, (*cv).var_name);
            if !prs.is_null() {
                if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_AUTOINIT {
                    if JS_AutoInitProperty(ctx, p, (*cv).var_name, pr, prs) != 0 {
                        return ptr::null_mut();
                    }
                    continue;
                }
                let var_ref = if (*prs).flags() as i32 & JS_PROP_TMASK != JS_PROP_VARREF {
                    let var_ref = js_global_object_get_uninitialized_var(ctx, p, (*cv).var_name);
                    if var_ref.is_null() {
                        return ptr::null_mut();
                    }
                    var_ref
                } else {
                    let var_ref = (*pr).u.var_ref;
                    (*js_rc(var_ref.cast())).ref_count += 1;
                    var_ref
                };
                if (*cv).var_kind() as u32 == JS_VAR_GLOBAL_FUNCTION_DECL
                    && (*prs).flags() as i32 & JS_PROP_CONFIGURABLE != 0
                {
                    if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_GETSET {
                        free_property((*ctx).rt, pr, (*prs).flags() as i32);
                        (*prs).set_flags((flags | JS_PROP_VARREF) as u32);
                        (*pr).u.var_ref = var_ref;
                        (*js_rc(var_ref.cast())).ref_count += 1;
                    } else {
                        assert_eq!((*prs).flags() as i32 & JS_PROP_TMASK, JS_PROP_VARREF);
                        (*prs).set_flags(((*prs).flags() as i32 & !JS_PROP_C_W_E | flags) as u32);
                    }
                    (*var_ref).is_const = 0;
                }
                return var_ref;
            }
            if (*p).extensible() == 0 {
                return js_global_object_get_uninitialized_var(ctx, p, (*cv).var_name);
            }
            break;
        }
    }
    let var_ref = if reused.is_null() {
        let p1 = JS_VALUE_GET_OBJ((*ctx).global_obj);
        js_global_object_find_uninitialized_var(ctx, p1, (*cv).var_name, (*cv).is_lexical() as i32)
    } else {
        reused
    };
    if var_ref.is_null() {
        return ptr::null_mut();
    }
    if (*cv).is_lexical() != 0 {
        (*var_ref).is_lexical = 1;
        (*var_ref).is_const = (*cv).is_const();
    }
    pr = add_property(ctx, p, (*cv).var_name, flags | JS_PROP_VARREF);
    if pr.is_null() {
        free_var_ref((*ctx).rt, var_ref);
        return ptr::null_mut();
    }
    (*pr).u.var_ref = var_ref;
    (*js_rc(var_ref.cast())).ref_count += 1;
    var_ref
}
// c: quickjs.c:17254
unsafe fn js_closure_global_var(ctx: *mut JSContext, cv: *mut JSClosureVar) -> *mut JSVarRef {
    let mut p = JS_VALUE_GET_OBJ((*ctx).global_var_obj);
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, (*cv).var_name);
    if !prs.is_null() {
        assert_eq!((*prs).flags() as i32 & JS_PROP_TMASK, JS_PROP_VARREF);
        let var_ref = (*pr).u.var_ref;
        (*js_rc(var_ref.cast())).ref_count += 1;
        return var_ref;
    }
    p = JS_VALUE_GET_OBJ((*ctx).global_obj);
    loop {
        let prs = find_own_property(&mut pr, p, (*cv).var_name);
        if !prs.is_null() {
            if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_AUTOINIT {
                if JS_AutoInitProperty(ctx, p, (*cv).var_name, pr, prs) != 0 {
                    return ptr::null_mut();
                }
                continue;
            }
            if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_VARREF {
                let var_ref = (*pr).u.var_ref;
                (*js_rc(var_ref.cast())).ref_count += 1;
                return var_ref;
            }
        }
        return js_global_object_get_uninitialized_var(ctx, p, (*cv).var_name);
    }
}
// c: quickjs.c:17288
unsafe fn js_closure2(
    ctx: *mut JSContext,
    func_obj: JSValue,
    b: *mut JSFunctionBytecode,
    cur_var_refs: *mut *mut JSVarRef,
    sf: *mut JSStackFrame,
    is_eval: JS_BOOL,
    _m: *mut JSModuleDef,
) -> JSValue {
    let p = JS_VALUE_GET_OBJ(func_obj);
    (*p).u.func.function_bytecode = b;
    (*p).u.func.home_object = ptr::null_mut();
    (*p).u.func.var_refs = ptr::null_mut();
    let success = (|| {
        if (*b).closure_var_count == 0 {
            return true;
        }
        let var_refs = js_mallocz(
            ctx,
            size_of::<*mut JSVarRef>() * (*b).closure_var_count as usize,
        )
        .cast::<*mut JSVarRef>();
        if var_refs.is_null() {
            return false;
        }
        (*p).u.func.var_refs = var_refs;
        if is_eval != 0 {
            for i in 0..(*b).closure_var_count {
                let cv = (*b).closure_var.add(i as usize);
                if (*cv).closure_type() == JS_CLOSURE_GLOBAL_DECL {
                    let mut flags = 0;
                    if (*cv).is_lexical() != 0 {
                        flags |= DEFINE_GLOBAL_LEX_VAR;
                    }
                    if (*cv).var_kind() as u32 == JS_VAR_GLOBAL_FUNCTION_DECL {
                        flags |= DEFINE_GLOBAL_FUNC_VAR;
                    }
                    if JS_CheckDefineGlobalVar(ctx, (*cv).var_name, flags) != 0 {
                        return false;
                    }
                }
            }
        }
        for i in 0..(*b).closure_var_count {
            let cv = (*b).closure_var.add(i as usize);
            let var_ref = match (*cv).closure_type() {
                JS_CLOSURE_MODULE_IMPORT => continue,
                JS_CLOSURE_MODULE_DECL => js_create_var_ref(ctx, (*cv).is_lexical() as i32),
                JS_CLOSURE_GLOBAL_DECL => {
                    js_closure_define_global_var(ctx, cv, (*b).is_direct_or_indirect_eval() as i32)
                }
                JS_CLOSURE_GLOBAL => js_closure_global_var(ctx, cv),
                JS_CLOSURE_LOCAL => get_var_ref(ctx, sf, (*cv).var_idx as i32, 0),
                JS_CLOSURE_ARG => get_var_ref(ctx, sf, (*cv).var_idx as i32, 1),
                JS_CLOSURE_REF | JS_CLOSURE_GLOBAL_REF => {
                    let var_ref = *cur_var_refs.add((*cv).var_idx as usize);
                    (*js_rc(var_ref.cast())).ref_count += 1;
                    var_ref
                }
                _ => std::process::abort(),
            };
            if var_ref.is_null() {
                return false;
            }
            *var_refs.add(i as usize) = var_ref;
        }
        true
    })();
    if success {
        func_obj
    } else {
        JS_FreeValue(ctx, func_obj);
        JS_EXCEPTION
    }
}
// c: quickjs.c:17367
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
        JS_ATOM_constructor,
        JS_DupValue(ctx, this_val),
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    ) < 0
    {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    obj
}
const func_kind_to_class_id: [u16; 4] = [
    JS_CLASS_BYTECODE_FUNCTION as u16,
    JS_CLASS_GENERATOR_FUNCTION as u16,
    JS_CLASS_ASYNC_FUNCTION as u16,
    JS_CLASS_ASYNC_GENERATOR_FUNCTION as u16,
];
// c: quickjs.c:17395
unsafe fn js_closure(
    ctx: *mut JSContext,
    bfunc: JSValue,
    cur_var_refs: *mut *mut JSVarRef,
    sf: *mut JSStackFrame,
    is_eval: JS_BOOL,
) -> JSValue {
    let b = JS_VALUE_GET_PTR(bfunc).cast::<JSFunctionBytecode>();
    let mut func_obj =
        JS_NewObjectClass(ctx, func_kind_to_class_id[(*b).func_kind() as usize] as i32);
    if JS_IsException(func_obj) != 0 {
        JS_FreeValue(ctx, bfunc);
        return JS_EXCEPTION;
    }
    func_obj = js_closure2(ctx, func_obj, b, cur_var_refs, sf, is_eval, ptr::null_mut());
    if JS_IsException(func_obj) != 0 {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    let name_atom = if (*b).func_name == JS_ATOM_NULL as u32 {
        JS_ATOM_empty_string
    } else {
        (*b).func_name
    };
    js_function_set_properties(ctx, func_obj, name_atom, (*b).defined_arg_count as i32);
    if (*b).func_kind() as u32 & JS_FUNC_GENERATOR != 0 {
        let proto_class_id = if (*b).func_kind() as u32 == JS_FUNC_ASYNC_GENERATOR {
            JS_CLASS_ASYNC_GENERATOR
        } else {
            JS_CLASS_GENERATOR
        };
        let proto = JS_NewObjectProto(ctx, *(*ctx).class_proto.add(proto_class_id as usize));
        if JS_IsException(proto) != 0 {
            JS_FreeValue(ctx, func_obj);
            return JS_EXCEPTION;
        }
        JS_DefinePropertyValue(ctx, func_obj, JS_ATOM_prototype, proto, JS_PROP_WRITABLE);
    } else if (*b).has_prototype() != 0 {
        JS_SetConstructorBit(ctx, func_obj, 1);
        JS_DefineAutoInitProperty(
            ctx,
            func_obj,
            JS_ATOM_prototype,
            JS_AUTOINIT_ID_PROTOTYPE,
            ptr::null_mut(),
            JS_PROP_WRITABLE,
        );
    }
    func_obj
}
