// quickjs.c:9407-9431,9652-10119,10924-10968. MIT.
// Follows the original setter/prototype/receiver ownership and lookup order.
unsafe fn call_setter(
    ctx: *mut JSContext,
    setter: *mut JSObject,
    this_obj: JSValueConst,
    mut val: JSValue,
    flags: i32,
) -> i32 {
    if !setter.is_null() {
        let func = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, setter.cast()));
        let ret = JS_CallFree(ctx, func, this_obj, 1, &mut val);
        JS_FreeValue(ctx, val);
        if JS_IsException(ret) != 0 {
            return -1;
        }
        JS_FreeValue(ctx, ret);
        1
    } else {
        JS_FreeValue(ctx, val);
        if flags & JS_PROP_THROW != 0
            || (flags & JS_PROP_THROW_STRICT != 0 && is_strict_mode(ctx) != 0)
        {
            JS_ThrowTypeError(ctx, c"no setter for property".as_ptr());
            -1
        } else {
            0
        }
    }
}
unsafe fn js_free_desc(ctx: *mut JSContext, desc: *mut JSPropertyDescriptor) {
    JS_FreeValue(ctx, (*desc).getter);
    JS_FreeValue(ctx, (*desc).setter);
    JS_FreeValue(ctx, (*desc).value);
}
pub unsafe fn JS_SetPropertyInternal(
    ctx: *mut JSContext,
    obj: JSValueConst,
    prop: JSAtom,
    mut val: JSValue,
    this_obj: JSValueConst,
    flags: i32,
) -> i32 {
    macro_rules! read_only_prop {
        () => {{
            JS_FreeValue(ctx, val);
            return JS_ThrowTypeErrorReadOnly(ctx, flags, prop);
        }};
    }
    let p: *mut JSObject;
    let mut p1: *mut JSObject;
    let mut lookup_own;
    if JS_VALUE_GET_TAG(this_obj) != JS_TAG_OBJECT {
        p = ptr::null_mut();
        if JS_VALUE_GET_TAG(obj) == JS_TAG_OBJECT {
            p1 = JS_VALUE_GET_PTR(obj).cast();
        } else {
            match JS_VALUE_GET_TAG(this_obj) {
                JS_TAG_NULL => {
                    JS_FreeValue(ctx, val);
                    JS_ThrowTypeErrorAtom(ctx, c"cannot set property '%s' of null".as_ptr(), prop);
                    return -1;
                }
                JS_TAG_UNDEFINED => {
                    JS_FreeValue(ctx, val);
                    JS_ThrowTypeErrorAtom(
                        ctx,
                        c"cannot set property '%s' of undefined".as_ptr(),
                        prop,
                    );
                    return -1;
                }
                _ => {
                    p1 = JS_VALUE_GET_PTR(JS_GetPrototypePrimitive(ctx, obj)).cast();
                }
            }
        }
        lookup_own = true;
    } else {
        p = JS_VALUE_GET_PTR(this_obj).cast();
        p1 = JS_VALUE_GET_PTR(obj).cast();
        lookup_own = p != p1;
        if !lookup_own {
            loop {
                let mut pr = ptr::null_mut();
                let prs = find_own_property(&mut pr, p1, prop);
                if prs.is_null() {
                    break;
                }
                let pf = (*prs).flags() as i32;
                if pf & (JS_PROP_TMASK | JS_PROP_WRITABLE | JS_PROP_LENGTH) == JS_PROP_WRITABLE {
                    set_value(ctx, ptr::addr_of_mut!((*pr).u.value), val);
                    return 1;
                } else if pf & JS_PROP_LENGTH != 0 {
                    debug_assert_eq!((*p).class_id as u32, JS_CLASS_ARRAY);
                    debug_assert_eq!(prop, crate::quickjs_atom::JS_ATOM_length);
                    return set_array_length(ctx, p, val, flags);
                } else if pf & JS_PROP_TMASK == JS_PROP_GETSET {
                    return call_setter(ctx, (*pr).u.getset.setter, this_obj, val, flags);
                } else if pf & JS_PROP_TMASK == JS_PROP_VARREF {
                    let vr = (*pr).u.var_ref;
                    if (*p).class_id as u32 == JS_CLASS_MODULE_NS || (*vr).is_const != 0 {
                        read_only_prop!();
                    }
                    set_value(ctx, (*vr).pvalue, val);
                    return 1;
                } else if pf & JS_PROP_TMASK == JS_PROP_AUTOINIT {
                    if JS_AutoInitProperty(ctx, p, prop, pr, prs) != 0 {
                        JS_FreeValue(ctx, val);
                        return -1;
                    }
                    continue;
                } else {
                    read_only_prop!();
                }
            }
        }
    }
    'prototype_walk: loop {
        if p1.is_null() {
            break;
        }
        if lookup_own {
            loop {
                let mut pr = ptr::null_mut();
                let prs = find_own_property(&mut pr, p1, prop);
                if prs.is_null() {
                    break;
                }
                let pf = (*prs).flags() as i32;
                if pf & JS_PROP_TMASK == JS_PROP_GETSET {
                    return call_setter(ctx, (*pr).u.getset.setter, this_obj, val, flags);
                } else if pf & JS_PROP_TMASK == JS_PROP_AUTOINIT {
                    // The source retry2 branch does not release val on failure.
                    if JS_AutoInitProperty(ctx, p1, prop, pr, prs) != 0 {
                        return -1;
                    }
                    continue;
                } else if pf & JS_PROP_WRITABLE == 0 {
                    read_only_prop!();
                } else {
                    break 'prototype_walk;
                }
            }
        }
        if (*p1).is_exotic() != 0 {
            if (*p1).fast_array() != 0 {
                let class = (*p1).class_id as u32;
                let typed = class >= JS_CLASS_UINT8C_ARRAY && class <= JS_CLASS_FLOAT64_ARRAY;
                let oob;
                if __JS_AtomIsTaggedInt(prop) != 0 {
                    let idx = __JS_AtomToUInt32(prop);
                    if idx < (*p1).u.array.count {
                        if p == p1 {
                            return JS_SetPropertyValue(
                                ctx,
                                this_obj,
                                JS_NewInt32(ctx, idx as i32),
                                val,
                                flags,
                            );
                        } else {
                            break;
                        }
                    }
                    oob = typed;
                } else if typed {
                    let ret = JS_AtomIsNumericIndex(ctx, prop);
                    if ret < 0 {
                        JS_FreeValue(ctx, val);
                        return -1;
                    }
                    oob = ret != 0;
                } else {
                    oob = false;
                }
                if oob {
                    if p == p1 {
                        if class == JS_CLASS_BIG_INT64_ARRAY || class == JS_CLASS_BIG_UINT64_ARRAY {
                            let mut v = 0;
                            if JS_ToBigInt64Free(ctx, &mut v, val) != 0 {
                                return -1;
                            }
                        } else {
                            val = JS_ToNumberFree(ctx, val);
                            JS_FreeValue(ctx, val);
                            if JS_IsException(val) != 0 {
                                return -1;
                            }
                        }
                    } else {
                        JS_FreeValue(ctx, val);
                    }
                    return 1;
                }
            } else {
                let em = (*(*(*ctx).rt).class_array.add((*p1).class_id as usize)).exotic;
                if !em.is_null() {
                    if let Some(set_property) = (*em).set_property {
                        let obj1 = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p1.cast()));
                        let ret = set_property(ctx, obj1, prop, val, this_obj, flags);
                        JS_FreeValue(ctx, obj1);
                        JS_FreeValue(ctx, val);
                        return ret;
                    }
                    if let Some(get_own_property) = (*em).get_own_property {
                        let mut desc: JSPropertyDescriptor = core::mem::zeroed();
                        let obj1 = JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, p1.cast()));
                        let ret = get_own_property(ctx, &mut desc, obj1, prop);
                        JS_FreeValue(ctx, obj1);
                        if ret < 0 {
                            JS_FreeValue(ctx, val);
                            return ret;
                        }
                        if ret != 0 {
                            if desc.flags & JS_PROP_GETSET != 0 {
                                let setter = if JS_IsUndefined(desc.setter) != 0 {
                                    ptr::null_mut()
                                } else {
                                    JS_VALUE_GET_PTR(desc.setter).cast()
                                };
                                let ret = call_setter(ctx, setter, this_obj, val, flags);
                                JS_FreeValue(ctx, desc.getter);
                                JS_FreeValue(ctx, desc.setter);
                                return ret;
                            } else {
                                JS_FreeValue(ctx, desc.value);
                                if desc.flags & JS_PROP_WRITABLE == 0 {
                                    read_only_prop!();
                                }
                                if p == p1 {
                                    let ret = JS_DefineProperty(
                                        ctx,
                                        this_obj,
                                        prop,
                                        val,
                                        JS_UNDEFINED,
                                        JS_UNDEFINED,
                                        JS_PROP_HAS_VALUE,
                                    );
                                    JS_FreeValue(ctx, val);
                                    return ret;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        p1 = (*(*p1).shape).proto;
        lookup_own = true;
    }
    if p.is_null() {
        JS_FreeValue(ctx, val);
        return JS_ThrowTypeErrorOrFalse(ctx, flags, c"not an object".as_ptr());
    }
    if (*p).extensible() == 0 {
        JS_FreeValue(ctx, val);
        return JS_ThrowTypeErrorOrFalse(ctx, flags, c"object is not extensible".as_ptr());
    }
    if p == JS_VALUE_GET_PTR(obj).cast() {
        if (*p).is_exotic() != 0 {
            if (*p).class_id as u32 == JS_CLASS_ARRAY
                && (*p).fast_array() != 0
                && __JS_AtomIsTaggedInt(prop) != 0
            {
                let idx = __JS_AtomToUInt32(prop);
                if idx == (*p).u.array.count {
                    return add_fast_array_element(ctx, p, val, flags);
                }
            }
        } else if (*p).class_id as u32 != JS_CLASS_GLOBAL_OBJECT {
            let pr = add_property(ctx, p, prop, JS_PROP_C_W_E);
            if pr.is_null() {
                JS_FreeValue(ctx, val);
                return -1;
            }
            (*pr).u.value = val;
            return 1;
        }
    } else {
        let mut desc: JSPropertyDescriptor = core::mem::zeroed();
        let ret = JS_GetOwnPropertyInternal(ctx, &mut desc, p, prop);
        if ret < 0 {
            JS_FreeValue(ctx, val);
            return ret;
        }
        if ret != 0 {
            if desc.flags & JS_PROP_GETSET != 0 {
                JS_FreeValue(ctx, desc.getter);
                JS_FreeValue(ctx, desc.setter);
                JS_FreeValue(ctx, val);
                return JS_ThrowTypeErrorOrFalse(ctx, flags, c"setter is forbidden".as_ptr());
            }
            JS_FreeValue(ctx, desc.value);
            if desc.flags & JS_PROP_WRITABLE == 0 || (*p).class_id as u32 == JS_CLASS_MODULE_NS {
                read_only_prop!();
            }
            let ret = JS_DefineProperty(
                ctx,
                this_obj,
                prop,
                val,
                JS_UNDEFINED,
                JS_UNDEFINED,
                JS_PROP_HAS_VALUE,
            );
            JS_FreeValue(ctx, val);
            return ret;
        }
    }
    let ret = JS_CreateProperty(
        ctx,
        p,
        prop,
        val,
        JS_UNDEFINED,
        JS_UNDEFINED,
        flags
            | JS_PROP_HAS_VALUE
            | JS_PROP_HAS_ENUMERABLE
            | JS_PROP_HAS_WRITABLE
            | JS_PROP_HAS_CONFIGURABLE
            | JS_PROP_C_W_E,
    );
    JS_FreeValue(ctx, val);
    ret
}
#[inline]
unsafe fn can_extend_fast_array(p: *mut JSObject) -> JS_BOOL {
    if (*p).extensible() == 0 {
        return 0;
    }
    let proto = (*(*p).shape).proto;
    if proto.is_null() {
        1
    } else {
        (*proto).is_std_array_prototype() as i32
    }
}
unsafe fn JS_SetPropertyValue(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSValue,
    val: JSValue,
    flags: i32,
) -> i32 {
    if JS_VALUE_GET_TAG(this_obj) == JS_TAG_OBJECT && JS_VALUE_GET_TAG(prop) == JS_TAG_INT {
        let p = JS_VALUE_GET_PTR(this_obj).cast::<JSObject>();
        let idx = JS_VALUE_GET_INT(prop) as u32;
        let i = idx as usize;
        match (*p).class_id as u32 {
            JS_CLASS_ARRAY => {
                if idx >= (*p).u.array.count {
                    if idx == (*p).u.array.count
                        && (*p).fast_array() != 0
                        && can_extend_fast_array(p) != 0
                    {
                        return add_fast_array_element(ctx, p, val, flags);
                    }
                } else {
                    set_value(ctx, (*p).u.array.u.values.add(i), val);
                    return 1;
                }
            }
            JS_CLASS_ARGUMENTS => {
                if idx < (*p).u.array.count {
                    set_value(ctx, (*p).u.array.u.values.add(i), val);
                    return 1;
                }
            }
            JS_CLASS_MAPPED_ARGUMENTS => {
                if idx < (*p).u.array.count {
                    set_value(ctx, (**(*p).u.array.u.var_refs.add(i)).pvalue, val);
                    return 1;
                }
            }
            JS_CLASS_UINT8C_ARRAY
            | JS_CLASS_INT8_ARRAY
            | JS_CLASS_UINT8_ARRAY
            | JS_CLASS_INT16_ARRAY
            | JS_CLASS_UINT16_ARRAY
            | JS_CLASS_INT32_ARRAY
            | JS_CLASS_UINT32_ARRAY => {
                let mut v = 0;
                let ret = if (*p).class_id as u32 == JS_CLASS_UINT8C_ARRAY {
                    JS_ToUint8ClampFree(ctx, &mut v, val)
                } else {
                    JS_ToInt32Free(ctx, &mut v, val)
                };
                if ret != 0 {
                    return -1;
                }
                if idx >= (*p).u.array.count {
                    return 1;
                }
                match (*p).class_id as u32 {
                    JS_CLASS_UINT8C_ARRAY | JS_CLASS_INT8_ARRAY | JS_CLASS_UINT8_ARRAY => {
                        *(*p).u.array.u.uint8_ptr.add(i) = v as u8
                    }
                    JS_CLASS_INT16_ARRAY | JS_CLASS_UINT16_ARRAY => {
                        *(*p).u.array.u.uint16_ptr.add(i) = v as u16
                    }
                    _ => *(*p).u.array.u.uint32_ptr.add(i) = v as u32,
                }
                return 1;
            }
            JS_CLASS_BIG_INT64_ARRAY | JS_CLASS_BIG_UINT64_ARRAY => {
                let mut v = 0;
                if JS_ToBigInt64Free(ctx, &mut v, val) != 0 {
                    return -1;
                }
                if idx < (*p).u.array.count {
                    *(*p).u.array.u.uint64_ptr.add(i) = v as u64;
                }
                return 1;
            }
            JS_CLASS_FLOAT16_ARRAY | JS_CLASS_FLOAT32_ARRAY | JS_CLASS_FLOAT64_ARRAY => {
                let mut d = 0.0;
                if JS_ToFloat64Free(ctx, &mut d, val) != 0 {
                    return -1;
                }
                if idx >= (*p).u.array.count {
                    return 1;
                }
                match (*p).class_id as u32 {
                    JS_CLASS_FLOAT16_ARRAY => {
                        *(*p).u.array.u.fp16_ptr.add(i) = crate::cutils_header::tofp16(d)
                    }
                    JS_CLASS_FLOAT32_ARRAY => *(*p).u.array.u.float_ptr.add(i) = d as f32,
                    _ => *(*p).u.array.u.double_ptr.add(i) = d,
                }
                return 1;
            }
            _ => {}
        }
    }
    let atom = JS_ValueToAtom(ctx, prop);
    JS_FreeValue(ctx, prop);
    if atom == crate::quickjs_atom::JS_ATOM_NULL {
        JS_FreeValue(ctx, val);
        return -1;
    }
    let ret = JS_SetPropertyInternal(ctx, this_obj, atom, val, this_obj, flags);
    JS_FreeAtom(ctx, atom);
    ret
}
// quickjs.h:969-973.
#[inline]
pub unsafe fn JS_SetProperty(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    val: JSValue,
) -> i32 {
    JS_SetPropertyInternal(ctx, this_obj, prop, val, this_obj, JS_PROP_THROW)
}
pub unsafe fn JS_SetPropertyUint32(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    idx: u32,
    val: JSValue,
) -> i32 {
    JS_SetPropertyValue(ctx, this_obj, JS_NewUint32(ctx, idx), val, JS_PROP_THROW)
}
pub unsafe fn JS_SetPropertyInt64(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    idx: i64,
    val: JSValue,
) -> i32 {
    if idx as u64 <= i32::MAX as u64 {
        return JS_SetPropertyValue(
            ctx,
            this_obj,
            JS_NewInt32(ctx, idx as i32),
            val,
            JS_PROP_THROW,
        );
    }
    let prop = JS_NewAtomInt64(ctx, idx);
    if prop == crate::quickjs_atom::JS_ATOM_NULL {
        JS_FreeValue(ctx, val);
        return -1;
    }
    let ret = JS_SetProperty(ctx, this_obj, prop, val);
    JS_FreeAtom(ctx, prop);
    ret
}
pub unsafe fn JS_SetPropertyStr(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: *const c_char,
    val: JSValue,
) -> i32 {
    let atom = JS_NewAtom(ctx, prop);
    if atom == crate::quickjs_atom::JS_ATOM_NULL {
        JS_FreeValue(ctx, val);
        return -1;
    }
    let ret = JS_SetPropertyInternal(ctx, this_obj, atom, val, this_obj, JS_PROP_THROW);
    JS_FreeAtom(ctx, atom);
    ret
}
pub unsafe fn JS_DeleteProperty(
    ctx: *mut JSContext,
    obj: JSValueConst,
    prop: JSAtom,
    flags: i32,
) -> i32 {
    let obj1 = JS_ToObject(ctx, obj);
    if JS_IsException(obj1) != 0 {
        return -1;
    }
    let p = JS_VALUE_GET_PTR(obj1).cast();
    let res = delete_property(ctx, p, prop);
    JS_FreeValue(ctx, obj1);
    if res != 0 {
        return res;
    }
    if flags & JS_PROP_THROW != 0 || (flags & JS_PROP_THROW_STRICT != 0 && is_strict_mode(ctx) != 0)
    {
        JS_ThrowTypeError(ctx, c"could not delete property".as_ptr());
        return -1;
    }
    0
}
pub unsafe fn JS_DeletePropertyInt64(
    ctx: *mut JSContext,
    obj: JSValueConst,
    idx: i64,
    flags: i32,
) -> i32 {
    if idx as u64 <= JS_ATOM_MAX_INT as u64 {
        return JS_DeleteProperty(ctx, obj, __JS_AtomFromUInt32(idx as u32), flags);
    }
    let prop = JS_NewAtomInt64(ctx, idx);
    if prop == crate::quickjs_atom::JS_ATOM_NULL {
        return -1;
    }
    let res = JS_DeleteProperty(ctx, obj, prop, flags);
    JS_FreeAtom(ctx, prop);
    res
}
