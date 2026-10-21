use super::SkCodecPriv::SkCodecs::{ColorProfile, DataSpace};
use crate::include::private::chromium::SkCodecsICCProfileChromium::SkCodecs::ICCProfileChromium;
use crate::modules::skcms::src::skcms_public::*;
use crate::src::core::{
    SkColorSpace::{SkColorSpace, SkNamedPrimaries, SkNamedTransferFn},
    SkData::SkData,
};
use std::sync::Arc;
// Native ordinary-static declaration correspondence (one existing inherent body
// per method below; no second declaration/provider or algorithm is introduced):
// Source: src/codec/SkCodecPriv.h:41-41
// ColorProfile::MakeICCProfile -> existing MakeICCProfile implementation.
// Source: src/codec/SkCodecPriv.h:45-45
// ColorProfile::MakeICCProfileWithSkCMS -> existing same-name implementation.
// Source: src/codec/SkCodecPriv.h:49-49
// ColorProfile::Make(sk_sp<SkColorSpace>) -> existing Make implementation.
// Source: src/codec/SkCodecPriv.h:53-54
// ColorProfile::Make(transfer,matrix) -> existing MakeRGB overload spelling.
// Source: src/codec/SkCodecPriv.h:58-61
// ColorProfile::MakeCICP -> existing same-name implementation.
// Source: src/codec/SkCodecPriv.h:81-81
// ColorProfile::getAndroidOutputColorSpace -> existing &self/nullable sk_sp result.
// Source: src/codec/SkCodecColorProfile.cpp:17-17
static mut gForceSkcmsForICCProfiles: bool = true;
// Source: src/codec/SkCodecColorProfile.cpp:23-58
fn cicp_get_android_sk_color_space(
    color_primaries: u8,
    transfer_characteristics: u8,
    matrix_coefficients: u8,
    full_range_flag: u8,
) -> Option<Arc<SkColorSpace>> {
    if matrix_coefficients != 0 {
        return None;
    }
    if full_range_flag != 1 {
        return None;
    }
    // Language validity only: these POD outputs must be valid before borrowing
    // for the existing typed setters. Failure never observes their initial bits.
    let mut primaries = skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    if !SkNamedPrimaries::GetCicpRaw(color_primaries, &mut primaries) {
        return None;
    }
    let mut trfn = skcms_TransferFunction {
        g: 0.0,
        a: 0.0,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: 0.0,
        f: 0.0,
    };
    if !SkNamedTransferFn::GetCicpRaw(transfer_characteristics, &mut trfn) {
        return None;
    }
    match transfer_characteristics {
        16 => {
            trfn = skcms_TransferFunction {
                g: -2.0,
                a: -1.55522297832,
                b: 1.86045365631,
                c: 32.0 / 2523.0,
                d: 2413.0 / 128.0,
                e: -2392.0 / 128.0,
                f: 8192.0 / 1305.0,
            };
        }
        18 => {
            let _ = skcms_TransferFunction_makeScaledHLGish(
                &mut trfn,
                0.314509843,
                2.0,
                2.0,
                1.0 / 0.17883277,
                0.28466892,
                0.55991073,
            );
        }
        _ => {}
    }
    SkColorSpace::MakeRGB(&trfn, &primaries)
}
impl ColorProfile {
    // Source: src/codec/SkCodecColorProfile.cpp:62-71
    pub fn MakeICCProfileWithSkCMS(data: Option<Arc<SkData>>) -> Option<Box<Self>> {
        if let Some(data) = data {
            let mut profile = core::mem::MaybeUninit::<skcms_ICCProfile>::uninit();
            if unsafe { skcms_Parse(data.data().cast(), data.size(), profile.as_mut_ptr()) } {
                return Some(Box::new(Self::new(
                    unsafe { profile.assume_init() },
                    Some(data),
                )));
            }
        }
        None
    }
}
// Source: src/codec/SkCodecColorProfile.cpp:73-75
/// Safety: native setting is not thread-safe; exclude concurrent profile parsing.
pub unsafe fn ForceSkcms(forceSkcms: bool) {
    unsafe {
        gForceSkcmsForICCProfiles = forceSkcms;
    }
}
impl ColorProfile {
    // Source: src/codec/SkCodecColorProfile.cpp:77-87
    pub fn MakeICCProfile(data: Option<Arc<SkData>>) -> Option<Box<Self>> {
        if unsafe { gForceSkcmsForICCProfiles } {
            return Self::MakeICCProfileWithSkCMS(data.clone());
        }
        // Source: src/codec/SkCodecColorProfile.cpp:83-83
        #[cfg(SK_CODEC_COLOR_PROFILE_PARSE_WITH_RUST)]
        {
            super::SkCodecColorProfileRust::MakeICCProfileWithRust(data.clone())
        }
        #[cfg(not(SK_CODEC_COLOR_PROFILE_PARSE_WITH_RUST))]
        {
            Self::MakeICCProfileWithSkCMS(data.clone())
        }
    }
    // Source: src/codec/SkCodecColorProfile.cpp:89-96
    pub fn Make(cs: Option<Arc<SkColorSpace>>) -> Option<Box<Self>> {
        let cs = cs?;
        let mut profile =
            unsafe { core::mem::MaybeUninit::<skcms_ICCProfile>::zeroed().assume_init() };
        cs.toProfile(&mut profile);
        Some(Box::new(Self::new(profile, None)))
    }
    // Source: src/codec/SkCodecColorProfile.cpp:98-106
    pub fn MakeRGB(trfn: &skcms_TransferFunction, toXYZD50: &skcms_Matrix3x3) -> Box<Self> {
        let mut profile =
            unsafe { core::mem::MaybeUninit::<skcms_ICCProfile>::zeroed().assume_init() };
        skcms_Init(&mut profile);
        skcms_SetTransferFunction(&mut profile, trfn);
        skcms_SetXYZD50(&mut profile, toXYZD50);
        Box::new(Self::new(profile, None))
    }
    // Source: src/codec/SkCodecColorProfile.cpp:108-121
    pub fn MakeCICP(
        color_primaries: u8,
        transfer_characteristics: u8,
        matrix_coefficients: u8,
        full_range_flag: u8,
    ) -> Box<Self> {
        let mut profile =
            unsafe { core::mem::MaybeUninit::<skcms_ICCProfile>::zeroed().assume_init() };
        skcms_Init(&mut profile);
        profile.has_CICP = true;
        profile.CICP.color_primaries = color_primaries;
        profile.CICP.transfer_characteristics = transfer_characteristics;
        profile.CICP.matrix_coefficients = matrix_coefficients;
        profile.CICP.video_full_range_flag = full_range_flag;
        Box::new(Self::new(profile, None))
    }
    // Source: src/codec/SkCodecColorProfile.cpp:123-127
    pub fn clone(&self) -> Box<Self> {
        let mut cloned = Box::new(Self::new(self.fProfile, self.fData.clone()));
        cloned.fRetainedData = self.fRetainedData.clone();
        cloned
    }
    // Source: src/codec/SkCodecColorProfile.cpp:129-140
    pub fn dataSpace(&self) -> DataSpace {
        match self.fProfile.data_color_space {
            x if x == skcms_Signature::skcms_Signature_RGB as u32 => DataSpace::kRGB,
            x if x == skcms_Signature::skcms_Signature_CMYK as u32 => DataSpace::kCMYK,
            x if x == skcms_Signature::skcms_Signature_Gray as u32 => DataSpace::kGray,
            _ => DataSpace::kOther,
        }
    }
    // Source: src/codec/SkCodecColorProfile.cpp:142-144
    pub fn getExactColorSpace(&self) -> Option<Arc<SkColorSpace>> {
        unsafe { SkColorSpace::Make(&self.fProfile) }
    }
    // Source: src/codec/SkCodecColorProfile.cpp:146-168
    pub fn getAndroidOutputColorSpace(&self) -> Option<Arc<SkColorSpace>> {
        if self.fProfile.has_CICP {
            let c = &self.fProfile.CICP;
            if let Some(cs) = cicp_get_android_sk_color_space(
                c.color_primaries,
                c.transfer_characteristics,
                c.matrix_coefficients,
                c.video_full_range_flag,
            ) {
                return Some(cs);
            }
        }
        if let Some(encodedSpace) = unsafe { SkColorSpace::Make(&self.fProfile) } {
            return Some(encodedSpace);
        }
        if self.fProfile.has_toXYZD50 {
            return SkColorSpace::MakeRGB(&SkNamedTransferFn::kSRGB, &self.fProfile.toXYZD50);
        }
        Some(SkColorSpace::MakeSRGB())
    }
    // Source: src/codec/SkCodecColorProfile.cpp:170-173
    // Native private ctor is exposed only crate-wide for the real Rust-parser friend.
    pub(crate) fn new(profile: skcms_ICCProfile, data: Option<Arc<SkData>>) -> Self {
        let fProfile = profile;
        let fData = data;
        let fRetainedData = None;
        Self {
            fRetainedData,
            fData,
            fProfile,
        }
    }
}
// Source: src/codec/SkCodecColorProfile.cpp:175-186
struct ICCProfileChromiumImpl {
    fICCProfile: Box<ColorProfile>,
}
impl ICCProfileChromiumImpl {
    fn new(colorProfile: Box<ColorProfile>) -> Self {
        Self {
            fICCProfile: colorProfile,
        }
    }
}
impl ICCProfileChromium for ICCProfileChromiumImpl {
    fn GetProfile(&self) -> &skcms_ICCProfile {
        unsafe { &*self.fICCProfile.profile() }
    }
}
// Source: src/codec/SkCodecColorProfile.cpp:190-196
pub fn MakeICCProfileChromium(data: Option<Arc<SkData>>) -> Option<Box<dyn ICCProfileChromium>> {
    let profile = ColorProfile::MakeICCProfile(data.clone())?;
    Some(Box::new(ICCProfileChromiumImpl::new(profile)))
}
// The selected Rust-parser branch above calls the actual existing producer in
// SkCodecColorProfileRust.rs; no parser or fallback is reimplemented here.
