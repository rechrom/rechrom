//! gfx::ColorSpace and cc paint-image values named by both compositor halves.
//! cc/paint (TargetColorParams, PaintImage::DecodingMode) is used on the main
//! and the impl thread in Chromium; these are plain values.

use std::cmp::Ordering;

// Source vocabulary: ui/gfx/color_space.h:50-153,413-429.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorPrimaryId {
    Invalid = 0,
    Bt709,
    Bt470M,
    Bt470Bg,
    Smpte170M,
    Smpte240M,
    Film,
    Bt2020,
    SmpteSt428_1,
    SmpteSt431_2,
    P3,
    XyzD50,
    AdobeRgb,
    AppleGenericRgb,
    WideGamutColorSpin,
    Custom,
    Ebu3213E,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorTransferId {
    Invalid = 0,
    Bt709,
    Bt709Apple,
    Gamma18,
    Gamma22,
    Gamma24,
    Gamma28,
    Smpte170M,
    Smpte240M,
    Linear,
    Log,
    LogSqrt,
    Iec61966_2_4,
    Bt1361Ecg,
    Srgb,
    Bt2020_10,
    Bt2020_12,
    Pq,
    SmpteSt428_1,
    Hlg,
    SrgbHdr,
    LinearHdr,
    Custom,
    CustomHdr,
    ScrgbLinear80Nits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorMatrixId {
    Invalid = 0,
    Rgb,
    Bt709,
    Fcc,
    Bt470Bg,
    Smpte170M,
    Smpte240M,
    Ycocg,
    Bt2020Ncl,
    Ydzdx,
    Gbr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorRangeId {
    Invalid = 0,
    Limited,
    Full,
    Derived,
}

/// IPC-safe gfx::ColorSpace value. Floating values compare by their native bit
/// representation, matching Chromium's field-wise serialized identity.
#[derive(Clone, Debug)]
pub struct RasterColorSpace {
    pub primaries: ColorPrimaryId,
    pub transfer: ColorTransferId,
    pub matrix: ColorMatrixId,
    pub range: ColorRangeId,
    pub custom_primary_matrix: [f32; 9],
    pub transfer_params: [f32; 7],
}
impl Default for RasterColorSpace {
    fn default() -> Self {
        Self {
            primaries: ColorPrimaryId::Invalid,
            transfer: ColorTransferId::Invalid,
            matrix: ColorMatrixId::Invalid,
            range: ColorRangeId::Invalid,
            custom_primary_matrix: [0.; 9],
            transfer_params: [0.; 7],
        }
    }
}
impl RasterColorSpace {
    // Source: ui/gfx/color_space.h:185-189.
    pub fn CreateSRGB() -> Self {
        Self {
            primaries: ColorPrimaryId::Bt709,
            transfer: ColorTransferId::Srgb,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    // Source: ui/gfx/color_space.h:221-225.
    pub fn CreateHDR10() -> Self {
        Self {
            primaries: ColorPrimaryId::Bt2020,
            transfer: ColorTransferId::Pq,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    pub fn CreateDisplayP3() -> Self {
        Self {
            primaries: ColorPrimaryId::P3,
            transfer: ColorTransferId::Srgb,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    pub fn CreateDisplayP3Hdr() -> Self {
        Self {
            primaries: ColorPrimaryId::P3,
            transfer: ColorTransferId::SrgbHdr,
            matrix: ColorMatrixId::Rgb,
            range: ColorRangeId::Full,
            ..Self::default()
        }
    }
    // Source: ui/gfx/color_space.cc:205-207.
    pub fn IsValid(&self) -> bool {
        self.primaries != ColorPrimaryId::Invalid
            && self.transfer != ColorTransferId::Invalid
            && self.matrix != ColorMatrixId::Invalid
            && self.range != ColorRangeId::Invalid
    }
    // Source: ui/gfx/color_space.cc:388-395.
    pub fn IsHDR(&self) -> bool {
        matches!(
            self.transfer,
            ColorTransferId::Pq
                | ColorTransferId::Hlg
                | ColorTransferId::LinearHdr
                | ColorTransferId::SrgbHdr
                | ColorTransferId::CustomHdr
                | ColorTransferId::ScrgbLinear80Nits
        )
    }
    // Source: ui/gfx/color_space.cc:640-667.
    pub fn IsSuitableForBlending(&self) -> bool {
        match self.transfer {
            ColorTransferId::Pq
            | ColorTransferId::Hlg
            | ColorTransferId::LinearHdr
            | ColorTransferId::ScrgbLinear80Nits => false,
            ColorTransferId::CustomHdr => self.transfer_params[6] >= 1.25,
            _ => true,
        }
    }
    /// Exact known-primary fast path used by the two cc call sites whose
    /// comparison operand is always `ColorSpace::CreateSRGB()`.
    // Source: ui/gfx/color_space.cc:94-113,830-840.
    pub fn ContainsSRGB(&self) -> bool {
        matches!(
            self.primaries,
            ColorPrimaryId::Bt709
                | ColorPrimaryId::Bt2020
                | ColorPrimaryId::SmpteSt428_1
                | ColorPrimaryId::SmpteSt431_2
                | ColorPrimaryId::P3
                | ColorPrimaryId::XyzD50
                | ColorPrimaryId::AdobeRgb
                | ColorPrimaryId::WideGamutColorSpin
        )
    }
    fn transfer_param_count(&self) -> usize {
        // Source: ui/gfx/color_space.cc:320-333.
        match self.transfer {
            ColorTransferId::Custom | ColorTransferId::CustomHdr => 7,
            ColorTransferId::Pq => 1,
            ColorTransferId::Hlg => 3,
            _ => 0,
        }
    }
    fn cmp_float_prefix(lhs: &[f32], rhs: &[f32], count: usize) -> Ordering {
        lhs[..count]
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .cmp(rhs[..count].iter().flat_map(|value| value.to_ne_bytes()))
    }
}
impl PartialEq for RasterColorSpace {
    fn eq(&self, rhs: &Self) -> bool {
        self.primaries == rhs.primaries
            && self.transfer == rhs.transfer
            && self.matrix == rhs.matrix
            && self.range == rhs.range
            && (self.primaries != ColorPrimaryId::Custom
                || Self::cmp_float_prefix(
                    &self.custom_primary_matrix,
                    &rhs.custom_primary_matrix,
                    9,
                ) == Ordering::Equal)
            && Self::cmp_float_prefix(
                &self.transfer_params,
                &rhs.transfer_params,
                self.transfer_param_count(),
            ) == Ordering::Equal
    }
}
impl Eq for RasterColorSpace {}
impl PartialOrd for RasterColorSpace {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for RasterColorSpace {
    // Source: ui/gfx/color_space.cc:421-462.
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.primaries
            .cmp(&rhs.primaries)
            .then_with(|| self.transfer.cmp(&rhs.transfer))
            .then_with(|| self.matrix.cmp(&rhs.matrix))
            .then_with(|| self.range.cmp(&rhs.range))
            .then_with(|| {
                (self.primaries == ColorPrimaryId::Custom)
                    .then(|| {
                        Self::cmp_float_prefix(
                            &self.custom_primary_matrix,
                            &rhs.custom_primary_matrix,
                            9,
                        )
                    })
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| {
                Self::cmp_float_prefix(
                    &self.transfer_params,
                    &rhs.transfer_params,
                    self.transfer_param_count(),
                )
            })
    }
}

// Source dependency: cc/paint/target_color_params.h:18-35. This typed paint
// boundary retains both native fields and the source default sRGB/no headroom.
#[derive(Clone, Debug, PartialEq)]
pub struct HostTargetColorParams<C> {
    pub color_space: C,
    pub hdr_headroom: Option<f32>,
}
impl<C: HostColorSpace> Default for HostTargetColorParams<C> {
    fn default() -> Self {
        Self {
            color_space: C::CreateSRGB(),
            hdr_headroom: None,
        }
    }
}
impl<C> HostTargetColorParams<C> {
    pub fn GetHdrHeadroom(&self) -> f32 {
        self.hdr_headroom.unwrap_or(0.)
    }
}
// Color math is supplied by the existing geometry/color owner.
// Source dependency: ui/gfx/color_space.h:65-92.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HostColorSpacePrimaryID {
    INVALID = 0,
    BT709,
    BT470M,
    BT470BG,
    SMPTE170M,
    SMPTE240M,
    FILM,
    BT2020,
    SMPTEST428_1,
    SMPTEST431_2,
    P3,
    XYZ_D50,
    ADOBE_RGB,
    APPLE_GENERIC_RGB,
    WIDE_GAMUT_COLOR_SPIN,
    CUSTOM,
    EBU_3213_E,
}
impl HostColorSpacePrimaryID {
    pub const kMaxValue: Self = Self::EBU_3213_E;
}
pub trait HostColorSpace: Clone + Ord {
    fn CreateSRGB() -> Self;
    fn IsValid(&self) -> bool;
    fn IsHDR(&self) -> bool;
    fn GetPrimaryID(&self) -> HostColorSpacePrimaryID;
    /// Exact fixed argument used by HostImpl: native Contains(CreateSRGB()).
    fn ContainsSRGB(&self) -> bool;
}

// Source dependency: cc/paint/paint_image.h:219-232.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HostImageDecodingMode {
    kUnspecified = 0,
    kSync = 1,
    kAsync = 2,
}

impl HostColorSpace for RasterColorSpace {
    fn CreateSRGB() -> Self {
        RasterColorSpace::CreateSRGB()
    }
    fn IsValid(&self) -> bool {
        RasterColorSpace::IsValid(self)
    }
    fn IsHDR(&self) -> bool {
        RasterColorSpace::IsHDR(self)
    }
    fn GetPrimaryID(&self) -> HostColorSpacePrimaryID {
        match self.primaries {
            ColorPrimaryId::Invalid => HostColorSpacePrimaryID::INVALID,
            ColorPrimaryId::Bt709 => HostColorSpacePrimaryID::BT709,
            ColorPrimaryId::Bt470M => HostColorSpacePrimaryID::BT470M,
            ColorPrimaryId::Bt470Bg => HostColorSpacePrimaryID::BT470BG,
            ColorPrimaryId::Smpte170M => HostColorSpacePrimaryID::SMPTE170M,
            ColorPrimaryId::Smpte240M => HostColorSpacePrimaryID::SMPTE240M,
            ColorPrimaryId::Film => HostColorSpacePrimaryID::FILM,
            ColorPrimaryId::Bt2020 => HostColorSpacePrimaryID::BT2020,
            ColorPrimaryId::SmpteSt428_1 => HostColorSpacePrimaryID::SMPTEST428_1,
            ColorPrimaryId::SmpteSt431_2 => HostColorSpacePrimaryID::SMPTEST431_2,
            ColorPrimaryId::P3 => HostColorSpacePrimaryID::P3,
            ColorPrimaryId::XyzD50 => HostColorSpacePrimaryID::XYZ_D50,
            ColorPrimaryId::AdobeRgb => HostColorSpacePrimaryID::ADOBE_RGB,
            ColorPrimaryId::AppleGenericRgb => HostColorSpacePrimaryID::APPLE_GENERIC_RGB,
            ColorPrimaryId::WideGamutColorSpin => HostColorSpacePrimaryID::WIDE_GAMUT_COLOR_SPIN,
            ColorPrimaryId::Custom => HostColorSpacePrimaryID::CUSTOM,
            ColorPrimaryId::Ebu3213E => HostColorSpacePrimaryID::EBU_3213_E,
        }
    }
    fn ContainsSRGB(&self) -> bool {
        RasterColorSpace::ContainsSRGB(self)
    }
}
