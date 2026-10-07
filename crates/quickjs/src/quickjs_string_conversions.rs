// quickjs.c:4252-4299,4421-4562. MIT. Depends on the original ToString path.
unsafe fn string_buffer_concat_value(s: *mut StringBuffer, v: JSValueConst) -> i32 {
    if (*s).error_status != 0 {
        return -1;
    }
    if JS_VALUE_GET_TAG(v) != JS_TAG_STRING {
        if JS_VALUE_GET_TAG(v) == JS_TAG_STRING_ROPE {
            let r = JS_VALUE_GET_PTR(v).cast::<JSStringRope>();
            if string_buffer_concat_value(s, (*r).left) != 0 {
                return -1;
            }
            return string_buffer_concat_value(s, (*r).right);
        } else {
            let v1 = JS_ToString((*s).ctx, v);
            if JS_IsException(v1) != 0 {
                return string_buffer_set_error(s);
            }
            let p = JS_VALUE_GET_PTR(v1).cast::<JSString>();
            let res = string_buffer_concat(s, p, 0, (*p).len());
            JS_FreeValue((*s).ctx, v1);
            return res;
        }
    }
    let p = JS_VALUE_GET_PTR(v).cast::<JSString>();
    string_buffer_concat(s, p, 0, (*p).len())
}
unsafe fn string_buffer_concat_value_free(s: *mut StringBuffer, mut v: JSValue) -> i32 {
    if (*s).error_status != 0 {
        JS_FreeValue((*s).ctx, v);
        return -1;
    }
    if JS_VALUE_GET_TAG(v) != JS_TAG_STRING {
        v = JS_ToStringFree((*s).ctx, v);
        if JS_IsException(v) != 0 {
            return string_buffer_set_error(s);
        }
    }
    let p = JS_VALUE_GET_PTR(v).cast::<JSString>();
    let res = string_buffer_concat(s, p, 0, (*p).len());
    JS_FreeValue((*s).ctx, v);
    res
}
unsafe fn JS_ConcatString3(
    ctx: *mut JSContext,
    str1: *const c_char,
    mut str2: JSValue,
    str3: *const c_char,
) -> JSValue {
    if JS_VALUE_GET_TAG(str2) != JS_TAG_STRING {
        str2 = JS_ToStringFree(ctx, str2);
        if JS_IsException(str2) != 0 {
            JS_FreeValue(ctx, str2);
            return JS_EXCEPTION;
        }
    }
    let p = JS_VALUE_GET_PTR(str2).cast::<JSString>();
    let len1 = core::ffi::CStr::from_ptr(str1).to_bytes().len() as i32;
    let len3 = core::ffi::CStr::from_ptr(str3).to_bytes().len() as i32;
    let mut b: StringBuffer = core::mem::zeroed();
    if string_buffer_init2(
        ctx,
        &mut b,
        len1.wrapping_add((*p).len() as i32).wrapping_add(len3),
        (*p).is_wide_char() as i32,
    ) != 0
    {
        JS_FreeValue(ctx, str2);
        return JS_EXCEPTION;
    }
    string_buffer_write8(&mut b, str1.cast(), len1);
    string_buffer_concat(&mut b, p, 0, (*p).len());
    string_buffer_write8(&mut b, str3.cast(), len3);
    JS_FreeValue(ctx, str2);
    string_buffer_end(&mut b)
}
pub unsafe fn JS_NewAtomString(ctx: *mut JSContext, str: *const c_char) -> JSValue {
    let atom = JS_NewAtom(ctx, str);
    if atom == JS_ATOM_NULL as u32 {
        return JS_EXCEPTION;
    }
    let val = JS_AtomToString(ctx, atom);
    JS_FreeAtom(ctx, atom);
    val
}
pub unsafe fn JS_ToCStringLen2(
    ctx: *mut JSContext,
    plen: *mut usize,
    val1: JSValueConst,
    cesu8: JS_BOOL,
) -> *const c_char {
    let val = if JS_VALUE_GET_TAG(val1) != JS_TAG_STRING {
        let v = JS_ToString(ctx, val1);
        if JS_IsException(v) != 0 {
            if !plen.is_null() {
                *plen = 0;
            }
            return ptr::null();
        }
        v
    } else {
        JS_DupValue(ctx, val1)
    };
    let str = JS_VALUE_GET_PTR(val).cast::<JSString>();
    let len = (*str).len() as i32;
    let str_new;
    let mut q;
    if (*str).is_wide_char() == 0 {
        let src = ptr::addr_of!((*str).u).cast::<u8>();
        let mut count = 0i32;
        for pos in 0..len {
            count = count.wrapping_add((*src.offset(pos as isize) >> 7) as i32);
        }
        if count == 0 {
            if !plen.is_null() {
                *plen = len as usize;
            }
            return src.cast();
        }
        str_new = js_alloc_string(ctx, len.wrapping_add(count), 0);
        if str_new.is_null() {
            if !plen.is_null() {
                *plen = 0;
            }
            return ptr::null();
        }
        q = ptr::addr_of_mut!((*str_new).u).cast::<u8>();
        for pos in 0..len {
            let c = *src.offset(pos as isize);
            if c < 0x80 {
                *q = c;
                q = q.add(1);
            } else {
                *q = (c >> 6) | 0xc0;
                q = q.add(1);
                *q = (c & 0x3f) | 0x80;
                q = q.add(1);
            }
        }
    } else {
        let src = ptr::addr_of!((*str).u).cast::<u16>();
        str_new = js_alloc_string(ctx, len.wrapping_mul(3), 0);
        if str_new.is_null() {
            if !plen.is_null() {
                *plen = 0;
            }
            return ptr::null();
        }
        q = ptr::addr_of_mut!((*str_new).u).cast::<u8>();
        let mut pos = 0;
        while pos < len {
            let mut c = *src.offset(pos as isize) as u32;
            pos += 1;
            if c < 0x80 {
                *q = c as u8;
                q = q.add(1);
            } else {
                if is_hi_surrogate(c) != 0 && pos < len && cesu8 == 0 {
                    let c1 = *src.offset(pos as isize) as u32;
                    if is_lo_surrogate(c1) != 0 {
                        pos += 1;
                        c = from_surrogate(c, c1);
                    }
                }
                // C preserves unmatched surrogates rather than replacing them.
                q = q.add(crate::cutils::unicode_to_utf8(q, c) as usize);
            }
        }
    }
    *q = 0;
    (*str_new).set_len(q.offset_from(ptr::addr_of_mut!((*str_new).u).cast::<u8>()) as u32);
    JS_FreeValue(ctx, val);
    if !plen.is_null() {
        *plen = (*str_new).len() as usize;
    }
    ptr::addr_of!((*str_new).u).cast()
    // The C allocation-failure path retains `val`; no extra free is inserted.
}
pub unsafe fn JS_FreeCString(ctx: *mut JSContext, p: *const c_char) {
    if p.is_null() {
        return;
    }
    let str = p
        .cast::<u8>()
        .sub(offset_of!(JSString, u))
        .cast_mut()
        .cast::<JSString>();
    JS_FreeValue(ctx, JS_MKPTR(JS_TAG_STRING, str.cast()));
}
