// Same-basename native header and implementation share this canonical file.
mod declaration {
    // Copyright 2023 Google LLC. BSD-style license in LICENSE.
    use crate::src::core::{SkColor::SkColor4f, SkColorSpace::SkColorSpace};
    use std::sync::Arc;
    // Source: include/private/SkGainmapInfo.h:71-74.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum BaseImageType {
        kSDR,
        kHDR,
    }
    // Source: include/private/SkGainmapInfo.h:85-88.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Type {
        kDefault,
        kApple,
    }
    // Source: include/private/SkGainmapInfo.h:43-96.
    #[derive(Clone)]
    pub struct SkGainmapInfo {
        // Source: include/private/SkGainmapInfo.h:48-48.
        pub fGainmapRatioMin: SkColor4f,
        // Source: include/private/SkGainmapInfo.h:49-49.
        pub fGainmapRatioMax: SkColor4f,
        // Source: include/private/SkGainmapInfo.h:50-50.
        pub fGainmapGamma: SkColor4f,
        // Source: include/private/SkGainmapInfo.h:55-55.
        pub fEpsilonSdr: SkColor4f,
        // Source: include/private/SkGainmapInfo.h:56-56.
        pub fEpsilonHdr: SkColor4f,
        // Source: include/private/SkGainmapInfo.h:65-65.
        pub fDisplayRatioSdr: f32,
        // Source: include/private/SkGainmapInfo.h:66-66.
        pub fDisplayRatioHdr: f32,
        // Source: include/private/SkGainmapInfo.h:75-75.
        pub fBaseImageType: BaseImageType,
        // Source: include/private/SkGainmapInfo.h:89-89.
        pub fType: Type,
        // Source: include/private/SkGainmapInfo.h:95-95.
        pub fGainmapMathColorSpace: Option<Arc<SkColorSpace>>,
    }
    // Source: include/private/SkGainmapInfo.h:48-96. C++ implicit constructor applies these exact member defaults.
    impl Default for SkGainmapInfo {
        fn default() -> Self {
            Self {
                fGainmapRatioMin: SkColor4f {
                    fR: 1.0,
                    fG: 1.0,
                    fB: 1.0,
                    fA: 1.0,
                },
                fGainmapRatioMax: SkColor4f {
                    fR: 2.0,
                    fG: 2.0,
                    fB: 2.0,
                    fA: 1.0,
                },
                fGainmapGamma: SkColor4f {
                    fR: 1.0,
                    fG: 1.0,
                    fB: 1.0,
                    fA: 1.0,
                },
                fEpsilonSdr: SkColor4f {
                    fR: 0.0,
                    fG: 0.0,
                    fB: 0.0,
                    fA: 1.0,
                },
                fEpsilonHdr: SkColor4f {
                    fR: 0.0,
                    fG: 0.0,
                    fB: 0.0,
                    fA: 1.0,
                },
                fDisplayRatioSdr: 1.0,
                fDisplayRatioHdr: 2.0,
                fBaseImageType: BaseImageType::kSDR,
                fType: Type::kDefault,
                fGainmapMathColorSpace: None,
            }
        }
    }
    // Source: include/private/SkGainmapInfo.h:127-136.
    impl PartialEq for SkGainmapInfo {
        fn eq(&self, other: &Self) -> bool {
            self.fGainmapRatioMin == other.fGainmapRatioMin
                && self.fGainmapRatioMax == other.fGainmapRatioMax
                && self.fGainmapGamma == other.fGainmapGamma
                && self.fEpsilonSdr == other.fEpsilonSdr
                && self.fEpsilonHdr == other.fEpsilonHdr
                && self.fDisplayRatioSdr == other.fDisplayRatioSdr
                && self.fDisplayRatioHdr == other.fDisplayRatioHdr
                && self.fBaseImageType == other.fBaseImageType
                && self.fType == other.fType
                && SkColorSpace::Equals(
                    self.fGainmapMathColorSpace.as_deref(),
                    other.fGainmapMathColorSpace.as_deref(),
                )
        }
        // Source: include/private/SkGainmapInfo.h:137-137
        fn ne(&self, other: &Self) -> bool {
            !self.eq(other)
        }
    }
}
pub use declaration::*;
// Copyright 2024 Google LLC. BSD-style license in LICENSE.
use crate::src::core::{
    SkColor::SkColor4f,
    SkColorSpace::SkColorSpace,
    SkData::SkData,
    SkStream::{SkDynamicMemoryWStream, SkMemoryStream, SkStream, SkWStream},
    SkStreamPriv,
};
use core::{ffi::c_longlong, pin::Pin};
use std::sync::Arc;
unsafe extern "C" {
    fn llround(x: f64) -> c_longlong;
}
// Source: src/codec/SkGainmapInfo.cpp:22-25.
const kIsMultiChannelMask: u8 = 1u8 << 7;
const kUseBaseColourSpaceMask: u8 = 1u8 << 6;
// Source: src/codec/SkGainmapInfo.cpp:27-36.
fn write_rational_be(mut s: Pin<&mut SkDynamicMemoryWStream>, x: f32) {
    let mut denominator: u32 = 0x10000000;
    if x.abs() > 1.0 {
        denominator = 0x1000;
    }
    let numerator = unsafe { llround(x as f64 * denominator as f64) } as i32;
    SkStreamPriv::WriteS32BE(s.as_mut(), numerator);
    SkStreamPriv::WriteU32BE(s.as_mut(), denominator);
}
// Source: src/codec/SkGainmapInfo.cpp:38-47.
fn write_positive_rational_be(mut s: Pin<&mut SkDynamicMemoryWStream>, x: f32) {
    let mut denominator: u32 = 0x10000000;
    if x > 1.0 {
        denominator = 0x1000;
    }
    let numerator = unsafe { llround(x as f64 * denominator as f64) } as u32;
    SkStreamPriv::WriteU32BE(s.as_mut(), numerator);
    SkStreamPriv::WriteU32BE(s.as_mut(), denominator);
}
// Source: src/codec/SkGainmapInfo.cpp:49-60.
fn read_rational_be(mut s: Pin<&mut dyn SkStream>, value: &mut f32) -> bool {
    let mut numerator = 0i32;
    let mut denominator = 0u32;
    if !SkStreamPriv::ReadS32BE(s.as_mut(), &mut numerator) {
        return false;
    }
    if !SkStreamPriv::ReadU32BE(s.as_mut(), &mut denominator) {
        return false;
    }
    *value = (numerator as f64 / denominator as f64) as f32;
    true
}
// Source: src/codec/SkGainmapInfo.cpp:62-73.
fn read_positive_rational_be(mut s: Pin<&mut dyn SkStream>, value: &mut f32) -> bool {
    let mut numerator = 0u32;
    let mut denominator = 0u32;
    if !SkStreamPriv::ReadU32BE(s.as_mut(), &mut numerator) {
        return false;
    }
    if !SkStreamPriv::ReadU32BE(s.as_mut(), &mut denominator) {
        return false;
    }
    *value = (numerator as f64 / denominator as f64) as f32;
    true
}
// Source: src/codec/SkGainmapInfo.cpp:75-95.
fn read_iso_gainmap_version(mut s: Pin<&mut dyn SkStream>) -> bool {
    let mut minimum_version = 0u16;
    if !SkStreamPriv::ReadU16BE(s.as_mut(), &mut minimum_version) {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),79; b"Failed to read ISO 21496-1 minimum version.\n\0".as_ptr());
        return false;
    }
    if minimum_version != 0 {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),83; b"Unsupported ISO 21496-1 minimum version.\n\0".as_ptr());
        return false;
    }
    let mut writer_version = 0u16;
    if !SkStreamPriv::ReadU16BE(s.as_mut(), &mut writer_version) {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),90; b"Failed to read ISO 21496-1 version.\n\0".as_ptr());
        return false;
    }
    true
}
// Source: src/codec/SkGainmapInfo.cpp:97-182.
fn read_iso_gainmap_info(mut s: Pin<&mut dyn SkStream>, info: &mut SkGainmapInfo) -> bool {
    if !read_iso_gainmap_version(s.as_mut()) {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),99; b"Failed to read ISO 21496-1 version.\n\0".as_ptr());
        return false;
    }
    let mut flags = 0u8;
    if !s.as_mut().readU8(&mut flags) {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),105; b"Failed to read ISO 21496-1 flags.\n\0".as_ptr());
        return false;
    }
    let isMultiChannel = flags & kIsMultiChannelMask != 0;
    let useBaseColourSpace = flags & kUseBaseColourSpaceMask != 0;
    let mut baseHdrHeadroom = 0.0;
    if !read_positive_rational_be(s.as_mut(), &mut baseHdrHeadroom) {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),113; b"Failed to read ISO 21496-1 base HDR headroom.\n\0".as_ptr());
        return false;
    }
    let mut altrHdrHeadroom = 0.0;
    if !read_positive_rational_be(s.as_mut(), &mut altrHdrHeadroom) {
        crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),118; b"Failed to read ISO 21496-1 altr HDR headroom.\n\0".as_ptr());
        return false;
    }
    let mut gainMapMin = [0.0f32; 3];
    let mut gainMapMax = [0.0f32; 3];
    let mut gamma = [0.0f32; 3];
    let mut baseOffset = [0.0f32; 3];
    let mut altrOffset = [0.0f32; 3];
    let channelCount = if isMultiChannel { 3 } else { 1 };
    for i in 0..channelCount {
        if !read_rational_be(s.as_mut(), &mut gainMapMin[i]) {
            crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),131; b"Failed to read ISO 21496-1 gainmap minimum.\n\0".as_ptr());
            return false;
        }
        if !read_rational_be(s.as_mut(), &mut gainMapMax[i]) {
            crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),135; b"Failed to read ISO 21496-1 gainmap maximum.\n\0".as_ptr());
            return false;
        }
        if !read_positive_rational_be(s.as_mut(), &mut gamma[i]) {
            crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),139; b"Failed to read ISO 21496-1 gamma.\n\0".as_ptr());
            return false;
        }
        if !read_rational_be(s.as_mut(), &mut baseOffset[i]) {
            crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),143; b"Failed to read ISO 21496-1 base offset.\n\0".as_ptr());
            return false;
        }
        if !read_rational_be(s.as_mut(), &mut altrOffset[i]) {
            crate::SkCodecPrintf!(@source b"src/codec/SkGainmapInfo.cpp\0".as_ptr(),147; b"Failed to read ISO 21496-1 altr offset.\n\0".as_ptr());
            return false;
        }
    }
    *info = SkGainmapInfo::default();
    if !useBaseColourSpace {
        info.fGainmapMathColorSpace = Some(SkColorSpace::MakeSRGB());
    }
    if baseHdrHeadroom < altrHdrHeadroom {
        info.fBaseImageType = BaseImageType::kSDR;
        info.fDisplayRatioSdr = baseHdrHeadroom.exp2();
        info.fDisplayRatioHdr = altrHdrHeadroom.exp2();
    } else {
        info.fBaseImageType = BaseImageType::kHDR;
        info.fDisplayRatioHdr = baseHdrHeadroom.exp2();
        info.fDisplayRatioSdr = altrHdrHeadroom.exp2();
    }
    for i in 0..3 {
        let j = if i >= channelCount { 0 } else { i };
        info.fGainmapRatioMin[i] = gainMapMin[j].exp2();
        info.fGainmapRatioMax[i] = gainMapMax[j].exp2();
        info.fGainmapGamma[i] = 1.0 / gamma[j];
        match info.fBaseImageType {
            BaseImageType::kSDR => {
                info.fEpsilonSdr[i] = baseOffset[j];
                info.fEpsilonHdr[i] = altrOffset[j];
            }
            BaseImageType::kHDR => {
                info.fEpsilonHdr[i] = baseOffset[j];
                info.fEpsilonSdr[i] = altrOffset[j];
            }
        }
    }
    true
}
impl SkGainmapInfo {
    // Source: src/codec/SkGainmapInfo.cpp:184-195.
    pub fn isUltraHDRv1Compatible(&self) -> bool {
        if self.fBaseImageType == BaseImageType::kHDR {
            return false;
        }
        if self.fGainmapMathColorSpace.is_some() {
            return false;
        }
        true
    }
    // Source: src/codec/SkGainmapInfo.cpp:197-203.
    pub fn ParseVersion(data: Option<&SkData>) -> bool {
        let Some(data) = data else {
            return false;
        };
        let mut s = unsafe { SkMemoryStream::MakeDirect(data.data().cast(), data.size()) };
        read_iso_gainmap_version(s.as_mut())
    }
    // Source: src/codec/SkGainmapInfo.cpp:205-211.
    pub fn Parse(data: Option<&SkData>, info: &mut Self) -> bool {
        let Some(data) = data else {
            return false;
        };
        let mut s = unsafe { SkMemoryStream::MakeDirect(data.data().cast(), data.size()) };
        read_iso_gainmap_info(s.as_mut(), info)
    }
    // Source: src/codec/SkGainmapInfo.cpp:213-218.
    pub fn SerializeVersion() -> Arc<SkData> {
        let mut s = core::pin::pin!(SkDynamicMemoryWStream::new());
        SkStreamPriv::WriteU16BE(s.as_mut(), 0);
        SkStreamPriv::WriteU16BE(s.as_mut(), 0);
        s.as_mut().detachAsData()
    }
}
// Source: src/codec/SkGainmapInfo.cpp:220-220.
fn is_single_channel(c: SkColor4f) -> bool {
    c.fR == c.fG && c.fG == c.fB
}
impl SkGainmapInfo {
    // Source: src/codec/SkGainmapInfo.cpp:222-271.
    pub fn serialize(&self) -> Arc<SkData> {
        let mut s = core::pin::pin!(SkDynamicMemoryWStream::new());
        SkStreamPriv::WriteU16BE(s.as_mut(), 0);
        SkStreamPriv::WriteU16BE(s.as_mut(), 0);
        let all_single_channel = is_single_channel(self.fGainmapRatioMin)
            && is_single_channel(self.fGainmapRatioMax)
            && is_single_channel(self.fGainmapGamma)
            && is_single_channel(self.fEpsilonSdr)
            && is_single_channel(self.fEpsilonHdr);
        let mut flags = 0u8;
        if self.fGainmapMathColorSpace.is_none() {
            flags |= kUseBaseColourSpaceMask;
        }
        if !all_single_channel {
            flags |= kIsMultiChannelMask;
        }
        s.as_mut().write8(flags as u32);
        match self.fBaseImageType {
            BaseImageType::kSDR => {
                write_positive_rational_be(s.as_mut(), self.fDisplayRatioSdr.log2());
                write_positive_rational_be(s.as_mut(), self.fDisplayRatioHdr.log2());
            }
            BaseImageType::kHDR => {
                write_positive_rational_be(s.as_mut(), self.fDisplayRatioHdr.log2());
                write_positive_rational_be(s.as_mut(), self.fDisplayRatioSdr.log2());
            }
        }
        for i in 0..if all_single_channel { 1 } else { 3 } {
            write_rational_be(s.as_mut(), self.fGainmapRatioMin[i].log2());
            write_rational_be(s.as_mut(), self.fGainmapRatioMax[i].log2());
            write_positive_rational_be(s.as_mut(), 1.0 / self.fGainmapGamma[i]);
            match self.fBaseImageType {
                BaseImageType::kSDR => {
                    write_rational_be(s.as_mut(), self.fEpsilonSdr[i]);
                    write_rational_be(s.as_mut(), self.fEpsilonHdr[i]);
                }
                BaseImageType::kHDR => {
                    write_rational_be(s.as_mut(), self.fEpsilonHdr[i]);
                    write_rational_be(s.as_mut(), self.fEpsilonSdr[i]);
                }
            }
        }
        s.as_mut().detachAsData()
    }
}
