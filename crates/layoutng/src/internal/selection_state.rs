// cpp: layoutng/internal/selection_state.h:13-33
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SelectionState {
    kNone = 0,
    kStart = 1,
    kInside = 2,
    kEnd = 3,
    kStartAndEnd = 4,
    kContain = 5,
}
