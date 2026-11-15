// Copyright 2012 the V8 project authors. All rights reserved.
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
// Source: fast-dtoa.cc:40-178,251-432,534-601,637-662.
// FAST_DTOA_SHORTEST branch (Grisu3), retaining the source's weed/fallback proof.
use super::{cached_powers, diy_fp::DiyFp, ieee::Double};

fn RoundWeed(
    buffer: &mut [u8],
    distance: u64,
    unsafe_interval: u64,
    mut rest: u64,
    ten_kappa: u64,
    unit: u64,
) -> bool {
    let small_distance = distance - unit;
    let big_distance = distance + unit;
    debug_assert!(rest <= unsafe_interval);
    while rest < small_distance
        && unsafe_interval - rest >= ten_kappa
        && (rest + ten_kappa < small_distance
            || small_distance - rest >= rest + ten_kappa - small_distance)
    {
        *buffer.last_mut().unwrap() -= 1;
        rest += ten_kappa;
    }
    if rest < big_distance
        && unsafe_interval - rest >= ten_kappa
        && (rest + ten_kappa < big_distance
            || big_distance - rest > rest + ten_kappa - big_distance)
    {
        return false;
    }
    2 * unit <= rest && rest <= unsafe_interval - 4 * unit
}
fn BiggestPowerTen(number: u32, bits: i32) -> (u32, i32) {
    const kSmallPowers: [u32; 11] = [
        0, 1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
    ];
    let mut guess = ((bits + 1) * 1233 >> 12) + 1;
    if number < kSmallPowers[guess as usize] {
        guess -= 1;
    }
    (kSmallPowers[guess as usize], guess)
}
fn DigitGen(low: DiyFp, w: DiyFp, high: DiyFp, buffer: &mut Vec<u8>) -> Option<i32> {
    debug_assert!(low.e == w.e && w.e == high.e);
    debug_assert!(low.f + 1 <= high.f - 1);
    debug_assert!((-60..=-32).contains(&w.e));
    let mut unit = 1u64;
    let too_low = DiyFp::new(low.f - unit, low.e);
    let too_high = DiyFp::new(high.f + unit, high.e);
    let mut unsafe_interval = DiyFp::Minus(too_high, too_low);
    let one = DiyFp::new(1u64 << -w.e, w.e);
    let mut integrals = (too_high.f >> -one.e) as u32;
    let mut fractionals = too_high.f & (one.f - 1);
    let (mut divisor, mut kappa) = BiggestPowerTen(integrals, 64 + one.e);
    buffer.clear();
    while kappa > 0 {
        let digit = integrals / divisor;
        debug_assert!(digit <= 9);
        buffer.push(b'0' + digit as u8);
        integrals %= divisor;
        kappa -= 1;
        let rest = ((integrals as u64) << -one.e) + fractionals;
        if rest < unsafe_interval.f {
            return RoundWeed(
                buffer,
                DiyFp::Minus(too_high, w).f,
                unsafe_interval.f,
                rest,
                (divisor as u64) << -one.e,
                unit,
            )
            .then_some(kappa);
        }
        divisor /= 10;
    }
    debug_assert!(one.e >= -60 && fractionals < one.f);
    loop {
        fractionals *= 10;
        unit *= 10;
        unsafe_interval.f *= 10;
        let digit = fractionals >> -one.e;
        debug_assert!(digit <= 9);
        buffer.push(b'0' + digit as u8);
        fractionals &= one.f - 1;
        kappa -= 1;
        if fractionals < unsafe_interval.f {
            return RoundWeed(
                buffer,
                DiyFp::Minus(too_high, w).f * unit,
                unsafe_interval.f,
                fractionals,
                one.f,
                unit,
            )
            .then_some(kappa);
        }
    }
}
pub(super) fn FastDtoa(v: f64, buffer: &mut Vec<u8>) -> Option<i32> {
    debug_assert!(v > 0.0 && !Double::new(v).IsSpecial());
    let d = Double::new(v);
    let w = d.AsNormalizedDiyFp();
    let (minus, plus) = d.NormalizedBoundaries();
    debug_assert_eq!(plus.e, w.e);
    let (ten_mk, mk) =
        cached_powers::GetCachedPowerForBinaryExponentRange(-60 - (w.e + 64), -32 - (w.e + 64));
    let scaled_w = DiyFp::Times(w, ten_mk);
    let scaled_minus = DiyFp::Times(minus, ten_mk);
    let scaled_plus = DiyFp::Times(plus, ten_mk);
    let kappa = DigitGen(scaled_minus, scaled_w, scaled_plus, buffer)?;
    Some(buffer.len() as i32 - mk + kappa)
}

