// Source: src/utils/SkDashPathPriv.h:24-29
pub use crate::src::utils::SkDashPath::{CalcDashParameters, FilterDashPath};
// Source: src/utils/SkDashPathPriv.h:34-34
pub const kMaxDashCount: f32 = 1_000_000.0;
// Source: src/utils/SkDashPathPriv.h:38-41
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeRecApplication {
    kDisallow,
    kAllow,
}
// Native declarations are exported from their original implementation module, not duplicated.
// Source: src/utils/SkDashPathPriv.h:49-55
pub use crate::src::utils::SkDashPath::{InternalFilter, ValidDashPath};
