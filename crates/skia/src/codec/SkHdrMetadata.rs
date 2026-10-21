//! Actual inline HDR metadata types and operations. Ordinary out-of-line
//! declarations remain pending their native implementation file, not fake traits.
use crate::src::core::SkColorSpace::SkColorSpacePrimaries;
// Source: include/private/SkHdrMetadata.h:38
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ContentLightLevelInformation {
    // Source: include/private/SkHdrMetadata.h:39
    pub fMaxCLL: f32,
    // Source: include/private/SkHdrMetadata.h:40
    pub fMaxFALL: f32,
}
impl Default for ContentLightLevelInformation {
    fn default() -> Self {
        Self {
            fMaxCLL: 0.0,
            fMaxFALL: 0.0,
        }
    }
}
impl ContentLightLevelInformation {
    // Source: include/private/SkHdrMetadata.h:63
    pub fn MakeUint16(maxCLL: u16, maxFALL: u16) -> Self {
        Self {
            fMaxCLL: maxCLL as f32,
            fMaxFALL: maxFALL as f32,
        }
    }
    // Source: include/private/SkHdrMetadata.h:66
    /// Safety: the source value must not be NaN. Native float-to-integer
    /// conversion is undefined for NaN; no replacement success/fallback is added.
    pub unsafe fn getUint16MaxCLL(&self) -> u16 {
        unsafe { native_u16_clamp_round(self.fMaxCLL) }
    }
    // Source: include/private/SkHdrMetadata.h:69
    /// Safety: the source value must not be NaN. Native float-to-integer
    /// conversion is undefined for NaN; no replacement success/fallback is added.
    pub unsafe fn getUint16MaxFALL(&self) -> u16 {
        unsafe { native_u16_clamp_round(self.fMaxFALL) }
    }
}
// Source: include/private/SkHdrMetadata.h:105
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MasteringDisplayColorVolume {
    // Source: include/private/SkHdrMetadata.h:106
    pub fDisplayPrimaries: SkColorSpacePrimaries,
    // Source: include/private/SkHdrMetadata.h:107
    pub fMaximumDisplayMasteringLuminance: f32,
    // Source: include/private/SkHdrMetadata.h:108
    pub fMinimumDisplayMasteringLuminance: f32,
}
fn zero_primaries() -> SkColorSpacePrimaries {
    SkColorSpacePrimaries {
        fRX: 0.0,
        fRY: 0.0,
        fGX: 0.0,
        fGY: 0.0,
        fBX: 0.0,
        fBY: 0.0,
        fWX: 0.0,
        fWY: 0.0,
    }
}
impl Default for MasteringDisplayColorVolume {
    fn default() -> Self {
        Self {
            fDisplayPrimaries: zero_primaries(),
            fMaximumDisplayMasteringLuminance: 0.0,
            fMinimumDisplayMasteringLuminance: 0.0,
        }
    }
}
// Source: include/private/SkHdrMetadata.h:155
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ControlPoint {
    pub fX: f32,
    pub fY: f32,
    pub fM: f32,
}
impl Default for ControlPoint {
    fn default() -> Self {
        Self {
            fX: 0.0,
            fY: 0.0,
            fM: 0.0,
        }
    }
}
// Source: include/private/SkHdrMetadata.h:161
impl PartialEq for ControlPoint {
    fn eq(&self, other: &Self) -> bool {
        self.fX == other.fX && self.fY == other.fY && self.fM == other.fM
    }
}
// Source: include/private/SkHdrMetadata.h:152
#[derive(Debug)]
pub struct GainCurve {
    pub fControlPoints: Vec<ControlPoint>,
}
impl Default for GainCurve {
    fn default() -> Self {
        Self {
            fControlPoints: Vec::new(),
        }
    }
}
impl Clone for GainCurve {
    fn clone(&self) -> Self {
        Self {
            fControlPoints: self.fControlPoints.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fControlPoints.clone_from(&other.fControlPoints);
    }
}
impl GainCurve {
    // Source: include/private/SkHdrMetadata.h:166
    pub const kMinNumControlPoints: usize = 1;
    // Source: include/private/SkHdrMetadata.h:167
    pub const kMaxNumControlPoints: usize = 32;
}
// Source: include/private/SkHdrMetadata.h:171
impl PartialEq for GainCurve {
    fn eq(&self, other: &Self) -> bool {
        self.fControlPoints == other.fControlPoints
    }
}
// Source: include/private/SkHdrMetadata.h:176
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ComponentMixingFunction {
    pub fRed: f32,
    pub fGreen: f32,
    pub fBlue: f32,
    pub fMax: f32,
    pub fMin: f32,
    pub fComponent: f32,
}
impl Default for ComponentMixingFunction {
    fn default() -> Self {
        Self {
            fRed: 0.0,
            fGreen: 0.0,
            fBlue: 0.0,
            fMax: 0.0,
            fMin: 0.0,
            fComponent: 0.0,
        }
    }
}
// Source: include/private/SkHdrMetadata.h:186
impl PartialEq for ComponentMixingFunction {
    fn eq(&self, other: &Self) -> bool {
        self.fRed == other.fRed
            && self.fGreen == other.fGreen
            && self.fBlue == other.fBlue
            && self.fMax == other.fMax
            && self.fMin == other.fMin
            && self.fComponent == other.fComponent
    }
}
// Source: include/private/SkHdrMetadata.h:192
// Private/native-container representation: reverse owning-member destruction;
// explicit constructors/copies still evaluate in original native member order.
#[derive(Debug)]
pub struct ColorGainFunction {
    pub fGainCurve: GainCurve,
    pub fComponentMixing: ComponentMixingFunction,
}
impl Default for ColorGainFunction {
    fn default() -> Self {
        Self {
            fComponentMixing: ComponentMixingFunction::default(),
            fGainCurve: GainCurve::default(),
        }
    }
}
impl Clone for ColorGainFunction {
    fn clone(&self) -> Self {
        Self {
            fComponentMixing: self.fComponentMixing,
            fGainCurve: self.fGainCurve.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fComponentMixing = other.fComponentMixing;
        self.fGainCurve.clone_from(&other.fGainCurve);
    }
}
// Source: include/private/SkHdrMetadata.h:200
impl PartialEq for ColorGainFunction {
    fn eq(&self, other: &Self) -> bool {
        self.fComponentMixing == other.fComponentMixing && self.fGainCurve == other.fGainCurve
    }
}
// Source: include/private/SkHdrMetadata.h:205
#[derive(Debug)]
pub struct AlternateImage {
    pub fColorGainFunction: ColorGainFunction,
    pub fHdrHeadroom: f32,
}
impl Default for AlternateImage {
    fn default() -> Self {
        Self {
            fHdrHeadroom: 0.0,
            fColorGainFunction: ColorGainFunction::default(),
        }
    }
}
impl Clone for AlternateImage {
    fn clone(&self) -> Self {
        Self {
            fHdrHeadroom: self.fHdrHeadroom,
            fColorGainFunction: self.fColorGainFunction.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fHdrHeadroom = other.fHdrHeadroom;
        self.fColorGainFunction
            .clone_from(&other.fColorGainFunction);
    }
}
// Source: include/private/SkHdrMetadata.h:213
impl PartialEq for AlternateImage {
    fn eq(&self, other: &Self) -> bool {
        self.fHdrHeadroom == other.fHdrHeadroom
            && self.fColorGainFunction == other.fColorGainFunction
    }
}
// Source: include/private/SkHdrMetadata.h:218
#[derive(Debug)]
pub struct HeadroomAdaptiveToneMap {
    pub fAlternateImages: Vec<AlternateImage>,
    pub fGainApplicationSpacePrimaries: SkColorSpacePrimaries,
    pub fBaselineHdrHeadroom: f32,
}
impl Default for HeadroomAdaptiveToneMap {
    fn default() -> Self {
        Self {
            fBaselineHdrHeadroom: 0.0,
            fGainApplicationSpacePrimaries: zero_primaries(),
            fAlternateImages: Vec::new(),
        }
    }
}
impl Clone for HeadroomAdaptiveToneMap {
    fn clone(&self) -> Self {
        Self {
            fBaselineHdrHeadroom: self.fBaselineHdrHeadroom,
            fGainApplicationSpacePrimaries: self.fGainApplicationSpacePrimaries,
            fAlternateImages: self.fAlternateImages.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fBaselineHdrHeadroom = other.fBaselineHdrHeadroom;
        self.fGainApplicationSpacePrimaries = other.fGainApplicationSpacePrimaries;
        self.fAlternateImages.clone_from(&other.fAlternateImages);
    }
}
impl HeadroomAdaptiveToneMap {
    // Source: include/private/SkHdrMetadata.h:227
    pub const kMaxNumAlternateImages: usize = 4;
}
// Source: include/private/SkHdrMetadata.h:231
impl PartialEq for HeadroomAdaptiveToneMap {
    fn eq(&self, other: &Self) -> bool {
        self.fBaselineHdrHeadroom == other.fBaselineHdrHeadroom
            && self.fGainApplicationSpacePrimaries == other.fGainApplicationSpacePrimaries
            && self.fAlternateImages == other.fAlternateImages
    }
}
// Source: include/private/SkHdrMetadata.h:150
#[derive(Debug)]
pub struct AdaptiveGlobalToneMap {
    pub fHeadroomAdaptiveToneMap: Option<HeadroomAdaptiveToneMap>,
    pub fHdrReferenceWhite: f32,
}
impl AdaptiveGlobalToneMap {
    // Source: include/private/SkHdrMetadata.h:238
    pub const kDefaultHdrReferenceWhite: f32 = 203.0;
}
impl Default for AdaptiveGlobalToneMap {
    fn default() -> Self {
        Self {
            fHdrReferenceWhite: Self::kDefaultHdrReferenceWhite,
            fHeadroomAdaptiveToneMap: None,
        }
    }
}
impl Clone for AdaptiveGlobalToneMap {
    fn clone(&self) -> Self {
        Self {
            fHdrReferenceWhite: self.fHdrReferenceWhite,
            fHeadroomAdaptiveToneMap: self.fHeadroomAdaptiveToneMap.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fHdrReferenceWhite = other.fHdrReferenceWhite;
        self.fHeadroomAdaptiveToneMap
            .clone_from(&other.fHeadroomAdaptiveToneMap);
    }
}
// Source: include/private/SkHdrMetadata.h:267
impl PartialEq for AdaptiveGlobalToneMap {
    fn eq(&self, other: &Self) -> bool {
        self.fHdrReferenceWhite == other.fHdrReferenceWhite
            && self.fHeadroomAdaptiveToneMap == other.fHeadroomAdaptiveToneMap
    }
}
// Source: include/private/SkHdrMetadata.h:275
// The actual implicit copy/default constructors are language mappings. Ordinary
// declared methods are not implemented by an invented metadata/provider trait.
#[derive(Debug)]
pub struct Metadata {
    // Source: include/private/SkHdrMetadata.h:352
    fAdaptiveGlobalToneMap: Option<AdaptiveGlobalToneMap>,
    // Source: include/private/SkHdrMetadata.h:351
    fMasteringDisplayColorVolume: Option<MasteringDisplayColorVolume>,
    // Source: include/private/SkHdrMetadata.h:350
    fContentLightLevelInformation: Option<ContentLightLevelInformation>,
}
impl Default for Metadata {
    fn default() -> Self {
        Self {
            fContentLightLevelInformation: None,
            fMasteringDisplayColorVolume: None,
            fAdaptiveGlobalToneMap: None,
        }
    }
}
impl Clone for Metadata {
    fn clone(&self) -> Self {
        Self {
            fContentLightLevelInformation: self.fContentLightLevelInformation,
            fMasteringDisplayColorVolume: self.fMasteringDisplayColorVolume,
            fAdaptiveGlobalToneMap: self.fAdaptiveGlobalToneMap.clone(),
        }
    }
    fn clone_from(&mut self, other: &Self) {
        self.fContentLightLevelInformation = other.fContentLightLevelInformation;
        self.fMasteringDisplayColorVolume = other.fMasteringDisplayColorVolume;
        self.fAdaptiveGlobalToneMap
            .clone_from(&other.fAdaptiveGlobalToneMap);
    }
}

use crate::src::core::{
    SkData::SkData,
    SkStream::{SkDynamicMemoryWStream, SkMemoryStream},
    SkStreamPriv,
    SkString::{SkString, SkStringPrintf},
};
use std::sync::Arc;
// Actual standard <cmath> float overloads, not a guessed C++ mangled symbol.
unsafe extern "C" {
    fn roundf(value: f32) -> f32;
    fn llroundf(value: f32) -> core::ffi::c_longlong;
}
// Source: include/private/SkHdrMetadata.h:69-73
unsafe fn native_u16_clamp_round(value: f32) -> u16 {
    let rounded = unsafe { roundf(value) };
    let clamped = if rounded < 0.0 {
        0.0
    } else if 65535.0 < rounded {
        65535.0
    } else {
        rounded
    };
    unsafe { clamped.to_int_unchecked::<u16>() }
}
// Source: src/codec/SkHdrMetadata.cpp:17-19
impl ContentLightLevelInformation {
    pub fn toString(&self) -> SkString {
        unsafe {
            SkStringPrintf(
                b"{maxCLL:%f, maxFALL:%f}\0".as_ptr(),
                self.fMaxCLL as f64,
                self.fMaxFALL as f64,
            )
        }
    }
    // Source: src/codec/SkHdrMetadata.cpp:21-39
    /// Safety: data is a live readable native SkData pointer; it is nonnull.
    pub unsafe fn parse(&mut self, data: *const SkData) -> bool {
        let data = unsafe { &*data };
        if data.size() != 4 {
            return false;
        }
        let mut stream = core::pin::pin!(unsafe {
            SkMemoryStream::from_memory(data.data(), data.size(), false)
        });
        let mut max_cll = 0u16;
        let mut max_fall = 0u16;
        if !SkStreamPriv::ReadU16BE(stream.as_mut(), &mut max_cll) {
            return false;
        }
        if !SkStreamPriv::ReadU16BE(stream.as_mut(), &mut max_fall) {
            return false;
        }
        self.fMaxCLL = max_cll as f32;
        self.fMaxFALL = max_fall as f32;
        true
    }
    // Source: src/codec/SkHdrMetadata.cpp:41-46
    /// Safety: max light values satisfy the native non-NaN integer conversion domain.
    pub unsafe fn serialize(&self) -> Arc<SkData> {
        let mut stream = core::pin::pin!(SkDynamicMemoryWStream::new());
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe { self.getUint16MaxCLL() });
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe { self.getUint16MaxFALL() });
        stream.as_mut().detachAsData()
    }
}
// Source: src/codec/SkHdrMetadata.cpp:50
const clli_png_luminance_divisor: f32 = 10000.0;
impl ContentLightLevelInformation {
    // Source: src/codec/SkHdrMetadata.cpp:52-70
    /// Safety: same live nonnull data lease as parse.
    pub unsafe fn parsePngChunk(&mut self, data: *const SkData) -> bool {
        let data = unsafe { &*data };
        if data.size() != 8 {
            return false;
        }
        let mut stream = core::pin::pin!(unsafe {
            SkMemoryStream::from_memory(data.data(), data.size(), false)
        });
        let mut max_cll_times_10000 = 0u32;
        let mut max_fall_times_10000 = 0u32;
        if !SkStreamPriv::ReadU32BE(stream.as_mut(), &mut max_cll_times_10000) {
            return false;
        }
        if !SkStreamPriv::ReadU32BE(stream.as_mut(), &mut max_fall_times_10000) {
            return false;
        }
        self.fMaxCLL = max_cll_times_10000 as f32 / clli_png_luminance_divisor;
        self.fMaxFALL = max_fall_times_10000 as f32 / clli_png_luminance_divisor;
        true
    }
    // Source: src/codec/SkHdrMetadata.cpp:72-77
    pub fn serializePngChunk(&self) -> Arc<SkData> {
        let mut stream = core::pin::pin!(SkDynamicMemoryWStream::new());
        let _ = SkStreamPriv::WriteU32BE(stream.as_mut(), unsafe {
            llroundf(self.fMaxCLL * clli_png_luminance_divisor)
        } as u32);
        let _ = SkStreamPriv::WriteU32BE(stream.as_mut(), unsafe {
            llroundf(self.fMaxFALL * clli_png_luminance_divisor)
        } as u32);
        stream.as_mut().detachAsData()
    }
}
// Source: src/codec/SkHdrMetadata.cpp:79-82
impl PartialEq for ContentLightLevelInformation {
    fn eq(&self, other: &Self) -> bool {
        self.fMaxCLL == other.fMaxCLL && self.fMaxFALL == other.fMaxFALL
    }
    // Source: include/private/SkHdrMetadata.h:94-96
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
// Source: src/codec/SkHdrMetadata.cpp:86-87
const mdcv_chrominance_divisor: f32 = 50000.0;
const mdcv_luminance_divisor: f32 = 10000.0;
impl MasteringDisplayColorVolume {
    // Source: src/codec/SkHdrMetadata.cpp:89-98
    pub fn toString(&self) -> SkString {
        unsafe {
            SkStringPrintf(b"{red:[%1.8f,%1.8f], green:[%1.8f,%1.8f], blue:[%1.8f,%1.8f], white:[%1.8f,%1.8f], maxLum:%f, minLum:%f}\0".as_ptr(),
            self.fDisplayPrimaries.fRX as f64,self.fDisplayPrimaries.fRY as f64,
            self.fDisplayPrimaries.fGX as f64,self.fDisplayPrimaries.fGY as f64,
            self.fDisplayPrimaries.fBX as f64,self.fDisplayPrimaries.fBY as f64,
            self.fDisplayPrimaries.fWX as f64,self.fDisplayPrimaries.fWY as f64,
            self.fMaximumDisplayMasteringLuminance as f64,self.fMinimumDisplayMasteringLuminance as f64)
        }
    }
    // Source: src/codec/SkHdrMetadata.cpp:100-135
    /// Safety: data is the same live nonnull native data lease required by parse.
    pub unsafe fn parse(&mut self, data: *const SkData) -> bool {
        let data = unsafe { &*data };
        if data.size() != 24 {
            return false;
        }
        let mut stream = core::pin::pin!(unsafe {
            SkMemoryStream::from_memory(data.data(), data.size(), false)
        });
        // Native unfilled values are never observed; initialized Rust scratch
        // permits the same writable integer references without extra heap storage.
        let mut chromaticities_times_50000 = [0u16; 8];
        for value in &mut chromaticities_times_50000 {
            if !SkStreamPriv::ReadU16BE(stream.as_mut(), value) {
                return false;
            }
        }
        let mut max_luminance_times_10000 = 0u32;
        let mut min_luminance_times_10000 = 0u32;
        if !SkStreamPriv::ReadU32BE(stream.as_mut(), &mut max_luminance_times_10000) {
            return false;
        }
        if !SkStreamPriv::ReadU32BE(stream.as_mut(), &mut min_luminance_times_10000) {
            return false;
        }
        self.fDisplayPrimaries = SkColorSpacePrimaries {
            fRX: chromaticities_times_50000[0] as f32 / mdcv_chrominance_divisor,
            fRY: chromaticities_times_50000[1] as f32 / mdcv_chrominance_divisor,
            fGX: chromaticities_times_50000[2] as f32 / mdcv_chrominance_divisor,
            fGY: chromaticities_times_50000[3] as f32 / mdcv_chrominance_divisor,
            fBX: chromaticities_times_50000[4] as f32 / mdcv_chrominance_divisor,
            fBY: chromaticities_times_50000[5] as f32 / mdcv_chrominance_divisor,
            fWX: chromaticities_times_50000[6] as f32 / mdcv_chrominance_divisor,
            fWY: chromaticities_times_50000[7] as f32 / mdcv_chrominance_divisor,
        };
        self.fMaximumDisplayMasteringLuminance =
            max_luminance_times_10000 as f32 / mdcv_luminance_divisor;
        self.fMinimumDisplayMasteringLuminance =
            min_luminance_times_10000 as f32 / mdcv_luminance_divisor;
        true
    }
    // Source: src/codec/SkHdrMetadata.cpp:137-152
    pub fn serialize(&self) -> Arc<SkData> {
        let mut stream = core::pin::pin!(SkDynamicMemoryWStream::new());
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fRX * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fRY * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fGX * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fGY * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fBX * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fBY * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fWX * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU16BE(stream.as_mut(), unsafe {
            llroundf(self.fDisplayPrimaries.fWY * mdcv_chrominance_divisor)
        } as u16);
        let _ = SkStreamPriv::WriteU32BE(stream.as_mut(), unsafe {
            llroundf(self.fMaximumDisplayMasteringLuminance * mdcv_luminance_divisor)
        } as u32);
        let _ = SkStreamPriv::WriteU32BE(stream.as_mut(), unsafe {
            llroundf(self.fMinimumDisplayMasteringLuminance * mdcv_luminance_divisor)
        } as u32);
        stream.as_mut().detachAsData()
    }
}
// Source: src/codec/SkHdrMetadata.cpp:154-165
impl PartialEq for MasteringDisplayColorVolume {
    fn eq(&self, other: &Self) -> bool {
        self.fDisplayPrimaries.fRX == other.fDisplayPrimaries.fRX
            && self.fDisplayPrimaries.fRY == other.fDisplayPrimaries.fRY
            && self.fDisplayPrimaries.fGX == other.fDisplayPrimaries.fGX
            && self.fDisplayPrimaries.fGY == other.fDisplayPrimaries.fGY
            && self.fDisplayPrimaries.fBX == other.fDisplayPrimaries.fBX
            && self.fDisplayPrimaries.fBY == other.fDisplayPrimaries.fBY
            && self.fDisplayPrimaries.fWX == other.fDisplayPrimaries.fWX
            && self.fDisplayPrimaries.fWY == other.fDisplayPrimaries.fWY
            && self.fMaximumDisplayMasteringLuminance == other.fMaximumDisplayMasteringLuminance
            && self.fMinimumDisplayMasteringLuminance == other.fMinimumDisplayMasteringLuminance
    }
    // Source: include/private/SkHdrMetadata.h:137-139
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Metadata {
    // Source: src/codec/SkHdrMetadata.cpp:166-168
    pub fn MakeEmpty() -> Self {
        Self::default()
    }
    // Source: src/codec/SkHdrMetadata.cpp:171-179
    /// Safety: a nonnull output points to independent writable native metadata storage.
    pub unsafe fn getContentLightLevelInformation(
        &self,
        output: *mut ContentLightLevelInformation,
    ) -> bool {
        let Some(value) = &self.fContentLightLevelInformation else {
            return false;
        };
        if !output.is_null() {
            unsafe {
                *output = *value;
            }
        }
        true
    }
    // Source: src/codec/SkHdrMetadata.cpp:181-189
    /// Safety: a nonnull output points to independent writable native metadata storage.
    pub unsafe fn getMasteringDisplayColorVolume(
        &self,
        output: *mut MasteringDisplayColorVolume,
    ) -> bool {
        let Some(value) = &self.fMasteringDisplayColorVolume else {
            return false;
        };
        if !output.is_null() {
            unsafe {
                *output = *value;
            }
        }
        true
    }
    // Source: src/codec/SkHdrMetadata.cpp:191-199
    /// Safety: a nonnull output points to independent initialized metadata storage.
    pub unsafe fn getAdaptiveGlobalToneMap(&self, output: *mut AdaptiveGlobalToneMap) -> bool {
        let Some(value) = &self.fAdaptiveGlobalToneMap else {
            return false;
        };
        if !output.is_null() {
            unsafe {
                (*output).clone_from(value);
            }
        }
        true
    }
    // Source: src/codec/SkHdrMetadata.cpp:201-204
    /// Safety: stored AGTM floating values satisfy the native serialization
    /// float-to-integer domain required by its actual serializer.
    pub unsafe fn getSerializedAgtm(&self) -> Option<Arc<SkData>> {
        self.fAdaptiveGlobalToneMap
            .as_ref()
            .and_then(|value| unsafe { value.serialize() })
    }
    // Source: src/codec/SkHdrMetadata.cpp:206-208
    pub fn setMasteringDisplayColorVolume(&mut self, value: &MasteringDisplayColorVolume) {
        self.fMasteringDisplayColorVolume = Some(*value);
    }
    // Source: src/codec/SkHdrMetadata.cpp:210-212
    pub fn setContentLightLevelInformation(&mut self, value: &ContentLightLevelInformation) {
        self.fContentLightLevelInformation = Some(*value);
    }
    // Source: src/codec/SkHdrMetadata.cpp:214-216
    pub fn setAdaptiveGlobalToneMap(&mut self, value: &AdaptiveGlobalToneMap) {
        if let Some(current) = self.fAdaptiveGlobalToneMap.as_mut() {
            current.clone_from(value);
        } else {
            self.fAdaptiveGlobalToneMap = Some(value.clone());
        }
    }
    // Source: src/codec/SkHdrMetadata.cpp:218-225
    /// Safety: the encoded nullable data obeys the actual AGTM parse input lease.
    pub unsafe fn setSerializedAgtm(&mut self, encoded: Option<Arc<SkData>>) {
        let mut agtm = AdaptiveGlobalToneMap::default();
        // Use the complete Data object, not its payload pointer.
        let data = encoded
            .as_deref()
            .map_or(core::ptr::null(), |data| data as *const SkData);
        if unsafe { agtm.parse(data) } {
            self.setAdaptiveGlobalToneMap(&agtm);
        } else {
            self.fAdaptiveGlobalToneMap = None;
        }
    }
    // Source: src/codec/SkHdrMetadata.cpp:227-234
    pub fn toString(&self) -> SkString {
        // Native temporary strings live through the entire printf expression.
        let clli = self
            .fContentLightLevelInformation
            .as_ref()
            .map(|value| value.toString());
        let mdcv = self
            .fMasteringDisplayColorVolume
            .as_ref()
            .map(|value| value.toString());
        let agtm = self
            .fAdaptiveGlobalToneMap
            .as_ref()
            .map(|value| value.toString());
        unsafe {
            SkStringPrintf(
                b"{clli:%s, mdcv:%s, agtm:%s}\0".as_ptr(),
                clli.as_ref()
                    .map_or(b"None\0".as_ptr(), |value| value.c_str()),
                mdcv.as_ref()
                    .map_or(b"None\0".as_ptr(), |value| value.c_str()),
                agtm.as_ref()
                    .map_or(b"None\0".as_ptr(), |value| value.c_str()),
            )
        }
    }
    // Source: src/codec/SkHdrMetadata.cpp:242-256
    /// Safety: headroom and gain metadata obey the actual AGTM image generation
    /// input domain; pixel conversion failures propagate as RasterError.
    pub unsafe fn makeToneMapColorFilter(
        &self,
        targetedHdrHeadroom: f32,
        inputColorSpace: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
    ) -> Result<Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>, crate::RasterError>
    {
        let mut agtm = AdaptiveGlobalToneMap::default();
        let mut scaleFactor = 1.0;
        if !crate::src::codec::SkHdrAgtmPriv::AgtmHelpers::PopulateToneMapAgtmParams(
            self,
            inputColorSpace,
            Some(&mut agtm),
            Some(&mut scaleFactor),
        ) {
            return Ok(None);
        }
        let Some(hatm) = agtm.fHeadroomAdaptiveToneMap.as_ref() else {
            return Ok(None);
        };
        unsafe {
            crate::src::codec::SkHdrAgtmPriv::AgtmHelpers::MakeColorFilter(
                hatm,
                targetedHdrHeadroom,
                scaleFactor,
            )
        }
    }
    // Source: include/private/SkHdrMetadata.h:341-342
    pub unsafe fn makeToneMapColorFilter_default(
        &self,
        targetedHdrHeadroom: f32,
    ) -> Result<Option<Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>, crate::RasterError>
    {
        unsafe { self.makeToneMapColorFilter(targetedHdrHeadroom, None) }
    }
}
// Source: src/codec/SkHdrMetadata.cpp:236-240
impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.fContentLightLevelInformation == other.fContentLightLevelInformation
            && self.fMasteringDisplayColorVolume == other.fMasteringDisplayColorVolume
            && self.fAdaptiveGlobalToneMap == other.fAdaptiveGlobalToneMap
    }
    // Source: include/private/SkHdrMetadata.h:345-347
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
// AGTM ordinary methods are supplied by actual SkHdrAgtm.cpp and
// SkHdrAgtmParse.cpp, with helper bodies in their canonical codec modules.

// Actual implicit special-member language mapping. Supported stdlib domain is
// the repository libc++ default std::allocator vector implementation:
// third_party/libc++/src/include/__vector/vector.h:913-920,945-950;
// __vector/layout.h:236-241,280-286,348-353,392-398;
// __memory/allocator.h:75. Both real layout branches leave source size/capacity
// empty. No promise is made for a different stateful allocator or stdlib.
fn native_vector_move<T>(source: &mut Vec<T>) -> Vec<T> {
    core::mem::take(source)
}
fn native_vector_clear<T>(target: &mut Vec<T>) {
    // libc++ destroys from end before deallocating, unlike Rust Vec's ordinary
    // forward element drop. No extra heap buffer or element Clone is used.
    while let Some(value) = target.pop() {
        drop(value);
    }
    drop(core::mem::replace(target, Vec::new()));
}
fn native_vector_assign<T>(target: &mut Vec<T>, source: &mut Vec<T>) {
    native_vector_clear(target);
    *target = native_vector_move(source);
}
// libc++ optional::__construct_from/__assign_from (optional:460-476) never
// disengage the source: recurse into its existing object, including Some/Some.
fn native_optional_move<T>(source: &mut Option<T>, ctor: fn(&mut T) -> T) -> Option<T> {
    source.as_mut().map(ctor)
}
fn native_optional_assign<T>(
    target: &mut Option<T>,
    source: &mut Option<T>,
    ctor: fn(&mut T) -> T,
    assign: fn(&mut T, &mut T),
    destroy_members: fn(&mut T),
) {
    if let Some(value) = source.as_mut() {
        if let Some(existing) = target.as_mut() {
            assign(existing, value);
        } else {
            *target = Some(ctor(value));
        }
    } else {
        if let Some(existing) = target.as_mut() {
            destroy_members(existing);
        }
        *target = None;
    }
}
macro_rules! native_move_raw {
    ($ty:ty) => {
        impl $ty {
            /// Safety: complete exclusive live objects; equal pointers select
            /// native self-move without constructing two aliasing mutable borrows.
            pub unsafe fn move_assign_raw(
                dst: core::ptr::NonNull<Self>,
                src: core::ptr::NonNull<Self>,
            ) {
                if dst == src {
                    unsafe {
                        (&mut *dst.as_ptr()).self_move_members();
                    }
                } else {
                    unsafe {
                        (&mut *dst.as_ptr()).move_assign(&mut *src.as_ptr());
                    }
                }
            }
        }
    };
}
// Source: include/private/SkHdrMetadata.h:152-168
impl GainCurve {
    pub fn move_from(source: &mut Self) -> Self {
        Self {
            fControlPoints: native_vector_move(&mut source.fControlPoints),
        }
    }
    pub fn move_assign(&mut self, source: &mut Self) {
        native_vector_assign(&mut self.fControlPoints, &mut source.fControlPoints);
    }
    fn self_move_members(&mut self) {
        native_vector_clear(&mut self.fControlPoints);
    }
}
native_move_raw!(GainCurve);
// Source: include/private/SkHdrMetadata.h:192-197
impl ColorGainFunction {
    pub fn move_from(source: &mut Self) -> Self {
        let fComponentMixing = source.fComponentMixing;
        let fGainCurve = GainCurve::move_from(&mut source.fGainCurve);
        Self {
            fComponentMixing,
            fGainCurve,
        }
    }
    pub fn move_assign(&mut self, source: &mut Self) {
        self.fComponentMixing = source.fComponentMixing;
        self.fGainCurve.move_assign(&mut source.fGainCurve);
    }
    fn self_move_members(&mut self) {
        self.fGainCurve.self_move_members();
    }
}
native_move_raw!(ColorGainFunction);
// Source: include/private/SkHdrMetadata.h:205-210
impl AlternateImage {
    pub fn move_from(source: &mut Self) -> Self {
        let fHdrHeadroom = source.fHdrHeadroom;
        let fColorGainFunction = ColorGainFunction::move_from(&mut source.fColorGainFunction);
        Self {
            fHdrHeadroom,
            fColorGainFunction,
        }
    }
    pub fn move_assign(&mut self, source: &mut Self) {
        self.fHdrHeadroom = source.fHdrHeadroom;
        self.fColorGainFunction
            .move_assign(&mut source.fColorGainFunction);
    }
    fn self_move_members(&mut self) {
        self.fColorGainFunction.self_move_members();
    }
}
native_move_raw!(AlternateImage);
// Source: include/private/SkHdrMetadata.h:218-228
impl HeadroomAdaptiveToneMap {
    pub fn move_from(source: &mut Self) -> Self {
        let fBaselineHdrHeadroom = source.fBaselineHdrHeadroom;
        let fGainApplicationSpacePrimaries = source.fGainApplicationSpacePrimaries;
        let fAlternateImages = native_vector_move(&mut source.fAlternateImages);
        Self {
            fBaselineHdrHeadroom,
            fGainApplicationSpacePrimaries,
            fAlternateImages,
        }
    }
    pub fn move_assign(&mut self, source: &mut Self) {
        self.fBaselineHdrHeadroom = source.fBaselineHdrHeadroom;
        self.fGainApplicationSpacePrimaries = source.fGainApplicationSpacePrimaries;
        native_vector_assign(&mut self.fAlternateImages, &mut source.fAlternateImages);
    }
    fn self_move_members(&mut self) {
        native_vector_clear(&mut self.fAlternateImages);
    }
}
native_move_raw!(HeadroomAdaptiveToneMap);
// Source: include/private/SkHdrMetadata.h:240-244
impl AdaptiveGlobalToneMap {
    pub fn move_from(source: &mut Self) -> Self {
        let fHdrReferenceWhite = source.fHdrReferenceWhite;
        let fHeadroomAdaptiveToneMap = native_optional_move(
            &mut source.fHeadroomAdaptiveToneMap,
            HeadroomAdaptiveToneMap::move_from,
        );
        Self {
            fHdrReferenceWhite,
            fHeadroomAdaptiveToneMap,
        }
    }
    pub fn move_assign(&mut self, source: &mut Self) {
        self.fHdrReferenceWhite = source.fHdrReferenceWhite;
        native_optional_assign(
            &mut self.fHeadroomAdaptiveToneMap,
            &mut source.fHeadroomAdaptiveToneMap,
            HeadroomAdaptiveToneMap::move_from,
            HeadroomAdaptiveToneMap::move_assign,
            HeadroomAdaptiveToneMap::self_move_members,
        );
    }
    fn self_move_members(&mut self) {
        if let Some(value) = self.fHeadroomAdaptiveToneMap.as_mut() {
            value.self_move_members();
        }
    }
}
native_move_raw!(AdaptiveGlobalToneMap);
// Source: include/private/SkHdrMetadata.h:350-352
impl Metadata {
    pub fn move_from(source: &mut Self) -> Self {
        let fContentLightLevelInformation = source.fContentLightLevelInformation;
        let fMasteringDisplayColorVolume = source.fMasteringDisplayColorVolume;
        let fAdaptiveGlobalToneMap = native_optional_move(
            &mut source.fAdaptiveGlobalToneMap,
            AdaptiveGlobalToneMap::move_from,
        );
        Self {
            fContentLightLevelInformation,
            fMasteringDisplayColorVolume,
            fAdaptiveGlobalToneMap,
        }
    }
    pub fn move_assign(&mut self, source: &mut Self) {
        self.fContentLightLevelInformation = source.fContentLightLevelInformation;
        self.fMasteringDisplayColorVolume = source.fMasteringDisplayColorVolume;
        native_optional_assign(
            &mut self.fAdaptiveGlobalToneMap,
            &mut source.fAdaptiveGlobalToneMap,
            AdaptiveGlobalToneMap::move_from,
            AdaptiveGlobalToneMap::move_assign,
            AdaptiveGlobalToneMap::self_move_members,
        );
    }
    pub(crate) fn self_move_members(&mut self) {
        if let Some(value) = self.fAdaptiveGlobalToneMap.as_mut() {
            value.self_move_members();
        }
    }
}
native_move_raw!(Metadata);
