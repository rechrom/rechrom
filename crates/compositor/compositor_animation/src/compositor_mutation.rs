use crate::{
    gfx_keyframe_model::{Direction, FillMode},
    keyframe_model::CompositorAnimationCurve,
};
use scroll_animation::{TimeDelta, TimeTicks};
use compositor_tree::ElementId;

/// Blink CompositorAnimation -> cc AnimationHost command. Curves are already
/// property-resolved; no CSS or Style value crosses into the compositor.
#[derive(Clone)]
pub enum PropertyAnimationMutation {
    Install {
        identity: u64,
        element: ElementId,
        curve: CompositorAnimationCurve,
        target_property: i32,
        iterations: f64,
        iteration_start: f64,
        start_delay: TimeDelta,
        direction: Direction,
        playback_rate: f64,
        fill_mode: FillMode,
        start_time: Option<TimeTicks>,
        hold_time: Option<TimeDelta>,
    },
    Remove {
        identity: u64,
    },
}

impl std::fmt::Debug for PropertyAnimationMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Install {
                identity,
                element,
                target_property,
                ..
            } => f
                .debug_struct("Install")
                .field("identity", identity)
                .field("element", element)
                .field("target_property", target_property)
                .finish_non_exhaustive(),
            Self::Remove { identity } => f
                .debug_struct("Remove")
                .field("identity", identity)
                .finish(),
        }
    }
}
