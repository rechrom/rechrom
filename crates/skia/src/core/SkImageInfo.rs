use crate::include::core::SkSize::SkISize;
use crate::src::core::SkRect::SkIRect;
// Copyright 2010 Google Inc. BSD-style license.
use crate::include::core::SkAlphaType::*;
use crate::include::core::SkColorType::*;
// Source: include/core/SkImageInfo.h:52-53
// Actual native canonical=nullptr default; no invented alpha normalization.
pub fn SkColorTypeValidateAlphaType_default(
    color_type: SkColorType,
    alpha_type: SkAlphaType,
) -> bool {
    SkColorTypeValidateAlphaType(color_type, alpha_type, None)
}

// Source: include/core/SkImageInfo.h:68-97
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkYUVColorSpace {
    kJPEG_Full_SkYUVColorSpace,
    kRec601_Limited_SkYUVColorSpace,
    kRec709_Full_SkYUVColorSpace,
    kRec709_Limited_SkYUVColorSpace,
    kBT2020_8bit_Full_SkYUVColorSpace,
    kBT2020_8bit_Limited_SkYUVColorSpace,
    kBT2020_10bit_Full_SkYUVColorSpace,
    kBT2020_10bit_Limited_SkYUVColorSpace,
    kBT2020_12bit_Full_SkYUVColorSpace,
    kBT2020_12bit_Limited_SkYUVColorSpace,
    kBT2020_16bit_Full_SkYUVColorSpace,
    kBT2020_16bit_Limited_SkYUVColorSpace,
    kFCC_Full_SkYUVColorSpace,
    kFCC_Limited_SkYUVColorSpace,
    kSMPTE240_Full_SkYUVColorSpace,
    kSMPTE240_Limited_SkYUVColorSpace,
    kYDZDX_Full_SkYUVColorSpace,
    kYDZDX_Limited_SkYUVColorSpace,
    kGBR_Full_SkYUVColorSpace,
    kGBR_Limited_SkYUVColorSpace,
    kYCgCo_8bit_Full_SkYUVColorSpace,
    kYCgCo_8bit_Limited_SkYUVColorSpace,
    kYCgCo_10bit_Full_SkYUVColorSpace,
    kYCgCo_10bit_Limited_SkYUVColorSpace,
    kYCgCo_12bit_Full_SkYUVColorSpace,
    kYCgCo_12bit_Limited_SkYUVColorSpace,
    kYCgCo_16bit_Full_SkYUVColorSpace,
    kYCgCo_16bit_Limited_SkYUVColorSpace,
    kIdentity_SkYUVColorSpace,
}
// Source: include/core/SkImageInfo.h:99-99
// Rust enums cannot repeat discriminants; associated constants retain C++ aliases.
impl SkYUVColorSpace {
    pub const kLastEnum_SkYUVColorSpace: Self = Self::kIdentity_SkYUVColorSpace;
    // Source: include/core/SkImageInfo.h:102-106
    pub const kJPEG_SkYUVColorSpace: Self = Self::kJPEG_Full_SkYUVColorSpace;
    pub const kRec601_SkYUVColorSpace: Self = Self::kRec601_Limited_SkYUVColorSpace;
    pub const kRec709_SkYUVColorSpace: Self = Self::kRec709_Limited_SkYUVColorSpace;
    pub const kBT2020_SkYUVColorSpace: Self = Self::kBT2020_8bit_Limited_SkYUVColorSpace;
}
// Source: include/core/SkImageInfo.h:146-147
// Source: include/core/SkImageInfo.h:149-152
impl SkColorInfo {
    pub fn colorType(&self) -> SkColorType {
        self.fColorType
    }
    pub fn alphaType(&self) -> SkAlphaType {
        self.fAlphaType
    }
    pub fn isOpaque(&self) -> bool {
        SkAlphaTypeIsOpaque(self.fAlphaType) || SkColorTypeIsAlwaysOpaque(self.fColorType)
    }
}
// Source: include/core/SkImageInfo.h:199-202
// Arc preserves shared immutable color-space ownership; Option represents the nullable sk_sp.
// Source: include/core/SkImageInfo.h:117-117
pub struct SkColorInfo {
    fColorSpace: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    fColorType: SkColorType,
    fAlphaType: SkAlphaType,
}
// Source: include/core/SkImageInfo.h:222-222
impl Default for SkImageInfo {
    fn default() -> Self {
        Self {
            fColorInfo: SkColorInfo::default(),
            fDimensions: SkISize {
                fWidth: 0,
                fHeight: 0,
            },
        }
    }
}
// Source: include/core/SkImageInfo.h:255-260
// Rust gives each C++ constructor overload a distinct name and retains borrow versus move ownership.
impl SkImageInfo {
    pub fn MakeColorInfo(dimensions: SkISize, colorInfo: &SkColorInfo) -> Self {
        Self {
            fColorInfo: colorInfo.clone(),
            fDimensions: dimensions,
        }
    }
    pub fn MakeColorInfoOwned(dimensions: SkISize, colorInfo: SkColorInfo) -> Self {
        Self {
            fColorInfo: colorInfo,
            fDimensions: dimensions,
        }
    }
}
// Source: include/core/SkImageInfo.h:363-365
impl SkImageInfo {
    pub fn MakeEmpty() -> Self {
        Self::MakeUnknown(0, 0)
    }
}
// Source: include/core/SkImageInfo.h:371-371
// Source: include/core/SkImageInfo.h:377-377
// Source: include/core/SkImageInfo.h:379-379
// Source: include/core/SkImageInfo.h:381-381
impl SkImageInfo {
    pub fn width(&self) -> i32 {
        self.fDimensions.fWidth
    }
    pub fn height(&self) -> i32 {
        self.fDimensions.fHeight
    }
    pub fn colorType(&self) -> SkColorType {
        self.fColorInfo.colorType()
    }
    pub fn alphaType(&self) -> SkAlphaType {
        self.fColorInfo.alphaType()
    }
}
// Source: include/core/SkImageInfo.h:405-405
impl SkImageInfo {
    pub fn isEmpty(&self) -> bool {
        self.fDimensions.fWidth <= 0 || self.fDimensions.fHeight <= 0
    }
}
// Source: include/core/SkImageInfo.h:410-410
impl SkImageInfo {
    pub fn colorInfo(&self) -> &SkColorInfo {
        &self.fColorInfo
    }
}
// Source: include/core/SkImageInfo.h:421-421
impl SkImageInfo {
    pub fn isOpaque(&self) -> bool {
        self.fColorInfo.isOpaque()
    }
}
// Source: include/core/SkImageInfo.h:427-427
// Source: include/core/SkImageInfo.h:433-433
impl SkImageInfo {
    pub fn dimensions(&self) -> SkISize {
        self.fDimensions
    }
    pub fn bounds(&self) -> SkIRect {
        SkIRect::make_wh(self.width(), self.height())
    }
}
// Source: include/core/SkImageInfo.h:441-441
impl SkImageInfo {
    pub fn gammaCloseToSRGB(&self) -> bool {
        self.fColorInfo.gammaCloseToSRGB()
    }
}
// Source: include/core/SkImageInfo.h:450-452
impl SkImageInfo {
    pub fn makeWH(&self, newWidth: i32, newHeight: i32) -> Self {
        Self::MakeColorInfo(
            SkISize {
                fWidth: newWidth,
                fHeight: newHeight,
            },
            &self.fColorInfo,
        )
    }
}
// Source: include/core/SkImageInfo.h:460-462
impl SkImageInfo {
    pub fn makeDimensions(&self, newSize: SkISize) -> Self {
        Self::MakeColorInfo(newSize, &self.fColorInfo)
    }
}
// Source: include/core/SkImageInfo.h:472-474
impl SkImageInfo {
    pub fn makeAlphaType(&self, newAlphaType: SkAlphaType) -> Self {
        Self::MakeColorInfoOwned(
            self.fDimensions,
            self.fColorInfo.makeAlphaType(newAlphaType),
        )
    }
}
// Source: include/core/SkImageInfo.h:481-483
impl SkImageInfo {
    pub fn makeColorType(&self, newColorType: SkColorType) -> Self {
        Self::MakeColorInfoOwned(
            self.fDimensions,
            self.fColorInfo.makeColorType(newColorType),
        )
    }
}
// Source: include/core/SkImageInfo.h:498-498
// Source: include/core/SkImageInfo.h:505-505
impl SkImageInfo {
    pub fn bytesPerPixel(&self) -> i32 {
        self.fColorInfo.bytesPerPixel()
    }
    pub fn shiftPerPixel(&self) -> i32 {
        self.fColorInfo.shiftPerPixel()
    }
}
// Source: include/core/SkImageInfo.h:513-515
// Source: include/core/SkImageInfo.h:523-529
impl SkImageInfo {
    pub fn minRowBytes64(&self) -> u64 {
        (self.width() as i64 * self.bytesPerPixel() as i64) as u64
    }
    pub fn minRowBytes(&self) -> usize {
        let minRowBytes = self.minRowBytes64();
        if minRowBytes > i32::MAX as u64 {
            return 0;
        }
        minRowBytes as usize
    }
}
// Source: include/core/SkImageInfo.h:551-553
// Source: include/core/SkImageInfo.h:561-563
impl PartialEq for SkImageInfo {
    fn eq(&self, other: &Self) -> bool {
        self.fDimensions.fWidth == other.fDimensions.fWidth
            && self.fDimensions.fHeight == other.fDimensions.fHeight
            && self.fColorInfo == other.fColorInfo
    }
}
// Source: include/core/SkImageInfo.h:584-586
// Source: include/core/SkImageInfo.h:594-596
impl SkImageInfo {
    pub fn computeMinByteSize(&self) -> usize {
        self.computeByteSize(self.minRowBytes())
    }
    pub fn ByteSizeOverflowed(byteSize: usize) -> bool {
        byteSize == usize::MAX
    }
}
// Source: include/core/SkImageInfo.h:604-611
// Source: include/core/SkImageInfo.h:616-616
impl SkImageInfo {
    pub fn validRowBytes(&self, rowBytes: usize) -> bool {
        if (rowBytes as u64) < self.minRowBytes64() {
            return false;
        }
        let shift = self.shiftPerPixel();
        let alignedRowBytes = rowBytes >> shift << shift;
        alignedRowBytes == rowBytes
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
// Source: include/core/SkImageInfo.h:624-625
// Source: include/core/SkImageInfo.h:627-628
// Source: include/core/SkImageInfo.h:630-632
// Source: include/core/SkImageInfo.h:214-214
pub struct SkImageInfo {
    fColorInfo: SkColorInfo,
    fDimensions: SkISize,
}
// Source: src/core/SkImageInfo.cpp:16-49
// Source: src/core/SkImageInfo.cpp:51-53
pub fn SkColorTypeBytesPerPixel(ct: SkColorType) -> i32 {
    match ct {
        kUnknown_SkColorType => 0,
        kAlpha_8_SkColorType => 1,
        kRGB_565_SkColorType => 2,
        kARGB_4444_SkColorType => 2,
        kRGBA_8888_SkColorType => 4,
        kBGRA_8888_SkColorType => 4,
        kRGB_888x_SkColorType => 4,
        kRGBA_1010102_SkColorType => 4,
        kRGB_101010x_SkColorType => 4,
        kBGRA_1010102_SkColorType => 4,
        kBGR_101010x_SkColorType => 4,
        kBGR_101010x_XR_SkColorType => 4,
        kBGRA_10101010_XR_SkColorType => 8,
        kRGBA_10x6_SkColorType => 8,
        kGray_8_SkColorType => 1,
        kRGBA_F16Norm_SkColorType => 8,
        kRGBA_F16_SkColorType => 8,
        kRGB_F16F16F16x_SkColorType => 8,
        kRGBA_F32_SkColorType => 16,
        kR8G8_unorm_SkColorType => 2,
        kA16_unorm_SkColorType => 2,
        kR16_unorm_SkColorType => 2,
        kR16G16_unorm_SkColorType => 4,
        kA16_float_SkColorType => 2,
        kR16_float_SkColorType => 2,
        kR16G16_float_SkColorType => 4,
        kR16G16B16A16_unorm_SkColorType => 8,
        kSRGBA_8888_SkColorType => 4,
        kR8_unorm_SkColorType => 1,
    }
}
pub fn SkColorTypeIsAlwaysOpaque(ct: SkColorType) -> bool {
    (super::SkImageInfoPriv::SkColorTypeChannelFlags(ct)
        & super::SkColor::kAlpha_SkColorChannelFlag)
        == 0
}
// Source: src/core/SkImageInfo.cpp:57-73
// Source: src/core/SkImageInfo.cpp:75-90
// Source: src/core/SkImageInfo.cpp:94-95
pub fn SkYUVColorSpaceIsLimitedRange(cs: SkYUVColorSpace) -> bool {
    use SkYUVColorSpace::*;
    match cs {
        kRec601_Limited_SkYUVColorSpace
        | kRec709_Limited_SkYUVColorSpace
        | kBT2020_8bit_Limited_SkYUVColorSpace
        | kBT2020_10bit_Limited_SkYUVColorSpace
        | kBT2020_12bit_Limited_SkYUVColorSpace
        | kBT2020_16bit_Limited_SkYUVColorSpace
        | kFCC_Limited_SkYUVColorSpace
        | kSMPTE240_Limited_SkYUVColorSpace
        | kYDZDX_Limited_SkYUVColorSpace
        | kGBR_Limited_SkYUVColorSpace
        | kYCgCo_8bit_Limited_SkYUVColorSpace
        | kYCgCo_10bit_Limited_SkYUVColorSpace
        | kYCgCo_12bit_Limited_SkYUVColorSpace
        | kYCgCo_16bit_Limited_SkYUVColorSpace => true,
        kJPEG_Full_SkYUVColorSpace
        | kRec709_Full_SkYUVColorSpace
        | kBT2020_8bit_Full_SkYUVColorSpace
        | kBT2020_10bit_Full_SkYUVColorSpace
        | kBT2020_12bit_Full_SkYUVColorSpace
        | kBT2020_16bit_Full_SkYUVColorSpace
        | kFCC_Full_SkYUVColorSpace
        | kSMPTE240_Full_SkYUVColorSpace
        | kYDZDX_Full_SkYUVColorSpace
        | kGBR_Full_SkYUVColorSpace
        | kYCgCo_8bit_Full_SkYUVColorSpace
        | kYCgCo_10bit_Full_SkYUVColorSpace
        | kYCgCo_12bit_Full_SkYUVColorSpace
        | kYCgCo_16bit_Full_SkYUVColorSpace
        | kIdentity_SkYUVColorSpace => false,
    }
}

// Source: src/core/SkImageInfo.cpp:99-100
// Source: src/core/SkImageInfo.cpp:102-103
// Source: src/core/SkImageInfo.cpp:105-106
// Source: src/core/SkImageInfo.cpp:108-109
// Source: src/core/SkImageInfo.cpp:111-112
impl Default for SkColorInfo {
    fn default() -> Self {
        Self {
            fColorSpace: None,
            fColorType: kUnknown_SkColorType,
            fAlphaType: kUnknown_SkAlphaType,
        }
    }
}
// Source: include/core/SkImageInfo.h:139-139
// Source: src/core/SkImageInfo.cpp:106-106
impl SkColorInfo {
    // Native default memberwise move keeps source enum values while moving the
    // existing nullable color-space owner, without retaining or replacing it.
    pub fn take_from(source: &mut Self) -> Self {
        Self {
            fColorSpace: source.fColorSpace.take(),
            fColorType: source.fColorType,
            fAlphaType: source.fAlphaType,
        }
    }
}
impl SkColorInfo {
    pub fn new(
        ct: SkColorType,
        at: SkAlphaType,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self {
            fColorSpace: cs,
            fColorType: ct,
            fAlphaType: at,
        }
    }
    pub fn colorSpace(&self) -> Option<&super::SkColorSpace::SkColorSpace> {
        self.fColorSpace.as_deref()
    }
    pub fn refColorSpace(&self) -> Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>> {
        self.fColorSpace.clone()
    }
    // Rust typed integration: borrow the existing native sk_sp owner token.
    pub(crate) fn colorSpaceOwner(
        &self,
    ) -> Option<&std::sync::Arc<super::SkColorSpace::SkColorSpace>> {
        self.fColorSpace.as_ref()
    }
}
// Source: src/core/SkImageInfo.cpp:114-117
// Source: src/core/SkImageInfo.cpp:119-119
impl PartialEq for SkColorInfo {
    fn eq(&self, other: &Self) -> bool {
        self.fColorType == other.fColorType
            && self.fAlphaType == other.fAlphaType
            && super::SkColorSpace::SkColorSpace::Equals(self.colorSpace(), other.colorSpace())
    }
}
// Source: src/core/SkImageInfo.cpp:121-123
// Source: src/core/SkImageInfo.cpp:125-127
// Source: src/core/SkImageInfo.cpp:129-131
// Source: src/core/SkImageInfo.cpp:133-133
// Source: src/core/SkImageInfo.cpp:135-137
// Source: src/core/SkImageInfo.cpp:139-139
impl SkColorInfo {
    pub fn makeAlphaType(&self, newAlphaType: SkAlphaType) -> Self {
        Self::new(self.colorType(), newAlphaType, self.refColorSpace())
    }
    pub fn makeColorType(&self, newColorType: SkColorType) -> Self {
        Self::new(newColorType, self.alphaType(), self.refColorSpace())
    }
    pub fn makeColorSpace(
        &self,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::new(self.colorType(), self.alphaType(), cs)
    }
    pub fn bytesPerPixel(&self) -> i32 {
        SkColorTypeBytesPerPixel(self.fColorType)
    }
    pub fn gammaCloseToSRGB(&self) -> bool {
        self.fColorSpace
            .as_ref()
            .is_some_and(|cs| cs.gammaCloseToSRGB())
    }
    pub fn shiftPerPixel(&self) -> i32 {
        super::SkImageInfoPriv::SkColorTypeShiftPerPixel(self.fColorType)
    }
}
// Source: src/core/SkImageInfo.cpp:143-147
// Source: src/core/SkImageInfo.cpp:149-155
// Source: src/core/SkImageInfo.cpp:160-162
impl SkImageInfo {
    pub fn computeOffset(&self, x: i32, y: i32, rowBytes: usize) -> usize {
        crate::SkASSERTF!(@source b"src/core/SkImageInfo.cpp\0".as_ptr(),144; @condition b"x >= 0 && x < this->width()\0".as_ptr(); x>=0&&x<self.width(),"x=%d; width=%d\n",x as core::ffi::c_int,self.width() as core::ffi::c_int);
        crate::SkASSERTF!(@source b"src/core/SkImageInfo.cpp\0".as_ptr(),145; @condition b"y >= 0 && y < this->height()\0".as_ptr(); y>=0&&y<self.height(),"y=%d; height=%d\n",y as core::ffi::c_int,self.height() as core::ffi::c_int);
        super::SkImageInfoPriv::SkColorTypeComputeOffset(self.colorType(), x, y, rowBytes)
    }
    pub fn computeByteSize(&self, rowBytes: usize) -> usize {
        if self.height() == 0 {
            return 0;
        }
        // checked integer operations retain SkSafeMath's failure propagation and signed addInt limit.
        let Some(rows) = self.height().checked_sub(1) else {
            return usize::MAX;
        };
        let Some(prior_rows) = (rows as usize).checked_mul(rowBytes) else {
            return usize::MAX;
        };
        let Some(last_row) = (self.width() as usize).checked_mul(self.bytesPerPixel() as usize)
        else {
            return usize::MAX;
        };
        let Some(bytes) = prior_rows.checked_add(last_row) else {
            return usize::MAX;
        };
        if bytes <= i32::MAX as usize {
            bytes
        } else {
            usize::MAX
        }
    }
}
// Source: src/core/SkImageInfo.cpp:164-164
// Source: src/core/SkImageInfo.cpp:166-166
// Source: src/core/SkImageInfo.cpp:168-170
// Source: src/core/SkImageInfo.cpp:172-174
// Source: src/core/SkImageInfo.cpp:176-179
// Source: src/core/SkImageInfo.cpp:181-183
// Source: src/core/SkImageInfo.cpp:185-188
// Source: src/core/SkImageInfo.cpp:190-192
// Source: src/core/SkImageInfo.cpp:194-196
// Source: src/core/SkImageInfo.cpp:198-200
// Source: src/core/SkImageInfo.cpp:202-204
// Source: src/core/SkImageInfo.cpp:206-208
// Source: src/core/SkImageInfo.cpp:210-212
// Source: src/core/SkImageInfo.cpp:214-216
// Source: src/core/SkImageInfo.cpp:218-220
// Source: src/core/SkImageInfo.cpp:222-224
// Source: src/core/SkImageInfo.cpp:226-228
impl SkImageInfo {
    pub fn colorSpace(&self) -> Option<&super::SkColorSpace::SkColorSpace> {
        self.fColorInfo.colorSpace()
    }
    pub fn refColorSpace(&self) -> Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>> {
        self.fColorInfo.refColorSpace()
    }
    // Rust typed integration: preserve the ImageInfo color-space owner borrow.
    pub(crate) fn colorSpaceOwner(
        &self,
    ) -> Option<&std::sync::Arc<super::SkColorSpace::SkColorSpace>> {
        self.fColorInfo.colorSpaceOwner()
    }
    pub fn makeColorSpace(
        &self,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::MakeColorInfoOwned(self.fDimensions, self.fColorInfo.makeColorSpace(cs))
    }
    // Source: include/core/SkImageInfo.h:238-238
    pub fn Make(width: i32, height: i32, ct: SkColorType, at: SkAlphaType) -> Self {
        Self::MakeWithColorSpace(width, height, ct, at, None)
    }
    // Source: include/core/SkImageInfo.h:239-240
    pub fn MakeWithColorSpace(
        width: i32,
        height: i32,
        ct: SkColorType,
        at: SkAlphaType,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::MakeColorInfoOwned(
            SkISize {
                fWidth: width,
                fHeight: height,
            },
            SkColorInfo::new(ct, at, cs),
        )
    }
    // Source: include/core/SkImageInfo.h:241-241
    pub fn MakeDimensions(dimensions: SkISize, ct: SkColorType, at: SkAlphaType) -> Self {
        Self::MakeDimensionsWithColorSpace(dimensions, ct, at, None)
    }
    // Source: include/core/SkImageInfo.h:242-243
    pub fn MakeDimensionsWithColorSpace(
        dimensions: SkISize,
        ct: SkColorType,
        at: SkAlphaType,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::MakeColorInfoOwned(dimensions, SkColorInfo::new(ct, at, cs))
    }
    pub fn MakeN32(width: i32, height: i32, at: SkAlphaType) -> Self {
        Self::MakeN32WithColorSpace(width, height, at, None)
    }
    pub fn MakeN32WithColorSpace(
        width: i32,
        height: i32,
        at: SkAlphaType,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::MakeWithColorSpace(width, height, kN32_SkColorType, at, cs)
    }
    pub fn MakeS32(width: i32, height: i32, at: SkAlphaType) -> Self {
        Self::MakeN32WithColorSpace(
            width,
            height,
            at,
            Some(super::SkColorSpace::SkColorSpace::MakeSRGB()),
        )
    }
    pub fn MakeN32Premul(width: i32, height: i32) -> Self {
        Self::MakeN32PremulWithColorSpace(width, height, None)
    }
    pub fn MakeN32PremulWithColorSpace(
        width: i32,
        height: i32,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::MakeN32WithColorSpace(width, height, kPremul_SkAlphaType, cs)
    }
    pub fn MakeN32PremulDimensions(dimensions: SkISize) -> Self {
        Self::MakeN32PremulDimensionsWithColorSpace(dimensions, None)
    }
    pub fn MakeN32PremulDimensionsWithColorSpace(
        dimensions: SkISize,
        cs: Option<std::sync::Arc<super::SkColorSpace::SkColorSpace>>,
    ) -> Self {
        Self::MakeDimensionsWithColorSpace(dimensions, kN32_SkColorType, kPremul_SkAlphaType, cs)
    }
    pub fn MakeA8(width: i32, height: i32) -> Self {
        Self::Make(width, height, kAlpha_8_SkColorType, kPremul_SkAlphaType)
    }
    pub fn MakeA8Dimensions(dimensions: SkISize) -> Self {
        Self::MakeDimensions(dimensions, kAlpha_8_SkColorType, kPremul_SkAlphaType)
    }
    pub fn MakeUnknown(width: i32, height: i32) -> Self {
        Self::Make(width, height, kUnknown_SkColorType, kUnknown_SkAlphaType)
    }
}
// Source: src/core/SkImageInfo.cpp:231-236
#[cfg(sk_debug)]
impl SkImageInfo {
    // Source: include/core/SkImageInfo.h:621-621
    pub fn validate(&self) {
        crate::SkASSERT!(@source b"src/core/SkImageInfo.cpp\0".as_ptr(),232; @condition b"fDimensions.width() >= 0\0".as_ptr(); self.width()>=0);
        crate::SkASSERT!(@source b"src/core/SkImageInfo.cpp\0".as_ptr(),233; @condition b"fDimensions.height() >= 0\0".as_ptr(); self.height()>=0);
        crate::SkASSERT!(@source b"src/core/SkImageInfo.cpp\0".as_ptr(),234; @condition b"SkColorTypeIsValid(this->colorType())\0".as_ptr(); super::SkImageInfoPriv::SkColorTypeIsValid(self.colorType()as u32));
        crate::SkASSERT!(@source b"src/core/SkImageInfo.cpp\0".as_ptr(),235; @condition b"SkAlphaTypeIsValid(this->alphaType())\0".as_ptr(); super::SkImageInfoPriv::SkAlphaTypeIsValid(self.alphaType()as u32));
    }
}
// Source: src/core/SkImageInfo.cpp:239-288
pub fn SkColorTypeValidateAlphaType(
    colorType: SkColorType,
    mut alphaType: SkAlphaType,
    canonical: Option<&mut SkAlphaType>,
) -> bool {
    match colorType {
        kUnknown_SkColorType => {
            alphaType = kUnknown_SkAlphaType;
        }
        kAlpha_8_SkColorType | kA16_unorm_SkColorType | kA16_float_SkColorType => {
            if alphaType == kUnpremul_SkAlphaType {
                alphaType = kPremul_SkAlphaType;
            }
            if alphaType == kUnknown_SkAlphaType {
                return false;
            }
        }
        kARGB_4444_SkColorType
        | kRGBA_8888_SkColorType
        | kSRGBA_8888_SkColorType
        | kBGRA_8888_SkColorType
        | kRGBA_1010102_SkColorType
        | kBGRA_1010102_SkColorType
        | kRGBA_10x6_SkColorType
        | kRGBA_F16Norm_SkColorType
        | kRGBA_F16_SkColorType
        | kRGBA_F32_SkColorType
        | kBGRA_10101010_XR_SkColorType
        | kR16G16B16A16_unorm_SkColorType => {
            if alphaType == kUnknown_SkAlphaType {
                return false;
            }
        }
        kGray_8_SkColorType
        | kR8G8_unorm_SkColorType
        | kR16_unorm_SkColorType
        | kR16_float_SkColorType
        | kR16G16_unorm_SkColorType
        | kR16G16_float_SkColorType
        | kRGB_565_SkColorType
        | kRGB_888x_SkColorType
        | kRGB_101010x_SkColorType
        | kBGR_101010x_SkColorType
        | kBGR_101010x_XR_SkColorType
        | kRGB_F16F16F16x_SkColorType
        | kR8_unorm_SkColorType => {
            alphaType = kOpaque_SkAlphaType;
        }
    }
    if let Some(canonical) = canonical {
        *canonical = alphaType;
    }
    true
}

// Source: src/core/SkImageInfo.cpp:105-109
// Defaulted native special members operate in declaration order: the same
// color-space owner moves, while the scalar color/alpha values remain in source.
impl SkColorInfo {
    pub fn move_from(source: &mut Self) -> Self {
        let color_space = source.fColorSpace.take();
        let color_type = source.fColorType;
        let alpha_type = source.fAlphaType;
        Self {
            fColorSpace: color_space,
            fColorType: color_type,
            fAlphaType: alpha_type,
        }
    }
    pub fn assign_move(&mut self, source: &mut Self) {
        let old = core::mem::replace(&mut self.fColorSpace, source.fColorSpace.take());
        // Native sk_sp::reset publishes its replacement before releasing old.
        drop(old);
        self.fColorType = source.fColorType;
        self.fAlphaType = source.fAlphaType;
    }
}
impl Clone for SkColorInfo {
    fn clone(&self) -> Self {
        Self {
            fColorSpace: self.fColorSpace.clone(),
            fColorType: self.fColorType,
            fAlphaType: self.fAlphaType,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        let retained = source.fColorSpace.clone();
        let old = core::mem::replace(&mut self.fColorSpace, retained);
        drop(old);
        self.fColorType = source.fColorType;
        self.fAlphaType = source.fAlphaType;
    }
}
// Source: include/core/SkImageInfo.h:624-632
// Implicit class operations move the ColorInfo member and copy dimensions;
// moving does not reset source dimensions or its encoding scalar values.
impl SkImageInfo {
    pub fn move_from(source: &mut Self) -> Self {
        let color_info = SkColorInfo::move_from(&mut source.fColorInfo);
        let dimensions = source.fDimensions;
        Self {
            fColorInfo: color_info,
            fDimensions: dimensions,
        }
    }
    pub fn assign_move(&mut self, source: &mut Self) {
        self.fColorInfo.assign_move(&mut source.fColorInfo);
        self.fDimensions = source.fDimensions;
    }
}
impl Clone for SkImageInfo {
    fn clone(&self) -> Self {
        Self {
            fColorInfo: self.fColorInfo.clone(),
            fDimensions: self.fDimensions,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.fColorInfo.clone_from(&source.fColorInfo);
        self.fDimensions = source.fDimensions;
    }
}
