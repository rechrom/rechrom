// Copyright 2020 Google LLC. BSD-style license. UNVERIFIED native translation.
use super::{
    SkData::SkData,
    SkImageInfo::{SkColorTypeBytesPerPixel, SkImageInfo, SkYUVColorSpace},
    SkImageInfoPriv::SkColorTypeChannelFlags,
    SkPixmap::SkPixmap,
    SkRectMemcpy::SkRectMemcpy,
    SkYUVAInfo::{kMaxPlanes, SkYUVAInfo, YUVALocations},
};
use crate::include::core::{
    SkAlphaType::SkAlphaType,
    SkColorType::{SkColorType, SkColorType::*},
    SkSize::SkISize,
};
use core::mem::{ManuallyDrop, MaybeUninit};
use std::sync::Arc;
// Source: include/core/SkYUVAPixmaps.h:34-35
pub use super::SkYUVAInfo::{PlaneConfig, Subsampling};
// Source: include/core/SkYUVAPixmaps.h:41-49
// Open integer enum preserves native static_cast<DataType>(kDataTypeCnt) in
// SupportedDataTypes::All, whose inclusive bound intentionally tests one unknown.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DataType(pub i32);
impl DataType {
    pub const kUnorm8: Self = Self(0);
    pub const kUnorm16: Self = Self(1);
    pub const kFloat16: Self = Self(2);
    pub const kUnorm10_Unorm2: Self = Self(3);
    pub const kLast: Self = Self::kUnorm10_Unorm2;
}
pub const kDataTypeCnt: usize = 4;
// Source: include/core/SkYUVAPixmaps.h:51-54
// Source: include/core/SkYUVAPixmaps.h:73-73
// The native bitset owns exactly sixteen flags; integer bits substitute its
// storage, with the same kDataTypeCnt*(channels-1)+datatype indexing.
#[derive(Clone, Copy, Debug, Default)]
pub struct SupportedDataTypes {
    fDataTypeSupport: u16,
}
// Source: include/core/SkYUVAPixmaps.h:30-35
// Source: include/core/SkYUVAPixmaps.h:160-164
#[derive(Clone)]
pub struct SkYUVAPixmapInfo {
    fYUVAInfo: SkYUVAInfo,
    fPlaneInfos: ManuallyDrop<[SkImageInfo; kMaxPlanes]>,
    fRowBytes: [usize; kMaxPlanes],
    fDataType: DataType,
}
// Source: include/core/SkYUVAPixmaps.h:160-164
// Native std::array destroys its elements from last to first. Keep the array's
// actual inline representation and ordinary partially initialized construction;
// only the complete class controls its already initialized element teardown.
impl Drop for SkYUVAPixmapInfo {
    fn drop(&mut self) {
        unsafe {
            for info in self.fPlaneInfos.iter_mut().rev() {
                core::ptr::drop_in_place(info);
            }
        }
    }
}
// Source: include/core/SkYUVAPixmaps.h:91-91
impl Default for SkYUVAPixmapInfo {
    fn default() -> Self {
        const _: () = assert!(kUnknown_SkColorType as i32 == 0);
        Self {
            fYUVAInfo: SkYUVAInfo::default(),
            fPlaneInfos: ManuallyDrop::new(core::array::from_fn(|_| SkImageInfo::default())),
            fRowBytes: [0; kMaxPlanes],
            fDataType: DataType::kUnorm8,
        }
    }
}
impl SkYUVAPixmapInfo {
    // Source: src/core/SkYUVAPixmaps.cpp:65-65
    // Source: src/core/SkYUVAPixmaps.cpp:94-94
    // Native assignment from the empty temporary replaces array elements in
    // forward order, distinct from destruction of a complete array. Transfer the
    // actual default temporary's initialized members without a duplicate drop.
    fn assign_default(&mut self) {
        let empty = ManuallyDrop::new(Self::default());
        self.fYUVAInfo = empty.fYUVAInfo;
        for i in 0..kMaxPlanes {
            self.fPlaneInfos[i] = unsafe { core::ptr::read(&empty.fPlaneInfos[i]) };
        }
        self.fRowBytes = empty.fRowBytes;
        self.fDataType = empty.fDataType;
    }
    // Uncredited native class-scope lexical bridges.
    pub const kMaxPlanes: usize = kMaxPlanes;
    pub const kDataTypeCnt: usize = kDataTypeCnt;
    // Source: include/core/SkYUVAPixmaps.h:116-116
    fn header_ne(&self, that: &Self) -> bool {
        !self.eq(that)
    }
    // Source: include/core/SkYUVAPixmaps.h:118-134
    pub fn yuvaInfo(&self) -> &SkYUVAInfo {
        &self.fYUVAInfo
    }
    pub fn yuvColorSpace(&self) -> SkYUVColorSpace {
        self.fYUVAInfo.yuvColorSpace()
    }
    pub fn numPlanes(&self) -> i32 {
        self.fYUVAInfo.numPlanes()
    }
    pub fn dataType(&self) -> DataType {
        self.fDataType
    }
    pub fn rowBytes(&self, i: i32) -> usize {
        self.fRowBytes[i as usize]
    }
    pub fn planeInfo(&self, i: i32) -> &SkImageInfo {
        &self.fPlaneInfos[i as usize]
    }
    // Source: include/core/SkYUVAPixmaps.h:154-154
    pub fn isValid(&self) -> bool {
        self.fYUVAInfo.isValid()
    }
}
// Source: include/core/SkYUVAPixmaps.h:171-174
// Source: include/core/SkYUVAPixmaps.h:253-256
// Views retain the actual owned Data allocation, or exactly the caller's external
// pointer. No RGBA conversion/cache or synthetic planar allocation is introduced.
#[derive(Clone)]
pub struct SkYUVAPixmaps {
    fPlanes: ManuallyDrop<[SkPixmap; kMaxPlanes]>,
    fData: ManuallyDrop<Option<Arc<SkData>>>,
    fYUVAInfo: SkYUVAInfo,
    fDataType: MaybeUninit<DataType>,
}
// Source: include/core/SkYUVAPixmaps.h:209-215
impl Default for SkYUVAPixmaps {
    fn default() -> Self {
        Self {
            fPlanes: ManuallyDrop::new(core::array::from_fn(|_| SkPixmap::default())),
            fData: ManuallyDrop::new(None),
            fYUVAInfo: SkYUVAInfo::default(),
            fDataType: MaybeUninit::uninit(),
        }
    }
}
impl Drop for SkYUVAPixmaps {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fData);
            for plane in self.fPlanes.iter_mut().rev() {
                core::ptr::drop_in_place(plane);
            }
        }
    }
}
impl SkYUVAPixmaps {
    pub const kMaxPlanes: usize = kMaxPlanes;
    // Source: include/core/SkYUVAPixmaps.h:218-222
    pub fn isValid(&self) -> bool {
        !self.fYUVAInfo.dimensions().isEmpty()
    }
    pub fn yuvaInfo(&self) -> &SkYUVAInfo {
        &self.fYUVAInfo
    }
    /// Safety: native fDataType is initialized only for a configured valid value.
    pub unsafe fn dataType(&self) -> DataType {
        self.fDataType.assume_init()
    }
    // Source: include/core/SkYUVAPixmaps.h:227-238
    pub fn numPlanes(&self) -> i32 {
        if self.isValid() {
            self.fYUVAInfo.numPlanes()
        } else {
            0
        }
    }
    pub fn planes(&self) -> &[SkPixmap; kMaxPlanes] {
        &self.fPlanes
    }
    pub fn plane(&self, i: i32) -> &SkPixmap {
        &self.fPlanes[i as usize]
    }
    // Source: include/core/SkYUVAPixmaps.h:247-247
    pub fn ownsStorage(&self) -> bool {
        self.fData.is_some()
    }
}
impl SupportedDataTypes {
    // Source: include/core/SkYUVAPixmaps.h:261-275
    pub const fn All() -> Self {
        let mut bits = 0u64;
        let mut c = 1;
        while c <= 4 {
            let mut dt = 0;
            while dt <= kDataTypeCnt {
                if SkYUVAPixmapInfo::DefaultColorTypeForDataType(DataType(dt as i32), c as i32)
                    as i32
                    != kUnknown_SkColorType as i32
                {
                    bits |= 1u64 << (dt + kDataTypeCnt * (c - 1));
                }
                dt += 1;
            }
            c += 1;
        }
        Self {
            fDataTypeSupport: bits as u16,
        }
    }
    // Source: include/core/SkYUVAPixmaps.h:277-289
    pub const fn supported(&self, config: PlaneConfig, kind: DataType) -> bool {
        let n = SkYUVAInfo::NumPlanes(config);
        let mut i = 0;
        while i < n {
            let c = SkYUVAInfo::NumChannelsInPlane(config, i) as usize;
            {
                const fn constant_check(c: usize) {
                    #[cfg(sk_debug)]
                    assert!(
                        c >= 1 && c <= 4,
                        "include/core/SkYUVAPixmaps.h:282: check(c >= 1 && c <= 4)"
                    );
                }
                fn runtime_check(c: usize) {
                    crate::SkASSERT!(@source b"include/core/SkYUVAPixmaps.h\0".as_ptr(),282; @condition b"c >= 1 && c <= 4\0".as_ptr(); c>=1&&c<=4);
                }
                unsafe {
                    core::intrinsics::const_eval_select((c,), constant_check, runtime_check);
                }
            }
            if self.fDataTypeSupport & (1u16 << (kind.0 as usize + (c - 1) * kDataTypeCnt)) == 0 {
                return false;
            }
            i += 1;
        }
        true
    }
}
impl SkYUVAPixmapInfo {
    // Source: include/core/SkYUVAPixmaps.h:291-333
    pub const fn DefaultColorTypeForDataType(kind: DataType, channels: i32) -> SkColorType {
        match channels {
            1 => match kind.0 {
                0 => kGray_8_SkColorType,
                1 => kA16_unorm_SkColorType,
                2 => kA16_float_SkColorType,
                _ => kUnknown_SkColorType,
            },
            2 => match kind.0 {
                0 => kR8G8_unorm_SkColorType,
                1 => kR16G16_unorm_SkColorType,
                2 => kR16G16_float_SkColorType,
                _ => kUnknown_SkColorType,
            },
            3 | 4 => match kind.0 {
                0 => kRGBA_8888_SkColorType,
                1 => kR16G16B16A16_unorm_SkColorType,
                2 => kRGBA_F16_SkColorType,
                3 => kRGBA_1010102_SkColorType,
                _ => kUnknown_SkColorType,
            },
            _ => kUnknown_SkColorType,
        }
    }
}
impl SupportedDataTypes {
    // Source: src/core/SkYUVAPixmaps.cpp:20-25
    pub fn enableDataType(&mut self, kind: DataType, channels: i32) {
        if channels < 1 || channels > 4 {
            return;
        }
        self.fDataTypeSupport |= 1u16 << (kind.0 as usize + (channels as usize - 1) * kDataTypeCnt);
    }
}
impl SkYUVAPixmapInfo {
    // Source: src/core/SkYUVAPixmaps.cpp:29-58
    pub fn NumChannelsAndDataType(ct: SkColorType) -> (i32, DataType) {
        match ct {
            kR8_unorm_SkColorType | kAlpha_8_SkColorType | kGray_8_SkColorType => {
                (1, DataType::kUnorm8)
            }
            kR16_unorm_SkColorType | kA16_unorm_SkColorType => (1, DataType::kUnorm16),
            kR16_float_SkColorType | kA16_float_SkColorType => (1, DataType::kFloat16),
            kR8G8_unorm_SkColorType => (2, DataType::kUnorm8),
            kR16G16_unorm_SkColorType => (2, DataType::kUnorm16),
            kR16G16_float_SkColorType => (2, DataType::kFloat16),
            kRGB_888x_SkColorType => (3, DataType::kUnorm8),
            kRGB_101010x_SkColorType => (3, DataType::kUnorm10_Unorm2),
            kRGB_F16F16F16x_SkColorType => (3, DataType::kFloat16),
            kRGBA_8888_SkColorType => (4, DataType::kUnorm8),
            kR16G16B16A16_unorm_SkColorType => (4, DataType::kUnorm16),
            kRGBA_F16_SkColorType | kRGBA_F16Norm_SkColorType => (4, DataType::kFloat16),
            kRGBA_1010102_SkColorType => (4, DataType::kUnorm10_Unorm2),
            _ => (0, DataType::kUnorm8),
        }
    }
    // Source: src/core/SkYUVAPixmaps.cpp:60-99
    pub fn new(
        yuva: &SkYUVAInfo,
        types: &[SkColorType; kMaxPlanes],
        rowBytes: Option<&[usize; kMaxPlanes]>,
    ) -> Self {
        let fYUVAInfo = *yuva;
        let mut result = Self::default();
        result.fYUVAInfo = fYUVAInfo;
        if !yuva.isValid() {
            result.assign_default();
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),66; @condition b"!this->isValid()\0".as_ptr(); !result.isValid());
            return result;
        }
        let mut dims = [SkISize::Make(0, 0); kMaxPlanes];
        let n = yuva.planeDimensions(&mut dims) as usize;
        let mut temp = [0; kMaxPlanes];
        let rowBytes = if let Some(row) = rowBytes {
            row
        } else {
            for i in 0..n {
                temp[i] =
                    (SkColorTypeBytesPerPixel(types[i]) as usize) * (dims[i].width() as usize);
            }
            &temp
        };
        let mut ok = true;
        for i in 0..n {
            result.fRowBytes[i] = rowBytes[i];
            result.fPlaneInfos[i] =
                SkImageInfo::MakeDimensions(dims[i], types[i], SkAlphaType::kUnpremul_SkAlphaType);
            let required = yuva.numChannelsInPlane(i as i32);
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),86; @condition b"numRequiredChannels > 0\0".as_ptr(); required>0);
            let (channels, datatype) = Self::NumChannelsAndDataType(types[i]);
            ok &= i == 0 || datatype == result.fDataType;
            ok &= channels >= required;
            ok &= result.fPlaneInfos[i].validRowBytes(result.fRowBytes[i]);
            result.fDataType = datatype;
        }
        if !ok {
            result.assign_default();
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),95; @condition b"!this->isValid()\0".as_ptr(); !result.isValid());
        } else {
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),97; @condition b"this->isValid()\0".as_ptr(); result.isValid());
        }
        result
    }
    // Source: src/core/SkYUVAPixmaps.cpp:101-111
    pub fn new_data_type(
        yuva: &SkYUVAInfo,
        kind: DataType,
        rowBytes: Option<&[usize; kMaxPlanes]>,
    ) -> Self {
        let mut types = [kUnknown_SkColorType; kMaxPlanes];
        for i in 0..yuva.numPlanes() {
            types[i as usize] = Self::DefaultColorTypeForDataType(kind, yuva.numChannelsInPlane(i));
        }
        Self::new(yuva, &types, rowBytes)
    }
}
// Source: src/core/SkYUVAPixmaps.cpp:113-119
impl PartialEq for SkYUVAPixmapInfo {
    fn eq(&self, that: &Self) -> bool {
        let result = self.fYUVAInfo == that.fYUVAInfo
            && self.fPlaneInfos == that.fPlaneInfos
            && self.fRowBytes == that.fRowBytes;
        crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),117; @condition b"!result || fDataType == that.fDataType\0".as_ptr(); !result||self.fDataType==that.fDataType);
        result
    }
    // Language-only virtual/operator forwarding to the native inline body.
    fn ne(&self, that: &Self) -> bool {
        self.header_ne(that)
    }
}
impl SkYUVAPixmapInfo {
    // Source: src/core/SkYUVAPixmaps.cpp:121-129
    pub fn computeTotalBytes(&self, mut sizes: Option<&mut [usize; kMaxPlanes]>) -> usize {
        if !self.isValid() {
            if let Some(sizes) = sizes.as_mut() {
                sizes.fill(0);
            }
            return 0;
        }
        self.fYUVAInfo.computeTotalBytes(&self.fRowBytes, sizes)
    }
    // Source: src/core/SkYUVAPixmaps.cpp:131-150
    /// Safety: memory is a live allocation of at least computeTotalBytes() bytes;
    /// callers retain it for every access through returned nonowning pixmaps.
    pub unsafe fn initPixmapsFromSingleAllocation(
        &self,
        memory: *mut u8,
        pixmaps: &mut [SkPixmap; kMaxPlanes],
    ) -> bool {
        if !self.isValid() {
            return false;
        }
        let mut addr = memory;
        let n = self.numPlanes() as usize;
        for i in 0..n {
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),140; @condition b"fPlaneInfos[i].validRowBytes(fRowBytes[i])\0".as_ptr(); self.fPlaneInfos[i].validRowBytes(self.fRowBytes[i]));
            pixmaps[i].reset_with(&self.fPlaneInfos[i], addr, self.fRowBytes[i]);
            let bytes = pixmaps[i].rowBytes() * (pixmaps[i].height() as usize);
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),143; @condition b"planeSize\0".as_ptr(); bytes!=0);
            addr = addr.add(bytes);
        }
        for i in n..kMaxPlanes {
            pixmaps[i] = SkPixmap::default();
        }
        true
    }
    // Source: src/core/SkYUVAPixmaps.cpp:152-158
    pub fn isSupported(&self, supported: &SupportedDataTypes) -> bool {
        if !self.isValid() {
            return false;
        }
        supported.supported(self.fYUVAInfo.planeConfig(), self.fDataType)
    }
}
impl SkYUVAPixmaps {
    // Source: src/core/SkYUVAPixmaps.cpp:161-171
    pub fn RecommendedRGBAColorType(kind: DataType) -> SkColorType {
        match kind.0 {
            0 => kRGBA_8888_SkColorType,
            1 | 2 => kRGBA_F16_SkColorType,
            3 => kRGBA_1010102_SkColorType,
            _ => crate::SkUNREACHABLE!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),170;),
        }
    }
    // Source: src/core/SkYUVAPixmaps.cpp:173-179
    /// Safety: returned uninitialized plane memory must be filled before reads;
    /// the canonical mutable SkData lease excludes concurrent pixel access.
    pub unsafe fn Allocate(info: &SkYUVAPixmapInfo) -> Self {
        if !info.isValid() {
            return Self::default();
        }
        Self::from_data(
            info,
            SkData::MakeUninitialized(info.computeTotalBytes(None)),
        )
    }
    // Source: src/core/SkYUVAPixmaps.cpp:181-189
    /// Safety: retained Data provides writable plane bytes with the original
    /// caller's exclusive write/read protocol; this does not invent a COW owner.
    pub unsafe fn FromData(info: &SkYUVAPixmapInfo, data: Arc<SkData>) -> Self {
        if !info.isValid() {
            return Self::default();
        }
        if info.computeTotalBytes(None) > data.size() {
            return Self::default();
        }
        Self::from_data(info, data)
    }
    // Source: src/core/SkYUVAPixmaps.cpp:191-210
    /// Safety: every source plane covers all declared rows and initialized pixels;
    /// destination has the native fresh unique writable backing allocation.
    pub unsafe fn MakeCopy(src: &Self) -> Self {
        if !src.isValid() {
            return Self::default();
        }
        let result = Self::Allocate(&src.pixmapsInfo());
        for i in 0..result.numPlanes() {
            let s = src.plane(i);
            let d = result.plane(i);
            SkRectMemcpy(
                d.writable_addr(),
                d.rowBytes(),
                s.addr(),
                s.rowBytes(),
                s.info().minRowBytes(),
                s.height(),
            );
        }
        result
    }
    // Source: src/core/SkYUVAPixmaps.cpp:212-221
    /// Safety: external memory outlives all copied views and covers the complete
    /// plane allocation; mutation/read leases remain owned by the caller.
    pub unsafe fn FromExternalMemory(info: &SkYUVAPixmapInfo, memory: *mut u8) -> Self {
        if !info.isValid() {
            return Self::default();
        }
        let mut pixmaps = core::array::from_fn(|_| SkPixmap::default());
        info.initPixmapsFromSingleAllocation(memory, &mut pixmaps);
        Self::from_pixmaps(info.yuvaInfo(), info.dataType(), &pixmaps)
    }
    // Source: src/core/SkYUVAPixmaps.cpp:222-236
    /// Safety: the original pixel owners outlive these retained nonowning views;
    /// native plane pixel sizes, initialization, and write leases are valid.
    pub unsafe fn FromExternalPixmaps(yuva: &SkYUVAInfo, pixmaps: &[SkPixmap; kMaxPlanes]) -> Self {
        let mut types = [kUnknown_SkColorType; kMaxPlanes];
        let mut row = [0; kMaxPlanes];
        for i in 0..yuva.numPlanes() as usize {
            types[i] = pixmaps[i].colorType();
            row[i] = pixmaps[i].rowBytes();
        }
        let info = SkYUVAPixmapInfo::new(yuva, &types, Some(&row));
        if !info.isValid() {
            return Self::default();
        }
        Self::from_pixmaps(yuva, info.dataType(), pixmaps)
    }
    // Source: src/core/SkYUVAPixmaps.cpp:238-246
    unsafe fn from_data(info: &SkYUVAPixmapInfo, data: Arc<SkData>) -> Self {
        let mut result = Self {
            fPlanes: ManuallyDrop::new(core::array::from_fn(|_| SkPixmap::default())),
            fData: ManuallyDrop::new(Some(data)),
            fYUVAInfo: *info.yuvaInfo(),
            fDataType: MaybeUninit::new(info.dataType()),
        };
        crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),242; @condition b"yuvaPixmapInfo.isValid()\0".as_ptr(); info.isValid());
        crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),243; @condition b"yuvaPixmapInfo.computeTotalBytes() <= fData->size()\0".as_ptr(); info.computeTotalBytes(None)<=result.fData.as_ref().unwrap().size());
        crate::SkAssertResult!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),244; @condition b"yuvaPixmapInfo.initPixmapsFromSingleAllocation(fData->writable_data(), fPlanes.data())\0".as_ptr(); info.initPixmapsFromSingleAllocation(result.fData.as_ref().unwrap().writable_data(),&mut result.fPlanes));
        result
    }
    // Source: src/core/SkYUVAPixmaps.cpp:248-253
    unsafe fn from_pixmaps(
        yuva: &SkYUVAInfo,
        kind: DataType,
        pixmaps: &[SkPixmap; kMaxPlanes],
    ) -> Self {
        let mut result = Self::default();
        result.fYUVAInfo = *yuva;
        result.fDataType = MaybeUninit::new(kind);
        for i in 0..yuva.numPlanes() as usize {
            result.fPlanes[i] = pixmaps[i].clone();
        }
        result
    }
    // Source: src/core/SkYUVAPixmaps.cpp:255-267
    pub fn pixmapsInfo(&self) -> SkYUVAPixmapInfo {
        if !self.isValid() {
            return SkYUVAPixmapInfo::default();
        }
        let mut types = [kUnknown_SkColorType; kMaxPlanes];
        let mut row = [0; kMaxPlanes];
        for i in 0..self.numPlanes() as usize {
            types[i] = self.fPlanes[i].colorType();
            row[i] = self.fPlanes[i].rowBytes();
        }
        SkYUVAPixmapInfo::new(&self.fYUVAInfo, &types, Some(&row))
    }
    // Source: src/core/SkYUVAPixmaps.cpp:269-279
    // PENDING the real YUVALocation definition/validation helper.
    pub fn toYUVALocations(&self) -> YUVALocations {
        let flags = core::array::from_fn(|i| SkColorTypeChannelFlags(self.fPlanes[i].colorType()));
        let result = self.fYUVAInfo.toYUVALocations(&flags);
        #[cfg(sk_debug)]
        {
            let mut planes = 0;
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),276; @condition b"SkYUVAInfo::YUVALocation::AreValidLocations(result, &numPlanes)\0".as_ptr(); super::SkYUVAInfoLocation::YUVALocation::AreValidLocations(&result,Some(&mut planes)));
            crate::SkASSERT!(@source b"src/core/SkYUVAPixmaps.cpp\0".as_ptr(),277; @condition b"numPlanes == this->numPlanes()\0".as_ptr(); planes==self.numPlanes());
        }
        result
    }
}
