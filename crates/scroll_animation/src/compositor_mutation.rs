//! Typed source CompositorAnimation transactions. Page routes values in order;
//! the receiving compositor owns every Animation/KeyframeEffect/Timeline.
use crate::{scroll_offset_animation_curve::ScrollOffsetAnimationCurve, TimeTicks};
use scroll_snap::{geometry::Vector2dF, ElementId};
#[derive(Clone)]
pub enum ScrollAnimationMutation {
    Create {
        animation: i32,
    },
    Dispose {
        animation: i32,
    },
    Attach {
        animation: i32,
        timeline: i32,
        element: ElementId,
    },
    Detach {
        animation: i32,
        timeline: i32,
    },
    AddKeyframeModel {
        animation: i32,
        model: i32,
        group: i32,
        curve: ScrollOffsetAnimationCurve,
        start: TimeTicks,
    },
    // Coordinator.cc:101-111. Removal itself can set interrupted, outside
    // commit. The receiver must test it immediately after the real removal.
    RemoveKeyframeModel {
        animation: i32,
        model: i32,
        element: ElementId,
        drop_scroll_delta_if_interrupted: bool,
    },
    AbortKeyframeModel {
        animation: i32,
        model: i32,
    },
    ImplOnlyAdjustment {
        element: ElementId,
        adjustment: Vector2dF,
    },
    ImplOnlyTakeover {
        element: ElementId,
    },
}
impl std::fmt::Debug for ScrollAnimationMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create { animation } => f.debug_tuple("Create").field(animation).finish(),
            Self::Dispose { animation } => f.debug_tuple("Dispose").field(animation).finish(),
            Self::Attach {
                animation,
                timeline,
                element,
            } => f
                .debug_tuple("Attach")
                .field(animation)
                .field(timeline)
                .field(element)
                .finish(),
            Self::Detach {
                animation,
                timeline,
            } => f
                .debug_tuple("Detach")
                .field(animation)
                .field(timeline)
                .finish(),
            Self::AddKeyframeModel {
                animation,
                model,
                group,
                start,
                ..
            } => f
                .debug_tuple("AddKeyframeModel")
                .field(animation)
                .field(model)
                .field(group)
                .field(start)
                .finish(),
            Self::RemoveKeyframeModel {
                animation,
                model,
                element,
                drop_scroll_delta_if_interrupted,
            } => f
                .debug_tuple("RemoveKeyframeModel")
                .field(animation)
                .field(model)
                .field(element)
                .field(drop_scroll_delta_if_interrupted)
                .finish(),
            Self::AbortKeyframeModel { animation, model } => f
                .debug_tuple("AbortKeyframeModel")
                .field(animation)
                .field(model)
                .finish(),
            Self::ImplOnlyAdjustment {
                element,
                adjustment,
            } => f
                .debug_tuple("ImplOnlyAdjustment")
                .field(element)
                .field(adjustment)
                .finish(),
            Self::ImplOnlyTakeover { element } => {
                f.debug_tuple("ImplOnlyTakeover").field(element).finish()
            }
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum ScrollAnimationCompletion {
    Finished { animation: i32, group: i32 },
    Aborted { animation: i32, group: i32 },
}
