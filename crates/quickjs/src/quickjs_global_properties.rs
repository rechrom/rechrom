// quickjs.c:10807-10919. MIT. Global binding checks and references.
const DEFINE_GLOBAL_LEX_VAR: i32 = 1 << 7;
const DEFINE_GLOBAL_FUNC_VAR: i32 = 1 << 6;
unsafe fn JS_ThrowSyntaxErrorVarRedeclaration(ctx: *mut JSContext, prop: JSAtom) -> JSValue {
    JS_ThrowSyntaxErrorAtom(ctx, c"redeclaration of '%s'".as_ptr(), prop)
}
unsafe fn JS_CheckDefineGlobalVar(ctx: *mut JSContext, prop: JSAtom, flags: i32) -> i32 {
    let p = JS_VALUE_GET_PTR((*ctx).global_obj).cast::<JSObject>();
    let prs = find_own_property1(p, prop);
    if flags & DEFINE_GLOBAL_LEX_VAR != 0 {
        if !prs.is_null() && (*prs).flags() as i32 & JS_PROP_CONFIGURABLE == 0 {
            JS_ThrowSyntaxErrorVarRedeclaration(ctx, prop);
            return -1;
        }
    } else {
        let define_error = prs.is_null() && (*p).extensible() == 0
            || flags & DEFINE_GLOBAL_FUNC_VAR != 0 && !prs.is_null() && {
                let pf = (*prs).flags() as i32;
                pf & JS_PROP_CONFIGURABLE == 0
                    && (pf & JS_PROP_TMASK == JS_PROP_GETSET
                        || pf & (JS_PROP_WRITABLE | JS_PROP_ENUMERABLE)
                            != (JS_PROP_WRITABLE | JS_PROP_ENUMERABLE))
            };
        if define_error {
            JS_ThrowTypeErrorAtom(ctx, c"cannot define variable '%s'".as_ptr(), prop);
            return -1;
        }
    }
    let p = JS_VALUE_GET_PTR((*ctx).global_var_obj).cast::<JSObject>();
    let prs = find_own_property1(p, prop);
    if !prs.is_null() {
        JS_ThrowSyntaxErrorVarRedeclaration(ctx, prop);
        return -1;
    }
    0
}
unsafe fn JS_GetGlobalVarRef(ctx: *mut JSContext, prop: JSAtom, sp: *mut JSValue) -> i32 {
    let p = JS_VALUE_GET_PTR((*ctx).global_var_obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, prop);
    if !prs.is_null() {
        if JS_IsUninitialized(*(*(*pr).u.var_ref).pvalue) != 0 {
            JS_ThrowReferenceErrorUninitialized(ctx, (*prs).atom);
            return -1;
        }
        if (*prs).flags() as i32 & JS_PROP_WRITABLE == 0 {
            return JS_ThrowTypeErrorReadOnly(ctx, JS_PROP_THROW, prop);
        }
        *sp = JS_DupValue(ctx, (*ctx).global_var_obj);
    } else {
        let ret = JS_HasProperty(ctx, (*ctx).global_obj, prop);
        if ret < 0 {
            return -1;
        }
        *sp = if ret != 0 {
            JS_DupValue(ctx, (*ctx).global_obj)
        } else {
            JS_UNDEFINED
        };
    }
    *sp.add(1) = JS_AtomToValue(ctx, prop);
    0
}
unsafe fn JS_DeleteGlobalVar(ctx: *mut JSContext, prop: JSAtom) -> i32 {
    let p = JS_VALUE_GET_PTR((*ctx).global_var_obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let prs = find_own_property(&mut pr, p, prop);
    if !prs.is_null() {
        return 0;
    }
    let ret = JS_HasProperty(ctx, (*ctx).global_obj, prop);
    if ret < 0 {
        return -1;
    }
    if ret != 0 {
        JS_DeleteProperty(ctx, (*ctx).global_obj, prop, 0)
    } else {
        1
    }
}
