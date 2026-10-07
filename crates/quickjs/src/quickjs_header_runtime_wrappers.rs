// quickjs.h:462-465,752-759. MIT. Runtime-backed inline wrappers.
// Included with the translated atom/string APIs when their scope is connected.
#[inline]
pub unsafe fn JS_AtomToCString(ctx: *mut JSContext, atom: JSAtom) -> *const c_char {
    JS_AtomToCStringLen(ctx, ptr::null_mut(), atom)
}
#[inline]
pub unsafe fn JS_ToCStringLen(
    ctx: *mut JSContext,
    plen: *mut usize,
    val: JSValueConst,
) -> *const c_char {
    JS_ToCStringLen2(ctx, plen, val, 0)
}
#[inline]
pub unsafe fn JS_ToCString(ctx: *mut JSContext, val: JSValueConst) -> *const c_char {
    JS_ToCStringLen2(ctx, ptr::null_mut(), val, 0)
}

// quickjs.h:1042-1050. Preserve the original function-pointer union conversion.
#[inline]
pub unsafe fn JS_NewCFunctionMagic(
    ctx: *mut JSContext, func: Option<JSCFunctionMagic>, name: *const c_char,
    length: i32, cproto: JSCFunctionEnum, magic: i32,
) -> JSValue {
    let ft = JSCFunctionType { generic_magic: func };
    JS_NewCFunction2(ctx, ft.generic, name, length, cproto, magic)
}
