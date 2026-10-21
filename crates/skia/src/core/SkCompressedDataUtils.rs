// Copyright 2020 Google LLC. BSD-style license.
use super::SkColorData::{SkB16ToB32, SkG16ToG32, SkR16ToR32};
use super::SkColorPriv::{
    SkGetPackedA32, SkGetPackedB32, SkGetPackedG32, SkGetPackedR32, SkPackARGB32,
};
use super::SkScalar::SkScalarRoundToInt;
use super::{SkBitmap::SkBitmap, SkData::SkData, SkMipmap::SkMipmap};
use crate::include::core::{SkSize::SkISize, SkTextureCompressionType::SkTextureCompressionType};
use std::sync::Arc;
// Source: src/core/SkCompressedDataUtils.h:22-31
pub const fn SkTextureCompressionTypeIsOpaque(compression: SkTextureCompressionType) -> bool {
    match compression {
        SkTextureCompressionType::kNone => true,
        SkTextureCompressionType::kETC2_RGB8_UNORM => true,
        SkTextureCompressionType::kBC1_RGB8_UNORM => true,
        SkTextureCompressionType::kBC1_RGBA8_UNORM => false,
    }
}
// Source: src/core/SkCompressedDataUtils.h:34-49
// Header forward declarations/prototypes resolve to the existing typed owners
// and public implementation signatures below; Rust has no separate prototypes.
// Source: src/core/SkCompressedDataUtils.cpp:27-30
#[repr(C)]
#[derive(Clone, Copy)]
struct ETC1Block {
    fHigh: u32,
    fLow: u32,
}
// Source: src/core/SkCompressedDataUtils.cpp:32-33
const kFlipBit: u32 = 0x1;
const kDiffBit: u32 = 0x2;
// Source: src/core/SkCompressedDataUtils.cpp:35-38
fn extend_4To8bits(b: i32) -> i32 {
    let c = b & 0xf;
    (c << 4) | c
}
// Source: src/core/SkCompressedDataUtils.cpp:40-43
fn extend_5To8bits(b: i32) -> i32 {
    let c = b & 0x1f;
    (c << 3) | (c >> 2)
}
// Source: src/core/SkCompressedDataUtils.cpp:45-49
fn extend_5plus3To8Bits(base: i32, diff: i32) -> i32 {
    const LOOKUP: [i32; 8] = [0, 1, 2, 3, -4, -3, -2, -1];
    extend_5To8bits((0x1f & base) + LOOKUP[(0x7 & diff) as usize])
}
// Source: src/core/SkCompressedDataUtils.cpp:51-52
const kNumETC1ModifierTables: usize = 8;
const kNumETC1PixelIndices: usize = 4;
// Source: src/core/SkCompressedDataUtils.cpp:56-65
static kETC1ModifierTables: [[i32; kNumETC1PixelIndices]; kNumETC1ModifierTables] = [
    [2, 8, -2, -8],
    [5, 17, -5, -17],
    [9, 29, -9, -29],
    [13, 42, -13, -42],
    [18, 60, -18, -60],
    [24, 80, -24, -80],
    [33, 106, -33, -106],
    [47, 183, -47, -183],
];
// Source: src/core/SkCompressedDataUtils.cpp:67-69
fn num_4x4_blocks(size: i32) -> i32 {
    ((size + 3) & !3) >> 2
}
// Source: src/core/SkCompressedDataUtils.cpp:72-81
fn xy_to_subblock_index(x: i32, y: i32, flip: bool) -> usize {
    crate::SkASSERT!(@source b"src/core/SkCompressedDataUtils.cpp\0".as_ptr(),73; @condition b"x >= 0 && x < 4\0".as_ptr(); x>=0&&x<4);
    crate::SkASSERT!(@source b"src/core/SkCompressedDataUtils.cpp\0".as_ptr(),74; @condition b"y >= 0 && y < 4\0".as_ptr(); y>=0&&y<4);
    if flip {
        if y < 2 {
            0
        } else {
            1
        }
    } else {
        if x < 2 {
            0
        } else {
            1
        }
    }
}
// Source: src/core/SkCompressedDataUtils.cpp:83-85
#[derive(Clone, Copy)]
struct IColor {
    fR: i32,
    fG: i32,
    fB: i32,
}
// Source: src/core/SkCompressedDataUtils.cpp:87-93
fn add_delta_and_clamp(col: &IColor, delta: i32) -> u32 {
    let r8 = (col.fR + delta).clamp(0, 255);
    let g8 = (col.fG + delta).clamp(0, 255);
    let b8 = (col.fB + delta).clamp(0, 255);
    SkPackARGB32(0xff, r8 as u32, g8 as u32, b8 as u32)
}
// Source: src/core/SkCompressedDataUtils.cpp:95-161
// Raw block reads retain the original native byte layout; read_unaligned replaces
// C++ typed loads without requiring extra alignment on the borrowed byte pointer.
unsafe fn decompress_etc1(dimensions: SkISize, srcData: *const u8, dst: &mut SkBitmap) -> bool {
    unsafe {
        let srcBlocks = srcData.cast::<ETC1Block>();
        let numXBlocks = num_4x4_blocks(dimensions.width());
        let numYBlocks = num_4x4_blocks(dimensions.height());
        for y in 0..numYBlocks {
            for x in 0..numXBlocks {
                let curBlock1 = srcBlocks
                    .add((y * numXBlocks + x) as usize)
                    .read_unaligned();
                let high = curBlock1.fHigh.swap_bytes();
                let low = curBlock1.fLow.swap_bytes();
                let flipped = high & kFlipBit != 0;
                let differential = high & kDiffBit != 0;
                // Rust initializes the local array; both source branches fill every channel.
                let mut colors = [IColor {
                    fR: 0,
                    fG: 0,
                    fB: 0,
                }; 2];
                if differential {
                    colors[0].fR = extend_5To8bits((high >> 27) as i32);
                    colors[1].fR = extend_5plus3To8Bits((high >> 27) as i32, (high >> 24) as i32);
                    colors[0].fG = extend_5To8bits((high >> 19) as i32);
                    colors[1].fG = extend_5plus3To8Bits((high >> 19) as i32, (high >> 16) as i32);
                    colors[0].fB = extend_5To8bits((high >> 11) as i32);
                    colors[1].fB = extend_5plus3To8Bits((high >> 11) as i32, (high >> 8) as i32);
                } else {
                    colors[0].fR = extend_4To8bits((high >> 28) as i32);
                    colors[1].fR = extend_4To8bits((high >> 24) as i32);
                    colors[0].fG = extend_4To8bits((high >> 20) as i32);
                    colors[1].fG = extend_4To8bits((high >> 16) as i32);
                    colors[0].fB = extend_4To8bits((high >> 12) as i32);
                    colors[1].fB = extend_4To8bits((high >> 8) as i32);
                }
                let tableIndex0 = (high >> 5) & 0x7;
                let tableIndex1 = (high >> 2) & 0x7;
                let tables = [
                    &kETC1ModifierTables[tableIndex0 as usize],
                    &kETC1ModifierTables[tableIndex1 as usize],
                ];
                let mut baseShift = 0;
                let offsetX = 4 * x;
                let offsetY = 4 * y;
                for i in 0..4 {
                    for j in 0..4 {
                        if offsetX + j >= dst.width() || offsetY + i >= dst.height() {
                            continue;
                        }
                        let subBlockIndex = xy_to_subblock_index(j, i, flipped);
                        let pixelIndex = ((low >> (baseShift + j * 4)) & 0x1)
                            | ((low >> (baseShift + j * 4 + 15)) & 0x2);
                        crate::SkASSERT!(@source b"src/core/SkCompressedDataUtils.cpp\0".as_ptr(),149; @condition b"subBlockIndex == 0 || subBlockIndex == 1\0".as_ptr(); subBlockIndex==0||subBlockIndex==1);
                        crate::SkASSERT!(@source b"src/core/SkCompressedDataUtils.cpp\0".as_ptr(),150; @condition b"pixelIndex >= 0 && pixelIndex < 4\0".as_ptr(); pixelIndex<4);
                        let delta = tables[subBlockIndex][pixelIndex as usize];
                        *dst.getAddr32(offsetX + j, offsetY + i) =
                            add_delta_and_clamp(&colors[subBlockIndex], delta);
                    }
                    baseShift += 1;
                }
            }
        }
        true
    }
}
// Source: src/core/SkCompressedDataUtils.cpp:164-168
#[repr(C)]
#[derive(Clone, Copy)]
struct BC1Block {
    fColor0: u16,
    fColor1: u16,
    fIndices: u32,
}
// Source: src/core/SkCompressedDataUtils.cpp:170-176
fn from565(rgb565: u16) -> u32 {
    let r8 = SkR16ToR32(((rgb565 >> 11) & 0x1f) as u32) as u8;
    let g8 = SkG16ToG32(((rgb565 >> 5) & 0x3f) as u32) as u8;
    let b8 = SkB16ToB32((rgb565 & 0x1f) as u32) as u8;
    SkPackARGB32(0xff, r8 as u32, g8 as u32, b8 as u32)
}
// Source: src/core/SkCompressedDataUtils.cpp:179-187
fn lerp(t: f32, col0: u32, col1: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkCompressedDataUtils.cpp\0".as_ptr(),180; @condition b"SkGetPackedA32(col0) == 0xFF && SkGetPackedA32(col1) == 0xFF\0".as_ptr(); SkGetPackedA32(col0)==0xff&&SkGetPackedA32(col1)==0xff);
    let r8 = SkScalarRoundToInt(
        t * SkGetPackedR32(col0) as f32 + (1.0 - t) * SkGetPackedR32(col1) as f32,
    ) as u8;
    let g8 = SkScalarRoundToInt(
        t * SkGetPackedG32(col0) as f32 + (1.0 - t) * SkGetPackedG32(col1) as f32,
    ) as u8;
    let b8 = SkScalarRoundToInt(
        t * SkGetPackedB32(col0) as f32 + (1.0 - t) * SkGetPackedB32(col1) as f32,
    ) as u8;
    SkPackARGB32(0xff, r8 as u32, g8 as u32, b8 as u32)
}
// Source: src/core/SkCompressedDataUtils.cpp:189-235
unsafe fn decompress_bc1(
    dimensions: SkISize,
    srcData: *const u8,
    isOpaque: bool,
    dst: &mut SkBitmap,
) -> bool {
    unsafe {
        let srcBlocks = srcData.cast::<BC1Block>();
        let numXBlocks = num_4x4_blocks(dimensions.width());
        let numYBlocks = num_4x4_blocks(dimensions.height());
        let mut colors = [0u32; 4];
        for y in 0..numYBlocks {
            for x in 0..numXBlocks {
                let curBlock = srcBlocks
                    .add((y * numXBlocks + x) as usize)
                    .read_unaligned();
                colors[0] = from565(curBlock.fColor0);
                colors[1] = from565(curBlock.fColor1);
                if curBlock.fColor0 <= curBlock.fColor1 {
                    colors[2] = SkPackARGB32(
                        0xff,
                        (SkGetPackedR32(colors[0]) + SkGetPackedR32(colors[1])) >> 1,
                        (SkGetPackedG32(colors[0]) + SkGetPackedG32(colors[1])) >> 1,
                        (SkGetPackedB32(colors[0]) + SkGetPackedB32(colors[1])) >> 1,
                    );
                    colors[3] = SkPackARGB32(if isOpaque { 0xff } else { 0 }, 0, 0, 0);
                } else {
                    colors[2] = lerp(2.0 / 3.0, colors[0], colors[1]);
                    colors[3] = lerp(1.0 / 3.0, colors[0], colors[1]);
                }
                let mut shift = 0;
                let offsetX = 4 * x;
                let offsetY = 4 * y;
                for i in 0..4 {
                    for j in 0..4 {
                        // Native for-loop increment runs even when the bounds branch continues.
                        if offsetX + j >= dst.width() || offsetY + i >= dst.height() {
                            shift += 2;
                            continue;
                        }
                        let index = (curBlock.fIndices >> shift) & 0x3;
                        *dst.getAddr32(offsetX + j, offsetY + i) = colors[index as usize];
                        shift += 2;
                    }
                }
            }
        }
        true
    }
}
// Source: src/core/SkCompressedDataUtils.cpp:237-252
/// Safety: data contains all blocks addressed by dimensions; dst owns writable
/// N32 pixel storage covering the requested dimensions for the complete call.
pub unsafe fn SkDecompress(
    data: Arc<SkData>,
    dimensions: SkISize,
    compressionType: SkTextureCompressionType,
    dst: &mut SkBitmap,
) -> bool {
    unsafe {
        let bytes = data.bytes();
        match compressionType {
            SkTextureCompressionType::kNone => false,
            SkTextureCompressionType::kETC2_RGB8_UNORM => decompress_etc1(dimensions, bytes, dst),
            SkTextureCompressionType::kBC1_RGB8_UNORM => {
                decompress_bc1(dimensions, bytes, true, dst)
            }
            SkTextureCompressionType::kBC1_RGBA8_UNORM => {
                decompress_bc1(dimensions, bytes, false, dst)
            }
        }
    }
}
// Source: src/core/SkCompressedDataUtils.cpp:254-288
// Vec<usize> preserves TArray's owned contiguous offsets and append order.
pub fn SkCompressedDataSize(
    compression: SkTextureCompressionType,
    mut dimensions: SkISize,
    mut individualMipOffsets: Option<&mut Vec<usize>>,
    mipmapped: bool,
) -> usize {
    crate::SkASSERT!(@source b"src/core/SkCompressedDataUtils.cpp\0".as_ptr(),256; @condition b"!individualMipOffsets || individualMipOffsets->empty()\0".as_ptr(); individualMipOffsets.as_ref().is_none_or(|offsets|offsets.is_empty()));
    let mut numMipLevels = 1;
    if mipmapped {
        numMipLevels = SkMipmap::ComputeLevelCountForSize(dimensions) + 1;
    }
    let mut totalSize = 0usize;
    match compression {
        SkTextureCompressionType::kNone => {}
        SkTextureCompressionType::kETC2_RGB8_UNORM
        | SkTextureCompressionType::kBC1_RGB8_UNORM
        | SkTextureCompressionType::kBC1_RGBA8_UNORM => {
            for _i in 0..numMipLevels {
                let numBlocks =
                    num_4x4_blocks(dimensions.width()) * num_4x4_blocks(dimensions.height());
                if let Some(offsets) = individualMipOffsets.as_deref_mut() {
                    offsets.push(totalSize);
                }
                const _: () =
                    assert!(std::mem::size_of::<ETC1Block>() == std::mem::size_of::<BC1Block>());
                totalSize = totalSize.wrapping_add(
                    (numBlocks as usize).wrapping_mul(std::mem::size_of::<ETC1Block>()),
                );
                dimensions = SkISize::Make(
                    std::cmp::max(1, dimensions.width() / 2),
                    std::cmp::max(1, dimensions.height() / 2),
                );
            }
        }
    }
    totalSize
}
// Source: src/core/SkCompressedDataUtils.cpp:290-301
pub fn SkCompressedBlockSize(compression: SkTextureCompressionType) -> usize {
    match compression {
        SkTextureCompressionType::kNone => 0,
        SkTextureCompressionType::kETC2_RGB8_UNORM => std::mem::size_of::<ETC1Block>(),
        SkTextureCompressionType::kBC1_RGB8_UNORM | SkTextureCompressionType::kBC1_RGBA8_UNORM => {
            std::mem::size_of::<BC1Block>()
        }
    }
}
// Source: src/core/SkCompressedDataUtils.cpp:303-306
pub fn SkCompressedFormatDataSize(
    compressionType: SkTextureCompressionType,
    dimensions: SkISize,
    mipmapped: bool,
) -> usize {
    SkCompressedDataSize(compressionType, dimensions, None, mipmapped)
}
