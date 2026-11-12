/****************************************************************
 *
 * The author of this software is David M. Gay.
 *
 * Copyright (c) 1991, 2000, 2001 by Lucent Technologies.
 * Copyright (C) 2002, 2005, 2006, 2007, 2008, 2010, 2012 Apple Inc.
 * All rights reserved.
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose without fee is hereby granted, provided that this entire notice
 * is included in all copies of any software which is or includes a copy
 * or modification of this software and in all copies of the supporting
 * documentation for such software.
 *
 * THIS SOFTWARE IS BEING PROVIDED "AS IS", WITHOUT ANY EXPRESS OR IMPLIED
 * WARRANTY.  IN PARTICULAR, NEITHER THE AUTHOR NOR LUCENT MAKES ANY
 * REPRESENTATION OR WARRANTY OF ANY KIND CONCERNING THE MERCHANTABILITY
 * OF THIS SOFTWARE OR ITS FITNESS FOR ANY PARTICULAR PURPOSE.
 *
 ***************************************************************/

/* Please send bug reports to David M. Gay (dmg at acm dot org,
 * with " at " changed at "@" and " dot " changed to ".").    */

/* On a machine with IEEE extended-precision registers, it is
 * necessary to specify double-precision (53-bit) rounding precision
 * before invoking strtod or dtoa.  If the machine uses (the equivalent
 * of) Intel 80x87 arithmetic, the call
 *    _control87(PC_53, MCW_PC);
 * does this with many compilers.  Whether this or another call is
 * appropriate depends on the compiler; for this to work, it may be
 * necessary to #include "float.h" or another system-dependent header
 * file.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/wtf/dtoa.cc:47-76,171-193
// Numeric algorithms live inside the translated double-conversion source owner.

// Source: dtoa.cc:57-62. No infinity/NaN symbols are accepted by this converter.
#[derive(Clone, Copy)]
pub struct StringToDoubleOptions {
    pub allow_leading_spaces: bool,
    pub allow_trailing_junk: bool,
    pub empty_string_value: f64,
    pub junk_string_value: f64,
}
const kStringToDoubleOptions: StringToDoubleOptions = StringToDoubleOptions {
    allow_leading_spaces: true,
    allow_trailing_junk: true,
    empty_string_value: 0.0,
    junk_string_value: 0.0,
};
pub struct ParsedDouble {
    pub value: f64,
    pub parsed_length: usize,
}
// Source leaves:
// base/third_party/double_conversion/double-conversion/double-to-string.cc:
//   EcmaScriptConverter:42-52; ToShortestIeeeNumber:167-202;
//   DoubleToAscii:379-438 -> FastDtoa / BignumDtoa.
// base/third_party/double_conversion/double-conversion/string-to-double.cc:
//   StringToIeee:419-746; StringToDouble:749-762 -> StrtodTrimmed.
// Implemented by double_conversion::ChromiumDoubleConversion, preserving these
// algorithms/configurations; the trait remains the narrow adapter boundary.
pub trait DoubleConversion {
    fn EcmaScriptToShortest(&self, number: f64) -> String;
    fn EcmaScriptToPrecision(&self, number: f64, precision: usize) -> Option<String>;
    fn StringToDouble(&self, string: &[u8], options: StringToDoubleOptions) -> ParsedDouble;
}

// Source: dtoa.cc:171-178. Saturation matches base::saturated_cast<int>.
pub fn ParseDouble(string: &[u8], converter: &dyn DoubleConversion) -> ParsedDouble {
    let length = string.len().min(i32::MAX as usize);
    converter.StringToDouble(&string[..length], kStringToDoubleOptions)
}
// Source: dtoa.cc:47-55,180-193.
pub fn ParseDoubleUtf16(string: &[u16], converter: &dyn DoubleConversion) -> ParsedDouble {
    const kConversionBufferSize: usize = 64;
    if string.len() > kConversionBufferSize {
        let conversion_buffer: Vec<u8> = string
            .iter()
            .map(|&ch| if ch <= 0x7f { ch as u8 } else { 0 })
            .collect();
        return ParseDouble(&conversion_buffer, converter);
    }
    let mut conversion_buffer = [0u8; kConversionBufferSize];
    for (i, &ch) in string.iter().enumerate() {
        conversion_buffer[i] = if ch <= 0x7f { ch as u8 } else { 0 };
    }
    ParseDouble(&conversion_buffer[..string.len()], converter)
}

/// WTF DoubleToStringConverter::ToStringWithFixedPrecision and its original
/// FormatStringTruncatingTrailingZerosIfNeeded, dtoa.cc:78-143. In particular,
/// source exponential results retain their significant trailing zeroes.
pub fn ToStringWithFixedPrecision(
    number: f64,
    precision: usize,
    converter: &dyn DoubleConversion,
) -> Option<String> {
    let mut result = converter.EcmaScriptToPrecision(number, precision)?;
    if result.as_bytes().contains(&b'e') {
        return Some(result);
    }
    if let Some(point) = result.as_bytes().iter().position(|&c| c == b'.') {
        let mut end = result.len();
        while end > point + 1 && result.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if end == point + 1 {
            end = point;
        }
        result.truncate(end);
    }
    Some(result)
}

/// WTF string_to_number.cc:233-271,361-364 (strict UTF-16 overload).
pub fn StringToDoubleUtf16(input: &[u16], converter: &dyn DoubleConversion) -> Option<f64> {
    let leading = input
        .iter()
        .take_while(|&&c| matches!(c, 9..=13 | 32))
        .count();
    let parsed = ParseDoubleUtf16(&input[leading..], converter);
    (parsed.parsed_length != 0 && parsed.parsed_length + leading == input.len())
        .then_some(parsed.value)
}
