//! Host-service boundary for browser-controls input policy.

use crate::browser_controls_state::BrowserControlsState;
use compositor_scheduler::{TimeDelta, TimeTicks};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserControlsFeature {
    ScrollSnapAnimation,
    HeightChangeCancelAnimations,
    SmoothScroll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserControlsDiagnostic {
    UpdateConstraints {
        constraints: BrowserControlsState,
        current: BrowserControlsState,
    },
    RatiosUnchanged,
    HasExistingAnimation(bool),
}

pub trait BrowserControlsReadEnvironment {
    type BeginMainFrameReason;
    fn OtherBeginMainFrameReason(&self) -> Self::BeginMainFrameReason;
    fn FeatureEnabled(&self, feature: BrowserControlsFeature) -> bool;
    fn MonotonicTimeNow(&self) -> TimeTicks;
}

pub trait BrowserControlsEnvironment: BrowserControlsReadEnvironment {
    fn EmitDiagnostic(&mut self, event: BrowserControlsDiagnostic);
}

pub use compositor_animation::tween::{Tween, TweenType};

pub fn InMillisecondsF(delta: TimeDelta) -> f64 {
    delta.InMicrosecondsF() / 1000.0
}
