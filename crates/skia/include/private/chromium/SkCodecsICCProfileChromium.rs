use crate::modules::skcms::src::skcms_public::skcms_ICCProfile;
use crate::src::core::SkData::SkData;
use std::sync::Arc;
// Source: include/private/chromium/SkCodecsICCProfileChromium.h:18-49
pub mod SkCodecs {
    use super::*;
    // Source: include/private/chromium/SkCodecsICCProfileChromium.h:22-24
    // Actual abstract interface and dynamically correct Box destruction. Native
    // default destructor/constructor have no fields or other parent state.
    pub trait ICCProfileChromium {
        // Source: include/private/chromium/SkCodecsICCProfileChromium.h:40-40
        fn GetProfile(&self) -> &skcms_ICCProfile;
    }
    impl dyn ICCProfileChromium {
        // Source: include/private/chromium/SkCodecsICCProfileChromium.h:32-32
        /// Safety: startup configuration excludes concurrent codec operations.
        pub unsafe fn ForceSkcms(forceSkcms: bool) {
            unsafe {
                crate::src::codec::SkCodecColorProfile::ForceSkcms(forceSkcms);
            }
        }
        // Source: include/private/chromium/SkCodecsICCProfileChromium.h:36-36
        pub fn Make(data: Option<Arc<SkData>>) -> Option<Box<dyn ICCProfileChromium>> {
            crate::src::codec::SkCodecColorProfile::MakeICCProfileChromium(data)
        }
    }
}
