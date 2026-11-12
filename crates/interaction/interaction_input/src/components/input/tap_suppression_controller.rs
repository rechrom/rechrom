// Chromium components/input/tap_suppression_controller.h:22-76;
// tap_suppression_controller.cc:13-73.
// The embedding supplies NowTicks, keeping the state machine clock-neutral.
use crate::base::time::time::TimeTicks;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub enabled: bool,
    pub max_cancel_to_down_microseconds: i64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            max_cancel_to_down_microseconds: 180_000,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Disabled,
    Nothing,
    LastCancelStoppedFling,
    SuppressingTaps,
}
pub struct TapSuppressionController {
    state: State,
    max_cancel_to_down_time: i64,
    fling_cancel_time: TimeTicks,
}
impl TapSuppressionController {
    pub fn new(config: Config) -> Self {
        Self {
            state: if config.enabled {
                State::Nothing
            } else {
                State::Disabled
            },
            max_cancel_to_down_time: config.max_cancel_to_down_microseconds,
            fling_cancel_time: TimeTicks::new(),
        }
    }
    pub fn GestureFlingCancelStoppedFling(&mut self, now: TimeTicks) {
        if matches!(self.state, State::Nothing | State::SuppressingTaps) {
            self.fling_cancel_time = now;
            self.state = State::LastCancelStoppedFling;
        }
    }
    pub fn ShouldSuppressTapDown(&mut self, now: TimeTicks) -> bool {
        match self.state {
            State::Disabled | State::Nothing => false,
            State::LastCancelStoppedFling => {
                if now.ToInternalValue() - self.fling_cancel_time.ToInternalValue()
                    < self.max_cancel_to_down_time
                {
                    self.state = State::SuppressingTaps;
                    true
                } else {
                    self.state = State::Nothing;
                    false
                }
            }
            State::SuppressingTaps => {
                self.state = State::Nothing;
                false
            }
        }
    }
    pub fn ShouldSuppressTapEnd(&self) -> bool {
        matches!(
            self.state,
            State::LastCancelStoppedFling | State::SuppressingTaps
        )
    }
}
