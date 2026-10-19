//! Implemented SkPathFillType subset. Inverse winding is handled by compat replay.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub enum SkPathFillType {
    /// Specifies that "inside" is computed by a non-zero sum of signed edge crossings.
    #[default]
    Winding,
    /// Specifies that "inside" is computed by an odd number of edge crossings.
    EvenOdd,
}
pub use SkPathFillType as FillRule;

// Source: include/core/SkPathTypes.h:46-57,65-86.
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathDirection {
    kCW = 0,
    kCCW = 1,
}
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPathVerb {
    kMove = 0,
    kLine = 1,
    kQuad = 2,
    kConic = 3,
    kCubic = 4,
    kClose = 5,
}
