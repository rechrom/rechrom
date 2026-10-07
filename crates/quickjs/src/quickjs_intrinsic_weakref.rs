// quickjs.c:60933..60967,61061..61117,61120..61126,61159..61195. MIT.
pub unsafe fn JS_GetOpaque2(
    ctx: *mut JSContext,
    obj: JSValueConst,
    class_id: JSClassID,
) -> *mut c_void {
    let p = JS_GetOpaque(obj, class_id);
    if p.is_null() {
        JS_ThrowTypeErrorInvalidClass(ctx, class_id as i32);
    }
    p
}
unsafe fn js_weakref_constructor(
    ctx: *mut JSContext,
    new_target: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    if JS_IsUndefined(new_target) != 0 {
        return JS_ThrowTypeError(ctx, c"constructor requires 'new'".as_ptr());
    }
    let arg = *argv;
    if js_weakref_is_target(arg) == 0 {
        return JS_ThrowTypeError(ctx, c"invalid target".as_ptr());
    }
    let obj = js_create_from_ctor(ctx, new_target, JS_CLASS_WEAK_REF as i32);
    if JS_IsException(obj) != 0 {
        return JS_EXCEPTION;
    }
    let wrd = js_mallocz(ctx, size_of::<JSWeakRefData>()).cast::<JSWeakRefData>();
    if wrd.is_null() {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    (*wrd).target = js_weakref_new(ctx, arg);
    (*wrd).weakref_header.weakref_type = JS_WEAKREF_TYPE_WEAKREF;
    list_add_tail(
        &mut (*wrd).weakref_header.link,
        &mut (*(*ctx).rt).weakref_list,
    );
    JS_SetOpaque(obj, wrd.cast());
    obj
}
unsafe fn js_weakref_deref(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
) -> JSValue {
    let wrd = JS_GetOpaque2(ctx, this_val, JS_CLASS_WEAK_REF).cast::<JSWeakRefData>();
    if wrd.is_null() {
        return JS_EXCEPTION;
    }
    if js_weakref_is_live((*wrd).target) != 0 {
        JS_DupValue(ctx, (*wrd).target)
    } else {
        JS_UNDEFINED
    }
}
static mut js_weakref_proto_funcs: [JSCFunctionListEntry; 2] = [
    JS_CFUNC_DEF(c"deref".as_ptr(), 0, Some(js_weakref_deref)),
    JS_PROP_STRING_DEF(
        c"[Symbol.toStringTag]".as_ptr(),
        c"WeakRef".as_ptr(),
        JS_PROP_CONFIGURABLE,
    ),
];
static js_weakref_class_def: [JSClassShortDef; 1] = [JSClassShortDef {
    class_name: crate::quickjs_atom::JS_ATOM_WeakRef,
    finalizer: Some(js_weakref_finalizer),
    gc_mark: None,
}];
unsafe fn js_finrec_constructor(
    ctx: *mut JSContext,
    new_target: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    if JS_IsUndefined(new_target) != 0 {
        return JS_ThrowTypeError(ctx, c"constructor requires 'new'".as_ptr());
    }
    let cb = *argv;
    if JS_IsFunction(ctx, cb) == 0 {
        return JS_ThrowTypeError(ctx, c"argument must be a function".as_ptr());
    }
    let obj = js_create_from_ctor(ctx, new_target, JS_CLASS_FINALIZATION_REGISTRY as i32);
    if JS_IsException(obj) != 0 {
        return JS_EXCEPTION;
    }
    let frd = js_mallocz(ctx, size_of::<JSFinalizationRegistryData>())
        .cast::<JSFinalizationRegistryData>();
    if frd.is_null() {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }
    (*frd).weakref_header.weakref_type = JS_WEAKREF_TYPE_FINREC;
    list_add_tail(
        &mut (*frd).weakref_header.link,
        &mut (*(*ctx).rt).weakref_list,
    );
    init_list_head(&mut (*frd).entries);
    (*frd).realm = JS_DupContext(ctx);
    (*frd).cb = JS_DupValue(ctx, cb);
    JS_SetOpaque(obj, frd.cast());
    obj
}
unsafe fn js_finrec_register(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let frd = JS_GetOpaque2(ctx, this_val, JS_CLASS_FINALIZATION_REGISTRY)
        .cast::<JSFinalizationRegistryData>();
    if frd.is_null() {
        return JS_EXCEPTION;
    }
    let target = *argv;
    let held_val = *argv.add(1);
    let token = if argc > 2 { *argv.add(2) } else { JS_UNDEFINED };
    if js_weakref_is_target(target) == 0 {
        return JS_ThrowTypeError(ctx, c"invalid target".as_ptr());
    }
    if js_same_value(ctx, target, held_val) != 0 {
        return JS_ThrowTypeError(ctx, c"held value cannot be the target".as_ptr());
    }
    if JS_IsUndefined(token) == 0 && js_weakref_is_target(token) == 0 {
        return JS_ThrowTypeError(ctx, c"invalid unregister token".as_ptr());
    }
    let fre = js_malloc(ctx, size_of::<JSFinRecEntry>()).cast::<JSFinRecEntry>();
    if fre.is_null() {
        return JS_EXCEPTION;
    }
    (*fre).target = js_weakref_new(ctx, target);
    (*fre).held_val = JS_DupValue(ctx, held_val);
    (*fre).token = js_weakref_new(ctx, token);
    list_add_tail(&mut (*fre).link, &mut (*frd).entries);
    JS_UNDEFINED
}
unsafe fn js_finrec_unregister(
    ctx: *mut JSContext,
    this_val: JSValueConst,
    _argc: i32,
    argv: *mut JSValueConst,
) -> JSValue {
    let frd = JS_GetOpaque2(ctx, this_val, JS_CLASS_FINALIZATION_REGISTRY)
        .cast::<JSFinalizationRegistryData>();
    if frd.is_null() {
        return JS_EXCEPTION;
    }
    let token = *argv;
    if js_weakref_is_target(token) == 0 {
        return JS_ThrowTypeError(ctx, c"invalid unregister token".as_ptr());
    }
    let mut removed = 0;
    for el in ListIter::new(&mut (*frd).entries, false, true) {
        let fre = el
            .cast::<u8>()
            .sub(offset_of!(JSFinRecEntry, link))
            .cast::<JSFinRecEntry>();
        if js_weakref_is_live((*fre).token) != 0 && js_same_value(ctx, (*fre).token, token) != 0 {
            js_weakref_free((*ctx).rt, (*fre).target);
            js_weakref_free((*ctx).rt, (*fre).token);
            JS_FreeValue(ctx, (*fre).held_val);
            list_del(&mut (*fre).link);
            js_free(ctx, fre.cast());
            removed = 1;
        }
    }
    JS_NewBool(ctx, removed)
}
static mut js_finrec_proto_funcs: [JSCFunctionListEntry; 3] = [
    JS_CFUNC_DEF(c"register".as_ptr(), 2, Some(js_finrec_register)),
    JS_CFUNC_DEF(c"unregister".as_ptr(), 1, Some(js_finrec_unregister)),
    JS_PROP_STRING_DEF(
        c"[Symbol.toStringTag]".as_ptr(),
        c"FinalizationRegistry".as_ptr(),
        JS_PROP_CONFIGURABLE,
    ),
];
static js_finrec_class_def: [JSClassShortDef; 1] = [JSClassShortDef {
    class_name: crate::quickjs_atom::JS_ATOM_FinalizationRegistry,
    finalizer: Some(js_finrec_finalizer),
    gc_mark: Some(js_finrec_mark),
}];
pub unsafe fn JS_AddIntrinsicWeakRef(ctx: *mut JSContext) -> i32 {
    let rt = (*ctx).rt;
    if JS_IsRegisteredClass(rt, JS_CLASS_WEAK_REF) == 0
        && init_class_range(
            rt,
            js_weakref_class_def.as_ptr(),
            JS_CLASS_WEAK_REF as i32,
            1,
        ) != 0
    {
        return -1;
    }
    let obj = JS_NewCConstructor(
        ctx,
        JS_CLASS_WEAK_REF as i32,
        c"WeakRef".as_ptr(),
        Some(js_weakref_constructor),
        1,
        JS_CFUNC_constructor_or_func,
        0,
        JS_UNDEFINED,
        ptr::null(),
        0,
        ptr::addr_of!(js_weakref_proto_funcs).cast(),
        2,
        0,
    );
    if JS_IsException(obj) != 0 {
        return -1;
    }
    JS_FreeValue(ctx, obj);
    if JS_IsRegisteredClass(rt, JS_CLASS_FINALIZATION_REGISTRY) == 0
        && init_class_range(
            rt,
            js_finrec_class_def.as_ptr(),
            JS_CLASS_FINALIZATION_REGISTRY as i32,
            1,
        ) != 0
    {
        return -1;
    }
    let obj = JS_NewCConstructor(
        ctx,
        JS_CLASS_FINALIZATION_REGISTRY as i32,
        c"FinalizationRegistry".as_ptr(),
        Some(js_finrec_constructor),
        1,
        JS_CFUNC_constructor_or_func,
        0,
        JS_UNDEFINED,
        ptr::null(),
        0,
        ptr::addr_of!(js_finrec_proto_funcs).cast(),
        3,
        0,
    );
    if JS_IsException(obj) != 0 {
        return -1;
    }
    JS_FreeValue(ctx, obj);
    0
}
