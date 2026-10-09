// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/css_property_parser.cc:257-297,384-418
//
// This file currently contains the complete generated-keyword lookup spine.
// The CSSPropertyParser class is added here as its value types are translated;
// keeping the helper in its Chromium source module preserves file ownership.

#![allow(non_snake_case)]

use crate::css_value_keywords::{kMaxCSSValueKeywordLength, kNumCSSValueKeywords, FindValue};
use foundation::{CSSValueID, StringView};

// cpp: css_property_parser.cc:257-297,384-418
// Both source storage-width paths and the unusual Latin-1 bitwise lowercase
// operation are retained exactly.
pub(crate) fn CssValueKeywordID(string: &StringView) -> CSSValueID {
    let length = string.length() as usize;
    if length == 0 || length > kMaxCSSValueKeywordLength {
        return CSSValueID::kInvalid;
    }
    let mut buffer = [0u8; kMaxCSSValueKeywordLength];
    if let Some(chars) = string.Span8() {
        let complete = length & !3;
        for (src, dst) in chars[..complete]
            .chunks_exact(4)
            .zip(buffer[..complete].chunks_exact_mut(4))
        {
            let mut x = u32::from_ne_bytes(src.try_into().expect("four bytes"));
            x |= (x & 0x40404040) >> 1;
            dst.copy_from_slice(&x.to_ne_bytes());
        }
        for i in complete..length {
            let c = chars[i];
            buffer[i] = c | ((c & 0x40) >> 1);
        }
    } else {
        for (i, &c) in string.Span16().iter().enumerate() {
            if c == 0 || c >= 0x7f {
                return CSSValueID::kInvalid;
            }
            buffer[i] = (c as u8).to_ascii_lowercase();
        }
    }
    let entry = FindValue(&buffer[..length]);
    #[cfg(debug_assertions)]
    {
        for (i, &c) in string.Span16().iter().enumerate() {
            buffer[i] = if (65..=90).contains(&c) {
                (c + 32) as u8
            } else {
                c as u8
            };
        }
        debug_assert_eq!(
            entry.map(|value| value as *const _),
            FindValue(&buffer[..length]).map(|value| value as *const _)
        );
    }
    entry.map_or(CSSValueID::kInvalid, |entry| {
        assert!((0..kNumCSSValueKeywords).contains(&entry.id));
        // SAFETY: the generated repr(i32) enum is contiguous and the generated
        // lookup table contains only values in that range.
        unsafe { std::mem::transmute::<i32, CSSValueID>(entry.id) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::String;

    #[test]
    fn keyword_lookup_preserves_both_source_width_paths() {
        assert_eq!(
            CssValueKeywordID(&StringView::from(&String::from_latin1(b"CaLc"))),
            CSSValueID::kCalc
        );
        assert_eq!(
            CssValueKeywordID(&StringView::from(&String::from_utf16(&[
                b'C' as u16,
                b'a' as u16,
                b'L' as u16,
                b'c' as u16,
            ]))),
            CSSValueID::kCalc
        );
        assert_eq!(
            CssValueKeywordID(&StringView::from(&String::from_utf16(&[0x161]))),
            CSSValueID::kInvalid
        );
    }
}
