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
