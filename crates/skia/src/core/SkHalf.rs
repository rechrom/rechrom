// Copyright 2014 Google Inc. BSD-style license.
// Source: src/core/SkHalf.h:16-16
// Source: src/core/SkHalf.h:18-23
pub type SkHalf = u16;
pub const SK_HalfNaN: u16 = 0x7c01;
pub const SK_HalfInfinity: u16 = 0x7c00;
pub const SK_HalfMin: u16 = 0x0400;
pub const SK_HalfMax: u16 = 0x7bff;
pub const SK_HalfEpsilon: u16 = 0x1400;
pub const SK_Half1: u16 = 0x3c00;
// Source: src/core/SkHalf.h:29-29
// Source: src/core/SkHalf.cpp:16-22
pub fn SkFloatToHalf(f: f32) -> SkHalf {
    if f.is_nan() {
        SK_HalfNaN
    } else {
        super::SkVx::to_half([f])[0]
    }
}
// Source: src/core/SkHalf.h:28-28
// Source: src/core/SkHalf.cpp:24-26
pub fn SkHalfToFloat(h: SkHalf) -> f32 {
    super::SkVx::from_half([h])[0]
}
