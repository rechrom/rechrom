// Translation status: complete
// Source: cc/input/scrollbar_controller.h:1-348
// Source: cc/input/scrollbar_controller.cc:1-853
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{
    scroll_utils::{ScrollUtils, kPixelsPerLineStep},
    scrollbar::{
        ScrollbarOrientation, ScrollbarPart, kAutoscrollMultiplier, kInitialAutoscrollTimerDelay,
        kOffSideMultiplier,
    },
};
use crate::{
    adapter::{
        geometry::{PointF, Rect, RectF, Vector2dF},
        input_handler::{InputLayerId, InputScrollNodeId},
        input_types::{ElementId, ScrollGranularity},
        scrollbar::*,
        time::TimeDelta,
    },
    cc::base::math_util::MathUtil,
};
use std::sync::Arc;
// Evaluate source call arguments before borrowing the live receiver. This is
// a Rust borrowing adaptation; evaluation order and values remain native.
macro_rules! host_call {
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        let a5 = $a5;
        let a6 = $a6;
        let a7 = $a7;
        $host.$method(a0, a1, a2, a3, a4, a5, a6, a7)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        let a5 = $a5;
        let a6 = $a6;
        $host.$method(a0, a1, a2, a3, a4, a5, a6)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        let a5 = $a5;
        $host.$method(a0, a1, a2, a3, a4, a5)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        $host.$method(a0, a1, a2, a3, a4)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr, $a3:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        $host.$method(a0, a1, a2, a3)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr, $a2:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        $host.$method(a0, a1, a2)
    }};
    ($host:ident, $method:ident($a0:expr, $a1:expr $(,)?)) => {{
        let a0 = $a0;
        let a1 = $a1;
        $host.$method(a0, a1)
    }};
    ($host:ident, $method:ident($a0:expr $(,)?)) => {{
        let a0 = $a0;
        $host.$method(a0)
    }};
    ($host:ident, $method:ident( $(,)?)) => {{
        $host.$method()
    }};
    ($host:ident, $($call:tt)*) => { $host.$($call)* };
}

