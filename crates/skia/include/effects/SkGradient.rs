// Copyright 2025 Google LLC. BSD-style license.
use crate::include::core::SkTileMode::SkTileMode;
use crate::src::core::SkColor::SkColor4f;
use crate::src::core::SkColorSpace::SkColorSpace;
use std::sync::Arc;

// Source: include/effects/SkGradient.h:21-23
// Rust modules express the C++ nested enum namespace without changing discriminants.
// Source: include/effects/SkGradient.h:24-24
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InPremul {
    kNo = 0,
    kYes = 1,
}
// Source: include/effects/SkGradient.h:26-55
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorSpace {
    kDestination,
    kSRGBLinear,
    kLab,
    kOKLab,
    kOKLabGamutMap,
    kLCH,
    kOKLCH,
    kOKLCHGamutMap,
    kSRGB,
    kHSL,
    kHWB,
    kDisplayP3,
    kRec2020,
    kProphotoRGB,
    kA98RGB,
}
// Source: include/effects/SkGradient.h:54-54
impl ColorSpace {
    pub const kLastColorSpace: Self = Self::kA98RGB;
}
// Source: include/effects/SkGradient.h:56-56
pub const kColorSpaceCount: usize = ColorSpace::kA98RGB as usize + 1;
// Source: include/effects/SkGradient.h:58-66
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HueMethod {
    kShorter,
    kLonger,
    kIncreasing,
    kDecreasing,
}
// Source: include/effects/SkGradient.h:65-65
impl HueMethod {
    pub const kLastHueMethod: Self = Self::kDecreasing;
}
// Source: include/effects/SkGradient.h:67-67
pub const kHueMethodCount: usize = HueMethod::kDecreasing as usize + 1;
// Source: include/effects/SkGradient.h:69-71
#[derive(Clone, Copy, Debug)]
pub struct Interpolation {
    pub fInPremul: InPremul,
    pub fColorSpace: ColorSpace,
    pub fHueMethod: HueMethod,
}
impl Default for Interpolation {
    fn default() -> Self {
        Self {
            fInPremul: InPremul::kNo,
            fColorSpace: ColorSpace::kDestination,
            fHueMethod: HueMethod::kShorter,
        }
    }
}
// Source: include/effects/SkGradient.h:74-78
impl Interpolation {
    pub fn FromFlags(flags: u32) -> Self {
        Self {
            fInPremul: if flags & 1 != 0 {
                InPremul::kYes
            } else {
                InPremul::kNo
            },
            fColorSpace: ColorSpace::kDestination,
            fHueMethod: HueMethod::kShorter,
        }
    }
}

// Source: include/effects/SkGradient.h:96-97
// Source: include/effects/SkGradient.h:120-125
// Borrowed stop spans keep the caller's lifetime; Arc retains the original sk_sp color space owner.
#[derive(Clone)]
pub struct Colors<'a> {
    fColors: &'a [SkColor4f],
    fPos: &'a [f32],
    fColorSpace: Option<Arc<SkColorSpace>>,
    fTileMode: SkTileMode,
}
// Source: include/effects/SkGradient.h:98-98
impl Default for Colors<'_> {
    fn default() -> Self {
        Self {
            fColors: &[],
            fPos: &[],
            fColorSpace: None,
            fTileMode: SkTileMode::kClamp,
        }
    }
}
// Source: include/effects/SkGradient.h:99-110
impl<'a> Colors<'a> {
    pub fn new(
        colors: &'a [SkColor4f],
        pos: &'a [f32],
        mode: SkTileMode,
        cs: Option<Arc<SkColorSpace>>,
    ) -> Self {
        crate::SkASSERT!(@source b"include/effects/SkGradient.h\0".as_ptr(),104; @condition b"fPos.size() == 0 || fPos.size() == fColors.size()\0".as_ptr(); pos.is_empty() || pos.len() == colors.len());
        Self {
            fColors: colors,
            fPos: if pos.len() != colors.len() { &[] } else { pos },
            fColorSpace: cs,
            fTileMode: mode,
        }
    }
    // Source: include/effects/SkGradient.h:112-113
    pub fn without_positions(
        colors: &'a [SkColor4f],
        mode: SkTileMode,
        cs: Option<Arc<SkColorSpace>>,
    ) -> Self {
        Self::new(colors, &[], mode, cs)
    }
    // Source: include/effects/SkGradient.h:115-118
    pub fn colors(&self) -> &'a [SkColor4f] {
        self.fColors
    }
    pub fn positions(&self) -> &'a [f32] {
        self.fPos
    }
    pub fn colorSpace(&self) -> &Option<Arc<SkColorSpace>> {
        &self.fColorSpace
    }
    pub fn tileMode(&self) -> SkTileMode {
        self.fTileMode
    }
}
// Source: include/effects/SkGradient.h:127-137
#[derive(Clone, Default)]
pub struct SkGradient<'a> {
    fColors: Colors<'a>,
    fInterpolation: Interpolation,
}
impl<'a> SkGradient<'a> {
    pub fn new(colors: Colors<'a>, interp: Interpolation) -> Self {
        Self {
            fColors: colors,
            fInterpolation: interp,
        }
    }
    pub fn colors(&self) -> &Colors<'a> {
        &self.fColors
    }
    pub fn interpolation(&self) -> &Interpolation {
        &self.fInterpolation
    }
}

// Public namespace projection of the original source-defined factories.
pub use crate::src::shaders::gradients::SkConicalGradient::TwoPointConicalGradient;
pub use crate::src::shaders::gradients::SkLinearGradient::LinearGradient;
pub use crate::src::shaders::gradients::SkRadialGradient::RadialGradient;
pub use crate::src::shaders::gradients::SkSweepGradient::SweepGradient;

// Source: include/effects/SkGradient.h:198-202
pub fn SweepGradient_default(
    center: crate::src::core::SkPoint::SkPoint,
    gradient: &SkGradient<'_>,
    matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    SweepGradient(center, 0.0, 360.0, gradient, matrix)
}
