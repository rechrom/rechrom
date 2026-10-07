// quickjs.c BigInt limb arithmetic, original C lines 11274..11593. MIT.
// Unsigned C arithmetic wraps; preserve it explicitly in debug and release.
fn ADDC(op1: js_limb_t, op2: js_limb_t, carry_in: js_limb_t) -> (js_limb_t, js_limb_t) {
    let a = op1.wrapping_add(op2);
    let k1 = (a < op1) as js_limb_t;
    let res = a.wrapping_add(carry_in);
    (res, ((res < carry_in) as js_limb_t) | k1)
}
fn js_limb_clz(a: js_limb_t) -> js_limb_t {
    a.leading_zeros() as js_limb_t
}
fn js_limb_safe_clz(a: js_limb_t) -> js_limb_t {
    if a == 0 {
        JS_LIMB_BITS as js_limb_t
    } else {
        js_limb_clz(a)
    }
}
unsafe fn mp_add(
    res: *mut js_limb_t,
    op1: *const js_limb_t,
    op2: *const js_limb_t,
    n: js_limb_t,
    mut carry: js_limb_t,
) -> js_limb_t {
    for i in 0..n as usize {
        let (v, c) = ADDC(*op1.add(i), *op2.add(i), carry);
        *res.add(i) = v;
        carry = c;
    }
    carry
}
unsafe fn mp_sub(
    res: *mut js_limb_t,
    op1: *const js_limb_t,
    op2: *const js_limb_t,
    n: i32,
    carry: js_limb_t,
) -> js_limb_t {
    let mut k = carry;
    for i in 0..n as usize {
        let v = *op1.add(i);
        let a = v.wrapping_sub(*op2.add(i));
        let k1 = (a > v) as js_limb_t;
        let v = a.wrapping_sub(k);
        k = ((v > a) as js_limb_t) | k1;
        *res.add(i) = v;
    }
    k
}
unsafe fn mp_neg(res: *mut js_limb_t, op2: *const js_limb_t, n: i32) -> js_limb_t {
    let mut carry = 1;
    for i in 0..n as usize {
        let v = (!*op2.add(i)).wrapping_add(carry);
        carry = (v < carry) as js_limb_t;
        *res.add(i) = v;
    }
    carry
}
unsafe fn mp_mul1(
    tabr: *mut js_limb_t,
    taba: *const js_limb_t,
    n: js_limb_t,
    b: js_limb_t,
    mut l: js_limb_t,
) -> js_limb_t {
    for i in 0..n as usize {
        let t = *taba.add(i) as js_dlimb_t * b as js_dlimb_t + l as js_dlimb_t;
        *tabr.add(i) = t as js_limb_t;
        l = (t >> JS_LIMB_BITS) as js_limb_t;
    }
    l
}
unsafe fn mp_div1(
    tabr: *mut js_limb_t,
    taba: *const js_limb_t,
    n: js_limb_t,
    b: js_limb_t,
    mut r: js_limb_t,
) -> js_limb_t {
    for i in (0..n as usize).rev() {
        let a1 = ((r as js_dlimb_t) << JS_LIMB_BITS) | *taba.add(i) as js_dlimb_t;
        *tabr.add(i) = (a1 / b as js_dlimb_t) as js_limb_t;
        r = (a1 % b as js_dlimb_t) as js_limb_t;
    }
    r
}
unsafe fn mp_add_mul1(
    tabr: *mut js_limb_t,
    taba: *const js_limb_t,
    n: js_limb_t,
    b: js_limb_t,
) -> js_limb_t {
    let mut l = 0;
    for i in 0..n as usize {
        let t = *taba.add(i) as js_dlimb_t * b as js_dlimb_t
            + l as js_dlimb_t
            + *tabr.add(i) as js_dlimb_t;
        *tabr.add(i) = t as js_limb_t;
        l = (t >> JS_LIMB_BITS) as js_limb_t;
    }
    l
}
unsafe fn mp_mul_basecase(
    result: *mut js_limb_t,
    op1: *const js_limb_t,
    op1_size: js_limb_t,
    op2: *const js_limb_t,
    op2_size: js_limb_t,
) {
    *result.add(op1_size as usize) = mp_mul1(result, op1, op1_size, *op2, 0);
    for i in 1..op2_size as usize {
        let r = mp_add_mul1(result.add(i), op1, op1_size, *op2.add(i));
        *result.add(i + op1_size as usize) = r;
    }
}
unsafe fn mp_sub_mul1(
    tabr: *mut js_limb_t,
    taba: *const js_limb_t,
    n: js_limb_t,
    b: js_limb_t,
) -> js_limb_t {
    let mut l = 0;
    for i in 0..n as usize {
        let t = (*tabr.add(i) as js_dlimb_t)
            .wrapping_sub(*taba.add(i) as js_dlimb_t * b as js_dlimb_t)
            .wrapping_sub(l as js_dlimb_t);
        *tabr.add(i) = t as js_limb_t;
        l = (0 as js_dlimb_t).wrapping_sub(t >> JS_LIMB_BITS) as js_limb_t;
    }
    l
}
// d >= 2^(JS_LIMB_BITS-1).
fn udiv1norm_init(d: js_limb_t) -> js_limb_t {
    let a1 = d.wrapping_neg().wrapping_sub(1);
    let a0 = js_limb_t::MAX;
    ((((a1 as js_dlimb_t) << JS_LIMB_BITS) | a0 as js_dlimb_t) / d as js_dlimb_t) as js_limb_t
}
unsafe fn udiv1norm(
    pr: *mut js_limb_t,
    a1: js_limb_t,
    a0: js_limb_t,
    d: js_limb_t,
    d_inv: js_limb_t,
) -> js_limb_t {
    let n1m = ((a0 as js_slimb_t) >> (JS_LIMB_BITS - 1)) as js_limb_t;
    let n_adj = a0.wrapping_add(n1m & d);
    let a = (d_inv as js_dlimb_t * a1.wrapping_sub(n1m) as js_dlimb_t)
        .wrapping_add(n_adj as js_dlimb_t);
    let mut q = ((a >> JS_LIMB_BITS) as js_limb_t).wrapping_add(a1);
    let a = (((a1 as js_dlimb_t) << JS_LIMB_BITS) | a0 as js_dlimb_t)
        .wrapping_sub(q as js_dlimb_t * d as js_dlimb_t)
        .wrapping_sub(d as js_dlimb_t);
    let ah = (a >> JS_LIMB_BITS) as js_limb_t;
    q = q.wrapping_add(1).wrapping_add(ah);
    *pr = (a as js_limb_t).wrapping_add(ah & d);
    q
}
const UDIV1NORM_THRESHOLD: js_limb_t = 3;
unsafe fn mp_div1norm(
    tabr: *mut js_limb_t,
    taba: *const js_limb_t,
    n: js_limb_t,
    b: js_limb_t,
    mut r: js_limb_t,
) -> js_limb_t {
    if n >= UDIV1NORM_THRESHOLD {
        let b_inv = udiv1norm_init(b);
        for i in (0..n as usize).rev() {
            *tabr.add(i) = udiv1norm(&mut r, r, *taba.add(i), b, b_inv);
        }
    } else {
        for i in (0..n as usize).rev() {
            let a1 = ((r as js_dlimb_t) << JS_LIMB_BITS) | *taba.add(i) as js_dlimb_t;
            *tabr.add(i) = (a1 / b as js_dlimb_t) as js_limb_t;
            r = (a1 % b as js_dlimb_t) as js_limb_t;
        }
    }
    r
}
// na >= nb; tabb[nb-1] has its high bit set. taba becomes the remainder.
unsafe fn mp_divnorm(
    tabq: *mut js_limb_t,
    taba: *mut js_limb_t,
    na: js_limb_t,
    tabb: *const js_limb_t,
    nb: js_limb_t,
) {
    let b1 = *tabb.add(nb as usize - 1);
    if nb == 1 {
        *taba = mp_div1norm(tabq, taba, na, b1, 0);
        return;
    }
    let n = na - nb;
    let b1_inv = if n >= UDIV1NORM_THRESHOLD {
        udiv1norm_init(b1)
    } else {
        0
    };
    let mut q = 1;
    for j in (0..nb as usize).rev() {
        if *taba.add(n as usize + j) != *tabb.add(j) {
            if *taba.add(n as usize + j) < *tabb.add(j) {
                q = 0;
            }
            break;
        }
    }
    *tabq.add(n as usize) = q;
    if q != 0 {
        mp_sub(
            taba.add(n as usize),
            taba.add(n as usize),
            tabb,
            nb as i32,
            0,
        );
    }
    for i in (0..n as usize).rev() {
        if *taba.add(i + nb as usize) >= b1 {
            q = js_limb_t::MAX;
        } else if b1_inv != 0 {
            let mut dummy_r = 0;
            q = udiv1norm(
                &mut dummy_r,
                *taba.add(i + nb as usize),
                *taba.add(i + nb as usize - 1),
                b1,
                b1_inv,
            );
        } else {
            let al = ((*taba.add(i + nb as usize) as js_dlimb_t) << JS_LIMB_BITS)
                | *taba.add(i + nb as usize - 1) as js_dlimb_t;
            q = (al / b1 as js_dlimb_t) as js_limb_t;
        }
        let r = mp_sub_mul1(taba.add(i), tabb, nb, q);
        let v = *taba.add(i + nb as usize);
        let a = v.wrapping_sub(r);
        let mut c = (a > v) as js_limb_t;
        *taba.add(i + nb as usize) = a;
        if c != 0 {
            loop {
                q = q.wrapping_sub(1);
                c = mp_add(taba.add(i), taba.add(i), tabb, nb, 0);
                if c != 0 {
                    let high = taba.add(i + nb as usize);
                    *high = (*high).wrapping_add(1);
                    if *high == 0 {
                        break;
                    }
                }
            }
        }
        *tabq.add(i) = q;
    }
}
unsafe fn mp_shl(tabr: *mut js_limb_t, taba: *const js_limb_t, n: i32, shift: i32) -> js_limb_t {
    let mut l = 0;
    for i in 0..n as usize {
        let v = *taba.add(i);
        *tabr.add(i) = (v << shift) | l;
        l = v >> (JS_LIMB_BITS as i32 - shift);
    }
    l
}
unsafe fn mp_shr(
    tab_r: *mut js_limb_t,
    tab: *const js_limb_t,
    n: i32,
    shift: i32,
    high: js_limb_t,
) -> js_limb_t {
    let mut l = high;
    for i in (0..n as usize).rev() {
        let a = *tab.add(i);
        *tab_r.add(i) = (a >> shift) | (l << (JS_LIMB_BITS as i32 - shift));
        l = a;
    }
    l & (((1 as js_limb_t) << shift) - 1)
}
unsafe fn js_bigint_set_si(buf: *mut JSBigIntBuf, a: js_slimb_t) -> *mut JSBigInt {
    let r = ptr::addr_of_mut!((*buf).big_int_buf).cast::<JSBigInt>();
    (*r).len = 1;
    *ptr::addr_of_mut!((*r).tab).cast::<js_limb_t>() = a as js_limb_t;
    r
}
unsafe fn js_bigint_set_si64(buf: *mut JSBigIntBuf, a: i64) -> *mut JSBigInt {
    #[cfg(target_pointer_width = "64")]
    {
        js_bigint_set_si(buf, a)
    }
    #[cfg(target_pointer_width = "32")]
    {
        let r = ptr::addr_of_mut!((*buf).big_int_buf).cast::<JSBigInt>();
        let tab = ptr::addr_of_mut!((*r).tab).cast::<js_limb_t>();
        if a >= i32::MIN as i64 && a <= i32::MAX as i64 {
            (*r).len = 1;
            *tab = a as js_limb_t;
        } else {
            (*r).len = 2;
            *tab = a as js_limb_t;
            *tab.add(1) = (a >> JS_LIMB_BITS) as js_limb_t;
        }
        r
    }
}
unsafe fn js_bigint_set_short(buf: *mut JSBigIntBuf, val: JSValueConst) -> *mut JSBigInt {
    js_bigint_set_si(buf, JS_VALUE_GET_SHORT_BIG_INT(val))
}
unsafe fn js_bigint_sign(a: *const JSBigInt) -> i32 {
    (*ptr::addr_of!((*a).tab)
        .cast::<js_limb_t>()
        .add((*a).len as usize - 1)
        >> (JS_LIMB_BITS - 1)) as i32
}
unsafe fn js_bigint_get_si_sat(a: *const JSBigInt) -> js_slimb_t {
    if (*a).len == 1 {
        *ptr::addr_of!((*a).tab).cast::<js_limb_t>() as js_slimb_t
    } else if js_bigint_sign(a) != 0 {
        js_slimb_t::MIN
    } else {
        js_slimb_t::MAX
    }
}
// quickjs.c C lines 12188..12288, 12338..12426.
unsafe fn js_bigint_get_mant_exp(_ctx: *mut JSContext, pexp: *mut i32, a: *const JSBigInt) -> u64 {
    let mut t = [0 as js_limb_t; 4 - JS_LIMB_BITS / 32];
    let n2 = t.len() as i32;
    let n1 = (*a).len as i32 - n2;
    let sgn = js_bigint_sign(a);
    let sign_mask = (sgn as js_limb_t).wrapping_neg();
    let tab = ptr::addr_of!((*a).tab).cast::<js_limb_t>();
    let mut low_bits = 0;
    let mut carry = sgn as js_limb_t;
    for i in 0..n1 {
        let v = (*tab.add(i as usize) ^ sign_mask).wrapping_add(carry);
        carry = (v < carry) as js_limb_t;
        low_bits |= v;
    }
    for j in 0..n2 {
        let i = j + n1;
        let v = if i < 0 {
            0
        } else {
            let v = (*tab.add(i as usize) ^ sign_mask).wrapping_add(carry);
            carry = (v < carry) as js_limb_t;
            v
        };
        t[j as usize] = v;
    }
    #[cfg(target_pointer_width = "32")]
    let (mut a1, mut a0) = (((t[2] as u64) << 32) | t[1] as u64, (t[0] as u64) << 32);
    #[cfg(target_pointer_width = "64")]
    let (mut a1, mut a0) = (t[1], t[0]);
    a0 |= (low_bits != 0) as u64;
    let shift;
    if a1 == 0 {
        shift = 64;
        a1 = a0;
        a0 = 0;
    } else {
        shift = a1.leading_zeros() as i32;
        if shift != 0 {
            a1 = (a1 << shift) | (a0 >> (64 - shift));
            a0 <<= shift;
        }
    }
    a1 |= (a0 != 0) as u64;
    *pexp = (*a).len as i32 * JS_LIMB_BITS as i32 - shift - 1;
    a1
}
fn shr_rndn(a: u64, n: i32) -> u64 {
    // The original int literal is used only for the valid n=10 call site.
    let addend = ((a >> n) & 1) + ((1u64 << (n - 1)) - 1);
    a.wrapping_add(addend) >> n
}
unsafe fn js_bigint_to_float64(ctx: *mut JSContext, a: *const JSBigInt) -> f64 {
    if (*a).len == 1 {
        return *ptr::addr_of!((*a).tab).cast::<js_limb_t>() as js_slimb_t as f64;
    }
    let sgn = js_bigint_sign(a);
    let mut e = 0;
    let mut mant = js_bigint_get_mant_exp(ctx, &mut e, a);
    if e > 1023 {
        mant = 0;
        e = 1024;
    } else {
        mant = (mant >> 1) | (mant & 1);
        mant = shr_rndn(mant, 10);
        if mant >= 1u64 << 53 {
            mant >>= 1;
            e += 1;
        }
        mant &= (1u64 << 52) - 1;
    }
    f64::from_bits(((sgn as u64) << 63) | (((e + 1023) as u64) << 52) | mant)
}
unsafe fn js_bigint_float64_cmp(ctx: *mut JSContext, a: *const JSBigInt, b: f64) -> i32 {
    let b1 = b.to_bits();
    let b_sign = (b1 >> 63) as i32;
    let mut e = ((b1 >> 52) & ((1 << 11) - 1)) as i32;
    let mut mant = b1 & ((1u64 << 52) - 1);
    let a_sign = js_bigint_sign(a);
    let tab = ptr::addr_of!((*a).tab).cast::<js_limb_t>();
    if e == 2047 {
        if mant != 0 {
            2
        } else {
            2 * b_sign - 1
        }
    } else if e == 0 && mant == 0 {
        if (*a).len == 1 && *tab == 0 {
            0
        } else {
            1 - 2 * a_sign
        }
    } else if (*a).len == 1 && *tab == 0 {
        2 * b_sign - 1
    } else if a_sign != b_sign {
        1 - 2 * a_sign
    } else {
        e -= 1023;
        let mut f = 0;
        let a_mant = js_bigint_get_mant_exp(ctx, &mut f, a);
        if f != e {
            if f < e {
                -1
            } else {
                1
            }
        } else {
            mant = (mant | (1u64 << 52)) << 11;
            if a_mant < mant {
                2 * a_sign - 1
            } else if a_mant > mant {
                1 - 2 * a_sign
            } else {
                0
            }
        }
    }
}
unsafe fn js_bigint_cmp(_ctx: *mut JSContext, a: *const JSBigInt, b: *const JSBigInt) -> i32 {
    let a_sign = js_bigint_sign(a);
    let b_sign = js_bigint_sign(b);
    if a_sign != b_sign {
        return 1 - 2 * a_sign;
    }
    if (*a).len != (*b).len {
        return if (*a).len < (*b).len {
            2 * a_sign - 1
        } else {
            1 - 2 * a_sign
        };
    }
    let ta = ptr::addr_of!((*a).tab).cast::<js_limb_t>();
    let tb = ptr::addr_of!((*b).tab).cast::<js_limb_t>();
    for i in (0..(*a).len as usize).rev() {
        if *ta.add(i) != *tb.add(i) {
            return if *ta.add(i) < *tb.add(i) { -1 } else { 1 };
        }
    }
    0
}
