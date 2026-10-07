// quickjs.c:9003-9032,13034-13090,13319-13380,13423-13483,
// 13518-13557. MIT. Remaining ToPrimitive/number-parser/ToString dependencies
// keep this source staged until full runtime conversion integration.
unsafe fn __JS_ToFloat64Free(ctx: *mut JSContext, pres: *mut f64, val: JSValue) -> i32 {
    let val = JS_ToNumberFree(ctx, val);
    if JS_IsException(val) != 0 {
        *pres = JS_FLOAT64_NAN;
        return -1;
    }
    *pres = match JS_VALUE_GET_NORM_TAG(val) {
        JS_TAG_INT => JS_VALUE_GET_INT(val) as f64,
        JS_TAG_FLOAT64 => JS_VALUE_GET_FLOAT64(val),
        _ => std::process::abort(),
    };
    0
}
#[inline]
unsafe fn JS_ToFloat64Free(ctx: *mut JSContext, pres: *mut f64, val: JSValue) -> i32 {
    let tag = JS_VALUE_GET_TAG(val) as u32;
    if tag <= JS_TAG_NULL as u32 {
        *pres = JS_VALUE_GET_INT(val) as f64;
        0
    } else if JS_TAG_IS_FLOAT64(tag as i32) != 0 {
        *pres = JS_VALUE_GET_FLOAT64(val);
        0
    } else {
        __JS_ToFloat64Free(ctx, pres, val)
    }
}
pub unsafe fn JS_ToFloat64(ctx: *mut JSContext, pres: *mut f64, val: JSValueConst) -> i32 {
    JS_ToFloat64Free(ctx, pres, JS_DupValue(ctx, val))
}
unsafe fn JS_ToNumber(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    JS_ToNumberFree(ctx, JS_DupValue(ctx, val))
}
unsafe fn JS_ToInt32Free(ctx: *mut JSContext, pres: *mut i32, mut val: JSValue) -> i32 {
    loop {
        let ret = match JS_VALUE_GET_NORM_TAG(val) {
            JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => JS_VALUE_GET_INT(val),
            JS_TAG_FLOAT64 => {
                let d = JS_VALUE_GET_FLOAT64(val);
                let u = d.to_bits();
                let e = ((u >> 52) & 0x7ff) as i32;
                if e <= 1023 + 30 {
                    d as i32
                } else if e <= 1023 + 30 + 53 {
                    let v = (u & ((1u64 << 52) - 1)) | (1u64 << 52);
                    let v = v << ((e - 1023) - 52 + 32);
                    let ret = (v >> 32) as i32;
                    if u >> 63 != 0 {
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
        *pres = ret;
        return 0;
    }
}
pub unsafe fn JS_ToInt32(ctx: *mut JSContext, pres: *mut i32, val: JSValueConst) -> i32 {
    JS_ToInt32Free(ctx, pres, JS_DupValue(ctx, val))
}
#[inline]
unsafe fn JS_ToUint32Free(ctx: *mut JSContext, pres: *mut u32, val: JSValue) -> i32 {
    JS_ToInt32Free(ctx, pres.cast(), val)
}
// quickjs.h runtime-backed inline wrapper.
pub unsafe fn JS_ToUint32(ctx: *mut JSContext, pres: *mut u32, val: JSValueConst) -> i32 {
    JS_ToInt32(ctx, pres.cast(), val)
}
unsafe fn JS_ToArrayLengthFree(
    ctx: *mut JSContext,
    plen: *mut u32,
    val: JSValue,
    is_array_ctor: JS_BOOL,
) -> i32 {
    let len = match JS_VALUE_GET_TAG(val) {
        JS_TAG_INT | JS_TAG_BOOL | JS_TAG_NULL => {
            let v = JS_VALUE_GET_INT(val);
            if v < 0 {
                JS_ThrowRangeError(ctx, c"invalid array length".as_ptr());
                return -1;
            }
            v as u32
        }
        tag => {
            if JS_TAG_IS_FLOAT64(tag) != 0 {
                let d = JS_VALUE_GET_FLOAT64(val);
                if !(d >= 0.0 && d <= u32::MAX as f64) {
                    JS_ThrowRangeError(ctx, c"invalid array length".as_ptr());
                    return -1;
                }
                let len = d as u32;
                if len as f64 != d {
                    JS_ThrowRangeError(ctx, c"invalid array length".as_ptr());
                    return -1;
                }
                len
            } else if is_array_ctor != 0 {
                let val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    return -1;
                }
                let mut len = 0;
                if JS_ToArrayLengthFree(ctx, &mut len, val, 1) != 0 {
                    return -1;
                }
                len
            } else {
                let mut len = 0;
                if JS_ToUint32(ctx, &mut len, val) != 0 {
                    JS_FreeValue(ctx, val);
                    return -1;
                }
                let val = JS_ToNumberFree(ctx, val);
                if JS_IsException(val) != 0 {
                    return -1;
                }
                let mut len1 = 0;
                if JS_ToArrayLengthFree(ctx, &mut len1, val, 0) != 0 {
                    return -1;
                }
                if len1 != len {
                    JS_ThrowRangeError(ctx, c"invalid array length".as_ptr());
                    return -1;
                }
                len
            }
        }
    };
    *plen = len;
    0
}
unsafe fn JS_NumberIsInteger(ctx: *mut JSContext, val: JSValueConst) -> i32 {
    if JS_IsNumber(val) == 0 {
        return 0;
    }
    let mut d = 0.0;
    if JS_ToFloat64(ctx, &mut d, val) != 0 {
        return -1;
    }
    (d.is_finite() && d.floor() == d) as i32
}
unsafe fn JS_NumberIsNegativeOrMinusZero(_ctx: *mut JSContext, val: JSValueConst) -> JS_BOOL {
    match JS_VALUE_GET_NORM_TAG(val) {
        JS_TAG_INT => (JS_VALUE_GET_INT(val) < 0) as i32,
        JS_TAG_FLOAT64 => (JS_VALUE_GET_FLOAT64(val).to_bits() >> 63) as i32,
        JS_TAG_SHORT_BIG_INT => (JS_VALUE_GET_SHORT_BIG_INT(val) < 0) as i32,
        JS_TAG_BIG_INT => js_bigint_sign(JS_VALUE_GET_PTR(val).cast()),
        _ => 0,
    }
}
// quickjs.c:8996-9001.
unsafe fn js_symbol_to_atom(ctx: *mut JSContext, val: JSValue) -> JSAtom {
    js_get_atom_index((*ctx).rt, JS_VALUE_GET_PTR(val).cast())
}
pub unsafe fn JS_ValueToAtom(ctx: *mut JSContext, val: JSValueConst) -> JSAtom {
    let tag = JS_VALUE_GET_TAG(val);
    if tag == JS_TAG_INT && JS_VALUE_GET_INT(val) as u32 <= JS_ATOM_MAX_INT {
        __JS_AtomFromUInt32(JS_VALUE_GET_INT(val) as u32)
    } else if tag == JS_TAG_SYMBOL {
        JS_DupAtom(
            ctx,
            js_get_atom_index((*ctx).rt, JS_VALUE_GET_PTR(val).cast()),
        )
    } else {
        let str = JS_ToPropertyKey(ctx, val);
        if JS_IsException(str) != 0 {
            return JS_ATOM_NULL as u32;
        }
        if JS_VALUE_GET_TAG(str) == JS_TAG_SYMBOL {
            js_symbol_to_atom(ctx, str)
        } else {
            JS_NewAtomStr(ctx, JS_VALUE_GET_PTR(str).cast())
        }
    }
}
// quickjs.c:3649-3722 canonical numeric index detection.
unsafe fn JS_AtomIsNumericIndex1(ctx: *mut JSContext, atom: JSAtom) -> JSValue {
    use crate::quickjs_atom::{
        JS_ATOM_Infinity, JS_ATOM_NaN, JS_ATOM_minus_Infinity, JS_ATOM_minus_zero,
    };
    if __JS_AtomIsTaggedInt(atom) != 0 {
        return JS_NewInt32(ctx, __JS_AtomToUInt32(atom) as i32);
    }
    let rt = (*ctx).rt;
    assert!(atom < (*rt).atom_size as u32);
    let p = *(*rt).atom_array.add(atom as usize);
    if (*p).atom_type() != JS_ATOM_TYPE_STRING {
        return JS_UNDEFINED;
    }
    match atom {
        JS_ATOM_minus_zero => return __JS_NewFloat64(ctx, -0.0),
        JS_ATOM_Infinity => return __JS_NewFloat64(ctx, f64::INFINITY),
        JS_ATOM_minus_Infinity => return __JS_NewFloat64(ctx, f64::NEG_INFINITY),
        JS_ATOM_NaN => return __JS_NewFloat64(ctx, JS_FLOAT64_NAN),
        _ => {}
    }
    if (*p).len() == 0 {
        return JS_UNDEFINED;
    }
    let c = string_get(p, 0);
    if !is_num(c) && c != b'-' as i32 {
        return JS_UNDEFINED;
    }
    let num = JS_ToNumber(ctx, JS_MKPTR(JS_TAG_STRING, p.cast()));
    if JS_IsException(num) != 0 {
        return num;
    }
    let str = JS_ToString(ctx, num);
    if JS_IsException(str) != 0 {
        JS_FreeValue(ctx, num);
        return str;
    }
    let ret = js_string_eq(ctx, p, JS_VALUE_GET_PTR(str).cast());
    JS_FreeValue(ctx, str);
    if ret != 0 {
        num
    } else {
        JS_FreeValue(ctx, num);
        JS_UNDEFINED
    }
}
unsafe fn JS_AtomIsNumericIndex(ctx: *mut JSContext, atom: JSAtom) -> i32 {
    let num = JS_AtomIsNumericIndex1(ctx, atom);
    if JS_IsUndefined(num) != 0 {
        return 0;
    }
    if JS_IsException(num) != 0 {
        return -1;
    }
    JS_FreeValue(ctx, num);
    1
}
