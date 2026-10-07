// quickjs.c:8210-8364,8817-9025,9029-9176. MIT.
// Staged: getter dispatch uses the original JS_CallFree dependency.
pub unsafe fn JS_GetPropertyInternal(
    ctx: *mut JSContext,
    obj: JSValueConst,
    prop: JSAtom,
    this_obj: JSValueConst,
    throw_ref_error: JS_BOOL,
) -> JSValue {
    js_host_error_stack_read(ctx, obj, prop);
    let tag = JS_VALUE_GET_TAG(obj);
    let mut p;
    if tag != JS_TAG_OBJECT {
        match tag {
            JS_TAG_NULL => {
                return JS_ThrowTypeErrorAtom(
                    ctx,
                    c"cannot read property '%s' of null".as_ptr(),
                    prop,
                )
            }
            JS_TAG_UNDEFINED => {
                return JS_ThrowTypeErrorAtom(
                    ctx,
                    c"cannot read property '%s' of undefined".as_ptr(),
                    prop,
                )
            }
            JS_TAG_EXCEPTION => return JS_EXCEPTION,
            JS_TAG_STRING | JS_TAG_STRING_ROPE => {
                let len = string_rope_get_len(obj);
                if __JS_AtomIsTaggedInt(prop) != 0 {
                    let idx = __JS_AtomToUInt32(prop);
                    if idx < len {
                        return js_new_string_char(ctx, string_rope_get(obj, idx) as u16);
                    }
                } else if prop == crate::quickjs_atom::JS_ATOM_length {
                    return JS_NewInt32(ctx, len as i32);
                }
            }
            _ => {}
        }
        p = JS_VALUE_GET_PTR(JS_GetPrototypePrimitive(ctx, obj)).cast::<JSObject>();
        if p.is_null() {
            return JS_UNDEFINED;
        }
    } else {
        p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    }
    loop {
        let mut pr = ptr::null_mut();
        let prs = find_own_property(&mut pr, p, prop);
        if !prs.is_null() {
            match (*prs).flags() as i32 & JS_PROP_TMASK {
                JS_PROP_GETSET => {
                    let getter = (*pr).u.getset.getter;
                    if getter.is_null() {
                        return JS_UNDEFINED;
                    }
                    let func = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, getter.cast()));
                    return JS_CallFree(ctx, func, this_obj, 0, ptr::null_mut());
                }
                JS_PROP_VARREF => {
                    let val = *(*(*pr).u.var_ref).pvalue;
                    if JS_IsUninitialized(val) != 0 {
                        return JS_ThrowReferenceErrorUninitialized(ctx, (*prs).atom);
                    }
                    return JS_DupValue(ctx, val);
                }
                JS_PROP_AUTOINIT => {
                    if JS_AutoInitProperty(ctx, p, prop, pr, prs) != 0 {
                        return JS_EXCEPTION;
                    }
                    continue;
                }
                _ => return JS_DupValue(ctx, (*pr).u.value),
            }
        }
        if (*p).is_exotic() != 0 {
            if (*p).fast_array() != 0 {
                let typed = ((*p).class_id as u32) >= JS_CLASS_UINT8C_ARRAY
                    && ((*p).class_id as u32) <= JS_CLASS_FLOAT64_ARRAY;
                if __JS_AtomIsTaggedInt(prop) != 0 {
                    let idx = __JS_AtomToUInt32(prop);
                    if idx < (*p).u.array.count {
                        return JS_GetPropertyUint32(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()), idx);
                    } else if typed {
                        return JS_UNDEFINED;
                    }
                } else if typed {
                    let ret = JS_AtomIsNumericIndex(ctx, prop);
                    if ret != 0 {
                        return if ret < 0 { JS_EXCEPTION } else { JS_UNDEFINED };
                    }
                }
            } else {
                let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
                if !em.is_null() {
                    if let Some(get_property) = (*em).get_property {
                        let obj1 = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
                        let ret = get_property(ctx, obj1, prop, this_obj);
                        JS_FreeValue(ctx, obj1);
                        return ret;
                    }
                    if let Some(get_own_property) = (*em).get_own_property {
                        let mut desc: JSPropertyDescriptor = core::mem::zeroed();
                        let obj1 = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
                        let ret = get_own_property(ctx, &mut desc, obj1, prop);
                        JS_FreeValue(ctx, obj1);
                        if ret < 0 {
                            return JS_EXCEPTION;
                        }
                        if ret != 0 {
                            if desc.flags & JS_PROP_GETSET != 0 {
                                JS_FreeValue(ctx, desc.setter);
                                return JS_CallFree(ctx, desc.getter, this_obj, 0, ptr::null_mut());
                            }
                            return desc.value;
                        }
                    }
                }
            }
        }
        p = (*(*p).shape).proto;
        if p.is_null() {
            break;
        }
    }
    if throw_ref_error != 0 {
        JS_ThrowReferenceErrorNotDefined(ctx, prop)
    } else {
        JS_UNDEFINED
    }
}
// quickjs.h inline wrapper.
pub unsafe fn JS_GetProperty(ctx: *mut JSContext, obj: JSValueConst, prop: JSAtom) -> JSValue {
    JS_GetPropertyInternal(ctx, obj, prop, obj, 0)
}
unsafe fn JS_GetOwnPropertyInternal(
    ctx: *mut JSContext,
    desc: *mut JSPropertyDescriptor,
    p: *mut JSObject,
    prop: JSAtom,
) -> i32 {
    loop {
        let mut pr = ptr::null_mut();
        let prs = find_own_property(&mut pr, p, prop);
        if !prs.is_null() {
            let typ = (*prs).flags() as i32 & JS_PROP_TMASK;
            if !desc.is_null() {
                (*desc).flags = (*prs).flags() as i32 & JS_PROP_C_W_E;
                (*desc).getter = JS_UNDEFINED;
                (*desc).setter = JS_UNDEFINED;
                (*desc).value = JS_UNDEFINED;
                match typ {
                    JS_PROP_GETSET => {
                        (*desc).flags |= JS_PROP_GETSET;
                        if !(*pr).u.getset.getter.is_null() {
                            (*desc).getter = JS_DupValue(
                                ctx,
                                JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.getter.cast()),
                            );
                        }
                        if !(*pr).u.getset.setter.is_null() {
                            (*desc).setter = JS_DupValue(
                                ctx,
                                JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.setter.cast()),
                            );
                        }
                    }
                    JS_PROP_VARREF => {
                        let val = *(*(*pr).u.var_ref).pvalue;
                        if JS_IsUninitialized(val) != 0 {
                            JS_ThrowReferenceErrorUninitialized(ctx, (*prs).atom);
                            return -1;
                        }
                        (*desc).value = JS_DupValue(ctx, val);
                    }
                    JS_PROP_AUTOINIT => {
                        if JS_AutoInitProperty(ctx, p, prop, pr, prs) != 0 {
                            return -1;
                        }
                        continue;
                    }
                    _ => (*desc).value = JS_DupValue(ctx, (*pr).u.value),
                }
            } else if typ == JS_PROP_VARREF && JS_IsUninitialized(*(*(*pr).u.var_ref).pvalue) != 0 {
                JS_ThrowReferenceErrorUninitialized(ctx, (*prs).atom);
                return -1;
            }
            return 1;
        }
        if (*p).is_exotic() != 0 {
            if (*p).fast_array() != 0 {
                if __JS_AtomIsTaggedInt(prop) != 0 {
                    let idx = __JS_AtomToUInt32(prop);
                    if idx < (*p).u.array.count {
                        if !desc.is_null() {
                            (*desc).flags = JS_PROP_C_W_E;
                            (*desc).getter = JS_UNDEFINED;
                            (*desc).setter = JS_UNDEFINED;
                            (*desc).value =
                                JS_GetPropertyUint32(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()), idx);
                        }
                        return 1;
                    }
                }
            } else {
                let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
                if !em.is_null() {
                    if let Some(get_own_property) = (*em).get_own_property {
                        return get_own_property(
                            ctx,
                            desc,
                            JS_MKPTR(JS_TAG_OBJECT, p.cast()),
                            prop,
                        );
                    }
                }
            }
        }
        return 0;
    }
}
pub unsafe fn JS_GetOwnProperty(
    ctx: *mut JSContext,
    desc: *mut JSPropertyDescriptor,
    obj: JSValueConst,
    prop: JSAtom,
) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        JS_ThrowTypeErrorNotAnObject(ctx);
        return -1;
    }
    JS_GetOwnPropertyInternal(ctx, desc, JS_VALUE_GET_PTR(obj).cast(), prop)
}
unsafe fn array_buffer_is_resizable(abuf: *const JSArrayBuffer) -> JS_BOOL {
    ((*abuf).max_byte_length >= 0) as i32
}
pub unsafe fn JS_PreventExtensions(ctx: *mut JSContext, obj: JSValueConst) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return 0;
    }
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    if (*p).is_exotic() != 0 {
        if ((*p).class_id as u32) >= JS_CLASS_UINT8C_ARRAY
            && ((*p).class_id as u32) <= JS_CLASS_FLOAT64_ARRAY
        {
            let ta = (*p).u.typed_array;
            let abuf = (*(*ta).buffer).u.array_buffer;
            if (*ta).track_rab != 0 || (array_buffer_is_resizable(abuf) != 0 && (*abuf).shared == 0)
            {
                return 0;
            }
        } else {
            let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
            if !em.is_null() {
                if let Some(prevent_extensions) = (*em).prevent_extensions {
                    return prevent_extensions(ctx, obj);
                }
            }
        }
    }
    (*p).set_extensible(0);
    1
}
pub unsafe fn JS_HasProperty(ctx: *mut JSContext, obj: JSValueConst, prop: JSAtom) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return 0;
    }
    let mut p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    loop {
        if (*p).is_exotic() != 0 {
            let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
            if !em.is_null() {
                if let Some(has_property) = (*em).has_property {
                    let obj1 = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
                    let ret = has_property(ctx, obj1, prop);
                    JS_FreeValue(ctx, obj1);
                    return ret;
                }
            }
        }
        JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
        let ret = JS_GetOwnPropertyInternal(ctx, ptr::null_mut(), p, prop);
        JS_FreeValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
        if ret != 0 {
            return ret;
        }
        if ((*p).class_id as u32) >= JS_CLASS_UINT8C_ARRAY
            && ((*p).class_id as u32) <= JS_CLASS_FLOAT64_ARRAY
        {
            let ret = JS_AtomIsNumericIndex(ctx, prop);
            if ret != 0 {
                return if ret < 0 { -1 } else { 0 };
            }
        }
        p = (*(*p).shape).proto;
        if p.is_null() {
            return 0;
        }
    }
}
unsafe fn JS_GetPropertyValue(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSValue,
) -> JSValue {
    if JS_VALUE_GET_TAG(this_obj) == JS_TAG_OBJECT && JS_VALUE_GET_TAG(prop) == JS_TAG_INT {
        let p = JS_VALUE_GET_PTR(this_obj).cast::<JSObject>();
        let idx = JS_VALUE_GET_INT(prop) as u32;
        let class_id = (*p).class_id as u32;
        // Match the C switch before reading the array union. Ordinary object
        // payloads do not initialize its count or element pointer fields.
        if (matches!(
            class_id,
            JS_CLASS_ARRAY | JS_CLASS_ARGUMENTS | JS_CLASS_MAPPED_ARGUMENTS
        ) || (class_id >= JS_CLASS_UINT8C_ARRAY && class_id <= JS_CLASS_FLOAT64_ARRAY))
            && idx < (*p).u.array.count
        {
            let i = idx as usize;
            let arr = (*p).u.array.u;
            match class_id {
                JS_CLASS_ARRAY | JS_CLASS_ARGUMENTS => return JS_DupValue(ctx, *arr.values.add(i)),
                JS_CLASS_MAPPED_ARGUMENTS => {
                    return JS_DupValue(ctx, *(*(*arr.var_refs.add(i))).pvalue)
                }
                JS_CLASS_INT8_ARRAY => return JS_NewInt32(ctx, *arr.int8_ptr.add(i) as i32),
                JS_CLASS_UINT8C_ARRAY | JS_CLASS_UINT8_ARRAY => {
                    return JS_NewInt32(ctx, *arr.uint8_ptr.add(i) as i32)
                }
                JS_CLASS_INT16_ARRAY => return JS_NewInt32(ctx, *arr.int16_ptr.add(i) as i32),
                JS_CLASS_UINT16_ARRAY => return JS_NewInt32(ctx, *arr.uint16_ptr.add(i) as i32),
                JS_CLASS_INT32_ARRAY => return JS_NewInt32(ctx, *arr.int32_ptr.add(i)),
                JS_CLASS_UINT32_ARRAY => return JS_NewUint32(ctx, *arr.uint32_ptr.add(i)),
                JS_CLASS_BIG_INT64_ARRAY => return JS_NewBigInt64(ctx, *arr.int64_ptr.add(i)),
                JS_CLASS_BIG_UINT64_ARRAY => return JS_NewBigUint64(ctx, *arr.uint64_ptr.add(i)),
                JS_CLASS_FLOAT16_ARRAY => {
                    return __JS_NewFloat64(
                        ctx,
                        crate::cutils_header::fromfp16(*arr.fp16_ptr.add(i)),
                    )
                }
                JS_CLASS_FLOAT32_ARRAY => {
                    return __JS_NewFloat64(ctx, *arr.float_ptr.add(i) as f64)
                }
                JS_CLASS_FLOAT64_ARRAY => return __JS_NewFloat64(ctx, *arr.double_ptr.add(i)),
                _ => {}
            }
        }
    }
    if JS_IsNull(this_obj) != 0 || JS_IsUndefined(this_obj) != 0 {
        JS_FreeValue(ctx, prop);
        return JS_ThrowTypeError(
            ctx,
            if JS_IsNull(this_obj) != 0 {
                c"cannot read property of null".as_ptr()
            } else {
                c"cannot read property of undefined".as_ptr()
            },
        );
    }
    let atom = JS_ValueToAtom(ctx, prop);
    JS_FreeValue(ctx, prop);
    if atom == JS_ATOM_NULL as u32 {
        return JS_EXCEPTION;
    }
    let ret = JS_GetProperty(ctx, this_obj, atom);
    JS_FreeAtom(ctx, atom);
    ret
}
pub unsafe fn JS_GetPropertyUint32(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    idx: u32,
) -> JSValue {
    JS_GetPropertyValue(ctx, this_obj, JS_NewUint32(ctx, idx))
}
unsafe fn JS_TryGetPropertyInt64(
    ctx: *mut JSContext,
    obj: JSValueConst,
    idx: i64,
    pval: *mut JSValue,
) -> i32 {
    let mut val = JS_UNDEFINED;
    let mut present;
    if idx as u64 <= JS_ATOM_MAX_INT as u64 {
        present = JS_HasProperty(ctx, obj, __JS_AtomFromUInt32(idx as u32));
        if present > 0 {
            val = JS_GetPropertyValue(ctx, obj, JS_NewInt32(ctx, idx as i32));
            if JS_IsException(val) != 0 {
                present = -1;
            }
        }
    } else {
        let prop = JS_NewAtomInt64(ctx, idx);
        present = -1;
        if prop != JS_ATOM_NULL as u32 {
            present = JS_HasProperty(ctx, obj, prop);
            if present > 0 {
                val = JS_GetProperty(ctx, obj, prop);
                if JS_IsException(val) != 0 {
                    present = -1;
                }
            }
            JS_FreeAtom(ctx, prop);
        }
    }
    *pval = val;
    present
}
unsafe fn JS_GetPropertyInt64(ctx: *mut JSContext, obj: JSValueConst, idx: i64) -> JSValue {
    if idx as u64 <= i32::MAX as u64 {
        return JS_GetPropertyValue(ctx, obj, JS_NewInt32(ctx, idx as i32));
    }
    let prop = JS_NewAtomInt64(ctx, idx);
    if prop == JS_ATOM_NULL as u32 {
        return JS_EXCEPTION;
    }
    let val = JS_GetProperty(ctx, obj, prop);
    JS_FreeAtom(ctx, prop);
    val
}
pub unsafe fn JS_GetPropertyStr(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: *const c_char,
) -> JSValue {
    let atom = JS_NewAtom(ctx, prop);
    if atom == JS_ATOM_NULL as u32 {
        return JS_EXCEPTION;
    }
    let ret = JS_GetProperty(ctx, this_obj, atom);
    JS_FreeAtom(ctx, atom);
    ret
}
