// Source: include/core/SkSurfaceProps.h:20-26
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPixelGeometry {
    kUnknown_SkPixelGeometry,
    kRGB_H_SkPixelGeometry,
    kBGR_H_SkPixelGeometry,
    kRGB_V_SkPixelGeometry,
    kBGR_V_SkPixelGeometry,
}
// Source: include/core/SkSurfaceProps.h:29-31
pub fn SkPixelGeometryIsRGB(geo: SkPixelGeometry) -> bool {
    matches!(
        geo,
        SkPixelGeometry::kRGB_H_SkPixelGeometry | SkPixelGeometry::kRGB_V_SkPixelGeometry
    )
}
// Source: include/core/SkSurfaceProps.h:34-36
pub fn SkPixelGeometryIsBGR(geo: SkPixelGeometry) -> bool {
    matches!(
        geo,
        SkPixelGeometry::kBGR_H_SkPixelGeometry | SkPixelGeometry::kBGR_V_SkPixelGeometry
    )
}
// Source: include/core/SkSurfaceProps.h:39-41
pub fn SkPixelGeometryIsH(geo: SkPixelGeometry) -> bool {
    matches!(
        geo,
        SkPixelGeometry::kRGB_H_SkPixelGeometry | SkPixelGeometry::kBGR_H_SkPixelGeometry
    )
}
// Source: include/core/SkSurfaceProps.h:44-46
pub fn SkPixelGeometryIsV(geo: SkPixelGeometry) -> bool {
    matches!(
        geo,
        SkPixelGeometry::kRGB_V_SkPixelGeometry | SkPixelGeometry::kBGR_V_SkPixelGeometry
    )
}
// Source: include/core/SkSurfaceProps.h:56-66
// C++ unscoped enum promotion to flag bits is represented by an explicit cast.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flags {
    kDefault_Flag = 0,
    kUseDeviceIndependentFonts_Flag = 1 << 0,
    kDynamicMSAA_Flag = 1 << 1,
    kAlwaysDither_Flag = 1 << 2,
    kPreservesTransparentDraws_Flag = 1 << 3,
}
// Source: include/core/SkSurfaceProps.h:53-55
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkSurfaceProps {
    // Source: include/core/SkSurfaceProps.h:113-113
    pub(crate) fFlags: u32,
    // Source: include/core/SkSurfaceProps.h:114-114
    pub(crate) fPixelGeometry: SkPixelGeometry,
    // Source: include/core/SkSurfaceProps.h:118-118
    pub(crate) fTextContrast: f32,
    // Source: include/core/SkSurfaceProps.h:119-119
    pub(crate) fTextGamma: f32,
}
impl SkSurfaceProps {
    // Source: include/core/SkSurfaceProps.h:56-57
    // Source: include/core/SkSurfaceProps.h:59-59
    // Source: include/core/SkSurfaceProps.h:62-62
    // Source: include/core/SkSurfaceProps.h:64-65
    pub const kDefault_Flag: u32 = 0;
    pub const kUseDeviceIndependentFonts_Flag: u32 = 1 << 0;
    pub const kDynamicMSAA_Flag: u32 = 1 << 1;
    pub const kAlwaysDither_Flag: u32 = 1 << 2;
    pub const kPreservesTransparentDraws_Flag: u32 = 1 << 3;
    // Source: include/core/SkSurfaceProps.h:77-79
    pub fn cloneWithPixelGeometry(&self, new_geometry: SkPixelGeometry) -> Self {
        Self::new(
            self.fFlags,
            new_geometry,
            self.fTextContrast,
            self.fTextGamma,
        )
    }
    // Source: include/core/SkSurfaceProps.h:81-84
    // Source: include/core/SkSurfaceProps.h:86-89
    pub const kMaxContrastInclusive: f32 = 1.;
    pub const kMinContrastInclusive: f32 = 0.;
    pub const kMaxGammaExclusive: f32 = 4.;
    pub const kMinGammaInclusive: f32 = 0.;
    pub fn flags(&self) -> u32 {
        self.fFlags
    }
    pub fn pixelGeometry(&self) -> SkPixelGeometry {
        self.fPixelGeometry
    }
    pub fn textContrast(&self) -> f32 {
        self.fTextContrast
    }
    pub fn textGamma(&self) -> f32 {
        self.fTextGamma
    }
    // Source: include/core/SkSurfaceProps.h:91-93
    // Source: include/core/SkSurfaceProps.h:95-97
    // Source: include/core/SkSurfaceProps.h:99-101
    pub fn isUseDeviceIndependentFonts(&self) -> bool {
        self.fFlags & Self::kUseDeviceIndependentFonts_Flag != 0
    }
    pub fn isAlwaysDither(&self) -> bool {
        self.fFlags & Self::kAlwaysDither_Flag != 0
    }
    pub fn preservesTransparentDraws(&self) -> bool {
        self.fFlags & Self::kPreservesTransparentDraws_Flag != 0
    }
}
// Source: include/core/SkSurfaceProps.h:103-106
// Source: include/core/SkSurfaceProps.h:108-110
impl PartialEq for SkSurfaceProps {
    fn eq(&self, that: &Self) -> bool {
        self.fFlags == that.fFlags
            && self.fPixelGeometry == that.fPixelGeometry
            && self.fTextContrast == that.fTextContrast
            && self.fTextGamma == that.fTextGamma
    }
}
