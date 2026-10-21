// Source: include/core/SkClipOp.h:13-17
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkClipOp {
    kDifference = 0,
    kIntersect = 1,
}
impl SkClipOp {
    pub const kMax_EnumValue: Self = Self::kIntersect;
}