// Source: cc/input/scrollbar_controller.h:173-185
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutoScrollDirection {
    kAutoscrollForward,
    kAutoscrollBackward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutoScrollStatus {
    kAutoscrollWaiting,
    kAutoscrollReady,
    kAutoscrollScrolling,
}
// Source: cc/input/scrollbar_controller.h:187-204
#[derive(Clone, Copy)]
struct AutoScrollState {
    direction: AutoScrollDirection,
    status: AutoScrollStatus,
    velocity: f32,
    scroll_layer_length: f32,
    pressed_scrollbar_part: ScrollbarPart,
}
impl Default for AutoScrollState {
    fn default() -> Self {
        Self {
            direction: AutoScrollDirection::kAutoscrollForward,
            status: AutoScrollStatus::kAutoscrollWaiting,
            velocity: 0.,
            scroll_layer_length: 0.,
            pressed_scrollbar_part: ScrollbarPart::kThumb,
        }
    }
}
// Source: cc/input/scrollbar_controller.h:206-221
#[derive(Clone, Copy, Default)]
struct DragState {
    drag_origin: PointF,
    scroll_position_at_start_: f32,
    scroller_displacement: f32,
    scroller_length_at_previous_move: f32,
}
// Source: cc/input/scrollbar_controller.h:223-229
#[derive(Clone, Copy)]
struct CapturedScrollbarMetadata {
    scroll_element_id: ElementId,
    orientation: ScrollbarOrientation,
}
// Rust representation of the source CancelableOnceClosure's callback state.
struct CancelableAutoscrollTask {
    token: AutoscrollTask,
    pending: bool,
}

// Source: cc/input/scrollbar_controller.h:313-343
pub struct ScrollbarController {
    owner_id_: u64,
    alive_: Arc<()>,
    scrollbar_scroll_is_active_: bool,
    last_known_pointer_position_: PointF,
    captured_scrollbar_metadata_: Option<CapturedScrollbarMetadata>,
    autoscroll_state_: Option<AutoScrollState>,
    drag_state_: Option<DragState>,
    drag_processed_for_current_frame_: bool,
    cancelable_autoscroll_task_: Option<CancelableAutoscrollTask>,
    task_generation_: u64,
}
impl ScrollbarController {
    // Source: cc/input/scrollbar_controller.cc:27-33
    pub fn new<H: ScrollbarHost + ?Sized>(host: &mut H) -> Self {
        Self::new_for_owner(host.controller_id())
    }
    // Source: cc/input/scrollbar_controller.cc:27-33. The native constructor
    // retains only the stable host identity; it makes no hit-test/tree query.
    pub fn new_for_owner(owner_id: u64) -> Self {
        Self {
            owner_id_: owner_id,
            alive_: Arc::new(()),
            scrollbar_scroll_is_active_: false,
            last_known_pointer_position_: PointF::new(0., 0.),
            captured_scrollbar_metadata_: None,
            autoscroll_state_: None,
            drag_state_: None,
            drag_processed_for_current_frame_: false,
            cancelable_autoscroll_task_: None,
            task_generation_: 0,
        }
    }
    // Source: cc/input/scrollbar_controller.h:149-152
    pub fn AutoscrollTaskIsScheduled(&self) -> bool {
        self.cancelable_autoscroll_task_.is_some()
    }
    pub fn ScrollbarScrollIsActive(&self) -> bool {
        self.scrollbar_scroll_is_active_
    }
    // Source: cc/input/scrollbar_controller.cc:35-38
    pub fn WillBeginImplFrame<H: ScrollbarHost + ?Sized>(&mut self, host: &mut H) {
        self.drag_processed_for_current_frame_ = false;
        self.RecomputeAutoscrollStateIfNeeded(host);
    }
    // Source: cc/input/scrollbar_controller.cc:41-52
    pub fn ScrollbarLayer<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> Option<InputLayerId> {
        let metadata = self.captured_scrollbar_metadata_?;
        let scrollbars = host.ScrollbarsFor(metadata.scroll_element_id);
        for scrollbar in scrollbars {
            if metadata.orientation == host.Scrollbar_orientation(&scrollbar) {
                return Some(scrollbar);
            }
        }
        None
    }
    // Source: cc/input/scrollbar_controller.cc:54-69
    pub fn HitTest<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        position_in_widget: PointF,
    ) -> Option<InputLayerId> {
        let layer_impl = self.GetLayerHitByPoint(host, position_in_widget);
        if !layer_impl
            .as_ref()
            .is_some_and(|layer| host.IsScrollbarLayer(layer))
        {
            return None;
        }
        let scrollbar = layer_impl.unwrap();
        if host.Scrollbar_OverlayScrollbarOpacity(&scrollbar) == 0. {
            None
        } else {
            Some(scrollbar)
        }
    }
    // Source: cc/input/scrollbar_controller.cc:73-150
    pub fn HandlePointerDown<H: ScrollbarHost + ?Sized>(
        &mut self,
        host: &mut H,
        position_in_widget: PointF,
        jump_key_modifier: bool,
    ) -> InputHandlerPointerResult {
        let Some(scrollbar) = self.HitTest(host, position_in_widget) else {
            return InputHandlerPointerResult::default();
        };
        self.captured_scrollbar_metadata_ = Some(CapturedScrollbarMetadata {
            scroll_element_id: host.Scrollbar_scroll_element_id(&scrollbar),
            orientation: host.Scrollbar_orientation(&scrollbar),
        });
        let mut scroll_result = InputHandlerPointerResult::default();
        scroll_result.target_scroller = host.Scrollbar_scroll_element_id(&scrollbar);
        scroll_result.type_ = PointerResultType::kScrollbarScroll;
        let scrollbar_part = self.GetScrollbarPartFromPointerDown(host, position_in_widget);
        let perform_jump_click_on_track =
            host.Scrollbar_JumpOnTrackClick(&scrollbar) != jump_key_modifier;
        // Must precede absolute jump delta calculation, which reads this position.
        self.last_known_pointer_position_ = position_in_widget;
        scroll_result.scroll_delta =
            self.GetScrollDeltaForScrollbarPart(host, scrollbar_part, perform_jump_click_on_track);
        self.scrollbar_scroll_is_active_ = true;
        scroll_result.scroll_units = self.Granularity(scrollbar_part, perform_jump_click_on_track);
        let trigger_jump_click_on_track_part = perform_jump_click_on_track
            && (scrollbar_part == ScrollbarPart::kBackTrack
                || scrollbar_part == ScrollbarPart::kForwardTrack);
        if scrollbar_part == ScrollbarPart::kThumb || trigger_jump_click_on_track_part {
            self.drag_state_ = Some(DragState::default());
            let mut clipped = false;
            let drag_origin = if trigger_jump_click_on_track_part {
                self.DragOriginForJumpClick(host, &scrollbar)
            } else {
                self.GetScrollbarRelativePosition(host, position_in_widget, &mut clipped)
            };
            self.drag_state_.as_mut().unwrap().drag_origin = drag_origin;
            debug_assert!(!clipped);
            self.drag_state_.as_mut().unwrap().scroll_position_at_start_ =
                host.Scrollbar_current_pos(&scrollbar);
            self.drag_state_
                .as_mut()
                .unwrap()
                .scroller_length_at_previous_move = host.Scrollbar_scroll_layer_length(&scrollbar);
        }
        if !VectorIsZero(scroll_result.scroll_delta) && !perform_jump_click_on_track {
            debug_assert_ne!(scrollbar_part, ScrollbarPart::kThumb);
            self.autoscroll_state_ = Some(AutoScrollState::default());
            let velocity = self.InitialDeltaToAutoscrollVelocity(host, scroll_result.scroll_delta);
            self.autoscroll_state_.as_mut().unwrap().velocity = velocity;
            self.autoscroll_state_
                .as_mut()
                .unwrap()
                .pressed_scrollbar_part = scrollbar_part;
            self.PostAutoscrollTask(host, kInitialAutoscrollTimerDelay);
        }
        scroll_result
    }
    // Source: cc/input/scrollbar_controller.cc:152-158
    fn PostAutoscrollTask<H: ScrollbarRegistrationHost + ?Sized>(
        &mut self,
        host: &mut H,
        delay: TimeDelta,
    ) {
        self.CancelAutoscrollTask();
        self.task_generation_ = self.task_generation_.wrapping_add(1);
        let token = AutoscrollTask {
            owner_id: self.owner_id_,
            generation: self.task_generation_,
            alive: Arc::downgrade(&self.alive_),
        };
        self.cancelable_autoscroll_task_ = Some(CancelableAutoscrollTask {
            token: token.clone(),
            pending: true,
        });
        host.PostDelayedAutoscrollTask(token, delay);
    }
    // Source: cc/input/scrollbar_controller.cc:160-172
    fn DragOriginForJumpClick<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        scrollbar: &InputLayerId,
    ) -> PointF {
        let thumb = host.Scrollbar_ComputeThumbQuadRect(&scrollbar);
        if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kHorizontal {
            PointF::new(thumb.x as f32 + thumb.width as f32 / 2., 0.)
        } else {
            PointF::new(0., thumb.y as f32 + thumb.height as f32 / 2.)
        }
    }
    // Source: cc/input/scrollbar_controller.cc:174-232
    fn SnapToDragOrigin<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        pointer_position_in_widget: PointF,
    ) -> bool {
        let Some(scrollbar) = self.ScrollbarLayer(host) else {
            return false;
        };
        if !host.Scrollbar_SupportsDragSnapBack(&scrollbar) {
            return false;
        }
        let mut clipped = false;
        let pointer_position_in_layer =
            self.GetScrollbarRelativePosition(host, pointer_position_in_widget, &mut clipped);
        if clipped {
            return false;
        }
        let orientation = host.Scrollbar_orientation(&scrollbar);
        let forward_track_rect = host.Scrollbar_ForwardTrackRect(&scrollbar);
        let mut track_thickness = if orientation == ScrollbarOrientation::kVertical {
            forward_track_rect.width
        } else {
            forward_track_rect.height
        };
        if track_thickness == 0 {
            let thumb_thickness = host.Scrollbar_ThumbThickness(&scrollbar);
            if thumb_thickness == 0 {
                return false;
            }
            track_thickness = thumb_thickness;
        }
        let gutter_thickness = (kOffSideMultiplier * track_thickness) as f32;
        let gutter_min_bound = if orientation == ScrollbarOrientation::kVertical {
            forward_track_rect.x as f32 - gutter_thickness
        } else {
            forward_track_rect.y as f32 - gutter_thickness
        };
        let gutter_max_bound = if orientation == ScrollbarOrientation::kVertical {
            (forward_track_rect.x + track_thickness) as f32 + gutter_thickness
        } else {
            (forward_track_rect.y + track_thickness) as f32 + gutter_thickness
        };
        let pointer_location = if orientation == ScrollbarOrientation::kVertical {
            pointer_position_in_layer.x()
        } else {
            pointer_position_in_layer.y()
        };
        pointer_location < gutter_min_bound || pointer_location > gutter_max_bound
    }
    // Source: cc/input/scrollbar_controller.cc:234-248
    fn Granularity(
        &self,
        scrollbar_part: ScrollbarPart,
        jump_key_modifier: bool,
    ) -> ScrollGranularity {
        let shift_click_on_scrollbar_track = jump_key_modifier
            && (scrollbar_part == ScrollbarPart::kForwardTrack
                || scrollbar_part == ScrollbarPart::kBackTrack);
        if shift_click_on_scrollbar_track || scrollbar_part == ScrollbarPart::kThumb {
            return ScrollGranularity::kScrollByPrecisePixel;
        }
        ScrollGranularity::kScrollByPixel
    }
    // Source: cc/input/scrollbar_controller.cc:250-281
    fn GetScrollDistanceForAbsoluteJump<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> f32 {
        let mut clipped = false;
        let pointer_position_in_layer = self.GetScrollbarRelativePosition(
            host,
            self.last_known_pointer_position_,
            &mut clipped,
        );
        if clipped {
            return 0.;
        }
        let scrollbar = self.bar(host);
        let pointer_location =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                pointer_position_in_layer.y()
            } else {
                pointer_position_in_layer.x()
            };
        let thumb_length = host.Scrollbar_ThumbLength(&scrollbar);
        let desired_thumb_origin = pointer_location - thumb_length as f32 / 2.;
        let thumb_rect = host.Scrollbar_ComputeThumbQuadRect(&scrollbar);
        let current_thumb_origin =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                thumb_rect.y
            } else {
                thumb_rect.x
            } as f32;
        let distance = (desired_thumb_origin - current_thumb_origin).abs().round();
        distance * self.GetScrollerToScrollbarRatio(host) * self.GetPageScaleFactorForScroll(host)
    }
    // Source: cc/input/scrollbar_controller.cc:283-307
    fn GetScrollDistanceForDragPosition<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        pointer_position_in_widget: PointF,
    ) -> f32 {
        let scrollbar = self.bar(host);
        let mut clipped = false;
        let scrollbar_relative_position =
            self.GetScrollbarRelativePosition(host, pointer_position_in_widget, &mut clipped);
        let drag = self.drag_state_.as_ref().unwrap();
        let pointer_delta =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                scrollbar_relative_position.y() - drag.drag_origin.y()
            } else {
                scrollbar_relative_position.x() - drag.drag_origin.x()
            };
        let new_offset = pointer_delta * self.GetScrollerToScrollbarRatio(host);
        let mut distance =
            drag.scroll_position_at_start_ + new_offset - host.Scrollbar_current_pos(&scrollbar);
        distance *= self.GetPageScaleFactorForScroll(host);
        distance
    }
    // Source: cc/input/scrollbar_controller.cc:310-394
    pub fn HandlePointerMove<H: ScrollbarHost + ?Sized>(
        &mut self,
        host: &mut H,
        position_in_widget: PointF,
    ) -> InputHandlerPointerResult {
        self.last_known_pointer_position_ = position_in_widget;
        self.RecomputeAutoscrollStateIfNeeded(host);
        let mut scroll_result = InputHandlerPointerResult::default();
        let Some(scrollbar) = self.ScrollbarLayer(host) else {
            return scroll_result;
        };
        if self.drag_state_.is_none() {
            return scroll_result;
        }
        scroll_result.type_ = PointerResultType::kScrollbarScroll;
        if self.drag_processed_for_current_frame_ {
            return scroll_result;
        }
        if self.SnapToDragOrigin(host, position_in_widget) {
            let delta = host.Scrollbar_current_pos(&scrollbar)
                - self.drag_state_.as_ref().unwrap().scroll_position_at_start_;
            scroll_result.scroll_units = ScrollGranularity::kScrollByPrecisePixel;
            scroll_result.scroll_delta =
                if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                    Vector2dF::new(0., -delta)
                } else {
                    Vector2dF::new(-delta, 0.)
                };
            self.drag_processed_for_current_frame_ = true;
            return scroll_result;
        }
        let target_node = host_call!(
            host,
            FindNodeFromElementId(host.Scrollbar_scroll_element_id(&scrollbar))
        );
        debug_assert!(target_node.is_some());
        let target_node = target_node.expect("scrollbar must target an existing ScrollNode");
        let mut distance = self.GetScrollDistanceForDragPosition(host, position_in_widget);
        if self
            .drag_state_
            .as_ref()
            .unwrap()
            .scroller_length_at_previous_move
            != host.Scrollbar_scroll_layer_length(&scrollbar)
        {
            let drag = self.drag_state_.as_mut().unwrap();
            drag.scroller_displacement = distance;
            drag.scroller_length_at_previous_move = host.Scrollbar_scroll_layer_length(&scrollbar);
            return scroll_result;
        }
        distance -= self.drag_state_.as_ref().unwrap().scroller_displacement;
        let scroll_delta =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                Vector2dF::new(0., distance)
            } else {
                Vector2dF::new(distance, 0.)
            };
        let clamped_scroll_delta = self.ComputeClampedDelta(host, &target_node, &scroll_delta);
        if VectorIsZero(clamped_scroll_delta) {
            return scroll_result;
        }
        scroll_result.scroll_units = ScrollGranularity::kScrollByPrecisePixel;
        scroll_result.scroll_delta = clamped_scroll_delta;
        self.drag_processed_for_current_frame_ = true;
        scroll_result
    }
    // Source: cc/input/scrollbar_controller.cc:396-412
    fn ComputeClampedDelta<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        target_node: &InputScrollNodeId,
        scroll_delta: &Vector2dF,
    ) -> Vector2dF {
        debug_assert!(!host.scrolls_inner_viewport(&target_node));
        if host.scrolls_outer_viewport(&target_node) {
            return host.ComputeClampedDelta(scroll_delta);
        }
        let clamped_delta = host.ComputeScrollDelta(target_node, scroll_delta);
        let scale_factor = self.GetPageScaleFactorForScroll(host);
        ScaleVector(clamped_delta, scale_factor)
    }
    // Source: cc/input/scrollbar_controller.cc:414-469
    fn GetScrollerToScrollbarRatio<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> f32 {
        let scrollbar = self.bar(host);
        let scroll_layer_length = host.Scrollbar_scroll_layer_length(&scrollbar);
        let scrollbar_track_length = host.Scrollbar_TrackLength(&scrollbar);
        let thumb_rect = host.Scrollbar_ComputeThumbQuadRect(&scrollbar);
        let scrollbar_thumb_length =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                thumb_rect.height
            } else {
                thumb_rect.width
            } as f32;
        let viewport_length = self.GetViewportLength(host);
        if scrollbar_track_length == scrollbar_thumb_length {
            return 0.;
        }
        (scroll_layer_length - viewport_length) / (scrollbar_track_length - scrollbar_thumb_length)
    }
    // Source: cc/input/scrollbar_controller.cc:471-480
    pub fn ResetState(&mut self) {
        self.drag_processed_for_current_frame_ = false;
        self.drag_state_ = None;
        self.autoscroll_state_ = None;
        self.captured_scrollbar_metadata_ = None;
        self.CancelAutoscrollTask();
    }
    // Source: cc/input/scrollbar_controller.cc:482-494
    pub fn DidRegisterScrollbar<H: ScrollbarRegistrationHost + ?Sized>(
        &mut self,
        host: &mut H,
        element_id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        if self.autoscroll_state_.is_some()
            && self
                .captured_scrollbar_metadata_
                .as_ref()
                .unwrap()
                .scroll_element_id
                == element_id
            && self
                .captured_scrollbar_metadata_
                .as_ref()
                .unwrap()
                .orientation
                == orientation
            && self.autoscroll_state_.as_ref().unwrap().status == AutoScrollStatus::kAutoscrollReady
        {
            self.PostAutoscrollTask(host, TimeDelta::Min());
        }
    }
    // Source: cc/input/scrollbar_controller.cc:496-506
    pub fn DidUnregisterScrollbar<H: ScrollbarRegistrationHost + ?Sized>(
        &mut self,
        host: &mut H,
        element_id: ElementId,
        orientation: ScrollbarOrientation,
    ) {
        if self.autoscroll_state_.is_some()
            && self
                .captured_scrollbar_metadata_
                .as_ref()
                .unwrap()
                .scroll_element_id
                == element_id
            && self
                .captured_scrollbar_metadata_
                .as_ref()
                .unwrap()
                .orientation
                == orientation
            && self.autoscroll_state_.as_ref().unwrap().status
                == AutoScrollStatus::kAutoscrollScrolling
        {
            host.ScrollAnimationAbort(element_id);
            self.autoscroll_state_.as_mut().unwrap().status = AutoScrollStatus::kAutoscrollReady;
        }
    }
    // Source: cc/input/scrollbar_controller.cc:508-581
    fn RecomputeAutoscrollStateIfNeeded<H: ScrollbarHost + ?Sized>(&mut self, host: &mut H) {
        if self.autoscroll_state_.is_none()
            || self.captured_scrollbar_metadata_.is_none()
            || self.autoscroll_state_.as_ref().unwrap().status
                != AutoScrollStatus::kAutoscrollScrolling
        {
            return;
        }
        // C++ leaves this uninitialized; Rust requires a value on inverse failure.
        let mut clipped = false;
        let scroller_relative_position = self.GetScrollbarRelativePosition(
            host,
            self.last_known_pointer_position_,
            &mut clipped,
        );
        if clipped {
            return;
        }
        let scrollbar = self.bar(host);
        let thumb_quad = host.Scrollbar_ComputeThumbQuadRect(&scrollbar);
        let (thumb_start, thumb_end, pointer_position) =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                (
                    thumb_quad.y,
                    thumb_quad.y + thumb_quad.height,
                    scroller_relative_position.y() as i32,
                )
            } else {
                (
                    thumb_quad.x,
                    thumb_quad.x + thumb_quad.width,
                    scroller_relative_position.x() as i32,
                )
            };
        if (self.autoscroll_state_.as_ref().unwrap().direction
            == AutoScrollDirection::kAutoscrollForward
            && thumb_end > pointer_position)
            || (self.autoscroll_state_.as_ref().unwrap().direction
                == AutoScrollDirection::kAutoscrollBackward
                && thumb_start < pointer_position)
        {
            ScrollbarHost::ScrollAnimationAbort(
                host,
                self.captured_scrollbar_metadata_
                    .as_ref()
                    .unwrap()
                    .scroll_element_id,
            );
        }
        if self.autoscroll_state_.as_ref().unwrap().direction
            == AutoScrollDirection::kAutoscrollForward
        {
            let scroll_layer_length = host.Scrollbar_scroll_layer_length(&scrollbar);
            if self.autoscroll_state_.as_ref().unwrap().scroll_layer_length != scroll_layer_length {
                host_call!(
                    host,
                    ScrollAnimationAbort(host.Scrollbar_scroll_element_id(&scrollbar))
                );
                self.StartAutoScrollAnimation(host);
            }
        }
        let scrollbar_part_rect = RectF::from(
            self.GetRectForScrollbarPart(
                host,
                self.autoscroll_state_
                    .as_ref()
                    .unwrap()
                    .pressed_scrollbar_part,
            ),
        );
        if !RectContainsPoint(scrollbar_part_rect, scroller_relative_position) {
            host_call!(
                host,
                ScrollAnimationAbort(host.Scrollbar_scroll_element_id(&scrollbar))
            );
        } else if RectContainsPoint(scrollbar_part_rect, scroller_relative_position)
            && !host_call!(
                host,
                IsElementAnimating(host.Scrollbar_scroll_element_id(&scrollbar))
            )
        {
            self.StartAutoScrollAnimation(host);
        }
    }
    // Source: cc/input/scrollbar_controller.cc:584-592
    fn InitialDeltaToAutoscrollVelocity<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        scroll_delta: Vector2dF,
    ) -> f32 {
        debug_assert!(self.captured_scrollbar_metadata_.is_some());
        let delta = if host_call!(host, Scrollbar_orientation(&self.bar(host)))
            == ScrollbarOrientation::kVertical
        {
            scroll_delta.y()
        } else {
            scroll_delta.x()
        };
        delta * kAutoscrollMultiplier
    }
    // Source: cc/input/scrollbar_controller.cc:594-603
    fn StartAutoScroll<H: ScrollbarHost + ?Sized>(&mut self, host: &mut H) {
        debug_assert!(self.autoscroll_state_.is_some());
        if self.ScrollbarLayer(host).is_some() {
            self.autoscroll_state_.as_mut().unwrap().status =
                AutoScrollStatus::kAutoscrollScrolling;
            self.StartAutoScrollAnimation(host);
        } else {
            self.autoscroll_state_.as_mut().unwrap().status = AutoScrollStatus::kAutoscrollReady;
        }
    }
    // Source: cc/input/scrollbar_controller.cc:605-649
    fn StartAutoScrollAnimation<H: ScrollbarHost + ?Sized>(&mut self, host: &mut H) {
        debug_assert!(self.drag_state_.is_none());
        debug_assert!(self.captured_scrollbar_metadata_.is_some());
        debug_assert!(self.autoscroll_state_.is_some());
        debug_assert!(self.ScrollbarLayer(host).is_some());
        debug_assert_eq!(
            self.autoscroll_state_.as_ref().unwrap().status,
            AutoScrollStatus::kAutoscrollScrolling
        );
        debug_assert_ne!(self.autoscroll_state_.as_ref().unwrap().velocity, 0.);
        let scrollbar = self.bar(host);

        let scroll_node = host_call!(
            host,
            FindNodeFromElementId(host.Scrollbar_scroll_element_id(&scrollbar))
        );
        if !(scroll_node.is_some() && self.scrollbar_scroll_is_active_) {
            return;
        }
        let scroll_node = scroll_node.unwrap();
        let scroll_layer_length = host.Scrollbar_scroll_layer_length(&scrollbar);
        let current_offset = host_call!(host, current_scroll_offset(host.element_id(&scroll_node)));
        let target_offset_in_orientation = if self.autoscroll_state_.as_ref().unwrap().velocity < 0.
        {
            0.
        } else {
            scroll_layer_length
        };
        let target_offset_2d =
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                PointF::new(current_offset.x(), target_offset_in_orientation)
            } else {
                PointF::new(target_offset_in_orientation, current_offset.y())
            };
        self.autoscroll_state_.as_mut().unwrap().scroll_layer_length = scroll_layer_length;
        self.autoscroll_state_.as_mut().unwrap().direction =
            if self.autoscroll_state_.as_ref().unwrap().velocity < 0. {
                AutoScrollDirection::kAutoscrollBackward
            } else {
                AutoScrollDirection::kAutoscrollForward
            };
        host_call!(host, ScrollAnimationAbort(host.element_id(&scroll_node)));
        host.AutoScrollAnimationCreate(
            &scroll_node,
            target_offset_2d,
            self.autoscroll_state_.as_ref().unwrap().velocity.abs(),
        );
    }
    // Source: cc/input/scrollbar_controller.cc:652-671
    pub fn HandlePointerUp<H: ScrollbarHost + ?Sized>(
        &mut self,
        host: &mut H,
        _position_in_widget: PointF,
    ) -> InputHandlerPointerResult {
        let mut scroll_result = InputHandlerPointerResult::default();
        if self.scrollbar_scroll_is_active_ {
            self.scrollbar_scroll_is_active_ = false;
            scroll_result.type_ = PointerResultType::kScrollbarScroll;
        }
        if self.autoscroll_state_.is_some()
            && self.autoscroll_state_.as_ref().unwrap().status
                == AutoScrollStatus::kAutoscrollScrolling
        {
            ScrollbarHost::ScrollAnimationAbort(
                host,
                self.captured_scrollbar_metadata_
                    .as_ref()
                    .unwrap()
                    .scroll_element_id,
            );
        }
        self.ResetState();
        scroll_result
    }
    // Source: cc/input/scrollbar_controller.cc:674-685
    fn GetLayerHitByPoint<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        position_in_widget: PointF,
    ) -> Option<InputLayerId> {
        let viewport_point = (position_in_widget.x() as i32, position_in_widget.y() as i32);
        let device_viewport_point = PointF::new(
            viewport_point.0 as f32 * host.device_scale_factor(),
            viewport_point.1 as f32 * host.device_scale_factor(),
        );
        let layer_impl = host.FindLayerThatIsHitByPoint(device_viewport_point);
        layer_impl
    }
    // Source: cc/input/scrollbar_controller.cc:687-708
    fn GetViewportLength<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> f32 {
        let scrollbar = self.bar(host);
        let scroll_node = host_call!(
            host,
            FindNodeFromElementId(host.Scrollbar_scroll_element_id(&scrollbar))
        );
        debug_assert!(scroll_node.is_some());
        let scroll_node = scroll_node.expect("scrollbar requires a scroll node");
        if !host.scrolls_outer_viewport(&scroll_node) {
            let bounds = host.container_bounds(&scroll_node);
            let length =
                if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                    bounds.height()
                } else {
                    bounds.width()
                } as f32;
            return length;
        }
        let viewport_size = host.GetInnerViewportSizeExcludingScrollbars();
        let length = if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
            viewport_size.height()
        } else {
            viewport_size.width()
        };
        length / self.GetPageScaleFactorForScroll(host)
    }
    // Source: cc/input/scrollbar_controller.cc:710-712
    fn GetPageScaleFactorForScroll<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> f32 {
        host.page_scale_factor_for_scroll()
    }
    // Source: cc/input/scrollbar_controller.cc:714-753
    fn GetScrollDistanceForScrollbarPart<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        scrollbar_part: ScrollbarPart,
        jump_key_modifier: bool,
    ) -> f32 {
        let mut scroll_delta = 0.;
        match scrollbar_part {
            ScrollbarPart::kBackButton | ScrollbarPart::kForwardButton => {
                scroll_delta = kPixelsPerLineStep as f32 * self.ScreenSpaceScaleFactor(host);
            }
            ScrollbarPart::kBackTrack | ScrollbarPart::kForwardTrack => {
                if jump_key_modifier {
                    scroll_delta = self.GetScrollDistanceForAbsoluteJump(host);
                } else {
                    let mut snapport_length = self.GetViewportLength(host) as i32;
                    let scrollbar = self.bar(host);
                    let target_node = host_call!(
                        host,
                        FindNodeFromElementId(host.Scrollbar_scroll_element_id(&scrollbar))
                    )
                    .expect("scrollbar requires a scroll node");
                    if let Some(snapport) = host.SnapContainerRect(&target_node) {
                        snapport_length = if host.Scrollbar_orientation(&scrollbar)
                            == ScrollbarOrientation::kHorizontal
                        {
                            snapport.x()
                        } else {
                            snapport.y()
                        } as i32;
                    }
                    scroll_delta = ScrollUtils::CalculatePageStep(snapport_length) as f32;
                }
            }
            _ => {
                scroll_delta = 0.;
            }
        }
        scroll_delta
    }
    // Source: cc/input/scrollbar_controller.cc:755-757
    fn ScreenSpaceScaleFactor<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> f32 {
        host.painted_device_scale_factor()
    }
    // Source: cc/input/scrollbar_controller.cc:759-783
    fn GetScrollbarRelativePosition<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        position_in_widget: PointF,
        clipped: &mut bool,
    ) -> PointF {
        let Some(scrollbar) = self.ScrollbarLayer(host) else {
            *clipped = true;
            return PointF::new(0., 0.);
        };
        let mut inverse_screen_space_transform = H::Transform::default();
        let scaled_screen_space_transform = host.Scrollbar_ScreenSpaceTransform(&scrollbar);
        if !scaled_screen_space_transform.GetInverse(&mut inverse_screen_space_transform) {
            return PointF::new(0., 0.);
        }
        MathUtil::ProjectPoint(&inverse_screen_space_transform, position_in_widget, clipped)
    }
    // Source: cc/input/scrollbar_controller.cc:786-800
    fn GetScrollbarPartFromPointerDown<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        position_in_widget: PointF,
    ) -> ScrollbarPart {
        let scrollbar = self.bar(host);
        let mut clipped = false;
        let scroller_relative_position =
            self.GetScrollbarRelativePosition(host, position_in_widget, &mut clipped);
        if clipped {
            return ScrollbarPart::kNoPart;
        }
        host.Scrollbar_IdentifyScrollbarPart(&scrollbar, scroller_relative_position)
    }
    // Source: cc/input/scrollbar_controller.cc:803-819
    fn GetRectForScrollbarPart<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        scrollbar_part: ScrollbarPart,
    ) -> Rect {
        let scrollbar = self.bar(host);
        if scrollbar_part == ScrollbarPart::kBackButton {
            return host.Scrollbar_BackButtonRect(&scrollbar);
        }
        if scrollbar_part == ScrollbarPart::kForwardButton {
            return host.Scrollbar_ForwardButtonRect(&scrollbar);
        }
        if scrollbar_part == ScrollbarPart::kBackTrack {
            return host.Scrollbar_BackTrackRect(&scrollbar);
        }
        if scrollbar_part == ScrollbarPart::kForwardTrack {
            return host.Scrollbar_ForwardTrackRect(&scrollbar);
        }
        Rect::new(0, 0, 0, 0)
    }
    // Source: cc/input/scrollbar_controller.cc:823-851
    fn GetScrollDeltaForScrollbarPart<H: ScrollbarHost + ?Sized>(
        &self,
        host: &mut H,
        scrollbar_part: ScrollbarPart,
        jump_key_modifier: bool,
    ) -> Vector2dF {
        let scrollbar = self.bar(host);
        let distance =
            self.GetScrollDistanceForScrollbarPart(host, scrollbar_part, jump_key_modifier);
        if scrollbar_part == ScrollbarPart::kBackButton {
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                Vector2dF::new(0., -distance)
            } else {
                Vector2dF::new(-distance, 0.)
            }
        } else if scrollbar_part == ScrollbarPart::kForwardButton {
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                Vector2dF::new(0., distance)
            } else {
                Vector2dF::new(distance, 0.)
            }
        } else if scrollbar_part == ScrollbarPart::kBackTrack {
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                Vector2dF::new(0., -distance)
            } else {
                Vector2dF::new(-distance, 0.)
            }
        } else if scrollbar_part == ScrollbarPart::kForwardTrack {
            if host.Scrollbar_orientation(&scrollbar) == ScrollbarOrientation::kVertical {
                Vector2dF::new(0., distance)
            } else {
                Vector2dF::new(distance, 0.)
            }
        } else {
            Vector2dF::new(0., 0.)
        }
    }
    // Rust equivalent of CancelableOnceClosure cancellation and pointer reset.
    fn CancelAutoscrollTask(&mut self) {
        if let Some(task) = self.cancelable_autoscroll_task_.take() {
            drop(task);
        }
    }
    fn bar<H: ScrollbarHost + ?Sized>(&self, host: &mut H) -> InputLayerId {
        self.ScrollbarLayer(host)
            .expect("source requires a registered scrollbar")
    }
    /// Delivers a queued typed task. Pointer existence survives invocation, just
    /// like the source's unique_ptr<CancelableOnceClosure>, while callback is once.
    pub fn RunAutoscrollTask<H: ScrollbarHost + ?Sized>(
        &mut self,
        host: &mut H,
        token: AutoscrollTask,
    ) {
        let Some(task) = self.cancelable_autoscroll_task_.as_mut() else {
            return;
        };
        if !token.IsAlive() || !task.token.matches(&token) || !task.pending {
            return;
        }
        task.pending = false;
        self.StartAutoScroll(host);
    }
}
// Source: cc/input/scrollbar_controller.cc:20-25
impl Drop for ScrollbarController {
    fn drop(&mut self) {
        self.CancelAutoscrollTask();
    }
}
