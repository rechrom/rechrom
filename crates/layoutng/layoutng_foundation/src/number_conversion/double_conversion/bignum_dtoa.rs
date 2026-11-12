// Copyright 2010 the V8 project authors. All rights reserved.
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
//       notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
//       copyright notice, this list of conditions and the following
//       disclaimer in the documentation and/or other materials provided
//       with the distribution.
//     * Neither the name of Google Inc. nor the names of its
//       contributors may be used to endorse or promote products derived
//       from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: bignum-dtoa.cc:39-45,93-184,205-293,364-641.
// BIGNUM_DTOA_SHORTEST and PRECISION share the original scaled-start owner.
use super::{bignum::Bignum, ieee::Double};

pub(super) fn BignumDtoa(v: f64, buffer: &mut Vec<u8>) -> i32 {
    debug_assert!(v > 0.0 && !Double::new(v).IsSpecial());
    let d = Double::new(v);
    let significand = d.Significand();
    let exponent = d.Exponent();
    let is_even = significand & 1 == 0;
    let normalized = NormalizedExponent(significand, exponent);
    let estimated_power = EstimatePower(normalized);
    let mut numerator = Bignum::new();
    let mut denominator = Bignum::new();
    let mut delta_minus = Bignum::new();
    let mut delta_plus = Bignum::new();
    InitialScaledStartValues(
        significand,
        exponent,
        d.LowerBoundaryIsCloser(),
        estimated_power,
        true,
        &mut numerator,
        &mut denominator,
        &mut delta_minus,
        &mut delta_plus,
    );
    let decimal_point = FixupMultiply10(
        estimated_power,
        is_even,
        &mut numerator,
        &denominator,
        &mut delta_minus,
        &mut delta_plus,
    );
    GenerateShortestDigits(
        &mut numerator,
        &denominator,
        &mut delta_minus,
        &mut delta_plus,
        is_even,
        buffer,
    );
    decimal_point
}
fn NormalizedExponent(mut significand: u64, mut exponent: i32) -> i32 {
    debug_assert_ne!(significand, 0);
    while significand & Double::kHiddenBit == 0 {
        significand <<= 1;
        exponent -= 1;
    }
    exponent
}
// Source: bignum-dtoa.cc:364-413. This estimate is intentionally the source's
// bounded log estimate, followed by exact bigit arithmetic and range fixup.
fn EstimatePower(exponent: i32) -> i32 {
    ((exponent + Double::kSignificandSize - 1) as f64 * 0.30102999566398114 - 1e-10).ceil() as i32
}
fn InitialScaledStartValues(
    significand: u64,
    exponent: i32,
    lower_closer: bool,
    power: i32,
    need_boundary_deltas: bool,
    numerator: &mut Bignum,
    denominator: &mut Bignum,
    minus: &mut Bignum,
    plus: &mut Bignum,
) {
    // Source: bignum-dtoa.cc:417-450.
    if exponent >= 0 {
        debug_assert!(power >= 0);
        numerator.AssignUInt64(significand);
        numerator.ShiftLeft(exponent);
        denominator.AssignPowerUInt16(10, power);
        if need_boundary_deltas {
            denominator.ShiftLeft(1);
            numerator.ShiftLeft(1);
            plus.AssignUInt16(1);
            plus.ShiftLeft(exponent);
            minus.AssignUInt16(1);
            minus.ShiftLeft(exponent);
        }
    // Source: bignum-dtoa.cc:454-485.
    } else if power >= 0 {
        numerator.AssignUInt64(significand);
        denominator.AssignPowerUInt16(10, power);
        denominator.ShiftLeft(-exponent);
        if need_boundary_deltas {
            denominator.ShiftLeft(1);
            numerator.ShiftLeft(1);
            plus.AssignUInt16(1);
            minus.AssignUInt16(1);
        }
    // Source: bignum-dtoa.cc:489-534.
    } else {
        numerator.AssignPowerUInt16(10, -power);
        if need_boundary_deltas {
            plus.AssignBignum(numerator);
            minus.AssignBignum(numerator);
        }
        numerator.MultiplyByUInt64(significand);
        denominator.AssignUInt16(1);
        denominator.ShiftLeft(-exponent);
        if need_boundary_deltas {
            numerator.ShiftLeft(1);
            denominator.ShiftLeft(1);
        }
    }
    // Source: bignum-dtoa.cc:575-601.
    if need_boundary_deltas && lower_closer {
        denominator.ShiftLeft(1);
        numerator.ShiftLeft(1);
        plus.ShiftLeft(1);
    }
}
// Source: bignum-dtoa.cc:618-641.
fn FixupMultiply10(
    power: i32,
    even: bool,
    numerator: &mut Bignum,
    denominator: &Bignum,
    minus: &mut Bignum,
    plus: &mut Bignum,
) -> i32 {
    let cmp = Bignum::PlusCompare(numerator, plus, denominator);
    if if even { cmp >= 0 } else { cmp > 0 } {
        return power + 1;
    }
    numerator.Times10();
    if Bignum::Compare(minus, plus) == 0 {
        minus.Times10();
        plus.AssignBignum(minus);
    } else {
        minus.Times10();
        plus.Times10();
    }
    power
}
// Source: bignum-dtoa.cc:205-293. The source aliases equal deltas; `same_delta`
// preserves that optimization without creating aliased mutable references.
fn GenerateShortestDigits(
    numerator: &mut Bignum,
    denominator: &Bignum,
    minus: &mut Bignum,
    plus: &mut Bignum,
    even: bool,
    buffer: &mut Vec<u8>,
) {
    let same_delta = Bignum::Compare(minus, plus) == 0;
    buffer.clear();
    loop {
        let digit = numerator.DivideModuloIntBignum(denominator);
        debug_assert!(digit <= 9);
        buffer.push(b'0' + digit as u8);
        let minus_cmp = Bignum::Compare(numerator, minus);
        let plus_cmp = Bignum::PlusCompare(
            numerator,
            if same_delta { minus } else { plus },
            denominator,
        );
        let in_minus = if even { minus_cmp <= 0 } else { minus_cmp < 0 };
        let in_plus = if even { plus_cmp >= 0 } else { plus_cmp > 0 };
        if !in_minus && !in_plus {
            numerator.Times10();
            minus.Times10();
            if !same_delta {
                plus.Times10();
            }
        } else if in_minus && in_plus {
            let compare = Bignum::PlusCompare(numerator, numerator, denominator);
            let last = buffer.last_mut().unwrap();
            if compare > 0 || (compare == 0 && (*last - b'0') % 2 != 0) {
                debug_assert_ne!(*last, b'9');
                *last += 1;
            }
            return;
        } else if in_minus {
            return;
        } else {
            let last = buffer.last_mut().unwrap();
            debug_assert_ne!(*last, b'9');
            *last += 1;
            return;
        }
    }
}

