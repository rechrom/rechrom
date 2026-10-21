// Source: src/core/SkSwizzlePriv.h:18-18
pub type Swizzle_8888_u32 = unsafe fn(*mut u32, *const u32, i32);
// Source: src/core/SkSwizzlePriv.h:19-25
pub use super::SkSwizzler_opts::{
    inverted_CMYK_to_BGR1, inverted_CMYK_to_RGB1, rgbA_to_BGRA, rgbA_to_RGBA, RGBA_to_BGRA,
    RGBA_to_bgrA, RGBA_to_rgbA,
};
// Source: src/core/SkSwizzlePriv.h:27-27
pub type Swizzle_8888_u8 = unsafe fn(*mut u32, *const u8, i32);
// Source: src/core/SkSwizzlePriv.h:28-32
pub use super::SkSwizzler_opts::{
    grayA_to_RGBA, grayA_to_rgbA, gray_to_RGB1, RGB_to_BGR1, RGB_to_RGB1,
};
// Source: src/core/SkSwizzlePriv.h:34-34
pub use super::SkSwizzler_opts::Init_Swizzler;
// Copyright 2016 Google Inc. BSD-style license.
// Source: src/core/SkSwizzlePriv.h:37-39
pub fn swizzle_rb(x: [f32; 4]) -> [f32; 4] {
    super::SkVx::shuffle(x, [2, 1, 0, 3])
}
// Source: src/core/SkSwizzlePriv.h:41-47
pub fn swizzle_rb_if_bgra(x: [f32; 4]) -> [f32; 4] {
    if super::SkColorPriv::SK_PMCOLOR_IS_BGRA {
        swizzle_rb(x)
    } else {
        x
    }
}
// Source: src/core/SkSwizzlePriv.h:49-51
pub fn Sk4f_fromL32(px: u32) -> [f32; 4] {
    px.to_ne_bytes().map(|v| v as f32 * (1.0 / 255.0))
}
// Source: src/core/SkSwizzlePriv.h:53-54
// Source: src/core/SkSwizzlePriv.h:57-60
pub fn Sk4f_toL32(px: [f32; 4]) -> u32 {
    let rounded = px.map(|v| v * 255.0 + 0.5);
    let bytes = super::SkVx::pin(rounded, [0.0; 4], [255.0; 4]).map(|v| v as u8);
    u32::from_ne_bytes(bytes)
}
