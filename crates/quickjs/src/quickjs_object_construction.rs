// quickjs.c:1780-1799,5613-5804,5831-5870,5882-5885,5919-5942.
// Pending production allocation/error/weak-GC integration. MIT.
unsafe fn js_trigger_gc(rt: *mut JSRuntime, size: usize) {
    #[cfg(feature = "force-gc-at-malloc")]
    let force_gc = true;
    #[cfg(not(feature = "force-gc-at-malloc"))]
    let force_gc =
        (*rt).malloc_ctx.malloc_state.malloc_size.wrapping_add(size) > (*rt).malloc_gc_threshold;
    if force_gc {
        JS_RunGC(rt);
        (*rt).malloc_gc_threshold = (*rt)
            .malloc_ctx
            .malloc_state
            .malloc_size
            .wrapping_add((*rt).malloc_ctx.malloc_state.malloc_size >> 1);
    }
}
unsafe fn JS_NewObjectFromShape(
    ctx: *mut JSContext,
    sh: *mut JSShape,
    class_id: JSClassID,
    props: *mut JSProperty,
) -> JSValue {
    js_trigger_gc((*ctx).rt, size_of::<JSObject>());
    let p = js_malloc(ctx, size_of::<JSObject>()).cast::<JSObject>();
    let mut failed = p.is_null();
    if !failed {
        (*p).class_id = class_id as u16;
        (*p).set_is_std_array_prototype(0);
        (*p).set_extensible(1);
        (*p).set_free_mark(0);
        (*p).set_is_exotic(0);
        (*p).set_fast_array(0);
        (*p).set_is_constructor(0);
        (*p).set_has_immutable_prototype(0);
        (*p).set_tmp_mark(0);
        (*p).set_is_HTMLDDA(0);
        (*p).weakref_count = 0;
        (*p).u.opaque = ptr::null_mut();
        (*p).shape = sh;
        (*p).prop = js_malloc(ctx, size_of::<JSProperty>() * (*sh).prop_size as usize).cast();
        if (*p).prop.is_null() {
            js_free(ctx, p.cast());
            failed = true;
        }
    }
    if failed {
        if !props.is_null() {
            let mut prs = get_shape_prop(sh);
            let mut i = 0;
            while i < (*sh).prop_count {
                free_property((*ctx).rt, props.add(i as usize), (*prs).flags() as i32);
                prs = prs.add(1);
                i += 1;
            }
        }
        js_free_shape((*ctx).rt, sh);
        return JS_EXCEPTION;
    }
    match class_id {
        JS_CLASS_OBJECT => {}
        JS_CLASS_ARRAY => {
            (*p).set_is_exotic(1);
            (*p).set_fast_array(1);
            (*p).u.array.u.values = ptr::null_mut();
            (*p).u.array.count = 0;
            (*p).u.array.u1.size = 0;
            if props.is_null() {
                let pr = if sh == (*ctx).array_shape {
                    (*p).prop
                } else {
                    // As in C, the first array's preallocated length property cannot fail.
                    add_property(
                        ctx,
                        p,
                        crate::quickjs_atom::JS_ATOM_length,
                        JS_PROP_WRITABLE | JS_PROP_LENGTH,
                    )
                };
                (*pr).u.value = JS_NewInt32(ctx, 0);
            }
        }
        JS_CLASS_C_FUNCTION => {
            (*(*p).prop).u.value = JS_UNDEFINED;
        }
        JS_CLASS_ARGUMENTS
        | JS_CLASS_MAPPED_ARGUMENTS
        | JS_CLASS_UINT8C_ARRAY
        | JS_CLASS_INT8_ARRAY
        | JS_CLASS_UINT8_ARRAY
        | JS_CLASS_INT16_ARRAY
        | JS_CLASS_UINT16_ARRAY
        | JS_CLASS_INT32_ARRAY
        | JS_CLASS_UINT32_ARRAY
        | JS_CLASS_BIG_INT64_ARRAY
        | JS_CLASS_BIG_UINT64_ARRAY
        | JS_CLASS_FLOAT16_ARRAY
        | JS_CLASS_FLOAT32_ARRAY
        | JS_CLASS_FLOAT64_ARRAY => {
            (*p).set_is_exotic(1);
            (*p).set_fast_array(1);
            (*p).u.array.u.ptr = ptr::null_mut();
            (*p).u.array.count = 0;
        }
        JS_CLASS_DATAVIEW => {
            (*p).u.array.u.ptr = ptr::null_mut();
            (*p).u.array.count = 0;
        }
        JS_CLASS_NUMBER | JS_CLASS_STRING | JS_CLASS_BOOLEAN | JS_CLASS_SYMBOL | JS_CLASS_DATE
        | JS_CLASS_BIG_INT => {
            (*p).u.object_data = JS_UNDEFINED;
            if !(*(*(*ctx).rt).class_array.add(class_id as usize))
                .exotic
                .is_null()
            {
                (*p).set_is_exotic(1);
            }
        }
        JS_CLASS_REGEXP => {
            (*p).u.regexp.pattern = ptr::null_mut();
            (*p).u.regexp.bytecode = ptr::null_mut();
        }
        JS_CLASS_GLOBAL_OBJECT => {
            (*p).u.global_object.uninitialized_vars = JS_UNDEFINED;
        }
        _ => {
            if !(*(*(*ctx).rt).class_array.add(class_id as usize))
                .exotic
                .is_null()
            {
                (*p).set_is_exotic(1);
            }
        }
    }
    (*js_rc(p.cast())).ref_count = 1;
    add_gc_object((*ctx).rt, &mut (*p).header, JS_GC_OBJ_TYPE_JS_OBJECT);
    if !props.is_null() {
        let mut i = 0;
        while i < (*sh).prop_count {
            ptr::copy_nonoverlapping(props.add(i as usize), (*p).prop.add(i as usize), 1);
            i += 1;
        }
    }
    JS_MKPTR(JS_TAG_OBJECT, p.cast())
}
unsafe fn get_proto_obj(proto_val: JSValueConst) -> *mut JSObject {
    if JS_VALUE_GET_TAG(proto_val) != JS_TAG_OBJECT {
        ptr::null_mut()
    } else {
        JS_VALUE_GET_PTR(proto_val).cast()
    }
}
pub unsafe fn JS_NewObjectProtoClass(
    ctx: *mut JSContext,
    proto_val: JSValueConst,
    class_id: JSClassID,
) -> JSValue {
    let proto = get_proto_obj(proto_val);
    let mut sh = find_hashed_shape_proto((*ctx).rt, proto);
    if !sh.is_null() {
        sh = js_dup_shape(sh);
    } else {
        sh = js_new_shape(ctx, proto);
        if sh.is_null() {
            return JS_EXCEPTION;
        }
    }
    JS_NewObjectFromShape(ctx, sh, class_id, ptr::null_mut())
}
unsafe fn JS_NewObjectProtoClassAlloc(
    ctx: *mut JSContext,
    proto_val: JSValueConst,
    class_id: JSClassID,
    mut n_alloc_props: i32,
) -> JSValue {
    let hash_size;
    if n_alloc_props <= JS_PROP_INITIAL_SIZE {
        n_alloc_props = JS_PROP_INITIAL_SIZE;
        hash_size = JS_PROP_INITIAL_HASH_SIZE;
    } else {
        let hash_bits = 32 - ((n_alloc_props - 1) as u32).leading_zeros();
        hash_size = 1i32.wrapping_shl(hash_bits);
    }
    let sh = js_new_shape_nohash(ctx, get_proto_obj(proto_val), hash_size, n_alloc_props);
    if sh.is_null() {
        return JS_EXCEPTION;
    }
    JS_NewObjectFromShape(ctx, sh, class_id, ptr::null_mut())
}
unsafe fn JS_SetObjectData(ctx: *mut JSContext, obj: JSValueConst, val: JSValue) -> i32 {
    if JS_VALUE_GET_TAG(obj) == JS_TAG_OBJECT {
        let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        match (*p).class_id as u32 {
            JS_CLASS_NUMBER | JS_CLASS_STRING | JS_CLASS_BOOLEAN | JS_CLASS_SYMBOL
            | JS_CLASS_DATE | JS_CLASS_BIG_INT => {
                JS_FreeValue(ctx, (*p).u.object_data);
                (*p).u.object_data = val;
                return 0;
            }
            _ => {}
        }
    }
    JS_FreeValue(ctx, val);
    if JS_IsException(obj) == 0 {
        JS_ThrowTypeError(ctx, c"invalid object type".as_ptr());
    }
    -1
}
pub unsafe fn JS_NewObjectClass(ctx: *mut JSContext, class_id: i32) -> JSValue {
    JS_NewObjectProtoClass(
        ctx,
        *(*ctx).class_proto.offset(class_id as isize),
        class_id as u32,
    )
}
pub unsafe fn JS_NewObjectProto(ctx: *mut JSContext, proto: JSValueConst) -> JSValue {
    JS_NewObjectProtoClass(ctx, proto, JS_CLASS_OBJECT)
}
pub unsafe fn JS_NewArray(ctx: *mut JSContext) -> JSValue {
    JS_NewObjectFromShape(
        ctx,
        js_dup_shape((*ctx).array_shape),
        JS_CLASS_ARRAY,
        ptr::null_mut(),
    )
}
pub unsafe fn JS_NewObject(ctx: *mut JSContext) -> JSValue {
    JS_NewObjectProtoClass(
        ctx,
        *(*ctx).class_proto.add(JS_CLASS_OBJECT as usize),
        JS_CLASS_OBJECT,
    )
}
fn js_class_has_bytecode(class_id: JSClassID) -> JS_BOOL {
    (class_id == JS_CLASS_BYTECODE_FUNCTION
        || class_id == JS_CLASS_GENERATOR_FUNCTION
        || class_id == JS_CLASS_ASYNC_FUNCTION
        || class_id == JS_CLASS_ASYNC_GENERATOR_FUNCTION) as i32
}
unsafe fn JS_GetFunctionBytecode(val: JSValueConst) -> *mut JSFunctionBytecode {
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT {
        return ptr::null_mut();
    }
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    if js_class_has_bytecode((*p).class_id as u32) == 0 {
        return ptr::null_mut();
    }
    (*p).u.func.function_bytecode
}
unsafe fn js_method_set_home_object(
    ctx: *mut JSContext,
    func_obj: JSValueConst,
    home_obj: JSValueConst,
) {
    if JS_VALUE_GET_TAG(func_obj) != JS_TAG_OBJECT {
        return;
    }
    let p = JS_VALUE_GET_PTR(func_obj).cast::<JSObject>();
    if js_class_has_bytecode((*p).class_id as u32) == 0 {
        return;
    }
    let b = (*p).u.func.function_bytecode;
    if (*b).need_home_object() != 0 {
        let old = (*p).u.func.home_object;
        if !old.is_null() {
            JS_FreeValue(ctx, JS_MKPTR(JS_TAG_OBJECT, old.cast()));
        }
        (*p).u.func.home_object = if JS_VALUE_GET_TAG(home_obj) == JS_TAG_OBJECT {
            JS_VALUE_GET_PTR(JS_DupValue(ctx, home_obj)).cast()
        } else {
            ptr::null_mut()
        };
    }
}
pub unsafe fn JS_NewError(ctx: *mut JSContext) -> JSValue {
    JS_NewObjectClass(ctx, JS_CLASS_ERROR as i32)
}
