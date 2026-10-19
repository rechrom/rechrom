// Translation status: complete
// Source: cc/input/single_scrollbar_animation_controller_thinning.h:1-125
// Source: cc/input/single_scrollbar_animation_controller_thinning.cc:1-357
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use super::{
    scrollbar::ScrollbarOrientation,
    scrollbar_animation_controller::ScrollbarAnimationControllerClient,
};
use crate::{
    adapter::{
        geometry::{cpp_clamp, PointF, Rect, RectF},
        input_handler::InputLayerId,
        input_types::ElementId,
        scrollbar_animation::ManhattanDistanceToPoint,
        time::{TimeDelta, TimeTicks},
    },
    cc::base::math_util::MathUtil,
};

// Source: cc/input/single_scrollbar_animation_controller_thinning.cc:20-41
fn DistanceToRect<C: ScrollbarAnimationControllerClient + ?Sized>(
    client: &mut C,
    point: PointF,
    scrollbar: &InputLayerId,
    rect: Rect,
) -> f32 {
    let device_rect = MathUtil::MapClippedRect(
        &client.Scrollbar_ScreenSpaceTransform(&scrollbar),
        RectF::from(rect),
    );
    ManhattanDistanceToPoint(device_rect, point) / client.Scrollbar_device_scale_factor(&scrollbar)
}
fn DistanceToScrollbar<C: ScrollbarAnimationControllerClient + ?Sized>(
    client: &mut C,
    point: PointF,
    scrollbar: &InputLayerId,
) -> f32 {
    let size = client.Scrollbar_bounds(&scrollbar);
    DistanceToRect(
        client,
        point,
        scrollbar,
        Rect::new(0, 0, size.width(), size.height()),
    )
}
fn DistanceToScrollbarThumb<C: ScrollbarAnimationControllerClient + ?Sized>(
    client: &mut C,
    point: PointF,
    scrollbar: &InputLayerId,
) -> f32 {
    let rect = client.Scrollbar_ComputeHitTestableExpandedThumbQuadRect(&scrollbar);
    DistanceToRect(client, point, scrollbar, rect)
}
// Source: cc/input/single_scrollbar_animation_controller_thinning.h:83
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnimationChange {
    kNone,
    kIncrease,
    kDecrease,
}
// Source: cc/input/single_scrollbar_animation_controller_thinning.h:96-120
pub struct SingleScrollbarAnimationControllerThinning {
    last_awaken_time_: TimeTicks,
    is_animating_: bool,
    scroll_element_id_: ElementId,
    orientation_: ScrollbarOrientation,
    captured_: bool,
    mouse_is_over_scrollbar_thumb_: bool,
    mouse_is_near_scrollbar_thumb_: bool,
    mouse_is_near_scrollbar_: bool,
    thickness_change_: AnimationChange,
    thinning_duration_: TimeDelta,
    tickmarks_showing_: bool,
    device_viewport_last_pointer_location_: PointF,
    idle_thickness_scale_: f32,
}
impl SingleScrollbarAnimationControllerThinning {
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:46-55
    pub fn Create<C: crate::adapter::scrollbar::ScrollbarInitializationClient + ?Sized>(
        scroll_element_id: ElementId,
        orientation: ScrollbarOrientation,
        client: &mut C,
        thinning_duration: TimeDelta,
        idle_thickness_scale: f32,
    ) -> Box<Self> {
        Box::new(Self::new(
            scroll_element_id,
            orientation,
            client,
            thinning_duration,
            idle_thickness_scale,
        ))
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:57-70
    fn new<C: crate::adapter::scrollbar::ScrollbarInitializationClient + ?Sized>(
        scroll_element_id: ElementId,
        orientation: ScrollbarOrientation,
        client: &mut C,
        thinning_duration: TimeDelta,
        idle_thickness_scale: f32,
    ) -> Self {
        let result = Self {
            last_awaken_time_: TimeTicks::default(),
            is_animating_: false,
            scroll_element_id_: scroll_element_id,
            orientation_: orientation,
            captured_: false,
            mouse_is_over_scrollbar_thumb_: false,
            mouse_is_near_scrollbar_thumb_: false,
            mouse_is_near_scrollbar_: false,
            thickness_change_: AnimationChange::kNone,
            thinning_duration_: thinning_duration,
            tickmarks_showing_: false,
            device_viewport_last_pointer_location_: PointF::new(-1., -1.),
            idle_thickness_scale_: idle_thickness_scale,
        };
        result.ApplyThumbThicknessScale(client, result.idle_thickness_scale_);
        result
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.h:39-50
    pub fn mouse_is_over_scrollbar_thumb(&self) -> bool {
        self.mouse_is_over_scrollbar_thumb_
    }
    pub fn mouse_is_near_scrollbar_thumb(&self) -> bool {
        self.mouse_is_near_scrollbar_thumb_
    }
    pub fn mouse_is_near_scrollbar(&self) -> bool {
        self.mouse_is_near_scrollbar_
    }
    pub fn captured(&self) -> bool {
        self.captured_
    }
    pub fn device_viewport_last_pointer_location(&self) -> PointF {
        self.device_viewport_last_pointer_location_
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:73-83
    fn GetScrollbar<C: ScrollbarAnimationControllerClient + ?Sized>(
        &self,
        client: &mut C,
    ) -> Option<InputLayerId> {
        for scrollbar in client.ScrollbarsFor(self.scroll_element_id_) {
            debug_assert!(client.Scrollbar_is_overlay_scrollbar(&scrollbar));
            if client.Scrollbar_orientation(&scrollbar) == self.orientation_ {
                return Some(scrollbar);
            }
        }
        None
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:85-96
    pub fn Animate<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        now: TimeTicks,
    ) -> bool {
        if !self.is_animating_ {
            return false;
        }
        if self.last_awaken_time_.is_null() {
            self.last_awaken_time_ = now;
        }
        let progress = self.AnimationProgressAtTime(now);
        self.RunAnimationFrame(client, progress);
        true
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:98-106
    fn AnimationProgressAtTime(&self, now: TimeTicks) -> f32 {
        if self.thinning_duration_.is_zero() {
            return 1.;
        }
        let delta = now - self.last_awaken_time_;
        cpp_clamp((delta / self.thinning_duration_) as f32, 0., 1.)
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:108-120
    fn RunAnimationFrame<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        progress: f32,
    ) {
        if self.captured_ {
            return;
        }
        self.ApplyThumbThicknessScale(client, self.ThumbThicknessScaleAt(progress));
        client.SetNeedsRedrawForScrollbarAnimation();
        if progress == 1. {
            self.StopAnimation();
            self.thickness_change_ = AnimationChange::kNone;
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:122-126
    pub fn StartAnimation<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.is_animating_ = true;
        self.last_awaken_time_ = TimeTicks::default();
        client.SetNeedsAnimateForScrollbarAnimation();
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:128-130
    pub fn StopAnimation(&mut self) {
        self.is_animating_ = false;
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:132-143
    pub fn DidScrollUpdate<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        if self.captured_ || !self.mouse_is_near_scrollbar_ {
            return;
        }
        self.CalculateThicknessShouldChange(client, self.device_viewport_last_pointer_location_);
        if self.thickness_change_ == AnimationChange::kNone {
            self.UpdateThumbThicknessScale(client);
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:145-165
    pub fn DidMouseDown<C: ScrollbarAnimationControllerClient + ?Sized>(&mut self, client: &mut C) {
        if client.IsFluentOverlayScrollbar() && !self.mouse_is_over_scrollbar_thumb_ {
            if let Some(scrollbar) = self.GetScrollbar(client) {
                let distance = DistanceToScrollbarThumb(
                    client,
                    self.device_viewport_last_pointer_location_,
                    &scrollbar,
                );
                self.mouse_is_over_scrollbar_thumb_ = distance == 0.;
            }
        }
        if !self.mouse_is_over_scrollbar_thumb_ {
            return;
        }
        self.captured_ = true;
        self.UpdateThumbThicknessScale(client);
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:167-186
    pub fn DidMouseUp<C: ScrollbarAnimationControllerClient + ?Sized>(&mut self, client: &mut C) {
        if !self.captured_ {
            return;
        }
        self.captured_ = false;
        self.StopAnimation();
        let decrease = !client.IsFluentOverlayScrollbar() && !self.mouse_is_near_scrollbar_thumb_;
        if decrease {
            self.thickness_change_ = AnimationChange::kDecrease;
            self.StartAnimation(client);
        } else {
            self.thickness_change_ = AnimationChange::kNone;
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:188-206
    pub fn DidMouseLeave<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.mouse_is_over_scrollbar_thumb_ = false;
        self.mouse_is_near_scrollbar_thumb_ = false;
        self.mouse_is_near_scrollbar_ = false;
        if self.captured_ {
            return;
        }
        if client.IsFluentOverlayScrollbar() && self.thickness_change_ == AnimationChange::kNone {
            return;
        }
        self.thickness_change_ = AnimationChange::kDecrease;
        self.StartAnimation(client);
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:208-212
    pub fn DidMouseMove<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        point: PointF,
    ) {
        self.CalculateThicknessShouldChange(client, point);
        self.device_viewport_last_pointer_location_ = point;
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:214-267
    fn CalculateThicknessShouldChange<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        point: PointF,
    ) {
        let Some(scrollbar) = self.GetScrollbar(client) else {
            return;
        };
        let distance = DistanceToScrollbar(client, point, &scrollbar);
        let thumb_distance = DistanceToScrollbarThumb(client, point, &scrollbar);
        let near = distance <= self.MouseMoveDistanceToTriggerFadeIn(client);
        let over_thumb = thumb_distance == 0.;
        let near_thumb = thumb_distance <= self.MouseMoveDistanceToTriggerExpand(client);
        let should_change;
        if client.IsFluentOverlayScrollbar() {
            let visible = client.Scrollbar_OverlayScrollbarOpacity(&scrollbar) > 0.;
            let moved_over = self.mouse_is_near_scrollbar_ != near;
            let far = !near && self.thickness_change_ == AnimationChange::kNone;
            should_change = !self.tickmarks_showing_ && visible && (moved_over || far);
        } else {
            should_change = self.mouse_is_near_scrollbar_thumb_ != near_thumb;
        }
        if !self.captured_ && should_change {
            let increase = if client.IsFluentOverlayScrollbar() {
                near
            } else {
                near_thumb
            };
            self.thickness_change_ = if increase {
                AnimationChange::kIncrease
            } else {
                AnimationChange::kDecrease
            };
            self.StartAnimation(client);
        }
        self.mouse_is_near_scrollbar_ = near;
        self.mouse_is_near_scrollbar_thumb_ = near_thumb;
        self.mouse_is_over_scrollbar_thumb_ = over_thumb;
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:269-276
    fn ThumbThicknessScaleAt(&self, progress: f32) -> f32 {
        assert_ne!(self.thickness_change_, AnimationChange::kNone);
        let factor = if self.thickness_change_ == AnimationChange::kIncrease {
            progress
        } else {
            1. - progress
        };
        ((1. - self.idle_thickness_scale_) * factor) + self.idle_thickness_scale_
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:278-299
    fn AdjustScale(
        &self,
        new_value: f32,
        current_value: f32,
        change: AnimationChange,
        min: f32,
        max: f32,
    ) -> f32 {
        let result;
        if change == AnimationChange::kIncrease && current_value > new_value {
            result = current_value;
        } else if change == AnimationChange::kDecrease && current_value < new_value {
            result = current_value;
        } else {
            result = new_value;
        }
        if result > max {
            return max;
        }
        if result < min {
            return min;
        }
        result
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:301-311
    fn CurrentForcedThumbThicknessScale<C: ScrollbarAnimationControllerClient + ?Sized>(
        &self,
        client: &mut C,
    ) -> f32 {
        let mut expanded;
        if client.IsFluentOverlayScrollbar() {
            expanded = self.mouse_is_near_scrollbar_ || self.tickmarks_showing_;
        } else {
            expanded = self.mouse_is_near_scrollbar_thumb_;
        }
        expanded |= self.captured_;
        if expanded {
            1.
        } else {
            self.idle_thickness_scale_
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:313-316
    fn UpdateThumbThicknessScale<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        self.StopAnimation();
        let forced_scale = self.CurrentForcedThumbThicknessScale(client);
        self.ApplyThumbThicknessScale(client, forced_scale);
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:318-322
    pub fn DidRequestShow<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
    ) {
        if self.thickness_change_ == AnimationChange::kNone {
            self.UpdateThumbThicknessScale(client);
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:324-337
    fn ApplyThumbThicknessScale<C: crate::adapter::scrollbar::ScrollbarInitializationClient + ?Sized>(
        &self,
        client: &mut C,
        scale: f32,
    ) {
        for scrollbar in client.ScrollbarsFor(self.scroll_element_id_) {
            if client.Scrollbar_orientation(&scrollbar) != self.orientation_ {
                continue;
            }
            debug_assert!(client.Scrollbar_is_overlay_scrollbar(&scrollbar));
            let adjusted = self.AdjustScale(
                scale,
                client.Scrollbar_thumb_thickness_scale_factor(&scrollbar),
                self.thickness_change_,
                self.idle_thickness_scale_,
                1.,
            );
            client.Scrollbar_SetThumbThicknessScaleFactor(&scrollbar, adjusted);
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:339-345
    pub fn UpdateTickmarksVisibility<C: ScrollbarAnimationControllerClient + ?Sized>(
        &mut self,
        client: &mut C,
        show: bool,
    ) {
        self.tickmarks_showing_ = show;
        if show {
            self.UpdateThumbThicknessScale(client);
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:347-350
    pub fn MouseMoveDistanceToTriggerExpand<C: ScrollbarAnimationControllerClient + ?Sized>(
        &self,
        client: &mut C,
    ) -> f32 {
        if client.IsFluentOverlayScrollbar() {
            0.
        } else {
            25.
        }
    }
    // Source: cc/input/single_scrollbar_animation_controller_thinning.cc:352-355
    pub fn MouseMoveDistanceToTriggerFadeIn<C: ScrollbarAnimationControllerClient + ?Sized>(
        &self,
        client: &mut C,
    ) -> f32 {
        if client.IsFluentOverlayScrollbar() {
            0.
        } else {
            30.
        }
    }
}
