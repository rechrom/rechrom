//! CPU raster-pipeline support for SkBlendMode.
pub use crate::include::core::SkBlendMode::{BlendMode, SkBlendMode};
use crate::raster::pipeline;
impl SkBlendMode {
    pub(crate) fn should_pre_scale_coverage(self) -> bool {
        // The most important things we do here are:
        //   1) never pre-scale with rgb coverage if the blend mode involves a source-alpha term;
        //   2) always pre-scale Plus.
        //
        // When we pre-scale with rgb coverage, we scale each of source r,g,b, with a distinct value,
        // and source alpha with one of those three values. This process destructively updates the
        // source-alpha term, so we can't evaluate blend modes that need its original value.
        //
        // Plus always requires pre-scaling as a specific quirk of its implementation in
        // RasterPipeline. This lets us put the clamp inside the blend mode itself rather
        // than as a separate stage that'd come after the lerp.
        //
        // This function is a finer-grained breakdown of SkBlendMode_SupportsCoverageAsAlpha().
        matches!(
            self,
            BlendMode::kDst |        // d              --> no sa term, ok!
            BlendMode::kDstOver |    // d + s*inv(da)  --> no sa term, ok!
            BlendMode::kPlus |               // clamp(s+d)     --> no sa term, ok!
            BlendMode::kDstOut |     // d * inv(sa)
            BlendMode::kSrcATop |         // s*da + d*inv(sa)
            BlendMode::kSrcOver |         // s + d*inv(sa)
            BlendMode::kXor // s*inv(da) + d*inv(sa)
        )
    }

    pub(crate) fn to_stage(self) -> Option<pipeline::Stage> {
        match self {
            BlendMode::kClear => Some(pipeline::Stage::Clear),
            BlendMode::kSrc => None, // This stage is a no-op.
            BlendMode::kDst => Some(pipeline::Stage::MoveDestinationToSource),
            BlendMode::kSrcOver => Some(pipeline::Stage::SourceOver),
            BlendMode::kDstOver => Some(pipeline::Stage::DestinationOver),
            BlendMode::kSrcIn => Some(pipeline::Stage::SourceIn),
            BlendMode::kDstIn => Some(pipeline::Stage::DestinationIn),
            BlendMode::kSrcOut => Some(pipeline::Stage::SourceOut),
            BlendMode::kDstOut => Some(pipeline::Stage::DestinationOut),
            BlendMode::kSrcATop => Some(pipeline::Stage::SourceAtop),
            BlendMode::kDstATop => Some(pipeline::Stage::DestinationAtop),
            BlendMode::kXor => Some(pipeline::Stage::Xor),
            BlendMode::kPlus => Some(pipeline::Stage::Plus),
            BlendMode::kModulate => Some(pipeline::Stage::Modulate),
            BlendMode::kScreen => Some(pipeline::Stage::Screen),
            BlendMode::kOverlay => Some(pipeline::Stage::Overlay),
            BlendMode::kDarken => Some(pipeline::Stage::Darken),
            BlendMode::kLighten => Some(pipeline::Stage::Lighten),
            BlendMode::kColorDodge => Some(pipeline::Stage::ColorDodge),
            BlendMode::kColorBurn => Some(pipeline::Stage::ColorBurn),
            BlendMode::kHardLight => Some(pipeline::Stage::HardLight),
            BlendMode::kSoftLight => Some(pipeline::Stage::SoftLight),
            BlendMode::kDifference => Some(pipeline::Stage::Difference),
            BlendMode::kExclusion => Some(pipeline::Stage::Exclusion),
            BlendMode::kMultiply => Some(pipeline::Stage::Multiply),
            BlendMode::kHue => Some(pipeline::Stage::Hue),
            BlendMode::kSaturation => Some(pipeline::Stage::Saturation),
            BlendMode::kColor => Some(pipeline::Stage::Color),
            BlendMode::kLuminosity => Some(pipeline::Stage::Luminosity),
        }
    }
}
