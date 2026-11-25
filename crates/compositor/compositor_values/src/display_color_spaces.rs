//! gfx::DisplayColorSpaces, named by cc (LayerTreeHostImpl raster color
//! space selection) and viz (CompositorFrameTransitionDirective).
use crate::color_space::RasterColorSpace;
use crate::shared_image_format::{SharedImageFormat, SinglePlaneFormat};
use crate::surface_id::ContentColorUsage;

/// Immutable native gfx::DisplayColorSpaces value. The six entries retain the
/// source usage/alpha indexing; callers can inject an already-resolved display
/// value without moving display policy into cc.
// Source: ui/gfx/display_color_spaces.h:49-159.
#[derive(Clone, Debug, PartialEq)]
pub struct RasterDisplayColorSpaces {
    color_spaces: [RasterColorSpace; 6],
    formats: [SharedImageFormat; 6],
    pub primaries: [f32; 8],
    pub sdr_max_luminance_nits: f32,
    pub hdr_max_luminance_relative: f32,
}
impl Default for RasterDisplayColorSpaces {
    // Source: ui/gfx/display_color_spaces.cc:17-56.
    fn default() -> Self {
        #[cfg(any(target_os = "macos", feature = "chromeos"))]
        let format = SinglePlaneFormat::BGRA_8888;
        #[cfg(not(any(target_os = "macos", feature = "chromeos")))]
        let format = SinglePlaneFormat::RGBA_8888;
        Self {
            color_spaces: std::array::from_fn(|_| RasterColorSpace::CreateSRGB()),
            formats: [format; 6],
            // SkNamedPrimariesExt::kSRGB / Rec709.
            primaries: [0.64, 0.33, 0.30, 0.60, 0.15, 0.06, 0.3127, 0.3290],
            sdr_max_luminance_nits: 203.,
            hdr_max_luminance_relative: 1.,
        }
    }
}
impl RasterDisplayColorSpaces {
    fn index(usage: ContentColorUsage, needs_alpha: bool) -> usize {
        let base = match usage {
            ContentColorUsage::kSRGB => 0,
            ContentColorUsage::kWideColorGamut => 2,
            ContentColorUsage::kHDR => 4,
        };
        base + usize::from(needs_alpha)
    }
    pub fn SetOutputColorSpaceAndFormat(
        &mut self,
        usage: ContentColorUsage,
        needs_alpha: bool,
        color_space: RasterColorSpace,
        format: SharedImageFormat,
    ) {
        let index = Self::index(usage, needs_alpha);
        self.color_spaces[index] = color_space;
        self.formats[index] = format;
    }
    pub fn GetOutputColorSpace(
        &self,
        usage: ContentColorUsage,
        needs_alpha: bool,
    ) -> RasterColorSpace {
        self.color_spaces[Self::index(usage, needs_alpha)].clone()
    }
    pub fn GetOutputFormat(
        &self,
        usage: ContentColorUsage,
        needs_alpha: bool,
    ) -> SharedImageFormat {
        self.formats[Self::index(usage, needs_alpha)]
    }
    // Source: ui/gfx/display_color_spaces.cc:157-161.
    pub fn SupportsHDR(&self) -> bool {
        self.GetOutputColorSpace(ContentColorUsage::kHDR, false)
            .IsHDR()
            || self
                .GetOutputColorSpace(ContentColorUsage::kHDR, true)
                .IsHDR()
            || self.hdr_max_luminance_relative > 1.
    }
    // Source: ui/gfx/display_color_spaces.cc:163-165.
    pub fn GetHdrHeadroom(&self) -> f32 {
        self.hdr_max_luminance_relative.log2()
    }
    // Source: ui/gfx/display_color_spaces.cc:116-155.
    pub fn GetRasterAndCompositeColorSpace(
        &self,
        mut usage: ContentColorUsage,
    ) -> RasterColorSpace {
        if !self.SupportsHDR() && usage == ContentColorUsage::kHDR {
            usage = ContentColorUsage::kWideColorGamut;
        }
        let result = self.GetOutputColorSpace(usage, true);
        if result.IsSuitableForBlending() {
            return result;
        }
        match usage {
            ContentColorUsage::kSRGB => RasterColorSpace::CreateSRGB(),
            ContentColorUsage::kHDR => RasterColorSpace::CreateDisplayP3Hdr(),
            ContentColorUsage::kWideColorGamut => RasterColorSpace::CreateDisplayP3(),
        }
    }
}
