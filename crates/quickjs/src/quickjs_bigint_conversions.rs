// quickjs.c:14612-14724. MIT. Staged with original ToPrimitive/call path.
unsafe fn JS_StringToBigInt(ctx: *mut JSContext, val: JSValue) -> JSValue {
    let mut len = 0;
    let str = JS_ToCStringLen2(ctx, &mut len, val, 0);
    JS_FreeValue(ctx, val);
    if str.is_null() {
        return JS_EXCEPTION;
    }
    let mut p = str;
    p = p.offset(skip_spaces(p) as isize);
    let ret = if p.offset_from(str) as usize == len {
        JS_NewBigInt64(ctx, 0)
    } else {
        let mut val = js_atof(
            ctx,
            p,
            &mut p,
            0,
            ATOD_INT_ONLY | ATOD_ACCEPT_BIN_OCT | ATOD_TYPE_BIG_INT,
        );
        p = p.offset(skip_spaces(p) as isize);
        if JS_IsException(val) == 0 && p.offset_from(str) as usize != len {
            JS_FreeValue(ctx, val);
            val = JS_NAN;
        }
        val
    };
    JS_FreeCString(ctx, str);
    ret
}
unsafe fn JS_StringToBigIntErr(ctx: *mut JSContext, val: JSValue) -> JSValue {
    let val = JS_StringToBigInt(ctx, val);
    if JS_VALUE_IS_NAN(val) != 0 {
        JS_ThrowSyntaxError(ctx, c"invalid bigint literal".as_ptr())
    } else {
        val
    }
}
unsafe fn JS_ToBigIntFree(ctx: *mut JSContext, mut val: JSValue) -> JSValue {
    loop {
        match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_SHORT_BIG_INT | JS_TAG_BIG_INT => return val,
            JS_TAG_BOOL => return __JS_NewShortBigInt(ctx, JS_VALUE_GET_INT(val) as js_slimb_t),
            JS_TAG_STRING | JS_TAG_STRING_ROPE => {
                val = JS_StringToBigIntErr(ctx, val);
                if JS_IsException(val) != 0 {
                    return val;
                }
            }
            JS_TAG_OBJECT => {
                val = JS_ToPrimitiveFree(ctx, val, HINT_NUMBER);
                if JS_IsException(val) != 0 {
                    return val;
                }
            }
            _ => {
                JS_FreeValue(ctx, val);
                return JS_ThrowTypeError(ctx, c"cannot convert to bigint".as_ptr());
            }
        }
    }
}
unsafe fn JS_ToBigInt(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    JS_ToBigIntFree(ctx, JS_DupValue(ctx, val))
}
unsafe fn JS_ToBigInt64Free(ctx: *mut JSContext, pres: *mut i64, val: JSValue) -> i32 {
    let val = JS_ToBigIntFree(ctx, val);
    if JS_IsException(val) != 0 {
        *pres = 0;
        return -1;
    }
    let res = if JS_VALUE_GET_TAG(val) == JS_TAG_SHORT_BIG_INT {
        JS_VALUE_GET_SHORT_BIG_INT(val) as u64
    } else {
        let p = JS_VALUE_GET_PTR(val).cast::<JSBigInt>();
        #[allow(unused_mut)]
        let mut res = *bigint_tab(p) as u64;
        #[cfg(target_pointer_width = "32")]
        {
            if (*p).len >= 2 {
                res |= (*bigint_tab(p).add(1) as u64) << 32;
            }
        }
        JS_FreeValue(ctx, val);
        res
    };
    *pres = res as i64;
    0
}
pub unsafe fn JS_ToBigInt64(ctx: *mut JSContext, pres: *mut i64, val: JSValueConst) -> i32 {
    JS_ToBigInt64Free(ctx, pres, JS_DupValue(ctx, val))
}
