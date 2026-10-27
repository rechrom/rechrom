//! Scrollbar input vocabulary shared by viewport and scrollbar controllers.

use compositor_scheduler::TimeDelta;

// Source: cc/input/scrollbar.h:15-27.
pub const kAutoscrollMultiplier: f32 = 20.;
pub const kInitialAutoscrollTimerDelay: TimeDelta = TimeDelta::Milliseconds(250);
pub const kOffSideMultiplier: i32 = 8;
pub const kDefaultWinScrollbarThickness: i32 = 17;

// Source: cc/input/scrollbar.h:31-40.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarOrientation {
    kHorizontal,
    kVertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarPart {
    kThumb,
    kBackButton,
    kForwardButton,
    kBackTrack,
    kForwardTrack,
    kNoPart,
}
