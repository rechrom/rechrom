//! LocalFrameView.h:242-259, Chromium 6c1d401f. Pure lifecycle invalidation
//! value shared by the owning Layout/Paint/FrameView stages. Updates take max.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntersectionObservationState {
    #[default]
    NotNeeded,
    ScrollAndVisibilityOnly,
    Desired,
    Required,
}
