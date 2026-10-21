// Copyright 2020 Google LLC. BSD-style license. UNVERIFIED native translation.
use super::{SkColor::*, SkImageInfo::SkYUVColorSpace, SkMatrix::SkMatrix, SkSafeMath::SkSafeMath};
use crate::include::{
    codec::SkEncodedOrigin::{
        SkEncodedOrigin, SkEncodedOriginToMatrix, SkEncodedOriginToMatrixInverse,
    },
    core::SkSize::SkISize,
};
// Source: include/core/SkYUVAInfo.h:28-29
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum YUVAChannels {
    kY,
    kU,
    kV,
    kA,
}
impl YUVAChannels {
    pub const kLast: Self = Self::kA;
}
pub const kYUVAChannelCount: usize = 4;
// Source: include/core/SkYUVAInfo.h:31-32
// Actual definition is in the separately owned native SkYUVAInfoLocation.h.
// Canonical location values and AreValidLocations retain their actual type.
pub use super::SkYUVAInfoLocation::YUVALocation;
pub type YUVALocations = [YUVALocation; kYUVAChannelCount];
// Source: include/core/SkYUVAInfo.h:47-65
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaneConfig {
    #[default]
    kUnknown,
    kY_U_V,
    kY_V_U,
    kY_UV,
    kY_VU,
    kYUV,
    kUYV,
    kY_U_V_A,
    kY_V_U_A,
    kY_UV_A,
    kY_VU_A,
    kYUVA,
    kUYVA,
}
impl PlaneConfig {
    pub const kLast: Self = Self::kUYVA;
}
// Source: include/core/SkYUVAInfo.h:73-84
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Subsampling {
    #[default]
    kUnknown,
    k444,
    k422,
    k420,
    k440,
    k411,
    k410,
}
impl Subsampling {
    pub const kLast: Self = Self::k410;
}
// Source: include/core/SkYUVAInfo.h:91-96
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Siting {
    #[default]
    kCentered,
}
// Source: include/core/SkYUVAInfo.h:98-98
pub const kMaxPlanes: usize = 4;
// Source: include/core/SkYUVAInfo.h:26-26
// Source: include/core/SkYUVAInfo.h:239-253
#[derive(Clone, Copy)]
pub struct SkYUVAInfo {
    fDimensions: SkISize,
    fPlaneConfig: PlaneConfig,
    fSubsampling: Subsampling,
    fYUVColorSpace: SkYUVColorSpace,
    fOrigin: SkEncodedOrigin,
    fSitingX: Siting,
    fSitingY: Siting,
}
// Source: include/core/SkYUVAInfo.h:143-144
impl Default for SkYUVAInfo {
    fn default() -> Self {
        Self {
            fDimensions: SkISize::Make(0, 0),
            fPlaneConfig: PlaneConfig::kUnknown,
            fSubsampling: Subsampling::kUnknown,
            fYUVColorSpace: SkYUVColorSpace::kIdentity_SkYUVColorSpace,
            fOrigin: SkEncodedOrigin::kTopLeft_SkEncodedOrigin,
            fSitingX: Siting::kCentered,
            fSitingY: Siting::kCentered,
        }
    }
}
impl SkYUVAInfo {
    // Lexical bridges for C++ class-scoped constants, no second stored value.
    pub const kMaxPlanes: usize = kMaxPlanes;
    pub const kYUVAChannelCount: usize = kYUVAChannelCount;
    // Source: include/core/SkYUVAInfo.h:160-161
    pub fn planeConfig(&self) -> PlaneConfig {
        self.fPlaneConfig
    }
    pub fn subsampling(&self) -> Subsampling {
        self.fSubsampling
    }
    // Source: include/core/SkYUVAInfo.h:163-165
    pub fn planeSubsamplingFactors(&self, i: i32) -> (i32, i32) {
        Self::PlaneSubsamplingFactors(self.fPlaneConfig, self.fSubsampling, i)
    }
    // Source: include/core/SkYUVAInfo.h:171-179
    pub fn dimensions(&self) -> SkISize {
        self.fDimensions
    }
    pub fn width(&self) -> i32 {
        self.fDimensions.width()
    }
    pub fn height(&self) -> i32 {
        self.fDimensions.height()
    }
    pub fn yuvColorSpace(&self) -> SkYUVColorSpace {
        self.fYUVColorSpace
    }
    pub fn sitingX(&self) -> Siting {
        self.fSitingX
    }
    pub fn sitingY(&self) -> Siting {
        self.fSitingY
    }
    pub fn origin(&self) -> SkEncodedOrigin {
        self.fOrigin
    }
    // Source: include/core/SkYUVAInfo.h:181-183
    pub fn originMatrix(&self) -> SkMatrix {
        SkEncodedOriginToMatrix(self.fOrigin, self.width(), self.height())
    }
    // Source: include/core/SkYUVAInfo.h:184-186
    pub fn inverseOriginMatrix(&self) -> SkMatrix {
        SkEncodedOriginToMatrixInverse(self.fOrigin, self.width(), self.height())
    }
    // Source: include/core/SkYUVAInfo.h:188-188
    pub fn hasAlpha(&self) -> bool {
        Self::HasAlpha(self.fPlaneConfig)
    }
    // Source: include/core/SkYUVAInfo.h:195-197
    pub fn planeDimensions(&self, out: &mut [SkISize; kMaxPlanes]) -> i32 {
        Self::PlaneDimensions(
            self.fDimensions,
            self.fPlaneConfig,
            self.fSubsampling,
            self.fOrigin,
            out,
        )
    }
    // Source: include/core/SkYUVAInfo.h:207-209
    pub fn numPlanes(&self) -> i32 {
        Self::NumPlanes(self.fPlaneConfig)
    }
    pub fn numChannelsInPlane(&self, i: i32) -> i32 {
        Self::NumChannelsInPlane(self.fPlaneConfig, i)
    }
    // Source: include/core/SkYUVAInfo.h:234-234
    fn header_ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
    // Source: include/core/SkYUVAInfo.h:236-236
    pub fn isValid(&self) -> bool {
        self.fPlaneConfig != PlaneConfig::kUnknown
    }
    // Source: include/core/SkYUVAInfo.h:256-273
    pub const fn NumPlanes(p: PlaneConfig) -> i32 {
        use PlaneConfig::*;
        match p {
            kUnknown => 0,
            kY_U_V | kY_V_U => 3,
            kY_UV | kY_VU => 2,
            kYUV | kUYV => 1,
            kY_U_V_A | kY_V_U_A => 4,
            kY_UV_A | kY_VU_A => 3,
            kYUVA | kUYVA => 1,
        }
    }
    // Source: include/core/SkYUVAInfo.h:275-309
    pub const fn NumChannelsInPlane(p: PlaneConfig, i: i32) -> i32 {
        use PlaneConfig::*;
        match p {
            kUnknown => 0,
            kY_U_V | kY_V_U => {
                if i >= 0 && i < 3 {
                    1
                } else {
                    0
                }
            }
            kY_UV | kY_VU => match i {
                0 => 1,
                1 => 2,
                _ => 0,
            },
            kYUV | kUYV => {
                if i == 0 {
                    3
                } else {
                    0
                }
            }
            kY_U_V_A | kY_V_U_A => {
                if i >= 0 && i < 4 {
                    1
                } else {
                    0
                }
            }
            kY_UV_A | kY_VU_A => match i {
                0 | 2 => 1,
                1 => 2,
                _ => 0,
            },
            kYUVA | kUYVA => {
                if i == 0 {
                    4
                } else {
                    0
                }
            }
        }
    }
}
// Source: src/core/SkYUVAInfo.cpp:16-27
fn is_plane_config_compatible_with_subsampling(p: PlaneConfig, s: Subsampling) -> bool {
    if p == PlaneConfig::kUnknown || s == Subsampling::kUnknown {
        return false;
    }
    s == Subsampling::k444
        || !matches!(
            p,
            PlaneConfig::kYUV | PlaneConfig::kYUVA | PlaneConfig::kUYV | PlaneConfig::kUYVA
        )
}
impl SkYUVAInfo {
    // Source: src/core/SkYUVAInfo.cpp:29-40
    pub fn SubsamplingFactors(s: Subsampling) -> (i32, i32) {
        use Subsampling::*;
        match s {
            kUnknown => (0, 0),
            k444 => (1, 1),
            k422 => (2, 1),
            k420 => (2, 2),
            k440 => (1, 2),
            k411 => (4, 1),
            k410 => (4, 2),
        }
    }
    // Source: src/core/SkYUVAInfo.cpp:42-75
    pub fn PlaneSubsamplingFactors(p: PlaneConfig, s: Subsampling, i: i32) -> (i32, i32) {
        if !is_plane_config_compatible_with_subsampling(p, s) || i < 0 || i > Self::NumPlanes(p) {
            return (0, 0);
        }
        use PlaneConfig::*;
        let sampled = match p {
            kUnknown => crate::SkUNREACHABLE!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),52;),
            kY_U_V | kY_V_U | kY_U_V_A | kY_V_U_A => i == 1 || i == 2,
            kY_UV | kY_VU | kY_UV_A | kY_VU_A => i == 1,
            kYUV | kUYV | kYUVA | kUYVA => false,
        };
        if sampled {
            Self::SubsamplingFactors(s)
        } else {
            (1, 1)
        }
    }
    // Source: src/core/SkYUVAInfo.cpp:77-142
    pub fn PlaneDimensions(
        dim: SkISize,
        p: PlaneConfig,
        s: Subsampling,
        origin: SkEncodedOrigin,
        out: &mut [SkISize; kMaxPlanes],
    ) -> i32 {
        out.fill(SkISize::Make(0, 0));
        if !is_plane_config_compatible_with_subsampling(p, s) {
            return 0;
        }
        let mut w = dim.width();
        let mut h = dim.height();
        if origin as i32 >= 5 {
            core::mem::swap(&mut w, &mut h);
        }
        let down2 = |x: i32| (x + 1) / 2;
        let down4 = |x: i32| (x + 3) / 4;
        let uv = match s {
            Subsampling::kUnknown => {
                crate::SkUNREACHABLE!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),97;)
            }
            Subsampling::k444 => SkISize::Make(w, h),
            Subsampling::k422 => SkISize::Make(down2(w), h),
            Subsampling::k420 => SkISize::Make(down2(w), down2(h)),
            Subsampling::k440 => SkISize::Make(w, down2(h)),
            Subsampling::k411 => SkISize::Make(down4(w), h),
            Subsampling::k410 => SkISize::Make(down4(w), down2(h)),
        };
        use PlaneConfig::*;
        match p {
            kUnknown => crate::SkUNREACHABLE!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),107;),
            kY_U_V | kY_V_U => {
                out[0] = SkISize::Make(w, h);
                out[1] = uv;
                out[2] = uv;
                3
            }
            kY_UV | kY_VU => {
                out[0] = SkISize::Make(w, h);
                out[1] = uv;
                2
            }
            kY_U_V_A | kY_V_U_A => {
                out[0] = SkISize::Make(w, h);
                out[3] = out[0];
                out[1] = uv;
                out[2] = uv;
                4
            }
            kY_UV_A | kY_VU_A => {
                out[0] = SkISize::Make(w, h);
                out[2] = out[0];
                out[1] = uv;
                3
            }
            kYUV | kUYV | kYUVA | kUYVA => {
                out[0] = SkISize::Make(w, h);
                crate::SkASSERT!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),138; @condition b"planeDimensions[0] == uvSize\0".as_ptr(); out[0]==uv);
                1
            }
        }
    }
}
// Source: src/core/SkYUVAInfo.cpp:144-189
fn channel_index_to_channel(flags: u32, i: i32, channel: &mut SkColorChannel) -> bool {
    let value = match flags {
        kGray_SkColorChannelFlag | kRed_SkColorChannelFlag => {
            if i == 0 {
                Some(SkColorChannel::kR)
            } else {
                None
            }
        }
        kGrayAlpha_SkColorChannelFlags => match i {
            0 => Some(SkColorChannel::kR),
            1 => Some(SkColorChannel::kA),
            _ => None,
        },
        kAlpha_SkColorChannelFlag => {
            if i == 0 {
                Some(SkColorChannel::kA)
            } else {
                None
            }
        }
        kRG_SkColorChannelFlags => {
            if i == 0 || i == 1 {
                Some(if i == 0 {
                    SkColorChannel::kR
                } else {
                    SkColorChannel::kG
                })
            } else {
                None
            }
        }
        kRGB_SkColorChannelFlags => match i {
            0 => Some(SkColorChannel::kR),
            1 => Some(SkColorChannel::kG),
            2 => Some(SkColorChannel::kB),
            _ => None,
        },
        kRGBA_SkColorChannelFlags => match i {
            0 => Some(SkColorChannel::kR),
            1 => Some(SkColorChannel::kG),
            2 => Some(SkColorChannel::kB),
            3 => Some(SkColorChannel::kA),
            _ => None,
        },
        _ => None,
    };
    if let Some(c) = value {
        *channel = c;
        true
    } else {
        false
    }
}
impl SkYUVAInfo {
    // Source: src/core/SkYUVAInfo.cpp:191-279
    // Canonical YUVALocation values preserve the original table and channel algorithm.
    pub fn GetYUVALocations(p: PlaneConfig, flags: &[u32; kMaxPlanes]) -> YUVALocations {
        use PlaneConfig::*;
        let table: [(i32, i32); 4] = match p {
            kUnknown => return [YUVALocation::default(); 4],
            kY_U_V => [(0, 0), (1, 0), (2, 0), (-1, -1)],
            kY_V_U => [(0, 0), (2, 0), (1, 0), (-1, -1)],
            kY_UV => [(0, 0), (1, 0), (1, 1), (-1, -1)],
            kY_VU => [(0, 0), (1, 1), (1, 0), (-1, -1)],
            kYUV => [(0, 0), (0, 1), (0, 2), (-1, -1)],
            kUYV => [(0, 1), (0, 0), (0, 2), (-1, -1)],
            kY_U_V_A => [(0, 0), (1, 0), (2, 0), (3, 0)],
            kY_V_U_A => [(0, 0), (2, 0), (1, 0), (3, 0)],
            kY_UV_A => [(0, 0), (1, 0), (1, 1), (2, 0)],
            kY_VU_A => [(0, 0), (1, 1), (1, 0), (2, 0)],
            kYUVA => [(0, 0), (0, 1), (0, 2), (0, 3)],
            kUYVA => [(0, 1), (0, 0), (0, 2), (0, 3)],
        };
        let mut locations = [YUVALocation::default(); 4];
        for i in 0..kYUVAChannelCount {
            let (plane, index) = table[i];
            let mut channel = SkColorChannel::kR;
            if plane >= 0 {
                if !channel_index_to_channel(flags[plane as usize], index, &mut channel) {
                    return [YUVALocation::default(); 4];
                }
                locations[i] = YUVALocation {
                    fPlane: plane,
                    fChannel: channel,
                };
            } else {
                crate::SkASSERT!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),274; @condition b"i == 3\0".as_ptr(); i==3);
                locations[i] = YUVALocation {
                    fPlane: -1,
                    fChannel: SkColorChannel::kR,
                };
            }
        }
        locations
    }
    // Source: src/core/SkYUVAInfo.cpp:281-300
    pub fn HasAlpha(p: PlaneConfig) -> bool {
        matches!(
            p,
            PlaneConfig::kY_U_V_A
                | PlaneConfig::kY_V_U_A
                | PlaneConfig::kY_UV_A
                | PlaneConfig::kY_VU_A
                | PlaneConfig::kYUVA
                | PlaneConfig::kUYVA
        )
    }
    // Source: src/core/SkYUVAInfo.cpp:302-323
    pub fn new(
        dim: SkISize,
        p: PlaneConfig,
        s: Subsampling,
        yuv: SkYUVColorSpace,
        origin: SkEncodedOrigin,
        x: Siting,
        y: Siting,
    ) -> Self {
        let mut result = Self {
            fDimensions: dim,
            fPlaneConfig: p,
            fSubsampling: s,
            fYUVColorSpace: yuv,
            fOrigin: origin,
            fSitingX: x,
            fSitingY: y,
        };
        if dim.isEmpty() || !is_plane_config_compatible_with_subsampling(p, s) {
            result = Self::default();
            crate::SkASSERT!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),319; @condition b"!this->isValid()\0".as_ptr(); !result.isValid());
            return result;
        }
        crate::SkASSERT!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),322; @condition b"this->isValid()\0".as_ptr(); result.isValid());
        result
    }
    // Source: src/core/SkYUVAInfo.cpp:325-356
    pub fn computeTotalBytes(
        &self,
        rowBytes: &[usize; kMaxPlanes],
        mut planeSizes: Option<&mut [usize; kMaxPlanes]>,
    ) -> usize {
        if !self.isValid() {
            return 0;
        }
        let mut safe = SkSafeMath::new();
        let mut total = 0;
        let mut dims = [SkISize::Make(0, 0); kMaxPlanes];
        let n = self.planeDimensions(&mut dims) as usize;
        for i in 0..n {
            crate::SkASSERT!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),335; @condition b"!planeDimensions[i].isEmpty()\0".as_ptr(); !dims[i].isEmpty());
            crate::SkASSERT!(@source b"src/core/SkYUVAInfo.cpp\0".as_ptr(),336; @condition b"rowBytes[i]\0".as_ptr(); rowBytes[i]!=0);
            let size = safe.mul(rowBytes[i], dims[i].height() as usize);
            if let Some(sizes) = planeSizes.as_mut() {
                sizes[i] = size;
            }
            total = safe.add(total, size);
        }
        if let Some(sizes) = planeSizes {
            if safe.ok() {
                for i in n..kMaxPlanes {
                    sizes[i] = 0;
                }
            } else {
                // Native source tests n, not i, in this overflow loop. Preserve the original
                // condition; Rust bounds checks stop the native out-of-bounds domain by panic.
                let mut i = 0;
                while n < kMaxPlanes {
                    sizes[i] = usize::MAX;
                    i += 1;
                }
            }
        }
        if safe.ok() {
            total
        } else {
            usize::MAX
        }
    }
    // Source: src/core/SkYUVAInfo.cpp:358-360
    pub fn toYUVALocations(&self, flags: &[u32; kMaxPlanes]) -> YUVALocations {
        Self::GetYUVALocations(self.fPlaneConfig, flags)
    }
    // Source: src/core/SkYUVAInfo.cpp:362-364
    pub fn makeSubsampling(&self, s: Subsampling) -> Self {
        Self::new(
            self.fDimensions,
            self.fPlaneConfig,
            s,
            self.fYUVColorSpace,
            self.fOrigin,
            self.fSitingX,
            self.fSitingY,
        )
    }
    // Source: src/core/SkYUVAInfo.cpp:366-368
    pub fn makeDimensions(&self, dim: SkISize) -> Self {
        Self::new(
            dim,
            self.fPlaneConfig,
            self.fSubsampling,
            self.fYUVColorSpace,
            self.fOrigin,
            self.fSitingX,
            self.fSitingY,
        )
    }
}
// Source: src/core/SkYUVAInfo.cpp:370-378
impl PartialEq for SkYUVAInfo {
    fn eq(&self, that: &Self) -> bool {
        self.fPlaneConfig == that.fPlaneConfig
            && self.fSubsampling == that.fSubsampling
            && self.fYUVColorSpace == that.fYUVColorSpace
            && self.fDimensions == that.fDimensions
            && self.fSitingX == that.fSitingX
            && self.fSitingY == that.fSitingY
            && self.fOrigin == that.fOrigin
    }
    // Language-only operator forwarding to the native inline body.
    fn ne(&self, that: &Self) -> bool {
        self.header_ne(that)
    }
}
impl Eq for SkYUVAInfo {}
