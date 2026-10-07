// quickjs.c:4723-4829,15093-15096,15767-15925. MIT.
const JS_STRING_ROPE_MAX_DEPTH: usize = 60;
unsafe fn string_rope_get(val: JSValueConst, idx: u32) -> i32 {
    if JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
        string_get(JS_VALUE_GET_PTR(val).cast(), idx as i32)
    } else {
        let r = JS_VALUE_GET_PTR(val).cast::<JSStringRope>();
        let len = string_rope_get_len((*r).left);
        if idx < len {
            string_rope_get((*r).left, idx)
        } else {
            string_rope_get((*r).right, idx.wrapping_sub(len))
        }
    }
}
#[repr(C)]
struct JSStringRopeIter {
    stack: [JSValueConst; JS_STRING_ROPE_MAX_DEPTH],
    stack_len: i32,
}
unsafe fn string_rope_iter_init(s: *mut JSStringRopeIter, val: JSValueConst) {
    (*s).stack_len = 0;
    (*s).stack[(*s).stack_len as usize] = val;
    (*s).stack_len += 1;
}
unsafe fn string_rope_iter_next(s: *mut JSStringRopeIter) -> *mut JSString {
    if (*s).stack_len == 0 {
        return ptr::null_mut();
    }
    (*s).stack_len -= 1;
    let mut val = (*s).stack[(*s).stack_len as usize];
    loop {
        if JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
            return JS_VALUE_GET_PTR(val).cast();
        }
        let r = JS_VALUE_GET_PTR(val).cast::<JSStringRope>();
        assert!((*s).stack_len < (JS_STRING_ROPE_MAX_DEPTH as i32));
        (*s).stack[(*s).stack_len as usize] = (*r).right;
        (*s).stack_len += 1;
        val = (*r).left;
    }
}
unsafe fn string_rope_get_len(val: JSValueConst) -> u32 {
    if JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
        (*JS_VALUE_GET_PTR(val).cast::<JSString>()).len()
    } else {
        (*JS_VALUE_GET_PTR(val).cast::<JSStringRope>()).len
    }
}
unsafe fn js_string_rope_compare(
    _ctx: *mut JSContext,
    op1: JSValueConst,
    op2: JSValueConst,
    eq_only: JS_BOOL,
) -> i32 {
    let len1 = string_rope_get_len(op1);
    let len2 = string_rope_get_len(op2);
    if eq_only != 0 && len1 != len2 {
        return 1;
    }
    let mut len = len1.min(len2);
    let mut it1: JSStringRopeIter = core::mem::zeroed();
    let mut it2: JSStringRopeIter = core::mem::zeroed();
    string_rope_iter_init(&mut it1, op1);
    string_rope_iter_init(&mut it2, op2);
    let mut p1 = string_rope_iter_next(&mut it1);
    let mut p2 = string_rope_iter_next(&mut it2);
    let mut pos1 = 0;
    let mut pos2 = 0;
    while len != 0 {
        let l = (*p1)
            .len()
            .wrapping_sub(pos1)
            .min((*p2).len().wrapping_sub(pos2))
            .min(len);
        let res = js_string_memcmp(p1, pos1 as i32, p2, pos2 as i32, l as i32);
        if res != 0 {
            return res;
        }
        len -= l;
        pos1 += l;
        if pos1 >= (*p1).len() {
            p1 = string_rope_iter_next(&mut it1);
            pos1 = 0;
        }
        pos2 += l;
        if pos2 >= (*p2).len() {
            p2 = string_rope_iter_next(&mut it2);
            pos2 = 0;
        }
    }
    if len1 == len2 {
        0
    } else if len1 < len2 {
        -1
    } else {
        1
    }
}
fn tag_is_string(tag: u32) -> JS_BOOL {
    (tag == JS_TAG_STRING as u32 || tag == JS_TAG_STRING_ROPE as u32) as i32
}
type JSStrictEqModeEnum = u32;
const JS_EQ_STRICT: JSStrictEqModeEnum = 0;
const JS_EQ_SAME_VALUE: JSStrictEqModeEnum = 1;
const JS_EQ_SAME_VALUE_ZERO: JSStrictEqModeEnum = 2;
unsafe fn js_strict_eq2(
    ctx: *mut JSContext,
    op1: JSValue,
    op2: JSValue,
    eq_mode: JSStrictEqModeEnum,
) -> JS_BOOL {
    let tag1 = JS_VALUE_GET_NORM_TAG(op1);
    let tag2 = JS_VALUE_GET_NORM_TAG(op2);
    let res = match tag1 {
        JS_TAG_BOOL => {
            if tag1 == tag2 {
                return (JS_VALUE_GET_INT(op1) == JS_VALUE_GET_INT(op2)) as i32;
            }
            0
        }
        JS_TAG_NULL | JS_TAG_UNDEFINED => (tag1 == tag2) as i32,
        JS_TAG_STRING | JS_TAG_STRING_ROPE => {
            if tag_is_string(tag2 as u32) == 0 {
                0
            } else if tag1 == JS_TAG_STRING && tag2 == JS_TAG_STRING {
                js_string_eq(
                    ctx,
                    JS_VALUE_GET_PTR(op1).cast(),
                    JS_VALUE_GET_PTR(op2).cast(),
                )
            } else {
                (js_string_rope_compare(ctx, op1, op2, 1) == 0) as i32
            }
        }
        JS_TAG_SYMBOL | JS_TAG_OBJECT => {
            (tag1 == tag2 && JS_VALUE_GET_PTR(op1) == JS_VALUE_GET_PTR(op2)) as i32
        }
        JS_TAG_INT | JS_TAG_FLOAT64 => {
            if tag2 == JS_TAG_INT || tag2 == JS_TAG_FLOAT64 {
                let d1 = if tag1 == JS_TAG_INT {
                    JS_VALUE_GET_INT(op1) as f64
                } else {
                    JS_VALUE_GET_FLOAT64(op1)
                };
                let d2 = if tag2 == JS_TAG_INT {
                    JS_VALUE_GET_INT(op2) as f64
                } else {
                    JS_VALUE_GET_FLOAT64(op2)
                };
                return if eq_mode >= JS_EQ_SAME_VALUE {
                    if d1.is_nan() || d2.is_nan() {
                        (d1.is_nan() == d2.is_nan()) as i32
                    } else if eq_mode == JS_EQ_SAME_VALUE_ZERO {
                        (d1 == d2) as i32
                    } else {
                        (d1.to_bits() == d2.to_bits()) as i32
                    }
                } else {
                    (d1 == d2) as i32
                };
            }
            0
        }
        JS_TAG_SHORT_BIG_INT | JS_TAG_BIG_INT => {
            if tag2 != JS_TAG_SHORT_BIG_INT && tag2 != JS_TAG_BIG_INT {
                0
            } else {
                let mut buf1: JSBigIntBuf = core::mem::zeroed();
                let mut buf2: JSBigIntBuf = core::mem::zeroed();
                let p1 = if tag1 == JS_TAG_SHORT_BIG_INT {
                    js_bigint_set_short(&mut buf1, op1)
                } else {
                    JS_VALUE_GET_PTR(op1).cast()
                };
                let p2 = if tag2 == JS_TAG_SHORT_BIG_INT {
                    js_bigint_set_short(&mut buf2, op2)
                } else {
                    JS_VALUE_GET_PTR(op2).cast()
                };
                (js_bigint_cmp(ctx, p1, p2) == 0) as i32
            }
        }
        _ => 0,
    };
    JS_FreeValue(ctx, op1);
    JS_FreeValue(ctx, op2);
    res
}
unsafe fn js_strict_eq(ctx: *mut JSContext, op1: JSValueConst, op2: JSValueConst) -> JS_BOOL {
    js_strict_eq2(
        ctx,
        JS_DupValue(ctx, op1),
        JS_DupValue(ctx, op2),
        JS_EQ_STRICT,
    )
}
pub unsafe fn JS_StrictEq(ctx: *mut JSContext, op1: JSValueConst, op2: JSValueConst) -> JS_BOOL {
    js_strict_eq(ctx, op1, op2)
}
unsafe fn js_same_value(ctx: *mut JSContext, op1: JSValueConst, op2: JSValueConst) -> JS_BOOL {
    js_strict_eq2(
        ctx,
        JS_DupValue(ctx, op1),
        JS_DupValue(ctx, op2),
        JS_EQ_SAME_VALUE,
    )
}
pub unsafe fn JS_SameValue(ctx: *mut JSContext, op1: JSValueConst, op2: JSValueConst) -> JS_BOOL {
    js_same_value(ctx, op1, op2)
}
unsafe fn js_same_value_zero(ctx: *mut JSContext, op1: JSValueConst, op2: JSValueConst) -> JS_BOOL {
    js_strict_eq2(
        ctx,
        JS_DupValue(ctx, op1),
        JS_DupValue(ctx, op2),
        JS_EQ_SAME_VALUE_ZERO,
    )
}
pub unsafe fn JS_SameValueZero(
    ctx: *mut JSContext,
    op1: JSValueConst,
    op2: JSValueConst,
) -> JS_BOOL {
    js_same_value_zero(ctx, op1, op2)
}
