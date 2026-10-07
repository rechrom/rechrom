// quickjs.c:11051-11061,39610-39662. Original public embedding helpers. MIT.
pub unsafe fn JS_GetAnyOpaque(obj: JSValueConst, class_id: *mut JSClassID) -> *mut c_void {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        *class_id = 0;
        return ptr::null_mut();
    }
    let p = JS_VALUE_GET_OBJ(obj);
    *class_id = (*p).class_id as JSClassID;
    (*p).u.opaque
}

pub unsafe fn JS_AddModuleExportList(
    ctx: *mut JSContext, m: *mut JSModuleDef,
    tab: *const JSCFunctionListEntry, len: i32,
) -> i32 {
    for i in 0..len {
        if JS_AddModuleExport(ctx, m, (*tab.add(i as usize)).name) != 0 {
            return -1;
        }
    }
    0
}

pub unsafe fn JS_SetModuleExportList(
    ctx: *mut JSContext, m: *mut JSModuleDef,
    tab: *const JSCFunctionListEntry, len: i32,
) -> i32 {
    for i in 0..len {
        let e = &*tab.add(i as usize);
        let val = match e.def_type as i32 {
            JS_DEF_CFUNC => JS_NewCFunction2(ctx, e.u.func.cfunc.generic, e.name,
                e.u.func.length as i32, e.u.func.cproto as JSCFunctionEnum, e.magic as i32),
            JS_DEF_PROP_STRING => JS_NewString(ctx, e.u.str),
            JS_DEF_PROP_INT32 => JS_NewInt32(ctx, e.u.i32),
            JS_DEF_PROP_INT64 => JS_NewInt64(ctx, e.u.i64),
            JS_DEF_PROP_DOUBLE => __JS_NewFloat64(ctx, e.u.f64),
            JS_DEF_OBJECT => JS_NewObjectProtoList(ctx,
                *(*ctx).class_proto.add(JS_CLASS_OBJECT as usize),
                e.u.prop_list.tab, e.u.prop_list.len as i32),
            _ => std::process::abort(),
        };
        if JS_SetModuleExport(ctx, m, e.name, val) != 0 {
            return -1;
        }
    }
    0
}
