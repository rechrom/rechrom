use crate::include::core::{SkAlphaType::SkAlphaType, SkColorType::SkColorType, SkSize::SkISize};
use crate::modules::skcms::src::skcms_public::skcms_PixelFormat;
use crate::src::core::{SkData::SkData, SkRect::SkIRect};
use std::sync::Arc;
// Source: src/codec/SkCodecPriv.h:24-28
#[cfg(SK_PRINT_CODEC_MESSAGES)]
#[macro_export]
macro_rules! SkCodecPrintf {(@source $file:expr,$line:expr;$format:expr $(,$argument:expr)* $(,)?)=>{$crate::SkDebugf!(@source $file,$line;$format $(,$argument)*);};}
#[cfg(not(SK_PRINT_CODEC_MESSAGES))]
#[macro_export]
macro_rules! SkCodecPrintf {
    ($($args:tt)*) => {{}};
}
// Source: src/codec/SkCodecPriv.h:29-103
pub mod SkCodecs {
    use super::*;
    use crate::modules::skcms::src::skcms_public::skcms_ICCProfile;
    // Source: src/codec/SkCodecPriv.h:32-32
    pub use crate::src::codec::SkCodec::SkCodecs::HasDecoder;
    // Source: src/codec/SkCodecPriv.h:67-72
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DataSpace {
        kRGB,
        kCMYK,
        kGray,
        kOther,
    }
    // Source: src/codec/SkCodecPriv.h:95-100
    // Private reverse destruction representation: retained opaque backing, data,
    // then profile POD. Actual pointers/owners are preserved; no extra owner.
    pub struct ColorProfile {
        // Source: src/codec/SkCodecPriv.h:100-100
        pub(crate) fRetainedData: Option<Arc<dyn core::any::Any>>,
        // Source: src/codec/SkCodecPriv.h:96-96
        pub(crate) fData: Option<Arc<SkData>>,
        // Source: src/codec/SkCodecPriv.h:95-95
        pub(crate) fProfile: skcms_ICCProfile,
    }
    impl ColorProfile {
        // Source: src/codec/SkCodecPriv.h:85-85
        pub fn profile(&self) -> *const skcms_ICCProfile {
            &self.fProfile
        }
        // Source: src/codec/SkCodecPriv.h:86-86
        pub fn data(&self) -> Option<Arc<SkData>> {
            self.fData.clone()
        }
    }
    // Out-of-line native factories/clone/color-space methods are implemented
    // once in src/codec/SkCodecColorProfile.rs; this module keeps header state
    // and real inline accessors only.
}
// Source: src/codec/SkCodecPriv.h:105-106
pub struct SkCodecPriv;
impl SkCodecPriv {
    // Source: src/codec/SkCodecPriv.h:107-110
    pub fn GetEncodedInfo(
        codec: &dyn super::SkCodec::SkCodecExt,
    ) -> &super::SkEncodedInfo::SkEncodedInfo {
        codec.getEncodedInfo()
    }
    // Source: src/codec/SkCodecPriv.h:117-117
    pub fn GetScaleFromSampleSize(sampleSize: i32) -> f32 {
        1.0 / (sampleSize as f32)
    }
    // Source: src/codec/SkCodecPriv.h:119-121
    pub fn IsValidSubset(subset: &SkIRect, imageDims: &SkISize) -> bool {
        SkIRect::MakeSize(imageDims).contains_rect(subset)
    }
    // Source: src/codec/SkCodecPriv.h:127-135
    pub fn GetSampledDimension(srcDimension: i32, sampleSize: i32) -> i32 {
        if sampleSize > srcDimension {
            return 1;
        }
        if sampleSize == 0 {
            return 0;
        }
        srcDimension / sampleSize
    }
    // Source: src/codec/SkCodecPriv.h:143-143
    pub fn GetStartCoord(sampleFactor: i32) -> i32 {
        sampleFactor / 2
    }
    // Source: src/codec/SkCodecPriv.h:153-153
    pub fn GetDstCoord(srcCoord: i32, sampleFactor: i32) -> i32 {
        srcCoord / sampleFactor
    }
    // Source: src/codec/SkCodecPriv.h:163-174
    pub fn IsCoordNecessary(srcCoord: i32, sampleFactor: i32, scaledDim: i32) -> bool {
        let startCoord = Self::GetStartCoord(sampleFactor);
        if srcCoord < startCoord || Self::GetDstCoord(srcCoord, sampleFactor) >= scaledDim {
            return false;
        }
        (srcCoord - startCoord) % sampleFactor == 0
    }
    // Source: src/codec/SkCodecPriv.h:176-191
    pub fn ValidAlpha(dstAlpha: SkAlphaType, srcIsOpaque: bool) -> bool {
        if SkAlphaType::kUnknown_SkAlphaType == dstAlpha {
            return false;
        }
        if srcIsOpaque {
            if SkAlphaType::kOpaque_SkAlphaType != dstAlpha {
                crate::SkCodecPrintf!(@source b"src/codec/SkCodecPriv.h\0".as_ptr(),26;b"Warning: an opaque image should be decoded as opaque - it is being decoded as non-opaque, which will draw slower\n\0".as_ptr());
            }
            return true;
        }
        dstAlpha != SkAlphaType::kOpaque_SkAlphaType
    }
    // Source: src/codec/SkCodecPriv.h:196-198
    pub fn GetColorPtr(colorTable: Option<&super::SkColorPalette::SkColorPalette>) -> *const u32 {
        colorTable.map_or(core::ptr::null(), |table| table.readColors())
    }
    // Source: src/codec/SkCodecPriv.h:203-205
    pub fn ComputeRowBytesPixelsPerByte(width: i32, pixelsPerByte: u32) -> usize {
        (width as u32)
            .wrapping_add(pixelsPerByte)
            .wrapping_sub(1)
            .wrapping_div(pixelsPerByte) as usize
    }
    // Source: src/codec/SkCodecPriv.h:210-212
    pub fn ComputeRowBytesBytesPerPixel(width: i32, bytesPerPixel: u32) -> usize {
        (width as u32).wrapping_mul(bytesPerPixel) as usize
    }
    // Source: src/codec/SkCodecPriv.h:217-227
    pub fn ComputeRowBytes(width: i32, bitsPerPixel: u32) -> usize {
        if bitsPerPixel < 16 {
            crate::SkASSERT!(@source b"src/codec/SkCodecPriv.h\0".as_ptr(),232;@condition b"0 == 8 % bitsPerPixel\0".as_ptr();0==8%bitsPerPixel);
            Self::ComputeRowBytesPixelsPerByte(width, 8 / bitsPerPixel)
        } else {
            crate::SkASSERT!(@source b"src/codec/SkCodecPriv.h\0".as_ptr(),236;@condition b"0 == bitsPerPixel % 8\0".as_ptr();0==bitsPerPixel%8);
            Self::ComputeRowBytesBytesPerPixel(width, bitsPerPixel / 8)
        }
    }
    // Source: src/codec/SkCodecPriv.h:233-233
    pub unsafe fn UnsafeGetByte(buffer: *const u8, i: u32) -> u8 {
        unsafe { *buffer.add(i as usize) }
    }
    // Source: src/codec/SkCodecPriv.h:239-247
    pub unsafe fn UnsafeGetShort(buffer: *const u8, i: u32) -> u16 {
        u16::from_le(unsafe { buffer.add(i as usize).cast::<u16>().read_unaligned() })
    }
    // Source: src/codec/SkCodecPriv.h:253-261
    pub unsafe fn UnsafeGetInt(buffer: *const u8, i: u32) -> u32 {
        u32::from_le(unsafe { buffer.add(i as usize).cast::<u32>().read_unaligned() })
    }
    // Source: src/codec/SkCodecPriv.h:269-277
    pub unsafe fn IsValidEndianMarker(data: *const u8, isLittleEndian: &mut bool) -> bool {
        let a = unsafe { *data };
        let b = unsafe { *data.add(1) };
        if (a != b'I' || b != b'I') && (a != b'M' || b != b'M') {
            return false;
        }
        *isLittleEndian = a == b'I';
        true
    }
    // Source: src/codec/SkCodecPriv.h:279-285
    pub unsafe fn GetEndianShort(data: *const u8, littleEndian: bool) -> u16 {
        let a = unsafe { *data } as u16;
        let b = unsafe { *data.add(1) } as u16;
        if littleEndian {
            (b << 8) | a
        } else {
            (a << 8) | b
        }
    }
    // Source: src/codec/SkCodecPriv.h:287-293
    pub unsafe fn GetEndianInt(data: *const u8, littleEndian: bool) -> u32 {
        let a = unsafe { *data } as u32;
        let b = unsafe { *data.add(1) } as u32;
        let c = unsafe { *data.add(2) } as u32;
        let d = unsafe { *data.add(3) } as u32;
        if littleEndian {
            (d << 24) | (c << 16) | (b << 8) | a
        } else {
            (a << 24) | (b << 16) | (c << 8) | d
        }
    }
    // Source: src/codec/SkCodecPriv.h:295-303
    pub fn PremultiplyARGBasRGBA(a: u32, mut r: u32, mut g: u32, mut b: u32) -> u32 {
        if a != 255 {
            r = crate::include::private::SkMath::SkMulDiv255Round(r, a);
            g = crate::include::private::SkMath::SkMulDiv255Round(g, a);
            b = crate::include::private::SkMath::SkMulDiv255Round(b, a);
        }
        crate::src::core::SkColorData::SkPackARGB_as_RGBA(a, r, g, b)
    }
    // Source: src/codec/SkCodecPriv.h:305-313
    pub fn PremultiplyARGBasBGRA(a: u32, mut r: u32, mut g: u32, mut b: u32) -> u32 {
        if a != 255 {
            r = crate::include::private::SkMath::SkMulDiv255Round(r, a);
            g = crate::include::private::SkMath::SkMulDiv255Round(g, a);
            b = crate::include::private::SkMath::SkMulDiv255Round(b, a);
        }
        crate::src::core::SkColorData::SkPackARGB_as_BGRA(a, r, g, b)
    }
    // Source: src/codec/SkCodecPriv.h:315-321
    pub fn IsRGBA(colorType: SkColorType) -> bool {
        #[cfg(SK_PMCOLOR_IS_RGBA)]
        {
            colorType != SkColorType::kBGRA_8888_SkColorType
        }
        #[cfg(not(SK_PMCOLOR_IS_RGBA))]
        {
            colorType == SkColorType::kRGBA_8888_SkColorType
        }
    }
    // Source: src/codec/SkCodecPriv.h:326-341
    pub fn ChoosePackColorProc(isPremul: bool, colorType: SkColorType) -> PackColorProc {
        let isRGBA = Self::IsRGBA(colorType);
        if isPremul {
            if isRGBA {
                Self::PremultiplyARGBasRGBA
            } else {
                Self::PremultiplyARGBasBGRA
            }
        } else {
            if isRGBA {
                crate::src::core::SkColorData::SkPackARGB_as_RGBA
            } else {
                crate::src::core::SkColorData::SkPackARGB_as_BGRA
            }
        }
    }
    // Source: src/codec/SkCodecPriv.h:343-346
    pub fn GetEncodedData(codec: &dyn super::SkCodec::SkCodec) -> Option<Arc<SkData>> {
        codec.getEncodedData()
    }
    // Source: src/codec/SkCodecPriv.h:113-115
    // Header API projection only. Actual .cpp body is staged in codec-format-section.rs.
    pub fn SelectXformFormat(
        colorType: SkColorType,
        forColorTable: bool,
        outFormat: &mut skcms_PixelFormat,
    ) -> bool {
        super::SkCodec::SkCodecPriv_SelectXformFormat(colorType, forColorTable, outFormat)
    }
}
// Source: src/codec/SkCodecPriv.h:324-324
pub type PackColorProc = fn(u32, u32, u32, u32) -> u32;
