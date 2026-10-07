// quickjs.c inferred function/class names and class construction. MIT.
// c: quickjs.c:10758
unsafe fn js_object_has_name(_ctx: *mut JSContext, obj: JSValueConst) -> JS_BOOL {
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, JS_VALUE_GET_OBJ(obj), JS_ATOM_name);
    if prs.is_null() {
        return 0;
    }
    if (*prs).flags() as i32 & JS_PROP_TMASK != JS_PROP_NORMAL {
        return 1;
    }
    let val = (*pr).u.value;
    if JS_VALUE_GET_TAG(val) != JS_TAG_STRING {
        return 1;
    }
    let p = JS_VALUE_GET_PTR(val).cast::<JSString>();
    ((*p).len() != 0) as i32
}
// c: quickjs.c:10777
unsafe fn JS_DefineObjectName(
    ctx: *mut JSContext,
    obj: JSValueConst,
    name: JSAtom,
    flags: i32,
) -> i32 {
    if name != JS_ATOM_NULL as u32
        && JS_IsObject(obj) != 0
        && js_object_has_name(ctx, obj) == 0
        && JS_DefinePropertyValue(ctx, obj, JS_ATOM_name, JS_AtomToString(ctx, name), flags) < 0
    {
        -1
    } else {
        0
    }
}
// c: quickjs.c:10789
unsafe fn JS_DefineObjectNameComputed(
    ctx: *mut JSContext,
    obj: JSValueConst,
    str: JSValueConst,
    flags: i32,
) -> i32 {
    if JS_IsObject(obj) != 0 && js_object_has_name(ctx, obj) == 0 {
        let prop = JS_ValueToAtom(ctx, str);
        if prop == JS_ATOM_NULL as u32 {
            return -1;
        }
        let name_str = js_get_function_name(ctx, prop);
        JS_FreeAtom(ctx, prop);
        if JS_IsException(name_str) != 0 {
            return -1;
        }
        if JS_DefinePropertyValue(ctx, obj, JS_ATOM_name, name_str, flags) < 0 {
            return -1;
        }
    }
    0
}
const JS_DEFINE_CLASS_HAS_HERITAGE: i32 = 1 << 0;
// c: quickjs.c:17452
unsafe fn js_op_define_class(
    ctx: *mut JSContext,
    sp: *mut JSValue,
    class_name: JSAtom,
    class_flags: i32,
    cur_var_refs: *mut *mut JSVarRef,
    sf: *mut JSStackFrame,
    is_computed_name: JS_BOOL,
) -> i32 {
    let mut parent_class = *sp.sub(2);
    let mut bfunc = *sp.sub(1);
    let mut proto = JS_UNDEFINED;
    let mut ctor = JS_UNDEFINED;
    let mut parent_proto = JS_UNDEFINED;
    let success = (|| {
        if class_flags & JS_DEFINE_CLASS_HAS_HERITAGE != 0 {
            if JS_IsNull(parent_class) != 0 {
                parent_proto = JS_NULL;
                parent_class = JS_DupValue(ctx, (*ctx).function_proto);
            } else {
                if JS_IsConstructor(ctx, parent_class) == 0 {
                    JS_ThrowTypeError(ctx, c"parent class must be constructor".as_ptr());
                    return false;
                }
                parent_proto = JS_GetProperty(ctx, parent_class, JS_ATOM_prototype);
                if JS_IsException(parent_proto) != 0 {
                    return false;
                }
                if JS_IsNull(parent_proto) == 0 && JS_IsObject(parent_proto) == 0 {
                    JS_ThrowTypeError(ctx, c"parent prototype must be an object or null".as_ptr());
                    return false;
                }
            }
        } else {
            parent_proto = JS_DupValue(ctx, *(*ctx).class_proto.add(JS_CLASS_OBJECT as usize));
            parent_class = JS_DupValue(ctx, (*ctx).function_proto);
        }
        proto = JS_NewObjectProto(ctx, parent_proto);
        if JS_IsException(proto) != 0 {
            return false;
        }
        let b = JS_VALUE_GET_PTR(bfunc).cast::<JSFunctionBytecode>();
        assert_eq!((*b).func_kind() as u32, JS_FUNC_NORMAL);
        ctor = JS_NewObjectProtoClass(ctx, parent_class, JS_CLASS_BYTECODE_FUNCTION);
        if JS_IsException(ctor) != 0 {
            return false;
        }
        ctor = js_closure2(ctx, ctor, b, cur_var_refs, sf, 0, ptr::null_mut());
        bfunc = JS_UNDEFINED;
        if JS_IsException(ctor) != 0 {
            return false;
        }
        js_method_set_home_object(ctx, ctor, proto);
        JS_SetConstructorBit(ctx, ctor, 1);
        JS_DefinePropertyValue(
            ctx,
            ctor,
            JS_ATOM_length,
            JS_NewInt32(ctx, (*b).defined_arg_count as i32),
            JS_PROP_CONFIGURABLE,
        );
        let name_ret = if is_computed_name != 0 {
            JS_DefineObjectNameComputed(ctx, ctor, *sp.sub(3), JS_PROP_CONFIGURABLE)
        } else {
            JS_DefineObjectName(ctx, ctor, class_name, JS_PROP_CONFIGURABLE)
        };
        if name_ret < 0 {
            return false;
        }
        if JS_DefinePropertyValue(
            ctx,
            proto,
            JS_ATOM_constructor,
            JS_DupValue(ctx, ctor),
            JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE | JS_PROP_THROW,
        ) < 0
        {
            return false;
        }
        if JS_DefinePropertyValue(
            ctx,
            ctor,
            JS_ATOM_prototype,
            JS_DupValue(ctx, proto),
            JS_PROP_THROW,
        ) < 0
        {
            return false;
        }
        set_cycle_flag(ctx, ctor);
        set_cycle_flag(ctx, proto);
        true
    })();
    if success {
        JS_FreeValue(ctx, parent_proto);
        JS_FreeValue(ctx, parent_class);
        *sp.sub(2) = ctor;
        *sp.sub(1) = proto;
        0
    } else {
        for v in [parent_class, parent_proto, bfunc, proto, ctor] {
            JS_FreeValue(ctx, v);
        }
        *sp.sub(2) = JS_UNDEFINED;
        *sp.sub(1) = JS_UNDEFINED;
        -1
    }
}
