// quickjs.c:1829-1934,2374-2383. MIT.
// Awaiting the production error/object/property path; exercised by the
// dependency-isolated StringBuffer oracle. No production error stub is used.
pub unsafe fn js_malloc(ctx: *mut JSContext, size: usize) -> *mut c_void {
    let p = js_malloc_rt((*ctx).rt, size);
    if p.is_null() {
        JS_ThrowOutOfMemory(ctx);
        return ptr::null_mut();
    }
    p
}
pub unsafe fn js_mallocz(ctx: *mut JSContext, size: usize) -> *mut c_void {
    let p = js_mallocz_rt((*ctx).rt, size);
    if p.is_null() {
        JS_ThrowOutOfMemory(ctx);
        return ptr::null_mut();
    }
    p
}
pub unsafe fn js_free(ctx: *mut JSContext, p: *mut c_void) {
    js_free_rt((*ctx).rt, p);
}
pub unsafe fn js_realloc(ctx: *mut JSContext, p: *mut c_void, size: usize) -> *mut c_void {
    let ret = js_realloc_rt((*ctx).rt, p, size);
    if ret.is_null() && size != 0 {
        JS_ThrowOutOfMemory(ctx);
        return ptr::null_mut();
    }
    ret
}
pub unsafe fn js_realloc2(
    ctx: *mut JSContext,
    p: *mut c_void,
    size: usize,
    pslack: *mut usize,
) -> *mut c_void {
    let ret = js_realloc_rt((*ctx).rt, p, size);
    if ret.is_null() && size != 0 {
        JS_ThrowOutOfMemory(ctx);
        return ptr::null_mut();
    }
    if !pslack.is_null() {
        let new_size = js_malloc_usable_size_rt((*ctx).rt, ret);
        *pslack = new_size.saturating_sub(size);
    }
    ret
}
pub unsafe fn js_malloc_usable_size(ctx: *mut JSContext, p: *const c_void) -> usize {
    js_malloc_usable_size_rt((*ctx).rt, p)
}
pub unsafe fn js_strndup(ctx: *mut JSContext, s: *const c_char, n: usize) -> *mut c_char {
    let p = js_malloc(ctx, n.wrapping_add(1)).cast::<c_char>();
    if !p.is_null() {
        ptr::copy_nonoverlapping(s, p, n);
        *p.add(n) = 0;
    }
    p
}
pub unsafe fn js_strdup(ctx: *mut JSContext, s: *const c_char) -> *mut c_char {
    js_strndup(ctx, s, core::ffi::CStr::from_ptr(s).to_bytes().len())
}
#[inline(never)]
unsafe fn js_realloc_array(
    ctx: *mut JSContext,
    parray: *mut *mut c_void,
    elem_size: i32,
    psize: *mut i32,
    req_size: i32,
) -> i32 {
    let mut new_size = req_size.max((*psize).wrapping_mul(3) / 2);
    let mut slack = 0;
    let new_array = js_realloc2(
        ctx,
        *parray,
        new_size.wrapping_mul(elem_size) as usize,
        &mut slack,
    );
    if new_array.is_null() {
        return -1;
    }
    new_size = new_size.wrapping_add((slack / elem_size as usize) as i32);
    *psize = new_size;
    *parray = new_array;
    0
}
#[inline]
unsafe fn js_resize_array(
    ctx: *mut JSContext,
    parray: *mut *mut c_void,
    elem_size: i32,
    psize: *mut i32,
    req_size: i32,
) -> i32 {
    if req_size > *psize {
        js_realloc_array(ctx, parray, elem_size, psize, req_size)
    } else {
        0
    }
}
unsafe fn js_alloc_string(ctx: *mut JSContext, max_len: i32, is_wide_char: i32) -> *mut JSString {
    let p = js_alloc_string_rt((*ctx).rt, max_len, is_wide_char);
    if p.is_null() {
        JS_ThrowOutOfMemory(ctx);
        return ptr::null_mut();
    }
    p
}
