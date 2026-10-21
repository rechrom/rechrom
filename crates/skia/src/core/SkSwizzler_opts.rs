// Copyright 2023 Google LLC. BSD-style license.
use super::SkSwizzlePriv::{Swizzle_8888_u32, Swizzle_8888_u8};
use crate::src::opts::SkSwizzler_opts as default;
// Native ARM64 default dispatch never replaces these bindings at runtime.
// Source: src/core/SkSwizzler_opts.cpp:21-21
pub static RGBA_to_BGRA: Swizzle_8888_u32 = default::RGBA_to_BGRA;
// Source: src/core/SkSwizzler_opts.cpp:22-22
pub static RGBA_to_rgbA: Swizzle_8888_u32 = default::RGBA_to_rgbA;
// Source: src/core/SkSwizzler_opts.cpp:23-23
pub static RGBA_to_bgrA: Swizzle_8888_u32 = default::RGBA_to_bgrA;
// Source: src/core/SkSwizzler_opts.cpp:24-24
pub static rgbA_to_RGBA: Swizzle_8888_u32 = default::rgbA_to_RGBA;
// Source: src/core/SkSwizzler_opts.cpp:25-25
pub static rgbA_to_BGRA: Swizzle_8888_u32 = default::rgbA_to_BGRA;
// Source: src/core/SkSwizzler_opts.cpp:26-26
pub static RGB_to_RGB1: Swizzle_8888_u8 = default::RGB_to_RGB1;
// Source: src/core/SkSwizzler_opts.cpp:27-27
pub static RGB_to_BGR1: Swizzle_8888_u8 = default::RGB_to_BGR1;
// Source: src/core/SkSwizzler_opts.cpp:28-28
pub static gray_to_RGB1: Swizzle_8888_u8 = default::gray_to_RGB1;
// Source: src/core/SkSwizzler_opts.cpp:29-29
pub static grayA_to_RGBA: Swizzle_8888_u8 = default::grayA_to_RGBA;
// Source: src/core/SkSwizzler_opts.cpp:30-30
pub static grayA_to_rgbA: Swizzle_8888_u8 = default::grayA_to_rgbA;
// Source: src/core/SkSwizzler_opts.cpp:31-31
pub static inverted_CMYK_to_RGB1: Swizzle_8888_u32 = default::inverted_CMYK_to_RGB1;
// Source: src/core/SkSwizzler_opts.cpp:32-32
pub static inverted_CMYK_to_BGR1: Swizzle_8888_u32 = default::inverted_CMYK_to_BGR1;
// Source: src/core/SkSwizzler_opts.cpp:38-38
// Source: src/core/SkSwizzler_opts.cpp:54-55
fn init() -> bool {
    true
}
// Source: src/core/SkSwizzler_opts.cpp:57-59
pub fn Init_Swizzler() {
    static INITIALIZED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let _ = INITIALIZED.get_or_init(init);
}
