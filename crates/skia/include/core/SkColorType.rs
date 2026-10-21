// Copyright 2022 Google LLC. BSD-style license.
// Source: include/core/SkColorType.h:32-32
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkColorType {
    // Source: include/core/SkColorType.h:34-34
    kUnknown_SkColorType,
    // Source: include/core/SkColorType.h:37-37
    kAlpha_8_SkColorType,
    // Source: include/core/SkColorType.h:41-41
    kRGB_565_SkColorType,
    // Source: include/core/SkColorType.h:45-45
    kARGB_4444_SkColorType,
    // Source: include/core/SkColorType.h:48-48
    kRGBA_8888_SkColorType,
    // Source: include/core/SkColorType.h:52-52
    kRGB_888x_SkColorType,
    // Source: include/core/SkColorType.h:56-56
    kBGRA_8888_SkColorType,
    // Source: include/core/SkColorType.h:59-59
    kRGBA_1010102_SkColorType,
    // Source: include/core/SkColorType.h:63-63
    kBGRA_1010102_SkColorType,
    // Source: include/core/SkColorType.h:67-67
    kRGB_101010x_SkColorType,
    // Source: include/core/SkColorType.h:71-71
    kBGR_101010x_SkColorType,
    // Source: include/core/SkColorType.h:77-77
    kBGR_101010x_XR_SkColorType,
    // Source: include/core/SkColorType.h:83-83
    kBGRA_10101010_XR_SkColorType,
    // Source: include/core/SkColorType.h:87-87
    kRGBA_10x6_SkColorType,
    // Source: include/core/SkColorType.h:90-90
    kGray_8_SkColorType,
    // Source: include/core/SkColorType.h:94-94
    kRGBA_F16Norm_SkColorType,
    // Source: include/core/SkColorType.h:98-98
    kRGBA_F16_SkColorType,
    // Source: include/core/SkColorType.h:102-102
    kRGB_F16F16F16x_SkColorType,
    // Source: include/core/SkColorType.h:105-105
    kRGBA_F32_SkColorType,
    // Source: include/core/SkColorType.h:108-108
    kR8G8_unorm_SkColorType,
    // Source: include/core/SkColorType.h:111-111
    kA16_float_SkColorType,
    // Source: include/core/SkColorType.h:116-116
    kR16_float_SkColorType,
    // Source: include/core/SkColorType.h:120-120
    kR16G16_float_SkColorType,
    // Source: include/core/SkColorType.h:123-123
    kA16_unorm_SkColorType,
    // Source: include/core/SkColorType.h:127-127
    kR16_unorm_SkColorType,
    // Source: include/core/SkColorType.h:131-131
    kR16G16_unorm_SkColorType,
    // Source: include/core/SkColorType.h:134-134
    kR16G16B16A16_unorm_SkColorType,
    // Source: include/core/SkColorType.h:139-139
    kSRGBA_8888_SkColorType,
    // Source: include/core/SkColorType.h:143-143
    kR8_unorm_SkColorType,
}
// Associated constants preserve C++ duplicate-valued enumerators.
// Source: include/core/SkColorType.h:145-145
pub const kLastEnum_SkColorType: SkColorType = SkColorType::kR8_unorm_SkColorType;

// Source: include/core/SkColorType.h:147-155
pub const kN32_SkColorType: SkColorType = {
    use crate::include::core::SkTypes::{
        SkPMColorChannel::{A, B, G, R},
        SK_PMCOLOR_BYTE_ORDER,
    };
    if SK_PMCOLOR_BYTE_ORDER(B, G, R, A) {
        SkColorType::kBGRA_8888_SkColorType
    } else if SK_PMCOLOR_BYTE_ORDER(R, G, B, A) {
        SkColorType::kRGBA_8888_SkColorType
    } else {
        panic!("SK_*32_SHIFT values must correspond to BGRA or RGBA byte order")
    }
};
pub use SkColorType::*;
// Source: include/core/SkColorType.h:157-157
pub const kSkColorTypeCnt: i32 = kLastEnum_SkColorType as i32 + 1;
