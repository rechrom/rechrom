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
// Source: string-to-double.cc:101-152,419-746,749-762.
// The configuration is WTF dtoa.cc:57-62: no radix flags, no separator,
// no Infinity/NaN symbols, no trailing spaces or spaces after the sign.
use super::strtod::StrtodTrimmed;
use crate::number_conversion::dtoa::{ParsedDouble, StringToDoubleOptions};
const kMaxSignificantDigits: usize = 772;

pub(super) fn StringToDouble(input: &[u8], options: StringToDoubleOptions) -> ParsedDouble {
    let junk = || ParsedDouble {
        value: options.junk_string_value,
        parsed_length: 0,
    };
    let mut current = 0;
    let end = input.len();
    if current == end {
        return ParsedDouble {
            value: options.empty_string_value,
            parsed_length: 0,
        };
    }
    if options.allow_leading_spaces {
        while current != end && IsWhitespace(input[current]) {
            current += 1;
        }
        if current == end {
            return ParsedDouble {
                value: options.empty_string_value,
                parsed_length: current,
            };
        }
    }
    let mut exponent = 0i32;
    let mut insignificant_digits = 0i32;
    let mut nonzero_dropped = false;
    let mut sign = false;
    if input[current] == b'+' || input[current] == b'-' {
        sign = input[current] == b'-';
        current += 1;
        // ALLOW_SPACES_AFTER_SIGN is absent. The source's AdvanceToNonspace
        // rejects either end or any space immediately following the sign.
        if current == end || IsWhitespace(input[current]) {
            return junk();
        }
    }
    let mut leading_zero = false;
    if input[current] == b'0' {
        current += 1;
        if current == end {
            return SignedZero(sign, current);
        }
        leading_zero = true;
        while input[current] == b'0' {
            current += 1;
            if current == end {
                return SignedZero(sign, current);
            }
        }
    }
    let mut buffer = Vec::with_capacity(kMaxSignificantDigits + 10);
    while current < end && input[current].is_ascii_digit() {
        if buffer.len() < kMaxSignificantDigits {
            buffer.push(input[current]);
        } else {
            insignificant_digits += 1;
            nonzero_dropped |= input[current] != b'0';
        }
        current += 1;
    }
    if current < end && input[current] == b'.' {
        current += 1;
        if current == end {
            if buffer.is_empty() && !leading_zero {
                return junk();
            }
            return ParsingDone(
                buffer,
                exponent,
                insignificant_digits,
                nonzero_dropped,
                sign,
                current,
            );
        }
        if buffer.is_empty() {
            while input[current] == b'0' {
                current += 1;
                if current == end {
                    return SignedZero(sign, current);
                }
                exponent -= 1;
            }
        }
        while current < end && input[current].is_ascii_digit() {
            if buffer.len() < kMaxSignificantDigits {
                buffer.push(input[current]);
                exponent -= 1;
            } else {
                nonzero_dropped |= input[current] != b'0';
            }
            current += 1;
        }
    }
    if !leading_zero && exponent == 0 && buffer.is_empty() {
        return junk();
    }
    if current < end && (input[current] == b'e' || input[current] == b'E') {
        let junk_begin = current;
        current += 1;
        let mut exponent_negative = false;
        if current < end && (input[current] == b'+' || input[current] == b'-') {
            exponent_negative = input[current] == b'-';
            current += 1;
        }
        if current == end || !input[current].is_ascii_digit() {
            if !options.allow_trailing_junk {
                return junk();
            }
            current = junk_begin;
        } else {
            let max_exponent = i32::MAX / 2;
            let mut num = 0i32;
            while current < end && input[current].is_ascii_digit() {
                let digit = (input[current] - b'0') as i32;
                if num >= max_exponent / 10
                    && !(num == max_exponent / 10 && digit <= max_exponent % 10)
                {
                    num = max_exponent;
                } else {
                    num = num * 10 + digit;
                }
                current += 1;
            }
            exponent += if exponent_negative { -num } else { num };
        }
    }
    if !options.allow_trailing_junk && current != end {
        return junk();
    }
    ParsingDone(
        buffer,
        exponent,
        insignificant_digits,
        nonzero_dropped,
        sign,
        current,
    )
}
// Source: string-to-double.cc:707-746, no ALLOW_OCTALS branch.
fn ParsingDone(
    mut buffer: Vec<u8>,
    mut exponent: i32,
    insignificant: i32,
    nonzero_dropped: bool,
    sign: bool,
    parsed_length: usize,
) -> ParsedDouble {
    exponent += insignificant;
    if nonzero_dropped {
        buffer.push(b'1');
        exponent -= 1;
    }
    let original_length = buffer.len();
    while buffer.last() == Some(&b'0') {
        buffer.pop();
    }
    exponent += (original_length - buffer.len()) as i32;
    let converted = StrtodTrimmed(&buffer, exponent);
    ParsedDouble {
        value: if sign { -converted } else { converted },
        parsed_length,
    }
}
fn SignedZero(sign: bool, parsed_length: usize) -> ParsedDouble {
    ParsedDouble {
        value: if sign { -0.0 } else { 0.0 },
        parsed_length,
    }
}
// Source: string-to-double.cc:112-135. The char overload used by WTF has the
// six ASCII whitespace values; non-ASCII UTF-16 units are projected to NUL by
// the WTF adapter before this leaf is called.
fn IsWhitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\r' | b'\n' | b'\t' | 11 | 12)
}
