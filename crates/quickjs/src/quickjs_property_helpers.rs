// quickjs.c:8167-8207,8906-8921,9244-9409,9433-9648,
// 17004-17022,17126-17153. MIT. Awaiting production conversion/call integration.
type JSAutoInitFunc = unsafe fn(*mut JSContext, *mut JSObject, JSAtom, *mut c_void) -> JSValue;
static js_autoinit_func_table: [JSAutoInitFunc; 3] = [
    js_instantiate_prototype,
    js_module_ns_autoinit,
    JS_InstantiateFunctionListItem2,
];
unsafe fn JS_AutoInitProperty(
    ctx: *mut JSContext,
    p: *mut JSObject,
    prop: JSAtom,
    pr: *mut JSProperty,
    mut prs: *mut JSShapeProperty,
) -> i32 {
    if js_shape_prepare_update(ctx, p, &mut prs) != 0 {
        return -1;
    }
    let realm = js_autoinit_get_realm(pr);
    let id = js_autoinit_get_id(pr);
    let func = js_autoinit_func_table[id as usize];
    let val = func(realm, p, prop, (*pr).u.init.opaque);
    js_autoinit_free((*ctx).rt, pr);
    (*prs).set_flags((*prs).flags() & !(JS_PROP_TMASK as u32));
    (*pr).u.value = JS_UNDEFINED;
    if JS_IsException(val) != 0 {
        return -1;
    }
    if id == JS_AUTOINIT_ID_MODULE_NS && JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
        (*prs).set_flags((*prs).flags() | JS_PROP_VARREF as u32);
        (*pr).u.var_ref = JS_VALUE_GET_PTR(val).cast();
        (*js_rc((*pr).u.var_ref.cast())).ref_count += 1;
    } else if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT {
        let var_ref = js_create_var_ref(ctx, 0);
        if var_ref.is_null() {
            return -1;
        }
        (*prs).set_flags((*prs).flags() | JS_PROP_VARREF as u32);
        (*pr).u.var_ref = var_ref;
        (*var_ref).u.value = val;
        (*var_ref).is_const = ((*prs).flags() as i32 & JS_PROP_WRITABLE == 0) as u8;
    } else {
        (*pr).u.value = val;
    }
    0
}
pub unsafe fn JS_IsExtensible(ctx: *mut JSContext, obj: JSValueConst) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return 0;
    }
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    if (*p).is_exotic() != 0 {
        let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
        if !em.is_null() {
            if let Some(is_extensible) = (*em).is_extensible {
                return is_extensible(ctx, obj);
            }
        }
    }
    (*p).extensible() as i32
}
#[inline(never)]
unsafe fn convert_fast_array_to_array(ctx: *mut JSContext, p: *mut JSObject) -> i32 {
    if js_shape_prepare_update(ctx, p, ptr::null_mut()) != 0 {
        return -1;
    }
    let len = (*p).u.array.count;
    let sh = (*p).shape;
    let new_count = ((*sh).prop_count as u32).wrapping_add(len);
    if new_count > (*sh).prop_size as u32
        && resize_properties(ctx, &mut (*p).shape, p, new_count) != 0
    {
        return -1;
    }
    if (*p).class_id as u32 == JS_CLASS_MAPPED_ARGUMENTS {
        let mut tab = (*p).u.array.u.var_refs;
        for i in 0..len {
            let pr = add_property(
                ctx,
                p,
                __JS_AtomFromUInt32(i),
                JS_PROP_C_W_E | JS_PROP_VARREF,
            );
            (*pr).u.var_ref = *tab;
            tab = tab.add(1);
        }
    } else {
        let mut tab = (*p).u.array.u.values;
        for i in 0..len {
            let pr = add_property(ctx, p, __JS_AtomFromUInt32(i), JS_PROP_C_W_E);
            (*pr).u.value = *tab;
            tab = tab.add(1);
        }
    }
    js_free(ctx, (*p).u.array.u.values.cast());
    (*p).u.array.count = 0;
    (*p).u.array.u.values = ptr::null_mut();
    (*p).u.array.u1.size = 0;
    (*p).set_fast_array(0);
    (*p).set_is_std_array_prototype(0);
    0
}
unsafe fn remove_global_object_property(
    ctx: *mut JSContext,
    p: *mut JSObject,
    prs: *mut JSShapeProperty,
    pr: *mut JSProperty,
) -> i32 {
    let var_ref = (*pr).u.var_ref;
    if (*js_rc(var_ref.cast())).ref_count == 1 {
        return 0;
    }
    let p1 = JS_VALUE_GET_PTR((*p).u.global_object.uninitialized_vars).cast::<JSObject>();
    let pr1 = add_property(ctx, p1, (*prs).atom, JS_PROP_C_W_E | JS_PROP_VARREF);
    if pr1.is_null() {
        return -1;
    }
    (*pr1).u.var_ref = var_ref;
    (*js_rc(var_ref.cast())).ref_count += 1;
    JS_FreeValue(ctx, (*var_ref).u.value);
    (*var_ref).is_lexical = 0;
    (*var_ref).is_const = 0;
    (*var_ref).u.value = JS_UNINITIALIZED;
    0
}
unsafe fn delete_property(ctx: *mut JSContext, p: *mut JSObject, atom: JSAtom) -> i32 {
    'redo: loop {
        let mut sh = (*p).shape;
        let h1 = (atom & (*sh).prop_hash_mask) as usize;
        let mut h = *ptr::addr_of!((*sh).hash_table).cast::<u32>().add(h1);
        let prop = get_shape_prop(sh);
        let mut lpr: *mut JSShapeProperty = ptr::null_mut();
        let mut lpr_idx = 0;
        while h != 0 {
            let mut pr = prop.add(h as usize - 1);
            if (*pr).atom == atom {
                if (*pr).flags() as i32 & JS_PROP_CONFIGURABLE == 0 {
                    return 0;
                }
                if !lpr.is_null() {
                    lpr_idx = lpr.offset_from(get_shape_prop(sh)) as u32;
                }
                if js_shape_prepare_update(ctx, p, &mut pr) != 0 {
                    return -1;
                }
                sh = (*p).shape;
                if !lpr.is_null() {
                    lpr = get_shape_prop(sh).add(lpr_idx as usize);
                    (*lpr).set_hash_next((*pr).hash_next());
                } else {
                    *ptr::addr_of_mut!((*sh).hash_table).cast::<u32>().add(h1) = (*pr).hash_next();
                }
                (*sh).deleted_prop_count += 1;
                let pr1 = (*p).prop.add(h as usize - 1);
                if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT
                    && (*pr).flags() as i32 & JS_PROP_TMASK == JS_PROP_VARREF
                    && remove_global_object_property(ctx, p, pr, pr1) != 0
                {
                    return -1;
                }
                free_property((*ctx).rt, pr1, (*pr).flags() as i32);
                JS_FreeAtom(ctx, (*pr).atom);
                (*pr).set_flags(0);
                (*pr).atom = JS_ATOM_NULL as u32;
                (*pr1).u.value = JS_UNDEFINED;
                if (*sh).deleted_prop_count >= 8
                    && (*sh).deleted_prop_count as u32 >= (*sh).prop_count as u32 / 2
                {
                    compact_properties(ctx, p);
                }
                return 1;
            }
            lpr = pr;
            h = (*pr).hash_next();
        }
        if (*p).is_exotic() != 0 {
            if (*p).fast_array() != 0 {
                let mut idx = 0;
                if JS_AtomIsArrayIndex(ctx, &mut idx, atom) != 0 && idx < (*p).u.array.count {
                    if matches!(
                        (*p).class_id as u32,
                        JS_CLASS_ARRAY | JS_CLASS_ARGUMENTS | JS_CLASS_MAPPED_ARGUMENTS
                    ) {
                        if idx == (*p).u.array.count - 1 {
                            if (*p).class_id as u32 == JS_CLASS_MAPPED_ARGUMENTS {
                                free_var_ref((*ctx).rt, *(*p).u.array.u.var_refs.add(idx as usize));
                            } else {
                                JS_FreeValue(ctx, *(*p).u.array.u.values.add(idx as usize));
                            }
                            (*p).u.array.count = idx;
                            return 1;
                        }
                        if convert_fast_array_to_array(ctx, p) != 0 {
                            return -1;
                        }
                        continue 'redo;
                    } else {
                        return 0;
                    }
                }
            } else {
                let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
                if !em.is_null() {
                    if let Some(delete_property) = (*em).delete_property {
                        return delete_property(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()), atom);
                    }
                }
            }
        }
        return 1;
    }
}
unsafe fn set_array_length(ctx: *mut JSContext, p: *mut JSObject, val: JSValue, flags: i32) -> i32 {
    let mut len = 0;
    if JS_ToArrayLengthFree(ctx, &mut len, val, 0) != 0 {
        return -1;
    }
    if (*get_shape_prop((*p).shape)).flags() as i32 & JS_PROP_WRITABLE == 0 {
        return JS_ThrowTypeErrorReadOnly(ctx, flags, crate::quickjs_atom::JS_ATOM_length);
    }
    if (*p).fast_array() != 0 {
        let old_len = (*p).u.array.count;
        if len < old_len {
            for i in len..old_len {
                JS_FreeValue(ctx, *(*p).u.array.u.values.add(i as usize));
            }
            (*p).u.array.count = len;
        }
        (*(*p).prop).u.value = JS_NewUint32(ctx, len);
    } else {
        let mut cur_len = 0;
        JS_ToUint32(ctx, &mut cur_len, (*(*p).prop).u.value);
        if len < cur_len {
            let d = cur_len - len;
            let mut sh = (*p).shape;
            if d <= (*sh).prop_count as u32 {
                while cur_len > len {
                    let atom = JS_NewAtomUInt32(ctx, cur_len - 1);
                    let ret = delete_property(ctx, p, atom);
                    JS_FreeAtom(ctx, atom);
                    // C checks only FALSE here; its negative return is preserved.
                    if ret == 0 {
                        break;
                    }
                    cur_len -= 1;
                }
            } else {
                cur_len = len;
                let mut i = 0;
                let mut pr = get_shape_prop(sh);
                while i < (*sh).prop_count {
                    let mut idx = 0;
                    if (*pr).atom != JS_ATOM_NULL as u32
                        && JS_AtomIsArrayIndex(ctx, &mut idx, (*pr).atom) != 0
                        && idx >= cur_len
                        && (*pr).flags() as i32 & JS_PROP_CONFIGURABLE == 0
                    {
                        cur_len = idx.wrapping_add(1);
                    }
                    i += 1;
                    pr = pr.add(1);
                }
                i = 0;
                pr = get_shape_prop(sh);
                while i < (*sh).prop_count {
                    let mut idx = 0;
                    if (*pr).atom != JS_ATOM_NULL as u32
                        && JS_AtomIsArrayIndex(ctx, &mut idx, (*pr).atom) != 0
                        && idx >= cur_len
                    {
                        delete_property(ctx, p, (*pr).atom);
                        sh = (*p).shape;
                        pr = get_shape_prop(sh).add(i as usize);
                    }
                    i += 1;
                    pr = pr.add(1);
                }
            }
        } else {
            cur_len = len;
        }
        set_value(ctx, &mut (*(*p).prop).u.value, JS_NewUint32(ctx, cur_len));
        if cur_len > len {
            return JS_ThrowTypeErrorOrFalse(ctx, flags, c"not configurable".as_ptr());
        }
    }
    1
}
unsafe fn expand_fast_array(ctx: *mut JSContext, p: *mut JSObject, new_len: u32) -> i32 {
    let mut new_size =
        (new_len as i32).max(((*p).u.array.u1.size.wrapping_mul(3) / 2) as i32) as u32;
    let mut slack = 0;
    let new_array_prop = js_realloc2(
        ctx,
        (*p).u.array.u.values.cast(),
        size_of::<JSValue>() * new_size as usize,
        &mut slack,
    )
    .cast::<JSValue>();
    if new_array_prop.is_null() {
        return -1;
    }
    new_size = (new_size as usize).wrapping_add(slack / size_of::<JSValue>()) as u32;
    (*p).u.array.u.values = new_array_prop;
    (*p).u.array.u1.size = new_size;
    0
}
#[inline]
unsafe fn add_fast_array_element(
    ctx: *mut JSContext,
    p: *mut JSObject,
    val: JSValue,
    flags: i32,
) -> i32 {
    let new_len = (*p).u.array.count.wrapping_add(1);
    if JS_VALUE_GET_TAG((*(*p).prop).u.value) == JS_TAG_INT {
        let array_len = JS_VALUE_GET_INT((*(*p).prop).u.value) as u32;
        if new_len > array_len {
            if (*get_shape_prop((*p).shape)).flags() as i32 & JS_PROP_WRITABLE == 0 {
                JS_FreeValue(ctx, val);
                return JS_ThrowTypeErrorReadOnly(ctx, flags, crate::quickjs_atom::JS_ATOM_length);
            }
            (*(*p).prop).u.value = JS_NewInt32(ctx, new_len as i32);
        }
    }
    if new_len > (*p).u.array.u1.size && expand_fast_array(ctx, p, new_len) != 0 {
        JS_FreeValue(ctx, val);
        return -1;
    }
    *(*p).u.array.u.values.add(new_len as usize - 1) = val;
    (*p).u.array.count = new_len;
    1
}
unsafe fn js_allocate_fast_array(ctx: *mut JSContext, len: i64) -> JSValue {
    if len > i32::MAX as i64 {
        return JS_ThrowRangeError(ctx, c"invalid array length".as_ptr());
    }
    let arr = JS_NewArray(ctx);
    if JS_IsException(arr) != 0 {
        return arr;
    }
    if len > 0 {
        let p = JS_VALUE_GET_PTR(arr).cast::<JSObject>();
        if expand_fast_array(ctx, p, len as u32) < 0 {
            JS_FreeValue(ctx, arr);
            return JS_EXCEPTION;
        }
        (*p).u.array.count = len as u32;
        for i in 0..len {
            *(*p).u.array.u.values.add(i as usize) = JS_UNDEFINED;
        }
        set_value(ctx, &mut (*(*p).prop).u.value, JS_NewInt32(ctx, len as i32));
    }
    arr
}
unsafe fn js_create_array(ctx: *mut JSContext, len: i32, tab: *const JSValueConst) -> JSValue {
    let obj = JS_NewArray(ctx);
    if JS_IsException(obj) != 0 {
        return JS_EXCEPTION;
    }
    if len > 0 {
        let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        if expand_fast_array(ctx, p, len as u32) < 0 {
            JS_FreeValue(ctx, obj);
            return JS_EXCEPTION;
        }
        (*p).u.array.count = len as u32;
        for i in 0..len {
            *(*p).u.array.u.values.add(i as usize) = JS_DupValue(ctx, *tab.add(i as usize));
        }
        set_value(ctx, &mut (*(*p).prop).u.value, JS_NewInt32(ctx, len));
    }
    obj
}
unsafe fn js_create_array_free(ctx: *mut JSContext, len: i32, tab: *mut JSValue) -> JSValue {
    let obj = JS_NewArray(ctx);
    if JS_IsException(obj) == 0 {
        if len <= 0 {
            return obj;
        }
        let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        if expand_fast_array(ctx, p, len as u32) >= 0 {
            (*p).u.array.count = len as u32;
            for i in 0..len {
                *(*p).u.array.u.values.add(i as usize) = *tab.add(i as usize);
            }
            set_value(ctx, &mut (*(*p).prop).u.value, JS_NewInt32(ctx, len));
            return obj;
        }
        JS_FreeValue(ctx, obj);
    }
    for i in 0..len {
        JS_FreeValue(ctx, *tab.add(i as usize));
    }
    JS_EXCEPTION
}
unsafe fn js_create_var_ref(ctx: *mut JSContext, is_lexical: JS_BOOL) -> *mut JSVarRef {
    let var_ref = js_malloc(ctx, size_of::<JSVarRef>()).cast::<JSVarRef>();
    if var_ref.is_null() {
        return ptr::null_mut();
    }
    (*js_rc(var_ref.cast())).ref_count = 1;
    (*var_ref).u.value = if is_lexical != 0 {
        JS_UNINITIALIZED
    } else {
        JS_UNDEFINED
    };
    (*var_ref).pvalue = ptr::addr_of_mut!((*var_ref).u.value);
    (*var_ref).is_detached = 1;
    (*var_ref).is_lexical = 0;
    (*var_ref).is_const = 0;
    add_gc_object((*ctx).rt, &mut (*var_ref).header, JS_GC_OBJ_TYPE_VAR_REF);
    var_ref
}
unsafe fn js_global_object_find_uninitialized_var(
    ctx: *mut JSContext,
    p: *mut JSObject,
    atom: JSAtom,
    is_lexical: JS_BOOL,
) -> *mut JSVarRef {
    let p1 = JS_VALUE_GET_PTR((*p).u.global_object.uninitialized_vars).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p1, atom);
    if !prs.is_null() {
        assert!((*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_VARREF);
        let var_ref = (*pr).u.var_ref;
        (*js_rc(var_ref.cast())).ref_count += 1;
        delete_property(ctx, p1, atom);
        if is_lexical == 0 {
            (*var_ref).u.value = JS_UNDEFINED;
        }
        var_ref
    } else {
        js_create_var_ref(ctx, is_lexical)
    }
}
