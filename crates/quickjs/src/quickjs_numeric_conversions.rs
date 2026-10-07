// quickjs.c:13092-13318,13381-13422,13475-13515. MIT.
// Staged together with the full original number/ToPrimitive/call path.
unsafe fn JS_ToIntegerFree(ctx: *mut JSContext, mut val: JSValue) -> JSValue {
    loop {
        return match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => {
                JS_NewInt32(ctx, JS_VALUE_GET_INT(val))
            }
            JS_TAG_FLOAT64 => {
                let d = JS_VALUE_GET_FLOAT64(val);
                if d.is_nan() {
                    JS_NewInt32(ctx, 0)
                } else {
                    JS_NewFloat64(ctx, d.trunc() + 0.0)
                }
            }
            _ => {
                val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    return val;
                }
                continue;
            }
        };
    }
}
unsafe fn JS_ToInt32SatFree(ctx: *mut JSContext, pres: *mut i32, mut val: JSValue) -> i32 {
    loop {
        *pres = match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => JS_VALUE_GET_INT(val),
            JS_TAG_EXCEPTION => {
                *pres = 0;
                return -1;
            }
            JS_TAG_FLOAT64 => {
                let d = JS_VALUE_GET_FLOAT64(val);
                if d.is_nan() {
                    0
                } else if d < (i32::MIN as f64) {
                    i32::MIN
                } else if d > i32::MAX as f64 {
                    i32::MAX
                } else {
                    d as i32
                }
            }
            _ => {
                val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    *pres = 0;
                    return -1;
                }
                continue;
            }
        };
        return 0;
    }
}
pub unsafe fn JS_ToInt32Sat(ctx: *mut JSContext, pres: *mut i32, val: JSValueConst) -> i32 {
    JS_ToInt32SatFree(ctx, pres, JS_DupValue(ctx, val))
}
pub unsafe fn JS_ToInt32Clamp(
    ctx: *mut JSContext,
    pres: *mut i32,
    val: JSValueConst,
    min: i32,
    max: i32,
    min_offset: i32,
) -> i32 {
    let res = JS_ToInt32SatFree(ctx, pres, JS_DupValue(ctx, val));
    if res == 0 {
        if *pres < min {
            *pres = (*pres).wrapping_add(min_offset);
            if *pres < min {
                *pres = min;
            }
        } else if *pres > max {
            *pres = max;
        }
    }
    res
}
unsafe fn JS_ToInt64SatFree(ctx: *mut JSContext, pres: *mut i64, mut val: JSValue) -> i32 {
    loop {
        *pres = match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => {
                JS_VALUE_GET_INT(val) as i64
            }
            JS_TAG_EXCEPTION => {
                *pres = 0;
                return -1;
            }
            JS_TAG_FLOAT64 => {
                let d = JS_VALUE_GET_FLOAT64(val);
                if d.is_nan() {
                    0
                } else if d < (i64::MIN as f64) {
                    i64::MIN
                } else if d >= 9223372036854775808.0 {
                    i64::MAX
                } else {
                    d as i64
                }
            }
            _ => {
                val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    *pres = 0;
                    return -1;
                }
                continue;
            }
        };
        return 0;
    }
}
pub unsafe fn JS_ToInt64Sat(ctx: *mut JSContext, pres: *mut i64, val: JSValueConst) -> i32 {
    JS_ToInt64SatFree(ctx, pres, JS_DupValue(ctx, val))
}
pub unsafe fn JS_ToInt64Clamp(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValueConst,
    min: i64,
    max: i64,
    neg_offset: i64,
) -> i32 {
    let res = JS_ToInt64SatFree(ctx, pres, JS_DupValue(ctx, val));
    if res == 0 {
        if *pres < 0 {
            *pres = (*pres).wrapping_add(neg_offset);
        }
        if *pres < min {
            *pres = min;
        } else if *pres > max {
            *pres = max;
        }
    }
    res
}
unsafe fn JS_ToInt64Free(ctx: *mut JSContext, pres: *mut i64, mut val: JSValue) -> i32 {
    loop {
        *pres = match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => {
                JS_VALUE_GET_INT(val) as i64
            }
            JS_TAG_FLOAT64 => {
                let d = JS_VALUE_GET_FLOAT64(val);
                let bits = d.to_bits();
                let e = ((bits >> 52) & 0x7ff) as i32;
                if e <= 1023 + 62 {
                    d as i64
                } else if e <= 1023 + 62 + 53 {
                    let v = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
                    let ret = (v << ((e - 1023) - 52)) as i64;
                    if bits >> 63 != 0 {
                        ret.wrapping_neg()
                    } else {
                        ret
                    }
                } else {
                    0
                }
            }
            _ => {
                val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    *pres = 0;
                    return -1;
                }
                continue;
            }
        };
        return 0;
    }
}
pub unsafe fn JS_ToInt64(ctx: *mut JSContext, pres: *mut i64, val: JSValueConst) -> i32 {
    JS_ToInt64Free(ctx, pres, JS_DupValue(ctx, val))
}
pub unsafe fn JS_ToInt64Ext(ctx: *mut JSContext, pres: *mut i64, val: JSValueConst) -> i32 {
    if JS_IsBigInt(ctx, val) != 0 {
        JS_ToBigInt64(ctx, pres, val)
    } else {
        JS_ToInt64(ctx, pres, val)
    }
}
unsafe fn JS_ToUint8ClampFree(ctx: *mut JSContext, pres: *mut i32, mut val: JSValue) -> i32 {
    loop {
        *pres = match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => {
                JS_VALUE_GET_INT(val).clamp(0, 255)
            }
            JS_TAG_FLOAT64 => {
                let d = JS_VALUE_GET_FLOAT64(val);
                if d.is_nan() || d < 0.0 {
                    0
                } else if d > 255.0 {
                    255
                } else {
                    d.round_ties_even() as i32
                }
            }
            _ => {
                val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    *pres = 0;
                    return -1;
                }
                continue;
            }
        };
        return 0;
    }
}
pub unsafe fn JS_ToIndex(ctx: *mut JSContext, plen: *mut u64, val: JSValueConst) -> i32 {
    let mut v = 0;
    if JS_ToInt64Sat(ctx, &mut v, val) != 0 {
        return -1;
    }
    if v < 0 || v > MAX_SAFE_INTEGER {
        JS_ThrowRangeError(ctx, c"invalid array index".as_ptr());
        *plen = 0;
        return -1;
    }
    *plen = v as u64;
    0
}
unsafe fn JS_ToLengthFree(ctx: *mut JSContext, plen: *mut i64, val: JSValue) -> i32 {
    let res = JS_ToInt64Clamp(ctx, plen, val, 0, MAX_SAFE_INTEGER, 0);
    JS_FreeValue(ctx, val);
    res
}
