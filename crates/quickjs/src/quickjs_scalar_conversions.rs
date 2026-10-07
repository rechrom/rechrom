// quickjs.c:12742-13032,13557-13668,4834-4864. MIT.
// Staged with original ToPrimitive/VM and BigInt parse/format dependencies.
const ATOD_INT_ONLY: i32 = 1 << 0;
const ATOD_ACCEPT_BIN_OCT: i32 = 1 << 2;
const ATOD_ACCEPT_LEGACY_OCTAL: i32 = 1 << 4;
const ATOD_ACCEPT_UNDERSCORES: i32 = 1 << 5;
const ATOD_ACCEPT_SUFFIX: i32 = 1 << 6;
const ATOD_TYPE_MASK: i32 = 3 << 7;
const ATOD_TYPE_FLOAT64: i32 = 0 << 7;
const ATOD_TYPE_BIG_INT: i32 = 1 << 7;
const ATOD_ACCEPT_PREFIX_AFTER_SIGN: i32 = 1 << 10;
#[allow(unused_assignments)] // Retain C sign/prefix parsing assignments.
unsafe fn js_atof(
    ctx: *mut JSContext,
    str: *const c_char,
    pp: *mut *const c_char,
    mut radix: i32,
    flags: i32,
) -> JSValue {
    let mut p = str;
    let mut p_start = p;
    let mut sep = if flags & ATOD_ACCEPT_UNDERSCORES != 0 {
        b'_' as i32
    } else {
        256
    };
    let mut has_legacy_octal = false;
    let mut is_neg = false;
    let mut atod_type = flags & ATOD_TYPE_MASK;
    let mut buf1 = [0i8; 64];
    let mut allocated: *mut c_char = ptr::null_mut();
    let val = 'parse: {
        let mut allow_prefix = true;
        if *p == b'+' as c_char {
            p = p.add(1);
            p_start = p_start.add(1);
            if flags & ATOD_ACCEPT_PREFIX_AFTER_SIGN == 0 {
                allow_prefix = false;
            }
        } else if *p == b'-' as c_char {
            p = p.add(1);
            p_start = p_start.add(1);
            is_neg = true;
            if flags & ATOD_ACCEPT_PREFIX_AFTER_SIGN == 0 {
                allow_prefix = false;
            }
        }
        if allow_prefix && *p == b'0' as c_char {
            let mut prefix = true;
            if matches!(*p.add(1) as u8, b'x' | b'X') && (radix == 0 || radix == 16) {
                p = p.add(2);
                radix = 16;
            } else if matches!(*p.add(1) as u8, b'o' | b'O')
                && radix == 0
                && flags & ATOD_ACCEPT_BIN_OCT != 0
            {
                p = p.add(2);
                radix = 8;
            } else if matches!(*p.add(1) as u8, b'b' | b'B')
                && radix == 0
                && flags & ATOD_ACCEPT_BIN_OCT != 0
            {
                p = p.add(2);
                radix = 2;
            } else if (*p.add(1) as u8) >= b'0'
                && (*p.add(1) as u8) <= b'9'
                && radix == 0
                && flags & ATOD_ACCEPT_LEGACY_OCTAL != 0
            {
                has_legacy_octal = true;
                sep = 256;
                let mut i = 1;
                while (*p.add(i) as u8) >= b'0' && (*p.add(i) as u8) <= b'7' {
                    i += 1;
                }
                if matches!(*p.add(i) as u8, b'8' | b'9') {
                    prefix = false;
                } else {
                    p = p.add(1);
                    radix = 8;
                }
            } else {
                prefix = false;
            }
            if prefix && to_digit(*p as u8 as i32) >= radix {
                break 'parse JS_NAN;
            }
        } else if flags & ATOD_INT_ONLY == 0 && atod_type == ATOD_TYPE_FLOAT64 {
            if crate::cutils::strstart(p, c"Infinity".as_ptr(), &mut p) != 0 {
                break 'parse JS_NewFloat64(
                    ctx,
                    if is_neg {
                        f64::NEG_INFINITY
                    } else {
                        f64::INFINITY
                    },
                );
            }
        }
        if radix == 0 {
            radix = 10;
        }
        let mut is_float = false;
        p_start = p;
        while to_digit(*p as u8 as i32) < radix
            || (*p as i32 == sep
                && (radix != 10 || p != p_start.add(1) || *p.sub(1) != b'0' as c_char)
                && to_digit(*p.add(1) as u8 as i32) < radix)
        {
            p = p.add(1);
        }
        if flags & ATOD_INT_ONLY == 0 && radix == 10 {
            if *p == b'.' as c_char && (p > p_start || to_digit(*p.add(1) as u8 as i32) < radix) {
                is_float = true;
                p = p.add(1);
                if *p as i32 == sep {
                    break 'parse JS_NAN;
                }
                while to_digit(*p as u8 as i32) < radix
                    || (*p as i32 == sep && to_digit(*p.add(1) as u8 as i32) < radix)
                {
                    p = p.add(1);
                }
            }
            if p > p_start && matches!(*p as u8, b'e' | b'E') {
                let mut p1 = p.add(1);
                is_float = true;
                if matches!(*p1 as u8, b'+' | b'-') {
                    p1 = p1.add(1);
                }
                if is_digit(*p1 as u8 as i32) != 0 {
                    p = p1.add(1);
                    while is_digit(*p as u8 as i32) != 0
                        || (*p as i32 == sep && is_digit(*p.add(1) as u8 as i32) != 0)
                    {
                        p = p.add(1);
                    }
                }
            }
        }
        if p == p_start {
            break 'parse JS_NAN;
        }
        let len = p.offset_from(p_start) as i32;
        let buf;
        if len.wrapping_add(2) as usize > buf1.len() {
            allocated = js_malloc_rt((*ctx).rt, len.wrapping_add(2) as usize).cast();
            if allocated.is_null() {
                break 'parse JS_ThrowOutOfMemory(ctx);
            }
            buf = allocated;
        } else {
            buf = buf1.as_mut_ptr();
        }
        let mut j = 0;
        if is_neg {
            *buf = b'-' as c_char;
            j += 1;
        }
        for i in 0..len {
            if *p_start.offset(i as isize) != b'_' as c_char {
                *buf.add(j) = *p_start.offset(i as isize);
                j += 1;
            }
        }
        *buf.add(j) = 0;
        if flags & ATOD_ACCEPT_SUFFIX != 0 && *p == b'n' as c_char {
            p = p.add(1);
            atod_type = ATOD_TYPE_BIG_INT;
        }
        match atod_type {
            ATOD_TYPE_FLOAT64 => {
                let mut atod_mem = crate::dtoa_header::JSATODTempMem::default();
                let d = crate::dtoa::js_atod(
                    buf,
                    ptr::null_mut(),
                    radix,
                    if is_float {
                        0
                    } else {
                        crate::dtoa_header::JS_ATOD_INT_ONLY
                    },
                    &mut atod_mem,
                );
                JS_NewFloat64(ctx, d)
            }
            ATOD_TYPE_BIG_INT => {
                if has_legacy_octal || is_float {
                    break 'parse JS_NAN;
                }
                let r = js_bigint_from_string(ctx, buf, radix);
                if r.is_null() {
                    JS_EXCEPTION
                } else {
                    JS_CompactBigInt(ctx, r)
                }
            }
            _ => std::process::abort(),
        }
    };
    if !allocated.is_null() {
        js_free_rt((*ctx).rt, allocated.cast());
    }
    if !pp.is_null() {
        *pp = p;
    }
    val
}
type JSToNumberHintEnum = u32;
const TON_FLAG_NUMBER: JSToNumberHintEnum = 0;
const TON_FLAG_NUMERIC: JSToNumberHintEnum = 1;
unsafe fn JS_ToNumberHintFree(
    ctx: *mut JSContext,
    mut val: JSValue,
    flag: JSToNumberHintEnum,
) -> JSValue {
    loop {
        return match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_BIG_INT | JS_TAG_SHORT_BIG_INT => {
                if flag != TON_FLAG_NUMERIC {
                    JS_FreeValue(ctx, val);
                    JS_ThrowTypeError(ctx, c"cannot convert bigint to number".as_ptr())
                } else {
                    val
                }
            }
            JS_TAG_FLOAT64 | JS_TAG_INT | JS_TAG_EXCEPTION => val,
            JS_TAG_BOOL | JS_TAG_NULL => JS_NewInt32(ctx, JS_VALUE_GET_INT(val)),
            JS_TAG_UNDEFINED => JS_NAN,
            JS_TAG_OBJECT => {
                val = JS_ToPrimitiveFree(ctx, val, HINT_NUMBER);
                if JS_IsException(val) != 0 {
                    return JS_EXCEPTION;
                }
                continue;
            }
            JS_TAG_STRING | JS_TAG_STRING_ROPE => {
                let mut len = 0;
                let str = JS_ToCStringLen2(ctx, &mut len, val, 0);
                JS_FreeValue(ctx, val);
                if str.is_null() {
                    return JS_EXCEPTION;
                }
                let mut p = str;
                p = p.offset(skip_spaces(p) as isize);
                let mut ret = if p.offset_from(str) as usize == len {
                    JS_NewInt32(ctx, 0)
                } else {
                    js_atof(ctx, p, &mut p, 0, ATOD_ACCEPT_BIN_OCT)
                };
                if p.offset_from(str) as usize != len && JS_IsException(ret) == 0 {
                    p = p.offset(skip_spaces(p) as isize);
                    if p.offset_from(str) as usize != len {
                        JS_FreeValue(ctx, ret);
                        ret = JS_NAN;
                    }
                }
                JS_FreeCString(ctx, str);
                ret
            }
            JS_TAG_SYMBOL => {
                JS_FreeValue(ctx, val);
                JS_ThrowTypeError(ctx, c"cannot convert symbol to number".as_ptr())
            }
            _ => {
                JS_FreeValue(ctx, val);
                JS_NAN
            }
        };
    }
}
unsafe fn JS_ToNumberFree(ctx: *mut JSContext, val: JSValue) -> JSValue {
    JS_ToNumberHintFree(ctx, val, TON_FLAG_NUMBER)
}
unsafe fn JS_ToNumericFree(ctx: *mut JSContext, val: JSValue) -> JSValue {
    JS_ToNumberHintFree(ctx, val, TON_FLAG_NUMERIC)
}
unsafe fn JS_ToNumeric(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    JS_ToNumericFree(ctx, JS_DupValue(ctx, val))
}
unsafe fn js_linearize_string_rope(ctx: *mut JSContext, rope: JSValue) -> JSValue {
    let r = JS_VALUE_GET_PTR(rope).cast::<JSStringRope>();
    if JS_VALUE_GET_TAG((*r).right) == JS_TAG_STRING
        && (*JS_VALUE_GET_PTR((*r).right).cast::<JSString>()).len() == 0
    {
        let ret = JS_DupValue(ctx, (*r).left);
        JS_FreeValue(ctx, rope);
        return ret;
    }
    let mut b: StringBuffer = core::mem::zeroed();
    if string_buffer_init2(ctx, &mut b, (*r).len as i32, (*r).is_wide_char as i32) != 0 {
        JS_FreeValue(ctx, rope);
        return JS_EXCEPTION;
    }
    if string_buffer_concat_value(&mut b, rope) != 0 {
        JS_FreeValue(ctx, rope);
        return JS_EXCEPTION;
    }
    let ret = string_buffer_end(&mut b);
    if (*js_rc(r.cast())).ref_count > 1 {
        JS_FreeValue(ctx, (*r).left);
        JS_FreeValue(ctx, (*r).right);
        (*r).left = JS_DupValue(ctx, ret);
        (*r).right = JS_AtomToString(ctx, JS_ATOM_empty_string);
    }
    JS_FreeValue(ctx, rope);
    ret
}
unsafe fn js_dtoa2(ctx: *mut JSContext, d: f64, radix: i32, n_digits: i32, flags: i32) -> JSValue {
    let mut static_buf = [0i8; 128];
    let len_max = crate::dtoa::js_dtoa_max_len(d, radix, n_digits, flags);
    let mut tmp_buf: *mut c_char = ptr::null_mut();
    let buf;
    if len_max > static_buf.len() as i32 - 1 {
        tmp_buf = js_malloc(ctx, len_max.wrapping_add(1) as usize).cast();
        if tmp_buf.is_null() {
            return JS_EXCEPTION;
        }
        buf = tmp_buf;
    } else {
        buf = static_buf.as_mut_ptr();
    }
    let mut dtoa_mem = crate::dtoa_header::JSDTOATempMem::default();
    let len = crate::dtoa::js_dtoa(buf, d, radix, n_digits, flags, &mut dtoa_mem);
    let res = js_new_string8_len(ctx, buf, len);
    js_free(ctx, tmp_buf.cast());
    res
}
unsafe fn js_bigint_to_string(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    js_bigint_to_string1(ctx, val, 10)
}
unsafe fn JS_ToStringInternal(
    ctx: *mut JSContext,
    val: JSValueConst,
    is_ToPropertyKey: JS_BOOL,
) -> JSValue {
    use crate::quickjs_atom::{JS_ATOM_false, JS_ATOM_null, JS_ATOM_true, JS_ATOM_undefined};
    match JS_VALUE_GET_NORM_TAG(val) {
        JS_TAG_STRING => JS_DupValue(ctx, val),
        JS_TAG_STRING_ROPE => js_linearize_string_rope(ctx, JS_DupValue(ctx, val)),
        JS_TAG_INT => {
            let mut buf = [0i8; 32];
            let len = crate::dtoa::i32toa(buf.as_mut_ptr(), JS_VALUE_GET_INT(val));
            js_new_string8_len(ctx, buf.as_ptr(), len as i32)
        }
        JS_TAG_BOOL => JS_AtomToString(
            ctx,
            if JS_VALUE_GET_BOOL(val) != 0 {
                JS_ATOM_true
            } else {
                JS_ATOM_false
            },
        ),
        JS_TAG_NULL => JS_AtomToString(ctx, JS_ATOM_null),
        JS_TAG_UNDEFINED => JS_AtomToString(ctx, JS_ATOM_undefined),
        JS_TAG_EXCEPTION => JS_EXCEPTION,
        JS_TAG_OBJECT => {
            let val1 = JS_ToPrimitive(ctx, val, HINT_STRING);
            if JS_IsException(val1) != 0 {
                return val1;
            }
            let ret = JS_ToStringInternal(ctx, val1, is_ToPropertyKey);
            JS_FreeValue(ctx, val1);
            ret
        }
        JS_TAG_FUNCTION_BYTECODE => js_new_string8(ctx, c"[function bytecode]".as_ptr()),
        JS_TAG_SYMBOL => {
            if is_ToPropertyKey != 0 {
                JS_DupValue(ctx, val)
            } else {
                JS_ThrowTypeError(ctx, c"cannot convert symbol to string".as_ptr())
            }
        }
        JS_TAG_FLOAT64 => js_dtoa2(
            ctx,
            JS_VALUE_GET_FLOAT64(val),
            10,
            0,
            crate::dtoa_header::JS_DTOA_FORMAT_FREE,
        ),
        JS_TAG_SHORT_BIG_INT | JS_TAG_BIG_INT => js_bigint_to_string(ctx, val),
        _ => js_new_string8(ctx, c"[unsupported type]".as_ptr()),
    }
}
pub unsafe fn JS_ToString(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    JS_ToStringInternal(ctx, val, 0)
}
unsafe fn JS_ToStringFree(ctx: *mut JSContext, val: JSValue) -> JSValue {
    let ret = JS_ToString(ctx, val);
    JS_FreeValue(ctx, val);
    ret
}
pub unsafe fn JS_ToPropertyKey(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    JS_ToStringInternal(ctx, val, 1)
}
