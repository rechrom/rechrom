// Original quickjs.c Math accumulator layout and host/stdlib boundaries. MIT.
use super::*;
pub(crate) type SumPreciseStateEnum = u32;
pub(crate) const SUM_PRECISE_STATE_FINITE: u32 = 0;
pub(crate) const SUM_PRECISE_STATE_INFINITY: u32 = 1;
pub(crate) const SUM_PRECISE_STATE_MINUS_INFINITY: u32 = 2;
pub(crate) const SUM_PRECISE_STATE_NAN: u32 = 3;
#[repr(C)]
pub(crate) struct SumPreciseState { pub(crate) state: SumPreciseStateEnum, pub(crate) counter: u32, pub(crate) n_limbs: i32, pub(crate) acc: [i64; 39] }
// The source seeds xorshift from gettimeofday's signed microsecond timestamp.
// SystemTime provides the same host wall-clock boundary through Rust std.
pub(crate) unsafe fn js_random_init(ctx: *mut JSContext) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_micros() as u64,
        Err(e) => 0u64.wrapping_sub(((e.duration().as_nanos() + 999) / 1000) as u64),
    };
    (*ctx).random_state = if now == 0 { 1 } else { now };
}
pub(crate) unsafe fn js_math_f64_fabs(a: f64) -> f64 { a.abs() }
pub(crate) unsafe fn js_math_f64_floor(a: f64) -> f64 { a.floor() }
pub(crate) unsafe fn js_math_f64_ceil(a: f64) -> f64 { a.ceil() }
pub(crate) unsafe fn js_math_f64_sqrt(a: f64) -> f64 { a.sqrt() }
pub(crate) unsafe fn js_math_f64_acos(a: f64) -> f64 { a.acos() }
pub(crate) unsafe fn js_math_f64_asin(a: f64) -> f64 { a.asin() }
pub(crate) unsafe fn js_math_f64_atan(a: f64) -> f64 { a.atan() }
pub(crate) unsafe fn js_math_f64_atan2(a: f64, b: f64) -> f64 { a.atan2(b) }
pub(crate) unsafe fn js_math_f64_cos(a: f64) -> f64 { a.cos() }
pub(crate) unsafe fn js_math_f64_exp(a: f64) -> f64 { a.exp() }
pub(crate) unsafe fn js_math_f64_log(a: f64) -> f64 { a.ln() }
pub(crate) unsafe fn js_math_f64_sin(a: f64) -> f64 { a.sin() }
pub(crate) unsafe fn js_math_f64_tan(a: f64) -> f64 { a.tan() }
pub(crate) unsafe fn js_math_f64_trunc(a: f64) -> f64 { a.trunc() }
pub(crate) unsafe fn js_math_f64_cosh(a: f64) -> f64 { a.cosh() }
pub(crate) unsafe fn js_math_f64_sinh(a: f64) -> f64 { a.sinh() }
pub(crate) unsafe fn js_math_f64_tanh(a: f64) -> f64 { a.tanh() }
pub(crate) unsafe fn js_math_f64_acosh(a: f64) -> f64 { a.acosh() }
pub(crate) unsafe fn js_math_f64_asinh(a: f64) -> f64 { a.asinh() }
// fdlibm atanh algorithm used by Apple's Libm (Source/ARM/atanh_freeBSD.c).
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
// Permission to use, copy, modify, and distribute this software is freely
// granted, provided that this notice is preserved.
pub(crate) unsafe fn js_math_f64_atanh(a: f64) -> f64 {
    let bits = a.to_bits();
    let magnitude = bits & 0x7fff_ffff_ffff_ffff;
    if magnitude > 0x3ff0_0000_0000_0000 { return (a - a) / (a - a); }
    if magnitude == 0x3ff0_0000_0000_0000 { return a / 0.0; }
    if magnitude < 0x3e30_0000_0000_0000 { return a; }
    let x = f64::from_bits(magnitude);
    let t = if magnitude < 0x3fe0_0000_0000_0000 {
        let twice = x + x;
        0.5 * (twice + twice * x / (1.0 - x)).ln_1p()
    } else { 0.5 * ((x + x) / (1.0 - x)).ln_1p() };
    if bits >> 63 == 0 { t } else { -t }
}
pub(crate) unsafe fn js_math_f64_expm1(a: f64) -> f64 { a.exp_m1() }
pub(crate) unsafe fn js_math_f64_log1p(a: f64) -> f64 { a.ln_1p() }
pub(crate) unsafe fn js_math_f64_log2(a: f64) -> f64 { a.log2() }
pub(crate) unsafe fn js_math_f64_log10(a: f64) -> f64 { a.log10() }
pub(crate) unsafe fn js_math_f64_cbrt(a: f64) -> f64 { a.cbrt() }
