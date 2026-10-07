// quickjs.c:5853..5861,5919..5942,5944..5990,6047..6074,
// 17367..17389,39402..39609,39659..39764. MIT.
unsafe fn js_function_set_properties(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    name: JSAtom,
    len: i32,
) {
    JS_DefinePropertyValue(
        ctx,
        func_obj,
        crate::quickjs_atom::JS_ATOM_length,
        JS_NewInt32(ctx, len),
        JS_PROP_CONFIGURABLE,
    );
    JS_DefinePropertyValue(
        ctx,
        func_obj,
        crate::quickjs_atom::JS_ATOM_name,
        JS_AtomToString(ctx, name),
        JS_PROP_CONFIGURABLE,
    );
}
unsafe fn js_get_function_name(ctx: *mut JSContext, name: JSAtom) -> JSValue {
    let mut name_str = JS_AtomToString(ctx, name);
    if JS_AtomSymbolHasDescription(ctx, name) != 0 {
        name_str = JS_ConcatString3(ctx, c"[".as_ptr(), name_str, c"]".as_ptr());
    }
    name_str
}
unsafe fn js_method_set_properties(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    name: JSAtom,
    flags: i32,
    home_obj: JSValueConst,
) -> i32 {
    let mut name_str = js_get_function_name(ctx, name);
    if flags & JS_PROP_HAS_GET != 0 {
        name_str = JS_ConcatString3(ctx, c"get ".as_ptr(), name_str, c"".as_ptr());
    } else if flags & JS_PROP_HAS_SET != 0 {
        name_str = JS_ConcatString3(ctx, c"set ".as_ptr(), name_str, c"".as_ptr());
    }
    if JS_IsException(name_str) != 0 {
        return -1;
    }
    if JS_DefinePropertyValue(
        ctx,
        func_obj,
        crate::quickjs_atom::JS_ATOM_name,
        name_str,
        JS_PROP_CONFIGURABLE,
    ) < 0
    {
        return -1;
    }
    js_method_set_home_object(ctx, func_obj, home_obj);
    0
}
unsafe fn JS_NewCFunction3(
    ctx: *mut JSContext,
    func: Option<JSCFunction>,
    name: *const c_char,
    length: i32,
    cproto: JSCFunctionEnum,
    magic: i32,
    proto_val: JSValueConst,
    n_fields: i32,
) -> JSValue {
    let func_obj = if n_fields > 0 {
        JS_NewObjectProtoClassAlloc(ctx, proto_val, JS_CLASS_C_FUNCTION, n_fields)
    } else {
        JS_NewObjectProtoClass(ctx, proto_val, JS_CLASS_C_FUNCTION)
    };
    if JS_IsException(func_obj) != 0 {
        return func_obj;
    }
    let p = JS_VALUE_GET_PTR(func_obj).cast::<JSObject>();
    (*p).u.cfunc.realm = JS_DupContext(ctx);
    (*p).u.cfunc.c_function.generic = func;
    (*p).u.cfunc.length = length as u8;
    (*p).u.cfunc.cproto = cproto as u8;
    (*p).u.cfunc.magic = magic as i16;
    (*p).set_is_constructor(
        (cproto == JS_CFUNC_constructor
            || cproto == JS_CFUNC_constructor_magic
            || cproto == JS_CFUNC_constructor_or_func
            || cproto == JS_CFUNC_constructor_or_func_magic) as u8,
    );
    let name = if name.is_null() { c"".as_ptr() } else { name };
    let name_atom = JS_NewAtom(ctx, name);
    if name_atom == JS_ATOM_NULL as u32 {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    js_function_set_properties(ctx, func_obj, name_atom, length);
    JS_FreeAtom(ctx, name_atom);
    func_obj
}
pub unsafe fn JS_NewCFunction2(
    ctx: *mut JSContext,
    func: Option<JSCFunction>,
    name: *const c_char,
    length: i32,
    cproto: JSCFunctionEnum,
    magic: i32,
) -> JSValue {
    JS_NewCFunction3(
        ctx,
        func,
        name,
        length,
        cproto,
        magic,
        (*ctx).function_proto,
        0,
    )
}
// quickjs.h inline wrapper, deliberately uses the same function-pointer slot.
pub unsafe fn JS_NewCFunction(
    ctx: *mut JSContext,
    func: Option<JSCFunction>,
    name: *const c_char,
    length: i32,
) -> JSValue {
    JS_NewCFunction2(ctx, func, name, length, JS_CFUNC_generic, 0)
}
pub unsafe fn JS_NewCFunctionData(
    ctx: *mut JSContext,
    func: Option<JSCFunctionData>,
    length: i32,
    magic: i32,
    data_len: i32,
    data: *mut JSValueConst,
) -> JSValue {
    let func_obj = JS_NewObjectProtoClass(ctx, (*ctx).function_proto, JS_CLASS_C_FUNCTION_DATA);
    if JS_IsException(func_obj) != 0 {
        return func_obj;
    }
    let s = js_malloc(
        ctx,
        size_of::<JSCFunctionDataRecord>()
            .wrapping_add((data_len as usize).wrapping_mul(size_of::<JSValue>())),
    )
    .cast::<JSCFunctionDataRecord>();
    if s.is_null() {
        JS_FreeValue(ctx, func_obj);
        return JS_EXCEPTION;
    }
    (*s).func = func;
    (*s).length = length as u8;
    (*s).data_len = data_len as u8;
    (*s).magic = magic as u16;
    for i in 0..data_len {
        *ptr::addr_of_mut!((*s).data)
            .cast::<JSValue>()
            .add(i as usize) = JS_DupValue(ctx, *data.add(i as usize));
    }
    JS_SetOpaque(func_obj, s.cast());
    js_function_set_properties(ctx, func_obj, JS_ATOM_empty_string, length);
    func_obj
}
unsafe fn check_function(ctx: *mut JSContext, obj: JSValueConst) -> i32 {
    if JS_IsFunction(ctx, obj) != 0 {
        return 0;
    }
    JS_ThrowTypeError(ctx, c"not a function".as_ptr());
    -1
}
unsafe fn check_exception_free(ctx: *mut JSContext, obj: JSValue) -> i32 {
    JS_FreeValue(ctx, obj);
    JS_IsException(obj)
}
unsafe fn find_atom(ctx: *mut JSContext, name: *const c_char) -> JSAtom {
    if *name == b'[' as c_char {
        let name = name.add(1);
        let len = std::ffi::CStr::from_ptr(name)
            .to_bytes()
            .len()
            .wrapping_sub(1);
        let mut atom = crate::quickjs_atom::JS_ATOM_Symbol_toPrimitive;
        while atom < crate::quickjs_atom::JS_ATOM_END {
            let p = *(*(*ctx).rt).atom_array.add(atom as usize);
            if (*p).len() as usize == len
                && core::slice::from_raw_parts(ptr::addr_of!((*p).u.str8).cast::<u8>(), len)
                    == core::slice::from_raw_parts(name.cast::<u8>(), len)
            {
                return JS_DupAtom(ctx, atom);
            }
            atom += 1;
        }
        std::process::abort();
    }
    JS_NewAtom(ctx, name)
}
unsafe fn JS_NewObjectProtoList(
    ctx: *mut JSContext,
    proto: JSValueConst,
    fields: *const JSCFunctionListEntry,
    n_fields: i32,
) -> JSValue {
    let obj = JS_NewObjectProtoClassAlloc(ctx, proto, JS_CLASS_OBJECT, n_fields);
    if JS_IsException(obj) != 0 {
        return obj;
    }
    if JS_SetPropertyFunctionList(ctx, obj, fields, n_fields) != 0 {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    obj
}
unsafe fn JS_InstantiateFunctionListItem2(
    ctx: *mut JSContext,
    _p: *mut JSObject,
    atom: JSAtom,
    opaque: *mut c_void,
) -> JSValue {
    let e = opaque.cast::<JSCFunctionListEntry>();
    match (*e).def_type as i32 {
        JS_DEF_CFUNC => JS_NewCFunction2(
            ctx,
            (*e).u.func.cfunc.generic,
            (*e).name,
            (*e).u.func.length as i32,
            (*e).u.func.cproto as i32,
            (*e).magic as i32,
        ),
        JS_DEF_PROP_STRING => JS_NewAtomString(ctx, (*e).u.str),
        JS_DEF_OBJECT => {
            let proto = if atom == crate::quickjs_atom::JS_ATOM_Symbol_unscopables {
                JS_NULL
            } else {
                *(*ctx).class_proto.add(JS_CLASS_OBJECT as usize)
            };
            JS_NewObjectProtoList(ctx, proto, (*e).u.prop_list.tab, (*e).u.prop_list.len)
        }
        _ => std::process::abort(),
    }
}
// snprintf("get/set %s") has a 64-byte buffer in the original.
unsafe fn function_accessor_name(prefix: &[u8; 4], name: *const c_char) -> [c_char; 64] {
    let mut buf = [0; 64];
    for (i, c) in prefix.iter().enumerate() {
        buf[i] = *c as c_char;
    }
    let n = std::ffi::CStr::from_ptr(name).to_bytes();
    for (i, c) in n.iter().take(59).enumerate() {
        buf[i + 4] = *c as c_char;
    }
    buf
}
unsafe fn JS_InstantiateFunctionListItem(
    ctx: *mut JSContext,
    obj: JSValueConst,
    atom: JSAtom,
    e: *const JSCFunctionListEntry,
) -> i32 {
    let mut prop_flags = (*e).prop_flags as i32;
    let val = match (*e).def_type as i32 {
        JS_DEF_ALIAS => {
            let atom1 = find_atom(ctx, (*e).u.alias.name);
            let base = match (*e).u.alias.base {
                -1 => obj,
                0 => (*ctx).global_obj,
                1 => *(*ctx).class_proto.add(JS_CLASS_ARRAY as usize),
                _ => std::process::abort(),
            };
            let val = JS_GetProperty(ctx, base, atom1);
            JS_FreeAtom(ctx, atom1);
            if JS_IsException(val) != 0 {
                return -1;
            }
            if atom == crate::quickjs_atom::JS_ATOM_Symbol_toPrimitive {
                prop_flags = JS_PROP_CONFIGURABLE;
            } else if atom == crate::quickjs_atom::JS_ATOM_Symbol_hasInstance {
                prop_flags = 0;
            }
            val
        }
        JS_DEF_CFUNC => {
            if atom == crate::quickjs_atom::JS_ATOM_Symbol_toPrimitive {
                prop_flags = JS_PROP_CONFIGURABLE;
            } else if atom == crate::quickjs_atom::JS_ATOM_Symbol_hasInstance {
                prop_flags = 0;
            }
            if JS_DefineAutoInitProperty(
                ctx,
                obj,
                atom,
                JS_AUTOINIT_ID_PROP,
                e.cast_mut().cast(),
                prop_flags,
            ) < 0
            {
                return -1;
            }
            return 0;
        }
        JS_DEF_CGETSET | JS_DEF_CGETSET_MAGIC => {
            let mut getter = JS_UNDEFINED;
            if (*e).u.getset.get.generic.is_some() {
                let buf = function_accessor_name(b"get ", (*e).name);
                getter = JS_NewCFunction2(
                    ctx,
                    (*e).u.getset.get.generic,
                    buf.as_ptr(),
                    0,
                    if (*e).def_type as i32 == JS_DEF_CGETSET_MAGIC {
                        JS_CFUNC_getter_magic
                    } else {
                        JS_CFUNC_getter
                    },
                    (*e).magic as i32,
                );
                if JS_IsException(getter) != 0 {
                    return -1;
                }
            }
            let mut setter = JS_UNDEFINED;
            if (*e).u.getset.set.generic.is_some() {
                let buf = function_accessor_name(b"set ", (*e).name);
                setter = JS_NewCFunction2(
                    ctx,
                    (*e).u.getset.set.generic,
                    buf.as_ptr(),
                    1,
                    if (*e).def_type as i32 == JS_DEF_CGETSET_MAGIC {
                        JS_CFUNC_setter_magic
                    } else {
                        JS_CFUNC_setter
                    },
                    (*e).magic as i32,
                );
                if JS_IsException(setter) != 0 {
                    JS_FreeValue(ctx, getter);
                    return -1;
                }
            }
            if JS_DefinePropertyGetSet(ctx, obj, atom, getter, setter, prop_flags) < 0 {
                return -1;
            }
            return 0;
        }
        JS_DEF_PROP_INT32 => JS_NewInt32(ctx, (*e).u.i32),
        JS_DEF_PROP_INT64 => JS_NewInt64(ctx, (*e).u.i64),
        JS_DEF_PROP_DOUBLE => __JS_NewFloat64(ctx, (*e).u.f64),
        JS_DEF_PROP_UNDEFINED => JS_UNDEFINED,
        JS_DEF_PROP_ATOM => JS_AtomToValue(ctx, (*e).u.i32 as JSAtom),
        JS_DEF_PROP_BOOL => JS_NewBool(ctx, (*e).u.i32),
        JS_DEF_PROP_STRING | JS_DEF_OBJECT => {
            if JS_DefineAutoInitProperty(
                ctx,
                obj,
                atom,
                JS_AUTOINIT_ID_PROP,
                e.cast_mut().cast(),
                prop_flags,
            ) < 0
            {
                return -1;
            }
            return 0;
        }
        _ => std::process::abort(),
    };
    if JS_DefinePropertyValue(ctx, obj, atom, val, prop_flags) < 0 {
        return -1;
    }
    0
}
pub unsafe fn JS_SetPropertyFunctionList(
    ctx: *mut JSContext,
    obj: JSValueConst,
    tab: *const JSCFunctionListEntry,
    len: i32,
) -> i32 {
    for i in 0..len {
        let e = tab.add(i as usize);
        let atom = find_atom(ctx, (*e).name);
        if atom == JS_ATOM_NULL as u32 {
            return -1;
        }
        let ret = JS_InstantiateFunctionListItem(ctx, obj, atom, e);
        JS_FreeAtom(ctx, atom);
        if ret != 0 {
            return -1;
        }
    }
    0
}
unsafe fn JS_SetConstructor2(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    proto: JSValueConst,
    proto_flags: i32,
    ctor_flags: i32,
) -> i32 {
    if JS_DefinePropertyValue(
        ctx,
        func_obj,
        crate::quickjs_atom::JS_ATOM_prototype,
        JS_DupValue(ctx, proto),
        proto_flags,
    ) < 0
    {
        return -1;
    }
    if JS_DefinePropertyValue(
        ctx,
        proto,
        crate::quickjs_atom::JS_ATOM_constructor,
        JS_DupValue(ctx, func_obj),
        ctor_flags,
    ) < 0
    {
        return -1;
    }
    set_cycle_flag(ctx, func_obj);
    set_cycle_flag(ctx, proto);
    0
}
pub unsafe fn JS_SetConstructor(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    proto: JSValueConst,
) -> i32 {
    JS_SetConstructor2(
        ctx,
        func_obj,
        proto,
        0,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
    )
}
const JS_NEW_CTOR_NO_GLOBAL: i32 = 1 << 0;
const JS_NEW_CTOR_PROTO_CLASS: i32 = 1 << 1;
const JS_NEW_CTOR_PROTO_EXIST: i32 = 1 << 2;
const JS_NEW_CTOR_READONLY: i32 = 1 << 3;
unsafe fn JS_NewCConstructor(
    ctx: *mut JSContext,
    class_id: i32,
    name: *const c_char,
    func: Option<JSCFunction>,
    length: i32,
    cproto: JSCFunctionEnum,
    magic: i32,
    mut parent_ctor: JSValueConst,
    ctor_fields: *const JSCFunctionListEntry,
    n_ctor_fields: i32,
    proto_fields: *const JSCFunctionListEntry,
    n_proto_fields: i32,
    flags: i32,
) -> JSValue {
    let mut ctor = JS_UNDEFINED;
    let ctor_flags = if flags & JS_NEW_CTOR_READONLY != 0 {
        JS_PROP_CONFIGURABLE
    } else {
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE
    };
    let parent_proto = if JS_IsUndefined(parent_ctor) != 0 {
        parent_ctor = (*ctx).function_proto;
        JS_DupValue(ctx, *(*ctx).class_proto.add(JS_CLASS_OBJECT as usize))
    } else {
        let v = JS_GetProperty(ctx, parent_ctor, crate::quickjs_atom::JS_ATOM_prototype);
        if JS_IsException(v) != 0 {
            return JS_EXCEPTION;
        }
        v
    };
    let proto = if flags & JS_NEW_CTOR_PROTO_EXIST != 0 {
        JS_DupValue(ctx, *(*ctx).class_proto.add(class_id as usize))
    } else {
        let proto_class_id = if flags & JS_NEW_CTOR_PROTO_CLASS != 0 {
            class_id as JSClassID
        } else {
            JS_CLASS_OBJECT
        };
        let v = JS_NewObjectProtoClassAlloc(ctx, parent_proto, proto_class_id, n_proto_fields + 1);
        if JS_IsException(v) == 0 && class_id >= 0 {
            *(*ctx).class_proto.add(class_id as usize) = JS_DupValue(ctx, v);
        }
        v
    };
    let failed = JS_IsException(proto) != 0
        || JS_SetPropertyFunctionList(ctx, proto, proto_fields, n_proto_fields) != 0;
    if !failed {
        ctor = JS_NewCFunction3(
            ctx,
            func,
            name,
            length,
            cproto,
            magic,
            parent_ctor,
            n_ctor_fields + 3,
        );
    }
    let failed = failed
        || JS_IsException(ctor) != 0
        || JS_SetPropertyFunctionList(ctx, ctor, ctor_fields, n_ctor_fields) != 0;
    let failed = failed
        || (flags & JS_NEW_CTOR_NO_GLOBAL == 0
            && JS_DefinePropertyValueStr(
                ctx,
                (*ctx).global_obj,
                name,
                JS_DupValue(ctx, ctor),
                JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
            ) < 0);
    if !failed {
        // The original ignores this final descriptor return value.
        JS_SetConstructor2(ctx, ctor, proto, 0, ctor_flags);
    }
    JS_FreeValue(ctx, proto);
    JS_FreeValue(ctx, parent_proto);
    if failed {
        JS_FreeValue(ctx, ctor);
        JS_EXCEPTION
    } else {
        ctor
    }
}
