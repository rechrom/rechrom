use super::SkColorSpaceXformSteps::SkColorSpaceXformSteps;
use super::SkImageInfo::SkImageInfo;
use crate::include::core::SkColorType::SkColorType::*;
// Source: src/core/SkConvertPixels.cpp:28-30
// Source: src/core/SkConvertPixels.cpp:32-38
// Source: src/core/SkConvertPixels.cpp:40-45
pub(crate) unsafe fn rect_memcpy(
    dstInfo: &SkImageInfo,
    dstPixels: *mut u8,
    dstRB: usize,
    srcInfo: &SkImageInfo,
    srcPixels: *const u8,
    srcRB: usize,
    steps: &SkColorSpaceXformSteps,
) -> bool {
    if dstInfo.colorType() != srcInfo.colorType() {
        return false;
    }
    if dstInfo.colorType() != kAlpha_8_SkColorType && steps.fFlags.mask() != 0b00000 {
        return false;
    }
    if srcPixels != dstPixels.cast_const() {
        super::SkRectMemcpy::SkRectMemcpy(
            dstPixels,
            dstRB,
            srcPixels,
            srcRB,
            dstInfo.minRowBytes(),
            dstInfo.height(),
        );
    }
    true
}
// Source: src/core/SkConvertPixels.cpp:47-86
unsafe fn swizzle_or_premul(
    dstInfo: &SkImageInfo,
    mut dstPixels: *mut u8,
    dstRB: usize,
    srcInfo: &SkImageInfo,
    mut srcPixels: *const u8,
    srcRB: usize,
    steps: &SkColorSpaceXformSteps,
) -> bool {
    use crate::src::opts::SkSwizzler_opts as swizzle;
    let is_8888 = |ct| matches!(ct, kRGBA_8888_SkColorType | kBGRA_8888_SkColorType);
    if !is_8888(dstInfo.colorType())
        || !is_8888(srcInfo.colorType())
        || steps.fFlags.linearize
        || steps.fFlags.gamut_transform
        || steps.fFlags.encode
    {
        return false;
    }
    let swapRB = dstInfo.colorType() != srcInfo.colorType();
    // The selected ARM64 implementation has the native NEON unpremul branch.
    // These are the existing original SkOpts default kernels, without a second dispatcher.
    let function: unsafe fn(*mut u32, *const u32, i32) = if steps.fFlags.premul {
        if swapRB {
            swizzle::RGBA_to_bgrA
        } else {
            swizzle::RGBA_to_rgbA
        }
    } else if steps.fFlags.unpremul {
        if swapRB {
            swizzle::rgbA_to_BGRA
        } else {
            swizzle::rgbA_to_RGBA
        }
    } else {
        crate::SkASSERT!(@source b"src/core/SkConvertPixels.cpp\0".as_ptr(),76; @condition b"swapRB\0".as_ptr(); swapRB);
        swizzle::RGBA_to_BGRA
    };
    for _ in 0..dstInfo.height() {
        function(dstPixels.cast(), srcPixels.cast(), dstInfo.width());
        dstPixels = dstPixels.wrapping_add(dstRB);
        srcPixels = srcPixels.wrapping_add(srcRB);
    }
    true
}
// Source: src/core/SkConvertPixels.cpp:88-94
// Source: src/core/SkConvertPixels.cpp:96-98
// Source: src/core/SkConvertPixels.cpp:101-103
// Source: src/core/SkConvertPixels.cpp:105-115
// Source: src/core/SkConvertPixels.cpp:117-135
// Source: src/core/SkConvertPixels.cpp:137-147
// Source: src/core/SkConvertPixels.cpp:149-161
// Source: src/core/SkConvertPixels.cpp:163-174
// Source: src/core/SkConvertPixels.cpp:176-187
// Source: src/core/SkConvertPixels.cpp:189-199
// Source: src/core/SkConvertPixels.cpp:201-211
// Source: src/core/SkConvertPixels.cpp:213-222
// Source: src/core/SkConvertPixels.cpp:225-246
// Source: src/core/SkConvertPixels.cpp:248-248
pub(crate) unsafe fn convert_to_alpha8(
    dstInfo: &SkImageInfo,
    mut dst: *mut u8,
    dstRB: usize,
    srcInfo: &SkImageInfo,
    src: *const u8,
    srcRB: usize,
    _steps: &SkColorSpaceXformSteps,
) -> bool {
    if dstInfo.colorType() != kAlpha_8_SkColorType {
        return false;
    }
    match srcInfo.colorType() {
        kUnknown_SkColorType | kAlpha_8_SkColorType => {
            crate::SkASSERT!(@source b"src/core/SkConvertPixels.cpp\0".as_ptr(),101; @condition b"false\0".as_ptr(); false);
            return false;
        }
        kA16_unorm_SkColorType => {
            let mut src16 = src.cast::<u16>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = (src16.add(x).read_unaligned() >> 8) as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src16 = src16.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kGray_8_SkColorType
        | kRGB_565_SkColorType
        | kR8G8_unorm_SkColorType
        | kR16_unorm_SkColorType
        | kR16_float_SkColorType
        | kR16G16_unorm_SkColorType
        | kR16G16_float_SkColorType
        | kRGB_888x_SkColorType
        | kRGB_101010x_SkColorType
        | kBGR_101010x_SkColorType
        | kBGR_101010x_XR_SkColorType
        | kRGB_F16F16F16x_SkColorType
        | kR8_unorm_SkColorType => {
            for _y in 0..srcInfo.height() {
                std::ptr::write_bytes(dst, 0xff, srcInfo.width() as usize);
                dst = dst.wrapping_add(dstRB);
            }
            return true;
        }
        kARGB_4444_SkColorType => {
            let mut src16 = src.cast::<u16>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) =
                        super::SkColorData::SkPacked4444ToA32(src16.add(x).read_unaligned() as u32)
                            as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src16 = src16.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kBGRA_8888_SkColorType | kRGBA_8888_SkColorType | kSRGBA_8888_SkColorType => {
            let mut src32 = src.cast::<u32>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = (src32.add(x).read_unaligned() >> 24) as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src32 = src32.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kRGBA_1010102_SkColorType | kBGRA_1010102_SkColorType => {
            let mut src32 = src.cast::<u32>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = ((src32.add(x).read_unaligned() >> 30) * 0x55) as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src32 = src32.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
            let mut src64 = src.cast::<u64>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = (255.0
                        * super::SkHalf::SkHalfToFloat(
                            (src64.add(x).read_unaligned() >> 48) as u16,
                        )) as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src64 = src64.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kRGBA_F32_SkColorType => {
            let mut rgba = src.cast::<f32>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = (255.0 * rgba.add(4 * x + 3).read_unaligned()) as u8;
                }
                dst = dst.wrapping_add(dstRB);
                rgba = rgba.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kA16_float_SkColorType => {
            let mut srcF16 = src.cast::<u16>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = (255.0
                        * super::SkHalf::SkHalfToFloat(srcF16.add(x).read_unaligned()))
                        as u8;
                }
                dst = dst.wrapping_add(dstRB);
                srcF16 = srcF16.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kBGRA_10101010_XR_SkColorType => {
            let mut src64 = src.cast::<u64>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    let kZero = 384i64;
                    let kRange = 510i64;
                    let kMaxU8 = 0xffi64;
                    let kMinU8 = 0i64;
                    let kDivisor = kRange / kMaxU8;
                    let raw = (src64.add(x).read_unaligned() >> 54) as i64;
                    let alpha = crate::include::private::SkTPin::SkTPin(
                        (raw - kZero) / kDivisor,
                        kMinU8,
                        kMaxU8,
                    );
                    *dst.add(x) = alpha as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src64 = src64.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
        kRGBA_10x6_SkColorType | kR16G16B16A16_unorm_SkColorType => {
            let mut src64 = src.cast::<u64>();
            for _y in 0..srcInfo.height() {
                for x in 0..srcInfo.width() {
                    let x = x as usize;
                    *dst.add(x) = ((src64.add(x).read_unaligned() >> 48) >> 8) as u8;
                }
                dst = dst.wrapping_add(dstRB);
                src64 = src64.cast::<u8>().wrapping_add(srcRB).cast();
            }
            return true;
        }
    }
}

// Source: src/core/SkConvertPixels.cpp:251-262
unsafe fn convert_with_pipeline(
    dstInfo: &SkImageInfo,
    dstRow: *mut u8,
    dstStride: i32,
    srcInfo: &SkImageInfo,
    srcRow: *const u8,
    srcStride: i32,
    steps: &SkColorSpaceXformSteps,
) -> Result<(), super::SkRasterPipeline::MissingStage> {
    use super::SkRasterPipelineOpContexts::MemoryCtx;
    let mut src = MemoryCtx {
        pixels: srcRow.cast_mut().cast(),
        stride: srcStride,
    };
    let mut dst = MemoryCtx {
        pixels: dstRow.cast(),
        stride: dstStride,
    };
    // Rust pinning adaptation preserves the original inline 256-byte arena lease.
    let mut storage = core::pin::pin!(super::SkArenaAlloc::SkSTArenaAlloc::<256>::new(0));
    let mut pipeline = super::SkRasterPipeline::SkRasterPipeline::new(storage.as_mut().arena());
    pipeline.appendLoad(srcInfo.colorType(), &mut src);
    steps.applyPipeline(&mut pipeline);
    pipeline.appendStore(dstInfo.colorType(), &mut dst);
    pipeline.run(0, 0, srcInfo.width() as usize, srcInfo.height() as usize)
}

// Source: src/core/SkConvertPixels.h:15-17
// Source: src/core/SkConvertPixels.cpp:264-292
/// Safety: source/destination spans must satisfy the native image-info dimensions,
/// row bytes, alignment, and synchronous read/write lease. The original allows
/// identical pixel pointers only for equal pixel widths. MissingStage remains a
/// typed execution error; native invalid conversion geometry still returns false.
pub unsafe fn SkConvertPixels(
    dstInfo: &SkImageInfo,
    dstPixels: *mut u8,
    dstRB: usize,
    srcInfo: &SkImageInfo,
    srcPixels: *const u8,
    srcRB: usize,
) -> Result<bool, super::SkRasterPipeline::MissingStage> {
    crate::SkASSERT!(@source b"src/core/SkConvertPixels.cpp\0".as_ptr(),266; @condition b"dstInfo.dimensions() == srcInfo.dimensions()\0".as_ptr(); (dstInfo.dimensions())==( srcInfo.dimensions()));
    crate::SkASSERT!(@source b"src/core/SkConvertPixels.cpp\0".as_ptr(),267; @condition b"SkImageInfoValidConversion(dstInfo, srcInfo)\0".as_ptr(); super::SkImageInfoPriv::SkImageInfoValidConversion(
        dstInfo, srcInfo
    ));
    let src_bpp = srcInfo.bytesPerPixel() as usize;
    let dst_bpp = dstInfo.bytesPerPixel() as usize;
    let srcStride = (srcRB / src_bpp) as i32;
    let dstStride = (dstRB / dst_bpp) as i32;
    if (srcStride as usize).wrapping_mul(src_bpp) != srcRB
        || (dstStride as usize).wrapping_mul(dst_bpp) != dstRB
    {
        return Ok(false);
    }
    if srcPixels == dstPixels.cast_const() && src_bpp != dst_bpp {
        return Ok(false);
    }
    let steps = SkColorSpaceXformSteps::new(
        srcInfo.colorSpace(),
        srcInfo.alphaType(),
        dstInfo.colorSpace(),
        dstInfo.alphaType(),
    );
    for function in [rect_memcpy, swizzle_or_premul, convert_to_alpha8] {
        if function(dstInfo, dstPixels, dstRB, srcInfo, srcPixels, srcRB, &steps) {
            return Ok(true);
        }
    }
    convert_with_pipeline(
        dstInfo, dstPixels, dstStride, srcInfo, srcPixels, srcStride, &steps,
    )?;
    Ok(true)
}
