//! dtoa.c: Bellard float64 printing/parsing algorithm. Copyright 2024 Fabrice
//! Bellard; MIT, see ../LICENSE. Enabled configuration: 32-bit limbs,
//! USE_POW5_TABLE, USE_FAST_INT, fixed caller-provided scratch memory.
use super::cutils::*;
pub use super::dtoa_header::*;
use core::{ffi::c_char, ptr};
include!("dtoa_tables.rs");
type slimb_t = i32;
type limb_t = u32;
type dlimb_t = u64;
type mp_size_t = isize;
const LIMB_LOG2_BITS: i32 = 5;
const LIMB_BITS: i32 = 1 << LIMB_LOG2_BITS;
#[allow(dead_code)]
const LIMB_DIGITS: i32 = 9;
#[allow(dead_code)]
const JS_RADIX_MAX: i32 = 36;
const DBIGNUM_LEN_MAX: usize = 52;
const MANT_LEN_MAX: usize = 18;
const JS_RNDN: i32 = 0;
const JS_RNDNA: i32 = 1;
const JS_RNDZ: i32 = 2;
const MUL_LOG2_RADIX_BASE_LOG2: i32 = 24;
#[repr(C)]
struct mpb_t {
    len: i32,
    tab: [limb_t; 0],
}
unsafe fn tab(r: *mut mpb_t) -> *mut limb_t {
    ptr::addr_of_mut!((*r).tab).cast()
}
unsafe fn ctab(r: *const mpb_t) -> *const limb_t {
    ptr::addr_of!((*r).tab).cast()
}
unsafe fn mp_add_ui(tab: *mut limb_t, b: limb_t, n: usize) -> limb_t {
    let mut k = b;
    for i in 0..n {
        if k == 0 {
            break;
        }
        let a = (*tab.add(i)).wrapping_add(k);
        k = (a < k) as limb_t;
        *tab.add(i) = a;
    }
    k
}
unsafe fn mp_mul1(
    tabr: *mut limb_t,
    taba: *const limb_t,
    n: limb_t,
    b: limb_t,
    mut l: limb_t,
) -> limb_t {
    for i in 0..n as usize {
        let t = (*taba.add(i) as dlimb_t) * b as dlimb_t + l as dlimb_t;
        *tabr.add(i) = t as limb_t;
        l = (t >> LIMB_BITS) as limb_t;
    }
    l
}
fn udiv1norm_init(d: limb_t) -> limb_t {
    let a1 = d.wrapping_neg().wrapping_sub(1);
    let a0 = limb_t::MAX;
    ((((a1 as dlimb_t) << LIMB_BITS) | a0 as dlimb_t) / d as dlimb_t) as limb_t
}
fn udiv1norm(pr: &mut limb_t, a1: limb_t, a0: limb_t, d: limb_t, d_inv: limb_t) -> limb_t {
    let n1m = ((a0 as slimb_t) >> (LIMB_BITS - 1)) as limb_t;
    let n_adj = a0.wrapping_add(n1m & d);
    let mut a = (d_inv as dlimb_t)
        .wrapping_mul(a1.wrapping_sub(n1m) as dlimb_t)
        .wrapping_add(n_adj as dlimb_t);
    let mut q = ((a >> LIMB_BITS) as limb_t).wrapping_add(a1);
    a = ((a1 as dlimb_t) << LIMB_BITS) | a0 as dlimb_t;
    a = a
        .wrapping_sub(q as dlimb_t * d as dlimb_t)
        .wrapping_sub(d as dlimb_t);
    let ah = (a >> LIMB_BITS) as limb_t;
    q = q.wrapping_add(1).wrapping_add(ah);
    *pr = (a as limb_t).wrapping_add(ah & d);
    q
}
unsafe fn mp_div1(
    tabr: *mut limb_t,
    taba: *const limb_t,
    n: limb_t,
    b: limb_t,
    mut r: limb_t,
) -> limb_t {
    for i in (0..n as usize).rev() {
        let a1 = ((r as dlimb_t) << LIMB_BITS) | *taba.add(i) as dlimb_t;
        *tabr.add(i) = (a1 / b as dlimb_t) as limb_t;
        r = (a1 % b as dlimb_t) as limb_t;
    }
    r
}
unsafe fn mp_shr(
    tab_r: *mut limb_t,
    tab: *const limb_t,
    n: mp_size_t,
    shift: i32,
    high: limb_t,
) -> limb_t {
    assert!(shift >= 1 && shift < LIMB_BITS);
    let mut l = high;
    for i in (0..n as usize).rev() {
        let a = *tab.add(i);
        *tab_r.add(i) = (a >> shift) | (l << (LIMB_BITS - shift));
        l = a;
    }
    l & ((1u32 << shift) - 1)
}
unsafe fn mp_shl(
    tab_r: *mut limb_t,
    tab: *const limb_t,
    n: mp_size_t,
    shift: i32,
    low: limb_t,
) -> limb_t {
    assert!(shift >= 1 && shift < LIMB_BITS);
    let mut l = low;
    for i in 0..n as usize {
        let a = *tab.add(i);
        *tab_r.add(i) = (a << shift) | l;
        l = a >> (LIMB_BITS - shift);
    }
    l
}
#[inline(never)]
unsafe fn mp_div1norm(
    tabr: *mut limb_t,
    taba: *const limb_t,
    n: limb_t,
    b: limb_t,
    mut r: limb_t,
    b_inv: limb_t,
    shift: i32,
) -> limb_t {
    if shift != 0 {
        r = (r << shift) | mp_shl(tabr, taba, n as mp_size_t, shift, 0);
    }
    for i in (0..n as usize).rev() {
        let a1 = r;
        *tabr.add(i) = udiv1norm(&mut r, a1, *taba.add(i), b, b_inv);
    }
    r >> shift
}
#[allow(dead_code)]
unsafe fn mpb_dump(str: &str, a: *const mpb_t) {
    print!("{str}= 0x");
    for i in (0..(*a).len as usize).rev() {
        print!("{:08x}", *ctab(a).add(i));
        if i != 0 {
            print!("_");
        }
    }
    println!();
}
unsafe fn mpb_renorm(r: *mut mpb_t) {
    while (*r).len > 1 && *tab(r).add((*r).len as usize - 1) == 0 {
        (*r).len -= 1;
    }
}
fn pow_ui(a: u32, b: u32) -> u64 {
    if b == 0 {
        return 1;
    }
    if b == 1 {
        return a as u64;
    }
    if (a == 5 || a == 10) && b <= 17 {
        let mut r = pow5_table[b as usize - 1] as u64;
        if b >= 14 {
            r |= (pow5h_table[b as usize - 14] as u64) << 32;
        }
        if a == 10 {
            r <<= b;
        }
        return r;
    }
    let mut r = a as u64;
    let n_bits = 32 - clz32(b);
    for i in (0..=n_bits - 2).rev() {
        r = r.wrapping_mul(r);
        if (b >> i) & 1 != 0 {
            r = r.wrapping_mul(a as u64);
        }
    }
    r
}
fn pow_ui_inv(pr_inv: &mut u32, pshift: &mut i32, a: u32, b: u32) -> u32 {
    let (r, r_inv, shift) = if a == 5 && b >= 1 && b <= 13 {
        let r = pow5_table[b as usize - 1];
        let shift = clz32(r);
        (r << shift, pow5_inv_table[b as usize - 1], shift)
    } else {
        let r = pow_ui(a, b) as u32;
        let shift = clz32(r);
        let r = r << shift;
        (r, udiv1norm_init(r), shift)
    };
    *pshift = shift;
    *pr_inv = r_inv;
    r
}
unsafe fn mpb_get_bit(r: *const mpb_t, k: i32) -> i32 {
    let l = (k as u32 / LIMB_BITS as u32) as usize;
    let k = k & (LIMB_BITS - 1);
    if l >= (*r).len as usize {
        0
    } else {
        ((*ctab(r).add(l) >> k) & 1) as i32
    }
}
unsafe fn mpb_shr_round(r: *mut mpb_t, mut shift: i32, rnd_mode: i32) {
    if shift == 0 {
        return;
    }
    if shift < 0 {
        shift = -shift;
        let l = (shift as u32 / LIMB_BITS as u32) as usize;
        shift &= LIMB_BITS - 1;
        if shift != 0 {
            let carry = mp_shl(tab(r), ctab(r), (*r).len as mp_size_t, shift, 0);
            *tab(r).add((*r).len as usize) = carry;
            (*r).len += 1;
            mpb_renorm(r);
        }
        if l > 0 {
            for i in (0..(*r).len as usize).rev() {
                *tab(r).add(i + l) = *ctab(r).add(i);
            }
            for i in 0..l {
                *tab(r).add(i) = 0;
            }
            (*r).len += l as i32;
        }
    } else {
        let add_one;
        if rnd_mode == JS_RNDN || rnd_mode == JS_RNDNA {
            let bit1 = mpb_get_bit(r, shift - 1);
            if bit1 != 0 {
                let mut bit2 = 0;
                if rnd_mode == JS_RNDNA {
                    bit2 = 1;
                } else if shift >= 2 {
                    let mut k = shift - 1;
                    let l = (k as u32 / LIMB_BITS as u32) as usize;
                    k &= LIMB_BITS - 1;
                    for i in 0..l.min((*r).len as usize) {
                        bit2 |= *ctab(r).add(i);
                    }
                    if l < (*r).len as usize {
                        bit2 |= *ctab(r).add(l) & ((1u32 << k) - 1);
                    }
                }
                add_one = if bit2 != 0 { 1 } else { mpb_get_bit(r, shift) };
            } else {
                add_one = 0;
            }
        } else {
            add_one = 0;
        }
        let l = (shift as u32 / LIMB_BITS as u32) as usize;
        shift &= LIMB_BITS - 1;
        if l >= (*r).len as usize {
            (*r).len = 1;
            *tab(r) = add_one as limb_t;
        } else {
            if l > 0 {
                (*r).len -= l as i32;
                for i in 0..(*r).len as usize {
                    *tab(r).add(i) = *ctab(r).add(i + l);
                }
            }
            if shift != 0 {
                mp_shr(tab(r), ctab(r), (*r).len as mp_size_t, shift, 0);
                mpb_renorm(r);
            }
            if add_one != 0 {
                let a = mp_add_ui(tab(r), 1, (*r).len as usize);
                if a != 0 {
                    *tab(r).add((*r).len as usize) = a;
                    (*r).len += 1;
                }
            }
        }
    }
}
unsafe fn mpb_cmp(a: *const mpb_t, b: *const mpb_t) -> i32 {
    if (*a).len < (*b).len {
        return -1;
    }
    if (*a).len > (*b).len {
        return 1;
    }
    for i in (0..(*a).len as usize).rev() {
        let (x, y) = (*ctab(a).add(i), *ctab(b).add(i));
        if x != y {
            return if x < y { -1 } else { 1 };
        }
    }
    0
}
unsafe fn mpb_set_u64(r: *mut mpb_t, m: u64) {
    *tab(r) = m as limb_t;
    *tab(r).add(1) = (m >> LIMB_BITS) as limb_t;
    (*r).len = if *ctab(r).add(1) == 0 { 1 } else { 2 };
}
unsafe fn mpb_get_u64(r: *const mpb_t) -> u64 {
    if (*r).len == 1 {
        *ctab(r) as u64
    } else {
        *ctab(r) as u64 | ((*ctab(r).add(1) as u64) << LIMB_BITS)
    }
}
unsafe fn mpb_floor_log2(a: *const mpb_t) -> i32 {
    let v = *ctab(a).add((*a).len as usize - 1);
    if v == 0 {
        -1
    } else {
        (*a).len * LIMB_BITS - 1 - clz32(v)
    }
}
fn mul_log2_radix(mut a: i32, radix: i32) -> i32 {
    if radix & (radix - 1) == 0 {
        let radix_bits = 31 - clz32(radix as u32);
        if a < 0 {
            a -= radix_bits - 1;
        }
        a / radix_bits
    } else {
        let mult = mul_log2_radix_table[radix as usize - 2];
        ((a as i64 * mult as i64) >> MUL_LOG2_RADIX_BASE_LOG2) as i32
    }
}
unsafe fn u32toa_len(buf: *mut c_char, mut n: u32, len: usize) {
    for i in (0..len).rev() {
        let digit = n % 10;
        n /= 10;
        *buf.add(i) = (digit + b'0' as u32) as c_char;
    }
}
unsafe fn u64toa_bin_len(buf: *mut c_char, mut n: u64, radix_bits: u32, len: i32) {
    let mask = (1u32 << radix_bits) - 1;
    for i in (0..len as usize).rev() {
        let mut digit = (n & mask as u64) as u32;
        n >>= radix_bits;
        digit += if digit < 10 {
            b'0' as u32
        } else {
            b'a' as u32 - 10
        };
        *buf.add(i) = digit as c_char;
    }
}
unsafe fn limb_to_a(buf: *mut c_char, mut n: limb_t, radix: u32, len: i32) {
    if radix == 10 {
        u32toa_len(buf, n, len as usize);
    } else {
        for i in (0..len as usize).rev() {
            let mut digit = n % radix;
            n /= radix;
            digit += if digit < 10 {
                b'0' as u32
            } else {
                b'a' as u32 - 10
            };
            *buf.add(i) = digit as c_char;
        }
    }
}
pub unsafe fn u32toa(buf: *mut c_char, mut n: u32) -> usize {
    let mut buf1 = [0u8; 10];
    let mut q = buf1.len();
    loop {
        q -= 1;
        buf1[q] = (n % 10) as u8 + b'0';
        n /= 10;
        if n == 0 {
            break;
        }
    }
    let len = buf1.len() - q;
    ptr::copy_nonoverlapping(buf1.as_ptr().add(q), buf.cast(), len);
    len
}
pub unsafe fn i32toa(buf: *mut c_char, n: i32) -> usize {
    if n >= 0 {
        u32toa(buf, n as u32)
    } else {
        *buf = b'-' as c_char;
        u32toa(buf.add(1), (n as u32).wrapping_neg()) + 1
    }
}
pub unsafe fn u64toa(buf: *mut c_char, mut n: u64) -> usize {
    if n < 0x100000000 {
        return u32toa(buf, n as u32);
    }
    let mut q = buf;
    let mut n1 = n / 1000000000;
    n %= 1000000000;
    if n1 >= 0x100000000 {
        let mut n2 = (n1 / 1000000000) as u32;
        n1 %= 1000000000;
        if n2 >= 10 {
            *q = (n2 / 10 + b'0' as u32) as c_char;
            q = q.add(1);
            n2 %= 10;
        }
        *q = (n2 + b'0' as u32) as c_char;
        q = q.add(1);
        u32toa_len(q, n1 as u32, 9);
        q = q.add(9);
    } else {
        q = q.add(u32toa(q, n1 as u32));
    }
    u32toa_len(q, n as u32, 9);
    q = q.add(9);
    q.offset_from(buf) as usize
}
pub unsafe fn i64toa(buf: *mut c_char, n: i64) -> usize {
    if n >= 0 {
        u64toa(buf, n as u64)
    } else {
        *buf = b'-' as c_char;
        u64toa(buf.add(1), (n as u64).wrapping_neg()) + 1
    }
}
pub unsafe fn u64toa_radix(buf: *mut c_char, mut n: u64, radix: u32) -> usize {
    if radix == 10 {
        return u64toa(buf, n);
    }
    if radix & (radix - 1) == 0 {
        let radix_bits = 31 - clz32(radix);
        let l = if n == 0 {
            1
        } else {
            (64 - clz64(n) + radix_bits - 1) / radix_bits
        };
        u64toa_bin_len(buf, n, radix_bits as u32, l);
        l as usize
    } else {
        let mut buf1 = [0u8; 41];
        let mut q = buf1.len();
        loop {
            let mut digit = (n % radix as u64) as u8;
            n /= radix as u64;
            digit += if digit < 10 { b'0' } else { b'a' - 10 };
            q -= 1;
            buf1[q] = digit;
            if n == 0 {
                break;
            }
        }
        let len = buf1.len() - q;
        ptr::copy_nonoverlapping(buf1.as_ptr().add(q), buf.cast(), len);
        len
    }
}
pub unsafe fn i64toa_radix(buf: *mut c_char, n: i64, radix: u32) -> usize {
    if n >= 0 {
        u64toa_radix(buf, n as u64, radix)
    } else {
        *buf = b'-' as c_char;
        u64toa_radix(buf.add(1), (n as u64).wrapping_neg(), radix) + 1
    }
}
unsafe fn output_digits(
    buf: *mut c_char,
    a: *mut mpb_t,
    radix: i32,
    n_digits1: i32,
    dot_pos: i32,
) -> i32 {
    let mut n_digits = n_digits1;
    let radix_bits = if radix & (radix - 1) == 0 {
        31 - clz32(radix as u32)
    } else {
        0
    };
    let digits_per_limb = digits_per_limb_table[radix as usize - 2] as i32;
    if radix_bits != 0 {
        loop {
            let n = min_int(n_digits, digits_per_limb);
            n_digits -= n;
            u64toa_bin_len(
                buf.add(n_digits as usize),
                *ctab(a) as u64,
                radix_bits as u32,
                n,
            );
            if n_digits == 0 {
                break;
            }
            mpb_shr_round(a, digits_per_limb * radix_bits, JS_RNDZ);
        }
    } else {
        while n_digits != 0 {
            let n = min_int(n_digits, digits_per_limb);
            n_digits -= n;
            let r = mp_div1(
                tab(a),
                ctab(a),
                (*a).len as limb_t,
                radix_base_table[radix as usize - 2],
                0,
            );
            mpb_renorm(a);
            limb_to_a(buf.add(n_digits as usize), r, radix as u32, n);
        }
    }
    let mut len = n_digits1;
    if dot_pos != n_digits1 {
        ptr::copy(
            buf.add(dot_pos as usize),
            buf.add(dot_pos as usize + 1),
            (n_digits1 - dot_pos) as usize,
        );
        *buf.add(dot_pos as usize) = b'.' as c_char;
        len += 1;
    }
    len
}
unsafe fn mul_pow(
    a: *mut mpb_t,
    radix1: i32,
    radix_shift: i32,
    mut f: i32,
    is_int: BOOL,
    e: i32,
) -> i32 {
    let mut e_offset = -f * radix_shift;
    if radix1 != 1 {
        let d = digits_per_limb_table[radix1 as usize - 2] as i32;
        if f >= 0 {
            let (mut b, mut n0) = (0u32, 0);
            while f != 0 {
                let n = min_int(f, d);
                if n != n0 {
                    b = pow_ui(radix1 as u32, n as u32) as limb_t;
                    n0 = n;
                }
                let h = mp_mul1(tab(a), ctab(a), (*a).len as limb_t, b, 0);
                if h != 0 {
                    *tab(a).add((*a).len as usize) = h;
                    (*a).len += 1;
                }
                f -= n;
            }
        } else {
            f = -f;
            let l = (f + d - 1) / d;
            e_offset += l * LIMB_BITS;
            let extra_bits = if is_int == 0 {
                max_int(e - mpb_floor_log2(a), 0)
            } else {
                max_int(2 + e - e_offset, 0)
            };
            e_offset += extra_bits;
            mpb_shr_round(a, -(l * LIMB_BITS + extra_bits), JS_RNDZ);
            let (mut b, mut b_inv, mut shift, mut n0, mut rem) = (0, 0, 0, 0, 0);
            while f != 0 {
                let n = min_int(f, d);
                if n != n0 {
                    b = pow_ui_inv(&mut b_inv, &mut shift, radix1 as u32, n as u32);
                    n0 = n;
                }
                let r = mp_div1norm(tab(a), ctab(a), (*a).len as limb_t, b, 0, b_inv, shift);
                rem |= r;
                mpb_renorm(a);
                f -= n;
            }
            *tab(a) |= (rem != 0) as limb_t;
        }
    }
    e_offset
}
unsafe fn mul_pow_round(
    tmp1: *mut mpb_t,
    m: u64,
    e: i32,
    radix1: i32,
    radix_shift: i32,
    f: i32,
    rnd_mode: i32,
) {
    mpb_set_u64(tmp1, m);
    let e_offset = mul_pow(tmp1, radix1, radix_shift, f, TRUE, e);
    mpb_shr_round(tmp1, -e + e_offset, rnd_mode);
}
unsafe fn round_to_d(pe: &mut i32, a: *mut mpb_t, e_offset: i32, rnd_mode: i32) -> u64 {
    let (mut e, mut m);
    if *ctab(a) == 0 && (*a).len == 1 {
        m = 0;
        e = 0;
    } else {
        e = mpb_floor_log2(a) + 1 - e_offset;
        let prec1 = 53;
        let e_min = -1021;
        let prec = if e < e_min {
            prec1 - (e_min - e)
        } else {
            prec1
        };
        mpb_shr_round(a, e + e_offset - prec, rnd_mode);
        m = mpb_get_u64(a);
        m = m.wrapping_shl((53 - prec) as u32);
        if m >= 1u64 << 53 {
            m >>= 1;
            e += 1;
        }
    }
    *pe = e;
    m
}
unsafe fn mul_pow_round_to_d(
    pe: &mut i32,
    a: *mut mpb_t,
    radix1: i32,
    radix_shift: i32,
    f: i32,
    rnd_mode: i32,
) -> u64 {
    let e_offset = mul_pow(a, radix1, radix_shift, f, FALSE, 55);
    round_to_d(pe, a, e_offset, rnd_mode)
}
pub fn js_dtoa_max_len(d: f64, radix: i32, n_digits: i32, flags: i32) -> i32 {
    let fmt = flags & JS_DTOA_FORMAT_MASK;
    let mut n;
    if fmt != JS_DTOA_FORMAT_FRAC {
        n = if fmt == JS_DTOA_FORMAT_FREE {
            dtoa_max_digits_table[radix as usize - 2] as i32
        } else {
            n_digits
        };
        if flags & JS_DTOA_EXP_MASK == JS_DTOA_EXP_DISABLED {
            let a = float64_as_uint64(d);
            let mut e = ((a >> 52) & 0x7ff) as i32;
            if e == 0x7ff {
                n = 0;
            } else {
                e -= 1023;
                n += 10 + mul_log2_radix(e - 1, radix).abs();
            }
        } else {
            n += 1 + 1 + 6;
        }
    } else {
        let a = float64_as_uint64(d);
        let mut e = ((a >> 52) & 0x7ff) as i32;
        if e == 0x7ff {
            n = 0;
        } else {
            e -= 1023;
            n = if e < 0 {
                1
            } else {
                2 + mul_log2_radix(e - 1, radix)
            };
            n += 1 + 1 + 1 + n_digits;
        }
    }
    max_int(n, 9)
}
unsafe fn dtoa_malloc(pptr: &mut *mut u64, size: usize) -> *mut mpb_t {
    let ret = *pptr;
    *pptr = (*pptr).add((size + 7) / 8);
    ret.cast()
}
fn dtoa_free(_ptr: *mut mpb_t) {}
unsafe fn emit(q: &mut *mut c_char, c: u8) {
    **q = c as c_char;
    *q = (*q).add(1);
}
unsafe fn dtoa_finish(
    buf: *mut c_char,
    q: *mut c_char,
    tmp1: *mut mpb_t,
    mant_max: *mut mpb_t,
) -> i32 {
    *q = 0;
    dtoa_free(mant_max);
    dtoa_free(tmp1);
    q.offset_from(buf) as i32
}
pub unsafe fn js_dtoa(
    buf: *mut c_char,
    d: f64,
    radix: i32,
    n_digits: i32,
    flags: i32,
    tmp_mem: *mut JSDTOATempMem,
) -> i32 {
    let mut mptr = (*tmp_mem).mem.as_mut_ptr();
    let tmp1 = dtoa_malloc(
        &mut mptr,
        core::mem::size_of::<mpb_t>() + core::mem::size_of::<limb_t>() * DBIGNUM_LEN_MAX,
    );
    let mant_max = dtoa_malloc(
        &mut mptr,
        core::mem::size_of::<mpb_t>() + core::mem::size_of::<limb_t>() * MANT_LEN_MAX,
    );
    assert!(
        mptr.offset_from((*tmp_mem).mem.as_ptr()) as usize
            <= core::mem::size_of::<JSDTOATempMem>() / 8
    );
    let fmt = flags & JS_DTOA_FORMAT_MASK;
    let radix_shift = ctz32(radix as u32);
    let radix1 = radix >> radix_shift;
    let a = float64_as_uint64(d);
    let sgn = (a >> 63) as i32;
    let mut e = ((a >> 52) & 0x7ff) as i32;
    let mut m = a & ((1u64 << 52) - 1);
    let mut q = buf;
    if e == 0x7ff {
        if m == 0 {
            if sgn != 0 {
                emit(&mut q, b'-');
            }
            ptr::copy_nonoverlapping(b"Infinity".as_ptr(), q.cast(), 8);
            q = q.add(8);
        } else {
            ptr::copy_nonoverlapping(b"NaN".as_ptr(), q.cast(), 3);
            q = q.add(3);
        }
        return dtoa_finish(buf, q, tmp1, mant_max);
    }
    let (mut E, mut P);
    if e == 0 && m == 0 {
        (*tmp1).len = 1;
        *tab(tmp1) = 0;
        E = 1;
        P = if fmt == JS_DTOA_FORMAT_FREE {
            1
        } else if fmt == JS_DTOA_FORMAT_FRAC {
            n_digits + 1
        } else {
            n_digits
        };
        if sgn != 0 && flags & JS_DTOA_MINUS_ZERO != 0 {
            emit(&mut q, b'-');
        }
    } else {
        if e == 0 {
            let l = clz64(m) - 11;
            e -= l - 1;
            m <<= l;
        } else {
            m |= 1u64 << 52;
        }
        if sgn != 0 {
            emit(&mut q, b'-');
        }
        e -= 1022;
        if fmt == JS_DTOA_FORMAT_FREE
            && e >= 1
            && e <= 53
            && m & ((1u64 << (53 - e)) - 1) == 0
            && flags & JS_DTOA_EXP_MASK != JS_DTOA_EXP_ENABLED
        {
            m >>= 53 - e;
            q = q.add(u64toa_radix(q, m, radix as u32));
            return dtoa_finish(buf, q, tmp1, mant_max);
        }
        E = 1 + mul_log2_radix(e - 1, radix);
        if fmt == JS_DTOA_FORMAT_FREE {
            let P_max = dtoa_max_digits_table[radix as usize - 2] as i32;
            let E0 = E;
            let (mut E_found, mut P_found, mut mant_found) = (0, 0, 0u64);
            P = P_max;
            loop {
                let mant_max1 = pow_ui(radix as u32, P as u32);
                E = E0;
                let mut mant;
                loop {
                    mul_pow_round(tmp1, m, e - 53, radix1, radix_shift, P - E, JS_RNDN);
                    mant = mpb_get_u64(tmp1);
                    if mant < mant_max1 {
                        break;
                    }
                    E += 1;
                }
                while mant % radix as u64 == 0 {
                    mant /= radix as u64;
                    P -= 1;
                }
                let found = if P_found == 0 {
                    true
                } else {
                    mpb_set_u64(tmp1, mant);
                    let mut e1 = 0;
                    let m1 = mul_pow_round_to_d(&mut e1, tmp1, radix1, radix_shift, E - P, JS_RNDN);
                    m1 == m && e1 == e
                };
                if found {
                    P_found = P;
                    E_found = E;
                    mant_found = mant;
                    if P == 1 {
                        break;
                    }
                    P -= 1;
                } else {
                    break;
                }
            }
            P = P_found;
            E = E_found;
            mpb_set_u64(tmp1, mant_found);
        } else if fmt == JS_DTOA_FORMAT_FRAC {
            assert!(n_digits >= 0 && n_digits <= JS_DTOA_MAX_DIGITS);
            mul_pow_round(tmp1, m, e - 53, radix1, radix_shift, n_digits, JS_RNDNA);
            let mut len = output_digits(
                q,
                tmp1,
                radix,
                max_int(E + 1, 1) + n_digits,
                max_int(E + 1, 1),
            );
            if *q == b'0' as c_char && len >= 2 && *q.add(1) != b'.' as c_char {
                len -= 1;
                ptr::copy(q.add(1), q, len as usize);
            }
            q = q.add(len as usize);
            return dtoa_finish(buf, q, tmp1, mant_max);
        } else {
            assert!(n_digits >= 1 && n_digits <= JS_DTOA_MAX_DIGITS);
            P = n_digits;
            (*mant_max).len = 1;
            *tab(mant_max) = 1;
            let pow_shift = mul_pow(mant_max, radix1, radix_shift, P, FALSE, 0);
            mpb_shr_round(mant_max, pow_shift, JS_RNDZ);
            loop {
                mul_pow_round(tmp1, m, e - 53, radix1, radix_shift, P - E, JS_RNDNA);
                if mpb_cmp(tmp1, mant_max) < 0 {
                    break;
                }
                E += 1;
            }
        }
    }
    let E_max = if fmt == JS_DTOA_FORMAT_FIXED {
        n_digits
    } else {
        dtoa_max_digits_table[radix as usize - 2] as i32 + 4
    };
    if flags & JS_DTOA_EXP_MASK == JS_DTOA_EXP_ENABLED
        || (flags & JS_DTOA_EXP_MASK == JS_DTOA_EXP_AUTO && (E <= -6 || E > E_max))
    {
        q = q.add(output_digits(q, tmp1, radix, P, 1) as usize);
        E -= 1;
        if radix == 10 {
            emit(&mut q, b'e');
        } else if radix1 == 1 && radix_shift <= 4 {
            E *= radix_shift;
            emit(&mut q, b'p');
        } else {
            emit(&mut q, b'@');
        }
        if E < 0 {
            emit(&mut q, b'-');
            E = -E;
        } else {
            emit(&mut q, b'+');
        }
        q = q.add(u32toa(q, E as u32));
    } else if E <= 0 {
        emit(&mut q, b'0');
        emit(&mut q, b'.');
        for _ in 0..-E {
            emit(&mut q, b'0');
        }
        q = q.add(output_digits(q, tmp1, radix, P, P) as usize);
    } else {
        q = q.add(output_digits(q, tmp1, radix, P, min_int(P, E)) as usize);
        for _ in 0..E - P {
            emit(&mut q, b'0');
        }
    }
    dtoa_finish(buf, q, tmp1, mant_max)
}
fn to_digit(c: i32) -> i32 {
    if c >= b'0' as i32 && c <= b'9' as i32 {
        c - b'0' as i32
    } else if c >= b'A' as i32 && c <= b'Z' as i32 {
        c - b'A' as i32 + 10
    } else if c >= b'a' as i32 && c <= b'z' as i32 {
        c - b'a' as i32 + 10
    } else {
        36
    }
}
unsafe fn mpb_mul1_base(r: *mut mpb_t, radix_base: limb_t, a: limb_t) {
    if *ctab(r) == 0 && (*r).len == 1 {
        *tab(r) = a;
    } else {
        if radix_base == 0 {
            for i in (0..=(*r).len as usize).rev() {
                *tab(r).add(i + 1) = *ctab(r).add(i);
            }
            *tab(r) = a;
        } else {
            let carry = mp_mul1(tab(r), ctab(r), (*r).len as limb_t, radix_base, a);
            *tab(r).add((*r).len as usize) = carry;
        }
        (*r).len += 1;
        mpb_renorm(r);
    }
}
pub unsafe fn js_atod(
    str: *const c_char,
    pnext: *mut *const c_char,
    mut radix: i32,
    flags: i32,
    tmp_mem: *mut JSATODTempMem,
) -> f64 {
    let mut mptr = (*tmp_mem).mem.as_mut_ptr();
    let tmp0 = dtoa_malloc(
        &mut mptr,
        core::mem::size_of::<mpb_t>() + core::mem::size_of::<limb_t>() * DBIGNUM_LEN_MAX,
    );
    assert!(
        mptr.offset_from((*tmp_mem).mem.as_ptr()) as usize
            <= core::mem::size_of::<JSATODTempMem>() / 8
    );
    let mut sep = if flags & JS_ATOD_ACCEPT_UNDERSCORES != 0 {
        b'_' as i32
    } else {
        256
    };
    let mut p = str;
    let mut is_neg = 0;
    let result = (|| -> Result<u64, ()> {
        if *p == b'+' as c_char {
            p = p.add(1);
        } else if *p == b'-' as c_char {
            is_neg = 1;
            p = p.add(1);
        }
        let p_start = p;
        if *p == b'0' as c_char {
            let mut has_prefix = false;
            let next = *p.add(1) as u8;
            if (next == b'x' || next == b'X') && (radix == 0 || radix == 16) {
                p = p.add(2);
                radix = 16;
                has_prefix = true;
            } else if (next == b'o' || next == b'O')
                && radix == 0
                && flags & JS_ATOD_ACCEPT_BIN_OCT != 0
            {
                p = p.add(2);
                radix = 8;
                has_prefix = true;
            } else if (next == b'b' || next == b'B')
                && radix == 0
                && flags & JS_ATOD_ACCEPT_BIN_OCT != 0
            {
                p = p.add(2);
                radix = 2;
                has_prefix = true;
            } else if next >= b'0'
                && next <= b'9'
                && radix == 0
                && flags & JS_ATOD_ACCEPT_LEGACY_OCTAL != 0
            {
                sep = 256;
                let mut i = 1;
                while *p.add(i) >= b'0' as c_char && *p.add(i) <= b'7' as c_char {
                    i += 1;
                }
                if *p.add(i) != b'8' as c_char && *p.add(i) != b'9' as c_char {
                    p = p.add(1);
                    radix = 8;
                    has_prefix = true;
                }
            }
            if has_prefix && to_digit(*p as u8 as i32) >= radix {
                return Err(());
            }
        } else if flags & JS_ATOD_INT_ONLY == 0 && strstart(p, c"Infinity".as_ptr(), &mut p) != 0 {
            return Ok(0x7ffu64 << 52);
        }
        if radix == 0 {
            radix = 10;
        }
        let (mut cur_limb, mut digit_count, mut limb_digit_count) = (0u32, 0i32, 0i32);
        let max_digits = atod_max_digits_table[radix as usize - 2] as i32;
        let digits_per_limb = digits_per_limb_table[radix as usize - 2] as i32;
        let radix_base = radix_base_table[radix as usize - 2];
        let radix_shift = ctz32(radix as u32);
        let radix1 = radix >> radix_shift;
        let radix_bits = if radix1 == 1 { radix_shift } else { 0 };
        (*tmp0).len = 1;
        *tab(tmp0) = 0;
        let (mut extra_digits, mut pos, mut dot_pos) = (0u32, 0i32, -1i32);
        loop {
            if *p == b'.' as c_char
                && (p > p_start || to_digit(*p.add(1) as i32) < radix)
                && flags & JS_ATOD_INT_ONLY == 0
            {
                if *p as i32 == sep {
                    return Err(());
                }
                if dot_pos >= 0 {
                    break;
                }
                dot_pos = pos;
                p = p.add(1);
            }
            if *p as i32 == sep && p > p_start && *p.add(1) == b'0' as c_char {
                p = p.add(1);
            }
            if *p != b'0' as c_char {
                break;
            }
            p = p.add(1);
            pos += 1;
        }
        let sig_pos = pos;
        loop {
            if *p == b'.' as c_char
                && (p > p_start || to_digit(*p.add(1) as i32) < radix)
                && flags & JS_ATOD_INT_ONLY == 0
            {
                if *p as i32 == sep {
                    return Err(());
                }
                if dot_pos >= 0 {
                    break;
                }
                dot_pos = pos;
                p = p.add(1);
            }
            if *p as i32 == sep && p > p_start && to_digit(*p.add(1) as i32) < radix {
                p = p.add(1);
            }
            let c = to_digit(*p as i32) as u32;
            if c >= radix as u32 {
                break;
            }
            p = p.add(1);
            pos += 1;
            if digit_count < max_digits {
                cur_limb = cur_limb.wrapping_mul(radix as u32).wrapping_add(c);
                limb_digit_count += 1;
                if limb_digit_count == digits_per_limb {
                    mpb_mul1_base(tmp0, radix_base, cur_limb);
                    cur_limb = 0;
                    limb_digit_count = 0;
                }
                digit_count += 1;
            } else {
                extra_digits |= c;
            }
        }
        if limb_digit_count != 0 {
            mpb_mul1_base(
                tmp0,
                pow_ui(radix as u32, limb_digit_count as u32) as limb_t,
                cur_limb,
            );
        }
        let is_zero = digit_count == 0;
        let expn_offset = if is_zero {
            0
        } else {
            if dot_pos < 0 {
                dot_pos = pos;
            }
            sig_pos + digit_count - dot_pos
        };
        if radix_bits != 0 && extra_digits != 0 {
            *tab(tmp0) |= 1;
        }
        let (mut expn, mut expn_overflow, mut is_bin_exp) = (0i32, false, false);
        if flags & JS_ATOD_INT_ONLY == 0
            && ((radix == 10 && (*p == b'e' as c_char || *p == b'E' as c_char))
                || (radix != 10
                    && (*p == b'@' as c_char
                        || (radix_bits >= 1
                            && radix_bits <= 4
                            && (*p == b'p' as c_char || *p == b'P' as c_char)))))
            && p > p_start
        {
            is_bin_exp = *p == b'p' as c_char || *p == b'P' as c_char;
            p = p.add(1);
            let mut exp_is_neg = false;
            if *p == b'+' as c_char {
                p = p.add(1);
            } else if *p == b'-' as c_char {
                exp_is_neg = true;
                p = p.add(1);
            }
            let c = to_digit(*p as i32);
            if c >= 10 {
                return Err(());
            }
            expn = c;
            p = p.add(1);
            loop {
                if *p as i32 == sep && to_digit(*p.add(1) as i32) < 10 {
                    p = p.add(1);
                }
                let c = to_digit(*p as i32);
                if c >= 10 {
                    break;
                }
                if !expn_overflow {
                    if expn > (i32::MAX - 2 - 9) / 10 {
                        expn_overflow = true;
                    } else {
                        expn = expn * 10 + c;
                    }
                }
                p = p.add(1);
            }
            if exp_is_neg {
                expn = -expn;
            }
            if !is_zero && expn_overflow {
                return Ok(if exp_is_neg { 0 } else { 0x7ffu64 << 52 });
            }
        }
        if p == p_start {
            return Err(());
        }
        if is_zero {
            return Ok(0);
        }
        let mut e = 0;
        let m;
        if radix_bits != 0 {
            if !is_bin_exp {
                expn = expn.wrapping_mul(radix_bits);
            }
            expn = expn.wrapping_sub(expn_offset.wrapping_mul(radix_bits));
            let expn1 = expn.wrapping_add(digit_count * radix_bits);
            if expn1 >= 1024 + radix_bits {
                return Ok(0x7ffu64 << 52);
            }
            if expn1 <= -1075 {
                return Ok(0);
            }
            m = round_to_d(&mut e, tmp0, -expn, JS_RNDN);
        } else {
            expn = expn.wrapping_sub(expn_offset);
            let expn1 = expn.wrapping_add(digit_count);
            if expn1 >= max_exponent[radix as usize - 2] as i32 + 1 {
                return Ok(0x7ffu64 << 52);
            }
            if expn1 <= min_exponent[radix as usize - 2] as i32 {
                return Ok(0);
            }
            m = mul_pow_round_to_d(&mut e, tmp0, radix1, radix_shift, expn, JS_RNDN);
        }
        Ok(if m == 0 {
            0
        } else if e > 1024 {
            0x7ffu64 << 52
        } else if e < -1073 {
            0
        } else if e < -1021 {
            m >> (-e - 1021)
        } else {
            ((e + 1022) as u64) << 52 | (m & ((1u64 << 52) - 1))
        })
    })();
    let dval = match result {
        Ok(a) => uint64_as_float64(a | ((is_neg as u64) << 63)),
        Err(()) => f64::NAN,
    };
    if !pnext.is_null() {
        *pnext = p;
    }
    dtoa_free(tmp0);
    dval
}
#[cfg(test)]
#[path = "../tests/dtoa_differential.rs"]
mod tests;