// Source: bignum-dtoa.cc:93-184 PRECISION; GenerateCountedDigits:295-330.
pub(super) fn BignumDtoaPrecision(v: f64, count: usize, buffer: &mut Vec<u8>) -> i32 {
    debug_assert!(v > 0.0 && !Double::new(v).IsSpecial() && count > 0);
    let d = Double::new(v);
    let significand = d.Significand();
    let exponent = d.Exponent();
    let power = EstimatePower(NormalizedExponent(significand, exponent));
    let mut numerator = Bignum::new();
    let mut denominator = Bignum::new();
    let mut minus = Bignum::new();
    let mut plus = Bignum::new();
    InitialScaledStartValues(
        significand,
        exponent,
        d.LowerBoundaryIsCloser(),
        power,
        false,
        &mut numerator,
        &mut denominator,
        &mut minus,
        &mut plus,
    );
    let mut point = FixupMultiply10(
        power,
        significand & 1 == 0,
        &mut numerator,
        &denominator,
        &mut minus,
        &mut plus,
    );
    buffer.clear();
    for _ in 0..count - 1 {
        let digit = numerator.DivideModuloIntBignum(&denominator);
        debug_assert!(digit <= 9);
        buffer.push(b'0' + digit as u8);
        numerator.Times10();
    }
    let mut digit = numerator.DivideModuloIntBignum(&denominator);
    if Bignum::PlusCompare(&numerator, &numerator, &denominator) >= 0 {
        digit += 1;
    }
    debug_assert!(digit <= 10);
    buffer.push(b'0' + digit as u8);
    for i in (1..count).rev() {
        if buffer[i] != b'0' + 10 {
            break;
        }
        buffer[i] = b'0';
        buffer[i - 1] += 1;
    }
    if buffer[0] == b'0' + 10 {
        buffer[0] = b'1';
        point += 1;
    }
    point
}
