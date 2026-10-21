// Source: src/codec/SkHdrAgtmPriv.h:24-124
// Header namespace identity; implementation remains at its real cpp target.
pub mod AgtmHelpers {
    // Source: src/codec/SkHdrAgtmPriv.h:60-68
    #[derive(Clone, Copy, Debug)]
    pub struct Weighting {
        pub fAlternateImageIndex: [u8; 2],
        pub fWeight: [f32; 2],
    }
    impl Weighting {
        pub const kInvalidIndex: u8 = 255;
    }
    impl Default for Weighting {
        fn default() -> Self {
            Self {
                fAlternateImageIndex: [255; 2],
                fWeight: [0.0; 2],
            }
        }
    }

    pub use crate::src::codec::SkHdrAgtm::{
        ApplyGain, ComputeWeighting, EvaluateColorGainFunction, EvaluateComponentMixingFunction,
        EvaluateGainCurve, GetGainApplicationSpace, MakeColorFilter, MakeGainCurveXYMImage,
        PopulateSlopeFromPCHIP, PopulateToneMapAgtmParams, PopulateUsingRwtmo, Validate,
        ValidateHeadroom,
    };
}
