// Copyright 2025 Google LLC. BSD-style license in LICENSE.
// Source: src/codec/SkCodecColorProfileRust.h:8-24.
use super::SkCodecPriv::SkCodecs::ColorProfile;
use crate::modules::skcms::src::skcms_public::skcms_ICCProfile;
use crate::rust::icc::{FFI_cpp, FFI};
use crate::src::core::SkData::SkData;
use std::sync::Arc;
// Source: src/codec/SkCodecColorProfileRust.cpp:14-41.
pub fn MakeICCProfileWithRust(data: Option<Arc<SkData>>) -> Option<Box<ColorProfile>> {
    if let Some(data) = data {
        let mut rust_profile = FFI_cpp::empty_icc_profile();
        // SkData owns these bytes for the complete synchronous parser call.
        let bytes = unsafe { core::slice::from_raw_parts(data.data().cast::<u8>(), data.size()) };
        if FFI::parse_icc_profile(bytes, &mut rust_profile) {
            // Moving Vec headers does not move their allocations. This same retained profile
            // supplies every borrowed skcms table/grid pointer until ColorProfile destruction.
            let retained = Arc::new(rust_profile);
            let mut profile = core::mem::MaybeUninit::<skcms_ICCProfile>::uninit();
            if !unsafe { FFI_cpp::ToSkcmsIccProfile(retained.as_ref(), profile.as_mut_ptr()) } {
                return None;
            }
            let mut result = Box::new(ColorProfile::new(
                unsafe { profile.assume_init() },
                Some(data),
            ));
            result.fRetainedData = Some(retained);
            return Some(result);
        }
    }
    None
}
