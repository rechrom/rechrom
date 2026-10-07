// Official quickjs.c 13658, 13670, 56744, 57131..57200. MIT.
// Shared conversion and TypedArray bounds helpers used by Array methods.
unsafe fn JS_ToLocaleStringFree(ctx: *mut JSContext, val: JSValue) -> JSValue {
    if JS_IsUndefined(val) != 0 || JS_IsNull(val) != 0 {
        JS_ToStringFree(ctx, val)
    } else {
        JS_InvokeFree(ctx, val, crate::quickjs_atom::JS_ATOM_toLocaleString, 0, ptr::null_mut())
    }
}
unsafe fn JS_ToStringCheckObject(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    let tag = JS_VALUE_GET_TAG(val) as u32;
    if tag == JS_TAG_NULL as u32 || tag == JS_TAG_UNDEFINED as u32 {
        return JS_ThrowTypeError(ctx, c"null or undefined are forbidden".as_ptr());
    }
    JS_ToString(ctx, val)
}
unsafe fn JS_ThrowTypeErrorArrayBufferOOB(ctx: *mut JSContext) -> JSValue {
    JS_ThrowTypeError(ctx, c"ArrayBuffer is detached or resized".as_ptr())
}
unsafe fn get_typed_array(ctx: *mut JSContext, this_val: JSValueConst) -> *mut JSObject {
    if JS_VALUE_GET_TAG(this_val) == JS_TAG_OBJECT {
        let p = JS_VALUE_GET_PTR(this_val).cast::<JSObject>();
        if (*p).class_id as u32 >= JS_CLASS_UINT8C_ARRAY && (*p).class_id as u32 <= JS_CLASS_FLOAT64_ARRAY {
            return p;
        }
    }
    JS_ThrowTypeError(ctx, c"not a TypedArray".as_ptr());
    ptr::null_mut()
}
unsafe fn typed_array_is_oob(p: *mut JSObject) -> JS_BOOL {
    assert!((*p).class_id as u32 >= JS_CLASS_UINT8C_ARRAY);
    assert!((*p).class_id as u32 <= JS_CLASS_FLOAT64_ARRAY);
    let ta = (*p).u.typed_array;
    let abuf = (*(*ta).buffer).u.array_buffer;
    if (*abuf).detached != 0 { return 1; }
    let len = (*abuf).byte_length;
    if (*ta).offset > len as u32 { return 1; }
    if (*ta).track_rab != 0 { return 0; }
    if (len as i64) < (*ta).offset as i64 + (*ta).length as i64 { return 1; }
    const LOG2: [u8; 12] = [0, 0, 0, 1, 1, 2, 2, 3, 3, 1, 2, 3];
    let size_elem = 1i32 << LOG2[(*p).class_id as usize - JS_CLASS_UINT8C_ARRAY as usize];
    let end = (*ta).offset as i64 + (*p).u.array.count as i64 * size_elem as i64;
    (end > len as i64) as i32
}
unsafe fn js_typed_array_get_length_unsafe(ctx: *mut JSContext, obj: JSValueConst) -> i32 {
    let p = get_typed_array(ctx, obj);
    if p.is_null() { return -1; }
    if typed_array_is_oob(p) != 0 {
        JS_ThrowTypeErrorArrayBufferOOB(ctx);
        return -1;
    }
    (*p).u.array.count as i32
}
// libc memcmp's exact lexicographic unsigned-byte comparison. No foreign FFI.
unsafe fn js_array_memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    for i in 0..n {
        let x = *a.cast::<u8>().add(i);
        let y = *b.cast::<u8>().add(i);
        if x != y { return x as i32 - y as i32; }
    }
    0
}
