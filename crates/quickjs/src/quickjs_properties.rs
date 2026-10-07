// quickjs.c:10126-10270,10350-10758. MIT.
// Full property creation/definition paths. Pending production integration of
// conversions, exotic callbacks, array helpers and automatic initializers.
unsafe fn JS_CreateProperty(
    ctx: *mut JSContext,
    p: *mut JSObject,
    prop: JSAtom,
    val: JSValueConst,
    getter: JSValueConst,
    setter: JSValueConst,
    flags: i32,
) -> i32 {
    if (*p).is_exotic() != 0 {
        if (*p).class_id as u32 == JS_CLASS_ARRAY {
            let mut idx = 0;
            let mut generic_array = false;
            if (*p).fast_array() != 0 {
                if __JS_AtomIsTaggedInt(prop) != 0 {
                    idx = __JS_AtomToUInt32(prop);
                    if idx == (*p).u.array.count {
                        if (*p).extensible() == 0 {
                            return JS_ThrowTypeErrorOrFalse(
                                ctx,
                                flags,
                                c"object is not extensible".as_ptr(),
                            );
                        }
                        if flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) == 0
                            && get_prop_flags(flags, 0) == JS_PROP_C_W_E
                        {
                            return add_fast_array_element(ctx, p, JS_DupValue(ctx, val), flags);
                        }
                    }
                    generic_array = true;
                } else if JS_AtomIsArrayIndex(ctx, &mut idx, prop) != 0 {
                    generic_array = true;
                }
                if generic_array && convert_fast_array_to_array(ctx, p) != 0 {
                    return -1;
                }
            } else if JS_AtomIsArrayIndex(ctx, &mut idx, prop) != 0 {
                generic_array = true;
            }
            if generic_array {
                let plen = (*p).prop;
                let mut len = 0;
                JS_ToUint32(ctx, &mut len, (*plen).u.value);
                if idx.wrapping_add(1) > len {
                    let pslen = get_shape_prop((*p).shape);
                    if (*pslen).flags() as i32 & JS_PROP_WRITABLE == 0 {
                        return JS_ThrowTypeErrorReadOnly(
                            ctx,
                            flags,
                            crate::quickjs_atom::JS_ATOM_length,
                        );
                    }
                    len = idx.wrapping_add(1);
                    set_value(ctx, &mut (*plen).u.value, JS_NewUint32(ctx, len));
                }
            }
        } else if (*p).class_id as u32 >= JS_CLASS_UINT8C_ARRAY
            && (*p).class_id as u32 <= JS_CLASS_FLOAT64_ARRAY
        {
            let ret = JS_AtomIsNumericIndex(ctx, prop);
            if ret != 0 {
                if ret < 0 {
                    return -1;
                }
                return JS_ThrowTypeErrorOrFalse(
                    ctx,
                    flags,
                    c"cannot create numeric index in typed array".as_ptr(),
                );
            }
        } else if flags & JS_PROP_NO_EXOTIC == 0 {
            let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
            if !em.is_null() {
                if let Some(define_own_property) = (*em).define_own_property {
                    return define_own_property(
                        ctx,
                        JS_MKPTR(JS_TAG_OBJECT, p.cast()),
                        prop,
                        val,
                        getter,
                        setter,
                        flags,
                    );
                }
                let ret = JS_IsExtensible(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()));
                if ret < 0 {
                    return -1;
                }
                if ret == 0 {
                    return JS_ThrowTypeErrorOrFalse(
                        ctx,
                        flags,
                        c"object is not extensible".as_ptr(),
                    );
                }
            }
        }
    }
    if (*p).extensible() == 0 {
        return JS_ThrowTypeErrorOrFalse(ctx, flags, c"object is not extensible".as_ptr());
    }
    let mut var_ref: *mut JSVarRef = ptr::null_mut();
    let mut delete_obj: *mut JSObject = ptr::null_mut();
    let mut prop_flags;
    if flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) != 0 {
        prop_flags = flags & (JS_PROP_CONFIGURABLE | JS_PROP_ENUMERABLE) | JS_PROP_GETSET;
    } else {
        prop_flags = flags & JS_PROP_C_W_E;
        if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT {
            let p1 = JS_VALUE_GET_PTR((*p).u.global_object.uninitialized_vars).cast::<JSObject>();
            let mut pr1 = ptr::null_mut();
            let prs1 = find_own_property(&mut pr1, p1, prop);
            if !prs1.is_null() {
                delete_obj = p1;
                var_ref = (*pr1).u.var_ref;
                (*js_rc(var_ref.cast())).ref_count += 1;
            } else {
                var_ref = js_create_var_ref(ctx, 0);
                if var_ref.is_null() {
                    return -1;
                }
            }
            (*var_ref).is_const = (prop_flags & JS_PROP_WRITABLE == 0) as u8;
            prop_flags |= JS_PROP_VARREF;
        }
    }
    let pr = add_property(ctx, p, prop, prop_flags);
    if pr.is_null() {
        if !var_ref.is_null() {
            free_var_ref((*ctx).rt, var_ref);
        }
        return -1;
    }
    if flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) != 0 {
        (*pr).u.getset.getter = ptr::null_mut();
        if flags & JS_PROP_HAS_GET != 0 && JS_IsFunction(ctx, getter) != 0 {
            (*pr).u.getset.getter = JS_VALUE_GET_PTR(JS_DupValue(ctx, getter)).cast();
        }
        (*pr).u.getset.setter = ptr::null_mut();
        if flags & JS_PROP_HAS_SET != 0 && JS_IsFunction(ctx, setter) != 0 {
            (*pr).u.getset.setter = JS_VALUE_GET_PTR(JS_DupValue(ctx, setter)).cast();
        }
    } else if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT {
        if !delete_obj.is_null() {
            delete_property(ctx, delete_obj, prop);
        }
        (*pr).u.var_ref = var_ref;
        *(*var_ref).pvalue = if flags & JS_PROP_HAS_VALUE != 0 {
            JS_DupValue(ctx, val)
        } else {
            JS_UNDEFINED
        };
    } else {
        (*pr).u.value = if flags & JS_PROP_HAS_VALUE != 0 {
            JS_DupValue(ctx, val)
        } else {
            JS_UNDEFINED
        };
    }
    1
}
pub unsafe fn JS_DefineProperty(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    mut val: JSValueConst,
    getter: JSValueConst,
    setter: JSValueConst,
    flags: i32,
) -> i32 {
    if JS_VALUE_GET_TAG(this_obj) != JS_TAG_OBJECT {
        JS_ThrowTypeErrorNotAnObject(ctx);
        return -1;
    }
    let p = JS_VALUE_GET_PTR(this_obj).cast::<JSObject>();
    // The original redo_prop_update label is needed after autoinit and after
    // converting fast-array elements into shape properties.
    'redo_prop_update: loop {
        let mut pr = ptr::null_mut();
        let mut prs = find_own_property(&mut pr, p, prop);
        if !prs.is_null() {
            if (*prs).flags() as i32 & JS_PROP_LENGTH != 0 && flags & JS_PROP_HAS_VALUE != 0 {
                let mut array_length = 0;
                if JS_ToArrayLengthFree(ctx, &mut array_length, JS_DupValue(ctx, val), 0) != 0 {
                    return -1;
                }
                val = JS_NewUint32(ctx, array_length);
                prs = find_own_property(&mut pr, p, prop);
                assert!(!prs.is_null());
            }
            if check_define_prop_flags((*prs).flags() as i32, flags) == 0 {
                return JS_ThrowTypeErrorOrFalse(
                    ctx,
                    flags,
                    c"property is not configurable".as_ptr(),
                );
            }
            if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_AUTOINIT {
                if JS_AutoInitProperty(ctx, p, prop, pr, prs) != 0 {
                    return -1;
                }
                continue 'redo_prop_update;
            }
            if flags
                & (JS_PROP_HAS_VALUE | JS_PROP_HAS_WRITABLE | JS_PROP_HAS_GET | JS_PROP_HAS_SET)
                != 0
            {
                if flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) != 0 {
                    let new_getter = if JS_IsFunction(ctx, getter) != 0 {
                        JS_VALUE_GET_PTR(getter).cast::<JSObject>()
                    } else {
                        ptr::null_mut()
                    };
                    let new_setter = if JS_IsFunction(ctx, setter) != 0 {
                        JS_VALUE_GET_PTR(setter).cast::<JSObject>()
                    } else {
                        ptr::null_mut()
                    };
                    if (*prs).flags() as i32 & JS_PROP_TMASK != JS_PROP_GETSET {
                        if js_shape_prepare_update(ctx, p, &mut prs) != 0 {
                            return -1;
                        }
                        if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_VARREF {
                            if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT
                                && remove_global_object_property(ctx, p, prs, pr) != 0
                            {
                                return -1;
                            }
                            free_var_ref((*ctx).rt, (*pr).u.var_ref);
                        } else {
                            JS_FreeValue(ctx, (*pr).u.value);
                        }
                        (*prs).set_flags(
                            (*prs).flags() & (JS_PROP_CONFIGURABLE | JS_PROP_ENUMERABLE) as u32
                                | JS_PROP_GETSET as u32,
                        );
                        (*pr).u.getset.getter = ptr::null_mut();
                        (*pr).u.getset.setter = ptr::null_mut();
                    } else if (*prs).flags() as i32 & JS_PROP_CONFIGURABLE == 0 {
                        if (flags & JS_PROP_HAS_GET != 0 && new_getter != (*pr).u.getset.getter)
                            || (flags & JS_PROP_HAS_SET != 0 && new_setter != (*pr).u.getset.setter)
                        {
                            return JS_ThrowTypeErrorOrFalse(
                                ctx,
                                flags,
                                c"property is not configurable".as_ptr(),
                            );
                        }
                    }
                    if flags & JS_PROP_HAS_GET != 0 {
                        if !(*pr).u.getset.getter.is_null() {
                            JS_FreeValue(
                                ctx,
                                JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.getter.cast()),
                            );
                        }
                        if !new_getter.is_null() {
                            JS_DupValue(ctx, getter);
                        }
                        (*pr).u.getset.getter = new_getter;
                    }
                    if flags & JS_PROP_HAS_SET != 0 {
                        if !(*pr).u.getset.setter.is_null() {
                            JS_FreeValue(
                                ctx,
                                JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.setter.cast()),
                            );
                        }
                        if !new_setter.is_null() {
                            JS_DupValue(ctx, setter);
                        }
                        (*pr).u.getset.setter = new_setter;
                    }
                } else {
                    if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_GETSET {
                        let var_ref = if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT {
                            let v = js_global_object_find_uninitialized_var(ctx, p, prop, 0);
                            if v.is_null() {
                                return -1;
                            }
                            v
                        } else {
                            ptr::null_mut()
                        };
                        if js_shape_prepare_update(ctx, p, &mut prs) != 0 {
                            if !var_ref.is_null() {
                                free_var_ref((*ctx).rt, var_ref);
                            }
                            return -1;
                        }
                        if !(*pr).u.getset.getter.is_null() {
                            JS_FreeValue(
                                ctx,
                                JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.getter.cast()),
                            );
                        }
                        if !(*pr).u.getset.setter.is_null() {
                            JS_FreeValue(
                                ctx,
                                JS_MKPTR(JS_TAG_OBJECT, (*pr).u.getset.setter.cast()),
                            );
                        }
                        if !var_ref.is_null() {
                            (*prs).set_flags(
                                (*prs).flags() & !(JS_PROP_TMASK as u32)
                                    | (JS_PROP_VARREF | JS_PROP_WRITABLE) as u32,
                            );
                            (*pr).u.var_ref = var_ref;
                        } else {
                            (*prs).set_flags(
                                (*prs).flags() & !((JS_PROP_TMASK | JS_PROP_WRITABLE) as u32),
                            );
                            (*pr).u.value = JS_UNDEFINED;
                        }
                    } else if (*prs).flags() as i32 & JS_PROP_TMASK != JS_PROP_VARREF
                        && (*prs).flags() as i32 & (JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE) == 0
                        && flags & JS_PROP_HAS_VALUE != 0
                    {
                        if js_same_value(ctx, val, (*pr).u.value) == 0 {
                            return JS_ThrowTypeErrorOrFalse(
                                ctx,
                                flags,
                                c"property is not configurable".as_ptr(),
                            );
                        }
                        return 1;
                    }
                    if (*prs).flags() as i32 & JS_PROP_TMASK == JS_PROP_VARREF {
                        if flags & JS_PROP_HAS_VALUE != 0 {
                            if (*p).class_id as u32 == JS_CLASS_MODULE_NS {
                                if js_same_value(ctx, val, *(*(*pr).u.var_ref).pvalue) == 0 {
                                    return JS_ThrowTypeErrorOrFalse(
                                        ctx,
                                        flags,
                                        c"property is not configurable".as_ptr(),
                                    );
                                }
                            } else {
                                set_value(ctx, (*(*pr).u.var_ref).pvalue, JS_DupValue(ctx, val));
                            }
                        }
                        if flags & (JS_PROP_HAS_WRITABLE | JS_PROP_WRITABLE) == JS_PROP_HAS_WRITABLE
                        {
                            if (*p).class_id as u32 == JS_CLASS_MODULE_NS {
                                return JS_ThrowTypeErrorOrFalse(
                                    ctx,
                                    flags,
                                    c"module namespace properties have writable = false".as_ptr(),
                                );
                            }
                            if js_shape_prepare_update(ctx, p, &mut prs) != 0 {
                                return -1;
                            }
                            if (*p).class_id as u32 == JS_CLASS_GLOBAL_OBJECT {
                                (*(*pr).u.var_ref).is_const = 1;
                                (*prs).set_flags((*prs).flags() & !(JS_PROP_WRITABLE as u32));
                            } else {
                                let val1 = JS_DupValue(ctx, *(*(*pr).u.var_ref).pvalue);
                                free_var_ref((*ctx).rt, (*pr).u.var_ref);
                                (*pr).u.value = val1;
                                (*prs).set_flags(
                                    (*prs).flags() & !((JS_PROP_TMASK | JS_PROP_WRITABLE) as u32),
                                );
                            }
                        }
                    } else if (*prs).flags() as i32 & JS_PROP_LENGTH != 0 {
                        let res = if flags & JS_PROP_HAS_VALUE != 0 {
                            set_array_length(ctx, p, JS_DupValue(ctx, val), flags)
                        } else {
                            1
                        };
                        if flags & (JS_PROP_HAS_WRITABLE | JS_PROP_WRITABLE) == JS_PROP_HAS_WRITABLE
                        {
                            prs = get_shape_prop((*p).shape);
                            if js_update_property_flags(
                                ctx,
                                p,
                                &mut prs,
                                (*prs).flags() as i32 & !JS_PROP_WRITABLE,
                            ) != 0
                            {
                                return -1;
                            }
                        }
                        return res;
                    } else {
                        if flags & JS_PROP_HAS_VALUE != 0 {
                            JS_FreeValue(ctx, (*pr).u.value);
                            (*pr).u.value = JS_DupValue(ctx, val);
                        }
                        if flags & JS_PROP_HAS_WRITABLE != 0
                            && js_update_property_flags(
                                ctx,
                                p,
                                &mut prs,
                                (*prs).flags() as i32 & !JS_PROP_WRITABLE
                                    | flags & JS_PROP_WRITABLE,
                            ) != 0
                        {
                            return -1;
                        }
                    }
                }
            }
            let mut mask = 0;
            if flags & JS_PROP_HAS_CONFIGURABLE != 0 {
                mask |= JS_PROP_CONFIGURABLE;
            }
            if flags & JS_PROP_HAS_ENUMERABLE != 0 {
                mask |= JS_PROP_ENUMERABLE;
            }
            if js_update_property_flags(
                ctx,
                p,
                &mut prs,
                (*prs).flags() as i32 & !mask | flags & mask,
            ) != 0
            {
                return -1;
            }
            return 1;
        }
        if (*p).fast_array() != 0 {
            if (*p).class_id as u32 == JS_CLASS_ARRAY {
                if __JS_AtomIsTaggedInt(prop) != 0 {
                    let idx = __JS_AtomToUInt32(prop);
                    if idx < (*p).u.array.count {
                        if get_prop_flags(flags, JS_PROP_C_W_E) != JS_PROP_C_W_E
                            || flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) != 0
                        {
                            if convert_fast_array_to_array(ctx, p) != 0 {
                                return -1;
                            }
                            continue 'redo_prop_update;
                        }
                        if flags & JS_PROP_HAS_VALUE != 0 {
                            set_value(
                                ctx,
                                (*p).u.array.u.values.add(idx as usize),
                                JS_DupValue(ctx, val),
                            );
                        }
                        return 1;
                    }
                }
            } else if (*p).class_id as u32 >= JS_CLASS_UINT8C_ARRAY
                && (*p).class_id as u32 <= JS_CLASS_FLOAT64_ARRAY
            {
                let mut numeric = true;
                let mut out_of_bounds = false;
                if __JS_AtomIsTaggedInt(prop) == 0 {
                    let num = JS_AtomIsNumericIndex1(ctx, prop);
                    if JS_IsUndefined(num) != 0 {
                        numeric = false;
                    } else {
                        if JS_IsException(num) != 0 {
                            return -1;
                        }
                        let ret = JS_NumberIsInteger(ctx, num);
                        if ret < 0 {
                            JS_FreeValue(ctx, num);
                            return -1;
                        }
                        if ret == 0 {
                            JS_FreeValue(ctx, num);
                            return JS_ThrowTypeErrorOrFalse(
                                ctx,
                                flags,
                                c"non integer index in typed array".as_ptr(),
                            );
                        }
                        let ret = JS_NumberIsNegativeOrMinusZero(ctx, num);
                        JS_FreeValue(ctx, num);
                        if ret != 0 {
                            return JS_ThrowTypeErrorOrFalse(
                                ctx,
                                flags,
                                c"negative index in typed array".as_ptr(),
                            );
                        }
                        if __JS_AtomIsTaggedInt(prop) == 0 {
                            out_of_bounds = true;
                        }
                    }
                }
                if numeric {
                    let idx = __JS_AtomToUInt32(prop);
                    if out_of_bounds || idx >= (*p).u.array.count {
                        return JS_ThrowTypeErrorOrFalse(
                            ctx,
                            flags,
                            c"out-of-bound index in typed array".as_ptr(),
                        );
                    }
                    if flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) != 0
                        || get_prop_flags(flags, JS_PROP_C_W_E) != JS_PROP_C_W_E
                    {
                        return JS_ThrowTypeErrorOrFalse(
                            ctx,
                            flags,
                            c"invalid descriptor flags".as_ptr(),
                        );
                    }
                    if flags & JS_PROP_HAS_VALUE != 0 {
                        return JS_SetPropertyValue(
                            ctx,
                            this_obj,
                            JS_NewInt32(ctx, idx as i32),
                            JS_DupValue(ctx, val),
                            flags,
                        );
                    }
                    return 1;
                }
            }
        }
        return JS_CreateProperty(ctx, p, prop, val, getter, setter, flags);
    }
}
unsafe fn JS_DefineAutoInitProperty(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    id: JSAutoInitIDEnum,
    opaque: *mut c_void,
    flags: i32,
) -> i32 {
    if JS_VALUE_GET_TAG(this_obj) != JS_TAG_OBJECT {
        return 0;
    }
    let p = JS_VALUE_GET_PTR(this_obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    if !find_own_property(&mut pr, p, prop).is_null() {
        std::process::abort();
    }
    pr = add_property(ctx, p, prop, flags & JS_PROP_C_W_E | JS_PROP_AUTOINIT);
    if pr.is_null() {
        return -1;
    }
    (*pr).u.init.realm_and_id = JS_DupContext(ctx) as usize;
    assert_eq!((*pr).u.init.realm_and_id & 3, 0);
    assert!(id <= 3);
    (*pr).u.init.realm_and_id |= id as usize;
    (*pr).u.init.opaque = opaque;
    1
}
pub unsafe fn JS_DefinePropertyValue(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    val: JSValue,
    flags: i32,
) -> i32 {
    let ret = JS_DefineProperty(
        ctx,
        this_obj,
        prop,
        val,
        JS_UNDEFINED,
        JS_UNDEFINED,
        flags
            | JS_PROP_HAS_VALUE
            | JS_PROP_HAS_CONFIGURABLE
            | JS_PROP_HAS_WRITABLE
            | JS_PROP_HAS_ENUMERABLE,
    );
    JS_FreeValue(ctx, val);
    ret
}
pub unsafe fn JS_DefinePropertyValueValue(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSValue,
    val: JSValue,
    flags: i32,
) -> i32 {
    let atom = JS_ValueToAtom(ctx, prop);
    JS_FreeValue(ctx, prop);
    if atom == JS_ATOM_NULL as u32 {
        JS_FreeValue(ctx, val);
        return -1;
    }
    let ret = JS_DefinePropertyValue(ctx, this_obj, atom, val, flags);
    JS_FreeAtom(ctx, atom);
    ret
}
pub unsafe fn JS_DefinePropertyValueUint32(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    idx: u32,
    val: JSValue,
    flags: i32,
) -> i32 {
    JS_DefinePropertyValueValue(ctx, this_obj, JS_NewUint32(ctx, idx), val, flags)
}
pub unsafe fn JS_DefinePropertyValueInt64(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    idx: i64,
    val: JSValue,
    flags: i32,
) -> i32 {
    JS_DefinePropertyValueValue(ctx, this_obj, JS_NewInt64(ctx, idx), val, flags)
}
pub unsafe fn JS_DefinePropertyValueStr(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: *const c_char,
    val: JSValue,
    flags: i32,
) -> i32 {
    let atom = JS_NewAtom(ctx, prop);
    if atom == JS_ATOM_NULL as u32 {
        JS_FreeValue(ctx, val);
        return -1;
    }
    let ret = JS_DefinePropertyValue(ctx, this_obj, atom, val, flags);
    JS_FreeAtom(ctx, atom);
    ret
}
pub unsafe fn JS_DefinePropertyGetSet(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    prop: JSAtom,
    getter: JSValue,
    setter: JSValue,
    flags: i32,
) -> i32 {
    let ret = JS_DefineProperty(
        ctx,
        this_obj,
        prop,
        JS_UNDEFINED,
        getter,
        setter,
        flags
            | JS_PROP_HAS_GET
            | JS_PROP_HAS_SET
            | JS_PROP_HAS_CONFIGURABLE
            | JS_PROP_HAS_ENUMERABLE,
    );
    JS_FreeValue(ctx, getter);
    JS_FreeValue(ctx, setter);
    ret
}
unsafe fn JS_CreateDataPropertyUint32(
    ctx: *mut JSContext,
    this_obj: JSValueConst,
    idx: i64,
    val: JSValue,
    flags: i32,
) -> i32 {
    JS_DefinePropertyValueValue(
        ctx,
        this_obj,
        JS_NewInt64(ctx, idx),
        val,
        flags | JS_PROP_C_W_E,
    )
}
