// Source: include/core/SkPathTypes.h:15-26
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkPathFillType {
    kWinding = 0,
    kEvenOdd = 1,
    kInverseWinding = 2,
    kInverseEvenOdd = 3,
}
impl SkPathFillType {
    pub const kDefault: Self = Self::kWinding;
}

// Source: include/core/SkPathTypes.h:28-30
#[inline]
pub fn SkPathFillType_IsEvenOdd(ft: SkPathFillType) -> bool {
    ft as u8 & 1 != 0
}

// Source: include/core/SkPathTypes.h:32-34
#[inline]
pub fn SkPathFillType_IsInverse(ft: SkPathFillType) -> bool {
    ft as u8 & 2 != 0
}

// Source: include/core/SkPathTypes.h:36-38
#[inline]
pub fn SkPathFillType_ToggleInverse(ft: SkPathFillType) -> SkPathFillType {
    match ft as u8 ^ 2 {
        0 => SkPathFillType::kWinding,
        1 => SkPathFillType::kEvenOdd,
        2 => SkPathFillType::kInverseWinding,
        3 => SkPathFillType::kInverseEvenOdd,
        _ => unreachable!(),
    }
}

// Source: include/core/SkPathTypes.h:40-42
#[inline]
pub fn SkPathFillType_ConvertToNonInverse(ft: SkPathFillType) -> SkPathFillType {
    if ft as u8 & 1 != 0 {
        SkPathFillType::kEvenOdd
    } else {
        SkPathFillType::kWinding
    }
}

// Source: include/core/SkPathTypes.h:44-51
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkPathDirection {
    kCW = 0,
    kCCW = 1,
}
impl SkPathDirection {
    pub const kDefault: Self = Self::kCW;
}

// Source: include/core/SkPathTypes.h:53-58
pub type SkPathSegmentMask = u32;
pub const kLine_SkPathSegmentMask: SkPathSegmentMask = 1 << 0;
pub const kQuad_SkPathSegmentMask: SkPathSegmentMask = 1 << 1;
pub const kConic_SkPathSegmentMask: SkPathSegmentMask = 1 << 2;
pub const kCubic_SkPathSegmentMask: SkPathSegmentMask = 1 << 3;

// Source: include/core/SkPathTypes.h:60-69
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkPathVerb {
    kMove = 0,
    kLine = 1,
    kQuad = 2,
    kConic = 3,
    kCubic = 4,
    kClose = 5,
}
impl SkPathVerb {
    pub const kLast_Verb: Self = Self::kClose;
}
