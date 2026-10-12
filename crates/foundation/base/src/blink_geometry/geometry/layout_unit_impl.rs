// C++: src/foundation/blink_geometry/geometry/layout_unit.cc

use std::fmt;

use super::layout_unit::{FixedPoint, FixedStorage};

// C++: String::Number(value.ToDouble(), 14) called by FromLayoutUnit.
// Keep the same 14 significant digits and the same fixed/scientific cutoff.
fn fixed_precision_string(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let negative = value.is_sign_negative();
    let absolute = value.abs();
    let scientific = format!("{:e}", absolute);
    let (_, exponent_text) = scientific.split_once('e').unwrap();
    let mut exponent: i32 = exponent_text.parse().unwrap();

    // String::Number rounds the binary double to 14 decimal significant
    // digits, including halfway values away from zero. Rust's formatter uses
    // ties-to-even, so compute the decimal rounding from the f64 bits.
    let bits = absolute.to_bits();
    let binary_exponent = ((bits >> 52) & 0x7ff) as i32;
    let mut numerator = ((bits & ((1_u64 << 52) - 1)) | (1_u64 << 52)) as i128;
    let mut denominator = 1_i128;
    let shift = 1075 - binary_exponent;
    if shift > 0 {
        denominator <<= shift as u32;
    } else {
        numerator <<= (-shift) as u32;
    }
    let decimal_shift = 13 - exponent;
    if decimal_shift > 0 {
        numerator *= 10_i128.pow(decimal_shift as u32);
    } else if decimal_shift < 0 {
        denominator *= 10_i128.pow((-decimal_shift) as u32);
    }
    let mut rounded = numerator / denominator;
    if (numerator % denominator) * 2 >= denominator {
        rounded += 1;
    }
    if rounded >= 100_000_000_000_000 {
        rounded /= 10;
        exponent += 1;
    }
    let all_digits = format!("{:014}", rounded);
    let digits = all_digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let sign = if negative { "-" } else { "" };

    if (-6..14).contains(&exponent) {
        let point = exponent + 1;
        if point <= 0 {
            return format!("{}0.{}{}", sign, "0".repeat((-point) as usize), digits);
        }
        let point = point as usize;
        if point >= digits.len() {
            return format!("{}{}{}", sign, digits, "0".repeat(point - digits.len()));
        }
        return format!("{}{}.{}", sign, &digits[..point], &digits[point..]);
    }

    let mantissa = format!("{}.{}", &all_digits[..1], &all_digits[1..]);
    format!("{}{}e{:+}", sign, mantissa, exponent)
}

// C++: anonymous-namespace FromLayoutUnit.
fn FromLayoutUnit<const FRACTIONAL_BITS: u32, Storage: FixedStorage>(
    value: FixedPoint<FRACTIONAL_BITS, Storage>,
) -> String {
    fixed_precision_string(value.ToDouble())
}

// C++ class FixedPoint<fractional_bits, Storage>::ToString -> Rust impl.
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> FixedPoint<FRACTIONAL_BITS, Storage> {
    pub fn ToString(&self) -> String {
        if self.RawValue() == Self::Max().RawValue() {
            return format!("Max({})", FromLayoutUnit(*self));
        }
        if self.RawValue() == Self::Min().RawValue() {
            return format!("Min({})", FromLayoutUnit(*self));
        }
        if self.RawValue() == Self::NearlyMax().RawValue() {
            return format!("NearlyMax({})", FromLayoutUnit(*self));
        }
        if self.RawValue() == Self::NearlyMin().RawValue() {
            return format!("NearlyMin({})", FromLayoutUnit(*self));
        }
        FromLayoutUnit(*self)
    }
}

// C++: operator<<(std::ostream&, FixedPoint).
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> fmt::Display
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn fmt(&self, stream: &mut fmt::Formatter<'_>) -> fmt::Result {
        stream.write_str(&self.ToString())
    }
}
