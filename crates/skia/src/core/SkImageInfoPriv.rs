// Copyright 2017 Google Inc. BSD-style license.
use super::SkColor::*;
use super::SkImageInfo::SkColorTypeBytesPerPixel;
use crate::include::core::SkAlphaType::kLastEnum_SkAlphaType;
use crate::include::core::SkColorType::*;
// Source: src/core/SkImageInfoPriv.h:15-48
pub fn SkColorTypeChannelFlags(ct: SkColorType) -> u32 {
    match ct {
        kUnknown_SkColorType => 0,
        kAlpha_8_SkColorType => kAlpha_SkColorChannelFlag,
        kRGB_565_SkColorType => kRGB_SkColorChannelFlags,
        kARGB_4444_SkColorType => kRGBA_SkColorChannelFlags,
        kRGBA_8888_SkColorType => kRGBA_SkColorChannelFlags,
        kRGB_888x_SkColorType => kRGB_SkColorChannelFlags,
        kBGRA_8888_SkColorType => kRGBA_SkColorChannelFlags,
        kRGBA_1010102_SkColorType => kRGBA_SkColorChannelFlags,
        kRGB_101010x_SkColorType => kRGB_SkColorChannelFlags,
        kBGRA_1010102_SkColorType => kRGBA_SkColorChannelFlags,
        kBGR_101010x_SkColorType => kRGB_SkColorChannelFlags,
        kBGR_101010x_XR_SkColorType => kRGB_SkColorChannelFlags,
        kBGRA_10101010_XR_SkColorType => kRGBA_SkColorChannelFlags,
        kRGBA_10x6_SkColorType => kRGBA_SkColorChannelFlags,
        kGray_8_SkColorType => kGray_SkColorChannelFlag,
        kRGBA_F16Norm_SkColorType => kRGBA_SkColorChannelFlags,
        kRGBA_F16_SkColorType => kRGBA_SkColorChannelFlags,
        kRGB_F16F16F16x_SkColorType => kRGB_SkColorChannelFlags,
        kRGBA_F32_SkColorType => kRGBA_SkColorChannelFlags,
        kR8G8_unorm_SkColorType => kRG_SkColorChannelFlags,
        kA16_unorm_SkColorType => kAlpha_SkColorChannelFlag,
        kR16_unorm_SkColorType => kRed_SkColorChannelFlag,
        kR16G16_unorm_SkColorType => kRG_SkColorChannelFlags,
        kA16_float_SkColorType => kAlpha_SkColorChannelFlag,
        kR16_float_SkColorType => kRed_SkColorChannelFlag,
        kR16G16_float_SkColorType => kRG_SkColorChannelFlags,
        kR16G16B16A16_unorm_SkColorType => kRGBA_SkColorChannelFlags,
        kSRGBA_8888_SkColorType => kRGBA_SkColorChannelFlags,
        kR8_unorm_SkColorType => kRed_SkColorChannelFlag,
    }
}
// Source: src/core/SkImageInfoPriv.h:50-65
pub fn SkColorTypeNumChannels(ct: SkColorType) -> i32 {
    match SkColorTypeChannelFlags(ct) {
        kRed_SkColorChannelFlag => 1,
        kAlpha_SkColorChannelFlag => 1,
        kGray_SkColorChannelFlag => 1,
        kGrayAlpha_SkColorChannelFlags => 2,
        kRG_SkColorChannelFlags => 2,
        kRGB_SkColorChannelFlags => 3,
        kRGBA_SkColorChannelFlags => 4,
        0 => 0,
        _ => {
            crate::SkDEBUGFAIL!(@source b"src/core/SkImageInfoPriv.h\0".as_ptr(),61; b"unexpected color channel flags\0".as_ptr());
            0
        }
    }
}
// Source: src/core/SkImageInfoPriv.h:67-69
// Source: src/core/SkImageInfoPriv.h:71-73
pub fn SkColorTypeIsAlphaOnly(ct: SkColorType) -> bool {
    SkColorTypeChannelFlags(ct) == kAlpha_SkColorChannelFlag
}
pub fn SkAlphaTypeIsValid(value: u32) -> bool {
    value <= kLastEnum_SkAlphaType as u32
}
// Source: src/core/SkImageInfoPriv.h:75-108
pub fn SkColorTypeShiftPerPixel(ct: SkColorType) -> i32 {
    match ct {
        kUnknown_SkColorType => 0,
        kAlpha_8_SkColorType => 0,
        kRGB_565_SkColorType => 1,
        kARGB_4444_SkColorType => 1,
        kRGBA_8888_SkColorType => 2,
        kRGB_888x_SkColorType => 2,
        kBGRA_8888_SkColorType => 2,
        kRGBA_1010102_SkColorType => 2,
        kRGB_101010x_SkColorType => 2,
        kBGRA_1010102_SkColorType => 2,
        kBGR_101010x_SkColorType => 2,
        kBGR_101010x_XR_SkColorType => 2,
        kBGRA_10101010_XR_SkColorType => 3,
        kRGBA_10x6_SkColorType => 3,
        kGray_8_SkColorType => 0,
        kRGBA_F16Norm_SkColorType => 3,
        kRGBA_F16_SkColorType => 3,
        kRGB_F16F16F16x_SkColorType => 3,
        kRGBA_F32_SkColorType => 4,
        kR8G8_unorm_SkColorType => 1,
        kA16_unorm_SkColorType => 1,
        kR16_unorm_SkColorType => 1,
        kR16G16_unorm_SkColorType => 2,
        kA16_float_SkColorType => 1,
        kR16_float_SkColorType => 1,
        kR16G16_float_SkColorType => 2,
        kR16G16B16A16_unorm_SkColorType => 3,
        kSRGBA_8888_SkColorType => 2,
        kR8_unorm_SkColorType => 0,
    }
}
// Source: src/core/SkImageInfoPriv.h:110-112
// Source: src/core/SkImageInfoPriv.h:114-116
// Source: src/core/SkImageInfoPriv.h:118-125
pub fn SkColorTypeMinRowBytes(ct: SkColorType, width: i32) -> usize {
    width.wrapping_mul(SkColorTypeBytesPerPixel(ct)) as usize
}
pub fn SkColorTypeIsValid(value: u32) -> bool {
    value <= kLastEnum_SkColorType as u32
}
pub fn SkColorTypeComputeOffset(ct: SkColorType, x: i32, y: i32, rowBytes: usize) -> usize {
    crate::SkASSERT!(@source b"src/core/SkImageInfoPriv.h\0".as_ptr(),119; @condition b"x >= 0\0".as_ptr(); x >= 0);
    crate::SkASSERT!(@source b"src/core/SkImageInfoPriv.h\0".as_ptr(),120; @condition b"y >= 0\0".as_ptr(); y >= 0);
    if ct == kUnknown_SkColorType {
        return 0;
    }
    (y as usize)
        .wrapping_mul(rowBytes)
        .wrapping_add((x as usize) << SkColorTypeShiftPerPixel(ct))
}
// Source: src/core/SkImageInfoPriv.h:127-151
// Source: src/core/SkImageInfoPriv.h:153-163
pub fn SkColorTypeIsNormalized(ct: SkColorType) -> bool {
    match ct {
        kUnknown_SkColorType
        | kAlpha_8_SkColorType
        | kRGB_565_SkColorType
        | kARGB_4444_SkColorType
        | kRGBA_8888_SkColorType
        | kRGB_888x_SkColorType
        | kBGRA_8888_SkColorType
        | kRGBA_1010102_SkColorType
        | kRGB_101010x_SkColorType
        | kBGRA_1010102_SkColorType
        | kBGR_101010x_SkColorType
        | kRGBA_10x6_SkColorType
        | kGray_8_SkColorType
        | kRGBA_F16Norm_SkColorType
        | kR8G8_unorm_SkColorType
        | kA16_unorm_SkColorType
        | kA16_float_SkColorType
        | kR16_unorm_SkColorType
        | kR16G16_unorm_SkColorType
        | kR16G16B16A16_unorm_SkColorType
        | kSRGBA_8888_SkColorType
        | kR8_unorm_SkColorType => true,
        kBGRA_10101010_XR_SkColorType
        | kBGR_101010x_XR_SkColorType
        | kRGB_F16F16F16x_SkColorType
        | kRGBA_F16_SkColorType
        | kRGBA_F32_SkColorType
        | kR16_float_SkColorType
        | kR16G16_float_SkColorType => false,
    }
}
// Source: src/core/SkImageInfoPriv.h:165-168
// Source: src/core/SkImageInfoPriv.h:170-171
// Source: src/core/SkImageInfoPriv.h:173-174
// Source: src/core/SkImageInfoPriv.h:176-184
// Source: src/core/SkImageInfoPriv.h:186-193
// Source: src/core/SkImageInfoPriv.h:195-205
// Source: src/core/SkImageInfoPriv.h:207-211
pub fn SkColorTypeMaxBitsPerChannel(ct: SkColorType) -> i32 {
    match ct {
        kUnknown_SkColorType => 0,
        kARGB_4444_SkColorType => 4,
        kRGB_565_SkColorType => 6,
        kAlpha_8_SkColorType
        | kRGBA_8888_SkColorType
        | kRGB_888x_SkColorType
        | kBGRA_8888_SkColorType
        | kGray_8_SkColorType
        | kR8G8_unorm_SkColorType
        | kSRGBA_8888_SkColorType
        | kR8_unorm_SkColorType => 8,
        kRGBA_1010102_SkColorType
        | kRGB_101010x_SkColorType
        | kBGRA_1010102_SkColorType
        | kBGR_101010x_SkColorType
        | kBGR_101010x_XR_SkColorType
        | kBGRA_10101010_XR_SkColorType
        | kRGBA_10x6_SkColorType => 10,
        kRGBA_F16Norm_SkColorType
        | kA16_unorm_SkColorType
        | kA16_float_SkColorType
        | kR16_unorm_SkColorType
        | kR16_float_SkColorType
        | kR16G16_unorm_SkColorType
        | kR16G16B16A16_unorm_SkColorType
        | kRGBA_F16_SkColorType
        | kRGB_F16F16F16x_SkColorType
        | kR16G16_float_SkColorType => 16,
        kRGBA_F32_SkColorType => 32,
    }
}
// Source: src/core/SkImageInfoPriv.h:216-218
pub fn SkColorInfoIsValid(info: &super::SkImageInfo::SkColorInfo) -> bool {
    info.colorType() != kUnknown_SkColorType
        && info.alphaType() != crate::include::core::SkAlphaType::kUnknown_SkAlphaType
}
// Source: src/core/SkImageInfoPriv.h:223-226
// Source: src/core/SkImageInfoPriv.h:228-231
// Source: src/core/SkImageInfoPriv.h:233-234
pub fn SkImageInfoIsValid(info: &super::SkImageInfo::SkImageInfo) -> bool {
    if info.width() <= 0 || info.height() <= 0 {
        return false;
    }
    let kMaxDimension = i32::MAX >> 2;
    if info.width() > kMaxDimension || info.height() > kMaxDimension {
        return false;
    }
    SkColorInfoIsValid(info.colorInfo())
}
// Source: src/core/SkImageInfoPriv.h:240-242
pub fn SkImageInfoValidConversion(
    dst: &super::SkImageInfo::SkImageInfo,
    src: &super::SkImageInfo::SkImageInfo,
) -> bool {
    SkImageInfoIsValid(dst) && SkImageInfoIsValid(src)
}
