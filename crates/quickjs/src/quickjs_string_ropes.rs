// quickjs.c:4651-4722,4868-5118. MIT. Original flat/rope concatenation.
const JS_STRING_ROPE_SHORT_LEN: u32 = 512;
const JS_STRING_ROPE_SHORT2_LEN: u32 = 8192;
unsafe fn JS_ConcatString1(
    ctx: *mut JSContext,
    p1: *const JSString,
    p2: *const JSString,
) -> JSValue {
    let len = (*p1).len().wrapping_add((*p2).len());
    if len > JS_STRING_LEN_MAX as u32 {
        return JS_ThrowInternalError(ctx, c"string too long".as_ptr());
    }
    let wide = (*p1).is_wide_char() | (*p2).is_wide_char();
    let p = js_alloc_string(ctx, len as i32, wide as i32);
    if p.is_null() {
        return JS_EXCEPTION;
    }
    if wide == 0 {
        ptr::copy_nonoverlapping(
            string_data8(p1.cast_mut()),
            string_data8(p),
            (*p1).len() as usize,
        );
        ptr::copy_nonoverlapping(
            string_data8(p2.cast_mut()),
            string_data8(p).add((*p1).len() as usize),
            (*p2).len() as usize,
        );
        *string_data8(p).add(len as usize) = 0;
    } else {
        copy_str16(string_data16(p), p1, 0, (*p1).len() as i32);
        copy_str16(
            string_data16(p).add((*p1).len() as usize),
            p2,
            0,
            (*p2).len() as i32,
        );
    }
    JS_MKPTR(JS_TAG_STRING, p.cast())
}
unsafe fn JS_ConcatStringInPlace(
    ctx: *mut JSContext,
    p1: *mut JSString,
    op2: JSValueConst,
) -> JS_BOOL {
    if JS_VALUE_GET_TAG(op2) == JS_TAG_STRING {
        let p2 = JS_VALUE_GET_PTR(op2).cast::<JSString>();
        if (*p2).len() == 0 {
            return 1;
        }
        if (*js_rc(p1.cast())).ref_count != 1 {
            return 0;
        }
        let size1 = js_malloc_usable_size(ctx, p1.cast());
        let len1 = (*p1).len();
        let len2 = (*p2).len();
        if (*p1).is_wide_char() != 0 {
            if size1 >= size_of::<JSString>() + ((len1.wrapping_add(len2)) << 1) as usize {
                if (*p2).is_wide_char() != 0 {
                    ptr::copy_nonoverlapping(
                        string_data16(p2),
                        string_data16(p1).add(len1 as usize),
                        len2 as usize,
                    );
                    (*p1).set_len(len1.wrapping_add(len2));
                    return 1;
                }
                for i in 0..len2 {
                    let len = (*p1).len();
                    *string_data16(p1).add(len as usize) = *string_data8(p2).add(i as usize) as u16;
                    (*p1).set_len(len + 1);
                }
                return 1;
            }
        } else if (*p2).is_wide_char() == 0
            && size1 >= size_of::<JSString>() + len1.wrapping_add(len2) as usize + 1
        {
            ptr::copy_nonoverlapping(
                string_data8(p2),
                string_data8(p1).add(len1 as usize),
                len2 as usize,
            );
            (*p1).set_len(len1.wrapping_add(len2));
            *string_data8(p1).add((*p1).len() as usize) = 0;
            return 1;
        }
    }
    0
}
unsafe fn JS_ConcatString2(ctx: *mut JSContext, op1: JSValue, op2: JSValue) -> JSValue {
    let p1 = JS_VALUE_GET_PTR(op1).cast::<JSString>();
    if JS_ConcatStringInPlace(ctx, p1, op2) != 0 {
        JS_FreeValue(ctx, op2);
        return op1;
    }
    let p2 = JS_VALUE_GET_PTR(op2).cast::<JSString>();
    let ret = JS_ConcatString1(ctx, p1, p2);
    JS_FreeValue(ctx, op1);
    JS_FreeValue(ctx, op2);
    ret
}
unsafe fn js_new_string_rope(ctx: *mut JSContext, op1: JSValue, op2: JSValue) -> JSValue {
    let (mut len, mut wide, mut depth) = if JS_VALUE_GET_TAG(op1) == JS_TAG_STRING {
        let p = JS_VALUE_GET_PTR(op1).cast::<JSString>();
        ((*p).len(), (*p).is_wide_char() as u8, 0)
    } else {
        let r = JS_VALUE_GET_PTR(op1).cast::<JSStringRope>();
        ((*r).len, (*r).is_wide_char, (*r).depth)
    };
    if JS_VALUE_GET_TAG(op2) == JS_TAG_STRING {
        let p = JS_VALUE_GET_PTR(op2).cast::<JSString>();
        len = len.wrapping_add((*p).len());
        wide |= (*p).is_wide_char() as u8;
    } else {
        let r = JS_VALUE_GET_PTR(op2).cast::<JSStringRope>();
        len = len.wrapping_add((*r).len);
        wide |= (*r).is_wide_char;
        depth = depth.max((*r).depth);
    }
    let r = if len > JS_STRING_LEN_MAX as u32 {
        JS_ThrowInternalError(ctx, c"string too long".as_ptr());
        ptr::null_mut()
    } else {
        js_malloc(ctx, size_of::<JSStringRope>()).cast::<JSStringRope>()
    };
    if r.is_null() {
        JS_FreeValue(ctx, op1);
        JS_FreeValue(ctx, op2);
        return JS_EXCEPTION;
    }
    (*js_rc(r.cast())).ref_count = 1;
    (*r).len = len;
    (*r).is_wide_char = wide;
    (*r).depth = depth + 1;
    (*r).left = op1;
    (*r).right = op2;
    let res = JS_MKPTR(JS_TAG_STRING_ROPE, r.cast());
    if (*r).depth as usize > JS_STRING_ROPE_MAX_DEPTH {
        let res2 = js_rebalancee_string_rope(ctx, res);
        JS_FreeValue(ctx, res);
        res2
    } else {
        res
    }
}
const ROPE_N_BUCKETS: usize = 44;
const rope_bucket_len: [u32; ROPE_N_BUCKETS] = [
    1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181, 6765, 10946,
    17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229, 832040, 1346269, 2178309, 3524578,
    5702887, 9227465, 14930352, 24157817, 39088169, 63245986, 102334155, 165580141, 267914296,
    433494437, 701408733, 1134903170,
];
unsafe fn js_rebalancee_string_rope_rec(
    ctx: *mut JSContext,
    buckets: *mut JSValue,
    val: JSValueConst,
) -> i32 {
    if JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
        let p = JS_VALUE_GET_PTR(val).cast::<JSString>();
        let len = (*p).len();
        if len == 0 {
            return 0;
        }
        let mut a = JS_NULL;
        let mut i = 0;
        while len >= rope_bucket_len[i + 1] {
            let b = *buckets.add(i);
            if JS_IsNull(b) == 0 {
                *buckets.add(i) = JS_NULL;
                if JS_IsNull(a) != 0 {
                    a = b;
                } else {
                    a = js_new_string_rope(ctx, b, a);
                    if JS_IsException(a) != 0 {
                        return -1;
                    }
                }
            }
            i += 1;
        }
        a = if JS_IsNull(a) == 0 {
            let a = js_new_string_rope(ctx, a, JS_DupValue(ctx, val));
            if JS_IsException(a) != 0 {
                return -1;
            }
            a
        } else {
            JS_DupValue(ctx, val)
        };
        while JS_IsNull(*buckets.add(i)) == 0 {
            a = js_new_string_rope(ctx, *buckets.add(i), a);
            *buckets.add(i) = JS_NULL;
            if JS_IsException(a) != 0 {
                return -1;
            }
            i += 1;
        }
        *buckets.add(i) = a;
    } else {
        let r = JS_VALUE_GET_PTR(val).cast::<JSStringRope>();
        // C intentionally discards both recursive return codes in this branch.
        js_rebalancee_string_rope_rec(ctx, buckets, (*r).left);
        js_rebalancee_string_rope_rec(ctx, buckets, (*r).right);
    }
    0
}
unsafe fn js_rebalancee_string_rope(ctx: *mut JSContext, rope: JSValueConst) -> JSValue {
    let mut buckets = [JS_NULL; ROPE_N_BUCKETS];
    let mut failed = js_rebalancee_string_rope_rec(ctx, buckets.as_mut_ptr(), rope) != 0;
    let mut a = JS_NULL;
    if !failed {
        for bucket in &mut buckets {
            let b = *bucket;
            if JS_IsNull(b) == 0 {
                *bucket = JS_NULL;
                if JS_IsNull(a) != 0 {
                    a = b;
                } else {
                    a = js_new_string_rope(ctx, b, a);
                    if JS_IsException(a) != 0 {
                        failed = true;
                        break;
                    }
                }
            }
        }
    }
    if failed {
        for b in buckets {
            JS_FreeValue(ctx, b);
        }
        JS_EXCEPTION
    } else if JS_IsNull(a) != 0 {
        JS_AtomToString(ctx, crate::quickjs_atom::JS_ATOM_empty_string)
    } else {
        a
    }
}
unsafe fn JS_ConcatString(ctx: *mut JSContext, mut op1: JSValue, mut op2: JSValue) -> JSValue {
    if !matches!(JS_VALUE_GET_TAG(op1), JS_TAG_STRING | JS_TAG_STRING_ROPE) {
        op1 = JS_ToStringFree(ctx, op1);
        if JS_IsException(op1) != 0 {
            JS_FreeValue(ctx, op2);
            return JS_EXCEPTION;
        }
    }
    if !matches!(JS_VALUE_GET_TAG(op2), JS_TAG_STRING | JS_TAG_STRING_ROPE) {
        op2 = JS_ToStringFree(ctx, op2);
        if JS_IsException(op2) != 0 {
            JS_FreeValue(ctx, op1);
            return JS_EXCEPTION;
        }
    }
    if JS_VALUE_GET_TAG(op2) == JS_TAG_STRING {
        let p2 = JS_VALUE_GET_PTR(op2).cast::<JSString>();
        if (*p2).len() == 0 {
            JS_FreeValue(ctx, op2);
            return op1;
        }
        if (*p2).len() <= JS_STRING_ROPE_SHORT_LEN {
            if JS_VALUE_GET_TAG(op1) == JS_TAG_STRING {
                let p1 = JS_VALUE_GET_PTR(op1).cast::<JSString>();
                if (*p1).len() <= JS_STRING_ROPE_SHORT2_LEN {
                    return JS_ConcatString2(ctx, op1, op2);
                } else {
                    return js_new_string_rope(ctx, op1, op2);
                }
            } else {
                let r1 = JS_VALUE_GET_PTR(op1).cast::<JSStringRope>();
                if JS_VALUE_GET_TAG((*r1).right) == JS_TAG_STRING
                    && (*JS_VALUE_GET_PTR((*r1).right).cast::<JSString>()).len()
                        <= JS_STRING_ROPE_SHORT_LEN
                {
                    let val = JS_ConcatString2(ctx, JS_DupValue(ctx, (*r1).right), op2);
                    if JS_IsException(val) != 0 {
                        JS_FreeValue(ctx, op1);
                        return JS_EXCEPTION;
                    }
                    let ret = js_new_string_rope(ctx, JS_DupValue(ctx, (*r1).left), val);
                    JS_FreeValue(ctx, op1);
                    return ret;
                }
            }
        }
    } else if JS_VALUE_GET_TAG(op1) == JS_TAG_STRING {
        let p1 = JS_VALUE_GET_PTR(op1).cast::<JSString>();
        if (*p1).len() == 0 {
            JS_FreeValue(ctx, op1);
            return op2;
        }
        let r2 = JS_VALUE_GET_PTR(op2).cast::<JSStringRope>();
        if JS_VALUE_GET_TAG((*r2).left) == JS_TAG_STRING
            && (*JS_VALUE_GET_PTR((*r2).left).cast::<JSString>()).len() <= JS_STRING_ROPE_SHORT_LEN
        {
            let val = JS_ConcatString2(ctx, op1, JS_DupValue(ctx, (*r2).left));
            if JS_IsException(val) != 0 {
                JS_FreeValue(ctx, op2);
                return JS_EXCEPTION;
            }
            let ret = js_new_string_rope(ctx, val, JS_DupValue(ctx, (*r2).right));
            JS_FreeValue(ctx, op2);
            return ret;
        }
    }
    js_new_string_rope(ctx, op1, op2)
}
