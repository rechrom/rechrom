// Source: src/core/SkPathMeasurePriv.h:18-23
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SkSegType {
    kLine_SegType,
    kQuad_SegType,
    kCubic_SegType,
    kConic_SegType,
}
// Source: src/core/SkPathMeasurePriv.h:33-36
pub struct SkPathMeasurePriv;
