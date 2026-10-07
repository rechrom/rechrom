#![allow(non_snake_case)]
use crate::style_resolver::selector::SourceSpace;
use std::ffi::{c_char, c_double};
unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> c_double;
}
// cpp: style_resolver/style_resolver.cc:830-837
// Use the platform C numeric conversion, as the source does. This is a C libc
// boundary, not a C++ resolver bridge. The explicit trailing NUL also retains
// std::string::c_str's behavior when an embedded NUL occurs in the input.
pub(crate) fn Number(text: &str) -> Option<f64> {
    let mut value = text
        .trim_matches(|c: char| c.source_space())
        .as_bytes()
        .to_vec();
    value.push(0);
    let start = value.as_ptr().cast::<c_char>();
    let mut end = std::ptr::null_mut();
    let result = unsafe { strtod(start, &mut end) };
    if end == start.cast_mut() || unsafe { *end } != 0 || !result.is_finite() {
        None
    } else {
        Some(result)
    }
}

// cpp: style_resolver/style_resolver.cc:1834-1840
pub(crate) fn PositiveInteger(value: &str) -> Option<u32> {
    Number(value)
        .filter(|n| *n >= 1.0 && *n <= 65535.0 && n.floor() == *n)
        .map(|n| n as u32)
}
