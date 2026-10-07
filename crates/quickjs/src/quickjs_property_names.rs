// quickjs.c:8552-8813. MIT. Property-key enumeration in source order.
unsafe fn num_keys_cmp(p1: *const c_void, p2: *const c_void, opaque: *mut c_void) -> i32 {
    let ctx = opaque.cast();
    let atom1 = (*p1.cast::<JSPropertyEnum>()).atom;
    let atom2 = (*p2.cast::<JSPropertyEnum>()).atom;
    let mut v1 = 0;
    let mut v2 = 0;
    let a1 = JS_AtomIsArrayIndex(ctx, &mut v1, atom1);
    let a2 = JS_AtomIsArrayIndex(ctx, &mut v2, atom2);
    assert!(a1 != 0 && a2 != 0);
    if v1 < v2 {
        -1
    } else if v1 == v2 {
        0
    } else {
        1
    }
}
pub unsafe fn JS_FreePropertyEnum(ctx: *mut JSContext, tab: *mut JSPropertyEnum, len: u32) {
    if !tab.is_null() {
        for i in 0..len {
            JS_FreeAtom(ctx, (*tab.add(i as usize)).atom);
        }
        js_free(ctx, tab.cast());
    }
}
unsafe fn JS_GetOwnPropertyNamesInternal(
    ctx: *mut JSContext,
    ptab: *mut *mut JSPropertyEnum,
    plen: *mut u32,
    p: *mut JSObject,
    flags: i32,
) -> i32 {
    *ptab = ptr::null_mut();
    *plen = 0;
    let mut num_keys_count = 0u32;
    let mut str_keys_count = 0u32;
    let mut sym_keys_count = 0u32;
    let mut exotic_keys_count = 0u32;
    let mut exotic_count = 0u32;
    let mut tab_exotic: *mut JSPropertyEnum = ptr::null_mut();
    let sh = (*p).shape;
    for i in 0..(*sh).prop_count as usize {
        let prs = get_shape_prop(sh).add(i);
        let atom = (*prs).atom;
        if atom == crate::quickjs_atom::JS_ATOM_NULL {
            continue;
        }
        let pf = (*prs).flags() as i32;
        let enumerable = pf & JS_PROP_ENUMERABLE != 0;
        let kind = JS_AtomGetKind(ctx, atom);
        if (flags & JS_GPN_ENUM_ONLY == 0 || enumerable) && (flags >> kind) & 1 != 0 {
            if pf & JS_PROP_TMASK == JS_PROP_VARREF
                && flags & (JS_GPN_SET_ENUM | JS_GPN_ENUM_ONLY) != 0
            {
                let var_ref = (*(*p).prop.add(i)).u.var_ref;
                if JS_IsUninitialized(*(*var_ref).pvalue) != 0 {
                    JS_ThrowReferenceErrorUninitialized(ctx, (*prs).atom);
                    return -1;
                }
            }
            let mut key = 0;
            if JS_AtomIsArrayIndex(ctx, &mut key, atom) != 0 {
                num_keys_count = num_keys_count.wrapping_add(1);
            } else if kind == JS_ATOM_KIND_STRING {
                str_keys_count = str_keys_count.wrapping_add(1);
            } else {
                sym_keys_count = sym_keys_count.wrapping_add(1);
            }
        }
    }
    if (*p).is_exotic() != 0 {
        if (*p).fast_array() != 0 {
            if flags & JS_GPN_STRING_MASK != 0 {
                num_keys_count = num_keys_count.wrapping_add((*p).u.array.count);
            }
        } else if (*p).class_id as u32 == JS_CLASS_STRING {
            if flags & JS_GPN_STRING_MASK != 0 {
                num_keys_count = num_keys_count.wrapping_add(js_string_obj_get_length(
                    ctx,
                    JS_MKPTR(JS_TAG_OBJECT, p.cast()),
                ));
            }
        } else {
            let em = (*(*(*ctx).rt).class_array.add((*p).class_id as usize)).exotic;
            if !em.is_null() {
                if let Some(get_names) = (*em).get_own_property_names {
                    if get_names(
                        ctx,
                        &mut tab_exotic,
                        &mut exotic_count,
                        JS_MKPTR(JS_TAG_OBJECT, p.cast()),
                    ) != 0
                    {
                        return -1;
                    }
                    for i in 0..exotic_count as usize {
                        let atom = (*tab_exotic.add(i)).atom;
                        let kind = JS_AtomGetKind(ctx, atom);
                        if (flags >> kind) & 1 != 0 {
                            let mut enumerable = false;
                            if flags & (JS_GPN_SET_ENUM | JS_GPN_ENUM_ONLY) != 0 {
                                let mut desc: JSPropertyDescriptor = core::mem::zeroed();
                                let res = JS_GetOwnPropertyInternal(ctx, &mut desc, p, atom);
                                if res < 0 {
                                    JS_FreePropertyEnum(ctx, tab_exotic, exotic_count);
                                    return -1;
                                }
                                if res != 0 {
                                    enumerable = desc.flags & JS_PROP_ENUMERABLE != 0;
                                    js_free_desc(ctx, &mut desc);
                                }
                                (*tab_exotic.add(i)).is_enumerable = enumerable as i32;
                            }
                            if flags & JS_GPN_ENUM_ONLY == 0 || enumerable {
                                exotic_keys_count = exotic_keys_count.wrapping_add(1);
                            }
                        }
                    }
                }
            }
        }
    }
    macro_rules! add_overflow {
        () => {{
            JS_ThrowOutOfMemory(ctx);
            JS_FreePropertyEnum(ctx, tab_exotic, exotic_count);
            return -1;
        }};
    }
    let mut atom_count = num_keys_count.wrapping_add(str_keys_count);
    if atom_count < str_keys_count {
        add_overflow!();
    }
    atom_count = atom_count.wrapping_add(sym_keys_count);
    if atom_count < sym_keys_count {
        add_overflow!();
    }
    atom_count = atom_count.wrapping_add(exotic_keys_count);
    if atom_count < exotic_keys_count || atom_count > i32::MAX as u32 {
        add_overflow!();
    }
    let tab_atom = js_malloc(
        ctx,
        size_of::<JSPropertyEnum>() * atom_count.max(1) as usize,
    )
    .cast::<JSPropertyEnum>();
    if tab_atom.is_null() {
        JS_FreePropertyEnum(ctx, tab_exotic, exotic_count);
        return -1;
    }
    let mut num_index = 0u32;
    let mut str_index = num_keys_count;
    let mut sym_index = str_index + str_keys_count;
    let mut num_sorted = true;
    let sh = (*p).shape;
    for i in 0..(*sh).prop_count as usize {
        let prs = get_shape_prop(sh).add(i);
        let atom = (*prs).atom;
        if atom == crate::quickjs_atom::JS_ATOM_NULL {
            continue;
        }
        let enumerable = (*prs).flags() as i32 & JS_PROP_ENUMERABLE != 0;
        let kind = JS_AtomGetKind(ctx, atom);
        if (flags & JS_GPN_ENUM_ONLY == 0 || enumerable) && (flags >> kind) & 1 != 0 {
            let mut key = 0;
            let j = if JS_AtomIsArrayIndex(ctx, &mut key, atom) != 0 {
                let j = num_index;
                num_index += 1;
                num_sorted = false;
                j
            } else if kind == JS_ATOM_KIND_STRING {
                let j = str_index;
                str_index += 1;
                j
            } else {
                let j = sym_index;
                sym_index += 1;
                j
            };
            (*tab_atom.add(j as usize)).atom = JS_DupAtom(ctx, atom);
            (*tab_atom.add(j as usize)).is_enumerable = enumerable as i32;
        }
    }
    if (*p).is_exotic() != 0 {
        if (*p).fast_array() != 0 || (*p).class_id as u32 == JS_CLASS_STRING {
            if flags & JS_GPN_STRING_MASK != 0 {
                let len = if (*p).fast_array() != 0 {
                    (*p).u.array.count
                } else {
                    js_string_obj_get_length(ctx, JS_MKPTR(JS_TAG_OBJECT, p.cast()))
                };
                for i in 0..len {
                    (*tab_atom.add(num_index as usize)).atom = __JS_AtomFromUInt32(i);
                    if (*tab_atom.add(num_index as usize)).atom == crate::quickjs_atom::JS_ATOM_NULL
                    {
                        JS_FreePropertyEnum(ctx, tab_atom, num_index);
                        return -1;
                    }
                    (*tab_atom.add(num_index as usize)).is_enumerable = 1;
                    num_index += 1;
                }
            }
        } else {
            // Exotic keys follow ordinary keys without reordering.
            for i in 0..exotic_count as usize {
                let atom = (*tab_exotic.add(i)).atom;
                let enumerable = (*tab_exotic.add(i)).is_enumerable;
                let kind = JS_AtomGetKind(ctx, atom);
                if (flags & JS_GPN_ENUM_ONLY == 0 || enumerable != 0) && (flags >> kind) & 1 != 0 {
                    (*tab_atom.add(sym_index as usize)).atom = atom;
                    (*tab_atom.add(sym_index as usize)).is_enumerable = enumerable;
                    sym_index += 1;
                } else {
                    JS_FreeAtom(ctx, atom);
                }
            }
            js_free(ctx, tab_exotic.cast());
        }
    }
    assert_eq!(num_index, num_keys_count);
    assert_eq!(str_index, num_keys_count + str_keys_count);
    assert_eq!(sym_index, atom_count);
    if num_keys_count != 0 && !num_sorted {
        crate::cutils::rqsort(
            tab_atom.cast(),
            num_keys_count as usize,
            size_of::<JSPropertyEnum>(),
            num_keys_cmp,
            ctx.cast(),
        );
    }
    *ptab = tab_atom;
    *plen = atom_count;
    0
}
pub unsafe fn JS_GetOwnPropertyNames(
    ctx: *mut JSContext,
    ptab: *mut *mut JSPropertyEnum,
    plen: *mut u32,
    obj: JSValueConst,
    flags: i32,
) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        JS_ThrowTypeErrorNotAnObject(ctx);
        return -1;
    }
    JS_GetOwnPropertyNamesInternal(ctx, ptab, plen, JS_VALUE_GET_PTR(obj).cast(), flags)
}
