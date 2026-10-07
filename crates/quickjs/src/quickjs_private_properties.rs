// quickjs.c:8365-8549. MIT. Private fields and method brands.
unsafe fn JS_ThrowTypeErrorPrivateNotFound(ctx: *mut JSContext, atom: JSAtom) -> JSValue {
    JS_ThrowTypeErrorAtom(
        ctx,
        c"private class field '%s' does not exist".as_ptr(),
        atom,
    )
}
unsafe fn JS_DefinePrivateField(
    ctx: *mut JSContext,
    obj: JSValueConst,
    name: JSValueConst,
    val: JSValue,
) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        JS_ThrowTypeErrorNotAnObject(ctx);
        JS_FreeValue(ctx, val);
        return -1;
    }
    if JS_VALUE_GET_TAG(name) != JS_TAG_SYMBOL {
        JS_ThrowTypeErrorNotASymbol(ctx);
        JS_FreeValue(ctx, val);
        return -1;
    }
    let prop = js_symbol_to_atom(ctx, name);
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, prop);
    if !prs.is_null() {
        JS_ThrowTypeErrorAtom(
            ctx,
            c"private class field '%s' already exists".as_ptr(),
            prop,
        );
        JS_FreeValue(ctx, val);
        return -1;
    }
    pr = add_property(ctx, p, prop, JS_PROP_C_W_E);
    if pr.is_null() {
        JS_FreeValue(ctx, val);
        return -1;
    }
    (*pr).u.value = val;
    0
}
unsafe fn JS_GetPrivateField(
    ctx: *mut JSContext,
    obj: JSValueConst,
    name: JSValueConst,
) -> JSValue {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return JS_ThrowTypeErrorNotAnObject(ctx);
    }
    if JS_VALUE_GET_TAG(name) != JS_TAG_SYMBOL {
        return JS_ThrowTypeErrorNotASymbol(ctx);
    }
    let prop = js_symbol_to_atom(ctx, name);
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, prop);
    if prs.is_null() {
        JS_ThrowTypeErrorPrivateNotFound(ctx, prop);
        return JS_EXCEPTION;
    }
    JS_DupValue(ctx, (*pr).u.value)
}
unsafe fn JS_SetPrivateField(
    ctx: *mut JSContext,
    obj: JSValueConst,
    name: JSValueConst,
    val: JSValue,
) -> i32 {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        JS_ThrowTypeErrorNotAnObject(ctx);
        JS_FreeValue(ctx, val);
        return -1;
    }
    if JS_VALUE_GET_TAG(name) != JS_TAG_SYMBOL {
        JS_ThrowTypeErrorNotASymbol(ctx);
        JS_FreeValue(ctx, val);
        return -1;
    }
    let prop = js_symbol_to_atom(ctx, name);
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, prop);
    if prs.is_null() {
        JS_ThrowTypeErrorPrivateNotFound(ctx, prop);
        JS_FreeValue(ctx, val);
        return -1;
    }
    set_value(ctx, ptr::addr_of_mut!((*pr).u.value), val);
    0
}
unsafe fn JS_AddBrand(ctx: *mut JSContext, obj: JSValueConst, home_obj: JSValueConst) -> i32 {
    if JS_VALUE_GET_TAG(home_obj) != JS_TAG_OBJECT {
        JS_ThrowTypeErrorNotAnObject(ctx);
        return -1;
    }
    let p = JS_VALUE_GET_PTR(home_obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, crate::quickjs_atom::JS_ATOM_Private_brand);
    let brand = if prs.is_null() {
        let brand = JS_NewSymbolFromAtom(
            ctx,
            crate::quickjs_atom::JS_ATOM_brand,
            JS_ATOM_TYPE_PRIVATE as u32,
        );
        if JS_IsException(brand) != 0 {
            return -1;
        }
        pr = add_property(
            ctx,
            p,
            crate::quickjs_atom::JS_ATOM_Private_brand,
            JS_PROP_C_W_E,
        );
        if pr.is_null() {
            JS_FreeValue(ctx, brand);
            return -1;
        }
        (*pr).u.value = JS_DupValue(ctx, brand);
        brand
    } else {
        JS_DupValue(ctx, (*pr).u.value)
    };
    let brand_atom = js_symbol_to_atom(ctx, brand);
    if JS_IsObject(obj) != 0 {
        let p1 = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        let prs = find_own_property(&mut pr, p1, brand_atom);
        if !prs.is_null() {
            JS_FreeAtom(ctx, brand_atom);
            JS_ThrowTypeError(ctx, c"private method is already present".as_ptr());
            return -1;
        }
        pr = add_property(ctx, p1, brand_atom, JS_PROP_C_W_E);
        JS_FreeAtom(ctx, brand_atom);
        if pr.is_null() {
            return -1;
        }
        (*pr).u.value = JS_UNDEFINED;
    } else {
        JS_FreeAtom(ctx, brand_atom);
    }
    0
}
unsafe fn JS_CheckBrand(ctx: *mut JSContext, obj: JSValueConst, func: JSValueConst) -> i32 {
    macro_rules! not_obj {
        () => {{
            JS_ThrowTypeErrorNotAnObject(ctx);
            return -1;
        }};
    }
    if JS_VALUE_GET_TAG(func) != JS_TAG_OBJECT {
        not_obj!();
    }
    let p1 = JS_VALUE_GET_PTR(func).cast::<JSObject>();
    if js_class_has_bytecode((*p1).class_id as u32) == 0 {
        not_obj!();
    }
    let home_obj = (*p1).u.func.home_object;
    if home_obj.is_null() {
        not_obj!();
    }
    let mut pr = ptr::null_mut();
    let prs = find_own_property(
        &mut pr,
        home_obj,
        crate::quickjs_atom::JS_ATOM_Private_brand,
    );
    if prs.is_null() {
        JS_ThrowTypeError(ctx, c"expecting <brand> private field".as_ptr());
        return -1;
    }
    let brand = (*pr).u.value;
    if JS_VALUE_GET_TAG(brand) != JS_TAG_SYMBOL || JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        not_obj!();
    }
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    (!find_own_property(&mut pr, p, js_symbol_to_atom(ctx, brand)).is_null()) as i32
}
