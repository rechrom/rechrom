// quickjs.c:11591-11603,11659-11707,11730-11761,11787-11857,
// 12423-12741,14578-14610. MIT. Staged with allocation/error/string paths.
const JS_BIGINT_MAX_SIZE: i32 = (1024 * 1024) / JS_LIMB_BITS as i32;
unsafe fn bigint_tab(p: *const JSBigInt) -> *mut js_limb_t {
    ptr::addr_of!((*p).tab).cast::<js_limb_t>().cast_mut()
}
unsafe fn js_bigint_new(ctx: *mut JSContext, len: i32) -> *mut JSBigInt {
    if len > JS_BIGINT_MAX_SIZE {
        JS_ThrowRangeError(ctx, c"BigInt is too large to allocate".as_ptr());
        return ptr::null_mut();
    }
    let r = js_malloc(
        ctx,
        core::mem::size_of::<JSBigInt>() + len as usize * core::mem::size_of::<js_limb_t>(),
    )
    .cast::<JSBigInt>();
    if r.is_null() {
        return r;
    }
    (*js_rc(r.cast())).ref_count = 1;
    (*r).len = len as u32;
    r
}
unsafe fn js_bigint_new_si(ctx: *mut JSContext, a: js_slimb_t) -> *mut JSBigInt {
    let r = js_bigint_new(ctx, 1);
    if !r.is_null() {
        *bigint_tab(r) = a as js_limb_t;
    }
    r
}
unsafe fn js_bigint_new_si64(ctx: *mut JSContext, a: i64) -> *mut JSBigInt {
    #[cfg(target_pointer_width = "64")]
    {
        js_bigint_new_si(ctx, a)
    }
    #[cfg(target_pointer_width = "32")]
    {
        if a >= i32::MIN as i64 && a <= i32::MAX as i64 {
            js_bigint_new_si(ctx, a as i32)
        } else {
            let r = js_bigint_new(ctx, 2);
            if !r.is_null() {
                *bigint_tab(r) = a as js_limb_t;
                *bigint_tab(r).add(1) = (a >> 32) as js_limb_t;
            }
            r
        }
    }
}
unsafe fn js_bigint_new_ui64(ctx: *mut JSContext, a: u64) -> *mut JSBigInt {
    if a <= i64::MAX as u64 {
        return js_bigint_new_si64(ctx, a as i64);
    }
    let r = js_bigint_new(ctx, ((65 + JS_LIMB_BITS - 1) / JS_LIMB_BITS) as i32);
    if r.is_null() {
        return r;
    }
    *bigint_tab(r) = a as js_limb_t;
    #[cfg(target_pointer_width = "64")]
    {
        *bigint_tab(r).add(1) = 0;
    }
    #[cfg(target_pointer_width = "32")]
    {
        *bigint_tab(r).add(1) = (a >> 32) as js_limb_t;
        *bigint_tab(r).add(2) = 0;
    }
    r
}
unsafe fn js_bigint_normalize1(
    ctx: *mut JSContext,
    mut a: *mut JSBigInt,
    mut l: i32,
) -> *mut JSBigInt {
    assert_eq!((*js_rc(a.cast())).ref_count, 1);
    while l > 1 {
        let v = *bigint_tab(a).add(l as usize - 1);
        if (v != 0 && v != js_limb_t::MAX)
            || (v & 1) != (*bigint_tab(a).add(l as usize - 2) >> (JS_LIMB_BITS - 1))
        {
            break;
        }
        l -= 1;
    }
    if l != (*a).len as i32 {
        (*a).len = l as u32;
        let a1 = js_realloc(
            ctx,
            a.cast(),
            core::mem::size_of::<JSBigInt>() + l as usize * core::mem::size_of::<js_limb_t>(),
        )
        .cast::<JSBigInt>();
        if !a1.is_null() {
            a = a1;
        }
    }
    a
}
unsafe fn js_bigint_normalize(ctx: *mut JSContext, a: *mut JSBigInt) -> *mut JSBigInt {
    js_bigint_normalize1(ctx, a, (*a).len as i32)
}
unsafe fn js_bigint_extend(
    ctx: *mut JSContext,
    mut r: *mut JSBigInt,
    op1: js_limb_t,
) -> *mut JSBigInt {
    let n2 = (*r).len;
    if (op1 != 0 && op1 != js_limb_t::MAX)
        || (op1 & 1) != (*bigint_tab(r).add(n2 as usize - 1) >> (JS_LIMB_BITS - 1))
    {
        let r1 = js_realloc(
            ctx,
            r.cast(),
            core::mem::size_of::<JSBigInt>()
                + (n2 as usize + 1) * core::mem::size_of::<js_limb_t>(),
        )
        .cast::<JSBigInt>();
        if r1.is_null() {
            js_free(ctx, r.cast());
            return ptr::null_mut();
        }
        r = r1;
        (*r).len = n2 + 1;
        *bigint_tab(r).add(n2 as usize) = op1;
    } else {
        r = js_bigint_normalize(ctx, r);
    }
    r
}
unsafe fn js_bigint_add(
    ctx: *mut JSContext,
    a: *const JSBigInt,
    b: *const JSBigInt,
    b_neg: i32,
) -> *mut JSBigInt {
    let n2 = (*a).len.max((*b).len);
    let n1 = (*a).len.min((*b).len);
    let r = js_bigint_new(ctx, n2 as i32);
    if r.is_null() {
        return r;
    }
    let mut carry = b_neg as js_limb_t;
    let mask = (b_neg as js_limb_t).wrapping_neg();
    for i in 0..n1 as usize {
        let (v, c) = ADDC(*bigint_tab(a).add(i), *bigint_tab(b).add(i) ^ mask, carry);
        *bigint_tab(r).add(i) = v;
        carry = c;
    }
    let a_sign = (js_bigint_sign(a) as js_limb_t).wrapping_neg();
    let b_sign = (js_bigint_sign(b) as js_limb_t).wrapping_neg() ^ mask;
    if (*a).len > (*b).len {
        for i in n1 as usize..n2 as usize {
            let (v, c) = ADDC(*bigint_tab(a).add(i), b_sign, carry);
            *bigint_tab(r).add(i) = v;
            carry = c;
        }
    } else if (*a).len < (*b).len {
        for i in n1 as usize..n2 as usize {
            let (v, c) = ADDC(a_sign, *bigint_tab(b).add(i) ^ mask, carry);
            *bigint_tab(r).add(i) = v;
            carry = c;
        }
    }
    js_bigint_extend(ctx, r, a_sign.wrapping_add(b_sign).wrapping_add(carry))
}
unsafe fn js_bigint_neg(ctx: *mut JSContext, a: *const JSBigInt) -> *mut JSBigInt {
    let mut buf: JSBigIntBuf = core::mem::zeroed();
    let b = js_bigint_set_si(&mut buf, 0);
    js_bigint_add(ctx, b, a, 1)
}
#[cfg(target_pointer_width = "64")]
static js_pow_dec: [js_limb_t; JS_LIMB_DIGITS as usize + 1] = [
    1,
    10,
    100,
    1000,
    10000,
    100000,
    1000000,
    10000000,
    100000000,
    1000000000,
    10000000000,
    100000000000,
    1000000000000,
    10000000000000,
    100000000000000,
    1000000000000000,
    10000000000000000,
    100000000000000000,
    1000000000000000000,
    10000000000000000000,
];
#[cfg(target_pointer_width = "32")]
static js_pow_dec: [js_limb_t; JS_LIMB_DIGITS as usize + 1] = [
    1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
];
#[cfg(target_pointer_width = "32")]
static digits_per_limb_table: [u8; 35] = [
    32, 20, 16, 13, 12, 11, 10, 10, 9, 9, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6,
];
#[cfg(target_pointer_width = "64")]
static digits_per_limb_table: [u8; 35] = [
    64, 40, 32, 27, 24, 22, 21, 20, 19, 18, 17, 17, 16, 16, 16, 15, 15, 15, 14, 14, 14, 14, 13, 13,
    13, 13, 13, 13, 13, 12, 12, 12, 12, 12, 12,
];
#[cfg(target_pointer_width = "32")]
static radix_base_table: [js_limb_t; 35] = [
    0x00000000, 0xcfd41b91, 0x00000000, 0x48c27395, 0x81bf1000, 0x75db9c97, 0x40000000, 0xcfd41b91,
    0x3b9aca00, 0x8c8b6d2b, 0x19a10000, 0x309f1021, 0x57f6c100, 0x98c29b81, 0x00000000, 0x18754571,
    0x247dbc80, 0x3547667b, 0x4c4b4000, 0x6b5a6e1d, 0x94ace180, 0xcaf18367, 0x0b640000, 0x0e8d4a51,
    0x1269ae40, 0x17179149, 0x1cb91000, 0x23744899, 0x2b73a840, 0x34e63b41, 0x40000000, 0x4cfa3cc1,
    0x5c13d840, 0x6d91b519, 0x81bf1000,
];
#[cfg(target_pointer_width = "64")]
static radix_base_table: [js_limb_t; 35] = [
    0x0000000000000000,
    0xa8b8b452291fe821,
    0x0000000000000000,
    0x6765c793fa10079d,
    0x41c21cb8e1000000,
    0x3642798750226111,
    0x8000000000000000,
    0xa8b8b452291fe821,
    0x8ac7230489e80000,
    0x4d28cb56c33fa539,
    0x1eca170c00000000,
    0x780c7372621bd74d,
    0x1e39a5057d810000,
    0x5b27ac993df97701,
    0x0000000000000000,
    0x27b95e997e21d9f1,
    0x5da0e1e53c5c8000,
    0xd2ae3299c1c4aedb,
    0x16bcc41e90000000,
    0x2d04b7fdd9c0ef49,
    0x5658597bcaa24000,
    0xa0e2073737609371,
    0x0c29e98000000000,
    0x14adf4b7320334b9,
    0x226ed36478bfa000,
    0x383d9170b85ff80b,
    0x5a3c23e39c000000,
    0x8e65137388122bcd,
    0xdd41bb36d259e000,
    0x0aee5720ee830681,
    0x1000000000000000,
    0x172588ad4f5f0981,
    0x211e44f7d02c1000,
    0x2ee56725f06e5c71,
    0x41c21cb8e1000000,
];
unsafe fn js_bigint_from_string(
    ctx: *mut JSContext,
    str: *const c_char,
    radix: i32,
) -> *mut JSBigInt {
    let mut p = str;
    let is_neg = (*p == b'-' as c_char) as i32;
    if is_neg != 0 {
        p = p.add(1);
    }
    while *p == b'0' as c_char {
        p = p.add(1);
    }
    let n_digits1 = core::ffi::CStr::from_ptr(p).to_bytes().len();
    if n_digits1 > JS_BIGINT_MAX_SIZE as usize * JS_LIMB_BITS {
        JS_ThrowRangeError(ctx, c"BigInt is too large to allocate".as_ptr());
        return ptr::null_mut();
    }
    let n_digits = n_digits1 as i32;
    let log2_radix = 32 - (radix as u32 - 1).leading_zeros() as i32;
    let n_bits = if radix == 10 {
        (n_digits * 27 + 7) / 8
    } else {
        n_digits * log2_radix
    };
    let n_limbs = 1.max(n_bits / JS_LIMB_BITS as i32 + 1);
    let mut r = js_bigint_new(ctx, n_limbs);
    if r.is_null() {
        return r;
    }
    if radix == 10 {
        let mut len = 1;
        *bigint_tab(r) = 0;
        loop {
            let mut v = 0 as js_limb_t;
            let mut i = 0;
            while i < JS_LIMB_DIGITS as usize {
                let c = to_digit(*p as i32) as js_limb_t;
                if c >= radix as js_limb_t {
                    break;
                }
                p = p.add(1);
                v = v.wrapping_mul(10).wrapping_add(c);
                i += 1;
            }
            if i == 0 {
                break;
            }
            if len == 1 && *bigint_tab(r) == 0 {
                *bigint_tab(r) = v;
            } else {
                let h = mp_mul1(
                    bigint_tab(r),
                    bigint_tab(r),
                    len as js_limb_t,
                    js_pow_dec[i],
                    v,
                );
                if h != 0 {
                    *bigint_tab(r).add(len as usize) = h;
                    len += 1;
                }
            }
        }
        if *bigint_tab(r).add(len as usize - 1) >> (JS_LIMB_BITS - 1) != 0 {
            *bigint_tab(r).add(len as usize) = 0;
            len += 1;
        }
        (*r).len = len as u32;
    } else {
        (*r).len = n_limbs as u32;
        ptr::write_bytes(bigint_tab(r), 0, n_limbs as usize);
        for i in 0..n_digits as usize {
            let c = to_digit(*p.add(n_digits as usize - 1 - i) as i32) as js_limb_t;
            assert!(c < radix as js_limb_t);
            let bit_pos = i * log2_radix as usize;
            let shift = bit_pos & (JS_LIMB_BITS - 1);
            let pos = bit_pos / JS_LIMB_BITS;
            *bigint_tab(r).add(pos) |= c << shift;
            if shift + log2_radix as usize > JS_LIMB_BITS {
                *bigint_tab(r).add(pos + 1) |= c >> (JS_LIMB_BITS - shift);
            }
        }
    }
    r = js_bigint_normalize(ctx, r);
    if is_neg != 0 {
        let r1 = js_bigint_neg(ctx, r);
        js_free(ctx, r.cast());
        r = r1;
    }
    r
}
static digits: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
unsafe fn js_u64toa(mut q: *mut c_char, n: i64, base: u32) -> *mut c_char {
    let mut n = n as u64;
    if base == 10 {
        loop {
            let d = n % 10;
            n /= 10;
            q = q.sub(1);
            *q = (b'0' + d as u8) as c_char;
            if n == 0 {
                break;
            }
        }
    } else {
        loop {
            let d = n % base as u64;
            n /= base as u64;
            q = q.sub(1);
            *q = digits[d as usize] as c_char;
            if n == 0 {
                break;
            }
        }
    }
    q
}
unsafe fn limb_to_a(mut q: *mut c_char, mut n: js_limb_t, radix: u32, len: i32) -> *mut c_char {
    if radix == 10 {
        for _ in 0..len {
            let d = n % 10;
            n /= 10;
            q = q.sub(1);
            *q = (b'0' + d as u8) as c_char;
        }
    } else {
        for _ in 0..len {
            let d = n % radix as js_limb_t;
            n /= radix as js_limb_t;
            q = q.sub(1);
            *q = digits[d as usize] as c_char;
        }
    }
    q
}
const JS_RADIX_MAX: i32 = 36;
unsafe fn js_bigint_to_string1(ctx: *mut JSContext, val: JSValueConst, radix: i32) -> JSValue {
    if JS_VALUE_GET_TAG(val) == JS_TAG_SHORT_BIG_INT {
        let mut buf = [0i8; 66];
        let len = crate::dtoa::i64toa_radix(
            buf.as_mut_ptr(),
            JS_VALUE_GET_SHORT_BIG_INT(val) as i64,
            radix as u32,
        );
        return js_new_string8_len(ctx, buf.as_ptr(), len as i32);
    }
    assert_eq!(JS_VALUE_GET_TAG(val), JS_TAG_BIG_INT);
    let mut r = JS_VALUE_GET_PTR(val).cast::<JSBigInt>();
    let mut tmp = ptr::null_mut();
    if (*r).len == 1 && *bigint_tab(r) == 0 {
        return js_new_string8_len(ctx, c"0".as_ptr(), 1);
    }
    let binary = radix & (radix - 1) == 0;
    let is_neg = js_bigint_sign(r);
    if is_neg != 0 {
        tmp = js_bigint_neg(ctx, r);
        if tmp.is_null() {
            return JS_EXCEPTION;
        }
        r = tmp;
    } else if !binary {
        tmp = js_bigint_new(ctx, (*r).len as i32);
        if tmp.is_null() {
            return JS_EXCEPTION;
        }
        ptr::copy_nonoverlapping(bigint_tab(r), bigint_tab(tmp), (*r).len as usize);
        r = tmp;
    }
    let log2_radix = 31 - (radix as u32).leading_zeros() as i32;
    let n_bits = (*r).len as i32 * JS_LIMB_BITS as i32
        - js_limb_safe_clz(*bigint_tab(r).add((*r).len as usize - 1)) as i32;
    let n_digits = (n_bits + log2_radix - 1) / log2_radix;
    let buf = js_malloc(ctx, (n_digits + is_neg + 1) as usize).cast::<c_char>();
    if buf.is_null() {
        js_free(ctx, tmp.cast());
        return JS_EXCEPTION;
    }
    let mut q = buf.add((n_digits + is_neg) as usize);
    *q = 0;
    let buf_end = q;
    if !binary {
        let base = radix_base_table[radix as usize - 2];
        let mut len = (*r).len;
        loop {
            while len > 1 && *bigint_tab(r).add(len as usize - 1) == 0 {
                len -= 1;
            }
            if len == 1 && *bigint_tab(r) < base {
                let v = *bigint_tab(r);
                if v != 0 {
                    q = js_u64toa(q, v as i64, radix as u32);
                }
                break;
            }
            let v = mp_div1(bigint_tab(r), bigint_tab(r), len as js_limb_t, base, 0);
            q = limb_to_a(
                q,
                v,
                radix as u32,
                digits_per_limb_table[radix as usize - 2] as i32,
            );
        }
    } else {
        for i in 0..n_digits {
            let bit_pos = i as usize * log2_radix as usize;
            let pos = bit_pos / JS_LIMB_BITS;
            let shift = bit_pos % JS_LIMB_BITS;
            let mut c = (*bigint_tab(r).add(pos) >> shift) as u32;
            if shift + log2_radix as usize > JS_LIMB_BITS && pos + 1 < (*r).len as usize {
                c |= (*bigint_tab(r).add(pos + 1) << (JS_LIMB_BITS - shift)) as u32;
            }
            c &= radix as u32 - 1;
            q = q.sub(1);
            *q = digits[c as usize] as c_char;
        }
    }
    if is_neg != 0 {
        q = q.sub(1);
        *q = b'-' as c_char;
    }
    js_free(ctx, tmp.cast());
    let res = js_new_string8_len(ctx, q, buf_end.offset_from(q) as i32);
    js_free(ctx, buf.cast());
    res
}
unsafe fn JS_CompactBigInt(ctx: *mut JSContext, p: *mut JSBigInt) -> JSValue {
    if (*p).len == 1 {
        let res = __JS_NewShortBigInt(ctx, *bigint_tab(p) as js_slimb_t);
        js_free(ctx, p.cast());
        res
    } else {
        JS_MKPTR(JS_TAG_BIG_INT, p.cast())
    }
}
pub unsafe fn JS_NewBigInt64(ctx: *mut JSContext, v: i64) -> JSValue {
    #[cfg(target_pointer_width = "64")]
    {
        __JS_NewShortBigInt(ctx, v)
    }
    #[cfg(target_pointer_width = "32")]
    {
        if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
            __JS_NewShortBigInt(ctx, v as i32)
        } else {
            let p = js_bigint_new_si64(ctx, v);
            if p.is_null() {
                JS_EXCEPTION
            } else {
                JS_MKPTR(JS_TAG_BIG_INT, p.cast())
            }
        }
    }
}
pub unsafe fn JS_NewBigUint64(ctx: *mut JSContext, v: u64) -> JSValue {
    if v <= (js_slimb_t::MAX as u64) {
        __JS_NewShortBigInt(ctx, v as js_slimb_t)
    } else {
        let p = js_bigint_new_ui64(ctx, v);
        if p.is_null() {
            JS_EXCEPTION
        } else {
            JS_MKPTR(JS_TAG_BIG_INT, p.cast())
        }
    }
}
