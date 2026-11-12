//! Chromium core/frame/frame_lifecycle.h:10-27 and .cc:11-28.
//! One retained lifecycle belongs to each actual Frame, independently of its
//! current Document lifecycle. Detachment may synchronously reenter itself.
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameLifecycleState {
    Attached,
    Detaching,
    Detached,
}
pub struct FrameLifecycle {
    state: Cell<FrameLifecycleState>,
}
impl Default for FrameLifecycle {
    fn default() -> Self {
        Self {
            state: Cell::new(FrameLifecycleState::Attached),
        }
    }
}
impl FrameLifecycle {
    pub fn GetState(&self) -> FrameLifecycleState {
        self.state.get()
    }
    pub fn AdvanceTo(&self, state: FrameLifecycleState) {
        debug_assert!(match state {
            FrameLifecycleState::Attached | FrameLifecycleState::Detached => {
                state > self.state.get()
            }
            FrameLifecycleState::Detaching => state >= self.state.get(),
        });
        self.state.set(state);
    }
}
