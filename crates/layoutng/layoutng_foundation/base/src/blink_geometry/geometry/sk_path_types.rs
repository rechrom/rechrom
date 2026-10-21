// C++: src/foundation/blink_geometry/geometry/SkPathTypes.h

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkPathFillType {
    kWinding,
    kEvenOdd,
    kInverseWinding,
    kInverseEvenOdd,
}

impl SkPathFillType {
    pub const kDefault: Self = Self::kWinding;
}

pub fn SkPathFillType_IsEvenOdd(ft: SkPathFillType) -> bool {
    (ft as u8 & 1) != 0
}

pub fn SkPathFillType_IsInverse(ft: SkPathFillType) -> bool {
    (ft as u8 & 2) != 0
}

pub fn SkPathFillType_ToggleInverse(ft: SkPathFillType) -> SkPathFillType {
    match (ft as u8) ^ 2 {
        0 => SkPathFillType::kWinding,
        1 => SkPathFillType::kEvenOdd,
        2 => SkPathFillType::kInverseWinding,
        3 => SkPathFillType::kInverseEvenOdd,
        _ => unreachable!(),
    }
}

pub fn SkPathFillType_ConvertToNonInverse(ft: SkPathFillType) -> SkPathFillType {
    if (ft as u8 & 1) == 0 {
        SkPathFillType::kWinding
    } else {
        SkPathFillType::kEvenOdd
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkPathDirection {
    kCW,
    kCCW,
}

impl SkPathDirection {
    pub const kDefault: Self = Self::kCW;
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkPathSegmentMask {
    kLine_SkPathSegmentMask = 1 << 0,
    kQuad_SkPathSegmentMask = 1 << 1,
    kConic_SkPathSegmentMask = 1 << 2,
    kCubic_SkPathSegmentMask = 1 << 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkPathVerb {
    kMove,
    kLine,
    kQuad,
    kConic,
    kCubic,
    kClose,
}

impl SkPathVerb {
    pub const kLast_Verb: Self = Self::kClose;
}
