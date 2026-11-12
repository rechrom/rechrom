// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source commands/smart_replace_cf.cc:36-94.

pub fn IsCharacterSmartReplaceExempt(c: u32, previous: bool) -> bool {
    let c = if c == 0xA0 { 0x20 } else { c };
    let whitespace = matches!(c, 0x09..=0x0D | 0x20 | 0x85 | 0xA0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000);
    if whitespace {
        return true;
    }
    // CFRange lengths are exclusive.
    let extra = 0;
    if [
        (0x1100, 256),
        (0x2E80, 352),
        (0x2FF0, 464),
        (0x3200, 29392),
        (0xAC00, 11183),
        (0xF900, 352),
        (0xFE30, 32),
        (0xFF00, 240),
        (0x20000, 0xA6D7),
        (0x2F800, 0x21E),
    ]
    .iter()
    .any(|&(start, length)| c >= start && c < start + length + extra)
    {
        return true;
    }
    let punctuation = if previous {
        "([\"'#$/-`{"
    } else {
        ")].,;:?'!\"%*-/}"
    };
    punctuation.bytes().any(|p| c == u32::from(p))
        || !previous && foundation::unicode::IsPunct(c as i32)
}
