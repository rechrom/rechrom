//! viz::SurfaceRange value operations used by both the main SurfaceLayer and
//! the impl SurfaceLayerImpl (components/viz/common/surfaces/surface_range.h:
//! default constructor, operator==, IsValid).
use super::commit_state::CommitTypes;

pub trait SurfaceRangeTypes: CommitTypes {
    fn DefaultSurfaceRange() -> Self::SurfaceRange;
    fn SurfaceRangesEqual(a: &Self::SurfaceRange, b: &Self::SurfaceRange) -> bool;
    fn SurfaceRangeIsValid(range: &Self::SurfaceRange) -> bool;
}
