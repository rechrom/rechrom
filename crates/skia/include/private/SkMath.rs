// Migrated unchanged in behavior from tiny-skia-0.12.0/src/math.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::raster::LengthU32;

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

// Perfectly safe.
pub const LENGTH_U32_ONE: LengthU32 = unsafe { LengthU32::new_unchecked(1) };

pub fn SkLeftShift(value: i32, shift: i32) -> i32 {
    ((value as u32) << shift) as i32
}

pub fn SkLeftShift64(value: i64, shift: i32) -> i64 {
    ((value as u64) << shift) as i64
}

pub fn bound<T: Ord + Copy>(min: T, value: T, max: T) -> T {
    max.min(value).max(min)
}

pub(crate) use crate::src::opts::SkRasterPipeline_opts::SK_OPTS_NS::approx_powf;

// Rust aliases distinguish the two C++ SkLeftShift overloads.
pub use SkLeftShift as left_shift;
pub use SkLeftShift64 as left_shift64;

// include/private/SkMath.h: rounded byte-channel multiplication.
pub fn SkMulDiv255Round(a: u8, b: u8) -> u8 {
    let v = u32::from(a) * u32::from(b) + 128;
    ((v + (v >> 8)) >> 8) as u8
}
