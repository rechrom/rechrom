// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
// BSD-3-Clause; see LICENSE and provenance in translation_map.json.
//! Fixed-point 26.6 routines from src/core/SkFDot6.h.
//! can_convert_to_fdot16 / small_scale are Rust scan-converter helpers.

use crate::include::private::SkFixed::{fdot16, FDot16};
use crate::raster::math::left_shift;
pub type SkFDot6 = i32;
use core::convert::TryFrom;
pub use SkFDot6 as FDot6;

pub const ONE: FDot6 = 64;

pub fn SkIntToFDot6(n: i32) -> FDot6 {
    debug_assert!(n as i16 as i32 == n);
    n << 6
}

pub fn SkFloatToFDot6(n: f32) -> FDot6 {
    (n * 64.0) as i32
}

pub fn SkFDot6Floor(n: FDot6) -> FDot6 {
    n >> 6
}

pub fn SkFDot6Ceil(n: FDot6) -> FDot6 {
    (n + 63) >> 6
}

pub fn SkFDot6Round(n: FDot6) -> FDot6 {
    (n + 32) >> 6
}

pub fn SkFDot6ToFixed(n: FDot6) -> FDot16 {
    debug_assert!((left_shift(n, 10) >> 10) == n);
    left_shift(n, 10)
}

pub fn SkFDot6Div(a: FDot6, b: FDot6) -> FDot16 {
    debug_assert_ne!(b, 0);

    if i16::try_from(a).is_ok() {
        left_shift(a, 16) / b
    } else {
        fdot16::div(a, b)
    }
}

pub fn can_convert_to_fdot16(n: FDot6) -> bool {
    let max_dot6 = i32::MAX >> (16 - 6);
    n.abs() <= max_dot6
}

pub fn small_scale(value: u8, dot6: FDot6) -> u8 {
    debug_assert!(dot6 as u32 <= 64);
    ((value as i32 * dot6) >> 6) as u8
}

// Rust compatibility spellings.
pub use SkFDot6Ceil as ceil;
pub use SkFDot6Div as div;
pub use SkFDot6Floor as floor;
pub use SkFDot6Round as round;
pub use SkFDot6ToFixed as to_fdot16;
pub use SkFloatToFDot6 as from_f32;
pub use SkIntToFDot6 as from_i32;
