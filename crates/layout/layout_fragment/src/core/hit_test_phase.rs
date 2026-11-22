// cpp: layoutng/internal/hit_test_phase.h:12-17
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum HitTestPhase {
    kSelfBlockBackground = 0,
    kDescendantBlockBackgrounds = 1,
    kFloat = 2,
    kForeground = 3,
}
