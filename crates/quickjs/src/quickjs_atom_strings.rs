// quickjs.c:3428-3443,3461-3524,3527-3548,3589-3643,3749-3786.
// Context-backed atom/string API; pending production error/string integration.
use crate::quickjs_atom::JS_ATOM_empty_string;
pub unsafe fn JS_NewAtomLen(ctx: *mut JSContext, str: *const c_char, len: usize) -> JSAtom {
    if len == 0 || (is_digit(*str as i32) == 0 && count_ascii(str.cast(), len) == len) {
        let atom = __JS_FindAtom((*ctx).rt, str, len, JS_ATOM_TYPE_STRING);
        if atom != 0 {
            return atom;
        }
    }
    let val = JS_NewStringLen(ctx, str, len);
    if JS_IsException(val) != 0 {
        return JS_ATOM_NULL as u32;
    }
    JS_NewAtomStr(ctx, JS_VALUE_GET_PTR(val).cast())
}
pub unsafe fn JS_NewAtom(ctx: *mut JSContext, str: *const c_char) -> JSAtom {
    JS_NewAtomLen(ctx, str, core::ffi::CStr::from_ptr(str).to_bytes().len())
}
pub unsafe fn JS_NewAtomUInt32(ctx: *mut JSContext, n: u32) -> JSAtom {
    if n <= JS_ATOM_MAX_INT {
        return __JS_AtomFromUInt32(n);
    }
    let mut buf = [0i8; 11];
    let len = crate::dtoa::u32toa(buf.as_mut_ptr(), n);
    let val = js_new_string8_len(ctx, buf.as_ptr(), len as i32);
    if JS_IsException(val) != 0 {
        return JS_ATOM_NULL as u32;
    }
    __JS_NewAtom((*ctx).rt, JS_VALUE_GET_PTR(val).cast(), JS_ATOM_TYPE_STRING)
}
unsafe fn JS_NewAtomInt64(ctx: *mut JSContext, n: i64) -> JSAtom {
    if n as u64 <= JS_ATOM_MAX_INT as u64 {
        return __JS_AtomFromUInt32(n as u32);
    }
    let mut buf = [0i8; 24];
    let len = crate::dtoa::i64toa(buf.as_mut_ptr(), n);
    let val = js_new_string8_len(ctx, buf.as_ptr(), len as i32);
    if JS_IsException(val) != 0 {
        return JS_ATOM_NULL as u32;
    }
    __JS_NewAtom((*ctx).rt, JS_VALUE_GET_PTR(val).cast(), JS_ATOM_TYPE_STRING)
}
unsafe fn JS_NewSymbol(ctx: *mut JSContext, p: *mut JSString, atom_type: u32) -> JSValue {
    let rt = (*ctx).rt;
    let atom = __JS_NewAtom(rt, p, atom_type);
    if atom == JS_ATOM_NULL as u32 {
        return JS_ThrowOutOfMemory(ctx);
    }
    JS_MKPTR(JS_TAG_SYMBOL, (*(*rt).atom_array.add(atom as usize)).cast())
}
unsafe fn JS_NewSymbolFromAtom(ctx: *mut JSContext, descr: JSAtom, atom_type: u32) -> JSValue {
    let rt = (*ctx).rt;
    assert!(__JS_AtomIsTaggedInt(descr) == 0);
    assert!(descr < (*rt).atom_size as u32);
    let p = *(*rt).atom_array.add(descr as usize);
    JS_DupValue(ctx, JS_MKPTR(JS_TAG_STRING, p.cast()));
    JS_NewSymbol(ctx, p, atom_type)
}
const ATOM_GET_STR_BUF_SIZE: usize = 64;
// quickjs.c:3548-3596. Debug/error spelling without property conversion.
unsafe fn JS_AtomGetStrRT(
    rt: *mut JSRuntime,
    buf: *mut c_char,
    buf_size: i32,
    atom: JSAtom,
) -> *const c_char {
    if __JS_AtomIsTaggedInt(atom) != 0 {
        let mut digit_buf = [0i8; 11];
        let len = crate::dtoa::u32toa(digit_buf.as_mut_ptr(), __JS_AtomToUInt32(atom));
        if buf_size != 0 {
            let n = len.min((buf_size as usize).wrapping_sub(1));
            ptr::copy_nonoverlapping(digit_buf.as_ptr(), buf, n);
            *buf.add(n) = 0;
        }
    } else {
        assert!(atom < (*rt).atom_size as u32);
        if atom == JS_ATOM_NULL as u32 {
            if buf_size != 0 {
                let n = 6usize.min((buf_size as usize).wrapping_sub(1));
                ptr::copy_nonoverlapping(c"<null>".as_ptr(), buf, n);
                *buf.add(n) = 0;
            }
        } else {
            let p = *(*rt).atom_array.add(atom as usize);
            assert!(atom_is_free(p) == 0);
            let mut q = buf;
            if !p.is_null() {
                if (*p).is_wide_char() == 0 {
                    let mut c = 0;
                    for i in 0..(*p).len() {
                        c |= *ptr::addr_of!((*p).u).cast::<u8>().add(i as usize);
                    }
                    if c < 0x80 {
                        return ptr::addr_of!((*p).u).cast();
                    }
                }
                for i in 0..(*p).len() {
                    let c = string_get(p, i as i32);
                    if q.offset_from(buf)
                        >= buf_size as isize - crate::cutils_header::UTF8_CHAR_LEN_MAX as isize
                    {
                        break;
                    }
                    if c < 128 {
                        *q = c as c_char;
                        q = q.add(1);
                    } else {
                        q = q.add(crate::cutils::unicode_to_utf8(q.cast(), c as u32) as usize);
                    }
                }
            }
            *q = 0;
        }
    }
    buf
}
unsafe fn JS_AtomGetStr(
    ctx: *mut JSContext,
    buf: *mut c_char,
    buf_size: i32,
    atom: JSAtom,
) -> *const c_char {
    JS_AtomGetStrRT((*ctx).rt, buf, buf_size, atom)
}
unsafe fn __JS_AtomToValue(ctx: *mut JSContext, atom: JSAtom, force_string: JS_BOOL) -> JSValue {
    if __JS_AtomIsTaggedInt(atom) != 0 {
        let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
        let len = crate::dtoa::u32toa(buf.as_mut_ptr(), __JS_AtomToUInt32(atom));
        js_new_string8_len(ctx, buf.as_ptr(), len as i32)
    } else {
        let rt = (*ctx).rt;
        assert!(atom < (*rt).atom_size as u32);
        let mut p = *(*rt).atom_array.add(atom as usize);
        if (*p).atom_type() == JS_ATOM_TYPE_STRING as u32 || force_string != 0 {
            if (*p).atom_type() != JS_ATOM_TYPE_STRING as u32
                && (*p).len() == 0
                && (*p).is_wide_char() != 0
            {
                p = *(*rt).atom_array.add(JS_ATOM_empty_string as usize);
            }
            JS_DupValue(ctx, JS_MKPTR(JS_TAG_STRING, p.cast()))
        } else {
            JS_DupValue(ctx, JS_MKPTR(JS_TAG_SYMBOL, p.cast()))
        }
    }
}
pub unsafe fn JS_AtomToValue(ctx: *mut JSContext, atom: JSAtom) -> JSValue {
    __JS_AtomToValue(ctx, atom, 0)
}
pub unsafe fn JS_AtomToString(ctx: *mut JSContext, atom: JSAtom) -> JSValue {
    __JS_AtomToValue(ctx, atom, 1)
}
unsafe fn JS_AtomIsArrayIndex(ctx: *mut JSContext, pval: *mut u32, atom: JSAtom) -> JS_BOOL {
    if __JS_AtomIsTaggedInt(atom) != 0 {
        *pval = __JS_AtomToUInt32(atom);
        return 1;
    }
    let rt = (*ctx).rt;
    assert!(atom < (*rt).atom_size as u32);
    let p = *(*rt).atom_array.add(atom as usize);
    let mut val = 0;
    if (*p).atom_type() == JS_ATOM_TYPE_STRING as u32
        && is_num_string(&mut val, p) != 0
        && val != u32::MAX
    {
        *pval = val;
        1
    } else {
        *pval = 0;
        0
    }
}
pub unsafe fn JS_AtomToCStringLen(
    ctx: *mut JSContext,
    plen: *mut usize,
    atom: JSAtom,
) -> *const c_char {
    let str = JS_AtomToString(ctx, atom);
    if JS_IsException(str) != 0 {
        if !plen.is_null() {
            *plen = 0;
        }
        return ptr::null();
    }
    let cstr = JS_ToCStringLen2(ctx, plen, str, 0);
    JS_FreeValue(ctx, str);
    cstr
}
unsafe fn js_atom_concat_str(ctx: *mut JSContext, name: JSAtom, str1: *const c_char) -> JSAtom {
    let str = JS_AtomToString(ctx, name);
    if JS_IsException(str) != 0 {
        return JS_ATOM_NULL as u32;
    }
    let mut len = 0;
    let cstr = JS_ToCStringLen2(ctx, &mut len, str, 0);
    if cstr.is_null() {
        JS_FreeCString(ctx, cstr);
        JS_FreeValue(ctx, str);
        return JS_ATOM_NULL as u32;
    }
    let len1 = core::ffi::CStr::from_ptr(str1).to_bytes().len();
    let cstr2 = js_malloc(ctx, len.wrapping_add(len1).wrapping_add(1)).cast::<c_char>();
    if cstr2.is_null() {
        JS_FreeCString(ctx, cstr);
        JS_FreeValue(ctx, str);
        return JS_ATOM_NULL as u32;
    }
    ptr::copy_nonoverlapping(cstr, cstr2, len);
    ptr::copy_nonoverlapping(str1, cstr2.add(len), len1);
    *cstr2.add(len.wrapping_add(len1)) = 0;
    let atom = JS_NewAtomLen(ctx, cstr2, len.wrapping_add(len1));
    js_free(ctx, cstr2.cast());
    JS_FreeCString(ctx, cstr);
    JS_FreeValue(ctx, str);
    atom
}
unsafe fn js_atom_concat_num(ctx: *mut JSContext, name: JSAtom, n: u32) -> JSAtom {
    let mut buf = [0i8; 16];
    let len = crate::dtoa::u32toa(buf.as_mut_ptr(), n);
    buf[len] = 0;
    js_atom_concat_str(ctx, name, buf.as_ptr())
}
