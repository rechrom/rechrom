// Migrated unchanged in behavior from tiny-skia-0.12.0/src/fixed_point.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Skia uses fixed points pretty chaotically, therefore we cannot use
// strongly typed wrappers. Which is unfortunate.

use crate::path::SaturateCast;

use crate::raster::math::{bound, left_shift, left_shift64};

/// A 26.6 fixed point.
pub type FDot6 = i32;

/// A 24.8 fixed point.
pub type FDot8 = i32;

/// A 16.16 fixed point.
pub type SkFixed = i32;
/// Legacy Rust spelling.
pub use SkFixed as FDot16;

pub use crate::src::core::SkFDot6 as fdot6;

pub mod fdot8 {
    use super::*;

    // Extracted from SkScan_Antihair.cpp

    pub fn from_fdot16(x: FDot16) -> FDot8 {
        (x + 0x80) >> 8
    }
}

pub const HALF: FDot16 = (1 << 16) / 2;
pub const ONE: FDot16 = 1 << 16;

// `from_f32` seems to lack a rounding step. For all fixed-point
// values, this version is as accurate as possible for (fixed -> float -> fixed). Rounding reduces
// accuracy if the intermediate floats are in the range that only holds integers (adding 0.5 to an
// odd integer then snaps to nearest even). Using double for the rounding math gives maximum
// accuracy for (float -> fixed -> float), but that's usually overkill.
pub fn SkFloatToFixed(x: f32) -> FDot16 {
    i32::saturate_from(x * ONE as f32)
}

pub fn SkFixedFloorToInt(x: FDot16) -> i32 {
    x >> 16
}

pub fn SkFixedCeilToInt(x: FDot16) -> i32 {
    (x + ONE - 1) >> 16
}

pub fn SkFixedRoundToInt(x: FDot16) -> i32 {
    (x + HALF) >> 16
}

// The divide may exceed 32 bits. Clamp to a signed 32 bit result.
pub fn SkFixedMul(a: FDot16, b: FDot16) -> FDot16 {
    ((i64::from(a) * i64::from(b)) >> 16) as FDot16
}

// The divide may exceed 32 bits. Clamp to a signed 32 bit result.
pub fn SkFixedDiv(numer: FDot6, denom: FDot6) -> FDot16 {
    let v = left_shift64(numer as i64, 16) / denom as i64;
    let n = bound(i32::MIN as i64, v, i32::MAX as i64);
    n as i32
}

pub fn fast_div(a: FDot6, b: FDot6) -> FDot16 {
    debug_assert!((left_shift(a, 16) >> 16) == a);
    debug_assert!(b != 0);
    left_shift(a, 16) / b
}

/// Legacy fdot16 namespace; implementation functions use upstream names.
pub mod fdot16 {
    pub use super::fast_div;
    pub use super::SkFixedCeilToInt as ceil_to_i32;
    pub use super::SkFixedDiv as div;
    pub use super::SkFixedFloorToInt as floor_to_i32;
    pub use super::SkFixedMul as mul;
    pub use super::SkFixedRoundToInt as round_to_i32;
    pub use super::SkFloatToFixed as from_f32;
    pub use super::{HALF, ONE};
}
