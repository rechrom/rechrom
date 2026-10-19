//! Live layer/tree/input/task interfaces consumed by the translated scrollbar
//! controller. Handles must retain canonical object identity and query current
//! owner state; no layer/property-tree snapshots or algorithms are stored here.

use super::input_handler::{InputLayerId, InputScrollNodeId};
use super::{
    geometry::{self, PointF, Rect, SizeF, Vector2dF},
    input_types::ElementId,
    time::TimeDelta,
};
use crate::cc::input::scrollbar::{ScrollbarOrientation, ScrollbarPart};
use std::sync::Weak;

pub trait ScrollbarTransform: geometry::Transform + Clone + Default {
    fn GetInverse(&self, result: &mut Self) -> bool;
}
pub trait ScrollbarLayerAccess {
    type Transform: ScrollbarTransform;
    fn Scrollbar_orientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation;
    fn Scrollbar_scroll_element_id(&mut self, layer: &InputLayerId) -> ElementId;
    fn Scrollbar_OverlayScrollbarOpacity(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_UsesAuraOverlayAnimator(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_has_find_in_page_tickmarks(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_JumpOnTrackClick(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_current_pos(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_scroll_layer_length(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_ComputeThumbQuadRect(&mut self, layer: &InputLayerId) -> Rect;
    fn Scrollbar_SupportsDragSnapBack(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_ForwardTrackRect(&mut self, layer: &InputLayerId) -> Rect;
    fn Scrollbar_BackTrackRect(&mut self, layer: &InputLayerId) -> Rect;
    fn Scrollbar_BackButtonRect(&mut self, layer: &InputLayerId) -> Rect;
    fn Scrollbar_ForwardButtonRect(&mut self, layer: &InputLayerId) -> Rect;
    fn Scrollbar_ThumbThickness(&mut self, layer: &InputLayerId) -> i32;
    fn Scrollbar_ThumbLength(&mut self, layer: &InputLayerId) -> i32;
    fn Scrollbar_TrackLength(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_ScreenSpaceTransform(&mut self, layer: &InputLayerId) -> Self::Transform;
    fn Scrollbar_IdentifyScrollbarPart(
        &mut self,
        layer: &InputLayerId,
        position: PointF,
    ) -> ScrollbarPart;
    fn Scrollbar_is_overlay_scrollbar(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_bounds(&mut self, layer: &InputLayerId) -> geometry::Size;
    fn Scrollbar_device_scale_factor(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_ComputeHitTestableExpandedThumbQuadRect(&mut self, layer: &InputLayerId) -> Rect;
    fn Scrollbar_thumb_thickness_scale_factor(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_SetThumbThicknessScaleFactor(&mut self, layer: &InputLayerId, scale: f32);
    fn Scrollbar_CanScrollOrientation(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        layer: &InputLayerId,
        opacity: f32,
        fade_out_animation: bool,
    );
}
#[derive(Clone, Debug)]
pub struct AutoscrollTask {
    pub owner_id: u64,
    pub generation: u64,
    pub(crate) alive: Weak<()>,
}
impl AutoscrollTask {
    pub fn IsAlive(&self) -> bool {
        self.alive.upgrade().is_some()
    }
    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.owner_id == other.owner_id
            && self.generation == other.generation
            && self.alive.ptr_eq(&other.alive)
    }
}
impl PartialEq for AutoscrollTask {
    fn eq(&self, other: &Self) -> bool {
        self.matches(other)
    }
}
impl Eq for AutoscrollTask {}
/// Registration only schedules the canonical autoscroll continuation or aborts
/// its native animation. It cannot query viewport/tree/input state.
/// Source callers: cc/input/scrollbar_controller.cc:147-158,482-506.
pub trait ScrollbarRegistrationHost {
    fn PostDelayedAutoscrollTask(&mut self, task: AutoscrollTask, delay: TimeDelta);
    fn ScrollAnimationAbort(&mut self, element: ElementId);
}
impl<H: ScrollbarHost + ?Sized> ScrollbarRegistrationHost for H {
    fn PostDelayedAutoscrollTask(&mut self, task: AutoscrollTask, delay: TimeDelta) {
        ScrollbarHost::PostDelayedAutoscrollTask(self, task, delay);
    }
    fn ScrollAnimationAbort(&mut self, element: ElementId) {
        ScrollbarHost::ScrollAnimationAbort(self, element);
    }
}
/// Every layer/node access resolves and validates its resident tree ID. There
/// are no node/layer/tree handles with implicit owner backpointers.
pub trait ScrollbarHost: ScrollbarLayerAccess {
    fn ScrollbarsFor(&mut self, element: ElementId) -> Vec<InputLayerId>;
    fn IsScrollbarLayer(&mut self, layer: &InputLayerId) -> bool;
    fn FindLayerThatIsHitByPoint(&mut self, point: PointF) -> Option<InputLayerId>;
    fn device_scale_factor(&mut self) -> f32;
    fn painted_device_scale_factor(&mut self) -> f32;
    fn page_scale_factor_for_scroll(&mut self) -> f32;
    fn FindNodeFromElementId(&mut self, element: ElementId) -> Option<InputScrollNodeId>;
    fn current_scroll_offset(&mut self, element: ElementId) -> PointF;
    fn scrolls_inner_viewport(&mut self, node: &InputScrollNodeId) -> bool;
    fn scrolls_outer_viewport(&mut self, node: &InputScrollNodeId) -> bool;
    fn element_id(&mut self, node: &InputScrollNodeId) -> ElementId;
    fn container_bounds(&mut self, node: &InputScrollNodeId) -> geometry::Size;
    fn SnapContainerRect(&mut self, node: &InputScrollNodeId) -> Option<geometry::RectF>;
    fn ComputeClampedDelta(&mut self, delta: &Vector2dF) -> Vector2dF;
    fn GetInnerViewportSizeExcludingScrollbars(&mut self) -> SizeF;
    /// Synchronously calls canonical InputHandler::ComputeScrollDeltaForState
    /// with a borrowed last-scroll-update state and current tree/settings.
    fn ComputeScrollDelta(&mut self, node: &InputScrollNodeId, delta: &Vector2dF) -> Vector2dF;
    fn ScrollAnimationAbort(&mut self, element: ElementId);
    fn IsElementAnimating(&mut self, element: ElementId) -> bool;
    fn AutoScrollAnimationCreate(
        &mut self,
        node: &InputScrollNodeId,
        target: PointF,
        velocity: f32,
    );
    fn controller_id(&self) -> u64;
    /// Queue after the current call stack, including TimeDelta::Min() delays.
    fn PostDelayedAutoscrollTask(&mut self, task: AutoscrollTask, delay: TimeDelta);
}
pub use super::layer_tree_impl::Color4f;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Insets {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

pub fn RectContainsPoint(rect: geometry::RectF, point: PointF) -> bool {
    point.x() >= rect.x()
        && point.x() < rect.right()
        && point.y() >= rect.y()
        && point.y() < rect.bottom()
}
pub fn VectorIsZero(vector: Vector2dF) -> bool {
    vector.x() == 0. && vector.y() == 0.
}
pub fn ScaleVector(vector: Vector2dF, scale: f32) -> Vector2dF {
    Vector2dF::new(vector.x() * scale, vector.y() * scale)
}

// Canonical input result declarations now live in their translated owner.
pub use crate::cc::input::input_handler::{InputHandlerPointerResult, PointerResultType};

/// The exact client subset read by native scrollbar animation constructors.
/// Source decisions remain in canonical Controller/Thinning; this does not
/// pretend initialization owns the later input/task/animation capabilities.
pub trait ScrollbarInitializationClient {
    fn ScrollbarsFor(&mut self, element: ElementId) -> Vec<InputLayerId>;
    fn Scrollbar_orientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation;
    fn Scrollbar_is_overlay_scrollbar(&mut self, layer: &InputLayerId) -> bool;
    fn Scrollbar_thumb_thickness_scale_factor(&mut self, layer: &InputLayerId) -> f32;
    fn Scrollbar_SetThumbThicknessScaleFactor(&mut self, layer: &InputLayerId, scale: f32);
}
impl<
        C: crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationControllerClient
            + ?Sized,
    > ScrollbarInitializationClient for C
{
    fn ScrollbarsFor(&mut self, element: ElementId) -> Vec<InputLayerId> {
        crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationControllerClient::ScrollbarsFor(self,element)
    }
    fn Scrollbar_orientation(&mut self, layer: &InputLayerId) -> ScrollbarOrientation {
        ScrollbarLayerAccess::Scrollbar_orientation(self, layer)
    }
    fn Scrollbar_is_overlay_scrollbar(&mut self, layer: &InputLayerId) -> bool {
        ScrollbarLayerAccess::Scrollbar_is_overlay_scrollbar(self, layer)
    }
    fn Scrollbar_thumb_thickness_scale_factor(&mut self, layer: &InputLayerId) -> f32 {
        ScrollbarLayerAccess::Scrollbar_thumb_thickness_scale_factor(self, layer)
    }
    fn Scrollbar_SetThumbThicknessScaleFactor(&mut self, layer: &InputLayerId, scale: f32) {
        ScrollbarLayerAccess::Scrollbar_SetThumbThicknessScaleFactor(self, layer, scale)
    }
}
