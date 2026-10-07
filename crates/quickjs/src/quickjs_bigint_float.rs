// quickjs.c:12291-12339. Exact IEEE-754 integer decomposition. MIT.
// pres: 0 = integer/allocation failure, 1 = fractional, 2 = NaN/infinity.
unsafe fn js_bigint_from_float64(
    ctx: *mut JSContext, pres: *mut i32, value: f64,
) -> *mut JSBigInt {
    let bits = value.to_bits();
    let negative = (bits >> 63) as i32;
    let mut exponent = ((bits >> 52) & ((1 << 11) - 1)) as i32;
    let mut mantissa = bits & ((1u64 << 52) - 1);
    if exponent == 2047 {
        *pres = 2;
        return ptr::null_mut();
    }
    if exponent == 0 && mantissa == 0 {
        *pres = 0;
        return js_bigint_new_si(ctx, 0);
    }
    exponent -= 1023;
    if exponent < 0 {
        *pres = 1;
        return ptr::null_mut();
    }
    mantissa |= 1u64 << 52;
    if exponent < 52 {
        let shift = 52 - exponent;
        if mantissa & ((1u64 << shift) - 1) != 0 {
            *pres = 1;
            return ptr::null_mut();
        }
        mantissa >>= shift;
        exponent = 0;
    } else {
        exponent -= 52;
    }
    if negative != 0 {
        mantissa = mantissa.wrapping_neg();
    }
    let mut buf: JSBigIntBuf = core::mem::zeroed();
    let integer = js_bigint_set_si64(&mut buf, mantissa as i64);
    *pres = 0;
    js_bigint_shl(ctx, integer, exponent as u32)
}
