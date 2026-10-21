use crate::include::core::SkPathTypes::SkPathDirection;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::SkRect;

// Source: include/private/SkPathRef.h:38-42
#[derive(Clone, Copy, Debug)]
pub struct SkPathRectInfo {
    pub fRect: SkRect,
    pub fDirection: SkPathDirection,
    pub fStartIndex: u8,
}

// Source: include/private/SkPathRef.h:44-48
#[derive(Clone, Copy, Debug)]
pub struct SkPathOvalInfo {
    pub fBounds: SkRect,
    pub fDirection: SkPathDirection,
    pub fStartIndex: u8,
}

// Source: include/private/SkPathRef.h:50-54
#[derive(Clone, Copy, Debug)]
pub struct SkPathRRectInfo {
    pub fRRect: SkRRect,
    pub fDirection: SkPathDirection,
    pub fStartIndex: u8,
}

// Source: include/private/SkPathRef.h:79-83
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkPathIsAType {
    kGeneral = 0,
    kOval = 1,
    kRRect = 2,
}

// Source: include/private/SkPathRef.h:85-88
#[derive(Clone, Copy, Debug)]
pub struct SkPathIsAData {
    pub fStartIndex: u8,
    pub fDirection: SkPathDirection,
}