// Source: fast-dtoa.cc:180-225. Counted Grisu retains its proof of rounding;
// uncertainty returns None and uses the source bignum precision fallback.
fn RoundWeedCounted(
    buffer: &mut [u8],
    rest: u64,
    ten_kappa: u64,
    unit: u64,
    kappa: &mut i32,
) -> bool {
    debug_assert!(rest < ten_kappa);
    if unit >= ten_kappa || ten_kappa - unit <= unit {
        return false;
    }
    if ten_kappa - rest > rest && ten_kappa - 2 * rest >= 2 * unit {
        return true;
    }
    if rest > unit && ten_kappa - (rest - unit) <= rest - unit {
        let last = buffer.len() - 1;
        buffer[last] += 1;
        for i in (1..buffer.len()).rev() {
            if buffer[i] != b'0' + 10 {
                break;
            }
            buffer[i] = b'0';
            buffer[i - 1] += 1;
        }
        if buffer[0] == b'0' + 10 {
            buffer[0] = b'1';
            *kappa += 1;
        }
        return true;
    }
    false
}
// Source: fast-dtoa.cc:434-512,603-634,637-662 (PRECISION).
pub(super) fn FastDtoaPrecision(v: f64, mut requested: usize, buffer: &mut Vec<u8>) -> Option<i32> {
    debug_assert!(v > 0.0 && !Double::new(v).IsSpecial() && requested > 0);
    let w = Double::new(v).AsNormalizedDiyFp();
    let (power, mk) =
        cached_powers::GetCachedPowerForBinaryExponentRange(-60 - (w.e + 64), -32 - (w.e + 64));
    let scaled = DiyFp::Times(w, power);
    debug_assert!((-60..=-32).contains(&scaled.e));
    let one = DiyFp {
        f: 1u64 << -scaled.e,
        e: scaled.e,
    };
    let mut error = 1u64;
    let mut integrals = (scaled.f >> -one.e) as u32;
    let mut fractionals = scaled.f & (one.f - 1);
    let (mut divisor, mut kappa) = BiggestPowerTen(integrals, 64 + one.e);
    buffer.clear();
    while kappa > 0 {
        let digit = integrals / divisor;
        debug_assert!(digit <= 9);
        buffer.push(b'0' + digit as u8);
        requested -= 1;
        integrals %= divisor;
        kappa -= 1;
        if requested == 0 {
            break;
        }
        divisor /= 10;
    }
    if requested == 0 {
        let rest = ((integrals as u64) << -one.e) + fractionals;
        return RoundWeedCounted(buffer, rest, (divisor as u64) << -one.e, error, &mut kappa)
            .then_some(buffer.len() as i32 - mk + kappa);
    }
    debug_assert!(one.f <= u64::MAX / 10);
    while requested > 0 && fractionals > error {
        fractionals *= 10;
        error *= 10;
        let digit = (fractionals >> -one.e) as u8;
        debug_assert!(digit <= 9);
        buffer.push(b'0' + digit);
        requested -= 1;
        fractionals &= one.f - 1;
        kappa -= 1;
    }
    if requested != 0 {
        return None;
    }
    RoundWeedCounted(buffer, fractionals, one.f, error, &mut kappa)
        .then_some(buffer.len() as i32 - mk + kappa)
}
