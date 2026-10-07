unsafe fn dump_names(out: &mut Vec<u8>, ctx: *mut JSContext, obj: JSValue, flags: i32) {
    let mut tab = ptr::null_mut();
    let mut count = 0;
    let ret = JS_GetOwnPropertyNames(ctx, &mut tab, &mut count, obj, flags);
    num(out, ret as u64, 4);
    num(out, count as u64, 4);
    dump_exception(out, ctx);
    if ret == 0 {
        for i in 0..count {
            let key = (*tab.add(i as usize)).atom;
            num(out, JS_AtomGetKind(ctx, key) as u64, 4);
            let val = JS_AtomToString(ctx, key);
            dump_val(out, val);
            JS_FreeValue(ctx, val);
            num(out, (*tab.add(i as usize)).is_enumerable as u64, 4);
        }
        JS_FreePropertyEnum(ctx, tab, count);
    }
}
unsafe fn names_private_global_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
    let name = JS_NewSymbolFromAtom(
        ctx,
        crate::quickjs_atom::JS_ATOM_name,
        JS_ATOM_TYPE_PRIVATE as u32,
    );
    let obj = JS_NewObject(ctx);
    let other = JS_NewObject(ctx);
    JS_PreventExtensions(ctx, obj);
    for target in [obj, other, JS_TRUE, JS_NULL] {
        num(
            out,
            JS_DefinePrivateField(ctx, target, name, JS_NewInt32(ctx, 91)) as u64,
            4,
        );
        dump_exception(out, ctx);
        num(
            out,
            JS_DefinePrivateField(ctx, target, name, JS_NewInt32(ctx, 92)) as u64,
            4,
        );
        dump_exception(out, ctx);
        let val = JS_GetPrivateField(ctx, target, name);
        dump_val(out, val);
        JS_FreeValue(ctx, val);
        dump_exception(out, ctx);
        num(
            out,
            JS_SetPrivateField(ctx, target, name, JS_NewInt32(ctx, 93)) as u64,
            4,
        );
        dump_exception(out, ctx);
    }
    num(
        out,
        JS_DefinePrivateField(ctx, obj, JS_TRUE, JS_NewInt32(ctx, 1)) as u64,
        4,
    );
    dump_exception(out, ctx);
    let home = JS_NewObject(ctx);
    let func = JS_NewObjectClass(ctx, JS_CLASS_BYTECODE_FUNCTION as i32);
    (*JS_VALUE_GET_PTR(func).cast::<JSObject>())
        .u
        .func
        .home_object = JS_VALUE_GET_PTR(home).cast();
    for target in [JS_NULL, obj, obj, other, JS_TRUE] {
        num(out, JS_AddBrand(ctx, target, home) as u64, 4);
        dump_exception(out, ctx);
        num(out, JS_CheckBrand(ctx, target, func) as u64, 4);
        dump_exception(out, ctx);
    }
    // Synthetic bytecode-class identity has no bytecode/finalizer in this fixture.
    (*JS_VALUE_GET_PTR(func).cast::<JSObject>())
        .u
        .func
        .home_object = ptr::null_mut();
    JS_FreeValue(ctx, func);
    JS_FreeValue(ctx, home);
    JS_FreeValue(ctx, other);
    JS_FreeValue(ctx, obj);
    JS_FreeValue(ctx, name);
    let obj = JS_NewObject(ctx);
    for (i, key) in [
        c"z",
        c"10",
        c"2",
        c"01",
        c"4294967294",
        c"4294967295",
        c"a",
        c"0",
    ]
    .into_iter()
    .enumerate()
    {
        JS_DefinePropertyValueStr(
            ctx,
            obj,
            key.as_ptr(),
            JS_NewInt32(ctx, i as i32),
            JS_PROP_CONFIGURABLE
                | JS_PROP_WRITABLE
                | if i % 2 != 0 { JS_PROP_ENUMERABLE } else { 0 },
        );
    }
    for kind in [JS_ATOM_TYPE_SYMBOL, JS_ATOM_TYPE_PRIVATE] {
        let key = JS_NewSymbolFromAtom(ctx, crate::quickjs_atom::JS_ATOM_name, kind as u32);
        JS_DefinePropertyValue(
            ctx,
            obj,
            js_symbol_to_atom(ctx, key),
            JS_NewInt32(ctx, 8),
            JS_PROP_C_W_E,
        );
        JS_FreeValue(ctx, key);
    }
    for flags in 0..64 {
        dump_names(out, ctx, obj, flags);
    }
    for fail in 1..8 {
        (*h).fail = (*h).calls + fail;
        dump_names(
            out,
            ctx,
            obj,
            JS_GPN_STRING_MASK | JS_GPN_SYMBOL_MASK | JS_GPN_SET_ENUM,
        );
        (*h).fail = 0;
    }
    let arr = JS_NewArray(ctx);
    for i in 0..4 {
        JS_SetPropertyUint32(ctx, arr, i, JS_NewInt32(ctx, i as i32));
    }
    for flags in 0..64 {
        dump_names(out, ctx, arr, flags);
    }
    JS_FreeValue(ctx, arr);
    JS_FreeValue(ctx, obj);
    let boxed = JS_ToObjectFree(ctx, JS_NewString(ctx, c"\u{100}ab".as_ptr()));
    let bp = JS_VALUE_GET_PTR(boxed).cast::<JSObject>();
    (*bp).set_is_exotic(1);
    for flags in 0..64 {
        dump_names(out, ctx, boxed, flags);
    }
    JS_FreeValue(ctx, boxed);
    (*ctx).global_obj = JS_NewObjectClass(ctx, JS_CLASS_GLOBAL_OBJECT as i32);
    let gp = JS_VALUE_GET_PTR((*ctx).global_obj).cast::<JSObject>();
    (*gp).u.global_object.uninitialized_vars = JS_NewObject(ctx);
    (*ctx).global_var_obj = JS_NewObject(ctx);
    let atom = crate::quickjs_atom::JS_ATOM_name;
    JS_DefinePropertyValue(
        ctx,
        (*ctx).global_obj,
        atom,
        JS_NewInt32(ctx, 17),
        JS_PROP_WRITABLE | JS_PROP_ENUMERABLE,
    );
    for flags in [0, DEFINE_GLOBAL_LEX_VAR, DEFINE_GLOBAL_FUNC_VAR] {
        num(out, JS_CheckDefineGlobalVar(ctx, atom, flags) as u64, 4);
        dump_exception(out, ctx);
    }
    let mut sp = [JS_UNDEFINED; 2];
    num(
        out,
        JS_GetGlobalVarRef(ctx, atom, sp.as_mut_ptr()) as u64,
        4,
    );
    for val in sp {
        dump_val(out, val);
        JS_FreeValue(ctx, val);
    }
    dump_exception(out, ctx);
    num(out, JS_DeleteGlobalVar(ctx, atom) as u64, 4);
    dump_exception(out, ctx);
    let lexicalp = JS_VALUE_GET_PTR((*ctx).global_var_obj).cast::<JSObject>();
    let pr = add_property(ctx, lexicalp, atom, JS_PROP_C_W_E | JS_PROP_VARREF);
    let vr = js_create_var_ref(ctx, 0);
    (*pr).u.var_ref = vr;
    (*vr).u.value = JS_UNINITIALIZED;
    for flags in [0, DEFINE_GLOBAL_LEX_VAR, DEFINE_GLOBAL_FUNC_VAR] {
        num(out, JS_CheckDefineGlobalVar(ctx, atom, flags) as u64, 4);
        dump_exception(out, ctx);
    }
    sp = [JS_UNDEFINED; 2];
    num(
        out,
        JS_GetGlobalVarRef(ctx, atom, sp.as_mut_ptr()) as u64,
        4,
    );
    dump_exception(out, ctx);
    dump_names(
        out,
        ctx,
        (*ctx).global_var_obj,
        JS_GPN_STRING_MASK | JS_GPN_SET_ENUM,
    );
    (*vr).u.value = JS_NewInt32(ctx, 88);
    sp = [JS_UNDEFINED; 2];
    num(
        out,
        JS_GetGlobalVarRef(ctx, atom, sp.as_mut_ptr()) as u64,
        4,
    );
    for val in sp {
        dump_val(out, val);
        JS_FreeValue(ctx, val);
    }
    dump_exception(out, ctx);
    num(out, JS_DeleteGlobalVar(ctx, atom) as u64, 4);
    dump_exception(out, ctx);
    JS_FreeValue(ctx, (*ctx).global_var_obj);
    JS_FreeValue(ctx, (*ctx).global_obj);
    (*ctx).global_var_obj = JS_UNDEFINED;
    (*ctx).global_obj = JS_UNDEFINED;
}
