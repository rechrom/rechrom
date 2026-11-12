/*
 * Copyright (C) 2012 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/wtf/decimal.h:47-179
// Source: third_party/blink/renderer/platform/wtf/decimal.cc:45-979
// Names and branches retain the source Decimal/EncodedData ownership. Integer
// arithmetic uses explicit wrapping where C++ unsigned arithmetic wraps.

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// Source: decimal.cc:47-52.
const kExponentMax: i32 = 1023;
const kExponentMin: i32 = -1023;
const kPrecision: i32 = 18;
const kMaxCoefficient: u64 = 0xDE0B6B3A763FFFF;

// Source: decimal.h:51-54,81-86.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    kPositive,
    kNegative,
}
impl Sign {
    fn from_negative(negative: bool) -> Self {
        if negative {
            Self::kNegative
        } else {
            Self::kPositive
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FormatClass {
    kClassInfinity,
    kClassNormal,
    kClassNaN,
    kClassZero,
}

// Source: decimal.h:56-95; decimal.cc:224-257.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncodedData {
    coefficient_: u64,
    exponent_: i16,
    format_class_: FormatClass,
    sign_: Sign,
}
impl EncodedData {
    fn special(sign: Sign, format_class: FormatClass) -> Self {
        Self {
            coefficient_: 0,
            exponent_: 0,
            format_class_: format_class,
            sign_: sign,
        }
    }
    pub fn new(sign: Sign, mut exponent: i32, mut coefficient: u64) -> Self {
        let mut format_class = if coefficient != 0 {
            FormatClass::kClassNormal
        } else {
            FormatClass::kClassZero
        };
        if (kExponentMin..=kExponentMax).contains(&exponent) {
            while coefficient > kMaxCoefficient {
                coefficient /= 10;
                exponent += 1;
            }
        }
        if exponent > kExponentMax {
            return Self::special(sign, FormatClass::kClassInfinity);
        }
        if exponent < kExponentMin {
            coefficient = 0;
            exponent = 0;
            format_class = FormatClass::kClassZero;
        }
        Self {
            coefficient_: coefficient,
            exponent_: exponent as i16,
            format_class_: format_class,
            sign_: sign,
        }
    }
    pub fn Coefficient(self) -> u64 {
        self.coefficient_
    }
    pub fn Exponent(self) -> i32 {
        self.exponent_ as i32
    }
    pub fn IsFinite(self) -> bool {
        !self.IsSpecial()
    }
    pub fn IsInfinity(self) -> bool {
        self.format_class_ == FormatClass::kClassInfinity
    }
    pub fn IsNaN(self) -> bool {
        self.format_class_ == FormatClass::kClassNaN
    }
    pub fn IsSpecial(self) -> bool {
        self.IsInfinity() || self.IsNaN()
    }
    pub fn IsZero(self) -> bool {
        self.format_class_ == FormatClass::kClassZero
    }
    pub fn GetSign(self) -> Sign {
        self.sign_
    }
    pub fn SetSign(&mut self, sign: Sign) {
        self.sign_ = sign;
    }
}

// Source: decimal.cc:54-115.
enum HandleResult {
    kBothFinite,
    kBothInfinity,
    kEitherNaN,
    kLhsIsInfinity,
    kRhsIsInfinity,
}
enum HandlerResult {
    kResultIsLhs,
    kResultIsRhs,
    kResultIsUnknown,
}
struct SpecialValueHandler {
    lhs_: Decimal,
    rhs_: Decimal,
    result_: HandlerResult,
}
impl SpecialValueHandler {
    fn new(lhs: Decimal, rhs: Decimal) -> Self {
        Self {
            lhs_: lhs,
            rhs_: rhs,
            result_: HandlerResult::kResultIsUnknown,
        }
    }
    fn Handle(&mut self) -> HandleResult {
        if self.lhs_.IsFinite() && self.rhs_.IsFinite() {
            return HandleResult::kBothFinite;
        }
        if self.lhs_.IsNaN() {
            self.result_ = HandlerResult::kResultIsLhs;
            return HandleResult::kEitherNaN;
        }
        if self.rhs_.IsNaN() {
            self.result_ = HandlerResult::kResultIsRhs;
            return HandleResult::kEitherNaN;
        }
        if self.lhs_.IsInfinity() {
            return if self.rhs_.IsInfinity() {
                HandleResult::kBothInfinity
            } else {
                HandleResult::kLhsIsInfinity
            };
        }
        debug_assert!(self.rhs_.IsInfinity());
        HandleResult::kRhsIsInfinity
    }
    fn Value(&self) -> Decimal {
        match self.result_ {
            HandlerResult::kResultIsLhs => self.lhs_,
            HandlerResult::kResultIsRhs => self.rhs_,
            HandlerResult::kResultIsUnknown => unreachable!(),
        }
    }
}

// Source: decimal.cc:117-183. Keep Chromium's limb helper and reduction points.
struct UInt128 {
    high_: u64,
    low_: u64,
}
impl UInt128 {
    fn High(&self) -> u64 {
        self.high_
    }
    fn Low(&self) -> u64 {
        self.low_
    }
    fn HighUInt32(x: u64) -> u32 {
        (x >> 32) as u32
    }
    fn LowUInt32(x: u64) -> u32 {
        x as u32
    }
    fn MakeUInt64(low: u32, high: u32) -> u64 {
        low as u64 | ((high as u64) << 32)
    }
    fn Multiply(u: u64, v: u64) -> Self {
        Self {
            low_: u.wrapping_mul(v),
            high_: Self::MultiplyHigh(u, v),
        }
    }
    fn MultiplyHigh(u: u64, v: u64) -> u64 {
        let u_low = Self::LowUInt32(u) as u64;
        let u_high = Self::HighUInt32(u) as u64;
        let v_low = Self::LowUInt32(v) as u64;
        let v_high = Self::HighUInt32(v) as u64;
        let partial_product = u_high * v_low + Self::HighUInt32(u_low * v_low) as u64;
        u_high * v_high
            + Self::HighUInt32(partial_product) as u64
            + Self::HighUInt32(u_low * v_high + Self::LowUInt32(partial_product) as u64) as u64
    }
}
impl DivAssign<u32> for UInt128 {
    fn div_assign(&mut self, divisor: u32) {
        debug_assert_ne!(divisor, 0);
        if self.high_ == 0 {
            self.low_ /= divisor as u64;
            return;
        }
        let dividend = [
            Self::LowUInt32(self.low_),
            Self::HighUInt32(self.low_),
            Self::LowUInt32(self.high_),
            Self::HighUInt32(self.high_),
        ];
        let mut quotient = [0u32; 4];
        let mut remainder = 0u32;
        for i in (0..4).rev() {
            let work = Self::MakeUInt64(dividend[i], remainder);
            remainder = (work % divisor as u64) as u32;
            quotient[i] = (work / divisor as u64) as u32;
        }
        self.low_ = Self::MakeUInt64(quotient[0], quotient[1]);
        self.high_ = Self::MakeUInt64(quotient[2], quotient[3]);
    }
}

// Source: decimal.cc:185-221.
fn CountDigits(x: u64) -> i32 {
    let mut number_of_digits = 0;
    let mut power_of_ten = 1u64;
    while x >= power_of_ten {
        number_of_digits += 1;
        if power_of_ten >= u64::MAX / 10 {
            break;
        }
        power_of_ten *= 10;
    }
    number_of_digits
}
fn ScaleDown(mut x: u64, mut n: i32) -> u64 {
    debug_assert!(n >= 0);
    while n > 0 && x != 0 {
        x /= 10;
        n -= 1;
    }
    x
}
fn ScaleUp(x: u64, mut n: i32) -> u64 {
    debug_assert!(n >= 0 && n <= kPrecision);
    let mut y = 1u64;
    let mut z = 10u64;
    loop {
        if n & 1 != 0 {
            y = y.wrapping_mul(z);
        }
        n >>= 1;
        if n == 0 {
            return x.wrapping_mul(y);
        }
        z = z.wrapping_mul(z);
    }
}

// Source: decimal.h:47-179; decimal.cc:259-272.
#[derive(Clone, Copy, Debug)]
pub struct Decimal {
    data_: EncodedData,
}
struct AlignedOperands {
    lhs_coefficient: u64,
    rhs_coefficient: u64,
    exponent: i32,
}
impl Default for Decimal {
    fn default() -> Self {
        Self::FromInt(0)
    }
}
impl Decimal {
    pub fn FromInt(i32: i32) -> Self {
        Self::new(
            if i32 < 0 {
                Sign::kNegative
            } else {
                Sign::kPositive
            },
            0,
            if i32 < 0 {
                -(i32 as i64) as u64
            } else {
                i32 as u64
            },
        )
    }
    pub fn new(sign: Sign, exponent: i32, coefficient: u64) -> Self {
        Self {
            data_: EncodedData::new(sign, exponent, coefficient),
        }
    }
    pub fn FromComponents(negative: bool, exponent: i32, coefficient: u64) -> Self {
        Self::new(Sign::from_negative(negative), exponent, coefficient)
    }
    pub fn FromEncodedData(data: EncodedData) -> Self {
        Self { data_: data }
    }
    pub fn Value(self) -> EncodedData {
        self.data_
    }
    pub fn Exponent(self) -> i32 {
        debug_assert!(self.IsFinite());
        self.data_.Exponent()
    }
    pub fn IsFinite(self) -> bool {
        self.data_.IsFinite()
    }
    pub fn IsInfinity(self) -> bool {
        self.data_.IsInfinity()
    }
    pub fn IsNaN(self) -> bool {
        self.data_.IsNaN()
    }
    pub fn IsNegative(self) -> bool {
        self.GetSign() == Sign::kNegative
    }
    pub fn IsPositive(self) -> bool {
        self.GetSign() == Sign::kPositive
    }
    pub fn IsSpecial(self) -> bool {
        self.data_.IsSpecial()
    }
    pub fn IsZero(self) -> bool {
        self.data_.IsZero()
    }
    fn GetSign(self) -> Sign {
        self.data_.GetSign()
    }
    fn InvertSign(sign: Sign) -> Sign {
        if sign == Sign::kNegative {
            Sign::kPositive
        } else {
            Sign::kNegative
        }
    }
    // Source: decimal.cc:523-527.
    pub fn Abs(self) -> Self {
        let mut result = self;
        result.data_.SetSign(Sign::kPositive);
        result
    }
    // Source: decimal.cc:529-574.
    fn AlignOperands(lhs: Self, rhs: Self) -> AlignedOperands {
        debug_assert!(lhs.IsFinite() && rhs.IsFinite());
        let lhs_exponent = lhs.Exponent();
        let rhs_exponent = rhs.Exponent();
        let mut exponent = lhs_exponent.min(rhs_exponent);
        let mut lhs_coefficient = lhs.data_.Coefficient();
        let mut rhs_coefficient = rhs.data_.Coefficient();
        if lhs_exponent > rhs_exponent {
            let number_of_lhs_digits = CountDigits(lhs_coefficient);
            if number_of_lhs_digits != 0 {
                let lhs_shift_amount = lhs_exponent - rhs_exponent;
                let overflow = number_of_lhs_digits + lhs_shift_amount - kPrecision;
                if overflow <= 0 {
                    lhs_coefficient = ScaleUp(lhs_coefficient, lhs_shift_amount);
                } else {
                    lhs_coefficient = ScaleUp(lhs_coefficient, lhs_shift_amount - overflow);
                    rhs_coefficient = ScaleDown(rhs_coefficient, overflow);
                    exponent += overflow;
                }
            }
        } else if lhs_exponent < rhs_exponent {
            let number_of_rhs_digits = CountDigits(rhs_coefficient);
            if number_of_rhs_digits != 0 {
                let rhs_shift_amount = rhs_exponent - lhs_exponent;
                let overflow = number_of_rhs_digits + rhs_shift_amount - kPrecision;
                if overflow <= 0 {
                    rhs_coefficient = ScaleUp(rhs_coefficient, rhs_shift_amount);
                } else {
                    rhs_coefficient = ScaleUp(rhs_coefficient, rhs_shift_amount - overflow);
                    lhs_coefficient = ScaleDown(lhs_coefficient, overflow);
                    exponent += overflow;
                }
            }
        }
        AlignedOperands {
            exponent,
            lhs_coefficient,
            rhs_coefficient,
        }
    }
    // Source: decimal.cc:580-599.
    pub fn Ceil(self) -> Self {
        if self.IsSpecial() || self.Exponent() >= 0 {
            return self;
        }
        let mut result = self.data_.Coefficient();
        let number_of_digits = CountDigits(result);
        let number_of_drop_digits = -self.Exponent();
        if number_of_digits <= number_of_drop_digits {
            return if self.IsPositive() {
                Self::FromInt(1)
            } else {
                Self::Zero(Sign::kPositive)
            };
        }
        result = ScaleDown(result, number_of_drop_digits);
        if self.IsPositive()
            && !IsMultiplePowersOfTen(self.data_.Coefficient(), number_of_drop_digits)
        {
            result += 1;
        }
        Self::new(self.GetSign(), 0, result)
    }
    // Source: decimal.cc:601-617.
    fn CompareTo(self, rhs: Self) -> Self {
        let result = self - rhs;
        match result.data_.format_class_ {
            FormatClass::kClassInfinity => Self::FromInt(if result.IsNegative() { -1 } else { 1 }),
            FormatClass::kClassNaN | FormatClass::kClassNormal => result,
            FormatClass::kClassZero => Self::Zero(Sign::kPositive),
        }
    }
    // Source: decimal.cc:619-638.
    pub fn Floor(self) -> Self {
        if self.IsSpecial() || self.Exponent() >= 0 {
            return self;
        }
        let mut result = self.data_.Coefficient();
        let number_of_digits = CountDigits(result);
        let number_of_drop_digits = -self.Exponent();
        if number_of_digits < number_of_drop_digits {
            return if self.IsPositive() {
                Self::Zero(Sign::kPositive)
            } else {
                Self::FromInt(-1)
            };
        }
        result = ScaleDown(result, number_of_drop_digits);
        if self.IsNegative()
            && !IsMultiplePowersOfTen(self.data_.Coefficient(), number_of_drop_digits)
        {
            result += 1;
        }
        Self::new(self.GetSign(), 0, result)
    }
    // Source: decimal.cc:640-648. The formatter is a required typed leaf;
    // no alternative binary-float formatting algorithm is substituted.
    pub fn FromDouble(double_value: f64, formatter: &dyn NumberToStringEcmaScript) -> Self {
        if double_value.is_finite() {
            return Self::FromString(&formatter.NumberToStringEcmaScript(double_value));
        }
        if double_value.is_infinite() {
            return Self::Infinity(Sign::from_negative(double_value < 0.0));
        }
        Self::Nan()
    }
    // Source: decimal.cc:650-842. Keep the source's state machine, including
    // truncation of excess significant digits and early exponent overflow.
    pub fn FromString(str: &str) -> Self {
        let mut exponent = 0i32;
        let mut exponent_sign = Sign::kPositive;
        let mut number_of_digits = 0;
        let mut number_of_digits_after_dot = 0;
        let mut number_of_extra_digits = 0;
        let mut sign = Sign::kPositive;
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            kStateDigit,
            kStateDot,
            kStateDotDigit,
            kStateE,
            kStateEDigit,
            kStateESign,
            kStateSign,
            kStateStart,
            kStateZero,
        }
        use State::*;
        let mut state = kStateStart;
        let mut accumulator = 0u64;
        for ch in str.encode_utf16() {
            let digit = (b'0' as u16..=b'9' as u16).contains(&ch);
            let nonzero_digit = (b'1' as u16..=b'9' as u16).contains(&ch);
            match state {
                kStateDigit => {
                    if digit {
                        if number_of_digits < kPrecision {
                            number_of_digits += 1;
                            accumulator *= 10;
                            accumulator += (ch - b'0' as u16) as u64;
                        } else {
                            number_of_extra_digits += 1;
                        }
                        continue;
                    }
                    if ch == b'.' as u16 {
                        state = kStateDot;
                        continue;
                    }
                    if ch == b'E' as u16 || ch == b'e' as u16 {
                        state = kStateE;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateDot | kStateDotDigit => {
                    if digit {
                        if number_of_digits < kPrecision {
                            number_of_digits += 1;
                            number_of_digits_after_dot += 1;
                            accumulator *= 10;
                            accumulator += (ch - b'0' as u16) as u64;
                        }
                        state = kStateDotDigit;
                        continue;
                    }
                    if ch == b'E' as u16 || ch == b'e' as u16 {
                        state = kStateE;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateE => {
                    if ch == b'+' as u16 {
                        exponent_sign = Sign::kPositive;
                        state = kStateESign;
                        continue;
                    }
                    if ch == b'-' as u16 {
                        exponent_sign = Sign::kNegative;
                        state = kStateESign;
                        continue;
                    }
                    if digit {
                        exponent = (ch - b'0' as u16) as i32;
                        state = kStateEDigit;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateEDigit => {
                    if digit {
                        exponent *= 10;
                        exponent += (ch - b'0' as u16) as i32;
                        if exponent > kExponentMax + kPrecision {
                            if accumulator != 0 {
                                return if exponent_sign == Sign::kNegative {
                                    Self::Zero(Sign::kPositive)
                                } else {
                                    Self::Infinity(sign)
                                };
                            }
                            return Self::Zero(sign);
                        }
                        state = kStateEDigit;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateESign => {
                    if digit {
                        exponent = (ch - b'0' as u16) as i32;
                        state = kStateEDigit;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateSign => {
                    if nonzero_digit {
                        accumulator = (ch - b'0' as u16) as u64;
                        number_of_digits = 1;
                        state = kStateDigit;
                        continue;
                    }
                    if ch == b'0' as u16 {
                        state = kStateZero;
                        continue;
                    }
                    if ch == b'.' as u16 {
                        state = kStateDot;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateStart => {
                    if nonzero_digit {
                        accumulator = (ch - b'0' as u16) as u64;
                        number_of_digits = 1;
                        state = kStateDigit;
                        continue;
                    }
                    if ch == b'-' as u16 {
                        sign = Sign::kNegative;
                        state = kStateSign;
                        continue;
                    }
                    if ch == b'+' as u16 {
                        sign = Sign::kPositive;
                        state = kStateSign;
                        continue;
                    }
                    if ch == b'0' as u16 {
                        state = kStateZero;
                        continue;
                    }
                    if ch == b'.' as u16 {
                        state = kStateDot;
                        continue;
                    }
                    return Self::Nan();
                }
                kStateZero => {
                    if ch == b'0' as u16 {
                        continue;
                    }
                    if nonzero_digit {
                        accumulator = (ch - b'0' as u16) as u64;
                        number_of_digits = 1;
                        state = kStateDigit;
                        continue;
                    }
                    if ch == b'.' as u16 {
                        state = kStateDot;
                        continue;
                    }
                    if ch == b'E' as u16 || ch == b'e' as u16 {
                        state = kStateE;
                        continue;
                    }
                    return Self::Nan();
                }
            }
        }
        if state == kStateZero {
            return Self::Zero(sign);
        }
        if state == kStateDigit || state == kStateEDigit || state == kStateDotDigit {
            let mut result_exponent = exponent
                * if exponent_sign == Sign::kNegative {
                    -1
                } else {
                    1
                }
                - number_of_digits_after_dot
                + number_of_extra_digits;
            if result_exponent < kExponentMin {
                return Self::Zero(Sign::kPositive);
            }
            let overflow = result_exponent - kExponentMax + 1;
            if overflow > 0 {
                if overflow + number_of_digits - number_of_digits_after_dot > kPrecision {
                    return Self::Infinity(sign);
                }
                accumulator = ScaleUp(accumulator, overflow);
                result_exponent -= overflow;
            }
            return Self::new(sign, result_exponent, accumulator);
        }
        Self::Nan()
    }
    // Source: decimal.cc:844-850,977-979.
    pub fn Infinity(sign: Sign) -> Self {
        Self::FromEncodedData(EncodedData::special(sign, FormatClass::kClassInfinity))
    }
    pub fn Nan() -> Self {
        Self::FromEncodedData(EncodedData::special(
            Sign::kPositive,
            FormatClass::kClassNaN,
        ))
    }
    pub fn Zero(sign: Sign) -> Self {
        Self::FromEncodedData(EncodedData::special(sign, FormatClass::kClassZero))
    }
    // Source: decimal.cc:852-859.
    pub fn Remainder(self, rhs: Self) -> Self {
        let quotient = self / rhs;
        if quotient.IsSpecial() {
            quotient
        } else {
            self - (if quotient.IsNegative() {
                quotient.Ceil()
            } else {
                quotient.Floor()
            }) * rhs
        }
    }
    // Source: decimal.cc:861-879.
    pub fn Round(self) -> Self {
        if self.IsSpecial() || self.Exponent() >= 0 {
            return self;
        }
        let mut result = self.data_.Coefficient();
        let number_of_digits = CountDigits(result);
        let number_of_drop_digits = -self.Exponent();
        if number_of_digits < number_of_drop_digits {
            return Self::Zero(Sign::kPositive);
        }
        result = ScaleDown(result, number_of_drop_digits - 1);
        if result % 10 >= 5 {
            result += 10;
        }
        result /= 10;
        Self::new(self.GetSign(), 0, result)
    }
    // Source: decimal.cc:881-892. String conversion remains a required source leaf.
    pub fn ToDouble(self, parser: &dyn StringToDouble) -> f64 {
        if self.IsFinite() {
            return parser.StringToDouble(&self.ToString()).unwrap_or(f64::NAN);
        }
        if self.IsInfinity() {
            return if self.IsNegative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        f64::NAN
    }
    // Source: decimal.cc:894-975.
    pub fn ToString(self) -> String {
        match self.data_.format_class_ {
            FormatClass::kClassInfinity => {
                return if self.IsNegative() {
                    "-Infinity"
                } else {
                    "Infinity"
                }
                .into()
            }
            FormatClass::kClassNaN => return "NaN".into(),
            FormatClass::kClassNormal | FormatClass::kClassZero => {}
        }
        let mut builder = String::new();
        if self.IsNegative() {
            builder.push('-');
        }
        let mut original_exponent = self.Exponent();
        let mut coefficient = self.data_.Coefficient();
        if original_exponent < 0 {
            const kMaxDigits: i32 = 15; // DBL_DIG.
            let mut last_digit = 0;
            while CountDigits(coefficient) > kMaxDigits {
                last_digit = coefficient % 10;
                coefficient /= 10;
                original_exponent += 1;
            }
            if last_digit >= 5 {
                coefficient += 1;
            }
            while original_exponent < 0 && coefficient != 0 && coefficient % 10 == 0 {
                coefficient /= 10;
                original_exponent += 1;
            }
        }
        let digits = coefficient.to_string();
        let mut coefficient_length = digits.len() as i32;
        let adjusted_exponent = original_exponent + coefficient_length - 1;
        if original_exponent <= 0 && adjusted_exponent >= -6 {
            if original_exponent == 0 {
                builder.push_str(&digits);
                return builder;
            }
            if adjusted_exponent >= 0 {
                for (i, digit) in digits.chars().enumerate() {
                    builder.push(digit);
                    if i as i32 == adjusted_exponent {
                        builder.push('.');
                    }
                }
                return builder;
            }
            builder.push_str("0.");
            for _ in adjusted_exponent + 1..0 {
                builder.push('0');
            }
            builder.push_str(&digits);
        } else {
            builder.push(digits.as_bytes()[0] as char);
            while coefficient_length >= 2
                && digits.as_bytes()[coefficient_length as usize - 1] == b'0'
            {
                coefficient_length -= 1;
            }
            if coefficient_length >= 2 {
                builder.push('.');
                builder.push_str(&digits[1..coefficient_length as usize]);
            }
            if adjusted_exponent != 0 {
                builder.push_str(if adjusted_exponent < 0 { "e" } else { "e+" });
                builder.push_str(&adjusted_exponent.to_string());
            }
        }
        builder
    }
}

// Source dependency: wtf_string.cc:341-344 -> DoubleToStringConverter /
// double-conversion shortest conversion. An embedding must supply this exact
// source leaf; Decimal does not silently replace it with std::fmt or f64 math.
pub trait NumberToStringEcmaScript {
    fn NumberToStringEcmaScript(&self, number: f64) -> String;
}
// Source dependency: wtf/text/string_to_number.h, StringToDouble. `None`
// represents failed conversion rather than a NaN arithmetic result.
pub trait StringToDouble {
    fn StringToDouble(&self, string: &str) -> Option<f64>;
}

// Source: decimal.cc:576-578.
fn IsMultiplePowersOfTen(coefficient: u64, n: i32) -> bool {
    coefficient == 0 || coefficient % ScaleUp(1, n) == 0
}

// Source: decimal.cc:274-292.
impl AddAssign for Decimal {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl SubAssign for Decimal {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl MulAssign for Decimal {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}
impl DivAssign for Decimal {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}
// Source: decimal.cc:294-301.
impl Neg for Decimal {
    type Output = Self;
    fn neg(self) -> Self {
        if self.IsNaN() {
            return self;
        }
        let mut result = self;
        result.data_.SetSign(Self::InvertSign(self.data_.GetSign()));
        result
    }
}
// Source: decimal.cc:303-340.
impl Add for Decimal {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let lhs = self;
        let lhs_sign = lhs.GetSign();
        let rhs_sign = rhs.GetSign();
        let mut handler = SpecialValueHandler::new(lhs, rhs);
        match handler.Handle() {
            HandleResult::kBothFinite => {}
            HandleResult::kBothInfinity => {
                return if lhs_sign == rhs_sign {
                    lhs
                } else {
                    Self::Nan()
                }
            }
            HandleResult::kEitherNaN => return handler.Value(),
            HandleResult::kLhsIsInfinity => return lhs,
            HandleResult::kRhsIsInfinity => return rhs,
        }
        let aligned_operands = Self::AlignOperands(lhs, rhs);
        let result = if lhs_sign == rhs_sign {
            aligned_operands
                .lhs_coefficient
                .wrapping_add(aligned_operands.rhs_coefficient)
        } else {
            aligned_operands
                .lhs_coefficient
                .wrapping_sub(aligned_operands.rhs_coefficient)
        };
        if lhs_sign == Sign::kNegative && rhs_sign == Sign::kPositive && result == 0 {
            return Self::new(Sign::kPositive, aligned_operands.exponent, 0);
        }
        if result as i64 >= 0 {
            Self::new(lhs_sign, aligned_operands.exponent, result)
        } else {
            Self::new(
                Self::InvertSign(lhs_sign),
                aligned_operands.exponent,
                (result as i64).wrapping_neg() as u64,
            )
        }
    }
}
// Source: decimal.cc:342-379.
impl Sub for Decimal {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let lhs = self;
        let lhs_sign = lhs.GetSign();
        let rhs_sign = rhs.GetSign();
        let mut handler = SpecialValueHandler::new(lhs, rhs);
        match handler.Handle() {
            HandleResult::kBothFinite => {}
            HandleResult::kBothInfinity => {
                return if lhs_sign == rhs_sign {
                    Self::Nan()
                } else {
                    lhs
                }
            }
            HandleResult::kEitherNaN => return handler.Value(),
            HandleResult::kLhsIsInfinity => return lhs,
            HandleResult::kRhsIsInfinity => return Self::Infinity(Self::InvertSign(rhs_sign)),
        }
        let aligned_operands = Self::AlignOperands(lhs, rhs);
        let result = if lhs_sign == rhs_sign {
            aligned_operands
                .lhs_coefficient
                .wrapping_sub(aligned_operands.rhs_coefficient)
        } else {
            aligned_operands
                .lhs_coefficient
                .wrapping_add(aligned_operands.rhs_coefficient)
        };
        if lhs_sign == Sign::kNegative && rhs_sign == Sign::kNegative && result == 0 {
            return Self::new(Sign::kPositive, aligned_operands.exponent, 0);
        }
        if result as i64 >= 0 {
            Self::new(lhs_sign, aligned_operands.exponent, result)
        } else {
            Self::new(
                Self::InvertSign(lhs_sign),
                aligned_operands.exponent,
                (result as i64).wrapping_neg() as u64,
            )
        }
    }
}
// Source: decimal.cc:381-415.
impl Mul for Decimal {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let lhs = self;
        let lhs_sign = lhs.GetSign();
        let rhs_sign = rhs.GetSign();
        let result_sign = if lhs_sign == rhs_sign {
            Sign::kPositive
        } else {
            Sign::kNegative
        };
        let mut handler = SpecialValueHandler::new(lhs, rhs);
        match handler.Handle() {
            HandleResult::kBothFinite => {
                let lhs_coefficient = lhs.data_.Coefficient();
                let rhs_coefficient = rhs.data_.Coefficient();
                let mut result_exponent = lhs.Exponent() + rhs.Exponent();
                let mut work = UInt128::Multiply(lhs_coefficient, rhs_coefficient);
                while work.High() != 0 {
                    work /= 10;
                    result_exponent += 1;
                }
                Self::new(result_sign, result_exponent, work.Low())
            }
            HandleResult::kBothInfinity => Self::Infinity(result_sign),
            HandleResult::kEitherNaN => handler.Value(),
            HandleResult::kLhsIsInfinity => {
                if rhs.IsZero() {
                    Self::Nan()
                } else {
                    Self::Infinity(result_sign)
                }
            }
            HandleResult::kRhsIsInfinity => {
                if lhs.IsZero() {
                    Self::Nan()
                } else {
                    Self::Infinity(result_sign)
                }
            }
        }
    }
}
// Source: decimal.cc:417-476.
impl Div for Decimal {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let lhs = self;
        let result_sign = if lhs.GetSign() == rhs.GetSign() {
            Sign::kPositive
        } else {
            Sign::kNegative
        };
        let mut handler = SpecialValueHandler::new(lhs, rhs);
        match handler.Handle() {
            HandleResult::kBothFinite => {}
            HandleResult::kBothInfinity => return Self::Nan(),
            HandleResult::kEitherNaN => return handler.Value(),
            HandleResult::kLhsIsInfinity => return Self::Infinity(result_sign),
            HandleResult::kRhsIsInfinity => return Self::Zero(result_sign),
        }
        debug_assert!(lhs.IsFinite() && rhs.IsFinite());
        if rhs.IsZero() {
            return if lhs.IsZero() {
                Self::Nan()
            } else {
                Self::Infinity(result_sign)
            };
        }
        let mut result_exponent = lhs.Exponent() - rhs.Exponent();
        if lhs.IsZero() {
            return Self::new(result_sign, result_exponent, 0);
        }
        let mut remainder = lhs.data_.Coefficient();
        let divisor = rhs.data_.Coefficient();
        let mut result = 0;
        loop {
            while remainder < divisor && result < kMaxCoefficient / 10 {
                remainder *= 10;
                result *= 10;
                result_exponent -= 1;
            }
            if remainder < divisor {
                break;
            }
            let quotient = remainder / divisor;
            if result > kMaxCoefficient - quotient {
                break;
            }
            result += quotient;
            remainder %= divisor;
            if remainder == 0 {
                break;
            }
        }
        if remainder > divisor / 2 {
            result += 1;
        }
        Self::new(result_sign, result_exponent, result)
    }
}

// Source: decimal.cc:478-489. Chromium Decimal's != is intentionally not the
// complement of == for NaN; preserve both operator definitions explicitly.
impl PartialEq for Decimal {
    fn eq(&self, rhs: &Self) -> bool {
        self.data_ == rhs.data_ || self.CompareTo(*rhs).IsZero()
    }
    fn ne(&self, rhs: &Self) -> bool {
        if self.data_ == rhs.data_ {
            return false;
        }
        let result = self.CompareTo(*rhs);
        if result.IsNaN() {
            return false;
        }
        !result.IsZero()
    }
}
// Source: decimal.cc:491-521.
impl PartialOrd for Decimal {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let result = self.CompareTo(*rhs);
        if result.IsNaN() {
            return if self.data_ == rhs.data_ {
                Some(Ordering::Equal)
            } else {
                None
            };
        }
        Some(if result.IsZero() {
            Ordering::Equal
        } else if result.IsNegative() {
            Ordering::Less
        } else {
            Ordering::Greater
        })
    }
    fn lt(&self, rhs: &Self) -> bool {
        let result = self.CompareTo(*rhs);
        if result.IsNaN() {
            return false;
        }
        !result.IsZero() && result.IsNegative()
    }
    fn le(&self, rhs: &Self) -> bool {
        if self.data_ == rhs.data_ {
            return true;
        }
        let result = self.CompareTo(*rhs);
        if result.IsNaN() {
            return false;
        }
        result.IsZero() || result.IsNegative()
    }
    fn gt(&self, rhs: &Self) -> bool {
        let result = self.CompareTo(*rhs);
        if result.IsNaN() {
            return false;
        }
        !result.IsZero() && result.IsPositive()
    }
    fn ge(&self, rhs: &Self) -> bool {
        if self.data_ == rhs.data_ {
            return true;
        }
        let result = self.CompareTo(*rhs);
        if result.IsNaN() {
            return false;
        }
        result.IsZero() || !result.IsNegative()
    }
}
