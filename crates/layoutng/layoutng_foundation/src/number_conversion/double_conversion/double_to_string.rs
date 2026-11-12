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
// Source: double-to-string.cc:42-202,379-438 (SHORTEST only).
// EcmaScriptConverter: UNIQUE_ZERO | EMIT_POSITIVE_EXPONENT_SIGN,
// "Infinity", "NaN", 'e', decimal exponent range [-6,21), exponent width 0.
use super::{bignum_dtoa::BignumDtoa, fast_dtoa::FastDtoa, ieee::Double};
pub(super) fn EcmaScriptToShortest(value: f64) -> String {
    let d = Double::new(value);
    if d.IsSpecial() {
        return if d.IsNan() {
            "NaN".into()
        } else if d.Sign() < 0 {
            "-Infinity".into()
        } else {
            "Infinity".into()
        };
    }
    let (digits, point, negative) = DoubleToAscii(value);
    let mut result = Vec::with_capacity(26);
    if negative && value != 0.0 {
        result.push(b'-');
    }
    let exponent = point - 1;
    if (-6..21).contains(&exponent) {
        CreateDecimalRepresentation(&digits, point, &mut result);
    } else {
        CreateExponentialRepresentation(&digits, exponent, &mut result);
    }
    // Every byte is emitted by the ASCII digit/sign/separator routines above.
    String::from_utf8(result).expect("double-conversion emits ASCII")
}
fn DoubleToAscii(mut v: f64) -> (Vec<u8>, i32, bool) {
    let negative = Double::new(v).Sign() < 0;
    if negative {
        v = -v;
    }
    if v == 0.0 {
        return (vec![b'0'], 1, negative);
    }
    let mut buffer = Vec::with_capacity(18);
    let point = match FastDtoa(v, &mut buffer) {
        Some(point) => point,
        None => BignumDtoa(v, &mut buffer),
    };
    (buffer, point, negative)
}
// Source: double-to-string.cc:75-119, min_exponent_width == 0.
fn CreateExponentialRepresentation(digits: &[u8], mut exponent: i32, out: &mut Vec<u8>) {
    out.push(digits[0]);
    if digits.len() != 1 {
        out.push(b'.');
        out.extend_from_slice(&digits[1..]);
    }
    out.push(b'e');
    if exponent < 0 {
        out.push(b'-');
        exponent = -exponent;
    } else {
        out.push(b'+');
    }
    debug_assert!(exponent < 10000);
    let mut buffer = [0; 5];
    let mut first = 5;
    if exponent == 0 {
        first -= 1;
        buffer[first] = b'0';
    } else {
        while exponent > 0 {
            first -= 1;
            buffer[first] = b'0' + (exponent % 10) as u8;
            exponent /= 10;
        }
    }
    out.extend_from_slice(&buffer[first..]);
}
// Source: double-to-string.cc:122-164. For shortest, digits_after_point is
// max(0, length - point), so the source's extra fractional padding is empty.
fn CreateDecimalRepresentation(digits: &[u8], point: i32, out: &mut Vec<u8>) {
    if point <= 0 {
        out.extend_from_slice(b"0.");
        out.resize(out.len() + (-point) as usize, b'0');
        out.extend_from_slice(digits);
    } else if point as usize >= digits.len() {
        out.extend_from_slice(digits);
        out.resize(out.len() + point as usize - digits.len(), b'0');
    } else {
        out.extend_from_slice(&digits[..point as usize]);
        out.push(b'.');
        out.extend_from_slice(&digits[point as usize..]);
    }
}

// Source: double-to-string.cc:298-365,379-438, EcmaScript flags/config:42-52.
pub(super) fn EcmaScriptToPrecision(value: f64, precision: usize) -> Option<String> {
    if Double::new(value).IsSpecial() {
        return Some(EcmaScriptToShortest(value));
    }
    if !(1..=120).contains(&precision) {
        return None;
    }
    let negative = Double::new(value).Sign() < 0;
    let absolute = value.abs();
    let mut digits = Vec::with_capacity(precision);
    let point = if absolute == 0.0 {
        digits.push(b'0');
        1
    } else {
        match super::fast_dtoa::FastDtoaPrecision(absolute, precision, &mut digits) {
            Some(point) => point,
            None => super::bignum_dtoa::BignumDtoaPrecision(absolute, precision, &mut digits),
        }
    };
    let mut result = Vec::with_capacity(precision + 16);
    if negative && value != 0.0 {
        result.push(b'-');
    }
    digits.resize(precision, b'0');
    if -point + 1 > 6 || point - precision as i32 > 0 {
        CreateExponentialRepresentation(&digits, point - 1, &mut result);
    } else {
        CreateDecimalRepresentation(&digits, point, &mut result);
    }
    Some(String::from_utf8(result).expect("source converter emits ASCII"))
}
