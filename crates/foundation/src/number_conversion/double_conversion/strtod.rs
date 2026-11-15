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
// Source: strtod.cc:39-97,157-499. Binary64 StrtodTrimmed and direct helpers.
// The source's exact short-number operations are followed by DiyFp error
// bounds and exact bignum boundary comparison; no host string parser is used.
use super::{bignum::Bignum, cached_powers, diy_fp::DiyFp, ieee::Double};
const kExactPowersOfTen: [f64; 23] = [
    1.0,
    10.0,
    100.0,
    1000.0,
    10000.0,
    100000.0,
    1000000.0,
    10000000.0,
    100000000.0,
    1000000000.0,
    10000000000.0,
    100000000000.0,
    1000000000000.0,
    10000000000000.0,
    100000000000000.0,
    1000000000000000.0,
    10000000000000000.0,
    100000000000000000.0,
    1000000000000000000.0,
    10000000000000000000.0,
    100000000000000000000.0,
    1000000000000000000000.0,
    10000000000000000000000.0,
];
fn ReadUint64(buffer: &[u8]) -> (u64, usize) {
    let mut result = 0u64;
    let mut i = 0;
    while i < buffer.len() && result <= u64::MAX / 10 - 1 {
        debug_assert!(buffer[i].is_ascii_digit());
        result = 10 * result + (buffer[i] - b'0') as u64;
        i += 1;
    }
    (result, i)
}
fn ReadDiyFp(buffer: &[u8]) -> (DiyFp, i32) {
    let (mut significand, read) = ReadUint64(buffer);
    if buffer.len() != read && buffer[read] >= b'5' {
        significand += 1;
    }
    (DiyFp::new(significand, 0), (buffer.len() - read) as i32)
}
// Source: strtod.cc:202-250, DOUBLE_CONVERSION_CORRECT_DOUBLE_OPERATIONS.
// These are IEEE single-rounding operations on exactly representable operands,
// including 10^0..10^22; they are not an approximate decimal accumulation.
fn DoubleStrtod(trimmed: &[u8], exponent: i32) -> Option<f64> {
    if trimmed.len() <= 15 {
        if exponent < 0 && -exponent < 23 {
            return Some(ReadUint64(trimmed).0 as f64 / kExactPowersOfTen[(-exponent) as usize]);
        }
        if (0..23).contains(&exponent) {
            return Some(ReadUint64(trimmed).0 as f64 * kExactPowersOfTen[exponent as usize]);
        }
        let remaining = 15 - trimmed.len() as i32;
        if exponent >= 0 && exponent - remaining < 23 {
            let mut result = ReadUint64(trimmed).0 as f64;
            result *= kExactPowersOfTen[remaining as usize];
            result *= kExactPowersOfTen[(exponent - remaining) as usize];
            return Some(result);
        }
    }
    None
}
fn AdjustmentPowerOfTen(exponent: i32) -> DiyFp {
    let (f, e) = match exponent {
        1 => (0xa0000000_00000000, -60),
        2 => (0xc8000000_00000000, -57),
        3 => (0xfa000000_00000000, -54),
        4 => (0x9c400000_00000000, -50),
        5 => (0xc3500000_00000000, -47),
        6 => (0xf4240000_00000000, -44),
        7 => (0x98968000_00000000, -40),
        _ => unreachable!("cached-power adjustment is in 1..8"),
    };
    DiyFp::new(f, e)
}
// Source: strtod.cc:283-393. Return (guess, proven-correct); an unproven guess
// is guaranteed to be the answer or its immediate lower binary64 neighbor.
fn DiyFpStrtod(buffer: &[u8], mut exponent: i32) -> (f64, bool) {
    let (mut input, remaining) = ReadDiyFp(buffer);
    const kDenominatorLog: i32 = 3;
    const kDenominator: u64 = 1 << kDenominatorLog;
    exponent += remaining;
    let mut error = if remaining == 0 { 0 } else { kDenominator / 2 };
    let mut old_e = input.e;
    input.Normalize();
    error <<= old_e - input.e;
    debug_assert!(exponent <= 340);
    if exponent < -348 {
        return (0.0, true);
    }
    let (cached, cached_exponent) = cached_powers::GetCachedPowerForDecimalExponent(exponent);
    if cached_exponent != exponent {
        let adjustment = exponent - cached_exponent;
        input.Multiply(AdjustmentPowerOfTen(adjustment));
        if 19 - (buffer.len() as i32) < adjustment {
            error += kDenominator / 2;
        }
    }
    input.Multiply(cached);
    let error_b = kDenominator / 2;
    let error_ab = if error == 0 { 0 } else { 1 };
    let fixed_error = kDenominator / 2;
    error += error_b + error_ab + fixed_error;
    old_e = input.e;
    input.Normalize();
    error <<= old_e - input.e;
    let order = 64 + input.e;
    let effective_size = Double::SignificandSizeForOrderOfMagnitude(order);
    let mut precision_count = 64 - effective_size;
    if precision_count + kDenominatorLog >= 64 {
        let shift = precision_count + kDenominatorLog - 64 + 1;
        input.f >>= shift;
        input.e += shift;
        error = (error >> shift) + 1 + kDenominator;
        precision_count -= shift;
    }
    debug_assert!(precision_count < 64);
    let mask = (1u64 << precision_count) - 1;
    let mut precision_bits = input.f & mask;
    let mut half_way = 1u64 << (precision_count - 1);
    precision_bits *= kDenominator;
    half_way *= kDenominator;
    let mut rounded = DiyFp::new(input.f >> precision_count, input.e + precision_count);
    if precision_bits >= half_way + error {
        rounded.f += 1;
    }
    let result = Double::FromDiyFp(rounded).value();
    let proven =
        !(half_way.wrapping_sub(error) < precision_bits && precision_bits < half_way + error);
    (result, proven)
}
fn CompareBufferWithDiyFp(buffer: &[u8], exponent: i32, diy: DiyFp) -> i32 {
    debug_assert!(buffer.len() as i32 + exponent <= 310);
    debug_assert!(buffer.len() as i32 + exponent > -324);
    debug_assert!(buffer.len() <= 780);
    let mut decimal = Bignum::new();
    let mut binary = Bignum::new();
    decimal.AssignDecimalString(buffer);
    binary.AssignUInt64(diy.f);
    if exponent >= 0 {
        decimal.MultiplyByPowerOfTen(exponent);
    } else {
        binary.MultiplyByPowerOfTen(-exponent);
    }
    if diy.e > 0 {
        binary.ShiftLeft(diy.e);
    } else {
        decimal.ShiftLeft(-diy.e);
    }
    Bignum::Compare(&decimal, &binary)
}
fn ComputeGuess(trimmed: &[u8], exponent: i32) -> (f64, bool) {
    if trimmed.is_empty() {
        return (0.0, true);
    }
    if exponent + trimmed.len() as i32 - 1 >= 309 {
        return (Double::Infinity(), true);
    }
    if exponent + trimmed.len() as i32 <= -324 {
        return (0.0, true);
    }
    if let Some(result) = DoubleStrtod(trimmed, exponent) {
        return (result, true);
    }
    let (guess, correct) = DiyFpStrtod(trimmed, exponent);
    (guess, correct || guess == Double::Infinity())
}
pub(super) fn StrtodTrimmed(trimmed: &[u8], exponent: i32) -> f64 {
    debug_assert!(trimmed.len() <= 780);
    debug_assert!(trimmed.iter().all(u8::is_ascii_digit));
    debug_assert!(trimmed.is_empty() || (trimmed[0] != b'0' && *trimmed.last().unwrap() != b'0'));
    let (guess, correct) = ComputeGuess(trimmed, exponent);
    if correct {
        return guess;
    }
    let d = Double::new(guess);
    let comparison = CompareBufferWithDiyFp(trimmed, exponent, d.UpperBoundary());
    if comparison < 0 || (comparison == 0 && d.Significand() & 1 == 0) {
        guess
    } else {
        d.NextDouble()
    }
}
